//! Earn Wind and the lake-country arrival with the continued Winona party.
use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::authored_route_fixture::indoor_walk;
use super::super::northstar_earned_gyms;
use super::super::rochester::flee;
use super::super::rochester_earned::try_battle;
use super::super::winona::{at as location, continue_in_place, finish, load, press, travel};
use super::lake_country_story::try_road_with;
use super::northstar_campaign_courier::capitol;
use super::{VisibleShellController, event, interact};

#[path = "northstar_campaign_wind_supplies.rs"]
mod supplies;
#[path = "northstar_campaign_wind_training.rs"]
mod training;

const CITY: &str = "MnStCloud";
const CLINIC: &str = "MnStCloudPokecenter1F";

fn arrival() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let Some(path) = std::env::var_os("CRYSTAL_NORTHSTAR_WIND_SAVE") else {
        return capitol();
    };
    let (root, r) = load();
    let mut c =
        VisibleShellController::load_save(root.clone(), r.clone(), PathBuf::from(path), None)
            .expect("actual earned Capitol/manifest return with matching pack identity");
    c.set_runtime_journal_enabled(false);
    let snapshot = c.snapshot().expect("actual campaign checkpoint");
    match snapshot.overworld.map_name.as_str() {
        "MnMinneapolis" => location(&mut c, "MnMinneapolis", (63, 26)),
        CITY => {
            let tile = snapshot.overworld.tile;
            if (tile.x, tile.y) == (8, 28) {
                assert!(event(&c, "EVENT_MN_GYM_5_TRAINER_1_DEFEATED"));
            } else {
                location(&mut c, CITY, (24, 34));
            }
        }
        CLINIC => location(&mut c, CLINIC, (10, 13)),
        _ => panic!("expected an actual four-badge return or earned R05 checkpoint"),
    }
    assert_eq!(
        c.shell.shell.session().state().badges.johto,
        [true, true, true, true, false, false, false, false]
    );
    for flag in [
        "EVENT_MN_DELIVERED_ROCHESTER_PARCEL",
        "EVENT_MN_GYM_4_DEFEATED",
        "EVENT_MN_ICE_CAPTIVE_RESCUED",
        "EVENT_MN_DISPATCH_NOTE",
        "EVENT_MN_COURIER_DEFEATED",
        "EVENT_MN_DEEP_BORE_MANIFEST_READ",
    ] {
        assert!(event(&c, flag), "earned prerequisite {flag}");
    }
    later_story(&c);
    (root, r, c)
}

fn later_story(c: &VisibleShellController) {
    for flag in [
        "EVENT_MN_BRAINERD_REPORT",
        "EVENT_MN_SURF_GIFT_RECEIVED",
        "EVENT_MN_HEADWATERS_REPORT",
        "EVENT_MN_DEPOT_CLEARED",
        "EVENT_MN_LIGHTHOUSE_SIGNAL",
        "EVENT_MN_SURVEY_RESTORED",
    ] {
        assert!(!event(c, flag), "travel cannot supply {flag}");
    }
}

pub(super) fn care(c: &mut VisibleShellController, r: &CrystalRuntime) {
    if c.snapshot().expect("care approach").overworld.map_name == CITY {
        indoor_walk(c, r, (6, 10));
        travel(c, GameButton::Up, CLINIC, (10, 13));
    }
    location(c, CLINIC, (10, 13));
    indoor_walk(c, r, (7, 5));
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
        Some("MN_ST_CLOUD_POKECENTER_1F")
    );
    indoor_walk(c, r, (10, 14));
    travel(c, GameButton::Down, CITY, (6, 10));
}

fn supplies(c: &mut VisibleShellController, r: &CrystalRuntime) {
    supplies::buy(c, r);
}

pub(super) fn enter_road(c: &mut VisibleShellController, r: &CrystalRuntime) {
    indoor_walk(c, r, (24, 34));
    travel(c, GameButton::Down, "MnR05C00R00", (24, 0));
}

pub(super) fn return_to_city(c: &mut VisibleShellController, r: &CrystalRuntime) {
    if c.snapshot()
        .expect("road return")
        .overworld
        .map_name
        .starts_with("MnR05")
    {
        assert!(try_road_with(c, r, 5, (24, 0), flee));
        travel(c, GameButton::Up, CITY, (24, 35));
        travel(c, GameButton::Up, CITY, (24, 34));
    } else {
        location(c, CLINIC, (10, 13));
    }
}

fn road_trainers(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    for (tile, facing, id, flag) in [
        (
            (10, 85),
            GameButton::Right,
            "MN_R05_T01",
            "EVENT_MN_R05_TRAINER_01",
        ),
        (
            (31, 64),
            GameButton::Down,
            "MN_R05_T02",
            "EVENT_MN_R05_TRAINER_02",
        ),
        (
            (19, 34),
            GameButton::Up,
            "MN_R05_T03",
            "EVENT_MN_R05_TRAINER_03",
        ),
        (
            (28, 9),
            GameButton::Right,
            "MN_R05_T04",
            "EVENT_MN_R05_TRAINER_04",
        ),
        (
            (33, 30),
            GameButton::Up,
            "MN_R05_T05",
            "EVENT_MN_R05_TRAINER_05",
        ),
    ] {
        for attempt in 0..4 {
            if event(c, flag) {
                break;
            }
            care(c, r);
            supplies(c, r);
            enter_road(c, r);
            if try_road_with(c, r, 5, tile, flee) {
                press(c, facing);
                press(c, GameButton::A);
                let _ = try_battle(c, r, id, flag);
            }
            return_to_city(c, r);
            continue_in_place(c, root, r, &format!("campaign-{id}-{attempt}"));
        }
        assert!(event(c, flag), "actual route victory {id}");
    }
}

fn route_supply(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    let flag = "EVENT_MN_R05_ITEM_01";
    if event(c, flag) {
        return;
    }
    for attempt in 0..4 {
        care(c, r);
        enter_road(c, r);
        if try_road_with(c, r, 5, (8, 89), flee) {
            let item = &r.data().items["SUPER_POTION"];
            let before = c.shell.shell.session().state().bag.quantity(item);
            press(c, GameButton::Right);
            press(c, GameButton::A);
            finish(c);
            assert!(event(c, flag));
            assert_eq!(
                c.shell.shell.session().state().bag.quantity(item),
                before + 1
            );
        }
        return_to_city(c, r);
        continue_in_place(
            c,
            root,
            r,
            &format!("campaign-earned-wind-route-supply-{attempt}"),
        );
        if event(c, flag) {
            return;
        }
    }
    assert!(event(c, flag));
}

#[test]
#[ignore = "earned campaign; optional CRYSTAL_NORTHSTAR_WIND_SAVE is the actual four-badge return"]
fn northstar_campaign_earned_wind_and_brainerd_arrival() {
    super::northstar_campaign_review::require_unboosted();
    let _ = wind();
}

#[test]
#[ignore = "requires CRYSTAL_NORTHSTAR_WIND_SAVE from an actual earned St Cloud checkpoint"]
fn northstar_campaign_earned_r05_recruitment() {
    super::northstar_campaign_review::require_unboosted();
    assert!(std::env::var_os("CRYSTAL_NORTHSTAR_WIND_SAVE").is_some());
    let (root, r, mut c) = arrival();
    assert!(matches!(
        c.snapshot()
            .expect("earned recruitment departure")
            .overworld
            .map_name
            .as_str(),
        CITY | CLINIC
    ));
    assert_eq!(
        c.snapshot().expect("earned partners").party.slots.len(),
        if r.data().maps.contains_key("MnBoundaryIntake") {
            4
        } else {
            2
        }
    );
    let prior_trainers = [
        "EVENT_MN_R05_TRAINER_01",
        "EVENT_MN_R05_TRAINER_02",
        "EVENT_MN_R05_TRAINER_03",
        "EVENT_MN_R05_TRAINER_04",
        "EVENT_MN_R05_TRAINER_05",
        "EVENT_MN_GYM_5_TRAINER_1_DEFEATED",
        "EVENT_MN_GYM_5_TRAINER_2_DEFEATED",
        "EVENT_MN_GYM_5_DEFEATED",
    ]
    .map(|flag| (flag, event(&c, flag)));
    care(&mut c, &r);
    training::recruit(&mut c, &r, &root);
    assert_eq!(
        c.snapshot()
            .expect("native third capture")
            .party
            .slots
            .len(),
        3
    );
    assert_eq!(
        c.shell.shell.session().state().badges.johto,
        [true, true, true, true, false, false, false, false]
    );
    for (flag, before) in prior_trainers {
        assert_eq!(event(&c, flag), before, "recruitment cannot supply {flag}");
    }
    later_story(&c);
    continue_in_place(&mut c, &root, &r, "campaign-earned-wind-local-partner");
}

pub(super) fn wind() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r, mut c) = arrival();
    super::northstar_campaign_review::boost(&mut c, &r);
    // Validate the actually published room. Current regional descendants can
    // retain the reviewed original courts while newer gym candidates stay
    // unselected; never replace gameplay content to fit a replay driver.
    northstar_earned_gyms::design(&r, 5);
    if c.snapshot().expect("earned departure").overworld.map_name == "MnMinneapolis" {
        indoor_walk(&mut c, &r, (40, 4));
        travel(&mut c, GameButton::Up, "MnR05C00R02", (22, 15));
        assert!(try_road_with(&mut c, &r, 5, (24, 0), flee));
        travel(&mut c, GameButton::Up, CITY, (24, 35));
        travel(&mut c, GameButton::Up, CITY, (24, 34));
        continue_in_place(&mut c, &root, &r, "campaign-earned-st-cloud-arrival");
    } else if c
        .snapshot()
        .expect("continued road recovery")
        .overworld
        .map_name
        == CLINIC
    {
        care(&mut c, &r);
    }
    if r.data().maps.contains_key("MnBoundaryIntake") && !event(&c, "EVENT_MN_REED_WORKER_RELEASED")
    {
        // Approach the visible participants before planning past their lane.
        // Their scene, rather than a geometry shortcut, opens this street.
        indoor_walk(&mut c, &r, (24, 7));
        for _ in 0..3 {
            press(&mut c, GameButton::Up);
            if c.has_pending_script_work() {
                break;
            }
        }
        super::super::northstar_ice_lair::choice(&mut c, false);
        finish(&mut c);
        assert!(!event(&c, "EVENT_MN_REED_WORKER_RELEASED"));
        press(&mut c, GameButton::Up);
        press(&mut c, GameButton::Up);
        super::super::northstar_ice_lair::choice(&mut c, true);
        super::super::northstar_ice_lair::page(&mut c, "MnArcSCReleasedText");
        assert!(!event(&c, "EVENT_MN_REED_WORKER_RELEASED"));
        finish(&mut c);
        assert!(event(&c, "EVENT_MN_REED_WORKER_RELEASED"));
        let snapshot = c.snapshot().expect("worker visibly released");
        let sera = snapshot.visible_object_runtime_tiles["MN_ARC_SC_SERA"];
        assert_eq!((sera.x, sera.y), (24, 9));
        assert!(
            !snapshot
                .visible_objects
                .iter()
                .any(|object| object.object_identifier.as_deref() == Some("MN_ARC_SC_REED"))
        );
        continue_in_place(
            &mut c,
            &root,
            &r,
            "campaign-earned-st-cloud-worker-released",
        );
    }
    // The unearned fifth badge still blocks the next road after four badges.
    indoor_walk(&mut c, &r, (24, 4));
    for _ in 0..8 {
        if c.snapshot().expect("Wind gate").ui.text.is_some() {
            break;
        }
        press(&mut c, GameButton::Up);
    }
    assert!(
        c.snapshot()
            .expect("specific missing leader")
            .ui
            .text
            .and_then(|t| t.asm_text)
            .is_some_and(|t| t.contains("SAVEN"))
    );
    press(&mut c, GameButton::Up);
    location(&mut c, CITY, (24, 3));
    finish(&mut c);
    location(&mut c, CITY, (24, 4));
    later_story(&c);
    care(&mut c, &r);
    road_trainers(&mut c, &r, &root);
    for id in ["MN_GYM_5_TRAINER_1", "MN_GYM_5_TRAINER_2", "MN_GYM_5"] {
        let flag = format!("EVENT_{id}_DEFEATED");
        if event(&c, &flag) {
            continue;
        }
        if id == "MN_GYM_5" {
            training::train(&mut c, &r, &root, 28);
            route_supply(&mut c, &r, &root);
        }
        for attempt in 0..4 {
            care(&mut c, &r);
            if id == "MN_GYM_5" {
                supplies::wake(&mut c, &r);
            }
            supplies(&mut c, &r);
            indoor_walk(&mut c, &r, (8, 28));
            travel(&mut c, GameButton::Up, "MnStCloudGym", (4, 19));
            interact(&mut c, &r, northstar_earned_gyms::opponent(&r, 5, id));
            if try_battle(&mut c, &r, id, &flag) {
                northstar_earned_gyms::leave(&mut c, &r, 5, CITY, (8, 28));
                continue_in_place(&mut c, &root, &r, &format!("campaign-{id}-win"));
                break;
            }
            location(&mut c, CLINIC, (10, 13));
            assert!(!c.shell.shell.session().state().badges.johto[4]);
            continue_in_place(&mut c, &root, &r, &format!("campaign-{id}-loss-{attempt}"));
            let level = c.snapshot().expect("prepared party").party.slots[0]
                .pokemon
                .level;
            let state = c.shell.shell.session().state();
            if id == "MN_GYM_5"
                && level >= 31
                && state.bag.quantity(&r.data().items["SUPER_POTION"]) == 0
                && state.bag.quantity(&r.data().items["TM_STEEL_WING"]) == 0
            {
                break;
            }
            if id == "MN_GYM_5" && attempt < 3 {
                training::train(&mut c, &r, &root, 29 + attempt);
            }
        }
        assert!(event(&c, &flag), "earned Wind opponent {id}");
    }
    assert_eq!(
        c.shell.shell.session().state().badges.johto,
        [true, true, true, true, true, false, false, false]
    );
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["TM_SWIFT"]),
        1
    );
    care(&mut c, &r);
    indoor_walk(&mut c, &r, (24, 4));
    travel(&mut c, GameButton::Up, "MnR06C00R01", (16, 39));
    assert!(try_road_with(&mut c, &r, 6, (42, 0), flee));
    travel(&mut c, GameButton::Up, "MnBrainerd", (32, 47));
    travel(&mut c, GameButton::Up, "MnBrainerd", (32, 46));
    later_story(&c);
    continue_in_place(&mut c, &root, &r, "campaign-earned-brainerd-arrival");
    (root, r, c)
}
