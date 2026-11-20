//! Local trainer-win fixtures isolate the actual keyboard-facing gym controls and their saved state.
#[path = "saven_court.rs"]
mod saven_court;

use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty, selected_vertical_menu};
use super::badge_trails::load;
use super::interior_life::fixture;
use super::r10_gallery::{continue_save, press, settle};
use super::winona::{at, travel};

const MILL: &str = "MnMinneapolisGym";
const FANS: &str = "MnStCloudGym";

fn set_earned(c: &mut VisibleShellController, flags: &[&str]) {
    for flag in flags {
        c.shell
            .shell
            .session_mut()
            .state_mut()
            .flags
            .set_event_flag(flag, true)
            .expect("trainer victory fixture");
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
}

fn event(c: &VisibleShellController, flag: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(flag)
        .expect("installed gym flag")
}

fn block(c: &VisibleShellController, runtime: &CrystalRuntime, map: &str, x: u16, y: u16) -> u16 {
    let room = &runtime.data().maps[map];
    c.shell
        .shell
        .session()
        .state()
        .map_block_overrides
        .get(map)
        .and_then(|blocks| blocks.get(&(x, y)))
        .copied()
        .unwrap_or(
            room.blocks[usize::from(y) * usize::from(room.attributes.width) + usize::from(x)],
        )
}

fn face_and_read(c: &mut VisibleShellController, facing: GameButton) {
    let before = c.snapshot().expect("switch approach").overworld;
    press(c, facing);
    assert_eq!(
        c.snapshot()
            .expect("standing before the switch")
            .overworld
            .tile,
        before.tile
    );
    press(c, GameButton::A);
}

fn expect_page(c: &mut VisibleShellController, label: &str, words: &str) {
    let snapshot = c.snapshot().expect("switch response");
    let page = snapshot.ui.text.expect("an actual switch reply must open");
    assert_eq!(page.label, label);
    assert_eq!(page.asm_text.as_deref(), Some(words));
    settle(c, label);
}

#[test]
#[ignore = "requires an installed Gym Walk archive in MN_BADGE_TRAILS_PACK"]
fn gym_walk_breaker_fronts_report_the_guard_and_save_both_bridges() {
    let (root, runtime) = load();
    for (bank, trainer, tile, facing, guard, denial, bridge_x, closed) in [
        (
            "WEST",
            "LENVI",
            (4, 16),
            GameButton::Up,
            "EVENT_MN_GYM_3_TRAINER_1_DEFEATED",
            "MnGymWalkWestFeederLockedText",
            2,
            1,
        ),
        (
            "EAST",
            "JOSSA",
            (10, 7),
            GameButton::Left,
            "EVENT_MN_GYM_3_TRAINER_2_DEFEATED",
            "MnGymWalkEastFeederLockedText",
            4,
            2,
        ),
    ] {
        let mut c = fixture(&root, &runtime, MILL, tile);
        if bank == "EAST" {
            // The first bridge must already be powered to stand in this gallery.
            set_earned(
                &mut c,
                &[
                    "EVENT_MN_GYM_3_TRAINER_1_DEFEATED",
                    "EVENT_MN_GYM_3_V2_WEST_FEED_ON",
                ],
            );
            let (state, overworld) = c.shell.shell.session_mut().state_and_overworld_mut();
            runtime
                .data()
                .apply_map_setup_callbacks(state, overworld, MILL, "MAPSETUP_DOOR")
                .expect("west bridge");
        }
        c.wait_frames(2).expect("complete map entry");
        let power = format!("EVENT_MN_GYM_3_V2_{bank}_FEED_ON");
        let console = format!("{MILL}{bank}_FEEDERRouting");
        face_and_read(&mut c, facing);
        expect_page(
            &mut c,
            denial,
            &format!(
                "Beat {trainer} to use\nthe {} breaker.",
                bank.to_ascii_lowercase()
            ),
        );
        assert!(!event(&c, &power));
        assert_eq!(block(&c, &runtime, MILL, bridge_x, 5), closed);
        set_earned(&mut c, &[guard]);
        face_and_read(&mut c, facing);
        expect_page(
            &mut c,
            &format!("{console}ONSuccessText"),
            &format!("{bank} feeder on.\nIts bridge opens."),
        );
        assert!(event(&c, &power));
        assert_eq!(block(&c, &runtime, MILL, bridge_x, 5), 12);
        assert_eq!(event(&c, "EVENT_MN_GYM_3_COURT_OPEN"), bank == "EAST");
        let mut c = continue_save(root.clone(), runtime.clone(), &mut c);
        at(&mut c, MILL, tile);
        assert!(event(&c, &power));
        assert_eq!(block(&c, &runtime, MILL, bridge_x, 5), 12);
        face_and_read(&mut c, facing);
        expect_page(
            &mut c,
            &format!("{console}OFFSuccessText"),
            &format!("{bank} feeder off.\nIts bridge closes."),
        );
        assert_eq!(block(&c, &runtime, MILL, bridge_x, 5), closed);
        assert!(!event(&c, "EVENT_MN_GYM_3_COURT_OPEN"));
    }
}

fn choose_fan(c: &mut VisibleShellController, west: bool) {
    face_and_read(c, GameButton::Up);
    for _ in 0..12 {
        let s = c.snapshot().expect("airworks screen");
        if let Some(menu) = s.ui.menu.filter(|m| m.window_open) {
            let choices =
                selected_vertical_menu(&menu, &c.shell.menu_cursor).expect("relay directions");
            let options: Vec<_> = choices
                .options
                .iter()
                .map(|s| s.trim_start_matches(['>', ' ']))
                .collect();
            assert_eq!(options, ["EAST", "WEST"]);
            if west {
                press(c, GameButton::Down);
            }
            assert_eq!(
                c.shell
                    .menu_cursor
                    .as_ref()
                    .expect("fan cursor")
                    .option_index,
                usize::from(west)
            );
            press(c, GameButton::A);
            return;
        }
        assert!(s.ui.text_window_open, "the dial must show its relay prompt");
        press(c, GameButton::A);
    }
    panic!("the dial did not offer a direction");
}

#[test]
#[ignore = "requires an installed Gym Walk archive in MN_BADGE_TRAILS_PACK"]
fn gym_walk_airworks_guide_leaves_the_south_dial_free_through_continue() {
    let (root, runtime) = load();
    let mut c = fixture(&root, &runtime, FANS, (7, 15));
    c.wait_frames(2).expect("complete map entry");
    travel(&mut c, GameButton::Up, FANS, (7, 14));
    let console = "MnStCloudGymLOWER_WEST_BANKRouting";
    choose_fan(&mut c, false);
    expect_page(
        &mut c,
        &format!("{console}EASTDeniedText"),
        "Beat WYREL first.",
    );
    assert!(!event(&c, "EVENT_MN_GYM_5_V2_RELAY_EAST"));
    set_earned(&mut c, &["EVENT_MN_GYM_5_TRAINER_1_DEFEATED"]);
    choose_fan(&mut c, false);
    expect_page(
        &mut c,
        &format!("{console}EASTSuccessText"),
        "Both fans: EAST.\nFollow the arrows.",
    );
    for (x, y) in [(4, 5), (5, 5), (4, 1), (5, 1)] {
        assert_eq!(
            block(&c, &runtime, FANS, x, y),
            3,
            "paired fan at ({x},{y})"
        );
    }
    let mut c = continue_save(root, runtime.clone(), &mut c);
    at(&mut c, FANS, (7, 14));
    assert!(event(&c, "EVENT_MN_GYM_5_V2_RELAY_EAST"));
    choose_fan(&mut c, true);
    expect_page(
        &mut c,
        &format!("{console}WESTSuccessText"),
        "Both fans: WEST.\nFollow the arrows.",
    );
    assert!(!event(&c, "EVENT_MN_GYM_5_V2_RELAY_EAST"));
    assert!(event(&c, "EVENT_MN_GYM_5_COURT_OPEN"));
    assert!(!event(&c, "EVENT_MN_GYM_5_TRAINER_2_DEFEATED"));
    for (x, y) in [(4, 5), (5, 5), (4, 1), (5, 1)] {
        assert_eq!(
            block(&c, &runtime, FANS, x, y),
            2,
            "paired fan at ({x},{y})"
        );
    }
}
