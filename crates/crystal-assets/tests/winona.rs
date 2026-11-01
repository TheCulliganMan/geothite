//! Opening publication preserves the whole original world and native art/audio.
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{
    CompiledGamePack, build_winona_opening_modpack, read_verified_compiled_game_pack,
};

struct Inputs {
    base: CompiledGamePack,
    result: CompiledGamePack,
    sources: Vec<String>,
}
fn inputs() -> &'static Inputs {
    static INPUT: OnceLock<Inputs> = OnceLock::new();
    INPUT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository");
        let base = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-northstar-city-story.crystalpack"),
        )
        .expect("completed northern/city input");
        let sources = [
            "towns/MnWinona.json",
            "towns/MnWinona.layout.json",
            "story.json",
        ]
        .into_iter()
        .map(|p| std::fs::read_to_string(root.join("planning/minnesota").join(p)).expect("source"))
        .collect::<Vec<_>>();
        let result = build_winona_opening_modpack(&base, &sources[0], &sources[1], &sources[2])
            .expect("opening");
        Inputs {
            base,
            result,
            sources,
        }
    })
}

#[test]
#[ignore = "requires the external integrated story pack"]
fn winona_publication_preserves_all_original_maps_art_audio_and_recovery_destinations() {
    let i = inputs();
    let before = i.base.data();
    let after = i.result.data();
    let mut restored = after.clone();
    assert_eq!(after.maps.len(), before.maps.len() + 6);
    for (name, module) in &before.maps {
        assert_eq!(
            after.maps.get(name),
            Some(module),
            "original map changed: {name}"
        );
    }
    for name in [
        "MnWinona",
        "MnWinonaHome1F",
        "MnWinonaHome2F",
        "MnWinonaPokecenter1F",
        "MnWinonaMart",
        "MnWinonaFieldStation",
    ] {
        let m = &after.maps[name];
        let constant = m.attributes.map_constant.as_ref().expect("constant");
        let mut uncached = after.clone();
        uncached.maps.remove(name);
        let mut assembled = uncached
            .assemble_map_module_from_compiled_payloads(name)
            .expect("real uncached assembly");
        for label in [
            m.attributes.map_scripts_label.as_ref(),
            m.attributes.map_events_label.as_ref(),
        ] {
            let label = label.expect("transport");
            if !m.scripts.contains_key(label) {
                assembled.scripts.remove(label);
            }
        }
        assert!(assembled == *m, "uncached opening module differs: {name}");
        restored.maps.remove(name);
        restored.map_attributes.remove(name);
        restored.runtime_map_metadata.remove(constant);
        restored.map_dimensions.remove(constant);
        restored.npcs.remove(name);
        restored
            .map_blocks
            .remove(m.attributes.blocks_label.as_ref().expect("blocks"));
        for label in m
            .scripts
            .keys()
            .chain(m.script_text_bodies.keys())
            .chain(m.attributes.map_scripts_label.iter())
            .chain(m.attributes.map_events_label.iter())
        {
            restored.map_scripts.remove(label);
        }
        restored.pokegear_landmarks.map_to_landmark.remove(name);
    }
    assert_eq!(after.runtime_spawn_points["0"].map_name, "MnWinonaHome2F");
    assert_eq!(
        (
            after.runtime_spawn_points["0"].tile_x,
            after.runtime_spawn_points["0"].tile_y
        ),
        (3, 3)
    );
    let old_home = &before.runtime_spawn_points["0"];
    let preserved = after
        .runtime_spawn_points
        .values()
        .find(|s| s.identifier >= 256 && s.map_constant == old_home.map_constant)
        .expect("original HOME recovery still available");
    let mut normalized = preserved.clone();
    normalized.identifier = 0;
    assert_eq!(normalized, *old_home);
    for (key, point) in &before.runtime_spawn_points {
        if key != "0" {
            assert_eq!(after.runtime_spawn_points.get(key), Some(point));
        }
    }
    restored.runtime_spawn_points = before.runtime_spawn_points.clone();
    let added = after
        .story_event_script_constants
        .global
        .keys()
        .filter(|name| {
            !before
                .story_event_script_constants
                .global
                .contains_key(*name)
        })
        .cloned()
        .collect::<BTreeSet<_>>();
    for (name, value) in &before.story_event_script_constants.global {
        assert_eq!(
            after.story_event_script_constants.global.get(name),
            Some(value),
            "original symbol changed: {name}"
        );
    }
    for name in added {
        assert!(
            name == "MAPGROUP_MINNESOTA_WINONA"
                || name == "EVENT_MN_HOME_INTRO"
                || name == "EVENT_MN_R01_PARCEL_RECEIVED"
                || name == "EVENT_MN_STARTER_RECEIVED"
                || name.starts_with("EVENT_MN_WINONA_")
        );
        restored.story_event_script_constants.global.remove(&name);
    }
    if let Some(old) = before
        .story_event_script_constants
        .maps
        .get("MnWinonaHome1F")
    {
        restored
            .story_event_script_constants
            .maps
            .insert("MnWinonaHome1F".into(), old.clone());
    } else {
        restored
            .story_event_script_constants
            .maps
            .remove("MnWinonaHome1F");
    }
    restored.marts.0.remove("MART_MN_WINONA");
    assert_eq!(
        after.pokegear_landmarks.landmarks.len(),
        before.pokegear_landmarks.landmarks.len() + 1
    );
    assert_eq!(
        restored
            .pokegear_landmarks
            .landmarks
            .pop()
            .expect("new landmark")
            .constant,
        "LANDMARK_MN_WINONA"
    );
    let mut expected_title = before.runtime_title_screen.clone();
    let key = expected_title.program.entrypoints["new_game"].clone();
    let landmark = after
        .pokegear_landmarks
        .landmarks
        .last()
        .expect("Winona landmark")
        .id;
    for op in &mut expected_title
        .program
        .blocks
        .get_mut(&key)
        .expect("new game")
        .operations
    {
        if op.op == "write_memory_byte"
            && op.fields.get("target").and_then(|v| v.as_str()) == Some("wPrevLandmark")
        {
            op.fields
                .insert("value".into(), serde_json::json!(landmark));
        }
    }
    assert!(
        after.runtime_title_screen == expected_title,
        "unrelated title program changed"
    );
    restored.runtime_title_screen = before.runtime_title_screen.clone();
    assert!(
        restored == *before,
        "unrelated original dataset field changed"
    );
    assert_eq!(i.result.runtime_files(), i.base.runtime_files());
    assert_eq!(i.result.compiled_audio(), i.base.compiled_audio());
    assert_eq!(
        i.result.audio_manifest().expect("new audio"),
        i.base.audio_manifest().expect("original audio")
    );
    let lab = &after.maps["MnWinonaFieldStation"];
    assert_eq!(lab.gift_pokemon_scripts.len(), 3);
    assert!(lab.gift_pokemon_scripts.iter().all(|g| g.level == 5));
    assert!(
        lab.script_flag_commands
            .iter()
            .all(|f| f.command != "setflag"),
        "no badge/engine grant"
    );
    assert!(
        !lab.script_economy_commands
            .iter()
            .any(|c| c.command == "givemoney")
    );
}

#[test]
#[ignore = "requires the external integrated story pack"]
fn winona_rejects_duplicate_wrong_starter_and_premature_parcel_contract() {
    let i = inputs();
    assert!(
        build_winona_opening_modpack(&i.result, &i.sources[0], &i.sources[1], &i.sources[2])
            .is_err()
    );
    let mut changed: serde_json::Value = serde_json::from_str(&i.sources[0]).expect("dossier");
    changed["opening"]["starters"][0]["level"] = serde_json::json!(50);
    assert!(
        build_winona_opening_modpack(&i.base, &changed.to_string(), &i.sources[1], &i.sources[2])
            .is_err()
    );
    let mut story: serde_json::Value = serde_json::from_str(&i.sources[2]).expect("story");
    story["milestones"]
        .as_array_mut()
        .expect("milestones")
        .iter_mut()
        .find(|m| m["id"] == "MS00")
        .expect("starter")["requires"] = serde_json::json!([]);
    assert!(
        build_winona_opening_modpack(&i.base, &i.sources[0], &i.sources[1], &story.to_string())
            .is_err()
    );
}
