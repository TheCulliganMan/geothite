//! Recompose the complete haulage floor while preserving its mining campaign.
use std::collections::{BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::authored_town_geometry;
use crate::iron_haulage_rock_art;
use crate::{CompiledGamePack, MapModule};

const ROOM: &str = "MnIronHaulage";

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
        ensure!(self.rows.len() == 14, "complete haulage rows required");
        let mut blocks = Vec::new();
        for row in &self.rows {
            let words = row.split_whitespace().collect::<Vec<_>>();
            ensure!(words.len() == 18, "complete haulage columns required");
            for word in words {
                blocks.push(u16::from_str_radix(word, 16).context("native mine metatile")?);
            }
        }
        Ok(blocks)
    }
}

/// Rebuilds the haulage floor as two rock galleries with native climbs, a
/// pressure-operated shutter and all five original active/passive ladder pairs.
/// The original actors, story programs, background controls, guards, identities,
/// dimensions and all other maps remain exact. A private native-art alias
/// preserves block13's meaning for the existing pressure callbacks. The dry
/// room's unused fishing reference is corrected to the regional lake group.
///
/// Geometry verifies closed and open pressure states with actual actor bodies.
/// This does not establish visual quality, earned access or campaign completion;
/// production input and native captures are separate acceptance requirements.
///
/// # Errors
/// Rejects a stale full-module/source binding, changed ports or actors, unsafe
/// closed/open geometry, unsupported art recipes and invalid resulting packs.
pub fn rebuild_iron_haulage_rock(
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
    let original = base
        .data
        .maps
        .get(ROOM)
        .context("installed haulage floor")?;
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
            && old.dimensions_metatiles == [18, 14]
            && new.dimensions_metatiles == [18, 14]
            && old.tileset == "underground"
            && old.palette_group == "day"
            && old.border_block == 0
            && new.tileset == iron_haulage_rock_art::ALIAS
            && new.palette_group == "nite"
            && new.border_block == 0x2e
            && original.attributes.tileset_name == old.tileset
            && original.attributes.width == 18
            && original.attributes.height == 14
            && original.blocks == old.blocks()?
            && original.events.warps.len() == 10,
        "complete haulage preimage/source differs"
    );
    ensure!(
        original
            .objects
            .iter()
            .map(|o| (o.script.as_str(), o.x, o.y))
            .eq([
                ("MnIronHaulageRuth", 5, 15),
                ("MN_LODE_ARC_WEIGH", 14, 7),
                ("MN_LODE_ARC_CONVEYOR", 30, 13)
            ]),
        "original Ruth and two LODE guard anchors required"
    );
    let label = original
        .attributes
        .blocks_label
        .as_ref()
        .context("haulage block label")?;
    ensure!(
        base.data
            .maps
            .iter()
            .all(|(id, m)| id == ROOM || m.attributes.blocks_label.as_ref() != Some(label)),
        "shared haulage block payload"
    );
    ensure!(
        crate::decode_base64_bytes(&base.data.map_blocks[label])?
            .into_iter()
            .map(u16::from)
            .eq(original.blocks.iter().copied()),
        "split haulage preimage differs"
    );
    let mut pack = iron_haulage_rock_art::install(base, &design)?;
    let mut room = original.clone();
    room.blocks = new.blocks()?;
    room.attributes.tileset_name = iron_haulage_rock_art::ALIAS.into();
    room.attributes.border_block = 0x2e;
    room.attributes.time_of_day = Some("nite".into());
    room.attributes.palette = Some("PALETTE_NITE".into());
    room.attributes.fishing_group = Some("FISHGROUP_MN_LAKE".into());
    ensure!(
        room.blocks[8 * 18 + 9] == 0x41 && room.blocks[5 * 18 + 14] == 0x40,
        "native shutter and retained valve control required"
    );
    validate_geometry(&pack, &room, false)?;
    validate_geometry(&pack, &room, true)?;
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

fn validate_geometry(pack: &CompiledGamePack, original: &MapModule, opened: bool) -> Result<()> {
    let mut room = original.clone();
    if opened {
        room.blocks[8 * 18 + 9] = 13;
    }
    let kind = |p| authored_town_geometry::kind(pack, &room, p);
    let bodies = room
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .collect::<BTreeSet<_>>();
    let mut floor = BTreeSet::new();
    let mut ladders = BTreeSet::new();
    for y in 0..28 {
        for x in 0..36 {
            let at = [x, y];
            let k = kind(at)?;
            ensure!(
                matches!(k, "WALL" | "FLOOR" | "UP_WALL" | "LADDER"),
                "unsupported mine collision {k}"
            );
            if x == 0 || y == 0 || x == 35 || y == 27 {
                ensure!(k == "WALL", "open mine boundary");
            }
            if k == "FLOOR" && !bodies.contains(&at) {
                floor.insert(at);
            }
            if k == "LADDER" {
                ladders.insert(at);
            }
        }
    }
    ensure!(
        ladders == BTreeSet::from([[5, 22], [7, 4], [27, 4], [29, 22], [13, 12]]),
        "all five original active stairs required"
    );
    for warp in &room.events.warps {
        ensure!(
            kind([warp.x, warp.y])?
                == if warp.index % 2 == 1 {
                    "LADDER"
                } else {
                    "FLOOR"
                },
            "active/passive ladder collision differs"
        );
    }
    for at in bodies {
        ensure!(kind(at)? == "FLOOR", "actor stands in rock");
    }
    let flood = |start: [u16; 2]| {
        let mut seen = BTreeSet::from([start]);
        let mut queue = VecDeque::from([start]);
        while let Some([x, y]) = queue.pop_front() {
            for p in [[x - 1, y], [x + 1, y], [x, y - 1], [x, y + 1]] {
                if floor.contains(&p) && seen.insert(p) {
                    queue.push_back(p);
                }
            }
        }
        seen
    };
    ensure!(
        floor.contains(&[4, 23]) && floor.contains(&[28, 23]) && floor.contains(&[29, 12]),
        "native arrivals and valve approach required"
    );
    let west = flood([4, 23]);
    let east = flood([28, 23]);
    if opened {
        ensure!(
            west == floor && east == floor,
            "opened shutter must join every ordinary floor"
        );
    } else {
        ensure!(
            west.is_disjoint(&east) && west.union(&east).copied().collect::<BTreeSet<_>>() == floor,
            "closed shutter must separate exactly the two working compartments"
        );
        ensure!(
            west.contains(&[6, 5])
                && west.contains(&[12, 13])
                && east.contains(&[26, 5])
                && east.contains(&[29, 12]),
            "retained ladder/valve sides differ"
        );
    }
    Ok(())
}
