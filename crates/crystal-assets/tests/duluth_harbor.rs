//! External current-parent publication and rejection regressions.

use crystal_assets::{
    read_verified_compiled_game_pack, revise_duluth_harbor, revise_duluth_shoreline,
};

#[test]
#[ignore = "requires CRYSTAL_DULUTH_SHORE_PARENT and checked-in layout sources"]
fn shore_rocks_preserve_events_water_art_and_reject_source_drift() {
    let parent = std::env::var_os("CRYSTAL_DULUTH_SHORE_PARENT").expect("current parent");
    let base = read_verified_compiled_game_pack(parent).expect("verified parent");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let before = std::fs::read_to_string(
        root.join("planning/minnesota/revisions/duluth-harbor-platforms-v01/after.layout.json"),
    )
    .expect("current Duluth source");
    let after = std::fs::read_to_string(root.join(
        "planning/minnesota/revisions/shore-rocks-open-routes-v01/MnDuluth.after.layout.json",
    ))
    .expect("rocky shoreline source");
    let result = revise_duluth_shoreline(&base, &before, &after).expect("rocky shore revision");
    let old = &base.data().maps["MnDuluth"];
    let city = &result.data().maps["MnDuluth"];
    let mut unchanged = city.clone();
    unchanged.blocks.clone_from(&old.blocks);
    assert_eq!(&unchanged, old, "all existing city events retained");
    for (index, &block) in old.blocks.iter().enumerate() {
        if block == 0x35 || index % 64 == 0 || index % 64 == 63 || index / 64 == 45 {
            assert_eq!(city.blocks[index], block, "harbor water and outer buoys");
        }
    }
    for (name, map) in &base.data().maps {
        if name != "MnDuluth" {
            assert_eq!(result.data().maps.get(name), Some(map), "unchanged {name}");
        }
    }
    assert_eq!(result.runtime_files(), base.runtime_files());
    assert_eq!(result.data().audio, base.data().audio);
    assert!(revise_duluth_shoreline(&result, &before, &after).is_err());
    let mut drift: serde_json::Value = serde_json::from_str(&after).expect("layout");
    drift["derived_blocks"][0]["source_block"] = "00".into();
    assert!(revise_duluth_shoreline(&base, &before, &drift.to_string()).is_err());
}

#[test]
#[ignore = "requires CRYSTAL_DULUTH_HARBOR_PARENT and checked-in layout sources"]
fn harbor_preserves_other_maps_audio_art_and_rejects_unrelated_terrain() {
    let parent = std::env::var_os("CRYSTAL_DULUTH_HARBOR_PARENT").expect("current parent");
    let base = read_verified_compiled_game_pack(parent).expect("verified parent");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let before = std::fs::read_to_string(
        root.join("planning/minnesota/revisions/duluth-garden-terrace-v02/after.layout.json"),
    )
    .expect("current terrain preimage");
    let after = std::fs::read_to_string(
        root.join("planning/minnesota/revisions/duluth-harbor-platforms-v01/after.layout.json"),
    )
    .expect("harbor layout");
    let result = revise_duluth_harbor(&base, &before, &after).expect("bounded harbor revision");
    for (name, map) in &base.data().maps {
        if name != "MnDuluth" {
            assert_eq!(
                result.data().maps.get(name),
                Some(map),
                "unchanged map {name}"
            );
        }
    }
    assert_eq!(result.runtime_files(), base.runtime_files());
    assert_eq!(result.data().audio, base.data().audio);
    assert_eq!(result.data().pokemon, base.data().pokemon);
    assert_eq!(result.data().trainers, base.data().trainers);
    assert!(revise_duluth_harbor(&result, &before, &after).is_err());
    let mut drift: serde_json::Value = serde_json::from_str(&after).expect("layout JSON");
    drift["layers"]
        .as_array_mut()
        .expect("layers")
        .push(serde_json::json!({
            "name":"unrelated upper street drift", "kind":"point", "position":[32,18], "block":"35"
        }));
    assert!(revise_duluth_harbor(&base, &before, &drift.to_string()).is_err());
}
