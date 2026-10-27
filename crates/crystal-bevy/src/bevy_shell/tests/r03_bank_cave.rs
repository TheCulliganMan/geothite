//! One local empty-party position/RNG fixture followed only by production input.
//! This does not earn River Valley access, badges, battles or regional progress.
use std::collections::{BTreeMap, VecDeque};
use std::path::PathBuf;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::world::map::Direction;
use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleFieldItemPhase, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{Point, RouteGrid, RouteSpec, indoor_walk};
use super::r10_gallery::{
    completed_visible_page, continue_save, flags, map, press, remember_dialogue, settle, tile,
};

const ROOM: &str = "MnR03Cave1F";
const OUTSIDE: &str = "MnR03C00R00";
const EVENT: &str = "EVENT_MN_R03_ITEM_02";
const ITEM: &str = "GREAT_BALL";

fn source() -> Value {
    let path = std::env::var_os("CRYSTAL_R03_BANK_SOURCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
                "../../planning/minnesota/renders/caves/r03-candidate/v02/MnR03Cave1F.candidate.json",
            )
        });
    serde_json::from_str(&std::fs::read_to_string(path).expect("frozen50738c06 source"))
        .expect("bank cave source")
}

fn fixture() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_R03_BANK_CONTROLLER_PACK")
            .expect("actual revised R03 native pack"),
    )
    .canonicalize()
    .expect("published bank cave pack");
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(path).expect("strict native pack"),
    )
    .expect("native runtime");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime
                .title_new_game_spawn_identifier()
                .expect("retained HOME"),
            map_name: OUTSIDE.to_string(),
            tile_x: 8,
            tile_y: 38,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".to_string()),
            ..Default::default()
        },
    )
    .expect("explicit local bank approach");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture name");
    let state = controller.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.pokemon.iter().all(Option::is_none));
    state.money = 1000;
    state.random_state = Default::default();
    *controller.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(
            (0_u32..262144)
                .map(|n| ((n * 73 + 19) % 256) as u8)
                .collect::<Vec<_>>(),
        );
    // All explicit setup ends here; no later location, flag, party or bag writes.
    controller.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    assert_eq!(tile(&mut controller), (8, 38));
    assert!(!collected(&controller));
    assert_eq!(quantity(&controller, &runtime), 0);
    let authored = source();
    let expected = authored["rows"]
        .as_array()
        .expect("frozen rows")
        .iter()
        .flat_map(|row| row.as_str().expect("row").split_whitespace())
        .map(|block| u16::from_str_radix(block, 16).expect("native block"))
        .collect::<Vec<_>>();
    let room = &runtime.data().maps[ROOM];
    assert_eq!([room.attributes.width, room.attributes.height], [8, 6]);
    assert_eq!(room.blocks, expected, "actual installed frozen geometry");
    assert_eq!(room.attributes.time_of_day.as_deref(), Some("nite"));
    assert!(room.script_text_commands.iter().any(|command| {
        command.source_script == "MN_R03_CAVE_WALKER"
            && command.command == "jumptextfaceplayer"
            && command.command_index == 0
            && command.text_label.as_deref() == Some("MN_R03_CAVE_WALKERText")
    }));
    (root, runtime, controller)
}

fn collected(controller: &VisibleShellController) -> bool {
    controller
        .shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(EVENT)
        .expect("original Great Ball event")
}

fn quantity(controller: &VisibleShellController, runtime: &CrystalRuntime) -> u16 {
    controller
        .shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&runtime.data().items[ITEM])
}

fn kind(runtime: &CrystalRuntime, at: Point) -> &str {
    let room = &runtime.data().maps[ROOM];
    let block = room.blocks[at.1 as usize / 2 * 8 + at.0 as usize / 2];
    &runtime.data().tilesets["cave"].collision[&format!("{block:02x}")]
        [at.1 as usize % 2 * 2 + at.0 as usize % 2]
}

fn facing(button: GameButton) -> Direction {
    match button {
        GameButton::Up => Direction::Up,
        GameButton::Down => Direction::Down,
        GameButton::Left => Direction::Left,
        GameButton::Right => Direction::Right,
        _ => panic!("movement direction required"),
    }
}

fn step(controller: &mut VisibleShellController, button: GameButton, target: Point) {
    let before = tile(controller);
    let previous_facing = controller
        .snapshot()
        .expect("before native step")
        .overworld
        .facing;
    press(controller, button);
    assert_eq!(
        controller
            .snapshot()
            .expect("native facing")
            .overworld
            .facing,
        facing(button)
    );
    if previous_facing != facing(button) {
        assert_eq!(
            tile(controller),
            before,
            "first directional press turns before walking"
        );
    }
    for _ in 0..8 {
        let at = tile(controller);
        if at == target {
            return;
        }
        assert_eq!(at, before, "unexpected intermediate native movement");
        press(controller, button);
    }
    assert_eq!(tile(controller), target);
}

fn blocked(controller: &mut VisibleShellController, button: GameButton) {
    let before = tile(controller);
    for _ in 0..4 {
        press(controller, button);
        assert_eq!(tile(controller), before, "actual native directional wall");
    }
}

fn outside(runtime: &CrystalRuntime) -> RouteGrid {
    RouteGrid::new_with_grass(
        runtime,
        RouteSpec {
            prefix: "MnR03",
            columns: 1,
            rows: 2,
            tileset: "mn_headwaters",
            start: (8, 38),
            doors: BTreeMap::from([((8, 37), ROOM)]),
        },
    )
}

fn enter(controller: &mut VisibleShellController, runtime: &CrystalRuntime) {
    outside(runtime).walk(controller, (8, 37));
    assert_eq!(map(controller), ROOM);
    assert_eq!(tile(controller), (7, 9), "original insidewarp1 arrival");
    indoor_walk(controller, runtime, (7, 8));
    assert_eq!(tile(controller), (7, 8));
}

fn leave(controller: &mut VisibleShellController, runtime: &CrystalRuntime) {
    indoor_walk(controller, runtime, (7, 8));
    indoor_walk(controller, runtime, (7, 9));
    assert_eq!(map(controller), OUTSIDE);
    assert_eq!(tile(controller), (8, 38), "original passive exteriorwarp2");
    controller
        .wait_frames(60)
        .expect("stable genuine passive return");
    assert_eq!(map(controller), OUTSIDE);
    assert_eq!(tile(controller), (8, 38));
    let grid = outside(runtime);
    grid.walk(controller, (9, 38));
    grid.walk(controller, (8, 38));
    assert_eq!(
        map(controller),
        OUTSIDE,
        "revisiting Floor must not reenter"
    );
    assert_eq!(tile(controller), (8, 38));
}

fn walker(controller: &mut VisibleShellController, runtime: &CrystalRuntime) {
    indoor_walk(controller, runtime, (5, 6));
    press(controller, GameButton::Up);
    assert_eq!(tile(controller), (5, 6));
    press(controller, GameButton::A);
    press(controller, GameButton::Down);
    assert_eq!(tile(controller), (5, 6), "original dialogue owns movement");
    let mut pages = BTreeMap::new();
    let mut diagnostics = VecDeque::new();
    for _ in 0..32 {
        controller
            .wait_frames(240)
            .expect("print original walker page");
        remember_dialogue(controller, &mut diagnostics);
        if let Some((index, page)) = completed_visible_page(controller) {
            assert_eq!(
                controller
                    .snapshot()
                    .expect("walker text identity")
                    .ui
                    .text
                    .as_ref()
                    .map(|text| text.label.as_str()),
                Some("MN_R03_CAVE_WALKERText")
            );
            pages.insert(index, page);
        }
        let snapshot = controller.snapshot().expect("walker input ownership");
        if !snapshot.ui.text_window_open && !controller.has_pending_script_work() {
            break;
        }
        press(controller, GameButton::A);
    }
    assert_eq!(pages.len(), 3, "{diagnostics:#?}");
    for (index, expected) in [
        "A pocket of dry\nstone by the bank.",
        "The path curves\naround that rock.",
        "Take the far cache\nand circle back.",
    ]
    .iter()
    .enumerate()
    {
        assert_eq!(
            pages.get(&index).map(String::as_str),
            Some(*expected),
            "{diagnostics:#?}"
        );
    }
    settle(controller, "after all three original walker pages");
}

fn around_nose(controller: &mut VisibleShellController, runtime: &CrystalRuntime) {
    for at in [(6, 5), (6, 3), (7, 3), (8, 3)] {
        assert_eq!(kind(runtime, at), "FLOOR");
        indoor_walk(controller, runtime, at);
        assert_eq!(tile(controller), at);
    }
    assert_eq!(kind(runtime, (8, 2)), "UP_WALL");
    assert_eq!(kind(runtime, (9, 2)), "UP_WALL");
    step(controller, GameButton::Up, (8, 2));
    blocked(controller, GameButton::Up);
    step(controller, GameButton::Right, (9, 2));
    blocked(controller, GameButton::Up);
    step(controller, GameButton::Down, (9, 3));
    step(controller, GameButton::Right, (10, 3));
    step(controller, GameButton::Right, (11, 3));
    step(controller, GameButton::Down, (11, 4));
    step(controller, GameButton::Down, (11, 5));
    assert_eq!(kind(runtime, (8, 5)), "WALL");
    assert_eq!(kind(runtime, (9, 8)), "WALL");
}

fn collect(controller: &mut VisibleShellController, runtime: &CrystalRuntime) {
    press(controller, GameButton::Right);
    assert_eq!(tile(controller), (11, 5));
    press(controller, GameButton::A);
    assert!(collected(controller));
    assert_eq!(quantity(controller, runtime), 1);
    let before = tile(controller);
    press(controller, GameButton::Down);
    assert_eq!(tile(controller), before, "pickup pocket text owns movement");
    let mut pages = BTreeMap::new();
    let mut diagnostics = VecDeque::new();
    for _ in 0..32 {
        controller
            .wait_frames(240)
            .expect("print original Great Ball pocket page");
        remember_dialogue(controller, &mut diagnostics);
        if let Some((index, page)) = completed_visible_page(controller) {
            assert_eq!(
                controller
                    .shell
                    .visible_field_item_notice
                    .as_ref()
                    .expect("native item notice")
                    .phase,
                VisibleFieldItemPhase::PocketText
            );
            pages.insert(index, page);
        }
        if controller.shell.field_notice.is_none()
            && controller.shell.visible_field_item_notice.is_none()
            && !controller.has_pending_script_work()
        {
            break;
        }
        press(controller, GameButton::A);
    }
    let item = runtime.data().items[ITEM].name.replace('_', " ");
    assert_eq!(runtime.data().items[ITEM].pocket, "BALL");
    assert_eq!(
        pages.get(&0),
        Some(&format!("CHRIS put the\n{item} in")),
        "{diagnostics:#?}"
    );
    assert_eq!(
        pages.get(&1),
        Some(&format!("{item} in\nthe BALL POCKET.")),
        "{diagnostics:#?}"
    );
    settle(controller, "after full original Great Ball notice");
}

#[test]
#[ignore = "requires actual revised R03 bank cave pack and frozen50738c06 source"]
fn r03_bank_cave_controller_native_roof_walker_three_pages_great_ball_once_and_continue() {
    let (root, runtime, mut controller) = fixture();
    let initial_flags = flags(&controller);
    enter(&mut controller, &runtime);
    walker(&mut controller, &runtime);
    assert_eq!(flags(&controller), initial_flags);
    around_nose(&mut controller, &runtime);
    collect(&mut controller, &runtime);
    let mut expected_flags = initial_flags;
    expected_flags.0.insert(EVENT.to_string());
    assert_eq!(flags(&controller), expected_flags);
    leave(&mut controller, &runtime);
    enter(&mut controller, &runtime);
    indoor_walk(&mut controller, &runtime, (11, 5));
    assert_eq!(kind(&runtime, (12, 5)), "FLOOR");
    step(&mut controller, GameButton::Right, (12, 5));
    press(&mut controller, GameButton::A);
    settle(
        &mut controller,
        "original collected tile cannot grant another ball",
    );
    assert_eq!(tile(&mut controller), (12, 5));
    assert_eq!(quantity(&controller, &runtime), 1);
    assert!(collected(&controller));
    controller = continue_save(root, runtime.clone(), &mut controller);
    assert_eq!(
        tile(&mut controller),
        (12, 5),
        "Continue on actual collected Floor"
    );
    assert_eq!(quantity(&controller, &runtime), 1);
    assert_eq!(flags(&controller), expected_flags);
    step(&mut controller, GameButton::Left, (11, 5));
    leave(&mut controller, &runtime);
    assert_eq!(quantity(&controller, &runtime), 1);
    assert_eq!(controller.shell.shell.session().state().money, 1000);
}
