//! Real mine chapter and eastern witness entries on the native rock core.
use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::interior_life;
use super::northstar_ice_lair::{event, page};
use super::r10_gallery::press;
use super::winona::{at, finish, load};

const CORE: &str = "MnIronDrillCore";
const SEEN: &str = "EVENT_MN_DRILL_TEST_WITNESSED";

fn output(name: &str) -> PathBuf {
    let p =
        PathBuf::from(std::env::var_os("CRYSTAL_IRON_CORE_REVIEW_DIR").expect("fresh core output"))
            .join(name);
    assert!(!p.exists(), "preserve every prior core review");
    std::fs::create_dir_all(&p).expect("core review directory");
    p
}

fn fixture(
    root: &AssetRoot,
    r: &CrystalRuntime,
    map: &str,
    at: (i16, i16),
    prior_core_access: bool,
) -> VisibleShellController {
    let mut c = interior_life::fixture(root, r, map, at);
    let state = c.shell.shell.session_mut().state_mut();
    for species in ["VOYAGEAR", "CINDORE", "FORTIGARD", "MUDMENDER"] {
        let mut p = r
            .data()
            .create_pokemon(species, 55, Dv::from_non_hp(8, 9, 8, 9))
            .expect("active original regional partner");
        assert!(p.moves.iter().all(|m| m.name != "SURF"));
        p.item = Some("SMOKE_BALL".into());
        assert!(state.storage.party.add_pokemon(p));
    }
    state.sync_party_from_storage();
    state.badges.johto = [true; 8];
    for flag in &crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS[..8] {
        state
            .flags
            .set_engine_flag(flag, true)
            .expect("declared regional badge access");
    }
    state
        .flags
        .set_event_flag("EVENT_MN_HEADWATERS_REPORT", true)
        .expect("declared prior regional report");
    if prior_core_access {
        for flag in [
            "EVENT_MN_DEPOT_FOREMAN_DEFEATED",
            "EVENT_MN_IRON_HAUL_PRESSURE_OFF",
            "EVENT_MN_IRON_VENT_PRESSURE_OFF",
            "EVENT_MN_DRILL_FEED_ISOLATED",
        ] {
            state
                .flags
                .set_event_flag(flag, true)
                .expect("declared separate eastern-entry access fixture");
        }
    }
    for (item, count) in [("HYPER_POTION", 25), ("MAX_REPEL", 10)] {
        assert!(
            state
                .bag
                .add_item(&r.data().items[item], count)
                .expect("declared chapter supplies")
        );
    }
    state.repel_steps_remaining = 250;
    state.active_repel_item = Some("MAX_REPEL".into());
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.set_runtime_journal_enabled(false);
    c
}

#[test]
#[ignore = "requires current CRYSTAL_WINONA_TEST_PACK and fresh CRYSTAL_IRON_CORE_REVIEW_DIR/CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR; initial regional access only"]
fn iron_core_controller_native_foreman_pressure_witness_rival_shutdown_crew_and_return() {
    let out = output("chapter");
    let (root, r) = load();
    let mut c = fixture(&root, &r, "MnR09Cave1F", (13, 18), false);
    c.save(out.join("initial-public-mine.crystalsave"))
        .expect("declared initial local access");
    for flag in [
        "EVENT_MN_DEPOT_FOREMAN_DEFEATED",
        "EVENT_MN_IRON_HAUL_PRESSURE_OFF",
        "EVENT_MN_IRON_VENT_PRESSURE_OFF",
        "EVENT_MN_DRILL_FEED_ISOLATED",
        SEEN,
        "EVENT_MN_REED_MINE_DEFEATED",
        "EVENT_MN_DRILL_STOPPED",
        "EVENT_MN_MINE_INSTRUMENT_RECOVERED",
    ] {
        assert!(!event(&c, flag), "chapter progress must use input: {flag}");
    }
    c = super::northstar_drill_lair::play_chapter(c, &root, &r);
    at(&mut c, "MnR09Cave1F", (19, 8));
    for flag in [
        "EVENT_MN_DEPOT_FOREMAN_DEFEATED",
        "EVENT_MN_IRON_HAUL_PRESSURE_OFF",
        "EVENT_MN_IRON_VENT_PRESSURE_OFF",
        "EVENT_MN_DRILL_FEED_ISOLATED",
        SEEN,
        "EVENT_MN_REED_MINE_DEFEATED",
        "EVENT_MN_DRILL_STOPPED",
        "EVENT_MN_MINE_INSTRUMENT_RECOVERED",
    ] {
        assert!(event(&c, flag), "native chapter result: {flag}");
    }
    c.save(out.join("final-public-mine-return.crystalsave"))
        .expect("actual complete chapter return");
}

#[test]
#[ignore = "requires current pack and fresh CRYSTAL_IRON_CORE_REVIEW_DIR; separate initial prior-core-access fixtures"]
fn iron_core_controller_native_eastern_witness_detours_avoid_original_guard() {
    let (root, r) = load();
    for x in [19, 20] {
        let out = output(&format!("east-{x}"));
        let mut c = fixture(&root, &r, CORE, (x, 13), true);
        c.save(out.join("initial-eastern-entry.crystalsave"))
            .expect("declared separate eastern access");
        assert!(!event(&c, SEEN));
        for _ in 0..4 {
            press(&mut c, GameButton::Up);
            if c.snapshot()
                .expect("native entry input")
                .ui
                .text_window_open
                || c.has_pending_script_work()
            {
                break;
            }
        }
        for (label, block) in [
            ("MnNDrillCrewCallText", 0x16),
            ("MnNDrillForemanText", 0x12),
        ] {
            page(&mut c, label);
            at(&mut c, CORE, (9, 9));
            assert!(!event(&c, SEEN));
            assert_eq!(
                c.shell.shell.session().overworld().map.metatile_ids[4 * 12 + 5],
                block
            );
            press(&mut c, GameButton::Left);
            at(&mut c, CORE, (9, 9));
            c.save(out.join(format!("played-frame-{block:02x}.crystalsave")))
                .expect("actual acknowledged machine frame");
        }
        finish(&mut c);
        assert!(event(&c, SEEN));
        for flag in [
            "EVENT_MN_REED_MINE_DEFEATED",
            "EVENT_MN_DRILL_STOPPED",
            "EVENT_MN_MINE_INSTRUMENT_RECOVERED",
        ] {
            assert!(!event(&c, flag), "witness cannot grant {flag}");
        }
        c.save(out.join("witnessed-only.crystalsave"))
            .expect("actual witnessed result");
    }
}
