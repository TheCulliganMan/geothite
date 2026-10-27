//! Explicit badge/signal fixtures; regional moves and services use public input.
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::world::movement::MovementMode;
use std::collections::BTreeMap;

use super::super::super::visible_field_dialogue_is_entirely_consumed;
use super::super::authored_route_fixture::{RouteGrid, RouteSpec, position};
use super::super::winona::{at, continued, travel};
use super::leech_shore::fixture;
use super::{VisibleShellController, event, finish, mark_runtime_snapshot_dirty, press, walk_to};

const TOWN: &str = "MnGrandMarais";
const GATE: &str = "MnGrandMaraisShoreGate";
const SIGNAL: &str = "EVENT_MN_LIGHTHOUSE_SIGNAL";
const CLINIC: &str = "MnGrandMaraisPokecenter";

pub(super) fn choice(c: &mut VisibleShellController, yes: bool) {
    for _ in 0..64 {
        let s = c.shell.shell.snapshot().expect("authoritative offer pages");
        if s.ui.pending_yes_no.is_some()
            && visible_field_dialogue_is_entirely_consumed(&c.shell, &s)
        {
            let target = usize::from(!yes);
            for _ in 0..4 {
                let index = c
                    .shell
                    .yes_no_cursor
                    .as_ref()
                    .expect("visible YES/NO")
                    .option_index;
                if index == target {
                    press(c, GameButton::A);
                    return;
                }
                press(
                    c,
                    if index < target {
                        GameButton::Down
                    } else {
                        GameButton::Up
                    },
                );
            }
            panic!("visible choice did not select requested answer");
        }
        press(c, GameButton::A);
    }
    panic!("offer never finished displaying");
}

fn signal(c: &mut VisibleShellController, on: bool) {
    c.shell
        .shell
        .session_mut()
        .state_mut()
        .flags
        .set_event_flag(SIGNAL, on)
        .expect("prior signal fixture");
    mark_runtime_snapshot_dirty(&mut c.shell);
}
fn until_map(c: &mut VisibleShellController, button: GameButton, map: &str) {
    for _ in 0..16 {
        if c.snapshot().expect("map change").overworld.map_name == map {
            return;
        }
        press(c, button);
    }
    panic!("native transition did not reach {map}");
}

#[test]
#[ignore = "external North Shore story pack; explicit eight badges/signal/empty-party setup"]
fn north_shore_story_controller_gate_both_lanes_continue_and_gunflint_seam() {
    for lane in 0..2 {
        let (root, r, _, mut c) = fixture(TOWN, [8, 18 + lane], 8, true);
        signal(&mut c, true);
        travel(&mut c, GameButton::Left, GATE, (8, 4 + lane as i16));
        c = continued(c, &root, &r, "shore-local-gate-east");
        walk_to(&mut c, &r, (1, 4 + lane));
        travel(
            &mut c,
            GameButton::Left,
            "MnR11C00R00",
            (42, 4 + lane as i16),
        );
        travel(&mut c, GameButton::Right, GATE, (1, 4 + lane as i16));
        walk_to(&mut c, &r, (8, 4 + lane));
        until_map(&mut c, GameButton::Right, TOWN);
        let s = c.snapshot().expect("gate town return");
        assert_eq!(s.overworld.tile.x, 8);
        assert_eq!(s.overworld.tile.y, 18 + lane as i16);
        walk_to(&mut c, &r, (4, 8 + lane));
        travel(
            &mut c,
            GameButton::Left,
            "MnR12C01R01",
            (63, 8 + lane as i16),
        );
        travel(&mut c, GameButton::Right, TOWN, (0, 8 + lane as i16));
        travel(&mut c, GameButton::Right, TOWN, (4, 8 + lane as i16));
        assert!(!event(&c, "EVENT_MN_GRAND_MARAIS_SIGNAL"));
        assert!(!event(&c, "EVENT_MN_MARAIS_SPARE_PART_RECEIVED"));
        assert!(
            c.shell
                .shell
                .session()
                .state()
                .storage
                .party
                .pokemon
                .iter()
                .all(Option::is_none)
        );
    }
}

#[test]
#[ignore = "external North Shore story pack; explicit missing prerequisite fixtures"]
fn north_shore_story_controller_missing_badge_signal_and_free_inbound() {
    for (count, has_signal, word) in [(7, true, "BADGES"), (8, false, "DULUTH")] {
        for (inside, y) in [(8, 18), (8, 19), (4, 8), (4, 9)] {
            let (_, _, _, mut c) = fixture(TOWN, [inside, y], count, true);
            signal(&mut c, has_signal);
            for _ in 0..8 {
                if c.snapshot().expect("refusal").ui.text.is_some() {
                    break;
                }
                press(&mut c, GameButton::Left);
            }
            assert!(
                c.snapshot()
                    .expect("missing requirement")
                    .ui
                    .text
                    .and_then(|t| t.asm_text)
                    .is_some_and(|t| t.contains(word))
            );
            press(&mut c, GameButton::Left);
            at(&mut c, TOWN, ((inside - 1) as i16, y as i16));
            finish(&mut c);
            at(&mut c, TOWN, (inside as i16, y as i16));
        }
    }
    for lane in 0..2 {
        let (_, r, _, mut c) = fixture("MnR12C01R01", [63, 8 + lane], 0, true);
        travel(&mut c, GameButton::Right, TOWN, (0, 8 + lane as i16));
        travel(&mut c, GameButton::Right, TOWN, (4, 8 + lane as i16));
        walk_to(&mut c, &r, (15, 10));
        until_map(&mut c, GameButton::Up, "MnGrandMaraisPokecenter");
        assert!(!event(&c, SIGNAL));
    }
}

#[test]
#[ignore = "external North Shore story pack; empty party/prior access setup"]
fn north_shore_story_controller_optional_part_decline_retry_no_demo_starter_and_continue() {
    let (root, r, _, mut c) = fixture(TOWN, [19, 12], 8, true);
    signal(&mut c, true);
    let money = c.shell.shell.session().state().money;
    let bags = c.shell.shell.session().state().bag.clone();
    walk_to(&mut c, &r, (6, 8));
    until_map(&mut c, GameButton::Up, "MnGrandMaraisFieldStation");
    super::interact(&mut c, &r, (5, 3));
    choice(&mut c, false);
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_MARAIS_SPARE_PART_RECEIVED"));
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .storage
            .party
            .pokemon
            .iter()
            .all(Option::is_none)
    );
    assert_eq!(c.shell.shell.session().state().bag, bags);
    assert_eq!(c.shell.shell.session().state().money, money);
    c = continued(c, &root, &r, "shore-local-part-declined");
    // The scientist remains available after the optional opening was declined.
    super::interact(&mut c, &r, (5, 3));
    choice(&mut c, true);
    assert!(!event(&c, "EVENT_MN_MARAIS_SPARE_PART_RECEIVED"));
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_MARAIS_SPARE_PART_RECEIVED"));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["MACHINE_PART"]),
        1
    );
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .storage
            .party
            .pokemon
            .iter()
            .all(Option::is_none)
    );
    c = continued(c, &root, &r, "shore-local-part-received");
    super::interact(&mut c, &r, (5, 3));
    finish(&mut c);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["MACHINE_PART"]),
        1
    );
    assert!(!event(&c, "EVENT_MN_GRAND_MARAIS_STARTER"));
    assert!(!event(&c, "EVENT_MN_GRAND_MARAIS_SIGNAL"));
    walk_to(&mut c, &r, (2, 6));
    until_map(&mut c, GameButton::Down, TOWN);
    walk_to(&mut c, &r, (15, 28));
    until_map(&mut c, GameButton::Up, "MnGrandMaraisLighthouse");
    super::interact(&mut c, &r, (5, 3));
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GRAND_MARAIS_SIGNAL"));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["MACHINE_PART"]),
        0
    );
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["SUPER_POTION"]),
        1
    );
    c = continued(c, &root, &r, "shore-local-lamp-repaired");
    super::interact(&mut c, &r, (5, 3));
    finish(&mut c);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["SUPER_POTION"]),
        1
    );
    assert_eq!(c.shell.shell.session().state().money, money);
    assert!(event(&c, SIGNAL));
}

#[test]
#[ignore = "external shore pack; injured legal partner/prior badge setup"]
fn north_shore_story_controller_native_care_and_unqualified_coast_return() {
    for lane in 0..2 {
        let (root, r, _, mut c) = fixture("MnR11C00R00", [42, 4 + lane], 0, false);
        {
            let state = c.shell.shell.session_mut().state_mut();
            let p = state.storage.party.pokemon[0]
                .as_mut()
                .expect("injured setup");
            p.hp = 1;
            p.moves[0].current_pp = 0;
            state.money = 1200;
            state.sync_party_from_storage();
        }
        mark_runtime_snapshot_dirty(&mut c.shell);
        travel(&mut c, GameButton::Right, GATE, (1, 4 + lane as i16));
        walk_to(&mut c, &r, (8, 4 + lane));
        travel(&mut c, GameButton::Right, TOWN, (8, 18 + lane as i16));
        walk_to(&mut c, &r, (15, 10));
        until_map(&mut c, GameButton::Up, CLINIC);
        walk_to(&mut c, &r, (3, 3));
        press(&mut c, GameButton::Up);
        press(&mut c, GameButton::A);
        press(&mut c, GameButton::Down);
        at(&mut c, CLINIC, (3, 3));
        finish(&mut c);
        let s = c.snapshot().expect("actual free care");
        assert!(s.party.slots.iter().all(|s| {
            s.pokemon.hp == s.pokemon.max_hp
                && s.pokemon
                    .moves
                    .iter()
                    .all(|m| m.current_pp == r.data().moves[&m.name].pp)
        }));
        assert_eq!(c.shell.shell.session().state().money, 1200);
        assert!(!event(&c, SIGNAL) && !event(&c, "EVENT_MN_GRAND_MARAIS_SIGNAL"));
        c = continued(c, &root, &r, "shore-local-native-care");
        walk_to(&mut c, &r, (3 + lane, 6));
        until_map(&mut c, GameButton::Down, TOWN);
        at(&mut c, TOWN, (15, 10));
        press(&mut c, GameButton::Down);
        at(&mut c, TOWN, (15, 11));
    }
}

#[test]
#[ignore = "external shore pack; legal Surf partner/eight prior badges setup"]
fn north_shore_story_controller_surf_sea_boundary_and_dismount_continue() {
    let (root, r, _, mut c) = fixture(TOWN, [29, 31], 8, false);
    {
        let state = c.shell.shell.session_mut().state_mut();
        state.storage.party.pokemon[0]
            .as_mut()
            .expect("legal Surf setup")
            .moves[2] = LearnedMove {
            name: "SURF".into(),
            current_pp: r.data().moves["SURF"].pp,
            pp_ups: 0,
        };
        state.sync_party_from_storage();
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    press(&mut c, GameButton::Down);
    press(&mut c, GameButton::A);
    for _ in 0..128 {
        if c.snapshot().expect("native Surf launch").overworld.mode == MovementMode::Surf {
            break;
        }
        press(&mut c, GameButton::A);
    }
    at(&mut c, TOWN, (29, 32));
    assert_eq!(
        c.snapshot().expect("mounted").overworld.mode,
        MovementMode::Surf
    );
    for y in 33..=36 {
        travel(&mut c, GameButton::Down, TOWN, (29, y));
    }
    for _ in 0..8 {
        press(&mut c, GameButton::Down);
    }
    at(&mut c, TOWN, (29, 36));
    c = continued(c, &root, &r, "shore-local-surf-boundary");
    assert_eq!(
        c.snapshot().expect("saved Surf mode").overworld.mode,
        MovementMode::Surf
    );
    for y in (31..=35).rev() {
        travel(&mut c, GameButton::Up, TOWN, (29, y));
    }
    assert_eq!(
        c.snapshot().expect("native dismount").overworld.mode,
        MovementMode::Normal
    );
    assert!(!event(&c, SIGNAL) && !event(&c, "EVENT_MN_GRAND_MARAIS_SIGNAL"));
}

#[test]
#[ignore = "external shore pack; weak legal partner/money/prior-access setup; actual losses"]
fn north_shore_story_controller_both_clinics_select_persistent_blackout_recovery() {
    for duluth in [true, false] {
        let (city, door, clinic, recovery) = if duluth {
            ("MnDuluth", (37, 12), "MnDuluthPokecenter1F", (10, 13))
        } else {
            (TOWN, (15, 10), CLINIC, (3, 5))
        };
        let (root, r, _, mut c) = fixture(city, [door.0, door.1], if duluth { 7 } else { 8 }, true);
        signal(&mut c, !duluth);
        {
            let state = c.shell.shell.session_mut().state_mut();
            let mut p = r
                .data()
                .create_pokemon("MAGIKARP", 5, Dv::default())
                .expect("weak legal setup");
            p.hp = 1;
            p.moves = vec![LearnedMove {
                name: "SPLASH".into(),
                current_pp: 0,
                pp_ups: 0,
            }];
            assert!(state.storage.party.add_pokemon(p));
            state.money = 1200;
            state.sync_party_from_storage();
        }
        mark_runtime_snapshot_dirty(&mut c.shell);
        if duluth {
            super::gym_eight_approach::care(&mut c, &r, city, door);
        } else {
            until_map(&mut c, GameButton::Up, clinic);
            walk_to(&mut c, &r, (3, 3));
            press(&mut c, GameButton::Up);
            press(&mut c, GameButton::A);
            finish(&mut c);
            walk_to(&mut c, &r, (3, 6));
            until_map(&mut c, GameButton::Down, city);
        }
        assert_eq!(c.shell.shell.session().state().money, 1200);
        c = continued(
            c,
            &root,
            &r,
            if duluth {
                "shore-local-duluth-care-before-loss"
            } else {
                "shore-local-marais-care-before-loss"
            },
        );
        let bag = c.shell.shell.session().state().bag.clone();
        let badges = c.shell.shell.session().state().badges.clone();
        if duluth {
            walk_to(&mut c, &r, (16, 12));
            travel(&mut c, GameButton::Up, "MnDuluthGym", (6, 17));
            super::interact(&mut c, &r, (3, 4));
            super::battle(&mut c, true);
            finish(&mut c);
        } else {
            walk_to(&mut c, &r, (4, 8));
            travel(&mut c, GameButton::Left, "MnR12C01R01", (63, 8));
            let start = position(&mut c);
            let grid = RouteGrid::new_with_grass(
                &r,
                RouteSpec {
                    prefix: "MnR12",
                    columns: 2,
                    rows: 3,
                    tileset: "mn_gunflint",
                    start,
                    doors: BTreeMap::new(),
                },
            );
            let mut fought = false;
            'approach: for button in grid.path(start, (107, 52)) {
                let before = position(&mut c);
                for _ in 0..8 {
                    press(&mut c, button);
                    if c.snapshot()
                        .expect("played loss encounter")
                        .battle
                        .is_some()
                    {
                        super::battle(&mut c, true);
                        finish(&mut c);
                        fought = true;
                        break 'approach;
                    }
                    if position(&mut c) != before {
                        break;
                    }
                }
                assert_ne!(position(&mut c), before);
            }
            if !fought {
                press(&mut c, GameButton::Up);
                press(&mut c, GameButton::A);
                super::battle(&mut c, true);
                finish(&mut c);
            }
        }
        at(&mut c, clinic, recovery);
        assert!(c.shell.shell.session().state().money < 1200);
        assert_eq!(c.shell.shell.session().state().bag, bag);
        assert_eq!(c.shell.shell.session().state().badges, badges);
        let s = c.snapshot().expect("native blackout healing");
        assert_eq!(s.party.slots[0].pokemon.hp, s.party.slots[0].pokemon.max_hp);
        assert_eq!(
            s.party.slots[0].pokemon.moves[0].current_pp,
            r.data().moves["SPLASH"].pp
        );
        assert!(!event(&c, "EVENT_MN_GRAND_MARAIS_SIGNAL"));
        let _ = continued(
            c,
            &root,
            &r,
            if duluth {
                "shore-local-duluth-blackout"
            } else {
                "shore-local-marais-blackout"
            },
        );
    }
}
