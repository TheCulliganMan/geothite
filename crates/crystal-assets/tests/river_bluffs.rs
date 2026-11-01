//! Two-ended cave publication retains old routes, source terrain and audio.
use std::fs;
use std::path::PathBuf;

use crystal_assets::{
    build_river_bluffs_modpack, build_twin_cities_modpack, read_verified_compiled_game_pack,
};
use serde_json::{Value, json};

#[test]
fn river_bluffs_composes_and_pairs_each_cave_exit_with_its_own_exterior() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository");
    let read = |name: &str| {
        fs::read_to_string(root.join("planning/minnesota/routes").join(name))
            .expect("authored input")
    };
    let source = read_verified_compiled_game_pack(root.join("content-packs/text-tui.crystalpack"))
        .expect("external source");
    let base =
        build_twin_cities_modpack(&source, &read("MN_R04.layout.json"), &read("MN_R04.json"))
            .expect("prior route");
    let layout = read("MN_R05.layout.json");
    let dossier = read("MN_R05.json");
    let result = build_river_bluffs_modpack(&base, &layout, &dossier).expect("native ridges");
    assert_eq!(result.data().maps.len(), base.data().maps.len() + 4);
    assert_eq!(result.compiled_audio(), base.compiled_audio());
    assert_eq!(
        result.audio_manifest().expect("audio"),
        base.audio_manifest().expect("audio")
    );
    assert_eq!(
        result.data().runtime_spawn_points,
        base.data().runtime_spawn_points
    );
    for (name, map) in &base.data().maps {
        assert_eq!(
            result.data().maps.get(name),
            Some(map),
            "changed map {name}"
        );
    }
    for (name, bytes) in base.runtime_files() {
        assert_eq!(
            result.runtime_files().get(name),
            Some(bytes),
            "changed source {name}"
        );
    }
    for (name, tileset) in &base.data().tilesets {
        assert_eq!(
            result.data().tilesets.get(name),
            Some(tileset),
            "changed source tileset {name}"
        );
    }
    for (name, height, connections, objects, trainers, signs) in [
        ("MnR05C00R00", 24, 1, 5, 3, 2),
        ("MnR05C00R01", 24, 2, 5, 2, 3),
        ("MnR05C00R02", 8, 1, 1, 0, 1),
    ] {
        let map = &result.data().maps[name];
        assert_eq!((map.attributes.width, map.attributes.height), (32, height));
        assert_eq!(map.attributes.connections.len(), connections);
        assert_eq!(map.objects.len(), objects);
        assert_eq!(map.trainer_scripts.len(), trainers);
        assert_eq!(map.events.bg_events.len(), signs);
    }
    let upper = &result.data().maps["MnR05C00R00"];
    let lower = &result.data().maps["MnR05C00R01"];
    let cave = &result.data().maps["MnR05RockCut"];
    assert_eq!(lower.events.warps[0].target_warp_id, 1);
    assert_eq!(upper.events.warps[0].target_warp_id, 2);
    assert_eq!(cave.events.warps.len(), 2);
    assert_eq!(
        Some(cave.events.warps[0].target_map_constant.as_str()),
        lower.attributes.map_constant.as_deref()
    );
    assert_eq!(
        Some(cave.events.warps[1].target_map_constant.as_str()),
        upper.attributes.map_constant.as_deref()
    );
    assert!(cave.events.warps.iter().all(|w| w.target_warp_id == 2));
    assert_eq!(cave.attributes.border_block, 1);
    assert!(
        cave.script_runtime_commands.iter().any(|command| {
            command.source_script == "MN_R05_CUT_WALKER"
                && command.command == "special"
                && command.args == ["HealParty"]
        }),
        "rock-cut rest must use the compiled native healing routine"
    );
    assert!(
        cave.script_object_commands.iter().any(|command| {
            command.source_script == "MN_R05_CUT_WALKER" && command.command == "faceplayer"
        }),
        "rest interaction must compile its native facing command"
    );
    assert!(build_river_bluffs_modpack(&result, &layout, &dossier).is_err());
    let mut bad: Value = serde_json::from_str(&dossier).expect("dossier");
    bad["required_flags"] = json!(["BADGE_3"]);
    assert!(
        build_river_bluffs_modpack(&base, &layout, &bad.to_string()).is_err(),
        "fourth-badge condition removed"
    );
    for bounds in [[10, 49, 13, 51], [18, 20, 20, 21], [18, 38, 20, 39]] {
        let mut bad: Value = serde_json::from_str(&layout).expect("layout");
        bad["layers"].as_array_mut().expect("layers").push(json!({"kind":"rect","name":"Removed traversal feature","material":"floor","bounds":bounds}));
        assert!(
            build_river_bluffs_modpack(&base, &bad.to_string(), &dossier).is_err(),
            "removed grass or one-way ledge"
        );
    }
    for change in 0..4 {
        let mut bad: Value = serde_json::from_str(&dossier).expect("dossier");
        match change {
            0 => bad["interiors"][0]["exit_doors"] = json!([[30, 27], [16, 77]]),
            1 => bad["doors"][1]["target_warp"] = json!(1),
            2 => bad["interiors"][0]["exits"][1] = json!([5, 17]),
            _ => bad["interiors"][0]["exit_doors"][1] = json!([16, 77]),
        }
        assert!(
            build_river_bluffs_modpack(&base, &layout, &bad.to_string()).is_err(),
            "ambiguous cave pairing {change}"
        );
    }
    let mut bad: Value = serde_json::from_str(&dossier).expect("dossier");
    bad["interiors"][0]["residents"][0]["service"] = json!("invented_service");
    assert!(build_river_bluffs_modpack(&base, &layout, &bad.to_string()).is_err());
}
