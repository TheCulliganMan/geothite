//! Initial Brainerd setup only; later story and regional travel use native input.
use std::collections::BTreeMap;

use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::authored_route_fixture::{RouteGrid, RouteSpec, position};
use super::super::rochester::flee;
use super::super::winona::{at, continued, travel};
use super::northstar_iron_course::signal_course;
use super::{VisibleShellController, event, finish, press, walk_to};

pub(super) fn road(c: &mut VisibleShellController, r: &CrystalRuntime, number: u8, to: (i16, i16)) {
    let (prefix, columns, rows, tileset) = match number {
        11 => ("MnR11", 2, 4, "mn_north_shore"),
        12 => ("MnR12", 2, 3, "mn_gunflint"),
        13 => ("MnR13", 1, 3, "johto"),
        _ => panic!("unsupported coastal road"),
    };
    let mut doors = BTreeMap::new();
    for row in 0..rows {
        for column in 0..columns {
            let m = r
                .data()
                .map_module(&format!("{prefix}C{column:02}R{row:02}"))
                .expect("road module");
            for w in &m.events.warps {
                doors.insert(
                    (column * 64 + w.x as i16, row * 48 + w.y as i16),
                    "authored room",
                );
            }
        }
    }
    let start = position(c);
    let grid = RouteGrid::new_with_grass(
        r,
        RouteSpec {
            prefix,
            columns,
            rows,
            tileset,
            start,
            doors,
        },
    );
    for button in grid.path(start, to) {
        let before = position(c);
        for _ in 0..8 {
            press(c, button);
            if c.snapshot().expect("road encounter").battle.is_some() {
                flee(c);
            }
            assert!(
                c.snapshot()
                    .expect("road or recovery")
                    .overworld
                    .map_name
                    .starts_with(prefix),
                "native battle recovery interrupted R{number}"
            );
            if position(c) != before {
                break;
            }
        }
        assert_ne!(position(c), before, "native coastal step");
    }
    assert_eq!(position(c), to);
}

fn care(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk_to(c, r, (15, 10));
    travel(c, GameButton::Up, "MnGrandMaraisPokecenter", (3, 6));
    walk_to(c, r, (3, 3));
    press(c, GameButton::Up);
    press(c, GameButton::A);
    let money = c.shell.shell.session().state().money;
    finish(c);
    let s = c.snapshot().expect("regional native care");
    assert!(s.party.slots.iter().all(|p| {
        p.pokemon.hp == p.pokemon.max_hp
            && p.pokemon
                .moves
                .iter()
                .all(|m| m.current_pp == r.data().moves[&m.name].pp)
    }));
    assert_eq!(c.shell.shell.session().state().money, money);
    walk_to(c, r, (3, 6));
    travel(c, GameButton::Down, "MnGrandMarais", (15, 10));
}

#[test]
#[ignore = "external shore pack; only original six-badge/Mew45/Cut/Brainerd setup is supplied"]
fn north_shore_journey_controller_earned_signal_full_coast_gunflint_and_return() {
    let (root, r, mut c) = signal_course();
    walk_to(&mut c, &r, (8, 10));
    travel(&mut c, GameButton::Down, "MnDuluth", (48, 36));
    super::gym_eight_approach::care(&mut c, &r, "MnDuluth", (37, 12));
    walk_to(&mut c, &r, (32, 4));
    travel(&mut c, GameButton::Up, "MnR11C00R03", (46, 47));
    road(&mut c, &r, 11, (42, 4));
    travel(&mut c, GameButton::Right, "MnGrandMaraisShoreGate", (1, 4));
    walk_to(&mut c, &r, (5, 4));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    finish(&mut c);
    walk_to(&mut c, &r, (8, 4));
    travel(&mut c, GameButton::Right, "MnGrandMarais", (8, 18));
    care(&mut c, &r);
    c = continued(c, &root, &r, "shore-earned-marais-care");
    walk_to(&mut c, &r, (4, 8));
    travel(&mut c, GameButton::Left, "MnR12C01R01", (63, 8));
    road(&mut c, &r, 12, (0, 56));
    travel(&mut c, GameButton::Left, "MnBoundaryWaters", (63, 24));
    travel(&mut c, GameButton::Left, "MnBoundaryWaters", (60, 24));
    c = continued(c, &root, &r, "shore-earned-boundary-arrival");
    travel(&mut c, GameButton::Right, "MnR12C00R01", (0, 8));
    road(&mut c, &r, 12, (127, 57));
    travel(&mut c, GameButton::Right, "MnGrandMarais", (0, 9));
    travel(&mut c, GameButton::Right, "MnGrandMarais", (4, 9));
    care(&mut c, &r);
    walk_to(&mut c, &r, (8, 19));
    travel(&mut c, GameButton::Left, "MnGrandMaraisShoreGate", (8, 5));
    walk_to(&mut c, &r, (1, 5));
    travel(&mut c, GameButton::Left, "MnR11C00R00", (42, 5));
    road(&mut c, &r, 11, (47, 191));
    travel(&mut c, GameButton::Down, "MnDuluth", (33, 0));
    travel(&mut c, GameButton::Down, "MnDuluth", (33, 4));
    at(&mut c, "MnDuluth", (33, 4));
    assert!(event(&c, "EVENT_MN_LIGHTHOUSE_SIGNAL"));
    for flag in [
        "EVENT_MN_GRAND_MARAIS_STARTER",
        "EVENT_MN_GRAND_MARAIS_SIGNAL",
        "EVENT_MN_MARAIS_SPARE_PART_RECEIVED",
        "EVENT_MN_SURVEY_RESTORED",
    ] {
        assert!(!event(&c, flag), "regional travel must not grant {flag}");
    }
    let _ = continued(c, &root, &r, "shore-earned-duluth-return");
}
