//! Explicit local fixtures, followed exclusively by production inputs/Continue.
use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::VisibleShellController;
use super::authored_route_fixture::{Point, indoor_walk};
use super::duluth_story::{finish, fixture_at, save_path, talk};

const CITY: &str = "MnDuluth";
const RANGE: &str = "MnR10C01R00";
const SHORE: &str = "MnR11C00R03";
const CLINIC: &str = "MnDuluthPokecenter1F";
const WORKSHOP: &str = "MnDuluthHarborWorkshop";
const DEPOT: &str = "EVENT_MN_DEPOT_CLEARED";
const SIGNAL: &str = "EVENT_MN_LIGHTHOUSE_SIGNAL";

fn press(c: &mut VisibleShellController, button: GameButton) {
    c.press(button).expect("production checkpoint input");
    c.wait_frames(16).expect("native presentation");
    assert_eq!(c.shell.last_error, None);
}
fn at(c: &mut VisibleShellController, map: &str, tile: Point) {
    let s = c.snapshot().expect("authoritative location");
    assert_eq!(s.overworld.map_name, map);
    assert_eq!((s.overworld.tile.x, s.overworld.tile.y), tile);
}
fn event(c: &VisibleShellController, name: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(name)
        .expect("native event")
}
fn walk(c: &mut VisibleShellController, runtime: &CrystalRuntime, to: Point) {
    let map = c.snapshot().expect("map").overworld.map_name;
    indoor_walk(c, runtime, to);
    at(c, &map, to);
    assert_eq!(c.shell.last_error, None);
}
fn advance(c: &mut VisibleShellController, button: GameButton, map: &str, tile: Point) {
    for _ in 0..16 {
        let s = c.snapshot().expect("native travel");
        if s.overworld.map_name == map && (s.overworld.tile.x, s.overworld.tile.y) == tile {
            return;
        }
        assert!(
            !s.ui.text_window_open,
            "unexpected refusal while advancing: {:?}",
            s.ui.text
        );
        press(c, button);
    }
    panic!("native input never reached {map} {tile:?}");
}
fn refusal(c: &mut VisibleShellController, button: GameButton, expected: &str) {
    for _ in 0..8 {
        if c.snapshot().expect("checkpoint").ui.text.is_some() {
            break;
        }
        press(c, button);
    }
    let s = c.snapshot().expect("refusal");
    assert_eq!(s.ui.text.as_ref().expect("native refusal").label, expected);
    let tile = s.overworld.tile;
    press(c, GameButton::Select);
    assert_eq!(c.snapshot().expect("text owner").overworld.tile, tile);
    finish(c);
}
fn continued(
    c: &mut VisibleShellController,
    root: &AssetRoot,
    runtime: &CrystalRuntime,
    stage: &str,
) -> VisibleShellController {
    let path = save_path(stage);
    let before = c.snapshot().expect("before Continue");
    let state = c.shell.shell.session().state().clone();
    c.save(&path).expect("actual production save");
    let mut loaded =
        VisibleShellController::load_save(root.clone(), runtime.clone(), path.clone(), None)
            .expect("production Continue");
    let after = loaded.snapshot().expect("continued state");
    assert_eq!(after.overworld.map_name, before.overworld.map_name);
    assert_eq!(after.overworld.tile, before.overworld.tile);
    assert_eq!(after.party, before.party);
    assert_eq!(after.bag, before.bag);
    let now = loaded.shell.shell.session().state();
    assert_eq!(now.money, state.money);
    assert_eq!(now.badges, state.badges);
    for flag in state
        .flags
        .event_flags
        .keys()
        .chain(now.flags.event_flags.keys())
    {
        assert_eq!(
            state.flags.is_event_flag_set(flag),
            now.flags.is_event_flag_set(flag),
            "Continue event {flag}"
        );
    }
    for flag in state
        .flags
        .engine_flags
        .keys()
        .chain(now.flags.engine_flags.keys())
    {
        assert_eq!(
            state.flags.is_engine_flag_set(flag),
            now.flags.is_engine_flag_set(flag),
            "Continue engine flag {flag}"
        );
    }
    if std::env::var_os("CRYSTAL_DULUTH_STORY_SAVE_DIR").is_none() {
        std::fs::remove_file(path).expect("remove own temporary save only");
    }
    loaded
}
fn enter(
    c: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    outside: Point,
    room: &str,
    arrival: Point,
) {
    walk(c, runtime, outside);
    advance(c, GameButton::Up, room, arrival);
}

#[test]
#[ignore = "requires native connected Duluth story pack"]
fn duluth_connections_controller_checks_each_badge_and_exact_departure_events_on_both_lanes() {
    for (badges, depot, label) in [
        ([false; 8], true, "MnDuluthRangeCheckBadgesText"),
        ([true; 8], false, "MnDuluthRangeCheckEvidenceText"),
    ] {
        let (root, runtime, mut c) = fixture_at(CITY, (6, 24), badges, depot, false, false);
        for y in [24, 25] {
            walk(&mut c, &runtime, (4, y));
            let before = c.shell.shell.session().state().clone();
            refusal(&mut c, GameButton::Left, label);
            at(&mut c, CITY, (4, y));
            let state = c.shell.shell.session().state();
            assert_eq!(state.flags, before.flags);
            assert_eq!(state.badges, before.badges);
            assert_eq!(state.bag, before.bag);
            assert_eq!(state.money, before.money);
        }
        let _loaded = continued(&mut c, &root, &runtime, "range-denied");
    }
    for missing in 0..9 {
        let mut badges = [true; 8];
        if missing < 8 {
            badges[missing] = false;
        }
        let initial_signal = missing < 8;
        let (root, runtime, mut c) = fixture_at(CITY, (32, 4), badges, true, initial_signal, false);
        for x in [32, 33] {
            walk(&mut c, &runtime, (x, 4));
            let before = c.shell.shell.session().state().clone();
            refusal(
                &mut c,
                GameButton::Up,
                if missing < 8 {
                    "MnDuluthShoreCheckBadgesText"
                } else {
                    "MnDuluthShoreCheckEvidenceText"
                },
            );
            at(&mut c, CITY, (x, 4));
            let state = c.shell.shell.session().state();
            assert_eq!(state.flags, before.flags);
            assert_eq!(state.badges, before.badges);
            assert_eq!(state.bag, before.bag);
            assert_eq!(state.money, before.money);
        }
        let _loaded = continued(&mut c, &root, &runtime, "shore-denied");
    }
}

#[test]
#[ignore = "requires native connected Duluth story pack"]
fn duluth_connections_controller_earned_harbor_reading_releases_shore_and_both_seams_continue() {
    let (root, runtime, mut c) = fixture_at(CITY, (32, 4), [true; 8], true, false, false);
    refusal(&mut c, GameButton::Up, "MnDuluthShoreCheckEvidenceText");
    at(&mut c, CITY, (32, 4));
    enter(&mut c, &runtime, (48, 36), WORKSHOP, (8, 9));
    walk(&mut c, &runtime, (10, 5));
    talk(&mut c);
    finish(&mut c);
    assert!(event(&c, SIGNAL));
    walk(&mut c, &runtime, (8, 10));
    advance(&mut c, GameButton::Down, CITY, (48, 36));
    let mut c = continued(&mut c, &root, &runtime, "earned-harbor");
    let before = c.shell.shell.session().state().clone();
    walk(&mut c, &runtime, (32, 4));
    advance(&mut c, GameButton::Up, SHORE, (46, 47));
    let mut c = continued(&mut c, &root, &runtime, "shore-outbound");
    advance(&mut c, GameButton::Down, CITY, (32, 3));
    advance(&mut c, GameButton::Down, CITY, (32, 4));
    walk(&mut c, &runtime, (4, 25));
    advance(&mut c, GameButton::Left, RANGE, (31, 25));
    let mut c = continued(&mut c, &root, &runtime, "range-outbound");
    advance(&mut c, GameButton::Right, CITY, (3, 25));
    advance(&mut c, GameButton::Right, CITY, (4, 25));
    walk(&mut c, &runtime, (6, 24));
    assert!(event(&c, SIGNAL) && event(&c, DEPOT));
    let state = c.shell.shell.session().state();
    assert_eq!(state.money, before.money);
    assert_eq!(state.bag, before.bag);
    assert_eq!(state.badges, before.badges);
    assert_eq!(state.storage, before.storage);
    let _loaded = continued(&mut c, &root, &runtime, "duluth-round-trip");
}

#[test]
#[ignore = "requires native connected Duluth story pack"]
fn duluth_connections_controller_inbound_routes_reach_actual_free_care_without_permission() {
    for (map, tile, button, inside) in [
        (RANGE, (31, 25), GameButton::Right, (3, 25)),
        (SHORE, (46, 47), GameButton::Down, (32, 3)),
    ] {
        let (root, runtime, mut c) = fixture_at(map, tile, [false; 8], false, false, true);
        advance(&mut c, button, CITY, inside);
        let injured = c.snapshot().expect("injured traveler");
        assert!(injured.party.slots[0].pokemon.hp < injured.party.slots[0].pokemon.max_hp);
        enter(&mut c, &runtime, (37, 12), CLINIC, (10, 13));
        walk(&mut c, &runtime, (7, 5));
        press(&mut c, GameButton::Up);
        press(&mut c, GameButton::A);
        finish(&mut c);
        let healed = c.snapshot().expect("actual free care");
        for slot in healed.party.slots {
            let p = slot.pokemon;
            assert_eq!(p.hp, p.max_hp);
            assert!(p.status.is_none());
            assert!(
                p.moves
                    .iter()
                    .all(|m| m.current_pp == runtime.data().moves[&m.name].pp)
            );
        }
        assert!(!event(&c, DEPOT) && !event(&c, SIGNAL));
        assert_eq!(c.shell.shell.session().state().money, 10000);
        let _loaded = continued(&mut c, &root, &runtime, "duluth-inbound-care");
    }
}
