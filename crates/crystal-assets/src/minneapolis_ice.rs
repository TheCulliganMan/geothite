//! Native ICE battle, separate captive release and separate recovered record.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use crystal_core::map::{BackgroundEvent, MapScriptSectionCommand};
use crystal_core::models::TrainerPartyPokemon;
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

const ENFORCER: &str = "MN_ICE_ENFORCER";
const CAPTIVE: &str = "MN_ICE_CAPTIVE_DISPATCHER";
const DEFEATED: &str = "EVENT_MN_ICE_ENFORCER_DEFEATED";
const RESCUED: &str = "EVENT_MN_ICE_CAPTIVE_RESCUED";
const NOTE: &str = "EVENT_MN_DISPATCH_NOTE";
const OFFICE: &str = "MnMinneapolisDispatchOffice";

pub(super) fn prepare(pack: &mut CompiledGamePack, story_json: &str) -> Result<()> {
    let story: Value = serde_json::from_str(story_json).context("ICE canonical story")?;
    for (id, requires, result) in [
        ("MS03_ICE", BTreeSet::from(["BADGE_2"]), DEFEATED),
        ("MS03_RESCUE", BTreeSet::from([DEFEATED]), RESCUED),
        ("MS03", BTreeSet::from(["BADGE_2", DEFEATED, RESCUED]), NOTE),
    ] {
        let ms = milestone(&story, id)?;
        let actual: Vec<String> = serde_json::from_value(ms["requires"].clone())?;
        ensure!(
            ms["location"] == OFFICE
                && ms["sets"] == json!([result])
                && actual.len() == requires.len()
                && actual.iter().map(String::as_str).collect::<BTreeSet<_>>() == requires,
            "ICE milestone prerequisites/results changed: {id}"
        );
    }
    let boss = story["bosses"]
        .as_array()
        .context("canonical bosses")?
        .iter()
        .find(|b| b["id"] == ENFORCER)
        .context("ICE enforcer party")?;
    let party: Vec<(String, u8)> = boss["party"]
        .as_array()
        .context("ICE party")?
        .iter()
        .map(|p| {
            Ok((
                p["species"].as_str().context("species")?.to_owned(),
                u8::try_from(p["level"].as_u64().context("level")?)?,
            ))
        })
        .collect::<Result<_>>()?;
    ensure!(
        party == [("RATICATE".into(), 19), ("GROWLITHE".into(), 20)],
        "retain the reserved ICE party"
    );
    let globals = &mut pack.data.story_event_script_constants.global;
    for flag in [DEFEATED, RESCUED, NOTE] {
        if let Some(id) = globals.get(flag) {
            ensure!(
                !globals
                    .iter()
                    .any(|(k, v)| k != flag && k.starts_with("EVENT_") && v == id),
                "event numeric collision"
            );
        } else {
            let next = globals
                .iter()
                .filter(|(k, _)| k.starts_with("EVENT_"))
                .map(|(_, v)| *v)
                .max()
                .unwrap_or(0)
                .checked_add(1)
                .context("ICE event identity overflow")?;
            globals.insert(flag.into(), next);
        }
    }
    ensure!(
        !pack.data.trainers.trainers.contains_key(ENFORCER)
            && !pack
                .data
                .story_event_script_constants
                .global
                .contains_key(ENFORCER),
        "ICE enforcer trainer already installed"
    );
    let mut trainer = pack
        .data
        .trainers
        .trainers
        .values()
        .find(|t| t.trainer_class == "GRUNTM")
        .context("original grunt trainer class")?
        .clone();
    let dvs = trainer.party.first().context("source trainer DVs")?.dvs;
    let next = pack
        .data
        .trainers
        .trainers
        .values()
        .filter(|t| t.trainer_class == "GRUNTM")
        .map(|t| {
            pack.data
                .story_event_script_constants
                .global
                .get(&t.trainer_id)
                .copied()
                .context("trainer numeric identity")
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .context("ICE trainer identity overflow")?;
    trainer.trainer_id = ENFORCER.into();
    trainer.name = "ICE ENFORCER".into();
    trainer.party = party
        .into_iter()
        .map(|(species, level)| {
            let p = pack.data.create_pokemon(&species, level, dvs)?;
            Ok(TrainerPartyPokemon {
                species,
                level,
                item: None,
                moves: p.moves,
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
        .insert(ENFORCER.into(), next);
    Ok(())
}
fn milestone<'a>(story: &'a Value, id: &str) -> Result<&'a Value> {
    let matches = story["milestones"]
        .as_array()
        .context("milestones")?
        .iter()
        .filter(|m| m["id"] == id)
        .collect::<Vec<_>>();
    ensure!(matches.len() == 1, "one canonical {id} required");
    Ok(matches[0])
}

pub(super) fn install(
    pack: &CompiledGamePack,
    map: &mut MapModule,
    town: &Value,
    story_json: &str,
) -> Result<()> {
    let scene = &town["ice_scene"];
    let story: Value = serde_json::from_str(story_json)?;
    ensure!(
        map.id == OFFICE
            && map.attributes.width == 10
            && map.attributes.height == 8
            && map.blocks.len() == 80
            && map.attributes.tileset_name == "underground"
            && scene["room"] == OFFICE
            && scene["enforcer"]["tile"] == json!([8, 9])
            && scene["enforcer"]["approach"] == json!([9, 9])
            && scene["captive"]["tile"] == json!([15, 5])
            && scene["release"]["tile"] == json!([12, 6])
            && scene["release"]["approach"] == json!([11, 6])
            && scene["release"]["gate_metatile"] == json!([6, 3])
            && scene["release"]["closed_block"] == "38"
            && scene["release"]["open_block"] == "05"
            && scene["note"]["tile"] == json!([9, 11])
            && scene["note"]["approach"] == json!([10, 11])
            && map.blocks[3 * 10 + 6] == 0x38,
        "ICE requires the actual closed native holding room"
    );
    for (id, sprite, at) in [
        (ENFORCER, "SPRITE_OFFICER", [8, 9]),
        (CAPTIVE, "SPRITE_FISHER", [15, 5]),
    ] {
        ensure!(
            !map.objects
                .iter()
                .any(|o| o.object_identifier.as_deref() == Some(id)),
            "ICE actor occupied"
        );
        authoring::npc(pack, map, id, sprite, at[0], at[1])?;
        let object = map.objects.last_mut().context("ICE actor")?;
        object.object_identifier = Some(id.into());
        object.spritemovedata = if id == CAPTIVE {
            "SPRITEMOVEDATA_STANDING_LEFT"
        } else {
            "SPRITEMOVEDATA_STANDING_DOWN"
        }
        .into();
        if id == CAPTIVE {
            object.event_flag = RESCUED.into();
        }
    }
    let mut texts = BTreeMap::<String, Vec<String>>::new();
    for id in ["MS03_ICE", "MS03_RESCUE", "MS03"] {
        let ms = milestone(&story, id)?;
        for (label, value) in ms["native_texts"]
            .as_object()
            .context("canonical ICE native text")?
        {
            let lines: Vec<String> = serde_json::from_value(value.clone())?;
            ensure!(
                !lines.is_empty()
                    && lines.iter().all(|l| !l.is_empty()
                        && l.chars().count() <= 18
                        && l.is_ascii()
                        && !l.contains(['\n', '\r', '"'])),
                "ICE native text width/font {label}"
            );
            ensure!(
                texts.insert(label.clone(), lines).is_none(),
                "duplicate ICE text"
            );
        }
    }
    let scripts = scripts();
    for label in scripts.keys().chain(texts.keys()) {
        ensure!(
            !map.scripts.contains_key(label) && !pack.data.map_scripts.contains_key(label),
            "ICE script collision {label}"
        );
    }
    map.scripts.extend(scripts);
    for (label, lines) in texts {
        authoring::text(
            map,
            &label,
            &lines.iter().map(String::as_str).collect::<Vec<_>>(),
        );
    }
    map.events.bg_events.extend([
        BackgroundEvent {
            x: 12,
            y: 6,
            event_type: "BGEVENT_READ".into(),
            script: "MnMPCellLatch".into(),
        },
        BackgroundEvent {
            x: 9,
            y: 11,
            event_type: "BGEVENT_READ".into(),
            script: "MnMPDispatchCopy".into(),
        },
    ]);
    let c = authoring::command;
    let mut movement = Vec::new();
    let source_steps: Vec<String> =
        serde_json::from_value(scene["captive_departure_steps"].clone())?;
    let expected = [
        "step_down",
        "step_down",
        "step_left",
        "step_left",
        "step_left",
        "step_left",
        "step_down",
        "step_down",
        "step_down",
        "step_down",
        "step_down",
        "step_down",
        "step_down",
        "step_down",
        "step_end",
    ];
    ensure!(
        source_steps.iter().map(String::as_str).eq(expected),
        "retain clear captive departure path"
    );
    let mut open = map.blocks.clone();
    open[3 * 10 + 6] = 5;
    let mut position = [15_u16, 5];
    for step in source_steps {
        match step.as_str() {
            "step_down" => position[1] += 1,
            "step_left" => {
                position[0] = position[0]
                    .checked_sub(1)
                    .context("captive movement underflow")?
            }
            "step_end" => {}
            _ => anyhow::bail!("unknown captive step"),
        }
        ensure!(
            position != [11, 6]
                && position != [8, 9]
                && matches!(
                    crate::authored_route_modpack::collision(
                        pack,
                        &open,
                        10,
                        position,
                        "underground"
                    )?,
                    "FLOOR" | "WARP_CARPET_DOWN"
                ),
            "captive departure crosses a wall, the player or guard"
        );
        movement.push(match step.as_str() {
            "step_down" => c("step", &["DOWN"]),
            "step_left" => c("step", &["LEFT"]),
            "step_end" => c("step_end", &[]),
            _ => anyhow::bail!("unsupported captive departure step"),
        });
    }
    ensure!(position == [11, 15], "captive must leave by the carpet");
    map.scripts
        .insert("MnMPCaptiveLeaves".into(), Value::Array(movement));
    if map.map_script_section_commands.is_empty() {
        map.map_script_section_commands.extend([
            MapScriptSectionCommand {
                command: "def_scene_scripts".into(),
                args: vec![],
                command_index: 0,
            },
            MapScriptSectionCommand {
                command: "def_callbacks".into(),
                args: vec![],
                command_index: 1,
            },
        ]);
    }
    map.map_script_section_commands
        .push(MapScriptSectionCommand {
            command: "callback".into(),
            args: vec!["MAPCALLBACK_TILES".into(), "MnMPRefreshCell".into()],
            command_index: map.map_script_section_commands.len(),
        });
    Ok(())
}
fn scripts() -> BTreeMap<String, Value> {
    let c = authoring::command;
    let mut scripts = BTreeMap::from([
        (
            ENFORCER.into(),
            json!([
                c("faceplayer", &[]),
                c("checkevent", &[DEFEATED]),
                c("iftrue", &["MnMPICEAfter"]),
                c("checkflag", &[BADGE_ENGINE_FLAGS[1]]),
                c("iffalse", &["MnMPICEBadgeDenied"]),
                c("opentext", &[]),
                c("writetext", &["MnMPICESeenText"]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("winlosstext", &["MnMPICEWonText", "MnMPICELostText"]),
                c("loadtrainer", &["GRUNTM", ENFORCER]),
                c("startbattle", &[]),
                c("reloadmapafterbattle", &[]),
                c("setevent", &[DEFEATED]),
                c("sjump", &["MnMPICEAfter"])
            ]),
        ),
        (
            CAPTIVE.into(),
            json!([c("jumptextfaceplayer", &["MnMPCellLockedText"])]),
        ),
        (
            "MnMPCellLatch".into(),
            json!([
                c("checkevent", &[RESCUED]),
                c("iftrue", &["MnMPCellAlready"]),
                c("checkevent", &[DEFEATED]),
                c("iffalse", &["MnMPCellLocked"]),
                c("opentext", &[]),
                c("writetext", &["MnMPCellReadyText"]),
                c("yesorno", &[]),
                c("iffalse", &["MnMPCellDeclined"]),
                c("writetext", &["MnMPCellReleaseText"]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("changeblock", &["12", "6", "5"]),
                c("applymovement", &[CAPTIVE, "MnMPCaptiveLeaves"]),
                c("disappear", &[CAPTIVE]),
                c("setevent", &[RESCUED]),
                c("end", &[])
            ]),
        ),
        (
            "MnMPDispatchCopy".into(),
            json!([
                c("checkevent", &[NOTE]),
                c("iftrue", &["MnMPDispatchDone"]),
                c("checkflag", &[BADGE_ENGINE_FLAGS[1]]),
                c("iffalse", &["MnMPDispatchLocked"]),
                c("checkevent", &[DEFEATED]),
                c("iffalse", &["MnMPDispatchLocked"]),
                c("checkevent", &[RESCUED]),
                c("iffalse", &["MnMPDispatchLocked"]),
                c("opentext", &[]),
                c("writetext", &["MnMPDispatchReadText"]),
                c("waitbutton", &[]),
                c("setevent", &[NOTE]),
                c("closetext", &[]),
                c("end", &[])
            ]),
        ),
        (
            "MnMPRefreshCell".into(),
            json!([
                c("checkevent", &[RESCUED]),
                c("iffalse", &["MnMPCellClosed"]),
                c("changeblock", &["12", "6", "5"]),
                c("end", &[])
            ]),
        ),
        (
            "MnMPCellClosed".into(),
            json!([c("changeblock", &["12", "6", "56"]), c("end", &[])]),
        ),
        (
            "MnMPCellDeclined".into(),
            json!([
                c("writetext", &["MnMPCellDeclinedText"]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("end", &[])
            ]),
        ),
    ]);
    for suffix in [
        "ICEAfter",
        "ICEBadgeDenied",
        "CellLocked",
        "CellAlready",
        "DispatchLocked",
        "DispatchDone",
    ] {
        scripts.insert(
            format!("MnMP{suffix}"),
            json!([c("jumptext", &[&format!("MnMP{suffix}Text")])]),
        );
    }
    scripts
}

/// Restore exactly one dispatcher: held in the office before rescue, home after.
pub(super) fn install_home(map: &mut MapModule, town: &Value) -> Result<()> {
    ensure!(
        map.id == "MnMinneapolisDispatcherHouse"
            && town["ice_scene"]["home"]["released_actor"] == "MN_MP_RESCUED_DISPATCHER",
        "actual dispatcher home"
    );
    let c = authoring::command;
    map.scripts.insert(
        "MnMPHomePresence".into(),
        json!([
            c("checkevent", &[RESCUED]),
            c("iftrue", &["MnMPHomeReleased"]),
            c("disappear", &["MN_MP_RESCUED_DISPATCHER"]),
            c("end", &[])
        ]),
    );
    map.scripts.insert(
        "MnMPHomeReleased".into(),
        json!([c("appear", &["MN_MP_RESCUED_DISPATCHER"]), c("end", &[])]),
    );
    if map.map_script_section_commands.is_empty() {
        map.map_script_section_commands.extend([
            MapScriptSectionCommand {
                command: "def_scene_scripts".into(),
                args: vec![],
                command_index: 0,
            },
            MapScriptSectionCommand {
                command: "def_callbacks".into(),
                args: vec![],
                command_index: 1,
            },
        ]);
    }
    map.map_script_section_commands
        .push(MapScriptSectionCommand {
            command: "callback".into(),
            args: vec!["MAPCALLBACK_OBJECTS".into(), "MnMPHomePresence".into()],
            command_index: map.map_script_section_commands.len(),
        });
    Ok(())
}
