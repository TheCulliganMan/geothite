//! Exact regional preservation and rejected unsafe SQ03/source contracts.
use std::path::PathBuf;

use crystal_assets::{build_international_falls_modpack, read_verified_compiled_game_pack};
use serde_json::{Value, json};

fn remove_alias(value: &mut Value) {
    match value {
        Value::Object(fields) => {
            fields.remove("mn_international_falls");
            for v in fields.values_mut() {
                remove_alias(v);
            }
        }
        Value::Array(values) => {
            for v in values {
                remove_alias(v);
            }
        }
        _ => {}
    }
}

#[test]
#[ignore = "requires completed CRYSTAL_FALLS_BASE and CRYSTAL_INTERIOR_PACK"]
fn international_falls_preserves_the_region_and_rejects_unsafe_sources() {
    let base = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_FALLS_BASE").expect("Moorhead parent"),
    ))
    .expect("strict parent");
    let child = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_INTERIOR_PACK").expect("Falls child"),
    ))
    .expect("strict child");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota/towns");
    let town =
        std::fs::read_to_string(root.join("MnInternationalFalls.json")).expect("town source");
    let layout = std::fs::read_to_string(root.join("MnInternationalFalls.layout.json"))
        .expect("layout source");
    let rebuilt = build_international_falls_modpack(&base, &town, &layout)
        .expect("deterministic source publication");
    assert_eq!(
        rebuilt.identity().expect("rebuilt identity"),
        child.identity().expect("child identity")
    );
    let before = base.data();
    let after = child.data();
    assert_eq!(after.maps.len(), before.maps.len() + 3);
    assert!(base.compiled_audio() == child.compiled_audio());
    for (name, bytes) in base.runtime_files() {
        assert!(
            child.runtime_files().get(name) == Some(bytes),
            "old native file {name}"
        );
    }
    for (name, definition) in &before.tilesets {
        assert_eq!(
            after.tilesets.get(name),
            Some(definition),
            "old tileset {name}"
        );
    }
    assert_eq!(after.tilesets.len(), before.tilesets.len() + 1);
    let mut moves = serde_json::to_value(&after.field_moves).expect("field moves");
    remove_alias(&mut moves);
    assert_eq!(
        moves,
        serde_json::to_value(&before.field_moves).expect("old rules")
    );
    for (name, old) in &before.maps {
        let new = &after.maps[name];
        if name == "MnBoundaryWaters" {
            assert_eq!(new.attributes, old.attributes);
            assert_eq!(new.blocks, old.blocks);
            assert_eq!(new.objects, old.objects);
            assert_eq!(new.events.warps, old.events.warps);
            assert_eq!(new.events.bg_events, old.events.bg_events);
            assert_eq!(new.events.coord_events, old.events.coord_events);
            for (key, program) in &old.scripts {
                assert_eq!(new.scripts.get(key), Some(program));
            }
            assert_eq!(new.scripts.len(), old.scripts.len() + 3);
            let a = serde_json::to_value(old).expect("old hub");
            let b = serde_json::to_value(new).expect("new hub");
            for (key, v) in a.as_object().expect("module") {
                if matches!(key.as_str(), "scripts" | "events") {
                    continue;
                }
                match (v, &b[key]) {
                    (Value::Array(old), Value::Array(new)) => {
                        assert!(new.starts_with(old), "old hub payload prefix {key}");
                        for addition in &new[old.len()..] {
                            assert!(
                                addition.to_string().contains("MnFalls"),
                                "unrelated new hub payload {key}"
                            );
                        }
                    }
                    _ => assert_eq!(v, &b[key], "unchanged hub payload {key}"),
                }
            }
        } else {
            let mut expected = old.clone();
            if matches!(name.as_str(), "MnR17C00R00" | "MnRainyRiverIcefields") {
                assert!(
                    new.attributes
                        .connections
                        .starts_with(&old.attributes.connections)
                );
                assert_eq!(
                    new.attributes.connections.len(),
                    old.attributes.connections.len() + 1
                );
                expected.attributes.connections = new.attributes.connections.clone();
                expected.attributes.connection_flags = new.attributes.connection_flags.clone();
                if name == "MnRainyRiverIcefields" {
                    for x in 0..3 {
                        assert_eq!(expected.blocks[14 * 32 + x], 0x3f);
                        expected.blocks[14 * 32 + x] = 0x3c;
                    }
                }
            }
            assert!(new == &expected, "exact old map preservation {name}");
        }
    }
    for (key, v) in &before.map_scripts {
        if key == "MnBoundaryWaters_MapScripts" {
            let mut expected = v.clone();
            expected
                .as_array_mut()
                .expect("native callback section")
                .push(json!({
                    "command":"callback","args":["MAPCALLBACK_NEWMAP","MnFallsVoyageursArrival"]
                }));
            assert_eq!(
                after.map_scripts.get(key),
                Some(&expected),
                "one appended native callback"
            );
        } else {
            assert_eq!(
                after.map_scripts.get(key),
                Some(v),
                "old split program {key}"
            );
        }
    }
    for (key, v) in &before.npcs {
        assert_eq!(after.npcs.get(key), Some(v), "old split actors {key}");
    }
    for (key, v) in &before.runtime_map_metadata {
        assert_eq!(after.runtime_map_metadata.get(key), Some(v));
    }
    for (key, v) in &before.runtime_spawn_points {
        assert_eq!(after.runtime_spawn_points.get(key), Some(v));
    }
    assert_eq!(
        after.runtime_spawn_points.len(),
        before.runtime_spawn_points.len() + 2
    );
    for (key, v) in &before.story_event_script_constants.global {
        assert_eq!(after.story_event_script_constants.global.get(key), Some(v));
    }
    assert_eq!(
        after
            .story_event_script_constants
            .global
            .keys()
            .filter(|k| !before.story_event_script_constants.global.contains_key(*k))
            .map(String::as_str)
            .collect::<Vec<_>>(),
        [
            "EVENT_MN_BORDER_CIRCUIT_REWARDED",
            "EVENT_MN_BORDER_REPORT",
            "EVENT_MN_FALLS_VOYAGEURS_ARRIVAL",
            "MAPGROUP_MINNESOTA_INTERNATIONAL_FALLS"
        ]
    );
    assert_eq!(
        after.story_event_script_constants.global["MAPGROUP_MINNESOTA_INTERNATIONAL_FALLS"],
        88
    );
    let foregrounds: Value = serde_json::from_slice(
        &child.runtime_files()["data/tilesets/mn_international_falls_foregrounds.json"],
    )
    .expect("private native snow foreground colors");
    assert!(
        !foregrounds
            .as_object()
            .expect("glyph color table")
            .is_empty()
    );
    assert_eq!(
        before.story_event_script_constants.maps,
        after.story_event_script_constants.maps
    );
    assert_eq!(
        before.pokegear_landmarks.landmarks,
        after.pokegear_landmarks.landmarks
    );
    for (key, v) in &before.marts.0 {
        assert_eq!(after.marts.0.get(key), Some(v));
    }
    let mut a = serde_json::to_value(before).expect("old dataset");
    let mut b = serde_json::to_value(after).expect("new dataset");
    for key in [
        "maps",
        "map_attributes",
        "map_blocks",
        "map_dimensions",
        "map_scripts",
        "npcs",
        "runtime_map_metadata",
        "runtime_spawn_points",
        "story_event_script_constants",
        "pokegear_landmarks",
        "marts",
        "tilesets",
        "field_moves",
    ] {
        a.as_object_mut().expect("dataset").remove(key);
        b.as_object_mut().expect("dataset").remove(key);
    }
    assert_eq!(
        a, b,
        "all unaffected catalog, encounters, title, battle and audio data"
    );
    for name in [
        "MnInternationalFalls",
        "MnInternationalFallsPokecenter1F",
        "MnInternationalFallsMart",
        "MnRainyRiverIcefields",
        "MnBoundaryWaters",
    ] {
        assert_eq!(
            after
                .assemble_map_module_from_compiled_payloads(name)
                .expect("split reconstruction"),
            after.maps[name]
        );
    }
    let t: Value = serde_json::from_str(&town).expect("dossier");
    for i in 0..8 {
        let mut bad = t.clone();
        match i {
            0 => {
                bad["interiors"].as_array_mut().expect("rooms").pop();
            }
            1 => bad["doors"][0]["tile"] = json!([8, 7]),
            2 => {
                bad["quest_contract"]["outlet_requires"]
                    .as_array_mut()
                    .expect("prerequisites")
                    .remove(7);
            }
            3 => bad["quest_contract"]["arrival_facing"] = json!("LEFT"),
            4 => bad["quest_contract"]["reward_quantity"] = json!(2),
            5 => bad["interiors"][0]["pc"]["approach"] = json!([9, 1]),
            6 => bad["quest_contract"]["arrival_map"] = json!("MnInternationalFalls"),
            _ => bad["quest_contract"]["arrival_mode"] = json!("coordinate_event"),
        };
        assert!(
            build_international_falls_modpack(&base, &bad.to_string(), &layout).is_err(),
            "unsafe source {i}"
        );
    }
    assert!(
        build_international_falls_modpack(&child, &town, &layout).is_err(),
        "duplicate installation"
    );
}
