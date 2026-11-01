//! Tighten Rochester's inhabited streets while retaining its current native game.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::authored_town_geometry;
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

const MAP: &str = "MnRochester";
const PARENT: &str = "54ddae67685118896bd00b9a02ef98d12482a15fca7be2502bc6133ba843aecf";
const SOURCE: &str = "3a58e14c9bd788e6b350c275966437b01e55199ff81d1564162bbb927e4a1296";
const MARKER: &str = "minnesota-rochester-care-green-v1";

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

fn hash(raw: &[u8]) -> String {
    format!("{:x}", Sha256::digest(raw))
}

fn parse_rows(rows: &[String]) -> Result<Vec<u16>> {
    ensure!(rows.len() == 16, "sixteen complete Rochester rows required");
    let mut blocks = Vec::with_capacity(320);
    for row in rows {
        let values = row
            .split_whitespace()
            .map(|word| u16::from_str_radix(word, 16))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(values.len() == 20, "twenty Rochester columns required");
        blocks.extend(values);
    }
    Ok(blocks)
}

fn geometry(pack: &CompiledGamePack, map: &MapModule, plan: &Proposal) -> Result<()> {
    let reached = authored_town_geometry::reachable(pack, map, [4, 18], false)?;
    for point in plan.public_door_returns.values() {
        ensure!(
            authored_town_geometry::kind(pack, map, *point)? == "FLOOR" && reached.contains(point),
            "Rochester public door approach is disconnected: {point:?}"
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
            authored_town_geometry::kind(pack, map, point)? == "FLOOR"
                && reached.contains(&point)
                && !map
                    .objects
                    .iter()
                    .any(|object| [object.x, object.y] == point),
            "Rochester Fly/recovery spawn lost its clear walking approach"
        );
    }
    for door in map.events.warps.iter().filter(|warp| warp.index % 2 == 1) {
        ensure!(
            authored_town_geometry::kind(pack, map, [door.x, door.y])? == "DOOR",
            "Rochester entrance lost its native door quadrant"
        );
    }
    for object in &map.objects {
        ensure!(
            authored_town_geometry::kind(pack, map, [object.x, object.y])? == "FLOOR",
            "Rochester actor is not on ordinary floor"
        );
    }
    for event in &map.events.bg_events {
        let [x, y] = [event.x, event.y];
        ensure!(
            authored_town_geometry::kind(pack, map, [x, y])? == "WALL"
                && [[x - 1, y], [x + 1, y], [x, y - 1], [x, y + 1]]
                    .iter()
                    .any(|point| reached.contains(point)),
            "Rochester sign must occupy a solid native post with a reading approach"
        );
    }
    for y in 0..32 {
        for x in 0..40 {
            ensure!(
                matches!(
                    authored_town_geometry::kind(pack, map, [x, y])?,
                    "FLOOR" | "WALL" | "DOOR"
                ),
                "care green must not add ice, water, encounter or incidental Cut gates"
            );
        }
    }
    ensure!(
        reached.contains(&[39, 18]) && reached.contains(&[39, 19]),
        "both first-route approach lanes must remain connected"
    );
    Ok(())
}

/// Moves Rochester's patient house and Gym onto a closer native town street.
///
/// Retains the exterior extent/connections, six room identities, every story
/// program, first-badge wardens, clinic/recovery, roster, art and audio. Corrects
/// the clinic sign's solid quadrant and relocates the guide and two other signs.
/// Returns an unselected pack; old saves and their terrain overrides are not
/// migrated. Native input and visual acceptance remain separate requirements.
///
/// # Errors
/// Rejects a changed source, parent or complete native preimage, damaged building
/// assemblies, stale anchors, unsafe door/sign/actor geometry or unrelated data
/// changes. The borrowed inputs remain unchanged on failure.
pub fn revise_rochester_care_green(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        hash(source.as_bytes()) == SOURCE,
        "reviewed Rochester source differs"
    );
    let plan: Proposal = serde_json::from_str(source)?;
    ensure!(
        plan.schema_version == 1
            && plan.map == MAP
            && plan.tileset == "johto_modern"
            && plan.dimensions_metatiles == [20, 16]
            && !plan.source_pixels_modified
            && base.identity.content_hash == PARENT
            && !base.report.manifests.iter().any(|entry| entry == MARKER),
        "complete reviewed Rochester parent required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let original = base.data.maps.get(MAP).context("current Rochester")?;
    ensure!(
        hash(&serde_json::to_vec(&serde_json::to_value(original)?)?) == plan.native_module_sha256
            && original.blocks == parse_rows(&plan.before_rows)?
            && original.attributes.tileset_name == plan.tileset
            && [original.attributes.width, original.attributes.height] == [20, 16],
        "complete Rochester map preimage differs"
    );
    let mut town = original.clone();
    town.blocks = parse_rows(&plan.rows)?;
    ensure!(
        original
            .blocks
            .iter()
            .zip(&town.blocks)
            .filter(|(a, b)| a != b)
            .count()
            == 117,
        "reviewed Rochester terrain scope differs"
    );
    for building in &plan.building_moves {
        let [x, y, width, height] = building.source_rect_metatiles;
        let [tx, ty] = building.destination_top_left_metatiles;
        for dy in 0..height {
            let before = usize::from((y + dy) * 20 + x);
            let after = usize::from((ty + dy) * 20 + tx);
            ensure!(
                original.blocks[before..before + usize::from(width)]
                    == town.blocks[after..after + usize::from(width)],
                "Rochester facade must retain its complete native assembly"
            );
        }
    }
    for y in 0..16 {
        for x in 0..20 {
            if x == 0 || x == 19 || y == 0 || y == 15 {
                let index = y * 20 + x;
                ensure!(
                    original.blocks[index] == town.blocks[index],
                    "Rochester perimeter changed"
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
            .context("reviewed Rochester warp")?;
        ensure!([warp.x, warp.y] == change.from, "stale Rochester warp");
        [warp.x, warp.y] = change.to;
    }
    for change in &plan.actor_moves {
        let actor = town
            .objects
            .iter_mut()
            .find(|actor| actor.object_identifier.as_deref() == Some(change.id.as_str()))
            .context("reviewed Rochester actor")?;
        ensure!([actor.x, actor.y] == change.from, "stale Rochester actor");
        [actor.x, actor.y] = change.to;
    }
    for change in &plan.sign_moves {
        let sign = town
            .events
            .bg_events
            .iter_mut()
            .find(|sign| sign.script == change.id)
            .context("reviewed Rochester sign")?;
        ensure!([sign.x, sign.y] == change.from, "stale Rochester sign");
        [sign.x, sign.y] = change.to;
    }
    authoring::refresh_event_section(&mut town);
    geometry(base, &town, &plan)?;
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
    // Reverse only this map's owned split payloads. Everything else, including
    // rooms, events, trainers, flags, art and audio, must recover the full parent.
    let mut residual = child.clone();
    crate::authored_town_publication::replace(&mut residual, original.clone())?;
    residual.identity = base.identity.clone();
    residual.report = base.report.clone();
    ensure!(
        residual == *base,
        "unrelated content changed during Rochester revision"
    );
    Ok(child)
}
