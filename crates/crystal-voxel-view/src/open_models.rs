//! Optional external CC0 scenery sources, separate from the checked-in catalog.
//! Model caches freeze each selected source on first load; restart to refresh.
use crate::model_storage::Source;

const MAX_BYTES: u64 = 16 * 1024 * 1024;

const MAX_BUNDLE_BYTES: usize = 64 * 1024 * 1024;
static BUNDLE: std::sync::OnceLock<
    Option<std::sync::Mutex<std::collections::BTreeMap<String, String>>>,
> =
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
        .set(Some(std::sync::Mutex::new(models)))
        .map_err(|_| "scenery sources are already frozen; restart to refresh".into())
}

fn parse_bundle(json: &str) -> Result<std::collections::BTreeMap<String, String>, String> {
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Bundle<'a> {
        version: u32,
        // Borrow each model's source rather than materializing every numeric
        // array in the bundle as a second generic JSON geometry tree.
        #[serde(borrow)]
        models: std::collections::BTreeMap<String, &'a serde_json::value::RawValue>,
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
        let json = value.get();
        if json.len() as u64 > MAX_BYTES {
            return Err(format!("open model exceeds 16 MiB: {path}"));
        }
        if path.starts_with("models/interiors/") {
            crate::interior_models::Model::parse(json).map(|_| ())
        } else {
            crate::new_bark_models::Model::parse(json).map(|_| ())
        }
        .map_err(|e| format!("invalid bundled scenery {path}: {e}"))?;
        models.insert(path, json.to_owned());
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

/// Load the designated external asset. Missing or invalid content stays missing;
/// callers retain its original 2D source instead of selecting another 3D model.
/// Called only by the selected model's OnceLock initializer. The typed cache
/// owns decoded geometry afterward, so its redundant bundled JSON can be freed.
pub(crate) fn load_required<T>(
    path: &str,
    parse: impl for<'a> Fn(Source<'a>) -> Result<T, String>,
) -> Option<T> {
    if let Some(bundle) = BUNDLE.get_or_init(|| None) {
        let json = bundle.lock().ok()?.remove(path)?;
        return parse(Source::Plain(&json)).ok();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let root = crate::open_people::model_root();
        return required_from_root(&root, path, parse);
    }
    #[cfg(target_arch = "wasm32")]
    None
}

#[cfg(not(target_arch = "wasm32"))]
fn required_from_root<T>(root: &std::path::Path, path: &str,
    parse: impl for<'a> Fn(Source<'a>) -> Result<T, String>) -> Option<T> {
    match read(root, path) {
        Ok(Some(json)) => match parse(Source::Plain(&json)) {
            Ok(asset) => Some(asset),
            Err(error) => { trace(path, &error); None }
        },
        Ok(None) => None,
        Err(error) => { trace(path, &error); None }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn trace(path: &str, error: &str) {
    // A renderer may share a fullscreen terminal process. No unconditional
    // stdout/stderr output, including when an optional file is malformed.
    if std::env::var_os("CRYSTAL_OPEN_MODEL_TRACE").is_some() {
        eprintln!("open model {path}: {error}; required 3D asset unavailable");
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    #[test]
    fn bundle_retains_exact_model_source_without_generic_json_expansion() {
        let model = r#"{ "primitives": [{ "positions": [0,0,-0.0, 2,0,0, 0,3,0, 0,0,4], "normals": [0,1,0, 0,1,0, 0,1,0, 0,1,0], "indices": [0,2,1, 0,1,3, 0,3,2, 1,2,3], "base_color": [0.25,0.5,0.75,1] }] }"#;
        let bundle = format!(r#"{{"version":1,"models":{{"models/interiors/television.mesh.json":{model}}}}}"#);
        let models = parse_bundle(&bundle).unwrap();
        assert_eq!(models["models/interiors/television.mesh.json"], model);
    }
    #[test]
    fn bundles_validate_real_geometry_and_reject_partial_or_out_of_scope_installs() {
        let model: serde_json::Value = crate::model_storage::parse(
            Source::Plain(include_str!("../../../assets/open-models/interiors/television.mesh.json")),
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
        // An installed bundle is the selected source. Missing bundle entries
        // cannot silently be filled from another local asset directory.
        assert!(load_required("models/new_bark/tree.mesh.json",
            |source| crate::new_bark_models::Model::parse(source)).is_none());
        let model = crate::interior_models::selected_model(crate::interior_models::ModelKind::Television).unwrap();
        assert!(!BUNDLE.get().unwrap().as_ref().unwrap().lock().unwrap()
            .contains_key("models/interiors/television.mesh.json"));
        assert!(std::ptr::eq(model,
            crate::interior_models::selected_model(crate::interior_models::ModelKind::Television).unwrap()),
            "selected geometry remains cached after releasing redundant source JSON");
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
    fn required_external_data_never_substitutes_missing_or_invalid_sources() {
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
            required_from_root(
                &root,
                "models/interiors/chair.mesh.json",
                |s| crate::model_storage::parse::<Vec<i32>>(s),
            )
        };
        assert_eq!(load(), None);
        std::fs::write(&file, "[11,12]").unwrap();
        assert_eq!(load(), Some(vec![11, 12]));
        std::fs::write(&file, "{bad JSON}").unwrap();
        assert_eq!(load(), None);
        std::fs::write(&file, [255_u8]).unwrap();
        assert_eq!(load(), None);
        std::fs::File::create(&file)
            .unwrap()
            .set_len(MAX_BYTES + 1)
            .unwrap();
        assert_eq!(load(), None);
    }
}
