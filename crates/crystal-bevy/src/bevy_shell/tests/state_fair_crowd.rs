//! Explicit isolated trainer fixtures plus the live overlapping midway crowd.
use std::collections::BTreeSet;

use crystal_core::input::GameButton;
use crystal_runtime::RuntimeBattleKind;
use serde_json::Value;

use super::{
    Point, VisibleShellController, at, continue_save, event, finish, fixture_at,
    mark_runtime_snapshot_dirty, press,
};

fn source() -> Value {
    serde_json::from_str(include_str!(
        "../../../../../planning/minnesota/zones/state-fairgrounds.json"
    ))
    .expect("fair source")
}

fn prior_battles(c: &mut VisibleShellController, active: &[&str]) {
    // Additional explicit setup, before any test input: isolate selected rivals.
    for t in source()["native_trainers"]
        .as_array()
        .expect("trainer contracts")
    {
        if !active.contains(&t["id"].as_str().expect("trainer ID")) {
            c.shell
                .shell
                .session_mut()
                .state_mut()
                .flags
                .set_event_flag(t["defeated_flag"].as_str().expect("flag"), true)
                .expect("explicit earlier trainer-win fixture");
        }
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
}

fn delta(facing: &str) -> (i16, i16, GameButton, GameButton) {
    match facing {
        "east" => (1, 0, GameButton::Left, GameButton::Right),
        "west" => (-1, 0, GameButton::Right, GameButton::Left),
        "north" => (0, -1, GameButton::Down, GameButton::Up),
        "south" => (0, 1, GameButton::Up, GameButton::Down),
        _ => panic!("trainer facing"),
    }
}

fn enter(c: &mut VisibleShellController, button: GameButton) {
    let from = at(c);
    for _ in 0..4 {
        press(c, button);
        if at(c) != from || c.snapshot().expect("sight").ui.text_window_open {
            return;
        }
    }
    panic!("fixture did not enter trainer sight");
}

fn win(c: &mut VisibleShellController, id: &str, flag: &str) {
    let before = c.snapshot().expect("challenge");
    let pp = |s: &crystal_runtime::RuntimeShellSnapshot| {
        s.party
            .slots
            .iter()
            .flat_map(|p| &p.pokemon.moves)
            .map(|m| u32::from(m.current_pp))
            .sum::<u32>()
    };
    let before_pp = pp(&before);
    let money = c.shell.shell.session().state().money;
    let mut observed = false;
    for _ in 0..768 {
        let s = c.snapshot().expect("native crowd battle");
        if let Some(b) = &s.battle {
            let RuntimeBattleKind::Trainer { trainer_id, .. } = &b.kind else {
                panic!("expected fair trainer");
            };
            assert_eq!(trainer_id, id);
            observed = true;
        } else if observed
            && event(c, flag)
            && !s.ui.text_window_open
            && !c.has_pending_script_work()
        {
            assert!(pp(&s) < before_pp, "actual selected moves spend PP");
            assert!(
                c.shell.shell.session().state().money > money,
                "native trainer reward"
            );
            return;
        }
        let button = if s.ui.menu.is_none() && s.ui.text.is_some() {
            GameButton::A
        } else if let Some(cursor) = &c.shell.battle_move_cursor {
            let b = s.battle.as_ref().expect("move menu battle");
            let target = b
                .player_moves
                .iter()
                .position(|m| m.name == "SURF")
                .expect("legal review move");
            if cursor.option_index < target {
                GameButton::Down
            } else if cursor.option_index > target {
                GameButton::Up
            } else {
                GameButton::A
            }
        } else if let Some(cursor) = &c.shell.battle_action_cursor {
            match cursor.option_index {
                0 => GameButton::A,
                1 => GameButton::Left,
                _ => GameButton::Up,
            }
        } else {
            GameButton::A
        };
        press(c, button);
    }
    panic!("trainer {id} did not finish a native battle");
}

#[test]
#[ignore = "requires the published fair crowd pack"]
fn state_fair_crowd_all_35_trainers_battle_reward_and_remember_defeat() {
    let design = source();
    let trainers = design["native_trainers"].as_array().expect("trainers");
    assert_eq!(trainers.len(), 35);
    for t in trainers {
        let id = t["id"].as_str().expect("ID");
        let flag = t["defeated_flag"].as_str().expect("flag");
        let x = t["tile"][0].as_i64().expect("x") as i16;
        let y = t["tile"][1].as_i64().expect("y") as i16;
        let range = t["sight_range"].as_i64().expect("range") as i16;
        let (dx, dy, toward, away) = delta(t["facing"].as_str().expect("facing"));
        let distance = if range > 0 { range + 1 } else { 1 };
        let spawn: Point = ((x + dx * distance) as u16, (y + dy * distance) as u16);
        let (root, r, mut c) = fixture_at(false, spawn, true);
        prior_battles(&mut c, &[id]);
        if range > 0 {
            enter(&mut c, toward);
        } else {
            press(&mut c, toward);
            press(&mut c, GameButton::A);
        }
        win(&mut c, id, flag);
        assert!(event(&c, flag));
        if range > 0 {
            enter(&mut c, away);
            enter(&mut c, toward);
            assert!(c.snapshot().expect("defeated sight").battle.is_none());
        }
        if id == "MN_FAIR_ORCHARD_TRAINER" {
            c = continue_save(&mut c, &root, &r);
            assert!(event(&c, flag), "Continue retains the new trainer flag");
        }
        eprintln!("Verified native fair trainer {id}");
    }
}

#[test]
#[ignore = "requires the published fair crowd pack"]
fn state_fair_crowd_overlapping_midway_sight_gives_two_real_battles() {
    let a = "MN_FAIR_MIDWAY_A_TRAINER";
    let b = "MN_FAIR_MIDWAY_B_TRAINER";
    let (_, _, mut c) = fixture_at(false, (13, 28), true);
    prior_battles(&mut c, &[a, b]);
    enter(&mut c, GameButton::Up);
    win(&mut c, a, "EVENT_MN_FAIR_MIDWAY_A_DEFEATED");
    enter(&mut c, GameButton::Down);
    enter(&mut c, GameButton::Up);
    win(&mut c, b, "EVENT_MN_FAIR_MIDWAY_B_DEFEATED");
}

fn tile(c: &VisibleShellController, id: &str) -> Point {
    let p = c
        .shell
        .shell
        .session()
        .overworld()
        .object_runtime_tile_by_id(id)
        .expect("live patrol");
    (p.x as u16, p.y as u16)
}

fn patrol_clock(c: &mut VisibleShellController) {
    // A reproducible DIV stream keeps native between-step waits short. Restore
    // this process-owned clock after Continue; saved game state is untouched.
    *c.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(vec![0; 32768]);
}

#[test]
#[ignore = "requires the published fair maze pack"]
fn state_fair_spread_crowd_walks_runs_rotates_and_has_text() {
    let design = source();
    let residents = design["native_map"]["residents"]
        .as_array()
        .expect("residents");
    assert_eq!(
        residents.len()
            + design["native_trainers"]
                .as_array()
                .expect("trainers")
                .len(),
        255
    );
    for actor in residents {
        assert!(!actor["lines"].as_array().expect("dialogue").is_empty());
    }
    for behavior in ["walk", "run", "rotate"] {
        let movement = design["native_ambient"]
            .as_array()
            .expect("ambient crowd")
            .iter()
            .find(|p| p["behavior"] == behavior)
            .expect("movement variety");
        let id = movement["id"].as_str().expect("actor ID");
        let actor = residents.iter().find(|a| a["id"] == id).expect("actor");
        let anchor = (
            actor["tile"][0].as_u64().expect("x") as u16,
            actor["tile"][1].as_u64().expect("y") as u16,
        );
        let (_, _, mut c) = fixture_at(false, (anchor.0, anchor.1 + 2), true);
        prior_battles(&mut c, &[]);
        patrol_clock(&mut c);
        let mut moved = false;
        let mut turned = false;
        let initial_facing = c.shell.shell.session().overworld().object_facings[id];
        for _ in 0..80 {
            c.wait_frames(4)
                .expect("production autonomous crowd frames");
            moved |= tile(&c, id) != anchor;
            turned |= c.shell.shell.session().overworld().object_facings[id] != initial_facing;
        }
        if behavior == "rotate" {
            assert!(!moved, "turning spectator keeps their place");
            assert!(turned, "spectator turns through native facings");
            press(&mut c, GameButton::Up);
            press(&mut c, GameButton::Up);
            press(&mut c, GameButton::A);
            assert!(
                c.snapshot()
                    .expect("new fairgoer dialogue")
                    .ui
                    .text_window_open,
                "the rotating visitor remains an interactable character"
            );
            finish(&mut c);
        } else {
            assert!(moved, "{behavior} visitor takes actual steps");
        }
    }
}

#[test]
#[ignore = "requires the published fair crowd pack"]
fn state_fair_crowd_four_patrols_wait_talk_loop_and_continue() {
    let design = source();
    for patrol in design["native_patrols"].as_array().expect("patrols") {
        let id = patrol["id"].as_str().expect("patrol ID");
        let actor = design["native_map"]["residents"]
            .as_array()
            .expect("residents")
            .iter()
            .find(|a| a["id"] == id)
            .expect("patrol actor");
        let anchor = (
            actor["tile"][0].as_u64().expect("x") as u16,
            actor["tile"][1].as_u64().expect("y") as u16,
        );
        let width = patrol["size_core"][0].as_u64().expect("width") as u16;
        let height = patrol["size_core"][1].as_u64().expect("height") as u16;
        let (root, r, mut c) = fixture_at(false, (anchor.0 + 1, anchor.1), true);
        prior_battles(&mut c, &[]);
        patrol_clock(&mut c);
        c.wait_frames(240).expect("wait while blocking next step");
        assert_eq!(tile(&c, id), anchor, "walker waits for the player");
        press(&mut c, GameButton::Left);
        press(&mut c, GameButton::A);
        assert!(
            c.snapshot()
                .expect("walker conversation")
                .ui
                .text_window_open
        );
        c.wait_frames(120).expect("read a page");
        assert_eq!(tile(&c, id), anchor);
        finish(&mut c);
        enter(&mut c, GameButton::Up);
        let mut previous = tile(&c, id);
        let mut seen = BTreeSet::from([previous]);
        let mut returns = 0;
        let mut saved = false;
        for _ in 0..2500 {
            c.wait_frames(4).expect("native autonomous frames");
            let at = tile(&c, id);
            if at == previous {
                continue;
            }
            assert_eq!(at.0.abs_diff(previous.0) + at.1.abs_diff(previous.1), 1);
            assert!(
                at.0 >= anchor.0
                    && at.0 <= anchor.0 + width
                    && at.1 >= anchor.1
                    && at.1 <= anchor.1 + height
            );
            assert!(
                at.0 == anchor.0
                    || at.0 == anchor.0 + width
                    || at.1 == anchor.1
                    || at.1 == anchor.1 + height
            );
            returns += usize::from(at == anchor);
            previous = at;
            seen.insert(at);
            if !saved && seen.len() >= 4 {
                c = continue_save(&mut c, &root, &r);
                patrol_clock(&mut c);
                assert_eq!(tile(&c, id), at, "Continue retains current loop position");
                saved = true;
            }
            if returns == 2 {
                break;
            }
        }
        assert!(
            saved && returns == 2,
            "{id}: saved={saved}, returns={returns}, position={previous:?}, seen={seen:?}"
        );
        assert_eq!(seen.len(), usize::from(2 * (width + height)));
        eprintln!("Verified two native laps and Continue for {id}");
    }
}
