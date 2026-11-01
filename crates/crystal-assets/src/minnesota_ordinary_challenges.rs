//! Convert ordinary Minnesota trainers to native trainer encounters.

use std::collections::BTreeSet;

use anyhow::{Result, ensure};
use serde_json::json;

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, authored_town_publication};

/// Converts the authored optional ordinary trainer prompts in an existing pack into
/// ordinary trainer-table encounters. The original pack and saves are untouched.
///
/// # Errors
/// Rejects invalid packs, inconsistent event rows, missing post-battle scripts, or a pack without matching challenges.
pub fn revise_minnesota_ordinary_challenges(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let mut pack = base.clone();
    let names = pack.data.map_name_by_constant_from_attributes()?;
    let mut count = 0;
    let map_ids: Vec<_> = pack
        .data
        .maps
        .keys()
        .filter(|id| id.starts_with("Mn"))
        .cloned()
        .collect();
    for map_id in map_ids {
        let mut map = pack.data.maps[&map_id].clone();
        let mut changed = BTreeSet::new();
        for object in &mut map.objects {
            let Some(id) = object.object_identifier.as_deref() else {
                continue;
            };
            if !id.starts_with("MN_EXTRA_")
                || object.object_type != "OBJECTTYPE_SCRIPT"
                || object.radius != 0
                || object.script != id
            {
                continue;
            }
            let Some(trainer) = pack.data.trainers.get(id) else {
                continue;
            };
            let Some(script) = map.scripts.get(id) else {
                continue;
            };
            let Some(replacement) = native_trainer_challenge(id, &trainer.trainer_class, script)
            else {
                continue;
            };
            let after = format!("{id}After");
            ensure!(
                map.scripts.contains_key(&after),
                "missing trainer after-script {id}"
            );
            map.scripts.insert(id.to_owned(), replacement);
            object.object_type = "OBJECTTYPE_TRAINER".into();
            object.radius = 3;
            count += 1;
            changed.insert(id.to_owned());
        }
        if !changed.is_empty() {
            let mut updated_rows = BTreeSet::new();
            for row in &mut map.map_event_section_commands {
                if row.command != "object_event"
                    || row.args.len() != 13
                    || !changed.contains(&row.args[11])
                {
                    continue;
                }
                ensure!(
                    row.args[9] == "OBJECTTYPE_SCRIPT" && row.args[10] == "0",
                    "unexpected object event row for {}",
                    row.args[11]
                );
                ensure!(
                    updated_rows.insert(row.args[11].clone()),
                    "duplicate object event row for {}",
                    row.args[11]
                );
                row.args[9] = "OBJECTTYPE_TRAINER".into();
                row.args[10] = "3".into();
            }
            ensure!(
                updated_rows == changed,
                "missing ordinary trainer event rows on {map_id}"
            );
            map.script_map_commands
                .retain(|command| !changed.contains(&command.source_script));
            map.script_map_commands.extend(
                crate::parse_script_map_commands(&map.id, &map.scripts, &names)?
                    .into_iter()
                    .filter(|command| changed.contains(&command.source_script)),
            );
            map.script_runtime_commands
                .retain(|command| !changed.contains(&command.source_script));
            map.script_runtime_commands.extend(
                crate::parse_script_runtime_commands(&map.id, &map.scripts)?
                    .into_iter()
                    .filter(|command| changed.contains(&command.source_script)),
            );
            map.script_object_commands
                .retain(|command| !changed.contains(&command.source_script));
            map.script_object_commands.extend(
                crate::parse_script_object_commands(&map.id, &map.scripts)?
                    .into_iter()
                    .filter(|command| changed.contains(&command.source_script)),
            );
            map.trainer_scripts.extend(
                crate::parse_trainer_scripts(&map.id, &map.scripts)?
                    .into_iter()
                    .filter(|(id, _)| changed.contains(id)),
            );
            map.scripted_trainer_battles
                .retain(|battle| !changed.contains(&battle.source_script));
            map.scripted_trainer_battles.extend(
                crate::parse_scripted_trainer_battles(&map.id, &map.scripts)?
                    .into_iter()
                    .filter(|battle| changed.contains(&battle.source_script)),
            );
            authored_town_publication::replace(&mut pack, map)?;
        }
    }
    ensure!(
        count > 0,
        "no matching ordinary Minnesota trainer challenges found"
    );
    pack.report
        .manifests
        .push("minnesota-ordinary-challenges-v1".into());
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

// Only the exact generated challenge is replaced: unrelated yes/no scripts,
// including story gates and optional services, must retain their behavior.
fn native_trainer_challenge(
    id: &str,
    class: &str,
    script: &serde_json::Value,
) -> Option<serde_json::Value> {
    let flag = script.get(1)?.get("args")?.get(0)?.as_str()?;
    let seen = format!("{id}Seen");
    let won = format!("{id}Won");
    let lost = format!("{id}Lost");
    let after = format!("{id}After");
    let declined = format!("{id}Declined");
    let c = authoring::command;
    let expected = json!([
        c("faceplayer", &[]),
        c("checkevent", &[flag]),
        c("iftrue", &[&after]),
        c("opentext", &[]),
        c("writetext", &[&seen]),
        c("yesorno", &[]),
        c("iffalse", &[&declined]),
        c("closetext", &[]),
        c("winlosstext", &[&won, &lost]),
        c("loadtrainer", &[class, id]),
        c("startbattle", &[]),
        c("reloadmapafterbattle", &[]),
        c("setevent", &[flag]),
        c("end", &[])
    ]);
    if *script != expected {
        return None;
    }
    Some(json!([c(
        "trainer",
        &[class, id, flag, &seen, &won, &lost, &after]
    )]))
}

#[cfg(test)]
mod tests {
    #[test]
    fn challenge_conversion_requires_the_complete_authored_pattern() {
        let id = "MN_EXTRA_R10_SURVEY";
        let c = crate::grand_marais_modpack::authoring::command;
        let program = serde_json::json!([
            c("faceplayer", &[]),
            c("checkevent", &["EVENT_SURVEY_DEFEATED"]),
            c("iftrue", &["MN_EXTRA_R10_SURVEYAfter"]),
            c("opentext", &[]),
            c("writetext", &["MN_EXTRA_R10_SURVEYSeen"]),
            c("yesorno", &[]),
            c("iffalse", &["MN_EXTRA_R10_SURVEYDeclined"]),
            c("closetext", &[]),
            c(
                "winlosstext",
                &["MN_EXTRA_R10_SURVEYWon", "MN_EXTRA_R10_SURVEYLost"]
            ),
            c("loadtrainer", &["HIKER", id]),
            c("startbattle", &[]),
            c("reloadmapafterbattle", &[]),
            c("setevent", &["EVENT_SURVEY_DEFEATED"]),
            c("end", &[])
        ]);
        let replacement =
            super::native_trainer_challenge(id, "HIKER", &program).expect("ordinary challenge");
        assert_eq!(
            replacement,
            serde_json::json!([c(
                "trainer",
                &[
                    "HIKER",
                    id,
                    "EVENT_SURVEY_DEFEATED",
                    "MN_EXTRA_R10_SURVEYSeen",
                    "MN_EXTRA_R10_SURVEYWon",
                    "MN_EXTRA_R10_SURVEYLost",
                    "MN_EXTRA_R10_SURVEYAfter"
                ]
            )])
        );
        assert!(super::native_trainer_challenge(id, "FISHER", &program).is_none());
        let mut story_gate = program.clone();
        story_gate
            .as_array_mut()
            .expect("script commands")
            .insert(1, c("checkevent", &["STORY_GATE"]));
        assert!(super::native_trainer_challenge(id, "HIKER", &story_gate).is_none());
    }

    #[test]
    #[ignore = "requires CRYSTAL_MINNESOTA_CYCLIST_PACK with the installed cyclists"]
    fn installed_cyclists_use_native_trainer_scripts_without_changing_parties_or_audio() {
        let path = std::env::var_os("CRYSTAL_MINNESOTA_CYCLIST_PACK").expect("external pack");
        let base = crate::read_verified_compiled_game_pack(path).expect("verified parent");
        let revised = super::revise_minnesota_ordinary_challenges(&base).expect("revision");
        assert_eq!(base.compiled_audio(), revised.compiled_audio());
        assert_eq!(base.runtime_files(), revised.runtime_files());
        assert_eq!(base.data().trainers, revised.data().trainers);
        for (name, original_map) in &base.data().maps {
            let revised_map = &revised.data().maps[name];
            let mut matching = std::collections::BTreeSet::new();
            if name.starts_with("Mn") {
                for object in &original_map.objects {
                    let Some(id) = object.object_identifier.as_deref() else {
                        continue;
                    };
                    if !id.starts_with("MN_EXTRA_")
                        || object.object_type != "OBJECTTYPE_SCRIPT"
                        || object.radius != 0
                        || object.script != id
                    {
                        continue;
                    }
                    let (Some(trainer), Some(program)) =
                        (base.data().trainers.get(id), original_map.scripts.get(id))
                    else {
                        continue;
                    };
                    let Some(expected) =
                        super::native_trainer_challenge(id, &trainer.trainer_class, program)
                    else {
                        continue;
                    };
                    matching.insert(id.to_owned());
                    assert_eq!(revised_map.scripts[id], expected, "converted {name}/{id}");
                    assert!(revised_map.trainer_scripts.contains_key(id));
                    let revised_object = revised_map
                        .objects
                        .iter()
                        .find(|entry| entry.object_identifier.as_deref() == Some(id))
                        .expect("revised actor");
                    let mut expected_object = object.clone();
                    expected_object.object_type = "OBJECTTYPE_TRAINER".into();
                    expected_object.radius = 3;
                    assert_eq!(
                        *revised_object, expected_object,
                        "actor preservation {name}/{id}"
                    );
                }
            }
            if matching.is_empty() {
                assert_eq!(original_map, revised_map, "untouched map {name}");
            } else {
                assert_eq!(original_map.blocks, revised_map.blocks, "terrain {name}");
                assert_eq!(
                    original_map.events, revised_map.events,
                    "warps and events {name}"
                );
                let mut expected_rows = original_map.map_event_section_commands.clone();
                for row in &mut expected_rows {
                    if row.command == "object_event"
                        && row.args.len() == 13
                        && matching.contains(&row.args[11])
                    {
                        row.args[9] = "OBJECTTYPE_TRAINER".into();
                        row.args[10] = "3".into();
                    }
                }
                assert_eq!(
                    expected_rows, revised_map.map_event_section_commands,
                    "event wire preservation {name}"
                );
                for (label, program) in &original_map.scripts {
                    if !original_map.objects.iter().any(|object| {
                        object.object_identifier.as_deref() == Some(label.as_str())
                            && revised_map.scripts.get(label) != Some(program)
                    }) {
                        assert_eq!(
                            revised_map.scripts.get(label),
                            Some(program),
                            "unrelated script {name}/{label}"
                        );
                    }
                }
            }
        }
        let map = &revised.data().maps["MnMinneapolis"];
        let cyclists: Vec<_> = map
            .objects
            .iter()
            .filter(|object| {
                object
                    .object_identifier
                    .as_deref()
                    .is_some_and(|id| id.starts_with("MN_EXTRA_MPLS_CYCLE_"))
            })
            .collect();
        assert_eq!(cyclists.len(), 18);
        for object in cyclists {
            let id = object
                .object_identifier
                .as_deref()
                .expect("cyclist identity");
            assert_eq!(object.object_type, "OBJECTTYPE_TRAINER");
            assert!(object.radius > 0);
            assert!(map.trainer_scripts.contains_key(id));
            assert_eq!(map.scripts[id][0]["command"], "trainer");
            assert_eq!(map.scripts[id].as_array().expect("commands").len(), 1);
            let original = base.data().maps["MnMinneapolis"]
                .objects
                .iter()
                .find(|old| old.object_identifier.as_deref() == Some(id))
                .expect("original actor");
            let mut expected = original.clone();
            expected.object_type = object.object_type.clone();
            expected.radius = object.radius;
            assert_eq!(&expected, object, "patrol and actor identity survive");
        }
    }
}
