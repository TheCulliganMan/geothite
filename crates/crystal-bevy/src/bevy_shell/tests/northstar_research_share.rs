//! Prior research/capacity is declared; the award, TOSS and held-item use are input.
#[path = "research_share_battle.rs"]
mod held_battle;
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_core::models::bag::ITEM_POCKET_CAPACITY;
use crystal_core::models::item::ITEM_POCKET_ITEM;

use super::super::{
    FieldPackAction, FieldPackPocket, StartMenuOption, VisibleShellController,
    mark_runtime_snapshot_dirty, visible_selected_pack_item_actions, visible_start_menu_options,
};
use super::interior_life::{fixture, talk};
use super::r10_gallery::continue_save;
use super::winona::{load, press};

fn quantity(c: &VisibleShellController) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&c.shell.shell.runtime().data().items["EXP_SHARE"])
}
fn claimed(c: &VisibleShellController) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set("EVENT_MN_DEX_REWARD_2")
        .expect("research award flag")
}
fn give_share(c: &mut VisibleShellController) {
    press(c, GameButton::Start);
    let snapshot = c.snapshot().expect("native Start");
    let target = visible_start_menu_options(&c.shell, &snapshot)
        .iter()
        .position(|entry| *entry == StartMenuOption::Pack)
        .expect("PACK");
    for _ in 0..16 {
        let selected = c
            .shell
            .start_menu_cursor
            .as_ref()
            .expect("native Start cursor")
            .option_index;
        if selected == target {
            press(c, GameButton::A);
            break;
        }
        press(
            c,
            if selected < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    for _ in 0..4 {
        if c.shell.field_pack_pocket == Some(FieldPackPocket::Items) {
            break;
        }
        press(c, GameButton::Right);
    }
    let snapshot = c.snapshot().expect("real research reward in PACK");
    assert_eq!(
        snapshot.item_description("EXP_SHARE"),
        Some("Full bench EXP.\nwhen held.")
    );
    let target = snapshot
        .bag
        .items
        .iter()
        .position(|item| item.item_id == "EXP_SHARE")
        .expect("earned EXP. SHARE");
    for _ in 0..32 {
        let selected = c
            .shell
            .bag_cursor
            .as_ref()
            .expect("native item cursor")
            .option_index;
        if selected == target {
            press(c, GameButton::A);
            break;
        }
        press(
            c,
            if selected < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    let snapshot = c.snapshot().expect("native held-item actions");
    let actions =
        visible_selected_pack_item_actions(&snapshot, &c.shell, &FieldPackPocket::Items, false)
            .expect("item actions");
    let target = actions
        .iter()
        .position(|action| *action == FieldPackAction::Give)
        .expect("GIVE");
    for _ in 0..8 {
        let selected = c
            .shell
            .field_pack_action_cursor
            .as_ref()
            .expect("native action cursor")
            .option_index;
        if selected == target {
            press(c, GameButton::A);
            break;
        }
        press(
            c,
            if selected < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    for _ in 0..8 {
        if c.shell.party_cursor == 1 {
            break;
        }
        press(c, GameButton::Down);
    }
    assert_eq!(c.shell.party_cursor, 1, "native held-item recipient");
    press(c, GameButton::A);
    for _ in 0..8 {
        press(c, GameButton::B);
    }
    assert_eq!(quantity(c), 0);
    assert_eq!(
        c.shell.shell.session().state().storage.party.pokemon[1]
            .as_ref()
            .expect("recipient")
            .item
            .as_deref(),
        Some("EXP_SHARE")
    );
}

#[test]
#[ignore = "requires the reviewed story revision with the fifty-caught EXP. SHARE award"]
fn research_share_fifty_caught_native_capacity_toss_retry_held_and_continue() {
    let (root, r) = load();
    let mut c = fixture(&root, &r, "MnWinonaFieldStation", (2, 8));
    {
        let state = c.shell.shell.session_mut().state_mut();
        for flag in [
            "EVENT_MN_STARTER_RECEIVED",
            "EVENT_MN_WINONA_BALLS_RECEIVED",
            "EVENT_MN_DEX_REWARD_0",
            "EVENT_MN_DEX_REWARD_1",
        ] {
            state
                .flags
                .set_event_flag(flag, true)
                .expect("declared prior opening and rewards");
        }
        for name in r
            .data()
            .story_event_script_constants
            .global
            .keys()
            .filter_map(|key| key.strip_prefix("MN_DEX_MEMBER_"))
            .take(50)
        {
            state.pokedex.record_caught(&r.data().pokemon[name]);
        }
        for (species, level) in [("CINDORE", 40), ("DEWDANT", 5), ("DEWDANT", 5)] {
            let mut partner = r
                .data()
                .create_pokemon(species, level, Dv::default())
                .expect("declared native research partners");
            partner.original_trainer_id = state.player_id;
            partner.original_trainer_name = state.player_name.clone();
            assert!(state.storage.party.add_pokemon(partner));
        }
        state.sync_party_from_storage();
        for (id, item) in r
            .data()
            .items
            .iter()
            .filter(|(id, item)| item.pocket == ITEM_POCKET_ITEM && id.as_str() != "EXP_SHARE")
            .take(ITEM_POCKET_CAPACITY)
        {
            assert!(
                state
                    .bag
                    .add_item(item, 1)
                    .unwrap_or_else(|error| panic!("declared capacity {id}: {error}"))
            );
        }
        assert_eq!(state.bag.items.len(), ITEM_POCKET_CAPACITY);
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    // No session mutation below this boundary.
    let before = c.shell.shell.session().state().bag.clone();
    talk(&mut c, (2, 9));
    assert_eq!(c.shell.shell.session().state().bag, before);
    assert!(!claimed(&c));
    c = continue_save(root.clone(), r.clone(), &mut c);
    super::northstar_progression_rewards::native_toss_first_pocket_item(
        &mut c,
        &FieldPackPocket::Items,
    );
    for _ in 0..8 {
        press(&mut c, GameButton::B);
    }
    talk(&mut c, (2, 9));
    assert!(claimed(&c));
    assert_eq!(quantity(&c), 1);
    c = continue_save(root.clone(), r.clone(), &mut c);
    talk(&mut c, (2, 9));
    assert_eq!(quantity(&c), 1);
    give_share(&mut c);
    c = continue_save(root.clone(), r.clone(), &mut c);
    talk(&mut c, (2, 9));
    assert!(claimed(&c));
    assert_eq!(quantity(&c), 0);
    assert_eq!(
        c.shell.shell.session().state().storage.party.pokemon[1]
            .as_ref()
            .expect("continued recipient")
            .item
            .as_deref(),
        Some("EXP_SHARE")
    );
    held_battle::verify_full_bench_award(&mut c, &r);
    let earned = c.snapshot().expect("earned research battle").party;
    c = continue_save(root, r, &mut c);
    assert_eq!(
        c.snapshot().expect("continued earned experience").party,
        earned
    );
}
