//! Runtime-read external source checks; no game content is embedded in code.
use std::path::PathBuf;

use serde_json::Value;

use super::rebuild_north_shore_passing_width;

#[test]
#[ignore = "requires CRYSTAL_NORTH_SHORE_WIDTH_PARENT and authored passing-width sources"]
fn north_shore_passing_width_preserves_content_and_rejects_gate_bypasses() {
    let base = crate::read_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_NORTH_SHORE_WIDTH_PARENT").expect("completed current parent"),
    )
    .expect("verified parent");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../planning/minnesota/revisions/north-shore-passing-width-v01");
    let before = std::fs::read_to_string(root.join("before.json")).expect("before");
    let after = std::fs::read_to_string(root.join("after.json")).expect("after");
    let gates = std::fs::read_to_string(root.join("gate-coordinates.json")).expect("gates");
    let child =
        rebuild_north_shore_passing_width(&base, &before, &after, &gates).expect("reviewed width");
    assert_eq!(child.data.maps.len(), base.data.maps.len());
    assert_eq!(child.audio_manifest, base.audio_manifest);
    assert_eq!(child.data.trainers, base.data.trainers);
    assert_eq!(child.data.wild_encounters, base.data.wild_encounters);
    for (name, map) in &base.data.maps {
        if !name.starts_with("MnR11C") && !["MnDuluth", "MnGrandMarais"].contains(&name.as_str()) {
            assert_eq!(child.data.maps[name], *map, "unrelated map {name}");
        }
    }
    for (path, bytes) in &base.runtime_files {
        if ![
            "data/tilesets/mn_north_shore_metatiles.bin",
            "data/tilesets/mn_north_shore.json",
        ]
        .contains(&path.as_str())
        {
            assert_eq!(
                child.runtime_files[path], *bytes,
                "unrelated resource {path}"
            );
        }
    }
    for town in ["MnDuluth", "MnGrandMarais"] {
        let old = &base.data.maps[town];
        let new = &child.data.maps[town];
        assert_eq!(new.events.warps, old.events.warps);
        assert_eq!(new.scripts, old.scripts);
        assert_eq!(new.objects, old.objects);
        assert_eq!(new.attributes, old.attributes);
        let mut incomplete: Value = serde_json::from_str(&gates).expect("gates");
        incomplete[town]["after"]
            .as_array_mut()
            .expect("events")
            .pop();
        assert!(
            rebuild_north_shore_passing_width(&base, &before, &after, &incomplete.to_string())
                .is_err(),
            "missing {town} checkpoint lane"
        );
    }
    let mut damaged: Value = serde_json::from_str(&after).expect("terrain");
    let row = damaged["MnR11C00R00"]["rows"][0]
        .as_str()
        .expect("row")
        .replacen("05", "01", 1);
    damaged["MnR11C00R00"]["rows"][0] = row.into();
    assert!(
        rebuild_north_shore_passing_width(&base, &before, &damaged.to_string(), &gates).is_err(),
        "unreviewed outer-edge opening"
    );
    assert!(
        rebuild_north_shore_passing_width(&child, &before, &after, &gates).is_err(),
        "repeat publication"
    );
}
