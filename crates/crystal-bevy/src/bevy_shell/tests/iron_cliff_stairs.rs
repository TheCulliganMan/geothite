//! Local empty-party geometry fixture followed only by production inputs.
use super::*;

fn step_to(controller: &mut VisibleShellController, button: GameButton, expected: Point) {
    let before = position(controller);
    for _ in 0..8 {
        controller.press(button).expect("native stair input");
        if position(controller) != before {
            break;
        }
    }
    assert_eq!(position(controller), expected);
}

#[test]
#[ignore = "requires the reviewed external Iron Range stair pack"]
fn iron_cliff_cave12_30_36_stairs_both_ways_and_both_dry_hop_lanes() {
    let (root, runtime, mut controller) = fixture((36, 38));
    let grid = grid(&runtime, (36, 38), true);
    for _ in 0..2 {
        for y in (34..38).rev() {
            step_to(&mut controller, GameButton::Up, (36, y));
        }
        grid.walk(&mut controller, (36, 35));
        blocked(&mut controller, GameButton::Left);
        blocked(&mut controller, GameButton::Right);
        grid.walk(&mut controller, (36, 37));
        blocked(&mut controller, GameButton::Left);
        blocked(&mut controller, GameButton::Right);
        grid.walk(&mut controller, (36, 34));
        for y in 35..=38 {
            step_to(&mut controller, GameButton::Down, (36, y));
        }
    }
    for x in [38, 39] {
        grid.walk(&mut controller, (x, 33));
        for _ in 0..8 {
            controller
                .press(GameButton::Down)
                .expect("native return drop");
            controller
                .wait_frames(16)
                .expect("native hop presentation settles");
            let snapshot = controller.snapshot().expect("uninterrupted ledge lane");
            assert_eq!(snapshot.phase, RuntimeShellPhase::Overworld);
            assert!(snapshot.ui.pending_text_wait.is_none());
            assert!(
                !controller.has_pending_script_work(),
                "Mina's course must remain separate"
            );
            assert_ne!(position(&mut controller), (x, 35));
            if position(&mut controller) == (x, 36) {
                break;
            }
        }
        assert_eq!(position(&mut controller), (x, 36));
        blocked(&mut controller, GameButton::Up);
        grid.walk(&mut controller, (36, 38));
    }
    grid.walk(&mut controller, (49, 39));
    for y in 40..=42 {
        step_to(&mut controller, GameButton::Down, (49, y));
    }
    for y in (39..42).rev() {
        step_to(&mut controller, GameButton::Up, (49, y));
    }
    grid.walk(&mut controller, (49, 41));
    blocked(&mut controller, GameButton::Left);
    grid.walk(&mut controller, (36, 38));
    let path = std::env::var_os("CRYSTAL_IRON_STAIRS_REACHED_SAVE")
        .map(PathBuf::from)
        .expect("retained actual stair save path");
    let before = controller.snapshot().expect("reached stair state");
    controller.save(&path).expect("save actual stair course");
    let mut loaded = VisibleShellController::load_save(root, runtime, path, None)
        .expect("Continue actual stair course");
    let after = loaded.snapshot().expect("continued stair state");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(position(&mut loaded), (36, 38));
    step_to(&mut loaded, GameButton::Up, (36, 37));
    step_to(&mut loaded, GameButton::Down, (36, 38));
}
