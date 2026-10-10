//! Optional external CC0 scenery sources, separate from the checked-in catalog.
//! Model caches freeze each selected source on first load; restart to refresh.
use crate::model_storage::Source;

const MAX_BYTES: u64 = 16 * 1024 * 1024;

const MAX_BUNDLE_BYTES: usize = 64 * 1024 * 1024;
static BUNDLE: std::sync::OnceLock<std::collections::BTreeMap<String, String>> =
    std::sync::OnceLock::new();

/// Install optional external furniture and foliage before the first scenery lookup.
/// The versioned JSON bundle is `{ "version": 1, "models": { "models/interiors/…": mesh } }`.
/// Foliage also accepts `models/new_bark/tree.mesh.json` and
/// `models/johto/tree_lod.mesh.json` and four whitelisted tree variant paths;
/// other scenery paths are unsupported.
/// Downloads and converted geometry belong in ignored external content, never
/// in the source catalog. A malformed bundle is rejected atomically; already
/// cached model choices cannot be changed without restarting the client.
pub fn install_open_model_bundle(json: &str) -> Result<(), String> {
    let models = parse_bundle(json)?;
    BUNDLE
        .set(models)
        .map_err(|_| "scenery sources are already frozen; restart to refresh".into())
}

fn parse_bundle(json: &str) -> Result<std::collections::BTreeMap<String, String>, String> {
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Bundle {
        version: u32,
        models: std::collections::BTreeMap<String, serde_json::Value>,
    }
    if json.len() > MAX_BUNDLE_BYTES {
        return Err("open scenery bundle exceeds 64 MiB".into());
    }
    let bundle: Bundle = serde_json::from_str(json).map_err(|e| e.to_string())?;
    if bundle.version != 1 || bundle.models.len() > 128 {
        return Err("unsupported scenery bundle version or model count".into());
    }
    let mut models = std::collections::BTreeMap::new();
    for (path, value) in bundle.models {
        if !supported_path(&path) {
            return Err(format!("unsupported bundled scenery path: {path}"));
        }
        let json = serde_json::to_string(&value).map_err(|e| e.to_string())?;
        if json.len() as u64 > MAX_BYTES {
            return Err(format!("open model exceeds 16 MiB: {path}"));
        }
        if path.starts_with("models/interiors/") {
            crate::interior_models::Model::parse(json.as_str()).map(|_| ())
        } else {
            crate::new_bark_models::Model::parse(json.as_str()).map(|_| ())
        }
        .map_err(|e| format!("invalid bundled scenery {path}: {e}"))?;
        models.insert(path, json);
    }
    Ok(models)
}

fn supported_path(path: &str) -> bool {
    (path.starts_with("models/interiors/")
        || matches!(
            path,
            "models/new_bark/tree.mesh.json" | "models/johto/tree_lod.mesh.json"
        ) || crate::foliage::VARIANT_PATHS.contains(&path))
        && !path
            .split('/')
            .any(|p| matches!(p, ".." | ".") || p.is_empty())
}

#[cfg(not(target_arch = "wasm32"))]
fn read(root: &std::path::Path, path: &str) -> Result<Option<String>, String> {
    use std::io::Read;
    // Only known rendering catalogs can be overridden, never game content.
    if !supported_path(path) {
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
    if let Some(json) = BUNDLE.get_or_init(Default::default).get(path) {
        // Bundles were validated before installation; retain the fallback if
        // a future model parser changes its accepted schema.
        if let Ok(model) = parse(Source::Plain(json)) {
            return model;
        }
    }
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
    fn bundles_validate_real_geometry_and_reject_partial_or_out_of_scope_installs() {
        let model: serde_json::Value = crate::model_storage::parse(
            crate::model_storage::include_model!("models/interiors/television.mesh.json"),
        )
        .unwrap();
        let valid = serde_json::json!({"version":1,"models":{"models/interiors/television.mesh.json":model}});
        let models = parse_bundle(&valid.to_string()).unwrap();
        assert!(models.contains_key("models/interiors/television.mesh.json"));
        let mut invalid = valid.clone();
        invalid["models"]["models/interiors/chair.mesh.json"] =
            serde_json::json!({"primitives":[]});
        assert!(parse_bundle(&invalid.to_string()).is_err());
        invalid = valid.clone();
        invalid["version"] = 2.into();
        assert!(parse_bundle(&invalid.to_string()).is_err());
        for path in [
            "models/interiors/../television.mesh.json",
            "models/battle_species/pidgeotto.mesh.json",
        ] {
            let mut invalid = valid.clone();
            invalid["models"][path] = model.clone();
            assert!(parse_bundle(&invalid.to_string()).is_err());
        }
    }

    #[test]
    fn installed_bundle_reaches_the_cached_renderer_and_cannot_be_replaced() {
        // Other catalog tests legitimately initialize the process-wide cache.
        // Run this real renderer lookup in a fresh filtered test process.
        const CHILD: &str = "GEOTHITE_OPEN_BUNDLE_TEST_CHILD";
        if std::env::var_os(CHILD).is_none() {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "open_models::tests::installed_bundle_reaches_the_cached_renderer_and_cannot_be_replaced", "--nocapture"])
                .env(CHILD, "1").output().unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            return;
        }
        let color = [0.123, 0.456, 0.789, 1.0];
        let fixture = serde_json::json!({"primitives":[{
            "positions":[0,0,0, 2,0,0, 0,3,0, 0,0,4],
            "normals":[0,1,0, 0,1,0, 0,1,0, 0,1,0],
            "indices":[0,2,1, 0,1,3, 0,3,2, 1,2,3], "base_color":color
        }], "preserve_aspect":true});
        let bundle = serde_json::json!({"version":1,"models":{"models/interiors/television.mesh.json":fixture}}).to_string();
        install_open_model_bundle(&bundle).unwrap();
        let model = crate::interior_models::model(crate::interior_models::ModelKind::Television);
        let mut mesh = crate::mesh::SurfaceMeshData::default();
        model.append_fitted(&mut mesh, [0., 2., 0., 4.], 0., 3.);
        assert_eq!(mesh.positions.len(), 4);
        assert_eq!(mesh.indices.len(), 12);
        assert_eq!(
            mesh.positions,
            [[0., 0., 0.], [2., 0., 0.], [0., 3., 0.], [0., 0., 4.]]
        );
        let lit = crate::interior_models::authored_face_color(color, bevy::prelude::Vec3::Y);
        assert!(mesh.colors.iter().all(|c| *c == lit));
        assert!(install_open_model_bundle(&bundle).is_err());
        let mut other = crate::mesh::SurfaceMeshData::default();
        crate::interior_models::model(crate::interior_models::ModelKind::Stool).append_fitted(
            &mut other,
            [0., 1., 0., 1.],
            0.,
            1.,
        );
        assert!(!other.positions.is_empty());
        assert!(other.colors.iter().any(|c| *c != color));
    }

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
