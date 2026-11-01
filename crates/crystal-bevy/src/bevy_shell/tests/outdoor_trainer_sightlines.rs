//! Preserve Mina's real battle while keeping her sight lane off the cliff rim.

use crystal_core::input::GameButton;

use super::{Setup, assert_at, fixture, load, move_to, press};

#[test]
#[ignore = "requires the reviewed external trainer-sightline candidate"]
fn iron_rim_stays_clear_while_mina_challenges_along_the_lower_bench() {
    let (root, runtime) = load();
    let map = "MnR09C00R00";
    let object = runtime.data().maps[map]
        .objects
        .iter()
        .find(|o| o.script == "MN_R09_T03")
        .expect("Mina's original actor");
    assert_eq!(object.spritemovedata, "SPRITEMOVEDATA_STANDING_RIGHT");
    assert_eq!([object.x, object.y], [41, 37]);
    let mut rim = fixture(&root, &runtime, Setup::at(map, (40, 34)));
    let before = rim.snapshot().expect("upper rim fixture");
    move_to(&mut rim, GameButton::Right, (41, 34));
    for button in [GameButton::Up, GameButton::Down] {
        for _ in 0..3 {
            press(&mut rim, button);
        }
        assert_at(&mut rim, map, (41, 34));
        assert!(matches!(
            rim.shell
                .shell
                .snapshot()
                .expect("runtime phase snapshot")
                .phase,
            crystal_runtime::RuntimeShellPhase::Overworld
        ));
        assert!(
            !rim.snapshot()
                .expect("upper rim has no trainer dialogue")
                .ui
                .text_window_open
        );
    }
    move_to(&mut rim, GameButton::Left, (40, 34));
    let after = rim.snapshot().expect("ordinary rim crossing");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);

    // Separate explicit placement on the lower bench, not a post-input teleport.
    let mut lower = fixture(&root, &runtime, Setup::at(map, (44, 36)));
    move_to(&mut lower, GameButton::Down, (44, 37));
    assert!(
        lower
            .snapshot()
            .expect("Mina's native challenge")
            .ui
            .text_window_open
            || lower.has_pending_script_work()
    );
    for _ in 0..96 {
        if lower
            .snapshot()
            .expect("native trainer continuation")
            .battle
            .is_some()
        {
            break;
        }
        press(&mut lower, GameButton::A);
    }
    assert!(
        lower
            .snapshot()
            .expect("Mina retains her battle")
            .battle
            .is_some()
    );
}
