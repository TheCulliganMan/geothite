//! One local Brainerd setup, then earned Pine, report, custody and harbor reveal.
use std::collections::BTreeMap;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::super::visible_field_dialog_pages;
use super::super::authored_route_fixture::{RouteGrid, RouteSpec, position};
use super::super::duluth_story::press as text_press;
use super::super::winona::{at, continued, travel};
use super::gym_eight_approach::{accept, care, road, try_road};
use super::itasca_story::report_course;
use super::leech_shore::{play_with_move, play_with_selected_move};
use super::{VisibleShellController, event, finish, interact, load, point, press, walk_to};

const FOREMAN: &str = "EVENT_MN_DEPOT_FOREMAN_DEFEATED";
const METER: &str = "EVENT_MN_MINE_INSTRUMENT_RECOVERED";
const DEPOT: &str = "EVENT_MN_DEPOT_CLEARED";
const SIGNAL: &str = "EVENT_MN_LIGHTHOUSE_SIGNAL";

fn harbor_fight(
    mut c: VisibleShellController,
    root: &AssetRoot,
    r: &CrystalRuntime,
    target: (u16, u16),
    trainer_flag: Option<&str>,
) -> VisibleShellController {
    for _ in 0..3 {
        let money = c.shell.shell.session().state().money;
        interact(&mut c, r, target);
        play_with_selected_move(&mut c, |b| match b.enemy_pokemon.species.id.as_str() {
            "SWINUB" | "PILOSWINE" | "SNEASEL" => "FLAMETHROWER",
            _ => "THUNDERBOLT",
        });
        finish(&mut c);
        if trainer_flag.map_or_else(
            || c.shell.shell.session().state().badges.johto[7],
            |f| event(&c, f),
        ) {
            return c;
        }
        at(&mut c, "MnDuluthPokecenter1F", (10, 13));
        assert!(c.shell.shell.session().state().money < money);
        assert!(!event(&c, SIGNAL));
        c = continued(c, root, r, "northstar-earned-harbor-loss");
        walk_to(&mut c, r, (10, 14));
        travel(&mut c, GameButton::Down, "MnDuluth", (37, 12));
        care(&mut c, r, "MnDuluth", (37, 12));
        walk_to(&mut c, r, (16, 12));
        travel(&mut c, GameButton::Up, "MnDuluthGym", (6, 17));
    }
    panic!("native Harbor retry exhausted");
}

#[test]
#[ignore = "requires a controller-earned foreman/meter save via CRYSTAL_NORTHSTAR_CUSTODY_SAVE"]
fn northstar_iron_course_played_wild_loss_preserves_earned_custody_and_continue() {
    let (root, r, _) = load();
    let path = std::env::var_os("CRYSTAL_NORTHSTAR_CUSTODY_SAVE").expect("earned meter save");
    let mut c = VisibleShellController::load_save(
        root.clone(),
        r.clone(),
        std::path::PathBuf::from(path),
        None,
    )
    .expect("Continue earned custody");
    assert!(event(&c, FOREMAN) && event(&c, METER) && !event(&c, DEPOT));
    assert!(event(&c, "EVENT_MN_HEADWATERS_READING") && event(&c, "EVENT_MN_HEADWATERS_REPORT"));
    walk_to(&mut c, &r, (13, 18));
    travel(&mut c, GameButton::Down, "MnR09C00R00", (40, 42));
    let initial_money = c.shell.shell.session().state().money;
    let initial_badges = c.shell.shell.session().state().badges.clone();
    let initial_bag = c.shell.shell.session().state().bag.clone();
    let grid = RouteGrid::new_with_grass(
        &r,
        RouteSpec {
            prefix: "MnR09",
            columns: 2,
            rows: 2,
            tileset: "mn_iron_range",
            start: (40, 42),
            doors: BTreeMap::from([((40, 41), "MnR09Cave1F"), ((40, 42), "MnR09Cave1F")]),
        },
    );
    let route = grid.path((40, 42), (54, 42));
    let steps = route.into_iter().chain((0..1024).map(|i| {
        if i % 2 == 0 {
            GameButton::Right
        } else {
            GameButton::Left
        }
    }));
    let mut played_loss = false;
    for button in steps {
        if c.snapshot().expect("wild approach").overworld.map_name == "MnBemidjiPokecenter1F" {
            played_loss = true;
            break;
        }
        let before = position(&mut c);
        for _ in 0..8 {
            press(&mut c, button);
            if c.snapshot().expect("actual encounter").battle.is_some() {
                // A deliberately weak, already learned move lets the real
                // wild battle resolve. No party HP, opponent or RNG is edited.
                play_with_move(&mut c, "CUT");
                finish(&mut c);
            }
            if c.snapshot()
                .expect("route or native recovery")
                .overworld
                .map_name
                == "MnBemidjiPokecenter1F"
            {
                played_loss = true;
                break;
            }
            if position(&mut c) != before {
                break;
            }
        }
        if played_loss {
            break;
        }
        assert_ne!(position(&mut c), before, "native grass step");
    }
    assert!(
        played_loss,
        "bounded native encounters must produce the reviewed loss"
    );
    at(&mut c, "MnBemidjiPokecenter1F", (10, 13));
    assert!(c.shell.shell.session().state().money < initial_money);
    assert_eq!(c.shell.shell.session().state().badges, initial_badges);
    assert_eq!(c.shell.shell.session().state().bag, initial_bag);
    assert!(
        c.snapshot()
            .expect("blackout care")
            .party
            .slots
            .iter()
            .all(|s| s.pokemon.hp == s.pokemon.max_hp
                && s.pokemon
                    .moves
                    .iter()
                    .all(|m| m.current_pp == r.data().moves[&m.name].pp))
    );
    for flag in [
        FOREMAN,
        METER,
        "EVENT_MN_HEADWATERS_READING",
        "EVENT_MN_HEADWATERS_REPORT",
    ] {
        assert!(event(&c, flag));
    }
    assert!(!event(&c, DEPOT) && !event(&c, SIGNAL));
    c = continued(c, &root, &r, "northstar-earned-custody-loss");
    walk_to(&mut c, &r, (10, 14));
    travel(&mut c, GameButton::Down, "MnBemidji", (31, 10));
    walk_to(&mut c, &r, (43, 25));
    travel(&mut c, GameButton::Right, "MnR09C00R01", (0, 3));
    assert!(event(&c, FOREMAN) && event(&c, METER) && !event(&c, DEPOT));
}

#[test]
#[ignore = "external Itasca pack; six prior badges/Mew45/Cut are initial setup only"]
fn northstar_iron_course_earns_pine_report_mine_handover_harbor_and_reveal() {
    let _ = signal_course();
}

pub(super) fn signal_course() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r, mut c) = report_course();
    for flag in [FOREMAN, METER, DEPOT, SIGNAL] {
        assert!(!event(&c, flag), "no supplied downstream progress: {flag}");
    }
    care(&mut c, &r, "MnBemidji", (31, 10));
    walk_to(&mut c, &r, (43, 25));
    travel(&mut c, GameButton::Right, "MnR09C00R01", (0, 3));
    c = continued(c, &root, &r, "northstar-iron-road-ready");
    let mut reached_mine = false;
    for attempt in 0..3 {
        let money = c.shell.shell.session().state().money;
        if try_road(&mut c, &r, 9, (40, 42)) {
            reached_mine = true;
            break;
        }
        at(&mut c, "MnBemidjiPokecenter1F", (10, 13));
        assert!(
            c.shell.shell.session().state().money < money,
            "actual blackout money loss"
        );
        assert!(
            c.snapshot()
                .expect("native blackout care")
                .party
                .slots
                .iter()
                .all(|s| s.pokemon.hp == s.pokemon.max_hp
                    && s.pokemon
                        .moves
                        .iter()
                        .all(|m| m.current_pp == r.data().moves[&m.name].pp))
        );
        for flag in [FOREMAN, METER, DEPOT, SIGNAL] {
            assert!(!event(&c, flag));
        }
        assert!(event(&c, "EVENT_MN_HEADWATERS_REPORT"));
        c = continued(
            c,
            &root,
            &r,
            &format!("northstar-native-road-blackout-{attempt}"),
        );
        walk_to(&mut c, &r, (10, 14));
        travel(&mut c, GameButton::Down, "MnBemidji", (31, 10));
        walk_to(&mut c, &r, (43, 25));
        travel(&mut c, GameButton::Right, "MnR09C00R01", (0, 3));
    }
    assert!(
        reached_mine,
        "three ordinary road attempts did not reach the mine"
    );
    travel(&mut c, GameButton::Up, "MnR09Cave1F", (13, 19));
    travel(&mut c, GameButton::Up, "MnR09Cave1F", (13, 18));
    interact(&mut c, &r, (7, 5));
    finish(&mut c);
    assert!(!event(&c, METER));
    interact(&mut c, &r, (9, 5));
    play_with_selected_move(&mut c, |b| {
        if b.enemy_pokemon.species.id == "MAGNETON" {
            "FLAMETHROWER"
        } else {
            "PSYCHIC_M"
        }
    });
    finish(&mut c);
    assert!(event(&c, FOREMAN) && !event(&c, METER) && !event(&c, DEPOT));
    c = continued(c, &root, &r, "northstar-earned-foreman");
    interact(&mut c, &r, (7, 5));
    accept(&mut c, METER);
    assert!(
        !event(&c, METER),
        "recovery text still needs acknowledgment"
    );
    finish(&mut c);
    assert!(event(&c, METER) && !event(&c, DEPOT));
    c = continued(c, &root, &r, "northstar-earned-meter");
    walk_to(&mut c, &r, (13, 18));
    travel(&mut c, GameButton::Down, "MnR09C00R00", (40, 42));
    let mut reached_depot_town = false;
    for attempt in 0..3 {
        let money = c.shell.shell.session().state().money;
        if try_road(&mut c, &r, 9, (127, 24)) {
            reached_depot_town = true;
            break;
        }
        at(&mut c, "MnBemidjiPokecenter1F", (10, 13));
        assert!(c.shell.shell.session().state().money < money);
        assert!(event(&c, FOREMAN) && event(&c, METER) && !event(&c, DEPOT));
        assert!(event(&c, "EVENT_MN_HEADWATERS_REPORT"));
        c = continued(
            c,
            &root,
            &r,
            &format!("northstar-custody-road-blackout-{attempt}"),
        );
        walk_to(&mut c, &r, (10, 14));
        travel(&mut c, GameButton::Down, "MnBemidji", (31, 10));
        walk_to(&mut c, &r, (43, 25));
        travel(&mut c, GameButton::Right, "MnR09C00R01", (0, 3));
    }
    assert!(
        reached_depot_town,
        "three ordinary road attempts did not reach Grand Rapids"
    );
    travel(&mut c, GameButton::Right, "MnGrandRapids", (0, 24));
    travel(&mut c, GameButton::Right, "MnGrandRapids", (3, 24));
    care(&mut c, &r, "MnGrandRapids", (9, 12));
    walk_to(&mut c, &r, (43, 24));
    for _ in 0..8 {
        if c.snapshot().expect("checkpoint").ui.text.is_some() {
            break;
        }
        super::press(&mut c, GameButton::Right);
    }
    assert_eq!(
        c.snapshot()
            .expect("custody denial")
            .ui
            .text
            .expect("text")
            .label,
        "MnGrandRapidsEastCheckEvidenceText"
    );
    finish(&mut c);
    assert!(!event(&c, DEPOT));
    walk_to(&mut c, &r, (27, 14));
    travel(&mut c, GameButton::Up, "MnGrandRapidsSurveyDepot", (12, 17));
    interact(&mut c, &r, (11, 9));
    accept(&mut c, DEPOT);
    assert!(
        !event(&c, DEPOT),
        "handover text still needs acknowledgment"
    );
    finish(&mut c);
    assert!(event(&c, DEPOT) && !event(&c, SIGNAL));
    c = continued(c, &root, &r, "northstar-earned-depot");
    walk_to(&mut c, &r, (12, 18));
    travel(&mut c, GameButton::Down, "MnGrandRapids", (27, 14));
    walk_to(&mut c, &r, (43, 24));
    travel(&mut c, GameButton::Right, "MnR10C00R00", (0, 10));
    road(&mut c, &r, 10, (95, 24));
    travel(&mut c, GameButton::Right, "MnDuluth", (0, 24));
    travel(&mut c, GameButton::Right, "MnDuluth", (3, 24));
    care(&mut c, &r, "MnDuluth", (37, 12));
    c = continued(c, &root, &r, "northstar-earned-duluth");
    // Harbor signal is still denied before the actual eighth gym win.
    walk_to(&mut c, &r, (48, 36));
    travel(&mut c, GameButton::Up, "MnDuluthHarborWorkshop", (8, 9));
    walk_to(&mut c, &r, (10, 5));
    text_press(&mut c, GameButton::Right);
    text_press(&mut c, GameButton::A);
    assert_eq!(
        c.snapshot()
            .expect("receiver prerequisite")
            .ui
            .text
            .expect("text")
            .label,
        "MN_HARBOR_RECEIVERDeniedText"
    );
    finish(&mut c);
    assert!(!event(&c, SIGNAL));
    walk_to(&mut c, &r, (8, 10));
    travel(&mut c, GameButton::Down, "MnDuluth", (48, 36));
    walk_to(&mut c, &r, (16, 12));
    travel(&mut c, GameButton::Up, "MnDuluthGym", (6, 17));
    let design: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../planning/minnesota/gyms/interiors.json"),
        )
        .expect("gym source"),
    )
    .expect("gym design");
    let gym = &design["gyms"][7];
    for step in gym["walkthrough"]
        .as_array()
        .expect("Harbor trainer course")
    {
        let t = gym["trainers"]
            .as_array()
            .expect("Harbor trainers")
            .iter()
            .find(|t| t["id"] == *step)
            .expect("native court trainer");
        let flag = format!("EVENT_{}_DEFEATED", step.as_str().expect("trainer id"));
        c = harbor_fight(c, &root, &r, point(&t["at"]), Some(&flag));
        assert!(event(&c, &flag));
        walk_to(&mut c, &r, (6, 18));
        travel(&mut c, GameButton::Down, "MnDuluth", (16, 12));
        care(&mut c, &r, "MnDuluth", (37, 12));
        walk_to(&mut c, &r, (16, 12));
        travel(&mut c, GameButton::Up, "MnDuluthGym", (6, 17));
    }
    assert!(event(&c, "EVENT_MN_GYM_8_COURT_OPEN"));
    assert!(!c.shell.shell.session().state().badges.johto[7]);
    walk_to(&mut c, &r, (6, 18));
    travel(&mut c, GameButton::Down, "MnDuluth", (16, 12));
    care(&mut c, &r, "MnDuluth", (37, 12));
    walk_to(&mut c, &r, (16, 12));
    travel(&mut c, GameButton::Up, "MnDuluthGym", (6, 17));
    c = harbor_fight(c, &root, &r, point(&gym["leader_at"]), None);
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
    assert!(!event(&c, SIGNAL));
    c = continued(c, &root, &r, "northstar-earned-harbor");
    walk_to(&mut c, &r, (6, 18));
    travel(&mut c, GameButton::Down, "MnDuluth", (16, 12));
    care(&mut c, &r, "MnDuluth", (37, 12));
    walk_to(&mut c, &r, (48, 36));
    travel(&mut c, GameButton::Up, "MnDuluthHarborWorkshop", (8, 9));
    walk_to(&mut c, &r, (10, 5));
    c = continued(c, &root, &r, "northstar-reveal-ready");
    text_press(&mut c, GameButton::Right);
    text_press(&mut c, GameButton::A);
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
        let s = c.snapshot().expect("reveal");
        assert_eq!(
            s.ui.text.as_ref().and_then(|t| t.asm_text.as_deref()),
            Some(page.as_str())
        );
        text_press(&mut c, GameButton::Down);
        at(&mut c, "MnDuluthHarborWorkshop", (10, 5));
        assert!(!event(&c, SIGNAL));
        text_press(&mut c, GameButton::A);
    }
    assert!(event(&c, SIGNAL));
    finish(&mut c);
    let after = c.shell.shell.session().state();
    assert_eq!(after.badges, before.badges);
    assert_eq!(after.bag, before.bag);
    assert_eq!(after.storage, before.storage);
    assert_eq!(after.money, before.money);
    c = continued(c, &root, &r, "northstar-earned-signal");
    text_press(&mut c, GameButton::Right);
    text_press(&mut c, GameButton::A);
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
    assert!(event(&c, SIGNAL) && !event(&c, "EVENT_MN_SURVEY_RESTORED"));
    (root, r, c)
}
