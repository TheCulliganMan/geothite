//! Explicit local geometry fixtures; every traversal uses production input.

use super::{CITY, Setup, assert_at, fixture, load, move_to, press};
use crystal_core::input::GameButton;
use std::path::PathBuf;

use super::super::authored_route_fixture::indoor_walk;

#[test]
#[ignore = "requires reviewed external Duluth cliff-stair pack and retained save directory"]
fn duluth_all_eleven_cave_flights_climb_descend_and_continue() {
    let (root, runtime) = load();
    let output = PathBuf::from(
        std::env::var_os("CRYSTAL_DULUTH_STAIRS_SAVES").expect("retained actual stair saves"),
    );
    std::fs::create_dir_all(&output).expect("stair save directory");
    // Upper and lower landing coordinates are gameplay tiles, not metatiles.
    // Two side approaches bend at the foot; the supply flight turns at its crest.
    let courses = [
        ("observation-west", (14, 13), (14, 18), (14, 13), (14, 18)),
        ("observation-east", (37, 15), (37, 18), (37, 15), (37, 18)),
        ("civic-upper", (46, 13), (46, 18), (46, 13), (46, 18)),
        ("hillside-west", (71, 11), (69, 16), (71, 11), (71, 16)),
        ("hillside-front", (80, 13), (80, 16), (80, 13), (80, 16)),
        ("outdoor-supply", (87, 10), (88, 12), (88, 10), (88, 12)),
        ("weather-west", (99, 11), (97, 16), (99, 11), (99, 16)),
        ("weather-front", (106, 13), (106, 16), (106, 13), (106, 16)),
        ("weather-east", (114, 9), (114, 12), (114, 9), (114, 12)),
        ("garden-north", (47, 25), (47, 27), (47, 25), (47, 27)),
        ("garden-corner", (56, 39), (56, 42), (56, 39), (56, 42)),
    ];
    for (name, upper, lower, crest, foot) in courses {
        let mut controller = fixture(&root, &runtime, Setup::at(CITY, upper));
        let before = controller.snapshot().expect("upper landing");
        if upper != crest {
            move_to(&mut controller, GameButton::Right, crest);
        }
        let step = (crest.0, foot.1 - 1);
        move_to(&mut controller, GameButton::Down, step);
        for button in [GameButton::Left, GameButton::Right]
            .into_iter()
            .filter(|button| name != "garden-north" || *button == GameButton::Left)
        {
            for _ in 0..3 {
                press(&mut controller, button);
            }
            assert_at(&mut controller, CITY, step);
        }
        move_to(&mut controller, GameButton::Down, foot);
        if foot != lower {
            move_to(&mut controller, GameButton::Left, lower);
        }
        controller
            .save(&output.join(format!("{name}-lower.crystalsave")))
            .expect("save actually reached lower landing");
        if foot != lower {
            move_to(&mut controller, GameButton::Right, foot);
        }
        move_to(&mut controller, GameButton::Up, crest);
        if crest != upper {
            move_to(&mut controller, GameButton::Left, upper);
        }
        assert_at(&mut controller, CITY, upper);
        let after = controller
            .snapshot()
            .expect("climbed back to real upper landing");
        assert_eq!(before.party, after.party, "{name} must not alter partners");
        assert_eq!(before.bag, after.bag, "{name} must not spend items");
        let path = output.join(format!("{name}.crystalsave"));
        controller
            .save(&path)
            .expect("save after both stair directions");
        let mut continued =
            super::VisibleShellController::load_save(root.clone(), runtime.clone(), path, None)
                .expect("Continue reached stair crest");
        assert_at(&mut continued, CITY, upper);
        let resumed = continued.snapshot().expect("continued partners and bag");
        assert_eq!(after.party, resumed.party);
        assert_eq!(after.bag, resumed.bag);
    }
    // The garden's east retaining wall stays continuous above the corner.
    let mut controller = fixture(&root, &runtime, Setup::at(CITY, (56, 37)));
    for _ in 0..3 {
        press(&mut controller, GameButton::Right);
    }
    assert_at(&mut controller, CITY, (56, 37));
}

#[test]
#[ignore = "requires reviewed external Duluth cliff-stair pack"]
fn duluth_high_cliff_edges_block_hops_in_both_directions() {
    let (root, runtime) = load();
    let faces = [
        (24, 14),
        (25, 14),
        (84, 14),
        (85, 14),
        (44, 40),
        (45, 40),
        (46, 40),
        (47, 40),
        (48, 40),
        (49, 40),
        (40, 40),
        (41, 40),
        (42, 40),
        (43, 40),
    ];
    for (x, y) in faces {
        let mut upper = fixture(&root, &runtime, Setup::at(CITY, (x, y)));
        let mut lower = fixture(&root, &runtime, Setup::at(CITY, (x, y + 2)));
        for _ in 0..3 {
            press(&mut upper, GameButton::Down);
            press(&mut lower, GameButton::Up);
        }
        assert_at(&mut upper, CITY, (x, y));
        assert_at(&mut lower, CITY, (x, y + 2));
    }
}

#[test]
#[ignore = "requires the reviewed palette-matched Duluth terrace pack and retained saves"]
fn duluth_garden_corner_stair_reaches_lower_street_continue_and_real_clinic() {
    let (root, runtime) = load();
    let output = PathBuf::from(
        std::env::var_os("CRYSTAL_DULUTH_TERRACE_SAVES").expect("retained terrace journey saves"),
    );
    std::fs::create_dir_all(&output).expect("terrace journey save directory");
    let mut controller = fixture(&root, &runtime, Setup::at(CITY, (55, 33)));
    let before = controller.snapshot().expect("upper garden fixture");
    move_to(&mut controller, GameButton::Down, (55, 39));
    move_to(&mut controller, GameButton::Right, (56, 39));
    move_to(&mut controller, GameButton::Down, (56, 41));
    for button in [GameButton::Left, GameButton::Right] {
        for _ in 0..3 {
            press(&mut controller, button);
        }
        assert_at(&mut controller, CITY, (56, 41));
    }
    move_to(&mut controller, GameButton::Down, (56, 42));
    controller
        .save(&output.join("reached-corner-foot.crystalsave"))
        .expect("save actual corner stair foot");
    // The shared short-segment helper permits eight taps. A facing change
    // consumes a tap, so use the actual street midpoint on this longer leg.
    move_to(&mut controller, GameButton::Right, (60, 42));
    move_to(&mut controller, GameButton::Right, (64, 42));
    move_to(&mut controller, GameButton::Up, (64, 37));
    // The abandoned mid-height spur cannot cut sideways through the cliff.
    for _ in 0..3 {
        press(&mut controller, GameButton::Left);
    }
    assert_at(&mut controller, CITY, (64, 37));
    let path = output.join("reached-main-street.crystalsave");
    controller
        .save(&path)
        .expect("save real lower-road journey");
    let mut continued = super::VisibleShellController::load_save(root, runtime.clone(), path, None)
        .expect("Continue actual terrace-to-street journey");
    assert_at(&mut continued, CITY, (64, 37));
    indoor_walk(&mut continued, &runtime, (69, 32));
    super::enter(&mut continued, super::CLINIC, (10, 13));
    indoor_walk(&mut continued, &runtime, (10, 15));
    for _ in 0..4 {
        if continued
            .snapshot()
            .expect("clinic return")
            .overworld
            .map_name
            == CITY
        {
            break;
        }
        press(&mut continued, GameButton::Down);
    }
    assert_at(&mut continued, CITY, (69, 32));
    indoor_walk(&mut continued, &runtime, (64, 37));
    move_to(&mut continued, GameButton::Down, (64, 42));
    move_to(&mut continued, GameButton::Left, (60, 42));
    move_to(&mut continued, GameButton::Left, (56, 42));
    move_to(&mut continued, GameButton::Up, (56, 39));
    move_to(&mut continued, GameButton::Left, (55, 39));
    move_to(&mut continued, GameButton::Up, (55, 33));
    let after = continued.snapshot().expect("returned to upper garden");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    continued
        .save(&output.join("returned-garden.crystalsave"))
        .expect("save completed corner stair and clinic journey");
}
