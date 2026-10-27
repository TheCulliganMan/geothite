//! Compact route publication and meaningful traversal failures.
use std::fs;
use std::path::PathBuf;

use crystal_assets::{build_central_lakes_modpack, read_verified_compiled_game_pack};
use serde_json::{Value, json};

#[test]
fn central_lakes_keeps_one_exterior_one_cabin_and_required_grass() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository");
    let base = read_verified_compiled_game_pack(root.join("content-packs/text-tui.crystalpack"))
        .expect("external source pack");
    let layout = fs::read_to_string(root.join("planning/minnesota/routes/MN_R15.layout.json"))
        .expect("authored terrain");
    let dossier = fs::read_to_string(root.join("planning/minnesota/routes/MN_R15.json"))
        .expect("authored events");
    let result = build_central_lakes_modpack(&base, &layout, &dossier).expect("compact overlay");
    assert_eq!(result.data().maps.len(), base.data().maps.len() + 2);
    let exterior = &result.data().maps["MnR15C00R00"];
    assert_eq!(
        (exterior.attributes.width, exterior.attributes.height),
        (32, 18)
    );
    assert_eq!(exterior.attributes.border_block, 0x0f);
    assert_eq!(exterior.attributes.connection_flags, None);
    assert!(exterior.attributes.connections.is_empty());
    assert_eq!(exterior.objects.len(), 6);
    assert_eq!(exterior.trainer_scripts.len(), 2);
    assert_eq!(exterior.events.warps.len(), 2);
    assert_eq!(
        (exterior.events.warps[1].x, exterior.events.warps[1].y),
        (53, 12)
    );
    assert!(
        result.data().maps["MnR15LakeCabin"]
            .events
            .warps
            .iter()
            .all(|w| w.target_warp_id == 2)
    );
    assert_eq!(exterior.events.bg_events.len(), 3);
    assert_eq!(result.data().maps["MnR15LakeCabin"].events.warps.len(), 2);
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
    assert_eq!(
        result.data().runtime_spawn_points,
        base.data().runtime_spawn_points
    );
    assert_eq!(result.compiled_audio(), base.compiled_audio());
    assert_eq!(
        result.audio_manifest().expect("audio"),
        base.audio_manifest().expect("audio")
    );
    assert!(build_central_lakes_modpack(&result, &layout, &dossier).is_err());

    // A physically connected dry bypass must fail even while the declared
    // grass itself remains intact. Existence of encounter tiles is insufficient.
    let mut bypass: Value = serde_json::from_str(&layout).expect("layout");
    bypass["layers"]
        .as_array_mut()
        .expect("layers")
        .push(json!({
            "kind":"path", "name":"Invalid dry bypass around required grass",
            "material":"floor", "width_metatiles":1,
            "points":[[2,12],[2,10],[6,10],[6,12]]
        }));
    let error = build_central_lakes_modpack(&base, &bypass.to_string(), &dossier)
        .expect_err("dry bypass must fail");
    assert!(error.to_string().contains("bypassed"), "{error:#}");
    let mut blocked: Value = serde_json::from_str(&layout).expect("layout");
    blocked["layers"]
        .as_array_mut()
        .expect("layers")
        .push(json!({
            "kind":"rect", "name":"Invalid wall across bridge", "material":"forest",
            "bounds":[15,14,18,15]
        }));
    assert!(build_central_lakes_modpack(&base, &blocked.to_string(), &dossier).is_err());
    let mut bad_gate: Value = serde_json::from_str(&dossier).expect("dossier");
    bad_gate["required_flags"] = json!(["BADGE_7"]);
    assert!(build_central_lakes_modpack(&base, &layout, &bad_gate.to_string()).is_err());
    let mut bad: Value = serde_json::from_str(&dossier).expect("dossier");
    bad["one_way_ledges"][0]["return_passage_core"] = json!([0, 0, 1, 1]);
    assert!(
        build_central_lakes_modpack(&base, &layout, &bad.to_string()).is_err(),
        "unrelated return passage accepted"
    );
    let mut bad: Value = serde_json::from_str(&dossier).expect("dossier");
    bad["one_way_ledges"][0]["minimum_return_steps"] = json!(500);
    assert!(
        build_central_lakes_modpack(&base, &layout, &bad.to_string()).is_err(),
        "return shortcut no longer meets its walking cost"
    );
}
