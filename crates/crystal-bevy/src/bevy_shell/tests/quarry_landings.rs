//! Start outside the quarry, earn both gates and walk the full width of its return lip.
use std::path::PathBuf;

use crystal_core::input::GameButton::{self, A, Down, Left, Right, Up};
use crystal_core::models::Dv;
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind};

use super::super::badge_trails::load;
use super::super::interior_life::{fixture, talk};
use super::super::northstar_ice_lair::event;
use super::super::rochester_earned::try_battle;
use super::super::winona::{at, press, travel};
use crate::bevy_shell::{RuntimeBadgeRegion, VisibleShellController, mark_runtime_snapshot_dirty};

const GYM: &str = "MnMankatoGym";
const CITY: &str = "MnMankato";
const STAIR: &str = "EVENT_MN_GYM_2_LOWER_STAIR_OPEN";
const COURT: &str = "EVENT_MN_GYM_2_COURT_OPEN";

fn walk(c: &mut VisibleShellController, direction: GameButton, x: i16, y: i16) {
    travel(c, direction, GYM, (x, y));
}

fn win_on_sight(c: &mut VisibleShellController, runtime: &CrystalRuntime, id: &str, name: &str) {
    let flag = format!("EVENT_{id}_DEFEATED");
    assert!(!event(c, &flag));
    c.wait_frames(128).expect("student approaches");
    let challenge = c
        .shell
        .field_notice
        .as_deref()
        .expect("the student initiates without A");
    assert!(
        challenge.starts_with(name),
        "{id} started with {challenge:?}"
    );
    for _ in 0..16 {
        if let Some(battle) = c.snapshot().expect("trainer challenge").battle {
            assert!(
                matches!(battle.kind, RuntimeBattleKind::Trainer { trainer_id, .. } if trainer_id == id)
            );
            assert!(try_battle(c, runtime, id, &flag));
            return;
        }
        press(c, A);
    }
    panic!("{id} failed to start a battle after the sight challenge");
}

fn climb_west_and_drop_in(c: &mut VisibleShellController) {
    walk(c, Up, 3, 9);
    walk(c, Left, 2, 9);
    walk(c, Up, 2, 5);
    walk(c, Right, 6, 5);
    walk(c, Down, 6, 6);
    walk(c, Down, 6, 8);
    walk(c, Down, 6, 12);
}

#[test]
#[ignore = "requires Quarry Landings in MN_BADGE_TRAILS_PACK"]
fn quarry_landings_earn_both_stair_gates_and_take_all_eight_drops() {
    let (root, runtime) = load();
    let mut c = fixture(&root, &runtime, CITY, (10, 12));
    c.shell
        .shell
        .award_badge(RuntimeBadgeRegion::Johto, 0)
        .expect("prior Care Badge");
    let state = c.shell.shell.session_mut().state_mut();
    state
        .flags
        .set_event_flag("EVENT_MN_GYM_1_DEFEATED", true)
        .expect("past Rochester");
    // Supply one regional partner so this route check can finish both real student battles.
    let mut partner = runtime
        .data()
        .create_pokemon("BOREALYNX", 40, Dv::default())
        .expect("regional partner");
    partner.original_trainer_name.clone_from(&state.player_name);
    partner.original_trainer_id = state.player_id;
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);

    travel(&mut c, Up, GYM, (6, 17));
    assert!(!event(&c, STAIR) && !event(&c, COURT));
    walk(&mut c, Right, 8, 17);
    let guide = talk(&mut c, (9, 17));
    assert!(
        guide.contains("MnMankatoGymGuideText") && guide.contains("West stairs climb."),
        "{guide}"
    );
    walk(&mut c, Left, 3, 17);
    walk(&mut c, Up, 3, 16);
    win_on_sight(&mut c, &runtime, "MN_GYM_2_TRAINER_1", "CALDREN:");
    assert!(event(&c, STAIR) && !event(&c, COURT));

    climb_west_and_drop_in(&mut c);
    walk(&mut c, Right, 12, 12);
    walk(&mut c, Up, 12, 8);
    win_on_sight(&mut c, &runtime, "MN_GYM_2_TRAINER_2", "JOREL:");
    assert!(event(&c, STAIR) && event(&c, COURT));
    // The newly opened east stair leads up to Torven. Backtrack into the bowl.
    walk(&mut c, Up, 12, 6);
    walk(&mut c, Down, 12, 12);
    walk(&mut c, Left, 6, 12);

    let review = std::env::var_os("MN_QUARRY_REVIEW_DIR").map(PathBuf::from);
    for x in 6..=13 {
        walk(&mut c, Right, x, 12);
        walk(&mut c, Down, x, 14);
        for _ in 0..4 {
            press(&mut c, Down);
            if c.snapshot().expect("first step off lip").overworld.tile.y != 14 {
                break;
            }
        }
        at(&mut c, GYM, (x, 16));
        if let Some(directory) = review.as_deref().filter(|_| matches!(x, 9 | 13)) {
            c.wait_frames(20).expect("settled quarry landing");
            std::fs::create_dir_all(directory).expect("quarry review directory");
            c.save(directory.join(format!("quarry-landed-{x}-16.crystalsave")))
                .expect("capture earned landing and both open gates");
        }
        press(&mut c, Up);
        press(&mut c, Up);
        at(&mut c, GYM, (x, 16));
        // Walk away on the landing strip, past the guide, and climb for the next drop.
        walk(&mut c, Left, 5, 16);
        walk(&mut c, Down, 5, 17);
        if x < 13 {
            walk(&mut c, Left, 3, 17);
            climb_west_and_drop_in(&mut c);
        }
    }
    walk(&mut c, Right, 6, 17);
    travel(&mut c, Down, CITY, (10, 12));
    assert!(event(&c, STAIR) && event(&c, COURT));
}
