//! External-pack regressions for installed-city preservation and stale contracts.

use std::path::PathBuf;
use std::sync::OnceLock;

use serde_json::Value;

use crate::authored_city_expansion::expand_authored_city;
use crate::authored_city_expansion_design::Expansion;
use crate::{CompiledGamePack, MapModule};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn source(variable: &str, fallback: &str) -> String {
    let path = std::env::var_os(variable)
        .map(PathBuf::from)
        .unwrap_or_else(|| root().join(fallback));
    std::fs::read_to_string(path).expect("actual hand-authored city input")
}
fn layout() -> String {
    source(
        "CRYSTAL_CITY_EXPANSION_LAYOUT",
        "planning/minnesota/cities/MnMinneapolis.layout.json",
    )
}
fn contract() -> String {
    source(
        "CRYSTAL_CITY_EXPANSION_CONTRACT",
        "planning/minnesota/cities/MnMinneapolis.expansion.json",
    )
}
fn base() -> &'static CompiledGamePack {
    static PACK: OnceLock<CompiledGamePack> = OnceLock::new();
    PACK.get_or_init(|| {
        let path = std::env::var_os("CRYSTAL_CITY_EXPANSION_BASE")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                root().join("content-packs/minnesota-northstar-itasca-quality.crystalpack")
            });
        crate::read_verified_compiled_game_pack(path).expect("completed pre-expansion city pack")
    })
}
fn expanded() -> &'static CompiledGamePack {
    static PACK: OnceLock<CompiledGamePack> = OnceLock::new();
    PACK.get_or_init(|| {
        expand_authored_city(base(), &layout(), &contract())
            .expect("native hand-authored city expansion")
    })
}

fn assert_program_prefixes(before: &MapModule, after: &MapModule, plan: &Expansion) {
    let before = serde_json::to_value(before).expect("native module");
    let after = serde_json::to_value(after).expect("expanded module");
    for (field, old) in before.as_object().expect("module fields") {
        if matches!(
            field.as_str(),
            "id" | "attributes" | "blocks" | "objects" | "events" | "map_event_section_commands"
        ) {
            continue;
        }
        let new = &after[field];
        match (old, new) {
            (Value::Array(old), Value::Array(new)) => {
                assert!(new.starts_with(old), "old compiled {field} changed")
            }
            (Value::Object(old), Value::Object(new)) => {
                for (label, value) in old {
                    if matches!(field.as_str(), "scripts" | "script_text_bodies")
                        && plan.text_revisions.iter().any(|r| r.label == *label)
                    {
                        continue;
                    }
                    assert_eq!(new.get(label), Some(value), "old {field}/{label} changed");
                }
            }
            _ => assert_eq!(new, old, "old module field {field} changed"),
        }
    }
}

#[test]
#[ignore = "requires the completed external pre-expansion city pack and exact authored source inputs"]
fn city_expansion_preserves_existing_rooms_programs_and_neighbors() {
    let plan: Expansion = serde_json::from_str(&contract()).expect("expansion contract");
    let result = expanded();
    let old_city = &base().data.maps[&plan.city];
    let city = &result.data.maps[&plan.city];
    assert_eq!(
        result.data.maps.len(),
        base().data.maps.len() + plan.rooms.len()
    );
    assert_eq!([city.attributes.width, city.attributes.height], [64, 48]);
    assert_program_prefixes(old_city, city, &plan);
    assert_eq!(
        city.objects[..old_city.objects.len()]
            .iter()
            .map(|o| &o.object_identifier)
            .collect::<Vec<_>>(),
        old_city
            .objects
            .iter()
            .map(|o| &o.object_identifier)
            .collect::<Vec<_>>()
    );
    for (name, original) in &base().data.maps {
        if name == &plan.city {
            continue;
        }
        let mut current = result.data.maps[name].clone();
        if plan.connections.iter().any(|c| c.target == *name) {
            current.attributes.connections = original.attributes.connections.clone();
        }
        assert_eq!(&current, original, "existing room/map {name} changed");
    }
    for (id, trainer) in &base().data.trainers.trainers {
        assert_eq!(result.data.trainers.trainers.get(id), Some(trainer));
    }
    for (id, mart) in &base().data.marts.0 {
        assert_eq!(result.data.marts.0.get(id), Some(mart));
    }
    assert_eq!(result.data.audio, base().data.audio);
    assert_eq!(result.audio_manifest, base().audio_manifest);
    assert_eq!(result.data.pokemon, base().data.pokemon);
    assert_eq!(result.data.moves, base().data.moves);
    assert_eq!(result.data.items, base().data.items);
    assert_eq!(result.data.wild_encounters, base().data.wild_encounters);
    for (path, bytes) in &base().runtime_files {
        assert_eq!(
            result.runtime_files.get(path),
            Some(bytes),
            "original native art/runtime file {path} changed"
        );
    }
}

#[test]
#[ignore = "requires the completed external pre-expansion city pack and exact authored source inputs"]
fn city_expansion_rejects_stale_native_anchors_links_and_art() {
    let original: Value = serde_json::from_str(&contract()).expect("contract");
    for mutation in 0..4 {
        let mut contract = original.clone();
        match mutation {
            0 => contract["native_module_sha256"] = Value::String("0".repeat(64)),
            1 => contract["dimensions_metatiles"][0] = Value::from(95),
            2 => contract["anchor_overrides"][0]["before"][0] = Value::from(999),
            3 => contract["connections"][0]["old_offset"] = Value::from(999),
            _ => unreachable!("four deliberate corruptions"),
        }
        assert!(
            expand_authored_city(base(), &layout(), &contract.to_string()).is_err(),
            "accepted corrupted city contract {mutation}"
        );
    }
    let mut bad_art: Value = serde_json::from_str(&layout()).expect("layout");
    bad_art["source_pixels_modified"] = Value::Bool(true);
    assert!(expand_authored_city(base(), &bad_art.to_string(), &contract()).is_err());
    assert!(
        expand_authored_city(expanded(), &layout(), &contract()).is_err(),
        "accepted repeated expansion of a changed city"
    );
}
