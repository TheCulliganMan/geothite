//! Follow the store directory, ride to every department and return through the lobby.
use crystal_core::input::GameButton::{A, B, Down, Up};
use crystal_runtime::CrystalRuntime;

use super::super::winona::{at, continue_in_place, press, travel};
use super::{VisibleShellController, fixture, load, talk, walk};

#[path = "store_floor_selector.rs"]
mod floor_selector;

const CAB: &str = "MnMinneapolisStoreElevator";
const LOBBY: &str = "MnMinneapolisDepartmentStore1F";

fn board(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk(c, r, (2, 1));
    travel(c, Up, CAB, (1, 3));
}

fn panel(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk(c, r, (3, 1));
    press(c, Up);
    press(c, A);
    assert!(
        c.shell.elevator_cursor.is_some(),
        "the wall controls offer the floor menu"
    );
    let snapshot = c.snapshot().expect("elevator panel");
    assert_eq!(snapshot.ui.elevators[0].floors.len(), 5);
}

#[test]
#[ignore = "requires the Lakeside Stops edition in CRYSTAL_INTERIOR_PACK"]
fn minneapolis_store_directory_and_lift_serve_all_five_departments() {
    let (root, runtime, _) = load();
    let mut visitor = fixture(&root, &runtime, "MnMinneapolis", (54, 36));
    travel(&mut visitor, Up, LOBBY, (7, 6));
    walk(&mut visitor, &runtime, (14, 1));
    let directory = talk(&mut visitor, (14, 0));
    for department in [
        "Balls & Repels",
        "Medicine",
        "Battle gear",
        "Vitamins",
        "Move machines",
    ] {
        assert!(directory.contains(department), "{directory}");
    }
    for (floor, sign) in [
        (5, "MOVE MACHINES"),
        (2, "MEDICINE"),
        (4, "VITAMINS"),
        (3, "BATTLE GEAR"),
        (1, "STORE DIRECTORY"),
    ] {
        board(&mut visitor, &runtime);
        panel(&mut visitor, &runtime);
        for _ in 1..floor {
            press(&mut visitor, Down);
        }
        press(&mut visitor, A);
        visitor.wait_frames(180).expect("finish the ride");
        let destination = format!("MnMinneapolisDepartmentStore{floor}F");
        at(&mut visitor, CAB, (3, 1));
        if floor == 5 {
            continue_in_place(&mut visitor, &root, &runtime, "store-after-ride");
        }
        walk(&mut visitor, &runtime, (1, 3));
        at(&mut visitor, &destination, (2, 1));
        walk(&mut visitor, &runtime, (3, 1));
        assert!(talk(&mut visitor, (3, 0)).contains(sign));
    }
    walk(&mut visitor, &runtime, (15, 0));
    at(&mut visitor, "MnMinneapolisDepartmentStore2F", (15, 1));
    walk(&mut visitor, &runtime, (15, 0));
    at(&mut visitor, LOBBY, (15, 1));
    walk(&mut visitor, &runtime, (7, 7));
    at(&mut visitor, "MnMinneapolis", (54, 36));
}

#[test]
#[ignore = "requires the Lakeside Stops edition in CRYSTAL_INTERIOR_PACK"]
fn boarding_floor_survives_cancel_current_floor_and_continue_in_the_cab() {
    let (root, runtime, _) = load();
    let mut visitor = fixture(&root, &runtime, "MnMinneapolis", (54, 36));
    travel(&mut visitor, Up, LOBBY, (7, 6));
    for (choice, exit) in [(B, (1, 3)), (A, (2, 3))] {
        board(&mut visitor, &runtime);
        panel(&mut visitor, &runtime);
        press(&mut visitor, choice);
        at(&mut visitor, CAB, (3, 1));
        assert!(visitor.shell.elevator_cursor.is_none());
        if choice == B {
            // A pre-fix save kept the boarding door only in previous_warp_index.
            visitor
                .shell
                .shell
                .session_mut()
                .state_mut()
                .backup_warp_index = None;
        }
        continue_in_place(&mut visitor, &root, &runtime, "store-lift");
        walk(&mut visitor, &runtime, exit);
        at(&mut visitor, LOBBY, (2, 1));
    }
}
