//! Literal reciprocal pad course, normal partners and genuine trainer wins.
use crystal_core::input::GameButton;
use crystal_core::models::Dv;

use super::super::winona::travel;
use super::rochester_readable::{talk, walk};
use super::{
    VisibleShellController, at, battle, can_approach, continue_save, controller, event, finish,
    load, mark_runtime_snapshot_dirty, press,
};

fn ride(c: &mut VisibleShellController, button: GameButton, from: (u16, u16), to: (u16, u16)) {
    let stand = at(c);
    assert_eq!(stand.0.abs_diff(from.0) + stand.1.abs_diff(from.1), 1);
    for _ in 0..8 {
        press(c, button);
        if at(c) != stand {
            break;
        }
    }
    c.wait_frames(160).expect("settle native reciprocal pad");
    assert_eq!(at(c), to, "paired native warp destination");
    c.wait_frames(100).expect("arrival does not bounce");
    assert_eq!(at(c), to);
}

#[test]
#[ignore = "requires minnesota-st-paul-loop.crystalpack via CRYSTAL_GYM_TEST_PACK"]
fn st_paul_loop_literal_ordinary_party_four_pairs_and_short_city_return() {
    let (root, r, _) = load();
    let design: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../planning/minnesota/gyms/st-paul-loop.json"
    ))
    .expect("loop source");
    let mut c = controller(&root, &r, &design["gyms"][0], false);
    // Initial fixtures: entrance, three prior badges, native level-up partners
    // and the shared deterministic divider. No fourth-gym progress is supplied.
    let state = c.shell.shell.session_mut().state_mut();
    state.storage.party.pokemon.fill(None);
    for (species, level) in [("QUILAVA", 27), ("GEODUDE", 25)] {
        assert!(
            state.storage.party.add_pokemon(
                r.data()
                    .create_pokemon(species, level, Dv::default())
                    .expect("ordinary native partner")
            )
        );
    }
    assert!(
        state.storage.party.pokemon[0]
            .as_ref()
            .expect("Quilava")
            .moves
            .iter()
            .any(|m| m.name == "EMBER")
    );
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert!(!can_approach(&mut c, &r, (11, 2)));
    walk(&mut c, "UUURRR");
    ride(&mut c, GameButton::Down, (5, 15), (11, 7));
    walk(&mut c, "U");
    for _ in 0..4 {
        press(&mut c, GameButton::Up);
    }
    assert_eq!(
        at(&mut c),
        (11, 6),
        "early return pad cannot bypass the grille"
    );
    ride(&mut c, GameButton::Down, (11, 7), (5, 15));
    walk(&mut c, "ULLLLUU");
    ride(&mut c, GameButton::Up, (1, 11), (1, 3));
    c = continue_save(&mut c, &root, &r);
    c.wait_frames(100).expect("Continue stays on arrival pad");
    assert_eq!(at(&mut c), (1, 3));
    walk(&mut c, "D");
    ride(&mut c, GameButton::Up, (1, 3), (1, 11));
    walk(&mut c, "R");
    ride(&mut c, GameButton::Left, (1, 11), (1, 3));
    walk(&mut c, "R");
    talk(&mut c, GameButton::Right);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_4_TRAINER_1_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_GYM_4_COURT_OPEN"));
    walk(&mut c, "DDDRRR");
    ride(&mut c, GameButton::Down, (5, 7), (9, 11));
    walk(&mut c, "R");
    ride(&mut c, GameButton::Left, (9, 11), (5, 7));
    walk(&mut c, "U");
    ride(&mut c, GameButton::Down, (5, 7), (9, 11));
    walk(&mut c, "RR");
    talk(&mut c, GameButton::Down);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_4_TRAINER_2_DEFEATED"));
    assert!(event(&c, "EVENT_MN_GYM_4_COURT_OPEN"));
    walk(&mut c, "RDDDDDL");
    ride(&mut c, GameButton::Down, (11, 17), (9, 7));
    walk(&mut c, "U");
    ride(&mut c, GameButton::Down, (9, 7), (11, 17));
    walk(&mut c, "L");
    ride(&mut c, GameButton::Right, (11, 17), (9, 7));
    c = continue_save(&mut c, &root, &r);
    walk(&mut c, "URRUUU");
    talk(&mut c, GameButton::Up);
    battle(&mut c, false);
    finish(&mut c);
    assert!(c.shell.shell.session().state().badges.johto[3]);
    let tm = &r.data().items["TM_STEEL_WING"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    c = continue_save(&mut c, &root, &r);
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    walk(&mut c, "DDD");
    ride(&mut c, GameButton::Down, (11, 7), (5, 15));
    walk(&mut c, "LLDDD");
    travel(&mut c, GameButton::Down, "MnStPaul", (54, 16));
    travel(&mut c, GameButton::Up, "MnStPaulGym", (2, 17));
    walk(&mut c, "D");
    travel(&mut c, GameButton::Down, "MnStPaul", (54, 16));
}

#[test]
#[ignore = "requires minnesota-st-paul-loop.crystalpack via CRYSTAL_GYM_TEST_PACK"]
fn st_paul_loop_reverse_order_still_requires_third_badge() {
    let (root, r, _) = load();
    let design: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../planning/minnesota/gyms/st-paul-loop.json"
    ))
    .expect("loop source");
    let mut c = controller(&root, &r, &design["gyms"][0], false);
    // Isolated permission negative: harness Mew100 and missing prior badge3.
    c.shell.shell.session_mut().state_mut().badges.johto[2] = false;
    mark_runtime_snapshot_dirty(&mut c.shell);
    walk(&mut c, "UUURRR");
    ride(&mut c, GameButton::Down, (5, 15), (11, 7));
    walk(&mut c, "L");
    ride(&mut c, GameButton::Left, (9, 7), (11, 17));
    walk(&mut c, "LUUUUU");
    talk(&mut c, GameButton::Right);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_4_TRAINER_2_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_GYM_4_COURT_OPEN"));
    walk(&mut c, "U");
    ride(&mut c, GameButton::Left, (9, 11), (5, 7));
    walk(&mut c, "LUUUU");
    talk(&mut c, GameButton::Left);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_4_COURT_OPEN"));
    walk(&mut c, "DDDR");
    ride(&mut c, GameButton::Down, (5, 7), (9, 11));
    walk(&mut c, "RRRDDDDDL");
    ride(&mut c, GameButton::Down, (11, 17), (9, 7));
    walk(&mut c, "URRUUU");
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_4_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_GYM_4_TM_RECEIVED"));
    assert!(!c.shell.shell.session().state().badges.johto[3]);
    assert!(c.snapshot().expect("leader refusal").battle.is_none());
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["TM_STEEL_WING"]),
        0
    );
}

#[test]
#[ignore = "requires minnesota-st-paul-loop.crystalpack via CRYSTAL_GYM_TEST_PACK"]
fn st_paul_loop_loss_leaves_court_locked() {
    let (root, r, _) = load();
    let design: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../planning/minnesota/gyms/st-paul-loop.json"
    ))
    .expect("loop source");
    let mut c = controller(&root, &r, &design["gyms"][0], true);
    // Explicit initial loss fixture: Magikarp5/1HP and three prior badges.
    walk(&mut c, "UUUUUL");
    ride(&mut c, GameButton::Up, (1, 11), (1, 3));
    walk(&mut c, "R");
    talk(&mut c, GameButton::Right);
    battle(&mut c, true);
    finish(&mut c);
    for flag in [
        "EVENT_MN_GYM_4_TRAINER_1_DEFEATED",
        "EVENT_MN_GYM_4_TRAINER_2_DEFEATED",
        "EVENT_MN_GYM_4_COURT_OPEN",
        "EVENT_MN_GYM_4_DEFEATED",
    ] {
        assert!(!event(&c, flag));
    }
    assert!(!c.shell.shell.session().state().badges.johto[3]);
    assert_ne!(
        c.snapshot().expect("ordinary recovery").overworld.map_name,
        "MnStPaulGym"
    );
}
