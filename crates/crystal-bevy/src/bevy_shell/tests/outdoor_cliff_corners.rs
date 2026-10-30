//! Local corner fixtures; all movement and Continue use the production owner.

use std::path::PathBuf;

use crystal_core::input::GameButton;
use serde_json::Value;

use super::{Setup, assert_at, fixture, load, move_to, press};

#[test]
#[ignore = "requires the external cliff-corners pack, authoring plan and retained save directory"]
fn outdoor_restored_corners_keep_inside_rims_and_deny_crossing_the_face() {
    let (root, runtime) = load();
    let plan = PathBuf::from(
        std::env::var_os("CRYSTAL_OUTDOOR_CORNER_PLAN").expect("reviewed corner plan"),
    );
    let plan: Value =
        serde_json::from_slice(&std::fs::read(plan).expect("corner plan")).expect("corner JSON");
    let output = PathBuf::from(
        std::env::var_os("CRYSTAL_OUTDOOR_CORNER_SAVES").expect("retained corner saves"),
    );
    std::fs::create_dir_all(&output).expect("corner save directory");
    let repairs = plan["corner_boulders"]
        .as_array()
        .expect("explicit corners");
    assert_eq!(repairs.len(), 7);
    for (index, repair) in repairs.iter().enumerate() {
        let map = repair["map"].as_str().expect("corner map");
        let x = i16::try_from(repair["metatile"][0].as_i64().expect("corner x")).expect("x range");
        let y = i16::try_from(repair["metatile"][1].as_i64().expect("corner y")).expect("y range");
        let west = repair["corner"] == "6c";
        let rim = (x * 2 + i16::from(west), y * 2);
        let inward = if west {
            GameButton::Right
        } else {
            GameButton::Left
        };
        let outward = if west {
            GameButton::Left
        } else {
            GameButton::Right
        };
        let next = (rim.0 + if west { 1 } else { -1 }, rim.1);
        let mut controller = fixture(&root, &runtime, Setup::at(map, rim));
        let before = controller.snapshot().expect("inside corner fixture");
        for direction in [outward, GameButton::Down] {
            for _ in 0..3 {
                press(&mut controller, direction);
            }
            assert_at(&mut controller, map, rim);
            assert!(matches!(
                controller
                    .shell
                    .shell
                    .snapshot()
                    .expect("runtime phase snapshot")
                    .phase,
                crystal_runtime::RuntimeShellPhase::Overworld
            ));
            assert!(
                !controller
                    .snapshot()
                    .expect("wall denial")
                    .ui
                    .text_window_open
            );
        }
        move_to(&mut controller, inward, next);
        move_to(&mut controller, outward, rim);
        let path = output.join(format!("corner-{index}.crystalsave"));
        assert!(!path.exists(), "retain prior corner saves");
        controller.save(&path).expect("save reached native corner");
        let mut continued =
            super::VisibleShellController::load_save(root.clone(), runtime.clone(), path, None)
                .expect("Continue reached corner");
        assert_at(&mut continued, map, rim);
        let after = continued.snapshot().expect("continued corner");
        assert_eq!(before.party, after.party);
        assert_eq!(before.bag, after.bag);
    }
}
