//! Literal Duluth course through native slides, wins and city returns.
use crystal_core::input::GameButton;

use super::super::winona::travel;
use super::rochester_readable::{talk, walk};
use super::{
    at, battle, can_approach, continue_save, controller, event, finish, gym_collision, load, press,
    slide_gym_ice,
};

#[test]
#[ignore = "requires the eight-gym harbor-ice review pack via CRYSTAL_GYM_TEST_PACK"]
fn duluth_harbor_ice_literal_slides_retreat_victory_and_both_exits() {
    let (root, r, _) = load();
    let design: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../planning/minnesota/gyms/duluth-harbor-ice-v01.json"
    ))
    .expect("harbor ice source");
    // Mew100, seven earlier badges and the real facade are explicit setup.
    let mut initial = design["gyms"][0].clone();
    initial["id"] = serde_json::json!("MnDuluth");
    initial["arrival_core"] = serde_json::json!([48, 32]);
    let mut c = controller(&root, &r, &initial, false);
    travel(&mut c, GameButton::Up, "MnDuluthGym", (6, 17));
    assert!(!can_approach(&mut c, &r, (9, 6)));
    assert!(!can_approach(&mut c, &r, (8, 3)));
    walk(&mut c, "DRRRRRUU");
    assert_eq!(at(&mut c), (11, 16));
    press(&mut c, GameButton::Up);
    assert_eq!(at(&mut c), (11, 16), "east gate starts closed");
    walk(&mut c, "DDLLLLLLLLLUU");
    slide_gym_ice(
        &mut c,
        &r,
        (2, 16),
        (2, 8),
        GameButton::Up,
        GameButton::Down,
    );
    slide_gym_ice(
        &mut c,
        &r,
        (2, 8),
        (4, 8),
        GameButton::Right,
        GameButton::Left,
    );
    walk(&mut c, "UUUL");
    talk(&mut c, GameButton::Up);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_8_WEST_RETURN_OPEN"));
    assert!(event(&c, "EVENT_MN_GYM_8_EAST_ENTRY_OPEN"));
    walk(&mut c, "RDDR");
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_8_TRAINER_3_DEFEATED"));
    walk(&mut c, "LD");
    slide_gym_ice(
        &mut c,
        &r,
        (4, 8),
        (4, 14),
        GameButton::Down,
        GameButton::Up,
    );
    walk(&mut c, "DDDDRRRRRRRUU");
    slide_gym_ice(
        &mut c,
        &r,
        (11, 16),
        (11, 12),
        GameButton::Up,
        GameButton::Down,
    );
    assert_eq!(gym_collision(&mut c, &r, (11, 12)), "FLOOR");
    c = continue_save(&mut c, &root, &r);
    slide_gym_ice(
        &mut c,
        &r,
        (11, 12),
        (9, 12),
        GameButton::Left,
        GameButton::Right,
    );
    slide_gym_ice(
        &mut c,
        &r,
        (9, 12),
        (9, 10),
        GameButton::Up,
        GameButton::Down,
    );
    slide_gym_ice(
        &mut c,
        &r,
        (9, 10),
        (9, 7),
        GameButton::Up,
        GameButton::Down,
    );
    // Retreat is possible before Tordel's battle, including the new stop.
    slide_gym_ice(
        &mut c,
        &r,
        (9, 7),
        (9, 10),
        GameButton::Down,
        GameButton::Up,
    );
    slide_gym_ice(
        &mut c,
        &r,
        (9, 10),
        (11, 10),
        GameButton::Right,
        GameButton::Left,
    );
    slide_gym_ice(
        &mut c,
        &r,
        (11, 10),
        (11, 12),
        GameButton::Down,
        GameButton::Up,
    );
    slide_gym_ice(
        &mut c,
        &r,
        (11, 12),
        (11, 16),
        GameButton::Down,
        GameButton::Up,
    );
    assert!(!event(&c, "EVENT_MN_GYM_8_EAST_RETURN_OPEN"));
    slide_gym_ice(
        &mut c,
        &r,
        (11, 16),
        (11, 12),
        GameButton::Up,
        GameButton::Down,
    );
    slide_gym_ice(
        &mut c,
        &r,
        (11, 12),
        (9, 12),
        GameButton::Left,
        GameButton::Right,
    );
    slide_gym_ice(
        &mut c,
        &r,
        (9, 12),
        (9, 10),
        GameButton::Up,
        GameButton::Down,
    );
    slide_gym_ice(
        &mut c,
        &r,
        (9, 10),
        (9, 7),
        GameButton::Up,
        GameButton::Down,
    );
    talk(&mut c, GameButton::Up);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_8_EAST_RETURN_OPEN"));
    assert!(!event(&c, "EVENT_MN_GYM_8_COURT_OPEN"));
    c = continue_save(&mut c, &root, &r);
    walk(&mut c, "LLLL");
    talk(&mut c, GameButton::Up);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_8_COURT_OPEN"));
    walk(&mut c, "RUUUUR");
    talk(&mut c, GameButton::Right);
    battle(&mut c, false);
    finish(&mut c);
    assert!(c.shell.shell.session().state().badges.johto[7]);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["TM_ICY_WIND"]),
        1
    );
    c = continue_save(&mut c, &root, &r);
    talk(&mut c, GameButton::Right);
    finish(&mut c);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["TM_ICY_WIND"]),
        1
    );
    walk(&mut c, "LDDDDLLD");
    slide_gym_ice(
        &mut c,
        &r,
        (4, 8),
        (4, 14),
        GameButton::Down,
        GameButton::Up,
    );
    walk(&mut c, "DDDDRR");
    travel(&mut c, GameButton::Down, "MnDuluth", (48, 32));
    travel(&mut c, GameButton::Up, "MnDuluthGym", (6, 17));
    walk(&mut c, "DR");
    travel(&mut c, GameButton::Down, "MnDuluth", (48, 32));
    assert!(event(&c, "EVENT_MN_GYM_8_COURT_OPEN"));
}
