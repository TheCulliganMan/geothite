//! The original Gym3-to-Gym4 walking approach through the existing river park.
use anyhow::{Context, Result, ensure};
use crystal_core::map::MapConnection;
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;
use serde_json::{Value, json};

use crate::authored_checkpoint::{Checkpoint, install as checkpoint};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

const BASE: &str = "f98c5446760288142f6e56727f910b585d9dc67cbd9eaec5674c4bc542960821";
const MANIFEST: &str = "minnesota-gym-four-approach-v2";
const TOWN: &str = "MnMinneapolis";
const ROUTE: &str = "MnR04C00R00";

/// Opens the native Minneapolis/R04 seam leading to St Paul's existing gym.
///
/// Departure requires Avrin's actual Current Badge. The dispatch record is a
/// separate courier prerequisite. Inbound travel is free. Only two modules change: one
/// forest mouth, reciprocal connections and a two-lane checkpoint. Existing
/// route terrain, battles, gyms, story events, source art, audio and recovery
/// are retained. No badge or story progress is granted.
///
/// # Errors
/// Returns an error for a stale/duplicate input, changed source contracts,
/// unsafe boundaries, an occupied checkpoint or mismatched native projections.
/// Inputs are unchanged; callers receive only a fully verified successor.
pub fn build_gym_four_approach(
    base: &CompiledGamePack,
    contract_json: &str,
    route_json: &str,
    story_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let contract: Value = serde_json::from_str(contract_json)?;
    let route: Value = serde_json::from_str(route_json)?;
    let story: Value = serde_json::from_str(story_json)?;
    ensure!(
        base.identity.content_hash == BASE && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "exact River Valley input required; stale/duplicate fourth-gym approach"
    );
    ensure!(
        contract
            == json!({
                "schema_version":1,"manifest":MANIFEST,"base_content_hash":BASE,
                "town":TOWN,"route":ROUTE,"mouth_metatile":[31,12],
                "town_direction":"east","route_direction":"west","offset":3,
                "town_edge":[[63,24],[63,25]],"route_edge":[[0,18],[0,19]],
                "checkpoint_bodies":[[60,24],[60,25]],
                "checkpoint_inside":[[59,24],[59,25]],"checkpoint_outside":[[61,24],[61,25]],
                "requires":["BADGE_3"]
            }),
        "fixed fourth-gym approach contract required"
    );
    ensure!(
        route["id"] == "MN_R04"
            && route["required_flags"] == json!(["BADGE_3"])
            && route["dimensions_metatiles"] == json!([28, 18]),
        "current R04 extent and badge contract required"
    );
    let chapter = story["quests"]
        .as_array()
        .context("story quests")?
        .iter()
        .find(|m| m["id"] == "Q03")
        .context("Twin Cities chapter")?;
    ensure!(
        chapter["requires"] == json!(["BADGE_3"]),
        "existing Current-only road prerequisite required"
    );
    let mut town = base
        .data
        .maps
        .get(TOWN)
        .context("installed Minneapolis")?
        .clone();
    let mut road = base.data.maps.get(ROUTE).context("installed R04")?.clone();
    ensure!(
        [town.attributes.width, town.attributes.height] == [32, 24]
            && town.blocks[12 * 32 + 31] == 5
            && [road.attributes.width, road.attributes.height] == [28, 18],
        "native city mouth or river park dimensions changed"
    );
    town.blocks[12 * 32 + 31] = 1;
    connect(&mut town, "east", ROUTE, 3)?;
    connect(&mut road, "west", TOWN, -3)?;
    let ids = [
        "MN_MP_CROSSING_WARDEN_NORTH".into(),
        "MN_MP_CROSSING_WARDEN_SOUTH".into(),
    ];
    checkpoint(
        base,
        &mut town,
        &Checkpoint {
            identifiers: &ids,
            bodies: [[60, 24], [60, 25]],
            inside: [[59, 24], [59, 25]],
            outside: [[61, 24], [61, 25]],
            root: "MnMPEastGate",
            incoming_facing: "LEFT",
            outward: "east",
            required_event: "",
            badge_flags: &["ENGINE_PLAINBADGE"],
            badge_lines: &[
                "Bring AVRIN's".into(),
                "CURRENT BADGE.".into(),
                "for the crossing.".into(),
            ],
            evidence_lines: &[],
        },
    )?;
    for coord in &mut town.events.coord_events {
        if coord.script_name.starts_with("MnMPEastGate") {
            coord.scene_id = "-1".into();
        }
    }
    let closed = crate::authored_town_geometry::reachable(base, &town, [56, 24], true)?;
    ensure!(
        !closed.contains(&[63, 24]) && !closed.contains(&[63, 25]),
        "crossing checkpoint has a bypass"
    );
    let mut open = town.clone();
    open.objects.retain(|o| {
        !ids.iter()
            .any(|id| o.object_identifier.as_ref() == Some(id))
    });
    crate::prairie_story_connections::perimeter(
        base,
        &open,
        [56, 24],
        &[[32, 47], [33, 47], [63, 24], [63, 25]],
    )?;
    crate::prairie_story_connections::perimeter(
        base,
        &road,
        [0, 18],
        &[[0, 18], [0, 19], [55, 18], [55, 19]],
    )?;
    let mut pack = base.clone();
    for mut map in [town, road] {
        authoring::finish(&pack, &mut map)?;
        map.script_runtime_commands = crate::parse_script_runtime_commands(&map.id, &map.scripts)?;
        map.script_object_commands = crate::parse_script_object_commands(&map.id, &map.scripts)?;
        map.script_variable_commands =
            crate::parse_script_variable_commands(&map.id, &map.scripts)?;
        map.script_movements =
            crate::parse_script_movements(&map.id, &map.scripts, &map.script_object_commands)?;
        crate::authored_town_publication::replace(&mut pack, map)?;
    }
    verify_projections(&pack)?;
    pack.report.manifests.push(MANIFEST.into());
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

fn connect(map: &mut MapModule, direction: &str, target: &str, offset: i32) -> Result<()> {
    ensure!(
        !map.attributes
            .connections
            .iter()
            .any(|c| c.direction == direction),
        "fourth-gym mouth already connected"
    );
    map.attributes.connections.push(MapConnection {
        direction: direction.into(),
        target_map: target.into(),
        offset,
    });
    map.attributes.connection_flags = Some(
        map.attributes
            .connections
            .iter()
            .map(|c| c.direction.to_ascii_uppercase())
            .collect::<Vec<_>>()
            .join(" | "),
    );
    Ok(())
}

fn verify_projections(pack: &CompiledGamePack) -> Result<()> {
    for y in [18, 19] {
        for (name, from, target, to) in [
            (
                TOWN,
                TilePosition::new(64, y + 6),
                ROUTE,
                TilePosition::new(0, y),
            ),
            (
                ROUTE,
                TilePosition::new(-1, y),
                TOWN,
                TilePosition::new(63, y + 6),
            ),
            (
                ROUTE,
                TilePosition::new(56, y),
                "MnStPaul",
                TilePosition::new(0, y + 6),
            ),
            (
                "MnStPaul",
                TilePosition::new(-1, y + 6),
                ROUTE,
                TilePosition::new(55, y),
            ),
        ] {
            let m = &pack.data.maps[name];
            let native = OverworldMapData::from_attributes(name, &m.attributes, m.blocks.clone());
            let transition =
                connection_for_tile_checked(&native, from)?.context("river crossing projection")?;
            let arrival = pack.data.resolve_connection_transition(&transition)?;
            ensure!(
                arrival.destination.map_name == target && arrival.destination.tile == to,
                "fourth-gym native projection differs"
            );
        }
    }
    Ok(())
}
