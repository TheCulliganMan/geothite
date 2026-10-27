//! A town overlay must not replace the user's source catalogs, maps or audio.
use std::path::PathBuf;

use crystal_assets::{
    build_grand_marais_modpack, build_grand_marais_region_extension,
    read_verified_compiled_game_pack,
};

#[test]
fn grand_marais_overlay_preserves_base_content_and_audio() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository");
    let base = read_verified_compiled_game_pack(root.join("content-packs/text-tui.crystalpack"))
        .expect("external source pack");
    let design =
        std::fs::read_to_string(root.join("planning/minnesota/playable/grand-marais.json"))
            .expect("authored town");
    let town = build_grand_marais_modpack(&base, &design).expect("typed town overlay");
    assert_eq!(town.data().maps.len(), base.data().maps.len() + 7);
    for (name, module) in &base.data().maps {
        assert_eq!(
            town.data().maps.get(name),
            Some(module),
            "base map {name} changed"
        );
    }
    assert_eq!(town.runtime_files(), base.runtime_files());
    assert_eq!(town.compiled_audio(), base.compiled_audio());
    assert_eq!(
        town.audio_manifest().expect("town audio manifest"),
        base.audio_manifest().expect("source audio manifest")
    );
    assert_eq!(town.data().audio, base.data().audio);
    assert_eq!(town.data().pokemon, base.data().pokemon);
    assert_eq!(town.data().trainers, base.data().trainers);
    assert_eq!(town.data().marts, base.data().marts);
    for (key, spawn) in &base.data().runtime_spawn_points {
        if key != "0" {
            assert_eq!(town.data().runtime_spawn_points.get(key), Some(spawn));
        }
    }
    assert_eq!(town.data().runtime_spawn_points.len(), 28);
    assert_eq!(
        town.data().runtime_spawn_points["0"].map_name,
        "MnGrandMaraisFieldStation"
    );
    let regional = build_grand_marais_region_extension(&base, &design)
        .expect("town extension retaining original HOME");
    assert_eq!(regional.data().maps, town.data().maps);
    assert_eq!(regional.data().wild_encounters, town.data().wild_encounters);
    assert_eq!(
        regional.data().runtime_spawn_points,
        base.data().runtime_spawn_points,
        "regional composition must retain every original HOME/recovery destination"
    );
    assert_eq!(regional.runtime_files(), base.runtime_files());
    assert_eq!(regional.compiled_audio(), base.compiled_audio());
    assert_eq!(
        regional.audio_manifest().expect("regional audio manifest"),
        base.audio_manifest().expect("source audio manifest")
    );
    assert!(
        build_grand_marais_region_extension(&regional, &design).is_err(),
        "duplicate regional town installation accepted"
    );
    assert!(
        build_grand_marais_modpack(&town, &design).is_err(),
        "duplicate installation accepted"
    );
    let mut misplaced_exit: serde_json::Value =
        serde_json::from_str(&design).expect("town design JSON");
    misplaced_exit["interiors"][0]["exits"][0] = serde_json::json!([0, 0]);
    assert!(
        build_grand_marais_modpack(&base, &misplaced_exit.to_string()).is_err(),
        "accepted an exit on the field station wall"
    );
    let mut missing_dialogue: serde_json::Value =
        serde_json::from_str(&design).expect("town design JSON");
    missing_dialogue["dialogue"]["MnGrandMaraisFieldStation"]
        .as_object_mut()
        .expect("field station dialogue")
        .remove("MnGrandMaraisWelcomeText");
    assert!(
        build_grand_marais_modpack(&base, &missing_dialogue.to_string()).is_err(),
        "accepted a starter scene without its dialogue"
    );
}
