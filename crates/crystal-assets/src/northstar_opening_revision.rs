//! Staged family departure, professor introduction and starter handover.
use anyhow::{Context, Result, ensure};
use crystal_core::map::{CoordEvent, MapScene, MapScriptSectionCommand};
use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring::command as c;
use crate::northstar_lair_authoring::say;
use crate::northstar_story_revision::publish;
use crate::{CompiledGamePack, MapModule};

const HOME: &str = "EVENT_MN_HOME_INTRO";
const INTRO: &str = "EVENT_MN_TAMARACK_INTRO";
const STARTER: &str = "EVENT_MN_STARTER_RECEIVED";
const PARCEL: &str = "EVENT_MN_R01_PARCEL_RECEIVED";
const BALLS: &str = "EVENT_MN_WINONA_BALLS_RECEIVED";
const DOCTOR: &str = "MN_W_TAMARACK";

pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    crate::northstar_lair_authoring::flag(pack, INTRO)?;
    home(pack)?;
    station(pack)
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
    map.events.coord_events.push(CoordEvent {
        x,
        y,
        scene_id: "-1".into(),
        script_name: script.into(),
    });
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

fn home(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = map(pack, "MnWinonaHome1F")?;
    // Some supported stair arrivals include the step onto (9,1) inside the
    // warp animation. Dispatch that same tile scene once the arrival settles;
    // entering through the front door never starts a remote conversation.
    put(
        &mut room,
        "MnWHomeArrival",
        vec![
            c("readvar", &["VAR_XCOORD"]),
            c("ifnotequal", &["9", "MnWHomeNoop"]),
            c("readvar", &["VAR_YCOORD"]),
            c("ifnotequal", &["1", "MnWHomeNoop"]),
            c("sjump", &["MnOverhaulMomScene"]),
        ],
    );
    movement(
        &mut room,
        "MnOverhaulMomApproaches",
        &[("UP", 2), ("RIGHT", 2)],
    );
    movement(
        &mut room,
        "MnOverhaulMomReturns",
        &[("LEFT", 2), ("DOWN", 2)],
    );
    let mut body = vec![
        c("checkevent", &[HOME]),
        c("iftrue", &["MnWHomeNoop"]),
        c("showemote", &["EMOTE_SHOCK", "MN_W_GUARDIAN", "15"]),
        c(
            "applymovement",
            &["MN_W_GUARDIAN", "MnOverhaulMomApproaches"],
        ),
        c("faceobject", &["MN_W_GUARDIAN", "PLAYER"]),
        c("turnobject", &["PLAYER", "DOWN"]),
    ];
    body.extend(say("MnWHomeIntroText"));
    body.extend([
        c("applymovement", &["MN_W_GUARDIAN", "MnOverhaulMomReturns"]),
        c("turnobject", &["MN_W_GUARDIAN", "LEFT"]),
        c("setevent", &[HOME]),
        c("setscene", &["SCENE_MN_W_HOME_READY"]),
        c("end", &[]),
    ]);
    put(&mut room, "MnOverhaulMomScene", body);
    let mut talk = vec![c("faceplayer", &[])];
    talk.extend(say("MnWHomeIntroText"));
    talk.extend([
        c("setevent", &[HOME]),
        c("setscene", &["SCENE_MN_W_HOME_READY"]),
        c("end", &[]),
    ]);
    put(&mut room, "MnOverhaulMomTalk", talk);
    let guardian = room
        .scripts
        .get_mut("MN_W_GUARDIAN")
        .and_then(Value::as_array_mut)
        .context("Mom interaction")?;
    for row in guardian {
        if *row == c("iffalse", &["MnWHomeArrival"]) {
            *row = c("iffalse", &["MnOverhaulMomTalk"]);
        }
    }
    coord(&mut room, 9, 1, "MnOverhaulMomScene");
    publish(pack, room)
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
    // Only Tamarack and the completed starter scene can perform this handover.
    let mut handover = vec![
        c("checkevent", &[PARCEL]),
        c("iftrue", &["MnWReminder"]),
        c("faceobject", &[DOCTOR, "PLAYER"]),
    ];
    handover.extend(say("MnWReadingsText"));
    handover.extend([
        c("setevent", &[PARCEL]),
        c("playsound", &["SFX_ITEM"]),
        c("waitsfx", &[]),
        c("scall", &["MnNSEReedEncounter"]),
        c("end", &[]),
    ]);
    put(&mut room, "MnWFollowup", handover);

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
    // Aide interception happens on departure, with the original retryable grant.
    movement(&mut room, "MnOverhaulAideApproaches", &[("RIGHT", 1)]);
    movement(&mut room, "MnOverhaulAideReturns", &[("LEFT", 1)]);
    let mut supplies = vec![
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
    ];
    put(
        &mut room,
        "MnOverhaulAideScene",
        std::mem::take(&mut supplies),
    );
    coord(&mut room, 4, 9, "MnOverhaulAideScene");
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
    publish(pack, room)
}
