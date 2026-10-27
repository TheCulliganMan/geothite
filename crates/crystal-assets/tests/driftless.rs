//! Compact starter-route publication retains the source world and other routes.
use crystal_assets::{
    build_central_lakes_modpack, build_driftless_modpack, read_verified_compiled_game_pack,
};
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;

#[test]
fn driftless_composes_with_central_lakes_and_preserves_source_content() {
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
        build_central_lakes_modpack(&source, &read("MN_R15.layout.json"), &read("MN_R15.json"))
            .expect("existing compact route");
    let layout = read("MN_R01.layout.json");
    let dossier = read("MN_R01.json");
    let result = build_driftless_modpack(&base, &layout, &dossier).expect("Driftless overlay");
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
    let outside = &result.data().maps["MnR01C00R00"];
    assert_eq!(
        (outside.attributes.width, outside.attributes.height),
        (32, 20)
    );
    assert_eq!(outside.blocks.len(), 640);
    assert_eq!(outside.trainer_scripts.len(), 3);
    assert_eq!(outside.objects.len(), 7);
    assert_eq!(outside.events.bg_events.len(), 5);
    assert_eq!(outside.events.warps.len(), 4);
    assert!(outside.attributes.connections.is_empty());
    assert_eq!(outside.attributes.connection_flags, None);
    assert!(
        result.data().maps["MnR01Cave1F"]
            .events
            .warps
            .iter()
            .all(|w| w.target_warp_id == 2)
    );
    assert!(
        result.data().maps["MnR01Farmhouse"]
            .events
            .warps
            .iter()
            .all(|w| w.target_warp_id == 4)
    );
    assert!(build_driftless_modpack(&result, &layout, &dossier).is_err());
    let mut bad: Value = serde_json::from_str(&dossier).expect("dossier");
    bad["required_flags"] = json!(["BADGE_1"]);
    assert!(
        build_driftless_modpack(&base, &layout, &bad.to_string()).is_err(),
        "starter gate changed"
    );
    let mut bad: Value = serde_json::from_str(&layout).expect("layout");
    bad["layers"].as_array_mut().expect("layers").push(json!({"kind":"rect","name":"Invalid dry grass bypass","material":"floor","bounds":[27,11,29,14]}));
    assert!(
        build_driftless_modpack(&base, &bad.to_string(), &dossier).is_err(),
        "shared grass removed"
    );
    let mut bad: Value = serde_json::from_str(&layout).expect("layout");
    bad["layers"].as_array_mut().expect("layers").push(json!({"kind":"rect","name":"Invalid decorative ledge","material":"floor","bounds":[13,8,15,9]}));
    assert!(
        build_driftless_modpack(&base, &bad.to_string(), &dossier).is_err(),
        "one-way ledge replaced with ordinary floor"
    );
    let mut bad: Value = serde_json::from_str(&dossier).expect("dossier");
    bad["one_way_ledges"][0]["landing"] = json!([29, 17]);
    assert!(
        build_driftless_modpack(&base, &layout, &bad.to_string()).is_err(),
        "ledge landing placed in its wall face"
    );
    let mut bad: Value = serde_json::from_str(&dossier).expect("dossier");
    bad["one_way_ledges"][0]["return_passage_core"] = json!([0, 0, 1, 1]);
    assert!(
        build_driftless_modpack(&base, &layout, &bad.to_string()).is_err(),
        "unrelated return passage accepted"
    );
    let mut bad: Value = serde_json::from_str(&dossier).expect("dossier");
    bad["one_way_ledges"][0]["minimum_return_steps"] = json!(500);
    assert!(
        build_driftless_modpack(&base, &layout, &bad.to_string()).is_err(),
        "return shortcut no longer meets its walking cost"
    );
    // A no-Cut route must still inventory every native canopy quadrant.
    // Restore the actual decorative tree caught by the source audit.
    let mut decorative: Value = serde_json::from_str(&layout).expect("layout");
    decorative["layers"]
        .as_array_mut()
        .expect("layers")
        .push(json!({
            "kind": "point", "name": "Invalid incidental Cut canopy",
            "position": [14,10], "block": "63"
        }));
    let error = build_driftless_modpack(&base, &decorative.to_string(), &dossier)
        .expect_err("undeclared Cut on a route without declared gates");
    assert!(
        error.to_string().contains("undeclared decorative Cut"),
        "{error:#}"
    );
}
