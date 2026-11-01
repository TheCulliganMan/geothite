//! Declared local fixtures; all subsequent travel, valve choices, flags and battles use input.
use std::path::{Path, PathBuf};

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::interior_life;
use super::northstar_ice_lair::{choice, event, path, talk, walk};
use super::r10_gallery::{press, settle};
use super::winona::{at, finish, load, travel};

const ROOM: &str = "MnIronVentilation";
const HAUL: &str = "EVENT_MN_IRON_HAUL_PRESSURE_OFF";
const VENT: &str = "EVENT_MN_IRON_VENT_PRESSURE_OFF";

fn output(name: &str) -> PathBuf {
    let p = PathBuf::from(std::env::var_os("CRYSTAL_IRON_VENT_REVIEW_DIR").expect("fresh output"))
        .join(name);
    assert!(!p.exists(), "retain prior evidence");
    std::fs::create_dir_all(&p).expect("review folder");
    p
}

fn fixture(
    root: &AssetRoot,
    r: &CrystalRuntime,
    map: &str,
    point: (i16, i16),
) -> VisibleShellController {
    let mut c = interior_life::fixture(root, r, map, point);
    let state = c.shell.shell.session_mut().state_mut();
    assert!(
        r.data().pokemon["VOYAGEAR"]
            .tmhm_learnset
            .iter()
            .any(|m| m == "SURF")
    );
    for species in ["VOYAGEAR", "CINDORE", "FORTIGARD", "MUDMENDER"] {
        let mut p = r
            .data()
            .create_pokemon(species, 55, Dv::from_non_hp(8, 9, 8, 9))
            .expect("original regional partner");
        if species == "VOYAGEAR" {
            p.moves[0] = LearnedMove {
                name: "SURF".into(),
                current_pp: r.data().moves["SURF"].pp,
                pp_ups: 0,
            };
            p.item = Some("SMOKE_BALL".into());
        }
        assert!(state.storage.party.add_pokemon(p));
    }
    state.sync_party_from_storage();
    state.badges.johto = [true; 8];
    for flag in &crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS[..8] {
        state
            .flags
            .set_engine_flag(flag, true)
            .expect("declared badge fixture");
    }
    for (item, n) in [("HYPER_POTION", 10), ("MAX_REPEL", 10), ("OLD_ROD", 1)] {
        assert!(
            state
                .bag
                .add_item(&r.data().items[item], n)
                .expect("declared supplies")
        );
    }
    state.registered_key_item = Some("OLD_ROD".into());
    state.repel_steps_remaining = 250;
    state.active_repel_item = Some("MAX_REPEL".into());
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.set_runtime_journal_enabled(false);
    c
}

fn continued(
    c: &mut VisibleShellController,
    root: &AssetRoot,
    r: &CrystalRuntime,
    path: &Path,
) -> VisibleShellController {
    c.save(path).expect("actual reached checkpoint");
    let mut next =
        VisibleShellController::load_save(root.clone(), r.clone(), path.to_path_buf(), None)
            .expect("real Continue");
    next.set_runtime_journal_enabled(false);
    next
}

fn cross(c: &mut VisibleShellController, r: &CrystalRuntime, from: &str, index: u16) {
    let port = r.data().maps[from]
        .events
        .warps
        .iter()
        .find(|w| w.index == index)
        .expect("active shaft");
    let target_name = r
        .data()
        .map_name_for_constant(&port.target_map_constant)
        .expect("paired room");
    let arrival = r.data().maps[&target_name]
        .events
        .warps
        .iter()
        .find(|w| i16::try_from(w.index).expect("native index") == port.target_warp_id)
        .expect("paired passive shaft");
    walk(c, r, (port.x as i16, port.y as i16 + 1));
    travel(
        c,
        GameButton::Up,
        &target_name,
        (arrival.x as i16, arrival.y as i16),
    );
    c.wait_frames(40).expect("passive shaft safety");
    at(c, &target_name, (arrival.x as i16, arrival.y as i16));
}

fn pair(c: &mut VisibleShellController, r: &CrystalRuntime, index: u16) {
    let port = r.data().maps[ROOM]
        .events
        .warps
        .iter()
        .find(|w| w.index == index)
        .expect("ventilation shaft");
    let target = r
        .data()
        .map_name_for_constant(&port.target_map_constant)
        .expect("target");
    let back = u16::try_from(port.target_warp_id).expect("paired index") - 1;
    cross(c, r, ROOM, index);
    cross(c, r, &target, back);
}

fn later_flags_unchanged(c: &VisibleShellController) {
    for flag in [
        "EVENT_MN_DRILL_FEED_ISOLATED",
        "EVENT_MN_DRILL_TEST_WITNESSED",
        "EVENT_MN_DRILL_STOPPED",
        "EVENT_MN_MINE_INSTRUMENT_RECOVERED",
    ] {
        assert!(
            !event(c, flag),
            "ventilation cannot grant later progress: {flag}"
        );
    }
}

#[test]
#[ignore = "requires current CRYSTAL_WINONA_TEST_PACK and fresh CRYSTAL_IRON_VENT_REVIEW_DIR; local chapter setup"]
fn iron_ventilation_controller_native_pressure_refusal_guards_fin_climbs_four_pairs_and_continue() {
    let out = output("journey");
    let (root, r) = load();
    let mut c = fixture(&root, &r, "MnIronDrillApproach", (24, 5));
    c.save(out.join("initial-mouth.crystalsave"))
        .expect("declared initial local setup");
    assert!(!event(&c, HAUL) && !event(&c, VENT));
    cross(&mut c, &r, "MnIronDrillApproach", 3);
    walk(&mut c, &r, (5, 16));
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    cross(&mut c, &r, "MnIronHaulage", 9);
    assert!(
        path(&mut c, &r, (13, 16)).is_none(),
        "east arrival cannot bypass the live crosscut"
    );
    walk(&mut c, &r, (24, 15));
    talk(&mut c, GameButton::Right);
    assert!(c.snapshot().expect("Vern speaks").ui.text_window_open);
    finish(&mut c);
    walk(&mut c, &r, (28, 8));
    if !event(&c, "EVENT_MN_LODE_ARC_SHIFT_DEFEATED") {
        talk(&mut c, GameButton::Right);
        assert!(super::rochester_earned::try_battle(
            &mut c,
            &r,
            "MN_LODE_ARC_SHIFT",
            "EVENT_MN_LODE_ARC_SHIFT_DEFEATED"
        ));
    }
    finish(&mut c);
    settle(&mut c, "native shift guard victory");
    assert!(event(&c, "EVENT_MN_LODE_ARC_SHIFT_DEFEATED"));
    cross(&mut c, &r, ROOM, 7);
    cross(&mut c, &r, "MnIronSump", 5);
    walk(&mut c, &r, (29, 12));
    talk(&mut c, GameButton::Up);
    choice(&mut c, true);
    finish(&mut c);
    assert!(event(&c, HAUL) && !event(&c, VENT));
    cross(&mut c, &r, "MnIronHaulage", 3);
    cross(&mut c, &r, "MnIronSump", 3);
    walk(&mut c, &r, (11, 10));
    if !event(&c, "EVENT_MN_LODE_ARC_VENT_DEFEATED") {
        talk(&mut c, GameButton::Left);
        assert!(super::rochester_earned::try_battle(
            &mut c,
            &r,
            "MN_LODE_ARC_VENT",
            "EVENT_MN_LODE_ARC_VENT_DEFEATED"
        ));
    }
    finish(&mut c);
    settle(&mut c, "native vent guard victory");
    assert!(event(&c, "EVENT_MN_LODE_ARC_VENT_DEFEATED"));
    walk(&mut c, &r, (17, 13));
    assert!(path(&mut c, &r, (26, 21)).is_none());
    for _ in 0..3 {
        press(&mut c, GameButton::Right);
        at(&mut c, ROOM, (17, 13));
    }
    walk(&mut c, &r, (13, 16));
    talk(&mut c, GameButton::Up);
    choice(&mut c, false);
    finish(&mut c);
    assert!(!event(&c, VENT));
    c = continued(&mut c, &root, &r, &out.join("declined-valve.crystalsave"));
    assert!(path(&mut c, &r, (26, 21)).is_none());
    later_flags_unchanged(&c);
    talk(&mut c, GameButton::Up);
    choice(&mut c, true);
    finish(&mut c);
    assert!(event(&c, VENT));
    walk(&mut c, &r, (17, 13));
    for x in [18, 19, 20] {
        travel(&mut c, GameButton::Right, ROOM, (x, 13));
    }
    c = continued(&mut c, &root, &r, &out.join("crosscut-open.crystalsave"));
    assert!(path(&mut c, &r, (4, 21)).is_some());
    for (x, bottom, top) in [(8, 10, 7), (28, 8, 5)] {
        walk(&mut c, &r, (x, bottom));
        for y in (top..bottom).rev() {
            travel(&mut c, GameButton::Up, ROOM, (x, y));
        }
        for y in top + 1..=bottom {
            travel(&mut c, GameButton::Down, ROOM, (x, y));
        }
    }
    // Both real sides of the fin loop are ordinary walkable returns.
    for p in [(5, 13), (5, 17), (9, 17), (9, 13)] {
        walk(&mut c, &r, p);
    }
    assert_eq!(
        interior_life::collision(&r, &r.data().maps[ROOM], (7, 14)),
        Some("WALL")
    );
    for index in [1, 3, 5, 7] {
        pair(&mut c, &r, index);
    }
    c.save(out.join("reached-all-shafts.crystalsave"))
        .expect("actual reached shaft course");
    cross(&mut c, &r, ROOM, 7);
    assert!(
        path(&mut c, &r, (35, 4)).is_some(),
        "real valves unlock the original sump gate"
    );
    cross(&mut c, &r, "MnIronSump", 7);
    cross(&mut c, &r, "MnIronPumpGallery", 1);
    cross(&mut c, &r, "MnIronSump", 5);
    cross(&mut c, &r, "MnIronHaulage", 1);
    at(&mut c, "MnIronDrillApproach", (24, 5));
    later_flags_unchanged(&c);
    c.save(out.join("final-return.crystalsave"))
        .expect("real mine-mouth return");
}
