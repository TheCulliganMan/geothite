//! Current Badge and R04 reached by the party earned in the Winona/ICE course.
use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::super::authored_route_fixture::indoor_walk;
use super::super::native_city_ports::east_road_port;
use super::super::northstar_earned_gyms;
use super::super::rochester_earned::try_battle;
use super::super::winona::{at as location, continue_in_place, load, travel};
use super::gym_four_approach::try_crossing;
use super::northstar_campaign_ice::{care, ice, supplies, train};
use super::{VisibleShellController, event, interact, point, touch_switch, walk_to};

fn arrival() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let Some(path) = std::env::var_os("CRYSTAL_NORTHSTAR_CURRENT_SAVE") else {
        return ice();
    };
    let (root, r) = load();
    let mut c =
        VisibleShellController::load_save(root.clone(), r.clone(), PathBuf::from(path), None)
            .expect("actual earned dispatcher-home checkpoint with matching pack identity");
    c.set_runtime_journal_enabled(false);
    location(&mut c, "MnMinneapolisDispatcherHouse", (6, 5));
    assert_eq!(
        c.shell.shell.session().state().badges.johto,
        [true, true, false, false, false, false, false, false]
    );
    for flag in [
        "EVENT_MN_DELIVERED_ROCHESTER_PARCEL",
        "EVENT_MN_GYM_1_DEFEATED",
        "EVENT_MN_GYM_2_DEFEATED",
        "EVENT_MN_ICE_ENFORCER_DEFEATED",
        "EVENT_MN_ICE_CAPTIVE_RESCUED",
    ] {
        assert!(event(&c, flag), "earned prerequisite {flag}");
    }
    if !event(&c, "EVENT_MN_DISPATCH_NOTE") {
        assert!(super::super::northstar_dispatch_course::has_physical_copy(
            &r
        ));
        assert!(event(&c, "EVENT_MN_ICE_HOLDING_POWER_OFF"));
        super::super::northstar_dispatch_course::read_copy_and_return_home(&mut c, &root, &r);
        indoor_walk(&mut c, &r, (6, 5));
    }
    assert!(event(&c, "EVENT_MN_DISPATCH_NOTE"));
    assert!(!event(&c, "EVENT_MN_COURIER_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_DEEP_BORE_MANIFEST_READ"));
    (root, r, c)
}

fn design(r: &CrystalRuntime) -> Value {
    if northstar_earned_gyms::selected(r) {
        return northstar_earned_gyms::design(r, 3).clone();
    }
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../planning/minnesota/gyms/interiors.json");
    let gyms: Value = serde_json::from_str(&std::fs::read_to_string(path).expect("gym contract"))
        .expect("authored gym design");
    gyms["gyms"][2].clone()
}

#[test]
#[ignore = "earned campaign; optional CRYSTAL_NORTHSTAR_CURRENT_SAVE is the actual dispatcher-home save"]
fn northstar_campaign_earned_current_badge_and_walks_to_st_paul() {
    super::northstar_campaign_review::require_unboosted();
    let _ = current();
}

pub(super) fn current() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    if let Some(path) = std::env::var_os("CRYSTAL_NORTHSTAR_CURRENT_EXIT_SAVE") {
        let (root, r) = load();
        let mut c =
            VisibleShellController::load_save(root.clone(), r.clone(), PathBuf::from(path), None)
                .expect("genuine same-pack Current Badge victory checkpoint");
        c.set_runtime_journal_enabled(false);
        location(&mut c, "MnMinneapolis", (64, 34));
        for flag in [
            "EVENT_MN_GYM_1_DEFEATED",
            "EVENT_MN_GYM_2_DEFEATED",
            "EVENT_MN_GYM_3_DEFEATED",
            "EVENT_MN_ICE_CAPTIVE_RESCUED",
            "EVENT_MN_DISPATCH_NOTE",
        ] {
            assert!(event(&c, flag), "earned Current exit prerequisite {flag}");
        }
        let gym = design(&r);
        return depart(root, r, c, gym);
    }
    let (root, r, mut c) = arrival();
    let gym = design(&r);
    indoor_walk(&mut c, &r, (5, 6));
    travel(&mut c, GameButton::Down, "MnMinneapolis", (23, 42));
    for (tile, id) in [
        ((1, 13), "MN_GYM_3_TRAINER_1"),
        ((8, 10), "MN_GYM_3_TRAINER_2"),
        ((4, 3), "MN_GYM_3"),
    ] {
        let flag = format!("EVENT_{id}_DEFEATED");
        for attempt in 0..4 {
            care(&mut c, &r);
            supplies(&mut c, &r);
            indoor_walk(&mut c, &r, (64, 34));
            travel(&mut c, GameButton::Up, "MnMinneapolisGym", (4, 15));
            let selected = northstar_earned_gyms::selected(&r);
            if selected && id != "MN_GYM_3_TRAINER_1" {
                if !event(&c, "EVENT_MN_GYM_3_FIRST_SWITCH_ON") {
                    touch_switch(&mut c, &r, &gym, "UPPER_LEFT");
                }
                assert!(event(&c, "EVENT_MN_GYM_3_FIRST_SWITCH_ON"));
                if id == "MN_GYM_3" && !event(&c, "EVENT_MN_GYM_3_SECOND_SWITCH_ON") {
                    touch_switch(&mut c, &r, &gym, "UPPER_MIDDLE");
                }
            } else if !selected && id == "MN_GYM_3" {
                for switch in ["UPPER_LEFT", "UPPER_MIDDLE"] {
                    touch_switch(&mut c, &r, &gym, switch);
                }
            }
            if id == "MN_GYM_3" {
                assert!(event(&c, "EVENT_MN_GYM_3_COURT_OPEN"));
            }
            let tile = if selected {
                northstar_earned_gyms::opponent(&r, 3, id)
            } else {
                tile
            };
            interact(&mut c, &r, tile);
            if try_battle(&mut c, &r, id, &flag) {
                // Native move-learning choices remain in force. Victory and
                // the saved trainer flag below establish the earned battle.
                walk_to(&mut c, &r, (4, 16));
                travel(&mut c, GameButton::Down, "MnMinneapolis", (64, 34));
                continue_in_place(&mut c, &root, &r, &format!("campaign-{id}-win"));
                break;
            }
            location(&mut c, "MnMinneapolisPokecenter1F", (10, 13));
            assert!(!c.shell.shell.session().state().badges.johto[2]);
            continue_in_place(&mut c, &root, &r, &format!("campaign-{id}-loss-{attempt}"));
            if attempt < 3 {
                train(&mut c, &r, &root, 22 + attempt);
            }
        }
        assert!(event(&c, &flag));
    }
    depart(root, r, c, gym)
}

pub(super) fn st_paul_arrival(r: &CrystalRuntime) -> (i16, i16) {
    let offset = r.data().maps["MnR04C00R00"]
        .attributes
        .connections
        .iter()
        .find(|link| link.direction == "east" && link.target_map == "MnStPaul")
        .expect("native St Paul road connection")
        .offset;
    (
        1,
        i16::try_from(18 - offset * 2).expect("native St Paul arrival coordinate"),
    )
}

fn depart(
    root: AssetRoot,
    r: CrystalRuntime,
    mut c: VisibleShellController,
    gym: Value,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    assert_eq!(
        c.shell.shell.session().state().badges.johto,
        [true, true, true, false, false, false, false, false]
    );
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["TM_THUNDERPUNCH"]),
        1
    );
    for attempt in 0..4 {
        care(&mut c, &r);
        let (approach, arrival) = east_road_port(&mut c, &r, "MnR04C00R00");
        indoor_walk(&mut c, &r, approach);
        travel(&mut c, GameButton::Right, "MnR04C00R00", arrival);
        if try_crossing(&mut c, &r, (55, 18)) {
            let arrival = st_paul_arrival(&r);
            travel(&mut c, GameButton::Right, "MnStPaul", (0, arrival.1));
            travel(&mut c, GameButton::Right, "MnStPaul", arrival);
            break;
        }
        location(&mut c, "MnMinneapolisPokecenter1F", (10, 13));
        continue_in_place(&mut c, &root, &r, &format!("campaign-r04-loss-{attempt}"));
    }
    location(&mut c, "MnStPaul", st_paul_arrival(&r));
    assert!(!event(&c, "EVENT_MN_COURIER_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_DEEP_BORE_MANIFEST_READ"));
    assert!(!event(&c, "EVENT_MN_GYM_4_DEFEATED"));
    continue_in_place(&mut c, &root, &r, "campaign-earned-st-paul-arrival");
    // Verify the actual contract used here still names the frozen room.
    assert_eq!(point(&gym["arrival_core"]), (4, 15));
    (root, r, c)
}
