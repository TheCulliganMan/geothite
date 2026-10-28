//! The prairie-forest observations' native, retryable ranger hand-in.
use anyhow::{Result, ensure};
use serde_json::{Value, json};

use crate::MapModule;
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
    for (suffix, lines) in [
        (
            "Badge",
            vec![
                "Bring the MINERALBADGE and your three habitat notes. The trail north has the observation signs.",
            ],
        ),
        (
            "Missing",
            vec![
                "One note each from prairie, pine and wetland. Look along the PRAIRIE FOREST trail north of town. Read all three signs, then return.",
            ],
        ),
        (
            "After",
            vec![
                "The wetland keeps the lake clear. The pines hold its shore. Your prairie note belongs beside them, not in a separate drawer.",
            ],
        ),
        (
            "Full",
            vec![
                "Your PACK is full. I will keep your MAX REPEL here. Make room and come back; your fieldwork still counts.",
            ],
        ),
    ] {
        let script = format!("MnDetroitNotes{suffix}");
        let text = format!("{script}Text");
        authoring::text(map, &text, &lines);
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
    authoring::text(
        map,
        "MnDetroitNotesReviewText",
        &[
            "All three habitats. Good. My old notebook called the marsh empty ground. Your wetland note proves how much that missed.",
        ],
    );
    authoring::text(
        map,
        "MnDetroitNotesRewardText",
        &[
            "One MAX REPEL for your next survey. Leave a quiet stretch for the creatures you came to notice.",
        ],
    );
    Ok(())
}
