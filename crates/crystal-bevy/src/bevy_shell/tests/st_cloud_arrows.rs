//! Literal wind puzzle with ordinary partners and no pathfinder steering.
use crystal_core::input::GameButton;
use crystal_core::models::Dv;

use super::super::winona::travel;
use super::gym_care::heal;
use super::rochester_readable::{talk, walk};
use super::{
    VisibleShellController, at, battle, can_approach, continue_save, controller, event, finish,
    load, mark_runtime_snapshot_dirty, press,
};

fn gust(
    c: &mut VisibleShellController,
    from: (u16, u16),
    to: (u16, u16),
    forward: GameButton,
    backward: GameButton,
) {
    assert_eq!(at(c), from);
    if from == to {
        for _ in 0..3 {
            press(c, forward);
            c.wait_frames(80).expect("opposing wind returns to bank");
            assert_eq!(at(c), from);
        }
        return;
    }
    for _ in 0..4 {
        c.press(forward).expect("enter native wind");
        c.wait_frames(1).expect("observe wind entry");
        if at(c) != from {
            break;
        }
    }
    assert_ne!(at(c), from);
    assert_ne!(
        at(c),
        to,
        "observe the actual forced movement before landing"
    );
    c.press(backward)
        .expect("opposing input cannot override wind");
    c.wait_frames(160)
        .expect("native wind carries player without more input");
    assert_eq!(at(c), to);
    c.wait_frames(80).expect("safe stone landing");
    assert_eq!(at(c), to);
}

#[test]
#[ignore = "requires minnesota-st-cloud-arrows.crystalpack via CRYSTAL_GYM_TEST_PACK"]
fn st_cloud_arrows_literal_ordinary_party_all_four_lanes_and_both_city_returns() {
    let (root, r, _) = load();
    let design: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../planning/minnesota/gyms/st-cloud-arrows.json"
    ))
    .expect("arrow course");
    let mut c = controller(&root, &r, &design["gyms"][0], false);
    // Initial fixtures only: entrance, four prior badges, native level-up party
    // and shared deterministic divider. All fifth-gym progress is earned below.
    let state = c.shell.shell.session_mut().state_mut();
    state.storage.party.pokemon.fill(None);
    for (species, level) in [("AMPHAROS", 31), ("GRAVELER", 30)] {
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
            .expect("Ampharos")
            .moves
            .iter()
            .any(|m| m.name == "THUNDERPUNCH")
    );
    assert!(
        state
            .bag
            .add_item(&r.data().items["HYPER_POTION"], 4)
            .expect("initial carried medicine")
    );
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert!(!can_approach(&mut c, &r, (14, 10)));
    assert!(!can_approach(&mut c, &r, (3, 3)));
    walk(&mut c, "UUURRR");
    gust(
        &mut c,
        (7, 16),
        (7, 16),
        GameButton::Right,
        GameButton::Right,
    );
    walk(&mut c, "LLL");
    talk(&mut c, GameButton::Up);
    battle(&mut c, false);
    finish(&mut c);
    heal(&mut c, &r);
    assert!(event(&c, "EVENT_MN_GYM_5_TRAINER_1_DEFEATED"));
    assert!(event(&c, "EVENT_MN_GYM_5_LOWER_CROSSING_OPEN"));
    assert!(!can_approach(&mut c, &r, (3, 3)));
    c = continue_save(&mut c, &root, &r);
    walk(&mut c, "RUUURR");
    gust(
        &mut c,
        (7, 13),
        (12, 13),
        GameButton::Right,
        GameButton::Left,
    );
    c = continue_save(&mut c, &root, &r);
    walk(&mut c, "RRRUUUUUUUUULLL");
    gust(&mut c, (12, 4), (12, 4), GameButton::Left, GameButton::Left);
    assert!(!can_approach(&mut c, &r, (3, 3)));
    walk(&mut c, "RRDDDDD");
    talk(&mut c, GameButton::Down);
    battle(&mut c, false);
    finish(&mut c);
    heal(&mut c, &r);
    assert!(event(&c, "EVENT_MN_GYM_5_TRAINER_2_DEFEATED"));
    assert!(event(&c, "EVENT_MN_GYM_5_COURT_OPEN"));
    c = continue_save(&mut c, &root, &r);
    walk(&mut c, "LL");
    gust(&mut c, (12, 9), (7, 9), GameButton::Left, GameButton::Right);
    walk(&mut c, "LLLUUUUUU");
    talk(&mut c, GameButton::Left);
    battle(&mut c, false);
    finish(&mut c);
    assert!(c.shell.shell.session().state().badges.johto[4]);
    let tm = &r.data().items["TM_SWIFT"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    c = continue_save(&mut c, &root, &r);
    let party = c.snapshot().expect("before repeat").party;
    talk(&mut c, GameButton::Left);
    finish(&mut c);
    assert_eq!(c.snapshot().expect("repeat does not rematch").party, party);
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    walk(&mut c, "DDRRR");
    gust(&mut c, (7, 5), (12, 5), GameButton::Right, GameButton::Left);
    walk(&mut c, "RRDDDLDDDDDDDDDL");
    gust(
        &mut c,
        (12, 17),
        (7, 17),
        GameButton::Left,
        GameButton::Right,
    );
    walk(&mut c, "LLLDDD");
    travel(&mut c, GameButton::Down, "MnStCloud", (8, 28));
    travel(&mut c, GameButton::Up, "MnStCloudGym", (4, 19));
    walk(&mut c, "DR");
    travel(&mut c, GameButton::Down, "MnStCloud", (8, 28));
}
