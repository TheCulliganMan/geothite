//! Connect the exposed faces of Minneapolis and St. Cloud's paired control cabinets.
use anyhow::{Context, Result, ensure};

use crate::authored_town_geometry::kind;
use crate::grand_marais_modpack::authoring::{command, refresh_event_section};
use crate::northstar_lair_authoring::read;
use crate::{CompiledGamePack, MapModule, authored_dialogue, authored_town_publication};

const MINNEAPOLIS: &str = "MnMinneapolisGym";
const ST_CLOUD: &str = "MnStCloudGym";

/// Extends the existing feeder and fan controls to their reachable switch faces.
///
/// Returns an error if a machine, its control program, or the guide's footprint has changed.
pub(super) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack
        .data
        .maps
        .get(MINNEAPOLIS)
        .context(MINNEAPOLIS)?
        .clone();
    ensure!(
        room.attributes.tileset_name == "mn_gym_3",
        "unexpected feeder tileset"
    );
    extend_cabinet(
        pack,
        &mut room,
        "WEST_FEEDER",
        [5, 14],
        0x0b,
        &[[4, 14], [4, 15], [5, 15]],
    )?;
    extend_cabinet(
        pack,
        &mut room,
        "EAST_FEEDER",
        [8, 7],
        0x0b,
        &[[9, 6], [9, 7]],
    )?;

    let after = "MN_GYM_3_TRAINER_1After";
    let reply = "MnGymWalkLenviBreakerText";
    bind_reply(
        &mut room,
        after,
        1,
        "jumptext",
        &format!("{after}Text"),
        reply,
    )?;
    add_reply(pack, &mut room, reply)?;
    for (feeder, reply) in [
        ("WEST", "MnGymWalkWestFeederLockedText"),
        ("EAST", "MnGymWalkEastFeederLockedText"),
    ] {
        for state in ["ON", "OFF"] {
            let gate = format!("{MINNEAPOLIS}{feeder}_FEEDERRouting{state}Denied");
            bind_reply(
                &mut room,
                &gate,
                0,
                "writetext",
                &format!("{gate}Text"),
                reply,
            )?;
        }
        add_reply(pack, &mut room, reply)?;
    }
    room.script_text_commands = crate::parse_script_text_commands(MINNEAPOLIS, &room.scripts)?;
    room.script_text_bodies = crate::parse_script_text_bodies(MINNEAPOLIS, &room.scripts)?;
    refresh_event_section(&mut room);
    authored_town_publication::replace(pack, room)?;

    let mut room = pack.data.maps.get(ST_CLOUD).context(ST_CLOUD)?.clone();
    ensure!(
        room.attributes.tileset_name == "mn_gym_5",
        "unexpected fan tileset"
    );
    for (bank, original, second) in [
        ("LOWER_WEST_BANK", [6, 13], [7, 13]),
        ("LOWER_EAST_BANK", [13, 13], [12, 13]),
        ("UPPER_WEST_BANK", [6, 5], [7, 5]),
        ("UPPER_EAST_BANK", [14, 3], [15, 3]),
    ] {
        extend_cabinet(pack, &mut room, bank, original, 0x04, &[second])?;
    }

    // The guide previously filled the only floor square facing the right-hand desk.
    ensure!(
        kind(pack, &room, [5, 14])? == "FLOOR"
            && room.objects.iter().all(|o| (o.x, o.y) != (5, 14)),
        "occupied airworks foyer"
    );
    let guide = room
        .objects
        .iter_mut()
        .find(|o| o.script == "MnStCloudGymGuide")
        .context("airworks guide")?;
    ensure!(
        (guide.x, guide.y) == (7, 14),
        "unexpected airworks guide position"
    );
    guide.x = 5;
    refresh_event_section(&mut room);
    authored_town_publication::replace(pack, room)
}

fn extend_cabinet(
    pack: &CompiledGamePack,
    room: &mut MapModule,
    name: &str,
    original: [u16; 2],
    block: u16,
    faces: &[[u16; 2]],
) -> Result<()> {
    let label = format!("{}{name}Routing", room.id);
    let [x, y] = original;
    let index = usize::from(y / 2) * usize::from(room.attributes.width) + usize::from(x / 2);
    ensure!(
        room.blocks.get(index) == Some(&block)
            && room.scripts.contains_key(&label)
            && room.events.bg_events.iter().any(|e| [e.x, e.y] == original
                && e.event_type == "BGEVENT_READ"
                && e.script == label),
        "missing original switch {label}"
    );
    for &at in faces {
        ensure!(
            at[0] / 2 == x / 2
                && at[1] / 2 == y / 2
                && kind(pack, room, at)? == "WALL"
                && room.events.bg_events.iter().all(|e| [e.x, e.y] != at),
            "unexpected switch face {at:?} in {label}"
        );
        read(room, at, &label);
    }
    Ok(())
}

fn bind_reply(
    room: &mut MapModule,
    script: &str,
    index: usize,
    op: &str,
    old: &str,
    new: &str,
) -> Result<()> {
    let row = room
        .scripts
        .get_mut(script)
        .and_then(|s| s.get_mut(index))
        .context("feeder reply")?;
    ensure!(*row == command(op, &[old]), "changed feeder reply {script}");
    *row = command(op, &[new]);
    Ok(())
}

fn add_reply(pack: &CompiledGamePack, room: &mut MapModule, label: &str) -> Result<()> {
    ensure!(
        !room.scripts.contains_key(label) && !pack.data.map_scripts.contains_key(label),
        "occupied feeder reply {label}"
    );
    authored_dialogue::install_canonical(room, label)
}
