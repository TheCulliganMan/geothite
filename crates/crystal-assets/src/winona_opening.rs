//! Native guardian, starter gifts and recoverable survey/supply handover.
use anyhow::{Context, Result, ensure};
use crystal_core::map::MapScene;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

pub(super) const HOME: &str = "EVENT_MN_HOME_INTRO";
pub(super) const STARTER: &str = "EVENT_MN_STARTER_RECEIVED";
pub(super) const PARCEL: &str = "EVENT_MN_R01_PARCEL_RECEIVED";
pub(super) const BALLS: &str = "EVENT_MN_WINONA_BALLS_RECEIVED";

#[derive(Deserialize)]
pub(super) struct Opening {
    pub new_game_spawn: u16,
    pub bedroom_start: [u16; 2],
    pub home_flag: String,
    pub starter_flag: String,
    pub parcel_flag: String,
    pub supplies_flag: String,
    pub starters: Vec<Starter>,
}

#[derive(Deserialize)]
pub(super) struct Starter {
    pub actor: String,
    pub species: String,
    pub level: u8,
    pub hidden_after: String,
    pub sprite: String,
    pub tile: [u16; 2],
    pub approach: [u16; 2],
}

pub(super) fn prepare(pack: &mut CompiledGamePack, opening: &Opening, story: &Value) -> Result<()> {
    ensure!(
        opening.new_game_spawn == 0
            && opening.bedroom_start == [3, 3]
            && opening.home_flag == HOME
            && opening.starter_flag == STARTER
            && opening.parcel_flag == PARCEL
            && opening.supplies_flag == BALLS,
        "canonical Winona opening identities required"
    );
    ensure!(
        opening.starters.len() == 3,
        "three native starter balls required"
    );
    for (index, (ball, species)) in opening
        .starters
        .iter()
        .zip(["CHIKORITA", "CYNDAQUIL", "TOTODILE"])
        .enumerate()
    {
        ensure!(
            ball.species == species
                && ball.level == 5
                && ball.sprite == "SPRITE_POKE_BALL"
                && ball.actor == format!("MN_W_{species}")
                && ball.hidden_after == format!("EVENT_MN_WINONA_{species}")
                && ball.tile == [6 + u16::try_from(index)?, 3]
                && ball.approach == [ball.tile[0], 4]
                && pack.data.pokemon.contains_key(species),
            "canonical starter species/level/tabletop anchor required"
        );
    }
    for (id, location, requires, sets) in [
        ("MS_HOME", "MnWinonaHome1F", vec![], vec![HOME]),
        (
            "MS00",
            "MnWinonaFieldStation",
            vec![HOME],
            vec![STARTER, PARCEL],
        ),
    ] {
        let milestone = story["milestones"]
            .as_array()
            .context("milestones")?
            .iter()
            .find(|m| m["id"] == id)
            .context("opening milestone")?;
        ensure!(
            milestone["location"] == location
                && milestone["requires"] == json!(requires)
                && milestone["sets"] == json!(sets),
            "opening story contract changed: {id}"
        );
    }
    let globals = &mut pack.data.story_event_script_constants.global;
    for flag in [HOME, STARTER, PARCEL, BALLS]
        .into_iter()
        .map(str::to_owned)
        .chain(opening.starters.iter().map(|s| s.hidden_after.clone()))
    {
        if let Some(id) = globals.get(&flag) {
            ensure!(
                !globals
                    .iter()
                    .any(|(other, n)| other != &flag && other.starts_with("EVENT_") && n == id),
                "event collision: {flag}"
            );
        } else {
            let next = globals
                .iter()
                .filter(|(name, _)| name.starts_with("EVENT_"))
                .map(|(_, n)| *n)
                .max()
                .context("native event table")?
                .checked_add(1)
                .context("opening event overflow")?;
            u16::try_from(next).context("opening event outside native range")?;
            globals.insert(flag, next);
        }
    }
    let local = pack
        .data
        .story_event_script_constants
        .maps
        .entry("MnWinonaHome1F".into())
        .or_default();
    ensure!(local.is_empty(), "Winona home scene namespace occupied");
    local.insert("SCENE_MN_W_HOME_INTRO".into(), 0);
    local.insert("SCENE_MN_W_HOME_READY".into(), 1);
    Ok(())
}

fn text(map: &mut MapModule, label: &str, lines: &[&str]) {
    authoring::text(map, label, lines);
}
fn story_text(map: &mut MapModule, story: &Value, id: &str, label: &str) -> Result<()> {
    let milestone = story["milestones"]
        .as_array()
        .context("milestones")?
        .iter()
        .find(|m| m["id"] == id)
        .context("opening text")?;
    let pages: Vec<Vec<String>> = serde_json::from_value(milestone["dialogue_pages"].clone())?;
    ensure!(
        !pages.is_empty()
            && pages
                .iter()
                .all(|p| p.len() == 2 && p.iter().all(|s| s.is_ascii() && s.len() <= 18)),
        "native opening page width: {id}"
    );
    text(
        map,
        label,
        &pages
            .iter()
            .flatten()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    );
    Ok(())
}

pub(super) fn install_home(map: &mut MapModule, story: &Value) -> Result<()> {
    let c = authoring::command;
    story_text(map, story, "MS_HOME", "MnWHomeIntroText")?;
    text(
        map,
        "MnWGuardianRestText",
        &["Take your time.", "You can rest here."],
    );
    map.scripts.insert(
        "MnWHomeArrival".into(),
        json!([
            c("checkevent", &[HOME]),
            c("iftrue", &["MnWHomeReady"]),
            c("opentext", &[]),
            c("writetext", &["MnWHomeIntroText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("setevent", &[HOME]),
            c("sjump", &["MnWHomeReady"])
        ]),
    );
    map.scripts.insert(
        "MnWHomeReady".into(),
        json!([c("setscene", &["SCENE_MN_W_HOME_READY"]), c("end", &[])]),
    );
    map.scripts
        .insert("MnWHomeNoop".into(), json!([c("end", &[])]));
    map.scripts.insert(
        "MN_W_GUARDIAN".into(),
        json!([
            c("checkevent", &[HOME]),
            c("iffalse", &["MnWHomeArrival"]),
            c("faceplayer", &[]),
            c("opentext", &[]),
            c("writetext", &["MnWGuardianRestText"]),
            c("waitbutton", &[]),
            c("special", &["HealParty"]),
            c("blackoutmod", &["MN_WINONA_HOME_1F"]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    map.scenes.scenes = vec![
        MapScene {
            scene_id: "SCENE_MN_W_HOME_INTRO".into(),
            script_name: Some("MnWHomeArrival".into()),
        },
        MapScene {
            scene_id: "SCENE_MN_W_HOME_READY".into(),
            script_name: Some("MnWHomeNoop".into()),
        },
    ];
    map.map_script_section_commands = serde_json::from_value(json!([
        {"command":"def_scene_scripts","args":[],"command_index":0},
        {"command":"scene_script","args":["MnWHomeArrival","SCENE_MN_W_HOME_INTRO"],"command_index":1},
        {"command":"scene_script","args":["MnWHomeNoop","SCENE_MN_W_HOME_READY"],"command_index":2},
        {"command":"def_callbacks","args":[],"command_index":3}
    ]))?;
    Ok(())
}

pub(super) fn install_station(
    pack: &CompiledGamePack,
    map: &mut MapModule,
    opening: &Opening,
    story: &Value,
) -> Result<()> {
    let c = authoring::command;
    story_text(map, story, "MS00", "MnWReadingsText")?;
    for (label, lines) in [
        (
            "MnWVisitHomeText",
            vec!["Talk to your family", "before you leave."],
        ),
        ("MnWChooseText", vec!["Choose a partner", "from the table."]),
        (
            "MnWDeclineText",
            vec!["Take your time.", "All three can wait."],
        ),
        ("MnWGiftFullText", vec!["Make room for a", "POKEMON first."]),
        (
            "MnWBallsText",
            vec!["Five POKE BALLS.", "Meet the wildlife."],
        ),
        (
            "MnWBallsFullText",
            vec!["No room for balls.", "Come back for them."],
        ),
        (
            "MnWReminderText",
            vec!["ROCHESTER needs", "the old readings."],
        ),
    ] {
        text(map, label, &lines);
    }
    map.scripts.insert(
        "MnWVisitHome".into(),
        json!([c("jumptextfaceplayer", &["MnWVisitHomeText"])]),
    );
    map.scripts.insert(
        "MnWChoose".into(),
        json!([c("jumptextfaceplayer", &["MnWChooseText"])]),
    );
    map.scripts.insert(
        "MN_W_TAMARACK".into(),
        json!([
            c("checkevent", &[HOME]),
            c("iffalse", &["MnWVisitHome"]),
            c("checkevent", &[STARTER]),
            c("iftrue", &["MnWFollowup"]),
            c("sjump", &["MnWChoose"])
        ]),
    );
    map.scripts.insert(
        "MnWFollowup".into(),
        json!([
            c("checkevent", &[PARCEL]),
            c("iftrue", &["MnWReminder"]),
            c("opentext", &[]),
            c("writetext", &["MnWReadingsText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("setevent", &[PARCEL]),
            c("scall", &["MnWSupplies"]),
            c("end", &[])
        ]),
    );
    map.scripts.insert(
        "MnWReminder".into(),
        json!([
            c("opentext", &[]),
            c("writetext", &["MnWReminderText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("scall", &["MnWSupplies"]),
            c("end", &[])
        ]),
    );
    map.scripts.insert(
        "MN_W_AIDE".into(),
        json!([
            c("checkevent", &[STARTER]),
            c("iffalse", &["MnWChoose"]),
            c("scall", &["MnWSupplies"]),
            c("end", &[])
        ]),
    );
    map.scripts.insert(
        "MnWSupplies".into(),
        json!([
            c("checkevent", &[BALLS]),
            c("iftrue", &["MnWSuppliesDone"]),
            c("giveitem", &["POKE_BALL", "5"]),
            c("iffalse", &["MnWSuppliesFull"]),
            c("setevent", &[BALLS]),
            c("opentext", &[]),
            c("writetext", &["MnWBallsText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    map.scripts.insert(
        "MnWSuppliesFull".into(),
        json!([
            c("opentext", &[]),
            c("writetext", &["MnWBallsFullText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    map.scripts
        .insert("MnWSuppliesDone".into(), json!([c("end", &[])]));
    map.scripts.insert(
        "MnWStarterDeclined".into(),
        json!([
            c("writetext", &["MnWDeclineText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    map.scripts.insert(
        "MnWStarterFull".into(),
        json!([c("jumptext", &["MnWGiftFullText"])]),
    );
    for ball in &opening.starters {
        let label = format!("{}Text", ball.actor);
        text(
            map,
            &label,
            &[&format!("Take {}", ball.species), "as your partner?"],
        );
        map.scripts.insert(
            ball.actor.clone(),
            json!([
                c("checkevent", &[STARTER]),
                c("iftrue", &["MnWFollowup"]),
                c("checkevent", &[HOME]),
                c("iffalse", &["MnWVisitHome"]),
                c("opentext", &[]),
                c("writetext", &[&label]),
                c("yesorno", &[]),
                c("iffalse", &["MnWStarterDeclined"]),
                c("closetext", &[]),
                c("givepoke", &[&ball.species, "5"]),
                c("ifequal", &["2", "MnWStarterFull"]),
                c("setevent", &[STARTER]),
                c("setevent", &[&ball.hidden_after]),
                c("disappear", &[&ball.actor]),
                c("sjump", &["MnWFollowup"])
            ]),
        );
        authoring::npc(
            pack,
            map,
            &ball.actor,
            &ball.sprite,
            i16::try_from(ball.tile[0])?,
            i16::try_from(ball.tile[1])?,
        )?;
        let object = map.objects.last_mut().context("starter ball object")?;
        object.object_identifier = Some(ball.actor.clone());
        object.event_flag = ball.hidden_after.clone();
        object.sprite_has_facings = false;
        object.spritemovedata = "SPRITEMOVEDATA_STILL".into();
    }
    Ok(())
}
