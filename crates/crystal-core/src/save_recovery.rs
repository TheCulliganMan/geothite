//! Non-destructive slot inspection and checked recovery writes.

use std::path::Path;

use super::{SaveError, SaveGame, SaveModpackIdentity};

/// Reads one save artifact without attempting recovery or modifying any file.
///
/// # Errors
/// Returns storage access, I/O, or browser value decoding failures.
pub fn read_save_slot_bytes(path: &Path) -> Result<Option<Vec<u8>>, SaveError> {
    if !save_slot_exists(path)? {
        return Ok(None);
    }
    #[cfg(target_arch = "wasm32")]
    return super::read_browser_save_bytes(path).map(Some);
    #[cfg(not(target_arch = "wasm32"))]
    std::fs::read(path)
        .map(Some)
        .map_err(|source| SaveError::Read {
            path: path.display().to_string(),
            source,
        })
}

/// Reports whether an artifact exists, including an undecodable browser value.
///
/// # Errors
/// Returns storage access or filesystem inspection failures.
pub fn save_slot_exists(path: &Path) -> Result<bool, SaveError> {
    #[cfg(target_arch = "wasm32")]
    {
        let storage = super::browser_save_storage().map_err(|source| SaveError::Read {
            path: path.display().to_string(),
            source,
        })?;
        storage
            .get_item(&super::browser_save_storage_key(path))
            .map(|value| value.is_some())
            .map_err(|error| SaveError::Read {
                path: path.display().to_string(),
                source: super::browser_storage_error("inspect save artifact", error),
            })
    }
    #[cfg(not(target_arch = "wasm32"))]
    path.try_exists().map_err(|source| SaveError::Read {
        path: path.display().to_string(),
        source,
    })
}

/// Creates an independent compatible save; existing primary/backup bytes survive.
///
/// # Errors
/// Rejects invalid state/identity, an occupied destination, or a storage failure.
pub fn write_new_compatible_save(
    path: &Path,
    state: super::GameState,
    identity: &SaveModpackIdentity,
    content_hash: &str,
) -> Result<(), SaveError> {
    super::validate_save_path(path)?;
    let save = SaveGame::new(state, identity.clone(), content_hash.to_owned())?;
    let bytes = super::encode_save_game_bytes(&save)?;
    if save_slot_exists(path)? || save_slot_exists(&super::save_backup_path(path))? {
        return Err(SaveError::InvalidState(
            "compatible save destination is occupied".into(),
        ));
    }
    write_new_artifact(path, &bytes)?;
    write_new_artifact(&super::save_backup_path(path), &bytes)
}

/// Restores a validated backup while archiving the damaged primary verbatim.
///
/// The backup is never rewritten. The caller must also validate the recovered
/// state against its complete runtime pack before invoking this function.
///
/// # Errors
/// Rejects invalid backup identity/state, a changed primary, or I/O.
pub fn recover_save_primary(
    path: &Path,
    expected_primary: Option<&[u8]>,
    bytes: &[u8],
    identity: &SaveModpackIdentity,
    content_hash: &str,
) -> Result<(), SaveError> {
    super::read_save_game_bytes_for_modpack(bytes, "recovery backup", identity, content_hash)?;
    let current = read_save_slot_bytes(path).ok().flatten();
    if current.as_deref() != expected_primary {
        return Err(SaveError::InvalidState(
            "primary changed during recovery; preserve it".into(),
        ));
    }
    if current.as_deref() == Some(bytes) {
        return Ok(());
    }
    #[cfg(target_arch = "wasm32")]
    {
        let storage = super::browser_save_storage().map_err(|source| SaveError::Read {
            path: path.display().to_string(),
            source,
        })?;
        let key = super::browser_save_storage_key(path);
        if let Some(original) = storage.get_item(&key).map_err(|error| SaveError::Read {
            path: path.display().to_string(),
            source: super::browser_storage_error("archive primary", error),
        })? {
            let archive = format!(
                "{key}.recovery-{}.original",
                super::sha256_hex(original.as_bytes())
            );
            storage
                .set_item(&archive, &original)
                .map_err(|error| SaveError::Write {
                    path: archive,
                    source: super::browser_storage_error("archive primary", error),
                })?;
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(original) = read_save_slot_bytes(path)? {
        let archive = path.with_file_name(format!(
            ".geothite-recovery-{}.original",
            super::sha256_hex(&original)
        ));
        match read_save_slot_bytes(&archive)? {
            None => write_new_artifact(&archive, &original)?,
            Some(archived) if archived == original => {}
            Some(_) => {
                return Err(SaveError::InvalidState(
                    "recovery archive contains different bytes; preserve every artifact".into(),
                ));
            }
        }
    }
    super::write_primary_save_bytes(path, bytes)
}

fn write_new_artifact(path: &Path, bytes: &[u8]) -> Result<(), SaveError> {
    #[cfg(target_arch = "wasm32")]
    {
        if save_slot_exists(path)? {
            return Err(SaveError::InvalidState(
                "save artifact appeared during migration".into(),
            ));
        }
        super::write_primary_save_bytes(path, bytes)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::io::Write;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| SaveError::Write {
                path: parent.display().to_string(),
                source,
            })?;
        }
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let temporary = std::path::PathBuf::from(format!(
            "{}.migration-{}-{stamp}",
            path.display(),
            std::process::id()
        ));
        let result = (|| -> std::io::Result<()> {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)?;
            file.write_all(bytes)?;
            file.sync_all()?;
            // Same-directory hard linking publishes complete bytes without
            // replacing a slot that another writer created in the meantime.
            std::fs::hard_link(&temporary, path)
        })();
        let _ = std::fs::remove_file(&temporary);
        result.map_err(|source| SaveError::Write {
            path: path.display().to_string(),
            source,
        })
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::{read_save_slot_bytes, recover_save_primary, write_new_compatible_save};
    use crate::save::{
        SaveGame, SaveModpackIdentity, encode_save_game_bytes, read_save_game_bytes_for_modpack,
        save_backup_path,
    };
    use crate::state::GameState;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    struct Directory(PathBuf);
    impl Directory {
        fn new() -> Self {
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "geothite-save-recovery-{}-{stamp}-{}",
                std::process::id(),
                DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).expect("owned test directory");
            Self(path)
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn identity() -> SaveModpackIdentity {
        SaveModpackIdentity::new("save-recovery-test", "a".repeat(64)).expect("identity")
    }

    fn saved_bytes() -> Vec<u8> {
        let state = GameState {
            money: 4321,
            ..GameState::default()
        };
        encode_save_game_bytes(&SaveGame::new(state, identity(), "b".repeat(64)).expect("save"))
            .expect("save bytes")
    }

    #[test]
    fn new_slot_retains_existing_primary_and_backup() {
        let dir = Directory::new();
        let path = dir.0.join("new.crystalsave");
        write_new_compatible_save(&path, GameState::default(), &identity(), &"b".repeat(64))
            .expect("new slot");
        let before = std::fs::read(&path).expect("primary");
        assert_eq!(
            std::fs::read(save_backup_path(&path)).expect("backup"),
            before
        );
        let different = GameState {
            money: 999,
            ..GameState::default()
        };
        assert!(write_new_compatible_save(&path, different, &identity(), &"b".repeat(64)).is_err());
        assert_eq!(std::fs::read(&path).expect("preserved primary"), before);
        std::fs::remove_file(&path).expect("simulate missing primary");
        assert!(
            write_new_compatible_save(&path, GameState::default(), &identity(), &"b".repeat(64))
                .is_err()
        );
        assert_eq!(
            std::fs::read(save_backup_path(&path)).expect("preserved backup"),
            before
        );
    }

    #[test]
    fn missing_or_corrupt_primary_recovers_without_changing_backup() {
        for original in [None, Some(b"truncated original".as_slice())] {
            let dir = Directory::new();
            let path = dir.0.join("current.crystalsave");
            let bytes = saved_bytes();
            std::fs::write(save_backup_path(&path), &bytes).expect("backup");
            if let Some(original) = original {
                std::fs::write(&path, original).expect("damaged primary");
            }
            recover_save_primary(&path, original, &bytes, &identity(), &"b".repeat(64))
                .expect("recover");
            assert_eq!(
                std::fs::read(save_backup_path(&path)).expect("backup unchanged"),
                bytes
            );
            let restored = read_save_slot_bytes(&path)
                .expect("read primary")
                .expect("restored bytes");
            assert_eq!(
                read_save_game_bytes_for_modpack(
                    &restored,
                    "restored",
                    &identity(),
                    &"b".repeat(64)
                )
                .expect("exact identity")
                .state()
                .money,
                4321
            );
            if let Some(original) = original {
                let archive = dir.0.join(format!(
                    ".geothite-recovery-{}.original",
                    crate::save::sha256_hex(original)
                ));
                assert_eq!(
                    std::fs::read(archive).expect("damaged bytes archived"),
                    original
                );
            }
        }
    }

    #[test]
    fn recovery_refuses_wrong_identity_or_changed_primary() {
        let dir = Directory::new();
        let path = dir.0.join("current.crystalsave");
        std::fs::write(&path, b"newer primary").expect("primary");
        let wrong = SaveModpackIdentity::new("different-pack", "c".repeat(64)).expect("identity");
        assert!(
            recover_save_primary(
                &path,
                Some(b"newer primary"),
                &saved_bytes(),
                &wrong,
                &"b".repeat(64)
            )
            .is_err()
        );
        assert!(
            recover_save_primary(
                &path,
                Some(b"earlier primary"),
                &saved_bytes(),
                &identity(),
                &"b".repeat(64)
            )
            .is_err()
        );
        assert_eq!(
            std::fs::read(&path).expect("concurrent bytes retained"),
            b"newer primary"
        );
    }
}
