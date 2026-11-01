//! Rochester's optional rhythm companion replaces its misplaced river habitat.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::northstar_rival_arc::{map, publish, put};

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = map(pack, "MnRochesterPatientHouse")?;
    let actor = "MN_SMALL_INTERIOR_ROCHESTER_PATIENT_FRIEND";
    let intro = "MN_SMALL_INTERIOR_ROCHESTER_PATIENT_FRIENDInteriorText";
    ensure!(
        room.scripts
            .get(actor)
            .and_then(serde_json::Value::as_array)
            == Some(&vec![
                c("faceplayer", &[]),
                c("opentext", &[]),
                c("writetext", &[intro]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("end", &[]),
            ]),
        "original resting-house friend required"
    );
    let claimed = "EVENT_MN_PIPULSE_COMPANION_RECEIVED";
    crate::northstar_lair_authoring::flag(pack, claimed)?;
    for (label, lines) in [
        (
            "MnClinicCompanionOfferText",
            vec![
                "This PIPULSE likes",
                "traveling company.",
                "Will you take it",
                "along?",
            ],
        ),
        (
            "MnClinicCompanionFullText",
            vec!["Your team is full.", "Come back with", "a space for it."],
        ),
        (
            "MnClinicCompanionThanksText",
            vec!["Take good care of", "each other."],
        ),
        (
            "MnClinicCompanionAfterText",
            vec!["How is PIPULSE?", "Rest here anytime."],
        ),
    ] {
        ensure!(
            !room.scripts.contains_key(label) && lines.iter().all(|l| l.chars().count() <= 18),
            "fresh native-width companion text required"
        );
        authoring::text(&mut room, label, &lines);
    }
    put(
        &mut room,
        actor,
        vec![
            c("faceplayer", &[]),
            c("opentext", &[]),
            c("checkevent", &[claimed]),
            c("iftrue", &["MnClinicCompanionAfter"]),
            c("writetext", &[intro]),
            c("waitbutton", &[]),
            c("checkevent", &["EVENT_MN_DELIVERED_ROCHESTER_PARCEL"]),
            c("iffalse", &["MnClinicCompanionClose"]),
            c("writetext", &["MnClinicCompanionOfferText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnClinicCompanionClose"]),
            c("readvar", &["VAR_PARTYCOUNT"]),
            c("ifequal", &["6", "MnClinicCompanionFull"]),
            c("closetext", &[]),
            c("givepoke", &["PIPULSE", "8"]),
            c("ifequal", &["2", "MnClinicCompanionGiftFailed"]),
            c("setevent", &[claimed]),
            c("opentext", &[]),
            c("writetext", &["MnClinicCompanionThanksText"]),
            c("waitbutton", &[]),
            c("sjump", &["MnClinicCompanionClose"]),
        ],
    );
    for (label, text) in [
        ("MnClinicCompanionFull", "MnClinicCompanionFullText"),
        ("MnClinicCompanionAfter", "MnClinicCompanionAfterText"),
    ] {
        put(
            &mut room,
            label,
            vec![
                c("writetext", &[text]),
                c("waitbutton", &[]),
                c("sjump", &["MnClinicCompanionClose"]),
            ],
        );
    }
    put(
        &mut room,
        "MnClinicCompanionGiftFailed",
        vec![c("opentext", &[]), c("sjump", &["MnClinicCompanionFull"])],
    );
    put(
        &mut room,
        "MnClinicCompanionClose",
        vec![c("closetext", &[]), c("end", &[])],
    );
    ensure!(
        room.objects.iter().any(|o| o.script == actor),
        "resting-house friend actor required"
    );
    publish(pack, room).context("publish clinic companion")
}
