//! Exact old-content preservation, native room reconstruction and unsafe-source rejection.
use std::path::PathBuf;

use crystal_assets::{build_moorhead_modpack, read_verified_compiled_game_pack};
use serde_json::{Value, json};

#[test]
#[ignore = "requires CRYSTAL_MOORHEAD_BASE and CRYSTAL_INTERIOR_PACK"]
fn moorhead_preserves_the_region_and_rejects_unsafe_or_weakened_source() {
    let base = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_MOORHEAD_BASE").expect("completed Detroit parent"),
    ))
    .expect("verified Moorhead parent");
    let published = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_INTERIOR_PACK").expect("written Moorhead publication"),
    ))
    .expect("verified Moorhead child");
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota/towns");
    let town = std::fs::read_to_string(dir.join("MnMoorhead.json")).expect("Moorhead dossier");
    let layout =
        std::fs::read_to_string(dir.join("MnMoorhead.layout.json")).expect("Moorhead layout");
    let rebuilt = build_moorhead_modpack(&base, &town, &layout).expect("deterministic publication");
    assert_eq!(
        rebuilt.identity().expect("rebuilt identity"),
        published.identity().expect("written identity")
    );
    let before = base.data();
    let after = published.data();
    assert_eq!(after.maps.len(), before.maps.len() + 3);
    let added = after
        .maps
        .keys()
        .filter(|n| !before.maps.contains_key(*n))
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert_eq!(
        added,
        ["MnMoorhead", "MnMoorheadMart", "MnMoorheadPokecenter1F"]
    );
    assert!(
        base.compiled_audio() == published.compiled_audio(),
        "all original audio programs and PCM remain exact"
    );
    assert!(
        base.runtime_files()
            .keys()
            .eq(published.runtime_files().keys()),
        "no duplicate or modified art/catalog files"
    );
    for (name, bytes) in base.runtime_files() {
        assert!(
            published.runtime_files().get(name) == Some(bytes),
            "original native file {name}"
        );
    }
    for (name, old) in &before.maps {
        let new = &after.maps[name];
        let mut expected = old.clone();
        if name == "MnR16C00R00" {
            let old_connections = &old.attributes.connections;
            let connections = &new.attributes.connections;
            assert_eq!(connections.len(), old_connections.len() + 1);
            assert_eq!(&connections[..old_connections.len()], old_connections);
            let added = connections.last().expect("one new west connection");
            assert_eq!(
                (&*added.direction, &*added.target_map, added.offset),
                ("west", "MnMoorhead", 6)
            );
            expected.attributes.connections = connections.clone();
            expected.attributes.connection_flags = new.attributes.connection_flags.clone();
        }
        assert!(
            expected == *new,
            "all original blocks, actors, trainers, events and programs: {name}"
        );
    }
    let mut old = serde_json::to_value(before).expect("parent dataset");
    let mut new = serde_json::to_value(after).expect("child dataset");
    for field in [
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
        "marts",
    ] {
        old.as_object_mut().expect("dataset").remove(field);
        new.as_object_mut().expect("dataset").remove(field);
    }
    assert!(
        old == new,
        "all unaffected catalog, art definitions, field moves, title, battle and encounter data"
    );
    for (name, value) in &before.map_scripts {
        assert!(
            after.map_scripts.get(name) == Some(value),
            "original split program {name}"
        );
    }
    for (name, value) in &before.npcs {
        assert!(
            after.npcs.get(name) == Some(value),
            "original split NPCs {name}"
        );
    }
    for (name, value) in &before.map_blocks {
        assert_eq!(
            after.map_blocks.get(name),
            Some(value),
            "original block bytes {name}"
        );
    }
    for (name, value) in &before.map_dimensions {
        assert_eq!(
            after.map_dimensions.get(name),
            Some(value),
            "original dimensions {name}"
        );
    }
    for (name, value) in &before.map_attributes {
        if name != "MnR16C00R00" {
            assert_eq!(
                after.map_attributes.get(name),
                Some(value),
                "original attributes {name}"
            );
        }
    }
    for (name, value) in &before.runtime_map_metadata {
        assert_eq!(after.runtime_map_metadata.get(name), Some(value));
    }
    for (name, value) in &before.runtime_spawn_points {
        assert_eq!(after.runtime_spawn_points.get(name), Some(value));
    }
    assert_eq!(
        after.runtime_spawn_points.len(),
        before.runtime_spawn_points.len() + 2
    );
    let clinic = after
        .runtime_spawn_points
        .values()
        .find(|s| s.map_name == "MnMoorheadPokecenter1F")
        .expect("native clinic recovery");
    assert_eq!((clinic.tile_x, clinic.tile_y), (3, 6));
    for (name, value) in &before.marts.0 {
        assert_eq!(after.marts.0.get(name), Some(value));
    }
    assert_eq!(after.marts.0.len(), before.marts.0.len() + 1);
    for (name, value) in &before.story_event_script_constants.global {
        assert_eq!(
            after.story_event_script_constants.global.get(name),
            Some(value)
        );
    }
    assert_eq!(
        before.story_event_script_constants.maps,
        after.story_event_script_constants.maps
    );
    assert_eq!(
        before.pokegear_landmarks.landmarks,
        after.pokegear_landmarks.landmarks
    );
    for (name, value) in &before.pokegear_landmarks.map_to_landmark {
        assert_eq!(
            after.pokegear_landmarks.map_to_landmark.get(name),
            Some(value)
        );
    }
    for name in added {
        let map = &after.maps[name];
        assert!(!map.objects.is_empty(), "real residents in {name}");
        assert_eq!(
            after
                .assemble_map_module_from_compiled_payloads(name)
                .expect("native split reconstruction"),
            *map
        );
    }
    let raw: Value = serde_json::from_str(&town).expect("source");
    for i in 0..5 {
        let mut bad = raw.clone();
        match i {
            0 => {
                bad["interiors"].as_array_mut().expect("rooms").pop();
            }
            1 => bad["doors"][0]["tile"] = json!([16, 7]),
            2 => bad["interiors"][0]["residents"][0]["counter_tile"] = json!([3, 3]),
            3 => bad["interiors"][0]["pc"]["approach"] = json!([9, 1]),
            _ => bad["quest_contract"]["reward_quantity"] = json!(2),
        }
        assert!(
            build_moorhead_modpack(&base, &bad.to_string(), &layout).is_err(),
            "reject unsafe source {i}"
        );
    }
    let raw: Value = serde_json::from_str(&layout).expect("layout");
    for (name, layers) in [
        (
            "rail gap",
            json!([{"name":"invalid rail gap","kind":"point","position":[5,6],"block":"01"}]),
        ),
        (
            "open river",
            json!([{"name":"invalid bank gap","kind":"point","position":[4,6],"block":"35"},{"name":"invalid rail gap","kind":"point","position":[5,6],"block":"01"}]),
        ),
        (
            "unbound north exit",
            json!([{"name":"invalid north exit","kind":"point","position":[9,0],"block":"06"}]),
        ),
    ] {
        let mut bad = raw.clone();
        bad["layers"]
            .as_array_mut()
            .expect("layers")
            .extend(layers.as_array().expect("mutation rows").iter().cloned());
        assert!(
            build_moorhead_modpack(&base, &town, &bad.to_string()).is_err(),
            "reject {name}"
        );
    }
    assert!(
        build_moorhead_modpack(&published, &town, &layout).is_err(),
        "duplicate publication"
    );
}
