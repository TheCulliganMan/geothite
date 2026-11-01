//! Native drill-room choreography and acknowledged crew recovery.
use anyhow::Result;
use serde_json::Value;

use crate::grand_marais_modpack::authoring::command as c;
use crate::northstar_drill_lair::{
    self as drill, CREW, FEED, OFF, RECOVERED, REED, SEEN, WON, jump, put,
};
use crate::northstar_lair_authoring as a;
use crate::{CompiledGamePack, MapModule};

pub(super) fn install(pack: &CompiledGamePack, map: &mut MapModule, source: &Value) -> Result<()> {
    a::text(
        map,
        source,
        &[
            "MnNDrillCrewCallText",
            "MnNDrillForemanText",
            "MnNDrillReedWorkText",
            "MnNDrillReedSeenText",
            "MnNDrillReedWonText",
            "MnNDrillReedLostText",
            "MnNDrillReedAfterText",
            "MnNDrillPowerText",
            "MnNDrillPowerDeniedText",
            "MnNDrillPowerDoneText",
            "MnNDrillPowerAfterText",
            "MnNDrillReedBlocksText",
            "MnNDrillMeterQuestText",
            "MnNDrillMeterAfterText",
            "MnNDrillCrewReadyText",
            "MnNDrillReedPostgameText",
        ],
    )?;
    a::actor(pack, map, REED, "SPRITE_RIVAL", [8, 8])?;
    a::actor(pack, map, "MN_NDRILL_ENGINEER", "SPRITE_SCIENTIST", [8, 5])?;
    a::actor(pack, map, CREW, "SPRITE_FISHER", [15, 9])?;
    jump(map, "MN_NDRILL_ENGINEER", "MnNDrillForemanText")?;
    put(map, "MnNDrillEnd", vec![c("end", &[])])?;
    put(
        map,
        "MnNDrillCoreActors",
        vec![
            c("disappear", &[CREW]),
            c("checkevent", &[OFF]),
            c("iffalse", &["MnNDrillCoreActorsEnd"]),
            // These authored rooms use walking-tile event coordinates. Reload the
            // crew at their final position after a completed shutdown and Continue.
            c("moveobject", &[CREW, "8", "7"]),
            c("appear", &[CREW]),
            c("endcallback", &[]),
        ],
    )?;
    put(map, "MnNDrillCoreActorsEnd", vec![c("endcallback", &[])])?;
    a::callback(map, "MAPCALLBACK_OBJECTS", "MnNDrillCoreActors");
    put(
        map,
        "MnNDrillCoreTiles",
        vec![
            c("checkevent", &[OFF]),
            c("iffalse", &["MnNDrillCoreActorsEnd"]),
            c("changeblock", &["10", "8", "22"]),
            c("endcallback", &[]),
        ],
    )?;
    a::callback(map, "MAPCALLBACK_TILES", "MnNDrillCoreTiles");
    drill::entrances(map)?;
    // Hold opposite strokes during the worker and foreman's lines. Both the
    // renderer and the text client can witness the changing machine before A
    // advances the scene, even when the controller settles timed presentation.
    let mut witness = vec![
        c("playsound", &["SFX_STRENGTH"]),
        c("changeblock", &["10", "8", "22"]),
        c("refreshmap", &[]),
        c("pause", &["12"]),
    ];
    witness.extend(a::say("MnNDrillCrewCallText"));
    witness.extend([
        c("changeblock", &["10", "8", "18"]),
        c("refreshmap", &[]),
        c("earthquake", &["12"]),
        c("pause", &["12"]),
    ]);
    witness.extend(a::say("MnNDrillForemanText"));
    witness.extend([
        c("playsound", &["SFX_STRENGTH"]),
        c("changeblock", &["10", "8", "22"]),
        c("refreshmap", &[]),
        c("pause", &["12"]),
        c("changeblock", &["10", "8", "18"]),
        c("refreshmap", &[]),
        c("earthquake", &["12"]),
        c("pause", &["12"]),
    ]);
    witness.extend(a::say("MnNDrillReedWorkText"));
    witness.extend([c("setevent", &[SEEN]), c("end", &[])]);
    put(map, "MnNDrillWitness", witness)?;
    let mut battle = vec![
        c("faceplayer", &[]),
        c("checkevent", &["EVENT_MN_SURVEY_RESTORED"]),
        c("iftrue", &["MnNDrillReedPostgame"]),
        c("checkevent", &[WON]),
        c("iftrue", &["MnNDrillReedAfter"]),
    ];
    battle.extend(a::say("MnNDrillReedSeenText"));
    battle.extend([
        c(
            "winlosstext",
            &["MnNDrillReedWonText", "MnNDrillReedLostText"],
        ),
        c("loadtrainer", &["RIVAL1", "MN_REED_MINE"]),
        c("startbattle", &[]),
        c("reloadmapafterbattle", &[]),
        c("setevent", &[WON]),
        c("sjump", &["MnNDrillReedAfter"]),
    ]);
    put(map, REED, battle)?;
    jump(map, "MnNDrillReedAfter", "MnNDrillReedAfterText")?;
    jump(map, "MnNDrillReedPostgame", "MnNDrillReedPostgameText")?;
    let mut stop = vec![
        c("checkevent", &[OFF]),
        c("iftrue", &["MnNDrillPowerAfter"]),
        c("checkevent", &[FEED]),
        c("iffalse", &["MnNDrillPowerDenied"]),
        c("checkevent", &[WON]),
        c("iffalse", &["MnNDrillReedBlocks"]),
        c("checkevent", &[SEEN]),
        c("iffalse", &["MnNDrillReedBlocks"]),
        c("opentext", &[]),
        c("writetext", &["MnNDrillPowerText"]),
        c("yesorno", &[]),
        c("iffalse", &["MnNDrillPowerDeclined"]),
        c("closetext", &[]),
        c("changeblock", &["10", "8", "22"]),
        c("refreshmap", &[]),
        c("pause", &["20"]),
        c("applymovement", &["PLAYER", "MnNDrillPlayerToCrew"]),
        c("appear", &[CREW]),
        c("applymovement", &[CREW, "MnNDrillCrewOut"]),
        c("faceobject", &[CREW, "PLAYER"]),
    ];
    stop.extend(a::say("MnNDrillPowerDoneText"));
    stop.extend([c("setevent", &[OFF]), c("end", &[])]);
    put(map, "MnNDrillStop", stop)?;
    let mut player = (0..7).map(|_| c("step", &["LEFT"])).collect::<Vec<_>>();
    player.extend((0..6).map(|_| c("step", &["DOWN"])));
    player.push(c("step_end", &[]));
    put(map, "MnNDrillPlayerToCrew", player)?;
    let mut crew = Vec::new();
    for (direction, steps) in [
        ("DOWN", 4),
        ("LEFT", 6),
        ("UP", 2),
        ("LEFT", 2),
        ("UP", 4),
        ("RIGHT", 1),
    ] {
        crew.extend((0..steps).map(|_| c("step", &[direction])));
    }
    crew.push(c("step_end", &[]));
    put(map, "MnNDrillCrewOut", crew)?;
    for (script, text) in [
        ("MnNDrillPowerDenied", "MnNDrillPowerDeniedText"),
        ("MnNDrillReedBlocks", "MnNDrillReedBlocksText"),
        ("MnNDrillPowerAfter", "MnNDrillPowerAfterText"),
        ("MnNDrillMeterAfter", "MnNDrillMeterAfterText"),
    ] {
        jump(map, script, text)?;
    }
    put(
        map,
        "MnNDrillPowerDeclined",
        vec![c("closetext", &[]), c("end", &[])],
    )?;
    a::read(map, [16, 2], "MnNDrillStop");
    put(
        map,
        CREW,
        vec![
            c("faceplayer", &[]),
            c("checkevent", &[OFF]),
            c("iffalse", &["MnNDrillEnd"]),
            c("checkevent", &[RECOVERED]),
            c("iftrue", &["MnNDrillMeterAfter"]),
            c("opentext", &[]),
            c("writetext", &["MnNDrillCrewReadyText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnNDrillPowerDeclined"]),
            c("writetext", &["MnNDrillMeterQuestText"]),
            c("waitbutton", &[]),
            c("setevent", &[RECOVERED]),
            c("closetext", &[]),
            c("end", &[]),
        ],
    )?;
    Ok(())
}
