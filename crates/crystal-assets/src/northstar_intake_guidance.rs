//! Give the intake crew directions for the first unfinished part of the rescue.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::CompiledGamePack;
use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::authored_town_publication;
use crate::grand_marais_modpack::authoring::command;
use crate::northstar_rival_arc::put;

const ROOM: &str = "MnBoundaryIntake";
const DENIED: &str = "MnVerdigrimDenied";
const REED: &str = "EVENT_MN_REED_BOUNDARY_DEFEATED";

#[derive(Deserialize)]
struct Story {
    intake_control_dialogue: BTreeMap<String, Vec<AuthoredDialoguePage>>,
}

/// Updates the common refusal and Finn's advice without changing the controls.
pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let previous = pack.data.maps.get(ROOM).context("Boundary intake")?;
    let mut room = previous.clone();
    ensure!(
        room.scripts
            .get(DENIED)
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            == Some([command("jumptext", &["MnVerdigrimDeniedText"])].as_slice()),
        "expected the intake's shared denial"
    );
    // All three controls use this routine only after a failed prerequisite.
    // Check the radio first: without that call, Reed won't accept a challenge.
    let mut denial = Vec::new();
    for (flag, branch, text) in [
        (
            "EVENT_MN_ARC_LINK_SENT",
            "MnIntakeNeedRadio",
            "MnIntakeRadioHintText",
        ),
        (
            "EVENT_MN_VERDIGRIM_COUNTER_CURRENT",
            "MnIntakeNeedLanding",
            "MnIntakeLandingHintText",
        ),
        (REED, "MnIntakeNeedReed", "MnVerdigrimDeniedText"),
        (
            "EVENT_MN_VERDIGRIM_DRILL_STOPPED",
            "MnIntakeNeedDrill",
            "MnIntakeDrillHintText",
        ),
    ] {
        ensure!(
            !room.scripts.contains_key(branch),
            "occupied intake branch {branch}"
        );
        denial.push(command("checkevent", &[flag]));
        denial.push(command("iffalse", &[branch]));
        put(&mut room, branch, vec![command("jumptext", &[text])]);
    }
    // Only the bypass can still be locked once Reed and the drill are clear.
    denial.push(command("jumptext", &["MnIntakeOutletHintText"]));
    put(&mut room, DENIED, denial);

    let finn = room
        .scripts
        .get_mut("MN_ARC_INTAKE_FINN")
        .and_then(Value::as_array_mut)
        .context("Finn's intake advice")?;
    ensure!(
        finn.last() == Some(&command("jumptext", &["MnArcIntakeFinnText"])),
        "expected Finn's operating instructions"
    );
    // Keep the previously installed dialogue positions. Only the last
    // instruction needs to branch for players who have yet to face Reed.
    *finn.last_mut().context("Finn's unfinished rescue")? = command("checkevent", &[REED]);
    finn.extend([
        command("iffalse", &[DENIED]),
        command("jumptext", &["MnArcIntakeFinnText"]),
    ]);

    authored_dialogue::install_canonical(&mut room, "MnVerdigrimDeniedText")?;
    let story: Story =
        serde_json::from_str(include_str!("../../../planning/minnesota/story.json"))?;
    for (label, pages) in story.intake_control_dialogue {
        ensure!(
            !room.scripts.contains_key(&label),
            "occupied intake text {label}"
        );
        room.scripts
            .insert(label, authored_dialogue::compile(&pages)?);
    }
    room.script_flag_commands = crate::parse_script_flag_commands(ROOM, &room.scripts)?;
    room.script_control_commands = crate::parse_script_control_commands(ROOM, &room.scripts)?;
    room.script_text_commands = crate::parse_script_text_commands(ROOM, &room.scripts)?;
    room.script_text_bodies = crate::parse_script_text_bodies(ROOM, &room.scripts)?;
    authored_town_publication::replace(pack, room)
}
