//! Native League registration, sequential battles and persistent run memory.
use anyhow::{Context, Result, ensure};
use crystal_core::map::MapScriptSectionCommand;
use crystal_core::models::TrainerPartyPokemon;
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::{CompiledGamePack, MapModule};

pub(crate) const ACTIVE: &str = "EVENT_MN_LEAGUE_RUN_ACTIVE";
pub(crate) const CHAMPION: &str = "EVENT_MN_LEAGUE_CHAMPION";
pub(crate) const RUN: &str = "wMnLeagueRun";

#[derive(Deserialize)]
pub(crate) struct Member {
    pub id: String,
    pub name: String,
    pub map: String,
    pub party: Vec<Party>,
}

#[derive(Deserialize)]
pub(crate) struct Party {
    pub species: String,
    pub level: u8,
}

pub(crate) fn allocate(pack: &mut CompiledGamePack, members: &[Member]) -> Result<()> {
    for name in [ACTIVE, CHAMPION] {
        let globals = &mut pack.data.story_event_script_constants.global;
        ensure!(!globals.contains_key(name), "unused League event {name}");
        let next = globals
            .iter()
            .filter(|(k, _)| k.starts_with("EVENT_"))
            .map(|(_, v)| *v)
            .max()
            .context("event catalog")?
            + 1;
        u16::try_from(next)?;
        globals.insert(name.into(), next);
    }
    for (member, class) in members.iter().zip(classes()) {
        let mut trainer = pack
            .data
            .trainers
            .trainers
            .values()
            .find(|t| t.trainer_class == class)
            .context("native League trainer class")?
            .clone();
        let dvs = trainer.party.first().context("native class DVs")?.dvs;
        let next = pack
            .data
            .trainers
            .trainers
            .values()
            .filter(|t| t.trainer_class == class)
            // Authored city opponents can be direct symbolic catalog entries
            // without a numeric class alias. Only installed numeric aliases
            // reserve a native class byte; do not invent IDs for those entries.
            .filter_map(|t| {
                pack.data
                    .story_event_script_constants
                    .global
                    .get(&t.trainer_id)
                    .copied()
            })
            .max()
            .context("trainer class catalog")?
            + 1;
        ensure!(
            (1..=255).contains(&next)
                && !pack.data.trainers.trainers.contains_key(&member.id)
                && !pack
                    .data
                    .story_event_script_constants
                    .global
                    .contains_key(&member.id),
            "unused League trainer"
        );
        trainer.trainer_id = member.id.clone();
        trainer.name = member.name.clone();
        trainer.party = member
            .party
            .iter()
            .map(|p| {
                let pokemon = pack.data.create_pokemon(&p.species, p.level, dvs)?;
                Ok(TrainerPartyPokemon {
                    species: p.species.clone(),
                    level: p.level,
                    item: None,
                    moves: pokemon.moves,
                    dvs,
                })
            })
            .collect::<Result<_>>()?;
        trainer.win_quote.clear();
        trainer.lose_quote.clear();
        pack.data.trainers.insert(trainer)?;
        pack.data
            .story_event_script_constants
            .global
            .insert(member.id.clone(), next);
    }
    Ok(())
}

pub(crate) fn classes() -> [&'static str; 5] {
    ["COOLTRAINERF", "HIKER", "SAGE", "COOLTRAINERM", "CHAMPION"]
}

pub(crate) fn callback(map: &mut MapModule, kind: &str, label: &str, body: Vec<Value>) {
    if map.map_script_section_commands.is_empty() {
        map.map_script_section_commands = serde_json::from_value(json!([
            {"command":"def_scene_scripts","args":[],"command_index":0},
            {"command":"def_callbacks","args":[],"command_index":1}
        ]))
        .expect("literal native callback section");
    }
    map.scripts.insert(label.into(), Value::Array(body));
    map.map_script_section_commands
        .push(MapScriptSectionCommand {
            command: "callback".into(),
            args: vec![kind.into(), label.into()],
            command_index: map.map_script_section_commands.len(),
        });
}

pub(crate) fn reset_on_entry(map: &mut MapModule) {
    callback(
        map,
        "MAPCALLBACK_NEWMAP",
        &format!("{}ResetRun", map.id),
        vec![
            c("clearevent", &[ACTIVE]),
            c("setval", &["0"]),
            c("writemem", &[RUN]),
            c("endcallback", &[]),
        ],
    );
}

fn dialogue(map: &mut MapModule, label: &str, lines: &[&str]) {
    let text = format!("{label}Text");
    authoring::text(map, &text, lines);
    map.scripts.insert(
        label.into(),
        json!([
            c("opentext", &[]),
            c("writetext", &[&text]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
}

pub(crate) fn registration(pack: &CompiledGamePack, map: &mut MapModule) -> Result<()> {
    let root = "MnElyRegister";
    let mut commands = vec![c("faceplayer", &[])];
    for flag in &BADGE_ENGINE_FLAGS[..8] {
        commands.extend([c("checkflag", &[flag]), c("iffalse", &["MnElyBadgeDenied"])]);
    }
    commands.extend([
        c("checkevent", &["EVENT_MN_SURVEY_RESTORED"]),
        c("iffalse", &["MnElySurveyDenied"]),
        c("opentext", &[]),
        c("writetext", &["MnElyRegistrationText"]),
        c("yesorno", &[]),
        c("iffalse", &["MnElyRegistrationDeclined"]),
        c("closetext", &[]),
        c("loadmem", &[RUN, "0"]),
        c("setevent", &[ACTIVE]),
        c("blackoutmod", &["MN_ELY_POKECENTER_1F"]),
        c("warpfacing", &["MN_LEAGUE_ROOM_1", "4", "16", "UP"]),
        c("end", &[]),
    ]);
    map.scripts.insert(root.into(), Value::Array(commands));
    authoring::text(
        map,
        "MnElyRegistrationText",
        &[
            "Four members, then CASEY. Bring your own supplies.",
            "Leaving or losing ends this run. Register now?",
        ],
    );
    dialogue(
        map,
        "MnElyBadgeDenied",
        &[
            "Earn all eight badges before registering.",
            "The clinic and shop remain open to you.",
        ],
    );
    dialogue(
        map,
        "MnElySurveyDenied",
        &["The public survey link must be restored before the League can open."],
    );
    dialogue(
        map,
        "MnElyRegistrationDeclined",
        &["Take your time. Healing and supplies are outside."],
    );
    authoring::npc(pack, map, root, "SPRITE_TEACHER", 5, 2)?;
    reset_on_entry(map);
    Ok(())
}

pub(crate) fn battle(
    pack: &CompiledGamePack,
    map: &mut MapModule,
    member: &Member,
    index: usize,
) -> Result<()> {
    let root = format!("{}Challenge", map.id);
    let denied = format!("{}NoRun", map.id);
    let after = format!("{}After", map.id);
    let before_text = format!("{}BeforeText", map.id);
    let won = format!("{}WonText", map.id);
    let lost = format!("{}LostText", map.id);
    let count = index.to_string();
    let previous = (index - 1).to_string();
    let (y, open, closed) = if index == 5 {
        ("0", "$0b", "$0a")
    } else {
        ("2", "$16", "$22")
    };
    let door = format!("{}RefreshDoor", map.id);
    let close = format!("{}CloseDoor", map.id);
    callback(
        map,
        "MAPCALLBACK_TILES",
        &door,
        vec![
            c("checkevent", &[ACTIVE]),
            c("iffalse", &[&close]),
            c("readmem", &[RUN]),
            c("ifless", &[&count, &close]),
            c("changeblock", &["4", y, open]),
            c("endcallback", &[]),
        ],
    );
    map.scripts.insert(
        close,
        json!([c("changeblock", &["4", y, closed]), c("endcallback", &[])]),
    );
    map.scripts.insert(
        root.clone(),
        json!([
            c("faceplayer", &[]),
            c("checkevent", &[ACTIVE]),
            c("iffalse", &[&denied]),
            c("readmem", &[RUN]),
            c("ifequal", &[&count, &after]),
            c("ifgreater", &[&count, &after]),
            c("ifnotequal", &[&previous, &denied]),
            c("opentext", &[]),
            c("writetext", &[&before_text]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("winlosstext", &[&won, &lost]),
            c("loadtrainer", &[classes()[index - 1], &member.id]),
            c("startbattle", &[]),
            c("reloadmapafterbattle", &[]),
            c("loadmem", &[RUN, &count]),
            c("changeblock", &["4", y, open]),
            c("refreshmap", &[]),
            c("sjump", &[&after]),
        ]),
    );
    let intro = match index {
        1 => "I'm FERN. Small changes become strength. Watch how this team works together.",
        2 => "FLINT. Rock remembers every strain. Let's see what your team can withstand.",
        3 => "I'm VEIL. A battle is shaped by what you cannot see coming. Stay attentive.",
        4 => "HALDEN. Power needs direction. Show me how you guide yours.",
        _ => {
            "I'm CASEY. You've come a long way together. Let me meet the team that brought you here."
        }
    };
    authoring::text(map, &before_text, &[intro]);
    authoring::text(map, &won, &["That was well earned."]);
    authoring::text(map, &lost, &["Rest, then try again."]);
    dialogue(
        map,
        &after,
        &[if index == 5 {
            "You and your team have earned your place. Go north to record this victory."
        } else {
            "The next door is open. Take what you learned with you."
        }],
    );
    dialogue(
        map,
        &denied,
        &["Register in the lobby and complete the earlier rooms first."],
    );
    let sprite = [
        "SPRITE_COOLTRAINER_F",
        "SPRITE_POKEFAN_M",
        "SPRITE_SAGE",
        "SPRITE_COOLTRAINER_M",
        "SPRITE_LANCE",
    ][index - 1];
    authoring::npc(pack, map, &root, sprite, 5, 7)?;
    Ok(())
}

pub(crate) fn hall(pack: &CompiledGamePack, map: &mut MapModule) -> Result<()> {
    map.scripts.insert(
        "MnElyRecordVictory".into(),
        json!([
            c("faceplayer", &[]),
            c("checkevent", &[ACTIVE]),
            c("iffalse", &["MnElyRecordDenied"]),
            c("readmem", &[RUN]),
            c("ifnotequal", &["5", "MnElyRecordDenied"]),
            c("opentext", &[]),
            c("writetext", &["MnElyRecordText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("setevent", &[CHAMPION]),
            c("clearevent", &[ACTIVE]),
            c("loadmem", &[RUN, "0"]),
            c("halloffame", &[]),
            c("end", &[]),
        ]),
    );
    authoring::text(
        map,
        "MnElyRecordText",
        &["Your partners made this journey with you. Their names belong in the Hall of Fame."],
    );
    dialogue(
        map,
        "MnElyRecordDenied",
        &["Complete a registered League run to record a new victory."],
    );
    authoring::npc(pack, map, "MnElyRecordVictory", "SPRITE_TEACHER", 5, 5)
}
