//! Keep Ivy, her aide and their advice consistent with the investigation.
use std::collections::BTreeMap;

use anyhow::{Context, Result};
use serde_json::json;

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring::command as c;
use crate::{CompiledGamePack, MapModule};

pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack
        .data
        .maps
        .get("MnRochesterPokecenter1F")
        .context("Rochester clinic")?
        .clone();
    apply_clinic_speakers(&mut room)?;
    for (script, sprite) in [
        ("MN_ROCHESTER_RECIPIENT", "SPRITE_TEACHER"),
        ("MN_RC_PATIENT", "SPRITE_SCIENTIST"),
    ] {
        room.objects
            .iter_mut()
            .find(|object| object.script == script)
            .context("clinic speaker")?
            .sprite = sprite.into();
    }
    apply_ivy_reminders(&mut room, None)?;
    let mut offer = vec![
        c("checkevent", &["EVENT_MN_REED_ROCHESTER_MET"]),
        c("iftrue", &["MnArcRCChoice"]),
        c("setevent", &["EVENT_MN_REED_ROCHESTER_MET"]),
        c("faceobject", &["MN_ARC_RC_REED", "PLAYER"]),
        c("faceobject", &["PLAYER", "MN_ARC_RC_REED"]),
    ];
    offer.extend(crate::northstar_lair_authoring::say("MnArcRCPathsText"));
    offer.push(c("sjump", &["MnArcRCChoice"]));
    crate::northstar_rival_arc::put(&mut room, "MnArcRCOffer", offer);
    room.events
        .coord_events
        .retain(|event| !(event.y == 12 && (6..=11).contains(&event.x)));
    for x in 8..=11 {
        let entry = format!("MnArcRCEntryX{x}");
        let approach = format!("MnArcRCApproachX{x}");
        // (8, 11) is a wall. Approach that lane from its clear eastern side.
        let mut steps: Vec<(&str, usize)> = Vec::new();
        if x == 8 {
            steps.push(("DOWN", 2));
        } else {
            if x > 9 {
                steps.push(("RIGHT", usize::from(x - 9)));
            }
            steps.push(("DOWN", 1));
        }
        crate::northstar_rival_arc::movement(&mut room, &approach, &steps);
        crate::northstar_rival_arc::put(
            &mut room,
            &entry,
            vec![
                c("checkevent", &["EVENT_MN_REED_ROCHESTER_MET"]),
                c("iftrue", &["MnArcRCEnd"]),
                c("checkevent", &["EVENT_MN_DELIVERED_ROCHESTER_PARCEL"]),
                c("iffalse", &["MnArcRCEnd"]),
                c("applymovement", &["MN_ARC_RC_REED", &approach]),
                c(
                    "turnobject",
                    &["MN_ARC_RC_REED", if x == 8 { "LEFT" } else { "DOWN" }],
                ),
                c(
                    "turnobject",
                    &["PLAYER", if x == 8 { "RIGHT" } else { "UP" }],
                ),
                c("sjump", &["MnArcRCOffer"]),
            ],
        );
        room.events
            .coord_events
            .push(crystal_core::map::CoordEvent {
                x,
                y: 12,
                scene_id: "-1".into(),
                script_name: entry,
            });
    }
    crate::northstar_story_revision::publish(pack, room)
}

fn apply_clinic_speakers(room: &mut MapModule) -> Result<()> {
    // Ivy leaves for the intake after the harbor crew is called. Her aide
    // remains beside the displaced Pokemon and carries the return update.
    for (script, flag) in [
        ("MN_ROCHESTER_RECIPIENT", "EVENT_MN_LIGHTHOUSE_SIGNAL"),
        ("MN_RC_PATIENT", "-1"),
    ] {
        room.objects
            .iter_mut()
            .find(|object| object.script == script)
            .context("clinic speaker")?
            .event_flag = flag.into();
    }
    room.scripts.insert(
        "MN_RC_PATIENT".into(),
        json!([
            c("faceplayer", &[]),
            c("checkevent", &["EVENT_MN_SURVEY_RESTORED"]),
            c("iftrue", &["MnOverhaulAideAfter"]),
            c("checkevent", &["EVENT_MN_LIGHTHOUSE_SIGNAL"]),
            c("iftrue", &["MnOverhaulAideAway"]),
            c("jumptext", &["MnArcHabitatText"]),
        ]),
    );
    for (root, text) in [
        ("MnOverhaulAideAfter", "MnArcHabitatAfterText"),
        ("MnOverhaulAideAway", "MnOverhaulClinicAideAwayText"),
    ] {
        room.scripts
            .insert(root.into(), json!([c("jumptext", &[text])]));
    }
    room.scripts.insert(
        "MN_ARC_GILLPUP".into(),
        json!([
            c("cry", &["GILLPUP"]),
            c("waitsfx", &[]),
            c("jumptext", &["MnOverhaulGillpupText"]),
        ]),
    );
    Ok(())
}

pub(crate) fn install_ivy_reminders(
    pack: &mut CompiledGamePack,
    texts: &BTreeMap<String, Vec<AuthoredDialoguePage>>,
) -> Result<()> {
    let mut room = pack
        .data
        .maps
        .get("MnRochesterPokecenter1F")
        .context("Rochester clinic")?
        .clone();
    let mut changed = apply_ivy_reminders(&mut room, Some(texts))?;
    if let (Some(away), Some(gillpup)) = (
        texts.get("MnOverhaulClinicAideAwayText"),
        texts.get("MnOverhaulGillpupText"),
    ) {
        apply_clinic_speakers(&mut room)?;
        for (label, pages) in [
            ("MnOverhaulClinicAideAwayText", away),
            ("MnOverhaulGillpupText", gillpup),
        ] {
            room.scripts
                .insert(label.into(), authored_dialogue::compile(pages)?);
        }
        changed = true;
    }
    if changed {
        crate::northstar_story_revision::publish(pack, room)?;
    }
    Ok(())
}

fn apply_ivy_reminders(
    room: &mut MapModule,
    texts: Option<&BTreeMap<String, Vec<AuthoredDialoguePage>>>,
) -> Result<bool> {
    let stages = [
        (
            "EVENT_MN_DEPOT_CLEARED",
            "MnOverhaulIvyHarbor",
            "MnOverhaulIvyHarborText",
        ),
        (
            "EVENT_MN_MINE_INSTRUMENT_RECOVERED",
            "MnOverhaulIvyMeter",
            "MnOverhaulIvyMeterText",
        ),
        (
            "EVENT_MN_HEADWATERS_REPORT",
            "MnOverhaulIvyMine",
            "MnOverhaulIvyMineText",
        ),
        (
            "EVENT_MN_BRAINERD_REPORT",
            "MnOverhaulIvyBrainerd",
            "MnOverhaulIvyBrainerdText",
        ),
        (
            "EVENT_MN_DEEP_BORE_MANIFEST_READ",
            "MnOverhaulIvyNorth",
            "MnOverhaulIvyNorthText",
        ),
        (
            "EVENT_MN_ICE_CAPTIVE_RESCUED",
            "MnOverhaulIvyEllisSafe",
            "MnOverhaulIvyEllisSafeText",
        ),
    ];
    if texts.is_some_and(|texts| {
        stages
            .iter()
            .any(|(_, _, label)| !texts.contains_key(*label))
    }) {
        return Ok(false);
    }
    let mut reminder = Vec::new();
    for (flag, root, text) in stages {
        if let Some(texts) = texts {
            room.scripts
                .insert(text.into(), authored_dialogue::compile(&texts[text])?);
        }
        reminder.extend([c("checkevent", &[flag]), c("iftrue", &[root])]);
        room.scripts
            .insert(root.into(), json!([c("jumptext", &[text])]));
    }
    reminder.push(c("jumptext", &["MnRCDeliveryAfterText"]));
    room.scripts
        .insert("MnRCDeliveryAfter".into(), json!(reminder));
    Ok(true)
}
