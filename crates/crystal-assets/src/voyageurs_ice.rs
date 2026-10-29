//! Additive Minnesota ice dungeon built from explicit native Ice Path rows.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::json;

use crate::CompiledGamePack;
use crate::authored_town_publication::{self, CityIdentity};
use crate::grand_marais_modpack::authoring::{self, command};

#[path = "voyageurs_ice_geometry.rs"]
mod geometry;

#[derive(Deserialize)]
struct Design {
    schema_version: u8,
    id: String,
    tileset: String,
    border_block: u8,
    exterior: Exterior,
    rooms: Vec<Room>,
}

#[derive(Deserialize)]
struct Exterior {
    map: String,
    constant: String,
    entrance: [u16; 2],
    #[serde(rename = "return")]
    return_tile: [u16; 2],
    arrival_warp: u16,
    return_warp: u16,
    changes: Vec<Change>,
}

#[derive(Deserialize)]
struct Change {
    metatile: [u16; 2],
    previous: u16,
    replacement: u16,
}

#[derive(Deserialize)]
struct Room {
    id: String,
    constant: String,
    dimensions_metatiles: [u16; 2],
    rows: Vec<String>,
    arrival: [u16; 2],
    warps: Vec<Warp>,
    residents: Vec<Resident>,
    items: Vec<Item>,
}

#[derive(Deserialize)]
struct Warp {
    tile: [u16; 2],
    target: String,
    target_warp: i16,
}

#[derive(Deserialize)]
struct Resident {
    id: String,
    tile: [i16; 2],
    sprite: String,
    lines: Vec<String>,
}

#[derive(Deserialize)]
struct Item {
    id: String,
    tile: [u16; 2],
    item: String,
    quantity: u8,
    flag: String,
}

/// Adds the two-floor Voyageurs Ice Cavern and its paired R20 entrance.
///
/// The supplied pack owns all source art, audio and native ice movement. This
/// builder adds explicit maps/scripts and two exterior metatiles, preserving
/// HOME, recovery points, original Ice Path and unrelated regional content.
///
/// # Errors
/// Returns an error for stale exterior terrain, occupied identities, invalid
/// native collision, unreachable rewards/warps, or invalid pack/script data.
/// The borrowed input pack is never modified.
pub fn build_voyageurs_ice_modpack(
    base: &CompiledGamePack,
    source_json: &str,
) -> Result<CompiledGamePack> {
    crate::minnesota_interior_lineage::require_compact_coast(base)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let design: Design = serde_json::from_str(source_json)?;
    ensure!(
        design.schema_version == 1
            && design.id == "minnesota-voyageurs-ice-v1"
            && design.tileset == "ice_path"
            && design.rooms.len() == 2
            && design.rooms[0].id == "MnVoyageursIce1F"
            && design.rooms[1].id == "MnVoyageursIceB1F",
        "canonical explicit two-floor ice dungeon required"
    );
    let mut pack = base.clone();
    for item in design.rooms.iter().flat_map(|r| &r.items) {
        ensure!(
            pack.data.items.contains_key(&item.item) && item.quantity > 0,
            "native ice reward"
        );
        let globals = &mut pack.data.story_event_script_constants.global;
        ensure!(
            !globals.contains_key(&item.flag),
            "ice reward already registered"
        );
        let next = globals
            .iter()
            .filter(|(key, _)| key.starts_with("EVENT_"))
            .map(|(_, value)| *value)
            .max()
            .context("native event catalog")?
            .checked_add(1)
            .context("event identity overflow")?;
        u16::try_from(next)?;
        globals.insert(item.flag.clone(), next);
    }
    let mut modules = Vec::new();
    for room in &design.rooms {
        let mut map = authoring::blank(&pack, "IcePath1F", &room.id, &room.constant)?;
        crate::regional_town_fishing::freshwater(&pack, &mut map)?;
        let [width, height] = room.dimensions_metatiles;
        ensure!(
            width <= 32 && height <= 24 && width > 0 && height > 0,
            "bounded ice room"
        );
        ensure!(
            room.rows.len() == usize::from(height)
                && room
                    .rows
                    .iter()
                    .all(|r| r.split_whitespace().count() == usize::from(width)),
            "complete explicit ice rows"
        );
        authoring::blocks(&mut map, width, height, &room.rows.join("\n"))?;
        map.attributes.border_block = design.border_block;
        for warp in &room.warps {
            let target = design
                .rooms
                .iter()
                .find(|r| r.id == warp.target)
                .map(|r| r.constant.as_str())
                .or_else(|| {
                    (warp.target == design.exterior.map)
                        .then_some(design.exterior.constant.as_str())
                })
                .context("registered ice warp destination")?;
            authoring::warp(
                &mut map,
                (warp.tile[0], warp.tile[1]),
                target,
                warp.target_warp,
            );
        }
        for resident in &room.residents {
            let text = format!("{}Text", resident.id);
            authoring::text(
                &mut map,
                &text,
                &resident
                    .lines
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
            );
            map.scripts.insert(
                resident.id.clone(),
                json!([command("jumptextfaceplayer", &[&text])]),
            );
            authoring::npc(
                &pack,
                &mut map,
                &resident.id,
                &resident.sprite,
                resident.tile[0],
                resident.tile[1],
            )?;
        }
        for item in &room.items {
            map.scripts.insert(
                item.id.clone(),
                json!([command(
                    "itemball",
                    &[&item.item, &item.quantity.to_string()]
                )]),
            );
            authoring::npc(
                &pack,
                &mut map,
                &item.id,
                "SPRITE_POKE_BALL",
                i16::try_from(item.tile[0])?,
                i16::try_from(item.tile[1])?,
            )?;
            let object = map.objects.last_mut().context("new ice pickup")?;
            object.event_flag = item.flag.clone();
            object.object_type = "OBJECTTYPE_ITEMBALL".into();
        }
        authoring::finish(&pack, &mut map)?;
        map.script_object_commands = crate::parse_script_object_commands(&map.id, &map.scripts)?;
        map.script_field_pickups =
            crate::parse_script_field_pickups(&map.id, &map.scripts, &map.objects)?;
        ensure!(
            map.script_field_pickups.len() == room.items.len(),
            "complete native ice pickup mutations"
        );
        geometry::validate(&pack, &map, room)?;
        modules.push((map, room.arrival));
    }
    // Avoid map-group identities already used by earlier northern extensions.
    let group_id = (84_u16..=255)
        .find(|id| {
            pack.data
                .runtime_map_metadata
                .values()
                .all(|map| map.group_id != *id)
        })
        .context("unused native ice map group")?;
    authored_town_publication::publish(
        &mut pack,
        modules,
        CityIdentity {
            group: "MINNESOTA_VOYAGEURS_ICE",
            group_id: i16::try_from(group_id)?,
            landmark: "LANDMARK_MN_VOYAGEURS_ICE",
            fallback_landmark: Some("LANDMARK_MN_VOYAGEURS_BORDER_LOOP"),
            constant: "MN_VOYAGEURS_ICE_1F",
            name: "ICE CAVERN",
            position: [66, 6],
            manifest: &design.id,
            town: "",
            clinic: "",
            gym: "",
        },
    )?;
    let exterior = &design.exterior;
    let mut map = pack
        .data
        .maps
        .get(&exterior.map)
        .context("Voyageurs entrance map")?
        .clone();
    ensure!(
        map.attributes.map_constant.as_deref() == Some(&exterior.constant)
            && map.events.warps.len() + 1 == usize::from(exterior.arrival_warp)
            && exterior.return_warp == exterior.arrival_warp + 1,
        "paired exterior warp preimage"
    );
    for change in &exterior.changes {
        let [x, y] = change.metatile;
        ensure!(
            x < map.attributes.width && y < map.attributes.height,
            "bounded entrance edit"
        );
        let block =
            &mut map.blocks[usize::from(y) * usize::from(map.attributes.width) + usize::from(x)];
        ensure!(
            *block == change.previous,
            "stale ice entrance terrain at {x},{y}"
        );
        *block = change.replacement;
    }
    ensure!(
        crate::authored_town_geometry::kind(&pack, &map, exterior.entrance)? == "CAVE"
            && crate::authored_town_geometry::kind(&pack, &map, exterior.return_tile)? == "FLOOR",
        "native cave mouth and dry return"
    );
    authoring::warp(
        &mut map,
        (exterior.entrance[0], exterior.entrance[1]),
        &design.rooms[0].constant,
        3,
    );
    authoring::warp(
        &mut map,
        (exterior.return_tile[0], exterior.return_tile[1]),
        &design.rooms[0].constant,
        3,
    );
    authoring::refresh_event_section(&mut map);
    authored_town_publication::replace(&mut pack, map)?;
    pack.report.manifests.push(design.id);
    pack.report.maps = pack.data.maps.len();
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
