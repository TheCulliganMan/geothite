//! Production input for the frozen32×72 coast after explicit initial setup.
//! No fixture earns Q07, a badge, the receiver signal or regional hub access.
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::asset::Assets;
use bevy::image::Image;
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::world::map::Direction;
use crystal_core::world::movement::MovementMode;
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind, RuntimeShellPhase};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell, load_tileset_art,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{
    Point, RouteGrid, RouteSpec, finish_text, indoor_walk, neighbors, position,
};

fn load_runtime() -> (AssetRoot, CrystalRuntime) {
    let path = std::env::var_os("CRYSTAL_NORTH_SHORE_TEST_PACK")
        .or_else(|| std::env::var_os("CRYSTAL_MINNESOTA_ROUTE_TEST_PACK"))
        .map(PathBuf::from)
        .expect("published external North Shore pack");
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified North Shore pack"),
    )
    .expect("native North Shore runtime");
    (root, runtime)
}

fn fixture(at: Point, moves: &[&str]) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, runtime) = load_runtime();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: format!("MnR11C00R{:02}", at.1 / 48),
            tile_x: at.0,
            tile_y: at.1 % 48,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit local North Shore fixture placement");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture name");
    if !moves.is_empty() {
        let mut pokemon = runtime
            .data()
            .create_pokemon("MEW", 60, Dv::default())
            .expect("native HM-compatible fixture partner");
        pokemon.moves = moves
            .iter()
            .map(|&name| LearnedMove {
                name: name.into(),
                current_pp: runtime.data().moves[name].pp,
                pp_ups: 0,
            })
            .collect();
        let state = controller.shell.shell.session_mut().state_mut();
        assert!(state.storage.party.add_pokemon(pokemon));
        state.badges.johto = [true; 8];
        state.sync_party_from_storage();
        state.random_state = Default::default();
        *controller.shell.shell.session_mut().divider_mut_for_tests() =
            crystal_core::random::RuntimeDividerSource::replay(
                (0_u32..262144)
                    .map(|n| ((n * 73 + 19) % 256) as u8)
                    .collect::<Vec<u8>>(),
            );
    }
    // The explicit setup ends here. Every subsequent game mutation is owned
    // by VisibleShellController::press; no state/position/divider rewrite.
    controller.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    assert_eq!(position(&mut controller), at);
    (root, runtime, controller)
}

fn grid(runtime: &CrystalRuntime, start: Point) -> RouteGrid {
    RouteGrid::new_with_grass(
        runtime,
        RouteSpec {
            prefix: "MnR11",
            columns: 1,
            rows: 3,
            tileset: "mn_north_shore",
            start,
            doors: BTreeMap::from([((51, 99), "MnR11SplitRockLighthouse")]),
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
        .metatile_at(at.0 / 2, (at.1 % 48) / 2)
        .expect("current native block");
    runtime.data().tilesets["mn_north_shore"].collision[&format!("{block:02x}")]
        [(at.1 % 2 * 2 + at.0 % 2) as usize]
        .clone()
}

fn event(controller: &VisibleShellController, name: &str) -> bool {
    controller
        .shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(name)
        .expect("registered authored event")
}

fn blocked(controller: &mut VisibleShellController, direction: GameButton) {
    let before = position(controller);
    for _ in 0..4 {
        controller.press(direction).expect("native barrier input");
        assert_eq!(position(controller), before);
    }
}

#[test]
#[ignore = "requires the published external North Shore pack"]
fn north_shore_animations_match_native_johto_and_cave_frames() {
    let (_, runtime) = load_runtime();
    let root = runtime
        .materialize_runtime_files()
        .expect("owned temporary runtime mount");
    let mut images = Assets::<Image>::default();
    let derived = load_tileset_art(
        &root,
        "mn_north_shore",
        "day",
        &runtime.data().tilesets["mn_north_shore"].palette_map,
        &mut images,
    )
    .expect("derived art");
    let johto = load_tileset_art(
        &root,
        "johto",
        "day",
        &runtime.data().tilesets["johto"].palette_map,
        &mut images,
    )
    .expect("source Johto art");
    let cave = load_tileset_art(
        &root,
        "cave",
        "day",
        &runtime.data().tilesets["cave"].palette_map,
        &mut images,
    )
    .expect("source cave art");
    for (tile, source_tile, source) in [(3, 3, &johto), (0x14, 0x14, &johto), (0x20, 0x40, &cave)] {
        let actual = &derived.animated_tiles[&tile];
        let expected = &source.animated_tiles[&source_tile];
        assert_eq!(
            (
                actual.frame_ticks,
                actual.phase_offset,
                actual.advance_on_phase_offset
            ),
            (
                expected.frame_ticks,
                expected.phase_offset,
                expected.advance_on_phase_offset
            )
        );
        assert_eq!(actual.frames.len(), expected.frames.len());
        for (a, b) in actual.frames.iter().zip(expected.frames.iter()) {
            assert_eq!(
                images.get(a).expect("derived frame").data,
                images.get(b).expect("source frame").data,
                "tile {tile:02x}: animation differs from source"
            );
        }
    }
}

#[test]
#[ignore = "requires the republished three-chunk external North Shore pack"]
fn north_shore_controller_walks_bridges_seams_events_and_both_keeper_room_returns() {
    let (_, runtime, mut controller) = fixture((34, 136), &[]);
    let grid = grid(&runtime, (34, 136));
    assert!(
        controller
            .snapshot()
            .expect("empty geometry party")
            .party
            .slots
            .is_empty()
    );
    for at in [
        (34, 140),
        (40, 2),
        (40, 6),
        (34, 47),
        (34, 48),
        (34, 47),
        (40, 95),
        (40, 96),
        (40, 95),
    ] {
        grid.walk(&mut controller, at);
        assert_eq!(
            controller
                .snapshot()
                .expect("native walking seam")
                .overworld
                .map_name,
            format!("MnR11C00R{:02}", at.1 / 48)
        );
    }
    for x in [40, 41] {
        for y in 67..=82 {
            grid.walk(&mut controller, (x, y));
            if (68..82).contains(&y) {
                assert_eq!(collision(&runtime, &controller, (x, y)), "LADDER");
            }
        }
        for y in (67..82).rev() {
            grid.walk(&mut controller, (x, y));
        }
    }
    grid.walk(&mut controller, (40, 75));
    assert_eq!(collision(&runtime, &controller, (39, 75)), "WATER");
    blocked(&mut controller, GameButton::Left);
    for x in [46, 47] {
        for y in 95..=100 {
            grid.walk(&mut controller, (x, y));
            if (96..100).contains(&y) {
                assert_eq!(collision(&runtime, &controller, (x, y)), "LADDER");
            }
        }
        for y in (95..100).rev() {
            grid.walk(&mut controller, (x, y));
        }
    }
    for y in 125..=132 {
        grid.walk(&mut controller, (35, y));
        if (126..132).contains(&y) {
            assert_eq!(collision(&runtime, &controller, (35, y)), "TALL_GRASS");
        }
    }
    for y in (125..132).rev() {
        grid.walk(&mut controller, (35, y));
    }
    for (at, expected) in [
        ((41, 11), "wind bends"),
        ((35, 31), "Rock pools"),
        ((27, 69), "Duluth"),
        ((47, 103), "From here"),
        ((43, 119), "Rest by the birch"),
        ((38, 8), "NORTH SHORE"),
        ((42, 30), "PEBBLE"),
        ((44, 54), "BASALT"),
        ((30, 66), "RIVER GORGE"),
        ((42, 92), "SPLIT ROCK"),
        ((36, 134), "DULUTH"),
    ] {
        assert!(
            grid.talk(&mut controller, at).contains(expected),
            "native interaction {at:?}"
        );
    }
    for at in [(49, 55), (49, 33), (45, 67), (43, 121)] {
        assert!(grid.talk(&mut controller, at).contains("found"));
    }
    for (at, target) in [((49, 58), (49, 59)), ((53, 104), (53, 105))] {
        grid.walk(&mut controller, at);
        assert_eq!(collision(&runtime, &controller, target), "WALL");
        blocked(&mut controller, GameButton::Down);
    }
    grid.walk(&mut controller, (25, 69));
    for carpet_x in [4, 5] {
        grid.walk(&mut controller, (51, 99));
        assert_eq!(
            controller
                .snapshot()
                .expect("native tower entry")
                .overworld
                .map_name,
            "MnR11SplitRockLighthouse"
        );
        let flags = controller.shell.shell.session().state().flags.clone();
        indoor_walk(&mut controller, &runtime, (7, 4));
        controller
            .press(GameButton::Up)
            .expect("face actual keeper");
        controller.press(GameButton::A).expect("keeper dialogue");
        let reader = controller.snapshot().expect("keeper text");
        assert!(
            reader
                .ui
                .text
                .as_ref()
                .and_then(|t| t.asm_text.as_ref())
                .is_some_and(|t| t.contains("Duluth sends"))
        );
        controller
            .press(GameButton::Down)
            .expect("keeper text owns movement");
        assert_eq!(
            controller
                .snapshot()
                .expect("keeper input owner")
                .overworld
                .tile,
            reader.overworld.tile
        );
        finish_text(&mut controller);
        assert_eq!(
            controller.shell.shell.session().state().flags,
            flags,
            "optional keeper must not award the receiver signal"
        );
        indoor_walk(&mut controller, &runtime, (carpet_x, 8));
        indoor_walk(&mut controller, &runtime, (carpet_x, 9));
        for _ in 0..4 {
            if controller
                .snapshot()
                .expect("carpet exit")
                .overworld
                .map_name
                != "MnR11SplitRockLighthouse"
            {
                break;
            }
            controller
                .press(GameButton::Down)
                .expect("take native carpet exit");
        }
        assert_eq!(
            controller
                .snapshot()
                .expect("passive exterior return")
                .overworld
                .map_name,
            "MnR11C00R02"
        );
        assert_eq!(position(&mut controller), (51, 100));
        grid.walk(&mut controller, (51, 101));
        grid.walk(&mut controller, (51, 100));
        assert_eq!(
            position(&mut controller),
            (51, 100),
            "passive Floor record cannot immediately reenter"
        );
    }
    let snapshot = controller.snapshot().expect("native four pickups");
    for id in ["HYPER_POTION", "ULTRA_BALL", "MAX_REPEL", "REVIVE"] {
        assert!(
            snapshot
                .bag
                .items
                .iter()
                .chain(&snapshot.bag.balls)
                .any(|s| s.item_id == id && s.quantity == 1),
            "{id}"
        );
    }
    for n in 1..=4 {
        assert!(event(&controller, &format!("EVENT_MN_R11_ITEM_{n:02}")));
    }
    grid.walk(&mut controller, (34, 136));
    controller
        .press(GameButton::Start)
        .expect("production Start");
    assert!(controller.snapshot().expect("Start menu").ui.menu.is_some());
    controller.press(GameButton::B).expect("close Start");
    if let Some(save) = std::env::var_os("CRYSTAL_NORTH_SHORE_GEOMETRY_SAVE") {
        controller
            .save(PathBuf::from(save))
            .expect("fresh geometry save");
    }
}

#[test]
#[ignore = "requires the republished three-chunk external North Shore pack"]
fn north_shore_controller_wins_will_spends_thunderbolt_pp_and_continues() {
    let (root, runtime, mut controller) = fixture((43, 14), &["THUNDERBOLT", "SURF"]);
    let grid = grid(&runtime, (43, 14));
    grid.walk(&mut controller, (43, 14));
    controller.press(GameButton::Up).expect("face Will");
    controller.press(GameButton::A).expect("challenge Will");
    let initial_pp = controller
        .snapshot()
        .expect("initial Thunderbolt PP")
        .party
        .slots[0]
        .pokemon
        .moves[0]
        .current_pp;
    let initial_surf_pp = controller.snapshot().expect("initial Surf PP").party.slots[0]
        .pokemon
        .moves[1]
        .current_pp;
    let mut opponents = BTreeSet::new();
    let mut selected_thunderbolt = 0;
    for _ in 0..1024 {
        let snapshot = controller.snapshot().expect("native trainer battle");
        let Some(battle) = &snapshot.battle else {
            if event(&controller, "EVENT_MN_R11_TRAINER_10") {
                break;
            }
            controller
                .press(GameButton::A)
                .expect("trainer introduction or victory script");
            continue;
        };
        assert!(
            matches!(&battle.kind, RuntimeBattleKind::Trainer { trainer_id, event_flag, .. } if trainer_id == "MN_R11_T10" && event_flag == "EVENT_MN_R11_TRAINER_10")
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
            let active = battle
                .active_player_party_index
                .expect("active fixture partner");
            let pokemon = &snapshot
                .party
                .slots
                .iter()
                .find(|slot| slot.index == active)
                .expect("active partner remains in the native party")
                .pokemon;
            let target = super::rochester_earned::best_attack(&runtime, battle, pokemon);
            if cursor.option_index > target {
                GameButton::Up
            } else if cursor.option_index < target {
                GameButton::Down
            } else {
                if battle.player_moves[target].name == "THUNDERBOLT" {
                    selected_thunderbolt += 1;
                }
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
        before.overworld.map_name, "MnR11C00R00",
        "fixture must not black out"
    );
    assert!(before.party.slots[0].pokemon.hp > 0);
    assert!(selected_thunderbolt > 0);
    assert!(
        before.party.slots[0].pokemon.moves[0].current_pp < initial_pp,
        "selected Thunderbolt must actually spend PP"
    );
    assert!(
        before.party.slots[0].pokemon.moves[1].current_pp < initial_surf_pp,
        "selected Surf must spend PP against the Ground opponent"
    );
    assert_eq!(
        opponents,
        runtime.data().trainers.trainers["MN_R11_T10"]
            .party
            .iter()
            .map(|pokemon| pokemon.species.clone())
            .collect::<BTreeSet<_>>()
    );
    assert!(event(&controller, "EVENT_MN_R11_TRAINER_10"));
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let save = std::env::temp_dir().join(format!(
        "geothite-r11-will-{}-{nonce}.crystalsave",
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
    assert!(event(&loaded, "EVENT_MN_R11_TRAINER_10"));
    assert!(!grid.talk(&mut loaded, (43, 13)).is_empty());
    assert!(
        loaded
            .snapshot()
            .expect("defeated trainer revisit")
            .battle
            .is_none()
    );
    if let Some(path) = std::env::var_os("CRYSTAL_NORTH_SHORE_JOURNEY_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("export local trainer save");
    }
    std::fs::remove_file(save).expect("remove fixture save");
}

pub(super) fn settle_water(controller: &mut VisibleShellController) {
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
            controller
                .wait_frames(16)
                .expect("native wild battle presentation");
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

pub(super) fn water_step(
    controller: &mut VisibleShellController,
    direction: GameButton,
    target: Point,
) {
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

pub(super) fn mount_surf(
    controller: &mut VisibleShellController,
    direction: GameButton,
    target: Point,
) {
    assert_eq!(
        controller
            .snapshot()
            .expect("normal shore mode")
            .overworld
            .mode,
        MovementMode::Normal
    );
    controller.press(direction).expect("face source water");
    controller.press(GameButton::A).expect("native Surf prompt");
    for _ in 0..128 {
        if controller.snapshot().expect("mount state").overworld.mode == MovementMode::Surf {
            controller.wait_frames(60).expect("mount presentation");
            settle_water(controller);
            assert_eq!(position(controller), target);
            return;
        }
        controller
            .press(GameButton::A)
            .expect("native Surf confirmation");
        controller.wait_frames(16).expect("native use text");
    }
    panic!("Surf prompt did not mount");
}

#[test]
#[ignore = "requires the republished three-chunk external North Shore pack"]
fn north_shore_controller_surfs_both_seams_and_rejects_visible_outer_sea_rocks() {
    let (_, runtime, mut controller) = fixture((49, 35), &["SURF", "WATERFALL"]);
    mount_surf(&mut controller, GameButton::Down, (49, 36));
    for x in 50..=59 {
        water_step(&mut controller, GameButton::Right, (x, 36));
    }
    for y in 37..=142 {
        water_step(&mut controller, GameButton::Down, (59, y));
        assert_eq!(
            controller
                .snapshot()
                .expect("sea chunk seam south")
                .overworld
                .map_name,
            format!("MnR11C00R{:02}", y / 48)
        );
    }
    buoy(&runtime, &mut controller, GameButton::Down);
    for y in (81..142).rev() {
        water_step(&mut controller, GameButton::Up, (59, y));
    }
    for x in 60..=62 {
        water_step(&mut controller, GameButton::Right, (x, 81));
    }
    buoy(&runtime, &mut controller, GameButton::Right);
    for y in (1..81).rev() {
        water_step(&mut controller, GameButton::Up, (62, y));
    }
    buoy(&runtime, &mut controller, GameButton::Up);
    for y in 2..=36 {
        water_step(&mut controller, GameButton::Down, (62, y));
    }
    for x in (49..62).rev() {
        water_step(&mut controller, GameButton::Left, (x, 36));
    }
    water_step(&mut controller, GameButton::Up, (49, 35));
    assert_eq!(
        controller
            .snapshot()
            .expect("ordinary dismount")
            .overworld
            .mode,
        MovementMode::Normal
    );
}

#[test]
#[ignore = "requires the rocky/rivers geography review pack"]
fn north_shore_controller_crosses_both_new_rivers_and_reaches_cascade_banks() {
    let (_, runtime, mut controller) = fixture((40, 112), &[]);
    let grid = grid(&runtime, (40, 112));
    for (north, south) in [(108, 111), (36, 43)] {
        for x in [40, 41] {
            grid.walk(&mut controller, (x, south + 1));
            for y in (north..=south).rev() {
                grid.walk(&mut controller, (x, y));
                assert_eq!(collision(&runtime, &controller, (x, y)), "LADDER");
            }
            grid.walk(&mut controller, (x, north - 1));
            for y in north..=south + 1 {
                grid.walk(&mut controller, (x, y));
            }
        }
    }
    for at in [
        (23, 24),
        (22, 27),
        (35, 29),
        (23, 103),
        (22, 107),
        (39, 107),
    ] {
        grid.walk(&mut controller, at);
        assert_eq!(collision(&runtime, &controller, at), "FLOOR");
    }
    for (bank, at) in [
        ((23, 24), (18, 24)),
        ((23, 103), (18, 104)),
        ((23, 103), (18, 100)),
    ] {
        // collision() reads the loaded chunk, so visit the corresponding bank.
        grid.walk(&mut controller, bank);
        assert_eq!(collision(&runtime, &controller, at), "WATERFALL");
    }
}

pub(super) fn climb_added_cascade(
    controller: &mut VisibleShellController,
    x: i16,
    base: i16,
    top: i16,
) {
    assert_eq!(position(controller), (x, base));
    // An extra Up when already facing Up enters the downward current, which
    // turns the player Down again. Contextual A needs the actual uphill facing.
    if controller
        .snapshot()
        .expect("cascade approach facing")
        .overworld
        .facing
        != Direction::Up
    {
        controller.press(GameButton::Up).expect("face cascade");
        settle_water(controller);
    }
    assert_eq!(position(controller), (x, base));
    assert_eq!(
        controller
            .snapshot()
            .expect("uphill facing")
            .overworld
            .facing,
        Direction::Up
    );
    controller
        .press(GameButton::A)
        .expect("contextual Waterfall prompt");
    for _ in 0..256 {
        if position(controller) == (x, top)
            && controller.shell.visible_waterfall_animation.is_none()
            && !controller.has_pending_script_work()
            && controller.shell.field_notice.is_none()
        {
            break;
        }
        if controller.shell.visible_waterfall_animation.is_some()
            && controller.shell.field_notice.is_none()
        {
            controller
                .wait_frames(16)
                .expect("native cascade animation");
        } else {
            controller
                .press(GameButton::A)
                .expect("confirm native Waterfall");
            controller
                .wait_frames(16)
                .expect("native source climb clock");
        }
    }
    assert!(controller.shell.flygon_field_actions.contains("waterfall"));
    assert_eq!(
        position(controller),
        (x, top),
        "native added cascade ascent"
    );
}

#[test]
#[ignore = "requires the rocky/rivers geography review pack"]
fn north_shore_controller_climbs_added_north_and_two_stage_south_cascades() {
    // Initial MEW60, badges and moves are explicit fixtures. All subsequent
    // movement, contextual HM use and returns use production input only.
    for (bank_y, base, stages) in [
        (27, 26, vec![(26, 19)]),
        (107, 106, vec![(106, 103), (102, 99)]),
    ] {
        let (_, runtime, mut controller) = fixture((22, bank_y), &["SURF", "WATERFALL"]);
        let highest = stages.last().expect("authored cascade stages").1;
        mount_surf(&mut controller, GameButton::Left, (21, bank_y));
        for x in (18..21).rev() {
            water_step(&mut controller, GameButton::Left, (x, bank_y));
        }
        water_step(&mut controller, GameButton::Up, (18, base));
        for (bottom, top) in stages {
            if position(&mut controller).1 != bottom {
                water_step(&mut controller, GameButton::Up, (18, bottom));
            }
            climb_added_cascade(&mut controller, 18, bottom, top);
            assert_eq!(collision(&runtime, &controller, (18, top)), "WATER");
        }
        for _ in 0..64 {
            if position(&mut controller) == (18, base) {
                break;
            }
            controller
                .press(GameButton::Down)
                .expect("native cascade descent");
            controller.wait_frames(16).expect("native downward current");
            settle_water(&mut controller);
            let at = position(&mut controller);
            assert_eq!(at.0, 18);
            assert!((highest..=base).contains(&at.1));
        }
        assert_eq!(position(&mut controller), (18, base));
        water_step(&mut controller, GameButton::Down, (18, bank_y));
        for x in 19..=22 {
            water_step(&mut controller, GameButton::Right, (x, bank_y));
        }
        assert_eq!(
            controller
                .snapshot()
                .expect("dry original bank return")
                .overworld
                .mode,
            MovementMode::Normal
        );
    }
}

#[test]
#[ignore = "requires the republished three-chunk external North Shore pack"]
fn north_shore_controller_uses_native_waterfall_and_returns_without_state_rewrites() {
    let (_, runtime, mut controller) = fixture((25, 71), &["SURF", "WATERFALL"]);
    mount_surf(&mut controller, GameButton::Down, (25, 72));
    for x in (20..25).rev() {
        water_step(&mut controller, GameButton::Left, (x, 72));
    }
    // Facing may take a legal initial Surf step onto the lowest waterfall tile;
    // the native contextual A/YES must still own the remaining ascent.
    controller
        .press(GameButton::Up)
        .expect("face native waterfall");
    settle_water(&mut controller);
    let origin = position(&mut controller);
    assert!([(20, 72), (20, 71)].contains(&origin));
    assert_eq!(collision(&runtime, &controller, (20, 70)), "WATERFALL");
    controller
        .press(GameButton::A)
        .expect("native Waterfall prompt");
    for _ in 0..256 {
        if position(&mut controller) == (20, 65)
            && controller.shell.visible_waterfall_animation.is_none()
            && !controller.has_pending_script_work()
            && controller.shell.field_notice.is_none()
        {
            break;
        }
        if controller.shell.visible_waterfall_animation.is_some()
            && controller.shell.field_notice.is_none()
        {
            // Animation owns input after the use text. Repeated A must not
            // reopen the contextual move while its native source climb waits.
            controller
                .wait_frames(16)
                .expect("native neutral Waterfall clock");
        } else {
            controller
                .press(GameButton::A)
                .expect("Waterfall confirmation/use dialogue");
            controller
                .wait_frames(16)
                .expect("native per-step waterfall presentation");
        }
    }
    assert!(
        controller.shell.flygon_field_actions.contains("waterfall"),
        "must execute the real native field move"
    );
    assert_eq!(
        position(&mut controller),
        (20, 65),
        "renderer-neutral Waterfall clock must execute native source steps; animation={:?}, pending={:?}",
        controller.shell.visible_waterfall_animation,
        controller
            .shell
            .shell
            .session()
            .state()
            .script_runtime
            .pending_waterfall_field_move
    );
    assert_eq!(collision(&runtime, &controller, (20, 65)), "WATER");
    for y in (61..65).rev() {
        water_step(&mut controller, GameButton::Up, (20, y));
    }
    assert_eq!(
        controller
            .snapshot()
            .expect("actual dry summit bank")
            .overworld
            .mode,
        MovementMode::Normal
    );
    assert_eq!(collision(&runtime, &controller, (20, 61)), "FLOOR");
    // Walk on the optional bank after dismounting. The old dry summit trapped
    // this branch because every adjacent water cell was unmountable Waterfall.
    water_step(&mut controller, GameButton::Right, (21, 61));
    water_step(&mut controller, GameButton::Left, (20, 61));
    mount_surf(&mut controller, GameButton::Down, (20, 62));
    for y in 63..=65 {
        water_step(&mut controller, GameButton::Down, (20, y));
    }
    // WATERFALL is a native downward current. A production press may settle
    // several forced tiles at once, so prove the real landing without assuming
    // an artificial stop at every intermediate cascade cell.
    for _ in 0..64 {
        if position(&mut controller) == (20, 72) {
            break;
        }
        controller
            .press(GameButton::Down)
            .expect("native downward cascade return");
        controller
            .wait_frames(16)
            .expect("forced current presentation");
        settle_water(&mut controller);
        let at = position(&mut controller);
        assert_eq!(at.0, 20);
        assert!((65..=72).contains(&at.1));
    }
    assert_eq!(position(&mut controller), (20, 72));
    for x in 21..=25 {
        water_step(&mut controller, GameButton::Right, (x, 72));
    }
    water_step(&mut controller, GameButton::Up, (25, 71));
    assert_eq!(
        controller
            .snapshot()
            .expect("safe bank return")
            .overworld
            .mode,
        MovementMode::Normal
    );
}
