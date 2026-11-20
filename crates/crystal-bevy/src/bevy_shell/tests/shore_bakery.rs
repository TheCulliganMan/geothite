//! Route 11's pastry is bought over a counter, packed, shared and carried into battle.
use std::path::PathBuf;

use crystal_core::input::GameButton::{A, B, Down, Left, Right, Start, Up};
use crystal_core::models::Dv;
use crystal_core::models::bag::ITEM_POCKET_CAPACITY;

use super::super::brainerd_guide::capture;
use super::super::minnesota_fossils::{finish, talk};
use super::super::northstar_ice_lair::choice;
use super::super::winona::{at, continue_in_place, press, travel};
use super::{VisibleShellController, fixture, load, mark_runtime_snapshot_dirty, walk};
use crate::bevy_shell::{
    FieldPackAction, FieldPackPocket, StartMenuOption, visible_field_dialog_pages,
    visible_selected_pack_item_actions, visible_start_menu_options,
};

const SHOP: &str = "MnNorthShorePieShop";
const ROAD: &str = "MnR11C00R01";
const PIE: &str = "MN_BLUEBERRY_SLICE";

fn purchase_state(c: &VisibleShellController) -> (u32, u16) {
    let state = c.shell.shell.session().state();
    (
        state.money,
        state
            .bag
            .quantity(&c.shell.shell.runtime().data().items[PIE]),
    )
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with the North Shore bakery"]
fn shore_bakery_sells_a_slice_that_heals_in_the_field_and_against_ivo() {
    let (root, runtime, _) = load();
    let mut c = fixture(&root, &runtime, ROAD, (31, 36));
    let mut mon = runtime
        .data()
        .create_pokemon("BANKWING", 85, Dv::default())
        .expect("regional flier");
    assert!(mon.hp > 101);
    mon.hp = 1;
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(mon));
    state.sync_party_from_storage();
    state.money = 700;
    mark_runtime_snapshot_dirty(&mut c.shell);

    travel(&mut c, Up, SHOP, (3, 6));
    walk(&mut c, &runtime, (3, 4));
    press(&mut c, Up);
    press(&mut c, A);
    assert_eq!(
        c.snapshot()
            .expect("pie note")
            .ui
            .text
            .expect("table's family note")
            .label,
        "MnShorePieTableText"
    );
    finish(&mut c);
    walk(&mut c, &runtime, (5, 3));
    press(&mut c, Right);
    press(&mut c, A);
    c.wait_frames(128).expect("printed bakery offer");
    let snap = c.shell.shell.presentation_snapshot().expect("bakery offer");
    let pages = visible_field_dialog_pages(&snap, &c.shell).expect("posted slice price");
    assert_eq!(
        pages.first().map(String::as_str),
        Some("Blueberry pie!\nA slice for 350?")
    );
    let frames = std::env::var_os("MN_LAKESIDE_FRAMES").map(PathBuf::from);
    c = capture(c, frames.as_deref(), "north-shore-pie-counter.png");
    choice(&mut c, false);
    finish(&mut c);
    assert_eq!(purchase_state(&c), (700, 0));
    for visit in 1..=2 {
        talk(&mut c, Right, &[true]);
        assert_eq!(purchase_state(&c), (700 - 350 * u32::from(visit), visit));
    }
    talk(&mut c, Right, &[true]);
    assert_eq!(purchase_state(&c), (0, 2));
    walk(&mut c, &runtime, (2, 7));
    at(&mut c, ROAD, (31, 36));
    continue_in_place(&mut c, &root, &runtime, "north-shore-paid-pie");
    assert_eq!(purchase_state(&c), (0, 2));

    // Both servings occupy the first and only item row in the visitor's Pack.
    press(&mut c, Start);
    let snap = c.snapshot().expect("Start");
    let pack = visible_start_menu_options(&c.shell, &snap)
        .iter()
        .position(|op| *op == StartMenuOption::Pack)
        .expect("Pack");
    let start = c
        .shell
        .start_menu_cursor
        .as_ref()
        .expect("fresh menu")
        .option_index;
    for _ in 0..pack.abs_diff(start) {
        press(&mut c, if pack > start { Down } else { Up });
    }
    press(&mut c, A);
    assert_eq!(
        c.snapshot().expect("carried slice").bag.items[0].item_id,
        PIE
    );
    press(&mut c, A);
    let snap = c.snapshot().expect("edible slice");
    let actions =
        visible_selected_pack_item_actions(&snap, &c.shell, &FieldPackPocket::Items, false)
            .expect("item menu");
    assert_eq!(actions.first(), Some(&FieldPackAction::Use));
    press(&mut c, A);
    press(&mut c, A);
    assert_eq!(
        c.snapshot().expect("fed partner").party.slots[0].pokemon.hp,
        51
    );
    assert_eq!(purchase_state(&c), (0, 1));
    for _ in 0..24 {
        if c.shell.field_notice.is_some() {
            press(&mut c, A);
        } else if c.shell.field_pack_pocket.is_some() || c.shell.start_menu_cursor.is_some() {
            press(&mut c, B);
        } else {
            break;
        }
    }
    // The east verge brings the player into Bird Keeper Ivo's line of sight.
    travel(&mut c, Down, ROAD, (31, 37));
    travel(&mut c, Right, ROAD, (37, 37));
    travel(&mut c, Up, ROAD, (37, 35));
    for _ in 0..128 {
        if c.shell.battle_action_cursor.is_some()
            && c.shell.battle_messages.is_empty()
            && c.shell.visible_battle_transition.is_none()
            && c.shell.visible_battle_sliding_intro.is_none()
            && c.shell.visible_send_out_animation.is_none()
        {
            break;
        }
        press(&mut c, A);
    }
    let battle = c
        .snapshot()
        .expect("bird keeper")
        .battle
        .expect("on-sight trainer battle");
    assert!(
        matches!(battle.kind,crystal_runtime::RuntimeBattleKind::Trainer{ref trainer_id,..} if trainer_id=="MN_R11_T06")
    );
    press(&mut c, Down);
    press(&mut c, A);
    assert_eq!(c.snapshot().expect("battle Pack").bag.items[0].item_id, PIE);
    press(&mut c, A);
    press(&mut c, A);
    assert!(c.shell.battle_pack_target_mode.is_some());
    press(&mut c, A);
    let after = c.snapshot().expect("pie and opponent response");
    assert!((52..=101).contains(&after.party.slots[0].pokemon.hp));
    assert!(after.battle.expect("fight continues").enemy_turns_taken > battle.enemy_turns_taken);
    assert_eq!(purchase_state(&c), (0, 0));
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with the North Shore bakery"]
fn shore_bakery_leaves_the_money_in_a_customers_wallet_when_the_pack_is_full() {
    let (root, runtime, _) = load();
    let mut c = fixture(&root, &runtime, SHOP, (5, 3));
    let state = c.shell.shell.session_mut().state_mut();
    state.money = 350;
    while state.bag.items.len() < ITEM_POCKET_CAPACITY {
        assert!(
            state
                .bag
                .add_item(&runtime.data().items["POTION"], 99)
                .expect("fill an Item slot")
        );
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    talk(&mut c, Right, &[true]);
    assert_eq!(purchase_state(&c), (350, 0));
    walk(&mut c, &runtime, (3, 7));
    at(&mut c, ROAD, (31, 36));
    travel(&mut c, Left, ROAD, (30, 36));
}
