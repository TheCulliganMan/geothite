//! Northern portage choices and every remaining cliff, reached from paid entry.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};

use crystal_core::input::GameButton;
use crystal_core::systems::safari::SafariPhase;
use crystal_core::world::movement::MovementMode;
use crystal_runtime::CrystalRuntime;

use super::{
    LakeGrid, Point, VisibleShellController, active, admit, button, continue_save,
    controller_fixture, global, launch, press, release_wild, save, step, travel,
};

fn launch_from_basecamp(c: &mut VisibleShellController, grid: &LakeGrid) {
    admit(c);
    travel(c, grid, (3, 90), false);
    launch(c, (3, 89));
}

// Include native down-hop candidates when minimizing grass. The planner may
// overestimate legal approaches to a ledge; actual controller input below
// must still execute every chosen edge, including the two-tile hop.
fn least_grass_path(grid: &LakeGrid, from: Point, to: Point) -> Vec<Point> {
    let mut costs = BTreeMap::from([(from, (0usize, 0usize))]);
    let mut previous = BTreeMap::from([(from, from)]);
    let mut pending = BinaryHeap::from([Reverse(((0usize, 0usize), from))]);
    while let Some(Reverse((cost, at))) = pending.pop() {
        if costs.get(&at) != Some(&cost) {
            continue;
        }
        if at == to {
            break;
        }
        let (x, y) = at;
        let mut nexts = vec![(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)];
        if grid.kind(at) == "HOP_DOWN" {
            nexts.push((x, y + 2));
        }
        for next in nexts {
            if grid.blocked.contains(&next)
                || !matches!(grid.kind(next), "FLOOR" | "TALL_GRASS" | "HOP_DOWN")
            {
                continue;
            }
            let candidate = (
                cost.0 + usize::from(grid.kind(next) == "TALL_GRASS"),
                cost.1 + 1,
            );
            if costs.get(&next).is_none_or(|old| candidate < *old) {
                costs.insert(next, candidate);
                previous.insert(next, at);
                pending.push(Reverse((candidate, next)));
            }
        }
    }
    assert!(
        previous.contains_key(&to),
        "ordinary/ledge path {from:?}->{to:?}"
    );
    let mut path = vec![to];
    while path.last().copied() != Some(from) {
        path.push(previous[path.last().expect("grass path tail")]);
    }
    path.reverse();
    path
}

fn hop_down(c: &mut VisibleShellController, landing: Point) {
    release_wild(c);
    let from = global(c);
    assert_eq!(landing, (from.0, from.1 + 2));
    let before = active(c).steps_remaining;
    for _ in 0..8 {
        if global(c) == landing {
            break;
        }
        press(c, GameButton::Down);
        assert_ne!(global(c), (from.0, from.1 + 1));
    }
    assert_eq!(global(c), landing);
    assert_eq!(active(c).steps_remaining, before - 1);
}

fn read_marker(c: &mut VisibleShellController, target: Point, name: &str, checkpoint: &str) {
    release_wild(c);
    let position = global(c);
    let before = active(c);
    press(c, button(position, target));
    press(c, GameButton::A);
    let snapshot = c.snapshot().expect("native campsite marker");
    let pages = super::super::super::super::visible_field_dialog_pages(&snapshot, &c.shell)
        .expect("actual campsite text")
        .join("\n");
    assert!(pages.contains(name), "{checkpoint}: {pages}");
    assert!(pages.contains("Leave no trace."));
    press(c, GameButton::Down);
    assert_eq!(global(c), position, "camp text owns movement");
    assert_eq!(
        active(c),
        before,
        "camp markers never refill or spend resources"
    );
    save(c, checkpoint);
    for _ in 0..64 {
        let snapshot = c.snapshot().expect("camp marker settling");
        if !snapshot.ui.text_window_open && !c.has_pending_script_work() {
            break;
        }
        press(c, GameButton::A);
    }
    assert!(
        !c.snapshot()
            .expect("camp marker closed")
            .ui
            .text_window_open
    );
    assert_eq!(active(c), before);
}

fn retire(
    c: &mut VisibleShellController,
    grid: &LakeGrid,
    runtime: &CrystalRuntime,
    bank: Option<(Point, Point)>,
    approach: Point,
    ranger: Point,
    name: &str,
) {
    if let Some((water, land)) = bank {
        travel(c, grid, water, true);
        step(c, land);
    }
    travel(c, grid, approach, false);
    release_wild(c);
    let before = active(c);
    assert_eq!(before.phase, SafariPhase::Exploring);
    assert_eq!(before.balls_remaining, 30);
    press(c, button(approach, ranger));
    press(c, GameButton::A);
    assert!(
        c.snapshot()
            .expect("remote ranger return offer")
            .safari_prompt
            .is_some()
    );
    assert_eq!(active(c), before);
    save(c, &format!("{name}-remote-return-offer"));
    press(c, GameButton::A);
    let returned = c.snapshot().expect("actual remote ranger return");
    let destination = &runtime.data().safari_zone.as_ref().expect("zone").return_to;
    assert!(returned.safari.is_none());
    assert_eq!(returned.overworld.map_name, destination.map_name);
    assert_eq!(returned.overworld.tile, destination.tile);
    assert_eq!(returned.overworld.mode, MovementMode::Normal);
    assert_eq!(c.shell.shell.session().state().money, 1000);
    assert_eq!(returned.party.slots.len(), 1);
    let partner = &returned.party.slots[0].pokemon;
    assert_eq!(partner.species.id, "GILLPUP");
    assert_eq!(partner.hp, partner.max_hp);
    assert!(partner.status.is_none());
    assert_eq!(partner.moves[0].name, "SURF");
    assert_eq!(partner.moves[0].current_pp, runtime.data().moves["SURF"].pp);
    save(c, &format!("{name}-returned-basecamp"));
    println!(
        "{name}: paid journey returned, 30 balls, {} travel steps spent",
        500 - before.steps_remaining
    );
}

struct Portage {
    name: &'static str,
    near_land: Point,
    near_water: Point,
    far_land: Point,
    far_water: Point,
    grass: usize,
    water_detour: Option<usize>,
}

#[test]
#[ignore = "requires enabled CRYSTAL_INTERIOR_PACK Safari review"]
fn safari_paid_northern_portages_pocket_and_connected_arm_choices() {
    let cases = [
        Portage {
            name: "pine-pocket",
            near_land: (29, 21),
            near_water: (30, 21),
            far_land: (22, 21),
            far_water: (21, 21),
            grass: 4,
            water_detour: None,
        },
        Portage {
            name: "north-pine-arms",
            near_land: (52, 25),
            near_water: (51, 25),
            far_land: (71, 29),
            far_water: (72, 29),
            grass: 8,
            water_detour: Some(39),
        },
        Portage {
            name: "northwind-portage",
            near_land: (76, 33),
            near_water: (75, 33),
            far_land: (97, 33),
            far_water: (98, 33),
            grass: 6,
            water_detour: Some(45),
        },
    ];
    for case in cases {
        let (root, runtime, mut c) = controller_fixture();
        let grid = LakeGrid::new(&runtime);
        // Regional partner/badge/money setup ends at the actual public ranger.
        launch_from_basecamp(&mut c, &grid);
        travel(&mut c, &grid, case.near_water, true);
        if case.name == "pine-pocket" {
            step(&mut c, case.near_land);
            travel(&mut c, &grid, (31, 28), false);
            read_marker(&mut c, (32, 28), "PINE CAMP", "pine-camp-marker");
            travel(&mut c, &grid, case.near_land, false);
            launch(&mut c, case.near_water);
        }
        let before = active(&mut c).steps_remaining;
        let walking_steps = grid.path(case.near_land, case.far_land, false).len() - 1;
        step(&mut c, case.near_land);
        assert!(
            travel(&mut c, &grid, case.far_land, false) >= case.grass,
            "{} forward habitat",
            case.name
        );
        save(&mut c, &format!("{}-far-bank", case.name));
        launch(&mut c, case.far_water);
        assert_eq!(
            active(&mut c).steps_remaining,
            before - u16::try_from(walking_steps + 2).expect("short portage")
        );
        save(&mut c, &format!("{}-far-water", case.name));
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        assert_eq!(
            c.snapshot().expect("far lake Continue").overworld.mode,
            MovementMode::Surf
        );
        step(&mut c, case.far_land);
        assert!(
            travel(&mut c, &grid, case.near_land, false) >= case.grass,
            "{} reverse habitat",
            case.name
        );
        launch(&mut c, case.near_water);
        if let Some(expected) = case.water_detour {
            let water_steps = grid.path(case.near_water, case.far_water, true).len() - 1;
            assert_eq!(water_steps, expected);
            assert!(
                water_steps > walking_steps + 2,
                "grass must buy a useful shorter crossing"
            );
            let before = active(&mut c).steps_remaining;
            travel(&mut c, &grid, case.far_water, true);
            assert_eq!(
                active(&mut c).steps_remaining,
                before - u16::try_from(water_steps).expect("bounded paddle")
            );
            save(&mut c, &format!("{}-paddle-alternative", case.name));
            if case.name == "northwind-portage" {
                step(&mut c, case.far_land);
                let route = least_grass_path(&grid, case.far_land, case.near_land);
                assert_eq!(route.len() - 1, 38);
                assert_eq!(
                    route
                        .iter()
                        .skip(1)
                        .filter(|p| grid.kind(**p) == "TALL_GRASS")
                        .count(),
                    1
                );
                let before = active(&mut c).steps_remaining;
                for tile in route.into_iter().skip(1) {
                    let from = global(&mut c);
                    if tile == (from.0, from.1 + 2) {
                        hop_down(&mut c, tile);
                    } else {
                        step(&mut c, tile);
                    }
                }
                assert_eq!(active(&mut c).steps_remaining, before - 38);
                release_wild(&mut c);
                save(&mut c, "northwind-drier-ledge-return");
                launch(&mut c, case.near_water);
            }
            retire(
                &mut c,
                &grid,
                &runtime,
                Some(((120, 43), (119, 43))),
                (115, 36),
                (115, 35),
                case.name,
            );
        } else {
            // The western pocket is intentionally separate water. The actual
            // east-to-west crossing above is its route from the main lake.
            // Paddle home across the clipped southern pair as well; that
            // lowest water connection must work in both directions.
            travel(&mut c, &grid, (63, 100), true);
            step(&mut c, (64, 100));
            assert_eq!(
                c.snapshot()
                    .expect("southern east water seam")
                    .overworld
                    .map_name,
                "MnBoundarySafariC01R02"
            );
            release_wild(&mut c);
            save(&mut c, "southern-water-seam-east");
            step(&mut c, (63, 100));
            assert_eq!(
                c.snapshot()
                    .expect("southern west water seam")
                    .overworld
                    .map_name,
                "MnBoundarySafariC00R02"
            );
            release_wild(&mut c);
            save(&mut c, "southern-water-seam-west");
            retire(
                &mut c,
                &grid,
                &runtime,
                Some(((3, 89), (3, 90))),
                (10, 104),
                (10, 103),
                case.name,
            );
        }
        c = continue_save(root, runtime, &mut c);
        assert!(
            c.snapshot()
                .expect("returned trip Continue")
                .safari
                .is_none()
        );
    }
}

struct Cliff {
    name: &'static str,
    bank: (Point, Point),
    upper: Point,
    hop: Point,
    landing: Point,
    return_steps: usize,
    return_grass: usize,
    return_bank: (Point, Point),
    ranger_approach: Point,
    ranger: Point,
}

#[test]
#[ignore = "requires enabled CRYSTAL_INTERIOR_PACK Safari review"]
fn safari_paid_remaining_cliffs_have_real_hops_climbs_and_remote_returns() {
    let cases = [
        Cliff {
            name: "granite-cliff",
            bank: ((120, 43), (119, 43)),
            upper: (107, 31),
            hop: (107, 32),
            landing: (107, 34),
            return_steps: 11,
            return_grass: 0,
            return_bank: ((120, 43), (119, 43)),
            ranger_approach: (115, 36),
            ranger: (115, 35),
        },
        Cliff {
            name: "northwind-upper",
            bank: ((75, 33), (76, 33)),
            upper: (93, 17),
            hop: (93, 18),
            landing: (93, 20),
            return_steps: 11,
            return_grass: 3,
            return_bank: ((120, 43), (119, 43)),
            ranger_approach: (115, 36),
            ranger: (115, 35),
        },
        Cliff {
            name: "northwind-lower",
            bank: ((75, 33), (76, 33)),
            upper: (83, 25),
            hop: (83, 26),
            landing: (83, 28),
            return_steps: 27,
            return_grass: 6,
            return_bank: ((120, 43), (119, 43)),
            ranger_approach: (115, 36),
            ranger: (115, 35),
        },
        Cliff {
            name: "eagle-point",
            bank: ((93, 97), (94, 97)),
            upper: (101, 79),
            hop: (101, 80),
            landing: (101, 82),
            return_steps: 15,
            return_grass: 1,
            return_bank: ((93, 97), (94, 97)),
            ranger_approach: (103, 100),
            ranger: (103, 99),
        },
    ];
    for case in cases {
        let (root, runtime, mut c) = controller_fixture();
        let grid = LakeGrid::new(&runtime);
        launch_from_basecamp(&mut c, &grid);
        travel(&mut c, &grid, case.bank.0, true);
        step(&mut c, case.bank.1);
        travel(&mut c, &grid, case.upper, false);
        save(&mut c, &format!("{}-upper-view", case.name));
        step(&mut c, case.hop);
        hop_down(&mut c, case.landing);
        release_wild(&mut c);
        save(&mut c, &format!("{}-landed", case.name));
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        let before = active(&mut c).steps_remaining;
        for _ in 0..4 {
            press(&mut c, GameButton::Up);
            assert_eq!(global(&mut c), case.landing);
            assert_eq!(active(&mut c).steps_remaining, before);
        }
        assert_eq!(
            grid.path(case.landing, case.upper, false).len() - 1,
            case.return_steps
        );
        assert!(travel(&mut c, &grid, case.upper, false) >= case.return_grass);
        assert_eq!(
            active(&mut c).steps_remaining,
            before - u16::try_from(case.return_steps).expect("bounded cliff return")
        );
        save(&mut c, &format!("{}-climbed-back", case.name));
        if case.name == "granite-cliff" {
            travel(&mut c, &grid, (115, 42), false);
            read_marker(&mut c, (116, 42), "GRANITE BAY CAMP", "granite-camp-marker");
        } else if case.name == "eagle-point" {
            travel(&mut c, &grid, (103, 94), false);
            read_marker(&mut c, (104, 94), "QUIET ISLAND CAMP", "quiet-camp-marker");
        }
        let return_bank = if case.bank != case.return_bank {
            // Descend the east shoulder rather than repeating the west-bank
            // approach; Granite Bay's ranger finishes the northern lake loop.
            travel(&mut c, &grid, (97, 33), false);
            launch(&mut c, (98, 33));
            Some(case.return_bank)
        } else {
            None
        };
        retire(
            &mut c,
            &grid,
            &runtime,
            return_bank,
            case.ranger_approach,
            case.ranger,
            case.name,
        );
        c = continue_save(root, runtime, &mut c);
        assert!(
            c.snapshot()
                .expect("cliff return Continue")
                .safari
                .is_none()
        );
    }
}
