//! Whole-pack preservation, fresh event allocation and source safety.
use std::path::PathBuf;

use serde_json::{Value, json};

use crate::{build_compact_interior_story_pack, read_verified_compiled_game_pack};

#[test]
#[ignore = "requires CRYSTAL_COMPACT_STORY_BASE and CRYSTAL_COMPACT_STORY_PACK"]
fn compact_interior_story_preserves_geography_services_and_existing_event_ids() {
    let base = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_COMPACT_STORY_BASE").expect("reviewed R09 parent"),
    ))
    .expect("strict parent");
    let child = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_COMPACT_STORY_PACK").expect("written room stories"),
    ))
    .expect("strict child");
    let stories = include_str!("../../../planning/minnesota/interior-life/stories.json");
    let supplement = std::env::var_os("CRYSTAL_COMPACT_STORY_SUPPLEMENT")
        .map(|path| std::fs::read_to_string(path).expect("explicit supplemental source"))
        .unwrap_or_else(|| {
            include_str!(
                "../../../planning/minnesota/interior-life/compact-story-supplement-v01.json"
            )
            .into()
        });
    let rebuilt = build_compact_interior_story_pack(&base, stories, &supplement)
        .expect("fresh source replay");
    assert!(
        rebuilt == child,
        "actual written pack equals the deterministic Rust replay"
    );
    assert!(base.compiled_audio() == child.compiled_audio());
    assert!(base.runtime_files() == child.runtime_files());
    let (before, after) = (base.data(), child.data());
    assert_eq!(before.maps.len(), after.maps.len());
    for (name, old) in &before.maps {
        let new = &after.maps[name];
        assert_eq!(
            old.attributes, new.attributes,
            "native map identity and geometry: {name}"
        );
        assert_eq!(old.blocks, new.blocks, "terrain: {name}");
        assert_eq!(
            old.events.warps, new.events.warps,
            "door/stair returns: {name}"
        );
        assert_eq!(
            old.events.coord_events, new.events.coord_events,
            "coordinate scenes: {name}"
        );
        if !name.starts_with("Mn")
            || name.contains("Gym")
            || matches!(
                old.attributes.environment.as_deref(),
                Some("TOWN" | "ROUTE" | "OUTDOOR")
            )
        {
            assert!(
                old == new,
                "all exterior, gym and nonregional content: {name}"
            );
        }
    }
    let mut old = serde_json::to_value(before).expect("parent data");
    let mut new = serde_json::to_value(after).expect("child data");
    for key in [
        "maps",
        "map_scripts",
        "npcs",
        "story_event_script_constants",
    ] {
        old.as_object_mut().expect("data object").remove(key);
        new.as_object_mut().expect("data object").remove(key);
    }
    assert!(
        old == new,
        "all other game content, art metadata and recovery remain exact"
    );
    let old = &before.story_event_script_constants;
    let new = &after.story_event_script_constants;
    assert_eq!(old.maps, new.maps);
    for (name, number) in &old.global {
        assert_eq!(
            new.global.get(name),
            Some(number),
            "existing event identity: {name}"
        );
    }
    let maximum = old
        .global
        .iter()
        .filter(|(name, _)| name.starts_with("EVENT_"))
        .map(|(_, number)| *number)
        .max()
        .expect("native events");
    let added = new
        .global
        .iter()
        .filter(|(name, _)| !old.global.contains_key(*name))
        .collect::<Vec<_>>();
    assert!(!added.is_empty());
    for (name, number) in &added {
        assert!(name.starts_with("EVENT_MN_INTERIOR_") && **number > maximum);
        assert_eq!(
            added.iter().filter(|(_, other)| *other == *number).count(),
            1
        );
    }
    let original: Value = serde_json::from_str(&supplement).expect("supplement");
    for (pointer, value) in [
        ("/base_content_hash", json!("stale")),
        ("/story_source_sha256", json!("unreviewed")),
        ("/manifest", json!("unreviewed")),
        ("/schema_version", json!(2)),
    ] {
        let mut invalid = original.clone();
        *invalid.pointer_mut(pointer).expect("case field") = value;
        assert!(build_compact_interior_story_pack(&base, stories, &invalid.to_string()).is_err());
    }
    let mut invalid = original.clone();
    invalid["rooms"].as_array_mut().expect("rooms").pop();
    assert!(build_compact_interior_story_pack(&base, stories, &invalid.to_string()).is_err());
    let mut invalid = original.clone();
    let duplicate = invalid["rooms"][0].clone();
    invalid["rooms"]
        .as_array_mut()
        .expect("rooms")
        .push(duplicate);
    assert!(build_compact_interior_story_pack(&base, stories, &invalid.to_string()).is_err());
    let mut invalid = original;
    invalid["rooms"][1]["conversations"][0]["actor"] = Value::Null;
    invalid["rooms"][1]["conversations"][0]["script"] = json!("MnElyNurse");
    assert!(
        build_compact_interior_story_pack(&base, stories, &invalid.to_string()).is_err(),
        "a local story must not replace the nurse service"
    );
    assert!(
        build_compact_interior_story_pack(&base, &(stories.to_owned() + "\n"), &supplement)
            .is_err()
    );
    assert!(build_compact_interior_story_pack(&child, stories, &supplement).is_err());
}
