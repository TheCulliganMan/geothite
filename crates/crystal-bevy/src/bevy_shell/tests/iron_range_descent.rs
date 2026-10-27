//! Local production-input fixtures after explicit initial position/party setup.
//!
//! These do not prove earned badges, Q06 progression, regional travel or balance.
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::world::movement::MovementMode;
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind, RuntimeShellPhase};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{
    Point, RouteGrid, RouteSpec, finish_text, indoor_walk, neighbors, position,
};

fn fixture(at: Point, populated: bool) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_IRON_RANGE_DESCENT_TEST_PACK").expect("external R10 pack"),
    );
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified published pack"),
    )
    .expect("runtime");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: format!("MnR10C{:02}R{:02}", at.0 / 64, at.1 / 48),
            tile_x: at.0 % 64,
            tile_y: at.1 % 48,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit R10 fixture placement");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture name");
    if populated {
        let mut pokemon = runtime
            .data()
            .create_pokemon("MEW", 60, Dv::default())
            .expect("source-created HM-compatible fixture partner");
        pokemon.moves = vec![LearnedMove {
            name: "SURF".into(),
            current_pp: runtime.data().moves["SURF"].pp,
            pp_ups: 0,
        }];
        let state = controller.shell.shell.session_mut().state_mut();
        assert!(state.storage.party.add_pokemon(pokemon));
        state.badges.johto = [true, true, true, true, true, true, true, false];
        state.sync_party_from_storage();
        state.random_state = Default::default();
        *controller.shell.shell.session_mut().divider_mut_for_tests() =
            crystal_core::random::RuntimeDividerSource::replay(
                (0_u32..262144)
                    .map(|n| ((n * 73 + 19) % 256) as u8)
                    .collect::<Vec<u8>>(),
            );
    }
    // All mutation above is explicit initial fixture setup. Below, gameplay is
    // driven only through the production controller, including random battles.
    controller.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    assert_eq!(position(&mut controller), at);
    (root, runtime, controller)
}

fn route_tileset(runtime: &CrystalRuntime) -> &'static str {
    if runtime.data().tilesets.contains_key("mn_r10_rockstairs") {
        "mn_r10_rockstairs"
    } else {
        "johto"
    }
}

fn grid(runtime: &CrystalRuntime, start: Point) -> RouteGrid {
    RouteGrid::new_with_grass(
        runtime,
        RouteSpec {
            prefix: "MnR10",
            columns: 2,
            rows: 1,
            tileset: route_tileset(runtime),
            start,
            doors: BTreeMap::from([((36, 17), "MnR10Cave1F")]),
        },
    )
}

fn collision(runtime: &CrystalRuntime, controller: &VisibleShellController, at: Point) -> String {
    let block = controller
        .shell
        .shell
        .session()
        .overworld()
        .map
        .metatile_at((at.0 % 64) / 2, (at.1 % 48) / 2)
        .expect("current native map block");
    runtime.data().tilesets[route_tileset(runtime)].collision[&format!("{block:02x}")]
        [(at.1 % 2 * 2 + at.0 % 2) as usize]
        .clone()
}

fn blocked(controller: &mut VisibleShellController, direction: GameButton) {
    let before = position(controller);
    for _ in 0..4 {
        controller.press(direction).expect("native barrier input");
        assert_eq!(position(controller), before, "barrier must reject movement");
    }
}

fn event(controller: &VisibleShellController, name: &str) -> bool {
    controller
        .shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(name)
        .expect("registered R10 event")
}

#[test]
#[ignore = "requires the published external Iron Range Descent pack"]
fn iron_range_descent_controller_walks_geometry_events_and_cave() {
    let (_, runtime, mut controller) = fixture((6, 10), false);
    let grid = grid(&runtime, (6, 10));
    assert!(
        controller
            .snapshot()
            .expect("empty party")
            .party
            .slots
            .is_empty()
    );
    for at in [
        (2, 10),
        (6, 10),
        (88, 24),
        (92, 24),
        (63, 24),
        (64, 24),
        (63, 24),
    ] {
        grid.walk(&mut controller, at);
        assert_eq!(
            controller
                .snapshot()
                .expect("clipped road seam")
                .overworld
                .map_name,
            format!("MnR10C{:02}R00", at.0 / 64)
        );
    }
    for x in 49..=56 {
        grid.walk(&mut controller, (x, 20));
        if (50..56).contains(&x) {
            assert_eq!(collision(&runtime, &controller, (x, 20)), "TALL_GRASS");
        }
    }
    for x in (49..56).rev() {
        grid.walk(&mut controller, (x, 20));
    }
    let stair_x = if route_tileset(&runtime) == "mn_r10_rockstairs"
        && runtime.data().tilesets[route_tileset(&runtime)].collision["80"][2] == "FLOOR"
    {
        10
    } else {
        11
    };
    for y in [22, 21, 20, 19, 20, 21, 22] {
        grid.walk(&mut controller, (stair_x, y));
    }
    grid.walk(&mut controller, (25, 19));
    for _ in 0..8 {
        controller
            .press(GameButton::Down)
            .expect("native ledge hop");
        assert_ne!(
            position(&mut controller),
            (25, 21),
            "walked through cliff face"
        );
        if position(&mut controller) == (25, 22) {
            break;
        }
    }
    assert_eq!(position(&mut controller), (25, 22));
    blocked(&mut controller, GameButton::Up);
    assert_eq!(grid.path((25, 22), (25, 19)).len(), 33);
    grid.walk(&mut controller, (25, 19));

    for (at, expected) in [
        ((4, 12), "GRAND RAPIDS"),
        ((8, 24), "CARRIER BANK"),
        ((38, 24), "DRY MINE"),
        ((48, 22), "PINE NECK"),
        ((90, 22), "DULUTH"),
        ((76, 28), "PUBLIC LANDING"),
        ((7, 25), "depot"),
        ((41, 23), "upper road"),
        ((67, 25), "SUPERIOR"),
    ] {
        assert!(
            grid.talk(&mut controller, at).contains(expected),
            "interaction {at:?}"
        );
    }
    for at in [(17, 27), (85, 19), (27, 9)] {
        assert!(grid.talk(&mut controller, at).contains("put the"));
    }
    for (at, direction, target) in [
        ((67, 26), GameButton::Down, (67, 27)),
        ((71, 28), GameButton::Left, (70, 28)),
        ((74, 29), GameButton::Right, (75, 29)),
        ((73, 29), GameButton::Down, (73, 30)),
    ] {
        grid.walk(&mut controller, at);
        assert_eq!(
            collision(&runtime, &controller, target),
            if target == (73, 30) { "WATER" } else { "WALL" }
        );
        blocked(&mut controller, direction);
    }
    grid.walk(&mut controller, (36, 17));
    assert_eq!(
        controller
            .snapshot()
            .expect("active cave entry")
            .overworld
            .map_name,
        "MnR10Cave1F"
    );
    indoor_walk(&mut controller, &runtime, (8, 5));
    controller
        .press(GameButton::Right)
        .expect("face gallery reader");
    controller.press(GameButton::A).expect("gallery reader");
    let reader = controller.snapshot().expect("reader text");
    assert!(
        reader
            .ui
            .text
            .as_ref()
            .and_then(|t| t.asm_text.as_ref())
            .is_some_and(|t| t.contains("Both passages"))
    );
    controller
        .press(GameButton::Down)
        .expect("reader dialogue owns movement");
    assert_eq!(
        controller
            .snapshot()
            .expect("reader input owner")
            .overworld
            .tile,
        reader.overworld.tile
    );
    finish_text(&mut controller);
    let reward_approach = if runtime.data().maps["MnR10Cave1F"].attributes.width == 18 {
        (28, 23)
    } else {
        (16, 5)
    };
    indoor_walk(&mut controller, &runtime, reward_approach);
    controller
        .press(GameButton::Right)
        .expect("face gallery reward");
    controller.press(GameButton::A).expect("Revive pickup");
    finish_text(&mut controller);
    indoor_walk(&mut controller, &runtime, (3, 13));
    assert_eq!(
        controller
            .snapshot()
            .expect("cave passive return")
            .overworld
            .map_name,
        "MnR10C00R00"
    );
    assert_eq!(position(&mut controller), (36, 18));
    grid.walk(&mut controller, (37, 18));
    grid.walk(&mut controller, (36, 18));
    assert_eq!(
        position(&mut controller),
        (36, 18),
        "passive arrival must not reenter"
    );
    grid.walk(&mut controller, (36, 17));
    assert_eq!(
        controller
            .snapshot()
            .expect("repeat cave entry")
            .overworld
            .map_name,
        "MnR10Cave1F"
    );
    // Arrival suppression may leave the player on the destination ladder.
    // Step clear before deliberately taking it again; do not rewrite position.
    indoor_walk(&mut controller, &runtime, (3, 12));
    indoor_walk(&mut controller, &runtime, (3, 13));
    assert_eq!(position(&mut controller), (36, 18));
    let snapshot = controller.snapshot().expect("four actual pickups");
    for item in ["HYPER_POTION", "ULTRA_BALL", "MAX_REPEL", "REVIVE"] {
        assert!(
            snapshot
                .bag
                .items
                .iter()
                .chain(&snapshot.bag.balls)
                .any(|s| s.item_id == item && s.quantity == 1),
            "{item}"
        );
    }
    for n in 1..=4 {
        assert!(event(&controller, &format!("EVENT_MN_R10_ITEM_{n:02}")));
    }
    grid.walk(&mut controller, (6, 10));
    controller
        .press(GameButton::Start)
        .expect("production Start menu");
    assert!(controller.snapshot().expect("Start").ui.menu.is_some());
    controller.press(GameButton::B).expect("close Start");
    if let Some(save) = std::env::var_os("CRYSTAL_IRON_RANGE_DESCENT_GEOMETRY_SAVE") {
        controller
            .save(PathBuf::from(save))
            .expect("export fresh empty-party geometry save");
    }
}

#[test]
#[ignore = "requires the published external Iron Range Descent pack"]
fn iron_range_descent_controller_wins_calder_and_continues() {
    let (root, runtime, mut controller) = fixture((6, 10), true);
    let grid = grid(&runtime, (6, 10));
    grid.walk(&mut controller, (9, 10));
    controller.press(GameButton::Down).expect("face Calder");
    controller.press(GameButton::A).expect("challenge Calder");
    let initial_pp = controller.snapshot().expect("initial Surf PP").party.slots[0]
        .pokemon
        .moves[0]
        .current_pp;
    let mut opponents = BTreeSet::new();
    let mut selected_surf = 0;
    for _ in 0..1024 {
        let snapshot = controller.snapshot().expect("native trainer battle");
        let Some(battle) = &snapshot.battle else {
            if event(&controller, "EVENT_MN_R10_TRAINER_01") {
                break;
            }
            controller
                .press(GameButton::A)
                .expect("trainer introduction or victory script");
            continue;
        };
        assert!(
            matches!(&battle.kind, RuntimeBattleKind::Trainer { trainer_id, event_flag, .. } if trainer_id == "MN_R10_T01" && event_flag == "EVENT_MN_R10_TRAINER_01")
        );
        opponents.insert(battle.enemy_pokemon.species.id.clone());
        let button = if snapshot.ui.menu.is_none() && snapshot.ui.text.is_some() {
            controller
                .wait_frames(30)
                .expect("battle text presentation");
            GameButton::A
        } else if snapshot.ui.menu.as_ref().is_some_and(|m| {
            m.layout
                .vertical_menus
                .iter()
                .flat_map(|v| &v.options)
                .any(|s| s.contains("change <PKMN>"))
        }) {
            GameButton::B
        } else if let Some(cursor) = &controller.shell.battle_move_cursor {
            assert_eq!(battle.player_moves[0].name, "SURF");
            assert!(battle.player_moves[0].current_pp > 0);
            if cursor.option_index > 0 {
                GameButton::Up
            } else {
                selected_surf += 1;
                GameButton::A
            }
        } else if let Some(cursor) = &controller.shell.battle_action_cursor {
            match cursor.option_index {
                0 => GameButton::A,
                1 => GameButton::Left,
                2 | 3 => GameButton::Up,
                _ => panic!("invalid battle command cursor"),
            }
        } else {
            GameButton::A
        };
        controller
            .press(button)
            .expect("authoritative trainer battle input");
    }
    finish_text(&mut controller);
    let before = controller.snapshot().expect("trainer victory");
    assert!(before.battle.is_none());
    assert_eq!(
        before.overworld.map_name, "MnR10C00R00",
        "fixture must not black out"
    );
    assert!(before.party.slots[0].pokemon.hp > 0);
    assert!(selected_surf > 0);
    assert!(
        before.party.slots[0].pokemon.moves[0].current_pp < initial_pp,
        "selected Surf must actually spend PP"
    );
    assert_eq!(
        opponents,
        BTreeSet::from(["GRAVELER".into(), "MACHOKE".into()])
    );
    assert!(event(&controller, "EVENT_MN_R10_TRAINER_01"));
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let save = std::env::temp_dir().join(format!(
        "geothite-r10-calder-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    controller
        .save(&save)
        .expect("production save after victory");
    let mut loaded =
        VisibleShellController::load_save(root, runtime, save.clone(), None).expect("Continue");
    let after = loaded.snapshot().expect("continued victory");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert!(event(&loaded, "EVENT_MN_R10_TRAINER_01"));
    assert!(!grid.talk(&mut loaded, (9, 11)).is_empty());
    assert!(
        loaded
            .snapshot()
            .expect("defeated trainer revisit")
            .battle
            .is_none()
    );
    if let Some(path) = std::env::var_os("CRYSTAL_IRON_RANGE_DESCENT_JOURNEY_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("export local trainer save");
    }
    std::fs::remove_file(save).expect("remove fixture save");
}

fn settle_water(controller: &mut VisibleShellController) {
    for _ in 0..256 {
        let snapshot = controller.snapshot().expect("native water activity");
        if let Some(battle) = snapshot.battle {
            assert!(matches!(battle.kind, RuntimeBattleKind::Wild { .. }));
            let button = if !controller.shell.battle_messages.is_empty() {
                GameButton::A
            } else if let Some(cursor) = &controller.shell.battle_action_cursor {
                match cursor.option_index {
                    0 | 1 => GameButton::Down,
                    2 => GameButton::Right,
                    3 => GameButton::A,
                    _ => panic!("invalid native action cursor"),
                }
            } else {
                GameButton::A
            };
            controller
                .press(button)
                .expect("RUN from incidental water encounter");
        } else if snapshot.phase == RuntimeShellPhase::Text
            || controller.has_pending_script_work()
            || controller.shell.field_notice.is_some()
            || !controller.shell.battle_messages.is_empty()
            || snapshot
                .ui
                .text
                .as_ref()
                .is_some_and(|t| t.label == "visible-shell:battle-message")
        {
            controller
                .press(GameButton::A)
                .expect("native Surf or encounter dialogue");
            controller.wait_frames(16).expect("field presentation");
        } else {
            return;
        }
    }
    panic!("water activity failed to release input");
}

fn water_step(controller: &mut VisibleShellController, direction: GameButton, target: Point) {
    for _ in 0..8 {
        if position(controller) == target {
            return;
        }
        controller.press(direction).expect("native Surf step");
        settle_water(controller);
    }
    assert_eq!(position(controller), target);
}

fn buoy(runtime: &CrystalRuntime, controller: &mut VisibleShellController, direction: GameButton) {
    let before = position(controller);
    let target = neighbors(before)
        .into_iter()
        .find_map(|(p, b)| (b == direction).then_some(p))
        .expect("outward neighbor");
    assert_eq!(collision(runtime, controller, target), "BUOY");
    for _ in 0..4 {
        controller
            .press(direction)
            .expect("Surf against visible native water-rock");
        settle_water(controller);
        assert_eq!(position(controller), before);
        assert_eq!(
            controller.snapshot().expect("blocked Surf").overworld.mode,
            MovementMode::Surf
        );
    }
}

#[test]
#[ignore = "requires the published external Iron Range Descent pack"]
fn iron_range_descent_controller_surfs_seam_and_blocks_outer_rocks() {
    let (_, runtime, mut controller) = fixture((73, 29), true);
    controller.press(GameButton::Down).expect("face lake");
    controller.press(GameButton::A).expect("native Surf prompt");
    for _ in 0..128 {
        if controller.snapshot().expect("Surf mount").overworld.mode == MovementMode::Surf {
            controller.wait_frames(60).expect("mount presentation");
            settle_water(&mut controller);
            break;
        }
        controller.press(GameButton::A).expect("Surf confirmation");
        controller.wait_frames(16).expect("Surf text presentation");
    }
    assert_eq!(
        controller.snapshot().expect("mounted").overworld.mode,
        MovementMode::Surf
    );
    assert_eq!(position(&mut controller), (73, 30));
    for x in (62..73).rev() {
        water_step(&mut controller, GameButton::Left, (x, 30));
        assert_eq!(
            controller
                .snapshot()
                .expect("water seam west")
                .overworld
                .map_name,
            format!("MnR10C{:02}R00", x / 64)
        );
    }
    for x in 63..=73 {
        water_step(&mut controller, GameButton::Right, (x, 30));
    }
    for y in 31..=38 {
        water_step(&mut controller, GameButton::Down, (73, y));
    }
    buoy(&runtime, &mut controller, GameButton::Down);
    for y in (34..38).rev() {
        water_step(&mut controller, GameButton::Up, (73, y));
    }
    for x in 74..=94 {
        water_step(&mut controller, GameButton::Right, (x, 34));
    }
    buoy(&runtime, &mut controller, GameButton::Right);
    for x in (73..94).rev() {
        water_step(&mut controller, GameButton::Left, (x, 34));
    }
    for y in (30..34).rev() {
        water_step(&mut controller, GameButton::Up, (73, y));
    }
    water_step(&mut controller, GameButton::Up, (73, 29));
    assert_eq!(
        controller
            .snapshot()
            .expect("native dismount")
            .overworld
            .mode,
        MovementMode::Normal
    );
}

#[test]
#[ignore = "requires the published native R10 outdoor rock-stair revision"]
fn iron_range_descent_controller_native_outdoor_stair_both_directions_and_side_wall() {
    let (_, runtime, mut controller) = fixture((11, 22), false);
    assert_eq!(route_tileset(&runtime), "mn_r10_rockstairs");
    let module = &runtime.data().maps["MnR10C00R00"];
    assert_eq!(module.attributes.tileset_name, "mn_r10_rockstairs");
    assert_eq!(
        module.blocks[10 * usize::from(module.attributes.width) + 5],
        0x80
    );
    let imported = &runtime.data().tilesets[route_tileset(&runtime)].collision["80"];
    assert!(
        imported == &runtime.data().tilesets["cave"].collision["12"]
            || imported == &runtime.data().tilesets["cave"].collision["30"],
        "outdoor stair must retain a complete original cave-stair collision"
    );
    let x = if imported[2] == "FLOOR" { 10 } else { 11 };
    let grid = grid(&runtime, (11, 22));
    for y in [22, 21] {
        grid.walk(&mut controller, (x, y));
    }
    blocked(
        &mut controller,
        if x == 10 {
            GameButton::Right
        } else {
            GameButton::Left
        },
    );
    for y in [20, 19] {
        grid.walk(&mut controller, (x, y));
    }
    grid.walk(&mut controller, (11, 19));
    for y in [19, 20, 21, 22] {
        grid.walk(&mut controller, (x, y));
    }
    grid.walk(&mut controller, (11, 22));
    assert_eq!(position(&mut controller), (11, 22));
    assert_eq!(controller.shell.last_error, None);
}
