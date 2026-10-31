//! Two native roadside businesses and their surrounding encounter grass.

use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use serde::Deserialize;

use crate::authored_route_design::{Interior, Sign};
use crate::authored_town_geometry;
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, CompiledMapExtension, MapModule, RuntimeSpawnPoint};

const MANIFEST: &str = "north-shore-shops-v1";

#[derive(Deserialize)]
struct Plan {
    schema_version: u16,
    id: String,
    parent_content_hash: String,
    shops: Vec<Shop>,
}

#[derive(Deserialize)]
struct Shop {
    id: String,
    constant: String,
    exterior_map: String,
    native_group_id: u16,
    native_map_id: u16,
    cells: Vec<Cell>,
    door: [u16; 2],
    porch: [u16; 2],
    road_approach: [u16; 2],
    sign: Sign,
    grass_bounds_core: Vec<[u16; 4]>,
    room: Interior,
}

#[derive(Deserialize)]
struct Cell {
    metatile: [u16; 2],
    before: String,
    after: String,
}

/// Adds an enterable pie shop and smokehouse to the selected compact North Shore.
///
/// Uses the supplied authoring contract and existing native facades, cafe art,
/// residents and script parsers. Each room has two carpet exits and a separate
/// passive arrival. New grass retains the parent's encounters; this operation
/// does not select new species or grant resources. Existing maps outside the two
/// shop sites, source art, audio, trainers, story, HOME and recovery stay intact.
/// The borrowed parent is unchanged.
///
/// # Errors
/// Rejects a stale parent, occupied map identity, changed terrain preimage,
/// blocked door/return/resident, inaccessible grass, or invalid resulting pack.
pub fn build_north_shore_shops(base: &CompiledGamePack, source: &str) -> Result<CompiledGamePack> {
    let plan: Plan = serde_json::from_str(source).context("North Shore shops source")?;
    ensure!(
        plan.schema_version == 1
            && plan.id == MANIFEST
            && plan.shops.len() == 2
            && plan.parent_content_hash == base.identity()?.content_hash,
        "shops require their reviewed current parent and two-site contract"
    );
    crate::validate_compact_superior_geography(base)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let mut result = base.clone();
    for shop in &plan.shops {
        install_shop(&mut result, shop)?;
    }
    result.report.maps = result.data.maps.len();
    result.report.manifests.push(MANIFEST.into());
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    Ok(result)
}

fn install_shop(pack: &mut CompiledGamePack, shop: &Shop) -> Result<()> {
    ensure!(
        shop.room.id == shop.id
            && shop.native_group_id == 41
            && !pack.data.maps.contains_key(&shop.id)
            && !pack
                .data
                .runtime_map_metadata
                .values()
                .any(|m| { m.group_id == shop.native_group_id && m.map_id == shop.native_map_id }),
        "shop room/native identity is occupied or inconsistent"
    );
    let mut outside = pack
        .data
        .maps
        .get(&shop.exterior_map)
        .context("North Shore shop site")?
        .clone();
    ensure!(
        outside.attributes.tileset_name == "mn_north_shore"
            && shop.exterior_map.starts_with("MnR11C00R")
            && outside.attributes.width == 32
            && outside.attributes.height == 24,
        "retain the warm compact North Shore footprint"
    );
    let mut seen = BTreeSet::new();
    for cell in &shop.cells {
        let [x, y] = cell.metatile;
        ensure!(x < 32 && y < 24 && seen.insert([x, y]), "invalid shop cell");
        let index = usize::from(y) * 32 + usize::from(x);
        ensure!(
            outside.blocks[index] == u16::from_str_radix(&cell.before, 16)?,
            "shop terrain preimage differs at {} ({x}, {y})",
            shop.exterior_map
        );
        // Keep every existing actor, warp and sign's metatile untouched.
        ensure!(
            !outside.objects.iter().any(|a| [a.x / 2, a.y / 2] == [x, y])
                && !outside
                    .events
                    .warps
                    .iter()
                    .any(|w| [w.x / 2, w.y / 2] == [x, y])
                && !outside
                    .events
                    .bg_events
                    .iter()
                    .any(|e| [e.x / 2, e.y / 2] == [x, y]),
            "shop patch overlaps an existing event"
        );
        outside.blocks[index] = u16::from_str_radix(&cell.after, 16)?;
    }
    let constant = outside
        .attributes
        .map_constant
        .clone()
        .context("coast constant")?;
    ensure!(
        authored_town_geometry::kind(pack, &outside, shop.door)? == "DOOR"
            && authored_town_geometry::kind(pack, &outside, shop.porch)? == "FLOOR"
            && authored_town_geometry::kind(pack, &outside, shop.road_approach)? == "FLOOR"
            && authored_town_geometry::kind(pack, &outside, shop.sign.tile)? == "WALL",
        "real facade door, dry road/porch and solid readable sign required"
    );
    let reachable = authored_town_geometry::reachable(pack, &outside, shop.road_approach, false)?;
    ensure!(
        reachable.contains(&shop.door) && reachable.contains(&shop.porch),
        "shop disconnected from road"
    );
    for &[x0, y0, x1, y1] in &shop.grass_bounds_core {
        ensure!(
            x0 < x1 && y0 < y1 && x1 <= 64 && y1 <= 48,
            "invalid habitat bounds"
        );
        for y in y0..y1 {
            for x in x0..x1 {
                ensure!(
                    authored_town_geometry::kind(pack, &outside, [x, y])? == "TALL_GRASS"
                        && reachable.contains(&[x, y]),
                    "shop habitat is not reachable native grass: {}, ({x}, {y})",
                    shop.id
                );
            }
        }
    }
    let entrance = i16::try_from(outside.events.warps.len() + 1)?;
    let return_index = entrance.checked_add(1).context("exterior return index")?;
    authoring::warp(
        &mut outside,
        (shop.door[0], shop.door[1]),
        &shop.constant,
        3,
    );
    authoring::warp(
        &mut outside,
        (shop.porch[0], shop.porch[1]),
        &shop.constant,
        3,
    );
    add_sign(pack, &mut outside, &shop.sign)?;
    authoring::refresh_event_section(&mut outside);

    let mut room = authoring::blank(pack, &shop.room.template, &shop.id, &shop.constant)?;
    let [width, height] = shop.room.dimensions_metatiles;
    authoring::blocks(&mut room, width, height, &shop.room.rows.join(" "))?;
    room.attributes.border_block = 0;
    room.attributes.location = outside.attributes.location.clone();
    room.attributes.map_group_constant = outside.attributes.map_group_constant.clone();
    crate::authored_route_modpack::validate_room(pack, &room, &shop.room)?;
    ensure!(
        shop.room.exits == [[2, 7], [3, 7]]
            && shop
                .room
                .passive_arrival
                .as_ref()
                .is_some_and(|a| a.index == 3 && a.tile == [3, 6]),
        "paired cafe carpets and passive floor arrival required"
    );
    for tile in shop.room.exits.iter().copied().chain([[3, 6]]) {
        authoring::warp(&mut room, (tile[0], tile[1]), &constant, return_index);
    }
    crate::authored_route_events::room_events(pack, &mut room, &shop.room)?;
    store_exterior(pack, outside)?;
    install_room(pack, shop, room, &constant)
}

fn add_sign(pack: &CompiledGamePack, outside: &mut MapModule, sign: &Sign) -> Result<()> {
    // Parse just the new sign: preserve every existing compiled story command.
    let constant = outside
        .attributes
        .map_constant
        .as_deref()
        .context("coast constant")?;
    let mut added = authoring::blank(pack, &outside.id, &outside.id, constant)?;
    let label = format!("{}Text", sign.id);
    ensure!(
        !outside.scripts.contains_key(&sign.id) && !outside.scripts.contains_key(&label),
        "occupied shop sign label"
    );
    authoring::text(
        &mut added,
        &label,
        &sign.lines.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    added.scripts.insert(
        sign.id.clone(),
        serde_json::json!([authoring::command("jumptext", &[&label])]),
    );
    authoring::finish(pack, &mut added)?;
    outside.scripts.extend(added.scripts);
    outside
        .script_text_commands
        .extend(added.script_text_commands);
    outside.script_text_bodies.extend(added.script_text_bodies);
    outside.events.bg_events.push(BackgroundEvent {
        x: sign.tile[0],
        y: sign.tile[1],
        event_type: "BGEVENT_READ".into(),
        script: sign.id.clone(),
    });
    Ok(())
}

fn store_exterior(pack: &mut CompiledGamePack, map: MapModule) -> Result<()> {
    let blocks = map
        .attributes
        .blocks_label
        .as_deref()
        .context("coast blocks")?;
    let bytes = map
        .blocks
        .iter()
        .map(|&b| u8::try_from(b))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    pack.data
        .map_blocks
        .insert(blocks.into(), crate::encode_base64_bytes(&bytes));
    crate::insert_compiled_map_split_payloads(
        &mut pack.data.map_scripts,
        &mut pack.data.npcs,
        &map.id,
        map.attributes
            .map_scripts_label
            .as_deref()
            .context("coast scripts")?,
        map.attributes
            .map_events_label
            .as_deref()
            .context("coast events")?,
        &map,
    )?;
    pack.data.maps.insert(map.id.clone(), map);
    Ok(())
}

fn install_room(
    pack: &mut CompiledGamePack,
    shop: &Shop,
    room: MapModule,
    outside: &str,
) -> Result<()> {
    let mut metadata = pack
        .data
        .runtime_map_metadata
        .get(outside)
        .context("coast metadata")?
        .clone();
    metadata.constant = shop.constant.clone();
    metadata.name = shop.id.clone();
    metadata.map_id = shop.native_map_id;
    metadata.width = room.attributes.width;
    metadata.height = room.attributes.height;
    metadata.environment = room
        .attributes
        .environment
        .clone()
        .context("shop environment")?;
    metadata.phone_service = room.attributes.phone_service;
    let group_name = metadata.group_name.clone();
    let spawn = pack
        .data
        .runtime_spawn_points
        .values()
        .map(|s| s.identifier)
        .max()
        .context("existing spawns")?
        .checked_add(1)
        .context("spawn overflow")?;
    crate::insert_compiled_map_extension(
        pack,
        CompiledMapExtension {
            manifest_id: format!("{MANIFEST}-{}", shop.id),
            map_name: shop.id.clone(),
            map_constant: shop.constant.clone(),
            metadata,
            spawn_key: spawn.to_string(),
            spawn: RuntimeSpawnPoint {
                identifier: spawn,
                map_constant: shop.constant.clone(),
                map_name: shop.id.clone(),
                group_name,
                group_id: i16::try_from(shop.native_group_id)?,
                map_id: i16::try_from(shop.native_map_id)?,
                tile_x: 3,
                tile_y: 6,
                metatile_x: 1,
                metatile_y: 3,
                subtile_x: 1,
                subtile_y: 0,
            },
            module: room,
            wild_encounters: None,
            start_new_game_here: false,
        },
    )?;
    pack.data.runtime_spawn_points.remove(&spawn.to_string());
    pack.data
        .pokegear_landmarks
        .map_to_landmark
        .insert(shop.id.clone(), "LANDMARK_MN_NORTH_SHORE".into());
    Ok(())
}
