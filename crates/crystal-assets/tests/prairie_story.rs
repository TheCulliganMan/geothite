//! Bounded R02/story overlay on the exact reviewed regional input.
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{
    CompiledGamePack, build_prairie_story_modpack, read_verified_compiled_game_pack,
};

struct Inputs {
    base: CompiledGamePack,
    result: CompiledGamePack,
    source: [String; 2],
}
fn input() -> &'static Inputs {
    static INPUT: OnceLock<Inputs> = OnceLock::new();
    INPUT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository");
        let base = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-northstar-prairie-base.crystalpack"),
        )
        .expect("captured regional base");
        let source = ["playable/prairie-story.json", "story.json"].map(|p| {
            std::fs::read_to_string(root.join("planning/minnesota").join(p)).expect("authoring")
        });
        let result =
            build_prairie_story_modpack(&base, &source[0], &source[1]).expect("prairie chapter");
        let written = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-northstar-prairie-story.crystalpack"),
        )
        .expect("written chapter");
        assert_eq!(
            result.identity().expect("built identity"),
            written.identity().expect("written identity")
        );
        Inputs {
            base,
            result,
            source,
        }
    })
}

#[test]
#[ignore = "requires the captured external regional base and written prairie chapter"]
fn prairie_story_preserves_world_audio_recovery_and_reconstructs_uncached_modules() {
    let i = input();
    let before = i.base.data();
    let after = i.result.data();
    assert_eq!(after.maps.len(), before.maps.len());
    let changed = ["MnRochester", "MnR02C00R00", "MnMankato"];
    for (name, map) in &before.maps {
        if !changed.contains(&name.as_str()) {
            assert!(after.maps.get(name) == Some(map), "unrelated map {name}");
        }
    }
    let mut labels = BTreeSet::new();
    let mut block_labels = BTreeSet::new();
    for name in changed {
        let old = &before.maps[name];
        let new = &after.maps[name];
        for map in [old, new] {
            labels.extend(
                map.scripts
                    .keys()
                    .chain(map.script_text_bodies.keys())
                    .cloned(),
            );
            labels.extend(
                map.attributes
                    .map_scripts_label
                    .iter()
                    .chain(map.attributes.map_events_label.iter())
                    .cloned(),
            );
            block_labels.extend(map.attributes.blocks_label.iter().cloned());
        }
        let mut uncached = after.clone();
        uncached.maps.remove(name);
        let mut rebuilt = uncached
            .assemble_map_module_from_compiled_payloads(name)
            .expect("uncached native module");
        for label in [
            new.attributes.map_scripts_label.as_ref(),
            new.attributes.map_events_label.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            if !new.scripts.contains_key(label) {
                rebuilt.scripts.remove(label);
            }
        }
        let actual = serde_json::to_value(&rebuilt).expect("uncached JSON");
        let expected = serde_json::to_value(new).expect("cached JSON");
        let fields = actual
            .as_object()
            .expect("map object")
            .keys()
            .filter(|k| actual[*k] != expected[*k])
            .collect::<Vec<_>>();
        assert!(rebuilt == *new, "uncached {name} differs in {fields:?}");
        for (label, script) in &old.scripts {
            if label != "MN_MK_GARDENERText" {
                assert!(
                    new.scripts.get(label) == Some(script),
                    "existing script {label}"
                );
            }
        }
        assert_eq!(&new.objects[..old.objects.len()], old.objects.as_slice());
        let diffs = old
            .blocks
            .iter()
            .zip(&new.blocks)
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(|(n, _)| n)
            .collect::<Vec<_>>();
        let expected = match name {
            "MnRochester" => vec![9 * 20],
            "MnMankato" => vec![9 * 24 + 23],
            _ => Vec::new(),
        };
        assert_eq!(diffs, expected, "only authored forest mouths change");
    }
    assert_eq!(
        after.maps["MnR02C00R00"].objects,
        before.maps["MnR02C00R00"].objects
    );
    assert_eq!(after.wild_encounters, before.wild_encounters);
    assert_eq!(
        after.maps["MnRochester"].objects.len(),
        before.maps["MnRochester"].objects.len() + 2
    );
    assert_eq!(
        after.maps["MnMankato"].objects.len(),
        before.maps["MnMankato"].objects.len() + 1
    );
    assert!(
        !after.maps["MnRochester"]
            .scripts
            .contains_key("MnRCPrairieCheckEvidence")
    );
    for (key, value) in &before.map_scripts {
        if !labels.contains(key) {
            assert_eq!(
                after.map_scripts.get(key),
                Some(value),
                "unrelated script payload {key}"
            );
        }
    }
    assert!(
        after
            .map_scripts
            .keys()
            .all(|k| before.map_scripts.contains_key(k) || labels.contains(k))
    );
    for (key, value) in &before.map_blocks {
        if !block_labels.contains(key) {
            assert_eq!(after.map_blocks.get(key), Some(value));
        }
    }
    for (name, attrs) in &before.map_attributes {
        if !changed.contains(&name.as_str()) {
            assert_eq!(after.map_attributes.get(name), Some(attrs));
        }
    }
    for (name, npcs) in &before.npcs {
        if !changed.contains(&name.as_str()) {
            assert_eq!(after.npcs.get(name), Some(npcs));
        }
    }
    let mut restored = after.clone();
    restored.maps = before.maps.clone();
    restored.map_scripts = before.map_scripts.clone();
    restored.map_blocks = before.map_blocks.clone();
    restored.map_attributes = before.map_attributes.clone();
    restored.npcs = before.npcs.clone();
    let number = restored
        .story_event_script_constants
        .global
        .remove("EVENT_MN_PRAIRIE_NOTEBOOK")
        .expect("only new event");
    assert_eq!(
        number,
        before
            .story_event_script_constants
            .global
            .iter()
            .filter(|(k, _)| k.starts_with("EVENT_"))
            .map(|(_, n)| *n)
            .max()
            .expect("old event table")
            + 1
    );
    assert!(restored == *before, "unrelated game data changed");
    assert_eq!(i.result.runtime_files(), i.base.runtime_files());
    assert_eq!(i.result.compiled_audio(), i.base.compiled_audio());
    assert_eq!(
        serde_json::to_value(i.result.audio_manifest().expect("result audio")).expect("JSON"),
        serde_json::to_value(i.base.audio_manifest().expect("base audio")).expect("JSON")
    );
}

#[test]
#[ignore = "requires the captured external regional base and written prairie chapter"]
fn prairie_story_rejects_duplicate_unsafe_seams_and_changed_notebook_prerequisites() {
    let i = input();
    assert!(build_prairie_story_modpack(&i.result, &i.source[0], &i.source[1]).is_err());
    for path in ["offset", "mouth_metatile"] {
        let mut source: serde_json::Value = serde_json::from_str(&i.source[0]).expect("chapter");
        source["links"][0][path] = if path == "offset" {
            serde_json::json!(3)
        } else {
            serde_json::json!([1, 9])
        };
        assert!(build_prairie_story_modpack(&i.base, &source.to_string(), &i.source[1]).is_err());
    }
    let mut source: serde_json::Value = serde_json::from_str(&i.source[0]).expect("chapter");
    source["notebook"]["tile"] = serde_json::json!([29, 25]);
    assert!(build_prairie_story_modpack(&i.base, &source.to_string(), &i.source[1]).is_err());
    let mut story: serde_json::Value = serde_json::from_str(&i.source[1]).expect("story");
    let milestone = story["milestones"]
        .as_array_mut()
        .expect("milestones")
        .iter_mut()
        .find(|m| m["id"] == "MS02")
        .expect("notebook");
    milestone["requires"] = serde_json::json!([]);
    assert!(build_prairie_story_modpack(&i.base, &i.source[0], &story.to_string()).is_err());
}
