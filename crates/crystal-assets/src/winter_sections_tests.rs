//! Fixture-backed native preservation and corruptions; no native pack is bundled.
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use super::{build_winter_sections_modpack, preserve};
use crate::CompiledGamePack;
use crate::winter_sections_design::{self as design, Plan, Route};
use crate::winter_sections_validation;

struct Fixture {
    base: CompiledGamePack,
    contract: String,
    sources: BTreeMap<String, String>,
}

fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository root");
        let contract =
            std::fs::read_to_string(root.join(
                "planning/minnesota/regions/ice/sections/source-v03/integration.contract.json",
            ))
            .expect("frozen winter contract");
        let plan = design::parse(&contract).expect("reviewed contract");
        let mut sources = BTreeMap::new();
        for route in &plan.routes {
            for name in route.sources.keys() {
                sources.insert(
                    name.clone(),
                    std::fs::read_to_string(root.join(name)).expect("frozen winter source"),
                );
            }
        }
        let path = PathBuf::from(
            std::env::var_os("CRYSTAL_WINTER_SECTIONS_TEST_PACK")
                .expect("completed winter region/mixed-art parent"),
        )
        .canonicalize()
        .expect("absolute completed pack");
        Fixture {
            base: crate::read_verified_compiled_game_pack(path).expect("verified native parent"),
            contract,
            sources,
        }
    })
}

fn built() -> &'static CompiledGamePack {
    static PACK: OnceLock<CompiledGamePack> = OnceLock::new();
    PACK.get_or_init(|| {
        let f = fixture();
        build_winter_sections_modpack(&f.base, &f.contract, &f.sources)
            .expect("exact winter sections")
    })
}

fn plan() -> Plan {
    design::parse(&fixture().contract).expect("frozen winter plan")
}

#[test]
#[ignore = "requires completed CRYSTAL_WINTER_SECTIONS_TEST_PACK"]
fn winter_sections_retains_all_old_gameplay_catalogues_audio_and_original_river() {
    let base = &fixture().base;
    let actual = built();
    let mut changed = Vec::new();
    for (name, old) in &base.data.maps {
        let new = &actual.data.maps[name];
        if new != old {
            changed.push(name.as_str());
            let mut normalized = new.clone();
            normalized.blocks = old.blocks.clone();
            normalized.attributes.tileset_name = old.attributes.tileset_name.clone();
            assert_eq!(
                &normalized, old,
                "all old indexed warps, programs and actors: {name}"
            );
        }
        if name.starts_with("MnMississippi")
            || name.contains("Cave")
            || name.starts_with("MnRainyRiver")
        {
            assert_eq!(new, old, "unrelated river/room/new region: {name}");
        }
    }
    assert_eq!(
        changed,
        ["MnR11C00R00", "MnR11C00R01", "MnR17C00R00", "MnR17C00R01"]
    );
    assert_eq!(actual.data.map_scripts, base.data.map_scripts);
    assert_eq!(actual.data.npcs, base.data.npcs);
    assert_eq!(actual.data.wild_encounters, base.data.wild_encounters);
    assert_eq!(
        actual.data.runtime_spawn_points,
        base.data.runtime_spawn_points
    );
    assert_eq!(actual.data.field_moves, base.data.field_moves);
    assert_eq!(actual.compiled_audio, base.compiled_audio);
    assert_eq!(actual.audio_manifest, base.audio_manifest);
    assert_eq!(actual.runtime_files, base.runtime_files);
}

#[test]
#[ignore = "requires completed CRYSTAL_WINTER_SECTIONS_TEST_PACK"]
fn winter_sections_rejects_stale_native_modules_sources_and_art() {
    let f = fixture();
    let mut damaged = f.sources.clone();
    let name = plan().routes[0].candidate_layout.clone();
    damaged.get_mut(&name).expect("candidate").push(' ');
    assert!(build_winter_sections_modpack(&f.base, &f.contract, &damaged).is_err());
    assert!(
        build_winter_sections_modpack(&f.base, &(f.contract.clone() + " "), &f.sources).is_err()
    );
    let mut damaged = f.base.clone();
    damaged
        .data
        .maps
        .get_mut("MnR17C00R00")
        .expect("old chunk")
        .objects[0]
        .x += 1;
    assert!(build_winter_sections_modpack(&damaged, &f.contract, &f.sources).is_err());
    let mut damaged = f.base.clone();
    damaged
        .data
        .tilesets
        .get_mut("mn_winter_outdoor")
        .expect("winter art")
        .collision
        .insert("3d".into(), vec!["FLOOR".into(); 4]);
    assert!(build_winter_sections_modpack(&damaged, &f.contract, &f.sources).is_err());
}

fn damage_cell(
    route: &Route,
    before: &[u16],
    after: &[u16],
    at: [u16; 2],
    block: u16,
) -> (Route, Vec<u16>) {
    let mut value: Value = serde_json::from_str(&fixture().contract).expect("contract JSON");
    let source = value["routes"]
        .as_array_mut()
        .expect("routes")
        .iter_mut()
        .find(|v| v["id"] == route.id)
        .expect("route");
    let cells = source["changes"].as_array_mut().expect("changes");
    let index =
        usize::from(at[1]) * usize::from(route.dimensions_metatiles[0]) + usize::from(at[0]);
    if let Some(cell) = cells.iter_mut().find(|cell| cell["at"] == json!(at)) {
        cell["after"] = json!(block);
    } else {
        cells.push(json!({"at":at,"before":before[index],"after":block}));
    }
    let revised: Route = serde_json::from_value(source.clone()).expect("damaged route");
    let mut blocks = after.to_vec();
    blocks[index] = block;
    (revised, blocks)
}

#[test]
#[ignore = "requires completed CRYSTAL_WINTER_SECTIONS_TEST_PACK"]
fn winter_sections_geometry_rejects_real_gate_grass_water_and_slide_corruptions() {
    let f = fixture();
    for route in &plan().routes {
        let before = design::compile(&f.sources, route, true).expect("previous native source");
        let after = design::compile(&f.sources, route, false).expect("revised source");
        winter_sections_validation::geometry(&f.base, route, &before, &after)
            .expect("valid whole route");
        let mutations = if route.id == "MN_R17" {
            vec![
                ([18, 34], 0x3c),
                ([11, 34], 0x3c),
                ([11, 32], 0x3c),
                ([8, 11], 0x3c),
                ([0, 50], 0x3c),
                ([24, 14], 0x3e),
            ]
        } else {
            vec![
                ([10, 33], 0x85),
                ([20, 0], 5),
                ([30, 20], 0x85),
                ([22, 14], 0x83),
                ([0, 50], 0x83),
                ([20, 4], 0x85),
            ]
        };
        for (at, block) in mutations {
            let (damaged, blocks) = damage_cell(route, &before, &after, at, block);
            assert!(
                winter_sections_validation::geometry(&f.base, &damaged, &before, &blocks).is_err(),
                "must reject actual geometry corruption after updating declared delta: {} {at:?}",
                route.id
            );
        }
    }
}

#[test]
#[ignore = "requires completed CRYSTAL_WINTER_SECTIONS_TEST_PACK"]
fn winter_sections_wall_stops_require_real_ordinary_turns_and_complete_entry_inventory() {
    let f = fixture();
    let plan = plan();
    let route = &plan.routes[1];
    let before = design::compile(&f.sources, route, true).expect("before");
    let after = design::compile(&f.sources, route, false).expect("after");
    let mut value: Value = serde_json::from_str(&f.contract).expect("contract");
    let courses = value["routes"][1]["slide_courses"]
        .as_array_mut()
        .expect("courses");
    let wall = courses
        .iter_mut()
        .find(|c| c["stop_kind"] == "intentional_wall_stop_on_ICE")
        .expect("native wall stop");
    wall["ordinary_turn_exits"] = json!([]);
    let damaged: Route = serde_json::from_value(value["routes"][1].clone()).expect("route");
    assert!(winter_sections_validation::geometry(&f.base, &damaged, &before, &after).is_err());
    let mut value: Value = serde_json::from_str(&f.contract).expect("contract");
    value["routes"][1]["slide_courses"]
        .as_array_mut()
        .expect("courses")
        .pop();
    let damaged: Route = serde_json::from_value(value["routes"][1].clone()).expect("route");
    assert!(winter_sections_validation::geometry(&f.base, &damaged, &before, &after).is_err());
}

#[test]
#[ignore = "requires completed CRYSTAL_WINTER_SECTIONS_TEST_PACK"]
fn winter_sections_residual_rejects_unrelated_program_port_recovery_and_audio_changes() {
    let base = &fixture().base;
    let actual = built();
    let expected = actual
        .data
        .maps
        .iter()
        .filter(|(name, map)| base.data.maps.get(*name) != Some(*map))
        .map(|(name, map)| (name.clone(), map.clone()))
        .collect::<BTreeMap<_, _>>();
    preserve(base, actual, &expected).expect("exact residual");
    let mut bad = actual.clone();
    bad.data
        .map_scripts
        .insert("UnrelatedWinterScript".into(), json!([]));
    assert!(preserve(base, &bad, &expected).is_err());
    let mut bad = actual.clone();
    bad.data
        .maps
        .get_mut("MnR20C00R00")
        .expect("old region neighbor")
        .attributes
        .connections
        .clear();
    assert!(preserve(base, &bad, &expected).is_err());
    let mut bad = actual.clone();
    bad.data
        .runtime_spawn_points
        .values_mut()
        .next()
        .expect("native recovery")
        .tile_x += 1;
    assert!(preserve(base, &bad, &expected).is_err());
    let mut bad = actual.clone();
    let audio = bad
        .audio_manifest
        .music
        .values_mut()
        .next()
        .expect("actual modular audio descriptor");
    audio.pcm_frame_count = Some(audio.pcm_frame_count.expect("source PCM frame count") + 1);
    assert!(preserve(base, &bad, &expected).is_err());
}

#[test]
#[ignore = "requires completed CRYSTAL_WINTER_SECTIONS_TEST_PACK"]
fn winter_sections_written_roundtrip_retains_complete_native_pack() {
    let pack = built();
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "geothite-winter-sections-{}-{stamp}.crystalpack",
        std::process::id()
    ));
    pack.write_preserving_storage(&path)
        .expect("write fresh external test pack");
    let loaded = crate::read_verified_compiled_game_pack(&path).expect("strict written readback");
    std::fs::remove_file(path).expect("remove own temporary native pack");
    assert_eq!(&loaded, pack);
}
