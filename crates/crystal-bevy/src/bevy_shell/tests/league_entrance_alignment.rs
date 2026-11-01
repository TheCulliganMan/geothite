//! Declared prior chapter requirements; both entrance lanes use native input.
use crystal_core::input::GameButton;
use crystal_core::models::Dv;

use super::super::mark_runtime_snapshot_dirty;
use super::interior_life::fixture;
use super::northstar_ice_lair::{page, walk};
use super::r10_gallery::{continue_save, settle};
use super::winona::{at, load, press, travel};

fn approach(c: &mut super::super::VisibleShellController) {
    for _ in 0..4 {
        let snapshot = c.snapshot().expect("approach guard through native facing");
        if snapshot.ui.text.is_some() || snapshot.overworld.tile.y == 42 {
            return;
        }
        press(c, GameButton::Down);
    }
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with canonical League entrance alignment"]
fn league_entrance_controller_both_lanes_deny_clear_return_and_continue() {
    let (root, runtime) = load();
    for (badges, signal, restored, denied) in [
        (0, false, false, Some("MnBoundaryLeagueCheckBadgesText")),
        (8, false, false, Some("MnBoundaryLeagueCheckSignalText")),
        (8, true, false, Some("MnBoundaryLeagueCheckRestoredText")),
        (8, true, true, None),
    ] {
        for x in [32, 33] {
            let mut c = fixture(&root, &runtime, "MnBoundaryWaters", (x, 41));
            {
                let state = c.shell.shell.session_mut().state_mut();
                state.storage.party.pokemon[0] = Some(
                    runtime
                        .data()
                        .create_pokemon("BOREALYNX", 60, Dv::default())
                        .expect("declared native gate partner"),
                );
                state.sync_party_from_storage();
                for i in 0..badges {
                    state.badges.johto[i] = true;
                    state
                        .flags
                        .set_engine_flag(
                            crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS[i],
                            true,
                        )
                        .expect("declared prior badge");
                }
                state
                    .flags
                    .set_event_flag("EVENT_MN_LIGHTHOUSE_SIGNAL", signal)
                    .expect("declared signal state");
                state
                    .flags
                    .set_event_flag("EVENT_MN_SURVEY_RESTORED", restored)
                    .expect("declared restoration state");
            }
            mark_runtime_snapshot_dirty(&mut c.shell);
            // No state, bag, party, flag, position or RNG writes below setup.
            let before = c.snapshot().expect("before gate");
            let money = c.shell.shell.session().state().money;
            approach(&mut c);
            if let Some(label) = denied {
                page(&mut c, label);
                let held = c.snapshot().expect("denial owns input").overworld.tile;
                press(&mut c, GameButton::Down);
                assert_eq!(
                    c.snapshot().expect("blocked during text").overworld.tile,
                    held
                );
                settle(&mut c, "denied gate returns inside");
                at(&mut c, "MnBoundaryWaters", (x, 41));
                c = continue_save(root.clone(), runtime.clone(), &mut c);
                approach(&mut c);
                page(&mut c, label);
                settle(&mut c, "continued denial");
                at(&mut c, "MnBoundaryWaters", (x, 41));
            } else {
                settle(&mut c, "cleared gate");
                at(&mut c, "MnBoundaryWaters", (x, 42));
                for y in 43..=47 {
                    travel(&mut c, GameButton::Down, "MnBoundaryWaters", (x, y));
                    settle(&mut c, "cleared departure");
                    at(&mut c, "MnBoundaryWaters", (x, y));
                    if y == 43 {
                        assert_eq!(
                            c.shell.shell.session().overworld().player.facing,
                            crystal_core::world::map::Direction::Down,
                            "native outward facing before the inward trigger"
                        );
                        let path = super::authored_route_fixture::reached_save_path(&format!(
                            "league-cleared-lane-{x}-before-inward.crystalsave"
                        ));
                        c.save(&path).expect("reached cleared gate diagnostic save");
                        super::authored_route_fixture::cleanup_reached_save(&path);
                        c.press(GameButton::Down).expect("outward trigger press");
                        at(&mut c, "MnBoundaryWaters", (x, 44));
                        c.wait_frames(1).expect("idle frame at outward trigger");
                        at(&mut c, "MnBoundaryWaters", (x, 44));
                    }
                }
                travel(&mut c, GameButton::Down, "MnR13C00R00", (x, 0));
                at(&mut c, "MnR13C00R00", (x, 0));
                c = continue_save(root.clone(), runtime.clone(), &mut c);
                walk(&mut c, &runtime, (31, 1));
                walk(&mut c, &runtime, (x, 0));
                travel(&mut c, GameButton::Up, "MnBoundaryWaters", (x, 47));
                at(&mut c, "MnBoundaryWaters", (x, 47));
                // The inward event at44 automatically walks two native steps
                // through the guard lane to42; neither44 nor43 is a stop.
                for y in [46, 45, 42, 41] {
                    travel(&mut c, GameButton::Up, "MnBoundaryWaters", (x, y));
                    settle(&mut c, "cleared return");
                    at(&mut c, "MnBoundaryWaters", (x, y));
                }
                c = continue_save(root.clone(), runtime.clone(), &mut c);
            }
            let after = c.snapshot().expect("gate resources");
            assert_eq!(after.party, before.party);
            assert_eq!(after.bag, before.bag);
            assert_eq!(c.shell.shell.session().state().money, money);
        }
    }
}
