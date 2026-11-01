//! Real external-pack preservation and rejection of story/receiver shortcuts.
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{CompiledGamePack, build_duluth_story_modpack};
use serde_json::{Value, json};

const ROOM: &str = "MnDuluthHarborWorkshop";
const LABELS: [&str; 3] = [
    "MN_HARBOR_RECEIVERText",
    "MN_HARBOR_RECEIVERDeniedText",
    "MN_HARBOR_RECEIVERDoneText",
];

fn inputs() -> &'static (CompiledGamePack, String) {
    static INPUTS: OnceLock<(CompiledGamePack, String)> = OnceLock::new();
    INPUTS.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository root");
        let path = std::env::var_os("CRYSTAL_DULUTH_STORY_BASE_PACK")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                root.join("content-packs/minnesota-northstar-midgame-cities.crystalpack")
            });
        (
            crystal_assets::read_verified_compiled_game_pack(path)
                .expect("external story/city input"),
            std::fs::read_to_string(root.join("planning/minnesota/story.json"))
                .expect("canonical story"),
        )
    })
}

#[test]
#[ignore = "requires completed external midgame/city pack"]
fn duluth_story_changes_only_three_canonical_text_bodies_and_retains_acknowledgment_gate() {
    let (base, source) = inputs();
    let pack = build_duluth_story_modpack(base, source).expect("native MS10");
    let story: Value = serde_json::from_str(source).expect("story");
    let texts = &story["milestones"]
        .as_array()
        .expect("milestones")
        .iter()
        .find(|m| m["id"] == "MS10")
        .expect("MS10")["native_texts"];
    let old = &base.data().maps[ROOM];
    let room = &pack.data().maps[ROOM];
    assert_eq!(
        pack.data()
            .assemble_map_module_from_compiled_payloads(ROOM)
            .expect("split mirrors"),
        *room
    );
    for label in LABELS {
        let expected: Vec<String> =
            serde_json::from_value(texts[label].clone()).expect("source lines");
        let commands = room.scripts[label].as_array().expect("text commands");
        let actual = commands
            .iter()
            .filter(|c| c["command"] != "done")
            .map(|c| {
                c["args"][0]
                    .as_str()
                    .expect("quoted source line")
                    .trim_matches('"')
                    .to_owned()
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }
    // Normalize only the three authorized text scripts and their parsed views.
    // Every object, command, section, tile, warp and other text must stay exact.
    let mut restored_room = serde_json::to_value(room).expect("new map");
    let original_room = serde_json::to_value(old).expect("old map");
    for field in ["scripts", "script_text_bodies"] {
        for label in LABELS {
            restored_room[field][label] = original_room[field][label].clone();
        }
    }
    assert_eq!(restored_room, original_room);
    let mut restored = pack.data().clone();
    restored.maps.insert(ROOM.into(), old.clone());
    assert_eq!(restored.map_scripts.len(), base.data().map_scripts.len());
    for (label, value) in &base.data().map_scripts {
        if LABELS.contains(&label.as_str()) {
            restored.map_scripts.insert(label.clone(), value.clone());
        } else {
            assert_eq!(
                restored.map_scripts.get(label),
                Some(value),
                "unrelated script {label}"
            );
        }
    }
    assert!(restored == *base.data(), "a catalog outside MS10 changed");
    assert_eq!(
        pack.audio_manifest().expect("new audio manifest"),
        base.audio_manifest().expect("old audio manifest")
    );
    assert_eq!(pack.compiled_audio(), base.compiled_audio());
    assert_eq!(pack.runtime_files(), base.runtime_files());
    assert!(
        build_duluth_story_modpack(&pack, source).is_err(),
        "duplicate install"
    );
}

#[test]
#[ignore = "requires completed external midgame/city pack"]
fn duluth_story_rejects_premature_signal_wrong_custody_and_malformed_text() {
    let (base, source) = inputs();
    for change in [
        "win_only",
        "missing_badge",
        "extra_result",
        "wide_text",
        "odd_page",
        "duplicate",
    ] {
        let mut story: Value = serde_json::from_str(source).expect("story");
        let milestones = story["milestones"].as_array_mut().expect("milestones");
        let index = milestones
            .iter()
            .position(|m| m["id"] == "MS10")
            .expect("MS10");
        let m = &mut milestones[index];
        match change {
            "win_only" => m["requires"][8] = json!("EVENT_MN_DEPOT_FOREMAN_DEFEATED"),
            "missing_badge" => {
                m["requires"].as_array_mut().expect("requires").remove(7);
            }
            "extra_result" => m["sets"]
                .as_array_mut()
                .expect("results")
                .push(json!("EVENT_MN_SURVEY_RESTORED")),
            "wide_text" => {
                m["native_texts"][LABELS[0]][0] = json!("This line cannot fit the native text box")
            }
            "odd_page" => {
                m["native_texts"][LABELS[0]]
                    .as_array_mut()
                    .expect("lines")
                    .pop();
            }
            "duplicate" => {
                let duplicate = m.clone();
                milestones.push(duplicate);
            }
            _ => unreachable!("fixed cases"),
        }
        assert!(
            build_duluth_story_modpack(base, &story.to_string()).is_err(),
            "accepted {change}"
        );
    }
}
