#[path = "pack_core/nickname_text_export.rs"]
mod nickname_text_export;
#[path = "pack_core/item_name_export.rs"]
mod item_name_export;
#[path = "pack_core/battle_oam_export.rs"]
mod battle_oam_export;
#[path = "pack_core/frontpic_animation_export.rs"]
mod frontpic_animation_export;
#[path = "pack_core/radio_text_export.rs"]
mod radio_text_export;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use crystal_assets::modpack::{
    ModpackCompileOptions, ModpackCompiler, ModpackManifest, ModpackMetadata,
};
use crystal_assets::{AssetRoot, COMPILED_GAME_PACK_FORMAT_VERSION};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const PACK_RELATIVE_PATH: &str = "content-packs/core-modular.crystalpack";
const BROWSER_PACK_RELATIVE_PATH: &str = "content-packs/core-modular.browser.crystalpack";

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let repository_root = args.next().context("usage: pack_core <repository-root>")?;
    if args.next().is_some() {
        anyhow::bail!("usage: pack_core <repository-root>");
    }

    export_core_pack(&PathBuf::from(repository_root))
}

fn export_core_pack(repository_root: &Path) -> Result<()> {
    export_pokegear_card_layouts(repository_root)?;
    item_name_export::export(repository_root).context("export source item names")?;
    radio_text_export::export(repository_root).context("export source radio text commands")?;
    nickname_text_export::export(repository_root).context("export source nickname text commands")?;
    frontpic_animation_export::export(repository_root).context("export source main and idle frontpic animations")?;
    battle_oam_export::export(repository_root).context("export source battle OAM")?;
    let asset_root = AssetRoot::new(repository_root);
    let compiler = ModpackCompiler::new(&asset_root);
    let core_manifest = ModpackManifest {
        metadata: ModpackMetadata {
            id: "core-modular".to_string(),
            name: "Pokemon Crystal".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            description: None,
        },
        priority: -100,
        ..ModpackManifest::default()
    };
    let compiled = compiler
        .compile(&[core_manifest], ModpackCompileOptions::default())
        .context("compile core modpack from exported content pack")?;
    let runtime_pack = asset_root
        .runtime_assets()
        .join("data")
        .join(PACK_RELATIVE_PATH);
    compiled
        .write_game_pack(&runtime_pack)
        .context("write core runtime game pack")?;

    let tracked_pack = repository_root.join(PACK_RELATIVE_PATH);
    fs::create_dir_all(
        tracked_pack
            .parent()
            .context("tracked core pack path has no parent")?,
    )
    .context("create tracked content-pack directory")?;
    fs::copy(&runtime_pack, &tracked_pack).with_context(|| {
        format!(
            "copy runtime pack {} to {}",
            runtime_pack.display(),
            tracked_pack.display()
        )
    })?;
    let browser_pack = repository_root.join(BROWSER_PACK_RELATIVE_PATH);
    compiled
        .write_browser_game_pack(&browser_pack)
        .context("write browser pack with on-demand audio synthesis")?;
    write_provenance(repository_root, &tracked_pack)?;
    println!("exported {}", tracked_pack.display());
    println!("exported {}", browser_pack.display());
    Ok(())
}

fn export_pokegear_card_layouts(repository_root: &Path) -> Result<()> {
    for &relative in crystal_assets::REQUIRED_POKEGEAR_RUNTIME_FILE_KEYS {
        let source = repository_root.join("vendor/pokecrystal").join(relative);
        let target = repository_root.join("apps/web/assets").join(relative);
        let bytes = fs::read(&source).with_context(|| format!("read source Pokégear layout {}", source.display()))?;
        anyhow::ensure!(!bytes.is_empty(), "source Pokégear layout {} is empty", source.display());
        fs::create_dir_all(target.parent().context("Pokégear layout has no parent")?)?;
        fs::write(&target, bytes).with_context(|| format!("export Pokégear layout {}", target.display()))?;
    }
    Ok(())
}

fn write_provenance(repository_root: &Path, tracked_pack: &Path) -> Result<()> {
    let lock_path = repository_root.join("asm-source.lock.json");
    let lock: Value = serde_json::from_slice(
        &fs::read(&lock_path).with_context(|| format!("read {}", lock_path.display()))?,
    )
    .context("parse asm-source.lock.json")?;
    let pack_hash = hex_sha256(
        &fs::read(tracked_pack)
            .with_context(|| format!("read exported pack {}", tracked_pack.display()))?,
    );
    let provenance = json!({
        "schema": 2,
        "pack_format": COMPILED_GAME_PACK_FORMAT_VERSION,
        "pack": PACK_RELATIVE_PATH,
        "pack_sha256": pack_hash,
        "asm": {
            "repository": required_lock_string(&lock, "/repository")?,
            "commit": required_lock_string(&lock, "/commit")?,
            "tree": required_lock_string(&lock, "/tree")?,
            "input_manifest_sha256": required_lock_string(&lock, "/input_manifest_sha256")?,
            "rom_sha1": required_lock_string(&lock, "/rom/sha1")?,
        },
        "toolchain": {
            "rgbds": required_lock_string(&lock, "/rgbds/version")?,
            "exporter": "rust/crates/crystal-assets/src/bin/pack_core.rs",
        },
    });
    let provenance_path = tracked_pack.with_extension("crystalpack.provenance.json");
    fs::write(
        &provenance_path,
        format!("{}\n", serde_json::to_string_pretty(&provenance)?),
    )
    .with_context(|| format!("write {}", provenance_path.display()))?;
    Ok(())
}

fn required_lock_string<'a>(lock: &'a Value, pointer: &str) -> Result<&'a str> {
    lock.pointer(pointer)
        .and_then(Value::as_str)
        .with_context(|| format!("asm-source.lock.json is missing {pointer}"))
}

fn hex_sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_pack_bytes_for_provenance() {
        assert_eq!(
            hex_sha256(b"core pack"),
            "5101837ad8764794719dddedd25f2de24f8650c84039ce365fd8ccd318f3fb90"
        );
    }
}
