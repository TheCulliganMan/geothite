//! A fixed ice route with ordinary partners, native slides and distinct gates.
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_runtime::CrystalRuntime;

use super::super::winona::travel;
use super::gym_care::heal;
use super::rochester_readable::{talk, walk};
use super::{
    VisibleShellController, at, battle_with_partner, can_approach, continue_save, controller,
    event, finish, gym_collision, load, mark_runtime_snapshot_dirty, press,
};

fn slide(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    from: (u16, u16),
    stop: (u16, u16),
    direction: GameButton,
    opposing: GameButton,
) {
    assert_eq!(at(c), from);
    for _ in 0..8 {
        c.press(direction).expect("enter ice");
        c.wait_frames(1).expect("observe live slide");
        if at(c) != from {
            break;
        }
    }
    let entered = at(c);
    assert_ne!(entered, from);
    if entered == stop {
        assert!(from.0.abs_diff(stop.0) + from.1.abs_diff(stop.1) <= 2);
    } else {
        assert_eq!(gym_collision(c, r, entered), "ICE");
        c.press(opposing).expect("ice retains its direction");
    }
    c.wait_frames(160).expect("native ice reaches its stop");
    assert_eq!(at(c), stop);
    c.wait_frames(80).expect("rest at stop");
    assert_eq!(at(c), stop);
}

fn ice_battle(c: &mut VisibleShellController) {
    let choose: fn(&str) -> Option<&'static str> = |enemy| match enemy {
        "SEEL" | "DEWGONG" | "LAPRAS" => Some("ELECTABUZZ"),
        "SNEASEL" => Some("TYPHLOSION"),
        "SWINUB" | "PILOSWINE" => Some("FERALIGATR"),
        _ => None,
    };
    battle_with_partner(c, false, Some(choose));
    finish(c);
}

#[test]
#[ignore = "requires minnesota-duluth-gold-gates.crystalpack via CRYSTAL_GYM_TEST_PACK"]
fn duluth_gates_literal_ordinary_party_two_ice_courses_all_trainers_and_city_returns() {
    let (root, r, _) = load();
    let design: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../planning/minnesota/gyms/duluth-gold-gates.json"
    ))
    .expect("gold gates source");
    let mut c = controller(&root, &r, &design["gyms"][0], false);
    // Initial fixtures only: entrance, seven prior badges, native partners
    // with legal moves (previously taught Surf), six carried Hyper Potions
    // and the shared deterministic divider.
    // No gym8 progress is given.
    let state = c.shell.shell.session_mut().state_mut();
    state.storage.party.pokemon.fill(None);
    for (species, level) in [
        ("AMPHAROS", 44),
        ("TYPHLOSION", 44),
        ("FERALIGATR", 44),
        ("ELECTABUZZ", 44),
    ] {
        let mut partner = r
            .data()
            .create_pokemon(species, level, Dv::from_non_hp(10, 10, 10, 10))
            .expect("ordinary native partner");
        if species == "FERALIGATR" {
            assert!(partner.species.tmhm_learnset.iter().any(|m| m == "SURF"));
            // Previously taught legal Surf; all later actions use native input.
            partner.moves[0] = LearnedMove {
                name: "SURF".into(),
                current_pp: r.data().moves["SURF"].pp,
                pp_ups: 0,
            };
        }
        assert!(state.storage.party.add_pokemon(partner));
    }
    assert!(
        state
            .bag
            .add_item(&r.data().items["HYPER_POTION"], 6)
            .expect("initial carried medicine")
    );
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert!(!can_approach(&mut c, &r, (9, 6)));
    assert!(!can_approach(&mut c, &r, (8, 3)));
    // The east launch is genuinely closed until Livra's win.
    walk(&mut c, "DRRRRRUU");
    assert_eq!(at(&mut c), (11, 16));
    press(&mut c, GameButton::Up);
    assert_eq!(at(&mut c), (11, 16));
    walk(&mut c, "DDLLLLLLLLLUU");
    assert_eq!(at(&mut c), (2, 16));
    slide(
        &mut c,
        &r,
        (2, 16),
        (2, 8),
        GameButton::Up,
        GameButton::Down,
    );
    slide(
        &mut c,
        &r,
        (2, 8),
        (4, 8),
        GameButton::Right,
        GameButton::Left,
    );
    walk(&mut c, "UUUL");
    assert_eq!(at(&mut c), (3, 5));
    talk(&mut c, GameButton::Up);
    ice_battle(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_8_TRAINER_1_DEFEATED"));
    assert!(event(&c, "EVENT_MN_GYM_8_WEST_RETURN_OPEN"));
    assert!(event(&c, "EVENT_MN_GYM_8_EAST_ENTRY_OPEN"));
    assert_eq!(gym_collision(&mut c, &r, (4, 14)), "FLOOR");
    assert_eq!(gym_collision(&mut c, &r, (11, 14)), "ICE");
    heal(&mut c, &r);
    c = continue_save(&mut c, &root, &r);
    walk(&mut c, "RDDR");
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_8_TRAINER_3_DEFEATED"));
    assert!(!can_approach(&mut c, &r, (8, 3)));
    walk(&mut c, "LD");
    slide(
        &mut c,
        &r,
        (4, 8),
        (4, 14),
        GameButton::Down,
        GameButton::Up,
    );
    walk(&mut c, "DDDDRRRRRRRUU");
    assert_eq!(at(&mut c), (11, 16));
    slide(
        &mut c,
        &r,
        (11, 16),
        (11, 10),
        GameButton::Up,
        GameButton::Down,
    );
    slide(
        &mut c,
        &r,
        (11, 10),
        (9, 10),
        GameButton::Left,
        GameButton::Right,
    );
    c = continue_save(&mut c, &root, &r);
    slide(
        &mut c,
        &r,
        (9, 10),
        (9, 7),
        GameButton::Up,
        GameButton::Down,
    );
    // Retreat before Tordel is legal; the cross gate remains closed.
    slide(
        &mut c,
        &r,
        (9, 7),
        (9, 10),
        GameButton::Down,
        GameButton::Up,
    );
    slide(
        &mut c,
        &r,
        (9, 10),
        (11, 10),
        GameButton::Right,
        GameButton::Left,
    );
    slide(
        &mut c,
        &r,
        (11, 10),
        (11, 16),
        GameButton::Down,
        GameButton::Up,
    );
    assert!(!event(&c, "EVENT_MN_GYM_8_EAST_RETURN_OPEN"));
    slide(
        &mut c,
        &r,
        (11, 16),
        (11, 10),
        GameButton::Up,
        GameButton::Down,
    );
    slide(
        &mut c,
        &r,
        (11, 10),
        (9, 10),
        GameButton::Left,
        GameButton::Right,
    );
    slide(
        &mut c,
        &r,
        (9, 10),
        (9, 7),
        GameButton::Up,
        GameButton::Down,
    );
    talk(&mut c, GameButton::Up);
    ice_battle(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_8_EAST_RETURN_OPEN"));
    assert!(!event(&c, "EVENT_MN_GYM_8_COURT_OPEN"));
    c = continue_save(&mut c, &root, &r);
    walk(&mut c, "LLLL");
    talk(&mut c, GameButton::Up);
    ice_battle(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_8_TRAINER_3_DEFEATED"));
    assert!(event(&c, "EVENT_MN_GYM_8_COURT_OPEN"));
    heal(&mut c, &r);
    walk(&mut c, "RUUUUR");
    assert_eq!(at(&mut c), (7, 3));
    talk(&mut c, GameButton::Right);
    ice_battle(&mut c);
    assert!(c.shell.shell.session().state().badges.johto[7]);
    assert!(event(&c, "EVENT_MN_GYM_8_DEFEATED"));
    let tm = &r.data().items["TM_ICY_WIND"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    c = continue_save(&mut c, &root, &r);
    let party = c.snapshot().expect("before repeat").party;
    talk(&mut c, GameButton::Right);
    finish(&mut c);
    assert_eq!(c.snapshot().expect("no rematch").party, party);
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    walk(&mut c, "LDDDDLLD");
    slide(
        &mut c,
        &r,
        (4, 8),
        (4, 14),
        GameButton::Down,
        GameButton::Up,
    );
    walk(&mut c, "DDDDRR");
    travel(&mut c, GameButton::Down, "MnDuluth", (16, 12));
    travel(&mut c, GameButton::Up, "MnDuluthGym", (6, 17));
    walk(&mut c, "DR");
    travel(&mut c, GameButton::Down, "MnDuluth", (16, 12));
    assert!(event(&c, "EVENT_MN_GYM_8_COURT_OPEN"));
}
