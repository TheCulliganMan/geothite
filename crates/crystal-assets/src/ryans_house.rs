//! Enterable native wooden cabin beside the existing Baptism River cascade.
use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use serde::Deserialize;
use serde_json::Value;

use crate::authored_route_design::{Door, Interior, RouteDesign, Sign};
use crate::authored_route_layout::Layout;
use crate::authored_route_modpack::{collision, validate_geometry, validate_room};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, CompiledMapExtension, RuntimeSpawnPoint};

const HOUSE: &str = "MnRyansHouse";
const CONSTANT: &str = "MN_RYANS_HOUSE";
const EXTERIOR: &str = "MnR11C00R00";
const MANIFEST: &str = "minnesota-ryans-house-v1";

#[derive(Deserialize)]
struct House {
    id: String,
    exterior_map: String,
    door: Door,
    sign: Sign,
    room: Interior,
    native_group_id: u16,
    native_map_id: u16,
}

/// Adds Ryan's optional house to the installed expanded North Shore.
///
/// Requires exact previous route terrain and actor anchors. Seven declared
/// metatiles make a wooden facade, porch and sign. River, cascades, existing
/// actors, regional links, story progression, art, audio and recovery remain
/// intact. The new room has two carpet exits and a passive FLOOR arrival.
/// No item, badge, story event or healing service is granted.
///
/// # Errors
/// Returns an error for incompatible/stale sources, occupied native identities,
/// unsafe route/room geometry, changed river terrain or verification failures.
/// All borrowed inputs remain unchanged.
pub fn build_ryans_house_modpack(
    base: &CompiledGamePack,
    previous_layout: &str,
    layout_json: &str,
    previous_dossier: &str,
    dossier_json: &str,
    house_json: &str,
) -> Result<CompiledGamePack> {
    let house: House = serde_json::from_str(house_json).context("Ryan's house source")?;
    let raw: Value = serde_json::from_str(house_json)?;
    let dossier: Value = serde_json::from_str(dossier_json)?;
    let design = RouteDesign::parse(dossier_json)?;
    let layout = Layout::parse(layout_json)?;
    let old = Layout::parse(previous_layout)?.compile()?;
    let blocks = layout.compile()?;
    ensure!(
        house.id == HOUSE
            && house.exterior_map == EXTERIOR
            && house.native_group_id == 41
            && house.native_map_id == 10
            && layout.id == "MN_R11"
            && layout.dimensions_metatiles == [56, 96]
            && layout.compiled_tileset == "mn_north_shore",
        "house requires the actual expanded Baptism River map"
    );
    ensure!(
        !base.data.maps.contains_key(HOUSE)
            && !base
                .data
                .runtime_map_metadata
                .values()
                .any(|m| m.group_id == 41 && m.map_id == 10)
            && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "Ryan's house identity is already occupied"
    );
    for (key, value) in [
        ("doors", &raw["door"]),
        ("interiors", &raw["room"]),
        ("signs", &raw["sign"]),
    ] {
        ensure!(
            dossier[key]
                .as_array()
                .context("route inventory")?
                .iter()
                .filter(|v| *v == value)
                .count()
                == 1,
            "house and route disagree on {key}"
        );
    }
    let patches = [
        ([11, 9], 0x2c),
        ([12, 9], 0x2d),
        ([11, 10], 0x2e),
        ([12, 10], 0x2f),
        ([11, 11], 0x02),
        ([12, 11], 0x02),
        ([13, 11], 0x45),
    ];
    let mut expected = old;
    for ([x, y], block) in patches {
        expected[y * 56 + x] = block;
    }
    ensure!(
        blocks == expected,
        "house may change only its seven declared bank metatiles"
    );
    let door = &house.door;
    ensure!(
        door.warp_index == 1
            && door.return_warp_index == Some(2)
            && door.tile == [23, 21]
            && door.return_tile == [23, 22]
            && door.arrival_tile == [23, 22]
            && door.target == HOUSE
            && door.target_warp == 3
            && door.collision == "DOOR"
            && house.sign.tile == [26, 22]
            && house.sign.id == "MN_R11_RYANS_HOUSE_SIGN",
        "retain the paired bank door, passive return and readable house sign"
    );
    let room = &house.room;
    ensure!(
        room.id == HOUSE
            && room.template == "VioletNicknameSpeechHouse"
            && room.dimensions_metatiles == [5, 4]
            && room.exits == [[5, 7], [6, 7]]
            && room.clear_arrival == Some([5, 6])
            && room
                .passive_arrival
                .as_ref()
                .is_some_and(|a| a.index == 3 && a.tile == [5, 6])
            && room.residents.len() == 1
            && room.residents[0].id == "MN_RYAN"
            && room.items.is_empty()
            && room.keeper_tile.is_none()
            && room.residents[0].service.is_none(),
        "retain the actual optional wood room, Ryan and two carpet returns"
    );
    let mut pack =
        crate::revise_authored_route_terrain(base, previous_layout, layout_json, previous_dossier)?;
    validate_geometry(&pack, &design, &blocks, &layout.compiled_tileset)?;
    let mut outside = pack.data.maps[EXTERIOR].clone();
    ensure!(
        outside.events.warps.is_empty(),
        "Baptism bank already has door records"
    );
    for (tile, kind) in [(door.tile, "DOOR"), (door.return_tile, "FLOOR")] {
        ensure!(
            collision(&pack, &outside.blocks, 32, tile, "mn_north_shore")? == kind,
            "unsafe house exterior {kind} anchor"
        );
        ensure!(
            !outside.objects.iter().any(|a| [a.x, a.y] == tile),
            "occupied house anchor"
        );
    }
    let outside_constant = outside
        .attributes
        .map_constant
        .clone()
        .context("bank map constant")?;
    authoring::warp(&mut outside, (23, 21), CONSTANT, 3);
    authoring::warp(&mut outside, (23, 22), CONSTANT, 3);
    let label = format!("{}Text", house.sign.id);
    authoring::text(
        &mut outside,
        &label,
        &house
            .sign
            .lines
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    );
    outside.scripts.insert(
        house.sign.id.clone(),
        serde_json::json!([authoring::command("jumptext", &[&label])]),
    );
    outside.events.bg_events.push(BackgroundEvent {
        x: 26,
        y: 22,
        event_type: "BGEVENT_READ".into(),
        script: house.sign.id,
    });
    authoring::finish(&pack, &mut outside)?;
    let mut cabin = authoring::blank(&pack, &room.template, HOUSE, CONSTANT)?;
    authoring::blocks(&mut cabin, 5, 4, &room.rows.join(" "))?;
    cabin.attributes.border_block = 0;
    cabin.attributes.location = outside.attributes.location.clone();
    cabin.attributes.map_group_constant = outside.attributes.map_group_constant.clone();
    validate_room(&pack, &cabin, room)?;
    for tile in [[5, 7], [6, 7], [5, 6]] {
        authoring::warp(&mut cabin, (tile[0], tile[1]), &outside_constant, 2);
    }
    crate::authored_route_events::room_events(&pack, &mut cabin, room)?;
    crate::insert_compiled_map_split_payloads(
        &mut pack.data.map_scripts,
        &mut pack.data.npcs,
        EXTERIOR,
        outside
            .attributes
            .map_scripts_label
            .as_deref()
            .context("bank scripts")?,
        outside
            .attributes
            .map_events_label
            .as_deref()
            .context("bank events")?,
        &outside,
    )?;
    pack.data.maps.insert(EXTERIOR.into(), outside);
    let mut metadata = pack.data.runtime_map_metadata[&outside_constant].clone();
    metadata.constant = CONSTANT.into();
    metadata.name = HOUSE.into();
    metadata.map_id = 10;
    metadata.width = 5;
    metadata.height = 4;
    metadata.environment = cabin
        .attributes
        .environment
        .clone()
        .context("house environment")?;
    metadata.phone_service = cabin.attributes.phone_service;
    let group_name = metadata.group_name.clone();
    pack.data
        .pokegear_landmarks
        .map_to_landmark
        .insert(HOUSE.into(), "LANDMARK_MN_NORTH_SHORE".into());
    let spawn = pack
        .data
        .runtime_spawn_points
        .values()
        .map(|s| s.identifier)
        .max()
        .context("existing spawns")?
        .checked_add(1)
        .context("temporary spawn overflow")?;
    crate::insert_compiled_map_extension(
        &mut pack,
        CompiledMapExtension {
            manifest_id: format!("{MANIFEST}-room"),
            map_name: HOUSE.into(),
            map_constant: CONSTANT.into(),
            metadata,
            spawn_key: spawn.to_string(),
            spawn: RuntimeSpawnPoint {
                identifier: spawn,
                map_constant: CONSTANT.into(),
                map_name: HOUSE.into(),
                group_name,
                group_id: 41,
                map_id: 10,
                tile_x: 5,
                tile_y: 6,
                metatile_x: 2,
                metatile_y: 3,
                subtile_x: 1,
                subtile_y: 0,
            },
            module: cabin,
            wild_encounters: None,
            start_new_game_here: false,
        },
    )?;
    pack.data.runtime_spawn_points.remove(&spawn.to_string());
    pack.report.manifests.push(MANIFEST.into());
    pack.report.maps = pack.data.maps.len();
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
