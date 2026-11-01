//! Whole-data preservation for the larger dungeon; source inputs stay external.
use std::collections::BTreeSet;
use std::path::PathBuf;

#[test]
#[ignore = "requires the immutable local drill parent; verifies every unrelated field"]
fn headquarters_preserves_other_chapters_trainers_audio_and_recovery_points() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let base = crate::read_verified_compiled_game_pack(
        root.join("content-packs/minnesota-northstar-drill-lair-v4.crystalpack"),
    )
    .expect("drill parent");
    let directory = root.join("planning/minnesota/lairs/ice-headquarters");
    let source = std::fs::read_to_string(directory.join("network.json")).expect("network");
    let layouts = ["operations", "records", "maintenance", "transfer"].map(|s| {
        std::fs::read_to_string(directory.join(format!("{s}.json"))).expect("authored floor")
    });
    let result = crate::build_northstar_ice_headquarters(&base, &source, &layouts)
        .expect("native headquarters");
    assert!(base.compiled_audio() == result.compiled_audio());
    for (name, bytes) in base.runtime_files() {
        assert_eq!(
            Some(bytes),
            result.runtime_files().get(name),
            "original runtime file {name}"
        );
    }
    assert!(
        result
            .runtime_files()
            .keys()
            .filter(|k| !base.runtime_files().contains_key(*k))
            .all(|k| k.contains("mn_ice_headquarters")),
        "only local native panel art files may be added"
    );
    assert!(base.audio_manifest().expect("audio") == result.audio_manifest().expect("audio"));
    assert_eq!(
        base.data().runtime_spawn_points,
        result.data().runtime_spawn_points
    );
    assert_eq!(result.data().maps.len(), base.data().maps.len() + 4);
    let before = serde_json::to_value(base.data()).expect("all parent data");
    let after = serde_json::to_value(result.data()).expect("all resulting data");
    let mutable = BTreeSet::from([
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
        "tilesets",
    ]);
    for (field, value) in before.as_object().expect("dataset") {
        if !mutable.contains(field.as_str()) {
            assert!(Some(value) == after.get(field), "unrelated data {field}");
        }
    }
    for (name, map) in &base.data().maps {
        if !matches!(name.as_str(), "MnIceCargoBasement" | "MnIceRelayFloor") {
            assert!(map == &result.data().maps[name], "unrelated map {name}");
        }
    }
    for (name, program) in &base.data().map_scripts {
        if matches!(
            name.as_str(),
            "MnIceCargoBasement_MapEvents" | "MnIceRelayFloor_MapEvents"
        ) {
            // Exactly these two wire sections acquire different destinations.
            // Their actors, background reads and coordinate events stay intact.
            let non_warps = |value: &serde_json::Value| {
                value
                    .as_array()
                    .expect("event rows")
                    .iter()
                    .filter(|r| r["command"] != "warp_event")
                    .cloned()
                    .collect::<Vec<_>>()
            };
            assert_eq!(
                non_warps(program),
                non_warps(&result.data().map_scripts[name])
            );
            continue;
        }
        assert_eq!(
            Some(program),
            result.data().map_scripts.get(name),
            "original program {name}"
        );
    }
    for (name, trainer) in &base.data().trainers.trainers {
        assert_eq!(
            Some(trainer),
            result.data().trainers.trainers.get(name),
            "original trainer {name}"
        );
    }
    for (name, tileset) in &base.data().tilesets {
        assert_eq!(
            Some(tileset),
            result.data().tilesets.get(name),
            "original tileset {name}"
        );
    }
    assert_eq!(
        result.data().tilesets["mn_ice_headquarters"].collision["40"][1],
        "WARP_PANEL"
    );
    for name in [
        "MnIceOperations",
        "MnIceRecords",
        "MnIceMaintenance",
        "MnIceTransfer",
    ] {
        let room = &result.data().maps[name];
        assert_eq!(
            room.objects
                .iter()
                .filter(|o| o.object_type == "OBJECTTYPE_TRAINER")
                .count(),
            1
        );
        for warp in &room.events.warps {
            assert!(warp.target_warp_id > 0);
        }
    }
    assert!(
        crate::build_northstar_ice_headquarters(&result, &source, &layouts).is_err(),
        "reject repeated overlay"
    );
    let mut bad: serde_json::Value = serde_json::from_str(&source).expect("network");
    bad["links"][0]["b"]["active"] = serde_json::json!([65001, 65000]);
    assert!(
        crate::build_northstar_ice_headquarters(&base, &bad.to_string(), &layouts).is_err(),
        "out-of-floor connection is an error"
    );
}
