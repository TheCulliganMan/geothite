//! Bounded River Valley connections on the captured Prairie Road chapter.
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{CompiledGamePack, connect_river_cities, read_verified_compiled_game_pack};
use serde_json::{Value, json};

struct Inputs {
    base: CompiledGamePack,
    result: CompiledGamePack,
    links: String,
    route: String,
}
fn input() -> &'static Inputs {
    static INPUT: OnceLock<Inputs> = OnceLock::new();
    INPUT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository root");
        let base = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-northstar-prairie-story.crystalpack"),
        )
        .expect("captured Prairie Road chapter");
        let links =
            std::fs::read_to_string(root.join("planning/minnesota/playable/river-city-links.json"))
                .expect("fixed reciprocal links");
        let route = std::fs::read_to_string(root.join("planning/minnesota/routes/MN_R03.json"))
            .expect("River Valley authoring");
        let result = connect_river_cities(&base, &links, &route).expect("River Valley successor");
        let written = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-northstar-river-story.crystalpack"),
        )
        .expect("written successor");
        assert_eq!(
            result.identity().expect("identity"),
            written.identity().expect("identity")
        );
        Inputs {
            base,
            result,
            links,
            route,
        }
    })
}

#[test]
#[ignore = "requires external Prairie Road base and written River Valley successor"]
fn river_cities_preserve_gyms_river_body_story_audio_and_uncached_modules() {
    let i = input();
    let before = i.base.data();
    let after = i.result.data();
    let changed = ["MnMankato", "MnMinneapolis", "MnR03C00R00", "MnR03C00R01"];
    assert_eq!(before.maps.len(), after.maps.len());
    for (id, map) in &before.maps {
        if !changed.contains(&id.as_str()) {
            assert_eq!(after.maps.get(id), Some(map), "unrelated map {id}");
        }
    }
    let mut labels = BTreeSet::new();
    let mut blocks = BTreeSet::new();
    for id in changed {
        let old = &before.maps[id];
        let new = &after.maps[id];
        assert_eq!(
            new.events.warps, old.events.warps,
            "all existing door pairings"
        );
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
            blocks.extend(map.attributes.blocks_label.iter().cloned());
        }
        for (label, script) in &old.scripts {
            assert_eq!(
                new.scripts.get(label),
                Some(script),
                "existing script {label}"
            );
        }
        assert_eq!(&new.objects[..old.objects.len()], old.objects.as_slice());
        assert_eq!(
            &new.attributes.connections[..old.attributes.connections.len()],
            old.attributes.connections.as_slice()
        );
        let differences = old
            .blocks
            .iter()
            .zip(&new.blocks)
            .enumerate()
            .filter_map(|(n, (a, b))| (a != b).then_some(n))
            .collect::<Vec<_>>();
        assert_eq!(
            differences,
            match id {
                "MnMankato" => vec![12],
                "MnMinneapolis" => vec![23 * 32 + 16],
                "MnR03C00R01" => [7, 8, 10, 11, 13, 18].map(|x| 7 * 24 + x).to_vec(),
                _ => Vec::new(),
            },
            "only two town mouths and six south-bank boundary caps change"
        );
        let mut uncached = after.clone();
        uncached.maps.remove(id);
        let mut rebuilt = uncached
            .assemble_map_module_from_compiled_payloads(id)
            .expect("uncached module");
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
        assert_eq!(rebuilt, *new, "cached and split-payload module {id}");
    }
    let guards = &after.maps["MnMankato"];
    assert_eq!(
        guards.objects.len(),
        before.maps["MnMankato"].objects.len() + 2
    );
    assert!(!guards.scripts.contains_key("MnMKRiverCheckEvidence"));
    let check = serde_json::to_string(&guards.scripts["MnMKRiverCheck"]).expect("guard program");
    assert!(check.contains("ENGINE_HIVEBADGE"));
    assert!(!check.contains("EVENT_MN_PRAIRIE_NOTEBOOK"));
    for (label, script) in &before.map_scripts {
        if !labels.contains(label) {
            assert_eq!(after.map_scripts.get(label), Some(script));
        }
    }
    assert!(
        after
            .map_scripts
            .keys()
            .all(|label| before.map_scripts.contains_key(label) || labels.contains(label))
    );
    for (label, data) in &before.map_blocks {
        if !blocks.contains(label) {
            assert_eq!(after.map_blocks.get(label), Some(data));
        }
    }
    for id in before
        .map_attributes
        .keys()
        .filter(|id| !changed.contains(&id.as_str()))
    {
        assert_eq!(after.map_attributes.get(id), before.map_attributes.get(id));
        assert_eq!(after.npcs.get(id), before.npcs.get(id));
    }
    let mut restored = after.clone();
    restored.maps = before.maps.clone();
    restored.map_scripts = before.map_scripts.clone();
    restored.map_blocks = before.map_blocks.clone();
    restored.map_attributes = before.map_attributes.clone();
    restored.npcs = before.npcs.clone();
    assert_eq!(
        restored, *before,
        "catalogs, globals, encounters, HOME, recovery unchanged"
    );
    assert_eq!(i.result.runtime_files(), i.base.runtime_files());
    assert_eq!(i.result.compiled_audio(), i.base.compiled_audio());
    assert_eq!(
        serde_json::to_value(i.result.audio_manifest().expect("audio")).expect("JSON"),
        serde_json::to_value(i.base.audio_manifest().expect("audio")).expect("JSON")
    );
}

#[test]
#[ignore = "requires external Prairie Road base and written River Valley successor"]
fn river_cities_reject_duplicate_changed_badge_and_unsafe_projection_contracts() {
    let i = input();
    assert!(connect_river_cities(&i.result, &i.links, &i.route).is_err());
    for (field, replacement) in [
        ("offset", json!(4)),
        ("town_edge", json!([[23, 0], [24, 0]])),
        ("mouth_metatile", json!([11, 0])),
    ] {
        let mut links: Value = serde_json::from_str(&i.links).expect("links");
        links["links"][0][field] = replacement;
        assert!(
            connect_river_cities(&i.base, &links.to_string(), &i.route).is_err(),
            "changed {field}"
        );
    }
    let mut route: Value = serde_json::from_str(&i.route).expect("route");
    route["required_flags"] = json!(["BADGE_1"]);
    assert!(connect_river_cities(&i.base, &i.links, &route.to_string()).is_err());
    let mut links: Value = serde_json::from_str(&i.links).expect("links");
    links["route_edge_closures"][0]["to"] = json!(1);
    assert!(connect_river_cities(&i.base, &links.to_string(), &i.route).is_err());
}
