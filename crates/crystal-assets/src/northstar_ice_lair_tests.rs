//! Whole-dataset preservation and native gate checks for the ICE overlay.
use std::collections::BTreeSet;
use std::path::PathBuf;

use serde_json::Value;

use crate::grand_marais_modpack::authoring::command as c;

fn source(name: &str) -> String {
    std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../planning/minnesota/lairs")
            .join(name),
    )
    .expect("authored lair source")
}

#[test]
#[ignore = "requires the reviewed external event-opening pack"]
fn ice_lair_preserves_unrelated_data_and_every_original_audio_program() {
    let parent = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../content-packs/minnesota-northstar-events-opening-v2.crystalpack")
        .canonicalize()
        .expect("canonical external opening pack");
    let base = crate::read_verified_compiled_game_pack(parent).expect("verified opening");
    let result = crate::build_northstar_ice_lair(
        &base,
        &source("minneapolis-events.json"),
        &source("minneapolis-cargo.json"),
        &source("minneapolis-relay.json"),
    )
    .expect("native lair overlay");
    assert!(base.compiled_audio() == result.compiled_audio());
    assert!(base.runtime_files() == result.runtime_files());
    assert!(
        base.audio_manifest().expect("parent audio")
            == result.audio_manifest().expect("overlay audio")
    );
    let before = serde_json::to_value(base.data()).expect("whole parent data");
    let after = serde_json::to_value(result.data()).expect("whole overlay data");
    let changed = BTreeSet::from([
        "maps",
        "map_scripts",
        "npcs",
        "map_attributes",
        "map_dimensions",
        "map_blocks",
        "runtime_map_metadata",
        "story_event_script_constants",
        "trainers",
        "pokegear_landmarks",
    ]);
    for (field, original) in before.as_object().expect("data fields") {
        if !changed.contains(field.as_str()) {
            assert!(original == &after[field], "unrelated dataset field {field}");
        }
    }
    let mut expected_landmarks = before["pokegear_landmarks"].clone();
    for room in ["MnIceCargoBasement", "MnIceRelayFloor"] {
        expected_landmarks["map_to_landmark"][room] =
            before["pokegear_landmarks"]["map_to_landmark"]["MnMinneapolisDispatchOffice"].clone();
    }
    assert!(
        expected_landmarks == after["pokegear_landmarks"],
        "only the two actual lair rooms inherit the Minneapolis landmark"
    );
    let rooms = BTreeSet::from([
        "MnMinneapolisDispatchOffice",
        "MnMinneapolisDispatcherHouse",
        "MnIceCargoBasement",
        "MnIceRelayFloor",
    ]);
    for (name, original) in before["maps"].as_object().expect("maps") {
        if !rooms.contains(name.as_str()) {
            assert!(original == &after["maps"][name], "unrelated map {name}");
        }
    }
    assert_eq!(result.data().maps.len(), base.data().maps.len() + 2);
    let mut labels = BTreeSet::new();
    for name in &rooms {
        let map = &result.data().maps[*name];
        labels.extend(map.scripts.keys().map(String::as_str));
        labels.insert(
            map.attributes
                .map_scripts_label
                .as_deref()
                .expect("script section"),
        );
        labels.insert(
            map.attributes
                .map_events_label
                .as_deref()
                .expect("event section"),
        );
        if let Some(original) = base.data().maps.get(*name) {
            for (label, body) in &original.scripts {
                if !matches!(
                    label.as_str(),
                    "MnMPCellLatch" | "MnMPDispatchCopy" | "MN_MP_RESCUED_DISPATCHER"
                ) {
                    assert!(
                        map.scripts.get(label) == Some(body),
                        "unrelated room program {name}/{label}"
                    );
                }
            }
        }
        for (label, body) in &map.scripts {
            assert!(
                result.data().map_scripts.get(label) == Some(body),
                "split script {label}"
            );
        }
    }
    for (label, body) in &base.data().map_scripts {
        if !labels.contains(label.as_str()) {
            assert!(
                result.data().map_scripts.get(label) == Some(body),
                "unrelated global program {label}"
            );
        }
    }
    for (name, body) in before["npcs"].as_object().expect("NPC payloads") {
        if !rooms.contains(name.as_str()) {
            assert!(body == &after["npcs"][name], "unrelated NPC payload {name}");
        }
    }
    for (id, original) in &base.data().trainers.trainers {
        assert!(
            serde_json::to_value(original).expect("trainer")
                == serde_json::to_value(&result.data().trainers.trainers[id])
                    .expect("retained trainer"),
            "original trainer {id}"
        );
    }
    let globals = &result.data().story_event_script_constants.global;
    for (key, value) in &base.data().story_event_script_constants.global {
        assert_eq!(globals.get(key), Some(value), "native constant {key}");
    }
    let office = &result.data().maps["MnMinneapolisDispatchOffice"];
    assert!(
        office.scripts["MnMPDispatchCopy"]
            .as_array()
            .expect("physical desk-copy program")
            .iter()
            .any(|row| row == &c("setevent", &["EVENT_MN_DISPATCH_NOTE"]))
    );
    let latch = office.scripts["MnMPCellLatch"].as_array().expect("latch");
    assert_eq!(
        latch[2],
        c("checkevent", &["EVENT_MN_ICE_HOLDING_POWER_OFF"])
    );
    let home = &result.data().maps["MnMinneapolisDispatcherHouse"];
    let witness = home.scripts["MN_MP_RESCUED_DISPATCHER"]
        .as_array()
        .expect("witness quest");
    assert!(witness.contains(&c("writetext", &["MnNICEEllisQuestText"])));
    assert!(!witness.contains(&c("setevent", &["EVENT_MN_DISPATCH_NOTE"])));
    let copy = office.scripts["MnMPDispatchCopy"]
        .as_array()
        .expect("desk copy");
    for flag in [
        "EVENT_MN_ICE_CAPTIVE_RESCUED",
        "EVENT_MN_ICE_HOLDING_POWER_OFF",
        "EVENT_MN_ICE_ENFORCER_DEFEATED",
    ] {
        assert!(copy.contains(&c("checkevent", &[flag])));
    }
    let result_index = copy
        .iter()
        .position(|row| row == &c("setevent", &["EVENT_MN_DISPATCH_NOTE"]))
        .expect("acknowledged desk-copy result");
    assert_eq!(copy[result_index - 1], c("promptbutton", &[]));
    assert_eq!(
        copy[result_index - 2],
        c("writetext", &["MnMPDispatchReadText"])
    );
    assert!(
        crate::build_northstar_ice_lair(
            &result,
            &source("minneapolis-events.json"),
            &source("minneapolis-cargo.json"),
            &source("minneapolis-relay.json")
        )
        .is_err(),
        "do not duplicate lairs or reward identities"
    );
    let invalid: Value = serde_json::json!({"schema_version":1,"id":"wrong"});
    assert!(
        crate::build_northstar_ice_lair(
            &base,
            &invalid.to_string(),
            &source("minneapolis-cargo.json"),
            &source("minneapolis-relay.json")
        )
        .is_err()
    );
}
