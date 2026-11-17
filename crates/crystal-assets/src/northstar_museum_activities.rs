//! Persistent mill routing and flood comparison using the existing exhibits.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring::command as c;
use crate::{CompiledGamePack, MapModule};

const MANIFEST: &str = "northstar-interactive-museums-v1";
const MILL_POWER: &str = "EVENT_MN_MILL_MODEL_POWER";
const MILL_GRAIN: &str = "EVENT_MN_MILL_MODEL_GRAIN";
const MILL_DONE: &str = "EVENT_MN_MILL_MODEL_COMPLETE";
const FLOOD_TODAY: &str = "EVENT_MN_FLOOD_CHART_TODAY";
const FLOOD_OLD: &str = "EVENT_MN_FLOOD_CHART_OLD";
const FLOOD_DONE: &str = "EVENT_MN_FLOOD_CHART_COMPLETE";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    dialogue: BTreeMap<String, BTreeMap<String, Vec<AuthoredDialoguePage>>>,
    menus: BTreeMap<String, Vec<String>>,
}

/// Adds distinct optional activities while retaining every existing prize claim.
///
/// The old clue-reading and full-Pack paths run unchanged. Their completed
/// return leads into these new activities, including for previously rewarded
/// saves. Activity completion never grants a second copy of either old prize.
pub(crate) fn install(pack: &mut CompiledGamePack, story_source: &str) -> Result<()> {
    ensure!(
        !pack.report.manifests.iter().any(|name| name == MANIFEST),
        "museum activities already installed"
    );
    let story: Value = serde_json::from_str(story_source)?;
    let source: Source = serde_json::from_value(story["refinement_museums"].clone())?;
    for flag in [
        MILL_POWER,
        MILL_GRAIN,
        MILL_DONE,
        FLOOD_TODAY,
        FLOOD_OLD,
        FLOOD_DONE,
    ] {
        crate::northstar_lair_authoring::flag(pack, flag)?;
    }
    for name in ["MnMinneapolisFoundryMuseum", "MnStPaulCivicMuseum"] {
        let mut map = pack.data.maps.get(name).context("existing museum")?.clone();
        let objects = map.objects.clone();
        let events = map.events.clone();
        let blocks = map.blocks.clone();
        for (label, pages) in source.dialogue.get(name).context("museum dialogue")? {
            ensure!(
                !map.scripts.contains_key(label),
                "occupied museum text {label}"
            );
            map.scripts
                .insert(label.clone(), authored_dialogue::compile(pages)?);
        }
        let reward_text = if name == "MnMinneapolisFoundryMuseum" {
            "MnMPExpansionMillTrailRewardText"
        } else {
            "MN_SPX_RIVERRewardText"
        };
        let reward_pages: Vec<AuthoredDialoguePage> =
            serde_json::from_value(story["dialogue_overhaul"]["texts"][name][reward_text].clone())?;
        ensure!(
            map.scripts.contains_key(reward_text),
            "existing museum prize text"
        );
        map.scripts.insert(
            reward_text.into(),
            authored_dialogue::compile(&reward_pages)?,
        );
        match name {
            "MnMinneapolisFoundryMuseum" => mill(&mut map, &source)?,
            _ => flood(&mut map, &source)?,
        }
        ensure!(
            map.objects == objects && map.events == events && map.blocks == blocks,
            "museum activities must retain geometry and existing actors"
        );
        // The general story publisher does not parse native vertical menus.
        crate::gym_scripts::finish(pack, &mut map)?;
        crate::northstar_story_revision::publish(pack, map)?;
    }
    pack.report.manifests.push(MANIFEST.into());
    Ok(())
}

fn mill(map: &mut MapModule, source: &Source) -> Result<()> {
    ensure!(
        map.scripts.get("MnMPExpansionMillTrailDone")
            == Some(&json!([c(
                "jumptextfaceplayer",
                &["MnMPExpansionMillTrailDoneText"]
            )])),
        "changed mill reward return"
    );
    // Both the original host program and its pending MAGNET claim are untouched.
    put(
        map,
        "MnMPExpansionMillTrailDone",
        vec![
            c("faceplayer", &[]),
            c("checkevent", &[MILL_DONE]),
            c("iftrue", &["MnMillDone"]),
            c("checkevent", &[MILL_POWER]),
            c("iftrue", &["MnMillHasPower"]),
            c("checkevent", &[MILL_GRAIN]),
            c("iftrue", &["MnMillNeedsPower"]),
            c("jumptext", &["MnMillHostOfferText"]),
        ],
    );
    put(
        map,
        "MnMillHasPower",
        vec![
            c("checkevent", &[MILL_GRAIN]),
            c("iftrue", &["MnMillDoor"]),
            c("jumptext", &["MnMillHostGrainText"]),
        ],
    );
    put(
        map,
        "MnMillNeedsPower",
        vec![c("jumptext", &["MnMillHostPowerText"])],
    );
    put(map, "MnMillDone", vec![c("jumptext", &["MnMillDoneText"])]);
    for (root, old, menu_root) in [
        (
            "MN_MP_MUSEUM_WHEEL_DISPLAY",
            "MN_MP_MUSEUM_WHEEL_DISPLAYText",
            "MnMillWheel",
        ),
        (
            "MN_MP_MUSEUM_GRAIN_DISPLAY",
            "MN_MP_MUSEUM_GRAIN_DISPLAYText",
            "MnMillGrain",
        ),
    ] {
        ensure!(
            map.scripts.get(root) == Some(&json!([c("jumptext", &[old])])),
            "changed mill exhibit"
        );
        put(
            map,
            root,
            vec![
                c("checkevent", &[MILL_DONE]),
                c("iftrue", &["MnMillDone"]),
                c("sjump", &[menu_root]),
            ],
        );
    }
    for (key, root, correct, wrong, flag) in [
        (
            "MillWheel",
            "MnMillWheel",
            "MnMillWheelReady",
            "MnMillWheelWrong",
            MILL_POWER,
        ),
        (
            "MillGrain",
            "MnMillGrain",
            "MnMillGrainReady",
            "MnMillGrainWrong",
            MILL_GRAIN,
        ),
    ] {
        menu(map, source, key, root, &[correct, wrong], "MnMillCancel")?;
        result(
            map,
            correct,
            &format!("{correct}Text"),
            &[c("setevent", &[flag])],
        );
        result(
            map,
            wrong,
            &format!("{wrong}Text"),
            &[c("clearevent", &[flag])],
        );
    }
    menu(
        map,
        source,
        "MillDoor",
        "MnMillDoor",
        &["MnMillSolved", "MnMillDoorWrong"],
        "MnMillCancel",
    )?;
    result(
        map,
        "MnMillSolved",
        "MnMillSolvedText",
        &[c("setevent", &[MILL_DONE])],
    );
    result(map, "MnMillDoorWrong", "MnMillDoorWrongText", &[]);
    result(map, "MnMillCancel", "MnMillCancelText", &[]);
    Ok(())
}

fn flood(map: &mut MapModule, source: &Source) -> Result<()> {
    ensure!(
        map.scripts.get("MN_SPX_RIVERDone")
            == Some(&json!([
                c("writetext", &["MN_SPX_RIVERDoneText"]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("end", &[])
            ])),
        "changed flood reward return"
    );
    // This branch is entered with the original host's text window open.
    put(
        map,
        "MN_SPX_RIVERDone",
        vec![
            c("closetext", &[]),
            c("checkevent", &[FLOOD_DONE]),
            c("iftrue", &["MnFloodDone"]),
            c("checkevent", &[FLOOD_OLD]),
            c("iftrue", &["MnFloodPath"]),
            c("checkevent", &[FLOOD_TODAY]),
            c("iftrue", &["MnFloodOld"]),
            c("sjump", &["MnFloodToday"]),
        ],
    );
    put(
        map,
        "MnFloodDone",
        vec![c("jumptext", &["MnFloodDoneText"])],
    );
    menu(
        map,
        source,
        "FloodToday",
        "MnFloodToday",
        &[
            "MnFloodTodayCorrect",
            "MnFloodTodayWrong",
            "MnFloodTodayWrong",
        ],
        "MnFloodCancel",
    )?;
    menu(
        map,
        source,
        "FloodOld",
        "MnFloodOld",
        &["MnFloodOldWrong", "MnFloodOldWrong", "MnFloodOldCorrect"],
        "MnFloodCancel",
    )?;
    menu(
        map,
        source,
        "FloodPath",
        "MnFloodPath",
        &["MnFloodPathWrong", "MnFloodSolved"],
        "MnFloodCancel",
    )?;
    for (root, flag, next) in [
        ("MnFloodTodayCorrect", FLOOD_TODAY, "MnFloodOld"),
        ("MnFloodOldCorrect", FLOOD_OLD, "MnFloodPath"),
    ] {
        put(
            map,
            root,
            vec![
                c("setevent", &[flag]),
                c("closetext", &[]),
                c("sjump", &[next]),
            ],
        );
    }
    for root in [
        "MnFloodTodayWrong",
        "MnFloodOldWrong",
        "MnFloodPathWrong",
        "MnFloodCancel",
    ] {
        result(map, root, &format!("{root}Text"), &[]);
    }
    result(
        map,
        "MnFloodSolved",
        "MnFloodSolvedText",
        &[c("setevent", &[FLOOD_DONE])],
    );
    Ok(())
}

fn menu(
    map: &mut MapModule,
    source: &Source,
    key: &str,
    root: &str,
    branches: &[&str],
    cancelled: &str,
) -> Result<()> {
    let labels = source.menus.get(key).context("canonical museum choices")?;
    ensure!(
        labels.len() == branches.len() && (2..=3).contains(&labels.len()),
        "museum menu choices"
    );
    ensure!(
        labels.iter().all(|label| !label.is_empty()
            && label.len() <= 15
            && label.is_ascii()
            && !label
                .chars()
                .any(|c| c.is_control() || ['"', '@', '<', '>'].contains(&c))),
        "native museum menu labels"
    );
    let header = format!("{root}Header");
    let data = format!("{root}MenuData");
    put(
        map,
        &header,
        vec![
            c("db", &["MENU_BACKUP_TILES"]),
            c("menu_coords", &["0", "0", "17", "9"]),
            c("dw", &[&data]),
            c("db", &["1"]),
        ],
    );
    let mut choices = vec![
        c("db", &["STATICMENU_CURSOR"]),
        c("db", &[&labels.len().to_string()]),
    ];
    choices.extend(
        labels
            .iter()
            .map(|label| c("db", &[&format!("\"{label}@\"")])),
    );
    put(map, &data, choices);
    let mut commands = vec![
        c("opentext", &[]),
        c("writetext", &[&format!("{root}PromptText")]),
        c("loadmenu", &[&header]),
        c("verticalmenu", &[]),
        c("closewindow", &[]),
    ];
    commands.extend(
        branches
            .iter()
            .enumerate()
            .map(|(index, branch)| c("ifequal", &[&(index + 1).to_string(), branch])),
    );
    // B is zero. It changes no puzzle state, and always closes the text window.
    commands.push(c("sjump", &[cancelled]));
    put(map, root, commands);
    Ok(())
}

fn result(map: &mut MapModule, root: &str, text: &str, effects: &[Value]) {
    let mut rows = effects.to_vec();
    rows.extend([
        c("writetext", &[text]),
        c("waitbutton", &[]),
        c("closetext", &[]),
        c("end", &[]),
    ]);
    put(map, root, rows);
}

fn put(map: &mut MapModule, label: &str, rows: Vec<Value>) {
    map.scripts.insert(label.into(), Value::Array(rows));
}
