//! Enter a cave from its road, then use the Pack's Escape Rope after Continue.
use crystal_core::input::GameButton;

use super::super::{
    FieldPackAction, FieldPackPocket, VisibleShellController, selected_field_pack_item_id,
    visible_selected_pack_item_actions, visible_start_menu_entries,
};
use super::badge_trails::load;
use super::interior_life::{fixture, walk};
use super::r10_gallery::{continue_save, map, press, tile};
use super::winona::travel;

fn use_rope(c: &mut VisibleShellController) {
    press(c, GameButton::Start);
    for _ in 0..9 {
        if visible_start_menu_entries(&c.shell)
            .expect("Start entries")
            .iter()
            .any(|s| s == ">PACK")
        {
            break;
        }
        press(c, GameButton::Down);
    }
    press(c, GameButton::A);
    assert_eq!(c.shell.field_pack_pocket, Some(FieldPackPocket::Items));
    press(c, GameButton::A);
    assert_eq!(
        selected_field_pack_item_id(&mut c.shell).expect("selected item"),
        "ESCAPE_ROPE"
    );
    let snapshot = c.snapshot().expect("rope actions");
    let choices =
        visible_selected_pack_item_actions(&snapshot, &c.shell, &FieldPackPocket::Items, false)
            .expect("visible rope actions");
    assert_eq!(
        choices[c
            .shell
            .field_pack_action_cursor
            .as_ref()
            .expect("action cursor")
            .option_index],
        FieldPackAction::Use
    );
    press(c, GameButton::A);
}

#[test]
#[ignore = "requires a Minnesota pack in MN_BADGE_TRAILS_PACK"]
fn cave_escape_trip_remembers_the_actual_mouth_through_continue() {
    let (root, runtime) = load();
    // Enter all fourteen outdoor cave mouths, including both ends of each
    // through cave and the postgame basin. The first Driftless trip also
    // descends to the fossil gallery before saving and unpacking the rope.
    for (road, bank, cave, inside, gallery) in [
        ("MnR01C00R00", (18, 18), "MnR01Cave1F", (9, 11), true),
        ("MnR01C00R00", (36, 20), "MnR01Cave1F", (31, 13), false),
        ("MnR05C00R00", (30, 28), "MnR05RockCut", (11, 5), false),
        ("MnR05C00R01", (16, 30), "MnR05RockCut", (5, 17), false),
        ("MnR20C00R00", (62, 32), "MnR20Cave1F", (5, 21), false),
        ("MnR20C01R00", (12, 32), "MnR20Cave1F", (23, 19), false),
        ("MnR03C00R00", (8, 38), "MnR03Cave1F", (7, 9), false),
        ("MnR09C00R00", (40, 42), "MnR09Cave1F", (13, 19), false),
        ("MnR10C00R00", (36, 18), "MnR10Cave1F", (3, 13), false),
        ("MnR12C00R00", (40, 16), "MnR12Cave1F", (17, 23), false),
        ("MnR13C00R01", (14, 6), "MnR13Cave1F", (7, 15), false),
        ("MnR17C00R00", (10, 44), "MnR17Cave1F", (9, 13), false),
        ("MnR20C01R00", (6, 34), "MnVoyageursIce1F", (5, 19), false),
        ("MnFormerShores", (18, 24), "MnInnerBasin", (2, 25), false),
    ] {
        let mut c = fixture(&root, &runtime, road, bank);
        let rope = &runtime.data().items["ESCAPE_ROPE"];
        let state = c.shell.shell.session_mut().state_mut();
        state
            .flags
            .set_engine_flag("STATUSFLAGS_NO_WILD_ENCOUNTERS_F", true)
            .expect("isolate the return trip from encounters");
        assert!(state.bag.add_item(rope, 2).expect("two ropes"));
        travel(&mut c, GameButton::Up, cave, inside);
        if gallery {
            walk(&mut c, &runtime, (10, 9));
            travel(
                &mut c,
                GameButton::Right,
                "MnDriftlessLampGallery",
                (18, 17),
            );
        }
        let mut c = continue_save(root.clone(), runtime.clone(), &mut c);
        let state = c.shell.shell.session().state();
        let mouth = (bank.0, bank.1 - 1);
        let door = runtime.data().maps[road]
            .events
            .warps
            .iter()
            .find(|w| (w.x as i16, w.y as i16) == mouth)
            .expect("actual exterior cave mouth");
        assert_eq!(state.dig_warp_map_name.as_deref(), Some(road));
        assert_eq!(state.dig_warp_index, Some(door.index));
        use_rope(&mut c);
        for _ in 0..48 {
            if map(&mut c) == road {
                break;
            }
            press(&mut c, GameButton::A);
        }
        assert_eq!(map(&mut c), road, "rope from {cave} via {mouth:?}");
        assert_eq!(
            tile(&mut c),
            bank,
            "step out onto the bank below the original mouth"
        );
        assert_eq!(c.shell.shell.session().state().bag.quantity(rope), 1);
        travel(&mut c, GameButton::Down, road, (bank.0, bank.1 + 1));
        travel(&mut c, GameButton::Up, road, bank);
        travel(&mut c, GameButton::Up, cave, inside);
    }
}

#[test]
#[ignore = "requires the Duluth dock stock in MN_BADGE_TRAILS_PACK"]
fn cave_escape_trip_buys_supplies_across_the_duluth_counter() {
    let (root, runtime) = load();
    let mut c = fixture(&root, &runtime, "MnDuluth", (12, 78));
    c.shell.shell.session_mut().state_mut().money = 850;
    travel(&mut c, GameButton::Up, "MnDuluthDockSupplies", (6, 9));
    walk(&mut c, &runtime, (5, 5));
    for id in ["ESCAPE_ROPE", "POTION"] {
        press(&mut c, GameButton::Left);
        press(&mut c, GameButton::A);
        super::rochester_earned::checkout(&mut c, &runtime, id, 1);
    }
    let mut c = continue_save(root, runtime.clone(), &mut c);
    let state = c.shell.shell.session().state();
    assert_eq!(state.money, 0);
    for id in ["ESCAPE_ROPE", "POTION"] {
        assert_eq!(state.bag.quantity(&runtime.data().items[id]), 1);
    }
    walk(&mut c, &runtime, (6, 9));
    travel(&mut c, GameButton::Down, "MnDuluth", (12, 78));
}
