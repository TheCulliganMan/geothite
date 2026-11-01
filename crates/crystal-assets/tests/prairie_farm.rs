//! Compact first-badge farm-route publication retains the source world and other routes.
use crystal_assets::{
    build_driftless_modpack, build_prairie_farm_modpack, read_verified_compiled_game_pack,
};
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;

#[test]
fn prairie_farm_composes_with_driftless_and_preserves_source_content() {
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
    let base = build_driftless_modpack(&source, &read("MN_R01.layout.json"), &read("MN_R01.json"))
        .expect("existing compact route");
    let layout = read("MN_R02.layout.json");
    let dossier = read("MN_R02.json");
    let result =
        build_prairie_farm_modpack(&base, &layout, &dossier).expect("Prairie Farm overlay");
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
    let outside = &result.data().maps["MnR02C00R00"];
    assert_eq!(
        (outside.attributes.width, outside.attributes.height),
        (30, 22)
    );
    assert_eq!(outside.blocks.len(), 660);
    assert_eq!(outside.trainer_scripts.len(), 3);
    assert_eq!(outside.objects.len(), 7);
    assert_eq!(outside.events.bg_events.len(), 5);
    assert_eq!(outside.events.warps.len(), 4);
    assert!(outside.attributes.connections.is_empty());
    assert_eq!(outside.attributes.connection_flags, None);
    assert!(
        result.data().maps["MnR02Farmhouse"]
            .events
            .warps
            .iter()
            .all(|w| w.target_warp_id == 2)
    );
    assert!(
        result.data().maps["MnR02ToolShed"]
            .events
            .warps
            .iter()
            .all(|w| w.target_warp_id == 4)
    );
    assert!(build_prairie_farm_modpack(&result, &layout, &dossier).is_err());
    let mut bad: Value = serde_json::from_str(&dossier).expect("dossier");
    bad["required_flags"] = json!(["BADGE_2"]);
    assert!(
        build_prairie_farm_modpack(&base, &layout, &bad.to_string()).is_err(),
        "first-badge gate changed"
    );
    let mut bad: Value = serde_json::from_str(&layout).expect("layout");
    bad["layers"].as_array_mut().expect("layers").push(json!({"kind":"rect","name":"Invalid dry grass bypass","material":"floor","bounds":[4,14,7,17]}));
    assert!(
        build_prairie_farm_modpack(&base, &bad.to_string(), &dossier).is_err(),
        "shared grass removed"
    );
    let mut bad: Value = serde_json::from_str(&layout).expect("layout");
    bad["layers"].as_array_mut().expect("layers").push(json!({"kind":"rect","name":"Invalid decorative ledge","material":"floor","bounds":[23,7,24,9]}));
    assert!(
        build_prairie_farm_modpack(&base, &bad.to_string(), &dossier).is_err(),
        "one-way ledge replaced with ordinary floor"
    );
    let mut bad: Value = serde_json::from_str(&dossier).expect("dossier");
    bad["one_way_ledges"][0]["landing"] = json!([46, 15]);
    assert!(
        build_prairie_farm_modpack(&base, &layout, &bad.to_string()).is_err(),
        "ledge landing placed in its wall face"
    );
    let mut bad: Value = serde_json::from_str(&dossier).expect("dossier");
    bad["one_way_ledges"][0]["return_passage_core"] = json!([0, 0, 1, 1]);
    assert!(
        build_prairie_farm_modpack(&base, &layout, &bad.to_string()).is_err(),
        "unrelated return passage accepted"
    );
    let mut bad: Value = serde_json::from_str(&dossier).expect("dossier");
    bad["one_way_ledges"][0]["minimum_return_steps"] = json!(500);
    assert!(
        build_prairie_farm_modpack(&base, &layout, &bad.to_string()).is_err(),
        "return shortcut no longer meets its walking cost"
    );
}
