//! Native gate preservation, split reconstruction and unsafe-source rejection.
use std::path::PathBuf;

use crystal_assets::{build_shore_gate_modpack, read_verified_compiled_game_pack};
use serde_json::{Value, json};

#[test]
#[ignore = "requires CRYSTAL_SHORE_GATE_BASE and CRYSTAL_INTERIOR_PACK"]
fn shore_gate_preserves_the_region_and_rejects_unsafe_source() {
    let base = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_SHORE_GATE_BASE").expect("Winona parent"),
    ))
    .expect("strict parent");
    let child = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_INTERIOR_PACK").expect("written gate child"),
    ))
    .expect("strict child");
    let source = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../planning/minnesota/towns/MnGrandMaraisShoreGate.json"),
    )
    .expect("authored gate");
    let rebuilt =
        build_shore_gate_modpack(&base, &source).expect("deterministic native publication");
    assert_eq!(
        rebuilt.identity().expect("rebuilt identity"),
        child.identity().expect("written identity")
    );
    assert!(base.compiled_audio() == child.compiled_audio());
    assert!(
        base.runtime_files() == child.runtime_files(),
        "no copied or modified art/audio/catalog"
    );
    let before = base.data();
    let after = child.data();
    assert_eq!(after.maps.len(), before.maps.len() + 1);
    assert_eq!(
        after
            .maps
            .keys()
            .filter(|k| !before.maps.contains_key(*k))
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["MnGrandMaraisShoreGate"]
    );
    for (name, old) in &before.maps {
        let new = &after.maps[name];
        if name != "MnMaraisCedarApproach" {
            assert!(new == old, "protected native room/story {name}");
            continue;
        }
        let mut expected = old.blocks.clone();
        for ([x, y], block) in [
            ([19, 12], 2),
            ([20, 12], 8),
            ([21, 12], 9),
            ([22, 12], 2),
            ([19, 13], 0x0e),
            ([20, 13], 0x10),
            ([21, 13], 0x11),
            ([22, 13], 0x0b),
        ] {
            expected[y * 32 + x] = block;
        }
        assert_eq!(new.blocks, expected);
        assert!(old.events.warps.is_empty());
        assert_eq!(new.events.warps.len(), 8);
        assert_eq!(new.attributes, old.attributes);
        assert_eq!(new.objects, old.objects);
        let old = serde_json::to_value(old).expect("old module");
        let new = serde_json::to_value(new).expect("new module");
        for (key, value) in old.as_object().expect("module") {
            if !matches!(
                key.as_str(),
                "blocks" | "events" | "map_event_section_commands"
            ) {
                assert_eq!(&new[key], value, "unchanged Cedar {key}");
            }
        }
    }
    for name in ["MnGrandMaraisShoreGate", "MnMaraisCedarApproach"] {
        assert_eq!(
            after
                .assemble_map_module_from_compiled_payloads(name)
                .expect("typed/raw native reconstruction"),
            after.maps[name]
        );
    }
    let gate = &after.maps["MnGrandMaraisShoreGate"];
    assert_eq!(gate.blocks, before.maps["IlexForestAzaleaGate"].blocks);
    assert_eq!(gate.events.warps.len(), 8);
    assert_eq!(gate.objects.len(), 2);
    assert!(gate.script_item_grants.is_empty() && gate.script_flag_commands.is_empty());
    assert_eq!(after.runtime_spawn_points, before.runtime_spawn_points);
    assert_eq!(
        after.pokegear_landmarks.landmarks,
        before.pokegear_landmarks.landmarks
    );
    for (name, value) in &before.pokegear_landmarks.map_to_landmark {
        assert_eq!(
            after.pokegear_landmarks.map_to_landmark.get(name),
            Some(value)
        );
    }
    for (name, value) in &before.map_attributes {
        assert_eq!(after.map_attributes.get(name), Some(value));
    }
    for (name, value) in &before.map_dimensions {
        assert_eq!(after.map_dimensions.get(name), Some(value));
    }
    for (name, value) in &before.runtime_map_metadata {
        assert_eq!(after.runtime_map_metadata.get(name), Some(value));
    }
    let exterior = &before.maps["MnMaraisCedarApproach"];
    for (name, value) in &before.map_blocks {
        if Some(name) != exterior.attributes.blocks_label.as_ref() {
            assert_eq!(after.map_blocks.get(name), Some(value));
        }
    }
    for (name, value) in &before.map_scripts {
        if Some(name) != exterior.attributes.map_events_label.as_ref() {
            assert_eq!(
                after.map_scripts.get(name),
                Some(value),
                "old program {name}"
            );
        }
    }
    for (name, value) in &before.npcs {
        assert_eq!(after.npcs.get(name), Some(value));
    }
    assert_eq!(
        after.story_event_script_constants.maps,
        before.story_event_script_constants.maps
    );
    for (name, value) in &before.story_event_script_constants.global {
        assert_eq!(
            after.story_event_script_constants.global.get(name),
            Some(value)
        );
    }
    assert_eq!(
        after
            .story_event_script_constants
            .global
            .keys()
            .filter(|n| !before.story_event_script_constants.global.contains_key(*n))
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["MAPGROUP_MINNESOTA_SHORE_GATE"]
    );
    let mut old = serde_json::to_value(before).expect("dataset");
    let mut new = serde_json::to_value(after).expect("dataset");
    for key in [
        "maps",
        "map_attributes",
        "map_blocks",
        "map_dimensions",
        "map_scripts",
        "npcs",
        "runtime_map_metadata",
        "runtime_spawn_points",
        "pokegear_landmarks",
        "story_event_script_constants",
    ] {
        old.as_object_mut().expect("dataset").remove(key);
        new.as_object_mut().expect("dataset").remove(key);
    }
    assert_eq!(
        old, new,
        "all other species, battle, item, encounter, audio and regional data"
    );
    let source: Value = serde_json::from_str(&source).expect("source");
    for (path, value) in [
        (vec!["rows", "0"], json!("00 00 00 00 00")),
        (vec!["residents", "0", "tile"], json!([5, 3])),
        (vec!["residents", "1", "tile"], json!([8, 5])),
        (vec!["passive_arrivals", "0", "tile"], json!([0, 4])),
        (vec!["exits", "0", "target_warp"], json!(1)),
        (
            vec!["exterior_binding", "sides", "0", "room_passive_indices"],
            json!([7, 8]),
        ),
        (
            vec!["exterior_binding", "patches", "0", "after"],
            json!("09"),
        ),
        (vec!["exterior_binding", "retained_connections"], json!([])),
    ] {
        let mut bad = source.clone();
        let mut field = &mut bad;
        for part in path {
            field = if let Ok(index) = part.parse::<usize>() {
                &mut field[index]
            } else {
                &mut field[part]
            };
        }
        *field = value;
        assert!(
            build_shore_gate_modpack(&base, &bad.to_string()).is_err(),
            "unsafe source accepted"
        );
    }
    assert!(
        build_shore_gate_modpack(&child, &source.to_string()).is_err(),
        "duplicate publication"
    );
}
