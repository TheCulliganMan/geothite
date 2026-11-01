//! Exact regional publication, native quadrant copies and bounded service changes.
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{
    CompiledGamePack, build_north_shore_story_modpack, read_verified_compiled_game_pack,
};
use serde_json::{Value, json};

struct Input {
    base: CompiledGamePack,
    result: CompiledGamePack,
    source: String,
}
fn input() -> &'static Input {
    static INPUT: OnceLock<Input> = OnceLock::new();
    INPUT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository");
        let base = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-northstar-itasca-story.crystalpack"),
        )
        .expect("Itasca input");
        let source = std::fs::read_to_string(
            root.join("planning/minnesota/towns/MnGrandMarais.connections.json"),
        )
        .expect("authored connections");
        let result =
            build_north_shore_story_modpack(&base, &source).expect("native shore publication");
        let written = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-northstar-north-shore-story.crystalpack"),
        )
        .expect("written shore");
        assert_eq!(
            result.identity().expect("identity"),
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
#[ignore = "requires completed external Itasca and North Shore story packs"]
fn north_shore_story_preserves_unrelated_content_and_reconstructs_native_maps() {
    let i = input();
    let old = i.base.data();
    let new = i.result.data();
    let changed = [
        "MnGrandMarais",
        "MnR11C00R00",
        "MnR12C01R01",
        "MnGrandMaraisFieldStation",
        "MnDuluthPokecenter1F",
        "MnGrandMaraisPokecenter",
    ];
    assert_eq!(new.maps.len(), old.maps.len() + 1);
    for (name, m) in &old.maps {
        let actual = &new.maps[name];
        if !changed.contains(&name.as_str()) {
            assert_eq!(actual, m, "unrelated map {name}");
            continue;
        }
        assert_eq!(&actual.objects[..m.objects.len()], &m.objects);
        for (label, body) in &m.scripts {
            if ![
                "MnGrandMaraisWelcome",
                "MnGrandMaraisWelcomeText",
                "MnGrandMaraisPartnerText",
                "MN_DULUTH_NURSE",
                "MnGrandMaraisNurse",
            ]
            .contains(&label.as_str())
            {
                assert_eq!(actual.scripts.get(label), Some(body), "old script {label}");
            }
        }
        assert_eq!(
            &actual.events.warps[..m.events.warps.len()],
            &m.events.warps
        );
    }
    let source: Value = serde_json::from_str(&i.source).expect("source");
    let mut expected = old.maps["MnGrandMarais"].blocks.clone();
    let mut road = old.maps["MnR11C00R00"].blocks.clone();
    for p in source["patches"].as_array().expect("patches") {
        let x = p["metatile"][0].as_u64().expect("x") as usize;
        let y = p["metatile"][1].as_u64().expect("y") as usize;
        let value = u16::from_str_radix(p["block"].as_str().expect("block"), 16).expect("hex");
        if p["map"] == "MnGrandMarais" {
            expected[y * 20 + x] = value;
        } else {
            road[y * 32 + x] = value;
        }
    }
    assert_eq!(new.maps["MnGrandMarais"].blocks, expected);
    assert_eq!(new.maps["MnR11C00R00"].blocks, road);
    assert_eq!(
        new.maps["MnGrandMaraisShoreGate"].blocks,
        old.maps["IlexForestAzaleaGate"].blocks
    );
    assert!(
        new.maps["MnGrandMaraisFieldStation"]
            .gift_pokemon_scripts
            .is_empty()
    );
    let welcome = &new.maps["MnGrandMaraisFieldStation"].scripts["MnGrandMaraisWelcome"];
    assert!(
        !welcome.to_string().contains("givepoke")
            && !welcome.to_string().contains("POKE_BALL")
            && !welcome.to_string().contains("OLD_ROD")
    );
    for name in changed.into_iter().chain(["MnGrandMaraisShoreGate"]) {
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
        let a = serde_json::to_value(&rebuilt).expect("rebuilt module");
        let b = serde_json::to_value(expected).expect("expected module");
        let differing = b
            .as_object()
            .expect("module object")
            .iter()
            .filter(|(k, v)| a.get(*k) != Some(*v))
            .map(|(k, _)| k.as_str())
            .collect::<Vec<_>>();
        assert!(
            differing.is_empty(),
            "uncached {name}: differing fields {differing:?}"
        );
    }
    for (key, old_spawn) in &old.runtime_spawn_points {
        assert_eq!(
            new.runtime_spawn_points.get(key),
            Some(old_spawn),
            "original HOME/recovery"
        );
    }
    assert_eq!(
        new.runtime_spawn_points.len(),
        old.runtime_spawn_points.len() + 2
    );
    for (name, x, y) in [
        ("MnDuluthPokecenter1F", 10, 13),
        ("MnGrandMaraisPokecenter", 3, 5),
    ] {
        let spawns = new
            .runtime_spawn_points
            .values()
            .filter(|s| s.map_name == name)
            .collect::<Vec<_>>();
        assert_eq!(spawns.len(), 1);
        assert_eq!((spawns[0].tile_x, spawns[0].tile_y), (x, y));
    }
    for (k, v) in &old.story_event_script_constants.global {
        assert_eq!(new.story_event_script_constants.global.get(k), Some(v));
    }
    for (k, v) in i.base.runtime_files() {
        assert_eq!(
            i.result.runtime_files().get(k),
            Some(v),
            "original source file {k}"
        );
    }
    assert_eq!(i.result.compiled_audio(), i.base.compiled_audio());
    let mut restored = new.clone();
    restored.maps = old.maps.clone();
    restored.map_scripts = old.map_scripts.clone();
    restored.map_blocks = old.map_blocks.clone();
    restored.map_dimensions = old.map_dimensions.clone();
    restored.map_attributes = old.map_attributes.clone();
    restored.npcs = old.npcs.clone();
    restored.runtime_map_metadata = old.runtime_map_metadata.clone();
    restored.story_event_script_constants = old.story_event_script_constants.clone();
    restored.pokegear_landmarks = old.pokegear_landmarks.clone();
    restored.tilesets = old.tilesets.clone();
    restored.runtime_spawn_points = old.runtime_spawn_points.clone();
    for (name, table) in &old.wild_encounters {
        if name == "MnGrandMarais" {
            let mut expected = table.clone();
            expected.water = old.wild_encounters["MnR11C00R00"].water.clone();
            expected.water_rate = old.wild_encounters["MnR11C00R00"].water_rate;
            assert_eq!(&new.wild_encounters[name], &expected);
        } else {
            assert_eq!(&new.wild_encounters[name], table);
        }
    }
    restored.wild_encounters = old.wild_encounters.clone();
    // The new tileset inherits Johto's existing Cut/Whirlpool replacements.
    // Every original entry remains exact; no other field behavior changes.
    for (k, v) in &old.field_moves.cut.replacements {
        assert_eq!(new.field_moves.cut.replacements.get(k), Some(v));
    }
    assert_eq!(
        new.field_moves.cut.replacements.get("mn_marais_shore"),
        old.field_moves.cut.replacements.get("johto")
    );
    for (k, v) in &old.field_moves.whirlpool.replacements {
        assert_eq!(new.field_moves.whirlpool.replacements.get(k), Some(v));
    }
    assert_eq!(
        new.field_moves
            .whirlpool
            .replacements
            .get("mn_marais_shore"),
        old.field_moves.whirlpool.replacements.get("johto")
    );
    restored.field_moves = old.field_moves.clone();
    let a = serde_json::to_value(&restored).expect("new catalogs");
    let b = serde_json::to_value(old).expect("original catalogs");
    for (name, value) in b.as_object().expect("catalog object") {
        assert!(
            a.get(name) == Some(value),
            "unrelated catalog {name} differs"
        );
    }
}

#[test]
#[ignore = "requires completed external Itasca and North Shore story packs"]
fn north_shore_story_rejects_duplicate_unsafe_arrivals_repainted_land_and_extra_lamp_gate() {
    let i = input();
    for n in 0..5 {
        let mut s: Value = serde_json::from_str(&i.source).expect("source");
        match n {
            0 => s["gate_pairs"][0]["room_arrivals"][0] = json!([0, 4]),
            1 => s["reef_recipes"][0]["base"] = json!("01"),
            2 => s["local_lamp_required"] = json!(true),
            3 => s["patches"][0]["block"] = json!("35"),
            4 => s["recovery_clinics"][1]["floor"] = json!([3, 2]),
            _ => unreachable!(),
        }
        assert!(
            build_north_shore_story_modpack(&i.base, &s.to_string()).is_err(),
            "mutation {n}"
        );
    }
    assert!(build_north_shore_story_modpack(&i.result, &i.source).is_err());
}
