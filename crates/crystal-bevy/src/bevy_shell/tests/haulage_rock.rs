//! Explicit local party/position setup, followed only by production input.
use std::path::PathBuf;

use crystal_core::input::GameButton;
use crystal_core::models::Dv;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::interior_life;
use super::northstar_ice_lair::{event, path, talk, walk};
use super::r10_gallery::{press, settle};
use super::winona::{at, finish, load, travel};

const ROOM: &str = "MnIronHaulage";
const HAUL: &str = "EVENT_MN_IRON_HAUL_PRESSURE_OFF";
const VENT: &str = "EVENT_MN_IRON_VENT_PRESSURE_OFF";

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK and fresh CRYSTAL_HAULAGE_ROCK_REVIEW_DIR; local fixture only"]
fn haulage_rock_controller_guards_pressure_network_all_five_pairs_and_continue() {
    let out =
        PathBuf::from(std::env::var_os("CRYSTAL_HAULAGE_ROCK_REVIEW_DIR").expect("fresh output"));
    assert!(!out.exists());
    std::fs::create_dir_all(&out).expect("review folder");
    let (root, r) = load();
    let mut c = interior_life::fixture(&root, &r, "MnIronDrillApproach", (24, 5));
    let state = c.shell.shell.session_mut().state_mut();
    for species in ["CINDORE", "VOYAGEAR", "FORTIGARD", "MUDMENDER"] {
        assert!(
            state.storage.party.add_pokemon(
                r.data()
                    .create_pokemon(species, 55, Dv::from_non_hp(8, 9, 8, 9))
                    .expect("original Minnesota local QA partner")
            )
        );
    }
    state.sync_party_from_storage();
    state.badges.johto = [true; 8];
    for flag in &crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS[..8] {
        state
            .flags
            .set_engine_flag(flag, true)
            .expect("declared badge fixture");
    }
    state.repel_steps_remaining = 250;
    state.active_repel_item = Some("MAX_REPEL".into());
    for (item, n) in [("HYPER_POTION", 10), ("MAX_REPEL", 10)] {
        assert!(
            state
                .bag
                .add_item(&r.data().items[item], n)
                .expect("declared supplies")
        );
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.set_runtime_journal_enabled(false);
    c.save(out.join("initial-approach.crystalsave"))
        .expect("initial fixture");
    assert!(!event(&c, HAUL) && !event(&c, VENT));
    walk(&mut c, &r, (25, 6));
    travel(&mut c, GameButton::Up, ROOM, (4, 23));
    assert!(
        path(&mut c, &r, (28, 23)).is_none(),
        "closed shutter prevents a free crosscut"
    );
    walk(&mut c, &r, (10, 10));
    // Both directions on the ordinary native30 climb.
    travel(&mut c, GameButton::Up, ROOM, (10, 9));
    travel(&mut c, GameButton::Up, ROOM, (10, 8));
    travel(&mut c, GameButton::Up, ROOM, (10, 7));
    travel(&mut c, GameButton::Down, ROOM, (10, 8));
    travel(&mut c, GameButton::Down, ROOM, (10, 9));
    travel(&mut c, GameButton::Down, ROOM, (10, 10));
    walk(&mut c, &r, (13, 7));
    talk(&mut c, GameButton::Right);
    if !event(&c, "EVENT_MN_LODE_ARC_WEIGH_DEFEATED") {
        assert!(super::rochester_earned::try_battle(
            &mut c,
            &r,
            "MN_LODE_ARC_WEIGH",
            "EVENT_MN_LODE_ARC_WEIGH_DEFEATED"
        ));
    }
    finish(&mut c);
    settle(&mut c, "weigh guard victory");
    assert!(event(&c, "EVENT_MN_LODE_ARC_WEIGH_DEFEATED"));
    eprintln!("haulage: west stair and native weigh guard passed");
    walk(&mut c, &r, (5, 23));
    travel(&mut c, GameButton::Up, "MnIronDrillApproach", (24, 5));
    at(&mut c, "MnIronDrillApproach", (24, 5));
    // Existing full pressure course exercises refusal, both independent valves,
    // alternate-level returns, callback restoration and the two-circuit sump gate.
    super::northstar_iron_workings::enter_gallery(&mut c, &r, &root);
    eprintln!("haulage: original pressure network reached the pump gallery");
    at(&mut c, "MnIronPumpGallery", (4, 19));
    assert!(event(&c, HAUL) && event(&c, VENT));
    super::northstar_iron_workings::leave_gallery(&mut c, &r);
    at(&mut c, "MnIronDrillApproach", (24, 5));
    walk(&mut c, &r, (25, 6));
    travel(&mut c, GameButton::Up, ROOM, (4, 23));
    walk(&mut c, &r, (29, 13));
    talk(&mut c, GameButton::Right);
    if !event(&c, "EVENT_MN_LODE_ARC_CONVEYOR_DEFEATED") {
        assert!(super::rochester_earned::try_battle(
            &mut c,
            &r,
            "MN_LODE_ARC_CONVEYOR",
            "EVENT_MN_LODE_ARC_CONVEYOR_DEFEATED"
        ));
    }
    finish(&mut c);
    settle(&mut c, "conveyor guard victory");
    for (button, target) in [
        (GameButton::Up, (26, 9)),
        (GameButton::Up, (26, 8)),
        (GameButton::Up, (26, 7)),
        (GameButton::Down, (26, 8)),
        (GameButton::Down, (26, 9)),
        (GameButton::Down, (26, 10)),
    ] {
        if target == (26, 9) && button == GameButton::Up {
            walk(&mut c, &r, (26, 10));
        }
        travel(&mut c, button, ROOM, target);
    }
    // Every original active/passive pair is crossed in both directions. Ports
    // are read from the actual native module; no coordinate or flag is written.
    for index in [1, 3, 5, 7, 9] {
        eprintln!("haulage: testing shaft pair {index}");
        let port = r.data().maps[ROOM]
            .events
            .warps
            .iter()
            .find(|w| w.index == index)
            .expect("retained active shaft");
        let target_map = r
            .data()
            .map_name_for_constant(&port.target_map_constant)
            .expect("paired map");
        let target = &r.data().maps[&target_map];
        let arrival = target
            .events
            .warps
            .iter()
            .find(|w| i16::try_from(w.index).expect("native warp index") == port.target_warp_id)
            .expect("paired passive record");
        walk(&mut c, &r, (port.x as i16, port.y as i16 + 1));
        travel(
            &mut c,
            GameButton::Up,
            &target_map,
            (arrival.x as i16, arrival.y as i16),
        );
        c.wait_frames(40).expect("passive arrival safety");
        at(&mut c, &target_map, (arrival.x as i16, arrival.y as i16));
        let back = target
            .events
            .warps
            .iter()
            .find(|w| w.index == arrival.index - 1)
            .expect("paired active record");
        walk(&mut c, &r, (back.x as i16, back.y as i16 + 1));
        let passive = r.data().maps[ROOM]
            .events
            .warps
            .iter()
            .find(|w| w.index == index + 1)
            .expect("home passive record");
        travel(
            &mut c,
            GameButton::Up,
            ROOM,
            (passive.x as i16, passive.y as i16),
        );
    }
    walk(&mut c, &r, (17, 17));
    travel(&mut c, GameButton::Right, ROOM, (18, 17));
    travel(&mut c, GameButton::Right, ROOM, (19, 17));
    travel(&mut c, GameButton::Right, ROOM, (20, 17));
    c.save(out.join("opened-crosscut.crystalsave"))
        .expect("reached gate checkpoint");
    c = VisibleShellController::load_save(
        root,
        r.clone(),
        out.join("opened-crosscut.crystalsave"),
        None,
    )
    .expect("real native Continue");
    assert!(event(&c, HAUL) && event(&c, VENT));
    travel(&mut c, GameButton::Left, ROOM, (19, 17));
    travel(&mut c, GameButton::Left, ROOM, (18, 17));
    travel(&mut c, GameButton::Left, ROOM, (17, 17));
    walk(&mut c, &r, (5, 23));
    travel(&mut c, GameButton::Up, "MnIronDrillApproach", (24, 5));
    at(&mut c, "MnIronDrillApproach", (24, 5));
    c.save(out.join("final-return.crystalsave"))
        .expect("final native return");
    press(&mut c, GameButton::Start);
    assert!(c.shell.start_menu_cursor.is_some());
    press(&mut c, GameButton::B);
}
