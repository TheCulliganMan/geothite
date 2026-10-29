//! Continue an earned campaign or an explicitly boosted review through Lake.
use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::authored_route_fixture::indoor_walk;
use super::super::northstar_earned_gyms;
use super::super::rochester_earned::try_battle;
use super::super::winona::{at, continue_in_place, finish, load, press, travel};
use super::northstar_campaign_review::boost;
use super::northstar_campaign_wind::wind;
use super::{VisibleShellController, event, interact};

const CITY: &str = "MnBrainerd";
const CLINIC: &str = "MnBrainerdPokecenter1F";
const REPORT: &str = "EVENT_MN_BRAINERD_REPORT";
const SURF: &str = "EVENT_MN_SURF_GIFT_RECEIVED";
const WON: &str = "EVENT_MN_GYM_6_DEFEATED";

fn arrival() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let Some(path) = std::env::var_os("CRYSTAL_NORTHSTAR_LAKE_SAVE") else {
        return wind();
    };
    let (root, r) = load();
    let mut c =
        VisibleShellController::load_save(root.clone(), r.clone(), PathBuf::from(path), None)
            .expect("actual same-pack Brainerd checkpoint");
    c.set_runtime_journal_enabled(false);
    let won = event(&c, WON);
    assert_eq!(
        c.shell.shell.session().state().badges.johto,
        [true, true, true, true, true, won, false, false]
    );
    match c
        .snapshot()
        .expect("Brainerd checkpoint")
        .overworld
        .map_name
        .as_str()
    {
        CITY if won => at(&mut c, CITY, (12, 36)),
        CITY => at(&mut c, CITY, (32, 46)),
        CLINIC => at(&mut c, CLINIC, (10, 13)),
        _ => panic!("expected a real Brainerd arrival, gym return or native recovery"),
    }
    for flag in [
        "EVENT_MN_ICE_CAPTIVE_RESCUED",
        "EVENT_MN_DISPATCH_NOTE",
        "EVENT_MN_DEEP_BORE_MANIFEST_READ",
        "EVENT_MN_GYM_5_DEFEATED",
    ] {
        assert!(event(&c, flag), "played prerequisite {flag}");
    }
    assert!(!event(&c, REPORT) && !event(&c, SURF));
    later(&c);
    (root, r, c)
}

fn later(c: &VisibleShellController) {
    for flag in [
        "EVENT_MN_HEADWATERS_READING",
        "EVENT_MN_HEADWATERS_REPORT",
        "EVENT_MN_DEPOT_FOREMAN_DEFEATED",
        "EVENT_MN_MINE_INSTRUMENT_RECOVERED",
        "EVENT_MN_DEPOT_CLEARED",
        "EVENT_MN_LIGHTHOUSE_SIGNAL",
        "EVENT_MN_SURVEY_RESTORED",
    ] {
        assert!(!event(c, flag), "Lake cannot supply {flag}");
    }
}

fn care(c: &mut VisibleShellController, r: &CrystalRuntime) {
    if c.snapshot().expect("care approach").overworld.map_name == CITY {
        indoor_walk(c, r, (9, 22));
        travel(c, GameButton::Up, CLINIC, (10, 13));
    }
    indoor_walk(c, r, (7, 5));
    let money = c.shell.shell.session().state().money;
    press(c, GameButton::Up);
    press(c, GameButton::A);
    finish(c);
    assert_eq!(c.shell.shell.session().state().money, money);
    assert!(
        c.snapshot()
            .expect("native free care")
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
    indoor_walk(c, r, (10, 14));
    travel(c, GameButton::Down, CITY, (9, 22));
}

fn supplies(c: &mut VisibleShellController, r: &CrystalRuntime) {
    let item = &r.data().items["SUPER_POTION"];
    let state = c.shell.shell.session().state();
    let needed = 5_u16.saturating_sub(state.bag.quantity(item));
    let quantity = (state.money / u32::from(item.price)).min(u32::from(needed)) as u16;
    if quantity == 0 {
        return;
    }
    indoor_walk(c, r, (19, 22));
    travel(c, GameButton::Up, "MnBrainerdMart", (6, 5));
    indoor_walk(c, r, (3, 3));
    press(c, GameButton::Left);
    press(c, GameButton::A);
    super::super::rochester_earned::checkout(c, r, "SUPER_POTION", quantity);
    indoor_walk(c, r, (6, 6));
    travel(c, GameButton::Down, CITY, (19, 22));
}

fn winch(c: &mut VisibleShellController, r: &CrystalRuntime, flag: &str) {
    let router = &northstar_earned_gyms::design(r, 6)["routers"][0];
    let at = super::point(&router["at"]);
    for _ in 0..2 {
        if event(c, flag) {
            return;
        }
        interact(c, r, at);
        if router["operator"].is_string() {
            let choice = router["modes"]
                .as_array()
                .expect("published dock modes")
                .iter()
                .position(|mode| mode["event"] == flag)
                .expect("dock direction");
            super::choose(c, choice);
        } else {
            assert!(c.snapshot().expect("physical winch").ui.menu.is_none());
        }
        finish(c);
    }
    assert!(event(c, flag), "played dock control {flag}");
}

fn report(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    indoor_walk(c, r, (25, 20));
    press(c, GameButton::Up);
    press(c, GameButton::A);
    if r.data()
        .story_event_script_constants
        .global
        .contains_key("EVENT_MN_TAMARACK_INTRO")
    {
        assert!(!event(c, REPORT) && !event(c, SURF));
        super::super::northstar_dialogue::acknowledge(c, "MS05", "MN_BRAINERD_RANGERText", REPORT);
    } else {
        for expected in ["The fish moved on", "Ask the fishers"] {
            assert!(!event(c, REPORT) && !event(c, SURF));
            assert!(
                c.snapshot()
                    .expect("separate ranger page")
                    .ui
                    .text
                    .and_then(|t| t.asm_text)
                    .is_some_and(|text| text.contains(expected))
            );
            press(c, GameButton::Down);
            at(c, CITY, (25, 20));
            press(c, GameButton::A);
        }
    }
    finish(c);
    assert!(event(c, REPORT) && event(c, SURF));
    let item = &r.data().items["HM_SURF"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(item), 1);
    if r.data().maps.contains_key("MnBoundaryIntake") {
        let slot = c
            .snapshot()
            .expect("earned Water partner")
            .party
            .slots
            .iter()
            .position(|p| p.pokemon.species.tmhm_learnset.iter().any(|m| m == "SURF"));
        if let Some(slot) = slot {
            super::northstar_campaign::learn_machine_on(c, r, "HM_SURF", "SURF", "GROWL", slot);
        }
        assert_eq!(c.shell.shell.session().state().bag.quantity(item), 1);
    }
    continue_in_place(c, root, r, "campaign-lake-ranger-report");
    let party = c.snapshot().expect("before ranger repeat").party;
    press(c, GameButton::Up);
    press(c, GameButton::A);
    finish(c);
    assert_eq!(c.snapshot().expect("ranger repeat").party, party);
    assert_eq!(c.shell.shell.session().state().bag.quantity(item), 1);
}

#[test]
#[ignore = "user-authorized boosted story review; optional actual Brainerd save"]
fn northstar_campaign_boosted_lake_and_ranger_report() {
    assert!(std::env::var_os("CRYSTAL_NORTHSTAR_REVIEW_LEVEL").is_some());
    let _ = lake();
}

pub(super) fn lake() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r, mut c) = arrival();
    boost(&mut c, &r);
    if c.snapshot()
        .expect("before ranger qualification")
        .overworld
        .map_name
        == CLINIC
    {
        care(&mut c, &r);
    }
    if !event(&c, WON) {
        indoor_walk(&mut c, &r, (25, 20));
        press(&mut c, GameButton::Up);
        press(&mut c, GameButton::A);
        assert!(
            c.snapshot()
                .expect("ranger requires Lake")
                .ui
                .text
                .and_then(|t| t.asm_text)
                .is_some_and(|text| text.contains("SIVREN"))
        );
        finish(&mut c);
        assert!(!event(&c, REPORT) && !event(&c, SURF));
        care(&mut c, &r);
        for id in ["MN_GYM_6_TRAINER_1", "MN_GYM_6_TRAINER_2", "MN_GYM_6"] {
            let flag = format!("EVENT_{id}_DEFEATED");
            if event(&c, &flag) {
                continue;
            }
            for attempt in 0..4 {
                care(&mut c, &r);
                supplies(&mut c, &r);
                let party = c.snapshot().expect("choose an earned Lake partner").party;
                if let Some(slot) = party.slots.iter().position(|slot| {
                    matches!(slot.pokemon.species.type1.as_str(), "ELECTRIC" | "GRASS")
                        || matches!(slot.pokemon.species.type2.as_str(), "ELECTRIC" | "GRASS")
                }) {
                    super::northstar_arc_campaign::switch_lead(&mut c, slot);
                }
                indoor_walk(&mut c, &r, (12, 36));
                travel(
                    &mut c,
                    GameButton::Up,
                    "MnBrainerdGym",
                    northstar_earned_gyms::arrival(&r, 6),
                );
                if id == "MN_GYM_6_TRAINER_2" {
                    winch(&mut c, &r, "EVENT_MN_GYM_6_WALK_CENTER");
                } else if id == "MN_GYM_6" {
                    winch(&mut c, &r, "EVENT_MN_GYM_6_WALK_LEFT");
                    assert!(event(&c, "EVENT_MN_GYM_6_COURT_OPEN"));
                }
                interact(&mut c, &r, northstar_earned_gyms::opponent(&r, 6, id));
                if try_battle(&mut c, &r, id, &flag) {
                    northstar_earned_gyms::leave(&mut c, &r, 6, CITY, (12, 36));
                    continue_in_place(&mut c, &root, &r, &format!("campaign-{id}-win"));
                    break;
                }
                let map = c
                    .snapshot()
                    .expect("native Lake recovery")
                    .overworld
                    .map_name;
                assert!(map == CLINIC || map == CITY, "native recovery in {map}");
                continue_in_place(&mut c, &root, &r, &format!("campaign-{id}-loss-{attempt}"));
            }
            assert!(event(&c, &flag), "played Lake opponent {id}");
        }
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&r.data().items["TM_RAIN_DANCE"]),
            1
        );
        at(&mut c, CITY, (12, 36));
        continue_in_place(&mut c, &root, &r, "campaign-lake-gym-return");
    }
    assert_eq!(
        c.shell.shell.session().state().badges.johto,
        [true, true, true, true, true, true, false, false]
    );
    assert!(!event(&c, REPORT) && !event(&c, SURF));
    report(&mut c, &r, &root);
    later(&c);
    (root, r, c)
}
