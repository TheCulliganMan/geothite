//! Cross the public threshold in each storefront and leave by both carpet tiles.
use crystal_core::input::GameButton::{A, B, Down, Right, Up};

use super::super::winona::travel;
use super::{Point, at, collision, fixture, load, press, walk};

const DULUTH: &str = "MnDuluth";
const MINNEAPOLIS: &str = "MnMinneapolis";
const ST_PAUL: &str = "MnStPaul";

/// City, shop, outside doorstep and open floor immediately inside the door.
const VISITS: &[(&str, &str, Point, Point)] = &[
    (DULUTH, "MnDuluthHillSupplies", (86, 10), (6, 10)),
    (DULUTH, "MnDuluthLakeOutfitter", (116, 22), (6, 10)),
    (DULUTH, "MnDuluthToolExchange", (24, 44), (6, 10)),
    (DULUTH, "MnDuluthMaritimeCoop1F", (106, 58), (8, 10)),
    (DULUTH, "MnDuluthDockSupplies", (12, 78), (6, 10)),
    (DULUTH, "MnDuluthPointOutfitter", (90, 78), (6, 10)),
    (MINNEAPOLIS, "MnMinneapolisSupplyCoop", (9, 12), (6, 6)),
    (MINNEAPOLIS, "MnMinneapolisGardenShop", (47, 18), (6, 6)),
    (
        MINNEAPOLIS,
        "MnMinneapolisDepartmentStore1F",
        (54, 36),
        (7, 6),
    ),
    (MINNEAPOLIS, "MnMinneapolisSouthMarket", (62, 60), (6, 6)),
    (ST_PAUL, "MnStPaulFairFoodHall", (5, 12), (3, 6)),
    (ST_PAUL, "MnStPaulFairGardenStand", (17, 14), (3, 6)),
    (ST_PAUL, "MnStPaulFairBerrySnack", (11, 14), (3, 6)),
    (ST_PAUL, "MnStPaulFairFestivalMail", (21, 20), (3, 6)),
    (ST_PAUL, "MnStPaulWestSeventhBakery", (5, 40), (3, 6)),
    (ST_PAUL, "MnStPaulLowertownExchange", (32, 40), (7, 6)),
    (ST_PAUL, "MnStPaulLowertownCrafts", (23, 42), (3, 6)),
    (ST_PAUL, "MnStPaulLowertownCafe", (55, 40), (3, 6)),
    (ST_PAUL, "MnStPaulRiversideSupplies", (7, 60), (3, 6)),
];

#[test]
#[ignore = "requires the Lakeside Stops edition in CRYSTAL_INTERIOR_PACK"]
fn storefronts_arrive_on_open_floor_and_return_via_both_carpets() {
    let (root, runtime, _) = load();
    for &(city, shop, doorstep, landing) in VISITS {
        let room = &runtime.data().maps[shop];
        let mut visitor = fixture(&root, &runtime, city, doorstep);
        assert_eq!(collision(&runtime, room, landing), Some("FLOOR"), "{shop}");
        let [left, right, ..] = room.events.warps.as_slice() else {
            panic!("missing exit pair in {shop}");
        };
        assert_eq!((left.x + 1, left.y), (right.x, right.y));
        for exit in [left, right] {
            let x = i16::try_from(exit.x).expect("carpet column");
            let y = i16::try_from(exit.y).expect("carpet row");
            assert_eq!(y, landing.1 + 1);
            assert_eq!(collision(&runtime, room, (x, y)), Some("WARP_CARPET_DOWN"));
            travel(&mut visitor, Up, shop, landing);
            walk(&mut visitor, &runtime, (x, y - 1));
            travel(&mut visitor, Down, city, doorstep);
        }
    }
}

#[test]
#[ignore = "requires the Lakeside Stops edition in CRYSTAL_INTERIOR_PACK"]
fn all_five_departments_serve_their_stock_across_a_reachable_counter() {
    let (root, runtime, _) = load();
    for (floor, approach, counter, facing, item) in [
        (1, (10, 3), (10, 2), Up, "POKE_BALL"),
        (2, (11, 5), (12, 5), Right, "POTION"),
        (3, (6, 3), (6, 2), Up, "X_ATTACK"),
        (4, (11, 5), (12, 5), Right, "PROTEIN"),
        (5, (8, 3), (8, 4), Down, "TM_HIDDEN_POWER"),
    ] {
        let map = format!("MnMinneapolisDepartmentStore{floor}F");
        let mart = format!("MART_MN_MP_DEPT_{floor}_CLERK");
        let room = &runtime.data().maps[&map];
        let mut shopper = fixture(&root, &runtime, &map, (2, 1));
        if floor == 5 {
            for x in 6..=9 {
                assert_eq!(collision(&runtime, room, (x, 7)), Some("FLOOR"));
                walk(&mut shopper, &runtime, (x, 7));
            }
        }
        walk(&mut shopper, &runtime, approach);
        assert_eq!(collision(&runtime, room, counter), Some("COUNTER"));
        press(&mut shopper, facing);
        assert_eq!(at(&mut shopper), approach);
        for _ in 0..32 {
            press(&mut shopper, A);
            if shopper.shell.shop_top_cursor.is_some()
                && shopper.shell.shop_welcome_seen
                && shopper.shell.shop_notice.is_none()
            {
                break;
            }
        }
        let shop = shopper
            .snapshot()
            .expect("department menu")
            .pending_shop
            .expect("open counter");
        assert_eq!(shop.mart_id, mart);
        assert_eq!(shop.inventory, runtime.data().marts.0[&mart]);
        assert_eq!(shop.inventory.first().map(String::as_str), Some(item));
        press(&mut shopper, A);
        assert!(
            shopper.shell.menu_cursor.is_some(),
            "browse the actual stock on floor {floor}"
        );
        if floor == 1 {
            let ball = &runtime.data().items[item];
            let before = shopper.shell.shell.session().state().clone();
            press(&mut shopper, A);
            assert_eq!(
                shopper
                    .shell
                    .shop_quantity
                    .as_ref()
                    .expect("buy one ball")
                    .item_id,
                item
            );
            press(&mut shopper, A);
            press(&mut shopper, A);
            let after = shopper.shell.shell.session().state();
            assert_eq!(after.money, before.money - u32::from(ball.price));
            assert_eq!(after.bag.quantity(ball), before.bag.quantity(ball) + 1);
        }
        for _ in 0..48 {
            let snapshot = shopper.snapshot().expect("checkout");
            if snapshot.pending_shop.is_none()
                && !snapshot.ui.text_window_open
                && !shopper.has_pending_script_work()
            {
                break;
            }
            let button = if snapshot.pending_shop.is_none() || shopper.shell.shop_notice.is_some() {
                A
            } else if let Some(cursor) = &shopper.shell.shop_top_cursor {
                if cursor.option_index == 2 { A } else { Down }
            } else {
                B
            };
            press(&mut shopper, button);
        }
        assert!(
            shopper
                .snapshot()
                .expect("closed shop")
                .pending_shop
                .is_none()
        );
        walk(&mut shopper, &runtime, (2, 1));
    }
}
