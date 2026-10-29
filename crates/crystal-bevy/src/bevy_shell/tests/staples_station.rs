//! Initial position/money are fixtures; every visit and purchase uses production input.
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::interior_life::{fixture, load, talk, walk};
use super::r10_gallery::{continue_save, press};

const ROAD: &str = "MnR15C00R00";
const JUNCTION: &str = "MnStaplesRailJunction";
const ROOM: &str = "MnStaplesStation";
const VISITED: &str = "EVENT_MN_STATION_STAPLES_VISITED";
const MART: &str = "MN_STAPLES_PARCELS";

fn visited(c: &VisibleShellController) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(VISITED)
        .expect("native event")
}

#[track_caller]
fn at(c: &mut VisibleShellController, map: &str, point: (i16, i16)) {
    let s = c.snapshot().expect("native Staples position").overworld;
    assert_eq!((s.map_name.as_str(), (s.tile.x, s.tile.y)), (map, point));
}

fn step(c: &mut VisibleShellController, button: GameButton, map: &str, point: (i16, i16)) {
    for _ in 0..12 {
        press(c, button);
        let s = c.snapshot().expect("native spur step").overworld;
        if s.map_name == map && (s.tile.x, s.tile.y) == point {
            return;
        }
    }
    at(c, map, point);
}

fn other_flags(c: &VisibleShellController) -> Value {
    let mut flags = serde_json::to_value(&c.shell.shell.session().state().flags).expect("flags");
    // Missing and stored-false flags have identical native getter behavior.
    for bank in flags.as_object_mut().expect("banks").values_mut() {
        bank.as_object_mut()
            .expect("flag bank")
            .retain(|key, value| key != VISITED && value.as_bool().expect("boolean"));
    }
    flags
}

fn enter(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk(c, r, (13, 7));
    at(c, ROOM, (9, 15));
    assert!(visited(c), "actual room entry records Staples alone");
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with the native Staples station"]
fn staples_station_controller_real_spur_people_carpets_and_remembered_visit() {
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, ROAD, (22, 12));
    let money = c.shell.shell.session().state().money;
    let bag = c.shell.shell.session().state().bag.clone();
    let flags = other_flags(&c);
    assert!(!visited(&c));
    // Walk from an unchanged old road tile through the original encounter grass.
    // The empty-party fixture does not claim a battle or earned regional access.
    for x in (18..22).rev() {
        step(&mut c, GameButton::Left, ROAD, (x, 12));
    }
    for y in (9..12).rev() {
        step(&mut c, GameButton::Up, ROAD, (18, y));
    }
    for lane in 0..2 {
        walk(&mut c, &r, (26 + lane, 0));
        step(&mut c, GameButton::Up, JUNCTION, (6 + lane, 19));
        step(&mut c, GameButton::Up, JUNCTION, (6 + lane, 18));
        assert!(!visited(&c), "the outdoor outpost is not a hall visit");
        walk(&mut c, &r, (6 + lane, 19));
        step(&mut c, GameButton::Down, ROAD, (26 + lane, 0));
        step(&mut c, GameButton::Down, ROAD, (26 + lane, 1));
    }
    walk(&mut c, &r, (26, 0));
    step(&mut c, GameButton::Up, JUNCTION, (6, 19));
    step(&mut c, GameButton::Up, JUNCTION, (6, 18));
    walk(&mut c, &r, (18, 9));
    assert!(talk(&mut c, (18, 8)).contains("STAPLES DEPOT"));
    walk(&mut c, &r, (17, 14));
    let words = talk(&mut c, (18, 14));
    assert!(words.contains("BRAINERD") && words.contains("DETROIT LAKES"));
    assert!(
        !visited(&c),
        "reading and talking outside cannot unlock a destination"
    );
    enter(&mut c, &r);
    walk(&mut c, &r, (3, 12));
    let words = talk(&mut c, (3, 11));
    assert!(words.contains("night switch") && words.contains("I still light it"));
    walk(&mut c, &r, (4, 11));
    for _ in 0..4 {
        press(&mut c, GameButton::Up);
        at(&mut c, ROOM, (4, 11));
    }
    for _ in 0..2 {
        let words = talk(&mut c, (4, 10));
        assert!(words.contains("signal ledger") && words.contains("worth a life"));
        at(&mut c, ROOM, (4, 11));
    }
    walk(&mut c, &r, (12, 14));
    let words = talk(&mut c, (13, 14));
    assert!(words.contains("seed packet") && words.contains("flower, too"));
    walk(&mut c, &r, (11, 10));
    for _ in 0..4 {
        press(&mut c, GameButton::Up);
    }
    at(&mut c, ROOM, (11, 10));
    assert!(talk(&mut c, (11, 9)).contains("not ready yet"));
    press(&mut c, GameButton::Start);
    assert!(
        c.shell.start_menu_cursor.is_some(),
        "conductor releases input"
    );
    press(&mut c, GameButton::B);
    c = continue_save(root.clone(), r.clone(), &mut c);
    assert!(visited(&c));
    for x in [8, 9] {
        walk(&mut c, &r, (x, 17));
        at(&mut c, JUNCTION, (13, 8));
        enter(&mut c, &r);
    }
    assert_eq!(c.shell.shell.session().state().money, money);
    assert_eq!(c.shell.shell.session().state().bag, bag);
    assert_eq!(
        other_flags(&c),
        flags,
        "no badge, courier or other station flag is granted"
    );
}

fn open_shop(c: &mut VisibleShellController, r: &CrystalRuntime) {
    enter(c, r);
    walk(c, r, (5, 14));
    press(c, GameButton::Up);
    press(c, GameButton::A);
    for _ in 0..64 {
        if c.shell.shop_top_cursor.is_some()
            && c.shell.shop_welcome_seen
            && c.shell.shop_notice.is_none()
        {
            break;
        }
        press(c, GameButton::A);
    }
    let shop = c
        .snapshot()
        .expect("parcel counter")
        .pending_shop
        .expect("production shop");
    assert_eq!(shop.mart_id, MART);
    assert_eq!(shop.inventory, r.data().marts.0[MART]);
    press(c, GameButton::Right);
    at(c, ROOM, (5, 14));
}

fn close_shop(c: &mut VisibleShellController) {
    for _ in 0..96 {
        let s = c.snapshot().expect("shop closing");
        if s.pending_shop.is_none() && !s.ui.text_window_open && !c.has_pending_script_work() {
            break;
        }
        let button = if s.pending_shop.is_none() || c.shell.shop_notice.is_some() {
            GameButton::A
        } else if c.shell.shop_quantity.is_some() {
            GameButton::B
        } else if let Some(cursor) = &c.shell.shop_top_cursor {
            if cursor.option_index == 2 {
                GameButton::A
            } else {
                GameButton::Down
            }
        } else {
            GameButton::B
        };
        press(c, button);
    }
    assert!(c.snapshot().expect("closed counter").pending_shop.is_none());
    press(c, GameButton::Start);
    assert!(c.shell.start_menu_cursor.is_some(), "shop releases input");
    press(c, GameButton::B);
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with Staples parcel stock"]
fn staples_station_controller_parcel_purchase_price_denial_and_continue() {
    let (root, r, _) = load();
    let item = &r.data().items["REPEL"];
    let mut c = fixture(&root, &r, JUNCTION, (13, 8));
    c.shell.shell.session_mut().state_mut().money = 10_000;
    mark_runtime_snapshot_dirty(&mut c.shell);
    let quantity = c.shell.shell.session().state().bag.quantity(item);
    let flags = other_flags(&c);
    open_shop(&mut c, &r);
    press(&mut c, GameButton::A); // BUY.
    press(&mut c, GameButton::A); // First stock item.
    assert_eq!(
        c.shell.shop_quantity.as_ref().expect("quantity").item_id,
        "REPEL"
    );
    press(&mut c, GameButton::A); // Quantity one; confirmation still owns payment.
    assert_eq!(c.shell.shell.session().state().money, 10_000);
    press(&mut c, GameButton::A);
    assert_eq!(
        c.shell.shell.session().state().money,
        10_000 - u32::from(item.price)
    );
    assert_eq!(
        c.shell.shell.session().state().bag.quantity(item),
        quantity + 1
    );
    close_shop(&mut c);
    c = continue_save(root.clone(), r.clone(), &mut c);
    assert!(visited(&c));
    assert_eq!(
        c.shell.shell.session().state().bag.quantity(item),
        quantity + 1
    );
    assert_eq!(other_flags(&c), flags);
    walk(&mut c, &r, (9, 17));
    at(&mut c, JUNCTION, (13, 8));
    let mut poor = fixture(&root, &r, JUNCTION, (13, 8));
    poor.shell.shell.session_mut().state_mut().money = 0;
    mark_runtime_snapshot_dirty(&mut poor.shell);
    let bag = poor.shell.shell.session().state().bag.clone();
    open_shop(&mut poor, &r);
    for _ in 0..4 {
        press(&mut poor, GameButton::A);
    }
    assert_eq!(
        poor.shell.shop_notice.as_deref(),
        Some("You don't have\nenough money.")
    );
    assert_eq!(poor.shell.shell.session().state().money, 0);
    assert_eq!(poor.shell.shell.session().state().bag, bag);
    close_shop(&mut poor);
    walk(&mut poor, &r, (8, 17));
    at(&mut poor, JUNCTION, (13, 8));
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with the active Minnesota Central Lakes encounters"]
fn staples_station_controller_original_grass_produces_regional_wild_battle_and_run() {
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, ROAD, (22, 12));
    let partner = r
        .data()
        .create_pokemon("VOYAGEAR", 80, Dv::default())
        .expect("explicit regional encounter fixture partner");
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    // No party/position/progression writes after this initial setup. Real native
    // encounter checks run on the original grass beside the new station spur.
    for i in 0..256 {
        press(
            &mut c,
            if (i / 4) % 2 == 0 {
                GameButton::Left
            } else {
                GameButton::Right
            },
        );
        if let Some(battle) = c.snapshot().expect("native regional grass").battle {
            assert!(matches!(
                battle.kind,
                crystal_runtime::RuntimeBattleKind::Wild { .. }
            ));
            assert!(
                battle.enemy_pokemon.species.int_id > 251,
                "the table's active-roster audit must also execute a Minnesota battle"
            );
            super::rochester::flee(&mut c);
            assert!(
                c.snapshot()
                    .expect("native RUN returns to road")
                    .battle
                    .is_none()
            );
            assert!(
                !visited(&c),
                "a wild battle cannot count as entering the station"
            );
            return;
        }
    }
    panic!("the original Central Lakes grass never produced a native encounter");
}
