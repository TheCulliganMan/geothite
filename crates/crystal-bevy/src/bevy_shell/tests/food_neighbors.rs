//! The Hillside cook sets a single warm roll aside for a traveler.
use crystal_core::input::GameButton::{B, Down, Up};
use crystal_core::models::bag::ITEM_POCKET_CAPACITY;
use crystal_core::models::item::ITEM_POCKET_ITEM;

use super::super::minnesota_fossils::talk as answer_cook;
use super::super::northstar_progression_rewards::native_toss_first_pocket_item;
use super::super::winona::{at, continue_in_place, press, travel};
use super::{VisibleShellController, fixture, load, mark_runtime_snapshot_dirty, talk, walk};
use crate::bevy_shell::FieldPackPocket;

const HOSTEL: &str = "MnDuluthHillsideHostel1F";
const ROLL: &str = "MN_BUTTER_ROLL";
const RECEIPT: &str = "EVENT_MN_DULUTH_HOSTEL_BUTTER_ROLL";

fn gift(c: &VisibleShellController) -> (bool, u16) {
    let state = c.shell.shell.session().state();
    (
        state
            .flags
            .is_event_flag_set(RECEIPT)
            .expect("cook's receipt"),
        state
            .bag
            .quantity(&c.shell.shell.runtime().data().items[ROLL]),
    )
}

#[test]
#[ignore = "requires the Fair Table edition in CRYSTAL_INTERIOR_PACK"]
fn hillside_hostel_butter_roll_full_pack_retry_and_continue() {
    let (root, runtime, _) = load();
    let mut c = fixture(&root, &runtime, "MnDuluth", (72, 10));
    let state = c.shell.shell.session_mut().state_mut();
    assert!(
        state
            .bag
            .add_item(&runtime.data().items["POTION"], 1)
            .expect("first tossable slot")
    );
    for (_, item) in runtime
        .data()
        .items
        .iter()
        .filter(|(id, item)| {
            item.pocket == ITEM_POCKET_ITEM && !matches!(id.as_str(), "POTION" | ROLL)
        })
        .take(ITEM_POCKET_CAPACITY - 1)
    {
        assert!(state.bag.add_item(item, 1).expect("full pack"));
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    travel(&mut c, Up, HOSTEL, (8, 10));
    walk(&mut c, &runtime, (10, 6));
    let full = c.snapshot().expect("filled pockets").bag;
    assert_eq!(full.items.len(), ITEM_POCKET_CAPACITY);
    answer_cook(&mut c, Up, &[false]);
    assert_eq!(gift(&c), (false, 0));
    answer_cook(&mut c, Up, &[true]);
    assert_eq!(gift(&c), (false, 0));
    assert_eq!(c.snapshot().expect("cook kept the portion").bag, full);
    continue_in_place(&mut c, &root, &runtime, "hillside-roll-set-aside");
    native_toss_first_pocket_item(&mut c, &FieldPackPocket::Items);
    for _ in 0..8 {
        press(&mut c, B);
    }
    assert_eq!(
        c.snapshot().expect("one free slot").bag.items.len(),
        ITEM_POCKET_CAPACITY - 1
    );
    answer_cook(&mut c, Up, &[true]);
    assert_eq!(gift(&c), (true, 1));
    continue_in_place(&mut c, &root, &runtime, "hillside-roll-received");
    let again = talk(&mut c, (10, 5));
    assert!(again.contains("MnHostelButterRollRepeatText"));
    assert_eq!(gift(&c), (true, 1));
    walk(&mut c, &runtime, (8, 10));
    travel(&mut c, Down, "MnDuluth", (72, 10));
    at(&mut c, "MnDuluth", (72, 10));
}

#[test]
#[ignore = "requires the Fair Table edition in CRYSTAL_INTERIOR_PACK"]
fn fair_table_neighbors_give_current_directions_and_pack_lunch_from_home() {
    let (root, runtime, _) = load();
    for (map, entry, spot, actor, label) in [
        (
            "MnBrainerdMart",
            (6, 6),
            (8, 5),
            (9, 5),
            "MnFairTableMartLunchText",
        ),
        (
            "MnMinneapolisBridgeHome",
            (3, 6),
            (2, 4),
            (2, 3),
            "MnFairTableBridgeWalkText",
        ),
    ] {
        let mut c = fixture(&root, &runtime, map, entry);
        walk(&mut c, &runtime, spot);
        assert!(talk(&mut c, actor).contains(label));
        walk(&mut c, &runtime, entry);
    }
}
