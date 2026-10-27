//! Continue actual earned Boundary Waters progress; never inject finale flags.
use crystal_core::input::GameButton;
use serde_json::Value;

use super::super::super::visible_field_dialog_pages;
use super::super::duluth_story::press as text_press;
use super::super::winona::{at, continued, travel};
use super::leech_shore::play_with_selected_move;
use super::north_shore_story::choice;
use super::{VisibleShellController, event, finish, interact, load, press, walk_to};

const CAMP: &str = "MnBoundaryWaters";
const ROOM: &str = "MnBoundaryWatersSurveyShelter";
const CHIEF: &str = "EVENT_MN_SURVEY_CHIEF_DEFEATED";
const RESTORED: &str = "EVENT_MN_SURVEY_RESTORED";

fn canonical() -> Value {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../planning/minnesota/story.json");
    serde_json::from_str(&std::fs::read_to_string(path).expect("canonical story"))
        .expect("story JSON")
}

fn pages(c: &mut VisibleShellController, story: &Value, milestone: &str, label: &str) {
    let raw = c
        .shell
        .shell
        .presentation_snapshot()
        .expect("native text stream");
    let actual = visible_field_dialog_pages(&raw, &c.shell).expect("native page boundaries");
    let expected = story["milestones"]
        .as_array()
        .expect("milestones")
        .iter()
        .find(|m| m["id"] == milestone)
        .expect("canonical milestone")["native_texts"][label]
        .as_array()
        .expect("canonical lines")
        .chunks(2)
        .map(|lines| {
            lines
                .iter()
                .map(|l| l.as_str().expect("line"))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, expected, "canonical native {label}");
    for page in expected {
        assert_eq!(
            c.snapshot()
                .expect("visible page")
                .ui
                .text
                .and_then(|t| t.asm_text),
            Some(page.clone())
        );
        let before = c.snapshot().expect("text owns input").overworld;
        text_press(c, GameButton::Down);
        let after = c.snapshot().expect("movement rejected").overworld;
        assert_eq!((after.map_name, after.tile), (before.map_name, before.tile));
        text_press(c, GameButton::A);
    }
}

fn enter(c: &mut VisibleShellController, r: &crystal_runtime::CrystalRuntime) {
    walk_to(c, r, (43, 20));
    travel(c, GameButton::Up, ROOM, (8, 9));
}

fn league_refusal(c: &mut VisibleShellController, r: &crystal_runtime::CrystalRuntime) {
    walk_to(c, r, (32, 41));
    for _ in 0..8 {
        if c.snapshot().expect("League refusal").ui.text.is_some() {
            break;
        }
        press(c, GameButton::Down);
    }
    assert!(
        c.snapshot()
            .expect("restoration prerequisite")
            .ui
            .text
            .and_then(|t| t.asm_text)
            .is_some_and(|t| t.contains("Restore the survey"))
    );
    at(c, CAMP, (32, 42));
    press(c, GameButton::Down);
    at(c, CAMP, (32, 42));
    finish(c);
    at(c, CAMP, (32, 41));
    assert!(!event(c, RESTORED));
}

#[test]
#[ignore = "requires the actual earned shore-arrival save via CRYSTAL_NORTHSTAR_BOUNDARY_SAVE"]
fn northstar_finale_controller_earned_chief_separate_receiver_and_league_release() {
    let (root, r, _) = load();
    let path =
        std::env::var_os("CRYSTAL_NORTHSTAR_BOUNDARY_SAVE").expect("earned Boundary arrival save");
    let mut c = VisibleShellController::load_save(root.clone(), r.clone(), path.into(), None)
        .expect("Continue actual earned regional journey");
    at(&mut c, CAMP, (60, 24));
    assert_eq!(c.shell.shell.session().state().badges.johto, [true; 8]);
    for flag in [
        "EVENT_MN_HEADWATERS_READING",
        "EVENT_MN_HEADWATERS_REPORT",
        "EVENT_MN_DEPOT_FOREMAN_DEFEATED",
        "EVENT_MN_MINE_INSTRUMENT_RECOVERED",
        "EVENT_MN_DEPOT_CLEARED",
        "EVENT_MN_LIGHTHOUSE_SIGNAL",
    ] {
        assert!(event(&c, flag), "earned upstream progress: {flag}");
    }
    assert!(!event(&c, CHIEF) && !event(&c, RESTORED));
    let story = canonical();
    league_refusal(&mut c, &r);
    enter(&mut c, &r);
    walk_to(&mut c, &r, (12, 4));
    text_press(&mut c, GameButton::Up);
    text_press(&mut c, GameButton::A);
    pages(&mut c, &story, "MS12", "MnBoundarySurveyReceiverLockedText");
    finish(&mut c);
    let money = c.shell.shell.session().state().money;
    interact(&mut c, &r, (3, 7));
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().money, money);
    assert!(
        c.snapshot()
            .expect("free field care")
            .party
            .slots
            .iter()
            .all(|p| p.pokemon.hp == p.pokemon.max_hp
                && p.pokemon
                    .moves
                    .iter()
                    .all(|m| m.current_pp == r.data().moves[&m.name].pp))
    );
    walk_to(&mut c, &r, (8, 5));
    c = continued(c, &root, &r, "northstar-earned-chief-ready");
    text_press(&mut c, GameButton::Right);
    text_press(&mut c, GameButton::A);
    assert!(!event(&c, CHIEF) && !event(&c, RESTORED));
    pages(&mut c, &story, "MS11", "MnBoundarySurveyChiefSeenText");
    play_with_selected_move(&mut c, |b| {
        if b.enemy_pokemon.species.id == "MAGNETON" {
            "FLAMETHROWER"
        } else {
            "PSYCHIC_M"
        }
    });
    finish(&mut c);
    assert!(event(&c, CHIEF) && !event(&c, RESTORED));
    assert!(
        c.snapshot().expect("chief defeated").party.slots[0]
            .pokemon
            .hp
            > 0
    );
    c = continued(c, &root, &r, "northstar-earned-chief-victory");
    let won = c.shell.shell.session().state().clone();
    text_press(&mut c, GameButton::Right);
    text_press(&mut c, GameButton::A);
    pages(&mut c, &story, "MS11", "MnBoundarySurveyChiefAfterText");
    finish(&mut c);
    walk_to(&mut c, &r, (8, 10));
    travel(&mut c, GameButton::Down, CAMP, (43, 20));
    league_refusal(&mut c, &r);
    enter(&mut c, &r);
    for yes in [false, true] {
        walk_to(&mut c, &r, (12, 4));
        text_press(&mut c, GameButton::Up);
        text_press(&mut c, GameButton::A);
        choice(&mut c, yes);
        assert!(!event(&c, RESTORED));
        if yes {
            pages(
                &mut c,
                &story,
                "MS12",
                "MnBoundarySurveyReceiverRestoreText",
            );
            assert!(event(&c, RESTORED));
            pages(&mut c, &story, "MS12", "MnBoundarySurveyReceiverDoneText");
        } else {
            pages(
                &mut c,
                &story,
                "MS12",
                "MnBoundarySurveyReceiverDeclinedText",
            );
        }
        finish(&mut c);
    }
    assert!(event(&c, CHIEF) && event(&c, RESTORED));
    let after = c.shell.shell.session().state();
    assert_eq!(
        (after.money, &after.badges, &after.bag, &after.storage),
        (won.money, &won.badges, &won.bag, &won.storage)
    );
    c = continued(c, &root, &r, "northstar-earned-survey-restored");
    text_press(&mut c, GameButton::Up);
    text_press(&mut c, GameButton::A);
    pages(
        &mut c,
        &story,
        "MS12",
        "MnBoundarySurveyReceiverAlreadyText",
    );
    finish(&mut c);
    assert!(
        !c.shell
            .shell
            .session()
            .state()
            .flags
            .is_engine_flag_set("ENGINE_SAFARI_ZONE")
            .expect("optional Safari remains separate")
    );
    interact(&mut c, &r, (3, 7));
    finish(&mut c);
    walk_to(&mut c, &r, (9, 10));
    travel(&mut c, GameButton::Down, CAMP, (43, 20));
    walk_to(&mut c, &r, (32, 41));
    travel(&mut c, GameButton::Down, "MnR13C00R00", (30, 0));
    c = continued(c, &root, &r, "northstar-earned-league-road");
    super::north_shore_journey::road(&mut c, &r, 13, (34, 126));
    at(&mut c, "MnR13C00R02", (34, 30));
    let _ = continued(c, &root, &r, "northstar-earned-ely-frontier");
}
