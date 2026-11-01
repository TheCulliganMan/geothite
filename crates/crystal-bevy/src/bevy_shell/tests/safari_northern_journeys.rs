//! Northern portage choices and every remaining cliff, reached from paid entry.

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

// These are current native cave12 flights in global 16px walking coordinates.
// The old high-cliff HOP_DOWN cells were deliberately retired, not shortcuts
// to restore. Every trip begins at the actual ranger and pays for admission.
#[test]
#[ignore = "requires enabled Safari on the current repaired preserve terrain"]
fn safari_paid_five_native_stairs_keep_cliff_faces_closed_and_remote_returns() {
    use super::ledges_and_exit::blocked;

    let cases = [
        ("palisade", (40, 58), (35, 58), ((36, 68), (36, 67))),
        ("northwind-upper", (96, 18), (93, 18), ((75, 33), (76, 33))),
        ("northwind-lower", (90, 26), (83, 26), ((75, 33), (76, 33))),
        (
            "granite-cliff",
            (110, 32),
            (107, 32),
            ((120, 43), (119, 43)),
        ),
        ("eagle-point", (106, 80), (101, 80), ((93, 97), (94, 97))),
    ];
    for (name, (x, y), closed_face, bank) in cases {
        let (root, runtime, mut c) = controller_fixture();
        let grid = LakeGrid::new(&runtime);
        assert!(!grid.cells.values().any(|kind| kind.starts_with("HOP_")));
        launch_from_basecamp(&mut c, &grid);
        travel(&mut c, &grid, bank.0, true);
        step(&mut c, bank.1);
        travel(&mut c, &grid, closed_face, false);
        assert_eq!(grid.kind((closed_face.0, closed_face.1 + 1)), "WALL");
        blocked(&mut c, GameButton::Down);
        save(&mut c, &format!("{name}-closed-cliff"));
        let above = (x + 1, y - 1);
        let below = (x + 1, y + 2);
        travel(&mut c, &grid, below, false);
        let before = active(&mut c).steps_remaining;
        step(&mut c, (x + 1, y + 1));
        assert_eq!(grid.kind((x, y + 1)), "WALL");
        blocked(&mut c, GameButton::Left);
        step(&mut c, (x + 1, y));
        step(&mut c, above);
        assert_eq!(active(&mut c).steps_remaining, before - 3);
        save(&mut c, &format!("{name}-upper-stair"));
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        let before = active(&mut c).steps_remaining;
        step(&mut c, (x + 1, y));
        step(&mut c, (x + 1, y + 1));
        blocked(&mut c, GameButton::Left);
        step(&mut c, below);
        assert_eq!(active(&mut c).steps_remaining, before - 3);
        save(&mut c, &format!("{name}-lower-stair-return"));
        if name == "granite-cliff" {
            travel(&mut c, &grid, (115, 42), false);
            read_marker(&mut c, (116, 42), "GRANITE BAY CAMP", "granite-camp-marker");
        } else if name == "eagle-point" {
            travel(&mut c, &grid, (103, 94), false);
            read_marker(&mut c, (104, 94), "QUIET ISLAND CAMP", "quiet-camp-marker");
        }
        travel(&mut c, &grid, bank.1, false);
        launch(&mut c, bank.0);
        let (return_bank, approach, ranger) = if name == "eagle-point" {
            (((93, 97), (94, 97)), (103, 100), (103, 99))
        } else {
            (((120, 43), (119, 43)), (115, 36), (115, 35))
        };
        retire(
            &mut c,
            &grid,
            &runtime,
            Some(return_bank),
            approach,
            ranger,
            name,
        );
        c = continue_save(root, runtime, &mut c);
        assert!(
            c.snapshot()
                .expect("actual returned Continue")
                .safari
                .is_none()
        );
    }
}
