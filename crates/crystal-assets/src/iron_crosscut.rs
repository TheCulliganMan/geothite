//! Fitted ore passages and a real one-way return in the installed mining approach.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;

#[path = "iron_crosscut_geometry.rs"]
mod geometry;

#[cfg(test)]
#[path = "iron_crosscut_tests.rs"]
mod tests;

const ROOM: &str = "MnIronDrillApproach";
const MARKER: &str = "minnesota-iron-crosscut-bends-v1";

#[derive(Deserialize)]
struct Terrain {
    schema_version: u8,
    id: String,
    tileset: String,
    dimensions_metatiles: [u16; 2],
    border_block: u16,
    palette_group: String,
    rows: Vec<String>,
}

impl Terrain {
    fn blocks(&self) -> Result<Vec<u16>> {
        ensure!(
            self.schema_version == 1
                && self.id == ROOM
                && self.tileset == "cave"
                && self.dimensions_metatiles == [16, 13]
                && self.border_block == 46
                && self.palette_group == "nite"
                && self.rows.len() == 13,
            "complete native crosscut source required"
        );
        let mut blocks = Vec::new();
        for row in &self.rows {
            let words = row.split_whitespace().collect::<Vec<_>>();
            ensure!(words.len() == 16, "complete crosscut columns required");
            for word in words {
                let block = u16::from_str_radix(word, 16).context("native cave metatile")?;
                ensure!(block < 64, "original cave art required");
                blocks.push(block);
            }
        }
        Ok(blocks)
    }
}

/// Rebuilds the existing Iron Range approach as a fitted ore crosscut with its
/// original Ether bench, actors, wild floor and paired ladders. A standalone
/// westward ledge provides a shorter return with an ordinary uphill route.
/// All nonterrain content and native assets remain exact. Existing saved map
/// overrides are not migrated.
///
/// # Errors
/// Rejects stale source/module bindings, changed original anchors, unsupported
/// art, stranded floor, incorrect stairs, cliff-adjacent or bypassable ledges,
/// unsafe borders, or an invalid resulting pack.
pub fn rebuild_iron_crosscut(
    base: &CompiledGamePack,
    before: &str,
    after: &str,
    contract: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let before: Terrain = serde_json::from_str(before)?;
    let after: Terrain = serde_json::from_str(after)?;
    let binding: Value = serde_json::from_str(contract)?;
    let original = base.data.maps.get(ROOM).context("installed crosscut")?;
    ensure!(
        binding["schema_version"] == 1
            && binding["map"] == ROOM
            && binding["native_module_sha256"]
                == format!(
                    "{:x}",
                    Sha256::digest(serde_json::to_vec(&serde_json::to_value(original)?)?)
                )
            && binding["preserved_warps"] == serde_json::to_value(&original.events.warps)?
            && original.attributes.width == 16
            && original.attributes.height == 13
            && original.attributes.tileset_name == "cave"
            && original.attributes.time_of_day.as_deref() == Some("nite")
            && original.attributes.border_block == 46
            && original.blocks == before.blocks()?
            && original.events.warps.len() == 4
            && original.events.coord_events.is_empty()
            && original.events.bg_events.is_empty()
            && original
                .objects
                .iter()
                .map(|o| (o.script.as_str(), o.x, o.y))
                .eq([
                    ("MN_LODE_ARC_ACCESS", 10, 12),
                    ("MN_LODE_ARC_SPOIL", 26, 18),
                    ("MN_IRON_APPROACH_ETHER", 5, 5)
                ])
            && !base.report.manifests.iter().any(|m| m == MARKER),
        "complete current crosscut preimage and original anchors required"
    );
    let mut room = original.clone();
    room.blocks = after.blocks()?;
    geometry::validate(base, &room)?;
    let mut pack = base.clone();
    crate::authored_town_publication::replace(&mut pack, room)?;
    for (name, map) in &base.data.maps {
        if name != ROOM {
            ensure!(
                pack.data.maps.get(name) == Some(map),
                "unrelated map changed: {name}"
            );
        }
    }
    ensure!(
        pack.runtime_files == base.runtime_files
            && pack.data.wild_encounters == base.data.wild_encounters
            && pack.data.trainers == base.data.trainers
            && pack.audio_manifest == base.audio_manifest
            && pack.compiled_audio == base.compiled_audio,
        "native art, wildlife, trainers and audio must remain exact"
    );
    pack.report.manifests.push(MARKER.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}
