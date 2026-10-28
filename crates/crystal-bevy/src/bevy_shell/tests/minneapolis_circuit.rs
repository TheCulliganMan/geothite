//! Literal staged circuit; only entrance/prior badges/ordinary party are fixtures.
use crystal_core::input::GameButton;
use crystal_core::models::Dv;

use super::super::winona::travel;
use super::rochester_readable::{talk, walk};
use super::{
    assert_switches, at, battle, can_approach, continue_save, controller, event, finish, load,
    mark_runtime_snapshot_dirty, press,
};

#[test]
#[ignore = "requires minnesota-minneapolis-circuit.crystalpack via CRYSTAL_GYM_TEST_PACK"]
fn minneapolis_circuit_literal_ordinary_party_reset_continue_and_both_city_returns() {
    let (root, r, _) = load();
    let design: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../planning/minnesota/gyms/minneapolis-circuit.json"
    ))
    .expect("staged source");
    let mut c = controller(&root, &r, &design["gyms"][0], false);
    // Explicit initial fixture, before all input: prior badges, ordinary native
    // level-up partners and the shared deterministic test divider.
    let state = c.shell.shell.session_mut().state_mut();
    state.storage.party.pokemon.fill(None);
    for species in ["GEODUDE", "BAYLEEF"] {
        assert!(
            state.storage.party.add_pokemon(
                r.data()
                    .create_pokemon(species, 23, Dv::default())
                    .expect("native partner")
            )
        );
    }
    assert!(
        state.storage.party.pokemon[0]
            .as_ref()
            .expect("Geodude")
            .moves
            .iter()
            .any(|m| m.name == "MAGNITUDE")
    );
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert!(!can_approach(&mut c, &r, (4, 6)));
    assert!(!can_approach(&mut c, &r, (4, 3)));
    walk(&mut c, "UUUL");
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    assert_switches(&mut c, false, false);
    walk(&mut c, "LL");
    talk(&mut c, GameButton::Down);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_3_TRAINER_1_DEFEATED"));
    walk(&mut c, "RRRR");
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    assert_switches(&mut c, false, false);
    walk(&mut c, "LL");
    talk(&mut c, GameButton::Up);
    let before = at(&mut c);
    press(&mut c, GameButton::Right);
    assert_eq!(at(&mut c), before, "switch text owns movement");
    finish(&mut c);
    assert_switches(&mut c, true, false);
    assert!(can_approach(&mut c, &r, (4, 6)));
    assert!(!can_approach(&mut c, &r, (4, 3)));
    c = continue_save(&mut c, &root, &r);
    walk(&mut c, "RR");
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    assert_switches(&mut c, true, false);
    walk(&mut c, "LUUUUU");
    talk(&mut c, GameButton::Up);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_3_TRAINER_2_DEFEATED"));
    walk(&mut c, "DDDRRR");
    talk(&mut c, GameButton::Down);
    finish(&mut c);
    assert_switches(&mut c, false, false);
    assert!(event(&c, "EVENT_MN_GYM_3_TRAINER_1_DEFEATED"));
    assert!(event(&c, "EVENT_MN_GYM_3_TRAINER_2_DEFEATED"));
    c = continue_save(&mut c, &root, &r);
    walk(&mut c, "LLL");
    for _ in 0..4 {
        press(&mut c, GameButton::Up);
    }
    assert_eq!(at(&mut c), (4, 10), "reset restores solid first barrier");
    walk(&mut c, "L");
    talk(&mut c, GameButton::Down);
    finish(&mut c);
    walk(&mut c, "RR");
    talk(&mut c, GameButton::Down);
    finish(&mut c);
    assert_switches(&mut c, true, true);
    c = continue_save(&mut c, &root, &r);
    walk(&mut c, "RRRDDL");
    talk(&mut c, GameButton::Down);
    finish(&mut c);
    assert_switches(&mut c, true, true);
    walk(&mut c, "LLLUUUUURUUUU");
    assert_eq!(at(&mut c), (5, 3));
    talk(&mut c, GameButton::Left);
    battle(&mut c, false);
    finish(&mut c);
    assert!(c.shell.shell.session().state().badges.johto[2]);
    let tm = &r.data().items["TM_THUNDERPUNCH"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    c = continue_save(&mut c, &root, &r);
    talk(&mut c, GameButton::Left);
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    walk(&mut c, "DDDDLDDDDDDDDD");
    travel(&mut c, GameButton::Down, "MnMinneapolis", (42, 24));
    travel(&mut c, GameButton::Up, "MnMinneapolisGym", (4, 15));
    walk(&mut c, "DR");
    travel(&mut c, GameButton::Down, "MnMinneapolis", (42, 24));
}

#[test]
#[ignore = "requires minnesota-minneapolis-circuit.crystalpack via CRYSTAL_GYM_TEST_PACK"]
fn minneapolis_circuit_loss_inside_first_lock_preserves_win_and_refuses_second() {
    let (root, r, _) = load();
    let design: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../planning/minnesota/gyms/minneapolis-circuit.json"
    ))
    .expect("staged source");
    let mut c = controller(&root, &r, &design["gyms"][0], true);
    // Explicit isolated second-stage loss fixture: the first win and switch
    // are supplied at setup. This test does not claim to earn either one.
    for flag in [
        "EVENT_MN_GYM_3_TRAINER_1_DEFEATED",
        "EVENT_MN_GYM_3_FIRST_SWITCH_ON",
    ] {
        c.shell
            .shell
            .session_mut()
            .state_mut()
            .flags
            .set_event_flag(flag, true)
            .expect("initial staged-loss fixture");
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    // Enter normally so the authored map callback projects the fixture flags
    // into barrier tiles before checking persistence.
    walk(&mut c, "D");
    travel(&mut c, GameButton::Down, "MnMinneapolis", (42, 24));
    travel(&mut c, GameButton::Up, "MnMinneapolisGym", (4, 15));
    c = continue_save(&mut c, &root, &r);
    walk(&mut c, "UUUUUUUU");
    talk(&mut c, GameButton::Up);
    battle(&mut c, true);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_3_TRAINER_1_DEFEATED"));
    assert!(event(&c, "EVENT_MN_GYM_3_FIRST_SWITCH_ON"));
    assert!(!event(&c, "EVENT_MN_GYM_3_TRAINER_2_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_GYM_3_SECOND_SWITCH_ON"));
    assert!(!event(&c, "EVENT_MN_GYM_3_DEFEATED"));
    assert!(!c.shell.shell.session().state().badges.johto[2]);
    assert_ne!(
        c.snapshot().expect("ordinary recovery").overworld.map_name,
        "MnMinneapolisGym"
    );
}

#[test]
#[ignore = "requires minnesota-minneapolis-circuit.crystalpack via CRYSTAL_GYM_TEST_PACK"]
fn minneapolis_circuit_both_locks_do_not_replace_second_badge() {
    let (root, r, _) = load();
    let design: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../planning/minnesota/gyms/minneapolis-circuit.json"
    ))
    .expect("staged source");
    let mut c = controller(&root, &r, &design["gyms"][0], false);
    // Missing prior badge is an initial negative fixture; all gym progress is earned.
    c.shell.shell.session_mut().state_mut().badges.johto[1] = false;
    mark_runtime_snapshot_dirty(&mut c.shell);
    walk(&mut c, "UUULLL");
    talk(&mut c, GameButton::Down);
    battle(&mut c, false);
    finish(&mut c);
    walk(&mut c, "RR");
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    walk(&mut c, "RUUUUU");
    talk(&mut c, GameButton::Up);
    battle(&mut c, false);
    finish(&mut c);
    walk(&mut c, "DDDR");
    talk(&mut c, GameButton::Down);
    finish(&mut c);
    assert_switches(&mut c, true, true);
    walk(&mut c, "UUUUUUU");
    talk(&mut c, GameButton::Left);
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_3_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_GYM_3_TM_RECEIVED"));
    assert!(!c.shell.shell.session().state().badges.johto[2]);
    assert!(c.snapshot().expect("leader refusal").battle.is_none());
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["TM_THUNDERPUNCH"]),
        0
    );
}
