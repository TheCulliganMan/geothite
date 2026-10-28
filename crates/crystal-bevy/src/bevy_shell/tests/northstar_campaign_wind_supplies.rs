//! Spend earned money and, if needed, sell an unused earned TM through the mart.
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::super::super::sellable_carried_item_ids;
use super::super::super::authored_route_fixture::indoor_walk;
use super::super::super::rochester_earned::checkout;
use super::super::super::winona::{press, travel};
use super::{CITY, VisibleShellController};

fn sell(c: &mut VisibleShellController, r: &CrystalRuntime, id: &str, count: u16) {
    for _ in 0..32 {
        if c.shell.shop_top_cursor.is_some()
            && c.shell.shop_welcome_seen
            && c.shell.shop_notice.is_none()
        {
            break;
        }
        press(c, GameButton::A);
    }
    let before = c.shell.shell.session().state().clone();
    let item = &r.data().items[id];
    assert!(before.bag.quantity(item) >= count && count > 0);
    assert!(
        item.tmhm_move.is_none()
            || c.snapshot()
                .expect("earned party")
                .party
                .slots
                .iter()
                .all(|s| !s
                    .pokemon
                    .species
                    .tmhm_learnset
                    .iter()
                    .any(|m| Some(m) == item.tmhm_move.as_ref()))
    );
    for _ in 0..4 {
        let index = c
            .shell
            .shop_top_cursor
            .as_ref()
            .expect("mart top")
            .option_index;
        if index == 1 {
            break;
        }
        press(
            c,
            if index < 1 {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    press(c, GameButton::A);
    let target = sellable_carried_item_ids(&c.snapshot().expect("native sell inventory"))
        .iter()
        .position(|carried| carried == id)
        .expect("earned unused TM");
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
            c,
            if index < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    press(c, GameButton::A);
    let quantity = c
        .shell
        .shop_quantity
        .as_ref()
        .expect("native sale quantity");
    assert!(quantity.selling);
    assert_eq!(quantity.item_id, id);
    for _ in 1..count {
        press(c, GameButton::Up);
    }
    press(c, GameButton::A);
    press(c, GameButton::A);
    let state = c.shell.shell.session().state();
    assert_eq!(state.bag.quantity(item), before.bag.quantity(item) - count);
    assert_eq!(
        state.money,
        before.money + u32::from(item.price) * u32::from(count) / 2
    );
    assert_eq!(state.flags, before.flags);
    assert_eq!(state.storage.party, before.storage.party);
    // Return to BUY through the same production shop; the sold TM stays gone.
    for _ in 0..48 {
        if c.shell.shop_notice.is_some() {
            press(c, GameButton::A);
        } else if c.shell.shop_top_cursor.is_some() {
            break;
        } else {
            press(c, GameButton::B);
        }
    }
    for _ in 0..4 {
        let index = c
            .shell
            .shop_top_cursor
            .as_ref()
            .expect("native mart top")
            .option_index;
        if index == 0 {
            break;
        }
        press(c, GameButton::Up);
    }
}

pub(super) fn buy(c: &mut VisibleShellController, r: &CrystalRuntime) {
    let medicine = &r.data().items["SUPER_POTION"];
    let party = c.snapshot().expect("earned party").party;
    let state = c.shell.shell.session().state();
    let needed = 3_u16.saturating_sub(state.bag.quantity(medicine));
    let sell_tm = ["TM_STEEL_WING", "TM_THUNDERPUNCH"].into_iter().find(|id| {
        let item = &r.data().items[*id];
        state.money < u32::from(medicine.price)
            && state.bag.quantity(item) == 1
            && party.slots.iter().all(|slot| {
                !slot
                    .pokemon
                    .species
                    .tmhm_learnset
                    .iter()
                    .any(|m| Some(m) == item.tmhm_move.as_ref())
            })
    });
    if needed == 0 || (state.money < u32::from(medicine.price) && sell_tm.is_none()) {
        return;
    }
    indoor_walk(c, r, (14, 10));
    travel(c, GameButton::Up, "MnStCloudMart", (6, 5));
    indoor_walk(c, r, (3, 3));
    press(c, GameButton::Left);
    press(c, GameButton::A);
    if let Some(id) = sell_tm {
        sell(c, r, id, 1);
    }
    let quantity = (c.shell.shell.session().state().money / u32::from(medicine.price))
        .min(u32::from(needed)) as u16;
    assert!(quantity > 0);
    checkout(c, r, "SUPER_POTION", quantity);
    indoor_walk(c, r, (6, 6));
    travel(c, GameButton::Down, CITY, (14, 10));
}

pub(super) fn wake(c: &mut VisibleShellController, r: &CrystalRuntime) {
    let item = &r.data().items["AWAKENING"];
    let balls = &r.data().items["POKE_BALL"];
    let state = c.shell.shell.session().state();
    if state.bag.quantity(item) > 0 {
        return;
    }
    let shortfall = u32::from(item.price).saturating_sub(state.money);
    let count = u16::try_from(shortfall.div_ceil(u32::from(balls.price) / 2))
        .expect("small native purchase");
    assert!(
        count <= state.bag.quantity(balls),
        "ordinary Awakening funding requires carried balls"
    );
    indoor_walk(c, r, (14, 10));
    travel(c, GameButton::Up, "MnStCloudMart", (6, 5));
    indoor_walk(c, r, (3, 3));
    press(c, GameButton::Left);
    press(c, GameButton::A);
    if count > 0 {
        sell(c, r, "POKE_BALL", count);
    }
    checkout(c, r, "AWAKENING", 1);
    indoor_walk(c, r, (6, 6));
    travel(c, GameButton::Down, CITY, (14, 10));
}

pub(super) fn balls(c: &mut VisibleShellController, r: &CrystalRuntime) {
    let item = &r.data().items["POKE_BALL"];
    let state = c.shell.shell.session().state();
    if state.bag.quantity(item) > 0 {
        return;
    }
    let count = (state.money / u32::from(item.price)).min(3) as u16;
    assert!(count > 0, "recruitment needs earned money for native balls");
    indoor_walk(c, r, (14, 10));
    travel(c, GameButton::Up, "MnStCloudMart", (6, 5));
    indoor_walk(c, r, (3, 3));
    press(c, GameButton::Left);
    press(c, GameButton::A);
    checkout(c, r, "POKE_BALL", count);
    indoor_walk(c, r, (6, 6));
    travel(c, GameButton::Down, CITY, (14, 10));
}
