//! Keep Ivy, her aide and their advice consistent with the investigation.
use anyhow::{Context, Result};
use serde_json::json;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring::command as c;

pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack
        .data
        .maps
        .get("MnRochesterPokecenter1F")
        .context("Rochester clinic")?
        .clone();
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
    room.objects
        .iter_mut()
        .find(|object| object.script == "MN_ROCHESTER_RECIPIENT")
        .context("Ivy's clinic actor")?
        .sprite = "SPRITE_TEACHER".into();
    room.objects
        .iter_mut()
        .find(|object| object.script == "MN_RC_PATIENT")
        .context("clinic aide actor")?
        .sprite = "SPRITE_SCIENTIST".into();
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
    let mut reminder = Vec::new();
    for (flag, root, text) in [
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
    ] {
        reminder.extend([c("checkevent", &[flag]), c("iftrue", &[root])]);
        room.scripts
            .insert(root.into(), json!([c("jumptext", &[text])]));
    }
    reminder.push(c("jumptext", &["MnRCDeliveryAfterText"]));
    room.scripts
        .insert("MnRCDeliveryAfter".into(), json!(reminder));
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
