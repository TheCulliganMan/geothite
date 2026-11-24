//! Directions at the first badge awards, the lake survey, and the Ely League.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use serde::Deserialize;
use serde_json::Value;

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::ely_league_scripts::CHAMPION;
use crate::grand_marais_modpack::authoring::command as c;
use crate::northstar_rival_arc::put;
use crate::{CompiledGamePack, MapModule, authored_town_publication, ely_league_rooms};
use crate::{northstar_lair_authoring, parse_script_audio_commands};

const HOME: &str = "MnWinonaHome1F";
const LOBBY: &str = "MnLeagueLobby";
const SUPPER: &str = "EVENT_MN_WINONA_CHAMPION_SUPPER";

#[derive(Deserialize)]
struct Story {
    journey_landmark_dialogue: BTreeMap<String, BTreeMap<String, Vec<AuthoredDialoguePage>>>,
}

/// Fill missing journey scenes and refresh older lake-country handoffs.
/// A Lakeside parent must already contain the other authored scenes.
pub(super) fn complete(pack: &mut CompiledGamePack) -> Result<()> {
    let has_journey_scenes = pack
        .data
        .story_event_script_constants
        .global
        .contains_key(SUPPER);
    if !has_journey_scenes {
        install(pack)?;
    }
    let story: Story =
        serde_json::from_str(include_str!("../../../planning/minnesota/story.json"))?;
    for (name, labels) in story.journey_landmark_dialogue {
        if has_journey_scenes && matches!(name.as_str(), "MnBrainerd" | "MnBemidji") {
            install_room(pack, &name, &labels)?;
        }
        let room = pack.data.maps.get(&name).context("missing journey map")?;
        for (label, pages) in labels {
            let expected = authored_dialogue::compile(&pages)?;
            ensure!(
                room.scripts.get(&label) == Some(&expected)
                    && pack.data.map_scripts.get(&label) == Some(&expected),
                "missing journey dialogue in {name}/{label}"
            );
        }
    }
    let lobby = &pack.data.maps[LOBBY];
    let registration = lobby
        .scripts
        .get("MnElyRegister")
        .and_then(Value::as_array)
        .context("Ely registration")?;
    for (i, flag) in BADGE_ENGINE_FLAGS[..8].iter().enumerate() {
        let branch = format!("MnElyMissingBadge{}", i + 1);
        let start = 1 + 2 * i;
        ensure!(
            registration.get(start..start + 2)
                == Some([c("checkflag", &[flag]), c("iffalse", &[&branch])].as_slice())
                && lobby.scripts.get(&branch)
                    == Some(&serde_json::json!([c(
                        "jumptext",
                        &[&format!("{branch}Text")]
                    )])),
            "Ely registrar cannot direct a player missing {flag}"
        );
    }
    let guardian = &pack.data.maps[HOME];
    ensure!(
        guardian
            .scripts
            .get("MN_W_GUARDIAN")
            .and_then(Value::as_array)
            .and_then(|s| s.get(3))
            == Some(&c("iftrue", &["MnWinonaSupper"])),
        "missing Winona homecoming"
    );
    Ok(())
}

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let story: Story =
        serde_json::from_str(include_str!("../../../planning/minnesota/story.json"))?;
    northstar_lair_authoring::flag(pack, SUPPER)?;
    for (id, labels) in story.journey_landmark_dialogue {
        install_room(pack, &id, &labels)?;
    }
    Ok(())
}

fn install_room(
    pack: &mut CompiledGamePack,
    id: &str,
    labels: &BTreeMap<String, Vec<AuthoredDialoguePage>>,
) -> Result<()> {
    let previous = pack
        .data
        .maps
        .get(id)
        .with_context(|| format!("story map {id}"))?;
    let mut room = previous.clone();
    match id {
        HOME => homecoming(&mut room)?,
        LOBBY => registrar(&mut room)?,
        "MnBrainerd" => ranger_handoff(&mut room)?,
        "MnBemidji" => officer_handoff(&mut room)?,
        _ => {}
    }
    for (label, pages) in labels {
        let program = authored_dialogue::compile(pages)
            .with_context(|| format!("native dialogue {id}/{label}"))?;
        if let Some(old) = previous.scripts.get(label) {
            ensure!(
                previous.script_text_bodies.contains_key(label)
                    && old.as_array().and_then(|rows| rows.last()) == Some(&c("done", &[])),
                "expected ordinary finished speech at {id}/{label}"
            );
        } else {
            ensure!(
                room.scripts
                    .values()
                    .filter_map(Value::as_array)
                    .flatten()
                    .any(|row| matches!(
                        row["command"].as_str(),
                        Some("writetext" | "jumptext" | "jumptextfaceplayer")
                    ) && row["args"][0] == label.as_str()),
                "new speech must have a speaker: {id}/{label}"
            );
        }
        room.scripts.insert(label.clone(), program);
    }
    if room.scripts != previous.scripts {
        ely_league_rooms::finish(pack, &mut room)?;
        room.script_audio_commands = parse_script_audio_commands(id, &room.scripts)?;
        room.map_event_section_commands
            .clone_from(&previous.map_event_section_commands);
        authored_town_publication::replace(pack, room)?;
    }
    Ok(())
}

fn replace_tail(map: &mut MapModule, label: &str, last: Value, tail: Vec<Value>) -> Result<()> {
    let script = map
        .scripts
        .get_mut(label)
        .and_then(Value::as_array_mut)
        .with_context(|| format!("{}/{label} script", map.id))?;
    ensure!(
        script.last() == Some(&last),
        "unexpected handoff in {}/{label}",
        map.id
    );
    script.pop();
    script.extend(tail);
    Ok(())
}

fn ranger_handoff(map: &mut MapModule) -> Result<()> {
    if map.scripts.contains_key("MnFlowRangerReading") {
        return Ok(());
    }
    let repeat = "MnBrainerdRangerRepeat";
    let text = "MnBrainerdRangerRepeatText";
    replace_tail(
        map,
        repeat,
        c("jumptextfaceplayer", &[text]),
        vec![
            c("checkevent", &[crate::itasca_events::READING]),
            c("iftrue", &["MnFlowRangerReading"]),
            c("jumptextfaceplayer", &[text]),
        ],
    )?;
    put(
        map,
        "MnFlowRangerReading",
        vec![c("jumptextfaceplayer", &["MnFlowRangerReadingText"])],
    );
    let before = "MnBrainerdRangerBeforeText";
    replace_tail(
        map,
        "MnBrainerdRangerBefore",
        c("jumptextfaceplayer", &[before]),
        vec![
            c("checkevent", &[crate::itasca_events::READING]),
            c("iftrue", &[repeat]),
            c("jumptextfaceplayer", &[before]),
        ],
    )?;
    for (label, text) in [
        ("MnBrainerdRangerGift", "MnBrainerdRangerGiftText"),
        ("MnBrainerdRangerFull", "MnBrainerdRangerFullText"),
    ] {
        replace_tail(
            map,
            label,
            c("jumptextfaceplayer", &[text]),
            vec![
                c("opentext", &[]),
                c("writetext", &[text]),
                c("promptbutton", &[]),
                c("closetext", &[]),
                c("sjump", &[repeat]),
            ],
        )?;
    }
    Ok(())
}

fn officer_handoff(map: &mut MapModule) -> Result<()> {
    if map.scripts.contains_key("MnFlowOfficerPine") {
        return Ok(());
    }
    let after = "MnBemidjiReportAfter";
    let text = "MnBemidjiReportAfterText";
    replace_tail(
        map,
        after,
        c("jumptextfaceplayer", &[text]),
        vec![
            c("checkflag", &["ENGINE_GLACIERBADGE"]),
            c("iffalse", &["MnFlowOfficerPine"]),
            c("checkevent", &["EVENT_MN_DEPOT_FOREMAN_DEFEATED"]),
            c("iftrue", &["MnFlowOfficerNell"]),
            c("jumptextfaceplayer", &[text]),
        ],
    )?;
    for (label, text) in [
        ("MnFlowOfficerPine", "MnFlowOfficerPineText"),
        ("MnFlowOfficerNell", "MnFlowOfficerNellText"),
    ] {
        put(map, label, vec![c("jumptextfaceplayer", &[text])]);
    }
    for label in ["MN_BEMIDJI_OFFICER", "MnBemidjiNotebookMatch"] {
        replace_tail(
            map,
            label,
            c("jumptext", &[text]),
            vec![c("sjump", &[after])],
        )?;
    }
    Ok(())
}

fn registrar(room: &mut MapModule) -> Result<()> {
    let registration = room
        .scripts
        .get_mut("MnElyRegister")
        .and_then(Value::as_array_mut)
        .context("Ely registration")?;
    // Substitute the destination of each existing refusal, preserving check order
    // and saved instruction positions through the registration and warp.
    for (index, flag) in BADGE_ENGINE_FLAGS[..8].iter().enumerate() {
        let start = 1 + index * 2;
        let pair = registration
            .get_mut(start..start + 2)
            .context("registrar badge test")?;
        ensure!(
            pair == [c("checkflag", &[flag]), c("iffalse", &["MnElyBadgeDenied"])],
            "unexpected registrar check for {flag}"
        );
        pair[1] = c("iffalse", &[&format!("MnElyMissingBadge{}", index + 1)]);
    }
    for number in 1..=8 {
        let label = format!("MnElyMissingBadge{number}");
        put(
            room,
            &label,
            vec![c("jumptext", &[&format!("{label}Text")])],
        );
    }
    Ok(())
}

fn homecoming(room: &mut MapModule) -> Result<()> {
    let talk = room
        .scripts
        .get_mut("MN_W_GUARDIAN")
        .and_then(Value::as_array_mut)
        .context("Winona guardian")?;
    let route = talk
        .get_mut(2..4)
        .context("guardian's survey-restored branch")?;
    ensure!(
        route
            == [
                c("checkevent", &["EVENT_MN_SURVEY_RESTORED"]),
                c("iftrue", &["MnWGuardianSurveyReturn"])
            ],
        "expected the rescue homecoming"
    );
    route[1] = c("iftrue", &["MnWinonaSupper"]);
    let rest = room
        .scripts
        .get("MnWGuardianSurveyReturn")
        .and_then(Value::as_array)
        .and_then(|rows| rows.get(4..))
        .context("Winona home rest and blackout destination")?;
    ensure!(
        rest == [
            c("special", &["HealParty"]),
            c("blackoutmod", &["MN_WINONA_HOME_1F"]),
            c("closetext", &[]),
            c("end", &[])
        ],
        "expected Mom's normal rest"
    );
    let rest = rest.to_vec();
    put(
        room,
        "MnWinonaSupper",
        vec![
            c("checkevent", &[CHAMPION]),
            c("iffalse", &["MnWGuardianSurveyReturn"]),
            c("faceplayer", &[]),
            c("opentext", &[]),
            c("checkevent", &[SUPPER]),
            c("iftrue", &["MnWinonaReturnForSupper"]),
            c("writetext", &["MnWinonaChampionSupperText"]),
            c("promptbutton", &[]),
            c("setevent", &[SUPPER]),
            c("sjump", &["MnWinonaRestAtHome"]),
        ],
    );
    put(
        room,
        "MnWinonaReturnForSupper",
        vec![
            c("writetext", &["MnWinonaChampionRestText"]),
            c("promptbutton", &[]),
            c("sjump", &["MnWinonaRestAtHome"]),
        ],
    );
    put(room, "MnWinonaRestAtHome", rest);
    Ok(())
}
