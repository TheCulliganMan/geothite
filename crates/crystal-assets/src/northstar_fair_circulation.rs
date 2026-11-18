//! Daily visitor groups and the State Fair's east-gate-to-grandstand avenue.
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::json;

use crate::authored_town_geometry;
use crate::authored_town_publication;
use crate::grand_marais_modpack::authoring;
use crate::northstar_cave_encounters;
use crate::{CompiledGamePack, MapModule};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fair {
    avenue: Vec<[u16; 2]>,
    shifts: Vec<Shift>,
    fountain: Fountain,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fountain {
    script: String,
    item: String,
    tile: [u16; 2],
    approach: [u16; 2],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Shift {
    scene: String,
    hours: [i16; 2],
    guests: Vec<u16>,
}

fn schedule(room: &mut MapModule, number: u16, [first, last]: [i16; 2]) -> Result<()> {
    let id = format!("MnFairMazeGuest{number:03}");
    let text = format!("{id}Text");
    ensure!(
        room.scripts.get(&id) == Some(&json!([{"command":"jumptextfaceplayer","args":[&text]}]))
            && room.script_text_bodies.contains_key(&text),
        "{id} must only provide its fair conversation"
    );
    let guest = room
        .objects
        .iter_mut()
        .find(|a| a.script == id)
        .context("missing fair visitor")?;
    ensure!(
        guest.object_type == "OBJECTTYPE_SCRIPT"
            && guest.object_identifier.as_deref() == Some(id.as_str())
            && guest.event_flag == "-1"
            && [guest.hram_x, guest.hram_y] == [-1, -1],
        "unexpected actor or previous schedule for {id}"
    );
    let event = room
        .map_event_section_commands
        .iter()
        .find(|r| r.command == "object_event" && r.args.get(11) == Some(&id))
        .context("missing fair object event")?;
    ensure!(
        event.args.len() == 13
            && event.args[..2] == [guest.x.to_string(), guest.y.to_string()]
            && event.args[6..8] == ["-1", "-1"],
        "unexpected map event or schedule for {id}"
    );
    [guest.hram_x, guest.hram_y] = [first, last];
    Ok(())
}

fn pave(room: &mut MapModule, corners: &[[u16; 2]]) -> Result<()> {
    ensure!(
        corners.first() == Some(&[47, 32]) && corners.last() == Some(&[5, 9]),
        "the fair avenue must join the Route 5 gate to the grandstand"
    );
    let mut blocks = BTreeSet::new();
    for turns in corners.windows(2) {
        let [ax, ay] = turns[0];
        let [bx, by] = turns[1];
        ensure!(
            (ax == bx) != (ay == by),
            "fair avenue must use straight runs"
        );
        for y in ay.min(by)..=ay.max(by) {
            for x in ax.min(bx)..=ax.max(bx) {
                ensure!(
                    x < room.attributes.width && y < room.attributes.height,
                    "fair avenue bounds"
                );
                blocks.insert(usize::from(y) * usize::from(room.attributes.width) + usize::from(x));
            }
        }
    }
    for i in blocks {
        ensure!(
            matches!(room.blocks[i], 0x01 | 0x02),
            "fair avenue crosses an existing feature"
        );
        room.blocks[i] = 0x71;
    }
    Ok(())
}

fn install_fountain(
    pack: &mut CompiledGamePack,
    room: &mut MapModule,
    fountain: &Fountain,
) -> Result<()> {
    let [x, y] = fountain.tile;
    ensure!(
        x.is_multiple_of(2)
            && y.is_multiple_of(2)
            && authored_town_geometry::kind(pack, room, fountain.tile)? == "WALL"
            && u32::from(x.abs_diff(fountain.approach[0]))
                + u32::from(y.abs_diff(fountain.approach[1]))
                == 1,
        "fair fountain must replace the top left end of a bench"
    );
    let index = usize::from(y / 2) * usize::from(room.attributes.width) + usize::from(x / 2);
    ensure!(
        room.blocks[index] == 0x86 && !room.blocks.contains(&0x87),
        "expected an unused drinking fountain and the original bench"
    );
    let collision = &pack
        .data
        .tilesets
        .get(&room.attributes.tileset_name)
        .context("native fair tiles")?
        .collision;
    ensure!(
        collision
            .get("86")
            .is_some_and(|c| c == &["WALL", "WALL", "FLOOR", "FLOOR"])
            && collision
                .get("87")
                .is_some_and(|c| c == &["WALL", "FLOOR", "FLOOR", "FLOOR"]),
        "fair fountain clearance"
    );
    room.blocks[index] = 0x87;
    // Keeping every scheduled person present is the most crowded possible case.
    let reachable = authored_town_geometry::reachable(pack, room, [95, 64], false)?;
    ensure!(
        reachable.contains(&fountain.approach)
            && !room.objects.iter().any(|o| [o.x, o.y] == fountain.tile)
            && !room.scripts.contains_key(&fountain.script),
        "fair reward access"
    );
    northstar_cave_encounters::add_hidden_item(
        pack,
        room,
        &fountain.script,
        &fountain.item,
        fountain.tile,
    )?;
    room.script_field_pickups =
        crate::parse_script_field_pickups(&room.id, &room.scripts, &room.objects)?;
    let flag = format!("EVENT_{}", fountain.script);
    ensure!(
        room.script_field_pickups
            .iter()
            .filter(|p| p.source_script == fountain.script
                && p.command == "hiddenitem"
                && p.item_id.as_ref() == Some(&fountain.item)
                && p.event_flag.as_ref() == Some(&flag)
                && p.quantity == 1)
            .count()
            == 1,
        "fair hidden item and Itemfinder index"
    );
    Ok(())
}

pub(super) fn install(pack: &mut CompiledGamePack, directory: &Path) -> Result<()> {
    let design: Fair = serde_json::from_slice(&fs::read(directory.join("fair.json"))?)?;
    let mut room = pack
        .data
        .maps
        .get("MnStateFairgrounds")
        .context("State Fairgrounds")?
        .clone();
    let ground = &pack
        .data
        .tilesets
        .get(&room.attributes.tileset_name)
        .context("native fair tiles")?
        .collision;
    for block in ["01", "02", "71"] {
        ensure!(
            ground
                .get(block)
                .is_some_and(|c| c == &["FLOOR", "FLOOR", "FLOOR", "FLOOR"]),
            "fair avenue collision {block}"
        );
    }
    pave(&mut room, &design.avenue)?;
    let mut scheduled = BTreeSet::new();
    for shift in design.shifts {
        let [first, last] = shift.hours;
        ensure!(
            0 <= first && first < last && last < 24 && !shift.guests.is_empty(),
            "invalid fair hours for {}",
            shift.scene
        );
        for number in shift.guests {
            ensure!(scheduled.insert(number), "duplicate fair visitor {number}");
            schedule(&mut room, number, shift.hours)
                .with_context(|| format!("fair group {}", shift.scene))?;
        }
    }
    install_fountain(pack, &mut room, &design.fountain)?;
    authoring::refresh_event_section(&mut room);
    authored_town_publication::replace(pack, room)
}
