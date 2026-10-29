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
    crate::northstar_story_revision::publish(pack, room)
}
