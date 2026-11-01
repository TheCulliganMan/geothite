//! Fit Brainerd's existing outfitter, street rail and wooded approaches.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::authored_town_geometry;
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

const MAP: &str = "MnBrainerd";
const PARENT: &str = "b1de4770a0c928a48ded99396a762b09bef835d3d948ce6660a6b0de493f6610";
const SOURCE: &str = "8ad8cd5a1e7347235009e103f183eb948d3218f66263a19b5c5d258125032bee";
const MARKER: &str = "minnesota-brainerd-lakeside-loop-v1";

#[derive(Deserialize)]
struct Proposal {
    schema_version: u8,
    map: String,
    tileset: String,
    dimensions_metatiles: [u16; 2],
    native_module_sha256: String,
    source_pixels_modified: bool,
    before_rows: Vec<String>,
    rows: Vec<String>,
    building_moves: Vec<BuildingMove>,
    warp_moves: Vec<WarpMove>,
    protected_rectangles_metatiles: Vec<ProtectedArea>,
    public_door_returns: BTreeMap<String, [u16; 2]>,
    public_lanes: Vec<Lane>,
    fishing_banks: Vec<Bank>,
    expected_changed_metatiles: usize,
}

#[derive(Deserialize)]
struct BuildingMove {
    source_rect_metatiles: [u16; 4],
    destination_top_left_metatiles: [u16; 2],
}

#[derive(Deserialize)]
struct WarpMove {
    index: u16,
    from_tile: [u16; 2],
    to_tile: [u16; 2],
}

#[derive(Deserialize)]
struct ProtectedArea {
    rectangle: [u16; 4],
}

#[derive(Deserialize)]
struct Lane {
    id: String,
    rectangle_core: [u16; 4],
}

#[derive(Deserialize)]
struct Bank {
    stand: [u16; 2],
    water: [u16; 2],
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn rows(source: &[String]) -> Result<Vec<u16>> {
    ensure!(source.len() == 24, "twenty-four Brainerd rows required");
    let mut blocks = Vec::with_capacity(768);
    for row in source {
        let row = row
            .split_whitespace()
            .map(|word| u16::from_str_radix(word, 16))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(row.len() == 32, "thirty-two Brainerd columns required");
        blocks.extend(row);
    }
    Ok(blocks)
}

fn geometry(
    pack: &CompiledGamePack,
    before: &MapModule,
    town: &MapModule,
    plan: &Proposal,
) -> Result<()> {
    ensure!(
        town.blocks[6 * 32 + 23..6 * 32 + 25] == [0x40, 0x42],
        "Brainerd pier needs its native left and right cap corners"
    );
    let reached = authored_town_geometry::reachable(pack, town, [9, 22], false)?;
    let clear = |at: [u16; 2]| -> Result<bool> {
        Ok(authored_town_geometry::kind(pack, town, at)? == "FLOOR"
            && reached.contains(&at)
            && !town.objects.iter().any(|o| [o.x, o.y] == at))
    };
    for lane in &plan.public_lanes {
        let [x, y, width, height] = lane.rectangle_core;
        for yy in y..y + height {
            for xx in x..x + width {
                ensure!(
                    clear([xx, yy])?,
                    "blocked Brainerd passing lane {}",
                    lane.id
                );
            }
        }
    }
    for point in plan.public_door_returns.values() {
        ensure!(clear(*point)?, "Brainerd public return blocked: {point:?}");
    }
    for spawn in pack
        .data
        .runtime_spawn_points
        .values()
        .filter(|s| s.map_name == MAP)
    {
        let at = [u16::try_from(spawn.tile_x)?, u16::try_from(spawn.tile_y)?];
        ensure!(clear(at)?, "Brainerd Fly/recovery arrival blocked");
    }
    for warp in &town.events.warps {
        let expected = if warp.index % 2 == 1 { "DOOR" } else { "FLOOR" };
        ensure!(
            authored_town_geometry::kind(pack, town, [warp.x, warp.y])? == expected,
            "Brainerd door/return collision differs"
        );
    }
    let neighbor_reached = |[x, y]: [u16; 2]| {
        [
            x.checked_sub(1).map(|x| [x, y]),
            x.checked_add(1).map(|x| [x, y]),
            y.checked_sub(1).map(|y| [x, y]),
            y.checked_add(1).map(|y| [x, y]),
        ]
        .into_iter()
        .flatten()
        .any(|p| reached.contains(&p))
    };
    for actor in &town.objects {
        let at = [actor.x, actor.y];
        ensure!(
            authored_town_geometry::kind(pack, town, at)? == "FLOOR" && neighbor_reached(at),
            "Brainerd actor lost its public approach"
        );
    }
    for sign in &town.events.bg_events {
        let at = [sign.x, sign.y];
        ensure!(
            authored_town_geometry::kind(pack, town, at)? == "WALL" && neighbor_reached(at),
            "Brainerd post lost its solid tile or public approach"
        );
    }
    for y in 0..48 {
        for x in 0..64 {
            let old = authored_town_geometry::kind(pack, before, [x, y])?;
            let new = authored_town_geometry::kind(pack, town, [x, y])?;
            ensure!(
                !matches!(new, "ICE" | "ICE_2B" | "CUT_TREE"),
                "incidental Brainerd gate"
            );
            if old == "WATER" || new == "WATER" {
                let i = usize::from(y / 2 * 32 + x / 2);
                ensure!(
                    town.blocks[i] == before.blocks[i],
                    "Brainerd water or native shore changed"
                );
            }
        }
    }
    for bank in &plan.fishing_banks {
        ensure!(
            clear(bank.stand)? && authored_town_geometry::kind(pack, town, bank.water)? == "WATER",
            "Brainerd dry fishing/launch bank blocked"
        );
    }
    Ok(())
}

/// Moves the existing outfitter to the lake street and clears both through lanes.
///
/// Preserves all rooms, actors, scripts, rewards, four guarded connections,
/// original Gym cliffs/stairs, public pier, water and fishing tables. Returns an
/// unselected pack; it does not migrate saved terrain or grant access/resources.
///
/// # Errors
/// Rejects source/parent drift, altered native assemblies, unsafe public paths,
/// blocked actors/posts/arrivals, changed water or unrelated content mutations.
/// Borrowed inputs remain unchanged on error.
pub fn revise_brainerd_lakeside_loop(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        hash(source.as_bytes()) == SOURCE,
        "reviewed Brainerd source differs"
    );
    let plan: Proposal = serde_json::from_str(source)?;
    ensure!(
        plan.schema_version == 1
            && plan.map == MAP
            && plan.tileset == "mn_outdoor_johto_stairs"
            && plan.dimensions_metatiles == [32, 24]
            && !plan.source_pixels_modified
            && base.identity.content_hash == PARENT
            && !base.report.manifests.iter().any(|m| m == MARKER),
        "current complete Brainerd parent required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let original = base.data.maps.get(MAP).context("current Brainerd town")?;
    ensure!(
        hash(&serde_json::to_vec(&serde_json::to_value(original)?)?) == plan.native_module_sha256
            && original.blocks == rows(&plan.before_rows)?
            && original.attributes.tileset_name == plan.tileset
            && [original.attributes.width, original.attributes.height] == [32, 24]
            && original.events.warps.len() == 10,
        "complete Brainerd preimage differs"
    );
    let mut town = original.clone();
    town.blocks = rows(&plan.rows)?;
    ensure!(
        plan.expected_changed_metatiles == 57
            && original
                .blocks
                .iter()
                .zip(&town.blocks)
                .filter(|(a, b)| a != b)
                .count()
                == 57,
        "Brainerd terrain delta differs"
    );
    for building in &plan.building_moves {
        let [x, y, width, height] = building.source_rect_metatiles;
        let [tx, ty] = building.destination_top_left_metatiles;
        for dy in 0..height {
            let a = usize::from((y + dy) * 32 + x);
            let b = usize::from((ty + dy) * 32 + tx);
            ensure!(
                original.blocks[a..a + usize::from(width)]
                    == town.blocks[b..b + usize::from(width)],
                "Brainerd facade must remain a complete native assembly"
            );
        }
    }
    for y in 0..24 {
        for x in 0..32 {
            let protected = x == 0
                || x == 31
                || y == 0
                || y == 23
                || plan.protected_rectangles_metatiles.iter().any(|area| {
                    let [px, py, width, height] = area.rectangle;
                    x >= px && x < px + width && y >= py && y < py + height
                });
            if protected {
                let i = usize::from(y * 32 + x);
                ensure!(
                    original.blocks[i] == town.blocks[i],
                    "protected Brainerd geometry changed"
                );
            }
        }
    }
    for change in &plan.warp_moves {
        let warp = town
            .events
            .warps
            .iter_mut()
            .find(|w| w.index == change.index)
            .context("Brainerd exterior warp")?;
        ensure!([warp.x, warp.y] == change.from_tile, "stale Brainerd warp");
        [warp.x, warp.y] = change.to_tile;
    }
    authoring::refresh_event_section(&mut town);
    geometry(base, original, &town, &plan)?;
    let mut child = base.clone();
    crate::authored_town_publication::replace(&mut child, town)?;
    // The installed town carries its private stair alias in the typed module;
    // its legacy attribute table still names Johto. This revision changes no
    // attributes, so preserve that exact existing table instead of normalizing it.
    let legacy_attributes = base
        .data
        .map_attributes
        .get(MAP)
        .context("original Brainerd attribute table")?;
    child
        .data
        .map_attributes
        .insert(MAP.into(), legacy_attributes.clone());
    child.report.manifests.push(MARKER.into());
    child.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        child.format_version,
        &child.data,
        &child.audio_manifest,
        &child.runtime_files,
        &child.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&child)?;
    crate::validate_compact_superior_geography(&child)?;
    let mut residual = child.clone();
    crate::authored_town_publication::replace(&mut residual, original.clone())?;
    residual
        .data
        .map_attributes
        .insert(MAP.into(), legacy_attributes.clone());
    residual.identity = base.identity.clone();
    residual.report = base.report.clone();
    ensure!(
        residual == *base,
        "unrelated Brainerd revision content changed"
    );
    Ok(child)
}
