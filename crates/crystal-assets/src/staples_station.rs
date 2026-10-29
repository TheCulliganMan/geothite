//! Staples' public signal room and parcel counter on a paired woodland spur.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::map::{MapConnection, MapScriptSectionCommand};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::authored_route_design::Sign;
use crate::authored_town_design::{Actor, Room, Town};
use crate::authored_town_publication::{self, CityIdentity};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, authored_town_geometry, authored_town_scripts};

pub(crate) const ROAD: &str = "MnR15C00R00";
pub(crate) const JUNCTION: &str = "MnStaplesRailJunction";
pub(crate) const ROOM: &str = "MnStaplesStation";
pub(crate) const VISITED: &str = "EVENT_MN_STATION_STAPLES_VISITED";
pub(crate) const MART: &str = "MN_STAPLES_PARCELS";
const MANIFEST: &str = "minnesota-staples-station-v2";
const JUNCTION_CONSTANT: &str = "MN_STAPLES_RAIL_JUNCTION";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    schema_version: u16,
    id: String,
    base_content_hash: String,
    exterior: Exterior,
    room: Room,
    road_binding: Binding,
    boarding: Value,
    visit_flag: String,
    mart: String,
    roster_policy: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Exterior {
    id: String,
    native_constant: String,
    template: String,
    tileset: String,
    border_block: u8,
    dimensions_metatiles: [u16; 2],
    rows: Vec<String>,
    arrival: [u16; 2],
    residents: Vec<Actor>,
    door: Entrance,
    sign: Sign,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entrance {
    tile: [u16; 2],
    return_tile: [u16; 2],
    entry_index: u16,
    return_index: u16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    map: String,
    dimensions_metatiles: [u16; 2],
    tileset: String,
    patches: Vec<Patch>,
    route_edge: [[u16; 2]; 2],
    junction_edge: [[u16; 2]; 2],
    route_direction: String,
    junction_direction: String,
    route_offset: i32,
    junction_offset: i32,
    world_origin_relative_to_route: [i16; 2],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Patch {
    position: [u16; 2],
    before: u16,
    after: u16,
}

/// Adds Staples' native station, local conversations and working parcel shop.
///
/// A new leaf outpost connects to Central Lakes West through nine explicit
/// forest cells. The first actual hall entry records only Staples' visit flag.
/// The conductor closes the platform pending the separate five-stop train service.
/// Existing route gates, grass, water, story, recovery, HOME, art and audio remain
/// intact. The touched route's teams, wild and local fishing bindings use active
/// Minnesota species, retaining levels, rates and unrelated source-region tables.
/// No recovery or Fly destination is installed for this station.
///
/// # Errors
/// Rejects stale or duplicate inputs, occupied native identities, changed seam
/// projections, incomplete native facades, unsafe arrivals or dialogue, bypassed
/// grass, an open platform, invalid shop stock or failed strict verification.
pub fn build_staples_station_pack(
    base: &CompiledGamePack,
    source_json: &str,
) -> Result<CompiledGamePack> {
    crate::minnesota_interior_lineage::require_compact_coast(base)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let value: Value = serde_json::from_str(source_json)?;
    let source: Source = serde_json::from_value(value.clone()).context("Staples station source")?;
    ensure!(
        source.schema_version == 2
            && source.id == "MN_STAPLES_JUNCTION_V2"
            && source.base_content_hash == base.identity.content_hash
            && source.visit_flag == VISITED
            && source.mart == MART
            && source.roster_policy == "story-pokemon.json"
            && !base.data.maps.contains_key(JUNCTION)
            && !base.data.maps.contains_key(ROOM)
            && !base.report.manifests.iter().any(|id| id == MANIFEST),
        "the exact source-bound compact parent and two uninstalled Staples maps are required"
    );
    let exterior = &source.exterior;
    let room = &source.room;
    let binding = &source.road_binding;
    ensure!(
        exterior.id == JUNCTION
            && exterior.native_constant == JUNCTION_CONSTANT
            && exterior.template == "NewBarkTown"
            && exterior.tileset == "johto_modern"
            && exterior.border_block == 5
            && exterior.dimensions_metatiles == [16, 10]
            && exterior.arrival == [6, 18]
            && exterior.door.tile == [13, 7]
            && exterior.door.return_tile == [13, 8]
            && exterior.door.entry_index == 1
            && exterior.door.return_index == 2
            && exterior.sign.id == "MN_STAPLES_DEPOT_SIGN"
            && exterior.sign.tile == [18, 8]
            && room.id == ROOM
            && room.native_constant == "MN_STAPLES_STATION"
            && room.template == "GoldenrodMagnetTrainStation"
            && room.tileset == "train_station"
            && room.border_block == 15
            && room.dimensions_metatiles == [9, 9]
            && room.passive_arrival.index == 3
            && room.passive_arrival.tile == [9, 15]
            && room.exits.len() == 2
            && room.pc.is_none()
            && room.signs.len() == 1
            && room.signs[0].id == "MN_STAPLES_SIGNAL_BOOK"
            && room.signs[0].tile == [4, 10],
        "complete native station, signal room and public entrance anchors required"
    );
    for (index, exit) in room.exits.iter().enumerate() {
        ensure!(
            exit.index == u16::try_from(index)? + 1
                && exit.tile == [[8, 17], [9, 17]][index]
                && exit.target == JUNCTION
                && exit.target_warp == 2,
            "both hall carpets must return to the same safe porch"
        );
    }
    ensure!(
        binding.map == ROAD
            && binding.dimensions_metatiles == [32, 18]
            && binding.tileset == "kanto"
            && binding.route_edge == [[26, 0], [27, 0]]
            && binding.junction_edge == [[6, 19], [7, 19]]
            && binding.route_direction == "north"
            && binding.junction_direction == "south"
            && binding.route_offset == 10
            && binding.junction_offset == -10
            && binding.world_origin_relative_to_route == [10, -10]
            && source.boarding["conductor_core"] == json!([11, 9])
            && source.boarding["approach_core"] == json!([11, 10])
            && source.boarding["platform_core"] == json!([11, 7])
            && source.boarding["train_door_core"] == json!([6, 5])
            && source.boarding["other_train_door_core"] == json!([11, 5])
            && source.boarding["ride_arrival_core"] == json!([11, 11]),
        "paired woodland seam and the guarded native boarding throat required"
    );
    crate::staples_station_geometry::residents(&exterior.residents, room)?;
    let mut pack = base.clone();
    crate::authored_city_expansion_events::flags(&mut pack, &[VISITED.into()])?;
    ensure!(
        pack.data
            .marts
            .0
            .insert(MART.into(), room.residents[2].stock.clone())
            .is_none(),
        "Staples parcel counter identity is occupied"
    );
    let old_road = &base.data.maps[ROAD];
    ensure!(
        old_road.attributes.width == 32
            && old_road.attributes.height == 18
            && old_road.attributes.tileset_name == "kanto"
            && old_road
                .attributes
                .connections
                .iter()
                .all(|c| c.direction != "north")
            && binding.patches.len() == 9,
        "original Central Lakes West and an unused northern seam required"
    );
    let mut road = old_road.clone();
    crate::staples_station_roster::install(&mut pack, &mut road)?;
    let mut seen = BTreeSet::new();
    for patch in &binding.patches {
        let [x, y] = patch.position;
        let expected = match [x, y] {
            [9..=12, 0] => 0x51,
            [13, 0] | [9, 1..=4] => 1,
            _ => anyhow::bail!("Staples spur extends beyond the nine forest cells"),
        };
        let index = usize::from(y) * 32 + usize::from(x);
        ensure!(
            seen.insert([x, y])
                && patch.before == 0x0f
                && patch.after == expected
                && road.blocks[index] == patch.before,
            "Staples spur must retain native rock caps and exact forest preimages"
        );
        road.blocks[index] = patch.after;
    }
    road.attributes.connections.push(MapConnection {
        direction: "north".into(),
        target_map: JUNCTION.into(),
        offset: 10,
    });
    road.attributes.connection_flags = Some(
        road.attributes
            .connections
            .iter()
            .map(|c| c.direction.to_ascii_uppercase())
            .collect::<Vec<_>>()
            .join(" | "),
    );
    let town: Town = serde_json::from_value(json!({
        "id":JUNCTION,"native_constant":JUNCTION_CONSTANT,"compiled_tileset":"johto_modern",
        "dimensions_metatiles":[16,10],"ports":[],"residents":value["exterior"]["residents"],
        "signs":[value["exterior"]["sign"]],"interiors":[value["room"]],
        "doors":[{"warp_index":1,"return_warp_index":2,"tile":[13,7],"return_tile":[13,8],
            "target":ROOM,"target_warp":3,"arrival_tile":[9,15]}]
    }))?;
    let mut junction = authoring::blank(&pack, &exterior.template, JUNCTION, JUNCTION_CONSTANT)?;
    authoring::blocks(&mut junction, 16, 10, &exterior.rows.join(" "))?;
    junction.attributes.tileset_name = exterior.tileset.clone();
    junction.attributes.border_block = 5;
    junction.attributes.environment = Some("ROUTE".into());
    junction.attributes.music = old_road.attributes.music.clone();
    junction.attributes.fishing_group = None;
    authoring::warp(&mut junction, (13, 7), "MN_STAPLES_STATION", 3);
    authoring::warp(&mut junction, (13, 8), JUNCTION_CONSTANT, 2);
    authored_town_scripts::ordinary(
        &pack,
        &mut junction,
        &exterior.residents,
        std::slice::from_ref(&exterior.sign),
        "",
        "",
    )?;
    authored_town_scripts::finish(&pack, &mut junction, &town)?;
    let mut station = crate::authored_town_rooms::build(&pack, &town, room, "", MART)?;
    station.scripts.insert(
        "MnStaplesFirstVisit".into(),
        json!([
            authoring::command("setevent", &[VISITED]),
            authoring::command("endcallback", &[])
        ]),
    );
    authored_town_scripts::finish(&pack, &mut station, &town)?;
    station
        .map_script_section_commands
        .push(MapScriptSectionCommand {
            command: "callback".into(),
            args: vec!["MAPCALLBACK_NEWMAP".into(), "MnStaplesFirstVisit".into()],
            command_index: station.map_script_section_commands.len(),
        });
    let edges = BTreeSet::from([[6, 19], [7, 19]]);
    let modules = vec![(junction, [6, 18]), (station, [9, 15])];
    authored_town_geometry::validate_with_edges(&pack, &town, &modules, &edges)?;
    crate::staples_station_geometry::native(&pack, old_road, &road, &modules[0].0, &modules[1].0)?;
    let [(mut junction, start), station]: [(crate::MapModule, [u16; 2]); 2] = modules
        .try_into()
        .map_err(|_| anyhow::anyhow!("two Staples modules required"))?;
    junction.attributes.connections.push(MapConnection {
        direction: "south".into(),
        target_map: ROAD.into(),
        offset: -10,
    });
    junction.attributes.connection_flags = Some("SOUTH".into());
    let location = old_road
        .attributes
        .location
        .as_deref()
        .context("Central Lakes location")?;
    authored_town_publication::publish(
        &mut pack,
        vec![(junction, start), station],
        CityIdentity {
            group: "MINNESOTA_STAPLES_JUNCTION",
            group_id: 92,
            landmark: "LANDMARK_MN_STAPLES_JUNCTION",
            fallback_landmark: Some(location),
            constant: JUNCTION_CONSTANT,
            name: "STAPLES DEPOT",
            position: [37, 48],
            manifest: MANIFEST,
            town: ROAD,
            clinic: "",
            gym: "",
        },
    )?;
    authored_town_publication::replace(&mut pack, road)?;
    crate::staples_station_geometry::seams(&pack)?;
    pack.report.maps = pack.data.maps.len();
    pack.report.manifests.push(MANIFEST.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    crate::minnesota_interior_lineage::require_compact_coast(&pack)?;
    Ok(pack)
}

#[cfg(test)]
#[path = "staples_station_tests.rs"]
mod tests;
