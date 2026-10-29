//! Local R19 geometry regression for the published oak/prairie candidate.
//! Placement is an explicit empty-party fixture. The dry watch-lane subcase also
//! starts with Merrit's defeat flag set; it does not claim an earned victory.
//! After initial setup every movement uses VisibleShellController::press.
use std::collections::BTreeMap;
use std::path::PathBuf;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_runtime::{CrystalRuntime, RuntimeShellPhase};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{Point, RouteGrid, RouteSpec, position};

const MERRIT: &str = "EVENT_MN_R19_TRAINER_01";

fn fixture(
    at: Point,
    merrit_already_defeated: bool,
) -> (CrystalRuntime, VisibleShellController, RouteGrid) {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_R19_QUALITY_TEST_PACK")
            .expect("published external R19 oak/prairie candidate pack"),
    )
    .canonicalize()
    .expect("existing native candidate pack");
    let root = AssetRoot::new(path.parent().expect("pack directory"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified external candidate"),
    )
    .expect("native R19 runtime");
    let shell = initialize_bevy_runtime_shell(
        root,
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: format!("MnR19C00R{:02}", at.1 / 48),
            tile_x: at.0,
            tile_y: at.1 % 48,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit initial empty-party placement");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("initial player name");
    if merrit_already_defeated {
        controller
            .shell
            .shell
            .session_mut()
            .state_mut()
            .flags
            .set_event_flag(MERRIT, true)
            .expect("explicit initial dry-route geometry fixture");
    }
    controller.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    let mut grid = RouteGrid::new_with_grass(
        &runtime,
        RouteSpec {
            prefix: "MnR19",
            columns: 1,
            rows: 2,
            tileset: "johto",
            start: at,
            doors: BTreeMap::new(),
        },
    );
    if merrit_already_defeated {
        // The planner is not game state. The native initial flag above is the
        // only reason this subcase may cross Merrit's sight column safely.
        for y in 22..=24 {
            grid.hazards.remove(&(43, y));
            grid.floor.insert((43, y));
        }
    }
    assert_eq!(position(&mut controller), at);
    assert!(
        controller
            .snapshot()
            .expect("initial party")
            .party
            .slots
            .is_empty()
    );
    assert_eq!(event(&controller), merrit_already_defeated);
    (runtime, controller, grid)
}

fn event(controller: &VisibleShellController) -> bool {
    controller
        .shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(MERRIT)
        .expect("existing native trainer event")
}

fn collision(runtime: &CrystalRuntime, at: Point) -> &str {
    let module = &runtime.data().maps[&format!("MnR19C00R{:02}", at.1 / 48)];
    let width = usize::from(module.attributes.width);
    let block = module.blocks[(at.1 as usize % 48) / 2 * width + at.0 as usize / 2];
    &runtime.data().tilesets[&module.attributes.tileset_name].collision[&format!("{block:02x}")]
        [(at.1 % 2 * 2 + at.0 % 2) as usize]
}

fn press(controller: &mut VisibleShellController, button: GameButton) {
    controller.press(button).expect("production R19 input");
    controller
        .wait_frames(16)
        .expect("settled field presentation");
    assert_eq!(controller.shell.last_error, None);
    let snapshot = controller.snapshot().expect("native field state");
    assert_eq!(snapshot.phase, RuntimeShellPhase::Overworld);
    assert!(
        snapshot.battle.is_none(),
        "empty-party geometry crossed trainer ownership"
    );
}

fn step(controller: &mut VisibleShellController, to: Point, context: &str) {
    let from = position(controller);
    let button = match (to.0 - from.0, to.1 - from.1) {
        (1, 0) => GameButton::Right,
        (-1, 0) => GameButton::Left,
        (0, 1) => GameButton::Down,
        (0, -1) => GameButton::Up,
        _ => panic!("{context}: expected ordinary adjacent step {from:?} -> {to:?}"),
    };
    for _ in 0..8 {
        press(controller, button);
        let at = position(controller);
        if at != from {
            assert_eq!(at, to, "{context}: ordinary input {button:?} from {from:?}");
            return;
        }
    }
    panic!("{context}: native input never moved {from:?} -> {to:?}");
}

fn blocked(controller: &mut VisibleShellController, button: GameButton, context: &str) {
    let before = position(controller);
    for _ in 0..3 {
        press(controller, button);
        assert_eq!(position(controller), before, "{context}: {button:?}");
    }
}

fn course(
    runtime: &CrystalRuntime,
    grid: &RouteGrid,
    controller: &mut VisibleShellController,
    waypoints: &[Point],
) -> (usize, usize) {
    let mut steps = 0;
    let mut grass = 0;
    for &target in waypoints {
        let from = position(controller);
        for button in grid.path(from, target) {
            let at = position(controller);
            let next = match button {
                GameButton::Left => (at.0 - 1, at.1),
                GameButton::Right => (at.0 + 1, at.1),
                GameButton::Up => (at.0, at.1 - 1),
                GameButton::Down => (at.0, at.1 + 1),
                _ => panic!("fixture planner returned nonmovement input"),
            };
            step(controller, next, "authored walking course");
            steps += 1;
            grass += usize::from(collision(runtime, next) == "TALL_GRASS");
        }
        assert_eq!(position(controller), target);
    }
    (steps, grass)
}

#[test]
#[ignore = "requires the published external R19 oak/prairie candidate"]
fn route_quality_r19_controller_headland_two_drop_lanes_returns_and_old_ramps() {
    let (runtime, mut controller, grid) = fixture((60, 12), false);
    let headland = [(50, 12), (50, 10), (46, 10), (46, 16), (48, 16), (49, 19)];
    // The same 28-step course now crosses the authored pine-bend habitat.
    // Its waypoints enter nine grass tiles; the independent minimum is seven.
    assert_eq!(course(&runtime, &grid, &mut controller, &headland), (28, 9));
    grid.walk(&mut controller, (35, 31));

    // The eastern course side is approached from its real upper Floor pocket.
    // The first Down walks onto the lip; it must not already perform the jump.
    step(&mut controller, (35, 32), "east shoulder lip approach");
    step(&mut controller, (36, 32), "actual upper course-side pocket");
    assert_eq!(collision(&runtime, (36, 33)), "WALL");
    blocked(
        &mut controller,
        GameButton::Down,
        "eastern course-side bypass",
    );
    step(&mut controller, (35, 32), "leave upper shoulder");
    step(&mut controller, (35, 31), "return from lip to upper road");

    for (x, expected_steps) in [(34, 9), (35, 11)] {
        grid.walk(&mut controller, (x, 31));
        assert_eq!(collision(&runtime, (x, 32)), "HOP_DOWN");
        assert_eq!(collision(&runtime, (x, 33)), "WALL");
        step(
            &mut controller,
            (x, 32),
            "ordinary first step onto native lip",
        );
        press(&mut controller, GameButton::Down);
        assert_eq!(
            position(&mut controller),
            (x, 34),
            "lane{x}: actual native hop"
        );
        blocked(
            &mut controller,
            GameButton::Up,
            "reverse hop rejects native Wall course",
        );
        if x == 34 {
            step(&mut controller, (33, 34), "western course-side approach");
            assert_eq!(collision(&runtime, (33, 33)), "WALL");
            blocked(
                &mut controller,
                GameButton::Up,
                "western course-side bypass",
            );
            step(&mut controller, (34, 34), "restore landing by actual input");
        }
        let (steps, grass) = course(&runtime, &grid, &mut controller, &[(x, 31)]);
        assert_eq!(steps, expected_steps, "lane{x}: ordinary uphill return");
        assert_eq!(grass, 3, "lane{x}: actual grass cost, no dry bypass");
    }
    // Existing two-way native74 ramps are preserved, not replaced with jumps.
    for x in [11, 19] {
        grid.walk(&mut controller, (x, 38));
        assert_eq!(course(&runtime, &grid, &mut controller, &[(x, 35)]), (3, 0));
        step(&mut controller, (x, 36), "old ramp upper-right quadrant");
        blocked(
            &mut controller,
            GameButton::Left,
            "old ramp upper-left Wall",
        );
        step(&mut controller, (x, 37), "old ramp lower-right quadrant");
        step(&mut controller, (x, 38), "old ramp ordinary lower arrival");
    }
    assert!(!event(&controller));
    assert!(
        controller
            .snapshot()
            .expect("unchanged empty party")
            .party
            .slots
            .is_empty()
    );
    // Keep the original prairie_forest_link observation/Headbutt/Continue tests
    // registered and run them separately; this leaf adds geometry coverage only.
}

#[test]
#[ignore = "requires the published external R19 oak/prairie candidate"]
fn route_quality_r19_controller_watch_route_zero_grass_and_eight_grass_alternative() {
    for defeated in [false, true] {
        let (runtime, mut controller, grid) = fixture((49, 23), defeated);
        if defeated {
            let direct = [
                (49, 24),
                (43, 24),
                (43, 22),
                (42, 22),
                (42, 21),
                (39, 21),
                (39, 23),
            ];
            let (_, grass) = course(&runtime, &grid, &mut controller, &direct);
            assert_eq!(
                grass, 0,
                "dry native direct battle route after explicit initial victory flag"
            );
            let reverse = [
                (39, 21),
                (42, 21),
                (42, 22),
                (43, 22),
                (43, 24),
                (49, 24),
                (49, 23),
            ];
            assert_eq!(course(&runtime, &grid, &mut controller, &reverse).1, 0);
        } else {
            let peaceful = [(49, 29), (35, 29), (35, 23), (39, 23)];
            assert_eq!(course(&runtime, &grid, &mut controller, &peaceful), (30, 8));
            let reverse = [(35, 23), (35, 29), (49, 29), (49, 23)];
            assert_eq!(course(&runtime, &grid, &mut controller, &reverse), (30, 8));
        }
        assert_eq!(
            event(&controller),
            defeated,
            "geometry must not defeat Merrit"
        );
        assert!(
            controller
                .snapshot()
                .expect("empty geometry party")
                .party
                .slots
                .is_empty()
        );
    }
}

/// Party, initial placement and RNG are local fixtures; encounters and the
/// journey thereafter use native inputs without rewriting game state.
#[test]
#[ignore = "requires the current R19 grass candidate"]
fn route_quality_r19_controller_pine_grass_wild_run_return_and_continue() {
    use super::authored_route_fixture::{cleanup_reached_save, escape_wild, reached_save_path};

    let (runtime, mut controller, grid) = fixture((47, 13), false);
    let partner = runtime
        .data()
        .create_pokemon("MEW", 60, crystal_core::models::Dv::default())
        .expect("explicit native fixture partner; not authored Minnesota content");
    let state = controller.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut controller.shell);

    let mut grass = 0;
    grid.walk_with_actions(
        &mut controller,
        (49, 20),
        |c| grass += usize::from(collision(&runtime, position(c)) == "TALL_GRASS"),
        escape_wild,
    );
    assert!(grass >= 7, "pine approach must cross the new habitat");
    grid.walk_with_actions(&mut controller, (49, 17), |_| {}, escape_wild);
    assert_eq!(collision(&runtime, (49, 17)), "TALL_GRASS");
    assert_eq!(collision(&runtime, (49, 18)), "TALL_GRASS");
    let mut encounters = 0;
    for index in 0..256 {
        let target = if index % 2 == 0 { (49, 18) } else { (49, 17) };
        grid.walk_with_actions(
            &mut controller,
            target,
            |_| {},
            |c| {
                encounters += 1;
                escape_wild(c);
            },
        );
        if encounters > 0 {
            break;
        }
    }
    assert!(
        encounters > 0,
        "new grass must start a native wild encounter"
    );
    grid.walk_with_actions(&mut controller, (47, 13), |_| {}, escape_wild);
    let before = controller.snapshot().expect("actual reached pine return");
    let flags = controller.shell.shell.session().state().flags.clone();
    let save = reached_save_path(&format!(
        "geothite-r19-grass-{}.crystalsave",
        std::process::id()
    ));
    controller.save(&save).expect("save actual pine return");
    let pack = PathBuf::from(std::env::var_os("CRYSTAL_R19_QUALITY_TEST_PACK").expect("R19 pack"));
    let root = AssetRoot::new(pack.parent().expect("external pack parent"));
    let mut loaded = VisibleShellController::load_save(root, runtime, save.clone(), None)
        .expect("Continue actual pine return");
    let after = loaded.snapshot().expect("continued resources");
    assert_eq!(position(&mut loaded), (47, 13));
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(loaded.shell.shell.session().state().flags, flags);
    if let Some(path) = std::env::var_os("CRYSTAL_R19_GRASS_REACHED_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("retain reached save");
    }
    cleanup_reached_save(&save);
}
