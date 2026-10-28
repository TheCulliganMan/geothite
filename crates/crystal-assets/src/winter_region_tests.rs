//! External native-pack tests; no exported source catalogs or build artifacts.
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;

use super::build_winter_region_modpack;
use crate::winter_region_design::{self as design, HUT, NEIGHBOR, OUTDOOR};
use crate::{CompiledGamePack, MapModule};

const LAYOUT: &str =
    include_str!("../../../planning/minnesota/regions/ice/MnRainyRiverIcefields.layout.json");
const DOSSIER: &str =
    include_str!("../../../planning/minnesota/regions/ice/MnRainyRiverIcefields.json");

fn fixture() -> CompiledGamePack {
    static BASE: OnceLock<CompiledGamePack> = OnceLock::new();
    BASE.get_or_init(|| {
        let path = PathBuf::from(
            std::env::var_os("CRYSTAL_WINTER_REGION_BASE")
                .expect("completed external art-added681-map parent required"),
        )
        .canonicalize()
        .expect("absolute completed winter parent");
        crate::read_verified_compiled_game_pack(path).expect("strict winter parent")
    })
    .clone()
}

fn built() -> (CompiledGamePack, CompiledGamePack) {
    let base = fixture();
    let pack = build_winter_region_modpack(&base, LAYOUT, DOSSIER).expect("native winter region");
    (base, pack)
}

#[test]
#[ignore = "requires completed external winter-art pack in CRYSTAL_WINTER_REGION_BASE"]
fn winter_region_native_maps_services_encounters_and_old_content() {
    let (base, pack) = built();
    assert_eq!(pack.data.maps.len(), 683);
    assert_eq!(
        pack.data.runtime_spawn_points,
        base.data.runtime_spawn_points
    );
    for name in [OUTDOOR, HUT] {
        assert_eq!(
            pack.data
                .pokegear_landmark_for_map(name)
                .expect("native landmark"),
            base.data
                .pokegear_landmark_for_map(NEIGHBOR)
                .expect("existing R20 landmark")
        );
    }
    let room = &pack.data.maps[HUT];
    assert!(room.script_runtime_commands.iter().any(|c| c.source_script
        == "MN_ICE_WARMING_KEEPER"
        && c.command_index == 4
        && c.command == "special"
        && c.args == ["HealParty"]));
    let wild = &pack.data.wild_encounters[OUTDOOR];
    assert_eq!(
        wild.grass_rates.as_ref().expect("icy grass rates")["day"],
        8
    );
    assert!(wild.water.is_none() && wild.water_rate.is_none());
    for name in base.data.maps.keys().filter(|n| n.as_str() != NEIGHBOR) {
        assert_eq!(
            pack.data.maps[name], base.data.maps[name],
            "entire old module {name}"
        );
    }
}

#[test]
#[ignore = "requires completed external winter-art pack in CRYSTAL_WINTER_REGION_BASE"]
fn winter_region_rejects_stale_sources_and_native_preimages() {
    let base = fixture();
    assert!(
        build_winter_region_modpack(&base, &LAYOUT.replacen("\"3f\"", "\"3c\"", 1), DOSSIER)
            .is_err()
    );
    assert!(
        build_winter_region_modpack(
            &base,
            LAYOUT,
            &DOSSIER.replacen("\"level\": 48", "\"level\": 1", 1)
        )
        .is_err()
    );
    let mut bad = base.clone();
    bad.data.maps.get_mut(NEIGHBOR).expect("R20").objects[0].x += 1;
    assert!(build_winter_region_modpack(&bad, LAYOUT, DOSSIER).is_err());
    let mut bad = base.clone();
    bad.data
        .runtime_map_metadata
        .values_mut()
        .next()
        .expect("native metadata")
        .group_id = 84;
    assert!(build_winter_region_modpack(&bad, LAYOUT, DOSSIER).is_err());
    let mut bad = base;
    bad.data
        .tilesets
        .get_mut("mn_winter_outdoor")
        .expect("winter art")
        .collision
        .insert(
            "3e".into(),
            vec![
                "FLOOR".into(),
                "FLOOR".into(),
                "FLOOR".into(),
                "FLOOR".into(),
            ],
        );
    assert!(build_winter_region_modpack(&bad, LAYOUT, DOSSIER).is_err());
}

#[test]
#[ignore = "requires completed external winter-art pack in CRYSTAL_WINTER_REGION_BASE"]
fn winter_region_geometry_rejects_unsafe_slides_boundaries_and_pairings() {
    let (_, pack) = built();
    let (_, region) = design::parse(LAYOUT, DOSSIER).expect("frozen source");
    for (at, block) in [
        ([4_u16, 0], 0x3e),
        ([18, 18], 0x3e),
        ([27, 18], 0x3e),
        ([9, 4], 0x3c),
        ([13, 18], 0x3f),
        ([4, 14], 0x3f),
        ([4, 13], 0x3f),
        ([9, 15], 0x3c),
        ([12, 18], 0x0f),
    ] {
        let mut map = pack.data.maps[OUTDOOR].clone();
        map.blocks[usize::from(at[1]) * 32 + usize::from(at[0])] = block;
        assert!(
            crate::winter_region_validation::geometry(&pack, &map, &pack.data.maps[HUT], &region)
                .is_err(),
            "accepted unsafe native cell {at:?}"
        );
    }
    let mut map = pack.data.maps[OUTDOOR].clone();
    map.objects[0].x = 60;
    map.objects[0].y = 44;
    assert!(
        crate::winter_region_validation::geometry(&pack, &map, &pack.data.maps[HUT], &region)
            .is_err()
    );
    let mut hut = pack.data.maps[HUT].clone();
    hut.events.warps[0].target_warp_id = 1;
    assert!(
        crate::winter_region_validation::geometry(&pack, &pack.data.maps[OUTDOOR], &hut, &region)
            .is_err()
    );
    let mut bad = pack;
    bad.data
        .maps
        .get_mut(NEIGHBOR)
        .expect("R20")
        .attributes
        .connections
        .retain(|c| c.direction != "west");
    assert!(crate::winter_region_validation::connections(&bad).is_err());
}

#[test]
#[ignore = "requires completed external winter-art pack in CRYSTAL_WINTER_REGION_BASE"]
fn winter_region_residual_rejects_unrelated_program_catalog_art_spawn_and_report_drift() {
    let (base, pack) = built();
    let (_, region) = design::parse(LAYOUT, DOSSIER).expect("frozen source");
    let expected = [OUTDOOR, HUT]
        .into_iter()
        .map(|n| (n.into(), pack.data.maps[n].clone()))
        .collect::<BTreeMap<String, MapModule>>();
    let constants = pack
        .data
        .story_event_script_constants
        .global
        .keys()
        .filter(|k| {
            !base
                .data
                .story_event_script_constants
                .global
                .contains_key(*k)
                && k.as_str() != design::GROUP_CONSTANT
        })
        .cloned()
        .collect::<Vec<_>>();
    crate::winter_region_validation::preserve(&base, &pack, &expected, &constants, &region)
        .expect("control residual");
    for case in 0..6 {
        let mut bad = pack.clone();
        match case {
            0 => bad
                .data
                .maps
                .get_mut("MnR01Cave1F")
                .expect("old cave")
                .scripts
                .clear(),
            1 => {
                bad.data
                    .story_event_script_constants
                    .global
                    .insert("EVENT_UNRELATED".into(), 1);
            }
            2 => {
                bad.runtime_files
                    .insert("data/tilesets/unrelated.bin".into(), vec![0]);
            }
            3 => {
                bad.data
                    .runtime_spawn_points
                    .values_mut()
                    .next()
                    .expect("old spawn")
                    .tile_x += 1
            }
            4 => bad.report.manifests.push("unrelated-manifest".into()),
            5 => {
                bad.data
                    .wild_encounters
                    .get_mut(NEIGHBOR)
                    .expect("old encounters")
                    .water_rate = Some(1);
            }
            _ => unreachable!("bounded cases"),
        }
        assert!(
            crate::winter_region_validation::preserve(&base, &bad, &expected, &constants, &region)
                .is_err(),
            "accepted residual drift {case}"
        );
    }
    assert_eq!(
        pack.data.pokemon.len(),
        251,
        "reserved Lutebrisk is not silently installed"
    );
}

#[test]
#[ignore = "requires completed external winter-art pack in CRYSTAL_WINTER_REGION_BASE"]
fn winter_region_true_uncached_and_written_roundtrip_preserve_complete_services() {
    let (_, pack) = built();
    let mut data = pack.data.clone();
    for name in [OUTDOOR, HUT] {
        data.maps.remove(name);
        assert_eq!(
            data.assemble_map_module_from_compiled_payloads(name)
                .expect("true uncached production map"),
            pack.data.maps[name]
        );
    }
    let directory =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.local/winter-region-tests");
    std::fs::create_dir_all(&directory).expect("ignored proof directory");
    let directory = directory.canonicalize().expect("absolute proof directory");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = directory.join(format!("winter-{}-{nonce}.crystalpack", std::process::id()));
    pack.write_preserving_storage(&path)
        .expect("native written pack");
    let written = crate::read_verified_compiled_game_pack(&path).expect("strict written roundtrip");
    assert_eq!(written, pack);
    assert_eq!(
        serde_json::to_value(&written.data.maps[HUT].script_runtime_commands)
            .expect("typed service rows"),
        serde_json::to_value(&pack.data.maps[HUT].script_runtime_commands)
            .expect("original typed service rows")
    );
    std::fs::remove_file(path).expect("remove only own temporary pack");
    assert_ne!(
        json!(written.identity().expect("identity").content_hash),
        json!("")
    );
}
