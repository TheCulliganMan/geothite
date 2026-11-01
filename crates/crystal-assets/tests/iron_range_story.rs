//! Real installed mine, additive story publication and invalid-source rejection.
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{
    CompiledGamePack, build_iron_range_story_modpack, refresh_boundary_basecamp_story_dialogue,
    refresh_grand_marais_story_dialogue,
};
use serde_json::{Value, json};

const MINE: &str = "MnR09Cave1F";

fn inputs() -> &'static (CompiledGamePack, String, String) {
    static INPUTS: OnceLock<(CompiledGamePack, String, String)> = OnceLock::new();
    INPUTS.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository root");
        let path = std::env::var_os("CRYSTAL_IRON_RANGE_STORY_BASE_PACK")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("content-packs/minnesota-geography-review.crystalpack"));
        (
            crystal_assets::read_verified_compiled_game_pack(path)
                .expect("actual installed mine pack"),
            std::fs::read_to_string(root.join("planning/minnesota/routes/MN_R09.json"))
                .expect("authored mine source"),
            std::fs::read_to_string(root.join("planning/minnesota/story.json"))
                .expect("authored story source"),
        )
    })
}

#[test]
#[ignore = "requires the completed external Iron Range source pack"]
fn iron_range_story_preserves_content_and_lowers_exact_evidence_chain() {
    let (base, dossier, story) = inputs();
    let pack = build_iron_range_story_modpack(base, dossier, story).expect("native Q06 overlay");
    let mut preserved = pack.data().clone();
    preserved
        .maps
        .insert(MINE.into(), base.data().maps[MINE].clone());
    preserved.trainers = base.data().trainers.clone();
    preserved.story_event_script_constants = base.data().story_event_script_constants.clone();
    assert_eq!(preserved, *base.data(), "changed unrelated game data");
    assert_eq!(pack.runtime_files(), base.runtime_files());
    assert_eq!(pack.compiled_audio(), base.compiled_audio());
    assert_eq!(
        pack.audio_manifest().expect("new audio"),
        base.audio_manifest().expect("source audio")
    );
    for (id, trainer) in &base.data().trainers.trainers {
        assert_eq!(pack.data().trainers.trainers.get(id), Some(trainer));
    }
    for (id, value) in &base.data().story_event_script_constants.global {
        assert_eq!(
            pack.data().story_event_script_constants.global.get(id),
            Some(value)
        );
    }
    let old = &base.data().maps[MINE];
    let room = &pack.data().maps[MINE];
    assert_eq!(room.attributes, old.attributes);
    assert_eq!(room.blocks, old.blocks);
    assert_eq!(room.events, old.events);
    assert_eq!(room.scenes, old.scenes);
    assert_eq!(&room.objects[..old.objects.len()], old.objects);
    assert_eq!(room.objects.len(), old.objects.len() + 2);
    for (label, body) in &old.scripts {
        assert_eq!(room.scripts.get(label), Some(body));
    }
    assert_eq!(
        pack.data()
            .assemble_map_module_from_compiled_payloads(MINE)
            .expect("native payload reconstruction"),
        *room
    );
    let trainer = &pack.data().trainers.trainers["MN_LODE_FOREMAN"];
    assert_eq!(
        trainer
            .party
            .iter()
            .map(|p| (p.species.as_str(), p.level))
            .collect::<Vec<_>>(),
        [("GRAVELER", 35), ("MACHOKE", 35), ("MAGNETON", 36)]
    );
    let battle = &room.scripted_trainer_battles[0];
    assert_eq!(battle.source_script, "MN_R09_DEPOT_FOREMAN");
    assert_eq!(battle.request.trainer_id, "MN_LODE_FOREMAN");
    assert_eq!(battle.request.battle_type, "BATTLETYPE_TRAINER");
    let setters = room
        .script_flag_commands
        .iter()
        .filter(|c| c.command == "setevent")
        .map(|c| {
            (
                c.source_script.as_str(),
                c.command_index,
                c.flag_id.as_str(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        setters,
        [
            (
                "MN_R09_DEPOT_FOREMAN",
                15,
                "EVENT_MN_DEPOT_FOREMAN_DEFEATED"
            ),
            (
                "MN_R09_SURVEY_CACHE",
                11,
                "EVENT_MN_MINE_INSTRUMENT_RECOVERED"
            ),
        ]
    );
    assert!(room.script_item_grants.is_empty());
    assert!(room.script_item_takes.is_empty());
    assert!(room.script_economy_commands.is_empty());
    let story: Value = serde_json::from_str(story).expect("story JSON");
    for milestone in story["milestones"].as_array().expect("milestones") {
        if !matches!(milestone["id"].as_str(), Some("MS08" | "MS08_CACHE")) {
            continue;
        }
        for (label, expected) in milestone["native_texts"].as_object().expect("native texts") {
            let lines = room.scripts[label]
                .as_array()
                .expect("lowered text")
                .iter()
                .filter(|c| matches!(c["command"].as_str(), Some("text" | "line" | "para")))
                .map(|c| {
                    c["args"][0]
                        .as_str()
                        .expect("text argument")
                        .trim_matches('"')
                })
                .collect::<Vec<_>>();
            assert_eq!(json!(lines), *expected, "Q06 text {label}");
            assert!(lines.iter().all(|line| line.chars().count() <= 18));
        }
    }
    assert!(
        build_iron_range_story_modpack(&pack, dossier, &story.to_string()).is_err(),
        "cannot install duplicate story actors"
    );
}

#[test]
#[ignore = "requires the completed external Iron Range source pack"]
fn iron_range_story_rejects_changed_party_blocked_anchors_and_false_payload() {
    let (base, dossier, story) = inputs();
    for (pointer, value) in [
        ("/interiors/0/story_trainers/0/party/0/level", json!(99)),
        ("/interiors/0/story_trainers/0/tile", json!([0, 0])),
        ("/interiors/0/story_trainers/0/sight_range", json!(3)),
        ("/interiors/0/story_objects/0/tile", json!([19, 7])),
        ("/interiors/0/story_objects/0/approach", json!([0, 0])),
        (
            "/interiors/0/story_objects/0/payload_storage",
            json!("invented_bag_item"),
        ),
    ] {
        let mut changed: Value = serde_json::from_str(dossier).expect("dossier");
        *changed.pointer_mut(pointer).expect("source field") = value;
        let error = build_iron_range_story_modpack(base, &changed.to_string(), story)
            .expect_err("invalid story source");
        assert!(
            !error.to_string().contains("missing field"),
            "must reject the real contract, not JSON parsing"
        );
    }
    let mut changed: Value = serde_json::from_str(story).expect("story");
    changed["milestones"]
        .as_array_mut()
        .expect("milestones")
        .iter_mut()
        .find(|m| m["id"] == "MS09")
        .expect("handover")["requires"] = json!(["EVENT_MN_DEPOT_FOREMAN_DEFEATED"]);
    assert!(
        build_iron_range_story_modpack(base, dossier, &changed.to_string()).is_err(),
        "handover cannot skip physical recovery"
    );
    assert_eq!(
        base.data().maps[MINE].objects.len(),
        1,
        "failed overlays must leave the actual input room unchanged"
    );
}

#[test]
#[ignore = "requires the completed external region with its native Q07 shelter"]
fn iron_range_story_finale_refresh_changes_only_text_and_is_idempotent() {
    let (base, _, story) = inputs();
    let pack = refresh_boundary_basecamp_story_dialogue(base)
        .expect("current Q07 dialogue on existing region");
    let shelter = "MnBoundaryWatersSurveyShelter";
    let mut preserved = pack.data().clone();
    preserved
        .maps
        .insert(shelter.into(), base.data().maps[shelter].clone());
    assert_eq!(preserved, *base.data());
    assert_eq!(pack.runtime_files(), base.runtime_files());
    assert_eq!(pack.compiled_audio(), base.compiled_audio());
    let old = &base.data().maps[shelter];
    let room = &pack.data().maps[shelter];
    let mut original_logic = room.clone();
    original_logic.scripts = old.scripts.clone();
    original_logic.script_text_commands = old.script_text_commands.clone();
    original_logic.script_text_bodies = old.script_text_bodies.clone();
    assert_eq!(
        original_logic, *old,
        "dialogue refresh must preserve every non-text table"
    );
    let story: Value = serde_json::from_str(story).expect("canonical story");
    let changed_labels = story["milestones"]
        .as_array()
        .expect("milestones")
        .iter()
        .filter(|m| matches!(m["id"].as_str(), Some("MS11" | "MS12")))
        .flat_map(|m| {
            m["native_texts"]
                .as_object()
                .expect("finale texts")
                .keys()
                .map(String::as_str)
        })
        .collect::<std::collections::BTreeSet<_>>();
    for (label, commands) in &old.scripts {
        if !changed_labels.contains(label.as_str()) {
            assert_eq!(
                room.scripts.get(label),
                Some(commands),
                "changed non-text Q07 script {label}"
            );
        }
    }
    for milestone in story["milestones"].as_array().expect("milestones") {
        if !matches!(milestone["id"].as_str(), Some("MS11" | "MS12")) {
            continue;
        }
        for (label, expected) in milestone["native_texts"].as_object().expect("finale texts") {
            let lines = room.scripts[label]
                .as_array()
                .expect("lowered text")
                .iter()
                .filter(|c| matches!(c["command"].as_str(), Some("text" | "line" | "para")))
                .map(|c| {
                    c["args"][0]
                        .as_str()
                        .expect("text argument")
                        .trim_matches('"')
                })
                .collect::<Vec<_>>();
            assert_eq!(json!(lines), *expected, "current Q07 text {label}");
        }
    }
    let repeated = refresh_boundary_basecamp_story_dialogue(&pack).expect("idempotent refresh");
    assert_eq!(repeated.data(), pack.data());
    assert_eq!(
        repeated.identity().expect("repeated identity"),
        pack.identity().expect("current identity")
    );
}

#[test]
#[ignore = "requires the completed external region with native Grand Marais"]
fn iron_range_story_grand_marais_refresh_preserves_geometry_and_all_local_quest_logic() {
    let (base, _, _) = inputs();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source =
        std::fs::read_to_string(root.join("planning/minnesota/playable/grand-marais.json"))
            .expect("actual town source");
    let pack =
        refresh_grand_marais_story_dialogue(base, &source).expect("current town story hints");
    let source_json: Value = serde_json::from_str(&source).expect("town dialogue JSON");
    let mut preserved = pack.data().clone();
    for (map, label) in [
        (
            "MnGrandMaraisFieldStation",
            "MnGrandMaraisTamarackThanksText",
        ),
        (
            "MnGrandMaraisFisherHouse",
            "MnGrandMaraisFisherHouseFisherText",
        ),
    ] {
        let old = &base.data().maps[map];
        let room = &pack.data().maps[map];
        let mut original_logic = room.clone();
        original_logic.scripts = old.scripts.clone();
        original_logic.script_text_commands = old.script_text_commands.clone();
        original_logic.script_text_bodies = old.script_text_bodies.clone();
        assert_eq!(original_logic, *old);
        for (name, commands) in &old.scripts {
            if name != label {
                assert_eq!(
                    room.scripts.get(name),
                    Some(commands),
                    "changed local quest script {name}"
                );
            }
        }
        let lines = room.scripts[label]
            .as_array()
            .expect("native text")
            .iter()
            .filter(|c| matches!(c["command"].as_str(), Some("text" | "line" | "para")))
            .map(|c| {
                c["args"][0]
                    .as_str()
                    .expect("text argument")
                    .trim_matches('"')
            })
            .collect::<Vec<_>>();
        assert_eq!(json!(lines), source_json["dialogue"][map][label]);
        preserved.maps.insert(map.into(), old.clone());
    }
    assert_eq!(preserved, *base.data());
    assert_eq!(pack.runtime_files(), base.runtime_files());
    assert_eq!(pack.compiled_audio(), base.compiled_audio());
    let repeated = refresh_grand_marais_story_dialogue(&pack, &source).expect("idempotent hints");
    assert_eq!(repeated.data(), pack.data());
}
