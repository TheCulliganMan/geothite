//! Exact bounded courier/R04 publication, including truly uncached reconstruction.
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{
    CompiledGamePack, build_courier_story_modpack, read_verified_compiled_game_pack,
};
use serde_json::{Value, json};

struct Inputs {
    base: CompiledGamePack,
    result: CompiledGamePack,
    source: [String; 3],
}
fn input() -> &'static Inputs {
    static INPUT: OnceLock<Inputs> = OnceLock::new();
    INPUT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository");
        let base = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-northstar-river-story.crystalpack"),
        )
        .expect("river base");
        let source = [
            "playable/courier-story.json",
            "story.json",
            "routes/MN_R04.json",
        ]
        .map(|p| std::fs::read_to_string(root.join("planning/minnesota").join(p)).expect("source"));
        let result = build_courier_story_modpack(&base, &source[0], &source[1], &source[2])
            .expect("courier chapter");
        let written = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-northstar-courier-story.crystalpack"),
        )
        .expect("written chapter");
        assert_eq!(
            result.identity().expect("identity"),
            written.identity().expect("identity")
        );
        Inputs {
            base,
            result,
            source,
        }
    })
}

#[test]
#[ignore = "requires external river base and written courier successor"]
fn courier_story_preserves_world_audio_gyms_and_uncached_script_modules() {
    let i = input();
    let before = i.base.data();
    let after = i.result.data();
    let changed = ["MnMinneapolis", "MnR04C00R00", "MnStPaulUnionDepot"];
    assert_eq!(before.maps.len(), after.maps.len());
    let mut labels = BTreeSet::new();
    let mut blocks = BTreeSet::new();
    for (id, map) in &before.maps {
        if !changed.contains(&id.as_str()) {
            assert_eq!(after.maps.get(id), Some(map), "unrelated map {id}");
        }
    }
    for id in changed {
        let old = &before.maps[id];
        let new = &after.maps[id];
        assert_eq!(
            old.events.warps, new.events.warps,
            "all existing door pairings"
        );
        assert_eq!(&new.objects[..old.objects.len()], old.objects.as_slice());
        for (label, script) in &old.scripts {
            if label != "MN_ST_PAUL_DEPOT_COURIER" {
                assert_eq!(new.scripts.get(label), Some(script), "old script {label}");
            }
        }
        assert_eq!(
            old.blocks
                .iter()
                .zip(&new.blocks)
                .enumerate()
                .filter_map(|(n, (a, b))| (a != b).then_some(n))
                .collect::<Vec<_>>(),
            if id == "MnMinneapolis" {
                vec![12 * 32 + 31]
            } else {
                vec![]
            }
        );
        for m in [old, new] {
            labels.extend(m.scripts.keys().chain(m.script_text_bodies.keys()).cloned());
            labels.extend(
                m.attributes
                    .map_scripts_label
                    .iter()
                    .chain(m.attributes.map_events_label.iter())
                    .cloned(),
            );
            blocks.extend(m.attributes.blocks_label.iter().cloned());
        }
        let mut uncached = after.clone();
        uncached.maps.remove(id);
        let mut rebuilt = uncached
            .assemble_map_module_from_compiled_payloads(id)
            .expect("uncached module");
        for label in new
            .attributes
            .map_scripts_label
            .iter()
            .chain(new.attributes.map_events_label.iter())
        {
            if !new.scripts.contains_key(label) {
                rebuilt.scripts.remove(label);
            }
        }
        let actual = serde_json::to_value(&rebuilt).expect("uncached JSON");
        let expected = serde_json::to_value(new).expect("cached JSON");
        let fields = actual
            .as_object()
            .expect("map fields")
            .keys()
            .filter(|key| actual[*key] != expected[*key])
            .collect::<Vec<_>>();
        assert!(rebuilt == *new, "uncached {id} differs in {fields:?}");
    }
    assert_eq!(
        after.maps["MnMinneapolis"].objects.len(),
        before.maps["MnMinneapolis"].objects.len() + 2
    );
    assert_eq!(
        after.maps["MnStPaulUnionDepot"].objects.len(),
        before.maps["MnStPaulUnionDepot"].objects.len() + 1
    );
    for (label, script) in &before.map_scripts {
        if !labels.contains(label) {
            assert_eq!(after.map_scripts.get(label), Some(script));
        }
    }
    assert!(
        after
            .map_scripts
            .keys()
            .all(|key| before.map_scripts.contains_key(key) || labels.contains(key))
    );
    for (label, payload) in &before.map_blocks {
        if !blocks.contains(label) {
            assert_eq!(after.map_blocks.get(label), Some(payload));
        }
    }
    for name in before
        .map_attributes
        .keys()
        .filter(|name| !changed.contains(&name.as_str()))
    {
        assert_eq!(
            after.map_attributes.get(name),
            before.map_attributes.get(name)
        );
        assert_eq!(after.npcs.get(name), before.npcs.get(name));
    }
    let mut restored = after.clone();
    restored.maps = before.maps.clone();
    restored.map_scripts = before.map_scripts.clone();
    restored.map_blocks = before.map_blocks.clone();
    restored.map_attributes = before.map_attributes.clone();
    restored.npcs = before.npcs.clone();
    let next = before
        .story_event_script_constants
        .global
        .iter()
        .filter(|(k, _)| k.starts_with("EVENT_"))
        .map(|(_, n)| *n)
        .max()
        .expect("events")
        + 1;
    for (flag, number) in [
        ("EVENT_MN_COURIER_DEFEATED", next),
        ("EVENT_MN_DEEP_BORE_MANIFEST_READ", next + 1),
    ] {
        assert_eq!(
            restored.story_event_script_constants.global.remove(flag),
            Some(number)
        );
    }
    assert!(
        restored
            .story_event_script_constants
            .global
            .remove("MN_LODE_COURIER")
            .is_some()
    );
    assert!(
        restored
            .trainers
            .trainers
            .remove("MN_LODE_COURIER")
            .is_some()
    );
    assert_eq!(
        restored, *before,
        "all unrelated catalogs, native events, encounters, HOME and recovery"
    );
    assert_eq!(i.result.runtime_files(), i.base.runtime_files());
    assert_eq!(i.result.compiled_audio(), i.base.compiled_audio());
    assert_eq!(
        serde_json::to_value(i.result.audio_manifest().expect("audio")).expect("JSON"),
        serde_json::to_value(i.base.audio_manifest().expect("audio")).expect("JSON")
    );
}

#[test]
#[ignore = "requires external river base and written courier successor"]
fn courier_story_rejects_duplicate_bad_anchor_boss_and_premature_record() {
    let i = input();
    assert!(
        build_courier_story_modpack(&i.result, &i.source[0], &i.source[1], &i.source[2]).is_err()
    );
    let mut chapter: Value = serde_json::from_str(&i.source[0]).expect("chapter");
    chapter["record"]["tile"] = json!([19, 16]);
    assert!(
        build_courier_story_modpack(&i.base, &chapter.to_string(), &i.source[1], &i.source[2])
            .is_err()
    );
    let mut story: Value = serde_json::from_str(&i.source[1]).expect("story");
    story["milestones"]
        .as_array_mut()
        .expect("milestones")
        .iter_mut()
        .find(|m| m["id"] == "MS04_MANIFEST")
        .expect("record")["requires"] = json!([]);
    assert!(
        build_courier_story_modpack(&i.base, &i.source[0], &story.to_string(), &i.source[2])
            .is_err()
    );
    let mut story: Value = serde_json::from_str(&i.source[1]).expect("story");
    story["bosses"]
        .as_array_mut()
        .expect("bosses")
        .iter_mut()
        .find(|m| m["id"] == "MN_LODE_COURIER")
        .expect("courier")["party"][0]["level"] = json!(24);
    assert!(
        build_courier_story_modpack(&i.base, &i.source[0], &story.to_string(), &i.source[2])
            .is_err()
    );
    let mut route: Value = serde_json::from_str(&i.source[2]).expect("route");
    route["required_flags"] = json!(["BADGE_3", "EVENT_MN_COURIER_DEFEATED"]);
    assert!(
        build_courier_story_modpack(&i.base, &i.source[0], &i.source[1], &route.to_string())
            .is_err()
    );
}
