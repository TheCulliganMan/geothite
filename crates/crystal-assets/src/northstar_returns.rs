//! Returning home and to allies after the acknowledged survey shutdown.
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::{CompiledGamePack, MapModule};

const PARENT: &str = "1c66ba1d8c4d06daa960a036691f03681f030e18c40f740e4a4de9a1a402fd54";
const MANIFEST: &str = "minnesota-northstar-returns-v1";
const RESTORED: &str = "EVENT_MN_SURVEY_RESTORED";
const DELIVERED: &str = "EVENT_MN_DELIVERED_ROCHESTER_PARCEL";

struct Branch {
    event: &'static str,
    script: &'static str,
    text: &'static str,
}

struct Scene {
    map: &'static str,
    root: &'static str,
    insertion: usize,
    original: Vec<Value>,
    branches: Vec<Branch>,
}

fn scenes() -> Vec<Scene> {
    vec![
        Scene {
            map: "MnWinonaHome1F",
            root: "MN_W_GUARDIAN",
            insertion: 2,
            original: vec![
                c("checkevent", &["EVENT_MN_HOME_INTRO"]),
                c("iffalse", &["MnWHomeArrival"]),
                c("faceplayer", &[]),
                c("opentext", &[]),
                c("writetext", &["MnWGuardianRestText"]),
                c("waitbutton", &[]),
                c("special", &["HealParty"]),
                c("blackoutmod", &["MN_WINONA_HOME_1F"]),
                c("closetext", &[]),
                c("end", &[]),
            ],
            branches: vec![Branch {
                event: RESTORED,
                script: "MnWGuardianSurveyReturn",
                text: "MnWGuardianSurveyReturnText",
            }],
        },
        Scene {
            map: "MnWinonaFieldStation",
            root: "MnWReminder",
            insertion: 0,
            original: vec![
                c("opentext", &[]),
                c("writetext", &["MnWReminderText"]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("scall", &["MnWSupplies"]),
                c("end", &[]),
            ],
            branches: vec![
                Branch {
                    event: RESTORED,
                    script: "MnWTamarackSurveyReturn",
                    text: "MnWTamarackSurveyReturnText",
                },
                Branch {
                    event: DELIVERED,
                    script: "MnWTamarackDeliveryReturn",
                    text: "MnWTamarackDeliveryReturnText",
                },
            ],
        },
        Scene {
            map: "MnMinneapolisDispatcherHouse",
            root: "MN_MP_DISPATCHER_PARTNERAfter",
            insertion: 0,
            original: vec![c(
                "jumptextfaceplayer",
                &["MN_MP_DISPATCHER_PARTNERAfterText"],
            )],
            branches: vec![Branch {
                event: RESTORED,
                script: "MnMPJoSurveyReturn",
                text: "MnMPJoSurveyReturnText",
            }],
        },
        Scene {
            map: "MnMinneapolisDispatcherHouse",
            root: "MN_MP_RESCUED_DISPATCHER",
            insertion: 0,
            original: vec![c("jumptextfaceplayer", &["MN_MP_RESCUED_DISPATCHERText"])],
            branches: vec![Branch {
                event: RESTORED,
                script: "MnMPEllisSurveyReturn",
                text: "MnMPEllisSurveyReturnText",
            }],
        },
        Scene {
            map: "MnDuluthHarborWorkshop",
            root: "MN_HARBOR_RECEIVERDone",
            insertion: 0,
            original: vec![c("jumptextfaceplayer", &["MN_HARBOR_RECEIVERDoneText"])],
            branches: vec![Branch {
                event: RESTORED,
                script: "MnHarborSurveyReturn",
                text: "MnHarborSurveyReturnText",
            }],
        },
    ]
}

/// Adds optional return conversations to the reviewed Northstar dialogue pack.
///
/// Existing quest flags select the conversations. No new progress or reward is
/// granted. The guardian retains healing and the home recovery location;
/// Tamarack retains the original once-only/retryable supplies call. The home
/// intro, starter, parcel, rescue presence and harbor access guards are retained.
/// Source text comes from `return_dialogue` in the canonical story JSON.
///
/// # Errors
/// Returns an error for a different parent, duplicate installation, changed
/// original programs, occupied new labels, invalid text, missing existing event
/// identifiers, or invalid resulting native data. The borrowed input is retained.
pub fn build_northstar_returns(base: &CompiledGamePack, source: &str) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity()?.content_hash == PARENT
            || crate::northstar_current_story::is_composing(base),
        "reviewed Northstar dialogue parent required"
    );
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "return conversations already installed"
    );
    let texts = texts(source)?;
    for event in [RESTORED, DELIVERED] {
        ensure!(
            base.data
                .story_event_script_constants
                .global
                .contains_key(event),
            "missing existing return event: {event}"
        );
    }
    let mut pack = base.clone();
    let mut maps = BTreeMap::<String, MapModule>::new();
    let mut changed = BTreeMap::<String, BTreeSet<String>>::new();
    for scene in scenes() {
        let room = match maps.entry(scene.map.to_owned()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(
                base.data
                    .maps
                    .get(scene.map)
                    .context("return room")?
                    .clone(),
            ),
        };
        ensure!(
            room.scripts.get(scene.root) == Some(&json!(scene.original)),
            "changed original return program: {}",
            scene.root
        );
        let mut checks = Vec::new();
        let labels = changed.entry(scene.map.into()).or_default();
        labels.insert(scene.root.into());
        for branch in scene.branches {
            for label in [branch.script, branch.text] {
                ensure!(
                    !room.scripts.contains_key(label) && !base.data.map_scripts.contains_key(label),
                    "return label occupied: {label}"
                );
                labels.insert(label.into());
            }
            // Clone the existing service tail: its only changed operand is the
            // selected text. Healing, recovery and supply retry stay identical.
            let mut body = scene.original[scene.insertion..].to_vec();
            let mut count = 0;
            for command in &mut body {
                if matches!(
                    command["command"].as_str(),
                    Some("writetext" | "jumptextfaceplayer")
                ) {
                    command["args"][0] = branch.text.into();
                    count += 1;
                }
            }
            ensure!(count == 1, "one existing return text operand required");
            room.scripts.insert(branch.script.into(), json!(body));
            authoring::text(
                room,
                branch.text,
                &texts[branch.text]
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
            );
            checks.extend([
                c("checkevent", &[branch.event]),
                c("iftrue", &[branch.script]),
            ]);
        }
        let mut root = scene.original;
        root.splice(scene.insertion..scene.insertion, checks);
        room.scripts.insert(scene.root.into(), json!(root));
    }
    for (name, mut room) in maps {
        // Rebuild typed indexes, including the cloned recovery commands, while
        // retaining the room's event wire rows and all geometry exactly.
        let event_rows = room.map_event_section_commands.clone();
        crate::ely_league_rooms::finish(base, &mut room)?;
        ensure!(
            room.map_event_section_commands == event_rows,
            "return dialogue must not normalize unrelated event rows"
        );
        for label in &changed[&name] {
            pack.data
                .map_scripts
                .insert(label.clone(), room.scripts[label].clone());
        }
        pack.data.maps.insert(name, room);
    }
    pack.report.manifests.push(MANIFEST.into());
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

fn texts(source: &str) -> Result<BTreeMap<String, Vec<String>>> {
    let source: Value = serde_json::from_str(source)?;
    let texts: BTreeMap<String, Vec<String>> =
        serde_json::from_value(source["return_dialogue"].clone())
            .context("canonical return dialogue")?;
    let expected = scenes()
        .into_iter()
        .flat_map(|s| s.branches.into_iter().map(|b| b.text))
        .collect::<BTreeSet<_>>();
    ensure!(
        texts.keys().map(String::as_str).collect::<BTreeSet<_>>() == expected,
        "exact return conversation labels required"
    );
    for (label, lines) in &texts {
        ensure!(
            !lines.is_empty()
                && lines.len() <= 8
                && lines.len() % 2 == 0
                && lines.iter().all(|l| !l.is_empty()
                    && l.len() <= 18
                    && l.is_ascii()
                    && !l.chars().any(|c| c.is_control() || c == '"')),
            "invalid native return text: {label}"
        );
    }
    Ok(texts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> String {
        std::fs::read_to_string(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../planning/minnesota/story.json"),
        )
        .expect("canonical story")
    }

    #[test]
    fn return_source_rejects_missing_extra_or_unrenderable_conversations() {
        let source: Value = serde_json::from_str(&source()).expect("story JSON");
        texts(&source.to_string()).expect("six native conversations");
        for invalid in 0..4 {
            let mut bad = source.clone();
            let body = bad["return_dialogue"]
                .as_object_mut()
                .expect("return dialogue");
            match invalid {
                0 => {
                    body.remove("MnWGuardianSurveyReturnText");
                }
                1 => {
                    body.insert("UnknownText".into(), json!(["Unrelated", "text"]));
                }
                2 => {
                    body.insert(
                        "MnWGuardianSurveyReturnText".into(),
                        json!(["This native line is too long", "text"]),
                    );
                }
                _ => {
                    body.insert("MnWGuardianSurveyReturnText".into(), json!(["One line"]));
                }
            }
            assert!(
                texts(&bad.to_string()).is_err(),
                "reject return source {invalid}"
            );
        }
    }

    #[test]
    #[ignore = "requires the ignored reviewed dialogue and return packs"]
    fn return_publication_preserves_everything_outside_the_five_guarded_programs() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository root");
        let base = crate::read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-northstar-dialogue.crystalpack"),
        )
        .expect("reviewed dialogue parent");
        let result = build_northstar_returns(&base, &source()).expect("native optional returns");
        assert!(base.compiled_audio() == result.compiled_audio());
        assert!(base.runtime_files() == result.runtime_files());
        let mut expected = serde_json::to_value(base.data()).expect("complete parent dataset");
        let actual = serde_json::to_value(result.data()).expect("complete returned dataset");
        let mut allowed = BTreeMap::<&str, BTreeSet<&str>>::new();
        for scene in scenes() {
            let labels = allowed.entry(scene.map).or_default();
            labels.insert(scene.root);
            for branch in scene.branches {
                labels.extend([branch.script, branch.text]);
            }
        }
        for (map, labels) in allowed {
            for label in &labels {
                expected["maps"][map]["scripts"][label] =
                    actual["maps"][map]["scripts"][label].clone();
                expected["map_scripts"][label] = actual["map_scripts"][label].clone();
                assert_eq!(
                    actual["maps"][map]["scripts"][label], actual["map_scripts"][label],
                    "split return program {label}"
                );
            }
            let mut old_bodies = expected["maps"][map]["script_text_bodies"].clone();
            let mut new_bodies = actual["maps"][map]["script_text_bodies"].clone();
            for label in &labels {
                old_bodies
                    .as_object_mut()
                    .expect("old bodies")
                    .remove(*label);
                new_bodies
                    .as_object_mut()
                    .expect("new bodies")
                    .remove(*label);
            }
            assert_eq!(
                old_bodies, new_bodies,
                "unrelated return-room text retained"
            );
            expected["maps"][map]["script_text_bodies"] =
                actual["maps"][map]["script_text_bodies"].clone();
            for cache in [
                "script_flag_commands",
                "script_control_commands",
                "script_text_commands",
                "script_object_commands",
                "script_map_commands",
                "script_runtime_commands",
            ] {
                let unrelated = |value: &Value| {
                    value
                        .as_array()
                        .expect("typed native commands")
                        .iter()
                        .filter(|row| {
                            !labels.contains(row["source_script"].as_str().expect("source script"))
                        })
                        .cloned()
                        .collect::<Vec<_>>()
                };
                assert_eq!(
                    unrelated(&expected["maps"][map][cache]),
                    unrelated(&actual["maps"][map][cache]),
                    "unrelated native indexes: {map}/{cache}"
                );
                expected["maps"][map][cache] = actual["maps"][map][cache].clone();
            }
        }
        for (field, value) in expected.as_object().expect("all dataset fields") {
            assert!(
                actual.get(field) == Some(value),
                "unexpected return data change: {field}"
            );
        }
        // The only new actions are conditional text selection. Service tails
        // retain their original operations, including supply retries and care.
        for scene in scenes() {
            let room = &result.data().maps[scene.map];
            for branch in scene.branches {
                let mut body = room.scripts[branch.script]
                    .as_array()
                    .expect("new native branch")
                    .clone();
                let original = &scene.original[scene.insertion..];
                for (command, before) in body.iter_mut().zip(original) {
                    if matches!(
                        command["command"].as_str(),
                        Some("writetext" | "jumptextfaceplayer")
                    ) {
                        command["args"] = before["args"].clone();
                    }
                }
                assert_eq!(body, original, "only dialogue changes in the service tail");
            }
        }
        let written = crate::read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-northstar-returns.crystalpack"),
        )
        .expect("published returns");
        assert_eq!(
            result.identity().expect("built identity"),
            written.identity().expect("written identity")
        );
        assert!(
            build_northstar_returns(&result, &source()).is_err(),
            "no duplicate publication"
        );
    }
}
