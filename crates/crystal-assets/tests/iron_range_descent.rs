//! Frozen carrier bench and gallery publication preserves the existing world.
use std::fs;
use std::path::PathBuf;

use crystal_assets::{
    CompiledGamePack, build_iron_range_descent_modpack, build_river_valley_modpack,
    read_verified_compiled_game_pack,
};
use serde_json::{Value, json};

fn inputs() -> (CompiledGamePack, String, String) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository path");
    let source = read_verified_compiled_game_pack(root.join("content-packs/text-tui.crystalpack"))
        .expect("supplied external source pack");
    let read = |name: &str| {
        fs::read_to_string(root.join("planning/minnesota/routes").join(name))
            .expect("authored route input")
    };
    let base =
        build_river_valley_modpack(&source, &read("MN_R03.layout.json"), &read("MN_R03.json"))
            .expect("existing River Valley overlay");
    (base, read("MN_R10.layout.json"), read("MN_R10.json"))
}

fn collision(pack: &CompiledGamePack, x: usize, y: usize) -> &str {
    let name = format!("MnR10C{:02}R{:02}", x / 64, y / 48);
    let module = &pack.data().maps[&name];
    let block = module.blocks[(y % 48) / 2 * usize::from(module.attributes.width) + (x % 64) / 2];
    &pack.data().tilesets["johto"].collision[&format!("{block:02x}")][y % 2 * 2 + x % 2]
}

#[test]
fn iron_range_descent_composes_with_river_valley_and_preserves_source_content() {
    let (base, layout, dossier) = inputs();
    let result = build_iron_range_descent_modpack(&base, &layout, &dossier)
        .expect("complete Iron Range Descent overlay");
    assert_eq!(result.data().maps.len(), base.data().maps.len() + 3);
    for (name, module) in &base.data().maps {
        assert_eq!(
            result.data().maps.get(name),
            Some(module),
            "changed map {name}"
        );
    }
    for (name, bytes) in base.runtime_files() {
        assert_eq!(
            result.runtime_files().get(name),
            Some(bytes),
            "changed source file {name}"
        );
    }
    for (name, trainer) in &base.data().trainers.trainers {
        assert_eq!(
            result.data().trainers.trainers.get(name),
            Some(trainer),
            "changed source opponent {name}"
        );
    }
    assert_eq!(result.data().pokemon, base.data().pokemon);
    assert_eq!(result.data().items, base.data().items);
    assert_eq!(result.data().marts, base.data().marts);
    assert_eq!(result.data().tilesets, base.data().tilesets);
    assert_eq!(
        result.data().runtime_spawn_points,
        base.data().runtime_spawn_points,
        "source HOME/recovery destinations changed"
    );
    assert_eq!(
        result.data().story_event_script_constants.global["SPAWN_HOME"],
        base.data().story_event_script_constants.global["SPAWN_HOME"]
    );
    assert_eq!(result.compiled_audio(), base.compiled_audio());
    assert_eq!(
        result.audio_manifest().expect("result audio manifest"),
        base.audio_manifest().expect("source audio manifest")
    );

    let west = &result.data().maps["MnR10C00R00"];
    let east = &result.data().maps["MnR10C01R00"];
    for (module, width, trainers, objects, signs) in [(west, 32, 4, 8, 4), (east, 16, 2, 4, 2)] {
        assert_eq!(
            (module.attributes.width, module.attributes.height),
            (width, 20)
        );
        assert_eq!(module.blocks.len(), usize::from(width) * 20);
        assert_eq!(module.attributes.tileset_name, "johto");
        assert_eq!(
            module.attributes.map_group_constant.as_deref(),
            Some("MAPGROUP_MINNESOTA_IRON_RANGE_DESCENT")
        );
        assert_eq!(module.trainer_scripts.len(), trainers);
        assert_eq!(module.objects.len(), objects);
        assert_eq!(module.events.bg_events.len(), signs);
        assert_eq!(
            result
                .data()
                .assemble_map_module_from_compiled_payloads(&module.id)
                .expect("split native module"),
            *module
        );
        assert!(result.data().wild_encounters.contains_key(&module.id));
        assert_eq!(module.attributes.connections.len(), 1);
    }
    let west_connection = &west.attributes.connections[0];
    assert_eq!(west_connection.direction, "east");
    assert_eq!(west_connection.target_map, east.id);
    assert_eq!(west_connection.offset, 0);
    let east_connection = &east.attributes.connections[0];
    assert_eq!(east_connection.direction, "west");
    assert_eq!(east_connection.target_map, west.id);
    assert_eq!(east_connection.offset, 0);
    assert_eq!(west.blocks[10 * 32 + 5], 0x74, "actual two-way ramp");
    assert_eq!(west.blocks[10 * 32 + 12], 0x56, "native south ledge");
    assert_eq!(west.blocks[10 * 32 + 13], 0x56, "native south ledge");
    assert_eq!(collision(&result, 25, 20), "HOP_DOWN");
    assert_eq!(collision(&result, 25, 21), "WALL");
    assert_eq!(collision(&result, 25, 22), "FLOOR");
    assert_eq!(collision(&result, 11, 20), "FLOOR");
    assert_eq!(collision(&result, 11, 21), "FLOOR");
    for y in 18..22 {
        for x in 50..56 {
            assert_eq!(collision(&result, x, y), "TALL_GRASS");
        }
    }

    assert_eq!(west.events.warps.len(), 2);
    assert!(east.events.warps.is_empty());
    assert_eq!((west.events.warps[0].x, west.events.warps[0].y), (36, 17));
    assert_eq!((west.events.warps[1].x, west.events.warps[1].y), (36, 18));
    assert_eq!(collision(&result, 36, 17), "CAVE");
    assert_eq!(collision(&result, 36, 18), "FLOOR");
    let room = &result.data().maps["MnR10Cave1F"];
    assert_eq!((room.attributes.width, room.attributes.height), (12, 8));
    assert_eq!(room.attributes.tileset_name, "cave");
    assert_eq!(room.blocks.len(), 96);
    assert_eq!(room.objects.len(), 2);
    assert_eq!(room.script_field_pickups.len(), 1);
    assert_eq!(room.events.warps.len(), 1);
    assert_eq!((room.events.warps[0].x, room.events.warps[0].y), (3, 13));
    assert_eq!(room.events.warps[0].target_warp_id, 2);
    assert_eq!(
        room.events.warps[0].target_map_constant,
        west.attributes
            .map_constant
            .as_deref()
            .expect("exterior constant")
    );
    let design: Value = serde_json::from_str(&dossier).expect("frozen dossier");
    for key in ["trainers", "residents", "items"] {
        for authored in design[key].as_array().expect("authored actor list") {
            let id = authored["id"].as_str().expect("actor ID");
            let x = authored["tile"][0].as_u64().expect("core x");
            let y = authored["tile"][1].as_u64().expect("core y");
            let name = format!("MnR10C{:02}R{:02}", x / 64, y / 48);
            let module = &result.data().maps[&name];
            let actor = module
                .objects
                .iter()
                .find(|actor| actor.script == id)
                .expect("compiled actor");
            assert_eq!(
                (i64::from(actor.x), i64::from(actor.y)),
                (
                    i64::try_from(x % 64).expect("local x"),
                    i64::try_from(y % 48).expect("local y")
                ),
                "misplaced actor {id}"
            );
            if key == "items" {
                assert_eq!(
                    actor.event_flag,
                    authored["flag"].as_str().expect("persistent pickup flag")
                );
            }
        }
    }
    let expected_room: Vec<u16> = design["interiors"][0]["rows"]
        .as_array()
        .expect("room rows")
        .iter()
        .flat_map(|row| row.as_str().expect("room row").split_whitespace())
        .map(|block| u16::from_str_radix(block, 16).expect("source block ID"))
        .collect();
    assert_eq!(room.blocks, expected_room);
    for (id, x, y) in [("MN_R10_GALLERY_READER", 9, 5), ("MN_R10_ITEM_04", 17, 5)] {
        let actor = room
            .objects
            .iter()
            .find(|actor| actor.script == id)
            .expect("gallery actor");
        assert_eq!((actor.x, actor.y), (x, y));
    }
    let expected_parties: [[(&str, u8); 2]; 6] = [
        [("GRAVELER", 34), ("MACHOKE", 34)],
        [("SLUGMA", 34), ("VULPIX", 35)],
        [("PIDGEOTTO", 35), ("SKARMORY", 36)],
        [("MAGNEMITE", 35), ("MACHOKE", 35)],
        [("GLIGAR", 36), ("ONIX", 35)],
        [("STANTLER", 36), ("TEDDIURSA", 36)],
    ];
    for (index, expected) in expected_parties.iter().enumerate() {
        let id = format!("MN_R10_T{:02}", index + 1);
        let trainer = &result.data().trainers.trainers[&id];
        let actual: Vec<_> = trainer
            .party
            .iter()
            .map(|member| (member.species.as_str(), member.level))
            .collect();
        assert_eq!(actual.as_slice(), expected, "changed authored party {id}");
    }

    let mut perimeter = 0;
    let mut buoys = 0;
    for y in 0..40 {
        for x in 0..96 {
            let terrain = collision(&result, x, y);
            assert_ne!(terrain, "CUT_TREE", "decorative Cut at {x},{y}");
            if x == 0 || x == 95 || y == 0 || y == 39 {
                perimeter += 1;
                assert_ne!(terrain, "WATER", "open outer Surf edge at {x},{y}");
                buoys += usize::from(terrain == "BUOY");
            }
        }
    }
    assert_eq!((perimeter, buoys), (268, 65));
    assert_eq!(collision(&result, 73, 29), "FLOOR");
    assert_eq!(collision(&result, 73, 30), "WATER");
    for y in 28..39 {
        assert_eq!(collision(&result, 63, y), "WATER");
        assert_eq!(collision(&result, 64, y), "WATER");
    }
    assert!(build_iron_range_descent_modpack(&result, &layout, &dossier).is_err());
}

#[test]
fn iron_range_descent_rejects_changed_progression_and_broken_native_geometry() {
    let (base, layout, dossier) = inputs();
    for flags in [
        json!(["BADGE_7"]),
        json!(["BADGE_6", "EVENT_MN_DEPOT_CLEARED"]),
    ] {
        let mut changed: Value = serde_json::from_str(&dossier).expect("dossier JSON");
        changed["required_flags"] = flags;
        assert!(
            build_iron_range_descent_modpack(&base, &layout, &changed.to_string()).is_err(),
            "changed Q06 release conditions accepted"
        );
    }
    for (name, layer) in [
        (
            "native ramp removed",
            json!({"kind":"point","position":[5,10],"block":"72"}),
        ),
        (
            "native ledge flattened",
            json!({"kind":"point","position":[12,10],"block":"01"}),
        ),
        (
            "required habitat removed",
            json!({"kind":"rect","material":"floor","bounds":[25,9,28,11]}),
        ),
    ] {
        let mut changed: Value = serde_json::from_str(&layout).expect("layout JSON");
        changed["layers"]
            .as_array_mut()
            .expect("terrain layers")
            .push(layer);
        assert!(
            build_iron_range_descent_modpack(&base, &changed.to_string(), &dossier).is_err(),
            "{name} accepted"
        );
    }
    let changes = [
        (
            "wall-face landing",
            "/one_way_ledges/0/landing",
            json!([25, 21]),
        ),
        (
            "unrelated ramp return",
            "/one_way_ledges/0/return_passage_core",
            json!([0, 0, 1, 1]),
        ),
        (
            "unearned return saving",
            "/one_way_ledges/0/minimum_return_steps",
            json!(500),
        ),
        (
            "unsafe exterior arrival",
            "/doors/0/arrival_tile",
            json!([36, 17]),
        ),
        (
            "exit outside native ladder",
            "/interiors/0/exits/0",
            json!([0, 0]),
        ),
        ("overlapping actor", "/trainers/1/tile", json!([9, 11])),
        (
            "unknown trainer class",
            "/trainers/0/trainer_class",
            json!("INVALID_CLASS"),
        ),
        ("missing opponent party", "/trainers/0/party", json!([])),
    ];
    for (name, pointer, value) in changes {
        let mut changed: Value = serde_json::from_str(&dossier).expect("dossier JSON");
        *changed.pointer_mut(pointer).expect("authored field") = value;
        assert!(
            build_iron_range_descent_modpack(&base, &layout, &changed.to_string()).is_err(),
            "{name} accepted"
        );
    }
}
