//! Rebuild the sump around a flooded western drift and its retained pump gate.
use std::collections::{BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{CompiledGamePack, MapModule};
use crate::{authored_town_geometry, iron_sump_shore_art};

const ROOM: &str = "MnIronSump";

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
        ensure!(self.rows.len() == 12, "complete sump rows required");
        let mut result = Vec::new();
        for row in &self.rows {
            let words = row.split_whitespace().collect::<Vec<_>>();
            ensure!(words.len() == 20, "complete sump columns required");
            for word in words {
                result.push(u16::from_str_radix(word, 16).context("native sump metatile")?);
            }
        }
        Ok(result)
    }
}

/// Replaces the complete sump terrain while retaining all original guards,
/// native identities, shaft pairs, two-circuit gate, programs and other maps.
/// The western basin offers optional Surf/fishing; dry progression remains
/// available. A private alias adds two native water/rock shore blocks without
/// changing existing graphics or collision. The fishing reference becomes the
/// active regional lake group.
///
/// Closed and open geometry includes actual actor bodies. Surf must remain
/// inside the western drift; it cannot bypass the pump gate or map boundary.
/// This verifies geometry, not earned access, visual quality or story completion.
///
/// # Errors
/// Rejects stale complete modules/sources, changed native ports or bodies,
/// unsafe dry/Surf geometry, unsupported art or an invalid resulting pack.
pub fn rebuild_iron_sump_basin(
    base: &CompiledGamePack,
    before: &str,
    after: &str,
    contract: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let old: Terrain = serde_json::from_str(before)?;
    let new: Terrain = serde_json::from_str(after)?;
    let design: Value = serde_json::from_str(after)?;
    let contract: Value = serde_json::from_str(contract)?;
    let original = base.data.maps.get(ROOM).context("installed sump")?;
    ensure!(
        contract["schema_version"] == 1
            && contract["map"] == ROOM
            && contract["native_module_sha256"]
                == format!(
                    "{:x}",
                    Sha256::digest(serde_json::to_vec(&serde_json::to_value(original)?)?)
                )
            && contract["preserved_warps"] == serde_json::to_value(&original.events.warps)?
            && old.schema_version == 1
            && new.schema_version == 1
            && old.id == ROOM
            && new.id == ROOM
            && old.dimensions_metatiles == [20, 12]
            && new.dimensions_metatiles == [20, 12]
            && old.tileset == "underground"
            && old.border_block == 0
            && old.palette_group == "day"
            && new.tileset == iron_sump_shore_art::ALIAS
            && new.border_block == 0x2e
            && new.palette_group == "nite"
            && original.attributes.tileset_name == old.tileset
            && original.attributes.width == 20
            && original.attributes.height == 12
            && original.blocks == old.blocks()?
            && original.events.warps.len() == 10,
        "complete sump preimage/source differs"
    );
    ensure!(
        original
            .objects
            .iter()
            .map(|o| (o.script.as_str(), o.x, o.y))
            .eq([("MN_LODE_ARC_SUMP", 14, 10), ("MN_LODE_ARC_DRAIN", 25, 17)]),
        "original sump guard anchors required"
    );
    let label = original
        .attributes
        .blocks_label
        .as_ref()
        .context("sump block label")?;
    ensure!(
        base.data
            .maps
            .iter()
            .all(|(id, m)| id == ROOM || m.attributes.blocks_label.as_ref() != Some(label)),
        "shared sump blocks"
    );
    ensure!(
        crate::decode_base64_bytes(
            base.data
                .map_blocks
                .get(label)
                .context("split sump blocks")?
        )?
        .into_iter()
        .map(u16::from)
        .eq(original.blocks.iter().copied()),
        "split sump preimage differs"
    );
    let mut pack = iron_sump_shore_art::install(base, &design)?;
    let mut room = original.clone();
    room.blocks = new.blocks()?;
    room.attributes.tileset_name = iron_sump_shore_art::ALIAS.into();
    room.attributes.border_block = 0x2e;
    room.attributes.time_of_day = Some("nite".into());
    room.attributes.palette = Some("PALETTE_NITE".into());
    room.attributes.fishing_group = Some("FISHGROUP_MN_LAKE".into());
    ensure!(
        room.blocks[3 * 20 + 16] == 0x41,
        "original two-circuit gate site required"
    );
    for opened in [false, true] {
        validate_geometry(&pack, &room, opened)?;
    }
    crate::authored_town_publication::replace(&mut pack, room)?;
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

fn flood(seed: [u16; 2], nodes: &BTreeSet<[u16; 2]>) -> BTreeSet<[u16; 2]> {
    let mut seen = BTreeSet::from([seed]);
    let mut queue = VecDeque::from([seed]);
    while let Some([x, y]) = queue.pop_front() {
        for p in [
            x.checked_sub(1).map(|x| [x, y]),
            y.checked_sub(1).map(|y| [x, y]),
            x.checked_add(1).map(|x| [x, y]),
            y.checked_add(1).map(|y| [x, y]),
        ]
        .into_iter()
        .flatten()
        {
            if nodes.contains(&p) && seen.insert(p) {
                queue.push_back(p);
            }
        }
    }
    seen
}

fn validate_geometry(pack: &CompiledGamePack, original: &MapModule, opened: bool) -> Result<()> {
    let mut room = original.clone();
    if opened {
        room.blocks[3 * 20 + 16] = 13;
    }
    let kind = |p| authored_town_geometry::kind(pack, &room, p);
    let bodies = room
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .collect::<BTreeSet<_>>();
    let mut floor = BTreeSet::new();
    let mut water = BTreeSet::new();
    let mut ladders = BTreeSet::new();
    for y in 0..24 {
        for x in 0..40 {
            let at = [x, y];
            let k = kind(at)?;
            ensure!(
                matches!(k, "WALL" | "UP_WALL" | "FLOOR" | "WATER" | "LADDER"),
                "unsupported sump collision {k}"
            );
            if x == 0 || y == 0 || x == 39 || y == 23 {
                ensure!(k == "WALL", "unconnected sump boundary");
            }
            if k == "FLOOR" && !bodies.contains(&at) {
                floor.insert(at);
            }
            if k == "WATER" {
                water.insert(at);
            }
            if k == "LADDER" {
                ladders.insert(at);
            }
        }
    }
    ensure!(
        ladders == BTreeSet::from([[5, 18], [9, 4], [31, 18], [35, 4], [25, 10]]),
        "five original sump shafts required"
    );
    for warp in &room.events.warps {
        ensure!(
            kind([warp.x, warp.y])?
                == if warp.index % 2 == 1 {
                    "LADDER"
                } else {
                    "FLOOR"
                },
            "sump active/passive collision differs"
        );
    }
    for at in bodies {
        ensure!(kind(at)? == "FLOOR", "sump guard stands in rock");
    }
    for at in [[4, 19], [30, 19], [34, 5], [12, 12], [32, 8]] {
        ensure!(
            floor.contains(&at),
            "sump arrival/bank/gate approach missing"
        );
    }
    let west = flood([4, 19], &floor);
    let east = flood([30, 19], &floor);
    let pocket = flood([34, 5], &floor);
    ensure!(
        west.is_disjoint(&east) && west.is_disjoint(&pocket),
        "western workings cannot bypass shaft routing"
    );
    ensure!(
        if opened {
            east == pocket
        } else {
            east.is_disjoint(&pocket)
        },
        "two-circuit pump gate geometry differs"
    );
    ensure!(
        west.union(&east)
            .copied()
            .chain(pocket.iter().copied())
            .collect::<BTreeSet<_>>()
            == floor,
        "orphan sump floor"
    );
    ensure!(
        water.len() == 38 && west.contains(&[12, 12]) && kind([11, 12])? == "WATER",
        "fitted basin and real fishing bank required"
    );
    let surf_nodes = floor.union(&water).copied().collect();
    let surf = flood([12, 12], &surf_nodes);
    ensure!(
        surf == west.union(&water).copied().collect(),
        "Surf escapes the western drift or bypasses the pump gate"
    );
    Ok(())
}
