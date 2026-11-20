//! Offer a free party rest at the Mill Workers Inn's existing desk.
use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use serde_json::json;

use crate::authored_dialogue;
use crate::grand_marais_modpack::authoring::{command, refresh_event_section};
use crate::northstar_rival_arc::put;
use crate::{CompiledGamePack, authored_town_publication};

const ROOM: &str = "MnMinneapolisRiverInn";
const HOST: &str = "MN_MP_RIVERINN_HOST";
const OFFER: &str = "MN_MP_RIVERINN_HOSTText";
const RESTED: &str = "MnMillInnRestedText";
const REST: &str = "MnMillInnOfferRest";
const TILESET: &str = "mn_mill_inn";
const LEAVE: &str = "MnMillInnLeave";

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack
        .data
        .maps
        .get(ROOM)
        .context("Mill Workers Inn")?
        .clone();
    ensure!(
        room.objects
            .iter()
            .any(|actor| actor.script == HOST && (actor.x, actor.y) == (2, 3))
            && room.scripts.get(HOST) == Some(&json!([command("jumptextfaceplayer", &[OFFER])]))
            && room.attributes.tileset_name == "traditional_house"
            && room.attributes.width == 4
            && room.attributes.height == 4
            && room.blocks.get(7) == Some(&0x04)
            && room.events.bg_events.is_empty()
            && !room.scripts.contains_key(REST)
            && !room.scripts.contains_key(LEAVE)
            && !room.scripts.contains_key(RESTED),
        "expected the original mill inn desk"
    );
    room.blocks[7] = bedside(pack)?;
    room.attributes.tileset_name = TILESET.into();
    for y in [2, 3] {
        room.events.bg_events.push(BackgroundEvent {
            x: 7,
            y,
            event_type: "BGEVENT_READ".into(),
            script: REST.into(),
        });
    }
    refresh_event_section(&mut room);
    authored_dialogue::install_canonical(&mut room, OFFER)?;
    authored_dialogue::install_canonical(&mut room, RESTED)?;
    put(
        &mut room,
        HOST,
        vec![command("faceplayer", &[]), command("sjump", &[REST])],
    );
    put(
        &mut room,
        REST,
        vec![
            command("opentext", &[]),
            command("writetext", &[OFFER]),
            command("yesorno", &[]),
            command("iffalse", &[LEAVE]),
            command("special", &["HealParty"]),
            command("writetext", &[RESTED]),
            command("waitbutton", &[]),
            command("closetext", &[]),
            command("end", &[]),
        ],
    );
    put(
        &mut room,
        LEAVE,
        vec![command("closetext", &[]), command("end", &[])],
    );
    room.script_object_commands = crate::parse_script_object_commands(ROOM, &room.scripts)?;
    room.script_control_commands = crate::parse_script_control_commands(ROOM, &room.scripts)?;
    room.script_text_commands = crate::parse_script_text_commands(ROOM, &room.scripts)?;
    room.script_text_bodies = crate::parse_script_text_bodies(ROOM, &room.scripts)?;
    room.script_runtime_commands = crate::parse_script_runtime_commands(ROOM, &room.scripts)?;
    authored_town_publication::replace(pack, room)
}

// Keep the inn's floor against the aisle and the native single bed at the wall.
fn bedside(pack: &mut CompiledGamePack) -> Result<u16> {
    let bed = crate::northstar_rival_art::import_block(
        pack,
        "traditional_house",
        TILESET,
        "players_house",
        0x22,
    )?;
    let blocks = pack
        .runtime_files
        .get_mut(&format!("data/tilesets/{TILESET}_metatiles.bin"))
        .context("native inn metatiles")?;
    let start = usize::from(bed) * 16;
    for row in [0, 4, 8, 12] {
        blocks.copy_within(start + row..start + row + 2, start + row + 2);
        blocks.copy_within(0x04 * 16 + row..0x04 * 16 + row + 2, start + row);
    }
    let tileset = pack
        .data
        .tilesets
        .get_mut(TILESET)
        .context("native inn tileset")?;
    let collision = tileset
        .collision
        .get_mut(&format!("{bed:02x}"))
        .context("native bed collision")?;
    ensure!(
        collision.as_slice() == ["WALL", "FLOOR", "WALL", "FLOOR"],
        "native single bed must occupy its left half"
    );
    collision.swap(0, 1);
    collision.swap(2, 3);
    pack.runtime_files.insert(
        format!("data/tilesets/{TILESET}.json"),
        serde_json::to_vec(&tileset.collision)?,
    );
    Ok(bed)
}
