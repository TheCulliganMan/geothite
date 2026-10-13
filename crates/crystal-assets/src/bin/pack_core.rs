#[path = "pack_core/nickname_text_export.rs"]
mod nickname_text_export;
#[path = "pack_core/item_name_export.rs"]
mod item_name_export;
#[path = "pack_core/source_items.rs"]
mod source_items;
#[path = "pack_core/source_scripts.rs"]
mod source_scripts;
#[path = "pack_core/source_global_scripts.rs"]
mod source_global_scripts;
#[path = "pack_core/source_species.rs"]
mod source_species;
#[path = "pack_core/source_progression.rs"]
mod source_progression;
#[path = "pack_core/source_manifest.rs"]
mod source_manifest;
#[path = "pack_core/source_playability.rs"]
mod source_playability;
#[path = "pack_core/source_special_tables.rs"]
mod source_special_tables;
#[path = "pack_core/source_roaming.rs"]
mod source_roaming;
#[path = "pack_core/source_metadata.rs"]
mod source_metadata;
#[path = "pack_core/source_economy.rs"]
mod source_economy;
#[path = "pack_core/source_fishing.rs"]
mod source_fishing;
#[path = "pack_core/source_gifts.rs"]
mod source_gifts;
#[path = "pack_core/source_presentation_data.rs"]
mod source_presentation_data;
#[path = "pack_core/source_title_screen.rs"]
mod source_title_screen;
#[path = "pack_core/source_title_catalog.rs"]
mod source_title_catalog;
#[path = "pack_core/source_asm_text.rs"]
mod source_asm_text;
#[path = "pack_core/source_trainers.rs"]
mod source_trainers;
#[path = "pack_core/source_pokegear.rs"]
mod source_pokegear;
#[path = "pack_core/source_wild.rs"]
mod source_wild;
#[path = "pack_core/source_animation_catalogs.rs"]
mod source_animation_catalogs;
#[path = "pack_core/source_phone.rs"]
mod source_phone;
#[path = "pack_core/source_tower_eggs.rs"]
mod source_tower_eggs;
#[path = "pack_core/source_battle_rules.rs"]
mod source_battle_rules;
#[path = "pack_core/source_npcs.rs"]
mod source_npcs;
#[path = "pack_core/source_map_geometry.rs"]
mod source_map_geometry;
#[path = "pack_core/source_world.rs"]
mod source_world;
#[path = "pack_core/source_expressions.rs"]
mod source_expressions;
#[path = "pack_core/source_syntax.rs"]
mod source_syntax;
#[path = "pack_core/source_audio.rs"]
mod source_audio;
#[path = "pack_core/source_rules.rs"]
mod source_rules;
#[path = "pack_core/source_field_moves.rs"]
mod source_field_moves;
#[path = "pack_core/source_field_encounters.rs"]
mod source_field_encounters;
#[path = "pack_core/source_tilesets.rs"]
mod source_tilesets;
#[path = "pack_core/source_constants.rs"]
mod source_constants;
#[path = "pack_core/battle_oam_export.rs"]
mod battle_oam_export;
#[path = "pack_core/source_battle_bundle.rs"]
mod source_battle_bundle;
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
    if let Some(stage) = args.next() {
        anyhow::ensure!(args.next().is_none(), "usage: pack_core <repository-root> [--export-world|--export-battle]");
        let root = PathBuf::from(repository_root);
        verify_pret_source_pin(&root)?;
        return match stage.as_str() {
            "--export-world" => source_world::export(&root),
            "--export-npcs" => source_npcs::export(&root),
            "--export-items" => item_name_export::export_source(&root),
            "--export-species" => source_species::export(&root),
            "--export-constants" => source_constants::export(&root),
            "--export-global-scripts" => source_global_scripts::export(&root),
            "--export-progression" => source_progression::export(&root),
            "--export-manifest" => source_manifest::export(&root),
            "--export-playability" => source_playability::export(&root),
            "--export-title" => source_title_screen::export(&root),
            "--export-title-unvalidated" => source_title_screen::export_unvalidated(&root),
            "--export-intro-trace" => {
                let trace = source_title_screen::intro_trace(&root.join("vendor/pokecrystal"))?;
                fs::write(root.join("intro-trace.json"), serde_json::to_vec(&trace)?)?;
                Ok(())
            },
            "--export-special-tables" => source_special_tables::export(&root),
            "--export-roaming" => source_roaming::export(&root),
            "--export-metadata" => source_metadata::export(&root),
            "--export-economy" => source_economy::export(&root),
            "--export-fishing" => source_fishing::export(&root),
            "--export-gifts" => source_gifts::export(&root),
            "--export-presentation-data" => source_presentation_data::export(&root),
            "--export-asm-text" => source_asm_text::export(&root),
            "--export-trainers" => source_trainers::export(&root),
            "--export-pokegear" => source_pokegear::export(&root),
            "--export-wild" => source_wild::export(&root),
            "--export-animation-catalogs" => source_animation_catalogs::export(&root),
            "--export-phone" => source_phone::export(&root),
            "--export-tower-eggs" => source_tower_eggs::export(&root),
            "--export-battle-rules" => source_battle_rules::export(&root),
            "--export-battle" => battle_oam_export::export(&root),
            "--export-audio" => source_audio::export(&root),
            "--export-rules" => source_rules::export(&root),
            "--export-field-moves" => source_field_moves::export(&root),
            "--export-field-encounters" => source_field_encounters::export(&root),
            _ => anyhow::bail!("unknown source export stage {stage}"),
        };
    }

    export_core_pack(&PathBuf::from(repository_root))
}

fn export_core_pack(repository_root: &Path) -> Result<()> {
    verify_pret_source_pin(repository_root)?;
    source_constants::export(repository_root).context("export source numeric constants")?;
    source_species::export(repository_root).context("export source Pokemon and moves")?;
    item_name_export::export_source(repository_root).context("export source item names")?;
    source_rules::export(repository_root).context("export source gameplay rules")?;
    source_battle_rules::export(repository_root).context("export source battle multiplier rules")?;
    source_field_moves::export(repository_root).context("export source field move rules")?;
    source_progression::export(repository_root).context("export source species progression")?;
    source_special_tables::export(repository_root).context("export authored special tables")?;
    source_roaming::export(repository_root).context("export authored roaming Pokemon")?;
    source_metadata::export(repository_root).context("export authored cries and initialization")?;
    source_economy::export(repository_root).context("export authored shops and currency")?;
    source_fishing::export(repository_root).context("export authored fishing tables")?;
    source_gifts::export(repository_root).context("export authored gift and happiness tables")?;
    source_tower_eggs::export(repository_root).context("export authored Battle Tower and Odd Eggs")?;
    source_presentation_data::export(repository_root).context("export authored presentation data")?;
    source_asm_text::export(repository_root).context("export authored global text")?;
    source_trainers::export(repository_root).context("export authored trainers")?;
    source_npcs::export(repository_root).context("export source map objects")?;
    source_world::export(repository_root).context("export source map geometry and scripts")?;
    source_pokegear::export(repository_root).context("export authored Pokegear metadata")?;
    source_wild::export(repository_root).context("export authored wild encounter tables")?;
    source_field_encounters::export(repository_root).context("export authored tree and rock encounters")?;
    source_global_scripts::export(repository_root).context("export authored global scripts")?;
    source_phone::export(repository_root).context("export authored phone catalogs")?;
    source_tilesets::export(repository_root).context("export source terrain and graphics")?;
    source_playability::export(repository_root).context("export authored playability start")?;
    export_pokegear_card_layouts(repository_root)?;
    radio_text_export::export(repository_root).context("export source radio text commands")?;
    nickname_text_export::export(repository_root).context("export source nickname text commands")?;
    frontpic_animation_export::export(repository_root).context("export source main and idle frontpic animations")?;
    battle_oam_export::export(repository_root).context("export source battle OAM")?;
    source_animation_catalogs::export(repository_root).context("export authored animation catalogs")?;
    source_audio::export(repository_root).context("export source audio programs")?;
    source_title_screen::export(repository_root).context("export authored title presentation")?;
    source_manifest::export(repository_root).context("index authored exported catalogs")?;
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

fn verify_pret_source_pin(repository: &Path) -> Result<()> {
    let pin: Value = serde_json::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../pret-source.lock.json")))?;
    let vendor = repository.join("vendor/pokecrystal");
    for (name, reference) in [("commit", "HEAD"), ("tree", "HEAD^{tree}")] {
        let output = std::process::Command::new("git").arg("-C").arg(&vendor).args(["rev-parse", reference]).output().context("read pret source identity")?;
        anyhow::ensure!(output.status.success(), "cannot read pret {name}");
        let actual = String::from_utf8(output.stdout)?;
        let expected = pin[name].as_str().context("pret source pin identity")?;
        anyhow::ensure!(actual.trim() == expected, "pret {name} {} differs from pinned {expected}; check out the pinned source before exporting", actual.trim());
    }
    let status = std::process::Command::new("git").arg("-C").arg(&vendor).args(["status", "--porcelain", "--untracked-files=no"]).output()?;
    anyhow::ensure!(status.status.success() && status.stdout.is_empty(), "pret tracked source differs from pinned checkout");
    let rgbds = std::fs::read_to_string(vendor.join(".rgbds-version"))?;
    anyhow::ensure!(Some(rgbds.trim()) == pin["rgbds"].as_str(), "pret RGBDS requirement differs from pin");
    let rom = vendor.join("pokecrystal.gbc");
    let hash = format!("{:x}", sha1::Sha1::digest(fs::read(&rom).context("build the pinned reference ROM before export")?));
    anyhow::ensure!(Some(hash.as_str()) == pin["reference_rom_sha1"].as_str(), "assembled reference ROM differs from pinned Crystal ROM");
    let lock = json!({
        "repository":pin["repository"],"commit":pin["commit"],"tree":pin["tree"],
        "input_manifest_sha256":hex_sha256(&serde_json::to_vec(&json!({"repository":pin["repository"],"commit":pin["commit"],"tree":pin["tree"]}))?),
        "rom":{"sha1":hash},"rgbds":{"version":pin["rgbds"]},
    });
    fs::write(repository.join("asm-source.lock.json"), serde_json::to_vec_pretty(&lock)?)?;
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
