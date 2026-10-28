//! Reed knowingly joins ICE; a real two-floor route replaces the paper gate.
use anyhow::{Context, Result, ensure};
use crystal_core::map::CoordEvent;
use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::northstar_lair_authoring as a;
use crate::{CompiledGamePack, MapModule};

const OFFICE: &str = "MnMinneapolisDispatchOffice";
const CARGO: &str = "MN_ICE_CARGO_BASEMENT";
const RELAY: &str = "MN_ICE_RELAY_FLOOR";
const JOINED: &str = "EVENT_MN_REED_JOINED_ICE";
const FEED: &str = "EVENT_MN_ICE_CARGO_FEED_OFF";
const WON: &str = "EVENT_MN_REED_RELAY_DEFEATED";
const POWER: &str = "EVENT_MN_ICE_HOLDING_POWER_OFF";
const REED: &str = "MN_NICE_REED";
const CAPTAIN: &str = "MN_ICE_ENFORCER";
const NOTE: &str = "EVENT_MN_DISPATCH_NOTE";
const RESCUED: &str = "EVENT_MN_ICE_CAPTIVE_RESCUED";
const TRAINER: &str = "MN_REED_RELAY";

/// Installs the two-floor ICE lair and Reed's witnessed recruitment.
///
/// A cargo breaker opens the relay route and a shorter return. Defeating Reed
/// permits a separate holding-power action; the original enforcer battle and
/// cell release remain necessary. Rescued Ellis gives the next quest at home;
/// the old carbon copy grants no progression. Original art/audio are retained.
///
/// # Errors
/// Returns an error for missing opening content, occupied identities, invalid
/// layouts/text, unexpected original gates, or native pack verification errors.
/// The borrowed parent is unchanged on failure.
pub fn build_northstar_ice_lair(
    base: &CompiledGamePack,
    events: &str,
    cargo: &str,
    relay: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.report
            .manifests
            .iter()
            .any(|s| s == "minnesota-northstar-events-opening-v2"),
        "install the acted opening first"
    );
    let source: Value = serde_json::from_str(events)?;
    ensure!(
        source["schema_version"] == 1 && source["id"] == "northstar-ice-lair-v2",
        "supported ICE source required"
    );
    let mut pack = base.clone();
    for f in [
        JOINED,
        FEED,
        WON,
        POWER,
        "EVENT_MN_ICE_CARGO_GUARD_DEFEATED",
        "EVENT_MN_ICE_RELAY_GUARD_DEFEATED",
    ] {
        a::flag(&mut pack, f)?;
    }
    a::trainer(
        &mut pack,
        TRAINER,
        "RIVAL1",
        "REED",
        &[("CROOKIT", 20), ("GOOSNIDE", 21)],
    )?;
    a::trainer(
        &mut pack,
        "MN_ICE_CARGO_GUARD",
        "GRUNTM",
        "ICE",
        &[("CINDORE", 18), ("MIDGEBIT", 19)],
    )?;
    a::trainer(
        &mut pack,
        "MN_ICE_RELAY_GUARD",
        "GRUNTM",
        "ICE",
        &[("CARTICK", 20), ("LEECHIMP", 21)],
    )?;
    let mut office = pack
        .data
        .maps
        .get(OFFICE)
        .context("original holding office")?
        .clone();
    install_office(&pack, &mut office, &source)?;
    let office_arrival = i16::try_from(office.events.warps.len() + 1)?;
    // Active ladder has a separate safe floor arrival, avoiding warp bounce.
    // The passive arrival must be floor even in the compiled closed-office
    // preimage: warp validation runs before the entry callback opens stairs.
    authoring::warp(&mut office, (5, 4), CARGO, 2);
    authoring::warp(&mut office, (5, 2), CARGO, 2);
    let mut cargo = a::room(&pack, &serde_json::from_str(cargo)?, CARGO)?;
    ensure!(cargo.id == "MnIceCargoBasement", "cargo identity");
    authoring::warp(
        &mut cargo,
        (5, 18),
        "MN_MINNEAPOLIS_DISPATCH_OFFICE",
        office_arrival,
    );
    authoring::warp(
        &mut cargo,
        (4, 19),
        "MN_MINNEAPOLIS_DISPATCH_OFFICE",
        office_arrival,
    );
    authoring::warp(&mut cargo, (23, 4), RELAY, 2);
    authoring::warp(&mut cargo, (22, 5), RELAY, 2);
    install_cargo(&pack, &mut cargo, &source)?;
    let mut relay = a::room(&pack, &serde_json::from_str(relay)?, RELAY)?;
    ensure!(relay.id == "MnIceRelayFloor", "relay identity");
    authoring::warp(&mut relay, (5, 14), CARGO, 4);
    authoring::warp(&mut relay, (4, 15), CARGO, 4);
    install_relay(&pack, &mut relay, &source)?;
    // New metadata is installed before replacing public rooms so all warp
    // targets exist when the final runtime verifier checks the combined pack.
    a::publish(&mut pack, cargo, OFFICE, [4, 19])?;
    a::publish(&mut pack, relay, OFFICE, [4, 15])?;
    crate::ely_league_rooms::finish(&pack, &mut office)?;
    crate::authored_town_publication::replace(&mut pack, office)?;
    install_home(&mut pack, &source)?;
    pack.report.maps = pack.data.maps.len();
    pack.report
        .manifests
        .push("minnesota-northstar-ice-lair-v2".into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

fn put(map: &mut MapModule, name: &str, body: Vec<Value>) -> Result<()> {
    ensure!(
        map.scripts.insert(name.into(), json!(body)).is_none(),
        "occupied ICE program {name}"
    );
    Ok(())
}
fn jump(map: &mut MapModule, script: &str, text: &str) -> Result<()> {
    put(map, script, vec![c("jumptext", &[text])])
}

fn install_office(pack: &CompiledGamePack, map: &mut MapModule, source: &Value) -> Result<()> {
    ensure!(
        map.attributes.width == 10 && map.attributes.height == 8 && map.blocks[12] == 0x0b,
        "original stair crate preimage"
    );
    a::text(
        map,
        source,
        &[
            "MnNICECaptainOfferText",
            "MnNICEReedQuestionText",
            "MnNICEEllisObjectsText",
            "MnNICECaptainAnswerText",
            "MnNICEReedAcceptText",
            "MnNICECaptainBadgeText",
            "MnNICEReedPlayerText",
            "MnNICEEllisCallText",
            "MnNICEPowerLockedText",
            "MnNICECopyFlavorText",
            "MnNICEJoinDeniedText",
        ],
    )?;
    a::actor(pack, map, REED, "SPRITE_RIVAL", [11, 9])?;
    put(
        map,
        REED,
        vec![c("jumptextfaceplayer", &["MnNICEReedPlayerText"])],
    )?;
    // The player stands at (11,8): captain, Reed and the captive all fit the
    // native camera. Paths do not cross the central shelf or occupied actors.
    put(
        map,
        "MnNICEPlayerWatches",
        vec![
            c("step", &["UP"]),
            c("step", &["UP"]),
            c("step", &["UP"]),
            c("step", &["UP"]),
            c("step", &["UP"]),
            c("step", &["RIGHT"]),
            c("step_end", &[]),
        ],
    )?;
    let mut movement = vec![
        c("step", &["LEFT"]),
        c("step", &["LEFT"]),
        c("step", &["UP"]),
        c("step", &["UP"]),
    ];
    movement.extend((0..4).map(|_| c("step", &["LEFT"])));
    movement.extend((0..5).map(|_| c("step", &["UP"])));
    movement.push(c("step_end", &[]));
    put(map, "MnNICEReedToStair", movement)?;
    let mut scene = vec![
        c("checkevent", &[JOINED]),
        c("iftrue", &["MnNICEEnd"]),
        c("appear", &[REED]),
        c("applymovement", &["PLAYER", "MnNICEPlayerWatches"]),
        c("faceobject", &[CAPTAIN, REED]),
        c("faceobject", &[REED, CAPTAIN]),
    ];
    for label in [
        "MnNICECaptainOfferText",
        "MnNICEReedQuestionText",
        "MnNICEEllisObjectsText",
        "MnNICECaptainAnswerText",
        "MnNICEReedAcceptText",
        "MnNICECaptainBadgeText",
    ] {
        scene.extend(a::say(label));
    }
    scene.extend([
        c("setevent", &[JOINED]),
        c("faceobject", &[REED, "PLAYER"]),
        c("turnobject", &["PLAYER", "DOWN"]),
    ]);
    scene.extend(a::say("MnNICEReedPlayerText"));
    scene.extend([
        c("changeblock", &["4", "2", "3"]),
        c("applymovement", &[REED, "MnNICEReedToStair"]),
        c("disappear", &[REED]),
    ]);
    scene.extend(a::say("MnNICEEllisCallText"));
    scene.push(c("end", &[]));
    // Both carpet lanes must witness recruitment. The right lane needs one
    // fewer horizontal step to reach the same native camera position.
    let mut right_scene = scene.clone();
    let movement = right_scene
        .iter_mut()
        .find(|row| **row == c("applymovement", &["PLAYER", "MnNICEPlayerWatches"]))
        .context("recruitment player movement")?;
    *movement = c("applymovement", &["PLAYER", "MnNICEPlayerWatchesRight"]);
    let mut right_steps = (0..5).map(|_| c("step", &["UP"])).collect::<Vec<_>>();
    right_steps.push(c("step_end", &[]));
    put(map, "MnNICEPlayerWatchesRight", right_steps)?;
    put(map, "MnNICEJoinRight", right_scene)?;
    put(map, "MnNICEJoin", scene)?;
    map.events.coord_events.push(CoordEvent {
        x: 10,
        y: 13,
        scene_id: "-1".into(),
        script_name: "MnNICEJoin".into(),
    });
    map.events.coord_events.push(CoordEvent {
        x: 11,
        y: 13,
        scene_id: "-1".into(),
        script_name: "MnNICEJoinRight".into(),
    });
    put(map, "MnNICEEnd", vec![c("end", &[])])?;
    put(
        map,
        "MnNICEOfficeTiles",
        vec![
            c("checkevent", &[JOINED]),
            c("iffalse", &["MnNICEEnd"]),
            c("changeblock", &["4", "2", "3"]),
            c("endcallback", &[]),
        ],
    )?;
    put(
        map,
        "MnNICEOfficeActors",
        vec![
            c("checkevent", &[JOINED]),
            c("iffalse", &["MnNICEEnd"]),
            c("disappear", &[REED]),
            c("endcallback", &[]),
        ],
    )?;
    a::callback(map, "MAPCALLBACK_TILES", "MnNICEOfficeTiles");
    a::callback(map, "MAPCALLBACK_OBJECTS", "MnNICEOfficeActors");
    let latch = map
        .scripts
        .get_mut("MnMPCellLatch")
        .and_then(Value::as_array_mut)
        .context("original cell latch")?;
    ensure!(
        latch[0] == c("checkevent", &[RESCUED]),
        "original release guard changed"
    );
    // Preserve the original completed-rescue branch, then require actual
    // holding shutdown before its enforcer/key/YES-NO release path.
    latch.splice(
        2..2,
        [
            c("checkevent", &[POWER]),
            c("iffalse", &["MnNICEPowerLocked"]),
        ],
    );
    jump(map, "MnNICEPowerLocked", "MnNICEPowerLockedText")?;
    map.scripts.insert(
        "MnMPDispatchCopy".into(),
        json!([c("jumptext", &["MnNICECopyFlavorText"])]),
    );
    Ok(())
}

fn install_cargo(pack: &CompiledGamePack, map: &mut MapModule, source: &Value) -> Result<()> {
    a::guard(
        pack,
        map,
        source,
        "MN_ICE_CARGO_GUARD",
        [12, 12],
        "EVENT_MN_ICE_CARGO_GUARD_DEFEATED",
    )?;
    a::text(
        map,
        source,
        &[
            "MnNICECargoUmaText",
            "MnNICEBreakerText",
            "MnNICEBreakerDoneText",
            "MnNICEBreakerAfterText",
        ],
    )?;
    a::actor(pack, map, "MN_NICE_UMA", "SPRITE_POKEFAN_F", [21, 17])?;
    put(
        map,
        "MN_NICE_UMA",
        vec![c("jumptextfaceplayer", &["MnNICECargoUmaText"])],
    )?;
    let opening = vec![
        c("changeblock", &["22", "6", "13"]),
        c("changeblock", &["8", "16", "13"]),
    ];
    let mut refresh = vec![c("checkevent", &[FEED]), c("iffalse", &["MnNICECargoEnd"])];
    refresh.extend(opening.clone());
    refresh.push(c("endcallback", &[]));
    put(map, "MnNICECargoTiles", refresh)?;
    put(map, "MnNICECargoEnd", vec![c("end", &[])])?;
    a::callback(map, "MAPCALLBACK_TILES", "MnNICECargoTiles");
    let mut breaker = vec![
        c("checkevent", &[FEED]),
        c("iftrue", &["MnNICEBreakerAfter"]),
        c("opentext", &[]),
        c("writetext", &["MnNICEBreakerText"]),
        c("yesorno", &[]),
        c("iffalse", &["MnNICECargoDeclined"]),
        c("closetext", &[]),
        c("setevent", &[FEED]),
    ];
    breaker.extend(opening);
    breaker.extend(a::say("MnNICEBreakerDoneText"));
    breaker.push(c("end", &[]));
    put(map, "MnNICEBreaker", breaker)?;
    jump(map, "MnNICEBreakerAfter", "MnNICEBreakerAfterText")?;
    put(
        map,
        "MnNICECargoDeclined",
        vec![c("closetext", &[]), c("end", &[])],
    )?;
    a::read(map, [23, 17], "MnNICEBreaker");
    Ok(())
}

fn install_relay(pack: &CompiledGamePack, map: &mut MapModule, source: &Value) -> Result<()> {
    a::guard(
        pack,
        map,
        source,
        "MN_ICE_RELAY_GUARD",
        [16, 11],
        "EVENT_MN_ICE_RELAY_GUARD_DEFEATED",
    )?;
    a::text(
        map,
        source,
        &[
            "MnNICEReedSeenText",
            "MnNICEReedWonText",
            "MnNICEReedLostText",
            "MnNICEReedAfterText",
            "MnNICEHoldingText",
            "MnNICEHoldingDoneText",
            "MnNICEHoldingDeniedText",
            "MnNICEHoldingAfterText",
        ],
    )?;
    a::actor(pack, map, "MN_NICE_RELAY_REED", "SPRITE_RIVAL", [21, 4])?;
    let mut fight = vec![
        c("faceplayer", &[]),
        c("checkevent", &[WON]),
        c("iftrue", &["MnNICEReedAfter"]),
    ];
    fight.extend(a::say("MnNICEReedSeenText"));
    fight.extend([
        c("winlosstext", &["MnNICEReedWonText", "MnNICEReedLostText"]),
        c("loadtrainer", &["RIVAL1", TRAINER]),
        c("startbattle", &[]),
        c("reloadmapafterbattle", &[]),
        c("setevent", &[WON]),
        c("sjump", &["MnNICEReedAfter"]),
    ]);
    put(map, "MN_NICE_RELAY_REED", fight)?;
    jump(map, "MnNICEReedAfter", "MnNICEReedAfterText")?;
    put(
        map,
        "MnNICEHolding",
        vec![
            c("checkevent", &[POWER]),
            c("iftrue", &["MnNICEHoldingAfter"]),
            c("checkevent", &[WON]),
            c("iffalse", &["MnNICEHoldingDenied"]),
            c("checkevent", &[FEED]),
            c("iffalse", &["MnNICEHoldingDenied"]),
            c("opentext", &[]),
            c("writetext", &["MnNICEHoldingText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnNICERelayDeclined"]),
            c("writetext", &["MnNICEHoldingDoneText"]),
            c("waitbutton", &[]),
            c("setevent", &[POWER]),
            c("closetext", &[]),
            c("end", &[]),
        ],
    )?;
    jump(map, "MnNICEHoldingAfter", "MnNICEHoldingAfterText")?;
    jump(map, "MnNICEHoldingDenied", "MnNICEHoldingDeniedText")?;
    put(
        map,
        "MnNICERelayDeclined",
        vec![c("closetext", &[]), c("end", &[])],
    )?;
    a::read(map, [20, 2], "MnNICEHolding");
    Ok(())
}

fn install_home(pack: &mut CompiledGamePack, source: &Value) -> Result<()> {
    let mut map = pack
        .data
        .maps
        .get("MnMinneapolisDispatcherHouse")
        .context("JO and ELLIS's home")?
        .clone();
    a::text(
        &mut map,
        source,
        &["MnNICEEllisQuestText", "MnNICEEllisAfterText"],
    )?;
    let mut quest = vec![
        c("faceplayer", &[]),
        c("checkevent", &["EVENT_MN_SURVEY_RESTORED"]),
        c("iftrue", &["MnMPEllisSurveyReturn"]),
        c("checkevent", &[NOTE]),
        c("iftrue", &["MnNICEEllisAfter"]),
        c("checkevent", &[RESCUED]),
        c("iffalse", &["MnNICEHomeEnd"]),
        c("checkevent", &[POWER]),
        c("iffalse", &["MnNICEHomeEnd"]),
        c("opentext", &[]),
        c("writetext", &["MnNICEEllisQuestText"]),
        c("waitbutton", &[]),
        c("setevent", &[NOTE]),
        c("closetext", &[]),
        c("end", &[]),
    ];
    ensure!(
        map.scripts.contains_key("MN_MP_RESCUED_DISPATCHER"),
        "original home actor program"
    );
    map.scripts.insert(
        "MN_MP_RESCUED_DISPATCHER".into(),
        json!(std::mem::take(&mut quest)),
    );
    jump(&mut map, "MnNICEEllisAfter", "MnNICEEllisAfterText")?;
    put(&mut map, "MnNICEHomeEnd", vec![c("end", &[])])?;
    crate::ely_league_rooms::finish(pack, &mut map)?;
    crate::authored_town_publication::replace(pack, map)
}
