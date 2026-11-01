//! The park overlay preserves the source game and rejects free bypasses.
use std::fs;
use std::path::PathBuf;

use crystal_assets::{
    build_river_valley_modpack, build_twin_cities_modpack, read_verified_compiled_game_pack,
};
use serde_json::{Value, json};

#[test]
fn twin_cities_composes_with_river_valley_and_keeps_native_park_art_and_choices() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository");
    let read = |file: &str| {
        fs::read_to_string(root.join("planning/minnesota/routes").join(file))
            .expect("authored input")
    };
    let source = read_verified_compiled_game_pack(root.join("content-packs/text-tui.crystalpack"))
        .expect("external source");
    let base =
        build_river_valley_modpack(&source, &read("MN_R03.layout.json"), &read("MN_R03.json"))
            .expect("existing route");
    let layout = read("MN_R04.layout.json");
    let dossier = read("MN_R04.json");
    let result = build_twin_cities_modpack(&base, &layout, &dossier).expect("native park");
    assert_eq!(result.data().maps.len(), base.data().maps.len() + 1);
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
        assert_eq!(result.data().maps.get(name), Some(map), "changed {name}");
    }
    for (name, bytes) in base.runtime_files() {
        assert_eq!(
            result.runtime_files().get(name),
            Some(bytes),
            "changed {name}"
        );
    }
    for (name, tileset) in &base.data().tilesets {
        assert_eq!(
            result.data().tilesets.get(name),
            Some(tileset),
            "changed {name}"
        );
    }
    let native = &result.data().maps["MnR04C00R00"];
    assert_eq!(
        (native.attributes.width, native.attributes.height),
        (28, 18)
    );
    assert_eq!(native.blocks.len(), 504);
    assert_eq!(native.objects.len(), 7);
    assert_eq!(native.trainer_scripts.len(), 2);
    assert_eq!(native.events.bg_events.len(), 5);
    assert!(native.events.warps.is_empty());
    assert!(native.attributes.connections.is_empty());
    assert_eq!(native.attributes.border_block, 0);
    let files = result.runtime_files();
    let original = &base.runtime_files()["data/tilesets/park_metatiles.bin"];
    let derived = &files["data/tilesets/mn_river_park_metatiles.bin"];
    assert_eq!(&derived[..original.len()], original);
    assert_eq!(derived.len(), 68 * 16);
    for (id, rows) in [
        (0x40, [(0x19, 2), (0x19, 3), (0x01, 2), (0x01, 3)]),
        (0x41, [(0x08, 2); 4]),
        (0x42, [(0x09, 2); 4]),
        (0x43, [(0x01, 0), (0x01, 1), (0x19, 2), (0x19, 3)]),
    ] {
        for (row, (block, source_row)) in rows.into_iter().enumerate() {
            assert_eq!(
                &derived[id * 16 + row * 4..id * 16 + row * 4 + 4],
                &original[block * 16 + source_row * 4..block * 16 + source_row * 4 + 4]
            );
        }
    }
    assert_eq!(
        files["gfx/tilesets/mn_river_park.2bpp"],
        files["gfx/tilesets/park.2bpp"]
    );
    assert_eq!(
        files["gfx/tilesets/mn_river_park.png"],
        files["gfx/tilesets/park.png"]
    );
    assert!(build_twin_cities_modpack(&result, &layout, &dossier).is_err());
    let mut bad: Value = serde_json::from_str(&dossier).expect("dossier");
    bad["required_flags"] = json!(["BADGE_2"]);
    assert!(
        build_twin_cities_modpack(&base, &layout, &bad.to_string()).is_err(),
        "changed third-badge condition"
    );
    let mut bad: Value = serde_json::from_str(&dossier).expect("dossier");
    bad["trainers"][0]["sight_range"] = json!(0);
    assert!(
        build_twin_cities_modpack(&base, &layout, &bad.to_string()).is_err(),
        "uncovered short lane"
    );
    for bounds in [[8, 4, 10, 7], [8, 8, 9, 9]] {
        let mut bad: Value = serde_json::from_str(&layout).expect("layout");
        bad["layers"].as_array_mut().expect("layers").push(json!({"kind":"rect","name":"Invalid free dry bypass","bounds":bounds,"material":"floor"}));
        assert!(
            build_twin_cities_modpack(&base, &bad.to_string(), &dossier).is_err(),
            "battle and grass both bypassable"
        );
    }
    let mut bad: Value = serde_json::from_str(&layout).expect("layout");
    bad["layers"].as_array_mut().expect("layers").push(json!({"kind":"rect","name":"Removed full bridge","bounds":[12,8,17,11],"material":"water"}));
    assert!(
        build_twin_cities_modpack(&base, &bad.to_string(), &dossier).is_err(),
        "river crossing lost"
    );
}
