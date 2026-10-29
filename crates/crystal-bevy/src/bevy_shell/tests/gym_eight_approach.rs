//! One initial mine fixture, then earned custody, road permission and Gym8.
use std::collections::BTreeMap;

use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use crystal_runtime::CrystalRuntime;
use serde_json::json;

use super::super::authored_route_fixture::{RouteGrid, RouteSpec, position};
use super::super::rochester::flee;
use super::super::winona::{at as location, travel};
use super::{
    VisibleShellController, at, battle, continue_save, controller, event, finish, interact, load,
    mark_runtime_snapshot_dirty, point, press, solve_puzzle, walk_to,
};

const FOREMAN: &str = "EVENT_MN_DEPOT_FOREMAN_DEFEATED";
const METER: &str = "EVENT_MN_MINE_INSTRUMENT_RECOVERED";
const DEPOT: &str = "EVENT_MN_DEPOT_CLEARED";

pub(super) fn accept(c: &mut VisibleShellController, pending_event: &str) {
    for _ in 0..64 {
        if c.snapshot()
            .expect("native offer")
            .ui
            .pending_yes_no
            .is_some()
        {
            break;
        }
        assert!(!event(c, pending_event));
        press(c, GameButton::A);
    }
    let prompt = c
        .snapshot()
        .expect("offer")
        .ui
        .pending_yes_no
        .expect("yes/no");
    for _ in 0..32 {
        let selected = c
            .shell
            .yes_no_cursor
            .as_ref()
            .expect("native cursor")
            .option_index;
        press(
            c,
            if selected == 0 {
                GameButton::A
            } else {
                GameButton::Up
            },
        );
        if c.snapshot()
            .expect("accepted offer")
            .ui
            .pending_yes_no
            .as_ref()
            != Some(&prompt)
        {
            return;
        }
    }
    panic!("native offer did not resolve");
}

pub(super) fn road(c: &mut VisibleShellController, r: &CrystalRuntime, number: u8, to: (i16, i16)) {
    assert!(
        try_road(c, r, number, to),
        "native blackout interrupted R{number}"
    );
}

/// Returns false only when ordinary battle recovery leaves the route.
pub(super) fn try_road(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    number: u8,
    to: (i16, i16),
) -> bool {
    let start = position(c);
    let (prefix, rows, doors) = match number {
        9 => (
            "MnR09",
            2,
            BTreeMap::from([((40, 41), "MnR09Cave1F"), ((40, 42), "MnR09Cave1F")]),
        ),
        10 => (
            "MnR10",
            1,
            BTreeMap::from([((36, 17), "MnR10Cave1F"), ((36, 18), "MnR10Cave1F")]),
        ),
        _ => panic!("unsupported final-gym road"),
    };
    let first = r
        .data()
        .map_module(&format!("{prefix}C00R00"))
        .expect("route");
    let tileset = if number == 9 {
        "mn_iron_range"
    } else if first.attributes.tileset_name == "mn_r10_rockstairs" {
        "mn_r10_rockstairs"
    } else {
        "johto"
    };
    assert_eq!(first.attributes.tileset_name, tileset);
    let grid = RouteGrid::new_with_grass(
        r,
        RouteSpec {
            prefix,
            columns: 2,
            rows,
            tileset,
            start,
            doors,
        },
    );
    for button in grid.path(start, to) {
        let before = position(c);
        for _ in 0..8 {
            press(c, button);
            if c.snapshot().expect("road encounter").battle.is_some() {
                flee(c);
            }
            if !c
                .snapshot()
                .expect("road or recovery")
                .overworld
                .map_name
                .starts_with(prefix)
            {
                return false;
            }
            if position(c) != before {
                break;
            }
        }
        assert_ne!(position(c), before, "native R{number} input must move");
    }
    assert_eq!(position(c), to);
    true
}

pub(super) fn care(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    city: &str,
    door: (u16, u16),
) {
    let clinic = format!("{city}Pokecenter1F");
    walk_to(c, r, door);
    travel(c, GameButton::Up, &clinic, (10, 13));
    let money = c.shell.shell.session().state().money;
    interact(c, r, (7, 4));
    finish(c);
    let snapshot = c.snapshot().expect("native free care");
    assert!(snapshot.party.slots.iter().all(|p| {
        p.pokemon.hp == p.pokemon.max_hp
            && p.pokemon.status.is_none()
            && p.pokemon
                .moves
                .iter()
                .all(|m| m.current_pp == r.data().moves[&m.name].pp)
    }));
    assert_eq!(c.shell.shell.session().state().money, money);
    walk_to(c, r, (10, 14));
    travel(c, GameButton::Down, city, (door.0 as i16, door.1 as i16));
}

#[test]
#[ignore = "requires the current regional gym pack via CRYSTAL_GYM_TEST_PACK"]
fn gym_eight_approach_controller_earns_meter_handover_road_and_harbor_badge_then_returns() {
    let (root, r, design) = load();
    let gym = &design["gyms"][7];
    let mut initial = gym.clone();
    initial["id"] = "MnR09Cave1F".into();
    initial["arrival_core"] = json!([13, 18]);
    let mut c = controller(&root, &r, &initial, false);
    // Explicit setup only: mine placement, seven badges, acknowledged report
    // and two legal level45 review partners. No downstream custody/gym flag is set.
    let mut partner = r
        .data()
        .create_pokemon("MEW", 45, Dv::default())
        .expect("partner");
    partner.moves = ["PSYCHIC_M", "FLAMETHROWER", "SURF", "THUNDERBOLT"]
        .into_iter()
        .map(|name| LearnedMove {
            name: name.into(),
            current_pp: r.data().moves[name].pp,
            pp_ups: 0,
        })
        .collect();
    let mut reserve = r
        .data()
        .create_pokemon("LAPRAS", 45, Dv::default())
        .expect("reserve partner");
    reserve.moves = ["SURF", "ICE_BEAM", "THUNDERBOLT", "PSYCHIC_M"]
        .into_iter()
        .map(|name| LearnedMove {
            name: name.into(),
            current_pp: r.data().moves[name].pp,
            pp_ups: 0,
        })
        .collect();
    let state = c.shell.shell.session_mut().state_mut();
    state.storage.party.pokemon.fill(None);
    assert!(state.storage.party.add_pokemon(partner));
    assert!(state.storage.party.add_pokemon(reserve));
    state.sync_party_from_storage();
    state
        .flags
        .set_event_flag("EVENT_MN_HEADWATERS_REPORT", true)
        .expect("upstream report fixture");
    for (i, flag) in BADGE_ENGINE_FLAGS.iter().enumerate() {
        state
            .flags
            .set_engine_flag(flag, i < 7)
            .expect("initial badge flags");
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    for flag in [
        FOREMAN,
        METER,
        DEPOT,
        "EVENT_MN_GYM_8_DEFEATED",
        "EVENT_MN_LIGHTHOUSE_SIGNAL",
    ] {
        assert!(!event(&c, flag));
    }

    interact(&mut c, &r, (7, 5));
    finish(&mut c);
    assert!(!event(&c, METER));
    interact(&mut c, &r, (9, 5));
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, FOREMAN) && !event(&c, METER) && !event(&c, DEPOT));
    c = continue_save(&mut c, &root, &r);
    interact(&mut c, &r, (7, 5));
    accept(&mut c, METER);
    assert!(
        !event(&c, METER),
        "taking the meter still requires reading its recovery text"
    );
    finish(&mut c);
    assert!(event(&c, METER) && !event(&c, DEPOT));
    c = continue_save(&mut c, &root, &r);
    walk_to(&mut c, &r, (13, 18));
    travel(&mut c, GameButton::Down, "MnR09C00R00", (40, 42));
    road(&mut c, &r, 9, (127, 24));
    travel(&mut c, GameButton::Right, "MnGrandRapids", (0, 24));
    travel(&mut c, GameButton::Right, "MnGrandRapids", (3, 24));
    care(&mut c, &r, "MnGrandRapids", (9, 12));
    walk_to(&mut c, &r, (43, 24));
    for _ in 0..8 {
        if c.snapshot().expect("east checkpoint").ui.text.is_some() {
            break;
        }
        press(&mut c, GameButton::Right);
    }
    assert_eq!(
        c.snapshot()
            .expect("real refusal")
            .ui
            .text
            .expect("text")
            .label,
        "MnGrandRapidsEastCheckEvidenceText"
    );
    finish(&mut c);
    location(&mut c, "MnGrandRapids", (43, 24));
    assert!(
        !event(&c, DEPOT),
        "winning and carrying evidence do not complete its handover"
    );
    walk_to(&mut c, &r, (27, 14));
    travel(&mut c, GameButton::Up, "MnGrandRapidsSurveyDepot", (12, 17));
    interact(&mut c, &r, (11, 9));
    accept(&mut c, DEPOT);
    finish(&mut c);
    assert!(event(&c, DEPOT));
    c = continue_save(&mut c, &root, &r);
    walk_to(&mut c, &r, (12, 18));
    travel(&mut c, GameButton::Down, "MnGrandRapids", (27, 14));
    walk_to(&mut c, &r, (43, 24));
    travel(&mut c, GameButton::Right, "MnR10C00R00", (0, 10));
    road(&mut c, &r, 10, (95, 24));
    travel(&mut c, GameButton::Right, "MnDuluth", (0, 24));
    travel(&mut c, GameButton::Right, "MnDuluth", (3, 24));
    care(&mut c, &r, "MnDuluth", (37, 12));
    c = continue_save(&mut c, &root, &r);
    walk_to(&mut c, &r, (16, 12));
    travel(&mut c, GameButton::Up, "MnDuluthGym", (6, 17));
    solve_puzzle(&mut c, &r, gym);
    assert!(event(&c, "EVENT_MN_GYM_8_COURT_OPEN"));
    assert!(!c.shell.shell.session().state().badges.johto[7]);
    c = continue_save(&mut c, &root, &r);
    // Use the newly opened return course and real clinic before the leader.
    // Training wins and the physical puzzle must survive leaving and Continue.
    walk_to(&mut c, &r, (6, 18));
    travel(&mut c, GameButton::Down, "MnDuluth", (16, 12));
    care(&mut c, &r, "MnDuluth", (37, 12));
    walk_to(&mut c, &r, (16, 12));
    travel(&mut c, GameButton::Up, "MnDuluthGym", (6, 17));
    assert!(event(&c, "EVENT_MN_GYM_8_COURT_OPEN"));
    for i in 1..=3 {
        assert!(event(&c, &format!("EVENT_MN_GYM_8_TRAINER_{i}_DEFEATED")));
    }
    interact(&mut c, &r, point(&gym["leader_at"]));
    battle(&mut c, false);
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().badges.johto, [true; 8]);
    let tm = &r.data().items["TM_ICY_WIND"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    assert!(!event(&c, "EVENT_MN_LIGHTHOUSE_SIGNAL"));
    c = continue_save(&mut c, &root, &r);
    interact(&mut c, &r, point(&gym["leader_at"]));
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    walk_to(&mut c, &r, (6, 18));
    travel(&mut c, GameButton::Down, "MnDuluth", (16, 12));
    care(&mut c, &r, "MnDuluth", (37, 12));
    walk_to(&mut c, &r, (4, 24));
    travel(&mut c, GameButton::Left, "MnR10C01R00", (31, 24));
    road(&mut c, &r, 10, (0, 10));
    travel(&mut c, GameButton::Left, "MnGrandRapids", (47, 24));
    travel(&mut c, GameButton::Left, "MnGrandRapids", (44, 24));
    c = continue_save(&mut c, &root, &r);
    assert!(event(&c, FOREMAN) && event(&c, METER) && event(&c, DEPOT));
    assert_eq!(c.shell.shell.session().state().badges.johto, [true; 8]);
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    assert_eq!(at(&mut c), (44, 24));
}
