//! Continue the carried party through custody, Harbor and its reveal.
use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::super::visible_field_dialog_pages;
use super::super::northstar_earned_gyms;
use super::super::rochester_earned::try_battle;
use super::super::winona::{at, continue_in_place, finish, load, press, travel};
use super::gym_eight_approach::{accept, care, road};
use super::northstar_campaign_pine::pine;
use super::northstar_campaign_review::boost;
use super::{VisibleShellController, event, interact, walk_to};

const FOREMAN: &str = "EVENT_MN_DEPOT_FOREMAN_DEFEATED";
const METER: &str = "EVENT_MN_MINE_INSTRUMENT_RECOVERED";
const DEPOT: &str = "EVENT_MN_DEPOT_CLEARED";
const SIGNAL: &str = "EVENT_MN_LIGHTHOUSE_SIGNAL";

fn arrival() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let Some(path) = std::env::var_os("CRYSTAL_NORTHSTAR_IRON_SAVE") else {
        return pine();
    };
    let (root, r) = load();
    let mut c =
        VisibleShellController::load_save(root.clone(), r.clone(), PathBuf::from(path), None)
            .expect("actual same-pack northern checkpoint");
    c.set_runtime_journal_enabled(false);
    let won = event(&c, "EVENT_MN_GYM_8_DEFEATED");
    assert_eq!(
        c.shell.shell.session().state().badges.johto,
        [true, true, true, true, true, true, true, won]
    );
    assert!(event(&c, "EVENT_MN_HEADWATERS_REPORT"));
    assert!(!event(&c, "EVENT_MN_SURVEY_RESTORED"));
    assert!(!event(&c, SIGNAL));
    let s = c.snapshot().expect("settled checkpoint");
    assert!(s.battle.is_none() && s.ui.text.is_none() && s.ui.menu.is_none());
    assert!(matches!(
        s.overworld.map_name.as_str(),
        "MnBemidji"
            | "MnR09Cave1F"
            | "MnGrandRapidsSurveyDepot"
            | "MnDuluth"
            | "MnDuluthGym"
            | "MnDuluthHarborWorkshop"
    ));
    (root, r, c)
}

fn mine(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    if c.snapshot().expect("mine approach").overworld.map_name == "MnBemidji" {
        care(c, r, "MnBemidji", (31, 10));
        walk_to(c, r, (43, 25));
        travel(c, GameButton::Right, "MnR09C00R01", (0, 3));
        road(c, r, 9, (40, 42));
        travel(c, GameButton::Up, "MnR09Cave1F", (13, 19));
        travel(c, GameButton::Up, "MnR09Cave1F", (13, 18));
    }
    if r.data().maps.contains_key("MnBoundaryIntake") && !event(c, METER) {
        walk_to(c, r, (13, 18));
        let loaded = super::super::northstar_ice_lair::save(c, root, r, "campaign-arc-mine-entry");
        let mut borrowed = std::mem::replace(c, loaded);
        borrowed = super::super::northstar_drill_lair::play_chapter(borrowed, root, r);
        *c = borrowed;
    } else {
        if !event(c, FOREMAN) {
            interact(c, r, (7, 5));
            finish(c);
            assert!(!event(c, METER));
            interact(c, r, (9, 5));
            assert!(try_battle(c, r, "MN_LODE_FOREMAN", FOREMAN));
            assert!(!event(c, METER) && !event(c, DEPOT));
            continue_in_place(c, root, r, "campaign-foreman-win");
        }
        if !event(c, METER) {
            interact(c, r, (7, 5));
            accept(c, METER);
            assert!(!event(c, METER), "recovery still needs acknowledgment");
            finish(c);
            assert!(event(c, METER) && !event(c, DEPOT));
            continue_in_place(c, root, r, "campaign-instrument-custody");
        }
    }
    walk_to(c, r, (13, 18));
    travel(c, GameButton::Down, "MnR09C00R00", (40, 42));
    road(c, r, 9, (127, 24));
    travel(c, GameButton::Right, "MnGrandRapids", (0, 24));
    travel(c, GameButton::Right, "MnGrandRapids", (3, 24));
    care(c, r, "MnGrandRapids", (9, 12));
    walk_to(c, r, (43, 24));
    for _ in 0..8 {
        if c.snapshot().expect("custody gate").ui.text.is_some() {
            break;
        }
        press(c, GameButton::Right);
    }
    assert_eq!(
        c.snapshot()
            .expect("depot denial")
            .ui
            .text
            .expect("text")
            .label,
        "MnGrandRapidsEastCheckEvidenceText"
    );
    finish(c);
    assert!(!event(c, DEPOT));
    walk_to(c, r, (27, 14));
    travel(c, GameButton::Up, "MnGrandRapidsSurveyDepot", (12, 17));
}

fn handover(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    if !event(c, DEPOT) {
        interact(c, r, (11, 9));
        accept(c, DEPOT);
        assert!(!event(c, DEPOT), "handover still needs acknowledgment");
        finish(c);
        assert!(event(c, DEPOT) && !event(c, SIGNAL));
        continue_in_place(c, root, r, "campaign-depot-handover");
    }
    walk_to(c, r, (12, 18));
    travel(c, GameButton::Down, "MnGrandRapids", (27, 14));
    walk_to(c, r, (43, 24));
    travel(c, GameButton::Right, "MnR10C00R00", (0, 10));
    road(c, r, 10, (95, 24));
    travel(c, GameButton::Right, "MnDuluth", (0, 24));
    travel(c, GameButton::Right, "MnDuluth", (3, 24));
    care(c, r, "MnDuluth", (37, 12));
    continue_in_place(c, root, r, "campaign-duluth-arrival");
}

fn receiver(c: &mut VisibleShellController, r: &CrystalRuntime) {
    if c.snapshot().expect("harbor approach").overworld.map_name == "MnDuluth" {
        walk_to(c, r, (48, 36));
        travel(c, GameButton::Up, "MnDuluthHarborWorkshop", (8, 9));
    }
    walk_to(c, r, (10, 5));
    press(c, GameButton::Right);
    press(c, GameButton::A);
}

fn gym(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    if c.snapshot().expect("gym approach").overworld.map_name == "MnDuluth" {
        receiver(c, r);
        assert_eq!(
            c.snapshot()
                .expect("early receiver")
                .ui
                .text
                .expect("text")
                .label,
            "MN_HARBOR_RECEIVERDeniedText"
        );
        finish(c);
        assert!(!event(c, SIGNAL));
        walk_to(c, r, (8, 10));
        travel(c, GameButton::Down, "MnDuluth", (48, 36));
        walk_to(c, r, (16, 12));
        travel(
            c,
            GameButton::Up,
            "MnDuluthGym",
            northstar_earned_gyms::arrival(r, 8),
        );
    }
    for id in [
        "MN_GYM_8_TRAINER_1",
        "MN_GYM_8_TRAINER_2",
        "MN_GYM_8_TRAINER_3",
        "MN_GYM_8",
    ] {
        let flag = format!("EVENT_{id}_DEFEATED");
        if event(c, &flag) {
            continue;
        }
        interact(c, r, northstar_earned_gyms::opponent(r, 8, id));
        assert!(try_battle(c, r, id, &flag), "played Harbor opponent {id}");
        continue_in_place(c, root, r, &format!("campaign-{id}-win"));
        northstar_earned_gyms::leave(c, r, 8, "MnDuluth", (16, 12));
        care(c, r, "MnDuluth", (37, 12));
        if id != "MN_GYM_8" {
            walk_to(c, r, (16, 12));
            travel(
                c,
                GameButton::Up,
                "MnDuluthGym",
                northstar_earned_gyms::arrival(r, 8),
            );
        }
    }
    assert_eq!(c.shell.shell.session().state().badges.johto, [true; 8]);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["TM_ICY_WIND"]),
        1
    );
    assert!(event(c, "EVENT_MN_GYM_8_COURT_OPEN") && !event(c, SIGNAL));
    continue_in_place(c, root, r, "campaign-harbor-city-return");
}

#[test]
#[ignore = "user-authorized boosted story review from an actual northern save"]
fn northstar_campaign_boosted_mine_depot_harbor_and_signal() {
    assert!(std::env::var_os("CRYSTAL_NORTHSTAR_REVIEW_LEVEL").is_some());
    let _ = iron();
}

pub(super) fn iron() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r, mut c) = arrival();
    boost(&mut c, &r);
    if matches!(
        c.snapshot()
            .expect("continued mine")
            .overworld
            .map_name
            .as_str(),
        "MnBemidji" | "MnR09Cave1F"
    ) {
        mine(&mut c, &r, &root);
    }
    if c.snapshot().expect("continued depot").overworld.map_name == "MnGrandRapidsSurveyDepot" {
        handover(&mut c, &r, &root);
    }
    assert!(event(&c, FOREMAN) && event(&c, METER) && event(&c, DEPOT));
    if !event(&c, "EVENT_MN_GYM_8_DEFEATED") {
        gym(&mut c, &r, &root);
    } else if c.snapshot().expect("gym exit").overworld.map_name == "MnDuluthGym" {
        northstar_earned_gyms::leave(&mut c, &r, 8, "MnDuluth", (16, 12));
    }
    if c.snapshot().expect("reveal approach").overworld.map_name == "MnDuluth" {
        walk_to(&mut c, &r, (48, 36));
        travel(&mut c, GameButton::Up, "MnDuluthHarborWorkshop", (8, 9));
    }
    walk_to(&mut c, &r, (10, 5));
    continue_in_place(&mut c, &root, &r, "campaign-harbor-reveal-ready");
    receiver(&mut c, &r);
    let raw = c
        .shell
        .shell
        .presentation_snapshot()
        .expect("receiver source");
    let pages = visible_field_dialog_pages(&raw, &c.shell).expect("canonical reveal pages");
    assert_eq!(pages.len(), 10);
    for phrase in [
        "KETTLURK",
        "BOUNDARY WATERS",
        "ICE kidnaps people",
        "drainage plan",
    ] {
        assert!(
            pages.iter().any(|p| p.contains(phrase)),
            "missing story fact {phrase}"
        );
    }
    let before = c.shell.shell.session().state().clone();
    for page in pages {
        assert!(!event(&c, SIGNAL));
        assert_eq!(
            c.snapshot()
                .expect("reveal page")
                .ui
                .text
                .as_ref()
                .and_then(|t| t.asm_text.as_deref()),
            Some(page.as_str())
        );
        press(&mut c, GameButton::Down);
        at(&mut c, "MnDuluthHarborWorkshop", (10, 5));
        press(&mut c, GameButton::A);
    }
    if r.data().maps.contains_key("MnBoundaryIntake") {
        assert!(
            !event(&c, SIGNAL),
            "Ivy's live call also requires acknowledgment"
        );
        super::super::northstar_ice_lair::page(&mut c, "MnArcHarborIntakeText");
        finish(&mut c);
    }
    assert!(event(&c, SIGNAL));
    finish(&mut c);
    let after = c.shell.shell.session().state();
    assert_eq!(after.badges, before.badges);
    assert_eq!(after.bag, before.bag);
    assert_eq!(after.storage, before.storage);
    assert_eq!(after.money, before.money);
    continue_in_place(&mut c, &root, &r, "campaign-harbor-signal");
    receiver(&mut c, &r);
    assert_eq!(
        c.snapshot()
            .expect("repeat reveal")
            .ui
            .text
            .expect("text")
            .label,
        "MN_HARBOR_RECEIVERDoneText"
    );
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_SURVEY_RESTORED"));
    (root, r, c)
}
