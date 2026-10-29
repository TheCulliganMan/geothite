//! Written-pack reconstruction, precise preservation and unsafe-input rejection.
use std::path::PathBuf;

use serde_json::{Value, json};

use crate::staples_station::{JUNCTION, MART, ROAD, ROOM, VISITED};
use crate::{build_staples_station_pack, read_verified_compiled_game_pack};

#[path = "staples_station_roster_tests.rs"]
mod roster;

#[test]
#[ignore = "requires CRYSTAL_STAPLES_BASE and the written CRYSTAL_INTERIOR_PACK"]
fn staples_station_preserves_native_region_and_rejects_unsafe_sources() {
    let base = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_STAPLES_BASE").expect("Split Rock parent"),
    ))
    .expect("strict parent");
    let child = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_INTERIOR_PACK").expect("written Staples child"),
    ))
    .expect("strict child");
    let source = std::fs::read_to_string(
        std::env::var_os("CRYSTAL_STAPLES_SOURCE")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../planning/minnesota/transit/MnStaplesStation.json")
            }),
    )
    .expect("authored station");
    let rebuilt = build_staples_station_pack(&base, &source).expect("deterministic publication");
    assert!(
        rebuilt == child,
        "written pack must equal a fresh deterministic Rust build"
    );
    assert!(
        base.runtime_files() == child.runtime_files(),
        "all native source art stays exact"
    );
    assert!(
        base.compiled_audio() == child.compiled_audio(),
        "all audio stays exact"
    );
    let (before, after) = (base.data(), child.data());
    assert_eq!(after.maps.len(), before.maps.len() + 2);
    let source: Value = serde_json::from_str(&source).expect("source");
    for (name, original) in &before.maps {
        let actual = &after.maps[name];
        if name != ROAD {
            assert!(
                actual == original,
                "every existing interior, gym, story and route: {name}"
            );
            continue;
        }
        let mut expected = original.clone();
        for patch in source["road_binding"]["patches"]
            .as_array()
            .expect("nine cells")
        {
            let x = patch["position"][0].as_u64().expect("x") as usize;
            let y = patch["position"][1].as_u64().expect("y") as usize;
            expected.blocks[y * 32 + x] = patch["after"].as_u64().expect("block") as u16;
        }
        expected
            .attributes
            .connections
            .push(crystal_core::map::MapConnection {
                direction: "north".into(),
                target_map: JUNCTION.into(),
                offset: 10,
            });
        expected.attributes.connection_flags = Some(
            expected
                .attributes
                .connections
                .iter()
                .map(|c| c.direction.to_ascii_uppercase())
                .collect::<Vec<_>>()
                .join(" | "),
        );
        expected.attributes.fishing_group = Some("FISHGROUP_MN_STAPLES_R15".into());
        assert!(
            actual == &expected,
            "only nine blocks, one connection and the local fishing binding may change on the old road"
        );
    }
    for name in [ROAD, JUNCTION, ROOM] {
        assert_eq!(
            after
                .assemble_map_module_from_compiled_payloads(name)
                .expect("native reconstruction"),
            after.maps[name]
        );
    }
    for (key, value) in &before.map_scripts {
        assert_eq!(
            after.map_scripts.get(key),
            Some(value),
            "old native program {key}"
        );
    }
    for (key, value) in &before.npcs {
        assert_eq!(
            after.npcs.get(key),
            Some(value),
            "old native NPC table {key}"
        );
    }
    for (key, value) in &before.map_blocks {
        if Some(key) != before.maps[ROAD].attributes.blocks_label.as_ref() {
            assert_eq!(
                after.map_blocks.get(key),
                Some(value),
                "old native blocks {key}"
            );
        }
    }
    for (key, value) in &before.runtime_map_metadata {
        assert_eq!(after.runtime_map_metadata.get(key), Some(value));
    }
    for (key, value) in &before.map_dimensions {
        assert_eq!(after.map_dimensions.get(key), Some(value));
    }
    assert_eq!(
        before.runtime_spawn_points, after.runtime_spawn_points,
        "no new recovery or HOME"
    );
    assert_eq!(
        before.pokegear_landmarks.landmarks,
        after.pokegear_landmarks.landmarks
    );
    for (name, landmark) in &before.pokegear_landmarks.map_to_landmark {
        assert_eq!(
            after.pokegear_landmarks.map_to_landmark.get(name),
            Some(landmark)
        );
    }
    for (name, stock) in &before.marts.0 {
        assert_eq!(after.marts.0.get(name), Some(stock));
    }
    assert_eq!(after.marts.0.len(), before.marts.0.len() + 1);
    assert_eq!(
        after.marts.0[MART],
        ["REPEL", "SUPER_POTION", "PARLYZ_HEAL", "FLOWER_MAIL"]
    );
    let old_constants = &before.story_event_script_constants;
    let new_constants = &after.story_event_script_constants;
    assert_eq!(old_constants.maps, new_constants.maps);
    for (key, value) in &old_constants.global {
        assert_eq!(new_constants.global.get(key), Some(value));
    }
    assert_eq!(new_constants.global.len(), old_constants.global.len() + 2);
    let event = new_constants.global[VISITED];
    assert!(
        !old_constants
            .global
            .iter()
            .any(|(key, value)| key.starts_with("EVENT_") && *value == event)
    );
    let station = &after.maps[ROOM];
    assert_eq!(station.objects.len(), 4);
    assert!(station.script_item_grants.is_empty());
    assert_eq!(station.script_flag_commands.len(), 1);
    assert!(
        station
            .map_script_section_commands
            .iter()
            .any(|command| command.command == "callback"
                && command.args == ["MAPCALLBACK_NEWMAP", "MnStaplesFirstVisit"])
    );
    assert!(after.maps[JUNCTION].script_flag_commands.is_empty());
    roster::verify(&base, &child);
    // The touched route's regional species bindings are verified separately.
    // Other catalogs, story and transport services remain exact.
    let mut old = serde_json::to_value(before).expect("old dataset");
    let mut new = serde_json::to_value(after).expect("new dataset");
    for key in [
        "maps",
        "map_attributes",
        "map_blocks",
        "map_dimensions",
        "map_scripts",
        "npcs",
        "runtime_map_metadata",
        "pokegear_landmarks",
        "story_event_script_constants",
        "marts",
        "trainers",
        "wild_encounters",
        "fishing",
    ] {
        old.as_object_mut().expect("old fields").remove(key);
        new.as_object_mut().expect("new fields").remove(key);
    }
    assert_eq!(old, new, "all other data remains unchanged");
    for (pointer, replacement) in [
        ("/base_content_hash", json!("stale")),
        ("/roster_policy", json!("legacy")),
        ("/road_binding/route_offset", json!(11)),
        ("/road_binding/patches/0/after", json!(0x50)),
        ("/road_binding/patches/1/position", json!([0, 0])),
        (
            "/exterior/rows/0",
            json!("01 01 01 01 01 01 01 01 01 01 01 01 01 01 01 01"),
        ),
        ("/room/rows/2", json!("0f 14 10 11 15 12 13 10 0f")),
        ("/room/passive_arrival/tile", json!([8, 17])),
        ("/room/residents/0/tile", json!([12, 9])),
        ("/room/residents/2/stock", json!(["POTION"])),
        ("/room/residents/2/service", json!("heal_party")),
        ("/room/exits/0/target_warp", json!(1)),
        ("/boarding/train_door_core", json!([5, 5])),
    ] {
        let mut invalid = source.clone();
        *invalid.pointer_mut(pointer).expect("counterexample field") = replacement;
        assert!(
            build_staples_station_pack(&base, &invalid.to_string()).is_err(),
            "unsafe source accepted: {pointer}"
        );
    }
    assert!(
        build_staples_station_pack(&child, &source.to_string()).is_err(),
        "duplicate station rejected"
    );
}
