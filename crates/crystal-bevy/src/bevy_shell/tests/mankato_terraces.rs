//! Literal native stair/drop course with ordinary partners and real city exits.
use crystal_core::input::GameButton;
use crystal_core::models::Dv;

use super::super::winona::travel;
use super::rochester_readable::{talk, walk};
use super::{
    at, battle, can_approach, can_approach_with_ledges, continue_save, controller, event, finish,
    load, mark_runtime_snapshot_dirty, press,
};

fn drop_south(c: &mut super::VisibleShellController, from: (u16, u16), to: (u16, u16)) {
    assert_eq!(at(c), from);
    for _ in 0..8 {
        press(c, GameButton::Down);
        if at(c) != from {
            break;
        }
    }
    c.wait_frames(80).expect("settle real hop");
    assert_eq!(at(c), to, "one native two-tile downward jump");
    for _ in 0..4 {
        press(c, GameButton::Up);
    }
    assert_eq!(at(c), to, "the low edge cannot be climbed backward");
}

#[test]
#[ignore = "requires minnesota-mankato-terraces.crystalpack via CRYSTAL_GYM_TEST_PACK"]
fn mankato_terraces_literal_stairs_drops_ordinary_party_and_both_city_returns() {
    let (root, r, design) = load();
    let gym = &design["gyms"][1];
    let mut c = controller(&root, &r, gym, false);
    // Local initial fixture: prior Care Badge, entrance, native level-up party
    // and the shared deterministic test divider. No second-gym wins are supplied.
    let state = c.shell.shell.session_mut().state_mut();
    state.storage.party.pokemon.fill(None);
    for (species, level) in [("BAYLEEF", 19), ("PIDGEY", 17)] {
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
            .expect("Bayleef")
            .moves
            .iter()
            .any(|m| m.name == "RAZOR_LEAF")
    );
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert!(!can_approach(&mut c, &r, (11, 11)));
    assert!(!can_approach(&mut c, &r, (11, 3)));
    walk(&mut c, "L");
    talk(&mut c, GameButton::Left);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_2_TRAINER_1_DEFEATED"));
    assert!(event(&c, "EVENT_MN_GYM_2_LOWER_STAIR_OPEN"));
    assert!(
        !can_approach_with_ledges(&mut c, &r, (11, 11), false),
        "the first downward jump is required, not a painted edge"
    );
    walk(&mut c, "ULLUUUUUUUUURRRD");
    drop_south(&mut c, (6, 8), (6, 10));
    // Retreat before Jorel without fabricating her victory or opening the court.
    walk(&mut c, "RRRRDDDD");
    drop_south(&mut c, (10, 14), (10, 16));
    assert!(!event(&c, "EVENT_MN_GYM_2_TRAINER_2_DEFEATED"));
    walk(&mut c, "DLLLLL");
    assert_eq!(at(&mut c), (5, 17));
    c = continue_save(&mut c, &root, &r);
    walk(&mut c, "ULLUUUUUUUUURRRD");
    drop_south(&mut c, (6, 8), (6, 10));
    walk(&mut c, "RRRRD");
    talk(&mut c, GameButton::Right);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_2_TRAINER_2_DEFEATED"));
    assert!(event(&c, "EVENT_MN_GYM_2_COURT_OPEN"));
    c = continue_save(&mut c, &root, &r);
    walk(&mut c, "URRUUUUUUU");
    assert_eq!(at(&mut c), (12, 3));
    talk(&mut c, GameButton::Left);
    battle(&mut c, false);
    finish(&mut c);
    assert!(c.shell.shell.session().state().badges.johto[1]);
    let tm = &r.data().items["TM_MUD_SLAP"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    c = continue_save(&mut c, &root, &r);
    talk(&mut c, GameButton::Left);
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    walk(&mut c, "DLLDD");
    drop_south(&mut c, (10, 6), (10, 8));
    walk(&mut c, "DDDDDD");
    drop_south(&mut c, (10, 14), (10, 16));
    walk(&mut c, "DLLLLD");
    travel(&mut c, GameButton::Down, "MnMankato", (10, 12));
    travel(&mut c, GameButton::Up, "MnMankatoGym", (6, 17));
    walk(&mut c, "DR");
    travel(&mut c, GameButton::Down, "MnMankato", (10, 12));
    assert!(event(&c, "EVENT_MN_GYM_2_TRAINER_1_DEFEATED"));
    assert!(event(&c, "EVENT_MN_GYM_2_TRAINER_2_DEFEATED"));
}
