//! Visit the station garden during Reed's return and find the records officer.
use crystal_core::input::GameButton::{Down, Up};
use crystal_core::world::map::TilePosition;

use super::super::winona::travel;
use super::{at, collision, fixture, load, mark_runtime_snapshot_dirty, talk, walk};

#[test]
#[ignore = "requires the public services edition in CRYSTAL_INTERIOR_PACK"]
fn st_paul_waiting_garden_stays_open_when_reed_returns() {
    let (root, runtime, _) = load();
    let mut visitor = fixture(&root, &runtime, "MnStPaul", (8, 8));
    visitor
        .shell
        .shell
        .session_mut()
        .state_mut()
        .flags
        .set_event_flag("EVENT_MN_COURIER_DEFEATED", true)
        .expect("Reed waits by the south entrance after the courier fight");
    mark_runtime_snapshot_dirty(&mut visitor.shell);
    travel(&mut visitor, Up, "MnStPaulUnionDepot", (15, 21));
    let inside = visitor
        .shell
        .shell
        .presentation_snapshot()
        .expect("depot actors");
    assert_eq!(
        inside.visible_object_runtime_tiles["MN_ARC_SP_REED"],
        TilePosition::new(22, 17)
    );
    walk(&mut visitor, &runtime, (7, 14));
    assert!(talk(&mut visitor, (7, 13)).contains("timetable"));
    // Cross to the open entrance above the timetable, clear of the pump scene.
    walk(&mut visitor, &runtime, (15, 13));
    walk(&mut visitor, &runtime, (22, 13));
    walk(&mut visitor, &runtime, (24, 15));
    assert!(talk(&mut visitor, (25, 15)).contains("bean in that pot"));
    walk(&mut visitor, &runtime, (22, 13));
    walk(&mut visitor, &runtime, (15, 13));
    walk(&mut visitor, &runtime, (15, 22));
    travel(&mut visitor, Down, "MnStPaul", (8, 8));
}

#[test]
#[ignore = "requires the public services edition in CRYSTAL_INTERIOR_PACK"]
fn st_paul_records_officer_has_a_finished_wall_and_a_public_approach() {
    let (root, runtime, _) = load();
    let mut visitor = fixture(&root, &runtime, "MnStPaul", (40, 12));
    let hall = "MnStPaulCapitolHall";
    travel(&mut visitor, Up, hall, (3, 5));
    assert_eq!(
        collision(&runtime, &runtime.data().maps[hall], (13, 0)),
        Some("WALL")
    );
    walk(&mut visitor, &runtime, (13, 2));
    assert!(talk(&mut visitor, (13, 1)).contains("MINA at UNION"));
    assert_eq!(at(&mut visitor), (13, 2));
    walk(&mut visitor, &runtime, (3, 6));
    travel(&mut visitor, Down, "MnStPaul", (40, 12));
}
