//! Prior badges/party/placement are setup; all chapter progress is native input.
use std::collections::BTreeMap;

use crystal_core::input::GameButton;
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind};

use super::super::authored_route_fixture::{RouteGrid, RouteSpec, position};
use super::super::rochester::flee;
use super::super::winona::{at, continued, travel};
use super::leech_shore::{fixture, pine_course};
use super::{VisibleShellController, battle, event, finish, mark_runtime_snapshot_dirty, press};

const PARK: &str = "MnLakeItasca";
const GATE: &str = "MnItascaGatehouse";
const READING: &str = "EVENT_MN_HEADWATERS_READING";
const REPORT: &str = "EVENT_MN_HEADWATERS_REPORT";

// Resolve real grass encounters only on the newly authored park surface.
// Other map helpers keep their established traversal behavior.
fn walk_to(c: &mut VisibleShellController, runtime: &CrystalRuntime, target: (u16, u16)) {
    if c.snapshot().expect("actual walking map").overworld.map_name == "MnBemidji" {
        // A static maze path can cross an unrelated house door. Replan each
        // real street step using live actors and floor collision instead.
        super::super::northstar_ice_lair::walk(
            c,
            runtime,
            (
                i16::try_from(target.0).expect("native city x"),
                i16::try_from(target.1).expect("native city y"),
            ),
        );
        return;
    }
    if c.snapshot().expect("actual walking map").overworld.map_name != PARK {
        super::walk_to(c, runtime, target);
        return;
    }
    let module = super::module(c, runtime);
    let start = super::at(c);
    let course = super::path(runtime, &module, start, target).expect("current native park path");
    for button in course {
        let before = super::at(c);
        for _ in 0..8 {
            press(c, button);
            if let Some(battle) = c.snapshot().expect("native park encounter").battle {
                assert!(matches!(battle.kind, RuntimeBattleKind::Wild { .. }));
                flee(c);
            }
            if super::at(c) != before {
                break;
            }
        }
        c.wait_frames(80).expect("settled native park movement");
        if let Some(battle) = c.snapshot().expect("settled park encounter").battle {
            assert!(matches!(battle.kind, RuntimeBattleKind::Wild { .. }));
            flee(c);
        }
    }
    assert_eq!(super::at(c), target, "current park journey");
}

fn text(c: &mut VisibleShellController) -> String {
    c.snapshot()
        .expect("visible text")
        .ui
        .text
        .and_then(|t| t.asm_text)
        .unwrap_or_default()
}
fn road(c: &mut VisibleShellController, r: &CrystalRuntime, prefix: &'static str, to: (i16, i16)) {
    let start = position(c);
    let grid = RouteGrid::new_with_grass(
        r,
        RouteSpec {
            prefix,
            columns: 2,
            rows: 2,
            tileset: if prefix == "MnR09" {
                "mn_iron_range"
            } else {
                "mn_headwaters"
            },
            start,
            doors: BTreeMap::new(),
        },
    );
    for _ in 0..4096 {
        let before = position(c);
        if before == to {
            return;
        }
        // Native hops may land beyond a single quadrant. Plan each next input
        // from the observed landing instead of consuming a stale route suffix.
        let button = *grid.path(before, to).first().expect("remaining road step");
        for _ in 0..8 {
            press(c, button);
            if let Some(b) = c.snapshot().expect("road encounter").battle {
                if matches!(b.kind, RuntimeBattleKind::Wild { .. }) {
                    flee(c);
                } else {
                    battle(c, false);
                    finish(c);
                }
            }
            if position(c) != before {
                break;
            }
        }
        if position(c) == before {
            if let Some(directory) = std::env::var_os("CRYSTAL_WINONA_SAVE_DIR") {
                c.save(std::path::PathBuf::from(directory).join(format!(
                    "itasca-road-blocked-{}-{}-{}.crystalsave",
                    std::process::id(),
                    before.0,
                    before.1
                )))
                .expect("retain the actual blocked road state");
            }
            panic!(
                "native {prefix} road step {button:?} from {before:?} did not move; target={to:?}, ui={:?}",
                c.snapshot().expect("actual blocked road state").ui
            );
        }
    }
    panic!("bounded native {prefix} road did not reach {to:?}");
}
pub(super) fn from_bemidji(c: &mut VisibleShellController, r: &CrystalRuntime, lane: u16) {
    walk_to(c, r, (4, 24 + lane));
    travel(c, GameButton::Left, "MnR14C01R00", (31, 26 + lane as i16));
    road(c, r, "MnR14", (5, 42 + lane as i16));
    travel(c, GameButton::Left, GATE, (8, 4 + lane as i16));
    walk_to(c, r, (1, 4 + lane));
    travel(c, GameButton::Left, PARK, (42, 24 + lane as i16));
}
pub(super) fn to_bemidji(c: &mut VisibleShellController, r: &CrystalRuntime, lane: u16) {
    walk_to(c, r, (42, 24 + lane));
    travel(c, GameButton::Right, GATE, (1, 4 + lane as i16));
    walk_to(c, r, (8, 4 + lane));
    travel(c, GameButton::Right, "MnR14C00R00", (5, 42 + lane as i16));
    road(c, r, "MnR14", (95, 26 + lane as i16));
    travel(c, GameButton::Right, "MnBemidji", (0, 24 + lane as i16));
    travel(c, GameButton::Right, "MnBemidji", (4, 24 + lane as i16));
}
pub(super) fn marker(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk_to(c, r, (19, 22));
    press(c, GameButton::Up);
    press(c, GameButton::A);
}
fn officer(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk_to(c, r, (21, 20));
    press(c, GameButton::Up);
    press(c, GameButton::A);
}
pub(super) fn refusal(c: &mut VisibleShellController, right: bool, lane: u16, expected: &str) {
    let button = if right {
        GameButton::Right
    } else {
        GameButton::Left
    };
    for _ in 0..8 {
        if !text(c).is_empty() {
            break;
        }
        press(c, button);
    }
    assert!(text(c).contains(expected), "refusal: {}", text(c));
    press(c, button);
    at(
        c,
        "MnBemidji",
        (if right { 44 } else { 3 }, 24 + lane as i16),
    );
    finish(c);
    at(
        c,
        "MnBemidji",
        (if right { 43 } else { 4 }, 24 + lane as i16),
    );
}
pub(super) fn acknowledge_marker(c: &mut VisibleShellController, r: &CrystalRuntime) {
    marker(c, r);
    if r.data()
        .story_event_script_constants
        .global
        .contains_key("EVENT_MN_TAMARACK_INTRO")
    {
        super::super::northstar_dialogue::acknowledge(c, "MS06", "MN_ITASCA_MARKERText", READING);
        finish(c);
        assert!(event(c, READING) && !event(c, REPORT));
        return;
    }
    assert!(text(c).contains("HEADWATERS"));
    assert!(!event(c, READING) && !event(c, REPORT));
    press(c, GameButton::Left);
    at(c, PARK, (19, 22));
    assert!(!event(c, READING));
    finish(c);
    assert!(event(c, READING) && !event(c, REPORT));
}
pub(super) fn acknowledge_report(c: &mut VisibleShellController, r: &CrystalRuntime) {
    officer(c, r);
    if r.data()
        .story_event_script_constants
        .global
        .contains_key("EVENT_MN_TAMARACK_INTRO")
    {
        super::super::northstar_dialogue::acknowledge(c, "MS07", "MnBemidjiComparisonText", REPORT);
        finish(c);
        assert!(event(c, REPORT));
        return;
    }
    assert!(
        text(c).contains("One meter log.") || text(c).contains("The same ink blot."),
        "comparison page one: {}",
        text(c)
    );
    assert!(!event(c, REPORT));
    press(c, GameButton::Left);
    at(c, "MnBemidji", (21, 20));
    assert!(!event(c, REPORT));
    press(c, GameButton::A);
    assert!(!event(c, REPORT), "second page still owns input");
    assert!(
        text(c).contains("copied"),
        "comparison page two: {}",
        text(c)
    );
    finish(c);
    assert!(event(c, REPORT));
}

#[test]
#[ignore = "external Itasca pack; explicit prior badges/empty party"]
fn itasca_story_controller_native_gatehouse_both_lanes_free_returns_and_continue() {
    for lane in 0..2 {
        let (root, r, _, mut c) = fixture("MnR14C00R00", [5, 42 + lane], 0, true);
        travel(&mut c, GameButton::Left, GATE, (8, 4 + lane as i16));
        c = continued(c, &root, &r, "itasca-local-gate-east-arrival");
        at(&mut c, GATE, (8, 4 + lane as i16));
        // Returning through the same mouth must preserve the lane.
        travel(
            &mut c,
            GameButton::Right,
            "MnR14C00R00",
            (5, 42 + lane as i16),
        );
        travel(&mut c, GameButton::Left, GATE, (8, 4 + lane as i16));
        walk_to(&mut c, &r, (5, 4));
        press(&mut c, GameButton::Up);
        press(&mut c, GameButton::A);
        assert!(text(&mut c).contains("LAKE ITASCA"));
        finish(&mut c);
        walk_to(&mut c, &r, (1, 4 + lane));
        travel(&mut c, GameButton::Left, PARK, (42, 24 + lane as i16));
        c = continued(c, &root, &r, "itasca-local-park-arrival");
        travel(&mut c, GameButton::Right, GATE, (1, 4 + lane as i16));
        travel(&mut c, GameButton::Left, PARK, (42, 24 + lane as i16));
        marker(&mut c, &r);
        assert!(text(&mut c).contains("PINE"));
        finish(&mut c);
        assert!(!event(&c, READING) && !event(&c, REPORT));
        travel_to_gate(&mut c, &r, lane);
        walk_to(&mut c, &r, (8, 7));
        for _ in 0..4 {
            press(&mut c, GameButton::Down);
        }
        at(&mut c, GATE, (8, 7));
        walk_to(&mut c, &r, (8, 4 + lane));
        travel(
            &mut c,
            GameButton::Right,
            "MnR14C00R00",
            (5, 42 + lane as i16),
        );
    }
}
fn travel_to_gate(c: &mut VisibleShellController, r: &CrystalRuntime, lane: u16) {
    walk_to(c, r, (42, 24 + lane));
    travel(c, GameButton::Right, GATE, (1, 4 + lane as i16));
}

#[test]
#[ignore = "external Itasca pack; explicit prerequisite fixtures"]
fn itasca_story_controller_badges_report_notebook_and_free_inbound_checks() {
    for lane in 0..2 {
        let (_, _, _, mut c) = fixture("MnBemidji", [4, 24 + lane], 5, true);
        refusal(&mut c, false, lane, "LAKE");
        for (badges, read, report, expected) in [
            (6, false, false, "PINE"),
            (6, true, true, "PINE"),
            (7, true, false, "marker"),
        ] {
            let (_, _, _, mut c) = fixture("MnBemidji", [43, 24 + lane], badges, true);
            let f = &mut c.shell.shell.session_mut().state_mut().flags;
            f.set_event_flag(READING, read).expect("reading setup");
            f.set_event_flag(REPORT, report).expect("report setup");
            f.set_event_flag("EVENT_MN_PRAIRIE_NOTEBOOK", true)
                .expect("optional notebook setup");
            mark_runtime_snapshot_dirty(&mut c.shell);
            refusal(&mut c, true, lane, expected);
        }
        let (_, _, _, mut c) = fixture("MnR09C00R01", [0, 2 + lane], 0, true);
        travel(
            &mut c,
            GameButton::Left,
            "MnBemidji",
            (47, 24 + lane as i16),
        );
        travel(
            &mut c,
            GameButton::Left,
            "MnBemidji",
            (43, 24 + lane as i16),
        );
        let (_, _, _, mut c) = fixture("MnR14C01R00", [31, 26 + lane], 0, true);
        travel(
            &mut c,
            GameButton::Right,
            "MnBemidji",
            (0, 24 + lane as i16),
        );
        travel(
            &mut c,
            GameButton::Right,
            "MnBemidji",
            (4, 24 + lane as i16),
        );
    }
}

#[test]
#[ignore = "external Itasca pack; six/seven prior badges and empty party setup"]
fn itasca_story_controller_early_roundtrip_reading_comparison_and_iron_road() {
    let (root, r, _, mut c) = fixture("MnBemidji", [4, 24], 6, true);
    c = continued(c, &root, &r, "itasca-local-road-ready");
    from_bemidji(&mut c, &r, 0);
    marker(&mut c, &r);
    assert!(text(&mut c).contains("PINE"));
    finish(&mut c);
    to_bemidji(&mut c, &r, 1);
    officer(&mut c, &r);
    finish(&mut c);
    assert!(!event(&c, READING) && !event(&c, REPORT));
    let (root, r, _, mut c) = fixture("MnBemidji", [4, 24], 7, true);
    officer(&mut c, &r);
    finish(&mut c);
    assert!(!event(&c, REPORT));
    c = continued(c, &root, &r, "itasca-local-pine-road-ready");
    from_bemidji(&mut c, &r, 0);
    acknowledge_marker(&mut c, &r);
    c = continued(c, &root, &r, "itasca-local-reading");
    marker(&mut c, &r);
    assert!(text(&mut c).contains("Reference copied"));
    finish(&mut c);
    to_bemidji(&mut c, &r, 0);
    walk_to(&mut c, &r, (43, 24));
    refusal(&mut c, true, 0, "marker");
    acknowledge_report(&mut c, &r);
    let money = c.shell.shell.session().state().money;
    c = continued(c, &root, &r, "itasca-local-report");
    officer(&mut c, &r);
    assert!(text(&mut c).contains("IRON RANGE"));
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().money, money);
    for lane in 0..2 {
        walk_to(&mut c, &r, (43, 24 + lane));
        travel(
            &mut c,
            GameButton::Right,
            "MnR09C00R01",
            (0, 2 + lane as i16),
        );
        if lane == 0 {
            road(&mut c, &r, "MnR09", (127, 24));
            travel(&mut c, GameButton::Right, "MnGrandRapids", (0, 24));
            travel(&mut c, GameButton::Left, "MnR09C01R00", (63, 24));
            road(&mut c, &r, "MnR09", (0, 50));
        }
        travel(
            &mut c,
            GameButton::Left,
            "MnBemidji",
            (47, 24 + lane as i16),
        );
        travel(
            &mut c,
            GameButton::Left,
            "MnBemidji",
            (43, 24 + lane as i16),
        );
    }
    assert!(event(&c, READING) && event(&c, REPORT));
    assert!(!event(&c, "EVENT_MN_PRAIRIE_NOTEBOOK"));
}

#[test]
#[ignore = "external Itasca pack; optional notebook and prior reading fixture"]
fn itasca_story_controller_optional_notebook_recognition_grants_no_extra_progress() {
    let (_, r, _, mut c) = fixture("MnBemidji", [21, 20], 7, true);
    let state = c.shell.shell.session_mut().state_mut();
    state
        .flags
        .set_event_flag(READING, true)
        .expect("prior independent reading");
    state
        .flags
        .set_event_flag("EVENT_MN_PRAIRIE_NOTEBOOK", true)
        .expect("optional paper setup");
    let bag = state.bag.clone();
    let money = state.money;
    let badges = state.badges.clone();
    mark_runtime_snapshot_dirty(&mut c.shell);
    officer(&mut c, &r);
    assert!(text(&mut c).contains("One meter log.") || text(&mut c).contains("The same ink blot."));
    assert!(!event(&c, REPORT));
    press(&mut c, GameButton::A);
    assert!(
        text(&mut c).contains("These were copied.") || text(&mut c).contains("Separate basins.")
    );
    assert!(!event(&c, REPORT));
    press(&mut c, GameButton::A);
    assert!(event(&c, REPORT));
    assert!(text(&mut c).contains("MANKATO"));
    finish(&mut c);
    officer(&mut c, &r);
    assert!(text(&mut c).contains("IRON RANGE"));
    finish(&mut c);
    let state = c.shell.shell.session().state();
    assert_eq!(state.bag, bag);
    assert_eq!(state.money, money);
    assert_eq!(state.badges, badges);
}

#[test]
#[ignore = "local Mew45/six-prior-badge setup; Pine and chapter earned through input"]
fn itasca_story_controller_continuous_pine_loss_retry_marker_report_and_iron_entry() {
    let (_, r, mut c) = report_course();
    walk_to(&mut c, &r, (43, 25));
    travel(&mut c, GameButton::Right, "MnR09C00R01", (0, 3));
    travel(&mut c, GameButton::Left, "MnBemidji", (47, 25));
    travel(&mut c, GameButton::Left, "MnBemidji", (43, 25));
    assert!(event(&c, READING) && event(&c, REPORT));
}

pub(super) fn report_course() -> (
    crystal_assets::AssetRoot,
    CrystalRuntime,
    VisibleShellController,
) {
    let (root, r, mut c) = pine_course();
    super::leech_shore::heal(&mut c, &r, "MnBemidji", (31, 10), "MnBemidjiPokecenter1F");
    assert!(c.shell.shell.session().state().badges.johto[6]);
    assert!(!event(&c, READING) && !event(&c, REPORT));
    from_bemidji(&mut c, &r, 0);
    acknowledge_marker(&mut c, &r);
    c = continued(c, &root, &r, "itasca-local-earned-reading");
    to_bemidji(&mut c, &r, 1);
    walk_to(&mut c, &r, (43, 25));
    refusal(&mut c, true, 1, "marker");
    acknowledge_report(&mut c, &r);
    c = continued(c, &root, &r, "itasca-local-earned-report");
    (root, r, c)
}
