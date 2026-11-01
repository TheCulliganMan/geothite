//! A witnessed excavation and three independent, persisted containment controls.
use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::northstar_lair_authoring as a;
use crate::northstar_rival_arc::{self as arc, FINAL_WON, LINK, RESTORED, map, publish, put};
use crate::verdigrim_intake_art::{self as art, IntakeArt};

const MANIFEST: &str = "verdigrim-intake-scene-v1";
const WOLF: &str = "MN_VERDIGRIM_INTAKE";
const FOREMAN: &str = "MN_VERDIGRIM_FOREMAN";
const EXPOSED: &str = "EVENT_MN_VERDIGRIM_EXPOSED";
const GREEN: &str = "EVENT_MN_VERDIGRIM_GREEN_WATER";
const COUNTER: &str = "EVENT_MN_VERDIGRIM_COUNTER_CURRENT";
const FLED: &str = "EVENT_MN_VERDIGRIM_FOREMAN_FLED";
const DRILL: &str = "EVENT_MN_VERDIGRIM_DRILL_STOPPED";
const OUTLET: &str = "EVENT_MN_VERDIGRIM_OUTLET_CLOSED";
const BYPASS: &str = "EVENT_MN_VERDIGRIM_CLEAN_BYPASS";
const PINNED: &str = "EVENT_MN_VERDIGRIM_PINNED";
const RETREATED: &str = "EVENT_MN_VERDIGRIM_RETREATED";
const CONTAINED: &str = "EVENT_MN_VERDIGRIM_CONTAINED";
const CAUGHT: &str = "EVENT_MN_VERDIGRIM_CAUGHT";

/// Adds Verdigrim's exposure, bounded green outflow, ordered controls and an
/// optional level-sixty encounter. Only acknowledged crew clearance grants
/// containment/restoration; only actual capture consumes the encounter.
/// Refusal and failed capture retain another attempt. Existing rival battles,
/// crew paths, outside maps, species and all audio programs remain unchanged.
///
/// # Errors
/// Rejects incompatible source/geometry, duplicate installation, missing reviewed
/// art/cry, occupied labels, malformed original scripts or invalid output.
pub fn build_verdigrim_scene(base: &CompiledGamePack, site: &str) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::verify_northstar_publication_geometry(base)?;
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "scene already installed"
    );
    let site: Value = serde_json::from_str(site)?;
    ensure!(
        site["map"] == "MnBoundaryIntake" && site["source_tileset"] == "mn_arc_intake",
        "current intake site required"
    );
    ensure!(
        site["anchors"]["exposure_camera"] == serde_json::json!([19, 11])
            && site["anchors"]["pinned_wolf"] == serde_json::json!([23, 11])
            && site["controls"]["outlet"]["read_tile"] == serde_json::json!([21, 19])
            && site["controls"]["bypass"]["read_tile"] == serde_json::json!([13, 11]),
        "scene source anchors differ from the authored movement contract"
    );
    ensure!(
        base.data.pokemon.contains_key("VERDIGRIM")
            && base
                .runtime_files
                .contains_key("gfx/pokemon/verdigrim/overworld.2bpp"),
        "reviewed wolf species and walking bank required"
    );
    ensure!(
        base.data
            .pokemon_cries
            .get("VERDIGRIM")
            .is_some_and(|cry| cry.cry == "CRY_MN_319" && cry.pitch == 0 && cry.length == 256),
        "original Verdigrim howl required"
    );
    let mut pack = base.clone();
    let mut room = map(&pack, "MnBoundaryIntake")?;
    ensure!(
        (room.attributes.width, room.attributes.height) == (16, 12)
            && room.attributes.tileset_name == "mn_arc_intake",
        "canonical intake geometry required"
    );
    ensure!(
        room.scripts
            .keys()
            .all(|name| !name.starts_with("MnVerdigrim")),
        "occupied scene labels"
    );
    for point in [[18, 9], [23, 9], [25, 6], [17, 12], [21, 20], [13, 12]] {
        ensure!(
            crate::authored_town_geometry::kind(&pack, &room, point)? == "FLOOR",
            "scene anchor must be native floor: {point:?}"
        );
    }
    let original_stop = room
        .scripts
        .get("MnArcIntakeStop")
        .and_then(Value::as_array)
        .context("original local drive")?;
    let release = original_stop
        .iter()
        .position(|command| command == &c("changeblock", &["20", "8", "7"]))
        .context("original outflow continuation")?;
    let mut crew = original_stop[release..].to_vec();
    let restoration = crew
        .iter()
        .position(|command| command == &c("setevent", &[RESTORED]))
        .context("acknowledged restoration")?;
    crew.insert(restoration, c("setevent", &[CONTAINED]));
    let previous_actors = room
        .scripts
        .get("MnArcIntakeActors")
        .context("original actor callback")?
        .clone();
    room.scripts
        .insert("MnVerdigrimPreviousActors".into(), previous_actors);
    for flag in [
        EXPOSED, GREEN, COUNTER, FLED, DRILL, OUTLET, BYPASS, PINNED, RETREATED, CONTAINED, CAUGHT,
    ] {
        a::flag(&mut pack, flag)?;
    }
    let native = art::install(&mut pack)?;
    room.attributes.tileset_name = art::TILESET.into();
    room.blocks[4 * 16 + 11] = native.copper_cover;
    room.blocks[9 * 16 + 10] = 0x16;
    room.blocks[5 * 16 + 6] = 0x16;
    ensure!(
        room.blocks[6 * 16 + 15] == 0x17,
        "preserve the native wall at the unconnected outflow edge"
    );
    let mut revealed = room.clone();
    revealed.blocks[4 * 16 + 11] = 7;
    revealed.blocks[5 * 16 + 11] = native.broken_line;
    ensure!(
        crate::authored_town_geometry::kind(&pack, &revealed, [23, 11])? == "FLOOR"
            && crate::authored_town_geometry::kind(&pack, &revealed, [23, 10])? == "WALL",
        "wolf platform must touch water without opening the chamber"
    );
    let surf_reachable =
        crate::authored_town_geometry::reachable(&pack, &revealed, [18, 18], true)?;
    ensure!(
        !surf_reachable.contains(&[22, 7]),
        "Surf must not bypass the sealed creature chamber"
    );
    for point in surf_reachable {
        if point[0] == 0 || point[0] == 31 || point[1] == 0 || point[1] == 23 {
            ensure!(
                crate::authored_town_geometry::kind(&pack, &revealed, point)? != "WATER",
                "unconnected intake water boundary at {point:?}"
            );
        }
    }
    revealed.blocks[4 * 16 + 10] = 7;
    ensure!(
        crate::authored_town_geometry::reachable(&pack, &revealed, [18, 18], false)?
            .contains(&[24, 6]),
        "the released chamber needs a dry encounter approach and return"
    );
    let lode_sprite = room
        .objects
        .iter()
        .find(|o| {
            o.object_identifier
                .as_deref()
                .is_some_and(|id| id.starts_with("MN_LODE_ARC"))
        })
        .context("existing Lode field sprite")?
        .sprite
        .clone();
    a::actor(&pack, &mut room, FOREMAN, &lode_sprite, [17, 7])?;
    a::actor(&pack, &mut room, WOLF, "SPRITE_VERDIGRIM", [23, 9])?;
    put(
        &mut room,
        FOREMAN,
        vec![c("jumptextfaceplayer", &["MnVerdigrimOrderText"])],
    );
    for (label, steps) in [
        ("MnVerdigrimForemanStep", vec![("DOWN", 1)]),
        ("MnVerdigrimPaw", vec![("LEFT", 1), ("RIGHT", 1)]),
        ("MnVerdigrimFailedPush", vec![("DOWN", 1), ("UP", 1)]),
        (
            "MnVerdigrimForemanLeaves",
            vec![("LEFT", 10), ("DOWN", 10), ("LEFT", 2)],
        ),
        ("MnVerdigrimFocus", vec![("RIGHT", 1), ("DOWN", 2)]),
        ("MnVerdigrimLook", vec![("RIGHT", 6), ("UP", 1)]),
        ("MnVerdigrimRetreat", vec![("RIGHT", 2), ("UP", 3)]),
        ("MnVerdigrimReturn", vec![("DOWN", 1), ("LEFT", 2)]),
    ] {
        arc::movement(&mut room, label, &steps);
    }
    put(
        &mut room,
        "MnVerdigrimDrop",
        vec![c("jump_step", &["DOWN"]), c("step_end", &[])],
    );
    let mut retreat = room.scripts["MnVerdigrimRetreat"]
        .as_array()
        .context("retreat movement")?
        .clone();
    retreat.insert(0, c("jump_step", &["UP"]));
    put(&mut room, "MnVerdigrimRetreat", retreat);
    texts(&mut room);
    let mut reveal = a::say("MnVerdigrimWarningText");
    reveal.push(c("applymovement", &[FOREMAN, "MnVerdigrimForemanStep"]));
    reveal.extend(a::say("MnVerdigrimOrderText"));
    reveal.extend([
        c("playsound", &["SFX_STRENGTH"]),
        c("earthquake", &["8"]),
        change(22, 8, 7),
        c("refreshmap", &[]),
        c("appear", &[WOLF]),
        c("cry", &["VERDIGRIM"]),
        c("applymovement", &[WOLF, "MnVerdigrimPaw"]),
        change(22, 10, native.broken_line),
        c("refreshmap", &[]),
        c("setevent", &[EXPOSED]),
        c("applymovement", &["PLAYER", "MnVerdigrimFocus"]),
        c("applymovement", &[WOLF, "MnVerdigrimDrop"]),
        c("setevent", &[PINNED]),
    ]);
    reveal.extend(a::say("MnVerdigrimExposedText"));
    reveal.extend(water(native.green_water, 10..15));
    reveal.extend([c("setevent", &[GREEN]), c("refreshmap", &[])]);
    reveal.extend(a::say("MnVerdigrimGreenText"));
    reveal.push(c(
        "applymovement",
        &["MN_ARC_KETTLURK", "MnVerdigrimFailedPush"],
    ));
    reveal.extend(a::say("MnVerdigrimCounterText"));
    reveal.extend([
        c("setevent", &[COUNTER]),
        c("applymovement", &[FOREMAN, "MnVerdigrimForemanLeaves"]),
        c("disappear", &[FOREMAN]),
        c("setevent", &[FLED]),
        c("end", &[]),
    ]);
    put(&mut room, "MnVerdigrimExposure", reveal);
    put(
        &mut room,
        "MnArcIntakeEntry",
        vec![
            c("checkevent", &[EXPOSED]),
            c("iftrue", &["MnArcIntakeEnd"]),
            c("setevent", &["EVENT_MN_INTAKE_FIRST_SEEN"]),
            c("scall", &["MnArcIntakeWitness"]),
            c("sjump", &["MnVerdigrimExposure"]),
        ],
    );
    let mut drill = prompt(
        DRILL,
        &[COUNTER, LINK, FINAL_WON],
        "MnVerdigrimDrillAskText",
    );
    drill.extend([
        change(16, 10, native.stopped_control),
        c("playsound", &["SFX_READ_TEXT_2"]),
        c("setevent", &[DRILL]),
        c("refreshmap", &[]),
    ]);
    drill.extend(a::say("MnVerdigrimDrillDoneText"));
    drill.push(c("end", &[]));
    put(&mut room, "MnArcIntakeStop", drill);
    let mut outlet = prompt(
        OUTLET,
        &[COUNTER, FINAL_WON, DRILL],
        "MnVerdigrimOutletAskText",
    );
    outlet.extend([
        change(20, 18, native.stopped_control),
        c("playsound", &["SFX_READ_TEXT_2"]),
        c("setevent", &[OUTLET]),
        c("refreshmap", &[]),
    ]);
    outlet.extend(a::say("MnVerdigrimOutletDoneText"));
    outlet.push(c("end", &[]));
    put(&mut room, "MnVerdigrimOutlet", outlet);
    let mut bypass = prompt(
        BYPASS,
        &[COUNTER, FINAL_WON, DRILL, OUTLET],
        "MnVerdigrimBypassAskText",
    );
    bypass.extend(water(0x40, 10..13));
    bypass.extend([
        change(12, 10, native.open_control),
        c("setevent", &[BYPASS]),
        c("refreshmap", &[]),
        c("applymovement", &["PLAYER", "MnVerdigrimLook"]),
    ]);
    bypass.extend(a::say("MnVerdigrimBypassDoneText"));
    bypass.extend([
        c("applymovement", &[WOLF, "MnVerdigrimRetreat"]),
        c("setevent", &[RETREATED]),
    ]);
    bypass.extend(a::say("MnVerdigrimRetreatText"));
    bypass.push(c("applymovement", &["PLAYER", "MnVerdigrimReturn"]));
    bypass.extend(crew);
    put(&mut room, "MnVerdigrimBypass", bypass);
    for (at, label) in [
        ([21, 19], "MnVerdigrimOutlet"),
        ([13, 11], "MnVerdigrimBypass"),
    ] {
        a::read(&mut room, at, label);
        room.events
            .bg_events
            .last_mut()
            .context("new control")?
            .event_type = "BGEVENT_UP".into();
    }
    put(
        &mut room,
        "MnVerdigrimDeclined",
        vec![c("closetext", &[]), c("end", &[])],
    );
    put(
        &mut room,
        "MnVerdigrimAlready",
        vec![c("jumptext", &["MnVerdigrimAlreadyText"])],
    );
    put(
        &mut room,
        "MnVerdigrimDenied",
        vec![c("jumptext", &["MnVerdigrimDeniedText"])],
    );
    callback(&mut room, &native);
    encounter(&mut room);
    room.scripted_wild_battles = crate::parse_scripted_wild_battles(&room.id, &room.scripts)?;
    publish(&mut pack, room)?;
    pack.report.manifests.push(MANIFEST.into());
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

fn change(x: u16, y: u16, block: u16) -> Value {
    c(
        "changeblock",
        &[&x.to_string(), &y.to_string(), &block.to_string()],
    )
}
fn water(block: u16, columns: std::ops::Range<u16>) -> Vec<Value> {
    columns.map(|x| change(x * 2, 12, block)).collect()
}
fn prompt(done: &str, required: &[&str], text: &str) -> Vec<Value> {
    let mut rows = vec![
        c("checkevent", &[done]),
        c("iftrue", &["MnVerdigrimAlready"]),
    ];
    for flag in required {
        rows.extend([
            c("checkevent", &[flag]),
            c("iffalse", &["MnVerdigrimDenied"]),
        ]);
    }
    rows.extend([
        c("opentext", &[]),
        c("writetext", &[text]),
        c("yesorno", &[]),
        c("iffalse", &["MnVerdigrimDeclined"]),
        c("closetext", &[]),
    ]);
    rows
}

fn callback(room: &mut crate::MapModule, native: &IntakeArt) {
    // NEWMAP runs on a physical entry, unlike Continue's tile/sprite callbacks.
    // Reset the temporary retreat here without undoing a saved hidden visit.
    put(
        room,
        "MnVerdigrimNewVisit",
        vec![
            c("checkevent", &[EXPOSED]),
            c("iffalse", &["MnVerdigrimVisitEnd"]),
            c("checkevent", &[CAUGHT]),
            c("iftrue", &["MnVerdigrimVisitEnd"]),
            c("appear", &[WOLF]),
            c("endcallback", &[]),
        ],
    );
    put(room, "MnVerdigrimVisitEnd", vec![c("endcallback", &[])]);
    a::callback(room, "MAPCALLBACK_NEWMAP", "MnVerdigrimNewVisit");
    put(
        room,
        "MnArcIntakeActors",
        vec![
            c("checkevent", &[FLED]),
            c("iffalse", &["MnVerdigrimRestoreExposure"]),
            c("disappear", &[FOREMAN]),
            c("sjump", &["MnVerdigrimRestoreExposure"]),
        ],
    );
    put(
        room,
        "MnVerdigrimRestoreExposure",
        vec![
            c("checkevent", &[EXPOSED]),
            c("iffalse", &["MnVerdigrimBeforeExposure"]),
            change(22, 8, 7),
            change(22, 10, native.broken_line),
            c("checkevent", &[RETREATED]),
            c("iffalse", &["MnVerdigrimRestorePinned"]),
            c("moveobject", &[WOLF, "25", "6"]),
            c("sjump", &["MnVerdigrimRestoreWolf"]),
        ],
    );
    put(
        room,
        "MnVerdigrimRestorePinned",
        vec![
            c("checkevent", &[PINNED]),
            c("iffalse", &["MnVerdigrimRestoreWolf"]),
            c("moveobject", &[WOLF, "23", "11"]),
            c("sjump", &["MnVerdigrimRestoreWolf"]),
        ],
    );
    put(
        room,
        "MnVerdigrimBeforeExposure",
        vec![
            c("disappear", &[WOLF]),
            c("sjump", &["MnVerdigrimPreviousActors"]),
        ],
    );
    put(
        room,
        "MnVerdigrimRestoreWolf",
        vec![
            c("checkevent", &[CAUGHT]),
            c("iftrue", &["MnVerdigrimHideWolf"]),
            // Fresh map objects are visible by default. Keep the saved hidden
            // state after RUN/KO; Continue must not undo that visit's retreat.
            c("sjump", &["MnVerdigrimRestoreWater"]),
        ],
    );
    put(
        room,
        "MnVerdigrimHideWolf",
        vec![
            c("disappear", &[WOLF]),
            c("sjump", &["MnVerdigrimRestoreWater"]),
        ],
    );
    let mut rows = vec![
        c("checkevent", &[GREEN]),
        c("iffalse", &["MnVerdigrimRestoreDrill"]),
    ];
    rows.extend(water(native.green_water, 10..15));
    rows.push(c("sjump", &["MnVerdigrimRestoreDrill"]));
    put(room, "MnVerdigrimRestoreWater", rows);
    put(
        room,
        "MnVerdigrimRestoreDrill",
        vec![
            c("checkevent", &[DRILL]),
            c("iffalse", &["MnVerdigrimRestoreOutlet"]),
            change(16, 10, native.stopped_control),
            c("sjump", &["MnVerdigrimRestoreOutlet"]),
        ],
    );
    put(
        room,
        "MnVerdigrimRestoreOutlet",
        vec![
            c("checkevent", &[OUTLET]),
            c("iffalse", &["MnVerdigrimRestoreBypass"]),
            change(20, 18, native.stopped_control),
            c("sjump", &["MnVerdigrimRestoreBypass"]),
        ],
    );
    let mut rows = vec![
        c("checkevent", &[BYPASS]),
        c("iffalse", &["MnVerdigrimPreviousActors"]),
    ];
    rows.extend(water(0x40, 10..13));
    rows.extend([
        change(12, 10, native.open_control),
        c("sjump", &["MnVerdigrimPreviousActors"]),
    ]);
    put(room, "MnVerdigrimRestoreBypass", rows);
}

fn encounter(room: &mut crate::MapModule) {
    use crystal_core::battle::capture::OPTIONAL_LEGENDARY_BATTLE_TYPE;
    put(
        room,
        WOLF,
        vec![
            c("faceplayer", &[]),
            c("checkevent", &[CONTAINED]),
            c("iffalse", &["MnVerdigrimDenied"]),
            c("opentext", &[]),
            c("writetext", &["MnVerdigrimEncounterText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnVerdigrimDeclined"]),
            c("closetext", &[]),
            c("cry", &["VERDIGRIM"]),
            c(
                "loadvar",
                &["VAR_BATTLETYPE", OPTIONAL_LEGENDARY_BATTLE_TYPE],
            ),
            c("loadwildmon", &["VERDIGRIM", "60"]),
            c("startbattle", &[]),
            c("reloadmapafterbattle", &[]),
            c("special", &["CheckCaughtCelebi"]),
            c("iftrue", &["MnVerdigrimCaught"]),
            c("disappear", &[WOLF]),
            c("jumptext", &["MnVerdigrimRetryText"]),
        ],
    );
    put(
        room,
        "MnVerdigrimCaught",
        vec![
            c("setevent", &[CAUGHT]),
            c("disappear", &[WOLF]),
            c("jumptext", &["MnVerdigrimCaughtText"]),
        ],
    );
}

fn texts(room: &mut crate::MapModule) {
    for (label, text) in [
        (
            "MnVerdigrimWarningText",
            "FINN: Green grit in the test sump. Stop cutting that seam.",
        ),
        (
            "MnVerdigrimOrderText",
            "FOREMAN: We have a schedule. Clear the rest of that copper.",
        ),
        (
            "MnVerdigrimExposedText",
            "The copper mass opens its eyes. VERDIGRIM crushes the test line with one huge forepaw.",
        ),
        (
            "MnVerdigrimGreenText",
            "Dark green water spills into the outflow. IVY: Back from the water. All of you.",
        ),
        (
            "MnVerdigrimCounterText",
            "KETTLURK pushes the current back. The wave curls green and stalls. FINN: That creature is pinning the return channel shut.",
        ),
        (
            "MnVerdigrimDrillAskText",
            "The drill feeds the damaged test line. Stop its drive?",
        ),
        (
            "MnVerdigrimDrillDoneText",
            "The drill stops. The dirty outlet is still open.",
        ),
        (
            "MnVerdigrimOutletAskText",
            "The contaminated outlet is open. Close the valve?",
        ),
        (
            "MnVerdigrimOutletDoneText",
            "The outlet is sealed. Green water remains in the sump. The clean bypass is still closed.",
        ),
        (
            "MnVerdigrimBypassAskText",
            "The clean bypass is isolated from the sump. Open it?",
        ),
        (
            "MnVerdigrimBypassDoneText",
            "The clean return opens. KETTLURK's current presses against the copper tail.",
        ),
        (
            "MnVerdigrimRetreatText",
            "VERDIGRIM lifts its tail and retreats to the exposed chamber. The clean current is moving again.",
        ),
        (
            "MnVerdigrimDeniedText",
            "Clear the intake warning and REED first. Stop the drill, seal the dirty outlet, then open the clean bypass.",
        ),
        ("MnVerdigrimAlreadyText", "That control is already set."),
        (
            "MnVerdigrimEncounterText",
            "VERDIGRIM watches from the copper chamber. Approach it?",
        ),
        (
            "MnVerdigrimRetryText",
            "VERDIGRIM withdraws deeper into the chamber. It may return when you do.",
        ),
        (
            "MnVerdigrimCaughtText",
            "VERDIGRIM was recorded. The sealed sump still needs the field crew's care.",
        ),
        (
            "MnArcIntakeFinnText",
            "Stop the drill. Seal the dirty outlet in the lower bay. Then open the clean bypass beside the west drive. We can clear the crew after that.",
        ),
        (
            "MnArcIntakeIvyText",
            "REED is guarding the drive. Keep clear of the green water. We need the drill, outlet and bypass set separately.",
        ),
    ] {
        authoring::text(room, label, &[text]);
    }
}
