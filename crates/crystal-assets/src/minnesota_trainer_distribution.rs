//! Reuse accepted regular trainer art across existing route and fair encounters.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, authored_town_publication};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Distribution {
    schema_version: u8,
    assignments: Vec<Assignment>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Assignment {
    trainer_id: String,
    trainer_class: String,
    map: String,
    script: String,
    tile: [u16; 2],
    previous_sprite: String,
}

/// Reuses installed regular trainer art without changing encounter behavior.
///
/// Only trainer classes, matching script class operands and exact actor sprites
/// change. Parties, names, positions, movement, flags, rewards and audio survive.
///
/// # Errors
/// Rejects boss/rival targets, missing art, stale or duplicate actors/identities,
/// unmatched battle scripts and invalid resulting packs.
pub fn distribute_minnesota_trainer_art(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let source: Distribution = serde_json::from_str(source)?;
    ensure!(
        source.schema_version == 1 && !source.assignments.is_empty(),
        "assignments required"
    );
    let mut assignments = BTreeMap::new();
    let mut actors = BTreeSet::new();
    for row in &source.assignments {
        let numbered_route = row.trainer_id.strip_prefix("MN_R").is_some_and(|tail| {
            tail.as_bytes()
                .get(..2)
                .is_some_and(|digits| digits.iter().all(u8::is_ascii_digit))
                && tail.as_bytes().get(2) == Some(&b'_')
        });
        ensure!(
            numbered_route || row.trainer_id.starts_with("MN_FAIR_"),
            "only ordinary route/fair opponents may be reassigned"
        );
        ensure!(
            (numbered_route && row.map.starts_with("MnR"))
                || (row.trainer_id.starts_with("MN_FAIR_") && row.map == "MnStateFairgrounds"),
            "encounter must remain on its route/fair map"
        );
        ensure!(
            row.trainer_class.starts_with("MN_")
                && !["MN_LEADER_", "MN_ELITE_", "MN_RIVAL_"]
                    .iter()
                    .any(|p| row.trainer_class.starts_with(p))
                && base
                    .data
                    .trainer_class_names
                    .contains_key(&row.trainer_class),
            "installed regular trainer class required"
        );
        let stem = row.trainer_class.to_ascii_lowercase();
        for kind in ["sprites", "trainers"] {
            for extension in ["png", "2bpp", "gbcpal"] {
                ensure!(
                    base.runtime_files
                        .contains_key(&format!("gfx/{kind}/{stem}.{extension}")),
                    "missing accepted art"
                );
            }
        }
        ensure!(
            assignments.insert(row.trainer_id.clone(), row).is_none(),
            "duplicate trainer"
        );
        ensure!(
            actors.insert((&row.map, &row.script, row.tile)),
            "duplicate actor"
        );
    }
    let mut pack = base.clone();
    for row in &source.assignments {
        pack.data
            .trainers
            .trainers
            .get_mut(&row.trainer_id)
            .context("existing opponent")?
            .trainer_class
            .clone_from(&row.trainer_class);
    }
    let mut actor_count = 0;
    let mut references = BTreeSet::new();
    for original in base.data.maps.values() {
        let mut map = original.clone();
        let mut changed = false;
        for row in source.assignments.iter().filter(|r| r.map == map.id) {
            let matches: Vec<_> = map
                .objects
                .iter_mut()
                .filter(|o| o.script == row.script && [o.x, o.y] == row.tile)
                .collect();
            ensure!(
                matches.len() == 1,
                "one exact actor required: {}",
                row.trainer_id
            );
            for object in matches {
                ensure!(
                    object.sprite == row.previous_sprite,
                    "stale actor: {}",
                    row.trainer_id
                );
                object.sprite = format!("SPRITE_{}", row.trainer_class);
                object.sprite_has_facings = true;
            }
            actor_count += 1;
            changed = true;
        }
        for body in map.scripts.values_mut() {
            let Some(commands) = body.as_array_mut() else {
                continue;
            };
            for command in commands {
                if !matches!(command["command"].as_str(), Some("trainer" | "loadtrainer")) {
                    continue;
                }
                let Some(id) = command["args"][1].as_str() else {
                    continue;
                };
                let Some(row) = assignments.get(id) else {
                    continue;
                };
                ensure!(
                    command["args"][0].as_str()
                        == Some(
                            &base
                                .data
                                .trainers
                                .get(id)
                                .context("old trainer")?
                                .trainer_class
                        ),
                    "stale script class"
                );
                references.insert(id.to_owned());
                command["args"][0] = row.trainer_class.clone().into();
                changed = true;
            }
        }
        if changed {
            authoring::refresh_event_section(&mut map);
            map.script_runtime_commands =
                crate::parse_script_runtime_commands(&map.id, &map.scripts)?;
            map.script_object_commands =
                crate::parse_script_object_commands(&map.id, &map.scripts)?;
            map.trainer_scripts = crate::parse_trainer_scripts(&map.id, &map.scripts)?;
            map.scripted_trainer_battles =
                crate::parse_scripted_trainer_battles(&map.id, &map.scripts)?;
            authored_town_publication::replace(&mut pack, map)?;
        }
    }
    ensure!(
        actor_count == assignments.len() && references.len() == assignments.len(),
        "all actors and battle scripts must match"
    );
    pack.report
        .manifests
        .push("minnesota-regular-trainer-distribution-v1".into());
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
