//! Restore accepted private cast bindings after the bounded story-program merge.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::CompiledGamePack;

const MARKER: &str = "northstar-current-cast-integration-v1";

#[derive(Deserialize)]
struct Binding {
    trainer_class: String,
    trainer_ids: Vec<String>,
    actors: Vec<Actor>,
}

#[derive(Deserialize)]
struct Actor {
    map: String,
    script: String,
    tile: [u16; 2],
    previous_sprite: String,
}

/// Installs the exact reviewed Reed and Tamarack art and existing actor bindings.
///
/// Teams, actor positions, terrain, story logic, original assets and audio are
/// preserved. This creates a new identity; it never migrates existing saves.
///
/// # Errors
/// Rejects a different donor, repeated installation, stale actor or trainer
/// preimages, occupied assets, missing event mirrors or invalid runtime data.
pub fn complete_northstar_story_cast(
    current: &CompiledGamePack,
    donor: &CompiledGamePack,
) -> Result<CompiledGamePack> {
    ensure!(
        donor.identity()?.content_hash
            == "74bfb7af0c8147c13812005770425d82a4dbf5a38655ac3e563d49708c7c11eb",
        "exact reviewed cast donor required"
    );
    ensure!(
        current
            .report
            .manifests
            .iter()
            .any(|m| m == "northstar-current-story-refresh-v2")
            && !current.report.manifests.iter().any(|m| m == MARKER),
        "fresh refreshed story required"
    );
    crate::verify_compiled_game_pack_for_runtime(current)?;
    let mut pack = current.clone();
    for source in [
        include_str!("../../../planning/minnesota/npcs/tamarack-ice/01-tamarack/binding-v1.json"),
        include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-bindings/01-reed.json"
        ),
    ] {
        let binding: Binding = serde_json::from_str(source)?;
        let sprite = format!("SPRITE_{}", binding.trainer_class);
        let stem = binding.trainer_class.to_ascii_lowercase();
        for folder in ["sprites", "trainers"] {
            for extension in ["2bpp", "gbcpal", "png"] {
                let path = format!("gfx/{folder}/{stem}.{extension}");
                ensure!(
                    !pack.runtime_files.contains_key(&path),
                    "preserve existing cast asset {path}"
                );
                let bytes = donor
                    .runtime_files
                    .get(&path)
                    .context("reviewed cast asset")?;
                pack.runtime_files.insert(path, bytes.clone());
            }
        }
        ensure!(
            !pack.data.sprite_palette_defaults.contains_key(&sprite),
            "preserve existing sprite registration"
        );
        pack.data.sprite_palette_defaults.insert(
            sprite.clone(),
            *donor
                .data
                .sprite_palette_defaults
                .get(&sprite)
                .context("donor sprite palette")?,
        );
        for id in &binding.trainer_ids {
            let trainer = pack
                .data
                .trainers
                .trainers
                .get_mut(id)
                .context("existing Reed trainer")?;
            ensure!(
                trainer.trainer_class == "RIVAL1",
                "stale trainer class {id}"
            );
            let mut expected = donor
                .data
                .trainers
                .trainers
                .get(id)
                .context("reviewed Reed trainer")?
                .clone();
            expected.trainer_class = "RIVAL1".into();
            ensure!(
                *trainer == expected,
                "preserve trainer party and mechanics {id}"
            );
            trainer.trainer_class.clone_from(&binding.trainer_class);
        }
        if !binding.trainer_ids.is_empty() {
            ensure!(
                !pack
                    .data
                    .trainer_class_names
                    .contains_key(&binding.trainer_class),
                "preserve existing class"
            );
            pack.data.trainer_class_names.insert(
                binding.trainer_class.clone(),
                donor
                    .data
                    .trainer_class_names
                    .get(&binding.trainer_class)
                    .context("reviewed class name")?
                    .clone(),
            );
        }
        for actor in binding.actors {
            let map = pack.data.maps.get_mut(&actor.map).context("cast map")?;
            let matches: Vec<_> = map
                .objects
                .iter_mut()
                .filter(|o| o.script == actor.script && [o.x, o.y] == actor.tile)
                .collect();
            ensure!(matches.len() == 1, "one exact cast actor required");
            let object = matches.into_iter().next().context("cast actor")?;
            ensure!(object.sprite == actor.previous_sprite, "stale actor sprite");
            object.sprite.clone_from(&sprite);
            object.sprite_has_facings = true;
            let mut matched = 0;
            for row in &mut map.map_event_section_commands {
                if row.command == "object_event" && event_matches(&row.args, &actor) {
                    ensure!(
                        row.args[2] == actor.previous_sprite,
                        "stale cached event sprite"
                    );
                    row.args[2].clone_from(&sprite);
                    matched += 1;
                }
            }
            ensure!(matched == 1, "one cached event actor required");
            let label = map
                .attributes
                .map_events_label
                .as_ref()
                .context("cast events label")?
                .clone();
            if let Some(body) = map.scripts.get_mut(&label) {
                patch_event_body(body, &actor, &sprite)?;
            }
            if let Some(body) = pack.data.map_scripts.get_mut(&label) {
                patch_event_body(body, &actor, &sprite)?;
            }
            let rows = pack
                .data
                .npcs
                .get_mut(&actor.map)
                .context("raw cast actors")?
                .as_array_mut()
                .context("raw actor array")?;
            let mut matched = 0;
            for row in rows {
                if row["script"] == actor.script
                    && row["x"] == actor.tile[0]
                    && row["y"] == actor.tile[1]
                {
                    ensure!(row["sprite"] == actor.previous_sprite, "stale raw sprite");
                    row["sprite"] = sprite.clone().into();
                    row["sprite_has_facings"] = true.into();
                    matched += 1;
                }
            }
            ensure!(matched == 1, "one raw cast actor required");
        }
    }
    for map in pack
        .data
        .maps
        .values()
        .filter(|map| map.id.starts_with("Mn"))
    {
        for request in map
            .trainer_scripts
            .values()
            .chain(
                map.scripted_trainer_battles
                    .iter()
                    .map(|battle| &battle.request),
            )
            .filter(|request| request.trainer_id.starts_with("MN_"))
        {
            let trainer = pack
                .data
                .trainers
                .trainers
                .get(&request.trainer_id)
                .context("regional trainer request catalog entry")?;
            ensure!(
                request.trainer_class == trainer.trainer_class,
                "regional trainer class mismatch: {} in {}",
                request.trainer_id,
                map.id
            );
        }
    }
    pack.report.manifests.push(MARKER.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    crate::verify_northstar_publication_geometry(&pack)?;
    crate::validate_compact_superior_geography(&pack)?;
    Ok(pack)
}

fn event_matches(args: &[String], actor: &Actor) -> bool {
    args.len() > 11
        && args[0] == actor.tile[0].to_string()
        && args[1] == actor.tile[1].to_string()
        && args[11] == actor.script
}

fn patch_event_body(body: &mut Value, actor: &Actor, sprite: &str) -> Result<()> {
    let mut matched = 0;
    for row in body.as_array_mut().context("raw cast event body")? {
        if row["command"] != "object_event" {
            continue;
        }
        let args: Vec<String> = serde_json::from_value(row["args"].clone())?;
        if event_matches(&args, actor) {
            ensure!(args[2] == actor.previous_sprite, "stale raw event sprite");
            row["args"][2] = sprite.into();
            matched += 1;
        }
    }
    ensure!(matched == 1, "one raw event actor required");
    Ok(())
}
