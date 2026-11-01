//! Compare the actual scene publication with its complete species/gym parent.
use std::collections::BTreeSet;
use std::path::PathBuf;

use crystal_assets::{build_northstar_relay_scene, read_verified_compiled_game_pack};
use serde_json::Value;

#[test]
#[ignore = "requires the completed ignored relay base and scene packs"]
fn northstar_relay_preserves_other_content_native_collision_and_audio() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let source = std::fs::read_to_string(root.join("planning/minnesota/northstar-relay.json"))
        .expect("scene source");
    let base = read_verified_compiled_game_pack(
        root.join("content-packs/minnesota-northstar-relay-base.crystalpack"),
    )
    .expect("actual complete parent");
    let regional = read_verified_compiled_game_pack(
        root.join("content-packs/minnesota-current-gyms-story.crystalpack"),
    )
    .expect("selected-gym regional parent");
    assert!(
        regional.data().maps == base.data().maps,
        "species preparation preserves every existing story/gym/cabin map"
    );
    assert_eq!(regional.data().trainers, base.data().trainers);
    assert_eq!(
        regional.data().runtime_spawn_points,
        base.data().runtime_spawn_points
    );
    assert!(
        regional.compiled_audio() == base.compiled_audio(),
        "species preparation preserves all audio"
    );
    drop(regional);
    let result = build_northstar_relay_scene(&base, &source).expect("native relay scene");
    let written = read_verified_compiled_game_pack(
        root.join("content-packs/minnesota-northstar-relay-v2.crystalpack"),
    )
    .expect("published scene");
    assert_eq!(
        result.identity().expect("rebuilt identity"),
        written.identity().expect("written identity")
    );
    assert!(
        base.compiled_audio() == result.compiled_audio(),
        "scene preserves all audio"
    );
    let name = "MnBoundaryWatersSurveyShelter";
    let old = &base.data().maps[name];
    let room = &result.data().maps[name];
    assert_eq!(old.events, room.events);
    assert_eq!(old.objects, room.objects);
    let mut attributes = room.attributes.clone();
    attributes.tileset_name = old.attributes.tileset_name.clone();
    assert_eq!(attributes, old.attributes);
    for (before, after) in old.blocks.iter().zip(&room.blocks) {
        assert_eq!(
            base.data().tilesets["lab"].collision[&format!("{before:02x}")],
            result.data().tilesets["mn_northstar_relay"].collision[&format!("{after:02x}")]
        );
    }
    let mut uncached = result.data().clone();
    uncached.maps.remove(name);
    let mut native = uncached
        .assemble_map_module_from_compiled_payloads(name)
        .expect("uncached scene");
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
    let cached = serde_json::to_value(room).expect("cached room");
    let rebuilt = serde_json::to_value(&native).expect("uncached room");
    let changed = cached
        .as_object()
        .expect("room object")
        .iter()
        .filter(|(field, value)| rebuilt.get(*field) != Some(*value))
        .map(|(field, _)| field.as_str())
        .collect::<Vec<_>>();
    assert!(
        changed.is_empty(),
        "uncached scene fields differ: {changed:?}"
    );
    let mut labels = BTreeSet::new();
    for map in [old, room] {
        labels.extend(map.scripts.keys().cloned());
        labels.extend(map.attributes.map_scripts_label.iter().cloned());
        labels.extend(map.attributes.map_events_label.iter().cloned());
    }
    let mut before = serde_json::to_value(base.data()).expect("parent dataset");
    let mut after = serde_json::to_value(result.data()).expect("scene dataset");
    for v in [&mut before, &mut after] {
        for field in ["maps", "map_attributes", "npcs"] {
            v[field].as_object_mut().expect("map catalog").remove(name);
        }
        for label in &labels {
            v["map_scripts"]
                .as_object_mut()
                .expect("scripts")
                .remove(label);
        }
        v["map_blocks"]
            .as_object_mut()
            .expect("blocks")
            .remove(old.attributes.blocks_label.as_ref().expect("block label"));
        v["tilesets"]
            .as_object_mut()
            .expect("tilesets")
            .remove("mn_northstar_relay");
    }
    for (field, value) in before.as_object().expect("parent fields") {
        assert!(
            after.get(field) == Some(value),
            "unrelated dataset field {field}"
        );
    }
    for (path, bytes) in base.runtime_files() {
        assert!(
            Some(bytes) == result.runtime_files().get(path),
            "original runtime file {path}"
        );
    }
    assert!(
        result
            .runtime_files()
            .keys()
            .filter(|p| !base.runtime_files().contains_key(*p))
            .all(|p| p.contains("mn_northstar_relay"))
    );
    assert!(build_northstar_relay_scene(&result, &source).is_err());
    let canonical: Value = serde_json::from_str(&source).expect("source JSON");
    for (field, value) in [
        ("capture_required", Value::Bool(true)),
        ("free_access", Value::Bool(false)),
        ("species", Value::String("LAPRAS".into())),
        ("unexpected", Value::Bool(true)),
    ] {
        let mut wrong = canonical.clone();
        wrong[field] = value;
        assert!(
            build_northstar_relay_scene(&base, &wrong.to_string()).is_err(),
            "reject changed contract {field}"
        );
    }
}
