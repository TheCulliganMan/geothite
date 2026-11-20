//! Halden lets a first-time challenger prepare between the third and fourth fights.
use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::grand_marais_modpack::authoring::command;
use crate::northstar_rival_arc::put;
use crate::{CompiledGamePack, authored_dialogue, authored_town_publication, gym_scripts};

const ROOM: &str = "MnLeagueRoom4";
const CHALLENGE: &str = "MnLeagueRoom4Challenge";
const BATTLE: &str = "MnLeagueRoom4BattleStart";
const READY: &str = "MnHaldenReady";
const WAIT: &str = "MnHaldenWaits";
const GAVE_TIME: &str = "EVENT_MN_HALDEN_GAVE_TIME";

pub(super) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack.data.maps.get(ROOM).context("Halden's room")?.clone();
    for label in [READY, WAIT] {
        ensure!(
            !pack.data.map_scripts.contains_key(label) && !room.scripts.contains_key(label),
            "occupied Halden script {label}"
        );
    }
    ensure!(
        !pack
            .data
            .story_event_script_constants
            .global
            .contains_key(GAVE_TIME),
        "occupied Halden memory flag"
    );
    let program = room
        .scripts
        .get_mut(CHALLENGE)
        .and_then(Value::as_array_mut)
        .context("Halden's first challenge")?;
    let previous = [
        command("opentext", &[]),
        command("writetext", &["MnLeagueRoom4BeforeText"]),
        command("promptbutton", &[]),
        command("closetext", &[]),
        command("sjump", &[BATTLE]),
    ];
    ensure!(program.ends_with(&previous), "unexpected Halden invitation");
    program.truncate(program.len() - previous.len());
    program.extend([
        command("opentext", &[]),
        command("checkevent", &[GAVE_TIME]),
        command("iftrue", &[READY]),
        command("writetext", &["MnLeagueRoom4BeforeText"]),
        command("promptbutton", &[]),
        command("sjump", &[READY]),
    ]);
    put(
        &mut room,
        READY,
        vec![
            command("writetext", &["MnHaldenReadyText"]),
            command("yesorno", &[]),
            command("iffalse", &[WAIT]),
            command("closetext", &[]),
            command("sjump", &[BATTLE]),
        ],
    );
    put(
        &mut room,
        WAIT,
        vec![
            command("setevent", &[GAVE_TIME]),
            command("writetext", &["MnHaldenWaitText"]),
            command("waitbutton", &[]),
            command("closetext", &[]),
            command("end", &[]),
        ],
    );
    for label in ["MnHaldenReadyText", "MnHaldenWaitText"] {
        authored_dialogue::install_canonical(&mut room, label)?;
    }
    room.script_control_commands = crate::parse_script_control_commands(ROOM, &room.scripts)?;
    room.script_flag_commands = crate::parse_script_flag_commands(ROOM, &room.scripts)?;
    room.script_text_commands = crate::parse_script_text_commands(ROOM, &room.scripts)?;
    room.script_text_bodies = crate::parse_script_text_bodies(ROOM, &room.scripts)?;
    gym_scripts::event(pack, GAVE_TIME)?;
    authored_town_publication::replace(pack, room)
}
