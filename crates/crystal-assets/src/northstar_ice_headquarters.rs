//! Cross-floor navigation replaces the linear cargo-to-relay connection.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::json;

use crate::grand_marais_modpack::authoring;
use crate::grand_marais_modpack::authoring::command as c;
use crate::northstar_ice_headquarters_events as events;
use crate::northstar_lair_authoring as a;
use crate::{CompiledGamePack, MapModule};

pub(crate) const KEY: &str = "EVENT_MN_ICE_HQ_CARD_RECEIVED";
const CARGO: &str = "MnIceCargoBasement";
const RELAY: &str = "MnIceRelayFloor";
const OFFICE: &str = "MnMinneapolisDispatchOffice";
const FLOORS: [(&str, &str); 4] = [
    ("MnIceOperations", "MN_ICE_OPERATIONS"),
    ("MnIceRecords", "MN_ICE_RECORDS"),
    ("MnIceMaintenance", "MN_ICE_MAINTENANCE"),
    ("MnIceTransfer", "MN_ICE_TRANSFER"),
];

#[derive(Deserialize)]
struct Endpoint {
    map: String,
    active: [u16; 2],
    arrival: [u16; 2],
}

#[derive(Deserialize)]
struct Link {
    id: String,
    kind: String,
    a: Endpoint,
    b: Endpoint,
}

#[derive(Deserialize)]
pub(crate) struct Door {
    id: String,
    map: String,
    metatile: [u16; 2],
    flag: String,
}

#[derive(Deserialize)]
struct Network {
    schema_version: u8,
    id: String,
    maps: Vec<String>,
    links: Vec<Link>,
    doors: Vec<Door>,
}

/// Adds four compartmented floors, eight panel pairs and six persistent doors.
///
/// Apply after the drill chapter and before the regional species binder. All
/// existing rescue/battle results remain independent. The parent is unchanged.
///
/// # Errors
/// Returns an error for an incompatible parent, occupied identifiers, malformed
/// floor/connection data, unsupported scripts or failed runtime validation.
pub fn build_northstar_ice_headquarters(
    base: &CompiledGamePack,
    network: &str,
    layouts: &[String],
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.report
            .manifests
            .iter()
            .any(|m| m == "minnesota-northstar-drill-lair-v2")
            && !base
                .report
                .manifests
                .iter()
                .any(|m| m == "northstar-minnesota-only-v1")
            && !base.data.maps.contains_key("MnIceOperations"),
        "unbound drilling parent required; do not overwrite a previous headquarters"
    );
    let source: Network = serde_json::from_str(network)?;
    ensure!(
        source.schema_version == 1
            && source.id == "northstar-ice-headquarters-v1"
            && layouts.len() == FLOORS.len()
            && source.maps == FLOORS.map(|(m, _)| m)
            && source.links.len() == 12
            && source.doors.len() == 6,
        "complete authored headquarters network required"
    );
    let mut pack = base.clone();
    install_panel_art(&mut pack)?;
    for flag in [
        KEY,
        "EVENT_MN_ICE_HQ_TESS_HELPED",
        "EVENT_MN_ICE_HQ_SUPPLIES",
    ] {
        a::flag(&mut pack, flag)?;
    }
    let mut maps = BTreeMap::new();
    for (index, &(name, constant)) in FLOORS.iter().enumerate() {
        let mut room = a::room(&pack, &serde_json::from_str(&layouts[index])?, constant)?;
        ensure!(
            room.id == name && room.attributes.width == 16 && room.attributes.height == 12,
            "authored floor identity/dimensions differ"
        );
        room.attributes.tileset_name = "mn_ice_headquarters".into();
        maps.insert(name.to_owned(), room);
    }
    for name in [CARGO, RELAY] {
        let mut room = pack.data.maps[name].clone();
        room.events.warps.retain(|w| {
            !matches!(
                w.target_map_constant.as_str(),
                "MN_ICE_RELAY_FLOOR" | "MN_ICE_CARGO_BASEMENT"
            )
        });
        for (index, warp) in room.events.warps.iter_mut().enumerate() {
            warp.index = u16::try_from(index + 1)?;
        }
        maps.insert(name.into(), room);
    }
    for link in &source.links {
        connect(&mut maps, link)?;
    }
    for door in &source.doors {
        a::flag(&mut pack, &door.flag)?;
        install_door(maps.get_mut(&door.map).context("door floor")?, door)?;
    }
    events::install(&mut pack, &mut maps)?;
    for &(name, _) in &FLOORS {
        let room = maps.remove(name).context("new floor")?;
        let arrival = if name == "MnIceRecords" {
            [8, 19]
        } else {
            [4, 19]
        };
        a::publish(&mut pack, room, OFFICE, arrival)?;
    }
    for (_, mut room) in maps {
        crate::ely_league_rooms::finish(&pack, &mut room)?;
        room.script_audio_commands = crate::parse_script_audio_commands(&room.id, &room.scripts)?;
        crate::authored_town_publication::replace(&mut pack, room)?;
    }
    pack.report.maps = pack.data.maps.len();
    pack.report.manifests.push(source.id);
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

fn connect(maps: &mut BTreeMap<String, MapModule>, link: &Link) -> Result<()> {
    ensure!(
        link.a.map != link.b.map && matches!(link.kind.as_str(), "panel" | "stairs"),
        "paired cross-floor connection required"
    );
    let a_id = i16::try_from(
        maps.get(&link.a.map)
            .context("origin floor")?
            .events
            .warps
            .len()
            + 2,
    )?;
    let b_id = i16::try_from(
        maps.get(&link.b.map)
            .context("destination floor")?
            .events
            .warps
            .len()
            + 2,
    )?;
    for (from, to, target_id) in [(&link.a, &link.b, b_id), (&link.b, &link.a, a_id)] {
        let constant = maps[&to.map]
            .attributes
            .map_constant
            .clone()
            .context("destination constant")?;
        let room = maps.get_mut(&from.map).context("connection floor")?;
        ensure!(from.active != from.arrival, "distinct safe panel arrival");
        ensure!(
            [from.active, from.arrival]
                .iter()
                .all(|[x, y]| *x < room.attributes.width * 2 && *y < room.attributes.height * 2),
            "connection lies outside its floor"
        );
        if FLOORS.iter().any(|(name, _)| *name == room.id) {
            let [x, y] = from.active;
            ensure!(x % 2 == 1 && y % 2 == 0, "native upper-right active tile");
            let index =
                usize::from(y / 2) * usize::from(room.attributes.width) + usize::from(x / 2);
            ensure!(
                room.blocks[index] == 7,
                "{} panel/stair is not clear floor",
                link.id
            );
            room.blocks[index] = if link.kind == "panel" { 0x40 } else { 0x04 };
        }
        for [x, y] in [from.active, from.arrival] {
            ensure!(
                !room.events.warps.iter().any(|w| w.x == x && w.y == y),
                "overlapping connection"
            );
            authoring::warp(room, (x, y), &constant, target_id);
        }
        if link.kind == "panel" {
            let label = format!("{}Panel{}Text", room.id, link.id);
            authoring::text(
                room,
                &label,
                &[&format!("TRANSFER {}", link.id), "Step onto the pad."],
            );
            let script = format!("{}Panel{}", room.id, link.id);
            room.scripts
                .insert(script.clone(), json!([c("jumptext", &[&label])]));
            a::read(room, [from.active[0] + 1, from.active[1]], &script);
        }
    }
    Ok(())
}

fn install_panel_art(pack: &mut CompiledGamePack) -> Result<()> {
    // Preserve facility collision. Only these floors gain an appended native
    // metal plate whose upper-right quadrant has real warp-panel behavior.
    let mut definition = pack.data.tilesets["facility"].clone();
    definition.collision.insert(
        "40".into(),
        ["FLOOR", "WARP_PANEL", "FLOOR", "FLOOR"]
            .map(String::from)
            .to_vec(),
    );
    let files = &pack.runtime_files;
    let mut metatiles = files
        .get("data/tilesets/facility_metatiles.bin")
        .context("native facility blocks")?
        .clone();
    ensure!(metatiles.len() == 64 * 16, "original facility block count");
    let plate = metatiles[0x1b * 16..0x1c * 16].to_vec();
    metatiles.extend(plate);
    let mut graphics = files
        .get("gfx/tilesets/facility.2bpp")
        .context("native facility glyphs")?
        .clone();
    // Stock facility stores one bank of 96 glyphs. Typed extensions use two
    // equal banks; append an empty second bank without changing source pixels.
    ensure!(
        graphics.len() == 96 * 16
            && metatiles
                .iter()
                .all(|t| definition.palette_map[usize::from(*t)] & 8 == 0),
        "native single-bank facility art required"
    );
    graphics.resize(192 * 16, 0);
    let png = crate::native_tile_png::from_2bpp(&graphics)?;
    let extension = crate::CompiledTilesetExtension {
        manifest_id: "northstar-ice-headquarters-panel-art-v1".into(),
        tileset_id: "mn_ice_headquarters".into(),
        behavior_source_tileset_id: Some("facility".into()),
        definition,
        metatiles,
        tile_graphics_2bpp: graphics,
        tile_graphics_png: png,
    };
    crate::validate_compiled_tileset_extension(&extension)?;
    crate::insert_compiled_tileset_extension(pack, extension)?;
    Ok(())
}

fn install_door(room: &mut MapModule, door: &Door) -> Result<()> {
    let [x, y] = door.metatile;
    ensure!(
        x < room.attributes.width && y < room.attributes.height,
        "door outside floor"
    );
    let index = usize::from(y) * usize::from(room.attributes.width) + usize::from(x);
    ensure!(
        room.blocks[index] == 0x19,
        "authored security door preimage"
    );
    let script = format!("MnIceHQDoor{}", door.id);
    let done = format!("{script}Done");
    let locked = format!("{script}Locked");
    let restore = format!("{script}Restore");
    let end = format!("{script}End");
    let open = c(
        "changeblock",
        &[&(x * 2).to_string(), &(y * 2).to_string(), "7"],
    );
    for (label, lines) in [
        (
            format!("{script}LockedText"),
            vec!["A card reader.", "The shutter holds."],
        ),
        (
            format!("{script}DoneText"),
            vec!["The card works.", "The shutter opens."],
        ),
    ] {
        authoring::text(room, &label, &lines);
    }
    room.scripts.insert(
        script.clone(),
        json!([
            c("checkevent", &[&door.flag]),
            c("iftrue", &[&end]),
            c("checkevent", &[KEY]),
            c("iffalse", &[&locked]),
            c("checkitem", &["CARD_KEY"]),
            c("iffalse", &[&locked]),
            c("setevent", &[&door.flag]),
            open.clone(),
            c("refreshmap", &[]),
            c("jumptext", &[&format!("{script}DoneText")])
        ]),
    );
    room.scripts.insert(
        locked,
        json!([c("jumptext", &[&format!("{script}LockedText")])]),
    );
    room.scripts.insert(end.clone(), json!([c("end", &[])]));
    room.scripts.insert(
        restore.clone(),
        json!([
            c("checkevent", &[&door.flag]),
            c("iffalse", &[&end]),
            open,
            c("end", &[])
        ]),
    );
    room.scripts.insert(
        done.clone(),
        json!([c("scall", &[&restore]), c("endcallback", &[])]),
    );
    a::callback(room, "MAPCALLBACK_TILES", &done);
    for dx in 0..2 {
        for dy in 0..2 {
            a::read(room, [x * 2 + dx, y * 2 + dy], &script);
        }
    }
    Ok(())
}
