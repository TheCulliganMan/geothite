//! Reuse accepted regular trainer art across existing Minnesota encounters.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::grand_marais_modpack::authoring;
use crate::minnesota_trainer_art::rebind_actor_event_source;
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
    #[serde(default)]
    previous_class: Option<String>,
    map: String,
    script: String,
    tile: [u16; 2],
    previous_sprite: String,
}

fn ordinary_minnesota_trainer(id: &str) -> bool {
    let digits = |text: &str| !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit());
    let route = id
        .strip_prefix("MN_R")
        .and_then(|tail| tail.split_once('_'))
        .is_some_and(|(number, trainer)| {
            number.len() == 2 && digits(number) && trainer.strip_prefix('T').is_some_and(digits)
        });
    let gym_aide = id
        .strip_prefix("MN_GYM_")
        .and_then(|tail| tail.split_once("_TRAINER_"))
        .is_some_and(|(gym, trainer)| digits(gym) && digits(trainer));
    route
        || gym_aide
        || [
            "MN_FAIR_",
            "MN_RIVER_",
            "MN_EXTRA_",
            "MN_DULUTH_X_",
            "MN_SPX_",
            "MN_COHESION_",
            "MN_MP_",
        ]
        .iter()
        .any(|prefix| id.starts_with(prefix))
        || matches!(id, "MN_ICE_BOARDER_NILS" | "MN_ICE_SKIER_ELSA")
}

/// Reuses installed regular trainer art without changing encounter behavior.
///
/// Only trainer classes, matching script class operands and exact actor sprites
/// change. Parties, names, positions, movement, flags, rewards and audio survive.
///
/// # Errors
/// Rejects boss/rival/faction targets, missing art, stale or duplicate
/// actors/identities, actors belonging to a different trainer, unmatched battle
/// scripts and invalid resulting packs. An optional `previous_class` pins an
/// assignment to its reviewed trainer-class preimage.
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
        ensure!(
            ordinary_minnesota_trainer(&row.trainer_id),
            "only ordinary Minnesota opponents may be reassigned"
        );
        ensure!(
            row.map.starts_with("Mn"),
            "encounter must remain in Minnesota"
        );
        let previous = base
            .data
            .trainers
            .get(&row.trainer_id)
            .context("existing opponent")?;
        ensure!(
            row.previous_class
                .as_ref()
                .is_none_or(|class| class == &previous.trainer_class),
            "stale trainer class: {}",
            row.trainer_id
        );
        ensure!(
            row.trainer_class.starts_with("MN_")
                && ![
                    "MN_LEADER_",
                    "MN_ELITE_",
                    "MN_RIVAL_",
                    "MN_ICE_",
                    "MN_PROF_"
                ]
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
        for row in source.assignments.iter().filter(|r| r.map == original.id) {
            let sprite = format!("SPRITE_{}", row.trainer_class);
            ensure!(
                map.trainer_scripts
                    .get(&row.script)
                    .is_some_and(|request| { request.trainer_id == row.trainer_id })
                    || map.scripted_trainer_battles.iter().any(|battle| {
                        battle.source_script == row.script
                            && battle.request.trainer_id == row.trainer_id
                    }),
                "actor must belong to the selected opponent: {}",
                row.trainer_id
            );
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
                object.sprite.clone_from(&sprite);
                object.sprite_has_facings = true;
            }
            rebind_actor_event_source(&mut map, &row.script, row.tile, &sprite)?;
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

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::PathBuf;

    use super::{Distribution, distribute_minnesota_trainer_art, ordinary_minnesota_trainer};

    #[test]
    fn regional_distribution_protects_story_opponents() {
        for id in [
            "MN_R02_T01",
            "MN_RIVER_BANK_TRAINER",
            "MN_EXTRA_MPLS_CYCLE_01",
            "MN_GYM_8_TRAINER_3",
            "MN_FAIR_RICE_TRAINER",
            "MN_DULUTH_X_T01",
            "MN_ICE_SKIER_ELSA",
        ] {
            assert!(ordinary_minnesota_trainer(id), "ordinary opponent: {id}");
        }
        for id in [
            "MN_CHAMPION",
            "MN_GYM_8",
            "MN_REED_WINONA",
            "MN_LEAGUE_4",
            "MN_ICE_ENFORCER",
            "MN_ICE_HQ_ARCHIVE",
            "MN_LODE_CHIEF",
            "MN_R02_BOSS",
            "FALKNER1",
        ] {
            assert!(!ordinary_minnesota_trainer(id), "protected identity: {id}");
        }
    }

    #[test]
    #[ignore = "requires CRYSTAL_DISTRIBUTION_PARENT and SOURCE with installed regional art"]
    fn regional_distribution_preserves_encounters_and_rejects_stale_preimages() {
        let fixture = |key| std::env::var_os(key).expect("explicit distribution fixture");
        let base = crate::read_verified_compiled_game_pack(fixture("CRYSTAL_DISTRIBUTION_PARENT"))
            .expect("review parent");
        let source = std::fs::read_to_string(fixture("CRYSTAL_DISTRIBUTION_SOURCE"))
            .expect("review assignments");
        let plan: Distribution = serde_json::from_str(&source).expect("plan");
        let output = distribute_minnesota_trainer_art(&base, &source).expect("regional assignment");
        assert_scaled_regional_roster(&output);
        let classes: BTreeMap<_, _> = plan
            .assignments
            .iter()
            .map(|row| (row.trainer_id.as_str(), &row.trainer_class))
            .collect();
        assert_eq!(base.runtime_files, output.runtime_files, "no art mutation");
        assert_eq!(base.compiled_audio(), output.compiled_audio(), "audio");
        for (id, trainer) in &base.data.trainers.trainers {
            let mut expected = trainer.clone();
            if let Some(class) = classes.get(id.as_str()) {
                expected.trainer_class.clone_from(class);
            }
            assert_eq!(
                output.data.trainers.get(id),
                Some(&expected),
                "party/AI/name: {id}"
            );
        }
        let mut before = serde_json::to_value(&base.data).expect("original data");
        let mut after = serde_json::to_value(&output.data).expect("result data");
        for value in [&mut before, &mut after] {
            let fields = value.as_object_mut().expect("game data");
            fields.remove("maps");
            fields.remove("trainers");
            fields.remove("map_scripts");
            fields.remove("npcs");
        }
        for (key, value) in before.as_object().expect("game data") {
            assert!(
                after.get(key) == Some(value),
                "unexpected game catalog or setting change: {key}"
            );
        }
        let mut expected_global_scripts = base.data.map_scripts.clone();
        let mut expected_global_npcs = base.data.npcs.clone();
        for (id, original) in &base.data.maps {
            let revised = &output.data.maps[id];
            assert_eq!(
                original.attributes, revised.attributes,
                "map attributes {id}"
            );
            assert_eq!(original.blocks, revised.blocks, "geometry {id}");
            assert_eq!(original.events, revised.events, "events {id}");
            assert_eq!(
                original.script_text_bodies, revised.script_text_bodies,
                "dialogue {id}"
            );
            let mut objects = original.objects.clone();
            for row in plan.assignments.iter().filter(|row| row.map == *id) {
                for object in &mut objects {
                    if object.script == row.script && [object.x, object.y] == row.tile {
                        object.sprite = format!("SPRITE_{}", row.trainer_class);
                        object.sprite_has_facings = true;
                    }
                }
            }
            assert_eq!(
                revised.objects, objects,
                "actor behavior and positions {id}"
            );
            if objects != original.objects {
                expected_global_npcs.insert(
                    id.clone(),
                    serde_json::to_value(&objects).expect("expected NPC mirror"),
                );
            }
            let mut events = original.map_event_section_commands.clone();
            for row in plan.assignments.iter().filter(|row| row.map == *id) {
                let x = row.tile[0].to_string();
                let y = row.tile[1].to_string();
                let mut matched = 0;
                for command in &mut events {
                    if command.command == "object_event"
                        && command.args.first() == Some(&x)
                        && command.args.get(1) == Some(&y)
                        && command.args.get(11) == Some(&row.script)
                    {
                        command.args[2] = format!("SPRITE_{}", row.trainer_class);
                        matched += 1;
                    }
                }
                assert_eq!(matched, 1, "one exact event mirror: {}", row.trainer_id);
            }
            assert!(
                revised.map_event_section_commands == events,
                "only the selected actor sprite operands change: {id}"
            );
            if events != original.map_event_section_commands {
                expected_global_scripts.insert(
                    original
                        .attributes
                        .map_events_label
                        .clone()
                        .expect("event section label"),
                    serde_json::Value::Array(
                        events
                            .iter()
                            .map(|command| {
                                serde_json::json!({
                                    "command": command.command,
                                    "args": command.args,
                                })
                            })
                            .collect(),
                    ),
                );
            }
            let mut scripts = original.scripts.clone();
            for body in scripts
                .values_mut()
                .filter_map(serde_json::Value::as_array_mut)
            {
                for command in body {
                    if matches!(command["command"].as_str(), Some("trainer" | "loadtrainer"))
                        && let Some(class) = command["args"][1]
                            .as_str()
                            .and_then(|trainer| classes.get(trainer))
                    {
                        command["args"][0] = (*class).clone().into();
                    }
                }
            }
            for row in plan.assignments.iter().filter(|row| row.map == *id) {
                let Some(label) = &original.attributes.map_events_label else {
                    continue;
                };
                let Some(body) = scripts
                    .get_mut(label)
                    .and_then(serde_json::Value::as_array_mut)
                else {
                    continue;
                };
                let x = row.tile[0].to_string();
                let y = row.tile[1].to_string();
                for command in body {
                    if command["command"].as_str() == Some("object_event")
                        && command["args"][0].as_str() == Some(x.as_str())
                        && command["args"][1].as_str() == Some(y.as_str())
                        && command["args"][11].as_str() == Some(&row.script)
                    {
                        command["args"][2] = format!("SPRITE_{}", row.trainer_class).into();
                    }
                }
            }
            assert!(
                revised.scripts == scripts,
                "only class operands change {id}"
            );
            for (label, body) in scripts {
                // The split payload uses the canonical typed event section,
                // which was checked separately above, rather than its raw source.
                if original.scripts.get(&label) != Some(&body)
                    && original.attributes.map_events_label.as_ref() != Some(&label)
                {
                    expected_global_scripts.insert(label, body);
                }
            }
        }
        let changed_script_labels: Vec<_> = output
            .data
            .map_scripts
            .iter()
            .filter(|(label, body)| expected_global_scripts.get(*label) != Some(*body))
            .map(|(label, _)| label)
            .take(8)
            .collect();
        assert!(
            output.data.map_scripts == expected_global_scripts,
            "the global script mirror changes only matching class/sprite operands: {changed_script_labels:?}"
        );
        assert!(
            output.data.npcs == expected_global_npcs,
            "the global NPC mirror changes only selected field sprites"
        );
        let mut stale: serde_json::Value = serde_json::from_str(&source).expect("JSON source");
        stale["assignments"][0]["previous_class"] = "STALE_CLASS".into();
        assert!(distribute_minnesota_trainer_art(&base, &stale.to_string()).is_err());
        stale["assignments"][0]["previous_class"] = serde_json::Value::Null;
        stale["assignments"][0]["previous_sprite"] = "SPRITE_STALE".into();
        assert!(distribute_minnesota_trainer_art(&base, &stale.to_string()).is_err());
    }

    fn assert_scaled_regional_roster(pack: &crate::CompiledGamePack) {
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota/pokemon");
        let read = |name| {
            serde_json::from_str::<serde_json::Value>(
                &std::fs::read_to_string(root.join(name)).expect("canonical roster source"),
            )
            .expect("canonical roster JSON")
        };
        let roster = read("roster.json");
        let curation = read("roster-curation.json");
        let retired: BTreeSet<_> = curation["retired_families"]
            .as_array()
            .expect("retired families")
            .iter()
            .map(|family| family["id"].as_str().expect("family ID"))
            .collect();
        let active: BTreeSet<_> = roster["families"]
            .as_array()
            .expect("roster families")
            .iter()
            .filter(|family| !retired.contains(family["id"].as_str().expect("family ID")))
            .flat_map(|family| family["members"].as_array().expect("family members"))
            .map(|member| {
                member["name"]
                    .as_str()
                    .expect("species name")
                    .to_ascii_uppercase()
            })
            .collect();
        // Check the production rematch path even when this world's automatic
        // scaling is disabled. Art distribution must preserve that setting.
        for (id, trainer) in &pack.data.trainers.trainers {
            if !id.starts_with("MN_") {
                continue;
            }
            let first_ace = trainer
                .party
                .iter()
                .map(|member| member.level)
                .max()
                .expect("authored trainer party");
            for ace in first_ace..=100 {
                let scaled = pack
                    .data
                    .rematch_trainer_at_ace(trainer, ace)
                    .expect("actual production team scaling and evolved learnsets");
                for member in scaled.party {
                    assert!(
                        active.contains(&member.species),
                        "active Minnesota evolution required: {id}, ace {ace}, {}",
                        member.species
                    );
                }
            }
        }
    }
}
