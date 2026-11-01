//! Empty-party local geometry fixtures followed only by production inputs.
//!
//! This leaf targets the final R08 wooded-side candidate. It proves
//! neither earned regional access nor trainer victories/encounter balance. The
//! populated Northern Lakes journey remains a separate regression. The long
//! whole-lake alternate (183/185 steps when the local passage is closed) is a
//! separately labeled source proof, not the local 13/11-step return tested here.
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_runtime::{CrystalRuntime, RuntimeShellPhase};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{Point, RouteGrid, RouteSpec, neighbors, position};

fn load() -> (AssetRoot, CrystalRuntime) {
    static LOADED: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            let path = PathBuf::from(
                std::env::var_os("CRYSTAL_R08_SHORE_CONTROLLER_PACK")
                    .expect("actual published R08 shore candidate pack"),
            )
            .canonicalize()
            .expect("canonical completed R08 pack");
            let root = AssetRoot::new(path.parent().expect("pack parent"));
            let runtime = CrystalRuntime::from_loaded_compiled_pack(
                &root,
                read_loaded_verified_compiled_game_pack(path).expect("strict published R08 pack"),
            )
            .expect("native R08 runtime");
            (root, runtime)
        })
        .clone()
}

fn kind(runtime: &CrystalRuntime, (x, y): Point) -> &str {
    assert!((0..80).contains(&x) && (0..96).contains(&y));
    let name = format!("MnR08C{:02}R{:02}", x / 64, y / 48);
    let module = &runtime.data().maps[&name];
    let local_x = (x % 64) as usize;
    let local_y = (y % 48) as usize;
    let block = module.blocks[local_y / 2 * usize::from(module.attributes.width) + local_x / 2];
    &runtime.data().tilesets["mn_headwaters"].collision[&format!("{block:02x}")]
        [local_y % 2 * 2 + local_x % 2]
}

fn fixture(at: Point) -> (AssetRoot, CrystalRuntime, VisibleShellController, RouteGrid) {
    let (root, runtime) = load();
    assert_eq!(kind(&runtime, at), "FLOOR", "legal initial standing tile");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime
                .title_new_game_spawn_identifier()
                .expect("retained HOME"),
            map_name: format!("MnR08C{:02}R{:02}", at.0 / 64, at.1 / 48),
            tile_x: at.0 % 64,
            tile_y: at.1 % 48,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".to_string()),
            ..Default::default()
        },
    )
    .expect("explicit local empty-party R08 placement");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture name");
    controller.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    assert_eq!(position(&mut controller), at);
    assert!(
        controller
            .snapshot()
            .expect("empty geometry party")
            .party
            .slots
            .is_empty()
    );
    // Setup ends here. No party, bag, badge, flag, mode, map or position writes
    // follow. The empty party uses native encounter/trainer eligibility rules.
    let grid = RouteGrid::new_with_grass(
        &runtime,
        RouteSpec {
            prefix: "MnR08",
            columns: 2,
            rows: 2,
            tileset: "mn_headwaters",
            start: at,
            doors: BTreeMap::new(),
        },
    );
    (root, runtime, controller, grid)
}

fn press(controller: &mut VisibleShellController, button: GameButton) {
    controller.press(button).expect("native R08 input");
    controller
        .wait_frames(16)
        .expect("native input presentation");
    let snapshot = controller.snapshot().expect("settled geometry input");
    assert!(
        snapshot.battle.is_none(),
        "geometry fixture cannot claim a battle"
    );
    assert_eq!(snapshot.phase, RuntimeShellPhase::Overworld);
    assert_eq!(controller.shell.last_error, None);
    assert!(snapshot.party.slots.is_empty());
}

fn step(controller: &mut VisibleShellController, button: GameButton, target: Point) {
    for _ in 0..8 {
        if position(controller) == target {
            return;
        }
        press(controller, button);
    }
    assert_eq!(
        position(controller),
        target,
        "turn-first native displacement"
    );
}

fn blocked(controller: &mut VisibleShellController, button: GameButton) {
    let before = position(controller);
    for _ in 0..4 {
        press(controller, button);
        assert_eq!(
            position(controller),
            before,
            "native side/reverse collision"
        );
    }
}

fn course(controller: &mut VisibleShellController, grid: &RouteGrid, points: &[Point]) {
    for &point in points {
        grid.walk(controller, point);
        assert_eq!(position(controller), point);
        let snapshot = controller.snapshot().expect("completed route waypoint");
        assert!(snapshot.party.slots.is_empty());
        assert_eq!(snapshot.phase, RuntimeShellPhase::Overworld);
        assert_eq!(controller.shell.last_error, None);
    }
}

fn ordinary_return(
    controller: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    path: &[Point],
) -> usize {
    assert_eq!(position(controller), path[0]);
    let mut grass_steps = 0;
    for pair in path.windows(2) {
        let (from, target) = (pair[0], pair[1]);
        assert_eq!(position(controller), from);
        assert!(matches!(kind(runtime, target), "FLOOR" | "TALL_GRASS"));
        let button = neighbors(from)
            .into_iter()
            .find_map(|(point, button)| (point == target).then_some(button))
            .expect("one ordinary step; a hop cannot substitute for ascent");
        step(controller, button, target);
        grass_steps += usize::from(kind(runtime, position(controller)) == "TALL_GRASS");
    }
    grass_steps
}

#[test]
#[ignore = "requires final published R08 shore pack; empty-party local geometry only"]
fn route_quality_r08_inner_shore_headland_and_south_bank_walk_both_directions() {
    let (_root, runtime, mut controller, grid) = fixture((73, 23));
    assert_eq!(kind(&runtime, (69, 33)), "FLOOR");
    assert_eq!(
        kind(&runtime, (73, 33)),
        "WALL",
        "old broad lane closes into forest"
    );
    let north = [(69, 25), (65, 26), (69, 27), (69, 33), (67, 35), (63, 37)];
    course(&mut controller, &grid, &north);
    let headland = [
        (49, 51),
        (53, 51),
        (53, 57),
        (63, 57),
        (63, 55),
        (69, 55),
        (69, 59),
        (73, 59),
        (73, 62),
    ];
    course(&mut controller, &grid, &headland);
    // The planner retains actual actor bodies and trainer sight as hazards;
    // navigation to the lower side can take the existing whole-lake road.
    course(
        &mut controller,
        &grid,
        &[(73, 65), (73, 71), (71, 72), (73, 77)],
    );
    course(&mut controller, &grid, &[(75, 79), (73, 80)]);
    let south = [
        (73, 87),
        (63, 87),
        (59, 88),
        (57, 88),
        (53, 88),
        (49, 89),
        (45, 87),
        (45, 85),
        (39, 85),
        (39, 87),
        (33, 87),
        (33, 89),
        (27, 89),
    ];
    course(&mut controller, &grid, &south);
    for points in [&south[..], &headland[..], &north[..]] {
        let reverse = points.iter().rev().copied().collect::<Vec<_>>();
        course(&mut controller, &grid, &reverse);
    }
    grid.walk(&mut controller, (73, 23));
}

#[test]
#[ignore = "requires final published R08 shore pack; exact native two-lane drop/return"]
fn route_quality_r08_pier_drop_both_lanes_reverse_sides_and_exact_grassy_returns() {
    let (_root, runtime, mut controller, grid) = fixture((73, 77));
    assert_eq!(kind(&runtime, (71, 79)), "WALL");
    assert_eq!(kind(&runtime, (74, 79)), "HEADBUTT_TREE");
    assert!(!kind(&runtime, (74, 79)).contains("CUT"));
    for x in [72, 73] {
        grid.walk(&mut controller, (x, 77));
        assert_eq!(kind(&runtime, (x, 78)), "HOP_DOWN");
        assert_eq!(kind(&runtime, (x, 79)), "WALL");
        // Move onto the actual native lip, then jump two strides over its wall.
        step(&mut controller, GameButton::Down, (x, 78));
        step(&mut controller, GameButton::Down, (x, 80));
        blocked(&mut controller, GameButton::Up);
        let mut path = vec![
            (73, 80),
            (73, 81),
            (73, 82),
            (74, 82),
            (75, 82),
            (75, 81),
            (75, 80),
            (75, 79),
            (75, 78),
            (75, 77),
            (74, 77),
            (73, 77),
        ];
        if x == 72 {
            path.insert(0, (72, 80));
            path.push((72, 77));
        }
        assert_eq!(path.len() - 1, if x == 72 { 13 } else { 11 });
        assert_eq!(ordinary_return(&mut controller, &runtime, &path), 4);
        assert_eq!(position(&mut controller), (x, 77));
    }
    grid.walk(&mut controller, (75, 79));
    blocked(&mut controller, GameButton::Left);
    grid.walk(&mut controller, (71, 80));
    blocked(&mut controller, GameButton::Up);
}

#[test]
#[ignore = "requires final published R08 shore pack; retained original Birch geometry"]
fn route_quality_r08_original_birch_drop_and_ordinary_ramp_return_remain() {
    let (_root, runtime, mut controller, grid) = fixture((35, 27));
    grid.walk(&mut controller, (29, 25));
    assert_eq!(kind(&runtime, (29, 26)), "HOP_DOWN");
    assert_eq!(kind(&runtime, (29, 27)), "WALL");
    step(&mut controller, GameButton::Down, (29, 26));
    step(&mut controller, GameButton::Down, (29, 28));
    blocked(&mut controller, GameButton::Up);
    let buttons = grid.path((29, 28), (29, 25));
    assert!(
        buttons.len() >= 9,
        "ordinary climb cannot use the reverse wall"
    );
    let mut path = vec![(29, 28)];
    for button in buttons {
        let from = *path.last().expect("current ordinary path point");
        let target = neighbors(from)
            .into_iter()
            .find_map(|(point, direction)| (direction == button).then_some(point))
            .expect("ordinary cardinal direction");
        path.push(target);
    }
    ordinary_return(&mut controller, &runtime, &path);
    assert!(
        path.contains(&(35, 27)),
        "original native ramp is the useful ascent"
    );
    assert_eq!(position(&mut controller), (29, 25));
}
