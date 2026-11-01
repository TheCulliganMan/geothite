//! Earn the courier evidence and Capitol with the actual Winona campaign party.
use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::authored_route_fixture::indoor_walk;
use super::super::courier_story::acknowledge_manifest;
use super::super::native_city_ports::city_port;
use super::super::northstar_earned_gyms;
use super::super::rochester_earned::{checkout, try_battle};
use super::super::winona::{at as location, continue_in_place, finish, load, press, travel};
use super::gym_four_approach::try_crossing;
use super::northstar_campaign_current::{current, st_paul_arrival};
use super::{VisibleShellController, event, interact, walk_to};

const CITY: &str = "MnStPaul";
const CLINIC: &str = "MnStPaulPokecenter1F";
const DEPOT: &str = "MnStPaulUnionDepot";
const COURIER: &str = "EVENT_MN_COURIER_DEFEATED";
const MANIFEST: &str = "EVENT_MN_DEEP_BORE_MANIFEST_READ";

fn arrival() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let Some(path) = std::env::var_os("CRYSTAL_NORTHSTAR_COURIER_SAVE") else {
        return current();
    };
    let (root, r) = load();
    let mut c =
        VisibleShellController::load_save(root.clone(), r.clone(), PathBuf::from(path), None)
            .expect("actual earned St Paul arrival with matching pack identity");
    c.set_runtime_journal_enabled(false);
    location(&mut c, CITY, st_paul_arrival(&r));
    assert_eq!(
        c.shell.shell.session().state().badges.johto,
        [true, true, true, false, false, false, false, false]
    );
    for flag in [
        "EVENT_MN_DELIVERED_ROCHESTER_PARCEL",
        "EVENT_MN_GYM_1_DEFEATED",
        "EVENT_MN_GYM_2_DEFEATED",
        "EVENT_MN_GYM_3_DEFEATED",
        "EVENT_MN_ICE_CAPTIVE_RESCUED",
        "EVENT_MN_DISPATCH_NOTE",
    ] {
        assert!(event(&c, flag), "actual earned prerequisite {flag}");
    }
    assert!(!event(&c, COURIER) && !event(&c, MANIFEST));
    (root, r, c)
}

fn care(c: &mut VisibleShellController, r: &CrystalRuntime) {
    if c.snapshot().expect("care approach").overworld.map_name == CITY {
        indoor_walk(c, r, city_port(r, CITY, CLINIC).0);
        travel(c, GameButton::Up, CLINIC, (3, 5));
    }
    assert_eq!(
        c.snapshot().expect("native clinic").overworld.map_name,
        CLINIC
    );
    indoor_walk(c, r, (3, 3));
    let money = c.shell.shell.session().state().money;
    press(c, GameButton::Up);
    press(c, GameButton::A);
    finish(c);
    assert!(
        c.snapshot()
            .expect("free native care")
            .party
            .slots
            .iter()
            .all(|s| {
                s.pokemon.hp == s.pokemon.max_hp
                    && s.pokemon.status.is_none()
                    && s.pokemon
                        .moves
                        .iter()
                        .all(|m| m.current_pp == r.data().moves[&m.name].pp)
            })
    );
    assert_eq!(c.shell.shell.session().state().money, money);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .last_spawn_map_constant
            .as_deref(),
        Some("MN_ST_PAUL_POKECENTER_1F")
    );
    indoor_walk(c, r, (3, 6));
    travel(c, GameButton::Down, CITY, city_port(r, CITY, CLINIC).0);
}

fn supplies(c: &mut VisibleShellController, r: &CrystalRuntime) {
    let item = &r.data().items["SUPER_POTION"];
    let state = c.shell.shell.session().state();
    let quantity = (state.money / u32::from(item.price))
        .min(u32::from(3_u16.saturating_sub(state.bag.quantity(item)))) as u16;
    if quantity == 0 {
        return;
    }
    indoor_walk(c, r, city_port(r, CITY, "MnStPaulMart").0);
    travel(c, GameButton::Up, "MnStPaulMart", (3, 5));
    indoor_walk(c, r, (3, 3));
    press(c, GameButton::Left);
    press(c, GameButton::A);
    checkout(c, r, "SUPER_POTION", quantity);
    indoor_walk(c, r, (3, 6));
    travel(
        c,
        GameButton::Down,
        CITY,
        city_port(r, CITY, "MnStPaulMart").0,
    );
}

#[test]
#[ignore = "earned campaign; optional CRYSTAL_NORTHSTAR_COURIER_SAVE is the actual St Paul arrival"]
fn northstar_campaign_earned_courier_manifest_capitol_and_return() {
    super::northstar_campaign_review::require_unboosted();
    let _ = capitol();
}

pub(super) fn capitol() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r, mut c) = arrival();
    let staged = r.data().maps[DEPOT]
        .scripts
        .contains_key("MnOverhaulDepotEntry");
    for attempt in 0..4 {
        care(&mut c, &r);
        supplies(&mut c, &r);
        indoor_walk(&mut c, &r, city_port(&r, CITY, DEPOT).0);
        let money = c.shell.shell.session().state().money;
        if staged {
            for _ in 0..12 {
                if c.snapshot().expect("depot arrival").overworld.map_name == DEPOT {
                    break;
                }
                press(&mut c, GameButton::Up);
            }
            c.wait_frames(512).expect("courier approach animation");
            location(&mut c, DEPOT, (19, 16));
            assert!(
                c.snapshot()
                    .expect("automatic interception")
                    .ui
                    .text_window_open
            );
            press(&mut c, GameButton::Down);
            location(&mut c, DEPOT, (19, 16));
        } else {
            travel(&mut c, GameButton::Up, DEPOT, (15, 21));
            indoor_walk(&mut c, &r, (19, 16));
        }
        if attempt == 0 && !staged {
            let before = c.snapshot().expect("guarded manifest");
            press(&mut c, GameButton::Left);
            press(&mut c, GameButton::A);
            assert!(c.snapshot().expect("visible refusal").ui.text.is_some());
            press(&mut c, GameButton::Down);
            location(&mut c, DEPOT, (19, 16));
            finish(&mut c);
            assert!(!event(&c, COURIER) && !event(&c, MANIFEST));
            let after = c.snapshot().expect("no free evidence");
            assert_eq!(before.party, after.party);
            assert_eq!(before.bag, after.bag);
        }
        if !staged {
            press(&mut c, GameButton::Right);
            press(&mut c, GameButton::A);
        }
        if try_battle(&mut c, &r, "MN_LODE_COURIER", COURIER) {
            assert!(c.shell.shell.session().state().money > money);
            continue_in_place(&mut c, &root, &r, "campaign-earned-courier-win");
            break;
        }
        location(&mut c, CLINIC, (3, 5));
        assert!(!event(&c, COURIER) && !event(&c, MANIFEST));
        assert!(!c.shell.shell.session().state().badges.johto[3]);
        continue_in_place(
            &mut c,
            &root,
            &r,
            &format!("campaign-courier-loss-{attempt}"),
        );
    }
    assert!(event(&c, COURIER) && !event(&c, MANIFEST));
    if r.data().maps.contains_key("MnBoundaryIntake") {
        indoor_walk(&mut c, &r, (15, 21));
        travel(&mut c, GameButton::Down, CITY, city_port(&r, CITY, DEPOT).0);
        for attempt in 0..4 {
            care(&mut c, &r);
            supplies(&mut c, &r);
            indoor_walk(&mut c, &r, city_port(&r, CITY, DEPOT).0);
            travel(&mut c, GameButton::Up, DEPOT, (15, 21));
            indoor_walk(&mut c, &r, (19, 16));
            super::super::northstar_ice_lair::talk(&mut c, GameButton::Left);
            super::super::northstar_ice_lair::choice(&mut c, true);
            if try_battle(
                &mut c,
                &r,
                "MN_REED_STPAUL",
                "EVENT_MN_REED_STPAUL_DEFEATED",
            ) {
                break;
            }
            assert!(!event(&c, MANIFEST));
            continue_in_place(
                &mut c,
                &root,
                &r,
                &format!("campaign-reed-depot-loss-{attempt}"),
            );
        }
        assert!(event(&c, "EVENT_MN_REED_STPAUL_DEFEATED"));
        finish(&mut c);
        super::super::northstar_ice_lair::talk(&mut c, GameButton::Left);
        super::super::northstar_ice_lair::choice(&mut c, true);
        finish(&mut c);
    } else {
        c = acknowledge_manifest(&root, &r, c);
    }
    assert!(event(&c, MANIFEST));
    continue_in_place(&mut c, &root, &r, "campaign-earned-manifest-read");
    indoor_walk(&mut c, &r, (15, 21));
    travel(&mut c, GameButton::Down, CITY, city_port(&r, CITY, DEPOT).0);
    for (tile, id) in [
        ((3, 3), "MN_GYM_4_TRAINER_1"),
        ((11, 12), "MN_GYM_4_TRAINER_2"),
        ((11, 2), "MN_GYM_4"),
    ] {
        let flag = format!("EVENT_{id}_DEFEATED");
        for attempt in 0..4 {
            care(&mut c, &r);
            supplies(&mut c, &r);
            indoor_walk(&mut c, &r, city_port(&r, CITY, "MnStPaulGym").0);
            travel(&mut c, GameButton::Up, "MnStPaulGym", (2, 17));
            let tile = if northstar_earned_gyms::selected(&r) {
                northstar_earned_gyms::opponent(&r, 4, id)
            } else {
                tile
            };
            interact(&mut c, &r, tile);
            if try_battle(&mut c, &r, id, &flag) {
                if northstar_earned_gyms::selected(&r) {
                    northstar_earned_gyms::leave(
                        &mut c,
                        &r,
                        4,
                        CITY,
                        city_port(&r, CITY, "MnStPaulGym").0,
                    );
                } else {
                    walk_to(&mut c, &r, (2, 18));
                    travel(
                        &mut c,
                        GameButton::Down,
                        CITY,
                        city_port(&r, CITY, "MnStPaulGym").0,
                    );
                }
                continue_in_place(&mut c, &root, &r, &format!("campaign-{id}-win"));
                break;
            }
            location(&mut c, CLINIC, (3, 5));
            assert!(!c.shell.shell.session().state().badges.johto[3]);
            assert!(event(&c, COURIER) && event(&c, MANIFEST));
            continue_in_place(&mut c, &root, &r, &format!("campaign-{id}-loss-{attempt}"));
        }
        assert!(event(&c, &flag));
    }
    assert_eq!(
        c.shell.shell.session().state().badges.johto,
        [true, true, true, true, false, false, false, false]
    );
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["TM_STEEL_WING"]),
        1
    );
    super::northstar_earned_services::st_paul(&mut c, &r, &root);
    for attempt in 0..4 {
        care(&mut c, &r);
        indoor_walk(&mut c, &r, st_paul_arrival(&r));
        travel(&mut c, GameButton::Left, "MnR04C00R00", (55, 18));
        if try_crossing(&mut c, &r, (0, 18)) {
            travel(&mut c, GameButton::Left, "MnMinneapolis", (79, 36));
            travel(&mut c, GameButton::Left, "MnMinneapolis", (76, 36));
            break;
        }
        location(&mut c, CLINIC, (3, 5));
        continue_in_place(
            &mut c,
            &root,
            &r,
            &format!("campaign-manifest-return-loss-{attempt}"),
        );
    }
    location(&mut c, "MnMinneapolis", (76, 36));
    assert!(event(&c, COURIER) && event(&c, MANIFEST));
    for flag in [
        "EVENT_MN_DEPOT_CLEARED",
        "EVENT_MN_LIGHTHOUSE_SIGNAL",
        "EVENT_MN_SURVEY_RESTORED",
    ] {
        assert!(!event(&c, flag), "later story {flag} must stay unearned");
    }
    super::northstar_campaign_ice::care(&mut c, &r);
    continue_in_place(
        &mut c,
        &root,
        &r,
        "campaign-earned-capitol-manifest-minneapolis-return",
    );
    (root, r, c)
}
