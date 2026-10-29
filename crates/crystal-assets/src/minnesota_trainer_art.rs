//! Private art bindings for an existing regional character and its battle variants.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, authored_town_publication};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    schema_version: u8,
    #[serde(default)]
    field_only: bool,
    trainer_class: String,
    class_name: String,
    trainer_ids: Vec<String>,
    actors: Vec<Actor>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Actor {
    map: String,
    script: String,
    tile: [u16; 2],
    previous_sprite: String,
}

/// Installs matched native art for one existing Minnesota character.
///
/// Trainer identities, parties, AI, rewards, constants, terrain and script flow
/// remain unchanged. Only the selected trainers' private art class, matching
/// loadtrainer/trainer operands, and explicitly selected actors are rebound.
/// A `field_only` binding requires an empty `trainer_ids` list and changes only
/// field actors and their art registration; it adds no battle or trainer class.
///
/// # Errors
/// Rejects missing/duplicate identities, occupied art/classes, stale actor
/// preimages, incomplete native banks and invalid resulting runtime packs.
pub fn bind_minnesota_trainer_art(
    base: &CompiledGamePack,
    source: &str,
    art: &BTreeMap<String, Vec<u8>>,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let binding: Binding = serde_json::from_str(source)?;
    let class = &binding.trainer_class;
    ensure!(
        binding.schema_version == 1
            && class.starts_with("MN_")
            && class.bytes().all(|b| b.is_ascii_uppercase() || b == b'_')
            && !binding.class_name.is_empty()
            && binding.class_name.chars().count() <= 18,
        "native private character class required"
    );
    let ids: BTreeSet<_> = binding.trainer_ids.iter().collect();
    ensure!(
        binding.field_only == ids.is_empty()
            && ids.len() == binding.trainer_ids.len()
            && !binding.actors.is_empty(),
        "unique battle identities (or explicit field_only) and field actors required"
    );
    let sprite = format!("SPRITE_{class}");
    ensure!(
        !base.data.trainer_class_names.contains_key(class)
            && !base.data.sprite_palette_defaults.contains_key(&sprite),
        "preserve existing character class"
    );
    super::minnesota_trainers::verify_original_npc_bundle(art, &class.to_ascii_lowercase())?;
    let mut pack = base.clone();
    let mut old_classes = BTreeMap::new();
    for id in &binding.trainer_ids {
        ensure!(id.starts_with("MN_"), "regional trainer identity required");
        let trainer = pack
            .data
            .trainers
            .trainers
            .get_mut(id)
            .context("existing trainer")?;
        old_classes.insert(id.clone(), trainer.trainer_class.clone());
        trainer.trainer_class.clone_from(class);
    }
    for (path, bytes) in art {
        ensure!(
            !pack.runtime_files.contains_key(path),
            "preserve existing art: {path}"
        );
        pack.runtime_files.insert(path.clone(), bytes.clone());
    }
    if !binding.field_only {
        pack.data
            .trainer_class_names
            .insert(class.clone(), binding.class_name);
    }
    pack.data.sprite_palette_defaults.insert(sprite.clone(), 0);
    let mut selected_actors = BTreeSet::new();
    for actor in &binding.actors {
        ensure!(
            actor.map.starts_with("Mn")
                && selected_actors.insert((&actor.map, &actor.script, actor.tile)),
            "unique regional actor required"
        );
    }
    let mut matched_actors = 0;
    let mut script_references = BTreeSet::new();
    for original in base.data.maps.values() {
        let mut map = original.clone();
        let mut changed = false;
        for actor in binding.actors.iter().filter(|a| a.map == map.id) {
            let matches: Vec<_> = map
                .objects
                .iter_mut()
                .filter(|o| o.script == actor.script && [o.x, o.y] == actor.tile)
                .collect();
            ensure!(
                matches.len() == 1,
                "one exact field actor required: {}",
                actor.script
            );
            for object in matches {
                ensure!(
                    object.sprite == actor.previous_sprite,
                    "stale field actor sprite"
                );
                object.sprite.clone_from(&sprite);
                object.sprite_has_facings = true;
            }
            matched_actors += 1;
            changed = true;
        }
        for body in map.scripts.values_mut() {
            let Some(commands) = body.as_array_mut() else {
                continue;
            };
            for command in commands {
                let kind = command["command"].as_str().unwrap_or("").to_owned();
                if !matches!(kind.as_str(), "loadtrainer" | "trainer") {
                    continue;
                }
                let Some(args) = command["args"].as_array_mut() else {
                    continue;
                };
                let Some(id) = args.get(1).and_then(|v| v.as_str()).map(str::to_owned) else {
                    continue;
                };
                let Some(old_class) = old_classes.get(&id) else {
                    continue;
                };
                ensure!(
                    args.first().and_then(|v| v.as_str()) == Some(old_class),
                    "stale battle class: {id}"
                );
                args[0] = serde_json::Value::String(class.clone());
                script_references.insert(id);
                changed = true;
            }
        }
        if changed {
            authoring::finish(&pack, &mut map)?;
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
        matched_actors == binding.actors.len(),
        "all selected actors must exist"
    );
    ensure!(
        script_references.len() == ids.len(),
        "every selected variant must have a battle script"
    );
    pack.report.manifests.push(format!(
        "native-character-art-{}-v1",
        class.to_ascii_lowercase()
    ));
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
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use super::bind_minnesota_trainer_art;
    use crate::read_verified_compiled_game_pack;

    #[test]
    #[ignore = "requires CRYSTAL_CAST_PARENT, BINDING and ART external fixtures"]
    fn existing_character_art_preserves_story_and_rejects_stale_binding() {
        let fixture = |key| PathBuf::from(std::env::var_os(key).expect("explicit cast fixture"));
        let base =
            read_verified_compiled_game_pack(fixture("CRYSTAL_CAST_PARENT")).expect("parent");
        let source = std::fs::read_to_string(fixture("CRYSTAL_CAST_BINDING")).expect("binding");
        let row: serde_json::Value = serde_json::from_str(&source).expect("JSON");
        let class = row["trainer_class"].as_str().expect("class");
        let stem = class.to_ascii_lowercase();
        let mut art = BTreeMap::new();
        for kind in ["sprites", "trainers"] {
            for extension in ["png", "2bpp", "gbcpal"] {
                let path = format!("gfx/{kind}/{stem}.{extension}");
                art.insert(
                    path.clone(),
                    std::fs::read(fixture("CRYSTAL_CAST_ART").join(path)).expect("native bank"),
                );
            }
        }
        let output =
            bind_minnesota_trainer_art(&base, &source, &art).expect("valid existing character");
        if row["field_only"].as_bool() == Some(true) {
            assert_eq!(
                output.data.trainer_class_names, base.data.trainer_class_names,
                "a field-only scientist must not gain a battle class"
            );
            assert_eq!(row["actors"].as_array().expect("actors").len(), 1);
            let sprite = format!("SPRITE_{class}");
            let previous = row["actors"][0]["previous_sprite"]
                .as_str()
                .expect("original field sprite");
            let mut normalized = serde_json::to_value(&output.data).expect("game data");
            normalized["sprite_palette_defaults"]
                .as_object_mut()
                .expect("sprite defaults")
                .remove(&sprite);
            restore_sprite(&mut normalized, &sprite, previous);
            assert_eq!(
                normalized,
                serde_json::to_value(&base.data).expect("original game data"),
                "all gameplay data must match after removing the field art binding"
            );
            let mut implicit = row.clone();
            implicit
                .as_object_mut()
                .expect("binding")
                .remove("field_only");
            assert!(
                bind_minnesota_trainer_art(&base, &implicit.to_string(), &art).is_err(),
                "empty battle identities require explicit field-only intent"
            );
        }
        assert_eq!(base.compiled_audio(), output.compiled_audio());
        for (path, bytes) in base.runtime_files() {
            assert_eq!(
                output.runtime_files().get(path),
                Some(bytes),
                "original resource {path}"
            );
        }
        assert_eq!(
            base.data.trainers.trainers.len(),
            output.data.trainers.trainers.len()
        );
        for (id, original) in &base.data.trainers.trainers {
            let mut expected = original.clone();
            if row["trainer_ids"]
                .as_array()
                .expect("variants")
                .iter()
                .any(|v| v.as_str() == Some(id))
            {
                expected.trainer_class = class.into();
            }
            assert_eq!(
                output.data.trainers.get(id),
                Some(&expected),
                "no party/AI/reward/identity change: {id}"
            );
        }
        for (id, original) in &base.data.maps {
            let revised = &output.data.maps[id];
            assert_eq!(original.blocks, revised.blocks, "terrain {id}");
            assert_eq!(original.events, revised.events, "events {id}");
            assert_eq!(
                original.script_text_bodies, revised.script_text_bodies,
                "dialogue {id}"
            );
            let mut expected_scripts = original.scripts.clone();
            for body in expected_scripts.values_mut() {
                if let Some(commands) = body.as_array_mut() {
                    for command in commands {
                        if matches!(command["command"].as_str(), Some("trainer" | "loadtrainer"))
                            && row["trainer_ids"]
                                .as_array()
                                .expect("IDs")
                                .contains(&command["args"][1])
                        {
                            command["args"][0] = class.into();
                        }
                    }
                }
            }
            assert_eq!(
                revised.scripts, expected_scripts,
                "only art-class operands change: {id}"
            );
            let mut expected_objects = original.objects.clone();
            for actor in row["actors"].as_array().expect("actors") {
                if actor["map"].as_str() == Some(id) {
                    for object in &mut expected_objects {
                        if actor["script"].as_str() == Some(&object.script)
                            && actor["tile"] == serde_json::json!([object.x, object.y])
                        {
                            object.sprite = format!("SPRITE_{class}");
                            object.sprite_has_facings = true;
                        }
                    }
                }
            }
            assert_eq!(
                revised.objects, expected_objects,
                "only requested actor art changes: {id}"
            );
        }
        assert!(
            bind_minnesota_trainer_art(&output, &source, &art).is_err(),
            "repeated publication"
        );
        let mut stale = row.clone();
        stale["actors"][0]["previous_sprite"] = "SPRITE_WRONG".into();
        assert!(bind_minnesota_trainer_art(&base, &stale.to_string(), &art).is_err());
        if !row["trainer_ids"].as_array().expect("IDs").is_empty() {
            let mut missing = row.clone();
            missing["trainer_ids"][0] = "MN_MISSING_CHARACTER".into();
            assert!(bind_minnesota_trainer_art(&base, &missing.to_string(), &art).is_err());
            let mut duplicate = row.clone();
            duplicate["trainer_ids"]
                .as_array_mut()
                .expect("IDs")
                .push(row["trainer_ids"][0].clone());
            assert!(bind_minnesota_trainer_art(&base, &duplicate.to_string(), &art).is_err());
        }
        art.remove(&format!("gfx/sprites/{stem}.2bpp"));
        assert!(
            bind_minnesota_trainer_art(&base, &source, &art).is_err(),
            "partial native bundle"
        );
    }

    fn restore_sprite(value: &mut serde_json::Value, sprite: &str, previous: &str) {
        match value {
            serde_json::Value::String(text) if text == sprite => {
                previous.clone_into(text);
            }
            serde_json::Value::Array(items) => {
                for item in items {
                    restore_sprite(item, sprite, previous);
                }
            }
            serde_json::Value::Object(fields) => {
                for field in fields.values_mut() {
                    restore_sprite(field, sprite, previous);
                }
            }
            _ => {}
        }
    }
}
