//! Fit the existing Inner Basin puzzle into connected native bedrock chambers.
use std::collections::{BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{CompiledGamePack, MapModule};

const MAP: &str = "MnInnerBasin";
const MARKER: &str = "minnesota-inner-basin-bedrock-v1";
const SOURCE_SHA256: &str = "13c1312655b95c6fd5c1bb8103a56602d1c13e9a6f0bd7790d8ccb05df1631e4";

#[derive(Deserialize)]
struct Proposal {
    schema_version: u8,
    parent_identity: Value,
    map: String,
    tileset: String,
    dimensions_metatiles: [u16; 2],
    border_block: u8,
    native_module_sha256: String,
    tileset_sha256: String,
    before_rows: Vec<String>,
    rows: Vec<String>,
}

fn rows(source: &[String]) -> Result<Vec<u16>> {
    ensure!(source.len() == 14, "fourteen complete basin rows required");
    let mut result = Vec::with_capacity(196);
    for row in source {
        let values = row
            .split_whitespace()
            .map(|s| u16::from_str_radix(s, 16))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(
            values.len() == 14 && values.iter().all(|&b| b < 64),
            "fourteen native cave columns required"
        );
        result.extend(values);
    }
    Ok(result)
}

fn kind<'a>(pack: &'a CompiledGamePack, map: &MapModule, [x, y]: [u16; 2]) -> Result<&'a str> {
    let block = map.blocks[usize::from(y / 2) * 14 + usize::from(x / 2)];
    let collision = pack.data.tilesets["cave"]
        .collision
        .get(&format!("{block:02x}"))
        .context("existing native cave collision")?;
    Ok(&collision[usize::from(y % 2) * 2 + usize::from(x % 2)])
}

fn reached(pack: &CompiledGamePack, map: &MapModule, surf: bool) -> Result<BTreeSet<[u16; 2]>> {
    let mut allowed = BTreeSet::new();
    for y in 0..28 {
        for x in 0..28 {
            let token = kind(pack, map, [x, y])?;
            ensure!(
                !matches!(token, "CUT_TREE" | "PIT" | "WHIRLPOOL") && !token.starts_with("HOP"),
                "no incidental field gate, pit or ledge"
            );
            if x == 0 || x == 27 || y == 0 || y == 27 {
                ensure!(
                    token == "WALL",
                    "complete closed native cave border required"
                );
            }
            if (token == "FLOOR" || surf && token == "WATER") && [x, y] != [14, 2] {
                allowed.insert([x, y]);
            }
        }
    }
    ensure!(
        allowed.contains(&[2, 25]),
        "original passive arrival required"
    );
    let mut visited = BTreeSet::from([[2, 25]]);
    let mut pending = VecDeque::from([[2u16, 25u16]]);
    while let Some([x, y]) = pending.pop_front() {
        for at in [
            x.checked_sub(1).map(|x| [x, y]),
            x.checked_add(1).map(|x| [x, y]),
            y.checked_sub(1).map(|y| [x, y]),
            y.checked_add(1).map(|y| [x, y]),
        ]
        .into_iter()
        .flatten()
        {
            if allowed.contains(&at) && visited.insert(at) {
                pending.push_back(at);
            }
        }
    }
    Ok(visited)
}

fn validate(pack: &CompiledGamePack, original: &MapModule, room: &MapModule) -> Result<()> {
    for at in [[1u16, 12], [4, 5], [9, 7], [5, 9], [7, 10], [7, 1]] {
        let index = usize::from(at[1]) * 14 + usize::from(at[0]);
        ensure!(
            room.blocks[index] == original.blocks[index],
            "original anchor metatile changed"
        );
    }
    ensure!(
        kind(pack, room, [3, 25])? == "LADDER",
        "original active stair required"
    );
    let closed = reached(pack, room, false)?;
    let closed_surf = reached(pack, room, true)?;
    for (wall, approach) in [
        ([9, 10], [10, 10]),
        ([18, 14], [17, 14]),
        ([11, 18], [12, 18]),
        ([14, 20], [15, 20]),
    ] {
        ensure!(
            kind(pack, room, wall)? == "WALL" && closed.contains(&approach),
            "original indicator/lever and ordinary approach required"
        );
    }
    ensure!(
        closed.contains(&[14, 8]) && !closed_surf.contains(&[14, 3]),
        "closed sill must protect the encounter even with Surf"
    );
    let mut opened = room.clone();
    ensure!(
        room.script_block_changes.len() == 9,
        "nine native drain changes required"
    );
    for change in &room.script_block_changes {
        let index = usize::from(change.y / 2) * 14 + usize::from(change.x / 2);
        ensure!(
            change.source_script == "MnBasinRestoreTerrain"
                && change.block_id == 2
                && room.blocks[index] == original.blocks[index],
            "native drain preimage and callback must remain exact"
        );
        opened.blocks[index] = change.block_id;
    }
    let drained = reached(pack, &opened, false)?;
    ensure!(
        drained.contains(&[14, 3]),
        "native drain must open the creature approach"
    );
    for y in 0..28 {
        for x in 0..28 {
            let at = [x, y];
            ensure!(
                (kind(pack, room, at)? == "WATER") == (kind(pack, original, at)? == "WATER"),
                "original local pool quadrants must remain exact"
            );
            if kind(pack, &opened, at)? == "FLOOR" && at != [14, 2] {
                ensure!(drained.contains(&at), "drained cave leaves isolated floor");
                if y >= 8 {
                    ensure!(
                        closed.contains(&at),
                        "lower chamber needs an ordinary return"
                    );
                }
            }
        }
    }
    Ok(())
}

/// Replaces the square boulder maze with a connected native bedrock enclosure.
///
/// Keeps the three indicators, lever, nine-block drain, voluntary Strandrak,
/// every original program/actor/warp and all unrelated content exact. Only this
/// room's terrain and solid border change. Returns an unselected candidate;
/// native gameplay/visual review is separate. Old saved overrides are not migrated.
///
/// # Errors
/// Rejects source/parent/module/art drift, unsupported terrain, altered anchors,
/// a bypassable sill, unreachable controls or unrelated residual changes.
pub fn rebuild_inner_basin_bedrock(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        format!("{:x}", Sha256::digest(source.as_bytes())) == SOURCE_SHA256,
        "reviewed Inner Basin source differs"
    );
    let plan: Proposal = serde_json::from_str(source)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let original = base.data.maps.get(MAP).context("current Inner Basin")?;
    ensure!(
        plan.schema_version == 1
            && plan.map == MAP
            && plan.tileset == "cave"
            && plan.dimensions_metatiles == [14, 14]
            && plan.border_block == 0x2e
            && serde_json::to_value(&base.identity)? == plan.parent_identity
            && original.attributes.tileset_name == "cave"
            && [original.attributes.width, original.attributes.height] == [14, 14]
            && original.attributes.palette.as_deref() == Some("PALETTE_NITE")
            && original.attributes.border_block == 0x1d
            && original.blocks == rows(&plan.before_rows)?
            && format!(
                "{:x}",
                Sha256::digest(serde_json::to_vec(&serde_json::to_value(original)?)?)
            ) == plan.native_module_sha256
            && format!(
                "{:x}",
                Sha256::digest(serde_json::to_vec(&serde_json::to_value(
                    &base.data.tilesets["cave"]
                )?)?)
            ) == plan.tileset_sha256
            && !base.report.manifests.iter().any(|m| m == MARKER),
        "complete current Inner Basin preimage required"
    );
    let mut room = original.clone();
    room.blocks = rows(&plan.rows)?;
    room.attributes.border_block = plan.border_block;
    validate(base, original, &room)?;
    let label = original
        .attributes
        .blocks_label
        .as_ref()
        .context("original block mirror")?;
    let bytes = room
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut child = base.clone();
    child
        .data
        .map_attributes
        .insert(MAP.into(), room.attributes.clone());
    child.data.maps.insert(MAP.into(), room);
    child
        .data
        .map_blocks
        .insert(label.clone(), crate::encode_base64_bytes(&bytes));
    child.report.manifests.push(MARKER.into());
    child.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        child.format_version,
        &child.data,
        &child.audio_manifest,
        &child.runtime_files,
        &child.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&child)?;
    let mut residual = child.clone();
    residual.data.maps.insert(MAP.into(), original.clone());
    residual
        .data
        .map_attributes
        .insert(MAP.into(), base.data.map_attributes[MAP].clone());
    residual
        .data
        .map_blocks
        .insert(label.clone(), base.data.map_blocks[label].clone());
    residual.report = base.report.clone();
    residual.identity = base.identity.clone();
    ensure!(
        residual == *base,
        "unrelated content changed during cave terrain repair"
    );
    Ok(child)
}
