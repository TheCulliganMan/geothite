//! Ely's registrar follows the survey report and Casey waits for an answer.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring::command as c;
use crate::{CompiledGamePack, authored_town_publication};

#[cfg(test)]
#[path = "northstar_league_return_tests.rs"]
mod tests;

const LOBBY: &str = "MnLeagueLobby";
const CASEY: &str = "MnLeagueChampion";

// The most advanced completed step determines who can act next.
const REPORTS: [(&str, &str); 7] = [
    ("EVENT_MN_ARC_LINK_SENT", "MnElyGoToIntake"),
    ("EVENT_MN_SURVEY_CHIEF_DEFEATED", "MnElyReadPumpTimes"),
    ("EVENT_MN_LIGHTHOUSE_SIGNAL", "MnElyFindShelter"),
    ("EVENT_MN_DEPOT_CLEARED", "MnElyCallHarbor"),
    ("EVENT_MN_MINE_INSTRUMENT_RECOVERED", "MnElyBringMeter"),
    ("EVENT_MN_HEADWATERS_REPORT", "MnElyLookForFinn"),
    ("EVENT_MN_HEADWATERS_READING", "MnElyTakeReading"),
];

#[derive(Deserialize)]
struct Dialogue {
    league_return_dialogue: BTreeMap<String, BTreeMap<String, Vec<AuthoredDialoguePage>>>,
}

pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let story: Dialogue =
        serde_json::from_str(include_str!("../../../planning/minnesota/story.json"))?;
    ensure!(
        story.league_return_dialogue.len() == 2,
        "Ely return dialogue locations"
    );
    for name in [LOBBY, CASEY] {
        let mut room = pack.data.maps.get(name).context("Ely League room")?.clone();
        for (label, pages) in story
            .league_return_dialogue
            .get(name)
            .context("Ely return pages")?
        {
            ensure!(
                room.scripts
                    .insert(label.clone(), authored_dialogue::compile(pages)?)
                    .is_none(),
                "occupied League text {label}"
            );
        }
        if name == LOBBY {
            let denied = room
                .scripts
                .get("MnElySurveyDenied")
                .context("survey handoff")?;
            ensure!(
                denied.get(1) == Some(&c("writetext", &["MnElySurveyDeniedText"])),
                "unexpected registrar handoff"
            );
            let mut handoff = Vec::new();
            for (flag, target) in REPORTS {
                ensure!(
                    pack.data
                        .story_event_script_constants
                        .global
                        .contains_key(flag),
                    "missing survey flag {flag}"
                );
                handoff.push(c("checkevent", &[flag]));
                handoff.push(c("iftrue", &[target]));
                ensure!(
                    room.scripts.contains_key(&format!("{target}Text")),
                    "missing {target} advice"
                );
                ensure!(
                    room.scripts
                        .insert(
                            target.into(),
                            json!([c("jumptext", &[&format!("{target}Text")])])
                        )
                        .is_none(),
                    "occupied registrar branch {target}"
                );
            }
            handoff.push(c("jumptext", &["MnElyLookAtItascaText"]));
            room.scripts
                .insert("MnElySurveyDenied".into(), Value::Array(handoff));
            let decline = room
                .scripts
                .get_mut("MnElyRegistrationDeclined")
                .and_then(Value::as_array_mut)
                .context("registration refusal")?;
            ensure!(
                decline.get(1) == Some(&c("writetext", &["MnElyRegistrationDeclinedText"])),
                "unexpected preparation reply"
            );
            decline[1] = c("writetext", &["MnElyPrepareOutsideText"]);
        } else {
            let label = "MnLeagueChampionRematchIntro";
            ensure!(
                room.scripts.get(label)
                    == Some(&json!([
                        c("opentext", &[]),
                        c("writetext", &["MnLeagueChampionRematchText"]),
                        c("promptbutton", &[]),
                        c("closetext", &[]),
                        c("sjump", &["MnLeagueChampionBattleStart"])
                    ])),
                "unexpected Champion invitation"
            );
            room.scripts.insert(
                label.into(),
                json!([
                    c("opentext", &[]),
                    c("writetext", &["MnLeagueChampionRematchText"]),
                    c("yesorno", &[]),
                    c("iffalse", &["MnCaseyWaits"]),
                    c("closetext", &[]),
                    c("sjump", &["MnLeagueChampionBattleStart"])
                ]),
            );
            ensure!(
                room.scripts
                    .insert(
                        "MnCaseyWaits".into(),
                        json!([
                            c("writetext", &["MnCaseyTakeABreathText"]),
                            c("waitbutton", &[]),
                            c("closetext", &[]),
                            c("end", &[])
                        ])
                    )
                    .is_none(),
                "occupied Champion reply"
            );
        }
        room.script_flag_commands = crate::parse_script_flag_commands(name, &room.scripts)?;
        room.script_control_commands = crate::parse_script_control_commands(name, &room.scripts)?;
        room.script_text_commands = crate::parse_script_text_commands(name, &room.scripts)?;
        room.script_text_bodies = crate::parse_script_text_bodies(name, &room.scripts)?;
        room.script_runtime_commands = crate::parse_script_runtime_commands(name, &room.scripts)?;
        authored_town_publication::replace(pack, room)?;
    }
    Ok(())
}
