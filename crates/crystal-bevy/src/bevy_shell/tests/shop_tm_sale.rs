//! Owned TM inventory must remain sellable through the production mart.
use crystal_core::input::GameButton;

use super::super::{
    VisibleShellController, mark_runtime_snapshot_dirty, sellable_carried_item_ids,
};
use super::initialized_mart_shell;
use super::winona::press;

#[test]
fn renderer_neutral_mart_projects_its_live_owner_without_extra_wait_frames() {
    let mut shell = initialized_mart_shell();
    shell
        .shell
        .add_bag_item("TM_STEEL_WING", 1)
        .expect("initial owned TM");
    shell.shop_welcome_seen = false;
    mark_runtime_snapshot_dirty(&mut shell);
    super::super::settle_visible_shell_until_input(&mut shell).expect("initial native welcome");
    let mut c = VisibleShellController { shell };
    let welcome = c.snapshot().expect("visible welcome");
    assert!(
        welcome
            .ui
            .text
            .and_then(|t| t.asm_text)
            .is_some_and(|t| t.contains("Welcome!"))
    );
    c.press(GameButton::A).expect("acknowledge welcome");
    let menu = c
        .snapshot()
        .expect("production mart top")
        .ui
        .menu
        .expect("visible shop menu");
    assert!(
        menu.layout.vertical_menus[0]
            .options
            .iter()
            .any(|s| s.contains(">BUY"))
    );
    c.press(GameButton::Down).expect("select SELL");
    c.press(GameButton::A).expect("open native sell list");
    let snapshot = c.snapshot().expect("owned TM in visible sell list");
    let target = sellable_carried_item_ids(&snapshot)
        .iter()
        .position(|id| id == "TM_STEEL_WING")
        .expect("owned TM");
    for _ in 0..64 {
        let index = c
            .shell
            .sell_cursor
            .as_ref()
            .expect("native sell cursor")
            .option_index;
        if index == target {
            break;
        }
        c.press(if index < target {
            GameButton::Down
        } else {
            GameButton::Up
        })
        .expect("native inventory selection");
    }
    c.press(GameButton::A).expect("quantity");
    assert!(
        c.snapshot()
            .expect("visible quantity")
            .ui
            .text
            .and_then(|t| t.asm_text)
            .is_some_and(|t| t.contains("How many?"))
    );
    c.press(GameButton::A).expect("confirmation");
    let menu = c
        .snapshot()
        .expect("visible confirmation")
        .ui
        .menu
        .expect("YES/NO owner");
    assert!(
        menu.layout.vertical_menus[0]
            .options
            .iter()
            .any(|s| s == ">YES")
    );
    c.press(GameButton::B).expect("cancel confirmation");
    let inventory = c.snapshot().expect("return to inventory");
    assert!(inventory.ui.menu.is_some());
    assert!(inventory.ui.text.is_none());
}

#[test]
fn mart_tm_sale_reads_owned_count_and_waits_for_confirmation() {
    let mut shell = initialized_mart_shell();
    // Only the initial carried TM/money and already-open shop are fixtures.
    // Every later selection, cancel and transaction uses production input.
    shell
        .shell
        .add_bag_item("TM_STEEL_WING", 1)
        .expect("initial owned TM");
    shell.shell.session_mut().state_mut().money = 100;
    mark_runtime_snapshot_dirty(&mut shell);
    let mut c = VisibleShellController { shell };
    let initial = c.snapshot().expect("initial mart inventory");
    let item = c.shell.shell.runtime().data().items["TM_STEEL_WING"].clone();
    assert_eq!(
        super::super::carried_item_quantity(&initial, "TM_STEEL_WING"),
        Some(1)
    );
    assert!(!super::super::shop_sell_item_label(&initial, "TM_STEEL_WING").contains("INVALID"));
    let target = sellable_carried_item_ids(&initial)
        .iter()
        .position(|id| id == "TM_STEEL_WING")
        .expect("TM in native sell list");
    press(&mut c, GameButton::Down);
    press(&mut c, GameButton::A);
    for _ in 0..64 {
        let index = c
            .shell
            .sell_cursor
            .as_ref()
            .expect("native sell cursor")
            .option_index;
        if index == target {
            break;
        }
        press(
            &mut c,
            if index < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    press(&mut c, GameButton::A);
    let sale = c
        .shell
        .shop_quantity
        .as_ref()
        .expect("owned TM quantity prompt");
    assert!(sale.selling);
    assert_eq!(sale.max_quantity, 1);
    assert_eq!(sale.item_id, "TM_STEEL_WING");
    press(&mut c, GameButton::B);
    assert_eq!(c.shell.shell.session().state().money, 100);
    assert_eq!(c.shell.shell.session().state().bag.quantity(&item), 1);
    press(&mut c, GameButton::A);
    press(&mut c, GameButton::A);
    assert!(
        c.shell
            .shop_quantity
            .as_ref()
            .expect("sale confirmation")
            .confirmation
            .is_some()
    );
    assert_eq!(c.shell.shell.session().state().money, 100);
    assert_eq!(c.shell.shell.session().state().bag.quantity(&item), 1);
    press(&mut c, GameButton::A);
    assert_eq!(
        c.shell.shell.session().state().money,
        100 + u32::from(item.price) / 2
    );
    assert_eq!(c.shell.shell.session().state().bag.quantity(&item), 0);
    assert!(c.shell.shop_notice.is_some());
    assert_eq!(c.snapshot().expect("after TM sale").party, initial.party);
}
