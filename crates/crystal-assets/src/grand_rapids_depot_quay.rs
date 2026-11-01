//! Fit Grand Rapids' existing care, records and waterfront into a closer circuit.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::authored_town_geometry;
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

const MAP: &str = "MnGrandRapids";
const PARENT: &str = "ab3db9284e6f786fa34b39f754ec082c30496c0a25b550299510ef2cfeeb264f";
const SOURCE: &str = "ae52b80c42b7b667d4234c4b8f83ea5f4590097de89ac422972b24f3443643f1";
const MARKER: &str = "minnesota-grand-rapids-depot-quay-v1";

#[derive(Deserialize)]
struct Proposal {
    schema_version: u8,
    map: String,
    tileset: String,
    dimensions_metatiles: [u16; 2],
    source_pixels_modified: bool,
    native_module_sha256: String,
    before_rows: Vec<String>,
    rows: Vec<String>,
    building_moves: Vec<BuildingMove>,
    warp_moves: Vec<WarpMove>,
    actor_moves: Vec<NamedMove>,
    sign_moves: Vec<NamedMove>,
    public_door_returns: BTreeMap<String, [u16; 2]>,
    river_approaches: Vec<[u16; 2]>,
}

#[derive(Deserialize)]
struct BuildingMove {
    source_rect_metatiles: [u16; 4],
    destination_top_left_metatiles: [u16; 2],
}

#[derive(Deserialize)]
struct WarpMove {
    index: u16,
    from: [u16; 2],
    to: [u16; 2],
}

#[derive(Deserialize)]
struct NamedMove {
    id: String,
    from: [u16; 2],
    to: [u16; 2],
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn rows(source: &[String]) -> Result<Vec<u16>> {
    ensure!(source.len() == 24, "complete Grand Rapids height required");
    let mut result = Vec::with_capacity(24 * 24);
    for row in source {
        let values = row
            .split_whitespace()
            .map(|word| u16::from_str_radix(word, 16))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(values.len() == 24, "complete Grand Rapids width required");
        result.extend(values);
    }
    Ok(result)
}

fn geometry(
    pack: &CompiledGamePack,
    original: &MapModule,
    after: &MapModule,
    plan: &Proposal,
) -> Result<()> {
    let reached = authored_town_geometry::reachable(pack, after, [6, 24], false)?;
    let occupied = |point| after.objects.iter().any(|obj| [obj.x, obj.y] == point);
    for point in plan
        .public_door_returns
        .values()
        .chain(&plan.river_approaches)
    {
        ensure!(
            authored_town_geometry::kind(pack, after, *point)? == "FLOOR"
                && reached.contains(point)
                && !occupied(*point),
            "Grand Rapids public service or quay approach is obstructed: {point:?}"
        );
    }
    for spawn in pack
        .data
        .runtime_spawn_points
        .values()
        .filter(|spawn| spawn.map_name == MAP)
    {
        let point = [u16::try_from(spawn.tile_x)?, u16::try_from(spawn.tile_y)?];
        ensure!(
            authored_town_geometry::kind(pack, after, point)? == "FLOOR"
                && reached.contains(&point)
                && !occupied(point),
            "Grand Rapids Fly/recovery landing is obstructed"
        );
    }
    for y in 0..48 {
        for x in 0..48 {
            let a = authored_town_geometry::kind(pack, original, [x, y])?;
            let b = authored_town_geometry::kind(pack, after, [x, y])?;
            ensure!(
                (a == "TALL_GRASS") == (b == "TALL_GRASS")
                    && matches!(a, "WATER" | "WATERFALL" | "BUOY")
                        == matches!(b, "WATER" | "WATERFALL" | "BUOY")
                    && !matches!(b, "ICE" | "ICE_2B" | "CUT_TREE"),
                "Grand Rapids habitat/water changed or an incidental ice/Cut gate was added"
            );
        }
    }
    for y in [24, 25] {
        for x in 4..44 {
            ensure!(
                authored_town_geometry::kind(pack, after, [x, y])? == "FLOOR" && !occupied([x, y]),
                "both ordinary street lanes must remain clear"
            );
        }
    }
    for y in 34..=38 {
        for x in [25, 26] {
            ensure!(
                reached.contains(&[x, y])
                    && authored_town_geometry::kind(pack, after, [x, y])? == "FLOOR"
                    && !occupied([x, y]),
                "both native pier lanes must remain clear"
            );
        }
    }
    for actor in &after.objects {
        ensure!(
            authored_town_geometry::kind(pack, after, [actor.x, actor.y])? == "FLOOR",
            "Grand Rapids actor lost its native floor"
        );
    }
    let (door_pairs, remainder) = after.events.warps.as_chunks::<2>();
    ensure!(
        remainder.is_empty(),
        "every Grand Rapids door needs its return pair"
    );
    for pair in door_pairs {
        ensure!(
            authored_town_geometry::kind(pack, after, [pair[0].x, pair[0].y])? == "DOOR"
                && pair[1].x == pair[0].x
                && pair[1].y == pair[0].y + 1
                && pair[1].target_map_constant == "MN_GRAND_RAPIDS",
            "Grand Rapids native door/passive return pair differs"
        );
    }
    for event in &after.events.bg_events {
        let [x, y] = [event.x, event.y];
        ensure!(
            authored_town_geometry::kind(pack, after, [x, y])? == "WALL"
                && [[x - 1, y], [x + 1, y], [x, y - 1], [x, y + 1]]
                    .iter()
                    .any(|point| reached.contains(point)),
            "Grand Rapids sign lost its solid post or reading approach"
        );
    }
    Ok(())
}

/// Moves Grand Rapids' existing services onto its street and waterfront circuit.
///
/// Preserves the winter alias, exact grass/water positions, lower riverbank,
/// native pier, checkpoint planes, room identities and all story programs.
/// Moves the worker and optional angler off the through street and pier lanes.
/// Returns an unselected pack; existing saved terrain is not migrated.
///
/// # Errors
/// Rejects source/parent/preimage drift, incomplete native assemblies, unsafe
/// door/sign/actor geometry, changed habitats or unrelated content mutations.
/// Borrowed inputs remain unchanged on error.
pub fn revise_grand_rapids_depot_quay(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        hash(source.as_bytes()) == SOURCE,
        "reviewed depot/quay source differs"
    );
    let plan: Proposal = serde_json::from_str(source)?;
    ensure!(
        plan.schema_version == 1
            && plan.map == MAP
            && plan.tileset == "mn_winter_grand_rapids"
            && plan.dimensions_metatiles == [24, 24]
            && !plan.source_pixels_modified
            && base.identity.content_hash == PARENT
            && !base.report.manifests.iter().any(|entry| entry == MARKER),
        "complete current Grand Rapids parent required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let original = base.data.maps.get(MAP).context("current Grand Rapids")?;
    ensure!(
        hash(&serde_json::to_vec(&serde_json::to_value(original)?)?) == plan.native_module_sha256
            && original.blocks == rows(&plan.before_rows)?
            && original.attributes.tileset_name == plan.tileset
            && [original.attributes.width, original.attributes.height] == [24, 24]
            && original.events.warps.len() == 6,
        "complete Grand Rapids preimage differs"
    );
    let mut town = original.clone();
    town.blocks = rows(&plan.rows)?;
    ensure!(
        original
            .blocks
            .iter()
            .zip(&town.blocks)
            .filter(|(a, b)| a != b)
            .count()
            == 123,
        "reviewed depot/quay terrain scope differs"
    );
    for building in &plan.building_moves {
        let [x, y, width, height] = building.source_rect_metatiles;
        let [tx, ty] = building.destination_top_left_metatiles;
        for dy in 0..height {
            let a = usize::from((y + dy) * 24 + x);
            let b = usize::from((ty + dy) * 24 + tx);
            ensure!(
                original.blocks[a..a + usize::from(width)]
                    == town.blocks[b..b + usize::from(width)],
                "Grand Rapids facade must retain its complete native assembly"
            );
        }
    }
    for y in 0..24 {
        for x in 0..24 {
            if !(2..22).contains(&x) || y == 0 || y >= 17 {
                let index = y * 24 + x;
                ensure!(
                    original.blocks[index] == town.blocks[index],
                    "Grand Rapids checkpoint planes, perimeter or lower shore changed"
                );
            }
        }
    }
    for change in &plan.warp_moves {
        let warp = town
            .events
            .warps
            .iter_mut()
            .find(|warp| warp.index == change.index)
            .context("reviewed Grand Rapids warp")?;
        ensure!([warp.x, warp.y] == change.from, "stale Grand Rapids warp");
        [warp.x, warp.y] = change.to;
    }
    for change in &plan.actor_moves {
        let actor = town
            .objects
            .iter_mut()
            .find(|obj| obj.object_identifier.as_deref() == Some(change.id.as_str()))
            .context("reviewed Grand Rapids actor")?;
        ensure!(
            [actor.x, actor.y] == change.from,
            "stale Grand Rapids actor"
        );
        [actor.x, actor.y] = change.to;
    }
    for change in &plan.sign_moves {
        let sign = town
            .events
            .bg_events
            .iter_mut()
            .find(|event| event.script == change.id)
            .context("reviewed Grand Rapids sign")?;
        ensure!([sign.x, sign.y] == change.from, "stale Grand Rapids sign");
        [sign.x, sign.y] = change.to;
    }
    authoring::refresh_event_section(&mut town);
    geometry(base, original, &town, &plan)?;
    let mut child = base.clone();
    crate::authored_town_publication::replace(&mut child, town)?;
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
    residual.identity = base.identity.clone();
    residual.report = base.report.clone();
    ensure!(
        residual == *base,
        "unrelated content changed during depot/quay revision"
    );
    Ok(child)
}
