//! Position and pocket fixtures; doors, gifts, repeats and Continue use input.
use crystal_core::input::GameButton;

use super::interior_life::{fixture, load, talk, walk};
use super::r10_gallery::{continue_save, press};

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with route fishing guru houses"]
fn minnesota_fishing_rods_controller_houses_doors_gifts_repeats_and_continue() {
    let (root, runtime, _) = load();
    for (outside, landing, room, rod) in [
        (
            "MnR01C00R00",
            (25, 34),
            "MnDriftlessFishingGuruHouse",
            "OLD_ROD",
        ),
        (
            "MnR03C00R00",
            (31, 30),
            "MnRiverValleyFishingGuruHouse",
            "GOOD_ROD",
        ),
        (
            "MnR06C00R00",
            (37, 40),
            "MnPineCountryFishingGuruHouse",
            "SUPER_ROD",
        ),
    ] {
        // Begin on the house floor, then actually exit and reenter. A route
        // return landing may now be grass; it is not an interior-floor fixture.
        let mut c = fixture(&root, &runtime, room, (2, 6));
        walk(&mut c, &runtime, (2, 7));
        let returned = c.snapshot().expect("actual initial house exit");
        assert_eq!(returned.overworld.map_name, outside);
        assert_eq!(
            (returned.overworld.tile.x, returned.overworld.tile.y),
            landing
        );
        let money = c.shell.shell.session().state().money;
        walk(&mut c, &runtime, (landing.0, landing.1 - 1));
        let entered = c.snapshot().expect("real house entry");
        assert_eq!(entered.overworld.map_name, room);
        assert_eq!((entered.overworld.tile.x, entered.overworld.tile.y), (2, 6));
        walk(&mut c, &runtime, (5, 4));
        let pages = talk(&mut c, (5, 3));
        assert!(pages.contains("ROD"), "{room}: {pages}");
        for _ in 0..2 {
            assert_eq!(
                c.shell
                    .shell
                    .session()
                    .state()
                    .bag
                    .quantity(&runtime.data().items[rod]),
                1
            );
            talk(&mut c, (5, 3));
        }
        assert_eq!(c.shell.shell.session().state().money, money, "free rod");
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        for carpet in [2, 3] {
            walk(&mut c, &runtime, (carpet, 7));
            let returned = c.snapshot().expect("real carpet return");
            assert_eq!(returned.overworld.map_name, outside);
            assert_eq!(
                (returned.overworld.tile.x, returned.overworld.tile.y),
                landing
            );
            walk(&mut c, &runtime, (landing.0, landing.1 - 1));
            assert_eq!(c.snapshot().expect("repeat entry").overworld.map_name, room);
        }
    }
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with route fishing guru houses"]
fn minnesota_fishing_rods_controller_full_pocket_retry_and_independent_rods() {
    let (root, runtime, _) = load();
    let mut c = fixture(&root, &runtime, "MnDriftlessFishingGuruHouse", (5, 4));
    let bag = &mut c.shell.shell.session_mut().state_mut().bag;
    bag.key_items.clear();
    assert!(
        bag.add_item(&runtime.data().items["SUPER_ROD"], 1)
            .expect("existing stronger rod fixture")
    );
    while bag.key_items.len() < crystal_core::models::bag::KEY_ITEM_POCKET_CAPACITY {
        assert!(
            bag.add_item(&runtime.data().items["BICYCLE"], 1)
                .expect("full pocket fixture")
        );
    }
    super::super::mark_runtime_snapshot_dirty(&mut c.shell);
    assert!(talk(&mut c, (5, 3)).contains("Make space"));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["OLD_ROD"]),
        0
    );
    assert!(
        c.shell
            .shell
            .session_mut()
            .state_mut()
            .bag
            .remove_item(&runtime.data().items["BICYCLE"], 1)
            .expect("free fixture slot")
    );
    super::super::mark_runtime_snapshot_dirty(&mut c.shell);
    talk(&mut c, (5, 3));
    for rod in ["OLD_ROD", "SUPER_ROD"] {
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&runtime.data().items[rod]),
            1,
            "independent {rod}"
        );
    }
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with fishing houses and fisherman hints"]
fn minnesota_fishing_rods_controller_defeated_fisher_advertises_the_super_rod_house() {
    let (root, runtime, _) = load();
    let mut c = fixture(&root, &runtime, "MnR06C00R00", (43, 42));
    // A legal strong regional partner replaces grinding; the defeat flag below
    // must be earned through the actual battle and selected move's PP spend.
    let pokemon = runtime
        .data()
        .create_pokemon("VOYAGEAR", 60, crystal_core::models::Dv::default())
        .expect("regional battle partner");
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(pokemon));
    state.sync_party_from_storage();
    super::super::mark_runtime_snapshot_dirty(&mut c.shell);
    press(&mut c, GameButton::Down);
    press(&mut c, GameButton::A);
    assert!(super::rochester_earned::try_battle(
        &mut c,
        &runtime,
        "MN_R06_T03",
        "EVENT_MN_R06_TRAINER_03"
    ));
    let pages = talk(&mut c, (43, 43));
    assert!(
        pages.contains("GURU") && pages.contains("SUPER ROD") && pages.contains("PINE COUNTRY"),
        "{pages}"
    );
    c = continue_save(root, runtime.clone(), &mut c);
    assert!(talk(&mut c, (43, 43)).contains("GURU"));
}
