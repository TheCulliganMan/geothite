//! Assemble external furniture/foliage for the browser without embedding assets.
use std::{collections::BTreeMap, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err(
            "usage: bundle_open_models EXTERNAL_CONVERTED_ROOT EXTERNAL_OUTPUT.json".into(),
        );
    }
    let root = PathBuf::from(&args[0]);
    let output = PathBuf::from(&args[1]);
    let mut models = BTreeMap::new();
    let interiors = match std::fs::read_dir(root.join("interiors")) {
        Ok(entries) => Some(entries),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.into()),
    };
    for entry in interiors.into_iter().flatten() {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_str().ok_or("model filename is not UTF-8")?;
        if !name.ends_with(".mesh.json") {
            continue;
        }
        if entry.metadata()?.len() > 16 * 1024 * 1024 {
            return Err("model exceeds 16 MiB".into());
        }
        let value: serde_json::Value = serde_json::from_slice(&std::fs::read(entry.path())?)?;
        models.insert(format!("models/interiors/{name}"), value);
    }
    for path in ["new_bark/tree.mesh.json", "johto/tree_lod.mesh.json",
        "johto/tree_variants/tree_2.mesh.json", "johto/tree_variants/tree_3.mesh.json",
        "johto/tree_variants/tree_4.mesh.json", "johto/tree_variants/tree_5.mesh.json"] {
        let file = root.join(path);
        if file.is_file() {
            if file.metadata()?.len() > 16 * 1024 * 1024 {
                return Err("model exceeds 16 MiB".into());
            }
            models.insert(
                format!("models/{path}"),
                serde_json::from_slice(&std::fs::read(file)?)?,
            );
        }
    }
    if models.is_empty() {
        return Err("no converted scenery meshes found".into());
    }
    let json = serde_json::to_string(&serde_json::json!({"version":1,"models":models}))?;
    crystal_voxel_view::install_open_model_bundle(&json)?;
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output, &json)?;
    use std::io::Write as _;
    let mut compressed = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    compressed.write_all(json.as_bytes())?;
    let mut gzip = output.into_os_string();
    gzip.push(".gz");
    std::fs::write(PathBuf::from(gzip), compressed.finish()?)?;
    Ok(())
}
