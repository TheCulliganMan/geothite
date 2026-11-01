//! A declared discovered site; flee and cave exit/reentry use native input.
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};

use super::super::super::mark_runtime_snapshot_dirty;
use super::super::interior_life::fixture;
use super::super::northstar_ice_lair::choice;
use super::super::r10_gallery::continue_save;
use super::super::winona::{at, load, press};
use super::{caught, finish, walk};

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the incisor gallery"]
fn castorock_controller_run_and_defeat_return_paths_and_reentry() {
    for defeat in [false, true] {
        retry_after_encounter(defeat);
    }
}

fn retry_after_encounter(defeat: bool) {
    let (root, runtime) = load();
    let mut c = fixture(&root, &runtime, "MnR03Cave1F", (23, 12));
    {
        let state = c.shell.shell.session_mut().state_mut();
        for flag in ["EVENT_MN_CASTOROCK_POOL", "EVENT_MN_CASTOROCK_PILLAR"] {
            state
                .flags
                .set_event_flag(flag, true)
                .expect("declared earlier clues");
        }
        let mut partner = runtime
            .data()
            .create_pokemon("VOYAGEAR", 40, Dv::default())
            .expect("native return-path partner");
        assert!(
            partner
                .species
                .tmhm_learnset
                .iter()
                .any(|m| m == "STRENGTH")
        );
        partner.item = Some("SMOKE_BALL".into());
        partner.moves.truncate(3);
        partner.moves.push(LearnedMove {
            name: "STRENGTH".into(),
            current_pp: runtime.data().moves["STRENGTH"].pp,
            pp_ups: 0,
        });
        state.storage.party.pokemon[0] = Some(partner);
        state.sync_party_from_storage();
        state.badges.johto[2] = true;
        state
            .flags
            .set_engine_flag("ENGINE_PLAINBADGE", true)
            .expect("declared Current Badge");
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    // No state, party, inventory, flags, position or RNG writes below.
    press(&mut c, GameButton::Down);
    press(&mut c, GameButton::A);
    finish(&mut c);
    walk(&mut c, &runtime, (21, 10));
    press(&mut c, GameButton::Down);
    press(&mut c, GameButton::A);
    choice(&mut c, true);
    c.wait_frames(128).expect("optional encounter begins");
    assert!(
        c.snapshot()
            .expect("real static encounter")
            .battle
            .is_some()
    );
    if defeat {
        assert!(super::super::rochester_earned::try_battle(
            &mut c, &runtime, "WILD", ""
        ));
        assert_eq!(
            c.shell.shell.session().state().battle_result & 0x3f,
            0,
            "actual wild defeat, not RUN"
        );
    } else {
        super::super::rochester::flee(&mut c);
    }
    finish(&mut c);
    assert!(!caught(&c));
    assert!(
        c.snapshot()
            .expect("creature retreats for this visit")
            .visible_objects
            .iter()
            .all(|object| object.object_identifier.as_deref() != Some("MN_R03_JAW_CASTOROCK"))
    );
    assert!(
        !c.shell
            .shell
            .session()
            .state()
            .pokedex
            .has_caught("CASTOROCK")
    );
    assert_eq!(
        c.snapshot().expect("no capture on RUN").party.slots.len(),
        1
    );
    c = continue_save(root, runtime.clone(), &mut c);
    assert!(!caught(&c));
    assert!(
        c.snapshot()
            .expect("continued in-progress visit")
            .visible_objects
            .iter()
            .all(|object| object.object_identifier.as_deref() != Some("MN_R03_JAW_CASTOROCK"))
    );
    // Continue preserves the in-progress visit. The boulder is still at its
    // original throat; prove the return buffer works without taking the pickup.
    walk(&mut c, &runtime, (14, 6));
    press(&mut c, GameButton::Left);
    press(&mut c, GameButton::A);
    choice(&mut c, true);
    finish(&mut c);
    for x in [12, 11, 10] {
        press(&mut c, GameButton::Left);
        c.wait_frames(64).expect("reverse push settles");
        let tile = c
            .shell
            .shell
            .session()
            .overworld()
            .object_runtime_tile_by_id("MN_R03_JAW_BOULDER")
            .expect("return boulder");
        assert_eq!((tile.x, tile.y), (x, 6));
        press(&mut c, GameButton::Left);
        c.wait_frames(64).expect("walk into vacated tile");
        if c.snapshot().expect("return step").battle.is_some() {
            super::super::rochester::flee(&mut c);
        }
        at(&mut c, "MnR03Cave1F", (x + 1, 6));
    }
    walk(&mut c, &runtime, (7, 8));
    for _ in 0..12 {
        if c.snapshot().expect("original cave exit").overworld.map_name != "MnR03Cave1F" {
            break;
        }
        press(&mut c, GameButton::Down);
        c.wait_frames(64).expect("original cave exit settles");
        if c.snapshot().expect("exit encounter").battle.is_some() {
            super::super::rochester::flee(&mut c);
        }
    }
    assert_eq!(
        c.snapshot().expect("outside cave").overworld.map_name,
        "MnR03C00R00",
        "exit state: {:?}",
        c.snapshot().expect("exit diagnosis").overworld
    );
    for _ in 0..12 {
        if c.snapshot().expect("cave reentry").overworld.map_name == "MnR03Cave1F" {
            break;
        }
        if c.snapshot().expect("outside encounter").battle.is_some() {
            super::super::rochester::flee(&mut c);
        }
        press(&mut c, GameButton::Up);
    }
    // A ladder warp arrives on its exact target tile, not the dry floor used
    // by the local access fixtures and reachability proof.
    at(&mut c, "MnR03Cave1F", (7, 9));
    c.wait_frames(128).expect("native ladder arrival settles");
    walk(&mut c, &runtime, (7, 8));
    at(&mut c, "MnR03Cave1F", (7, 8));
    assert!(!caught(&c));
    let overworld = c.shell.shell.session().overworld();
    let actor = overworld
        .objects
        .iter()
        .find(|object| object.object_identifier.as_deref() == Some("MN_R03_JAW_CASTOROCK"))
        .expect("original creature actor");
    assert!(
        overworld.is_object_visible(actor),
        "map callback restores the uncaught creature"
    );
    assert!(
        !c.shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_R03_ITEM_02")
            .expect("untaken original pickup")
    );
}
