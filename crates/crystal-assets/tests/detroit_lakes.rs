//! Native Detroit publication, exact preservation and meaningful rejection cases.
use std::path::PathBuf;

use crystal_assets::{build_detroit_lakes_modpack, read_verified_compiled_game_pack};
use serde_json::{Value, json};

#[test]
#[ignore = "requires CRYSTAL_DETROIT_BASE and CRYSTAL_INTERIOR_PACK"]
fn detroit_lakes_preserves_original_content_and_rejects_unsafe_room_contracts() {
    let base = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_DETROIT_BASE").expect("completed parent"),
    ))
    .expect("verified parent");
    let published = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_INTERIOR_PACK").expect("Detroit publication"),
    ))
    .expect("verified Detroit pack");
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota");
    let town = std::fs::read_to_string(directory.join("towns/MnDetroitLakes.json")).expect("town");
    let layout = std::fs::read_to_string(directory.join("towns/MnDetroitLakes.layout.json"))
        .expect("layout");
    let station = std::fs::read_to_string(directory.join("transit/MnDetroitLakesStation.json"))
        .expect("station");
    let rebuilt = build_detroit_lakes_modpack(&base, &town, &layout, &station)
        .expect("deterministic native publication");
    assert_eq!(
        rebuilt.identity().expect("rebuilt identity"),
        published.identity().expect("written identity")
    );
    let before = base.data();
    let after = published.data();
    assert_eq!(after.maps.len(), before.maps.len() + 6);
    assert_eq!(base.compiled_audio(), published.compiled_audio());
    for (name, bytes) in base.runtime_files() {
        assert_eq!(
            published.runtime_files().get(name),
            Some(bytes),
            "original file {name}"
        );
    }
    for (name, old) in &before.maps {
        let new = &after.maps[name];
        let mut expected = old.clone();
        if matches!(
            name.as_str(),
            "MnR15C00R00" | "MnDetroitLakesApproach" | "MnR19C00R01"
        ) {
            expected.attributes.connections = new.attributes.connections.clone();
            expected.attributes.connection_flags = new.attributes.connection_flags.clone();
            if name == "MnR19C00R01" {
                for x in [11, 13] {
                    assert_eq!(old.blocks[5 * 32 + x], 2);
                    expected.blocks[5 * 32 + x] = 5;
                }
            }
        }
        assert_eq!(&expected, new, "all original room/program data: {name}");
    }
    let mut old = serde_json::to_value(before).expect("original dataset");
    let mut new = serde_json::to_value(after).expect("published dataset");
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
        "tilesets",
        "field_moves",
    ] {
        old.as_object_mut().expect("dataset").remove(field);
        new.as_object_mut().expect("dataset").remove(field);
    }
    assert_eq!(
        old, new,
        "all unaffected game/catalog/title/encounter datasets"
    );
    let mut field_moves = after.field_moves.clone();
    field_moves.cut.replacements.remove("mn_detroit_lakes");
    field_moves
        .whirlpool
        .replacements
        .remove("mn_detroit_lakes");
    assert_eq!(
        field_moves, before.field_moves,
        "all original field-move behavior"
    );
    for (name, script) in &before.map_scripts {
        assert_eq!(
            after.map_scripts.get(name),
            Some(script),
            "original split program {name}"
        );
    }
    for (name, actors) in &before.npcs {
        assert_eq!(
            after.npcs.get(name),
            Some(actors),
            "original split actors {name}"
        );
    }
    for (name, attrs) in &before.map_attributes {
        if !matches!(
            name.as_str(),
            "MnR15C00R00" | "MnDetroitLakesApproach" | "MnR19C00R01"
        ) {
            assert_eq!(after.map_attributes.get(name), Some(attrs));
        }
    }
    let changed_blocks = before.maps["MnR19C00R01"]
        .attributes
        .blocks_label
        .as_ref()
        .expect("R19 blocks");
    for (label, bytes) in &before.map_blocks {
        if label != changed_blocks {
            assert_eq!(after.map_blocks.get(label), Some(bytes));
        }
    }
    for (name, metadata) in &before.runtime_map_metadata {
        assert_eq!(after.runtime_map_metadata.get(name), Some(metadata));
    }
    for (name, spawn) in &before.runtime_spawn_points {
        assert_eq!(after.runtime_spawn_points.get(name), Some(spawn));
    }
    for (name, size) in &before.map_dimensions {
        assert_eq!(after.map_dimensions.get(name), Some(size));
    }
    for (name, definition) in &before.tilesets {
        assert_eq!(after.tilesets.get(name), Some(definition));
    }
    for (name, stock) in &before.marts.0 {
        assert_eq!(after.marts.0.get(name), Some(stock));
    }
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
        after.pokegear_landmarks.landmarks[..before.pokegear_landmarks.landmarks.len()],
        before.pokegear_landmarks.landmarks
    );
    for (name, area) in &before.pokegear_landmarks.map_to_landmark {
        assert_eq!(
            after.pokegear_landmarks.map_to_landmark.get(name),
            Some(area)
        );
    }
    assert_eq!(
        after.runtime_spawn_points.len(),
        before.runtime_spawn_points.len() + 2
    );
    let clinic = after
        .runtime_spawn_points
        .values()
        .find(|s| s.map_name == "MnDetroitLakesPokecenter1F")
        .expect("native recovery");
    assert_eq!((clinic.tile_x, clinic.tile_y), (8, 13));
    for name in [
        "MnDetroitLakesPokecenter1F",
        "MnDetroitLakesMart",
        "MnDetroitLakesStation",
        "MnDetroitLakesCanoeCabin",
        "MnDetroitLakesRangerCabin",
    ] {
        let map = &after.maps[name];
        assert!(!map.objects.is_empty(), "inhabited {name}");
        assert_eq!(map.events.warps.len(), 3);
        assert_eq!(
            after
                .assemble_map_module_from_compiled_payloads(name)
                .expect("strict split reconstruction"),
            *map
        );
    }
    let raw: Value = serde_json::from_str(&town).expect("town JSON");
    for mutation in 0..5 {
        let mut wrong = raw.clone();
        match mutation {
            0 => {
                wrong["interiors"].as_array_mut().expect("rooms").pop();
            }
            1 => wrong["doors"][0]["tile"] = json!([10, 9]),
            2 => wrong["interiors"][0]["residents"][0]["counter_tile"] = json!([7, 5]),
            3 => wrong["interiors"][0]["residents"][0]["tile"] = json!([0, 0]),
            _ => wrong["quest_contract"]["requires"] = json!(["BADGE_6"]),
        }
        assert!(
            build_detroit_lakes_modpack(&base, &wrong.to_string(), &layout, &station).is_err(),
            "reject unsafe source mutation {mutation}"
        );
    }
    let mut open: Value = serde_json::from_str(&station).expect("station");
    open["residents"][0]["tile"] = json!([14, 11]);
    assert!(
        build_detroit_lakes_modpack(&base, &town, &layout, &open.to_string()).is_err(),
        "conductor cannot leave an open train platform"
    );
    assert!(
        build_detroit_lakes_modpack(&published, &town, &layout, &station).is_err(),
        "duplicate publication"
    );
}
