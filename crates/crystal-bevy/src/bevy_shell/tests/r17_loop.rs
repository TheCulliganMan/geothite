//! Local empty-party fixture; subsequent shelter travel uses production input.
use std::path::PathBuf;

use crystal_core::input::GameButton;

use super::super::VisibleShellController;
use super::interior_life::{self, collision, fixture, load, walk};
use super::northstar_ice_lair::event;
use super::r10_gallery::{flags, press, settle, tile};
use super::winona::{at, finish, travel};

const ROOM: &str = "MnR17Cave1F";
const OUTSIDE: &str = "MnR17C00R00";
const EVENT: &str = "EVENT_MN_R17_ITEM_04";

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK and fresh CRYSTAL_R17_LOOP_REVIEW_DIR; local fixture only"]
fn r17_loop_controller_both_arms_reader_revive_and_continue() {
    let out = PathBuf::from(std::env::var_os("CRYSTAL_R17_LOOP_REVIEW_DIR").expect("fresh output"));
    assert!(!out.exists(), "retain previous gameplay evidence");
    std::fs::create_dir_all(&out).expect("review output");
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, OUTSIDE, (10, 44));
    c.save(out.join("initial-outside.crystalsave"))
        .expect("initial local fixture");
    let mut expected_flags = flags(&c);
    let original_money = c.shell.shell.session().state().money;
    let quantity = |c: &VisibleShellController| {
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["REVIVE"])
    };
    assert_eq!(quantity(&c), 0);
    travel(&mut c, GameButton::Up, ROOM, (9, 13));
    walk(&mut c, &r, (9, 12));
    walk(&mut c, &r, (7, 10));
    let text = interior_life::talk(&mut c, (6, 10));
    assert!(text.to_lowercase().contains("lichen"), "{text}");
    at(&mut c, ROOM, (7, 10));
    // Force the west arm, northern crossing and east arm individually. A
    // shortest-path visit to one camera alone would not exercise the circuit.
    for target in [(7, 9), (5, 9), (5, 7), (5, 6)] {
        walk(&mut c, &r, target);
        at(&mut c, ROOM, target);
    }
    press(&mut c, GameButton::Right);
    assert_eq!(tile(&mut c), (5, 6), "central granite blocks the west side");
    press(&mut c, GameButton::Up);
    assert_eq!(tile(&mut c), (5, 6), "uncollected Revive occupies its tile");
    press(&mut c, GameButton::A);
    assert!(event(&c, EVENT));
    assert_eq!(quantity(&c), 1);
    press(&mut c, GameButton::Down);
    assert_eq!(tile(&mut c), (5, 6), "item notice owns movement");
    finish(&mut c);
    settle(&mut c, "original Revive collected");
    expected_flags.0.insert(EVENT.into());
    assert_eq!(flags(&c), expected_flags);
    travel(&mut c, GameButton::Up, ROOM, (5, 5));
    for target in [(6, 5), (8, 5), (8, 6)] {
        walk(&mut c, &r, target);
        at(&mut c, ROOM, target);
    }
    press(&mut c, GameButton::Left);
    assert_eq!(tile(&mut c), (8, 6), "central granite blocks the east side");
    for target in [(8, 7), (8, 8), (7, 8), (7, 9), (7, 10)] {
        walk(&mut c, &r, target);
        at(&mut c, ROOM, target);
    }
    for p in [(6, 6), (7, 6), (6, 7), (7, 7)] {
        assert_eq!(collision(&r, &r.data().maps[ROOM], p), Some("WALL"));
    }
    c.save(out.join("circuit-collected.crystalsave"))
        .expect("save circuit");
    c = VisibleShellController::load_save(
        root.clone(),
        r.clone(),
        out.join("circuit-collected.crystalsave"),
        None,
    )
    .expect("real Continue");
    at(&mut c, ROOM, (7, 10));
    assert_eq!(flags(&c), expected_flags);
    assert_eq!(quantity(&c), 1);
    walk(&mut c, &r, (5, 6));
    travel(&mut c, GameButton::Up, ROOM, (5, 5));
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert_eq!(quantity(&c), 1, "reward cannot repeat after Continue");
    walk(&mut c, &r, (9, 12));
    travel(&mut c, GameButton::Down, OUTSIDE, (10, 44));
    c.wait_frames(80).expect("passive mouth settling");
    at(&mut c, OUTSIDE, (10, 44));
    travel(&mut c, GameButton::Down, OUTSIDE, (10, 45));
    travel(&mut c, GameButton::Up, OUTSIDE, (10, 44));
    travel(&mut c, GameButton::Up, ROOM, (9, 13));
    walk(&mut c, &r, (5, 6));
    travel(&mut c, GameButton::Up, ROOM, (5, 5));
    assert_eq!(quantity(&c), 1);
    assert_eq!(flags(&c), expected_flags);
    assert_eq!(c.shell.shell.session().state().money, original_money);
    press(&mut c, GameButton::Start);
    assert!(c.shell.start_menu_cursor.is_some());
    press(&mut c, GameButton::B);
    c.save(out.join("returned-collected.crystalsave"))
        .expect("final reached state");
}
