//! Optional external CC0 scenery sources, separate from the checked-in catalog.
//! Model caches freeze each selected source on first load; restart to refresh.
use crate::model_storage::Source;

#[cfg(not(target_arch = "wasm32"))]
const MAX_BYTES: u64 = 16 * 1024 * 1024;

#[cfg(not(target_arch = "wasm32"))]
fn read(root: &std::path::Path, path: &str) -> Result<Option<String>, String> {
    use std::io::Read;
    // Only known rendering catalogs can be overridden, never game content.
    if !(path.starts_with("models/interiors/")
        || matches!(
            path,
            "models/new_bark/tree.mesh.json" | "models/johto/tree_lod.mesh.json"
        ))
        || path
            .split('/')
            .any(|p| matches!(p, ".." | ".") || p.is_empty())
    {
        return Err("unsupported open model path".into());
    }
    let path = root.join(path.trim_start_matches("models/"));
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("open model exceeds 16 MiB".into());
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|e| e.to_string())
}

pub(crate) fn load<T>(
    path: &str,
    fallback: Source<'_>,
    parse: impl for<'a> Fn(Source<'a>) -> Result<T, String>,
) -> T {
    #[cfg(not(target_arch = "wasm32"))]
    {
        static ROOT: std::sync::OnceLock<Option<std::path::PathBuf>> = std::sync::OnceLock::new();
        let root = ROOT.get_or_init(|| {
            std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT")
                .filter(|p| !p.is_empty())
                .map(Into::into)
        });
        load_from_root(root.as_deref(), path, fallback, parse)
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = path;
        parse(fallback).expect("validated authored model fallback")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn load_from_root<T>(
    root: Option<&std::path::Path>,
    path: &str,
    fallback: Source<'_>,
    parse: impl for<'a> Fn(Source<'a>) -> Result<T, String>,
) -> T {
    if let Some(root) = root {
        let result = read(root, path);
        match result {
            Ok(Some(json)) => match parse(Source::Plain(&json)) {
                Ok(model) => return model,
                Err(error) => trace(path, &error),
            },
            Err(error) => trace(path, &error),
            Ok(None) => {}
        }
    }
    parse(fallback).expect("validated authored model fallback")
}

#[cfg(not(target_arch = "wasm32"))]
fn trace(path: &str, error: &str) {
    // A renderer may share a fullscreen terminal process. No unconditional
    // stdout/stderr output, including when an optional file is malformed.
    if std::env::var_os("CRYSTAL_OPEN_MODEL_TRACE").is_some() {
        eprintln!("open model {path}: {error}; using authored fallback");
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    #[test]
    fn external_sources_are_scoped_and_missing_files_are_optional() {
        let root = std::env::temp_dir().join("geothite-open-model-missing");
        assert!(
            read(&root, "models/interiors/chair.mesh.json")
                .unwrap()
                .is_none()
        );
        assert!(read(&root, "models/../battle_species/pidgeotto.glb").is_err());
        assert!(read(&root, "models/battle_species/pidgeotto.glb").is_err());
        assert!(read(&root, "/etc/passwd").is_err());
    }

    #[test]
    fn valid_external_data_is_used_and_bad_data_retains_the_authored_fallback() {
        let root = std::env::temp_dir().join(format!(
            "geothite-open-model-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        std::fs::create_dir_all(root.join("interiors")).unwrap();
        let _cleanup = Cleanup(root.clone());
        let file = root.join("interiors/chair.mesh.json");
        let load = || {
            load_from_root(
                Some(&root),
                "models/interiors/chair.mesh.json",
                Source::Plain("[7]"),
                |s| crate::model_storage::parse::<Vec<i32>>(s),
            )
        };
        assert_eq!(load(), [7]);
        std::fs::write(&file, "[11,12]").unwrap();
        assert_eq!(load(), [11, 12]);
        std::fs::write(&file, "{bad JSON}").unwrap();
        assert_eq!(load(), [7]);
        std::fs::write(&file, [255_u8]).unwrap();
        assert_eq!(load(), [7]);
        std::fs::File::create(&file)
            .unwrap()
            .set_len(MAX_BYTES + 1)
            .unwrap();
        assert_eq!(load(), [7]);
    }
}
