//! Native publication must preserve the source world and its existing overlays.
use std::fs;
use std::path::PathBuf;

use crystal_assets::{
    build_league_approach_modpack, build_pine_country_modpack, build_river_bluffs_modpack,
    read_verified_compiled_game_pack,
};
use serde_json::{Value, json};

#[test]
fn league_approach_composes_three_clipped_maps_and_shelter_without_replacing_content() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository");
    let base = read_verified_compiled_game_pack(root.join("content-packs/text-tui.crystalpack"))
        .expect("external source pack");
    let read = |name: &str| {
        fs::read_to_string(root.join("planning/minnesota/routes").join(name))
            .expect("authored route JSON")
    };
    let bluffs =
        build_river_bluffs_modpack(&base, &read("MN_R05.layout.json"), &read("MN_R05.json"))
            .expect("River Bluffs overlay");
    let forest =
        build_pine_country_modpack(&bluffs, &read("MN_R06.layout.json"), &read("MN_R06.json"))
            .expect("Pine Country overlay");
    let layout = read("MN_R13.layout.json");
    let dossier = read("MN_R13.json");
    let result = build_league_approach_modpack(&forest, &layout, &dossier)
        .expect("complete League Approach overlay");
    assert_eq!(result.data().maps.len(), forest.data().maps.len() + 4);
    assert_eq!(
        result.data().runtime_spawn_points,
        forest.data().runtime_spawn_points
    );
    assert_eq!(result.compiled_audio(), forest.compiled_audio());
    assert_eq!(
        result.audio_manifest().expect("audio"),
        forest.audio_manifest().expect("source audio")
    );
    for (name, module) in &forest.data().maps {
        assert_eq!(
            result.data().maps.get(name),
            Some(module),
            "changed map {name}"
        );
    }
    for (name, bytes) in forest.runtime_files() {
        assert_eq!(
            result.runtime_files().get(name),
            Some(bytes),
            "changed source file {name}"
        );
    }
    assert_eq!(
        result.data().tilesets["johto"],
        base.data().tilesets["johto"]
    );
    let mut totals = [0, 0, 0, 0];
    for row in 0..3 {
        for col in 0..1 {
            let name = format!("MnR13C{col:02}R{row:02}");
            let module = &result.data().maps[&name];
            assert_eq!(
                (
                    module.attributes.width,
                    module.attributes.height,
                    module.attributes.tileset_name.as_str()
                ),
                (32, if row == 2 { 16 } else { 24 }, "johto")
            );
            assert_eq!(
                result
                    .data()
                    .assemble_map_module_from_compiled_payloads(&name)
                    .expect("split payload"),
                *module
            );
            assert!(result.data().wild_encounters.contains_key(&name));
            totals[0] += module.trainer_scripts.len();
            totals[1] += module.script_field_pickups.len();
            totals[2] += module.events.bg_events.len();
            totals[3] += module.objects.len();
            for connection in &module.attributes.connections {
                let opposite = match connection.direction.as_str() {
                    "north" => "south",
                    "south" => "north",
                    "west" => "east",
                    "east" => "west",
                    _ => panic!("cardinal connection required"),
                };
                assert!(
                    result.data().maps[&connection.target_map]
                        .attributes
                        .connections
                        .iter()
                        .any(|back| back.target_map == name
                            && back.direction == opposite
                            && back.offset == 0)
                );
            }
        }
    }
    assert_eq!(totals, [7, 4, 6, 17]);
    let room = &result.data().maps["MnR13Cave1F"];
    assert_eq!(
        (
            room.attributes.width,
            room.attributes.height,
            room.attributes.border_block
        ),
        (10, 8, 1)
    );
    assert_eq!(
        (
            room.objects.len(),
            room.events.warps.len(),
            room.script_field_pickups.len()
        ),
        (2, 1, 1)
    );
    assert!(
        build_league_approach_modpack(&result, &layout, &dossier).is_err(),
        "duplicate publication accepted"
    );
    let mut bad: Value = serde_json::from_str(&dossier).expect("dossier");
    bad["required_flags"] = json!([
        "BADGE_1", "BADGE_2", "BADGE_3", "BADGE_4", "BADGE_5", "BADGE_6", "BADGE_7", "BADGE_8"
    ]);
    assert!(
        build_league_approach_modpack(&forest, &layout, &bad.to_string()).is_err(),
        "Q08 survey gate lost"
    );
    bad = serde_json::from_str(&dossier).expect("dossier");
    bad["interiors"][0]["exits"][0] = json!([0, 0]);
    assert!(
        build_league_approach_modpack(&forest, &layout, &bad.to_string()).is_err(),
        "exit on solid wall accepted"
    );
}
