//! Literal player instructions: no path finder or session-position writes.
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use serde_json::json;

use super::super::winona::travel;
use super::{
    VisibleShellController, at, battle, continue_save, controller, event, finish, load,
    mark_runtime_snapshot_dirty, press,
};

pub(super) fn walk(c: &mut VisibleShellController, instructions: &str) -> Vec<[u16; 2]> {
    let start = at(c);
    let mut points = vec![[start.0, start.1]];
    for instruction in instructions.chars() {
        let (button, dx, dy) = match instruction {
            'U' => (GameButton::Up, 0, -1),
            'R' => (GameButton::Right, 1, 0),
            'D' => (GameButton::Down, 0, 1),
            'L' => (GameButton::Left, -1, 0),
            _ => panic!("cardinal player instruction required"),
        };
        let before = at(c);
        let target = ((before.0 as i16 + dx) as u16, (before.1 as i16 + dy) as u16);
        for _ in 0..8 {
            press(c, button);
            let snapshot = c.snapshot().expect("normal movement");
            assert_eq!(snapshot.overworld.map_name, "MnRochesterGym");
            assert!(snapshot.battle.is_none() && !snapshot.ui.text_window_open);
            if at(c) != before {
                break;
            }
        }
        assert_eq!(
            at(c),
            target,
            "literal {instruction} must move exactly one visible tile"
        );
        points.push([target.0, target.1]);
    }
    points
}

pub(super) fn talk(c: &mut VisibleShellController, facing: GameButton) {
    let stand = at(c);
    press(c, facing);
    assert_eq!(at(c), stand, "trainer occupies the adjacent tile");
    press(c, GameButton::A);
}

#[test]
#[ignore = "requires the Rochester readable candidate via CRYSTAL_GYM_TEST_PACK"]
fn rochester_readable_literal_walkthrough_wins_with_starter_party_and_both_city_returns() {
    let (root, r, design) = load();
    let gym = &design["gyms"][0];
    let mut c = controller(&root, &r, gym, false);
    // Explicit puzzle fixture: arrival, delivered parcel, zero badges and two
    // ordinary species with their native level-up moves. No gym victory or badge is seeded.
    let state = c.shell.shell.session_mut().state_mut();
    state.storage.party.pokemon.fill(None);
    for (species, level) in [("TOTODILE", 16), ("HOOTHOOT", 14)] {
        let partner = r
            .data()
            .create_pokemon(species, level, Dv::default())
            .expect("native partner");
        assert!(state.storage.party.add_pokemon(partner));
    }
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert_eq!(c.shell.shell.session().state().badges.johto, [false; 8]);
    let first = walk(&mut c, "U");
    assert_eq!(at(&mut c), (6, 16));
    talk(&mut c, GameButton::Left);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_1_TRAINER_1_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_GYM_1_TRAINER_2_DEFEATED"));
    let second = walk(&mut c, "RRRRUUUULLLLLLLUUURR");
    assert_eq!(at(&mut c), (5, 9));
    talk(&mut c, GameButton::Up);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_1_TRAINER_2_DEFEATED"));
    assert!(!c.shell.shell.session().state().badges.johto[0]);
    c = continue_save(&mut c, &root, &r);
    let third = walk(&mut c, "RRRRRUUUUUULLL");
    assert_eq!(at(&mut c), (7, 3));
    talk(&mut c, GameButton::Left);
    battle(&mut c, false);
    finish(&mut c);
    assert!(c.shell.shell.session().state().badges.johto[0]);
    let item = &r.data().items["TM_HEADBUTT"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(item), 1);
    c = continue_save(&mut c, &root, &r);
    talk(&mut c, GameButton::Left);
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().bag.quantity(item), 1);
    let returning = walk(&mut c, "RRRDDDDDDLLLLLLLDDDRRRRRRRDDDDLLLLDD");
    assert_eq!(at(&mut c), (6, 18));
    travel(&mut c, GameButton::Down, "MnRochester", (30, 24));
    travel(&mut c, GameButton::Up, "MnRochesterGym", (6, 17));
    walk(&mut c, "DR");
    travel(&mut c, GameButton::Down, "MnRochester", (30, 24));
    assert!(event(&c, "EVENT_MN_GYM_1_TRAINER_1_DEFEATED"));
    assert!(event(&c, "EVENT_MN_GYM_1_TRAINER_2_DEFEATED"));
    if let Some(path) = std::env::var_os("CRYSTAL_GYM_WALK_PROOF") {
        let proof = json!({"map":"MnRochesterGym", "content_hash":r.pack_identity().content_hash,
            "fixture":"Totodile16/Hoothoot14; parcel supplied",
            "phases":[{"name":"Tavel","points":first},{"name":"Elvik","points":second},
                {"name":"Varel","points":third},{"name":"Return","points":returning}],
            "badge_earned":true,"tm_quantity":1,"both_city_returns":true});
        std::fs::write(path, serde_json::to_vec_pretty(&proof).expect("proof JSON"))
            .expect("external route proof");
    }
}
