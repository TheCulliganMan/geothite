//! Earn route-house rods through native travel, conversation and Continue.
use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind};

use super::super::VisibleShellController;
use super::interior_life::{talk, walk};
use super::rochester::{flee, trail_with};
use super::rochester_earned::try_battle;
use super::winona::{continue_in_place, travel};

pub(super) fn encounter(c: &mut VisibleShellController, r: &CrystalRuntime) {
    match c
        .snapshot()
        .expect("actual rod approach battle")
        .battle
        .expect("battle")
        .kind
    {
        RuntimeBattleKind::Wild { .. } => flee(c),
        RuntimeBattleKind::Trainer {
            trainer_id,
            event_flag,
            ..
        } => {
            assert!(
                try_battle(c, r, &trainer_id, &event_flag),
                "actual rod detour trainer loss"
            );
        }
        _ => panic!("unexpected rod approach battle"),
    }
}

pub(super) fn claim(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    root: &AssetRoot,
    room: &str,
    outside: &str,
    item: &str,
) {
    travel(c, GameButton::Up, room, (2, 6));
    walk(c, r, (5, 4));
    let money = c.shell.shell.session().state().money;
    talk(c, (5, 3));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items[item]),
        1
    );
    continue_in_place(c, root, r, &format!("campaign-earned-{item}"));
    talk(c, (5, 3));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items[item]),
        1,
        "once-only rod after Continue"
    );
    assert_eq!(
        c.shell.shell.session().state().money,
        money,
        "free fishing lesson"
    );
    walk(c, r, (2, 7));
    assert_eq!(
        c.snapshot().expect("actual guru return").overworld.map_name,
        outside
    );
}

pub(super) fn old_rod(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    walk(c, r, (38, 18));
    travel(c, GameButton::Right, "MnR01C00R00", (0, 24));
    trail_with(c, r, (25, 34), |c| encounter(c, r));
    claim(
        c,
        r,
        root,
        "MnDriftlessFishingGuruHouse",
        "MnR01C00R00",
        "OLD_ROD",
    );
    super::earned_route_machines::flash(c, r, root);
    trail_with(c, r, (0, 24), |c| encounter(c, r));
    travel(c, GameButton::Left, "MnRochester", (38, 18));
}

#[test]
#[ignore = "requires current Minnesota pack; declared room access, actual doors/gifts/Continue only"]
fn earned_rods_native_claim_helper_all_three_houses_and_continue() {
    let (root, r) = super::winona::load();
    for (room, outside, item) in [
        ("MnDriftlessFishingGuruHouse", "MnR01C00R00", "OLD_ROD"),
        ("MnRiverValleyFishingGuruHouse", "MnR03C00R00", "GOOD_ROD"),
        ("MnPineCountryFishingGuruHouse", "MnR06C00R00", "SUPER_ROD"),
    ] {
        let mut c = super::interior_life::fixture(&root, &r, room, (2, 6));
        walk(&mut c, &r, (2, 7));
        assert_eq!(
            c.snapshot()
                .expect("original guru mouth")
                .overworld
                .map_name,
            outside
        );
        claim(&mut c, &r, &root, room, outside, item);
    }
}
