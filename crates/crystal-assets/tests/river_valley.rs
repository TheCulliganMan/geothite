//! Compact second-badge river-route publication retains the source world and other routes.
use crystal_assets::{
    build_prairie_farm_modpack, build_river_valley_modpack, read_verified_compiled_game_pack,
};
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;

#[test]
fn river_valley_composes_with_prairie_farm_and_preserves_source_content() {
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
        build_prairie_farm_modpack(&source, &read("MN_R02.layout.json"), &read("MN_R02.json"))
            .expect("existing compact route");
    let layout = read("MN_R03.layout.json");
    let dossier = read("MN_R03.json");
    let result =
        build_river_valley_modpack(&base, &layout, &dossier).expect("River Valley overlay");
    assert_eq!(result.data().maps.len(), base.data().maps.len() + 3);
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
    let outside = &result.data().maps["MnR03C00R00"];
    assert_eq!(
        (outside.attributes.width, outside.attributes.height),
        (24, 24)
    );
    assert_eq!(outside.blocks.len(), 576);
    assert_eq!(outside.trainer_scripts.len(), 3);
    assert_eq!(outside.objects.len(), 5);
    assert_eq!(outside.events.bg_events.len(), 4);
    assert_eq!(outside.events.warps.len(), 2);
    assert_eq!(outside.attributes.connections.len(), 1);
    let lower = &result.data().maps["MnR03C00R01"];
    assert_eq!((lower.attributes.width, lower.attributes.height), (24, 8));
    assert_eq!(lower.blocks.len(), 192);
    assert_eq!(lower.objects.len(), 4);
    assert_eq!(lower.trainer_scripts.len(), 1);
    assert_eq!(lower.events.bg_events.len(), 1);
    assert_eq!(lower.attributes.connections.len(), 1);
    assert!(
        result.data().maps["MnR03Cave1F"]
            .events
            .warps
            .iter()
            .all(|w| w.target_warp_id == 2)
    );
    assert!(build_river_valley_modpack(&result, &layout, &dossier).is_err());
    let mut bad: Value = serde_json::from_str(&dossier).expect("dossier");
    bad["required_flags"] = json!(["BADGE_3"]);
    assert!(
        build_river_valley_modpack(&base, &layout, &bad.to_string()).is_err(),
        "second-badge gate changed"
    );
    let mut bad: Value = serde_json::from_str(&layout).expect("layout");
    bad["layers"].as_array_mut().expect("layers").push(json!({"kind":"rect","name":"Invalid dry grass bypass","material":"floor","bounds":[13,17,17,20]}));
    assert!(
        build_river_valley_modpack(&base, &bad.to_string(), &dossier).is_err(),
        "shared grass removed"
    );
    let mut bad: Value = serde_json::from_str(&layout).expect("layout");
    bad["layers"].as_array_mut().expect("layers").push(json!({"kind":"rect","name":"Invalid decorative ledge","material":"floor","bounds":[18,20,20,21]}));
    assert!(
        build_river_valley_modpack(&base, &bad.to_string(), &dossier).is_err(),
        "one-way ledge replaced with ordinary floor"
    );
    let mut bad: Value = serde_json::from_str(&dossier).expect("dossier");
    bad["one_way_ledges"][0]["landing"] = json!([37, 41]);
    assert!(
        build_river_valley_modpack(&base, &layout, &bad.to_string()).is_err(),
        "ledge landing placed in its wall face"
    );
    for bounds in [[9, 13, 13, 16], [14, 26, 18, 29]] {
        let mut bad: Value = serde_json::from_str(&layout).expect("layout");
        bad["layers"].as_array_mut().expect("layers").push(
            json!({"kind":"rect","name":"Removed bridge span","material":"water","bounds":bounds}),
        );
        assert!(
            build_river_valley_modpack(&base, &bad.to_string(), &dossier).is_err(),
            "missing footbridge accepted"
        );
    }
    let mut bad: Value = serde_json::from_str(&dossier).expect("dossier");
    bad["one_way_ledges"][0]["return_passage_core"] = json!([0, 0, 1, 1]);
    assert!(
        build_river_valley_modpack(&base, &layout, &bad.to_string()).is_err(),
        "unrelated return passage accepted"
    );
    let mut bad: Value = serde_json::from_str(&dossier).expect("dossier");
    bad["one_way_ledges"][0]["minimum_return_steps"] = json!(500);
    assert!(
        build_river_valley_modpack(&base, &layout, &bad.to_string()).is_err(),
        "return shortcut no longer meets its walking cost"
    );
}
