//! Literal pool course with a physical control, ordinary partners and real exits.
use crystal_core::input::GameButton;
use crystal_core::models::Dv;

use super::super::winona::travel;
use super::rochester_readable::{talk, walk};
use super::{
    VisibleShellController, at, battle, battle_with_partner, can_approach, continue_save,
    controller, event, finish, gym_collision, load, mark_runtime_snapshot_dirty, press,
};

fn pool_battle(c: &mut VisibleShellController) {
    let choose: fn(&str) -> Option<&'static str> = |enemy| match enemy {
        "POLIWAG" | "GOLDEEN" | "POLIWHIRL" => Some("AMPHAROS"),
        "WOOPER" | "CHINCHOU" => Some("SUNFLORA"),
        "QUAGSIRE" | "LANTURN" => Some("VENUSAUR"),
        _ => None,
    };
    battle_with_partner(c, false, Some(choose));
}

fn toggle(c: &mut VisibleShellController) {
    assert_eq!(at(c), (5, 13));
    talk(c, GameButton::Right);
    assert!(
        c.snapshot().expect("physical winch text").ui.menu.is_none(),
        "A changes the pier without a menu"
    );
    let stand = at(c);
    press(c, GameButton::Down);
    assert_eq!(at(c), stand, "winch text owns input");
    finish(c);
}
fn mode(c: &mut VisibleShellController, center: bool, left: bool, court: bool) {
    assert_eq!(event(c, "EVENT_MN_GYM_6_WALK_CENTER"), center);
    assert_eq!(event(c, "EVENT_MN_GYM_6_CENTER_WALK_OPEN"), center);
    assert_eq!(event(c, "EVENT_MN_GYM_6_WALK_LEFT"), left);
    assert_eq!(event(c, "EVENT_MN_GYM_6_LEFT_WALK_OPEN"), court);
    assert_eq!(event(c, "EVENT_MN_GYM_6_COURT_OPEN"), court);
}

#[test]
#[ignore = "requires minnesota-brainerd-winch.crystalpack via CRYSTAL_GYM_TEST_PACK"]
fn brainerd_winch_literal_ordinary_party_reversals_continue_and_both_city_returns() {
    let (root, r, _) = load();
    let design: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../planning/minnesota/gyms/brainerd-winch.json"
    ))
    .expect("winch source");
    let mut c = controller(&root, &r, &design["gyms"][0], false);
    // Initial local fixtures only: five prior badges, entrance, native level-up
    // partners and shared deterministic divider. No gym6 progress is supplied.
    let state = c.shell.shell.session_mut().state_mut();
    state.storage.party.pokemon.fill(None);
    for (species, level) in [("SUNFLORA", 36), ("AMPHAROS", 35), ("VENUSAUR", 36)] {
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
            .expect("Sunflora")
            .moves
            .iter()
            .any(|m| m.name == "RAZOR_LEAF")
    );
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    walk(&mut c, "RU");
    toggle(&mut c);
    mode(&mut c, false, false, false);
    assert!(!can_approach(&mut c, &r, (2, 7)));
    assert!(!can_approach(&mut c, &r, (5, 3)));
    walk(&mut c, "LUUUL");
    talk(&mut c, GameButton::Left);
    pool_battle(&mut c);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_6_TRAINER_1_DEFEATED"));
    talk(&mut c, GameButton::Left);
    assert!(
        c.snapshot()
            .expect("Lorla repeat is a clue")
            .ui
            .menu
            .is_none()
    );
    finish(&mut c);
    mode(&mut c, false, false, false);
    walk(&mut c, "RRDDD");
    toggle(&mut c);
    mode(&mut c, true, false, false);
    assert_eq!(gym_collision(&mut c, &r, (5, 9)), "FLOOR");
    assert_eq!(gym_collision(&mut c, &r, (3, 9)), "WATER");
    // Pressing again too soon retracts the center walk; the control remains safe.
    toggle(&mut c);
    mode(&mut c, false, true, false);
    assert!(!can_approach(&mut c, &r, (2, 7)));
    assert!(!can_approach(&mut c, &r, (5, 3)));
    toggle(&mut c);
    mode(&mut c, true, false, false);
    walk(&mut c, "UUUU");
    assert_eq!(at(&mut c), (5, 9));
    c = continue_save(&mut c, &root, &r);
    walk(&mut c, "UULL");
    talk(&mut c, GameButton::Left);
    pool_battle(&mut c);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_6_TRAINER_2_DEFEATED"));
    mode(&mut c, true, false, false);
    walk(&mut c, "RRDDDDDD");
    toggle(&mut c);
    mode(&mut c, false, true, true);
    assert_eq!(gym_collision(&mut c, &r, (5, 9)), "WATER");
    assert_eq!(gym_collision(&mut c, &r, (3, 9)), "FLOOR");
    walk(&mut c, "UUU");
    for _ in 0..3 {
        press(&mut c, GameButton::Up);
    }
    assert_eq!(
        at(&mut c),
        (5, 10),
        "the retracted center pier is genuinely water"
    );
    walk(&mut c, "DDD");
    toggle(&mut c);
    mode(&mut c, true, false, false);
    assert!(!can_approach(&mut c, &r, (5, 3)));
    toggle(&mut c);
    mode(&mut c, false, true, true);
    assert!(
        c.snapshot()
            .expect("prepared leader attempt")
            .party
            .slots
            .iter()
            .all(|p| p.pokemon.hp > 0)
    );
    walk(&mut c, "UUULLUUURRRRUUULL");
    assert_eq!(at(&mut c), (5, 4));
    c = continue_save(&mut c, &root, &r);
    talk(&mut c, GameButton::Up);
    pool_battle(&mut c);
    finish(&mut c);
    assert!(c.shell.shell.session().state().badges.johto[5]);
    let tm = &r.data().items["TM_RAIN_DANCE"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    c = continue_save(&mut c, &root, &r);
    let party = c.snapshot().expect("before repeat").party;
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    assert_eq!(c.snapshot().expect("no rematch").party, party);
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    walk(&mut c, "RRDDDLLLLDDDRRDDDD");
    travel(&mut c, GameButton::Down, "MnBrainerd", (12, 36));
    travel(&mut c, GameButton::Up, "MnBrainerdGym", (4, 14));
    travel(&mut c, GameButton::Down, "MnBrainerd", (12, 36));
}

#[test]
#[ignore = "requires minnesota-brainerd-winch.crystalpack via CRYSTAL_GYM_TEST_PACK"]
fn brainerd_winch_solved_piers_still_require_fifth_badge() {
    let (root, r, _) = load();
    let design: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../planning/minnesota/gyms/brainerd-winch.json"
    ))
    .expect("winch source");
    let mut c = controller(&root, &r, &design["gyms"][0], false);
    // Isolated permission fixture: Mew100 and an initially missing badge5.
    c.shell.shell.session_mut().state_mut().badges.johto[4] = false;
    mark_runtime_snapshot_dirty(&mut c.shell);
    walk(&mut c, "UUUUL");
    talk(&mut c, GameButton::Left);
    battle(&mut c, false);
    finish(&mut c);
    walk(&mut c, "RRDDD");
    toggle(&mut c);
    walk(&mut c, "UUUUUULL");
    talk(&mut c, GameButton::Left);
    battle(&mut c, false);
    finish(&mut c);
    walk(&mut c, "RRDDDDDD");
    toggle(&mut c);
    walk(&mut c, "UUULLUUURRRRUUULL");
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_6_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_GYM_6_TM_RECEIVED"));
    assert!(!c.shell.shell.session().state().badges.johto[5]);
    assert!(c.snapshot().expect("leader refusal").battle.is_none());
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["TM_RAIN_DANCE"]),
        0
    );
}
