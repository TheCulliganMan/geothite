//! Continue the boosted party through Pine and the independent Itasca report.
use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::authored_route_fixture::indoor_walk;
use super::super::northstar_earned_gyms;
use super::super::rochester_earned::try_battle;
use super::super::winona::{at, continue_in_place, finish, load, press, travel};
use super::itasca_story::{
    acknowledge_marker, acknowledge_report, from_bemidji, marker, refusal,
    to_bemidji as from_itasca,
};
use super::leech_shore::{from_brainerd, to_bemidji};
use super::northstar_campaign_lake::lake;
use super::northstar_campaign_review::boost;
use super::{VisibleShellController, cut_gym_tree, event, interact, learn_gym_cut_replacing};

const CITY: &str = "MnBemidji";
const CLINIC: &str = "MnBemidjiPokecenter1F";
const GYM: &str = "MnBemidjiGym";
const PARK: &str = "MnLakeItasca";
const WON: &str = "EVENT_MN_GYM_7_DEFEATED";
const READING: &str = "EVENT_MN_HEADWATERS_READING";
const REPORT: &str = "EVENT_MN_HEADWATERS_REPORT";

fn later(c: &VisibleShellController) {
    for flag in [
        "EVENT_MN_DEPOT_FOREMAN_DEFEATED",
        "EVENT_MN_MINE_INSTRUMENT_RECOVERED",
        "EVENT_MN_DEPOT_CLEARED",
        "EVENT_MN_LIGHTHOUSE_SIGNAL",
        "EVENT_MN_SURVEY_RESTORED",
    ] {
        assert!(!event(c, flag), "the headwaters cannot supply {flag}");
    }
}

fn arrival() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let Some(path) = std::env::var_os("CRYSTAL_NORTHSTAR_PINE_SAVE") else {
        return lake();
    };
    let (root, r) = load();
    let mut c =
        VisibleShellController::load_save(root.clone(), r.clone(), PathBuf::from(path), None)
            .expect("actual same-pack Lake or Pine checkpoint");
    c.set_runtime_journal_enabled(false);
    let won = event(&c, WON);
    assert_eq!(
        c.shell.shell.session().state().badges.johto,
        [true, true, true, true, true, true, won, false]
    );
    match c
        .snapshot()
        .expect("actual northern checkpoint")
        .overworld
        .map_name
        .as_str()
    {
        "MnBrainerd" => at(&mut c, "MnBrainerd", (25, 20)),
        CITY if won => at(&mut c, CITY, (10, 10)),
        CITY => at(&mut c, CITY, (24, 46)),
        CLINIC => at(&mut c, CLINIC, (10, 13)),
        PARK if won && event(&c, READING) => at(&mut c, PARK, (19, 22)),
        GYM => {
            let snapshot = c.snapshot().expect("native gym checkpoint");
            assert!(
                snapshot.battle.is_none()
                    && snapshot.ui.text.is_none()
                    && snapshot.ui.menu.is_none()
            );
        }
        _ => panic!("expected the actual lake report, Bemidji arrival, gym, reading or recovery"),
    }
    for flag in [
        "EVENT_MN_ICE_CAPTIVE_RESCUED",
        "EVENT_MN_DISPATCH_NOTE",
        "EVENT_MN_DEEP_BORE_MANIFEST_READ",
        "EVENT_MN_BRAINERD_REPORT",
        "EVENT_MN_SURF_GIFT_RECEIVED",
        "EVENT_MN_GYM_6_DEFEATED",
    ] {
        assert!(event(&c, flag), "played prerequisite {flag}");
    }
    assert!(!event(&c, REPORT));
    later(&c);
    (root, r, c)
}

fn pages(c: &mut VisibleShellController, expected: &[&str]) {
    let mut text = String::new();
    for _ in 0..32 {
        let Some(page) = c.snapshot().expect("witness page").ui.text else {
            break;
        };
        text.push_str(&page.asm_text.expect("native witness text"));
        text.push('\n');
        press(c, GameButton::A);
    }
    assert!(
        c.snapshot()
            .expect("witness releases input")
            .ui
            .text
            .is_none()
    );
    for part in expected {
        assert!(
            text.contains(part),
            "missing witness wording {part}: {text}"
        );
    }
}

fn witnesses(c: &mut VisibleShellController, r: &CrystalRuntime) {
    interact(c, r, (35, 21));
    pages(c, &["LODE", "nets", "empty"]);
    indoor_walk(c, r, (9, 36));
    travel(c, GameButton::Up, "MnLeechLakeFisherHouse", (5, 6));
    indoor_walk(c, r, (6, 4));
    press(c, GameButton::Up);
    press(c, GameButton::A);
    pages(c, &["ICE", "dates"]);
    indoor_walk(c, r, (5, 6));
    travel(c, GameButton::Down, "MnLeechLake", (9, 36));
    later(c);
    assert!(!event(c, READING) && !event(c, REPORT));
}

fn care(c: &mut VisibleShellController, r: &CrystalRuntime) {
    if c.snapshot().expect("Bemidji care").overworld.map_name == CITY {
        indoor_walk(c, r, (31, 10));
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
    travel(c, GameButton::Down, CITY, (31, 10));
}

fn gym(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    if c.snapshot().expect("Pine approach").overworld.map_name != GYM {
        care(c, r);
        indoor_walk(c, r, (10, 10));
        travel(c, GameButton::Up, GYM, northstar_earned_gyms::arrival(r, 7));
    }
    if let Some(slot) = c
        .snapshot()
        .expect("choose a carried Pine partner")
        .party
        .slots
        .iter()
        .position(|slot| {
            slot.pokemon.species.type1 == "FIRE" || slot.pokemon.species.type2 == "FIRE"
        })
    {
        super::northstar_arc_campaign::switch_lead(c, slot);
    }
    let design = northstar_earned_gyms::design(r, 7);
    if !c
        .snapshot()
        .expect("actual Cut knowledge")
        .party
        .slots
        .iter()
        .any(|p| p.pokemon.moves.iter().any(|m| m.name == "CUT"))
    {
        assert!(
            c.snapshot()
                .expect("legal Cut recipient")
                .party
                .slots
                .iter()
                .any(|p| p
                    .pokemon
                    .species
                    .tmhm_learnset
                    .iter()
                    .any(|name| name == "CUT"))
        );
        learn_gym_cut_replacing(c, r, design, Some("SMOKESCREEN"));
        assert!(
            c.snapshot()
                .expect("actual Cut lesson")
                .party
                .slots
                .iter()
                .any(|slot| slot.pokemon.moves.iter().any(|m| m.name == "CUT"))
        );
        continue_in_place(c, root, r, "campaign-pine-learned-cut");
    }
    for id in ["MN_GYM_7_TRAINER_1", "MN_GYM_7_TRAINER_2", "MN_GYM_7"] {
        let flag = format!("EVENT_{id}_DEFEATED");
        if event(c, &flag) {
            continue;
        }
        let hedge = match id {
            "MN_GYM_7_TRAINER_2" => Some(0),
            "MN_GYM_7" => Some(1),
            _ => None,
        };
        if let Some(index) = hedge {
            let gate = &design["cut_gates"][index];
            if !event(c, gate["event"].as_str().expect("native hedge event")) {
                cut_gym_tree(c, r, gate);
            }
        }
        interact(c, r, northstar_earned_gyms::opponent(r, 7, id));
        assert!(try_battle(c, r, id, &flag), "played Pine opponent {id}");
        continue_in_place(c, root, r, &format!("campaign-{id}-win"));
        if id != "MN_GYM_7" {
            northstar_earned_gyms::leave(c, r, 7, CITY, (10, 10));
            care(c, r);
            indoor_walk(c, r, (10, 10));
            travel(c, GameButton::Up, GYM, northstar_earned_gyms::arrival(r, 7));
        }
    }
    if northstar_earned_gyms::selected(r) {
        assert!(event(c, "EVENT_MN_GYM_7_RETURN_OPEN"));
    }
    assert!(
        !event(c, "EVENT_MN_GYM_7_TRAINER_3_DEFEATED"),
        "Brindel stays optional"
    );
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["TM_GIGA_DRAIN"]),
        1
    );
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["HM_CUT"]),
        1
    );
    northstar_earned_gyms::leave(c, r, 7, CITY, (10, 10));
    continue_in_place(c, root, r, "campaign-pine-city-return");
}

#[test]
#[ignore = "user-authorized boosted story review from an actual Lake or Pine save"]
fn northstar_campaign_boosted_pine_itasca_and_independent_report() {
    assert!(std::env::var_os("CRYSTAL_NORTHSTAR_REVIEW_LEVEL").is_some());
    let _ = pine();
}

pub(super) fn pine() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r, mut c) = arrival();
    boost(&mut c, &r);
    if c.snapshot().expect("lake departure").overworld.map_name == "MnBrainerd" {
        indoor_walk(&mut c, &r, (32, 4));
        from_brainerd(&mut c, &r);
        witnesses(&mut c, &r);
        to_bemidji(&mut c, &r);
        continue_in_place(&mut c, &root, &r, "campaign-bemidji-arrival");
    }
    if !event(&c, WON) {
        if c.snapshot().expect("continued approach").overworld.map_name == CLINIC {
            care(&mut c, &r);
        }
        if c.snapshot().expect("early park visit").overworld.map_name == CITY
            && !event(&c, "EVENT_MN_GYM_7_TRAINER_1_DEFEATED")
        {
            from_bemidji(&mut c, &r, 0);
            marker(&mut c, &r);
            if r.data()
                .story_event_script_constants
                .global
                .contains_key("EVENT_MN_TAMARACK_INTRO")
            {
                super::super::northstar_dialogue::acknowledge(
                    &mut c,
                    "MS06",
                    "MnItascaMarkerBeforeText",
                    READING,
                );
            } else {
                assert!(
                    c.snapshot()
                        .expect("early marker refusal")
                        .ui
                        .text
                        .and_then(|t| t.asm_text)
                        .is_some_and(|text| text.contains("PINE"))
                );
            }
            finish(&mut c);
            assert!(!event(&c, READING) && !event(&c, REPORT));
            from_itasca(&mut c, &r, 1);
        }
        gym(&mut c, &r, &root);
    } else if c
        .snapshot()
        .expect("won Pine checkpoint")
        .overworld
        .map_name
        == GYM
    {
        northstar_earned_gyms::leave(&mut c, &r, 7, CITY, (10, 10));
    }
    assert_eq!(
        c.shell.shell.session().state().badges.johto,
        [true, true, true, true, true, true, true, false]
    );
    if !event(&c, READING) {
        from_bemidji(&mut c, &r, 0);
        acknowledge_marker(&mut c, &r);
        continue_in_place(&mut c, &root, &r, "campaign-itasca-reading");
    }
    at(&mut c, PARK, (19, 22));
    from_itasca(&mut c, &r, 1);
    indoor_walk(&mut c, &r, (43, 25));
    refusal(
        &mut c,
        true,
        1,
        if r.data()
            .story_event_script_constants
            .global
            .contains_key("EVENT_MN_TAMARACK_INTRO")
        {
            "ITASCA"
        } else {
            "marker"
        },
    );
    acknowledge_report(&mut c, &r);
    assert!(event(&c, READING) && event(&c, REPORT));
    later(&c);
    continue_in_place(&mut c, &root, &r, "campaign-headwaters-report");
    (root, r, c)
}
