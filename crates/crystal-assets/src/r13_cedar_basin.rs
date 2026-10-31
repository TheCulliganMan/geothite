//! Native Cedar Shelter geometry; its original programs remain authoritative.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::authored_town_geometry;
use crate::{CompiledGamePack, MapModule};

const ROOM: &str = "MnR13Cave1F";

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
                && self.dimensions_metatiles == [10, 8]
                && self.border_block == 46
                && self.palette_group == "nite"
                && self.rows.len() == 8,
            "complete native Cedar Shelter identity required"
        );
        let mut blocks = Vec::new();
        for row in &self.rows {
            let words = row.split_whitespace().collect::<Vec<_>>();
            ensure!(words.len() == 10, "complete Cedar Shelter columns required");
            for word in words {
                let block = u16::from_str_radix(word, 16).context("native cave block")?;
                ensure!(block <= 0x3f, "original native cave art required");
                blocks.push(block);
            }
        }
        Ok(blocks)
    }
}

/// Fits the care stair and opens the sheltered fishing bank in Cedar Shelter.
/// Keeps dimensions, forced night palette, original actors, healing commands,
/// fishing tables, once-only cache, exterior pairing and all other maps intact.
///
/// # Errors
/// Rejects stale complete-map inputs, unsupported art, disconnected walking
/// space, open water boundaries, inaccessible services or an invalid child pack.
pub fn rebuild_r13_cedar_basin(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
    contract_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let before: Terrain = serde_json::from_str(before_json)?;
    let after: Terrain = serde_json::from_str(after_json)?;
    let contract: Value = serde_json::from_str(contract_json)?;
    let original = base.data.maps.get(ROOM).context("native Cedar Shelter")?;
    ensure!(
        contract["schema_version"] == 1
            && contract["map"] == ROOM
            && contract["native_module_sha256"]
                == format!(
                    "{:x}",
                    Sha256::digest(serde_json::to_vec(&serde_json::to_value(original)?)?)
                )
            && contract["preserved_warps"] == serde_json::to_value(&original.events.warps)?
            && original.attributes.width == 10
            && original.attributes.height == 8
            && original.attributes.tileset_name == "cave"
            && original.attributes.border_block == 46
            && original.attributes.time_of_day.as_deref() == Some("nite")
            && before.blocks()? == original.blocks,
        "complete native Cedar Shelter preimage differs"
    );
    let mut room = original.clone();
    room.blocks = after.blocks()?;
    ensure!(
        room.blocks != original.blocks,
        "Cedar Shelter terrain unchanged"
    );
    validate_geometry(base, &room)?;
    let mut pack = base.clone();
    crate::authored_town_publication::replace(&mut pack, room)?;
    pack.report
        .manifests
        .push("minnesota-r13-cedar-basin-v1".into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    crate::validate_compact_superior_geography(&pack)?;
    Ok(pack)
}

fn validate_geometry(pack: &CompiledGamePack, room: &MapModule) -> Result<()> {
    let kind = |p| authored_town_geometry::kind(pack, room, p);
    let seen = authored_town_geometry::reachable(pack, room, [7, 14], false)?;
    for at in [
        [3, 7],
        [5, 5],
        [10, 5],
        [10, 7],
        [10, 8],
        [13, 6],
        [12, 9],
        [12, 11],
        [12, 12],
        [14, 11],
    ] {
        ensure!(seen.contains(&at), "Cedar Shelter approach blocked: {at:?}");
    }
    ensure!(
        room.blocks[3 * 10 + 5] == 0x30
            && room.blocks[5 * 10 + 6] == 0x36
            && kind([11, 7])? == "WALL"
            && kind([13, 10])? == "WALL"
            && kind([4, 7])? == "WATER"
            && kind([7, 15])? == "LADDER",
        "fitted stairs, fishing bank and original mouth required"
    );
    let mut water = 0;
    for y in 0..16 {
        for x in 0..20 {
            let at = [x, y];
            let k = kind(at)?;
            let actor = room.objects.iter().any(|o| [o.x, o.y] == at);
            if k == "FLOOR" && !actor {
                ensure!(seen.contains(&at), "stranded cave floor: {at:?}");
            }
            if k == "WATER" {
                water += 1;
                ensure!(x > 0 && x < 19 && y > 0 && y < 15, "open cave water edge");
            }
            if actor {
                ensure!(k == "FLOOR", "original cave actor buried");
            }
        }
    }
    ensure!(water == 24, "complete contained Cedar basin required");
    Ok(())
}
