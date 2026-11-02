//! Staged family departure, professor introduction, starter handover,
//! Pokégear & Pokédex handoff, Winona trail wardens, bedroom cousin tracker,
//! progressive Tamarack guidance, and Route 01 cliff fossil hunter.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::map::{CoordEvent, MapScene, MapScriptSectionCommand};
use serde_json::{Value, json};

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring::command as c;
use crate::northstar_lair_authoring::say;
use crate::northstar_story_revision::publish;
use crate::{CompiledGamePack, MapModule};

const HOME: &str = "EVENT_MN_HOME_INTRO";
const INTRO: &str = "EVENT_MN_TAMARACK_INTRO";
const STARTER: &str = "EVENT_MN_STARTER_RECEIVED";
const PARCEL: &str = "EVENT_MN_R01_PARCEL_RECEIVED";
const BALLS: &str = "EVENT_MN_WINONA_BALLS_RECEIVED";
const TRAIL_OPENED_FLAG: &str = "EVENT_MN_WINONA_TRAIL_OPENED";
const DOCTOR: &str = "MN_W_TAMARACK";

pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    crate::northstar_lair_authoring::flag(pack, INTRO)?;
    crate::northstar_lair_authoring::flag(pack, TRAIL_OPENED_FLAG)?;
    home(pack)?;
    install_home_2f(pack)?;
    install_winona_town(pack)?;
    station(pack)?;
    install_r01_cliff_fossil(pack)
}

fn map(pack: &CompiledGamePack, name: &str) -> Result<MapModule> {
    pack.data
        .maps
        .get(name)
        .cloned()
        .with_context(|| format!("opening room {name}"))
}

fn put(map: &mut MapModule, label: &str, commands: Vec<Value>) {
    map.scripts.insert(label.into(), json!(commands));
}

fn movement(map: &mut MapModule, label: &str, steps: &[(&str, usize)]) {
    let mut body = Vec::new();
    for &(direction, count) in steps {
        body.extend((0..count).map(|_| c("step", &[direction])));
    }
    body.push(c("step_end", &[]));
    put(map, label, body);
}

fn coord(map: &mut MapModule, x: u16, y: u16, script: &str) {
    if !map
        .events
        .coord_events
        .iter()
        .any(|e| (e.x, e.y) == (x, y) && e.script_name == script)
    {
        map.events.coord_events.push(CoordEvent {
            x,
            y,
            scene_id: "-1".into(),
            script_name: script.into(),
        });
    }
}

fn ensure_object_callback(map: &mut MapModule, script_name: &str) -> Result<()> {
    if !map
        .map_script_section_commands
        .iter()
        .any(|row| row.command == "def_callbacks")
    {
        map.map_script_section_commands
            .push(MapScriptSectionCommand {
                command: "def_callbacks".into(),
                args: Vec::new(),
                command_index: map.map_script_section_commands.len(),
            });
    }
    if !map.map_script_section_commands.iter().any(|row| {
        row.command == "callback"
            && row.args.first().map(String::as_str) == Some("MAPCALLBACK_OBJECTS")
            && row.args.get(1).map(String::as_str) == Some(script_name)
    }) {
        map.map_script_section_commands
            .push(MapScriptSectionCommand {
                command: "callback".into(),
                args: vec!["MAPCALLBACK_OBJECTS".into(), script_name.into()],
                command_index: map.map_script_section_commands.len(),
            });
    }
    for (index, row) in map.map_script_section_commands.iter_mut().enumerate() {
        row.command_index = index;
    }
    Ok(())
}

fn scene(map: &mut MapModule, script: &str, name: &str) -> Result<()> {
    ensure!(map.scenes.scenes.is_empty(), "occupied lab scene table");
    map.scenes.scenes.push(MapScene {
        scene_id: name.into(),
        script_name: Some(script.into()),
    });
    map.map_script_section_commands.insert(
        1,
        MapScriptSectionCommand {
            command: "scene_script".into(),
            args: vec![script.into(), name.into()],
            command_index: 1,
        },
    );
    for (index, row) in map.map_script_section_commands.iter_mut().enumerate() {
        row.command_index = index;
    }
    Ok(())
}

pub(crate) fn home(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = map(pack, "MnWinonaHome1F")?;
    apply_home_1f_scripts(&mut room)?;
    publish(pack, room)
}

pub(crate) fn apply_home_1f_scripts(room: &mut MapModule) -> Result<()> {
    ensure_object_callback(room, "MnWHomeGearCallback")?;
    put(
        room,
        "MnWHomeGearCallback",
        vec![
            c("checkevent", &[HOME]),
            c("iffalse", &["MnWHomeGearCallbackDone"]),
            c("setflag", &["ENGINE_POKEGEAR"]),
            c("setflag", &["ENGINE_PHONE_CARD"]),
            c("setflag", &["ENGINE_MAP_CARD"]),
            c("endcallback", &[]),
        ],
    );
    put(room, "MnWHomeGearCallbackDone", vec![c("endcallback", &[])]);
    install_stair_approach(room);
    movement(room, "MnOverhaulMomApproaches", &[("UP", 2), ("RIGHT", 2)]);
    movement(room, "MnOverhaulMomReturns", &[("LEFT", 2), ("DOWN", 2)]);
    install_door_approaches(room);
    movement(room, "MnOverhaulMomDoorRightApproaches", &[("DOWN", 1)]);
    movement(room, "MnOverhaulMomDoorRightReturns", &[("UP", 1)]);
    authored_dialogue::install_canonical(room, "MnWHomeGearReceivedText")?;
    put(
        room,
        "MnWGrantPokegear",
        vec![
            c("opentext", &[]),
            c("writetext", &["MnWHomeIntroText"]),
            c("promptbutton", &[]),
            c("playsound", &["SFX_ITEM"]),
            c("waitsfx", &[]),
            c("setflag", &["ENGINE_POKEGEAR"]),
            c("setflag", &["ENGINE_PHONE_CARD"]),
            c("setflag", &["ENGINE_MAP_CARD"]),
            c("writetext", &["MnWHomeGearReceivedText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[]),
        ],
    );
    put(
        room,
        "MnOverhaulMomScene",
        vec![
            c("checkevent", &[HOME]),
            c("iftrue", &["MnWHomeNoop"]),
            c("showemote", &["EMOTE_SHOCK", "MN_W_GUARDIAN", "15"]),
            c(
                "applymovement",
                &["MN_W_GUARDIAN", "MnOverhaulMomApproaches"],
            ),
            c("faceobject", &["MN_W_GUARDIAN", "PLAYER"]),
            c("turnobject", &["PLAYER", "DOWN"]),
            c("scall", &["MnWGrantPokegear"]),
            c("applymovement", &["MN_W_GUARDIAN", "MnOverhaulMomReturns"]),
            c("turnobject", &["MN_W_GUARDIAN", "LEFT"]),
            c("setevent", &[HOME]),
            c("setscene", &["SCENE_MN_W_HOME_READY"]),
            c("end", &[]),
        ],
    );
    put(
        room,
        "MnOverhaulMomDoorLeft",
        vec![
            c("checkevent", &[HOME]),
            c("iftrue", &["MnWHomeNoop"]),
            c("showemote", &["EMOTE_SHOCK", "MN_W_GUARDIAN", "15"]),
            c(
                "applymovement",
                &["MN_W_GUARDIAN", "MnOverhaulMomDoorLeftApproaches"],
            ),
            c("faceobject", &["MN_W_GUARDIAN", "PLAYER"]),
            c("turnobject", &["PLAYER", "UP"]),
            c("scall", &["MnWGrantPokegear"]),
            c(
                "applymovement",
                &["MN_W_GUARDIAN", "MnOverhaulMomDoorLeftReturns"],
            ),
            c("turnobject", &["MN_W_GUARDIAN", "LEFT"]),
            c("setevent", &[HOME]),
            c("setscene", &["SCENE_MN_W_HOME_READY"]),
            c("end", &[]),
        ],
    );
    put(
        room,
        "MnOverhaulMomDoorRight",
        vec![
            c("checkevent", &[HOME]),
            c("iftrue", &["MnWHomeNoop"]),
            c("showemote", &["EMOTE_SHOCK", "MN_W_GUARDIAN", "15"]),
            c(
                "applymovement",
                &["MN_W_GUARDIAN", "MnOverhaulMomDoorRightApproaches"],
            ),
            c("faceobject", &["MN_W_GUARDIAN", "PLAYER"]),
            c("turnobject", &["PLAYER", "UP"]),
            c("scall", &["MnWGrantPokegear"]),
            c(
                "applymovement",
                &["MN_W_GUARDIAN", "MnOverhaulMomDoorRightReturns"],
            ),
            c("turnobject", &["MN_W_GUARDIAN", "LEFT"]),
            c("setevent", &[HOME]),
            c("setscene", &["SCENE_MN_W_HOME_READY"]),
            c("end", &[]),
        ],
    );
    put(
        room,
        "MnOverhaulMomTalk",
        vec![
            c("faceplayer", &[]),
            c("scall", &["MnWGrantPokegear"]),
            c("setevent", &[HOME]),
            c("setscene", &["SCENE_MN_W_HOME_READY"]),
            c("end", &[]),
        ],
    );
    let guardian = room
        .scripts
        .get_mut("MN_W_GUARDIAN")
        .and_then(Value::as_array_mut)
        .context("Mom interaction")?;
    for row in guardian.iter_mut() {
        if *row == c("iffalse", &["MnWHomeArrival"]) {
            *row = c("iffalse", &["MnOverhaulMomTalk"]);
        }
    }
    coord(room, 6, 6, "MnOverhaulMomDoorLeft");
    coord(room, 7, 6, "MnOverhaulMomDoorRight");
    Ok(())
}

pub(crate) fn install_stair_approach(room: &mut MapModule) {
    // A stair warp may land directly on the trigger, so dispatch the same
    // scene after arrival as well as when the player steps onto the tile.
    put(
        room,
        "MnWHomeArrival",
        vec![
            c("readvar", &["VAR_XCOORD"]),
            c("ifnotequal", &["9", "MnWHomeNoop"]),
            c("readvar", &["VAR_YCOORD"]),
            c("ifnotequal", &["1", "MnWHomeNoop"]),
            c("sjump", &["MnOverhaulMomScene"]),
        ],
    );
    coord(room, 9, 1, "MnOverhaulMomScene");
}

pub(crate) fn install_door_approaches(room: &mut MapModule) {
    movement(
        room,
        "MnOverhaulMomDoorLeftApproaches",
        &[("DOWN", 1), ("LEFT", 1)],
    );
    movement(
        room,
        "MnOverhaulMomDoorLeftReturns",
        &[("RIGHT", 1), ("UP", 1)],
    );
    for (x, name, approach, returns) in [
        (
            6,
            "Left",
            vec![("DOWN", 1), ("LEFT", 1), ("DOWN", 1)],
            vec![("UP", 1), ("RIGHT", 1), ("UP", 1)],
        ),
        (7, "Right", vec![("DOWN", 2)], vec![("UP", 2)]),
    ] {
        let script = format!("MnOverhaulMomExit{name}");
        let approach_label = format!("{script}Approaches");
        let return_label = format!("{script}Returns");
        movement(room, &approach_label, &approach);
        movement(room, &return_label, &returns);
        put(
            room,
            &script,
            vec![
                c("checkevent", &[HOME]),
                c("iftrue", &["MnWHomeNoop"]),
                c("showemote", &["EMOTE_SHOCK", "MN_W_GUARDIAN", "15"]),
                c("applymovement", &["MN_W_GUARDIAN", &approach_label]),
                c("faceobject", &["MN_W_GUARDIAN", "PLAYER"]),
                c("turnobject", &["PLAYER", "UP"]),
                c("scall", &["MnWGrantPokegear"]),
                c("applymovement", &["MN_W_GUARDIAN", &return_label]),
                c("turnobject", &["MN_W_GUARDIAN", "LEFT"]),
                c("setevent", &[HOME]),
                c("setscene", &["SCENE_MN_W_HOME_READY"]),
                c("end", &[]),
            ],
        );
        coord(room, x, 7, &script);
        if let Some(event) = room
            .events
            .coord_events
            .iter_mut()
            .find(|event| event.x == x && event.y == 7 && event.script_name == script)
        {
            event.scene_id = "SCENE_MN_W_HOME_INTRO".into();
        }
    }
}

pub(crate) fn install_home_2f(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = map(pack, "MnWinonaHome2F")?;
    ensure!(
        room.objects
            .iter()
            .any(|o| o.script == "MN_SMALL_INTERIOR_WINONA_COUSIN"),
        "Winona bedroom cousin actor required"
    );
    let stages = [
        (
            "EVENT_MN_LEAGUE_CHAMPION",
            "MnWinonaCousinChampion",
            "MnWinonaCousinChampionText",
        ),
        (
            "EVENT_MN_SURVEY_RESTORED",
            "MnWinonaCousinRestored",
            "MnWinonaCousinRestoredText",
        ),
        (
            "EVENT_MN_DEPOT_CLEARED",
            "MnWinonaCousinIronRange",
            "MnWinonaCousinIronRangeText",
        ),
        (
            "EVENT_MN_BRAINERD_REPORT",
            "MnWinonaCousinBrainerd",
            "MnWinonaCousinBrainerdText",
        ),
        (
            "EVENT_MN_COURIER_DEFEATED",
            "MnWinonaCousinStPaul",
            "MnWinonaCousinStPaulText",
        ),
        (
            "EVENT_MN_ICE_CAPTIVE_RESCUED",
            "MnWinonaCousinMinneapolis",
            "MnWinonaCousinMinneapolisText",
        ),
        (
            "EVENT_MN_DELIVERED_ROCHESTER_PARCEL",
            "MnWinonaCousinRochester",
            "MnWinonaCousinRochesterText",
        ),
        (
            "EVENT_MN_STARTER_RECEIVED",
            "MnWinonaCousinStarter",
            "MnWinonaCousinStarterText",
        ),
    ];
    let mut dispatch = vec![c("faceplayer", &[])];
    for (flag, branch, text_label) in stages {
        dispatch.push(c("checkevent", &[flag]));
        dispatch.push(c("iftrue", &[branch]));
        put(&mut room, branch, vec![c("jumptext", &[text_label])]);
        authored_dialogue::install_canonical(&mut room, text_label)?;
    }
    dispatch.push(c(
        "jumptext",
        &["MN_SMALL_INTERIOR_WINONA_COUSINInteriorText"],
    ));
    authored_dialogue::install_canonical(&mut room, "MN_SMALL_INTERIOR_WINONA_COUSINInteriorText")?;
    put(&mut room, "MN_SMALL_INTERIOR_WINONA_COUSIN", dispatch);
    publish(pack, room)
}

pub(crate) fn install_winona_town(pack: &mut CompiledGamePack) -> Result<()> {
    crate::northstar_lair_authoring::flag(pack, TRAIL_OPENED_FLAG)?;
    let mut town = map(pack, "MnWinona")?;
    ensure!(
        town.objects
            .iter()
            .any(|o| o.object_identifier.as_deref() == Some("MN_W_TRAIL_WARDEN_NORTH"))
            && town
                .objects
                .iter()
                .any(|o| o.object_identifier.as_deref() == Some("MN_W_TRAIL_WARDEN_SOUTH")),
        "Winona trail wardens required"
    );
    for object in &mut town.objects {
        if matches!(
            object.object_identifier.as_deref(),
            Some("MN_W_TRAIL_WARDEN_NORTH" | "MN_W_TRAIL_WARDEN_SOUTH")
        ) {
            object.event_flag = "-1".into();
            object.script = "MnWTrailWardenTalk".into();
        }
    }
    ensure_object_callback(&mut town, "MnWTrailWardenCallback")?;
    put(&mut town, "MnWTrailWardenNoop", vec![c("end", &[])]);
    put(
        &mut town,
        "MnWTrailWardenCallback",
        vec![
            c("checkevent", &[HOME]),
            c("iffalse", &["MnWTrailWardenCheckStarter"]),
            c("setflag", &["ENGINE_POKEGEAR"]),
            c("setflag", &["ENGINE_PHONE_CARD"]),
            c("setflag", &["ENGINE_MAP_CARD"]),
            c("sjump", &["MnWTrailWardenCheckStarter"]),
        ],
    );
    put(
        &mut town,
        "MnWTrailWardenCheckStarter",
        vec![
            c("checkevent", &[STARTER]),
            c("iffalse", &["MnWTrailWardenCallbackDone"]),
            c("setevent", &[TRAIL_OPENED_FLAG]),
            c("moveobject", &["MN_W_TRAIL_WARDEN_NORTH", "5", "17"]),
            c("moveobject", &["MN_W_TRAIL_WARDEN_SOUTH", "6", "17"]),
            c("turnobject", &["MN_W_TRAIL_WARDEN_NORTH", "DOWN"]),
            c("turnobject", &["MN_W_TRAIL_WARDEN_SOUTH", "DOWN"]),
            c("endcallback", &[]),
        ],
    );
    put(
        &mut town,
        "MnWTrailWardenCallbackDone",
        vec![c("endcallback", &[])],
    );
    movement(&mut town, "MnWTrailWardenStepBack", &[("RIGHT", 1)]);
    authored_dialogue::install_canonical(&mut town, "MnWTrailWardenText")?;
    authored_dialogue::install_canonical(&mut town, "MnWTrailWardenClearedText")?;
    put(
        &mut town,
        "MnWTrailWardenNorth",
        vec![
            c("checkevent", &[STARTER]),
            c("iftrue", &["MnWTrailWardenNoop"]),
            c("turnobject", &["MN_W_TRAIL_WARDEN_NORTH", "RIGHT"]),
            c("turnobject", &["PLAYER", "LEFT"]),
            c("opentext", &[]),
            c("writetext", &["MnWTrailWardenText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("applymovement", &["PLAYER", "MnWTrailWardenStepBack"]),
            c("end", &[]),
        ],
    );
    put(
        &mut town,
        "MnWTrailWardenSouth",
        vec![
            c("checkevent", &[STARTER]),
            c("iftrue", &["MnWTrailWardenNoop"]),
            c("turnobject", &["MN_W_TRAIL_WARDEN_SOUTH", "RIGHT"]),
            c("turnobject", &["PLAYER", "LEFT"]),
            c("opentext", &[]),
            c("writetext", &["MnWTrailWardenText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("applymovement", &["PLAYER", "MnWTrailWardenStepBack"]),
            c("end", &[]),
        ],
    );
    put(
        &mut town,
        "MnWTrailWardenTalk",
        vec![
            c("faceplayer", &[]),
            c("checkevent", &[STARTER]),
            c("iftrue", &["MnWTrailWardenCleared"]),
            c("jumptext", &["MnWTrailWardenText"]),
        ],
    );
    put(
        &mut town,
        "MnWTrailWardenCleared",
        vec![c("jumptext", &["MnWTrailWardenClearedText"])],
    );
    put(&mut town, "MnWTrailCheck", vec![c("end", &[])]);
    put(&mut town, "MnWTrailCheckInward", vec![c("end", &[])]);
    town.events
        .coord_events
        .retain(|e| !matches!((e.x, e.y), (3, 18) | (3, 19) | (1, 18) | (1, 19)));
    coord(&mut town, 3, 18, "MnWTrailWardenNorth");
    coord(&mut town, 3, 19, "MnWTrailWardenSouth");
    publish(pack, town)
}

fn station(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = map(pack, "MnWinonaFieldStation")?;
    movement(&mut room, "MnOverhaulEnterLab", &[("UP", 8)]);
    movement(&mut room, "MnOverhaulLabStepUp", &[("UP", 1)]);
    movement(&mut room, "MnOverhaulLabStepLeft", &[("LEFT", 1)]);
    let entry = vec![
        c("checkevent", &[INTRO]),
        c("iftrue", &["MnNSEEnd"]),
        c("checkevent", &[HOME]),
        c("iffalse", &["MnNSEEnd"]),
        c("readvar", &["VAR_YCOORD"]),
        c("ifequal", &["11", "MnOverhaulLabLowerCarpet"]),
        c("sjump", &["MnOverhaulLabAisle"]),
    ];
    put(
        &mut room,
        "MnOverhaulLabLowerCarpet",
        vec![
            c("applymovement", &["PLAYER", "MnOverhaulLabStepUp"]),
            c("sjump", &["MnOverhaulLabAisle"]),
        ],
    );
    put(
        &mut room,
        "MnOverhaulLabAisle",
        vec![
            c("readvar", &["VAR_XCOORD"]),
            c("ifequal", &["5", "MnOverhaulLabRightCarpet"]),
            c("sjump", &["MnOverhaulLabWalk"]),
        ],
    );
    put(
        &mut room,
        "MnOverhaulLabRightCarpet",
        vec![
            c("applymovement", &["PLAYER", "MnOverhaulLabStepLeft"]),
            c("sjump", &["MnOverhaulLabWalk"]),
        ],
    );
    let mut walk = vec![
        c("applymovement", &["PLAYER", "MnOverhaulEnterLab"]),
        c("turnobject", &["PLAYER", "RIGHT"]),
        c("faceobject", &[DOCTOR, "PLAYER"]),
    ];
    walk.extend(say("MnOverhaulTamarackIntroText"));
    walk.extend([c("setevent", &[INTRO]), c("end", &[])]);
    put(&mut room, "MnOverhaulLabWalk", walk);
    put(&mut room, "MnOverhaulLabEntry", entry);
    scene(&mut room, "MnOverhaulLabEntry", "SCENE_MN_OVERHAUL_LAB")?;
    pack.data
        .story_event_script_constants
        .maps
        .entry(room.id.clone())
        .or_default()
        .insert("SCENE_MN_OVERHAUL_LAB".into(), 0);

    for (index, actor) in ["MN_W_CHIKORITA", "MN_W_CYNDAQUIL", "MN_W_TOTODILE"]
        .iter()
        .enumerate()
    {
        let mut body = room
            .scripts
            .get(*actor)
            .and_then(Value::as_array)
            .context("installed starter program")?
            .clone();
        let give = body
            .iter()
            .find(|row| row["command"] == "givepoke")
            .context("regional starter gift")?;
        let species = give["args"][0]
            .as_str()
            .context("regional starter")?
            .to_owned();
        ensure!(
            species != "CHIKORITA" && species != "CYNDAQUIL" && species != "TOTODILE",
            "Minnesota starter binding required"
        );
        let description = format!("MnOverhaulOtherStarter{index}");
        body[1] = c("iftrue", &[&description]);
        put(
            &mut room,
            &description,
            vec![c("jumptext", &[&format!("{description}Text")])],
        );
        let open = body
            .iter()
            .position(|row| row["command"] == "opentext")
            .context("starter confirmation")?;
        body.splice(
            open..open,
            [
                c("pokepic", &[&species]),
                c("cry", &[&species]),
                c("waitbutton", &[]),
                c("closepokepic", &[]),
            ],
        );
        let set = body
            .iter()
            .position(|row| row == &c("setevent", &[STARTER]))
            .context("starter completion")?;
        body.splice(
            set..set,
            [
                c("opentext", &[]),
                c("writetext", &["MnOverhaulStarterReceivedText"]),
                c("playsound", &["SFX_CAUGHT_MON"]),
                c("waitsfx", &[]),
                c("promptbutton", &[]),
                c("closetext", &[]),
            ],
        );
        let path = format!("MnOverhaulStarterReturn{index}");
        movement(&mut room, &path, &[("LEFT", index + 2), ("UP", 2)]);
        body.pop();
        body.extend([
            c("applymovement", &["PLAYER", &path]),
            c("turnobject", &["PLAYER", "RIGHT"]),
            c("faceobject", &[DOCTOR, "PLAYER"]),
            c("sjump", &["MnWFollowup"]),
        ]);
        put(&mut room, actor, body);
    }

    // The player now stands beside Tamarack; Reed comes to that conversation.
    movement(&mut room, "MnNSEReedEnters", &[("UP", 6)]);
    movement(&mut room, "MnNSEReedLeaves", &[("DOWN", 7)]);
    if let Some(body) = room
        .scripts
        .get_mut("MnNSEReedEncounter")
        .and_then(Value::as_array_mut)
    {
        let at = body
            .iter()
            .position(|row| *row == c("appear", &["MN_NSE_REED"]))
            .context("Reed arrival")?;
        body.insert(at + 1, c("playmusic", &["MUSIC_RIVAL_ENCOUNTER"]));
        for row in body.iter_mut() {
            if *row == c("turnobject", &["PLAYER", "LEFT"]) {
                *row = c("turnobject", &["PLAYER", "DOWN"]);
            }
        }
        // Reed puts his case down before Tamarack goes to collect it.
        let at = body
            .iter()
            .position(|row| row == &c("appear", &["MN_NSE_EGG_CASE"]))
            .context("egg placement")?;
        body.splice(
            at..at,
            [c(
                "applymovement",
                &["MN_NSE_REED", "MnOverhaulReedPutsCase"],
            )],
        );
        let at = body
            .iter()
            .position(|row| *row == c("appear", &["MN_NSE_EGG_CASE"]))
            .context("placed egg case")?;
        body.splice(
            at + 1..at + 1,
            [
                c("faceobject", &["MN_NSE_REED", DOCTOR]),
                c("faceobject", &[DOCTOR, "MN_NSE_REED"]),
            ],
        );
        let at = body
            .iter()
            .position(|row| row == &c("applymovement", &[DOCTOR, "MnNSEDoctorCollects"]))
            .context("doctor collects case")?;
        body.insert(
            at,
            c(
                "applymovement",
                &["MN_NSE_REED", "MnOverhaulReedStepsAside"],
            ),
        );
    }
    let body = room
        .scripts
        .get_mut("MnNSEReedEncounter")
        .and_then(Value::as_array_mut)
        .context("rival scene")?;
    let at = body
        .iter()
        .position(|row| *row == c("disappear", &["MN_NSE_REED"]))
        .context("Reed departure")?;
    body.insert(at + 1, c("playmapmusic", &[]));
    let at = body
        .iter()
        .position(|row| *row == c("writetext", &["MnNSEReedChallengeText"]))
        .context("rival challenge")?;
    body.splice(
        at - 1..at - 1,
        [
            c(
                "applymovement",
                &["MN_NSE_REED", "MnOverhaulReedChallenges"],
            ),
            c("faceobject", &["MN_NSE_REED", "PLAYER"]),
            c("turnobject", &["PLAYER", "DOWN"]),
        ],
    );
    let at = body
        .iter()
        .position(|row| *row == c("loadtrainer", &["RIVAL1", "MN_REED_WINONA"]))
        .context("rival trainer identity")?;
    body.insert(at, c("setlasttalked", &["MN_NSE_REED"]));
    let at = body
        .iter()
        .position(|row| *row == c("writetext", &["MnNSETamarackQuestText"]))
        .context("departure reminder")?;
    body.insert(at - 1, c("turnobject", &["PLAYER", "RIGHT"]));
    movement(
        &mut room,
        "MnOverhaulReedPutsCase",
        &[("RIGHT", 1), ("DOWN", 1)],
    );
    movement(&mut room, "MnOverhaulReedStepsAside", &[("LEFT", 1)]);
    movement(&mut room, "MnOverhaulReedChallenges", &[("UP", 1)]);

    for (script, text, sound) in [
        ("MnWSupplies", "MnWBallsText", "SFX_ITEM"),
        ("MnNSEEggOffer", "MnNSEEggGivenText", "SFX_GET_EGG"),
    ] {
        let body = room
            .scripts
            .get_mut(script)
            .and_then(Value::as_array_mut)
            .context("opening receipt")?;
        let at = body
            .iter()
            .position(|row| *row == c("writetext", &[text]))
            .context("opening receipt text")?;
        body.splice(
            at + 1..at + 1,
            [c("playsound", &[sound]), c("waitsfx", &[])],
        );
    }
    // Keep supplies retryable through the aide and preserve later gift services.
    for label in [
        "MnWReminder",
        "MnWTamarackDeliveryReturn",
        "MnWTamarackSurveyReturn",
    ] {
        if let Some(body) = room.scripts.get_mut(label).and_then(Value::as_array_mut) {
            body.retain(|row| *row != c("scall", &["MnWSupplies"]));
        }
    }

    let mut changed = BTreeSet::new();
    apply_field_station_upgrades(&mut room, &mut changed)?;
    publish(pack, room)
}

pub(crate) fn apply_field_station_upgrades(
    room: &mut MapModule,
    changed: &mut BTreeSet<String>,
) -> Result<()> {
    ensure_object_callback(room, "MnWStationSyncCallback")?;
    put(
        room,
        "MnWStationSyncCallback",
        vec![
            c("checkevent", &[PARCEL]),
            c("iffalse", &["MnWStationSyncDone"]),
            c("setflag", &["ENGINE_POKEDEX"]),
            c("endcallback", &[]),
        ],
    );
    changed.insert("MnWStationSyncCallback".into());
    put(room, "MnWStationSyncDone", vec![c("endcallback", &[])]);
    changed.insert("MnWStationSyncDone".into());

    authored_dialogue::install_canonical(room, "MnWPokedexReceivedText")?;
    changed.insert("MnWPokedexReceivedText".into());

    // Only Tamarack and the completed starter scene can perform this handover.
    put(
        room,
        "MnWFollowup",
        vec![
            c("checkevent", &[PARCEL]),
            c("iftrue", &["MnWReminder"]),
            c("faceobject", &[DOCTOR, "PLAYER"]),
            c("opentext", &[]),
            c("writetext", &["MnWReadingsText"]),
            c("promptbutton", &[]),
            c("playsound", &["SFX_DEX_FANFARE_80_109"]),
            c("waitsfx", &[]),
            c("setflag", &["ENGINE_POKEDEX"]),
            c("setevent", &[PARCEL]),
            c("writetext", &["MnWPokedexReceivedText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("scall", &["MnNSEReedEncounter"]),
            c("end", &[]),
        ],
    );
    changed.insert("MnWFollowup".into());

    // Ensure Reed stays at (4, 3) adjacent to the player at (4, 2) after reloadmapafterbattle.
    if let Some(body) = room
        .scripts
        .get_mut("MnNSEReedEncounter")
        .and_then(Value::as_array_mut)
    {
        if let Some(reload_idx) = body
            .iter()
            .position(|row| *row == c("reloadmapafterbattle", &[]))
        {
            let post_battle_move = c("moveobject", &["MN_NSE_REED", "4", "3"]);
            let search_end = (reload_idx + 5).min(body.len());
            if !body[reload_idx..search_end].contains(&post_battle_move)
                && let Some(rel_appear) = body[reload_idx..]
                    .iter()
                    .position(|row| *row == c("appear", &["MN_NSE_REED"]))
            {
                let appear_idx = reload_idx + rel_appear;
                body.splice(
                    appear_idx..=appear_idx,
                    [
                        post_battle_move,
                        c("appear", &["MN_NSE_REED"]),
                        c("faceobject", &["MN_NSE_REED", "PLAYER"]),
                        c("turnobject", &["PLAYER", "DOWN"]),
                    ],
                );
            }
        }
        changed.insert("MnNSEReedEncounter".into());
    }

    // Aide interception happens on departure at either (4, 9) or (5, 9).
    movement(room, "MnOverhaulAideApproaches", &[("RIGHT", 1)]);
    changed.insert("MnOverhaulAideApproaches".into());
    movement(room, "MnOverhaulAideReturns", &[("LEFT", 1)]);
    changed.insert("MnOverhaulAideReturns".into());
    movement(room, "MnOverhaulAideRightApproaches", &[("RIGHT", 2)]);
    changed.insert("MnOverhaulAideRightApproaches".into());
    movement(room, "MnOverhaulAideRightReturns", &[("LEFT", 2)]);
    changed.insert("MnOverhaulAideRightReturns".into());
    put(
        room,
        "MnOverhaulAideScene",
        vec![
            c("checkevent", &[PARCEL]),
            c("iffalse", &["MnNSEEnd"]),
            c("checkevent", &[BALLS]),
            c("iftrue", &["MnNSEEnd"]),
            c("applymovement", &["MN_W_AIDE", "MnOverhaulAideApproaches"]),
            c("faceobject", &["MN_W_AIDE", "PLAYER"]),
            c("turnobject", &["PLAYER", "LEFT"]),
            c("scall", &["MnWSupplies"]),
            c("applymovement", &["MN_W_AIDE", "MnOverhaulAideReturns"]),
            c("end", &[]),
        ],
    );
    changed.insert("MnOverhaulAideScene".into());
    put(
        room,
        "MnOverhaulAideRightScene",
        vec![
            c("checkevent", &[PARCEL]),
            c("iffalse", &["MnNSEEnd"]),
            c("checkevent", &[BALLS]),
            c("iftrue", &["MnNSEEnd"]),
            c(
                "applymovement",
                &["MN_W_AIDE", "MnOverhaulAideRightApproaches"],
            ),
            c("faceobject", &["MN_W_AIDE", "PLAYER"]),
            c("turnobject", &["PLAYER", "LEFT"]),
            c("scall", &["MnWSupplies"]),
            c(
                "applymovement",
                &["MN_W_AIDE", "MnOverhaulAideRightReturns"],
            ),
            c("end", &[]),
        ],
    );
    changed.insert("MnOverhaulAideRightScene".into());
    coord(room, 4, 9, "MnOverhaulAideScene");
    coord(room, 5, 9, "MnOverhaulAideRightScene");

    if let Some(dex_root) = room
        .scripts
        .get_mut("MnRewardDex")
        .and_then(Value::as_array_mut)
    {
        let gate = [
            c("checkevent", &[BALLS]),
            c("iffalse", &["MnRewardDexOriginal"]),
        ];
        if !dex_root.windows(2).any(|w| w == gate) {
            let insert_at = usize::from(dex_root.first() == Some(&c("faceplayer", &[])));
            dex_root.splice(insert_at..insert_at, gate);
            changed.insert("MnRewardDex".into());
        }
    }

    // Progressive guidance from Dr. Tamarack as milestones are completed.
    let guidance_stages = [
        (
            "EVENT_MN_LEAGUE_CHAMPION",
            "MnWTamarackGuideChampion",
            "MnWTamarackGuideChampionText",
        ),
        (
            "EVENT_MN_SURVEY_RESTORED",
            "MnWTamarackGuideRestored",
            "MnWTamarackGuideRestoredText",
        ),
        (
            "EVENT_MN_ARC_LINK_SENT",
            "MnWTamarackGuideIntake",
            "MnWTamarackGuideIntakeText",
        ),
        (
            "EVENT_MN_HARBOR_INSPECTED",
            "MnWTamarackGuideBoundary",
            "MnWTamarackGuideBoundaryText",
        ),
        (
            "EVENT_MN_DEPOT_CLEARED",
            "MnWTamarackGuideDuluth",
            "MnWTamarackGuideDuluthText",
        ),
        (
            "EVENT_MN_MINE_INSTRUMENT_RECOVERED",
            "MnWTamarackGuideDepot",
            "MnWTamarackGuideDepotText",
        ),
        (
            "EVENT_MN_HEADWATERS_REPORT",
            "MnWTamarackGuideMine",
            "MnWTamarackGuideMineText",
        ),
        (
            "EVENT_MN_BRAINERD_REPORT",
            "MnWTamarackGuideNorth",
            "MnWTamarackGuideNorthText",
        ),
        (
            "EVENT_MN_DEEP_BORE_MANIFEST_READ",
            "MnWTamarackGuideBrainerd",
            "MnWTamarackGuideBrainerdText",
        ),
        (
            "EVENT_MN_COURIER_DEFEATED",
            "MnWTamarackGuideManifest",
            "MnWTamarackGuideManifestText",
        ),
        (
            "EVENT_MN_ICE_CAPTIVE_RESCUED",
            "MnWTamarackGuideStPaul",
            "MnWTamarackGuideStPaulText",
        ),
    ];
    let mut post_egg_dispatch = vec![c("faceplayer", &[])];
    for (flag, branch, text_label) in guidance_stages {
        post_egg_dispatch.push(c("checkevent", &[flag]));
        post_egg_dispatch.push(c("iftrue", &[branch]));
        put(
            room,
            branch,
            vec![
                c("opentext", &[]),
                c("writetext", &[text_label]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("special", &["HealParty"]),
                c("checkevent", &["EVENT_MN_TAMARACK_EGG_RECEIVED"]),
                c("iftrue", &["MnNSEEnd"]),
                c("scall", &["MnNSEEggOffer"]),
                c("end", &[]),
            ],
        );
        authored_dialogue::install_canonical(room, text_label)?;
        changed.insert(branch.into());
        changed.insert(text_label.into());
    }
    post_egg_dispatch.extend([
        c("opentext", &[]),
        c("writetext", &["MnNSEEggAfterText"]),
        c("waitbutton", &[]),
        c("closetext", &[]),
        c("special", &["HealParty"]),
        c("end", &[]),
    ]);
    put(room, "MnNSETamarackPostEgg", post_egg_dispatch);
    changed.insert("MnNSETamarackPostEgg".into());
    put(
        room,
        "MnNSEEggAfter",
        vec![c("sjump", &["MnNSETamarackPostEgg"])],
    );
    changed.insert("MnNSEEggAfter".into());
    // The rival arc reaches its completed-story branch before the egg return.
    // Keep the reward wrapper, but let this branch use the current guidance too.
    put(
        room,
        "MN_W_TAMARACKArcReturn",
        vec![c("sjump", &["MnNSETamarackPostEgg"])],
    );
    changed.insert("MN_W_TAMARACKArcReturn".into());
    for label in [
        "MnWReminder",
        "MnWTamarackDeliveryReturn",
        "MnWTamarackSurveyReturn",
    ] {
        // Guidance follows the furthest completed milestone even if the player
        // declined the optional egg or reached milestones in a different order.
        let mut dispatch = Vec::new();
        for (flag, _, _) in guidance_stages {
            dispatch.push(c("checkevent", &[flag]));
            dispatch.push(c("iftrue", &["MnNSETamarackPostEgg"]));
        }
        dispatch.extend([
            c("checkevent", &["EVENT_MN_TAMARACK_EGG_RECEIVED"]),
            c("iftrue", &["MnNSETamarackPostEgg"]),
        ]);
        let body = room
            .scripts
            .get_mut(label)
            .and_then(Value::as_array_mut)
            .with_context(|| format!("Tamarack return branch {label}"))?;
        body.splice(0..0, dispatch);
        changed.insert(label.into());
    }
    Ok(())
}

pub(crate) fn install_r01_cliff_fossil(pack: &mut CompiledGamePack) -> Result<()> {
    let Some(mut bluff) = pack.data.maps.get("MnR01C00R00").cloned() else {
        return Ok(());
    };
    let Some(actor_index) = bluff
        .objects
        .iter()
        .position(|o| o.script == "MnFossilBluff")
    else {
        return Ok(());
    };
    bluff.objects[actor_index].x = 26;
    bluff.objects[actor_index].y = 14;
    bluff.objects[actor_index].spritemovedata = "SPRITEMOVEDATA_STANDING_DOWN".into();
    for label in [
        "MnFossilSlabText",
        "MnFossilLensChoiceText",
        "MnFossilLensConfirmText",
        "MnFossilSpireConfirmText",
        "MnFossilDirectionsText",
    ] {
        authored_dialogue::install_canonical(&mut bluff, label)?;
    }
    publish(pack, bluff)
}

pub(crate) fn compile_story_map_texts(
    room: &mut MapModule,
    story_root: &Value,
    map_name: &str,
    changed: &mut BTreeSet<String>,
) -> Result<()> {
    let Some(texts) = story_root["dialogue_overhaul"]["texts"][map_name].as_object() else {
        return Ok(());
    };
    for (label, raw_pages) in texts {
        let pages: Vec<AuthoredDialoguePage> = serde_json::from_value(raw_pages.clone())
            .with_context(|| format!("story dialogue {map_name}/{label}"))?;
        room.scripts
            .insert(label.clone(), authored_dialogue::compile(&pages)?);
        changed.insert(label.clone());
    }
    Ok(())
}
