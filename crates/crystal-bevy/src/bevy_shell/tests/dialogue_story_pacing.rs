//! Declared investigation states supplement the separate earned campaign.
use crystal_core::input::GameButton;
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;

use super::super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::super::minnesota_gyms::interact;
use super::super::northstar_dialogue::{acknowledge, timely_evidence};
use super::super::northstar_ice_lair::event;
use super::super::winona::{continue_in_place, finish, press};
use super::fixture;

const READING: &str = "EVENT_MN_HEADWATERS_READING";
const REPORT: &str = "EVENT_MN_HEADWATERS_REPORT";
const SIGNAL: &str = "EVENT_MN_LIGHTHOUSE_SIGNAL";
const LINK: &str = "EVENT_MN_ARC_LINK_SENT";
const RESTORED: &str = "EVENT_MN_SURVEY_RESTORED";
const CHAMPION: &str = "EVENT_MN_LEAGUE_CHAMPION";

fn badges(c: &mut VisibleShellController, count: usize) {
    let state = c.shell.shell.session_mut().state_mut();
    for (i, flag) in BADGE_ENGINE_FLAGS[..8].iter().enumerate() {
        state.badges.johto[i] = i < count;
        state
            .flags
            .set_engine_flag(flag, i < count)
            .expect("declared prior badge");
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
}

#[test]
#[ignore = "requires the pacing candidate and isolated saves; six prior badges are declared"]
fn overhaul_public_reference_and_comparison_do_not_require_the_pine_badge() {
    let (root, r, mut c) = fixture("MnLakeItasca", (19, 22), &[]);
    assert!(timely_evidence(&r), "matching pacing candidate required");
    badges(&mut c, 6);
    let before = c.shell.shell.session().state().clone();
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    continue_in_place(&mut c, &root, &r, "pacing-unread-public-reference");
    acknowledge(&mut c, "MS06", "MN_ITASCA_MARKERText", READING);
    finish(&mut c);
    assert!(event(&c, READING) && !event(&c, REPORT));
    let after = c.shell.shell.session().state();
    assert_eq!(after.badges, before.badges);
    assert_eq!(after.bag, before.bag);

    let (root, r, mut c) = fixture("MnBemidji", (21, 20), &[READING]);
    badges(&mut c, 6);
    let before = c.shell.shell.session().state().clone();
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    continue_in_place(&mut c, &root, &r, "pacing-unread-independent-comparison");
    acknowledge(&mut c, "MS07", "MnBemidjiComparisonText", REPORT);
    finish(&mut c);
    assert!(event(&c, REPORT));
    assert_eq!(c.shell.shell.session().state().badges, before.badges);
    assert_eq!(c.shell.shell.session().state().bag, before.bag);
    // Accepting evidence does not grant the travel credential or open the mine.
    super::super::authored_route_fixture::indoor_walk(&mut c, &r, (43, 25));
    press(&mut c, GameButton::Right);
    assert!(
        c.snapshot()
            .expect("mine travel safeguard")
            .ui
            .text
            .and_then(|t| t.asm_text)
            .is_some_and(|text| text.contains("PINE"))
    );
    finish(&mut c);
    assert!(!c.shell.shell.session().state().badges.johto[6]);
    assert!(!event(&c, "EVENT_MN_MINE_INSTRUMENT_RECOVERED"));
}

#[test]
#[ignore = "requires the pacing candidate; seven prior badges and the recovered pump log are declared"]
fn overhaul_harbor_accepts_the_pump_log_before_final_gym_training() {
    let (root, r, mut c) = fixture(
        "MnDuluthHarborWorkshop",
        (8, 9),
        &["EVENT_MN_DEPOT_CLEARED"],
    );
    assert!(timely_evidence(&r), "matching pacing candidate required");
    badges(&mut c, 7);
    let before = c.shell.shell.session().state().clone();
    super::super::authored_route_fixture::indoor_walk(&mut c, &r, (10, 5));
    press(&mut c, GameButton::Right);
    press(&mut c, GameButton::A);
    continue_in_place(&mut c, &root, &r, "pacing-harbor-evidence-before-final-gym");
    for label in [
        "MN_HARBOR_RECEIVERText",
        "MnOverhaulHarborEvidenceText",
        "MnOverhaulHarborReactionText",
        "MnOverhaulHarborDirectionsText",
        "MnArcHarborIntakeText",
    ] {
        acknowledge(&mut c, "MS10", label, SIGNAL);
    }
    finish(&mut c);
    assert!(event(&c, SIGNAL));
    assert!(!event(&c, LINK) && !event(&c, RESTORED));
    let after = c.shell.shell.session().state();
    assert_eq!(after.badges, before.badges);
    assert_eq!(after.bag, before.bag);
    assert_eq!(after.money, before.money);
    press(&mut c, GameButton::A);
    acknowledge(&mut c, "PACING", "MN_HARBOR_RECEIVERDeniedText", CHAMPION);
    finish(&mut c);
    assert!(!c.shell.shell.session().state().badges.johto[7]);
}

#[test]
#[ignore = "requires the pacing candidate; chief defeat/radio/shutdown are explicit chapter fixtures"]
fn overhaul_chief_reacts_to_radio_exposure_and_completed_shutdown() {
    for (flags, label) in [
        (vec![], "MnBoundarySurveyChiefAfterText"),
        (vec![LINK], "MnOverhaulChiefExposedText"),
        (vec![LINK, RESTORED], "MnOverhaulChiefStoppedText"),
    ] {
        let mut flags = flags;
        flags.push("EVENT_MN_SURVEY_CHIEF_DEFEATED");
        let (root, r, mut c) = fixture("MnBoundaryWatersSurveyShelter", (12, 4), &flags);
        assert!(timely_evidence(&r), "matching pacing candidate required");
        let actor = r.data().maps["MnBoundaryWatersSurveyShelter"]
            .objects
            .iter()
            .find(|actor| actor.script == "MN_SURVEY_CHIEF")
            .expect("native chief actor");
        let target = (actor.x, actor.y);
        let before = c.shell.shell.session().state().clone();
        interact(&mut c, &r, target);
        continue_in_place(&mut c, &root, &r, &format!("pacing-chief-{label}"));
        acknowledge(&mut c, "PACING", label, CHAMPION);
        finish(&mut c);
        let after = c.shell.shell.session().state();
        assert_eq!(after.flags, before.flags);
        assert_eq!(after.badges, before.badges);
        assert_eq!(after.bag, before.bag);
        assert!(
            c.snapshot()
                .expect("conversation, no repeated battle")
                .battle
                .is_none()
        );
    }
}

#[test]
#[ignore = "requires the pacing candidate; cumulative investigation and return states are declared"]
fn overhaul_officer_return_advice_tracks_meter_custody_radio_and_shutdown() {
    let mut flags = vec![READING, REPORT];
    for (event, label) in [
        (
            "EVENT_MN_MINE_INSTRUMENT_RECOVERED",
            "MnOverhaulOfficerMeterText",
        ),
        ("EVENT_MN_DEPOT_CLEARED", "MnOverhaulOfficerHarborText"),
        (SIGNAL, "MnOverhaulOfficerNorthText"),
        (LINK, "MnOverhaulOfficerIntakeText"),
        (RESTORED, "MnOverhaulOfficerDoneText"),
    ] {
        flags.push(event);
        let (root, r, mut c) = fixture("MnBemidji", (21, 20), &flags);
        assert!(timely_evidence(&r), "matching pacing candidate required");
        let before = c.shell.shell.session().state().clone();
        press(&mut c, GameButton::Up);
        press(&mut c, GameButton::A);
        continue_in_place(&mut c, &root, &r, &format!("pacing-officer-{label}"));
        acknowledge(&mut c, "PACING", label, CHAMPION);
        finish(&mut c);
        let after = c.shell.shell.session().state();
        assert_eq!(after.flags, before.flags);
        assert_eq!(after.bag, before.bag);
        assert_eq!(after.badges, before.badges);
    }
}

#[test]
#[ignore = "requires the pacing candidate; prior report/custody/radio/ending states are declared"]
fn overhaul_reference_ranger_and_depot_returns_keep_completed_actions() {
    let (root, r, mut c) = fixture("MnLakeItasca", (19, 22), &[READING, REPORT]);
    assert!(timely_evidence(&r), "matching pacing candidate required");
    let before = c.shell.shell.session().state().clone();
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    continue_in_place(&mut c, &root, &r, "pacing-compared-public-record");
    acknowledge(&mut c, "PACING", "MnOverhaulItascaComparedText", CHAMPION);
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().flags, before.flags);

    for (map, root_script, tile, flags, label) in [
        (
            "MnBrainerd",
            "MnBrainerdRangerRepeat",
            (25, 20),
            vec![
                REPORT,
                "EVENT_MN_BRAINERD_REPORT",
                "EVENT_MN_SURF_GIFT_RECEIVED",
            ],
            "MnOverhaulRangerComparedText",
        ),
        (
            "MnGrandRapidsSurveyDepot",
            "MnGrandRapidsDepotDone",
            (4, 7),
            vec!["EVENT_MN_DEPOT_CLEARED", SIGNAL],
            "MnOverhaulDepotNorthText",
        ),
        (
            "MnGrandRapidsSurveyDepot",
            "MnGrandRapidsDepotDone",
            (4, 7),
            vec!["EVENT_MN_DEPOT_CLEARED", SIGNAL, LINK],
            "MnOverhaulDepotIntakeText",
        ),
        (
            "MnGrandRapidsSurveyDepot",
            "MnGrandRapidsDepotDone",
            (4, 7),
            vec!["EVENT_MN_DEPOT_CLEARED", SIGNAL, LINK, RESTORED],
            "MnOverhaulDepotDoneText",
        ),
    ] {
        let (root, r, mut c) = fixture(map, tile, &flags);
        badges(&mut c, 8);
        let module = &r.data().maps[map];
        let actor = module
            .objects
            .iter()
            .find(|actor| {
                module
                    .scripts
                    .get(&actor.script)
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|body| body.iter().any(|row| row["args"][0] == root_script))
            })
            .expect("actual actor owns this return branch");
        let target = (actor.x, actor.y);
        let before = c.shell.shell.session().state().clone();
        interact(&mut c, &r, target);
        continue_in_place(&mut c, &root, &r, &format!("pacing-return-{label}"));
        acknowledge(&mut c, "PACING", label, CHAMPION);
        finish(&mut c);
        let after = c.shell.shell.session().state();
        assert_eq!(after.flags, before.flags);
        assert_eq!(after.badges, before.badges);
        assert_eq!(after.bag, before.bag);
        assert_eq!(after.money, before.money);
    }
}
