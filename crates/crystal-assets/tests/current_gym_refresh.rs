use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{
    read_minnesota_gym_design, read_verified_compiled_game_pack, refresh_current_minnesota_gyms,
};
use serde_json::Value;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root")
}

#[test]
fn current_gym_selection_matches_sources_and_rejects_ambiguous_or_escaping_indexes() {
    let folder = root().join("planning/minnesota/gyms");
    let source = read_minnesota_gym_design(folder.join("current.json")).expect("current index");
    let merged: Value = serde_json::from_str(&source).expect("merged design");
    let index: Value =
        serde_json::from_str(&std::fs::read_to_string(folder.join("current.json")).expect("index"))
            .expect("index JSON");
    for (slot, revision) in index["overrides"]
        .as_array()
        .expect("eight revisions")
        .iter()
        .enumerate()
    {
        let raw: Value = serde_json::from_str(
            &std::fs::read_to_string(folder.join(revision["source"].as_str().expect("source")))
                .expect("revision"),
        )
        .expect("revision JSON");
        assert_eq!(merged["gyms"][slot], raw["gyms"][0]);
    }
    let historical = folder.join("interiors.json");
    assert_eq!(
        read_minnesota_gym_design(&historical).expect("raw source"),
        std::fs::read_to_string(historical).expect("unchanged raw source")
    );

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let temporary =
        std::env::temp_dir().join(format!("gym-selection-{}-{nonce}", std::process::id()));
    std::fs::create_dir(&temporary).expect("owned selection scratch");
    for path in std::fs::read_dir(folder).expect("authoring sources") {
        let path = path.expect("source entry").path();
        if path.extension().is_some_and(|x| x == "json") {
            std::fs::copy(&path, temporary.join(path.file_name().expect("filename")))
                .expect("copy authored JSON");
        }
    }
    let selected = temporary.join("current.json");
    let mut duplicate = index.clone();
    duplicate["overrides"]
        .as_array_mut()
        .expect("overrides")
        .push(index["overrides"][0].clone());
    std::fs::write(&selected, duplicate.to_string()).expect("duplicate fixture");
    assert!(read_minnesota_gym_design(&selected).is_err());
    let mut wrong_room = index.clone();
    wrong_room["overrides"][0]["map"] = "MnDuluthGym".into();
    std::fs::write(&selected, wrong_room.to_string()).expect("mismatched fixture");
    assert!(read_minnesota_gym_design(&selected).is_err());
    let mut escape = index.clone();
    escape["base"] = "../interiors.json".into();
    std::fs::write(&selected, escape.to_string()).expect("escaping fixture");
    assert!(read_minnesota_gym_design(&selected).is_err());
    let mut unknown = index;
    unknown["ignore_trainer_requirements"] = true.into();
    std::fs::write(&selected, unknown.to_string()).expect("unknown-field fixture");
    assert!(read_minnesota_gym_design(&selected).is_err());
    std::fs::remove_dir_all(temporary).expect("remove owned selection scratch");
}

fn preserve(parent: &str, published: &str) {
    let base = read_verified_compiled_game_pack(
        root().join(format!("content-packs/{parent}.crystalpack")),
    )
    .expect("frozen regional parent");
    let selected = read_minnesota_gym_design(root().join("planning/minnesota/gyms/current.json"))
        .expect("selected source");
    let revised =
        refresh_current_minnesota_gyms(&base, &selected).expect("recompile current rooms");
    let written = read_verified_compiled_game_pack(
        root().join(format!("content-packs/{published}.crystalpack")),
    )
    .expect("written current gyms");
    assert_eq!(
        revised.identity().expect("recompiled identity"),
        written.identity().expect("written identity")
    );
    assert_eq!(base.compiled_audio(), revised.compiled_audio());
    assert_eq!(base.data().trainers, revised.data().trainers);
    let source: Value = serde_json::from_str(&selected).expect("source");
    let mut names = BTreeSet::new();
    let mut script_labels = BTreeSet::new();
    let mut constants = BTreeSet::new();
    let mut block_labels = BTreeSet::new();
    let mut tilesets = BTreeSet::new();
    for gym in source["gyms"].as_array().expect("eight rooms") {
        let name = gym["id"].as_str().expect("room");
        let old = &base.data().maps[name];
        let room = &revised.data().maps[name];
        names.insert(name.to_string());
        constants.insert(gym["constant"].as_str().expect("constant").to_string());
        tilesets.insert(room.attributes.tileset_name.clone());
        let mut attributes = room.attributes.clone();
        attributes.width = old.attributes.width;
        attributes.height = old.attributes.height;
        attributes.border_block = old.attributes.border_block;
        assert_eq!(
            attributes, old.attributes,
            "retained native identity {name}"
        );
        for (current, previous) in room.events.warps.iter().zip(&old.events.warps).take(3) {
            let mut normalized = current.clone();
            normalized.x = previous.x;
            normalized.y = previous.y;
            assert_eq!(&normalized, previous, "retained exterior wire {name}");
        }
        assert_eq!(room.events.warps.len(), old.events.warps.len());
        for module in [old, room] {
            script_labels.extend(module.scripts.keys().cloned());
            script_labels.extend(module.script_text_bodies.keys().cloned());
            script_labels.extend(module.attributes.map_scripts_label.iter().cloned());
            script_labels.extend(module.attributes.map_events_label.iter().cloned());
            block_labels.extend(module.attributes.blocks_label.iter().cloned());
        }
        let mut uncached = revised.data().clone();
        uncached.maps.remove(name);
        let mut native = uncached
            .assemble_map_module_from_compiled_payloads(name)
            .expect("uncached native room");
        for label in [
            room.attributes.map_scripts_label.as_ref(),
            room.attributes.map_events_label.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            if !room.scripts.contains_key(label) {
                native.scripts.remove(label);
            }
        }
        assert_eq!(*room, native, "cached/native room {name}");
    }
    let mut before = serde_json::to_value(base.data()).expect("parent data");
    let mut after = serde_json::to_value(revised.data()).expect("revised data");
    for (field, owned) in [
        ("maps", &names),
        ("map_attributes", &names),
        ("npcs", &names),
        ("map_scripts", &script_labels),
        ("map_blocks", &block_labels),
        ("map_dimensions", &constants),
        ("tilesets", &tilesets),
    ] {
        for value in [&mut before, &mut after] {
            value[field]
                .as_object_mut()
                .expect("catalogue")
                .retain(|key, _| !owned.contains(key));
        }
    }
    for constant in &constants {
        for field in ["width", "height"] {
            after["runtime_map_metadata"][constant][field] =
                before["runtime_map_metadata"][constant][field].clone();
        }
    }
    for value in [&mut before, &mut after] {
        value["field_moves"]["cut"]["replacements"]
            .as_object_mut()
            .expect("Cut rules")
            .remove("mn_gym_7");
    }
    let original = before["story_event_script_constants"]["global"]
        .as_object()
        .expect("original constants");
    let globals = after["story_event_script_constants"]["global"]
        .as_object_mut()
        .expect("current constants");
    let last_event = original
        .iter()
        .filter(|(name, _)| name.starts_with("EVENT_"))
        .map(|(_, id)| id.as_i64().expect("event number"))
        .max()
        .expect("events");
    let added = globals
        .keys()
        .filter(|key| !original.contains_key(*key))
        .cloned()
        .collect::<Vec<_>>();
    assert!(!added.is_empty());
    for key in added {
        assert!(
            key.starts_with("EVENT_MN_GYM_"),
            "only gym events may be appended: {key}"
        );
        assert!(
            globals
                .remove(&key)
                .expect("new event")
                .as_i64()
                .expect("number")
                > last_event
        );
    }
    for (field, value) in before.as_object().expect("dataset") {
        assert!(
            value == &after[field],
            "unrelated dataset field changed: {field}"
        );
    }
    assert_eq!(base.runtime_files().len(), revised.runtime_files().len());
    for (path, bytes) in base.runtime_files() {
        let owned = tilesets.iter().any(|tileset| {
            [
                format!("data/tilesets/{tileset}.json"),
                format!("data/tilesets/{tileset}_metatiles.bin"),
                format!("data/tilesets/{tileset}_palette_map.json"),
                format!("gfx/tilesets/{tileset}.2bpp"),
                format!("gfx/tilesets/{tileset}.png"),
            ]
            .contains(path)
        });
        if !owned {
            assert_eq!(
                Some(bytes),
                revised.runtime_files().get(path),
                "unrelated payload {path}"
            );
        }
    }
    assert!(refresh_current_minnesota_gyms(&revised, &selected).is_err());
}

#[test]
#[ignore = "requires the frozen Ely story and published current gym packs"]
fn current_gyms_preserve_complete_story_branch_and_native_rooms() {
    preserve(
        "minnesota-northstar-ely-league",
        "minnesota-current-gyms-story",
    );
}

#[test]
#[ignore = "requires the compact Twin Cities parent and published current gym packs"]
fn current_gyms_preserve_complete_compact_city_branch_and_native_rooms() {
    preserve(
        "minnesota-twin-cities-compact-v01",
        "minnesota-current-gyms-cities",
    );
}

#[test]
#[ignore = "requires the frozen Ely story pack"]
fn current_gyms_reject_historical_or_changed_reviewed_sources() {
    let base = read_verified_compiled_game_pack(
        root().join("content-packs/minnesota-northstar-ely-league.crystalpack"),
    )
    .expect("Ely parent");
    let historical = std::fs::read_to_string(root().join("planning/minnesota/gyms/interiors.json"))
        .expect("historical source");
    assert!(refresh_current_minnesota_gyms(&base, &historical).is_err());
    let selected = read_minnesota_gym_design(root().join("planning/minnesota/gyms/current.json"))
        .expect("selected source");
    let source: Value = serde_json::from_str(&selected).expect("selected JSON");
    for (slot, field, value) in [
        (0, "reward", Value::from("TM_SURF")),
        (7, "leader", Value::from("STALE_LEADER")),
    ] {
        let mut changed = source.clone();
        changed["gyms"][slot][field] = value;
        assert!(refresh_current_minnesota_gyms(&base, &changed.to_string()).is_err());
    }
}
