//! Two physical switches, one lane change and earned native trainer victories.
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use serde_json::Value;

use super::super::winona::travel;
use super::rochester_readable::{talk, walk};
use super::{
    at, battle, can_approach, continue_save, controller, event, finish, load,
    mark_runtime_snapshot_dirty, module, press,
};

const LEFT: &str = "EVENT_MN_GYM_1_LOWER_LEFT";
const RIGHT: &str = "EVENT_MN_GYM_1_LOWER_RIGHT";
const UPPER: &str = "EVENT_MN_GYM_1_UPPER_OPEN";

#[test]
#[ignore = "requires minnesota-rochester-gates.crystalpack via CRYSTAL_GYM_TEST_PACK"]
fn rochester_gates_literal_controls_require_both_trainers_change_lanes_and_return() {
    let (root, r, _) = load();
    let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository");
    let design: Value = serde_json::from_str(
        &std::fs::read_to_string(repo.join("planning/minnesota/gyms/rochester-gates.json"))
            .expect("authored gate course"),
    )
    .expect("design");
    let gym = &design["gyms"][0];
    let mut c = controller(&root, &r, gym, false);
    // Local initial fixture only: ordinary level-up partners, delivered parcel,
    // arrival and the shared deterministic test divider. No downstream grants.
    let state = c.shell.shell.session_mut().state_mut();
    state.storage.party.pokemon.fill(None);
    for (species, level) in [("TOTODILE", 16), ("HOOTHOOT", 14)] {
        assert!(
            state.storage.party.add_pokemon(
                r.data()
                    .create_pokemon(species, level, Dv::default())
                    .expect("ordinary partner")
            )
        );
    }
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert!(!can_approach(&mut c, &r, (4, 6)));
    assert!(!can_approach(&mut c, &r, (10, 2)));
    let closed = module(&mut c, &r).blocks;
    // The lower switch is a physical wall button, not a quiz/menu choice.
    walk(&mut c, "LLLLU");
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    assert!(!event(&c, LEFT) && !event(&c, RIGHT));
    assert_eq!(module(&mut c, &r).blocks, closed);
    walk(&mut c, "DRR");
    talk(&mut c, GameButton::Up);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_1_TRAINER_1_DEFEATED"));
    assert!(
        !can_approach(&mut c, &r, (4, 6)),
        "battle alone does not move a gate"
    );
    walk(&mut c, "LLU");
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    assert!(event(&c, LEFT) && !event(&c, RIGHT));
    assert!(can_approach(&mut c, &r, (4, 6)));
    assert!(!can_approach(&mut c, &r, (10, 2)));
    // Try the other lane too: its upper gate is still physically closed.
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    assert!(event(&c, RIGHT) && !event(&c, LEFT));
    walk(&mut c, "DRRRRRRUUUUUR");
    assert_eq!(at(&mut c), (9, 6));
    let blocked = at(&mut c);
    for _ in 0..8 {
        press(&mut c, GameButton::Up);
    }
    assert_eq!(at(&mut c), blocked);
    walk(&mut c, "LDDDDDLLLLLLU");
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    assert!(event(&c, LEFT) && !event(&c, RIGHT));
    walk(&mut c, "DRRRUUUULLLU");
    assert_eq!(at(&mut c), (2, 6));
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    assert!(!event(&c, UPPER), "Elvik's switch refuses a skipped battle");
    walk(&mut c, "DRR");
    talk(&mut c, GameButton::Up);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_1_TRAINER_2_DEFEATED"));
    assert!(
        !event(&c, UPPER),
        "second battle alone does not move the upper gate"
    );
    walk(&mut c, "LLU");
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    assert!(event(&c, UPPER));
    // Reversible gate, with no trainer flags or rewards erased on either flip.
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    assert!(!event(&c, UPPER));
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    c = continue_save(&mut c, &root, &r);
    assert!(event(&c, LEFT) && event(&c, UPPER));
    assert!(
        !can_approach(&mut c, &r, (10, 2)),
        "return to swap lanes is necessary"
    );
    walk(&mut c, "DRRRDDDDLLLU");
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    assert!(event(&c, RIGHT) && !event(&c, LEFT));
    assert!(can_approach(&mut c, &r, (10, 2)));
    walk(&mut c, "DRRRRRRUUUUUUUUUR");
    assert_eq!(at(&mut c), (9, 2));
    talk(&mut c, GameButton::Right);
    battle(&mut c, false);
    finish(&mut c);
    assert!(c.shell.shell.session().state().badges.johto[0]);
    let item = &r.data().items["TM_HEADBUTT"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(item), 1);
    c = continue_save(&mut c, &root, &r);
    talk(&mut c, GameButton::Right);
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().bag.quantity(item), 1);
    walk(&mut c, "LDDDDDDDDDLLD");
    travel(&mut c, GameButton::Down, "MnRochester", (30, 24));
    travel(&mut c, GameButton::Up, "MnRochesterGym", (6, 11));
    walk(&mut c, "DR");
    travel(&mut c, GameButton::Down, "MnRochester", (30, 24));
    assert!(event(&c, RIGHT) && event(&c, UPPER));
}
