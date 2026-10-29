//! Isolated discovery fixtures; encounter choices and capture use native input.
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::interior_life::fixture;
use super::northstar_ice_lair::choice;
use super::r10_gallery::continue_save;
use super::winona::{load, press};

fn caught(c: &VisibleShellController) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set("EVENT_MN_CASTOROCK_CAUGHT")
        .expect("Castorock capture flag")
}

fn finish(c: &mut VisibleShellController) {
    for _ in 0..128 {
        c.wait_frames(64).expect("encounter presentation");
        let snapshot = c.snapshot().expect("encounter dialogue");
        assert!(snapshot.battle.is_none(), "handle battle explicitly");
        assert!(snapshot.ui.pending_yes_no.is_none(), "answer explicitly");
        if !snapshot.ui.text_window_open && !c.has_pending_script_work() {
            return;
        }
        press(c, GameButton::A);
    }
    panic!("Castorock dialogue did not finish");
}

fn walk(c: &mut VisibleShellController, runtime: &CrystalRuntime, to: (i16, i16)) {
    for _ in 0..512 {
        if c.snapshot().expect("cave travel").battle.is_some() {
            super::rochester::flee(c);
        }
        let path = super::northstar_ice_lair::path(c, runtime, to)
            .expect("dry route around the live boulder");
        let Some(button) = path.first() else {
            return;
        };
        press(c, *button);
    }
    panic!("cave route did not arrive at {to:?}");
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the incisor gallery"]
fn castorock_controller_strength_gate_and_three_acknowledged_clues() {
    let (root, runtime) = load();
    for (badge, knows_strength) in [(false, true), (true, false), (true, true)] {
        let mut c = fixture(&root, &runtime, "MnR03Cave1F", (12, 6));
        {
            let state = c.shell.shell.session_mut().state_mut();
            let mut partner = runtime
                .data()
                .create_pokemon("VOYAGEAR", 40, Dv::default())
                .expect("native Strength partner");
            assert!(
                partner
                    .species
                    .tmhm_learnset
                    .iter()
                    .any(|m| m == "STRENGTH")
            );
            partner.item = Some("SMOKE_BALL".into());
            // Known moves are declared fixture state. Teaching the HM through
            // PACK requires a separate acquisition/teaching regression.
            partner.moves.retain(|m| m.name != "STRENGTH");
            if knows_strength {
                partner.moves.truncate(3);
                partner.moves.push(LearnedMove {
                    name: "STRENGTH".into(),
                    current_pp: runtime.data().moves["STRENGTH"].pp,
                    pp_ups: 0,
                });
            }
            state.storage.party.pokemon[0] = Some(partner);
            state.sync_party_from_storage();
            state.badges.johto[2] = badge;
            state
                .flags
                .set_engine_flag("ENGINE_PLAINBADGE", badge)
                .expect("declared Current Badge fixture");
        }
        mark_runtime_snapshot_dirty(&mut c.shell);
        // All subsequent travel, dialogue, pushes and clues use native input.
        assert!(super::northstar_ice_lair::path(&mut c, &runtime, (21, 10)).is_none());
        press(&mut c, GameButton::Right);
        press(&mut c, GameButton::A);
        if !badge || !knows_strength {
            finish(&mut c);
            super::winona::at(&mut c, "MnR03Cave1F", (12, 6));
            assert!(super::northstar_ice_lair::path(&mut c, &runtime, (21, 10)).is_none());
            continue;
        }
        choice(&mut c, true);
        finish(&mut c);
        for x in [14, 15] {
            press(&mut c, GameButton::Right);
            if c.snapshot().expect("pushed step").battle.is_some() {
                super::rochester::flee(&mut c);
            }
            let tile = c
                .shell
                .shell
                .session()
                .overworld()
                .object_runtime_tile_by_id("MN_R03_JAW_BOULDER")
                .expect("native pushed boulder");
            assert_eq!((tile.x, tile.y), (x, 6));
        }
        for (at, facing, flag) in [
            ((3, 5), GameButton::Left, "EVENT_MN_CASTOROCK_POOL"),
            ((19, 4), GameButton::Right, "EVENT_MN_CASTOROCK_PILLAR"),
            ((23, 12), GameButton::Down, "EVENT_MN_CASTOROCK_BED"),
        ] {
            walk(&mut c, &runtime, at);
            press(&mut c, facing);
            press(&mut c, GameButton::A);
            assert!(
                !c.shell
                    .shell
                    .session()
                    .state()
                    .flags
                    .is_event_flag_set(flag)
                    .expect("clue before acknowledgement")
            );
            finish(&mut c);
            assert!(
                c.shell
                    .shell
                    .session()
                    .state()
                    .flags
                    .is_event_flag_set(flag)
                    .expect("acknowledged observation")
            );
        }
        assert!(
            c.shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set("EVENT_MN_CASTOROCK_TRACKS_READ")
                .expect("discovery")
        );
        assert!(!caught(&c));
        assert!(
            c.snapshot()
                .expect("revealed Castorock")
                .visible_objects
                .iter()
                .any(|object| object.object_identifier.as_deref() == Some("MN_R03_JAW_CASTOROCK"))
        );
    }
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the incisor gallery"]
fn castorock_controller_decline_actual_capture_and_continue() {
    let (root, runtime) = load();
    // This isolates the encounter after discovery. It does not claim that the
    // Strength gate or three observations were earned by this fixture.
    let mut c = fixture(&root, &runtime, "MnR03Cave1F", (21, 10));
    {
        let state = c.shell.shell.session_mut().state_mut();
        state
            .flags
            .set_event_flag("EVENT_MN_CASTOROCK_TRACKS_READ", true)
            .expect("declared discovered-track fixture");
        state.storage.party.pokemon[0] = Some(
            runtime
                .data()
                .create_pokemon("VOYAGEAR", 40, Dv::default())
                .expect("native encounter partner"),
        );
        state.sync_party_from_storage();
        assert!(
            state
                .bag
                .add_item(&runtime.data().items["MASTER_BALL"], 1)
                .expect("declared capture item")
        );
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    // Continue runs the native map callback with the declared discovery state.
    // No state, party, item, flag, position or RNG writes below this line.
    c = continue_save(root.clone(), runtime.clone(), &mut c);
    press(&mut c, GameButton::Down);
    press(&mut c, GameButton::A);
    choice(&mut c, false);
    finish(&mut c);
    assert!(!caught(&c));
    assert_eq!(
        c.snapshot().expect("declined encounter").party.slots.len(),
        1
    );
    press(&mut c, GameButton::A);
    choice(&mut c, true);
    for _ in 0..128 {
        if c.snapshot().expect("static encounter").battle.is_some() {
            break;
        }
        c.wait_frames(64).expect("battle transition");
    }
    assert!(super::rochester_earned::try_recruit(&mut c, &runtime));
    finish(&mut c);
    assert!(caught(&c));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["MASTER_BALL"]),
        0
    );
    assert_eq!(c.snapshot().expect("capture party").party.slots.len(), 2);
    c = continue_save(root, runtime, &mut c);
    assert!(caught(&c));
    assert_eq!(c.snapshot().expect("continued party").party.slots.len(), 2);
    assert!(
        c.snapshot()
            .expect("captured actor hidden")
            .visible_objects
            .iter()
            .all(|object| object.object_identifier.as_deref() != Some("MN_R03_JAW_CASTOROCK"))
    );
}
