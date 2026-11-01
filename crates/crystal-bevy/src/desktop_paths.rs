//! Installed resources are read-only; user data survives app replacements.
use std::path::{Path, PathBuf};

pub fn save_stem(id: &str, modpack_hash: &str, content_hash: &str) -> String {
    let label: String = id
        .chars()
        .take(16)
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '+') {
                character
            } else {
                '_'
            }
        })
        .collect();
    format!("{label}-{modpack_hash}-{content_hash}")
}

/// Check the actual filesystem's limit, including the longest recovery-write
/// suffix. The uniquely named probe never uses a real save's name.
pub fn check_save_writes(path: &Path) -> std::io::Result<()> {
    use std::io::Write;
    let parent = path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent)?;
    let mut atomic_name = path
        .file_name()
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing save filename")
        })?
        .to_os_string();
    atomic_name.push(".bak.tmp-4294967295");
    #[cfg(windows)]
    let length = {
        use std::os::windows::ffi::OsStrExt;
        atomic_name.encode_wide().count()
    };
    #[cfg(not(windows))]
    let length = atomic_name.as_encoded_bytes().len();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
        ^ u64::from(std::process::id());
    let marker = format!(".g{nonce:016x}");
    let probe = parent.join(format!(
        "{marker}{}",
        "a".repeat(length.saturating_sub(marker.len()))
    ));
    let mut output = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)?;
    let result = output.write_all(b"save directory check");
    drop(output);
    let cleanup = std::fs::remove_file(&probe);
    result.and(cleanup)
}

/// Preserve old portable saves during a filename migration. Publishing with a
/// hard link is atomic and cannot replace a newer save from another launch.
pub fn copy_save_if_missing(source: &Path, destination: &Path) -> std::io::Result<()> {
    let mut input = std::fs::File::open(source)?;
    let parent = destination.parent().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing save directory")
    })?;
    std::fs::create_dir_all(parent)?;
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temporary = parent.join(format!(
        ".geothite-migrate-{}-{nonce:x}",
        std::process::id()
    ));
    let mut output = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let written = std::io::copy(&mut input, &mut output).and_then(|_| output.sync_all());
    drop(output);
    let published = written.and_then(|_| match std::fs::hard_link(&temporary, destination) {
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        result => result,
    });
    let cleanup = std::fs::remove_file(&temporary);
    published.and(cleanup)
}

pub fn resources(executable: &Path) -> Option<PathBuf> {
    let directory = executable.parent()?;
    if directory.file_name()? == "MacOS" && directory.parent()?.file_name()? == "Contents" {
        Some(directory.parent()?.join("Resources"))
    } else {
        Some(directory.to_path_buf())
    }
}

#[cfg(any(feature = "desktop-app", test))]
pub fn save_root() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    let root = std::env::var_os("HOME")
        .map(|home| PathBuf::from(home).join("Library/Application Support/Geothite"));
    #[cfg(target_os = "windows")]
    let root = std::env::var_os("LOCALAPPDATA").map(|data| PathBuf::from(data).join("Geothite"));
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let root = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
        .map(|data| data.join("geothite"));
    root.map(|root| root.join("saves"))
}

#[cfg(all(target_os = "macos", feature = "desktop-app"))]
pub fn show_startup_error(message: &str) {
    use std::ffi::{CString, c_char, c_void};
    if let Some(home) = std::env::var_os("HOME") {
        let directory = PathBuf::from(home).join("Library/Logs/Geothite");
        let _ = std::fs::create_dir_all(&directory);
        let _ = std::fs::write(directory.join("startup-error.log"), message);
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFStringCreateWithCString(
            allocator: *const c_void,
            string: *const c_char,
            encoding: u32,
        ) -> *const c_void;
        fn CFRelease(value: *const c_void);
        fn CFUserNotificationDisplayAlert(
            timeout: f64,
            flags: usize,
            icon: *const c_void,
            sound: *const c_void,
            localization: *const c_void,
            title: *const c_void,
            message: *const c_void,
            default_button: *const c_void,
            alternate_button: *const c_void,
            other_button: *const c_void,
            response: *mut usize,
        ) -> i32;
    }
    let null = std::ptr::null();
    let message = CString::new(message.replace('\0', " ")).expect("sanitized startup error");
    // CoreFoundation owns each created string until it is released after the
    // synchronous alert. 0x08000100 is kCFStringEncodingUTF8 from Apple's SDK.
    unsafe {
        let title = CFStringCreateWithCString(null, c"Geothite".as_ptr(), 0x08000100);
        let body = CFStringCreateWithCString(null, message.as_ptr(), 0x08000100);
        if !title.is_null() && !body.is_null() {
            let mut response = 0;
            CFUserNotificationDisplayAlert(
                0.0,
                0,
                null,
                null,
                null,
                title,
                body,
                null,
                null,
                null,
                &mut response,
            );
        }
        if !title.is_null() {
            CFRelease(title);
        }
        if !body.is_null() {
            CFRelease(body);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relocated_mac_bundle_and_portable_layouts() {
        assert_eq!(
            resources(Path::new(
                "/Volumes/Game Disk/Geothite.app/Contents/MacOS/Geothite"
            )),
            Some(PathBuf::from(
                "/Volumes/Game Disk/Geothite.app/Contents/Resources"
            ))
        );
        assert_eq!(
            resources(Path::new("/opt/geothite/Geothite")),
            Some(PathBuf::from("/opt/geothite"))
        );
    }

    #[test]
    fn installed_saves_are_outside_the_application() {
        let root = save_root().expect("user data directory");
        assert!(root.is_absolute());
        assert_eq!(root.file_name().unwrap(), "saves");
        assert!(!root.to_string_lossy().contains(".app/"));
    }

    #[test]
    fn long_mod_names_leave_room_for_atomic_save_and_backup_files() {
        let directory =
            std::env::temp_dir().join(format!("geothite-save-size-{}", std::process::id()));
        std::fs::create_dir(&directory).unwrap();
        let stem = save_stem(&"🐉".repeat(150), &"a".repeat(64), &"b".repeat(64));
        assert!(stem.contains(&"a".repeat(64)) && stem.contains(&"b".repeat(64)));
        let good = check_save_writes(&directory.join(format!("{stem}.crystalsave")));
        let bad = check_save_writes(&directory.join(format!("{}.crystalsave", "c".repeat(235))));
        let remaining = std::fs::read_dir(&directory).unwrap().count();
        std::fs::remove_dir(&directory).unwrap();
        good.expect("bounded save and atomic recovery writes must fit");
        assert!(bad.is_err(), "oversized recovery writes must be detected");
        assert_eq!(remaining, 0, "write probes must clean up");
    }

    #[test]
    fn save_migration_preserves_originals_and_never_overwrites_newer_progress() {
        let directory =
            std::env::temp_dir().join(format!("geothite-save-copy-{}", std::process::id()));
        std::fs::create_dir(&directory).unwrap();
        let source = directory.join("legacy.crystalsave");
        let destination = directory.join("current.crystalsave");
        std::fs::write(&source, b"original").unwrap();
        copy_save_if_missing(&source, &destination).unwrap();
        assert_eq!(std::fs::read(&destination).unwrap(), b"original");
        std::fs::write(&destination, b"newer progress").unwrap();
        copy_save_if_missing(&source, &destination).unwrap();
        assert_eq!(std::fs::read(&source).unwrap(), b"original");
        assert_eq!(std::fs::read(&destination).unwrap(), b"newer progress");
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 2);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
