//! Frozen three-chunk North Shore publication preserves the original world.
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{
    CompiledGamePack, build_grand_marais_modpack, build_north_shore_modpack,
    read_verified_compiled_game_pack,
};
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::state::GameState;
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::{
    MovementMode, PlayerMovementState, StepOptions, StepOutcome, attempt_step,
};
use serde_json::{Value, json};

fn inputs() -> &'static (CompiledGamePack, CompiledGamePack, String, String) {
    static INPUTS: OnceLock<(CompiledGamePack, CompiledGamePack, String, String)> = OnceLock::new();
    INPUTS.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository");
        let source =
            read_verified_compiled_game_pack(root.join("content-packs/text-tui.crystalpack"))
                .expect("supplied source pack");
        let town = fs::read_to_string(root.join("planning/minnesota/playable/grand-marais.json"))
            .expect("existing town design");
        let base =
            build_grand_marais_modpack(&source, &town).expect("existing Grand Marais overlay");
        let layout = fs::read_to_string(root.join("planning/minnesota/routes/MN_R11.layout.json"))
            .expect("layout");
        let dossier = fs::read_to_string(root.join("planning/minnesota/routes/MN_R11.json"))
            .expect("dossier");
        let pack = build_north_shore_modpack(&base, &layout, &dossier)
            .expect("three native chunks and keeper room");
        (base, pack, layout, dossier)
    })
}

fn collision(pack: &CompiledGamePack, x: usize, y: usize) -> &str {
    let map = &pack.data().maps[&format!("MnR11C00R{:02}", y / 48)];
    let block = map.blocks[(y % 48) / 2 * 32 + x / 2];
    &pack.data().tilesets["mn_north_shore"].collision[&format!("{block:02x}")][y % 2 * 2 + x % 2]
}

#[test]
fn north_shore_publishes_three_chunks_and_passive_keeper_room_preserving_source_content() {
    let (base, pack, layout, dossier) = inputs();
    assert_eq!(pack.data().maps.len(), base.data().maps.len() + 4);
    for (name, original) in &base.data().maps {
        assert_eq!(
            pack.data().maps.get(name),
            Some(original),
            "changed source/town map {name}"
        );
    }
    for (name, bytes) in base.runtime_files() {
        assert_eq!(
            pack.runtime_files().get(name),
            Some(bytes),
            "changed source file {name}"
        );
    }
    for (name, trainer) in &base.data().trainers.trainers {
        assert_eq!(
            pack.data().trainers.trainers.get(name),
            Some(trainer),
            "changed source trainer {name}"
        );
    }
    for (name, tileset) in &base.data().tilesets {
        assert_eq!(
            pack.data().tilesets.get(name),
            Some(tileset),
            "changed source tileset {name}"
        );
    }
    for (name, encounters) in &base.data().wild_encounters {
        assert_eq!(
            pack.data().wild_encounters.get(name),
            Some(encounters),
            "changed source encounters {name}"
        );
    }
    for (name, value) in &base.data().story_event_script_constants.global {
        assert_eq!(
            pack.data().story_event_script_constants.global.get(name),
            Some(value),
            "changed original constant {name}"
        );
    }
    assert_eq!(
        pack.data().runtime_spawn_points,
        base.data().runtime_spawn_points,
        "HOME/recovery changed"
    );
    assert_eq!(pack.data().pokemon, base.data().pokemon);
    assert_eq!(pack.data().items, base.data().items);
    assert_eq!(pack.data().marts, base.data().marts);
    assert_eq!(pack.compiled_audio(), base.compiled_audio());
    assert_eq!(
        pack.audio_manifest().expect("result audio"),
        base.audio_manifest().expect("source audio")
    );

    let mut counts = (0, 0, 0, 0);
    for row in 0..3 {
        let name = format!("MnR11C00R{row:02}");
        let map = &pack.data().maps[&name];
        assert_eq!((map.attributes.width, map.attributes.height), (32, 24));
        assert_eq!(map.blocks.len(), 768);
        assert_eq!(map.attributes.tileset_name, "mn_north_shore");
        assert_eq!(
            map.attributes.map_group_constant.as_deref(),
            Some("MAPGROUP_MINNESOTA_NORTH_SHORE")
        );
        assert_eq!(
            pack.data()
                .assemble_map_module_from_compiled_payloads(&name)
                .expect("split native payload"),
            *map
        );
        assert!(pack.data().wild_encounters.contains_key(&name));
        assert_eq!(
            map.attributes.connections.len(),
            if row == 1 { 2 } else { 1 }
        );
        for neighbor in &map.attributes.connections {
            let opposite = match neighbor.direction.as_str() {
                "north" => "south",
                "south" => "north",
                _ => panic!("a one-column coast has only north/south neighbors"),
            };
            assert!(
                pack.data().maps[&neighbor.target_map]
                    .attributes
                    .connections
                    .iter()
                    .any(|c| c.direction == opposite && c.target_map == name && c.offset == 0)
            );
        }
        counts.0 += map.trainer_scripts.len();
        counts.1 += map.script_field_pickups.len();
        counts.2 += map.events.bg_events.len();
        counts.3 += map.objects.len();
    }
    assert_eq!(counts, (10, 4, 6, 19));
    for name in ["MnR11C01R00", "MnR11C01R04", "MnR11C00R03", "MnR11C00R05"] {
        assert!(
            !pack.data().maps.contains_key(name),
            "superseded map survived: {name}"
        );
    }
    // Genuine original brick bridges retain all four LADDER quadrants.
    for (x, y0, y1) in [(20, 34, 41), (23, 48, 50)] {
        for y in y0..y1 {
            let map = &pack.data().maps[&format!("MnR11C00R{:02}", y / 24)];
            assert_eq!(map.blocks[(y % 24) * 32 + x], 0x06);
            for dy in 0..2 {
                for dx in 0..2 {
                    assert_eq!(collision(pack, x * 2 + dx, y * 2 + dy), "LADDER");
                }
            }
        }
    }
    let middle = &pack.data().maps["MnR11C00R01"];
    for y in 33..36 {
        assert_eq!(middle.blocks[(y - 24) * 32 + 10], 0x80);
        for dy in 0..2 {
            for dx in 0..2 {
                assert_eq!(collision(pack, 20 + dx, y * 2 + dy), "WATERFALL");
            }
        }
    }
    assert_eq!(
        collision(pack, 20, 65),
        "WATER",
        "ordinary summit pool feeds the cascade"
    );
    assert_eq!(
        collision(pack, 20, 61),
        "FLOOR",
        "safe optional summit bank"
    );
    assert_eq!(
        collision(pack, 20, 62),
        "WATER",
        "native Surf relaunch from the bank"
    );
    let source_art = &base.runtime_files()["data/tilesets/johto_metatiles.bin"];
    let derived_art = &pack.runtime_files()["data/tilesets/mn_north_shore_metatiles.bin"];
    assert_eq!(&derived_art[..source_art.len()], source_art);
    assert_eq!(&derived_art[source_art.len()..], &[0x20; 16]);
    assert_eq!(
        pack.data().tilesets["mn_north_shore"].collision["80"],
        base.data().tilesets["cave"].collision["2c"]
    );
    for (block, original) in &base.data().tilesets["johto"].collision {
        assert_eq!(
            pack.data().tilesets["mn_north_shore"].collision.get(block),
            Some(original)
        );
    }
    for y in 126..132 {
        for x in 32..38 {
            assert_eq!(collision(pack, x, y), "TALL_GRASS");
        }
    }
    for (y, xs) in [(48, 52..63), (96, 56..63)] {
        for x in xs {
            assert_eq!(collision(pack, x, y - 1), "WATER");
            assert_eq!(collision(pack, x, y), "WATER");
        }
    }
    let mut perimeter = 0;
    let mut buoys = 0;
    for y in 0..144 {
        for x in 0..64 {
            let kind = collision(pack, x, y);
            assert_ne!(kind, "CUT_TREE", "undeclared decorative Cut at {x},{y}");
            if x == 0 || x == 63 || y == 0 || y == 143 {
                perimeter += 1;
                buoys += usize::from(kind == "BUOY");
                assert!(
                    !matches!(kind, "WATER" | "WATERFALL"),
                    "open outer water at {x},{y}"
                );
            }
        }
    }
    assert_eq!((perimeter, buoys), (412, 174));

    let exterior = &pack.data().maps["MnR11C00R02"];
    assert_eq!(exterior.events.warps.len(), 2);
    assert_eq!(
        (exterior.events.warps[0].x, exterior.events.warps[0].y),
        (51, 3)
    );
    assert_eq!(
        (exterior.events.warps[1].x, exterior.events.warps[1].y),
        (51, 4)
    );
    assert_eq!(
        exterior.events.warps[0].target_map_constant,
        "MN_R11_SPLIT_ROCK_LIGHTHOUSE"
    );
    assert_eq!(exterior.events.warps[0].target_warp_id, 1);
    assert_eq!(collision(pack, 51, 99), "DOOR");
    assert_eq!(collision(pack, 51, 100), "FLOOR");
    let room = &pack.data().maps["MnR11SplitRockLighthouse"];
    assert_eq!((room.attributes.width, room.attributes.height), (6, 5));
    assert_eq!(room.attributes.tileset_name, "lighthouse");
    assert_eq!(room.attributes.border_block, 0x3e);
    assert_eq!(room.objects.len(), 1);
    let keeper = &room.objects[0];
    assert_eq!(keeper.script, "MnR11Keeper");
    assert_eq!((keeper.x, keeper.y), (7, 3));
    assert_eq!(keeper.sprite, "SPRITE_SAILOR");
    assert_eq!(room.events.warps.len(), 2);
    for (warp, x) in room.events.warps.iter().zip([4, 5]) {
        assert_eq!((warp.x, warp.y), (x, 9));
        assert_eq!(warp.target_map_constant, "MN_R11_C00R02");
        assert_eq!(warp.target_warp_id, 2);
    }
    let design: Value = serde_json::from_str(dossier).expect("dossier JSON");
    let rows: Vec<u16> = design["interiors"][0]["rows"]
        .as_array()
        .expect("room rows")
        .iter()
        .flat_map(|r| r.as_str().expect("row").split_whitespace())
        .map(|b| u16::from_str_radix(b, 16).expect("native block"))
        .collect();
    assert_eq!(room.blocks, rows);
    assert_eq!(
        room.blocks,
        vec![
            0x0c, 0x3e, 0x29, 0x29, 0x3e, 0x0c, 0x3e, 0x27, 0x27, 0x27, 0x27, 0x3e, 0x3e, 0x18,
            0x27, 0x27, 0x2f, 0x3e, 0x3e, 0x27, 0x27, 0x27, 0x27, 0x3e, 0x3c, 0x3d, 0x2e, 0x3d,
            0x3d, 0x3c,
        ]
    );
    for key in ["trainers", "residents", "items"] {
        for actor in design[key].as_array().expect("actor array") {
            let id = actor["id"].as_str().expect("actor ID");
            let x = actor["tile"][0].as_u64().expect("x") as usize;
            let y = actor["tile"][1].as_u64().expect("y") as usize;
            let map = &pack.data().maps[&format!("MnR11C00R{:02}", y / 48)];
            let actual = map
                .objects
                .iter()
                .find(|o| o.script == id)
                .expect("native actor");
            assert_eq!(
                (usize::from(actual.x), usize::from(actual.y)),
                (x, y % 48),
                "{id}"
            );
            if key == "items" {
                assert_eq!(
                    actual.event_flag,
                    actor["flag"].as_str().expect("pickup flag")
                );
            }
        }
    }
    let parties: [[(&str, u8); 2]; 10] = [
        [("GRAVELER", 38), ("ONIX", 38)],
        [("SUNFLORA", 39), ("NOCTOWL", 38)],
        [("LANTURN", 40), ("MANTINE", 40)],
        [("URSARING", 40), ("GRAVELER", 39)],
        [("QUAGSIRE", 40), ("SEAKING", 40)],
        [("NOCTOWL", 41), ("FEAROW", 40)],
        [("ONIX", 41), ("MAGCARGO", 41)],
        [("SUDOWOODO", 42), ("AMPHAROS", 41)],
        [("GYARADOS", 42), ("QWILFISH", 41)],
        [("PIDGEOT", 43), ("NOCTOWL", 42)],
    ];
    for (index, expected) in parties.iter().enumerate() {
        let id = format!("MN_R11_T{:02}", index + 1);
        let actual: Vec<_> = pack.data().trainers.trainers[&id]
            .party
            .iter()
            .map(|p| (p.species.as_str(), p.level))
            .collect();
        assert_eq!(
            actual.as_slice(),
            expected,
            "changed original authored party {id}"
        );
        assert!(
            pack.data()
                .story_event_script_constants
                .global
                .contains_key(&format!("EVENT_MN_R11_TRAINER_{:02}", index + 1))
        );
    }
    assert!(
        build_north_shore_modpack(pack, layout, dossier).is_err(),
        "duplicate installation accepted"
    );
}

#[test]
fn north_shore_rejects_native_geometry_failures_with_specific_errors() {
    let (base, _, layout, dossier) = inputs();
    // Every added layer has a real name and valid native shape. Assertions name
    // the physical failure, so a serde/schema rejection cannot satisfy a test.
    for (name, layer, expected) in [
        (
            "necessary bridge removed",
            json!({"kind":"point","name":"test gap in necessary brick bridge","position":[20,37],"block":"35"}),
            "route ends disconnected",
        ),
        (
            "east sea open",
            json!({"kind":"point","name":"test unclosed east sea","position":[31,40],"block":"35"}),
            "reachable outer water",
        ),
        (
            "south sea open",
            json!({"kind":"point","name":"test unclosed south sea","position":[27,71],"block":"35"}),
            "reachable outer water",
        ),
        (
            "north sea open",
            json!({"kind":"point","name":"test unclosed north sea","position":[27,0],"block":"35"}),
            "reachable outer water",
        ),
        (
            "required grass removed",
            json!({"kind":"rect","name":"test habitat flattened","material":"floor","bounds":[16,63,19,66]}),
            "required grass crossing contains another terrain",
        ),
        (
            "active tower door erased",
            json!({"kind":"point","name":"test missing tower doorway","position":[25,49],"block":"01"}),
            "authored door or return is blocked",
        ),
        (
            "resident buried",
            json!({"kind":"point","name":"test canopy over resident","position":[20,5],"block":"05"}),
            "not on clear floor",
        ),
        (
            "incidental Cut tree",
            json!({"kind":"point","name":"test decorative Cut off main road","position":[22,6],"block":"5b"}),
            "undeclared decorative Cut",
        ),
    ] {
        let mut changed: Value = serde_json::from_str(layout).expect("layout JSON");
        changed["layers"]
            .as_array_mut()
            .expect("layers")
            .push(layer);
        let error = build_north_shore_modpack(base, &changed.to_string(), dossier).expect_err(name);
        assert!(
            format!("{error:#}").contains(expected),
            "{name}: unexpected failure {error:#}"
        );
    }
    for (name, pointer, value, expected) in [
        (
            "carpet moved onto wall",
            "/interiors/0/exits/0",
            json!([0, 0]),
            "room exit collision differs",
        ),
        (
            "keeper moved onto furniture",
            "/interiors/0/residents/0/tile",
            json!([0, 0]),
            "room actor stands on furniture",
        ),
        (
            "room arrival moved onto wall",
            "/interiors/0/clear_arrival",
            json!([0, 0]),
            "room arrival is blocked",
        ),
        (
            "actors overlap",
            "/trainers/1/tile",
            json!([35, 119]),
            "overlapping route actors",
        ),
        (
            "unknown trainer class",
            "/trainers/0/trainer_class",
            json!("HIKER_TYPO"),
            "unknown source class",
        ),
    ] {
        let mut changed: Value = serde_json::from_str(dossier).expect("dossier JSON");
        *changed
            .pointer_mut(pointer)
            .expect("existing authored field") = value;
        let error = build_north_shore_modpack(base, layout, &changed.to_string()).expect_err(name);
        assert!(
            format!("{error:#}").contains(expected),
            "{name}: unexpected failure {error:#}"
        );
    }
}

#[test]
fn north_shore_summit_native_dismount_and_relaunch_reject_the_previous_dry_trap() {
    let data = inputs().1.data();
    let map = data
        .overworld_map("MnR11C00R01")
        .expect("native middle chunk");
    let tileset = data
        .tileset_collision("mn_north_shore")
        .expect("numeric source collision");
    let mut state = GameState::reset_wram_for_new_game();
    let mut pokemon = data
        .create_pokemon("MEW", 60, Dv::default())
        .expect("explicit component partner");
    pokemon.moves = ["SURF", "WATERFALL"]
        .into_iter()
        .map(|name| LearnedMove {
            name: name.into(),
            current_pp: data.moves[name].pp,
            pp_ups: 0,
        })
        .collect();
    assert!(state.storage.party.add_pokemon(pokemon));
    state.badges.johto = [true; 8];
    state.sync_party_from_storage();
    let options = StepOptions {
        force_step_after_turn: true,
        ..Default::default()
    };
    let mut player =
        PlayerMovementState::new(TilePosition::new(20, 24)).with_mode(MovementMode::Surf);
    player.facing = Direction::Up;
    let ascent = data
        .apply_waterfall_field_move(&state, &state.storage, &map, &tileset, &mut player, 0)
        .expect("native ascent into spring");
    assert_eq!(ascent.to_tile, TilePosition::new(20, 17));
    assert_eq!(ascent.steps, 7);
    for y in (13..17).rev() {
        assert!(
            matches!(attempt_step(&mut player, Direction::Up, &map, &tileset, options), StepOutcome::Moved { to, .. } if to==TilePosition::new(20,y))
        );
    }
    assert_eq!(
        player.mode,
        MovementMode::Normal,
        "ordinary bank step dismounts"
    );
    assert!(matches!(
        attempt_step(
            &mut player,
            Direction::Down,
            &map,
            &tileset,
            StepOptions::default()
        ),
        StepOutcome::Turned { .. }
    ));
    let relaunched = data
        .apply_surf_field_move(&state, &state.storage, &map, &tileset, &mut player, 0)
        .expect("real Surf relaunch from dry summit bank");
    assert_eq!(relaunched.to_tile, TilePosition::new(20, 14));
    assert_eq!(player.mode, MovementMode::Surf);

    // Recreate only the six prior dry summit blocks in a test-local map. The
    // real native Waterfall+walk+Surf rules must reproduce the old failure;
    // this is not a published pack mutation or an alternate movement model.
    let mut dry = map.clone();
    for y in 7..9 {
        for x in 9..12 {
            dry.metatile_ids[y * 32 + x] = 0x71;
        }
    }
    let mut previous =
        PlayerMovementState::new(TilePosition::new(20, 24)).with_mode(MovementMode::Surf);
    previous.facing = Direction::Up;
    data.apply_waterfall_field_move(&state, &state.storage, &dry, &tileset, &mut previous, 0)
        .expect("previous native dry ascent");
    assert_eq!(previous.tile, TilePosition::new(20, 17));
    assert!(matches!(
        attempt_step(&mut previous, Direction::Right, &dry, &tileset, options),
        StepOutcome::Moved { .. }
    ));
    assert_eq!(previous.mode, MovementMode::Normal);
    assert!(matches!(
        attempt_step(
            &mut previous,
            Direction::Down,
            &dry,
            &tileset,
            StepOptions::default()
        ),
        StepOutcome::Turned { .. }
    ));
    let error = data
        .apply_surf_field_move(&state, &state.storage, &dry, &tileset, &mut previous, 0)
        .expect_err("native Waterfall collision is not a valid Surf mount");
    assert!(
        format!("{error:#}").contains("blocked"),
        "unexpected previous-trap failure: {error:#}"
    );
}
