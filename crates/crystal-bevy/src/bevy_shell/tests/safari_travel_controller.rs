//! Paid ranger-to-lake journeys, without beyond-gate or budget injection.

#[path = "safari_cut_controller.rs"]
mod cut_gate;
#[path = "safari_departure_controller.rs"]
mod departures;

#[path = "safari_ledge_exit_controller.rs"]
mod ledges_and_exit;
#[path = "safari_northern_journeys.rs"]
mod northern_journeys;

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crystal_core::input::GameButton;
use crystal_core::systems::safari::{SafariEndReason, SafariPhase};
use crystal_core::world::movement::MovementMode;
use crystal_runtime::CrystalRuntime;

use super::{
    Point, VisibleShellController, active, button, continue_save, controller_fixture, press, run,
    save,
};

struct LakeGrid {
    cells: BTreeMap<Point, String>,
    blocked: BTreeSet<Point>,
}

impl LakeGrid {
    fn new(runtime: &CrystalRuntime) -> Self {
        let zone = runtime
            .data()
            .safari_zone
            .as_ref()
            .expect("review habitats");
        let mut cells = BTreeMap::new();
        let mut blocked = BTreeSet::new();
        for (name, map) in &zone.maps {
            let Some([ox, oy]) = map.origin_core else {
                continue;
            };
            let module = &runtime.data().maps[name];
            let height = i16::try_from(module.attributes.height).expect("native map height fits");
            let width = i16::try_from(module.attributes.width).expect("native map width fits");
            for y in 0..height * 2 {
                for x in 0..width * 2 {
                    let kind = super::collision(runtime, module, (x, y))
                        .expect("native preserve collision");
                    cells.insert((ox as i16 + x, oy as i16 + y), kind.to_string());
                }
            }
            for object in &module.objects {
                if object
                    .object_identifier
                    .as_ref()
                    .is_some_and(|id| zone.gate_court.guard_identifiers.contains(id))
                {
                    // Actual admission hides these bodies. All other native
                    // actors remain obstacles in the input itinerary.
                    continue;
                }
                blocked.insert((ox as i16 + object.x as i16, oy as i16 + object.y as i16));
            }
        }
        Self { cells, blocked }
    }

    fn kind(&self, p: Point) -> &str {
        self.cells.get(&p).map_or("OUTSIDE", String::as_str)
    }

    fn path(&self, from: Point, to: Point, water: bool) -> Vec<Point> {
        let mut previous = BTreeMap::from([(from, from)]);
        let mut queue = VecDeque::from([from]);
        while let Some(at) = queue.pop_front() {
            if at == to {
                break;
            }
            for next in [
                (at.0 - 1, at.1),
                (at.0 + 1, at.1),
                (at.0, at.1 - 1),
                (at.0, at.1 + 1),
            ] {
                let allowed = if water {
                    self.kind(next) == "WATER"
                } else {
                    matches!(self.kind(next), "FLOOR" | "TALL_GRASS")
                };
                if allowed && !self.blocked.contains(&next) && !previous.contains_key(&next) {
                    previous.insert(next, at);
                    queue.push_back(next);
                }
            }
        }
        assert!(
            previous.contains_key(&to),
            "native path {from:?}->{to:?}, water={water}"
        );
        let mut result = vec![to];
        while result.last().copied() != Some(from) {
            result.push(previous[result.last().expect("path tail")]);
        }
        result.reverse();
        result
    }
}

fn global(c: &mut VisibleShellController) -> Point {
    let snapshot = c.snapshot().expect("actual field position");
    let map = &c
        .shell
        .runtime
        .data()
        .safari_zone
        .as_ref()
        .expect("zone")
        .maps[&snapshot.overworld.map_name];
    let [x, y] = map.origin_core.expect("outdoor itinerary");
    (
        x as i16 + snapshot.overworld.tile.x,
        y as i16 + snapshot.overworld.tile.y,
    )
}

fn release_wild(c: &mut VisibleShellController) {
    if matches!(active(c).phase, SafariPhase::Encounter(_)) {
        let before = active(c);
        run(c);
        assert_eq!(active(c).steps_remaining, before.steps_remaining);
        assert_eq!(active(c).balls_remaining, before.balls_remaining);
    }
}

fn step(c: &mut VisibleShellController, target: Point) {
    release_wild(c);
    let from = global(c);
    let remaining = active(c).steps_remaining;
    for _ in 0..8 {
        if global(c) == target {
            break;
        }
        press(c, button(from, target));
    }
    assert_eq!(global(c), target, "actual native adjacent step");
    assert_eq!(
        active(c).steps_remaining,
        remaining - 1,
        "one successful travel debit"
    );
    assert!(c.snapshot().expect("capture-only field").battle.is_none());
}

fn travel(c: &mut VisibleShellController, grid: &LakeGrid, to: Point, water: bool) -> usize {
    let path = grid.path(global(c), to, water);
    let grass = path
        .iter()
        .skip(1)
        .filter(|p| grid.kind(**p) == "TALL_GRASS")
        .count();
    for p in path.into_iter().skip(1) {
        step(c, p);
        assert_eq!(
            c.snapshot()
                .expect("native walking or paddling")
                .overworld
                .mode,
            if water {
                MovementMode::Surf
            } else {
                MovementMode::Normal
            }
        );
    }
    release_wild(c);
    grass
}

fn launch(c: &mut VisibleShellController, target: Point) {
    release_wild(c);
    let from = global(c);
    let remaining = active(c).steps_remaining;
    press(c, button(from, target));
    assert_eq!(global(c), from, "water-facing input is not travel");
    assert_eq!(active(c).steps_remaining, remaining);
    for _ in 0..128 {
        if c.snapshot().expect("Surf script").overworld.mode == MovementMode::Surf {
            break;
        }
        press(c, GameButton::A);
    }
    c.wait_frames(60).expect("native Surf animation");
    assert_eq!(global(c), target, "native UsedSurfScript launch");
    assert_eq!(
        active(c).steps_remaining,
        remaining - 1,
        "launch debits once"
    );
    release_wild(c);
}

fn admit(c: &mut VisibleShellController) {
    press(c, GameButton::Up);
    press(c, GameButton::A);
    press(c, GameButton::A);
    assert_eq!(active(c).phase, SafariPhase::Exploring);
    assert_eq!(active(c).steps_remaining, 500);
    assert_eq!(c.shell.shell.session().state().money, 1000);
}

#[test]
#[ignore = "requires enabled current-parent CRYSTAL_INTERIOR_PACK Safari review"]
fn safari_paid_lake_portages_cabin_seams_and_surf_continue() {
    let (root, runtime, mut c) = controller_fixture();
    let mut original_party = c.snapshot().expect("regional partner").party;
    let grid = LakeGrid::new(&runtime);
    // Initial partner/badge/money/divider setup ends before the ranger input.
    admit(&mut c);
    travel(&mut c, &grid, (3, 90), false);
    launch(&mut c, (3, 89));
    save(&mut c, "paid-first-surf");
    c = continue_save(root.clone(), runtime.clone(), &mut c);
    assert_eq!(
        c.snapshot().expect("Surf Continue").overworld.mode,
        MovementMode::Surf
    );
    step(&mut c, (3, 90));
    assert!(
        travel(&mut c, &grid, (13, 92), false) >= 4,
        "reed portage must cross grass"
    );
    launch(&mut c, (14, 92));
    save(&mut c, "paid-reed-pocket");
    // Reed Pocket is deliberately separate water. Return by its grassy
    // portage before taking the connected main-lake paddle to Dorothy.
    step(&mut c, (13, 92));
    assert!(travel(&mut c, &grid, (3, 90), false) >= 4);
    launch(&mut c, (3, 89));
    travel(&mut c, &grid, (63, 75), true);
    step(&mut c, (64, 75));
    assert_eq!(
        c.snapshot().expect("east Surf seam").overworld.map_name,
        "MnBoundarySafariC01R01"
    );
    step(&mut c, (63, 75));
    assert_eq!(
        c.snapshot().expect("west Surf seam").overworld.map_name,
        "MnBoundarySafariC00R01"
    );
    travel(&mut c, &grid, (55, 58), true);
    step(&mut c, (56, 58));
    assert!(
        travel(&mut c, &grid, (85, 61), false) >= 6,
        "island portage must cross grass"
    );
    save(&mut c, "paid-island-portage");
    c = continue_save(root.clone(), runtime.clone(), &mut c);
    travel(&mut c, &grid, (71, 56), false);
    let before = active(&mut c).steps_remaining;
    for _ in 0..8 {
        press(&mut c, GameButton::Up);
        if c.snapshot().expect("real cabin door").overworld.map_name == "MnBoundaryRootBeerCabin" {
            break;
        }
    }
    assert_eq!(
        c.snapshot().expect("native cabin").overworld.map_name,
        "MnBoundaryRootBeerCabin"
    );
    assert_eq!(
        active(&mut c).steps_remaining,
        before - 1,
        "door step counts once"
    );
    save(&mut c, "paid-cabin");
    c = continue_save(root.clone(), runtime.clone(), &mut c);
    // The actual passive FLOOR arrival is beside the two native carpets.
    let tile = c.snapshot().expect("native cabin arrival").overworld.tile;
    assert_eq!((tile.x, tile.y), (5, 6));
    let remaining = active(&mut c).steps_remaining;
    press(&mut c, GameButton::Down);
    if c.snapshot().expect("cabin carpet").overworld.map_name == "MnBoundaryRootBeerCabin" {
        press(&mut c, GameButton::Down);
    }
    assert_eq!(global(&mut c), (71, 56), "real cabin return");
    assert!(
        active(&mut c).steps_remaining < remaining,
        "carpet travel debits"
    );
    travel(&mut c, &grid, (85, 61), false);
    launch(&mut c, (86, 61));
    save(&mut c, "paid-second-lake");
    c = continue_save(root, runtime, &mut c);
    let after = c.snapshot().expect("unchanged Surf partner").party;
    assert_eq!(after.slots.len(), original_party.slots.len());
    for (expected, actual) in original_party.slots.iter_mut().zip(&after.slots) {
        // Ordinary walking may earn native happiness. Safari Run must preserve
        // every other partner field, including HP, status, level and move PP.
        assert!(actual.pokemon.happiness >= expected.pokemon.happiness);
        expected.pokemon.happiness = actual.pokemon.happiness;
    }
    assert_eq!(after, original_party);
    assert_eq!(active(&mut c).balls_remaining, 30);
    assert_eq!(c.shell.shell.session().state().money, 1000);
}

#[test]
#[ignore = "requires enabled current-parent CRYSTAL_INTERIOR_PACK Safari review"]
fn safari_paid_five_hundred_real_steps_end_once_after_continue() {
    let (root, runtime, mut c) = controller_fixture();
    let zone = runtime.data().safari_zone.as_ref().expect("habitat config");
    let grid = LakeGrid::new(&runtime);
    admit(&mut c);
    let start = global(&mut c);
    let neighbor = [
        (start.0 - 1, start.1),
        (start.0 + 1, start.1),
        (start.0, start.1 - 1),
        (start.0, start.1 + 1),
    ]
    .into_iter()
    .find(|p| grid.kind(*p) == "FLOOR" && !grid.blocked.contains(p))
    .expect("two safe court tiles");
    for _ in 0..500 {
        let target = if global(&mut c) == start {
            neighbor
        } else {
            start
        };
        step(&mut c, target);
    }
    let expired = active(&mut c);
    assert_eq!(expired.steps_remaining, 0);
    assert_eq!(
        expired.phase,
        SafariPhase::Ending(SafariEndReason::StepsExhausted)
    );
    save(&mut c, "paid-step-limit");
    c = continue_save(root.clone(), runtime.clone(), &mut c);
    let position = global(&mut c);
    press(&mut c, GameButton::Left);
    assert_eq!(global(&mut c), position, "ending owns movement");
    press(&mut c, GameButton::A);
    let returned = c.snapshot().expect("real safe return");
    assert_eq!(returned.overworld.map_name, zone.return_to.map_name);
    assert_eq!(returned.overworld.tile, zone.return_to.tile);
    assert!(returned.safari.is_none());
    assert_eq!(returned.overworld.mode, MovementMode::Normal);
    assert_eq!(c.shell.shell.session().state().money, 1000);
    save(&mut c, "paid-step-limit-return");
    c = continue_save(root, runtime, &mut c);
    assert!(c.snapshot().expect("returned Continue").safari.is_none());
    assert_eq!(c.shell.shell.session().state().money, 1000);
}

#[test]
#[ignore = "requires enabled CRYSTAL_INTERIOR_PACK and reached CRYSTAL_SAFARI_ENDING_SAVE"]
fn safari_paid_ending_keeps_field_instead_of_empty_battle_canvas() {
    use std::path::PathBuf;

    use bevy::prelude::{Sprite, Vec2, With};

    use super::super::super::{
        BattleCommandMarker, BattlePortraitMarker, PLAYFIELD_HEIGHT, PLAYFIELD_WIDTH,
        SceneDialogMarker,
    };

    let (root, runtime, _) = super::load();
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_SAFARI_ENDING_SAVE")
            .expect("actual reached step-exhaustion save"),
    );
    let mut controller = VisibleShellController::load_save(root, runtime, path, None)
        .expect("native ending Continue");
    assert!(matches!(
        active(&mut controller).phase,
        SafariPhase::Ending(_)
    ));
    let mut app = super::super::battle_render_regression_app(controller.shell);
    for _ in 0..4 {
        app.update();
    }
    let world = app.world_mut();
    let shell = world.resource::<super::super::super::BevyRuntimeShell>();
    assert!(shell.last_error.is_none());
    assert!(!super::super::super::safari_visible::battle_active(shell));
    assert!(
        world
            .query_filtered::<&Sprite, With<SceneDialogMarker>>()
            .iter(world)
            .count()
            > 1,
        "return text and frame share the canonical field presenter"
    );
    assert_eq!(
        world
            .query_filtered::<&Sprite, With<BattlePortraitMarker>>()
            .iter(world)
            .count(),
        0,
        "no battle portrait over field return dialogue"
    );
    assert!(
        !world
            .query_filtered::<&Sprite, With<BattleCommandMarker>>()
            .iter(world)
            .any(|sprite| sprite.custom_size == Some(Vec2::new(PLAYFIELD_WIDTH, PLAYFIELD_HEIGHT))),
        "return dialogue must not cover the map with a white battle LCD"
    );
    let mut controller = VisibleShellController {
        shell: world
            .remove_resource::<super::super::super::BevyRuntimeShell>()
            .expect("native field owner"),
    };
    press(&mut controller, GameButton::A);
    assert!(
        controller
            .snapshot()
            .expect("actual return")
            .safari
            .is_none()
    );
    app.insert_resource(controller.shell);
    for _ in 0..4 {
        app.update();
    }
    let world = app.world_mut();
    assert!(
        world
            .resource::<super::super::super::BevyRuntimeShell>()
            .last_error
            .is_none()
    );
    assert_eq!(
        world
            .query_filtered::<&Sprite, With<SceneDialogMarker>>()
            .iter(world)
            .count(),
        0,
        "return must remove the complete field dialogue"
    );
}
