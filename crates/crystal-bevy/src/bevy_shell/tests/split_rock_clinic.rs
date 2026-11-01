//! Explicit party/poison fixtures; real clinic travel, services and recovery use input.
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use serde_json::Value;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::interior_life::{fixture, load, talk, walk};
use super::r10_gallery::{continue_save, press};

const ROOM: &str = "MnSplitRockPokecenter1F";
const SHORE: &str = "MnR11C00R02";

#[track_caller]
fn at(c: &mut VisibleShellController, map: &str, point: (i16, i16)) {
    let s = c.snapshot().expect("native clinic position").overworld;
    assert_eq!((s.map_name.as_str(), (s.tile.x, s.tile.y)), (map, point));
}

fn logical_flags(c: &VisibleShellController) -> Value {
    let mut flags = serde_json::to_value(&c.shell.shell.session().state().flags).expect("flags");
    // The production flag getters treat a missing key and stored false equally.
    // Map entry/Continue may materialize false movement flags without setting them.
    for bank in flags.as_object_mut().expect("flag banks").values_mut() {
        bank.as_object_mut()
            .expect("flag map")
            .retain(|_, value| value.as_bool().expect("flag value"));
    }
    flags
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with the composed Split Rock clinic"]
fn split_rock_clinic_controller_public_approach_people_both_carpets_and_continue() {
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, SHORE, (34, 47));
    let money = c.shell.shell.session().state().money;
    let bag = c.shell.shell.session().state().bag.clone();
    let flags = logical_flags(&c);
    walk(&mut c, &r, (33, 33));
    at(&mut c, ROOM, (3, 5));
    walk(&mut c, &r, (2, 5));
    let words = talk(&mut c, (1, 5));
    assert!(
        words.contains("granddaughter") && words.contains("mittens") && words.contains("stitch")
    );
    walk(&mut c, &r, (6, 5));
    let words = talk(&mut c, (6, 4));
    assert!(
        words.contains("fog") && words.contains("Coming home") && words.contains("what matters")
    );
    c = continue_save(root.clone(), r.clone(), &mut c);
    for carpet in [3, 4] {
        walk(&mut c, &r, (carpet, 7));
        at(&mut c, SHORE, (33, 34));
        walk(&mut c, &r, (31, 34));
        let words = talk(&mut c, (30, 34));
        assert!(words.contains("SHORE CLINIC") && words.contains("Care is free"));
        walk(&mut c, &r, (33, 33));
        at(&mut c, ROOM, (3, 5));
    }
    press(&mut c, GameButton::Start);
    assert!(c.snapshot().expect("native Start").ui.menu.is_some());
    press(&mut c, GameButton::B);
    assert!(
        c.snapshot()
            .expect("Start releases input")
            .ui
            .menu
            .is_none()
    );
    walk(&mut c, &r, (2, 5));
    assert_eq!(c.shell.shell.session().state().money, money);
    assert_eq!(c.shell.shell.session().state().bag, bag);
    assert_eq!(logical_flags(&c), flags);
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with the composed Split Rock clinic"]
fn split_rock_clinic_controller_free_care_pc_and_actual_poison_recovery() {
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, SHORE, (33, 34));
    let mut partner = r
        .data()
        .create_pokemon("VOYAGEAR", 25, Dv::default())
        .expect("regional care fixture");
    partner.hp = 10;
    partner.status = Some("POISON".into());
    for move_ in &mut partner.moves {
        move_.current_pp = 0;
    }
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    state.money = 0;
    let badges = state.badges.clone();
    mark_runtime_snapshot_dirty(&mut c.shell);
    walk(&mut c, &r, (33, 33));
    at(&mut c, ROOM, (3, 5));
    walk(&mut c, &r, (3, 3));
    talk(&mut c, (3, 2));
    let snapshot = c.snapshot().expect("production nurse care");
    let healed = &snapshot.party.slots[0].pokemon;
    assert_eq!(healed.hp, healed.max_hp);
    assert!(healed.status.is_none());
    assert!(
        healed
            .moves
            .iter()
            .all(|m| m.current_pp == r.data().moves[&m.name].pp)
    );
    let state = c.shell.shell.session().state();
    assert_eq!(state.money, 0);
    assert_eq!(state.badges, badges);
    assert_eq!(
        state.last_spawn_map_constant.as_deref(),
        Some("MN_SPLIT_ROCK_POKECENTER_1F")
    );
    walk(&mut c, &r, (9, 2));
    press(&mut c, GameButton::Up);
    for _ in 0..96 {
        press(&mut c, GameButton::A);
        if c.shell.pc_hub_cursor.is_some() {
            break;
        }
    }
    assert!(c.shell.pc_hub_cursor.is_some(), "native PC opens");
    let point = c.snapshot().expect("PC owns input").overworld.tile;
    press(&mut c, GameButton::Left);
    assert_eq!(
        c.snapshot().expect("PC blocks walking").overworld.tile,
        point
    );
    for _ in 0..96 {
        if c.shell.pc_hub_cursor.is_none() && !c.has_pending_script_work() {
            break;
        }
        press(&mut c, GameButton::B);
    }
    c = continue_save(root.clone(), r.clone(), &mut c);
    walk(&mut c, &r, (3, 7));
    at(&mut c, SHORE, (33, 34));
    // A separate explicit poisoning fixture starts here. The registered recovery
    // is the one earned from the real nurse above; no position/warp is rewritten.
    let state = c.shell.shell.session_mut().state_mut();
    let battle_result = state.battle_result;
    let partner = state.storage.party.pokemon[0]
        .as_mut()
        .expect("regional partner");
    partner.hp = 1;
    partner.status = Some("POISON".into());
    state.sync_party_from_storage();
    state.money = 101;
    mark_runtime_snapshot_dirty(&mut c.shell);
    let mut walked = false;
    for _ in 0..128 {
        if c.shell.field_notice.is_some()
            || !c.shell.battle_messages.is_empty()
            || c.shell.visible_blackout_phase.is_some()
            || c.snapshot()
                .expect("poison input owner")
                .ui
                .text_window_open
        {
            press(&mut c, GameButton::A);
        } else {
            for button in [
                GameButton::Right,
                GameButton::Right,
                GameButton::Left,
                GameButton::Left,
            ] {
                let before = c.snapshot().expect("poison walk origin").overworld;
                press(&mut c, button);
                let after = c.snapshot().expect("poison walk result").overworld;
                walked |= before.map_name == SHORE
                    && after.map_name == SHORE
                    && before.tile != after.tile;
            }
        }
        if c.snapshot().expect("native recovery").overworld.map_name == ROOM {
            break;
        }
    }
    assert!(walked, "poison recovery must follow actual native walking");
    at(&mut c, ROOM, (3, 5));
    let recovered = c.snapshot().expect("actual clinic recovery");
    assert_eq!(
        recovered.party.slots[0].pokemon.hp,
        recovered.party.slots[0].pokemon.max_hp
    );
    assert!(recovered.party.slots[0].pokemon.status.is_none());
    assert_eq!(c.shell.shell.session().state().money, 50);
    assert_eq!(c.shell.shell.session().state().battle_result, battle_result);
    assert_eq!(c.shell.shell.session().state().badges, badges);
    c = continue_save(root, r, &mut c);
    at(&mut c, ROOM, (3, 5));
}
