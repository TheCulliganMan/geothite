//! External-pack preservation and authoring safety for non-gym room stories.
use std::path::PathBuf;

use crystal_assets::{build_minnesota_interior_life_modpack, read_verified_compiled_game_pack};
use serde_json::Value;

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_BASE and CRYSTAL_INTERIOR_PACK external packs"]
fn interior_life_preserves_original_programs_services_gyms_and_audio() {
    let base = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_INTERIOR_BASE").expect("interior parent"),
    ))
    .expect("verified interior parent");
    let pack = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_INTERIOR_PACK").expect("interior publication"),
    ))
    .expect("verified interior publication");
    let source = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../planning/minnesota/interior-life/stories.json"),
    )
    .expect("authored rooms");
    let plan: Value = serde_json::from_str(&source).expect("room contract");
    let before = base.data();
    let after = pack.data();
    assert_eq!(base.runtime_files(), pack.runtime_files());
    assert_eq!(base.compiled_audio(), pack.compiled_audio());
    // Everything outside the split map/script payloads and newly allocated event
    // constants is byte-equivalent at the serialized data boundary.
    let mut old_data = serde_json::to_value(before).expect("old data");
    let mut new_data = serde_json::to_value(after).expect("new data");
    for key in [
        "maps",
        "map_scripts",
        "npcs",
        "story_event_script_constants",
    ] {
        old_data.as_object_mut().expect("old fields").remove(key);
        new_data.as_object_mut().expect("new fields").remove(key);
    }
    assert_eq!(
        old_data, new_data,
        "services/species/terrain/title/recovery stay exact"
    );
    for (key, value) in &before.story_event_script_constants.global {
        assert_eq!(
            Some(value),
            after.story_event_script_constants.global.get(key)
        );
    }
    for (id, old) in &before.maps {
        let new = &after.maps[id];
        let Some(room) = plan["rooms"]
            .as_array()
            .expect("rooms")
            .iter()
            .find(|r| r["map"] == *id)
        else {
            assert_eq!(old, new, "unrelated map, including gyms: {id}");
            continue;
        };
        assert_eq!(old.attributes, new.attributes);
        assert_eq!(old.blocks, new.blocks);
        assert_eq!(old.events, new.events);
        assert_eq!(old.objects, new.objects[..old.objects.len()]);
        assert!(!new.objects.is_empty(), "inhabited {id}");
        for (label, script) in &old.scripts {
            let replaced = room["texts"]
                .as_array()
                .is_some_and(|texts| texts.iter().any(|t| t["label"] == *label))
                || room["conversations"]
                    .as_array()
                    .is_some_and(|cast| cast.iter().any(|c| c["script"] == *label));
            if !replaced {
                assert_eq!(
                    Some(script),
                    new.scripts.get(label),
                    "original script {label}"
                );
            }
        }
        assert_eq!(
            old.script_runtime_commands,
            new.script_runtime_commands[..old.script_runtime_commands.len()]
        );
        assert_eq!(
            old.script_object_commands,
            new.script_object_commands[..old.script_object_commands.len()]
        );
        assert_eq!(
            old.script_control_commands,
            new.script_control_commands[..old.script_control_commands.len()]
        );
        assert_eq!(
            old.script_flag_commands,
            new.script_flag_commands[..old.script_flag_commands.len()]
        );
        assert_eq!(
            old.script_item_grants,
            new.script_item_grants[..old.script_item_grants.len()]
        );
        assert_eq!(old.script_shop_commands, new.script_shop_commands);
        assert_eq!(old.trainer_scripts, new.trainer_scripts);
        for body in new.script_text_bodies.values() {
            if !old.script_text_bodies.contains_key(&body.label)
                || room["texts"]
                    .as_array()
                    .is_some_and(|ts| ts.iter().any(|t| t["label"] == body.label))
            {
                for command in &body.commands {
                    assert!(
                        command
                            .args
                            .iter()
                            .all(|s| s.trim_matches('"').chars().count() <= 18),
                        "full native line: {}",
                        body.label
                    );
                }
            }
        }
    }
    assert_eq!(
        build_minnesota_interior_life_modpack(&base, &source)
            .expect("repeat build")
            .identity()
            .expect("identity"),
        pack.identity().expect("identity")
    );
    assert!(build_minnesota_interior_life_modpack(&pack, &source).is_err());
    let mut invalid = plan.clone();
    invalid["rooms"].as_array_mut().expect("rooms").pop();
    assert!(
        build_minnesota_interior_life_modpack(&base, &invalid.to_string()).is_err(),
        "full coverage cannot silently omit an interior"
    );
    invalid = plan.clone();
    invalid["rooms"][0]["map"] = "MnDuluthGym".into();
    assert!(build_minnesota_interior_life_modpack(&base, &invalid.to_string()).is_err());
    invalid = plan.clone();
    invalid["rooms"][0]["conversations"][0]["actor"]["tile"] = serde_json::json!([0, 0]);
    assert!(build_minnesota_interior_life_modpack(&base, &invalid.to_string()).is_err());
    invalid = plan.clone();
    invalid["rooms"][0]["conversations"][0]
        .as_object_mut()
        .expect("actor")
        .remove("actor");
    invalid["rooms"][0]["conversations"][0]["script"] = "MN_DULUTH_X_BEACHSHELTER_F1_HOST".into();
    assert!(
        build_minnesota_interior_life_modpack(&base, &invalid.to_string()).is_err(),
        "cannot replace healer"
    );
    invalid = plan.clone();
    let room = invalid["rooms"]
        .as_array_mut()
        .expect("rooms")
        .iter_mut()
        .find(|room| room["map"] == "MnDuluthHarborWorkshop")
        .expect("harbor story room");
    room["texts"][0]["label"] = "MN_HARBOR_RECEIVERText".into();
    let error = build_minnesota_interior_life_modpack(&base, &invalid.to_string())
        .expect_err("flavor cannot displace the acknowledged harbor reveal");
    assert!(format!("{error:#}").contains("reviewed Northstar story text"));
}
