//! Earned intake containment through the three authored controls, without state writes.
use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::{VisibleShellController, choice, continue_in_place, event, finish, talk, walk};

pub(super) fn present(r: &CrystalRuntime) -> bool {
    r.data()
        .maps
        .get("MnBoundaryIntake")
        .is_some_and(|room| room.scripts.contains_key("MnVerdigrimBypass"))
}

pub(super) fn restore(c: &mut VisibleShellController, root: &AssetRoot, r: &CrystalRuntime) {
    assert!(event(c, "EVENT_MN_VERDIGRIM_EXPOSED"));
    assert!(event(c, "EVENT_MN_VERDIGRIM_COUNTER_CURRENT"));
    assert!(event(c, "EVENT_MN_REED_BOUNDARY_DEFEATED"));
    for (at, flag, stage) in [
        (
            (17, 12),
            "EVENT_MN_VERDIGRIM_DRILL_STOPPED",
            "campaign-intake-drill-stopped",
        ),
        (
            (21, 20),
            "EVENT_MN_VERDIGRIM_OUTLET_CLOSED",
            "campaign-intake-outlet-closed",
        ),
        (
            (13, 12),
            "EVENT_MN_VERDIGRIM_CLEAN_BYPASS",
            "campaign-intake-clean-bypass",
        ),
    ] {
        if event(c, flag) {
            continue;
        }
        assert!(!event(c, "EVENT_MN_SURVEY_RESTORED"));
        walk(c, r, at);
        talk(c, GameButton::Up);
        choice(c, false);
        finish(c);
        assert!(
            !event(c, flag),
            "refusal must retain the current control: {flag}"
        );
        talk(c, GameButton::Up);
        choice(c, true);
        finish(c);
        assert!(event(c, flag), "acknowledged native control: {flag}");
        continue_in_place(c, root, r, stage);
        assert!(
            event(c, flag),
            "control persists through actual Continue: {flag}"
        );
    }
    for flag in [
        "EVENT_MN_VERDIGRIM_EXPOSED",
        "EVENT_MN_VERDIGRIM_DRILL_STOPPED",
        "EVENT_MN_VERDIGRIM_OUTLET_CLOSED",
        "EVENT_MN_VERDIGRIM_CLEAN_BYPASS",
        "EVENT_MN_VERDIGRIM_CONTAINED",
        "EVENT_MN_KETTLURK_RELEASED",
        "EVENT_MN_SURVEY_RESTORED",
    ] {
        assert!(event(c, flag), "earned intake containment: {flag}");
    }
}
