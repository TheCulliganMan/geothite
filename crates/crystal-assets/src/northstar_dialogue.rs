//! A text-only publication of the reviewed Northstar scenes.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;

const PAGE_SCENES: [(&str, &str, &str); 9] = [
    ("MS_HOME", "MnWinonaHome1F", "MnWHomeIntroText"),
    ("MS00", "MnWinonaFieldStation", "MnWReadingsText"),
    ("MS01", "MnRochesterPokecenter1F", "MnRCDeliveryText"),
    ("MS02", "MnMankato", "MnMKNotebookText"),
    ("MS04", "MnStPaulUnionDepot", "MnSPCourierSeenText"),
    ("MS04_MANIFEST", "MnStPaulUnionDepot", "MnSPManifestText"),
    ("MS05", "MnBrainerd", "MN_BRAINERD_RANGERText"),
    ("MS06", "MnLakeItasca", "MN_ITASCA_MARKERText"),
    ("MS07", "MnBemidji", "MnBemidjiComparisonText"),
];
const NATIVE_SCENES: [(&str, &str, &str, usize); 11] = [
    ("MS03_ICE", "MnMinneapolisDispatchOffice", "MnMPICE", 5),
    ("MS03_RESCUE", "MnMinneapolisDispatchOffice", "MnMPCell", 5),
    ("MS03", "MnMinneapolisDispatchOffice", "MnMPDispatch", 3),
    ("MS04", "MnStPaulUnionDepot", "MnSPCourier", 5),
    ("MS04_MANIFEST", "MnStPaulUnionDepot", "MnSPManifest", 2),
    ("MS08", "MnR09Cave1F", "MnIronRangeForeman", 6),
    ("MS08_CACHE", "MnR09Cave1F", "MnIronRangeCache", 5),
    ("MS09", "MnGrandRapidsSurveyDepot", "MnGrandRapidsDepot", 5),
    ("MS10", "MnDuluthHarborWorkshop", "MN_HARBOR_RECEIVER", 3),
    (
        "MS11",
        "MnBoundaryWatersSurveyShelter",
        "MnBoundarySurveyChief",
        6,
    ),
    (
        "MS12",
        "MnBoundaryWatersSurveyShelter",
        "MnBoundarySurveyReceiver",
        6,
    ),
];
const SUPPORT: [(&str, &str); 7] = [
    ("MnWinonaHome1F", "MnWGuardianRestText"),
    ("MnWinonaFieldStation", "MnWReminderText"),
    ("MnRochesterPokecenter1F", "MnRCDeliveryAfterText"),
    (
        "MnMinneapolisDispatcherHouse",
        "MN_MP_DISPATCHER_PARTNERText",
    ),
    (
        "MnMinneapolisDispatcherHouse",
        "MN_MP_DISPATCHER_PARTNERAfterText",
    ),
    (
        "MnMinneapolisDispatcherHouse",
        "MN_MP_RESCUED_DISPATCHERText",
    ),
    ("MnStPaulUnionDepot", "MN_ST_PAUL_DEPOT_COURIERText"),
];

/// Identifies reviewed chapter text that local flavor overlays must preserve.
pub(crate) fn is_reviewed_story_text(map: &str, label: &str) -> bool {
    PAGE_SCENES
        .iter()
        .any(|(_, room, text)| *room == map && *text == label)
        || SUPPORT
            .iter()
            .any(|(room, text)| *room == map && *text == label)
        || NATIVE_SCENES.iter().any(|(_, room, prefix, _)| {
            *room == map && label.starts_with(*prefix) && label.ends_with("Text")
        })
        || (label.starts_with("MnLeagueChampion") && label.ends_with("Text"))
        || label == "MnElyRecordText"
}

/// Refreshes the installed Northstar dialogue from its canonical story source.
///
/// Rewrites only existing text programs in the listed story scenes. Native
/// page boundaries and both serialized script views are rebuilt together.
/// Progression, battles, geometry, art, items, audio and runtime files are
/// retained. Reapplying the same source is idempotent. Use this final stage
/// after publishing the relay and supplies when rebuilding an older pack.
///
/// # Errors
/// Returns an error for missing/duplicate milestones, moved scenes, unexpected
/// text labels, invalid native lines, an absent installed text, or invalid pack.
/// The borrowed pack is never modified.
pub fn refresh_northstar_dialogue(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let texts = source_texts(source)?;
    let mut pack = base.clone();
    for (map_name, replacements) in texts {
        let mut map = base
            .data
            .maps
            .get(&map_name)
            .context("installed story room")?
            .clone();
        for (label, lines) in replacements {
            ensure!(
                map.script_text_bodies.contains_key(&label),
                "missing installed story text {map_name}/{label}"
            );
            let old = map
                .scripts
                .get(&label)
                .and_then(Value::as_array)
                .context("existing native text program")?;
            ensure!(
                old.iter().all(|c| matches!(
                    c["command"].as_str(),
                    Some("text" | "line" | "para" | "cont" | "done" | "prompt")
                )),
                "story replacement must target text only: {label}"
            );
            authoring::text(
                &mut map,
                &label,
                &lines.iter().map(String::as_str).collect::<Vec<_>>(),
            );
            pack.data
                .map_scripts
                .insert(label.clone(), map.scripts[&label].clone());
        }
        // Only text caches can change. Do not re-author event sections or
        // normalize unrelated executable programs in these already-built maps.
        map.script_text_commands = crate::parse_script_text_commands(&map_name, &map.scripts)?;
        map.script_text_bodies = crate::parse_script_text_bodies(&map_name, &map.scripts)?;
        pack.data.maps.insert(map_name, map);
    }
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

type Texts = BTreeMap<String, BTreeMap<String, Vec<String>>>;

fn milestone<'a>(source: &'a Value, id: &str, map: &str) -> Result<&'a Value> {
    let found = source["milestones"]
        .as_array()
        .context("story milestones")?
        .iter()
        .filter(|m| m["id"] == id)
        .collect::<Vec<_>>();
    ensure!(found.len() == 1, "one milestone required: {id}");
    let logical_map = if id == "MS01" { "MnRochester" } else { map };
    ensure!(
        found[0]["location"] == logical_map,
        "moved story scene: {id}"
    );
    Ok(found[0])
}

fn add(texts: &mut Texts, map: &str, label: &str, lines: Vec<String>) -> Result<()> {
    ensure!(
        !lines.is_empty()
            && lines.len() <= 24
            && lines.iter().all(|l| !l.is_empty()
                && l.len() <= 18
                && l.is_ascii()
                && !l.chars().any(|c| c.is_control() || c == '"')),
        "invalid native story lines: {label}"
    );
    ensure!(
        texts
            .entry(map.into())
            .or_default()
            .insert(label.into(), lines)
            .is_none(),
        "duplicate story text: {label}"
    );
    Ok(())
}

fn source_texts(source: &str) -> Result<Texts> {
    let source: Value = serde_json::from_str(source).context("canonical Northstar story")?;
    let mut texts = Texts::new();
    for (id, map, label) in PAGE_SCENES {
        let m = milestone(&source, id, map)?;
        let pages: Vec<Vec<String>> = serde_json::from_value(m["dialogue_pages"].clone())?;
        ensure!(
            !pages.is_empty() && pages.iter().all(|p| p.len() == 2),
            "complete native pages required: {id}"
        );
        add(
            &mut texts,
            map,
            label,
            pages.into_iter().flatten().collect(),
        )?;
    }
    for (id, map, prefix, count) in NATIVE_SCENES {
        let m = milestone(&source, id, map)?;
        let native: BTreeMap<String, Vec<String>> =
            serde_json::from_value(m["native_texts"].clone())?;
        ensure!(
            native.len() == count
                && native
                    .keys()
                    .all(|k| k.starts_with(prefix) && k.ends_with("Text")),
            "unexpected scene texts: {id}"
        );
        for (label, lines) in native {
            add(&mut texts, map, &label, lines)?;
        }
    }
    let support: Texts = serde_json::from_value(source["supporting_dialogue"].clone())?;
    let mut expected = SUPPORT
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    for (map, labels) in support {
        for (label, lines) in labels {
            let before = expected.len();
            expected.retain(|(m, l)| *m != map || *l != label);
            ensure!(expected.len() + 1 == before, "unexpected support text");
            add(&mut texts, &map, &label, lines)?;
        }
    }
    ensure!(expected.is_empty(), "missing supporting dialogue");
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
        .expect("canonical story source")
    }

    #[test]
    fn local_flavor_protection_covers_every_reviewed_story_text() {
        let source = source();
        for (map, labels) in source_texts(&source).expect("canonical story") {
            for label in labels.keys() {
                assert!(is_reviewed_story_text(&map, label), "{map}/{label}");
            }
        }
        let story: Value = serde_json::from_str(&source).expect("canonical milestones");
        for milestone in story["milestones"].as_array().expect("milestones") {
            if let Some(labels) = milestone["native_texts"].as_object() {
                let map = milestone["location"].as_str().expect("scene location");
                for label in labels.keys() {
                    assert!(is_reviewed_story_text(map, label), "{map}/{label}");
                }
            }
        }
        assert!(!is_reviewed_story_text(
            "MnDuluthHarborWorkshop",
            "MnDuluthHarborWorkshop_MN_DULUTH_WORKSHOP_KEEPERText"
        ));
    }

    #[test]
    fn story_revision_rejects_moved_missing_duplicate_or_unrenderable_scenes() {
        let source = source();
        let canonical: Value = serde_json::from_str(&source).expect("canonical story");
        source_texts(&source).expect("complete renderable dialogue");
        for mutation in 0..5 {
            let mut wrong = canonical.clone();
            match mutation {
                0 => wrong["milestones"][0]["location"] = "WrongRoom".into(),
                1 => wrong["milestones"]
                    .as_array_mut()
                    .expect("milestones")
                    .push(canonical["milestones"][0].clone()),
                2 => {
                    wrong["supporting_dialogue"]["MnWinonaHome1F"]["MnWGuardianRestText"] =
                        serde_json::json!(["a line too long for the native window"])
                }
                3 => {
                    wrong["supporting_dialogue"]["MnWinonaHome1F"]
                        .as_object_mut()
                        .expect("home")
                        .remove("MnWGuardianRestText");
                }
                _ => {
                    wrong["supporting_dialogue"]["MnWinonaHome1F"]["NotAStoryText"] =
                        serde_json::json!(["Unexpected"])
                }
            }
            assert!(
                source_texts(&wrong.to_string()).is_err(),
                "reject mutation {mutation}"
            );
        }
    }

    #[test]
    #[ignore = "requires the ignored completed Northstar supplies pack"]
    fn story_revision_preserves_all_executable_content_and_audio_and_is_idempotent() {
        let source = source();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository root");
        let base = crate::read_verified_compiled_game_pack(
            std::env::var_os("CRYSTAL_STORY_TEXT_BASE")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| {
                    root.join("content-packs/minnesota-northstar-supplies.crystalpack")
                }),
        )
        .expect("actual completed supplies pack");
        let result = refresh_northstar_dialogue(&base, &source).expect("reviewed dialogue");
        assert!(base.compiled_audio() == result.compiled_audio());
        assert!(base.runtime_files() == result.runtime_files());
        let mut expected = serde_json::to_value(base.data()).expect("complete source dataset");
        let actual = serde_json::to_value(result.data()).expect("complete resulting dataset");
        for (map, labels) in source_texts(&source).expect("approved changes") {
            for label in labels.keys() {
                expected["maps"][&map]["scripts"][label] =
                    actual["maps"][&map]["scripts"][label].clone();
                expected["map_scripts"][label] = actual["map_scripts"][label].clone();
                assert_eq!(
                    actual["maps"][&map]["scripts"][label], actual["map_scripts"][label],
                    "native split mirror {label}"
                );
            }
            for field in ["script_text_commands", "script_text_bodies"] {
                expected["maps"][&map][field] = actual["maps"][&map][field].clone();
            }
            // Reparse from the raw native text rather than trusting the caches.
            let room = &result.data().maps[&map];
            assert_eq!(
                room.script_text_commands,
                crate::parse_script_text_commands(&map, &room.scripts).expect("native commands")
            );
            assert_eq!(
                room.script_text_bodies,
                crate::parse_script_text_bodies(&map, &room.scripts).expect("native bodies")
            );
        }
        for (field, value) in expected.as_object().expect("dataset fields") {
            assert!(
                actual.get(field) == Some(value),
                "unexpected content change: {field}"
            );
        }
        assert_eq!(
            result.identity().expect("revised identity"),
            refresh_northstar_dialogue(&result, &source)
                .expect("idempotent repeat")
                .identity()
                .expect("repeat identity")
        );
        // The reviewed dialogue artifact is an immutable input to later
        // chapters. Current authoring text may legitimately differ from it.
        // Verify a fresh round trip unless a caller names an expected artifact.
        let explicit = std::env::var_os("CRYSTAL_STORY_TEXT_PACK").map(std::path::PathBuf::from);
        let output = explicit.clone().unwrap_or_else(|| {
            std::env::temp_dir().join(format!(
                "northstar-dialogue-{}-{}.crystalpack",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("clock")
                    .as_nanos(),
            ))
        });
        if explicit.is_none() {
            result
                .write_preserving_storage(&output)
                .expect("fresh dialogue publication");
        }
        let written = crate::read_verified_compiled_game_pack(&output);
        if explicit.is_none() {
            std::fs::remove_file(output).expect("remove temporary dialogue round trip");
        }
        let written = written.expect("published dialogue pack");
        assert_eq!(
            written.identity().expect("written identity"),
            result.identity().expect("rebuilt identity")
        );
    }
}
