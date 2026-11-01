//! Update the installed story without changing travel, battle or other content.
use std::collections::BTreeSet;
use std::path::PathBuf;

use crystal_assets::{
    build_duluth_story_modpack, read_verified_compiled_game_pack,
    refresh_boundary_basecamp_story_dialogue, refresh_duluth_story_dialogue,
};
use serde_json::Value;

#[test]
#[ignore = "requires the completed external Northstar city/Northwoods pack"]
fn northstar_refresh_changes_only_canonical_reveal_and_finale_text_and_is_idempotent() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let path = std::env::var_os("CRYSTAL_NORTHSTAR_REFRESH_BASE_PACK")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            root.join("content-packs/minnesota-northstar-duluth-expanded.crystalpack")
        });
    let base = read_verified_compiled_game_pack(path).expect("installed native chapters");
    let source = std::fs::read_to_string(root.join("planning/minnesota/story.json"))
        .expect("canonical story");
    let story: Value = serde_json::from_str(&source).expect("story");
    let pack = refresh_duluth_story_dialogue(&base, &source).expect("refresh MS10");
    let pack = refresh_boundary_basecamp_story_dialogue(&pack).expect("refresh Q07");
    let mut restored = pack.data().clone();
    let mut labels = BTreeSet::new();
    for (name, ids) in [
        ("MnDuluthHarborWorkshop", vec!["MS10"]),
        ("MnBoundaryWatersSurveyShelter", vec!["MS11", "MS12"]),
    ] {
        let old = &base.data().maps[name];
        let new = &pack.data().maps[name];
        let mut normalized = new.clone();
        for milestone in story["milestones"]
            .as_array()
            .expect("milestones")
            .iter()
            .filter(|m| ids.contains(&m["id"].as_str().expect("milestone id")))
        {
            for (label, expected) in milestone["native_texts"]
                .as_object()
                .expect("canonical native texts")
            {
                let lines = new.scripts[label]
                    .as_array()
                    .expect("lowered text")
                    .iter()
                    .filter(|c| matches!(c["command"].as_str(), Some("text" | "line" | "para")))
                    .map(|c| {
                        c["args"][0]
                            .as_str()
                            .expect("native line")
                            .trim_matches('"')
                    })
                    .collect::<Vec<_>>();
                assert_eq!(serde_json::json!(lines), *expected, "{label}");
                normalized
                    .scripts
                    .insert(label.clone(), old.scripts[label].clone());
                normalized
                    .script_text_bodies
                    .insert(label.clone(), old.script_text_bodies[label].clone());
                labels.insert(label.clone());
            }
        }
        assert_eq!(new.script_text_commands, old.script_text_commands);
        assert!(normalized == *old, "non-text map content changed: {name}");
        restored.maps.insert(name.into(), old.clone());
        assert_eq!(
            pack.data()
                .assemble_map_module_from_compiled_payloads(name)
                .expect("split text consistency"),
            *new
        );
    }
    assert_eq!(restored.map_scripts.len(), base.data().map_scripts.len());
    for label in labels {
        if let Some(value) = base.data().map_scripts.get(&label) {
            restored.map_scripts.insert(label, value.clone());
        }
    }
    assert!(
        restored == *base.data(),
        "content outside authorized dialogue changed"
    );
    assert_eq!(
        pack.audio_manifest().expect("new audio"),
        base.audio_manifest().expect("old audio")
    );
    assert_eq!(pack.compiled_audio(), base.compiled_audio());
    assert_eq!(pack.runtime_files(), base.runtime_files());
    let repeated = refresh_duluth_story_dialogue(&pack, &source).expect("repeat MS10");
    let repeated = refresh_boundary_basecamp_story_dialogue(&repeated).expect("repeat Q07");
    assert!(repeated.data() == pack.data(), "refresh must be idempotent");
    assert!(
        build_duluth_story_modpack(&pack, &source).is_err(),
        "fresh installation still rejects duplicates"
    );
}
