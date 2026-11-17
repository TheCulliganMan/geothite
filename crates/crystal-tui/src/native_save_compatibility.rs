//! Scope automatic release migration to installer-owned, checksum-named slots.

use std::path::Path;

/// Explicit custom load/save paths retain ordinary same-pack recovery. They do
/// not gain automatic cross-release lookup merely because the launcher supplies
/// its save directory. A malformed/non-owned name therefore returns `None`.
pub(crate) fn owned_slot_suffix(
    directory: &Path,
    destination: &Path,
    pack_hash: &str,
) -> Option<String> {
    if destination.parent() != Some(directory) {
        return None;
    }
    let suffix = destination.file_name()?.to_str()?.strip_prefix(pack_hash)?;
    if suffix == ".crystalsave" {
        return Some(suffix.to_owned());
    }
    let session = suffix
        .strip_prefix("-session-")?
        .strip_suffix(".crystalsave")?;
    (!session.is_empty()
        && session.len() <= 64
        && session
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')))
    .then(|| suffix.to_owned())
}

#[cfg(test)]
mod tests {
    use super::owned_slot_suffix;
    use std::path::Path;

    #[test]
    fn installed_default_and_named_slots_preserve_their_identity() {
        let directory = Path::new("/owned/saves");
        let hash = "a".repeat(64);
        for suffix in [".crystalsave", "-session-weekend_2.crystalsave"] {
            assert_eq!(
                owned_slot_suffix(directory, &directory.join(format!("{hash}{suffix}")), &hash),
                Some(suffix.into())
            );
        }
    }

    #[test]
    fn explicit_custom_paths_do_not_import_an_unrelated_previous_slot() {
        let directory = Path::new("/owned/saves");
        let hash = "a".repeat(64);
        for path in [
            Path::new("/custom/progress.crystalsave").to_path_buf(),
            directory.join("custom.crystalsave"),
            directory.join(format!("{hash}-session-.crystalsave")),
            directory.join(format!("{hash}-session-other.slot.crystalsave")),
            directory.join(format!(
                "{}{}.crystalsave",
                "b".repeat(64),
                "-session-weekend"
            )),
        ] {
            assert_eq!(owned_slot_suffix(directory, &path, &hash), None);
        }
    }
}
