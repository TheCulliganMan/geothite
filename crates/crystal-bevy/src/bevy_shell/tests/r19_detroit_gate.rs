//! Gatehouse denial, qualified passage and free return through production input.
use std::path::PathBuf;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{finish_text, indoor_walk};
use super::winona::{at, press, travel};

const ROOM: &str = "MnR19DetroitGate";
const TOWN: &str = "MnDetroitLakes";
const ROUTE: &str = "MnR19C00R01";

fn fixture(
    map: &str,
    x: i16,
    y: i16,
    qualified: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_R19_GATE_TEST_PACK").expect("published gate candidate"),
    );
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified gate pack"),
    )
    .expect("gate runtime");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: map.into(),
            tile_x: x,
            tile_y: y,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit local gate fixture");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture name");
    // Initial badge/position/empty party are geometry setup, not earned access.
    let state = controller.shell.shell.session_mut().state_mut();
    state.badges.johto = [
        qualified, qualified, qualified, qualified, qualified, qualified, false, false,
    ];
    mark_runtime_snapshot_dirty(&mut controller.shell);
    controller.set_runtime_journal_enabled(false);
    at(&mut controller, map, (x, y));
    (root, runtime, controller)
}

fn leave_south(c: &mut VisibleShellController, r: &CrystalRuntime) {
    indoor_walk(c, r, (4, 6));
    travel(c, GameButton::Down, TOWN, (25, 4));
    press(c, GameButton::Down);
    at(c, TOWN, (25, 5));
}

#[test]
#[ignore = "requires published R19 gate candidate"]
fn r19_detroit_gate_denies_repeated_northbound_entry_and_allows_town_return() {
    let (_, runtime, mut c) = fixture(TOWN, 25, 5, false);
    travel(&mut c, GameButton::Up, ROOM, (4, 6));
    indoor_walk(&mut c, &runtime, (4, 3));
    if let Some(dir) = std::env::var_os("CRYSTAL_R19_GATE_SAVE_DIR") {
        let dir = PathBuf::from(dir);
        std::fs::create_dir_all(&dir).expect("denial source save directory");
        c.save(&dir.join(format!(
            "geothite-r19-gate-unqualified-{}.crystalsave",
            std::process::id()
        )))
        .expect("save actual unqualified gate approach for production replay");
    }
    for _ in 0..3 {
        let history = c.shell.dialogue_log_events.len();
        // The forced return faces south. Match the production move command:
        // allow its bounded directional retries to turn and enter the check.
        for _ in 0..4 {
            c.press(GameButton::Up).expect("native denial input");
            if c.snapshot().expect("denial approach").ui.text.is_some() {
                break;
            }
            at(&mut c, ROOM, (4, 3));
        }
        let denial_snapshot = c.snapshot().expect("visible denial textbox");
        assert!(
            denial_snapshot.ui.text.is_some(),
            "denial UI missing: phase={:?}; position={:?}; ui={:?}; field_notice={:?}; active_cursor={:?}; dialogue={:?}; actions={:?}",
            denial_snapshot.phase,
            denial_snapshot.overworld.tile,
            denial_snapshot.ui,
            c.shell.field_notice,
            c.shell.active_script_cursor,
            c.shell
                .dialogue_log_events
                .iter()
                .skip(history)
                .collect::<Vec<_>>(),
            c.shell
                .last_audio_events
                .iter()
                .rev()
                .take(16)
                .collect::<Vec<_>>(),
        );
        finish_text(&mut c);
        let shown = c
            .shell
            .dialogue_log_events
            .iter()
            .skip(history)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            shown.contains("needs six BADGES.") && shown.contains("Please return"),
            "native denial pages missing: {shown}"
        );
        at(&mut c, ROOM, (4, 3));
        assert_eq!(
            c.snapshot()
                .expect("no badge grants")
                .progression
                .badges
                .johto,
            [false; 8]
        );
    }
    leave_south(&mut c, &runtime);
    assert!(c.shell.last_error.is_none());
}

#[test]
#[ignore = "requires published R19 gate candidate"]
fn r19_detroit_gate_qualified_round_trip_and_inside_continue() {
    let (root, runtime, mut c) = fixture(TOWN, 25, 5, true);
    travel(&mut c, GameButton::Up, ROOM, (4, 6));
    indoor_walk(&mut c, &runtime, (4, 3));
    let dir = std::env::var_os("CRYSTAL_R19_GATE_SAVE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    std::fs::create_dir_all(&dir).expect("gate save directory");
    let save = dir.join(format!(
        "geothite-r19-gate-inside-{}.crystalsave",
        std::process::id()
    ));
    let before = c.snapshot().expect("before Continue");
    c.save(&save).expect("native inside gate save");
    let mut c = VisibleShellController::load_save(root, runtime.clone(), save, None)
        .expect("Continue inside gate");
    let after = c.snapshot().expect("after Continue");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    at(&mut c, ROOM, (4, 3));
    indoor_walk(&mut c, &runtime, (4, 1));
    travel(&mut c, GameButton::Up, ROUTE, (25, 8));
    travel(&mut c, GameButton::Down, ROOM, (4, 1));
    leave_south(&mut c, &runtime);
    assert!(c.shell.last_error.is_none());
}

#[test]
#[ignore = "requires published R19 gate candidate"]
fn r19_detroit_gate_unqualified_southbound_return_is_free() {
    let (_, runtime, mut c) = fixture(ROUTE, 25, 8, false);
    travel(&mut c, GameButton::Down, ROOM, (4, 1));
    leave_south(&mut c, &runtime);
    assert_eq!(
        c.snapshot()
            .expect("inbound badge state")
            .progression
            .badges
            .johto,
        [false; 8]
    );
    assert!(c.shell.last_error.is_none());
}
