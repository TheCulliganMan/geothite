//! The prairie-forest observations' native, retryable ranger hand-in.
use anyhow::{Result, ensure};
use serde_json::{Value, json};

use crate::MapModule;
use crate::authored_dialogue;
use crate::grand_marais_modpack::authoring;

pub(crate) const COMPLETED: &str = "EVENT_MN_LAKE_OBSERVATIONS_REWARDED";

pub(crate) fn install(map: &mut MapModule, quest: &Value) -> Result<()> {
    let requires: Vec<String> = serde_json::from_value(quest["requires"].clone())?;
    ensure!(
        quest["id"] == "SQ01"
            && quest["room"] == map.id
            && quest["actor"] == "MN_DETROIT_LAKE_RANGER"
            && quest["reward_item"] == "MAX_REPEL"
            && quest["reward_quantity"] == 1
            && quest["once_flag"] == COMPLETED
            && requires
                == [
                    "BADGE_6",
                    "EVENT_MN_LAKE_OBSERVATION_01",
                    "EVENT_MN_LAKE_OBSERVATION_02",
                    "EVENT_MN_LAKE_OBSERVATION_03",
                ],
        "retain the exact SQ01 hand-in, badge, notes and one-time reward"
    );
    ensure!(
        map.objects
            .iter()
            .filter(|o| o.script == "MN_DETROIT_LAKE_RANGER")
            .count()
            == 1,
        "one real ranger must receive the observations"
    );
    let c = authoring::command;
    let mut commands = vec![
        c("faceplayer", &[]),
        c("opentext", &[]),
        c("checkevent", &[COMPLETED]),
        c("iftrue", &["MnDetroitNotesAfter"]),
        c("checkflag", &["ENGINE_MINERALBADGE"]),
        c("iffalse", &["MnDetroitNotesBadge"]),
    ];
    for flag in &requires[1..] {
        commands.extend([
            c("checkevent", &[flag]),
            c("iffalse", &["MnDetroitNotesMissing"]),
        ]);
    }
    commands.extend([
        c("writetext", &["MnDetroitNotesReviewText"]),
        c("waitbutton", &[]),
        c("giveitem", &["MAX_REPEL", "1"]),
        c("iffalse", &["MnDetroitNotesFull"]),
        c("setevent", &[COMPLETED]),
        c("writetext", &["MnDetroitNotesRewardText"]),
        c("waitbutton", &[]),
        c("closetext", &[]),
        c("end", &[]),
    ]);
    map.scripts
        .insert("MN_DETROIT_LAKE_RANGER".into(), json!(commands));
    for suffix in ["Badge", "Missing", "After", "Full"] {
        let script = format!("MnDetroitNotes{suffix}");
        let text = format!("{script}Text");
        authored_dialogue::install_canonical(map, &text)?;
        map.scripts.insert(
            script,
            json!([
                c("writetext", &[&text]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("end", &[])
            ]),
        );
    }
    for label in ["MnDetroitNotesReviewText", "MnDetroitNotesRewardText"] {
        authored_dialogue::install_canonical(map, label)?;
    }
    Ok(())
}
