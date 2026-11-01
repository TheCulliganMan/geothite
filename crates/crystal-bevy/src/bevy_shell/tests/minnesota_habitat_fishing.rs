//! Conditional time-table fish: native bank, casting, capture and Continue.
use crystal_core::input::GameButton;

use super::super::r10_gallery::continue_save;
use super::super::rochester::flee;
use super::super::rochester_earned::try_recruit_with_ball;
use super::super::winona::press;
use super::{checkpoint, fixture, verify_capture};

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the Minnesota ocean time table"]
fn habitats_keypout_time_table_native_bank_capture_and_continue() {
    // Explicit prior Good Rod, one capture ball, local partner and noon time.
    // No state, RNG, inventory or position writes occur after fixture setup.
    let (root, runtime, mut controller) = fixture("MnWinona", [25, 8], false, true);
    assert_eq!(
        runtime.data().maps["MnWinona"]
            .attributes
            .fishing_group
            .as_deref(),
        Some("FISHGROUP_MN_OCEAN")
    );
    press(&mut controller, GameButton::Right);
    checkpoint(&mut controller, "keypout-bank-start");
    for cast in 0..256 {
        controller
            .press(GameButton::Select)
            .expect("native Good Rod cast");
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
                (25, 8)
            );
            if snapshot.battle.is_some()
                || (controller.shell.visible_fishing_animation.is_none()
                    && controller.shell.field_notice.is_none()
                    && !controller.has_pending_script_work())
            {
                break;
            }
            press(&mut controller, GameButton::A);
        }
        if let Some(battle) = controller.snapshot().expect("actual fishing result").battle {
            if battle.enemy_pokemon.species.id == "KEYPOUT" {
                assert!(try_recruit_with_ball(
                    &mut controller,
                    &runtime,
                    "MASTER_BALL"
                ));
                verify_capture(&controller, &runtime, "KEYPOUT");
                controller = continue_save(root, runtime.clone(), &mut controller);
                verify_capture(&controller, &runtime, "KEYPOUT");
                checkpoint(&mut controller, "keypout-caught");
                return;
            }
            flee(&mut controller);
        }
        controller
            .wait_frames(17 + cast % 11)
            .expect("ordinary shore idle");
    }
    panic!("native Good Rod did not encounter the installed time-table Keypout");
}
