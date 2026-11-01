//! Compact fair publication preserves the city, river and original room stories.
use std::path::PathBuf;

use crystal_core::map::MapConnection;
use serde_json::{Value, json};

use crate::{CompiledGamePack, read_verified_compiled_game_pack};

fn pack(name: &str) -> CompiledGamePack {
    read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os(name).expect("external fair fixture"),
    ))
    .expect("strict fair fixture")
}

#[test]
#[ignore = "requires compact CRYSTAL_FAIR_BASE/TEST_PACK/SOURCE/LAYOUT external fixtures"]
fn state_fair_woodland_preserves_the_region_and_rejects_false_entrances() {
    let base = pack("CRYSTAL_FAIR_BASE");
    let child = pack("CRYSTAL_FAIR_TEST_PACK");
    let source = std::fs::read_to_string(
        std::env::var_os("CRYSTAL_FAIR_SOURCE").expect("candidate dossier"),
    )
    .expect("dossier source");
    let layout =
        std::fs::read_to_string(std::env::var_os("CRYSTAL_FAIR_LAYOUT").expect("candidate layout"))
            .expect("layout source");
    let rebuilt = super::build_state_fair_modpack(&base, &layout, &source)
        .expect("deterministic fair publication");
    assert!(
        rebuilt == child,
        "strict written pack equals the Rust replay"
    );
    assert_eq!(child.data.maps.len(), base.data.maps.len() + 4);
    assert_eq!(base.compiled_audio(), child.compiled_audio());
    for (name, bytes) in &base.runtime_files {
        assert_eq!(
            child.runtime_files.get(name),
            Some(bytes),
            "original source file {name}"
        );
    }
    for (name, original) in &base.data.maps {
        let mut expected = original.clone();
        if name == "MnR05C00R02" {
            for x in 0..9 {
                assert_eq!(expected.blocks[4 * 32 + x], 0x05);
                expected.blocks[4 * 32 + x] = 0x01;
            }
            expected.attributes.connections.push(MapConnection {
                direction: "west".into(),
                target_map: "MnStateFairgrounds".into(),
                offset: -28,
            });
            expected.attributes.connection_flags = Some("NORTH | SOUTH | EAST | WEST".into());
        }
        assert!(
            child.data.maps[name] == expected,
            "all other native map content exact: {name}"
        );
    }
    for (name, body) in &base.data.map_scripts {
        assert_eq!(
            child.data.map_scripts.get(name),
            Some(body),
            "original raw program {name}"
        );
    }
    for (name, actors) in &base.data.npcs {
        assert_eq!(
            child.data.npcs.get(name),
            Some(actors),
            "original actors {name}"
        );
    }
    assert_eq!(base.data.pokemon, child.data.pokemon);
    assert_eq!(base.data.moves, child.data.moves);
    assert_eq!(
        base.data.runtime_title_screen,
        child.data.runtime_title_screen
    );
    for (name, definition) in &base.data.tilesets {
        assert_eq!(child.data.tilesets.get(name), Some(definition));
    }
    for (name, item) in &base.data.items {
        assert_eq!(child.data.items.get(name), Some(item));
    }
    assert_eq!(child.data.items.len(), base.data.items.len() + 16);
    for (name, trainer) in &base.data.trainers.trainers {
        assert_eq!(child.data.trainers.trainers.get(name), Some(trainer));
    }
    for (name, encounters) in &base.data.wild_encounters {
        assert_eq!(child.data.wild_encounters.get(name), Some(encounters));
    }
    assert_eq!(
        child.data.wild_encounters.len(),
        base.data.wild_encounters.len() + 1
    );
    for name in [
        "MnR05C00R02",
        "MnStateFairgrounds",
        "MnFairGrandstand",
        "MnFairBarn",
        "MnFairDairy",
    ] {
        assert_eq!(
            child
                .data
                .assemble_map_module_from_compiled_payloads(name)
                .expect("native split-payload reconstruction"),
            child.data.maps[name]
        );
    }
    let dossier: Value = serde_json::from_str(&source).expect("authored dossier");
    for index in 0..6 {
        let mut bad = dossier.clone();
        match index {
            0 => bad["native_city_connection"]["fair_to_city_offset"] = json!(27),
            1 => bad["native_city_connection"]["edits"][0]["position"] = json!([0, 3]),
            2 => bad["native_city_connection"]["edits"][0]["after"] = json!("02"),
            3 => bad["native_map"]["interiors"][2]["residents"][1]["tile"] = json!([6, 2]),
            4 => {
                bad["native_map"]["interiors"][2]["residents"][1]["sprite"] = json!("SPRITE_TAUROS")
            }
            _ => bad["native_trainers"][0]["party"][0]["species"] = json!("MEW"),
        }
        assert!(
            super::build_state_fair_modpack(&base, &layout, &bad.to_string()).is_err(),
            "reject unsafe woodland/source contract {index}"
        );
    }
    let mut opened: Value = serde_json::from_str(&layout).expect("native layout");
    opened["layers"]
        .as_array_mut()
        .expect("explicit terrain")
        .push(json!({
            "name":"invalid old gate", "kind":"point", "position":[8,35], "block":"01"
        }));
    assert!(
        super::build_state_fair_modpack(&base, &opened.to_string(), &source).is_err(),
        "the old unconnected city-side entrance must stay visibly closed"
    );
    assert!(
        super::build_state_fair_modpack(&child, &layout, &source).is_err(),
        "duplicate installation"
    );
}
