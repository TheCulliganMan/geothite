//! Declared contained chapter and full party; capture uses production menus.
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, PcBox};

use super::super::interior_life::fixture;
use super::super::northstar_ice_lair::{choice, event, walk};
use super::super::r10_gallery::{continue_save, settle};
use super::super::winona::{load, press, travel};
use super::{CONTAINED, RESTORED, VisibleShellController, mark_runtime_snapshot_dirty};

const CAUGHT: &str = "EVENT_MN_VERDIGRIM_CAUGHT";

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the Verdigrim intake scene"]
fn verdigrim_controller_full_party_capture_uses_selected_pc_and_continue() {
    let (root, runtime) = load();
    let mut c = fixture(&root, &runtime, "MnBoundaryIntake", (4, 18));
    {
        let state = c.shell.shell.session_mut().state_mut();
        for &flag in super::CLEARED {
            state
                .flags
                .set_event_flag(flag, true)
                .expect("declared cleared chapter");
        }
        for slot in 0..6 {
            state.storage.party.pokemon[slot] = Some(
                runtime
                    .data()
                    .create_pokemon("POUCHID", 60, Dv::default())
                    .expect("declared native full party"),
            );
        }
        state.current_pc_box = 3;
        while state.storage.pc_boxes.len() <= state.current_pc_box {
            state
                .storage
                .pc_boxes
                .push(PcBox::new(state.storage.pc_boxes.len()));
        }
        state.sync_party_from_storage();
        assert!(
            state
                .bag
                .add_item(&runtime.data().items["MASTER_BALL"], 1)
                .expect("declared capture item")
        );
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    // No party, bag, flags, position, battle or RNG writes after this setup.
    let before = c.shell.shell.session().state().storage.clone();
    travel(
        &mut c,
        GameButton::Right,
        "MnBoundaryWatersSurveyShelter",
        (12, 7),
    );
    walk(&mut c, &runtime, (13, 7));
    travel(&mut c, GameButton::Down, "MnBoundaryIntake", (4, 19));
    walk(&mut c, &runtime, (24, 6));
    press(&mut c, GameButton::Right);
    press(&mut c, GameButton::A);
    choice(&mut c, true);
    for _ in 0..128 {
        if c.snapshot().expect("optional encounter").battle.is_some() {
            break;
        }
        c.wait_frames(64).expect("native battle transition");
    }
    let battle = c
        .snapshot()
        .expect("actual full-party battle")
        .battle
        .expect("battle");
    assert_eq!(battle.enemy_pokemon.species.id, "VERDIGRIM");
    assert_eq!(battle.enemy_pokemon.level, 60);
    capture(&mut c);
    let state = c.shell.shell.session().state();
    assert!(event(&c, CAUGHT) && event(&c, CONTAINED) && event(&c, RESTORED));
    assert_eq!(state.storage.party, before.party);
    assert_eq!(state.current_pc_box, 3);
    let boxed = state.storage.pc_boxes[3].pokemon[0]
        .as_ref()
        .expect("selected-PC capture");
    assert_eq!(boxed.species.id, "VERDIGRIM");
    assert_eq!(boxed.level, 60);
    assert_eq!(state.storage.pc_boxes[3].filled_slots(), 1);
    for (index, pc) in state.storage.pc_boxes.iter().enumerate() {
        if index != 3 {
            assert_eq!(pc, &before.pc_boxes[index]);
        }
    }
    assert!(state.pokedex.has_caught("VERDIGRIM"));
    assert_eq!(state.bag.quantity(&runtime.data().items["MASTER_BALL"]), 0);
    let storage = state.storage.clone();
    c = continue_save(root.clone(), runtime.clone(), &mut c);
    assert_eq!(c.shell.shell.session().state().storage, storage);
    assert!(event(&c, CAUGHT));
    assert!(
        c.snapshot()
            .expect("captured actor stays hidden")
            .visible_objects
            .iter()
            .all(|o| o.object_identifier.as_deref() != Some("MN_VERDIGRIM_INTAKE"))
    );
    walk(&mut c, &runtime, (4, 18));
    travel(
        &mut c,
        GameButton::Right,
        "MnBoundaryWatersSurveyShelter",
        (12, 7),
    );
    walk(&mut c, &runtime, (13, 7));
    travel(&mut c, GameButton::Down, "MnBoundaryIntake", (4, 19));
    walk(&mut c, &runtime, (24, 6));
    assert!(
        c.snapshot()
            .expect("captured encounter stays gone on new visit")
            .visible_objects
            .iter()
            .all(|o| o.object_identifier.as_deref() != Some("MN_VERDIGRIM_INTAKE"))
    );
    press(&mut c, GameButton::Right);
    press(&mut c, GameButton::A);
    assert!(
        c.snapshot()
            .expect("no duplicate encounter")
            .battle
            .is_none()
    );
    assert_eq!(c.shell.shell.session().state().storage, storage);
    assert!(event(&c, CAUGHT) && event(&c, CONTAINED) && event(&c, RESTORED));
    c = continue_save(root, runtime, &mut c);
    assert_eq!(c.shell.shell.session().state().storage, storage);
    assert!(event(&c, CAUGHT));
}

fn capture(c: &mut VisibleShellController) {
    for _ in 0..256 {
        let snapshot = c.snapshot().expect("native capture menus");
        if event(c, CAUGHT) && snapshot.battle.is_none() {
            settle(c, "capture continuation");
            return;
        }
        if let Some(choice) = &c.shell.pending_name_choice {
            if choice.nickname_pages.len() > 1 {
                press(c, GameButton::A);
            } else {
                press(c, GameButton::Down);
                press(c, GameButton::A);
            }
            continue;
        }
        let button = if c.shell.pokedex_menu_open
            || (snapshot.ui.menu.is_none() && snapshot.ui.text.is_some())
            || c.shell.field_pack_action_cursor.is_some()
            || c.shell.ball_cursor.is_some()
        {
            GameButton::A
        } else if c.shell.bag_cursor.is_some() {
            GameButton::Right
        } else if let Some(cursor) = &c.shell.battle_action_cursor {
            match cursor.option_index {
                0 | 1 => GameButton::Down,
                3 => GameButton::Left,
                _ => GameButton::A,
            }
        } else {
            GameButton::A
        };
        press(c, button);
    }
    panic!("native capture did not finish");
}
