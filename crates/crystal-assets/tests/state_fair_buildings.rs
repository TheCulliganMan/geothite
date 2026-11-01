//! Exact preservation and repeat-application guards for the facade-only repair.
use std::fs;

use crystal_assets::{self, revise_state_fair_buildings};

#[test]
#[ignore = "requires CRYSTAL_FAIR_BUILDINGS_BASE and CRYSTAL_FAIR_TEST_PACK"]
fn state_fair_buildings_preserve_gameplay_and_original_pixels() {
    let base = crystal_assets::read_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_FAIR_BUILDINGS_BASE").expect("explicit fair parent"),
    )
    .expect("verified parent");
    let revised = crystal_assets::read_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_FAIR_TEST_PACK").expect("explicit repaired pack"),
    )
    .expect("verified fair repair");
    let source = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../planning/minnesota/zones/state-fairgrounds.layout.json"
    ))
    .expect("fair layout");
    let rebuilt = revise_state_fair_buildings(&base, &source).expect("repeatable repair");
    assert_eq!(
        rebuilt.identity().expect("rebuilt identity"),
        revised.identity().expect("repaired identity")
    );
    assert!(
        revise_state_fair_buildings(&revised, &source).is_err(),
        "do not apply twice"
    );
    let old = base.data();
    let new = revised.data();
    let mut expected = serde_json::to_value(old).expect("parent catalogs");
    let actual = serde_json::to_value(new).expect("repaired catalogs");
    for field in ["maps", "map_blocks", "tilesets"] {
        expected[field] = actual[field].clone();
    }
    assert_eq!(
        expected, actual,
        "all other catalogs, trainers, encounters and audio remain exact"
    );
    for (name, map) in &old.maps {
        let mut expected = map.clone();
        if name == "MnStateFairgrounds" {
            expected.blocks = new.maps[name].blocks.clone();
            assert_eq!(
                map.blocks
                    .iter()
                    .zip(&expected.blocks)
                    .filter(|(a, b)| a != b)
                    .count(),
                15
            );
        }
        assert_eq!(
            expected, new.maps[name],
            "map events and unrelated terrain: {name}"
        );
    }
    assert_eq!(old.maps.len(), new.maps.len());
    let label = old.maps["MnStateFairgrounds"]
        .attributes
        .blocks_label
        .as_ref()
        .expect("fair block label");
    let mut expected_blocks = old.map_blocks.clone();
    expected_blocks.insert(label.clone(), new.map_blocks[label].clone());
    assert_eq!(expected_blocks, new.map_blocks);
    for (name, tileset) in &old.tilesets {
        let mut expected = tileset.clone();
        if name == "mn_state_fair_native" {
            for block in ["8b", "8c", "8d"] {
                expected
                    .collision
                    .insert(block.into(), vec!["WALL".into(); 4]);
            }
        }
        assert_eq!(
            expected, new.tilesets[name],
            "original glyph palettes and collision: {name}"
        );
    }
    assert_eq!(old.tilesets.len(), new.tilesets.len());
    let meta = "data/tilesets/mn_state_fair_native_metatiles.bin";
    let collisions = "data/tilesets/mn_state_fair_native.json";
    for (path, bytes) in base.runtime_files() {
        if path == meta {
            assert!(
                revised.runtime_files()[path].starts_with(bytes),
                "original metatile prefix"
            );
            assert_eq!(revised.runtime_files()[path].len(), bytes.len() + 48);
        } else if path != collisions {
            assert_eq!(
                Some(bytes),
                revised.runtime_files().get(path),
                "original file: {path}"
            );
        }
    }
    assert_eq!(base.runtime_files().len(), revised.runtime_files().len());
    assert_eq!(base.compiled_audio(), revised.compiled_audio());
    // A stale terrain preimage must fail instead of silently reverting other fair work.
    let mut drift: serde_json::Value = serde_json::from_str(&source).expect("layout JSON");
    drift["layers"]
        .as_array_mut()
        .expect("layers")
        .push(serde_json::json!({
            "name": "unrelated paving drift", "kind": "point", "position": [44, 32], "block": "02"
        }));
    assert!(revise_state_fair_buildings(&base, &drift.to_string()).is_err());
    let mut invalid: serde_json::Value = serde_json::from_str(&source).expect("layout JSON");
    invalid["derived_blocks"][11]["tile_order"][3] = 16.into();
    assert!(revise_state_fair_buildings(&base, &invalid.to_string()).is_err());
    invalid = serde_json::from_str(&source).expect("layout JSON");
    invalid["derived_blocks"][11]["source_block"] = "16".into();
    invalid["derived_blocks"][11]["collision"][3] = "DOOR".into();
    assert!(revise_state_fair_buildings(&base, &invalid.to_string()).is_err());
}
