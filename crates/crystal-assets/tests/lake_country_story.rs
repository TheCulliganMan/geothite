//! Exact six-map road/report publication with uncached native reconstruction.
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{
    CompiledGamePack, build_lake_country_story_modpack, read_verified_compiled_game_pack,
};
use serde_json::{Value, json};

struct Inputs {
    base: CompiledGamePack,
    result: CompiledGamePack,
    source: [String; 4],
}
fn input() -> &'static Inputs {
    static INPUT: OnceLock<Inputs> = OnceLock::new();
    INPUT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository");
        let base = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-northstar-courier-story.crystalpack"),
        )
        .expect("courier base");
        let source = [
            "playable/lake-country-story.json",
            "story.json",
            "routes/MN_R05.json",
            "routes/MN_R06.json",
        ]
        .map(|p| {
            std::fs::read_to_string(root.join("planning/minnesota").join(p)).expect("authoring")
        });
        let result =
            build_lake_country_story_modpack(&base, &source[0], &source[1], &source[2], &source[3])
                .expect("lake-country story");
        let written = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-northstar-lake-country.crystalpack"),
        )
        .expect("written successor");
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
#[ignore = "requires completed external courier base and lake-country successor"]
fn lake_country_preserves_gyms_encounters_flags_audio_and_uncached_modules() {
    let i = input();
    let old = i.base.data();
    let new = i.result.data();
    let changed = [
        "MnMinneapolis",
        "MnR05C00R02",
        "MnStCloud",
        "MnR06C00R00",
        "MnR06C00R01",
        "MnBrainerd",
    ];
    assert_eq!(old.maps.len(), new.maps.len());
    let mut labels = BTreeSet::new();
    let mut block_labels = BTreeSet::new();
    for (name, map) in &old.maps {
        if !changed.contains(&name.as_str()) {
            assert_eq!(new.maps.get(name), Some(map), "unrelated map {name}");
        }
    }
    for name in changed {
        let a = &old.maps[name];
        let b = &new.maps[name];
        assert_eq!(a.events.warps, b.events.warps);
        assert_eq!(&b.objects[..a.objects.len()], a.objects.as_slice());
        for (label, body) in &a.scripts {
            if ![
                "MN_BRAINERD_RANGERText",
                "MnBrainerdRangerGiftText",
                "MnBrainerdRangerRepeatText",
            ]
            .contains(&label.as_str())
            {
                assert_eq!(b.scripts.get(label), Some(body), "old script {label}");
            }
        }
        let expected = match name {
            "MnMinneapolis" => vec![16],
            "MnR06C00R00" => vec![20, 22],
            "MnR06C00R01" => vec![19 * 32 + 6, 19 * 32 + 7, 19 * 32 + 9, 19 * 32 + 10],
            "MnBrainerd" => vec![16, 12 * 32, 12 * 32 + 31],
            _ => vec![],
        };
        assert_eq!(
            a.blocks
                .iter()
                .zip(&b.blocks)
                .enumerate()
                .filter_map(|(n, (a, b))| (a != b).then_some(n))
                .collect::<Vec<_>>(),
            expected,
            "only declared mouths/caps"
        );
        for m in [a, b] {
            labels.extend(m.scripts.keys().chain(m.script_text_bodies.keys()).cloned());
            labels.extend(
                m.attributes
                    .map_scripts_label
                    .iter()
                    .chain(m.attributes.map_events_label.iter())
                    .cloned(),
            );
            block_labels.extend(m.attributes.blocks_label.iter().cloned());
        }
        let mut uncached = new.clone();
        uncached.maps.remove(name);
        let mut rebuilt = uncached
            .assemble_map_module_from_compiled_payloads(name)
            .expect("uncached map");
        for label in b
            .attributes
            .map_scripts_label
            .iter()
            .chain(b.attributes.map_events_label.iter())
        {
            if !b.scripts.contains_key(label) {
                rebuilt.scripts.remove(label);
            }
        }
        let actual = serde_json::to_value(&rebuilt).expect("JSON");
        let expected = serde_json::to_value(b).expect("JSON");
        let fields = actual
            .as_object()
            .expect("map")
            .keys()
            .filter(|k| actual[*k] != expected[*k])
            .collect::<Vec<_>>();
        assert!(rebuilt == *b, "uncached {name} differs in {fields:?}");
    }
    for (label, body) in &old.map_scripts {
        if !labels.contains(label) {
            assert_eq!(new.map_scripts.get(label), Some(body));
        }
    }
    assert!(
        new.map_scripts
            .keys()
            .all(|l| old.map_scripts.contains_key(l) || labels.contains(l))
    );
    for (label, body) in &old.map_blocks {
        if !block_labels.contains(label) {
            assert_eq!(new.map_blocks.get(label), Some(body));
        }
    }
    for name in old
        .map_attributes
        .keys()
        .filter(|n| !changed.contains(&n.as_str()))
    {
        assert_eq!(new.map_attributes.get(name), old.map_attributes.get(name));
        assert_eq!(new.npcs.get(name), old.npcs.get(name));
    }
    let mut restored = new.clone();
    restored.maps = old.maps.clone();
    restored.map_scripts = old.map_scripts.clone();
    restored.map_blocks = old.map_blocks.clone();
    restored.map_attributes = old.map_attributes.clone();
    restored.npcs = old.npcs.clone();
    assert_eq!(
        restored, *old,
        "all other data/catalogs/numbers/HOME/recovery stay exact"
    );
    assert_eq!(i.result.runtime_files(), i.base.runtime_files());
    assert_eq!(i.result.compiled_audio(), i.base.compiled_audio());
    assert_eq!(
        serde_json::to_value(i.result.audio_manifest().expect("audio")).expect("JSON"),
        serde_json::to_value(i.base.audio_manifest().expect("audio")).expect("JSON")
    );
}
#[test]
#[ignore = "requires completed external courier base and lake-country successor"]
fn lake_country_rejects_duplicate_bad_mouth_unsealed_edge_and_added_story_gate() {
    let i = input();
    let build = |base: &CompiledGamePack, chapter: &str, r05: &str| {
        build_lake_country_story_modpack(base, chapter, &i.source[1], r05, &i.source[3])
    };
    assert!(build(&i.result, &i.source[0], &i.source[2]).is_err());
    for (field, value) in [("offset", json!(4)), ("mouth", json!([15, 0]))] {
        let mut source: Value = serde_json::from_str(&i.source[0]).expect("chapter");
        source["links"][0][field] = value;
        assert!(build(&i.base, &source.to_string(), &i.source[2]).is_err());
    }
    let mut source: Value = serde_json::from_str(&i.source[0]).expect("chapter");
    source["caps"].as_array_mut().expect("caps").pop();
    assert!(build(&i.base, &source.to_string(), &i.source[2]).is_err());
    let mut r05: Value = serde_json::from_str(&i.source[2]).expect("road");
    r05["required_flags"] = json!(["BADGE_3", "BADGE_4", "EVENT_MN_COURIER_DEFEATED"]);
    assert!(build(&i.base, &i.source[0], &r05.to_string()).is_err());
}
