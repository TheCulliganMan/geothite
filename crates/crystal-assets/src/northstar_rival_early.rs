//! An early competing path and a concrete reason for Reed's recruitment visit.
use anyhow::Result;
use crystal_core::map::CoordEvent;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring::command as c;
use crate::northstar_lair_authoring as a;
use crate::northstar_rival_arc::{self as arc, ArcSource, EARLY, EARLY_WON};

pub(super) fn install(pack: &mut CompiledGamePack, source: &ArcSource) -> Result<()> {
    let mut clinic = arc::map(pack, "MnRochesterPokecenter1F")?;
    arc::texts(&mut clinic, source, "MnArcRC");
    arc::actor(pack, &mut clinic, "MN_ARC_RC_REED", "SPRITE_RIVAL", [9, 10])?;
    clinic.objects.last_mut().expect("new rival").event_flag = "EVENT_MN_REED_JOINED_ICE".into();
    arc::put(
        &mut clinic,
        "MN_ARC_RC_REED",
        vec![
            c("faceplayer", &[]),
            c("checkevent", &[EARLY_WON]),
            c("iftrue", &["MnArcRCAfter"]),
            c("checkevent", &["EVENT_MN_DELIVERED_ROCHESTER_PARCEL"]),
            c("iffalse", &["MnArcRCWaiting"]),
            c("sjump", &["MnArcRCOffer"]),
        ],
    );
    let mut offer = vec![
        c("checkevent", &[EARLY]),
        c("iftrue", &["MnArcRCChoice"]),
        c("setevent", &[EARLY]),
        c("faceobject", &["MN_ARC_RC_REED", "PLAYER"]),
        c("faceobject", &["PLAYER", "MN_ARC_RC_REED"]),
    ];
    offer.extend(a::say("MnArcRCPathsText"));
    offer.extend([c("sjump", &["MnArcRCChoice"])]);
    arc::put(&mut clinic, "MnArcRCOffer", offer);
    arc::put(
        &mut clinic,
        "MnArcRCChoice",
        vec![
            c("opentext", &[]),
            c("writetext", &["MnArcRCChallengeText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnArcRCDeclined"]),
            c("closetext", &[]),
            c("sjump", &["MnArcRCBattle"]),
        ],
    );
    arc::battle(
        &mut clinic,
        "MnArcRCBattle",
        "MN_REED_ROCHESTER",
        EARLY_WON,
        "MnArcRCBeforeText",
        "MnArcRCWonText",
    );
    arc::jump(&mut clinic, "MnArcRCAfter", "MnArcRCAfterText");
    arc::jump(&mut clinic, "MnArcRCWaiting", "MnArcRCWaitingText");
    arc::jump(&mut clinic, "MnArcRCDeclined", "MnArcRCDeclinedText");
    arc::put(&mut clinic, "MnArcRCEnd", vec![c("end", &[])]);
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
        arc::movement(&mut clinic, &approach, &steps);
        arc::put(
            &mut clinic,
            &entry,
            vec![
                c("checkevent", &[EARLY]),
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
        clinic.events.coord_events.push(CoordEvent {
            x,
            y: 12,
            scene_id: "-1".into(),
            script_name: entry,
        });
    }
    arc::publish(pack, clinic)?;
    let mut lab = arc::map(pack, "MnWinonaFieldStation")?;
    arc::texts(&mut lab, source, "MnNSEReed");
    arc::publish(pack, lab)?;
    let mut office = arc::map(pack, "MnMinneapolisDispatchOffice")?;
    arc::replace_texts(&mut office, source, "MnNICEReed");
    arc::publish(pack, office)?;
    let mut relay = arc::map(pack, "MnIceRelayFloor")?;
    arc::replace_texts(&mut relay, source, "MnNICEReed");
    if let Some(reed) = relay
        .objects
        .iter_mut()
        .find(|o| o.object_identifier.as_deref() == Some("MN_NICE_RELAY_REED"))
    {
        reed.event_flag = arc::DEPOT.into();
    }
    arc::publish(pack, relay)
}
