//! Read the actual sign and counter from the approach a visitor uses in Ely.
use std::path::{Path, PathBuf};

use crystal_core::input::GameButton::{A, Right, Up};

use super::super::{RuntimeBadgeRegion, VisibleShellController, mark_runtime_snapshot_dirty};
use super::badge_trails::load;
use super::brainerd_guide::capture;
use super::duluth_story::press;
use super::interior_life::fixture;
use super::northstar_ice_lair::{choice, event, page};
use super::r10_gallery::settle;
use super::winona::travel;

const RESTORED: &str = "EVENT_MN_SURVEY_RESTORED";
const CHAMPION: &str = "EVENT_MN_LEAGUE_CHAMPION";
const TOWN: &str = "MnEly";
const LOBBY: &str = "MnLeagueLobby";

/// Both the ready visitor and the survey-delayed visitor earned the Harbor Badge.
fn prepare(c: &mut VisibleShellController, milestones: &[&str]) {
    for slot in 0..8 {
        c.shell
            .shell
            .award_badge(RuntimeBadgeRegion::Johto, slot)
            .expect("visitor's badge");
    }
    let flags = &mut c.shell.shell.session_mut().state_mut().flags;
    flags
        .set_event_flag("EVENT_MN_GYM_8_DEFEATED", true)
        .expect("Harbor Gym win");
    for &flag in milestones {
        flags
            .set_event_flag(flag, true)
            .expect("survey or title milestone");
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
}

fn read(
    mut c: VisibleShellController,
    label: &str,
    pages: &[&str],
    frames: Option<&Path>,
) -> VisibleShellController {
    for (index, expected) in pages.iter().enumerate() {
        if index > 0 {
            press(&mut c, A);
        }
        let snap = c.snapshot().expect("player's current page");
        let text = snap.ui.text.expect("a printed page");
        assert_eq!(text.label, label);
        assert_eq!(text.asm_text.as_deref(), Some(*expected));
        if let Some(directory) = frames {
            c = capture(c, Some(directory), &format!("{label}-{}.png", index + 1));
        }
    }
    c
}

fn approach_trail_sign(c: &mut VisibleShellController) {
    // Earlier editions read the paving to the west of the standing board.
    let x = c.shell.shell.runtime().data().maps[TOWN]
        .events
        .bg_events
        .iter()
        .find(|e| e.script == "MnElyTrailSign")
        .expect("League directions")
        .x;
    travel(c, Up, TOWN, (20, 4));
    if x == 21 {
        travel(c, Right, TOWN, (21, 4));
        press(c, Up);
    } else {
        assert_eq!(x, 20, "reviewed sign location");
    }
    press(c, A);
}

#[test]
#[ignore = "requires an Ely Challenge edition in MN_BADGE_TRAILS_PACK"]
fn ely_sign_and_registrar_explain_the_actual_run_and_survey() {
    let (root, runtime) = load();
    let frames = std::env::var_os("MN_ELY_CHALLENGE_FRAMES").map(PathBuf::from);
    let mut c = fixture(&root, &runtime, TOWN, (20, 5));
    prepare(&mut c, &["EVENT_MN_MINE_INSTRUMENT_RECOVERED"]);
    approach_trail_sign(&mut c);
    c = read(
        c,
        "MnWayLeagueHoldText",
        &[
            "ELY LEAGUE",
            "Waiting on the\nintake crew.",
            "Ask at the lobby\ndesk for news.",
        ],
        frames.as_deref(),
    );
    settle(&mut c, "leave the sign");
    // Use the real doors, then ask the same question at the desk. At this
    // stage the right destination is the Grand Rapids depot, before the shelter.
    super::minnesota_gyms::walk_to(&mut c, &runtime, (25, 14));
    travel(&mut c, Up, LOBBY, (1, 4));
    super::minnesota_gyms::walk_to(&mut c, &runtime, (5, 4));
    press(&mut c, Up);
    press(&mut c, A);
    page(&mut c, "MnElyBringMeterText");
    settle(&mut c, "leave the desk with the depot directions");
    assert!(!event(&c, "EVENT_MN_LEAGUE_RUN_ACTIVE"));

    let mut challenger = fixture(&root, &runtime, LOBBY, (5, 5));
    prepare(&mut challenger, &[RESTORED]);
    travel(&mut challenger, Up, LOBBY, (5, 4));
    press(&mut challenger, A);
    challenger = read(
        challenger,
        "MnElyRegistrationText",
        &[
            "REGISTRAR: Ready\nto register?",
            "FERN first. Four\nwins, then CASEY.",
            "Got your medicine?\nNo nurse past me.",
            "You can go back\nto earlier rooms.",
            "The run ends when\nyou lose a match",
            "or come back to\nthis lobby.",
        ],
        frames.as_deref(),
    );
    choice(&mut challenger, false);
    page(&mut challenger, "MnElyPrepareOutsideText");
    challenger = read(
        challenger,
        "MnElyPrepareOutsideText",
        &[
            "REGISTRAR: Take\nyour time.",
            "The clinic is\nsouthwest of us.",
            "The MART is south,\non the other path.",
        ],
        None,
    );
    settle(&mut challenger, "the challenge was declined");
    assert!(!event(&challenger, "EVENT_MN_LEAGUE_RUN_ACTIVE"));

    let mut returning = fixture(&root, &runtime, TOWN, (20, 5));
    prepare(&mut returning, &[RESTORED, CHAMPION]);
    approach_trail_sign(&mut returning);
    returning = read(
        returning,
        "MnWayLeagueAgainText",
        &[
            "ELY LEAGUE",
            "Sign in at the\nlobby desk.",
            "Four wins before\nCASEY.",
            "You can return to\nearlier rooms.",
            "Returning to the\nlobby ends a run.",
        ],
        frames.as_deref(),
    );
    settle(&mut returning, "leave the returning challenger sign");
}
