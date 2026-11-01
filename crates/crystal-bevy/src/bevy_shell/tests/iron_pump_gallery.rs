//! Local chapter setup, then real pressure travel, guards, feed choices and shafts.
use std::path::{Path, PathBuf};

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::interior_life;
use super::northstar_ice_lair::{choice, event, path, talk, walk};
use super::r10_gallery::{press, settle};
use super::winona::{at, finish, load, travel};

const ROOM: &str = "MnIronPumpGallery";
const FEED: &str = "EVENT_MN_DRILL_FEED_ISOLATED";

fn output() -> PathBuf {
    let p =
        PathBuf::from(std::env::var_os("CRYSTAL_IRON_PUMP_REVIEW_DIR").expect("fresh pump output"));
    assert!(!p.exists(), "retain every prior review");
    std::fs::create_dir_all(&p).expect("pump review folder");
    p
}

fn fixture(root: &AssetRoot, r: &CrystalRuntime) -> VisibleShellController {
    let mut c = interior_life::fixture(root, r, "MnIronDrillApproach", (24, 5));
    let state = c.shell.shell.session_mut().state_mut();
    for species in ["VOYAGEAR", "CINDORE", "FORTIGARD", "MUDMENDER"] {
        let mut p = r
            .data()
            .create_pokemon(species, 55, Dv::from_non_hp(8, 9, 8, 9))
            .expect("original regional partner");
        assert!(
            p.moves.iter().all(|m| m.name != "SURF"),
            "dry course needs no Surf partner"
        );
        p.item = Some("SMOKE_BALL".into());
        assert!(state.storage.party.add_pokemon(p));
    }
    state.sync_party_from_storage();
    state.badges.johto = [true; 8];
    for flag in &crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS[..8] {
        state
            .flags
            .set_engine_flag(flag, true)
            .expect("declared prior badge access");
    }
    state
        .flags
        .set_event_flag("EVENT_MN_DEPOT_FOREMAN_DEFEATED", true)
        .expect("declared completed foreman chapter");
    for (item, count) in [("HYPER_POTION", 25), ("MAX_REPEL", 10)] {
        assert!(
            state
                .bag
                .add_item(&r.data().items[item], count)
                .expect("declared supplies")
        );
    }
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
    save: &Path,
) -> VisibleShellController {
    c.save(save).expect("reached checkpoint");
    let mut next =
        VisibleShellController::load_save(root.clone(), r.clone(), save.to_path_buf(), None)
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
        .expect("original active shaft");
    let target = r
        .data()
        .map_name_for_constant(&port.target_map_constant)
        .expect("paired room");
    let arrival = r.data().maps[&target]
        .events
        .warps
        .iter()
        .find(|w| i16::try_from(w.index).expect("native index") == port.target_warp_id)
        .expect("safe passive record");
    walk(c, r, (port.x as i16, port.y as i16 + 1));
    for attempt in 0..12 {
        press(c, GameButton::Up);
        let state = c.snapshot().expect("shaft input");
        if state.overworld.map_name == target {
            break;
        }
        if state.ui.text_window_open || state.battle.is_some() || c.has_pending_script_work() {
            // The original west guard sees the core return ladder. Resolve
            // that actual battle before stepping off and retrying the shaft.
            assert_eq!(from, "MnIronDrillCore", "unexpected shaft interruption");
            finish(c);
            settle(c, "native core shaft guard");
            assert!(event(c, "EVENT_MN_LODE_ARC_CORE_WEST_DEFEATED"));
            if c.snapshot()
                .expect("guard yielded shaft")
                .overworld
                .map_name
                == target
            {
                break;
            }
            walk(c, r, (port.x as i16, port.y as i16 + 1));
        }
        assert!(attempt < 11, "shaft input never reached its paired arrival");
    }
    at(c, &target, (arrival.x as i16, arrival.y as i16));
    c.wait_frames(40).expect("passive arrival safety");
    at(c, &target, (arrival.x as i16, arrival.y as i16));
}

fn gates(c: &VisibleShellController, expected: u16) {
    let live = &c.shell.shell.session().overworld().map;
    for i in [3 * 16 + 12, 8 * 16 + 7] {
        assert_eq!(
            live.metatile_ids[i], expected,
            "actual feed-operated opening"
        );
    }
}

fn later_unchanged(c: &VisibleShellController) {
    for flag in [
        "EVENT_MN_DRILL_TEST_WITNESSED",
        "EVENT_MN_REED_MINE_DEFEATED",
        "EVENT_MN_DRILL_STOPPED",
        "EVENT_MN_MINE_INSTRUMENT_RECOVERED",
    ] {
        assert!(!event(c, flag), "isolating feed cannot grant {flag}");
    }
}

#[test]
#[ignore = "requires current CRYSTAL_WINONA_TEST_PACK, fresh CRYSTAL_IRON_PUMP_REVIEW_DIR and CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR; declared local chapter access"]
fn iron_pump_controller_native_guards_feed_refusal_shortcut_two_climbs_shafts_and_continue() {
    let out = output();
    let (root, r) = load();
    let mut c = fixture(&root, &r);
    c.save(out.join("initial-mouth.crystalsave"))
        .expect("pre-input local setup");
    assert!(
        !event(&c, FEED)
            && !event(&c, "EVENT_MN_IRON_HAUL_PRESSURE_OFF")
            && !event(&c, "EVENT_MN_IRON_VENT_PRESSURE_OFF")
    );
    super::northstar_iron_workings::enter_gallery(&mut c, &r, &root);
    at(&mut c, ROOM, (4, 19));
    gates(&c, 0x41);
    assert!(
        path(&mut c, &r, (25, 4)).is_none(),
        "drill shaft sealed before feed isolation"
    );
    let before = path(&mut c, &r, (25, 18))
        .expect("winding maintenance route")
        .len();
    assert!(before >= 40);
    c.save(out.join("reached-gallery.crystalsave"))
        .expect("actual pressure route");
    walk(&mut c, &r, (12, 8));
    finish(&mut c);
    settle(&mut c, "pump guard victory");
    assert!(event(&c, "EVENT_MN_LODE_ARC_PUMP_DEFEATED"));
    c.save(out.join("pump-guard-won.crystalsave"))
        .expect("actual guard battle");
    walk(&mut c, &r, (25, 18));
    finish(&mut c);
    settle(&mut c, "intake guard victory");
    assert!(event(&c, "EVENT_MN_LODE_ARC_INTAKE_DEFEATED"));
    c.save(out.join("both-guards-won.crystalsave"))
        .expect("actual two guard battles");
    walk(&mut c, &r, (16, 17));
    press(&mut c, GameButton::Left);
    at(&mut c, ROOM, (16, 17));
    walk(&mut c, &r, (24, 8));
    press(&mut c, GameButton::Up);
    at(&mut c, ROOM, (24, 8));
    walk(&mut c, &r, (25, 18));
    talk(&mut c, GameButton::Up);
    choice(&mut c, false);
    finish(&mut c);
    assert!(!event(&c, FEED));
    c = continued(&mut c, &root, &r, &out.join("declined-feed.crystalsave"));
    gates(&c, 0x41);
    later_unchanged(&c);
    let closed_return = path(&mut c, &r, (4, 19))
        .expect("ordinary closed return")
        .len();
    talk(&mut c, GameButton::Up);
    choice(&mut c, true);
    finish(&mut c);
    assert!(event(&c, FEED));
    gates(&c, 13);
    let open_return = path(&mut c, &r, (4, 19)).expect("new shorter return").len();
    assert!(
        open_return + 12 <= closed_return,
        "isolation must open a useful crosscut"
    );
    std::fs::write(
        out.join("return-lengths.json"),
        format!("{{\"closed\":{closed_return},\"open\":{open_return}}}\n"),
    )
    .expect("route evidence");
    c = continued(&mut c, &root, &r, &out.join("isolated-feed.crystalsave"));
    gates(&c, 13);
    walk(&mut c, &r, (16, 17));
    for x in [15, 14, 13] {
        travel(&mut c, GameButton::Left, ROOM, (x, 17));
    }
    for x in [6, 16] {
        walk(&mut c, &r, (x, 10));
        for y in (7..10).rev() {
            travel(&mut c, GameButton::Up, ROOM, (x, y));
        }
        for y in 8..=10 {
            travel(&mut c, GameButton::Down, ROOM, (x, y));
        }
    }
    walk(&mut c, &r, (24, 8));
    for y in [7, 6, 5] {
        travel(&mut c, GameButton::Up, ROOM, (24, y));
    }
    cross(&mut c, &r, ROOM, 3);
    cross(&mut c, &r, "MnIronDrillCore", 1);
    gates(&c, 13);
    cross(&mut c, &r, ROOM, 1);
    cross(&mut c, &r, "MnIronSump", 7);
    gates(&c, 13);
    c = continued(
        &mut c,
        &root,
        &r,
        &out.join("shafts-and-climbs.crystalsave"),
    );
    gates(&c, 13);
    later_unchanged(&c);
    super::northstar_iron_workings::leave_gallery(&mut c, &r);
    at(&mut c, "MnIronDrillApproach", (24, 5));
    later_unchanged(&c);
    c.save(out.join("final-mouth-return.crystalsave"))
        .expect("native return to mine mouth");
}
