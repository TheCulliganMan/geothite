//! The remote link leads to a real intake, its rival and a separately freed current.
use anyhow::{Context, Result, ensure};
use crystal_core::map::CoordEvent;
use serde_json::Value;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;
use crate::grand_marais_modpack::authoring::command as c;
use crate::northstar_lair_authoring as a;
use crate::northstar_rival_arc::{self as arc, ArcSource, FINAL_WON, LINK, RELEASED, RESTORED};

const ROOM: &str = "MnBoundaryIntake";
const CONSTANT: &str = "MN_BOUNDARY_INTAKE";
const REED: &str = "MN_ARC_INTAKE_REED";
const KETTLE: &str = "MN_ARC_KETTLURK";
const IVY: &str = "MN_ARC_INTAKE_IVY";
const FINN: &str = "MN_ARC_INTAKE_FINN";

pub(super) fn install(pack: &mut CompiledGamePack, source: &ArcSource, layout: &str) -> Result<()> {
    let water = pack.data.tilesets["cave"]
        .collision
        .iter()
        .find(|(_, q)| q.iter().all(|v| v == "WATER"))
        .map(|(id, _)| u16::from_str_radix(id, 16))
        .context("native cave water")??;
    let water =
        crate::northstar_rival_art::import_block(pack, "facility", "mn_arc_intake", "cave", water)?;
    ensure!(water == 0x40, "appended intake water identity");
    let stair = crate::northstar_rival_art::import_block(
        pack,
        "mn_northstar_relay",
        "mn_arc_shelter",
        "facility",
        4,
    )?;
    let shutter = crate::northstar_rival_art::import_block(
        pack,
        "mn_arc_shelter",
        "mn_arc_shelter_gate",
        "facility",
        0x19,
    )?;
    let mut room = a::room(pack, &serde_json::from_str(layout)?, CONSTANT)?;
    ensure!(room.id == ROOM, "intake identity");
    room.attributes.tileset_name = "mn_arc_intake".into();
    arc::texts(&mut room, source, "MnArcFinal");
    arc::texts(&mut room, source, "MnArcIntake");
    let mut shelter = arc::map(pack, "MnBoundaryWatersSurveyShelter")?;
    shelter.attributes.tileset_name = "mn_arc_shelter_gate".into();
    shelter.blocks[4 * 8 + 6] = shutter;
    let arrival = i16::try_from(shelter.events.warps.len() + 2)?;
    authoring::warp(&mut shelter, (13, 8), CONSTANT, 2);
    authoring::warp(&mut shelter, (12, 7), CONSTANT, 2);
    authoring::warp(
        &mut room,
        (5, 18),
        "MN_BOUNDARY_WATERS_SURVEY_SHELTER",
        arrival,
    );
    authoring::warp(
        &mut room,
        (4, 19),
        "MN_BOUNDARY_WATERS_SURVEY_SHELTER",
        arrival,
    );
    arc::texts(&mut shelter, source, "MnArcLink");
    arc::put(
        &mut shelter,
        "MN_BOUNDARY_SURVEY_RECEIVER",
        vec![
            c("checkevent", &[RESTORED]),
            c("iftrue", &["MnBoundarySurveyReceiverAlready"]),
            c("checkevent", &[LINK]),
            c("iftrue", &["MnArcLinkAfter"]),
            c("checkevent", &["EVENT_MN_SURVEY_CHIEF_DEFEATED"]),
            c("iffalse", &["MnBoundarySurveyReceiverLocked"]),
            c("opentext", &[]),
            c("writetext", &["MnArcLinkReadyText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnBoundarySurveyReceiverDeclined"]),
            c("writetext", &["MnArcLinkText"]),
            c("waitbutton", &[]),
            c("setevent", &[LINK]),
            c("changeblock", &["12", "8", &stair.to_string()]),
            c("refreshmap", &[]),
            c("closetext", &[]),
            c("end", &[]),
        ],
    );
    arc::jump(&mut shelter, "MnArcLinkAfter", "MnArcLinkAfterText");
    arc::put(
        &mut shelter,
        "MnArcShelterHatch",
        vec![
            c("checkevent", &[LINK]),
            c("iffalse", &["MnArcShelterHatchEnd"]),
            c("changeblock", &["12", "8", &stair.to_string()]),
            c("endcallback", &[]),
        ],
    );
    arc::put(
        &mut shelter,
        "MnArcShelterHatchEnd",
        vec![c("endcallback", &[])],
    );
    a::callback(&mut shelter, "MAPCALLBACK_TILES", "MnArcShelterHatch");
    arc::publish(pack, shelter)?;
    for (id, sprite, at) in [
        (REED, "SPRITE_RIVAL", [15, 9]),
        (KETTLE, "SPRITE_KETTLURK", [23, 7]),
        (IVY, "SPRITE_TEACHER", [9, 5]),
        (FINN, "SPRITE_POKEFAN_M", [11, 9]),
    ] {
        arc::actor(pack, &mut room, id, sprite, at)?;
    }
    arc::put(
        &mut room,
        REED,
        vec![
            c("faceplayer", &[]),
            c("checkevent", &[FINAL_WON]),
            c("iftrue", &["MnArcFinalAfter"]),
            c("checkevent", &[LINK]),
            c("iffalse", &["MnArcFinalLocked"]),
            c("opentext", &[]),
            c("writetext", &["MnArcFinalReadyText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnArcFinalDeclined"]),
            c("closetext", &[]),
            c("sjump", &["MnArcFinalBattle"]),
        ],
    );
    arc::battle(
        &mut room,
        "MnArcFinalBattle",
        "MN_REED_BOUNDARY",
        FINAL_WON,
        "MnArcFinalBeforeText",
        "MnArcFinalWonText",
    );
    for (label, text) in [
        ("MnArcFinalAfter", "MnArcFinalAfterText"),
        ("MnArcFinalLocked", "MnArcFinalLockedText"),
        ("MnArcFinalDeclined", "MnArcFinalDeclinedText"),
        ("MnArcIntakeDenied", "MnArcIntakeDeniedText"),
        ("MnArcIntakeAfter", "MnArcIntakeAfterText"),
    ] {
        arc::jump(&mut room, label, text);
    }
    arc::put(
        &mut room,
        IVY,
        vec![
            c("faceplayer", &[]),
            c("checkevent", &[RESTORED]),
            c("iftrue", &["MnArcIntakeAfter"]),
            c("jumptext", &["MnArcIntakeIvyText"]),
        ],
    );
    arc::jump(&mut room, FINN, "MnArcIntakeFinnText");
    arc::put(
        &mut room,
        KETTLE,
        vec![
            c("pokepic", &["KETTLURK"]),
            c("waitbutton", &[]),
            c("closepokepic", &[]),
            c("end", &[]),
        ],
    );
    let mut stop = vec![
        c("checkevent", &[RESTORED]),
        c("iftrue", &["MnArcIntakeAfter"]),
        c("checkevent", &[LINK]),
        c("iffalse", &["MnArcIntakeDenied"]),
        c("checkevent", &[FINAL_WON]),
        c("iffalse", &["MnArcIntakeDenied"]),
        c("opentext", &[]),
        c("writetext", &["MnArcIntakeReadyText"]),
        c("yesorno", &[]),
        c("iffalse", &["MnArcIntakeDeclined"]),
        c("closetext", &[]),
        c("changeblock", &["20", "8", "7"]),
        c("refreshmap", &[]),
        c("applymovement", &["PLAYER", "MnArcWatchOutflow"]),
    ];
    stop.extend(a::say("MnArcIntakeOpenText"));
    stop.push(c("applymovement", &[KETTLE, "MnArcKettlurkEscapes"]));
    stop.extend(a::say("MnArcIntakeEscapeText"));
    stop.extend([
        c("disappear", &[KETTLE]),
        c("setevent", &[RELEASED]),
        c("applymovement", &["PLAYER", "MnArcWatchCrew"]),
        c("appear", &[IVY]),
        c("appear", &[FINN]),
        c("applymovement", &[IVY, "MnArcIvyCrosses"]),
        c("applymovement", &[FINN, "MnArcFinnCrosses"]),
    ]);
    stop.extend(a::say("MnArcIntakeCrewText"));
    stop.push(c("appear", &[REED]));
    stop.extend(a::say("MnArcIntakeReedExitText"));
    stop.extend([
        c("applymovement", &[REED, "MnArcReedLeaves"]),
        c("disappear", &[REED]),
    ]);
    stop.extend(a::say("MnArcIntakeDoneText"));
    stop.extend([c("setevent", &[RESTORED]), c("end", &[])]);
    arc::put(&mut room, "MnArcIntakeStop", stop);
    arc::put(
        &mut room,
        "MnArcIntakeDeclined",
        vec![c("closetext", &[]), c("end", &[])],
    );
    a::read(&mut room, [17, 11], "MnArcIntakeStop");
    room.events
        .bg_events
        .last_mut()
        .context("local drive")?
        .event_type = "BGEVENT_UP".into();
    arc::movement(&mut room, "MnArcWatchOutflow", &[("RIGHT", 1)]);
    arc::movement(&mut room, "MnArcWatchCrew", &[("LEFT", 5), ("DOWN", 3)]);
    arc::movement(
        &mut room,
        "MnArcKettlurkEscapes",
        &[("DOWN", 2), ("LEFT", 4), ("DOWN", 4), ("RIGHT", 3)],
    );
    arc::movement(&mut room, "MnArcIvyCrosses", &[("DOWN", 10)]);
    arc::movement(&mut room, "MnArcFinnCrosses", &[("DOWN", 6)]);
    arc::movement(&mut room, "MnArcReedLeaves", &[("DOWN", 8), ("LEFT", 6)]);
    let mut seen = vec![
        c("checkevent", &[RESTORED]),
        c("iftrue", &["MnArcIntakeEnd"]),
        c("playsound", &["SFX_STRENGTH"]),
        c("earthquake", &["8"]),
    ];
    seen.extend(a::say("MnArcIntakeSeenText"));
    seen.push(c("end", &[]));
    arc::put(&mut room, "MnArcIntakeWitness", seen);
    // A short entry scene is tied to the active stair, not a repeating barrier.
    let flag = "EVENT_MN_INTAKE_FIRST_SEEN";
    a::flag(pack, flag)?;
    arc::put(
        &mut room,
        "MnArcIntakeEntry",
        vec![
            c("checkevent", &[flag]),
            c("iftrue", &["MnArcIntakeEnd"]),
            c("setevent", &[flag]),
            c("sjump", &["MnArcIntakeWitness"]),
        ],
    );
    for x in 2u16..=19 {
        let movement = format!("MnArcIntakeWatchFrom{x}");
        arc::movement(
            &mut room,
            &movement,
            &[("RIGHT", usize::from(19 - x)), ("UP", 8), ("LEFT", 1)],
        );
        let entry = format!("MnArcIntakeEnter{x}");
        arc::put(
            &mut room,
            &entry,
            vec![
                c("checkevent", &[flag]),
                c("iftrue", &["MnArcIntakeEnd"]),
                c("applymovement", &["PLAYER", &movement]),
                c("sjump", &["MnArcIntakeEntry"]),
            ],
        );
        room.events.coord_events.push(CoordEvent {
            x,
            y: 17,
            scene_id: "-1".into(),
            script_name: entry,
        });
    }
    arc::put(&mut room, "MnArcIntakeEnd", vec![c("end", &[])]);
    arc::put(
        &mut room,
        "MnArcIntakeActors",
        vec![
            c("checkevent", &[RESTORED]),
            c("iffalse", &["MnArcIntakeCallbackEnd"]),
            c("changeblock", &["20", "8", "7"]),
            c("disappear", &[REED]),
            c("disappear", &[KETTLE]),
            c("disappear", &[IVY]),
            c("moveobject", &[IVY, "9", "15"]),
            c("appear", &[IVY]),
            c("disappear", &[FINN]),
            c("moveobject", &[FINN, "11", "15"]),
            c("appear", &[FINN]),
            c("endcallback", &[]),
        ],
    );
    arc::put(
        &mut room,
        "MnArcIntakeCallbackEnd",
        vec![c("endcallback", &[])],
    );
    a::callback(&mut room, "MAPCALLBACK_TILES", "MnArcIntakeActors");
    a::publish(pack, room, "MnBoundaryWatersSurveyShelter", [4, 19])?;
    let cradle =
        crate::northstar_rival_art::import_block(pack, "house", "mn_arc_harbor", "facility", 0x19)?;
    let mut harbor = arc::map(pack, "MnDuluthHarborWorkshop")?;
    harbor.attributes.tileset_name = "mn_arc_harbor".into();
    harbor.blocks[13] = cradle;
    arc::texts(&mut harbor, source, "MnArcHarbor");
    let root = harbor
        .scripts
        .get_mut("MN_HARBOR_RECEIVER")
        .and_then(Value::as_array_mut)
        .context("harbor reveal")?;
    let at = root
        .iter()
        .position(|v| *v == c("setevent", &["EVENT_MN_LIGHTHOUSE_SIGNAL"]))
        .context("separate harbor acknowledgement")?;
    let mut reveal = vec![
        c("writetext", &["MnArcHarborIntakeText"]),
        c("waitbutton", &[]),
    ];
    root.splice(at..at, reveal.drain(..));
    arc::publish(pack, harbor)
}
