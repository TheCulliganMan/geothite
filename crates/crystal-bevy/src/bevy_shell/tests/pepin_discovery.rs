//! Declared local river access; subsequent observations, Surf and battles use input.
use std::collections::{BTreeMap, VecDeque};

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use crystal_core::world::movement::MovementMode;
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::authored_route_fixture::{cleanup_reached_save, reached_save_path};
use super::interior_life::fixture;
use super::northstar_ice_lair::{choice, event, walk};
use super::r10_gallery::settle;
use super::winona::{at, load, press, travel};

const BANK: &str = "MnMississippiLowerWoodsC00R01";
const REEDS: &str = "MnMississippiLowerWoodsC01R01";
const INLET: &str = "MnMississippiLowerWoodsC01R00";
const ACTOR: &str = "MN_PEPIN_PEPIVERN";

fn flag(name: &str) -> String {
    format!("EVENT_MN_PEPIN_{name}")
}

fn setup(
    map: &str,
    point: (i16, i16),
    prior: &[&str],
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, runtime) = load();
    let mut c = fixture(&root, &runtime, map, point);
    assert!(
        runtime.data().pokemon["GILLPUP"]
            .tmhm_learnset
            .iter()
            .any(|name| name == "SURF")
    );
    let mut partner = runtime
        .data()
        .create_pokemon("GILLPUP", 60, Dv::default())
        .expect("legal native Surf partner");
    partner.moves = vec![LearnedMove {
        name: "SURF".into(),
        current_pp: runtime.data().moves["SURF"].pp,
        pp_ups: 0,
    }];
    partner.item = Some("SMOKE_BALL".into());
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    state.badges.johto[3] = true;
    state
        .flags
        .set_engine_flag(BADGE_ENGINE_FLAGS[3], true)
        .expect("declared prior Surf badge");
    for name in prior {
        state
            .flags
            .set_event_flag(&flag(name), true)
            .expect("declared prior observation fixture");
    }
    assert!(
        state
            .bag
            .add_item(&runtime.data().items["MASTER_BALL"], 1)
            .expect("declared capture resource")
    );
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.wait_frames(64)
        .expect("initial native river presentation");
    // No later position, mode, party, bag, flag, HP, PP or RNG writes.
    (root, runtime, c)
}

fn continued(
    c: &mut VisibleShellController,
    root: &AssetRoot,
    r: &CrystalRuntime,
    name: &str,
) -> VisibleShellController {
    let path = reached_save_path(name);
    c.save(&path).expect("production river save");
    let loaded = VisibleShellController::load_save(root.clone(), r.clone(), path.clone(), None)
        .expect("same-pack river Continue");
    cleanup_reached_save(&path);
    loaded
}

fn visible(c: &mut VisibleShellController) -> bool {
    c.snapshot()
        .expect("native river actors")
        .visible_objects
        .iter()
        .any(|object| object.object_identifier.as_deref() == Some(ACTOR))
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with native Pepin discovery"]
fn pepin_controller_land_observations_wait_for_acknowledgment_and_continue() {
    let (root, r, mut c) = setup(BANK, (22, 11), &[]);
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    assert!(c.snapshot().expect("held sketch page").ui.text.is_some());
    assert!(!event(&c, &flag("DOCK_SKETCH_READ")));
    press(&mut c, GameButton::Right);
    at(&mut c, BANK, (22, 11));
    c = continued(&mut c, &root, &r, "pepin-held-sketch.crystalsave");
    assert!(!event(&c, &flag("DOCK_SKETCH_READ")));
    settle(&mut c, "acknowledged dock sketch");
    assert!(event(&c, &flag("DOCK_SKETCH_READ")));
    walk(&mut c, &r, (23, 10));
    press(&mut c, GameButton::Right);
    press(&mut c, GameButton::A);
    assert!(!event(&c, &flag("EDDY_OBSERVED")));
    settle(&mut c, "acknowledged dry-land eddy");
    assert!(event(&c, &flag("EDDY_OBSERVED")));
    assert!(!event(&c, &flag("REEDS_OBSERVED")));
    assert!(!event(&c, &flag("SAFE_INLET_IDENTIFIED")));
    c = continued(&mut c, &root, &r, "pepin-land-observations.crystalsave");
    assert!(event(&c, &flag("DOCK_SKETCH_READ")) && event(&c, &flag("EDDY_OBSERVED")));
    assert_eq!(
        c.snapshot().expect("dry viewpoint").overworld.mode,
        MovementMode::Normal
    );
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with native Pepin discovery"]
fn pepin_controller_reeds_require_actual_surf_and_operator_acknowledgment() {
    let (root, r, mut c) = setup(REEDS, (50, 19), &["DOCK_SKETCH_READ", "EDDY_OBSERVED"]);
    press(&mut c, GameButton::Down);
    press(&mut c, GameButton::A);
    settle(&mut c, "land viewpoint needs Surf");
    assert!(!event(&c, &flag("REEDS_OBSERVED")));
    walk(&mut c, &r, (50, 19));
    super::minneapolis_falls::mount(&mut c, GameButton::Left, REEDS, (49, 19));
    travel(&mut c, GameButton::Down, REEDS, (49, 20));
    press(&mut c, GameButton::Right);
    press(&mut c, GameButton::A);
    assert!(!event(&c, &flag("REEDS_OBSERVED")));
    c = continued(&mut c, &root, &r, "pepin-held-surf-observation.crystalsave");
    assert_eq!(
        c.snapshot().expect("continued Surf").overworld.mode,
        MovementMode::Surf
    );
    settle(&mut c, "acknowledged Surf-only sighting");
    assert!(event(&c, &flag("REEDS_OBSERVED")));
    travel(&mut c, GameButton::Up, REEDS, (49, 19));
    travel(&mut c, GameButton::Right, REEDS, (50, 19));
    assert_eq!(
        c.snapshot().expect("native dismount").overworld.mode,
        MovementMode::Normal
    );
    walk(&mut c, &r, (51, 16));
    press(&mut c, GameButton::Left);
    press(&mut c, GameButton::A);
    assert!(!event(&c, &flag("SAFE_INLET_IDENTIFIED")));
    c = continued(
        &mut c,
        &root,
        &r,
        "pepin-held-operator-comparison.crystalsave",
    );
    settle(&mut c, "operator identifies still inlet");
    assert!(event(&c, &flag("SAFE_INLET_IDENTIFIED")));
    assert!(!event(&c, &flag("CAUGHT")));
}

// Plan ordinary current-map WATER only; each edge still uses native movement.
pub(super) fn paddle(c: &mut VisibleShellController, r: &CrystalRuntime, target: (i16, i16)) {
    let snapshot = c.snapshot().expect("current Surf state");
    assert_eq!(snapshot.overworld.mode, MovementMode::Surf);
    let name = snapshot.overworld.map_name;
    let start = (snapshot.overworld.tile.x, snapshot.overworld.tile.y);
    let map = &r.data().maps[&name];
    let collision = &r.data().tilesets[&map.attributes.tileset_name].collision;
    let mut prior = BTreeMap::from([(start, (start, GameButton::A))]);
    let mut queue = VecDeque::from([start]);
    while let Some((x, y)) = queue.pop_front() {
        if (x, y) == target {
            break;
        }
        for (next, button) in [
            ((x - 1, y), GameButton::Left),
            ((x + 1, y), GameButton::Right),
            ((x, y - 1), GameButton::Up),
            ((x, y + 1), GameButton::Down),
        ] {
            if next.0 < 0
                || next.1 < 0
                || next.0 >= map.attributes.width as i16 * 2
                || next.1 >= map.attributes.height as i16 * 2
                || prior.contains_key(&next)
            {
                continue;
            }
            if snapshot
                .visible_objects
                .iter()
                .any(|object| (object.x as i16, object.y as i16) == next)
            {
                continue;
            }
            let before = (x, y);
            let x = next.0 as usize;
            let y = next.1 as usize;
            let block = map.blocks[y / 2 * usize::from(map.attributes.width) + x / 2];
            if collision[&format!("{block:02x}")][y % 2 * 2 + x % 2] == "WATER" {
                prior.insert(next, (before, button));
                queue.push_back(next);
            }
        }
    }
    assert!(
        prior.contains_key(&target),
        "connected native water {name} {start:?} -> {target:?}"
    );
    let mut course = Vec::new();
    let mut point = target;
    while point != start {
        let (before, button) = prior[&point];
        course.push((point, button));
        point = before;
    }
    for (point, button) in course.into_iter().rev() {
        for _ in 0..8 {
            if c.snapshot().expect("river encounter").battle.is_some() {
                super::rochester::flee(c);
                settle(c, "ordinary river escape");
            }
            let s = c.snapshot().expect("river step");
            if (s.overworld.tile.x, s.overworld.tile.y) == point {
                break;
            }
            press(c, button);
        }
        if c.snapshot()
            .expect("river encounter after movement")
            .battle
            .is_some()
        {
            super::rochester::flee(c);
            settle(c, "ordinary river escape");
        }
        at(c, &name, point);
    }
}

fn ask(c: &mut VisibleShellController, yes: bool) {
    press(c, GameButton::Up);
    press(c, GameButton::A);
    choice(c, yes);
}

fn capture(c: &mut VisibleShellController) {
    for _ in 0..256 {
        let snapshot = c.snapshot().expect("native Pepivern capture menus");
        if event(c, &flag("CAUGHT")) && snapshot.battle.is_none() {
            settle(c, "capture continuation");
            return;
        }
        if let Some(choice) = &c.shell.pending_name_choice {
            if choice.nickname_pages.len() > 1 {
                press(c, GameButton::A);
            } else {
                press(c, GameButton::Down);
                press(c, GameButton::A);
            }
            continue;
        }
        let button = if c.shell.pokedex_menu_open
            || (snapshot.ui.menu.is_none() && snapshot.ui.text.is_some())
            || c.shell.field_pack_action_cursor.is_some()
            || c.shell.ball_cursor.is_some()
        {
            GameButton::A
        } else if c.shell.bag_cursor.is_some() {
            GameButton::Right
        } else if let Some(cursor) = &c.shell.battle_action_cursor {
            match cursor.option_index {
                0 | 1 => GameButton::Down,
                3 => GameButton::Left,
                _ => GameButton::A,
            }
        } else {
            GameButton::A
        };
        press(c, button);
    }
    panic!("native Pepivern capture did not finish");
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with native Pepin discovery"]
fn pepin_controller_refusal_run_hidden_continue_physical_retry_and_capture() {
    let (root, r, mut c) = setup(INLET, (33, 32), &["SAFE_INLET_IDENTIFIED"]);
    super::minneapolis_falls::mount(&mut c, GameButton::Right, INLET, (34, 32));
    // The fixture declares a comparison earned elsewhere after its initial
    // placement callback has already run. Enter the inlet physically so its
    // normal OBJECTS callback sees that prior comparison, just as in play.
    paddle(&mut c, &r, (0, 30));
    travel(
        &mut c,
        GameButton::Left,
        "MnMississippiLowerWoodsC00R00",
        (63, 30),
    );
    travel(&mut c, GameButton::Right, INLET, (0, 30));
    paddle(&mut c, &r, (24, 22));
    assert!(visible(&mut c));
    let bag = c.shell.shell.session().state().bag.clone();
    ask(&mut c, false);
    settle(&mut c, "voluntary observation without battle");
    assert!(visible(&mut c));
    assert!(c.shell.shell.session().state().bag == bag);
    ask(&mut c, true);
    for _ in 0..128 {
        if c.shell.battle_action_cursor.is_some() {
            break;
        }
        press(&mut c, GameButton::A);
    }
    assert!(
        c.snapshot()
            .expect("actual Pepivern encounter")
            .battle
            .is_some()
    );
    super::rochester::flee(&mut c);
    settle(&mut c, "optional Pepivern escape");
    assert!(!event(&c, &flag("CAUGHT")));
    assert!(!visible(&mut c));
    c = continued(&mut c, &root, &r, "pepin-withdrawn-visit.crystalsave");
    assert!(!visible(&mut c));
    paddle(&mut c, &r, (0, 30));
    travel(
        &mut c,
        GameButton::Left,
        "MnMississippiLowerWoodsC00R00",
        (63, 30),
    );
    travel(&mut c, GameButton::Right, INLET, (0, 30));
    paddle(&mut c, &r, (24, 22));
    assert!(visible(&mut c));
    ask(&mut c, true);
    capture(&mut c);
    assert!(event(&c, &flag("CAUGHT")));
    assert!(!visible(&mut c));
    let state = c.shell.shell.session().state();
    assert!(state.pokedex.has_caught("PEPIVERN"));
    assert!(
        state
            .storage
            .party
            .pokemon
            .iter()
            .flatten()
            .any(|pokemon| pokemon.species.id == "PEPIVERN")
    );
    assert_eq!(state.bag.quantity(&r.data().items["MASTER_BALL"]), 0);
    c = continued(&mut c, &root, &r, "pepin-caught.crystalsave");
    assert!(!visible(&mut c));
    assert!(event(&c, &flag("CAUGHT")));
}
