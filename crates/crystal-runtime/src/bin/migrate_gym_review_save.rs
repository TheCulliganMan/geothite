//! Convert one selected-world save into a separate eight-gym review save.
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::save::{SAVE_EXTENSION, read_save_game_bytes_for_modpack, save_backup_path};
use crystal_runtime::{CrystalRuntime, migrate_gym_review_save};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let usage = "usage: migrate_gym_review_save SOURCE_PACK TARGET_PACK SOURCE_SAVE NEW_SAVE";
    let source_pack = PathBuf::from(args.next().context(usage)?);
    let target_pack = PathBuf::from(args.next().context(usage)?);
    let source_save = PathBuf::from(args.next().context(usage)?);
    let target_save = PathBuf::from(args.next().context(usage)?);
    ensure!(args.next().is_none(), "{usage}");
    migrate(&source_pack, &target_pack, &source_save, &target_save)?;
    println!("Created {}", target_save.display());
    Ok(())
}

fn migrate(
    source_pack: &Path,
    target_pack: &Path,
    source_save: &Path,
    target_save: &Path,
) -> Result<()> {
    ensure!(
        target_save.extension().and_then(|s| s.to_str()) == Some(SAVE_EXTENSION),
        "destination must have a .{SAVE_EXTENSION} extension"
    );
    let name = target_save.file_name().context("destination filename")?;
    let parent = target_save
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent = fs::canonicalize(parent).context("resolve destination directory")?;
    let target_save = parent.join(name);
    ensure!(
        path_absent(&target_save)? && path_absent(&save_backup_path(&target_save))?,
        "destination or its backup already exists"
    );
    let source_pack = fs::canonicalize(source_pack).context("resolve source pack")?;
    let target_pack = fs::canonicalize(target_pack).context("resolve target pack")?;
    let source_root = AssetRoot::new(source_pack.parent().context("source pack directory")?);
    let target_root = AssetRoot::new(target_pack.parent().context("target pack directory")?);
    let source = CrystalRuntime::from_loaded_compiled_pack(
        &source_root,
        read_loaded_verified_compiled_game_pack(&source_pack)?,
    )?;
    let target = CrystalRuntime::from_loaded_compiled_pack(
        &target_root,
        read_loaded_verified_compiled_game_pack(&target_pack)?,
    )?;
    // Read the primary directly: the usual save loader can repair it from a
    // backup, which would change the source during what should be a copy.
    let bytes = fs::read(source_save).context("read source save")?;
    let saved = read_save_game_bytes_for_modpack(
        &bytes,
        source_save.display().to_string(),
        source.modpack(),
        &source.pack_identity().content_hash,
    )?;
    let state = migrate_gym_review_save(&source, &target, &target_root, saved.state())?;
    let staging = StagingDirectory::create(&parent)?;
    // The staged filename is identical to the destination, so the normal save
    // writer validates the final slot name before publication.
    let staged = staging.0.join(name);
    target.save_game(&staged, state)?;
    target.load_save(&staged).context("verify migrated save")?;
    ensure!(
        path_absent(&save_backup_path(&target_save))?,
        "destination backup appeared during migration"
    );
    // Linking within the same directory publishes a complete file atomically
    // and fails if another process created the destination in the meantime.
    fs::hard_link(&staged, &target_save).context("publish new save without overwriting")?;
    Ok(())
}

fn path_absent(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(false),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(true),
        Err(error) => Err(error).with_context(|| format!("inspect {}", path.display())),
    }
}

struct StagingDirectory(PathBuf);

impl StagingDirectory {
    fn create(parent: &Path) -> Result<Self> {
        for index in 0..1000 {
            let path = parent.join(format!(
                ".geothite-gym-migration-{}-{index}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error).context("create migration staging directory"),
            }
        }
        anyhow::bail!("could not reserve a migration staging directory")
    }
}

impl Drop for StagingDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[cfg(all(test, feature = "test-fixtures"))]
mod tests {
    use super::{StagingDirectory, migrate};
    use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
    use crystal_runtime::{CrystalRuntime, RuntimeGameShell};
    use std::fs;
    use std::path::PathBuf;

    #[test]
    #[ignore = "requires CRYSTAL_GYM_MIGRATION_SOURCE and CRYSTAL_GYM_MIGRATION_TARGET"]
    fn migration_publishes_a_separate_verified_save_and_refuses_existing_slots() {
        let source_pack =
            PathBuf::from(std::env::var_os("CRYSTAL_GYM_MIGRATION_SOURCE").expect("source pack"));
        let target_pack =
            PathBuf::from(std::env::var_os("CRYSTAL_GYM_MIGRATION_TARGET").expect("target pack"));
        let root = AssetRoot::new(source_pack.parent().expect("pack directory"));
        let source = CrystalRuntime::from_loaded_compiled_pack(
            &root,
            read_loaded_verified_compiled_game_pack(&source_pack).expect("source pack"),
        )
        .expect("source runtime");
        let target = CrystalRuntime::from_loaded_compiled_pack(
            &root,
            read_loaded_verified_compiled_game_pack(&target_pack).expect("target pack"),
        )
        .expect("target runtime");
        let mut game = RuntimeGameShell::new_game_at_runtime_tile(
            root,
            source.clone(),
            source.title_new_game_spawn_identifier().expect("spawn"),
            "MnRochesterGym",
            6,
            17,
        )
        .expect("local gym fixture");
        game.session_mut().state_mut().money = 4321;
        let temp = StagingDirectory::create(&std::env::temp_dir()).expect("test directory");
        let old = temp.0.join("original.crystalsave");
        let new = temp.0.join("review.crystalsave");
        source
            .save_game(&old, game.session().state().clone())
            .expect("source fixture");
        let old_bytes = fs::read(&old).expect("source bytes");
        let old_backup = fs::read(format!("{}.bak", old.display())).expect("source backup");
        migrate(&source_pack, &target_pack, &old, &new).expect("migrate to new slot");
        assert_eq!(fs::read(&old).expect("source intact"), old_bytes);
        assert_eq!(
            fs::read(format!("{}.bak", old.display())).expect("backup intact"),
            old_backup
        );
        assert_eq!(target.load_save(&new).expect("target Continue").money, 4321);
        let new_bytes = fs::read(&new).expect("new bytes");
        assert!(migrate(&source_pack, &target_pack, &old, &new).is_err());
        assert_eq!(fs::read(&new).expect("destination intact"), new_bytes);
        assert!(migrate(&source_pack, &target_pack, &old, &old).is_err());
        let occupied = temp.0.join("occupied.crystalsave");
        fs::write(format!("{}.bak", occupied.display()), b"keep").expect("existing backup");
        assert!(migrate(&source_pack, &target_pack, &old, &occupied).is_err());
        assert_eq!(
            fs::read(format!("{}.bak", occupied.display())).expect("backup preserved"),
            b"keep"
        );
        assert!(
            migrate(
                &target_pack,
                &source_pack,
                &old,
                &temp.0.join("wrong.crystalsave")
            )
            .is_err()
        );
        assert!(
            migrate(
                &source_pack,
                &target_pack,
                &old,
                &temp.0.join("bad name.crystalsave")
            )
            .is_err()
        );
        assert!(!temp.0.join("bad name.crystalsave").exists());
        fs::write(&old, b"damaged primary").expect("damaged source fixture");
        let rejected = temp.0.join("rejected.crystalsave");
        assert!(migrate(&source_pack, &target_pack, &old, &rejected).is_err());
        assert_eq!(
            fs::read(&old).expect("primary remains damaged"),
            b"damaged primary"
        );
        assert_eq!(
            fs::read(format!("{}.bak", old.display())).expect("backup unchanged"),
            old_backup
        );
        assert!(!rejected.exists());
        assert!(fs::read_dir(&temp.0).expect("test directory").all(|entry| {
            !entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .starts_with(".geothite-gym-migration-")
        }));
    }
}
