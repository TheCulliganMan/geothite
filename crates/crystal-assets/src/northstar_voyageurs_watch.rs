//! Give skiers directions for their current leg of the Voyageurs border trip.
use anyhow::{Context, Result, ensure};
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use serde_json::Value;

use crate::authored_dialogue::install_canonical;
use crate::authored_town_publication;
use crate::grand_marais_modpack::authoring::command;
use crate::international_falls_story::{ARRIVED, GIVEN};
use crate::northstar_rival_arc::put;
use crate::{CompiledGamePack, MapModule};

pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let clinic = "MnInternationalFallsPokecenter1F";
    let mut room = pack.data.maps.get(clinic).context("Falls clinic")?.clone();
    let register = room
        .scripts
        .get_mut("MnFallsRegister")
        .and_then(Value::as_array_mut)
        .context("Falls registration")?;
    ensure!(
        register.ends_with(&[
            command("writetext", &["MnFallsregistered_linesText"]),
            command("promptbutton", &[]),
            command("closetext", &[]),
            command("end", &[]),
        ]),
        "expected the clinic's original registration handoff"
    );
    register.truncate(register.len() - 2);
    register.push(command("sjump", &["MnFallsWaiting"]));

    // Use the same eight regional badge flags as the actual icefield checkpoint.
    let mut waiting = Vec::new();
    for badge in &BADGE_ENGINE_FLAGS[..8] {
        waiting.push(command("checkflag", &[badge]));
        waiting.push(command("iffalse", &["MnFallsWatchBadges"]));
    }
    waiting.push(command("checkevent", &["EVENT_MN_LIGHTHOUSE_SIGNAL"]));
    waiting.push(command("iffalse", &["MnFallsWatchHarbor"]));
    let ready = room
        .scripts
        .get("MnFallsWaiting")
        .and_then(Value::as_array)
        .context("Falls waiting room")?;
    ensure!(
        ready.first() == Some(&command("writetext", &["MnFallswaiting_linesText"])),
        "expected the clinic's outbound reminder"
    );
    waiting.extend_from_slice(ready);
    put(&mut room, "MnFallsWaiting", waiting);
    for (label, text) in [
        ("MnFallsWatchBadges", "MnFallsWatchBadgesText"),
        ("MnFallsWatchHarbor", "MnFallsWatchHarborText"),
    ] {
        ensure!(
            !room.scripts.contains_key(label),
            "occupied clinic branch {label}"
        );
        put(
            &mut room,
            label,
            vec![
                command("writetext", &[text]),
                command("waitbutton", &[]),
                command("closetext", &[]),
                command("end", &[]),
            ],
        );
    }
    for label in [
        "MnFallsregister_linesText",
        "MnFallsregistered_linesText",
        "MnFallswaiting_linesText",
        "MnFallsWatchBadgesText",
        "MnFallsWatchHarborText",
    ] {
        install_canonical(&mut room, label)?;
    }
    publish(pack, room)?;

    let mut town = pack
        .data
        .maps
        .get("MnInternationalFalls")
        .context("Falls trailhead")?
        .clone();
    install_canonical(&mut town, "MN_FALLS_TRAIL_NOTICEText")?;
    publish(pack, town)?;

    let mut camp = pack
        .data
        .maps
        .get("MnBoundaryWaters")
        .context("Voyageurs west watch")?
        .clone();
    let ranger = "MN_BOUNDARY_BORDER_RANGER";
    let original = command("sjump", &["MnWayNorthWatch"]);
    ensure!(
        camp.scripts
            .get(ranger)
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            == Some(std::slice::from_ref(&original)),
        "expected the camp ranger's regional directions"
    );
    // Leave the shore and League advice available after the radio acknowledgement.
    put(
        &mut camp,
        ranger,
        vec![
            command("checkevent", &[ARRIVED]),
            command("iffalse", &["MnWayNorthWatch"]),
            command("checkevent", &[GIVEN]),
            command("iftrue", &["MnWayNorthWatch"]),
            command("faceplayer", &[]),
            command("opentext", &[]),
            command("writetext", &["MnFallsWatchArrivedText"]),
            command("waitbutton", &[]),
            command("closetext", &[]),
            command("checkevent", &["EVENT_MN_SURVEY_RESTORED"]),
            command("iffalse", &["MnFallsWatchEnd"]),
            command("checkevent", &["EVENT_MN_LEAGUE_CHAMPION"]),
            command("iftrue", &["MnFallsWatchEnd"]),
            command("sjump", &["MnWayLeagueTrail"]),
        ],
    );
    ensure!(
        !camp.scripts.contains_key("MnFallsWatchEnd"),
        "occupied west watch exit"
    );
    put(&mut camp, "MnFallsWatchEnd", vec![command("end", &[])]);
    install_canonical(&mut camp, "MnFallsWatchArrivedText")?;
    publish(pack, camp)
}

fn publish(pack: &mut CompiledGamePack, mut room: MapModule) -> Result<()> {
    room.script_object_commands = crate::parse_script_object_commands(&room.id, &room.scripts)?;
    room.script_flag_commands = crate::parse_script_flag_commands(&room.id, &room.scripts)?;
    room.script_control_commands = crate::parse_script_control_commands(&room.id, &room.scripts)?;
    room.script_text_commands = crate::parse_script_text_commands(&room.id, &room.scripts)?;
    room.script_text_bodies = crate::parse_script_text_bodies(&room.id, &room.scripts)?;
    authored_town_publication::replace(pack, room)
}
