//! Preserve a completed regional parent when adding the two native ice floors.
use std::path::PathBuf;

use crystal_assets::{build_voyageurs_ice_modpack, read_verified_compiled_game_pack};
use serde_json::Value;

fn retains_original(before: &Value, after: &Value) {
    match before {
        Value::Object(fields) => {
            for (key, value) in fields {
                retains_original(value, after.get(key).expect("original metadata entry"));
            }
        }
        Value::Array(values) => assert!(
            after
                .as_array()
                .expect("original metadata array")
                .starts_with(values),
            "existing metadata array entries stay exact"
        ),
        _ => assert_eq!(before, after),
    }
}

#[test]
#[ignore = "requires CRYSTAL_ICE_DUNGEON_BASE and CRYSTAL_ICE_DUNGEON_PACK"]
fn voyageurs_ice_composition_preserves_existing_interiors_programs_and_recovery() {
    let base = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_ICE_DUNGEON_BASE").expect("completed regional parent"),
    ))
    .expect("verified regional parent");
    let pack = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_ICE_DUNGEON_PACK").expect("composed ice pack"),
    ))
    .expect("verified ice output");
    let source = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../planning/minnesota/ice-caves/voyageurs-ice.json"),
    )
    .expect("explicit ice source");
    let plan: Value = serde_json::from_str(&source).expect("ice contract");
    let exterior = plan["exterior"]["map"].as_str().expect("entrance map");
    let before = base.data();
    let after = pack.data();
    assert_eq!(after.maps.len(), before.maps.len() + 2);
    assert_eq!(base.runtime_files(), pack.runtime_files());
    assert_eq!(base.compiled_audio(), pack.compiled_audio());

    for (name, old) in &before.maps {
        let new = &after.maps[name];
        if name != exterior {
            assert_eq!(
                old, new,
                "unrelated exterior/interior, including gyms: {name}"
            );
            continue;
        }
        let mut expected_blocks = old.blocks.clone();
        for change in plan["exterior"]["changes"]
            .as_array()
            .expect("entrance edits")
        {
            let index = change["metatile"][1].as_u64().expect("edit y") as usize
                * usize::from(old.attributes.width)
                + change["metatile"][0].as_u64().expect("edit x") as usize;
            assert_eq!(u64::from(expected_blocks[index]), change["previous"]);
            expected_blocks[index] = change["replacement"].as_u64().expect("new block") as u16;
        }
        assert_eq!(
            new.blocks, expected_blocks,
            "only the declared cave-mouth terrain"
        );
        let count = old.events.warps.len();
        assert_eq!(new.events.warps.len(), count + 2);
        assert_eq!(old.events.warps, new.events.warps[..count]);
        for (offset, key) in ["entrance", "return"].into_iter().enumerate() {
            let warp = &new.events.warps[count + offset];
            assert_eq!(usize::from(warp.index), count + offset + 1);
            assert_eq!(u64::from(warp.x), plan["exterior"][key][0]);
            assert_eq!(u64::from(warp.y), plan["exterior"][key][1]);
            assert_eq!(
                warp.target_map,
                plan["rooms"][0]["constant"].as_str().expect("upper floor")
            );
            assert_eq!(warp.target_warp_id, 3, "passive dry arrival");
        }
        let mut restored = new.clone();
        restored.blocks = old.blocks.clone();
        restored.events.warps = old.events.warps.clone();
        restored.map_event_section_commands = old.map_event_section_commands.clone();
        assert_eq!(
            restored, *old,
            "all original entrance-map programs and actors"
        );
    }

    // Remove only the additive map indexes, checking their original entries
    // separately. Every remaining dataset field, including HOME and recovery,
    // must match in full rather than relying on a map-count smoke check.
    let old_exterior = &before.maps[exterior];
    let mut old_data = serde_json::to_value(before).expect("old dataset");
    let mut new_data = serde_json::to_value(after).expect("new dataset");
    for key in [
        "maps",
        "map_scripts",
        "map_attributes",
        "map_dimensions",
        "map_blocks",
        "runtime_map_metadata",
        "story_event_script_constants",
        "npcs",
        "pokegear_landmarks",
    ] {
        let mut old = old_data
            .as_object_mut()
            .expect("old fields")
            .remove(key)
            .expect("old index");
        let mut new = new_data
            .as_object_mut()
            .expect("new fields")
            .remove(key)
            .expect("new index");
        if key == "maps" {
            continue;
        }
        let changed_label = match key {
            "map_scripts" => old_exterior.attributes.map_events_label.as_deref(),
            "map_blocks" => old_exterior.attributes.blocks_label.as_deref(),
            _ => None,
        };
        if let Some(label) = changed_label {
            assert!(
                old.as_object_mut()
                    .expect("old index")
                    .remove(label)
                    .is_some()
            );
            assert!(
                new.as_object_mut()
                    .expect("new index")
                    .remove(label)
                    .is_some()
            );
        }
        retains_original(&old, &new);
    }
    assert_eq!(
        old_data, new_data,
        "all other data, audio and recovery stay exact"
    );
    assert_eq!(
        build_voyageurs_ice_modpack(&base, &source)
            .expect("deterministic ice rebuild")
            .identity()
            .expect("rebuilt identity"),
        pack.identity().expect("published identity")
    );
    assert!(
        build_voyageurs_ice_modpack(&pack, &source).is_err(),
        "duplicate ice publication"
    );
    let mut invalid = plan.clone();
    invalid["exterior"]["changes"][0]["previous"] = serde_json::json!(0);
    assert!(
        build_voyageurs_ice_modpack(&base, &invalid.to_string()).is_err(),
        "stale cave-mouth preimage"
    );
    let mut invalid = plan;
    invalid["rooms"][0]["residents"][0]["tile"] = serde_json::json!([0, 0]);
    assert!(
        build_voyageurs_ice_modpack(&base, &invalid.to_string()).is_err(),
        "buried guide rejected"
    );
}
