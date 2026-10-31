//! Native rock maintenance gallery retaining both feed-operated openings.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::authored_town_geometry;
use crate::{CompiledGamePack, MapModule};

const ROOM: &str = "MnIronPumpGallery";
const TILESET: &str = "mn_iron_haulage_rock";
const GATES: [usize; 2] = [3 * 16 + 12, 8 * 16 + 7];

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
        ensure!(self.rows.len() == 12, "complete pump gallery rows required");
        let mut blocks = Vec::new();
        for row in &self.rows {
            let words = row.split_whitespace().collect::<Vec<_>>();
            ensure!(words.len() == 16, "complete pump gallery columns required");
            for word in words {
                blocks.push(u16::from_str_radix(word, 16).context("native pump gallery metatile")?);
            }
        }
        Ok(blocks)
    }
}

/// Replaces the whole pump gallery with a winding rock maintenance route,
/// retaining its original guards, feed console, both shaft pairs and programs.
/// The original feed command opens numeric13 at both the drill doorway and
/// service crosscut. The already installed native mine alias preserves that
/// floor meaning, so no native graphics or tileset is changed by this stage.
///
/// Geometry includes actual guard bodies. Before feed isolation, each guard's
/// sight controls the approach and the drill shaft stays sealed. Isolation
/// opens a useful shorter return. No field move is required or introduced.
/// Visual acceptance and earned campaign access require separate review.
///
/// # Errors
/// Rejects changed full-module or source preimages, changed actors/ports,
/// incompatible art, invalid boundaries, bypassed gates or an invalid pack.
pub fn rebuild_iron_pump_gallery(
    base: &CompiledGamePack,
    before: &str,
    after: &str,
    contract: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let old: Terrain = serde_json::from_str(before)?;
    let new: Terrain = serde_json::from_str(after)?;
    let contract: Value = serde_json::from_str(contract)?;
    let original = base.data.maps.get(ROOM).context("installed pump gallery")?;
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
            && old.dimensions_metatiles == [16, 12]
            && new.dimensions_metatiles == [16, 12]
            && old.tileset == "underground"
            && old.border_block == 0
            && old.palette_group == "day"
            && new.tileset == TILESET
            && new.border_block == 46
            && new.palette_group == "nite"
            && original.attributes.tileset_name == old.tileset
            && original.attributes.width == 16
            && original.attributes.height == 12
            && original.blocks == old.blocks()?
            && original.events.warps.len() == 4,
        "complete pump gallery preimage/source differs"
    );
    ensure!(
        original
            .objects
            .iter()
            .map(|o| (o.script.as_str(), o.x, o.y))
            .eq([("MN_LODE_ARC_PUMP", 12, 5), ("MN_LODE_ARC_INTAKE", 25, 13)]),
        "original pump and intake guard anchors required"
    );
    let label = original
        .attributes
        .blocks_label
        .as_ref()
        .context("pump gallery blocks label")?;
    ensure!(
        base.data
            .maps
            .iter()
            .all(|(id, m)| id == ROOM || m.attributes.blocks_label.as_ref() != Some(label)),
        "shared pump gallery blocks"
    );
    ensure!(
        crate::decode_base64_bytes(
            base.data
                .map_blocks
                .get(label)
                .context("split pump gallery blocks")?
        )?
        .into_iter()
        .map(u16::from)
        .eq(original.blocks.iter().copied()),
        "split pump gallery preimage differs"
    );
    let art = base
        .data
        .tilesets
        .get(TILESET)
        .context("reviewed native mine art")?;
    ensure!(
        art.collision["0d"].iter().all(|k| k == "FLOOR")
            && art.collision["42"]
                .iter()
                .map(String::as_str)
                .eq(["FLOOR", "LADDER", "FLOOR", "FLOOR"])
            && art.collision["41"].iter().all(|k| k == "WALL"),
        "reviewed native mine callback/shaft/grille art required"
    );
    let mut room = original.clone();
    room.blocks = new.blocks()?;
    room.attributes.tileset_name = TILESET.into();
    room.attributes.border_block = 46;
    room.attributes.time_of_day = Some("nite".into());
    room.attributes.palette = Some("PALETTE_NITE".into());
    room.attributes.fishing_group = Some("FISHGROUP_MN_LAKE".into());
    ensure!(
        GATES.iter().all(|i| room.blocks[*i] == 0x41)
            && room.blocks[8 * 16 + 12] == 0x40
            && room.blocks[4 * 16 + 3] == 0x30
            && room.blocks[4 * 16 + 8] == 0x36,
        "both feed shutters, console and native climbs required"
    );
    ensure!(
        !base
            .report
            .manifests
            .iter()
            .any(|id| id == "minnesota-iron-pump-gallery-rock-v1"),
        "pump gallery is already rebuilt"
    );
    let long = validate_geometry(base, &room, false)?;
    let short = validate_geometry(base, &room, true)?;
    ensure!(long >= short + 12, "feed must open a useful shorter return");
    let mut pack = base.clone();
    crate::authored_town_publication::replace(&mut pack, room)?;
    pack.report
        .manifests
        .push("minnesota-iron-pump-gallery-rock-v1".into());
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

fn distances(seed: [u16; 2], nodes: &BTreeSet<[u16; 2]>) -> BTreeMap<[u16; 2], usize> {
    let mut result = BTreeMap::from([(seed, 0)]);
    let mut queue = VecDeque::from([seed]);
    while let Some([x, y]) = queue.pop_front() {
        for next in [
            x.checked_sub(1).map(|x| [x, y]),
            x.checked_add(1).map(|x| [x, y]),
            y.checked_sub(1).map(|y| [x, y]),
            y.checked_add(1).map(|y| [x, y]),
        ]
        .into_iter()
        .flatten()
        {
            if nodes.contains(&next) && !result.contains_key(&next) {
                result.insert(next, result[&[x, y]] + 1);
                queue.push_back(next);
            }
        }
    }
    result
}

fn validate_geometry(pack: &CompiledGamePack, original: &MapModule, opened: bool) -> Result<usize> {
    let mut room = original.clone();
    if opened {
        for i in GATES {
            room.blocks[i] = 13;
        }
    }
    let kind = |p| authored_town_geometry::kind(pack, &room, p);
    let bodies = room
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .collect::<BTreeSet<_>>();
    let mut floor = BTreeSet::new();
    let mut ladders = BTreeSet::new();
    for y in 0..24 {
        for x in 0..32 {
            let at = [x, y];
            let k = kind(at)?;
            ensure!(
                matches!(k, "FLOOR" | "WALL" | "UP_WALL" | "LADDER"),
                "unsupported pump gallery collision {k}"
            );
            if x == 0 || y == 0 || x == 31 || y == 23 {
                ensure!(k == "WALL", "unconnected pump gallery boundary");
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
        ladders == BTreeSet::from([[5, 18], [25, 4]]),
        "two original pump gallery shafts required"
    );
    for warp in &room.events.warps {
        ensure!(
            kind([warp.x, warp.y])?
                == if warp.index % 2 == 1 {
                    "LADDER"
                } else {
                    "FLOOR"
                },
            "pump gallery active/passive collision differs"
        );
        if warp.index % 2 == 1 {
            ensure!(
                kind([warp.x, warp.y + 1])? == "FLOOR",
                "shaft approach blocked"
            );
        }
    }
    for at in bodies {
        ensure!(kind(at)? == "FLOOR", "pump guard stands in rock");
    }
    let main = distances([4, 19], &floor);
    let core = distances([24, 5], &floor);
    ensure!(
        if opened {
            main.keys().eq(core.keys())
        } else {
            main.keys().all(|p| !core.contains_key(p))
        },
        "feed-operated core partition differs"
    );
    ensure!(
        main.keys()
            .chain(core.keys())
            .copied()
            .collect::<BTreeSet<_>>()
            == floor,
        "orphan pump gallery floor"
    );
    ensure!(
        [
            [25, 18],
            [13, 17],
            [16, 17],
            [24, 8],
            [6, 10],
            [6, 7],
            [16, 10],
            [16, 7]
        ]
        .iter()
        .all(|at| main.contains_key(at)),
        "pump gallery story or climb approach missing"
    );
    if !opened {
        for sight in [
            BTreeSet::from([[12, 6], [12, 7], [12, 8]]),
            BTreeSet::from([[26, 13], [27, 13], [28, 13]]),
        ] {
            ensure!(
                sight.iter().all(|p| floor.contains(p)),
                "preserved guard sight must cross ordinary floor"
            );
            let safe = floor.difference(&sight).copied().collect();
            ensure!(
                !distances([4, 19], &safe).contains_key(&[25, 18]),
                "unintended free bypass around an original pump guard"
            );
        }
    }
    main.get(&[25, 18])
        .copied()
        .context("reachable feed console")
}
