//! Recompose the original ventilation level around its pressure crosscut.
use std::collections::{BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{CompiledGamePack, MapModule};
use crate::{authored_town_geometry, iron_ventilation_art};

const ROOM: &str = "MnIronVentilation";

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
        ensure!(self.rows.len() == 13, "complete ventilation rows required");
        let mut blocks = Vec::new();
        for row in &self.rows {
            let words = row.split_whitespace().collect::<Vec<_>>();
            ensure!(words.len() == 18, "complete ventilation columns required");
            for word in words {
                blocks.push(u16::from_str_radix(word, 16).context("native ventilation metatile")?);
            }
        }
        Ok(blocks)
    }
}

/// Rebuilds the complete ventilation level as native rock shelves, a western
/// fin loop and an eastern service cleft. Preserves every original actor,
/// shaft pair, control, program and unrelated map. The private alias retains
/// block7 as floor for the original pressure callback, without editing any
/// original native graphics or tileset.
///
/// Closed/open geometry includes actual bodies, legal shaft approaches and
/// solid outer boundaries. This does not establish visual acceptance, earned
/// regional access or campaign completion; native input review is separate.
///
/// # Errors
/// Rejects changed complete-module or terrain preimages, changed ports/actors,
/// unsupported art, unsafe compartments and an invalid resulting pack.
pub fn rebuild_iron_ventilation_cleft(
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
        .context("installed ventilation floor")?;
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
            && old.dimensions_metatiles == [18, 13]
            && new.dimensions_metatiles == [18, 13]
            && old.tileset == "facility"
            && old.border_block == 23
            && old.palette_group == "day"
            && new.tileset == iron_ventilation_art::ALIAS
            && new.border_block == 46
            && new.palette_group == "nite"
            && original.attributes.tileset_name == old.tileset
            && original.attributes.width == 18
            && original.attributes.height == 13
            && original.blocks == old.blocks()?
            && original.events.warps.len() == 8,
        "complete ventilation preimage/source differs"
    );
    ensure!(
        original
            .objects
            .iter()
            .map(|o| (o.script.as_str(), o.x, o.y))
            .eq([
                ("MnIronVentilationVern", 25, 15),
                ("MN_LODE_ARC_VENT", 10, 10),
                ("MN_LODE_ARC_SHIFT", 29, 8)
            ]),
        "original Vern and guard anchors required"
    );
    let label = original
        .attributes
        .blocks_label
        .as_ref()
        .context("ventilation blocks label")?;
    ensure!(
        base.data
            .maps
            .iter()
            .all(|(id, m)| id == ROOM || m.attributes.blocks_label.as_ref() != Some(label)),
        "shared ventilation blocks"
    );
    ensure!(
        crate::decode_base64_bytes(
            base.data
                .map_blocks
                .get(label)
                .context("split ventilation blocks")?
        )?
        .into_iter()
        .map(u16::from)
        .eq(original.blocks.iter().copied()),
        "split ventilation preimage differs"
    );
    let mut pack = iron_ventilation_art::install(base, &design)?;
    let mut room = original.clone();
    room.blocks = new.blocks()?;
    room.attributes.tileset_name = iron_ventilation_art::ALIAS.into();
    room.attributes.border_block = 46;
    room.attributes.time_of_day = Some("nite".into());
    room.attributes.palette = Some("PALETTE_NITE".into());
    room.attributes.fishing_group = Some("FISHGROUP_MN_LAKE".into());
    ensure!(
        room.blocks[6 * 18 + 9] == 0x41 && room.blocks[7 * 18 + 6] == 0x40,
        "retained ventilation grille/control required"
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

fn flood(seed: [u16; 2], nodes: &BTreeSet<[u16; 2]>) -> BTreeSet<[u16; 2]> {
    let mut seen = BTreeSet::from([seed]);
    let mut queue = VecDeque::from([seed]);
    while let Some([x, y]) = queue.pop_front() {
        for p in [
            x.checked_sub(1).map(|x| [x, y]),
            x.checked_add(1).map(|x| [x, y]),
            y.checked_sub(1).map(|y| [x, y]),
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
        room.blocks[6 * 18 + 9] = 7;
    }
    let kind = |p| authored_town_geometry::kind(pack, &room, p);
    let bodies = room
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .collect::<BTreeSet<_>>();
    let mut floor = BTreeSet::new();
    let mut ladders = BTreeSet::new();
    for y in 0..26 {
        for x in 0..36 {
            let at = [x, y];
            let k = kind(at)?;
            ensure!(
                matches!(k, "FLOOR" | "WALL" | "UP_WALL" | "LADDER"),
                "unsupported ventilation collision {k}"
            );
            if x == 0 || y == 0 || x == 35 || y == 25 {
                ensure!(k == "WALL", "unconnected ventilation boundary");
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
        ladders == BTreeSet::from([[5, 20], [7, 4], [27, 20], [29, 4]]),
        "four original ventilation shafts required"
    );
    for warp in &room.events.warps {
        ensure!(
            kind([warp.x, warp.y])?
                == if warp.index % 2 == 1 {
                    "LADDER"
                } else {
                    "FLOOR"
                },
            "ventilation active/passive collision differs"
        );
        if warp.index % 2 == 1 {
            ensure!(
                kind([warp.x, warp.y + 1])? == "FLOOR",
                "shaft approach blocked"
            );
        }
    }
    for at in bodies {
        ensure!(kind(at)? == "FLOOR", "ventilation actor stands in rock");
    }
    let west = flood([4, 21], &floor);
    let east = flood([26, 21], &floor);
    ensure!(
        if opened {
            west == east
        } else {
            west.is_disjoint(&east)
        },
        "ventilation pressure partition differs"
    );
    ensure!(
        west.union(&east).copied().collect::<BTreeSet<_>>() == floor,
        "orphan ventilation floor"
    );
    ensure!(
        [[6, 5], [13, 16], [11, 10], [8, 10], [8, 7]]
            .iter()
            .all(|p| west.contains(p))
            && [[28, 5], [24, 15], [28, 8]]
                .iter()
                .all(|p| east.contains(p)),
        "original shelf/valve/guide approaches missing"
    );
    Ok(())
}
