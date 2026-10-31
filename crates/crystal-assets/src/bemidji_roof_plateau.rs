//! Complete the native timber roof and close an unsupported plateau side exit.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;

const MAP: &str = "MnBemidji";
const TILESET: &str = "mn_bemidji_plateau_stairs";
const TEMPORARY_ART: &str = "mn_bemidji_roof_plateau_import";
const MARKER: &str = "minnesota-bemidji-roof-plateau-v1";

#[derive(Deserialize)]
struct Terrain {
    schema_version: u8,
    id: String,
    tileset: String,
    dimensions_metatiles: [u16; 2],
    rows: Vec<String>,
}

impl Terrain {
    fn blocks(&self) -> Result<Vec<u16>> {
        ensure!(
            self.schema_version == 1
                && self.id == MAP
                && self.tileset == TILESET
                && self.dimensions_metatiles == [24, 24]
                && self.rows.len() == 24,
            "complete Bemidji source identity required"
        );
        let mut blocks = Vec::new();
        for row in &self.rows {
            let words = row.split_whitespace().collect::<Vec<_>>();
            ensure!(words.len() == 24, "complete Bemidji row required");
            for word in words {
                blocks.push(u16::from_str_radix(word, 16).context("native Bemidji block")?);
            }
        }
        Ok(blocks)
    }
}

/// Adds Violet City's missing timber roof row and joins the east plateau face.
///
/// Only three exterior cells and two appended native roof blocks change. The
/// existing private tileset identity, native south stairs, indexed doors, actors,
/// story programs and other maps remain intact. This returns a verified pack;
/// it does not select it, migrate saves or establish gameplay/visual acceptance.
///
/// # Errors
/// Rejects stale complete map/source bindings, shared private art, unexpected
/// edits, insufficient native glyph storage or invalid resulting content.
pub fn rebuild_bemidji_roof_plateau(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
    contract_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let contract: Value = serde_json::from_str(contract_json)?;
    let original = base.data.maps.get(MAP).context("installed Bemidji")?;
    ensure!(
        contract["schema_version"] == 1
            && contract["map"] == MAP
            && contract["tileset"] == TILESET
            && [original.attributes.width, original.attributes.height] == [24, 24]
            && original.attributes.tileset_name == TILESET
            && format!(
                "{:x}",
                Sha256::digest(serde_json::to_vec(&serde_json::to_value(original)?)?)
            ) == contract["native_module_sha256"]
                .as_str()
                .context("complete map hash")?,
        "complete installed Bemidji preimage differs"
    );
    ensure!(
        base.data
            .maps
            .values()
            .filter(|map| map.attributes.tileset_name == TILESET)
            .count()
            == 1
            && !base.report.manifests.iter().any(|marker| marker == MARKER),
        "Bemidji art must be private and revision absent"
    );
    let before: Terrain = serde_json::from_str(before_json)?;
    let after: Terrain = serde_json::from_str(after_json)?;
    ensure!(
        before.blocks()? == original.blocks,
        "complete terrain preimage differs"
    );
    let mut expected = original.blocks.clone();
    for (x, y, old, new) in [(4, 16, 2, 0x85), (5, 16, 1, 0x86), (9, 8, 4, 0x69)] {
        ensure!(
            expected[y * 24 + x] == old,
            "roof or plateau anchor differs"
        );
        expected[y * 24 + x] = new;
    }
    ensure!(
        after.blocks()? == expected,
        "only two roof cells and one cliff join may change"
    );
    let native = base
        .data
        .tilesets
        .get("johto")
        .context("native timber roof")?;
    let recipes = [("85", "2c"), ("86", "2d")].map(|(id, source_block)| {
        serde_json::json!({"id":id,"source_tileset":"johto","source_block":source_block,
            "collision":native.collision.get(source_block)})
    });
    ensure!(
        base.runtime_files
            .get(&format!("data/tilesets/{TILESET}_metatiles.bin"))
            .is_some_and(|bytes| bytes.len() == 133 * 16),
        "current complete Bemidji art required"
    );
    let art = serde_json::json!({"tileset":TILESET,"source_pixels_modified":false,"derived_blocks":recipes});
    let mut result = crate::authored_city_expansion_art::install(
        base,
        &serde_json::to_string(&art)?,
        TEMPORARY_ART,
        TILESET,
        "MnBemidjiRoofPlateau",
    )?;
    // Reuse the already supported private alias. Only Bemidji names it; the
    // shared Johto sheets and existing seasonal/stair behavior stay unchanged.
    let definition = result
        .data
        .tilesets
        .remove(TEMPORARY_ART)
        .context("imported roof art")?;
    result.data.tilesets.insert(TILESET.into(), definition);
    for suffix in ["metatiles.bin", "palette_map.json"] {
        let imported = result
            .runtime_files
            .remove(&format!("data/tilesets/{TEMPORARY_ART}_{suffix}"))
            .context("imported roof runtime table")?;
        result
            .runtime_files
            .insert(format!("data/tilesets/{TILESET}_{suffix}"), imported);
    }
    for (prefix, suffix) in [
        ("data/tilesets", "json"),
        ("gfx/tilesets", "2bpp"),
        ("gfx/tilesets", "png"),
    ] {
        let imported = result
            .runtime_files
            .remove(&format!("{prefix}/{TEMPORARY_ART}.{suffix}"))
            .context("imported roof runtime art")?;
        result
            .runtime_files
            .insert(format!("{prefix}/{TILESET}.{suffix}"), imported);
    }
    result
        .data
        .field_moves
        .cut
        .replacements
        .remove(TEMPORARY_ART);
    result
        .data
        .field_moves
        .whirlpool
        .replacements
        .remove(TEMPORARY_ART);
    let mut town = original.clone();
    town.blocks = expected;
    crate::authored_town_publication::replace(&mut result, town)?;
    result.report = base.report.clone();
    result.report.manifests.push(MARKER.into());
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    // Complete residual comparison: only the declared town/block payload and
    // the five files/definition of its existing private tileset may differ.
    let mut residual = result.clone();
    residual.data.maps.insert(MAP.into(), original.clone());
    let label = original
        .attributes
        .blocks_label
        .as_ref()
        .context("Bemidji blocks label")?;
    residual.data.map_blocks.insert(
        label.clone(),
        base.data
            .map_blocks
            .get(label)
            .context("original blocks")?
            .clone(),
    );
    residual.data.tilesets.insert(
        TILESET.into(),
        base.data
            .tilesets
            .get(TILESET)
            .context("original private art")?
            .clone(),
    );
    for path in base
        .runtime_files
        .keys()
        .filter(|path| path.contains(TILESET))
    {
        residual.runtime_files.insert(
            path.clone(),
            base.runtime_files
                .get(path)
                .context("original runtime art")?
                .clone(),
        );
    }
    residual.identity = base.identity.clone();
    residual.report = base.report.clone();
    ensure!(
        residual == *base,
        "unrelated content changed during roof/cliff repair"
    );
    Ok(result)
}
