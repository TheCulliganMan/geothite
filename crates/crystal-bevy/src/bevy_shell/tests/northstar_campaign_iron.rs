//! Continue the carried party through custody, Harbor and its reveal.
#[path = "northstar_mine_training.rs"]
mod training;

use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::super::visible_field_dialog_pages;
use super::super::northstar_earned_gyms;
use super::super::rochester_earned::{checkout, try_battle};
use super::super::winona::{at, continue_in_place, finish, load, press, travel};
use super::gym_eight_approach::{accept, care, road};
use super::northstar_campaign_pine::pine;
use super::northstar_campaign_review::boost;
use super::{VisibleShellController, event, interact, walk_to};

const FOREMAN: &str = "EVENT_MN_DEPOT_FOREMAN_DEFEATED";
const METER: &str = "EVENT_MN_MINE_INSTRUMENT_RECOVERED";
const DEPOT: &str = "EVENT_MN_DEPOT_CLEARED";
const SIGNAL: &str = "EVENT_MN_LIGHTHOUSE_SIGNAL";

fn supplies(c: &mut VisibleShellController, r: &CrystalRuntime) {
    let item = &r.data().items["HYPER_POTION"];
    let state = c.shell.shell.session().state();
    let quantity = 2_u16
        .saturating_sub(state.bag.quantity(item))
        .min((state.money / u32::from(item.price)) as u16);
    if quantity == 0 {
        return;
    }
    let (door, arrival) =
        super::super::native_city_ports::city_port(r, "MnBemidji", "MnBemidjiMart");
    walk_to(
        c,
        r,
        (
            u16::try_from(door.0).expect("native Mart entrance x is on the city map"),
            u16::try_from(door.1).expect("native Mart entrance y is on the city map"),
        ),
    );
    travel(c, GameButton::Up, "MnBemidjiMart", arrival);
    walk_to(c, r, (3, 3));
    press(c, GameButton::Left);
    press(c, GameButton::A);
    checkout(c, r, "HYPER_POTION", quantity);
    walk_to(c, r, (6, 6));
    travel(c, GameButton::Down, "MnBemidji", door);
}

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
    assert!(
        !event(&c, SIGNAL)
            || (super::super::northstar_dialogue::timely_evidence(&r) && event(&c, DEPOT)),
        "only the pacing revision accepts recovered evidence before the final Gym"
    );
    let s = c.snapshot().expect("settled checkpoint");
    assert!(s.battle.is_none() && s.ui.text.is_none() && s.ui.menu.is_none());
    assert!(
        s.overworld.map_name.starts_with("MnR08C")
            || matches!(
                s.overworld.map_name.as_str(),
                "MnBemidji"
                    | "MnBemidjiPokecenter1F"
                    | "MnR09Cave1F"
                    | "MnIronDrillCore"
                    | "MnGrandRapidsSurveyDepot"
                    | "MnDuluth"
                    | "MnDuluthGym"
                    | "MnDuluthHarborWorkshop"
            )
    );
    (root, r, c)
}

fn mine(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    if c.snapshot()
        .expect("actual recruitment checkpoint")
        .overworld
        .map_name
        .starts_with("MnR08C")
    {
        training::return_from_recruitment(c, r);
        continue_in_place(c, root, r, "campaign-mine-income-return");
    }
    if c.snapshot()
        .expect("native mine loss recovery")
        .overworld
        .map_name
        == "MnBemidjiPokecenter1F"
    {
        walk_to(c, r, (10, 14));
        travel(c, GameButton::Down, "MnBemidji", (31, 10));
    }
    if c.snapshot()
        .expect("completed drill Continue")
        .overworld
        .map_name
        == "MnIronDrillCore"
    {
        assert!(event(c, METER));
        let loaded = super::super::northstar_ice_lair::save(c, root, r, "campaign-crew-return");
        let borrowed = std::mem::replace(c, loaded);
        *c = super::super::northstar_drill_lair::leave_chapter(borrowed, r);
        continue_in_place(c, root, r, "campaign-mine-return");
    }
    if c.snapshot().expect("mine approach").overworld.map_name == "MnBemidji" {
        care(c, r, "MnBemidji", (31, 10));
        // Prepare a varied carried party before committing to the mine, using
        // ordinary encounters and purchases rather than waiting for a loss.
        training::prepare(c, r, root);
        care(c, r, "MnBemidji", (31, 10));
        if event(c, FOREMAN)
            && let Some(slot) = c
                .snapshot()
                .expect("carried mine retry team")
                .party
                .slots
                .iter()
                .filter(|slot| {
                    slot.pokemon.moves.iter().any(|learned| {
                        let definition = &r.data().moves[&learned.name];
                        learned.current_pp > 0
                            && definition.power >= 60
                            && definition.move_type == "GROUND"
                    })
                })
                .max_by_key(|slot| slot.pokemon.level)
                .map(|slot| slot.index)
        {
            // This guard's Fire/Steel partner has already been encountered.
            // Choose the carried Ground coverage through the native Party UI.
            super::northstar_arc_campaign::switch_lead(c, slot);
        }
        supplies(c, r);
        walk_to(c, r, (43, 25));
        travel(c, GameButton::Right, "MnR09C00R01", (0, 3));
        road(c, r, 9, (40, 42));
        travel(c, GameButton::Up, "MnR09Cave1F", (13, 19));
        travel(c, GameButton::Up, "MnR09Cave1F", (13, 18));
    }
    if r.data().maps.contains_key("MnBoundaryIntake") && !event(c, METER) {
        super::super::northstar_ice_lair::walk(c, r, (13, 18));
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
    super::super::northstar_ice_lair::walk(c, r, (13, 18));
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
    let connection = r.data().maps["MnR10C01R00"]
        .attributes
        .connections
        .iter()
        .find(|link| link.direction == "east" && link.target_map == "MnDuluth")
        .expect("published Duluth inlet");
    let y = 24 - i16::try_from(connection.offset * 2).expect("native inlet offset");
    travel(c, GameButton::Right, "MnDuluth", (0, y));
    travel(c, GameButton::Right, "MnDuluth", (3, y));
    care(c, r, "MnDuluth", (69, 32));
    continue_in_place(c, root, r, "campaign-duluth-arrival");
}

fn receiver(c: &mut VisibleShellController, r: &CrystalRuntime) {
    if c.snapshot().expect("harbor approach").overworld.map_name == "MnDuluth" {
        walk_to(c, r, (72, 42));
        travel(c, GameButton::Up, "MnDuluthHarborWorkshop", (8, 9));
    }
    walk_to(c, r, (10, 5));
    press(c, GameButton::Right);
    press(c, GameButton::A);
}

fn gym(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    let signal_before = event(c, SIGNAL);
    assert_eq!(
        signal_before,
        super::super::northstar_dialogue::timely_evidence(r),
        "the pacing course delivers the evidence before final training"
    );
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
        assert_eq!(event(c, SIGNAL), signal_before);
        walk_to(c, r, (8, 10));
        travel(c, GameButton::Down, "MnDuluth", (72, 42));
        walk_to(c, r, (48, 32));
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
        northstar_earned_gyms::leave(c, r, 8, "MnDuluth", (48, 32));
        care(c, r, "MnDuluth", (69, 32));
        if id != "MN_GYM_8" {
            walk_to(c, r, (48, 32));
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
    assert!(event(c, "EVENT_MN_GYM_8_COURT_OPEN"));
    assert_eq!(event(c, SIGNAL), signal_before);
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
    let map = c.snapshot().expect("continued mine").overworld.map_name;
    if map.starts_with("MnR08C")
        || matches!(
            map.as_str(),
            "MnBemidji" | "MnBemidjiPokecenter1F" | "MnR09Cave1F" | "MnIronDrillCore"
        )
    {
        mine(&mut c, &r, &root);
    }
    if c.snapshot().expect("continued depot").overworld.map_name == "MnGrandRapidsSurveyDepot" {
        handover(&mut c, &r, &root);
    }
    assert!(event(&c, FOREMAN) && event(&c, METER) && event(&c, DEPOT));
    // Evidence reaches the crew before the final training requirement. The
    // same ordinary Gym and coast traversal still follow; no badge is granted.
    if super::super::northstar_dialogue::timely_evidence(&r) && !event(&c, SIGNAL) {
        reveal(&mut c, &r, &root);
    }
    if !event(&c, "EVENT_MN_GYM_8_DEFEATED") {
        if c.snapshot()
            .expect("leave the planning crew")
            .overworld
            .map_name
            == "MnDuluthHarborWorkshop"
        {
            walk_to(&mut c, &r, (8, 9));
            travel(&mut c, GameButton::Down, "MnDuluth", (72, 42));
        }
        gym(&mut c, &r, &root);
    } else if c.snapshot().expect("gym exit").overworld.map_name == "MnDuluthGym" {
        northstar_earned_gyms::leave(&mut c, &r, 8, "MnDuluth", (48, 32));
    }
    if !event(&c, SIGNAL) {
        reveal(&mut c, &r, &root);
    }
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

fn reveal(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    if c.snapshot().expect("reveal approach").overworld.map_name == "MnDuluth" {
        walk_to(c, r, (72, 42));
        travel(c, GameButton::Up, "MnDuluthHarborWorkshop", (8, 9));
    }
    walk_to(c, r, (10, 5));
    continue_in_place(c, root, r, "campaign-harbor-reveal-ready");
    receiver(c, r);
    let before = c.shell.shell.session().state().clone();
    if r.data()
        .story_event_script_constants
        .global
        .contains_key("EVENT_MN_TAMARACK_INTRO")
    {
        for label in [
            "MN_HARBOR_RECEIVERText",
            "MnOverhaulHarborEvidenceText",
            "MnOverhaulHarborReactionText",
            "MnOverhaulHarborDirectionsText",
        ] {
            super::super::northstar_ice_lair::page(c, label);
            super::super::northstar_dialogue::acknowledge(c, "MS10", label, SIGNAL);
        }
    } else {
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
        for page in pages {
            assert!(!event(c, SIGNAL));
            assert_eq!(
                c.snapshot()
                    .expect("reveal page")
                    .ui
                    .text
                    .as_ref()
                    .and_then(|t| t.asm_text.as_deref()),
                Some(page.as_str())
            );
            press(c, GameButton::Down);
            at(c, "MnDuluthHarborWorkshop", (10, 5));
            press(c, GameButton::A);
        }
    }
    if r.data().maps.contains_key("MnBoundaryIntake") {
        assert!(
            !event(c, SIGNAL),
            "the intake directions also require acknowledgment"
        );
        super::super::northstar_ice_lair::page(c, "MnArcHarborIntakeText");
        finish(c);
    }
    assert!(event(c, SIGNAL));
    finish(c);
    let after = c.shell.shell.session().state();
    assert_eq!(after.badges, before.badges);
    assert_eq!(after.bag, before.bag);
    assert_eq!(after.storage, before.storage);
    assert_eq!(after.money, before.money);
    continue_in_place(c, root, r, "campaign-harbor-signal");
}
