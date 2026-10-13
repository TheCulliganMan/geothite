//! Enumerate exported Rust catalogs into the external compiler manifest.
use anyhow::{Context, Result, ensure};
use crystal_assets::{ContentPack, ContentPackFiles, ContentPackIndex};
use serde_json::json;
use std::path::Path;

pub fn export(root: &Path) -> Result<()> {
    let directory = root.join("apps/web/assets/data/content-packs");
    let pack_root = directory.join("core-modular");
    let mut files = serde_json::to_value(ContentPackFiles::default())?;
    for (category, value) in files.as_object_mut().context("content file categories")? {
        let path = pack_root.join(category);
        if !path.exists() { continue; }
        let mut names = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let path = entry?.path();
            if path.extension().is_some_and(|extension| extension == "json") {
                names.push(format!("{category}/{}", path.file_name().context("catalog filename")?.to_str().context("UTF-8 catalog filename")?));
            }
        }
        names.sort();
        *value = json!(names);
    }
    for category in ["map_scripts", "map_blocks", "map_attributes", "map_dimensions", "npcs", "items", "field_moves", "field_box_items", "runtime_spawn_points", "runtime_map_metadata", "runtime_title_screen", "asm_text", "tilesets", "playability"] {
        ensure!(files[category].as_array().is_some_and(|values| !values.is_empty()), "required authored source category missing: {category}");
    }
    let generated = ContentPack {
        id: "core-modular".into(), enabled: true, priority: -100,
        path: "content-packs/core-modular".into(), compiled: None,
        files: serde_json::from_value(files)?,
    };
    let mut indexed = generated.clone();
    indexed.compiled = Some("content-packs/core-modular.crystalpack".into());
    indexed.files = ContentPackFiles::default();
    let index = ContentPackIndex { version: 1, packs: vec![indexed] };
    index.validate()?;
    std::fs::write(directory.join("core-modular.generated.json"), serde_json::to_vec_pretty(&generated)?)?;
    std::fs::write(directory.join("index.json"), serde_json::to_vec_pretty(&index)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exported_index_uses_compiled_pack_and_generated_manifest_retains_sources() {
        let root = std::env::temp_dir().join(format!("geothite-source-manifest-{}-{}", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let result = (|| -> Result<()> {
            let directory = root.join("apps/web/assets/data/content-packs");
            for category in ["map_scripts", "map_blocks", "map_attributes", "map_dimensions", "npcs", "items",
                "field_moves", "field_box_items", "runtime_spawn_points", "runtime_map_metadata",
                "runtime_title_screen", "asm_text", "tilesets", "playability", "pokemon"] {
                let path = directory.join("core-modular").join(category);
                std::fs::create_dir_all(&path)?;
                std::fs::write(path.join(if category == "pokemon" { "ABRA.json" } else { "catalog.json" }), "{}")?;
            }
            export(&root)?;
            let generated: ContentPack = serde_json::from_slice(&std::fs::read(directory.join("core-modular.generated.json"))?)?;
            let index: ContentPackIndex = serde_json::from_slice(&std::fs::read(directory.join("index.json"))?)?;
            index.validate()?;
            ensure!(generated.compiled.is_none(), "compiler source manifest must remain authored");
            ensure!(serde_json::to_value(&generated.files)?["pokemon"] == json!(["pokemon/ABRA.json"]),
                "compiler source manifest lost its Pokemon catalog");
            ensure!(index.packs[0].compiled.as_deref() == Some("content-packs/core-modular.crystalpack"),
                "published index lost its compiled pack reference");
            ensure!(serde_json::to_value(&index.packs[0].files)? == serde_json::to_value(ContentPackFiles::default())?,
                "published index still declares raw files beside compiled content");
            Ok(())
        })();
        let _ = std::fs::remove_dir_all(&root);
        result.unwrap();
    }
}
