//! Cross the open paving, then face and read each regional board or plaque.
use std::path::{Path, PathBuf};

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton::{self, A, Down, Left, Right, Up};
use crystal_core::world::map::TilePosition;
use crystal_core::world::session::OverworldInteractionTarget;
use crystal_runtime::CrystalRuntime;

use super::super::brainerd_guide::capture;
use super::super::interior_life::{collision, fixture, walk};
use super::super::r10_gallery::settle;
use super::super::winona::{at, press, travel};
use super::load;
use crate::bevy_shell::{
    VisibleShellController, mark_runtime_snapshot_dirty, visible_field_dialog_pages,
};

type Point = (i16, i16);

struct Board {
    map: &'static str,
    script: &'static str,
    standing: Point,
    paving: Point,
    reading: Point,
    facing: GameButton,
}

const LEAGUE: Board = Board {
    map: "MnEly",
    script: "MnElyTrailSign",
    standing: (21, 3),
    paving: (20, 3),
    reading: (20, 3),
    facing: Right,
};
const CAMP: Board = Board {
    map: "MnEly",
    script: "MnElyCampSign",
    standing: (15, 21),
    paving: (14, 21),
    reading: (14, 21),
    facing: Right,
};
const DETROIT: Board = Board {
    map: "MnDetroitLakesApproach",
    script: "MN_COHESION_DETROIT_SIGN",
    standing: (8, 12),
    paving: (8, 13),
    reading: (8, 13),
    facing: Up,
};
const PRESERVE: Board = Board {
    map: "MnPreservePineEntry",
    script: "MN_COHESION_PRESERVE_NOTICE",
    standing: (12, 10),
    paving: (13, 11),
    reading: (12, 11),
    facing: Up,
};
const ARRIVAL: Board = Board {
    map: "MnMinneapolis",
    script: "MN_MP_ARRIVALINFO_SIGN",
    standing: (40, 58),
    paving: (40, 60),
    reading: (40, 59),
    facing: Up,
};

const CEDAR: Board = Board {
    map: "MnR08C00R01",
    script: "MN_R08_SIGN_02",
    standing: (22, 6),
    paving: (23, 4),
    reading: (22, 5),
    facing: Down,
};
const GAME_CORNER: Board = Board {
    map: "MnMinneapolis",
    script: "MnCornerSign",
    standing: (55, 51),
    paving: (55, 52),
    reading: (55, 52),
    facing: Up,
};

// A render test app advances its own game clock. Render a disposable native
// checkpoint so taking a screenshot cannot send extra ticks into this journey.
fn capture_board(c: &mut VisibleShellController, directory: Option<&Path>, name: &str) {
    let Some(directory) = directory else {
        return;
    };
    let before = c.snapshot().expect("state reached with the controller");
    let path = super::super::authored_route_fixture::reached_save_path(&format!(
        "direction-board-{}-{name}.crystalsave",
        std::process::id()
    ));
    c.save(&path).expect("sign checkpoint");
    let mut copy = VisibleShellController::load_save(
        c.shell.asset_root.clone(),
        c.shell.shell.runtime().clone(),
        path.clone(),
        None,
    )
    .expect("native render checkpoint");
    super::super::authored_route_fixture::cleanup_reached_save(&path);
    let after = copy.snapshot().expect("restored frame");
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.overworld.facing, after.overworld.facing);
    assert_eq!(before.ui.text, after.ui.text);
    capture(copy, Some(directory), name);
}

fn read_board(
    root: &AssetRoot,
    runtime: &CrystalRuntime,
    board: &Board,
    label: &str,
    words: &str,
    flags: &[&str],
    frames: Option<&Path>,
) -> VisibleShellController {
    let room = &runtime.data().maps[board.map];
    assert_eq!(collision(runtime, room, board.standing), Some("WALL"));
    let event = room
        .events
        .bg_events
        .iter()
        .find(|event| event.script == board.script)
        .expect("installed direction board");
    assert_eq!(
        (
            i16::try_from(event.x).expect("regional x"),
            i16::try_from(event.y).expect("regional y")
        ),
        board.standing
    );
    assert_eq!(event.event_type, "BGEVENT_READ");
    assert!(!room.events.bg_events.iter().any(|e| (
        i16::try_from(e.x).expect("regional x"),
        i16::try_from(e.y).expect("regional y")
    ) == board.paving));
    let mut c = fixture(
        root,
        runtime,
        board.map,
        (board.paving.0, board.paving.1 + 1),
    );
    for flag in flags {
        c.shell
            .shell
            .session_mut()
            .state_mut()
            .flags
            .set_event_flag(flag, true)
            .expect("declared League stage");
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    travel(&mut c, Up, board.map, board.paving);
    if board.reading != board.paving {
        walk(&mut c, runtime, board.reading);
    }
    press(&mut c, board.facing);
    at(&mut c, board.map, board.reading);
    let read = c
        .shell
        .shell
        .current_overworld_interaction_checked()
        .expect("native facing interaction")
        .expect("visible sign has a real interaction");
    assert_eq!(
        read.target_tile,
        TilePosition::new(board.standing.0, board.standing.1)
    );
    assert_eq!(
        read.target,
        OverworldInteractionTarget::Background {
            event_type: "BGEVENT_READ".into()
        }
    );
    assert_eq!(read.script, board.script);
    let origin = c.snapshot().expect("sign approach").trainer.money;
    capture_board(&mut c, frames, &format!("{label}-board.png"));
    press(&mut c, A);
    let snap = c.snapshot().expect("board after A");
    assert_eq!(
        snap.ui.text.as_ref().expect("visible native page").label,
        label
    );
    let raw = c
        .shell
        .shell
        .presentation_snapshot()
        .expect("full sign program");
    let pages = visible_field_dialog_pages(&raw, &c.shell).expect("signed directions");
    assert!(
        pages.join(" ").replace('\n', " ").contains(words),
        "{board_name} printed {pages:?}",
        board_name = board.script
    );
    capture_board(&mut c, frames, &format!("{label}-reading.png"));
    settle(&mut c, "direction board must release walking");
    let closed = c.snapshot().expect("no charge for directions");
    assert_eq!(closed.trainer.money, origin);
    assert!(closed.safari.is_none());
    c
}

#[test]
#[ignore = "requires the physical direction boards edition via MN_BADGE_TRAILS_PACK"]
fn regional_direction_boards_native_read_walk_and_follow_the_sign() {
    let (root, runtime) = load();
    let frames = std::env::var_os("MN_DIRECTION_BOARD_FRAMES").map(PathBuf::from);
    let dir = frames.as_deref();
    for (flags, label, words) in [
        (&[][..], "MnElyTrailSignText", "Eight BADGES to enter."),
        (
            &["EVENT_MN_GYM_8_DEFEATED"][..],
            "MnWayLeagueHoldText",
            "Waiting on the intake crew.",
        ),
        (
            &["EVENT_MN_GYM_8_DEFEATED", "EVENT_MN_SURVEY_RESTORED"][..],
            "MnWayLeagueReadyText",
            "FERN takes the first match.",
        ),
        (
            &[
                "EVENT_MN_GYM_8_DEFEATED",
                "EVENT_MN_SURVEY_RESTORED",
                "EVENT_MN_LEAGUE_CHAMPION",
            ][..],
            "MnWayLeagueAgainText",
            "Returning to the lobby ends a run.",
        ),
    ] {
        let mut c = read_board(&root, &runtime, &LEAGUE, label, words, flags, dir);
        if label == "MnWayLeagueReadyText" {
            walk(&mut c, &runtime, (25, 14));
            travel(&mut c, Up, "MnLeagueLobby", (1, 4));
        } else {
            travel(&mut c, Down, LEAGUE.map, (20, 4));
        }
    }
    let mut camp = read_board(
        &root,
        &runtime,
        &CAMP,
        "MnElyCampSignText",
        "Clinic and shop open to travelers.",
        &[],
        dir,
    );
    walk(&mut camp, &runtime, (11, 22));
    travel(&mut camp, Up, "MnElyPokecenter1F", (3, 5));
    let mut detroit = read_board(
        &root,
        &runtime,
        &DETROIT,
        "MN_COHESION_DETROIT_SIGNText",
        "WOODLAND APPROACH Follow the pine trail.",
        &[],
        dir,
    );
    travel(&mut detroit, Right, DETROIT.map, (9, 13));
    let mut preserve = read_board(
        &root,
        &runtime,
        &PRESERVE,
        "MN_COHESION_PRESERVE_NOTICEText",
        "The trail to the shelter is free.",
        &[],
        dir,
    );
    travel(&mut preserve, Down, PRESERVE.map, (12, 12));
    let mut city = read_board(
        &root,
        &runtime,
        &ARRIVAL,
        "MN_MP_ARRIVALINFO_SIGNText",
        "SOUTH ARRIVAL DESK",
        &[],
        dir,
    );
    travel(&mut city, Down, ARRIVAL.map, ARRIVAL.paving);
    travel(&mut city, Right, ARRIVAL.map, (43, 60));
    travel(&mut city, Up, "MnMinneapolisArrivalInfo", (2, 6));
    travel(&mut city, Down, ARRIVAL.map, (43, 60));
    travel(&mut city, Left, ARRIVAL.map, ARRIVAL.paving);

    let mut cedar = read_board(
        &root,
        &runtime,
        &CEDAR,
        "MN_R08_SIGN_02Text",
        "CEDAR INLET BRIDGE OR SHORE",
        &[],
        dir,
    );
    travel(&mut cedar, Right, CEDAR.map, (24, 5));
    travel(&mut cedar, Down, CEDAR.map, (24, 7));
    let mut corner = read_board(
        &root,
        &runtime,
        &GAME_CORNER,
        "MnCornerSignText",
        "MINNEAPOLIS GAME CORNER Slots! Card Flip! Coin prizes!",
        &[],
        dir,
    );
    travel(&mut corner, Right, GAME_CORNER.map, (56, 52));
    travel(&mut corner, Up, "MnMinneapolisCornerHome", (3, 12));
    travel(&mut corner, Down, GAME_CORNER.map, (56, 52));
    travel(&mut corner, Left, GAME_CORNER.map, (55, 52));
    for already_defeated in [false, true] {
        graphical_game_corner(&root, &runtime, already_defeated);
    }
}

fn graphical_game_corner(root: &AssetRoot, runtime: &CrystalRuntime, already_defeated: bool) {
    use bevy::input::keyboard::{Key, KeyboardInput};
    use bevy::input::{ButtonState, InputPlugin};
    use bevy::prelude::{App, Entity, KeyCode};

    use super::super::menu_render_test_app;
    use super::super::shore_watch_bank_walk::keyboard_event;
    use crate::bevy_shell::BevyRuntimeShell;
    use crystal_core::models::Dv;
    use crystal_core::world::map::Direction;
    use crystal_runtime::RuntimeBattleKind;

    fn position(app: &App) -> (String, TilePosition) {
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(shell.last_error, None);
        let state = shell.shell.session().snapshot();
        (state.map_name, state.tile)
    }
    fn confirm(app: &mut App) {
        for state in [ButtonState::Pressed, ButtonState::Released] {
            app.world_mut()
                .write_message(KeyboardInput {
                    key_code: KeyCode::KeyZ,
                    logical_key: Key::Character("z".into()),
                    state,
                    text: None,
                    repeat: false,
                    window: Entity::PLACEHOLDER,
                })
                .expect("native confirm");
            app.update();
        }
    }
    fn walk(app: &mut App, direction: Direction, map: &str, x: i16, y: i16) {
        let expected = (map.to_owned(), TilePosition::new(x, y));
        keyboard_event(app, direction, ButtonState::Pressed);
        for _ in 0..120 {
            app.update();
            if position(app) == expected {
                keyboard_event(app, direction, ButtonState::Released);
                for _ in 0..12 {
                    app.update();
                }
                assert_eq!(
                    position(app),
                    expected,
                    "released direction stays on arrival tile"
                );
                return;
            }
        }
        let actual = position(app);
        let shell = app.world().resource::<BevyRuntimeShell>();
        let snap = shell
            .shell
            .presentation_snapshot()
            .expect("diagnose graphical doorway");
        panic!(
            "arcade traversal {actual:?} expected {expected:?}; facing {:?}, nearby actors {:?}, text {:?}, phase {:?}, step boundary {:?}, pending warp {}, field travel arrival {}, pending script {:?}, walk ticks {}, active warp {:?}, walking actors {:?}, natural {}, battle {:?}, special {:?}, step from {:?}, phone {}, fade {}, menu {:?}, notice {:?}",
            snap.overworld.facing,
            snap.visible_object_runtime_tiles
                .iter()
                .filter(|(_, t)| t.x.abs_diff(56) + t.y.abs_diff(51) <= 3)
                .collect::<Vec<_>>(),
            snap.ui.text.map(|t| t.label),
            snap.phase,
            shell.pending_overworld_step_boundary,
            shell.pending_overworld_warp_scene.is_some(),
            shell.pending_field_travel_arrival,
            snap.ui.pending_text_wait,
            shell.player_walk_frame_ticks,
            shell.visible_walk_warp_phase,
            shell.object_walk_from.keys().collect::<Vec<_>>(),
            shell.natural_movement.enabled,
            snap.battle.map(|b| b.kind),
            shell.special_boundary,
            shell.player_walk_from,
            shell.pokegear_phone_call.is_some(),
            shell.screen_fade.is_some(),
            snap.ui.menu.map(|m| m.menu_id),
            shell.field_notice
        );
    }
    const RIDER: &str = "MN_EXTRA_MPLS_CYCLE_17";
    const WON: &str = "EVENT_MN_EXTRA_MPLS_CYCLE_17";
    let mut c = fixture(root, runtime, GAME_CORNER.map, (55, 53));
    let state = c.shell.shell.session_mut().state_mut();
    let partner = runtime
        .data()
        .create_pokemon("KINDLIT", 22, Dv::default())
        .expect("legal local visitor's partner");
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    if already_defeated {
        state
            .flags
            .set_event_flag(WON, true)
            .expect("return visitor has already battled SAM");
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    let mut app = menu_render_test_app(c.shell);
    app.add_plugins(InputPlugin);
    app.update();
    app.update();
    walk(&mut app, Direction::Up, GAME_CORNER.map, 55, 52);
    let target = app
        .world()
        .resource::<BevyRuntimeShell>()
        .shell
        .current_overworld_interaction_checked()
        .expect("native facing")
        .expect("arcade plaque");
    assert_eq!(target.script, GAME_CORNER.script);
    confirm(&mut app);
    let text = app
        .world()
        .resource::<BevyRuntimeShell>()
        .shell
        .presentation_snapshot()
        .expect("native reading")
        .ui
        .text
        .expect("printed building name");
    assert_eq!(text.label, "MnCornerSignText");
    for _ in 0..10 {
        for _ in 0..32 {
            app.update();
        }
        let ui = app
            .world()
            .resource::<BevyRuntimeShell>()
            .shell
            .presentation_snapshot()
            .expect("arcade note")
            .ui;
        if !ui.text_window_open && ui.pending_text_wait.is_none() {
            break;
        }
        confirm(&mut app);
    }
    walk(&mut app, Direction::Right, GAME_CORNER.map, 56, 52);
    if already_defeated {
        walk(&mut app, Direction::Up, "MnMinneapolisCornerHome", 3, 12);
        walk(&mut app, Direction::Down, GAME_CORNER.map, 56, 52);
        return;
    }

    // The cyclist is two spaces to the west when he sees the player. No
    // confirm press is sent between leaving the sign and his challenge.
    let mut saw_sam = false;
    for _ in 0..120 {
        let shell = app.world().resource::<BevyRuntimeShell>();
        if let Some(sight) = &shell.pending_trainer_sight {
            assert_eq!(sight.object_id, RIDER);
            saw_sam = true;
        }
        if shell
            .field_notice
            .as_deref()
            .is_some_and(|text| text.contains("at least I found"))
        {
            break;
        }
        app.update();
    }
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert!(saw_sam, "SAM spotted the first-time visitor");
    let notice = shell
        .field_notice
        .as_ref()
        .expect("SAM's on-sight challenge");
    assert!(
        notice.contains("Just rode all the") && notice.contains("at least I found"),
        "{notice}"
    );
    let sight = shell
        .shell
        .presentation_snapshot()
        .expect("cyclist approached");
    assert_eq!(
        sight.visible_object_runtime_tiles[RIDER],
        TilePosition::new(55, 52)
    );
    assert_eq!(sight.overworld.facing, Direction::Left);
    assert!(
        !shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set(WON)
            .expect("unbeaten cyclist")
    );
    for _ in 0..24 {
        for _ in 0..16 {
            app.update();
        }
        let shell = app.world().resource::<BevyRuntimeShell>();
        if let Some(battle) = shell
            .shell
            .presentation_snapshot()
            .expect("actual native battle")
            .battle
        {
            assert!(
                matches!(battle.kind,RuntimeBattleKind::Trainer { ref trainer_id, .. } if trainer_id==RIDER)
            );
            return;
        }
        confirm(&mut app);
    }
    panic!("accepting SAM's challenge never entered his battle");
}
