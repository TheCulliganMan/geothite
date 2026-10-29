//! Actual publication, native reconstruction, recovery and regional preservation.
use std::path::PathBuf;

use crate::{build_split_rock_clinic_pack, read_verified_compiled_game_pack};
use serde_json::{Value, json};

const ROOM: &str = "MnSplitRockPokecenter1F";
const SHORE: &str = "MnR11C00R02";

#[test]
#[ignore = "requires CRYSTAL_SPLIT_ROCK_BASE and CRYSTAL_INTERIOR_PACK"]
fn split_rock_clinic_preserves_the_region_and_rejects_unsafe_source() {
    let base = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_SPLIT_ROCK_BASE").expect("fishing-home parent"),
    ))
    .expect("strict parent");
    let child = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_INTERIOR_PACK").expect("written clinic child"),
    ))
    .expect("strict child");
    let source = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../planning/minnesota/interior-life/split-rock-clinic.json"),
    )
    .expect("authored clinic");
    let rebuilt = build_split_rock_clinic_pack(&base, &source).expect("deterministic publication");
    assert_eq!(
        rebuilt.identity().expect("rebuilt identity"),
        child.identity().expect("written identity")
    );
    assert!(
        base.runtime_files() == child.runtime_files(),
        "source assets remain exact"
    );
    assert!(
        base.compiled_audio() == child.compiled_audio(),
        "all audio remains exact"
    );
    let before = base.data();
    let after = child.data();
    assert_eq!(after.maps.len(), before.maps.len() + 1);
    let source: Value = serde_json::from_str(&source).expect("source");
    for (name, old) in &before.maps {
        let new = &after.maps[name];
        if name != SHORE {
            assert!(
                new == old,
                "every unrelated room, gym and story remains exact: {name}"
            );
            continue;
        }
        let mut blocks = old.blocks.clone();
        for patch in source["exterior_binding"]["patches"]
            .as_array()
            .expect("fifteen cells")
        {
            let x = patch["position"][0].as_u64().expect("x") as usize;
            let y = patch["position"][1].as_u64().expect("y") as usize;
            blocks[y * 32 + x] = patch["after"].as_u64().expect("block") as u16;
        }
        assert_eq!(new.blocks, blocks);
        assert_eq!(new.attributes, old.attributes);
        assert_eq!(new.objects, old.objects);
        assert_eq!(new.events.warps[..old.events.warps.len()], old.events.warps);
        assert_eq!(new.events.warps.len(), old.events.warps.len() + 2);
        assert_eq!(
            new.events.bg_events[..old.events.bg_events.len()],
            old.events.bg_events
        );
        assert_eq!(new.events.bg_events.len(), old.events.bg_events.len() + 1);
        assert_eq!(new.events.coord_events, old.events.coord_events);
        let original = serde_json::to_value(old).expect("old module");
        let mut restored = serde_json::to_value(new).expect("new module");
        for (key, value) in original.as_object().expect("fields") {
            match key.as_str() {
                "blocks" | "events" | "map_event_section_commands" => restored[key] = value.clone(),
                _ => match value {
                    Value::Array(values) => {
                        let list = restored[key].as_array_mut().expect("native table");
                        assert!(list.starts_with(values), "original typed commands: {key}");
                        list.truncate(values.len());
                    }
                    Value::Object(values) => {
                        let table = restored[key].as_object_mut().expect("native programs");
                        for (label, body) in values {
                            assert_eq!(
                                table.get(label),
                                Some(body),
                                "original native program {label}"
                            );
                        }
                        table.retain(|label, _| values.contains_key(label));
                    }
                    _ => assert_eq!(&restored[key], value),
                },
            }
        }
        assert_eq!(
            restored, original,
            "all existing shore programs and battle data"
        );
    }
    for name in [ROOM, SHORE] {
        assert_eq!(
            after
                .assemble_map_module_from_compiled_payloads(name)
                .expect("typed/raw reconstruction"),
            after.maps[name]
        );
    }
    let clinic = &after.maps[ROOM];
    let mut native = before.maps["CherrygrovePokecenter1F"].blocks.clone();
    native[15] = 4;
    assert_eq!(clinic.blocks, native);
    assert_eq!(clinic.objects.len(), 3);
    assert_eq!(clinic.events.warps.len(), 3);
    assert!(clinic.script_item_grants.is_empty() && clinic.script_flag_commands.is_empty());
    let recovery = after
        .runtime_spawn_points
        .iter()
        .filter(|(key, _)| !before.runtime_spawn_points.contains_key(*key))
        .collect::<Vec<_>>();
    assert_eq!(recovery.len(), 1);
    assert_eq!(recovery[0].1.map_name, ROOM);
    assert_eq!((recovery[0].1.tile_x, recovery[0].1.tile_y), (3, 5));
    for (key, value) in &before.runtime_spawn_points {
        assert_eq!(
            after.runtime_spawn_points.get(key),
            Some(value),
            "existing recovery/home {key}"
        );
    }
    assert_eq!(
        before.pokegear_landmarks.landmarks,
        after.pokegear_landmarks.landmarks
    );
    for (key, value) in &before.pokegear_landmarks.map_to_landmark {
        assert_eq!(
            after.pokegear_landmarks.map_to_landmark.get(key),
            Some(value)
        );
    }
    for (key, value) in &before.map_attributes {
        assert_eq!(after.map_attributes.get(key), Some(value));
    }
    for (key, value) in &before.map_dimensions {
        assert_eq!(after.map_dimensions.get(key), Some(value));
    }
    for (key, value) in &before.runtime_map_metadata {
        assert_eq!(after.runtime_map_metadata.get(key), Some(value));
    }
    for (key, value) in &before.map_blocks {
        if Some(key) != before.maps[SHORE].attributes.blocks_label.as_ref() {
            assert_eq!(after.map_blocks.get(key), Some(value));
        }
    }
    for (key, value) in &before.map_scripts {
        if Some(key) != before.maps[SHORE].attributes.map_events_label.as_ref() {
            assert_eq!(
                after.map_scripts.get(key),
                Some(value),
                "original script {key}"
            );
        }
    }
    for (key, value) in &before.npcs {
        assert_eq!(after.npcs.get(key), Some(value));
    }
    assert_eq!(
        before.story_event_script_constants.maps,
        after.story_event_script_constants.maps
    );
    for (key, value) in &before.story_event_script_constants.global {
        assert_eq!(
            after.story_event_script_constants.global.get(key),
            Some(value)
        );
    }
    assert_eq!(
        after
            .story_event_script_constants
            .global
            .keys()
            .filter(|key| !before
                .story_event_script_constants
                .global
                .contains_key(*key))
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["MAPGROUP_MINNESOTA_SPLIT_ROCK_CLINIC"]
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
        old.as_object_mut().expect("old fields").remove(key);
        new.as_object_mut().expect("new fields").remove(key);
    }
    assert_eq!(
        old, new,
        "all other species, encounters, inventory, battles and story"
    );
    for (pointer, replacement) in [
        ("/room/rows/0", json!("00 00 00 00 00")),
        ("/room/passive_arrival/tile", json!([3, 7])),
        ("/room/pc/tile", json!([7, 0])),
        ("/room/residents/0/counter_tile", json!([3, 3])),
        ("/room/residents/1/tile", json!([3, 5])),
        ("/room/residents/2/service", json!("heal_party")),
        ("/room/exits/0/target_warp", json!(3)),
        ("/exterior_binding/patches/0/position", json!([0, 15])),
        ("/exterior_binding/patches/1/after", json!(5)),
        ("/exterior_binding/patches/2/before", json!(0)),
    ] {
        let mut invalid = source.clone();
        *invalid.pointer_mut(pointer).expect("counterexample field") = replacement;
        assert!(
            build_split_rock_clinic_pack(&base, &invalid.to_string()).is_err(),
            "unsafe clinic source accepted: {pointer}"
        );
    }
    assert!(
        build_split_rock_clinic_pack(&child, &source.to_string()).is_err(),
        "duplicate rejected"
    );
}
