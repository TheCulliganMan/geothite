//! Declared chapter setups supplement the separate earned New Game course.
use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::CrystalRuntime;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController, arm_visible_current_scene_script,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::winona::{at, continue_in_place, finish, load, press};

fn fixture(
    map: &str,
    tile: (i16, i16),
    flags: &[&str],
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, runtime) = load();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: map.into(),
            tile_x: tile.0,
            tile_y: tile.1,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit scene fixture");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture name");
    let state = controller.shell.shell.session_mut().state_mut();
    for flag in flags {
        state
            .flags
            .set_event_flag(flag, true)
            .expect("declared chapter access");
    }
    let partner = runtime
        .data()
        .create_pokemon("KINDLIT", 35, Dv::default())
        .expect("native fixture partner");
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    controller.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    (root, runtime, controller)
}

#[test]
#[ignore = "explicit entrance fixtures; requires revised CRYSTAL_WINONA_TEST_PACK"]
fn overhaul_lab_all_authored_carpet_arrivals_reach_the_same_introduction() {
    for tile in [(4, 10), (4, 11), (5, 10), (5, 11)] {
        let (root, runtime, mut controller) =
            fixture("MnWinonaFieldStation", tile, &["EVENT_MN_HOME_INTRO"]);
        arm_visible_current_scene_script(&mut controller.shell, "declared_lab_entrance")
            .expect("native arrival dispatch");
        controller.wait_frames(512).expect("aisle movement");
        at(&mut controller, "MnWinonaFieldStation", (4, 2));
        let snapshot = controller.snapshot().expect("introduction");
        assert_eq!(
            snapshot.ui.text.expect("professor text").label,
            "MnOverhaulTamarackIntroText"
        );
        let doctor = snapshot.visible_object_runtime_tiles["MN_W_TAMARACK"];
        assert_eq!((doctor.x, doctor.y), (5, 2));
        press(&mut controller, GameButton::Left);
        at(&mut controller, "MnWinonaFieldStation", (4, 2));
        finish(&mut controller);
        continue_in_place(&mut controller, &root, &runtime, "lab-entrance-complete");
        arm_visible_current_scene_script(&mut controller.shell, "declared_lab_reentry")
            .expect("repeat arrival");
        controller.wait_frames(64).expect("settle reentry");
        assert!(
            !controller
                .snapshot()
                .expect("once-only entry")
                .ui
                .text_window_open
        );
    }
}

#[test]
#[ignore = "explicit depot entrance fixtures; requires revised CRYSTAL_WINONA_TEST_PACK"]
fn overhaul_depot_all_authored_arrivals_stage_the_courier_before_battle() {
    for tile in [
        (14, 23),
        (15, 23),
        (16, 23),
        (17, 23),
        (14, 22),
        (15, 22),
        (16, 22),
        (17, 22),
        (15, 21),
        (17, 10),
    ] {
        let (_, _, mut controller) =
            fixture("MnStPaulUnionDepot", tile, &["EVENT_MN_DISPATCH_NOTE"]);
        let state = controller.shell.shell.session_mut().state_mut();
        state.badges.johto[2] = true;
        state
            .flags
            .set_engine_flag("ENGINE_PLAINBADGE", true)
            .expect("declared Current badge");
        mark_runtime_snapshot_dirty(&mut controller.shell);
        // Re-arm the production map-scene scheduler after the fixture's flags
        // are installed. All approach steps and waits are native script work.
        arm_visible_current_scene_script(&mut controller.shell, "declared_depot_entrance")
            .expect("native arrival dispatch");
        controller.wait_frames(1024).expect("courier approach");
        at(&mut controller, "MnStPaulUnionDepot", (19, 16));
        let snapshot = controller.snapshot().expect("courier introduction");
        assert!(snapshot.ui.text_window_open && snapshot.battle.is_none());
        for (actor, expected) in [
            ("MN_ST_PAUL_DEPOT_COURIER", (20, 16)),
            ("MN_NSTP_MINA", (18, 16)),
        ] {
            let tile = snapshot
                .visible_object_runtime_tiles
                .get(actor)
                .expect("visible participant");
            assert_eq!((tile.x, tile.y), expected);
        }
        assert!(
            !controller
                .shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set("EVENT_MN_COURIER_DEFEATED")
                .expect("victory flag")
        );
        press(&mut controller, GameButton::Down);
        at(&mut controller, "MnStPaulUnionDepot", (19, 16));
    }
}

#[test]
#[ignore = "declared full-pocket fixture; requires revised CRYSTAL_WINONA_TEST_PACK"]
fn overhaul_aide_refusal_native_toss_retry_and_repeat_preserve_reward() {
    use super::super::{FieldPackAction, FieldPackPocket, StartMenuOption};
    use crystal_core::models::bag::{BALL_POCKET_CAPACITY, MAX_ITEM_STACK};

    let (root, runtime, mut controller) = fixture(
        "MnWinonaFieldStation",
        (4, 8),
        &[
            "EVENT_MN_HOME_INTRO",
            "EVENT_MN_TAMARACK_INTRO",
            "EVENT_MN_STARTER_RECEIVED",
            "EVENT_MN_R01_PARCEL_RECEIVED",
            "EVENT_MN_REED_FIRST_MET",
            "EVENT_MN_REED_WINONA_DEFEATED",
        ],
    );
    let ball = &runtime.data().items["POKE_BALL"];
    for _ in 0..BALL_POCKET_CAPACITY {
        assert!(
            controller
                .shell
                .shell
                .session_mut()
                .state_mut()
                .bag
                .add_item(ball, MAX_ITEM_STACK)
                .expect("declared full ball pocket")
        );
    }
    mark_runtime_snapshot_dirty(&mut controller.shell);
    // All state changes below this fixture boundary use production input.
    let full = controller.shell.shell.session().state().bag.clone();
    press(&mut controller, GameButton::Down);
    controller.wait_frames(128).expect("aide approach");
    at(&mut controller, "MnWinonaFieldStation", (4, 9));
    let snapshot = controller.snapshot().expect("capacity refusal");
    let aide = snapshot.visible_object_runtime_tiles["MN_W_AIDE"];
    assert_eq!((aide.x, aide.y), (3, 9));
    assert_eq!(
        snapshot.ui.text.expect("refusal text").label,
        "MnWBallsFullText"
    );
    finish(&mut controller);
    assert_eq!(controller.shell.shell.session().state().bag, full);
    assert!(
        !controller
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_WINONA_BALLS_RECEIVED")
            .expect("unclaimed supplies")
    );
    continue_in_place(&mut controller, &root, &runtime, "aide-full-refusal");
    press(&mut controller, GameButton::Start);
    let snapshot = controller.snapshot().expect("Start");
    let target = super::super::visible_start_menu_options(&controller.shell, &snapshot)
        .iter()
        .position(|option| *option == StartMenuOption::Pack)
        .expect("Pack");
    for _ in 0..16 {
        let selected = controller
            .shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index;
        if selected == target {
            press(&mut controller, GameButton::A);
            break;
        }
        press(
            &mut controller,
            if selected < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    for _ in 0..4 {
        if controller.shell.field_pack_pocket == Some(FieldPackPocket::Balls) {
            break;
        }
        press(&mut controller, GameButton::Right);
    }
    press(&mut controller, GameButton::A);
    let snapshot = controller.snapshot().expect("ball actions");
    let target = super::super::visible_selected_pack_item_actions(
        &snapshot,
        &controller.shell,
        &FieldPackPocket::Balls,
        false,
    )
    .expect("ball actions")
    .iter()
    .position(|action| *action == FieldPackAction::Toss)
    .expect("Toss");
    for _ in 0..8 {
        let selected = controller
            .shell
            .field_pack_action_cursor
            .as_ref()
            .expect("ball action cursor")
            .option_index;
        if selected == target {
            press(&mut controller, GameButton::A);
            break;
        }
        press(
            &mut controller,
            if selected < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    for _ in 0..4 {
        press(&mut controller, GameButton::Up);
    }
    assert_eq!(
        controller
            .shell
            .pack_toss
            .as_ref()
            .expect("quantity")
            .quantity,
        5
    );
    press(&mut controller, GameButton::A);
    controller.wait_frames(240).expect("native toss question");
    press(&mut controller, GameButton::A);
    for _ in 0..96 {
        controller
            .wait_frames(128)
            .expect("finish native Pack text");
        press(&mut controller, GameButton::B);
        let snapshot = controller.snapshot().expect("Pack closure");
        if controller.shell.bag_cursor.is_none()
            && controller.shell.start_menu_cursor.is_none()
            && controller.shell.field_notice.is_none()
            && snapshot.ui.menu.is_none()
            && !snapshot.ui.text_window_open
        {
            break;
        }
    }
    assert_eq!(
        controller.shell.shell.session().state().bag.quantity(ball),
        full.quantity(ball) - 5
    );
    super::authored_route_fixture::indoor_walk(&mut controller, &runtime, (3, 9));
    press(&mut controller, GameButton::Left);
    press(&mut controller, GameButton::A);
    finish(&mut controller);
    assert!(
        controller
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_WINONA_BALLS_RECEIVED")
            .expect("claimed supplies")
    );
    assert_eq!(
        controller.shell.shell.session().state().bag.quantity(ball),
        full.quantity(ball)
    );
    continue_in_place(&mut controller, &root, &runtime, "aide-supplies-retried");
    press(&mut controller, GameButton::A);
    finish(&mut controller);
    assert_eq!(
        controller.shell.shell.session().state().bag.quantity(ball),
        full.quantity(ball)
    );
}
