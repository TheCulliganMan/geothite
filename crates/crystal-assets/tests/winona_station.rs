//! Preservation, native split reconstruction and rejected unsafe depot contracts.
use std::path::PathBuf;

use crystal_assets::{build_winona_station_modpack, read_verified_compiled_game_pack};
use serde_json::{Value, json};

#[test]
#[ignore = "requires CRYSTAL_WINONA_STATION_BASE and CRYSTAL_INTERIOR_PACK"]
fn winona_station_preserves_the_region_and_rejects_unsafe_source() {
    let base = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_WINONA_STATION_BASE").expect("Falls parent"),
    ))
    .expect("strict parent");
    let child = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_INTERIOR_PACK").expect("published depot"),
    ))
    .expect("strict child");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota");
    let town = std::fs::read_to_string(root.join("towns/MnWinona.json")).expect("town");
    let station =
        std::fs::read_to_string(root.join("transit/MnWinonaStation.json")).expect("station");
    let rebuilt =
        build_winona_station_modpack(&base, &town, &station).expect("deterministic rebuild");
    assert_eq!(
        rebuilt.identity().expect("rebuilt identity"),
        child.identity().expect("written identity")
    );
    let before = base.data();
    let after = child.data();
    assert_eq!(after.maps.len(), before.maps.len() + 1);
    assert_eq!(
        after
            .maps
            .keys()
            .filter(|name| !before.maps.contains_key(*name))
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["MnWinonaStation"]
    );
    assert!(base.compiled_audio() == child.compiled_audio());
    assert!(
        base.runtime_files() == child.runtime_files(),
        "no copied/modified source art or catalog"
    );
    for (name, old) in &before.maps {
        let new = &after.maps[name];
        if name != "MnWinona" {
            assert!(new == old, "untouched map/room/story {name}");
            continue;
        }
        assert_eq!(new.attributes, old.attributes);
        assert_eq!(new.objects, old.objects);
        assert_eq!(new.events.coord_events, old.events.coord_events);
        assert_eq!(&new.events.warps[..8], old.events.warps);
        assert_eq!(new.events.warps.len(), 10);
        let door = &new.events.warps[8];
        assert_eq!((door.x, door.y, door.target_warp_id), (19, 13, 3));
        assert_eq!(door.target_map_constant, "MN_WINONA_STATION");
        assert_eq!(new.events.warps[9].target_map_constant, "MN_WINONA");
        assert_eq!((new.events.warps[9].x, new.events.warps[9].y), (19, 14));
        assert_eq!(new.events.bg_events.len(), old.events.bg_events.len() + 1);
        assert_eq!(
            &new.events.bg_events[..old.events.bg_events.len()],
            old.events.bg_events
        );
        let mut expected = old.blocks.clone();
        for ([x, y], block) in [
            ([9, 5], 0x18),
            ([10, 5], 0x19),
            ([9, 6], 0x16),
            ([10, 6], 0x1e),
            ([11, 6], 0x45),
        ] {
            expected[y * 16 + x] = block;
        }
        assert_eq!(new.blocks, expected);
        let original = serde_json::to_value(old).expect("old module");
        let revised = serde_json::to_value(new).expect("new module");
        for (key, value) in original.as_object().expect("module") {
            if matches!(
                key.as_str(),
                "blocks" | "events" | "map_event_section_commands"
            ) {
                continue;
            }
            match (value, &revised[key]) {
                (Value::Object(old), Value::Object(new)) => {
                    for (name, value) in old {
                        assert_eq!(new.get(name), Some(value), "old {key}/{name}");
                    }
                    for name in new.keys().filter(|name| !old.contains_key(*name)) {
                        assert!(
                            name.starts_with("MN_WINONA_DEPOT_SIGN"),
                            "only one new outdoor placard"
                        );
                    }
                }
                (Value::Array(old), Value::Array(new)) => {
                    assert!(new.starts_with(old), "old {key} prefix");
                    assert!(
                        new[old.len()..]
                            .iter()
                            .all(|v| v.to_string().contains("MN_WINONA_DEPOT_SIGN")),
                        "only that placard's compiled commands may append to {key}"
                    );
                }
                _ => assert_eq!(value, &revised[key], "unchanged {key}"),
            }
        }
    }
    for name in ["MnWinona", "MnWinonaStation"] {
        assert_eq!(
            after
                .assemble_map_module_from_compiled_payloads(name)
                .expect("complete split reconstruction"),
            after.maps[name],
            "raw and typed native source must agree"
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
    let blocks_label = before.maps["MnWinona"]
        .attributes
        .blocks_label
        .as_ref()
        .expect("blocks label");
    for (name, value) in &before.map_blocks {
        if name != blocks_label {
            assert_eq!(after.map_blocks.get(name), Some(value));
        }
    }
    let events_label = before.maps["MnWinona"]
        .attributes
        .map_events_label
        .as_ref()
        .expect("events label");
    for (name, value) in &before.map_scripts {
        if name != events_label {
            assert_eq!(
                after.map_scripts.get(name),
                Some(value),
                "old native program {name}"
            );
        }
    }
    for (name, value) in &before.npcs {
        assert_eq!(after.npcs.get(name), Some(value));
    }
    assert_eq!(
        before.story_event_script_constants.maps,
        after.story_event_script_constants.maps
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
            .filter(|name| !before
                .story_event_script_constants
                .global
                .contains_key(*name))
            .map(String::as_str)
            .collect::<Vec<_>>(),
        [
            "EVENT_MN_STATION_WINONA_VISITED",
            "EVENT_MN_WINONA_DEPOT_DRY_KIT_GIVEN",
            "EVENT_MN_WINONA_DEPOT_LOG_READ",
            "EVENT_MN_WINONA_DEPOT_MASON_HEARD",
            "MAPGROUP_MINNESOTA_WINONA_STATION"
        ]
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
        "all other catalog, encounter, title, service, battle and field data"
    );
    let raw: Value = serde_json::from_str(&station).expect("source");
    let mut bad = raw.clone();
    bad["passive_arrival"]["tile"] = json!([11, 9]);
    let mut cases = vec![bad];
    let mut bad = raw.clone();
    bad["exterior_binding"]["patches"][0]["before"] = json!("01");
    cases.push(bad);
    let mut bad = raw.clone();
    bad["residents"][0]["tile"] = json!([10, 9]);
    cases.push(bad);
    let mut bad = raw.clone();
    bad["residents"][1]["approach"] = json!([6, 5]);
    cases.push(bad);
    let mut bad = raw.clone();
    bad["exits"][0]["target_warp"] = json!(9);
    cases.push(bad);
    let mut bad = raw.clone();
    bad["story_contract"]["reward_quantity"] = json!(2);
    cases.push(bad);
    let mut bad = raw.clone();
    bad["story_contract"]["visit_flag"] = json!("EVENT_MN_COURIER_DEFEATED");
    cases.push(bad);
    let mut bad = raw;
    bad["exterior_binding"]["retained_edge_lanes"]
        .as_array_mut()
        .expect("lanes")
        .pop();
    cases.push(bad);
    for bad in cases {
        assert!(
            build_winona_station_modpack(&base, &town, &bad.to_string()).is_err(),
            "unsafe source accepted"
        );
    }
    assert!(
        build_winona_station_modpack(&child, &town, &station).is_err(),
        "duplicate publication"
    );
}
