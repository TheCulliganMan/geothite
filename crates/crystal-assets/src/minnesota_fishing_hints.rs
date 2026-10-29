//! Fishing-house directions in defeated fishermen's native dialogue.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;

pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let policy = crate::northstar_regional_roster::RegionalRoster::parse(
        include_str!("../../../planning/minnesota/story-pokemon.json"),
        include_str!("../../../planning/minnesota/pokemon/roster.json"),
        include_str!("../../../planning/minnesota/pokemon/roster-curation.json"),
        include_str!("../../../planning/minnesota/pokemon/species-profiles.json"),
    )?;
    let maps = pack
        .data
        .maps
        .keys()
        .filter(|id| id.starts_with("Mn"))
        .cloned()
        .collect::<Vec<_>>();
    let mut fishermen = BTreeSet::new();
    for id in maps {
        let mut map = pack.data.maps[&id].clone();
        let mut labels = BTreeSet::new();
        for trainer in map
            .trainer_scripts
            .values()
            .filter(|t| t.trainer_class == "FISHER")
        {
            fishermen.insert(trainer.trainer_id.clone());
            labels.insert(trainer.win_text.clone());
            if let Some(commands) = map.scripts.get(&trainer.callback).and_then(Value::as_array) {
                for command in commands {
                    if matches!(
                        command["command"].as_str(),
                        Some("jumptextfaceplayer" | "jumptext" | "writetext")
                    ) && let Some(label) = command["args"][0].as_str()
                    {
                        labels.insert(label.into());
                    }
                }
            }
        }
        // Existing bank anglers and the first route's creek child advertise the
        // same houses without becoming additional item dispensers.
        if id.starts_with("MnR") {
            for actor in map.objects.iter().filter(|o| {
                o.object_type == "OBJECTTYPE_SCRIPT"
                    && (o.sprite == "SPRITE_FISHER" || o.script == "MN_R01_CREEK_CHILD")
            }) {
                if let Some(commands) = map.scripts.get(&actor.script).and_then(Value::as_array) {
                    for command in commands {
                        if matches!(
                            command["command"].as_str(),
                            Some("jumptextfaceplayer" | "jumptext" | "writetext")
                        ) && let Some(label) = command["args"][0].as_str()
                        {
                            labels.insert(label.into());
                        }
                    }
                }
            }
        }
        if labels.is_empty() {
            continue;
        }
        let lines: &[&str] = if id.starts_with("MnR01") || id.starts_with("MnR02") {
            &[
                "Need a ROD?",
                "The GURU's house",
                "is on DRIFTLESS",
                "TRAIL, by the",
                "lower creek bank.",
                "He gives OLD RODS!",
            ]
        } else if id.starts_with("MnR03")
            || id.starts_with("MnR04")
            || id.starts_with("MnMississippiLower")
            || id.starts_with("MnMississippiConfluence")
            || id.starts_with("MnMinneapolis")
        {
            &[
                "Know the ROD GURU?",
                "His little house",
                "is by the middle",
                "river crossing on",
                "RIVER VALLEY.",
                "A free GOOD ROD!",
            ]
        } else {
            &[
                "Try the ROD GURU!",
                "His house is above",
                "the lakeside deck",
                "on PINE COUNTRY RD",
                "south of BRAINERD.",
                "A free SUPER ROD!",
            ]
        };
        for label in labels {
            let body = map
                .scripts
                .get_mut(&label)
                .and_then(Value::as_array_mut)
                .context(format!("fisher text {id}/{label}"))?;
            let terminal = body.pop().context("fisher text terminator")?;
            ensure!(
                terminal["command"] == "done",
                "fisher hint requires native done terminator"
            );
            for (index, line) in lines.iter().enumerate() {
                // Native text rows are at most eighteen characters.
                ensure!(
                    line.chars().count() <= 18,
                    "fisher direction exceeds native row"
                );
                body.push(authoring::command(
                    if index % 2 == 0 { "para" } else { "line" },
                    &[&format!("\"{line}\"")],
                ));
            }
            body.push(terminal);
        }
        map.script_text_commands = crate::parse_script_text_commands(&id, &map.scripts)?;
        map.script_text_bodies = crate::parse_script_text_bodies(&id, &map.scripts)?;
        crate::authored_town_publication::replace(pack, map)?;
    }
    ensure!(!fishermen.is_empty(), "no Minnesota fishermen found");
    for id in fishermen {
        let mut trainer = pack
            .data
            .trainers
            .trainers
            .get(&id)
            .context("fisher party")?
            .clone();
        for member in &mut trainer.party {
            let species = policy.target(pack, &member.species, Some(member.level))?;
            if species != member.species {
                member.moves = pack
                    .data
                    .create_pokemon(&species, member.level, member.dvs)?
                    .moves;
                member.species = species;
            }
        }
        pack.data.trainers.trainers.insert(id, trainer);
    }
    Ok(())
}
