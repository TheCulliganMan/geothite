//! Exact new settlement and bounded road publication, including uncached maps.
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{
    CompiledGamePack, build_leech_shore_modpack, read_verified_compiled_game_pack,
};
use serde_json::{Value, json};

struct Input {
    base: CompiledGamePack,
    result: CompiledGamePack,
    source: [String; 4],
}
fn input() -> &'static Input {
    static INPUT: OnceLock<Input> = OnceLock::new();
    INPUT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository root");
        let base = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-northstar-lake-country.crystalpack"),
        )
        .expect("lake-country base");
        let source = [
            "towns/MnLeechLake.json",
            "towns/MnLeechLake.layout.json",
            "routes/MN_R07.json",
            "routes/MN_R08.json",
        ]
        .map(|p| {
            std::fs::read_to_string(root.join("planning/minnesota").join(p))
                .expect("authored shore")
        });
        let result =
            build_leech_shore_modpack(&base, &source[0], &source[1], &source[2], &source[3])
                .expect("shore publication");
        let written = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-northstar-leech-shore.crystalpack"),
        )
        .expect("written shore");
        assert_eq!(
            result.identity().expect("built identity"),
            written.identity().expect("written identity")
        );
        Input {
            base,
            result,
            source,
        }
    })
}
#[test]
#[ignore = "requires completed external lake-country and shore packs"]
fn leech_shore_preserves_gyms_story_audio_recovery_and_uncached_maps() {
    let i = input();
    let old = i.base.data();
    let new = i.result.data();
    let changed = [
        "MnBrainerd",
        "MnR07C00R00",
        "MnR07C00R01",
        "MnR08C00R00",
        "MnR08C00R01",
        "MnBemidji",
    ];
    let added = [
        "MnLeechLake",
        "MnLeechLakeCareCabin",
        "MnLeechLakeFisherHouse",
    ];
    assert_eq!(new.maps.len(), old.maps.len() + 3);
    for (name, map) in &old.maps {
        if !changed.contains(&name.as_str()) {
            assert_eq!(new.maps.get(name), Some(map), "unrelated map {name}");
            continue;
        }
        let actual = &new.maps[name];
        assert_eq!(actual.events.warps, map.events.warps);
        assert_eq!(&actual.objects[..map.objects.len()], map.objects.as_slice());
        for (label, body) in &map.scripts {
            assert_eq!(actual.scripts.get(label), Some(body), "old story {label}");
        }
        let expected = match name.as_str() {
            "MnBrainerd" => vec![16],
            "MnR07C00R00" => vec![11],
            "MnR07C00R01" => vec![23 * 32 + 11],
            "MnR08C00R00" => vec![17 * 32, 18 * 32],
            "MnR08C00R01" => vec![23 * 32 + 11, 23 * 32 + 12, 23 * 32 + 14, 23 * 32 + 15],
            "MnBemidji" => vec![12, 12 * 24, 12 * 24 + 23],
            _ => unreachable!(),
        };
        let actual_indices = map
            .blocks
            .iter()
            .zip(&actual.blocks)
            .enumerate()
            .filter_map(|(n, (a, b))| (a != b).then_some(n))
            .collect::<Vec<_>>();
        assert_eq!(actual_indices, expected, "only declared mouths/caps");
    }
    for name in changed.into_iter().chain(added) {
        let expected = &new.maps[name];
        let mut uncached = new.clone();
        uncached.maps.remove(name);
        let mut rebuilt = uncached
            .assemble_map_module_from_compiled_payloads(name)
            .expect("uncached reconstruction");
        for label in expected
            .attributes
            .map_scripts_label
            .iter()
            .chain(expected.attributes.map_events_label.iter())
        {
            if !expected.scripts.contains_key(label) {
                rebuilt.scripts.remove(label);
            }
        }
        assert_eq!(&rebuilt, expected, "uncached {name}");
    }
    for (key, meta) in &old.runtime_map_metadata {
        assert_eq!(new.runtime_map_metadata.get(key), Some(meta));
    }
    assert_eq!(
        new.runtime_map_metadata.len(),
        old.runtime_map_metadata.len() + 3
    );
    for (key, spawn) in &old.runtime_spawn_points {
        assert_eq!(new.runtime_spawn_points.get(key), Some(spawn));
    }
    let recovery = new
        .runtime_spawn_points
        .iter()
        .filter(|(k, _)| !old.runtime_spawn_points.contains_key(*k))
        .map(|(_, v)| v.map_name.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        recovery,
        ["MnLeechLake", "MnLeechLakeCareCabin"]
            .into_iter()
            .collect()
    );
    for (key, value) in &old.story_event_script_constants.global {
        assert_eq!(
            new.story_event_script_constants.global.get(key),
            Some(value)
        );
    }
    assert_eq!(
        new.story_event_script_constants.global.len(),
        old.story_event_script_constants.global.len() + 1
    );
    assert_eq!(
        new.story_event_script_constants.global["MAPGROUP_MINNESOTA_LEECH_SHORE"],
        78
    );
    assert_eq!(
        new.story_event_script_constants.maps,
        old.story_event_script_constants.maps
    );
    for (key, value) in &old.pokegear_landmarks.map_to_landmark {
        assert_eq!(new.pokegear_landmarks.map_to_landmark.get(key), Some(value));
    }
    assert!(
        new.pokegear_landmarks
            .landmarks
            .starts_with(&old.pokegear_landmarks.landmarks)
    );
    let mut restored = new.clone();
    restored.maps = old.maps.clone();
    restored.map_scripts = old.map_scripts.clone();
    restored.map_blocks = old.map_blocks.clone();
    restored.map_dimensions = old.map_dimensions.clone();
    restored.map_attributes = old.map_attributes.clone();
    restored.npcs = old.npcs.clone();
    restored.runtime_map_metadata = old.runtime_map_metadata.clone();
    restored.runtime_spawn_points = old.runtime_spawn_points.clone();
    restored.story_event_script_constants = old.story_event_script_constants.clone();
    restored.pokegear_landmarks = old.pokegear_landmarks.clone();
    assert_eq!(
        restored, *old,
        "all other catalogs/flags/encounters/HOME/recovery remain exact"
    );
    assert_eq!(i.result.runtime_files(), i.base.runtime_files());
    assert_eq!(i.result.compiled_audio(), i.base.compiled_audio());
}
#[test]
#[ignore = "requires completed external lake-country and shore packs"]
fn leech_shore_rejects_duplicates_carpet_arrivals_rail_gaps_counter_bodies_and_extra_gates() {
    let i = input();
    for mutation in 0..5 {
        let mut town: Value = serde_json::from_str(&i.source[0]).expect("town");
        let mut layout: Value = serde_json::from_str(&i.source[1]).expect("layout");
        let mut route: Value = serde_json::from_str(&i.source[2]).expect("route");
        match mutation {
            0 => town["doors"][0]["target_warp"] = json!(1),
            1 => town["interiors"][0]["residents"][0]["tile"] = json!([7,4]),
            2 => layout["layers"].as_array_mut().expect("layers").push(json!({"name":"broken pier end","kind":"point","position":[20,10],"block":"01"})),
            3 => layout["layers"].as_array_mut().expect("layers").push(json!({"name":"unbound edge","kind":"path","points":[[0,0],[16,0]],"width_metatiles":1,"material":"floor"})),
            4 => route["required_flags"] = json!(["BADGE_6","EVENT_MN_SURF_GIFT_RECEIVED"]), _ => unreachable!(),
        }
        assert!(
            build_leech_shore_modpack(
                &i.base,
                &town.to_string(),
                &layout.to_string(),
                &route.to_string(),
                &i.source[3]
            )
            .is_err(),
            "mutation {mutation}"
        );
    }
    assert!(
        build_leech_shore_modpack(
            &i.result,
            &i.source[0],
            &i.source[1],
            &i.source[2],
            &i.source[3]
        )
        .is_err()
    );
}
