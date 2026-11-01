//! Reed retaliates at the depot and the player interrupts a coercive transfer.
use anyhow::{Context, Result};
use crystal_core::map::CoordEvent;
use serde_json::Value;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring::command as c;
use crate::northstar_lair_authoring as a;
use crate::northstar_rival_arc::{self as arc, ArcSource, DEPOT, DEPOT_WON, WORKER};

pub(super) fn install(pack: &mut CompiledGamePack, source: &ArcSource) -> Result<()> {
    depot(pack, source)?;
    worker(pack, source)?;
    let mut mine = arc::map(pack, "MnIronDrillCore")?;
    arc::texts(&mut mine, source, "MnNDrillReed");
    mine.objects
        .iter_mut()
        .find(|o| o.object_identifier.as_deref() == Some("MN_NDRILL_REED"))
        .context("mine rival")?
        .event_flag = arc::LINK.into();
    mine.objects
        .iter_mut()
        .find(|o| o.object_identifier.as_deref() == Some("MN_NDRILL_FINN"))
        .context("rescued mine worker")?
        .event_flag = "EVENT_MN_DEPOT_CLEARED".into();
    arc::publish(pack, mine)
}
fn depot(pack: &mut CompiledGamePack, source: &ArcSource) -> Result<()> {
    let mut room = arc::map(pack, "MnStPaulUnionDepot")?;
    arc::texts(&mut room, source, "MnArcSP");
    arc::actor(pack, &mut room, "MN_ARC_SP_REED", "SPRITE_RIVAL", [22, 17])?;
    let mina = room
        .scripts
        .get_mut("MN_NSTP_MINA")
        .and_then(Value::as_array_mut)
        .context("Mina offer")?;
    let position = mina
        .iter()
        .position(|r| *r == c("opentext", &[]))
        .context("retryable Mina prompt")?;
    mina.splice(
        position..position,
        [
            c("checkevent", &[DEPOT_WON]),
            c("iffalse", &["MnArcSPChoice"]),
        ],
    );
    arc::put(
        &mut room,
        "MN_ARC_SP_REED",
        vec![
            c("faceplayer", &[]),
            c("checkevent", &[DEPOT_WON]),
            c("iftrue", &["MnArcSPAfter"]),
            c("checkevent", &["EVENT_MN_COURIER_DEFEATED"]),
            c("iffalse", &["MnArcSPEnd"]),
            c("sjump", &["MnArcSPChoice"]),
        ],
    );
    let mut choice = vec![
        c("checkevent", &[DEPOT]),
        c("iftrue", &["MnArcSPReady"]),
        c("setevent", &[DEPOT]),
        c("appear", &["MN_ARC_SP_REED"]),
        c("faceobject", &["MN_ARC_SP_REED", "MN_NSTP_MINA"]),
        c("faceobject", &["MN_NSTP_MINA", "MN_ARC_SP_REED"]),
    ];
    choice.extend(a::say("MnArcSPBeforeText"));
    choice.extend([c("sjump", &["MnArcSPReady"])]);
    arc::put(&mut room, "MnArcSPChoice", choice);
    arc::put(
        &mut room,
        "MnArcSPReady",
        vec![
            c("opentext", &[]),
            c("writetext", &["MnArcSPReadyText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnArcSPDeclined"]),
            c("closetext", &[]),
            c("sjump", &["MnArcSPBattle"]),
        ],
    );
    arc::battle(
        &mut room,
        "MnArcSPBattle",
        "MN_REED_STPAUL",
        DEPOT_WON,
        "MnArcSPBattleText",
        "MnArcSPWonText",
    );
    let rows = room
        .scripts
        .get_mut("MnArcSPBattle")
        .and_then(Value::as_array_mut)
        .context("depot battle")?;
    rows.insert(rows.len() - 1, c("disappear", &["MN_ARC_SP_REED"]));
    arc::jump(&mut room, "MnArcSPAfter", "MnArcSPAfterText");
    arc::jump(&mut room, "MnArcSPDeclined", "MnArcSPDeclinedText");
    arc::put(&mut room, "MnArcSPEnd", vec![c("end", &[])]);
    arc::put(
        &mut room,
        "MnArcSPActors",
        vec![
            c("checkevent", &[DEPOT_WON]),
            c("iftrue", &["MnArcSPHide"]),
            c("checkevent", &["EVENT_MN_COURIER_DEFEATED"]),
            c("iffalse", &["MnArcSPHide"]),
            c("appear", &["MN_ARC_SP_REED"]),
            c("endcallback", &[]),
        ],
    );
    arc::put(
        &mut room,
        "MnArcSPHide",
        vec![c("disappear", &["MN_ARC_SP_REED"]), c("endcallback", &[])],
    );
    a::callback(&mut room, "MAPCALLBACK_OBJECTS", "MnArcSPActors");
    arc::publish(pack, room)
}
fn worker(pack: &mut CompiledGamePack, source: &ArcSource) -> Result<()> {
    let mut room = arc::map(pack, "MnStCloud")?;
    arc::texts(&mut room, source, "MnArcSC");
    arc::texts(&mut room, source, "MnArcSera");
    arc::actor(pack, &mut room, "MN_ARC_SC_REED", "SPRITE_RIVAL", [25, 5])?;
    arc::actor(
        pack,
        &mut room,
        "MN_ARC_SC_SERA",
        "SPRITE_POKEFAN_F",
        [24, 5],
    )?;
    arc::put(
        &mut room,
        "MN_ARC_SC_REED",
        vec![c("jumptext", &["MnArcSCBeforeText"])],
    );
    arc::put(
        &mut room,
        "MN_ARC_SC_SERA",
        vec![
            c("checkevent", &[WORKER]),
            c("iftrue", &["MnArcSCAfter"]),
            c("sjump", &["MnArcSCScene"]),
        ],
    );
    let mut scene = vec![
        c("checkevent", &[WORKER]),
        c("iftrue", &["MnArcSCEnd"]),
        c("checkevent", &[DEPOT_WON]),
        c("iffalse", &["MnArcSCEnd"]),
    ];
    scene.extend(a::say("MnArcSCBeforeText"));
    scene.extend([
        c("opentext", &[]),
        c("writetext", &["MnArcSCChoiceText"]),
        c("yesorno", &[]),
        c("iffalse", &["MnArcSCDeclined"]),
        c("closetext", &[]),
    ]);
    // The two entry lanes stage a safe common pose before either NPC moves.
    scene.extend([c(
        "applymovement",
        &["MN_ARC_SC_REED", "MnArcSCReedStepsBack"],
    )]);
    scene.extend(a::say("MnArcSCStandText"));
    scene.extend([
        c("applymovement", &["MN_ARC_SC_SERA", "MnArcSeraLeaves"]),
        c("applymovement", &["MN_ARC_SC_REED", "MnArcSCReedLeaves"]),
        c("disappear", &["MN_ARC_SC_REED"]),
    ]);
    scene.extend(a::say("MnArcSCReleasedText"));
    scene.extend([c("setevent", &[WORKER]), c("end", &[])]);
    arc::put(&mut room, "MnArcSCScene", scene);
    arc::jump(&mut room, "MnArcSCDeclined", "MnArcSCDeclinedText");
    arc::jump(&mut room, "MnArcSCAfter", "MnArcSCAfterText");
    arc::put(&mut room, "MnArcSCEnd", vec![c("end", &[])]);
    arc::movement(&mut room, "MnArcSCReedStepsBack", &[("UP", 1)]);
    arc::movement(
        &mut room,
        "MnArcSeraLeaves",
        &[("RIGHT", 1), ("DOWN", 4), ("LEFT", 1)],
    );
    arc::movement(&mut room, "MnArcSCReedLeaves", &[("DOWN", 8), ("RIGHT", 4)]);
    arc::movement(&mut room, "MnArcSCPlayerLeft", &[("DOWN", 1)]);
    arc::movement(&mut room, "MnArcSCPlayerRight", &[("LEFT", 1), ("DOWN", 1)]);
    for (x, movement) in [(24, "MnArcSCPlayerLeft"), (25, "MnArcSCPlayerRight")] {
        let label = format!("MnArcSCEntry{x}");
        arc::put(
            &mut room,
            &label,
            vec![
                c("checkevent", &[WORKER]),
                c("iftrue", &["MnArcSCEnd"]),
                c("checkevent", &[DEPOT_WON]),
                c("iffalse", &["MnArcSCEnd"]),
                c("applymovement", &["PLAYER", movement]),
                c("sjump", &["MnArcSCScene"]),
            ],
        );
        room.events.coord_events.push(CoordEvent {
            x,
            y: 6,
            scene_id: "-1".into(),
            script_name: label,
        });
    }
    arc::put(
        &mut room,
        "MnArcSCActors",
        vec![
            c("checkevent", &[WORKER]),
            c("iftrue", &["MnArcSCWorkerHome"]),
            c("checkevent", &[DEPOT_WON]),
            c("iffalse", &["MnArcSCHide"]),
            c("appear", &["MN_ARC_SC_REED"]),
            c("appear", &["MN_ARC_SC_SERA"]),
            c("endcallback", &[]),
        ],
    );
    arc::put(
        &mut room,
        "MnArcSCWorkerHome",
        vec![
            c("disappear", &["MN_ARC_SC_REED"]),
            c("disappear", &["MN_ARC_SC_SERA"]),
            c("moveobject", &["MN_ARC_SC_SERA", "24", "9"]),
            c("appear", &["MN_ARC_SC_SERA"]),
            c("endcallback", &[]),
        ],
    );
    arc::put(
        &mut room,
        "MnArcSCHide",
        vec![
            c("disappear", &["MN_ARC_SC_REED"]),
            c("disappear", &["MN_ARC_SC_SERA"]),
            c("endcallback", &[]),
        ],
    );
    a::callback(&mut room, "MAPCALLBACK_OBJECTS", "MnArcSCActors");
    arc::publish(pack, room)
}
