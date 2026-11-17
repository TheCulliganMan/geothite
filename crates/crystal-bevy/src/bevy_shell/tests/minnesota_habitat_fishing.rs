//! Conditional time-table fish: native bank, casting, capture and Continue.
use crystal_core::input::GameButton;

use super::super::r10_gallery::continue_save;
use super::super::rochester::flee;
use super::super::rochester_earned::try_recruit_with_ball;
use super::super::winona::press;
use super::{checkpoint, fixture, fixture_with_partner, verify_capture};
use crate::bevy_shell::mark_runtime_snapshot_dirty;

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the Minnesota ocean time table"]
fn habitats_keypout_time_table_native_bank_capture_and_continue() {
    replay_good_rod("MnWinona", [25, 8], "FISHGROUP_MN_OCEAN", "KEYPOUT");
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the regional ecology revision"]
fn region_ecology_fishes_catches_and_continues_in_the_river_valley() {
    replay_good_rod(
        "MnR03C00R00",
        [11, 6],
        "FISHGROUP_MN_ECO_MNR03C00R00",
        "CLINKLAW",
    );
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the campaign polish opening tables"]
fn campaign_polish_opening_old_rod_catches_and_continues() {
    let bank = [25, 8];
    let (root, runtime, mut controller) =
        fixture_with_partner("MnWinona", bank, false, false, 12, "KINDLIT", 5);
    let state = controller.shell.shell.session_mut().state_mut();
    assert!(
        state
            .bag
            .add_item(&runtime.data().items["OLD_ROD"], 1)
            .expect("declared prior Old Rod")
    );
    state.registered_key_item = Some("OLD_ROD".into());
    mark_runtime_snapshot_dirty(&mut controller.shell);
    let level = catch_from_bank(&mut controller, &runtime, bank, "SCUTLET");
    // The inherited Wild Challenge can raise the catch to level 6.
    assert!(
        (4..=6).contains(&level),
        "opening Scutlet was level {level}"
    );
    let controller = continue_save(root, runtime.clone(), &mut controller);
    verify_capture(&controller, &runtime, "SCUTLET");
}

fn replay_good_rod(map: &str, bank: [i16; 2], group: &str, target: &str) {
    // Prior Good Rod and a local partner are fixture setup.
    let (root, runtime, mut controller) = fixture(map, bank, false, true);
    assert_eq!(
        runtime.data().maps[map].attributes.fishing_group.as_deref(),
        Some(group)
    );
    catch_from_bank(&mut controller, &runtime, bank, target);
    controller = continue_save(root, runtime.clone(), &mut controller);
    verify_capture(&controller, &runtime, target);
    checkpoint(&mut controller, &format!("{target}-caught"));
}

fn catch_from_bank(
    controller: &mut super::VisibleShellController,
    runtime: &crystal_runtime::CrystalRuntime,
    bank: [i16; 2],
    target: &str,
) -> u8 {
    press(controller, GameButton::Right);
    checkpoint(controller, &format!("{target}-bank-start"));
    for cast in 0..256 {
        controller
            .press(GameButton::Select)
            .expect("native registered rod cast");
        assert!(controller.shell.visible_fishing_animation.is_some());
        for _ in 0..256 {
            if controller
                .shell
                .visible_fishing_animation
                .as_ref()
                .is_some_and(|animation| {
                    animation.phase == super::super::super::VisibleFishingPhase::AwaitText
                })
            {
                break;
            }
            controller.wait_frames(1).expect("native casting clock");
        }
        for _ in 0..128 {
            let snapshot = controller.snapshot().expect("native fishing notice");
            assert_eq!(
                (snapshot.overworld.tile.x, snapshot.overworld.tile.y),
                (bank[0], bank[1])
            );
            if snapshot.battle.is_some()
                || (controller.shell.visible_fishing_animation.is_none()
                    && controller.shell.field_notice.is_none()
                    && !controller.has_pending_script_work())
            {
                break;
            }
            press(controller, GameButton::A);
        }
        if let Some(battle) = controller.snapshot().expect("actual fishing result").battle {
            if battle.enemy_pokemon.species.id == target {
                assert!(try_recruit_with_ball(controller, runtime, "MASTER_BALL"));
                verify_capture(controller, runtime, target);
                return battle.enemy_pokemon.level;
            }
            flee(controller);
        }
        controller
            .wait_frames(17 + cast % 11)
            .expect("ordinary shore idle");
    }
    panic!("native rod did not encounter {target} at {bank:?}");
}
