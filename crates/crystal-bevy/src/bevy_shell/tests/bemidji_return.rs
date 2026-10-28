//! Literal Cut-garden route and a shorter victory return, using ordinary partners.
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};

use super::super::winona::travel;
use super::rochester_readable::{talk, walk};
use super::{
    VisibleShellController, at, battle, can_approach, continue_save, controller, event, finish,
    gym_collision, load, mark_runtime_snapshot_dirty, press,
};

fn prune(
    c: &mut VisibleShellController,
    r: &crystal_runtime::CrystalRuntime,
    from: (u16, u16),
    tree: (u16, u16),
    flag: &str,
) {
    assert_eq!(at(c), from);
    assert_eq!(gym_collision(c, r, tree), "CUT_TREE");
    press(c, GameButton::Up);
    assert_eq!(at(c), from, "the native tree blocks walking");
    for _ in 0..64 {
        if gym_collision(c, r, tree) == "FLOOR" {
            break;
        }
        press(c, GameButton::A);
    }
    finish(c);
    c.wait_frames(80).expect("native Cut animation settles");
    assert_eq!(gym_collision(c, r, tree), "FLOOR");
    walk(c, "U");
    assert_eq!(at(c), tree);
    assert!(event(c, flag));
}

#[test]
#[ignore = "requires minnesota-bemidji-return.crystalpack via CRYSTAL_GYM_TEST_PACK"]
fn bemidji_return_literal_ordinary_party_native_cut_both_trainers_and_short_return() {
    let (root, r, _) = load();
    let design: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../planning/minnesota/gyms/bemidji-return.json"
    ))
    .expect("return source");
    let mut c = controller(&root, &r, &design["gyms"][0], false);
    // Initial fixtures: six prior badges, entrance, native ordinary partners,
    // a previously taught legal Cut/HM and the shared deterministic divider.
    // All seventh-gym victories, both prunings and the return gate are earned.
    let state = c.shell.shell.session_mut().state_mut();
    state.storage.party.pokemon.fill(None);
    for (species, level) in [("TYPHLOSION", 41), ("CROBAT", 40), ("SANDSLASH", 39)] {
        let mut partner = r
            .data()
            .create_pokemon(species, level, Dv::default())
            .expect("ordinary partner");
        if species == "SANDSLASH" {
            assert!(partner.species.tmhm_learnset.iter().any(|m| m == "CUT"));
            partner.moves[0] = LearnedMove {
                name: "CUT".into(),
                current_pp: r.data().moves["CUT"].pp,
                pp_ups: 0,
            };
        }
        assert!(state.storage.party.add_pokemon(partner));
    }
    assert!(
        state
            .bag
            .add_item(&r.data().items["HM_CUT"], 1)
            .expect("previously owned HM")
    );
    assert!(
        state.storage.party.pokemon[0]
            .as_ref()
            .expect("Typhlosion")
            .moves
            .iter()
            .any(|m| m.name == "FLAME_WHEEL")
    );
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert!(!can_approach(&mut c, &r, (8, 9)));
    assert!(!can_approach(&mut c, &r, (5, 3)));
    assert_eq!(gym_collision(&mut c, &r, (7, 12)), "WALL");
    walk(&mut c, "UUURR");
    talk(&mut c, GameButton::Right);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_7_TRAINER_1_DEFEATED"));
    walk(&mut c, "LLLU");
    prune(
        &mut c,
        &r,
        (3, 13),
        (3, 12),
        "EVENT_MN_GYM_7_CUT_SOUTH_HEDGE_PRUNED",
    );
    c = continue_save(&mut c, &root, &r);
    walk(&mut c, "ULUUURRRRRR");
    talk(&mut c, GameButton::Down);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_7_TRAINER_2_DEFEATED"));
    assert!(
        !event(&c, "EVENT_MN_GYM_7_TRAINER_3_DEFEATED"),
        "Brindel remains optional"
    );
    assert!(!event(&c, "EVENT_MN_GYM_7_RETURN_OPEN"));
    assert!(!can_approach(&mut c, &r, (5, 3)));
    walk(&mut c, "LU");
    prune(
        &mut c,
        &r,
        (7, 7),
        (7, 6),
        "EVENT_MN_GYM_7_CUT_NORTH_HEDGE_PRUNED",
    );
    c = continue_save(&mut c, &root, &r);
    assert_eq!(gym_collision(&mut c, &r, (7, 12)), "WALL");
    walk(&mut c, "ULLU");
    talk(&mut c, GameButton::Up);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_7_RETURN_OPEN"));
    assert!(c.shell.shell.session().state().badges.johto[6]);
    assert_eq!(gym_collision(&mut c, &r, (7, 12)), "FLOOR");
    let tm = &r.data().items["TM_GIGA_DRAIN"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    c = continue_save(&mut c, &root, &r);
    let party = c.snapshot().expect("before repeat").party;
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    assert_eq!(c.snapshot().expect("no rematch").party, party);
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    let walked = walk(&mut c, concat!("RR", "DDDDDDDDD", "L", "DD", "LL", "DD"));
    assert_eq!(
        walked.len() - 1,
        18,
        "literal shortcut reaches the entrance in eighteen steps"
    );
    assert_eq!(at(&mut c), (4, 17));
    walk(&mut c, "D");
    travel(&mut c, GameButton::Down, "MnBemidji", (10, 10));
    travel(&mut c, GameButton::Up, "MnBemidjiGym", (4, 17));
    walk(&mut c, "DR");
    travel(&mut c, GameButton::Down, "MnBemidji", (10, 10));
    assert!(event(&c, "EVENT_MN_GYM_7_CUT_SOUTH_HEDGE_PRUNED"));
    assert!(event(&c, "EVENT_MN_GYM_7_CUT_NORTH_HEDGE_PRUNED"));
}
