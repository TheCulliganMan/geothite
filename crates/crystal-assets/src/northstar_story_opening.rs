//! Reed's first on-screen choice and Tamarack's real egg-care reward.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use crystal_core::map::MapScriptSectionCommand;
use crystal_core::models::TrainerPartyPokemon;
use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::{CompiledGamePack, MapModule};

const LAB: &str = "MnWinonaFieldStation";
const REED: &str = "MN_NSE_REED";
const CASE: &str = "MN_NSE_EGG_CASE";
const MET: &str = "EVENT_MN_REED_FIRST_MET";
const WON: &str = "EVENT_MN_REED_WINONA_DEFEATED";
const EGG: &str = "EVENT_MN_TAMARACK_EGG_RECEIVED";
const TRAINER: &str = "MN_REED_WINONA";
const DOCTOR: &str = "MN_W_TAMARACK";

pub(crate) fn install(pack: &mut CompiledGamePack, source: &Value) -> Result<()> {
    ensure!(
        source["schema_version"] == 1
            && source["id"] == "northstar-events-v2"
            && source["rival"]["name"] == "REED"
            && source["rival"]["sprite"] == "SPRITE_RIVAL"
            && source["tamarack"]["egg"]["species"] == "GILLPUP"
            && source["tamarack"]["egg"]["level"] == 5,
        "supported event-driven Northstar opening required"
    );
    events(pack)?;
    trainer(pack)?;
    let mut room = pack
        .data
        .maps
        .get(LAB)
        .context("native Winona field station")?
        .clone();
    let texts: BTreeMap<String, Vec<String>> =
        serde_json::from_value(source["opening_texts"].clone())?;
    for (label, lines) in &texts {
        ensure!(
            label.starts_with("MnNSE")
                && label.ends_with("Text")
                && !room.scripts.contains_key(label)
                && !pack.data.map_scripts.contains_key(label)
                && !lines.is_empty()
                && lines.len() <= 8
                && lines.iter().all(|l| !l.is_empty()
                    && l.len() <= 18
                    && l.is_ascii()
                    && !l.chars().any(|c| c.is_control() || c == '"')),
            "invalid/occupied opening text: {label}"
        );
        authoring::text(
            &mut room,
            label,
            &lines.iter().map(String::as_str).collect::<Vec<_>>(),
        );
    }
    for label in [
        "MnNSEReedArrivalText",
        "MnNSEReedCaseText",
        "MnNSETamarackCaseText",
        "MnNSEReedDismissText",
        "MnNSETamarackWarmText",
        "MnNSEReedChallengeText",
        "MnNSEReedWonText",
        "MnNSEReedLostText",
        "MnNSEReedAfterText",
        "MnNSETamarackQuestText",
        "MnNSETamarackRestText",
        "MnNSEEggOfferText",
        "MnNSEEggDeclineText",
        "MnNSEEggFullText",
        "MnNSEEggGivenText",
        "MnNSEEggAfterText",
    ] {
        ensure!(texts.contains_key(label), "missing opening text {label}");
    }
    // Every path is checked as actual native floor, rather than assuming that
    // the lab's shelves, machine and occupied starter table can be crossed.
    for tile in [
        [4, 9],
        [4, 8],
        [4, 7],
        [4, 6],
        [4, 10],
        [5, 2],
        [5, 3],
        [5, 4],
        [5, 5],
        [5, 6],
        [5, 7],
    ] {
        ensure!(
            matches!(
                crate::authored_route_modpack::collision(
                    pack,
                    &room.blocks,
                    room.attributes.width,
                    tile,
                    "lab",
                )?,
                "FLOOR" | "WARP_CARPET_DOWN"
            ),
            "opening movement crosses scenery at {tile:?}"
        );
    }
    for (id, sprite, at) in [
        (REED, "SPRITE_RIVAL", [4, 9]),
        (CASE, "SPRITE_POKE_BALL", [5, 5]),
    ] {
        ensure!(
            !room.objects.iter().any(|o| [o.x, o.y] == at),
            "opening actor overlaps existing actor"
        );
        authoring::npc(pack, &mut room, id, sprite, at[0] as i16, at[1] as i16)?;
        let actor = room.objects.last_mut().context("new opening actor")?;
        actor.object_identifier = Some(id.into());
        if id == CASE {
            actor.sprite_has_facings = false;
            actor.spritemovedata = "SPRITEMOVEDATA_STILL".into();
        }
        put(&mut room, id, vec![c("end", &[])])?;
    }
    ensure!(
        !room
            .map_script_section_commands
            .iter()
            .any(|v| v.command == "callback"
                && v.args.first().is_some_and(|a| a == "MAPCALLBACK_OBJECTS")),
        "field station already has an object callback"
    );
    put(
        &mut room,
        "MnNSEHideVisitors",
        vec![
            c("disappear", &[REED]),
            c("disappear", &[CASE]),
            c("endcallback", &[]),
        ],
    )?;
    room.map_script_section_commands
        .push(MapScriptSectionCommand {
            command: "callback".into(),
            args: vec!["MAPCALLBACK_OBJECTS".into(), "MnNSEHideVisitors".into()],
            command_index: room.map_script_section_commands.len(),
        });
    for (label, direction, steps) in [
        ("MnNSEReedEnters", "UP", 3),
        ("MnNSEReedLeaves", "DOWN", 4),
        ("MnNSEDoctorCollects", "DOWN", 2),
        ("MnNSEDoctorReturns", "UP", 2),
    ] {
        let mut body = (0..steps)
            .map(|_| c("step", &[direction]))
            .collect::<Vec<_>>();
        body.push(c("step_end", &[]));
        put(&mut room, label, body)?;
    }
    let mut scene = vec![
        c("checkevent", &[MET]),
        c("iftrue", &["MnNSEEnd"]),
        c("setevent", &[MET]),
        c("appear", &[REED]),
        c("applymovement", &[REED, "MnNSEReedEnters"]),
        c("faceobject", &[REED, "PLAYER"]),
        c("turnobject", &["PLAYER", "LEFT"]),
    ];
    scene.extend(say("MnNSEReedArrivalText"));
    scene.extend(say("MnNSEReedCaseText"));
    scene.extend([c("faceobject", &[DOCTOR, REED])]);
    scene.extend(say("MnNSETamarackCaseText"));
    scene.push(c("appear", &[CASE]));
    scene.extend(say("MnNSEReedDismissText"));
    scene.extend([
        c("applymovement", &[DOCTOR, "MnNSEDoctorCollects"]),
        c("turnobject", &[DOCTOR, "DOWN"]),
        c("disappear", &[CASE]),
    ]);
    scene.extend(say("MnNSETamarackWarmText"));
    scene.extend([
        c("applymovement", &[DOCTOR, "MnNSEDoctorReturns"]),
        c("faceobject", &[DOCTOR, "PLAYER"]),
    ]);
    scene.extend(say("MnNSEReedChallengeText"));
    scene.extend([
        c("winlosstext", &["MnNSEReedWonText", "MnNSEReedLostText"]),
        c("loadtrainer", &["RIVAL1", TRAINER]),
        c("startbattle", &[]),
        c("reloadmapafterbattle", &[]),
        c("setevent", &[WON]),
        c("appear", &[REED]),
        c("faceobject", &[REED, "PLAYER"]),
    ]);
    scene.extend(say("MnNSEReedAfterText"));
    scene.extend([
        c("applymovement", &[REED, "MnNSEReedLeaves"]),
        c("disappear", &[REED]),
        c("faceobject", &[DOCTOR, "PLAYER"]),
    ]);
    scene.extend(say("MnNSETamarackQuestText"));
    scene.extend(say("MnNSETamarackRestText"));
    scene.push(c("special", &["HealParty"]));
    scene.push(c("end", &[]));
    put(&mut room, "MnNSEReedEncounter", scene)?;
    put(&mut room, "MnNSEEnd", vec![c("end", &[])])?;
    let offer = vec![
        c("checkevent", &["EVENT_MN_DELIVERED_ROCHESTER_PARCEL"]),
        c("iffalse", &["MnNSEEnd"]),
        c("checkevent", &[MET]),
        c("iffalse", &["MnNSEEnd"]),
        c("checkevent", &[EGG]),
        c("iftrue", &["MnNSEEggAfter"]),
        c("opentext", &[]),
        c("writetext", &["MnNSEEggOfferText"]),
        c("yesorno", &[]),
        c("iffalse", &["MnNSEEggDeclined"]),
        c("giveegg", &["GILLPUP", "5"]),
        c("ifnotequal", &["2", "MnNSEEggFull"]),
        c("setevent", &[EGG]),
        c("writetext", &["MnNSEEggGivenText"]),
        c("waitbutton", &[]),
        c("closetext", &[]),
        c("end", &[]),
    ];
    put(&mut room, "MnNSEEggOffer", offer)?;
    for (script, text) in [
        ("MnNSEEggDeclined", "MnNSEEggDeclineText"),
        ("MnNSEEggFull", "MnNSEEggFullText"),
        ("MnNSEEggAfter", "MnNSEEggAfterText"),
    ] {
        let mut body = say(text);
        body.push(c("end", &[]));
        put(&mut room, script, body)?;
    }
    for (root, target) in [
        ("MnWFollowup", "MnNSEReedEncounter"),
        ("MnWReminder", "MnNSEEggOffer"),
        ("MnWTamarackDeliveryReturn", "MnNSEEggOffer"),
        ("MnWTamarackSurveyReturn", "MnNSEEggOffer"),
    ] {
        let body = room
            .scripts
            .get_mut(root)
            .and_then(Value::as_array_mut)
            .context("original recoverable field-station handover")?;
        ensure!(
            body.len() >= 2
                && body[body.len() - 2] == c("scall", &["MnWSupplies"])
                && body.last() == Some(&c("end", &[])),
            "changed field-station supply tail: {root}"
        );
        body.insert(body.len() - 1, c("scall", &[target]));
    }
    crate::ely_league_rooms::finish(pack, &mut room)?;
    crate::authored_town_publication::replace(pack, room)?;
    let mut clinic = pack
        .data
        .maps
        .get("MnRochesterPokecenter1F")
        .context("Ivy's existing clinic scene")?
        .clone();
    let lines: Vec<String> =
        serde_json::from_value(source["quest_texts"]["MnRCDeliveryText"].clone())?;
    ensure!(
        lines.len() == 10
            && lines.iter().all(|l| !l.is_empty()
                && l.len() <= 18
                && l.is_ascii()
                && !l.chars().any(|c| c.is_control() || c == '"')),
        "complete Ivy quest pages required"
    );
    ensure!(
        clinic.script_text_bodies.contains_key("MnRCDeliveryText"),
        "existing Ivy handover required"
    );
    authoring::text(
        &mut clinic,
        "MnRCDeliveryText",
        &lines.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    crate::ely_league_rooms::finish(pack, &mut clinic)?;
    crate::authored_town_publication::replace(pack, clinic)?;
    Ok(())
}

fn say(label: &str) -> Vec<Value> {
    vec![
        c("opentext", &[]),
        c("writetext", &[label]),
        c("waitbutton", &[]),
        c("closetext", &[]),
    ]
}

fn put(room: &mut MapModule, label: &str, body: Vec<Value>) -> Result<()> {
    ensure!(
        room.scripts.insert(label.into(), json!(body)).is_none(),
        "event opening namespace collision: {label}"
    );
    Ok(())
}

fn events(pack: &mut CompiledGamePack) -> Result<()> {
    let globals = &mut pack.data.story_event_script_constants.global;
    for event in [MET, WON, EGG] {
        ensure!(
            !globals.contains_key(event),
            "event opening already present"
        );
        let next = globals
            .iter()
            .filter(|(name, _)| name.starts_with("EVENT_"))
            .map(|(_, id)| *id)
            .max()
            .context("existing native events")?
            .checked_add(1)
            .context("event identity overflow")?;
        globals.insert(event.into(), next);
    }
    Ok(())
}

fn trainer(pack: &mut CompiledGamePack) -> Result<()> {
    ensure!(
        !pack.data.trainers.trainers.contains_key(TRAINER),
        "Reed opening trainer occupied"
    );
    let mut trainer = pack
        .data
        .trainers
        .trainers
        .values()
        .find(|t| t.trainer_class == "RIVAL1")
        .context("native rival class")?
        .clone();
    let next = pack
        .data
        .trainers
        .trainers
        .values()
        .filter(|t| t.trainer_class == "RIVAL1")
        .map(|t| {
            pack.data
                .story_event_script_constants
                .global
                .get(&t.trainer_id)
                .copied()
                .context("native rival identity")
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .max()
        .context("existing rival parties")?
        .checked_add(1)
        .context("rival identity overflow")?;
    let dvs = trainer.party.first().context("native rival DVs")?.dvs;
    // This is the player's first battle, before any route training. Reed's
    // advantage comes later from ICE's backing, not an opening damage race
    // that a full-health starter can lose by selecting its ordinary attack.
    let pokemon = pack.data.create_pokemon("CROOKIT", 3, dvs)?;
    trainer.trainer_id = TRAINER.into();
    trainer.name = "REED".into();
    trainer.party = vec![TrainerPartyPokemon {
        species: "CROOKIT".into(),
        level: 3,
        item: None,
        moves: pokemon.moves,
        dvs,
    }];
    trainer.win_quote.clear();
    trainer.lose_quote.clear();
    pack.data.trainers.insert(trainer)?;
    pack.data
        .story_event_script_constants
        .global
        .insert(TRAINER.into(), next);
    Ok(())
}
