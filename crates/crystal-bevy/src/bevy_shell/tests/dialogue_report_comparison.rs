//! Inspect the late comparison and reaction on the actual 2D frame schedule.
use bevy::prelude::{App, ButtonInput, KeyCode};
use crystal_core::input::GameButton;
use crystal_core::world::map::Direction;

use super::super::super::{BevyRuntimeShell, VisibleShellController, mark_runtime_snapshot_dirty};
use super::super::integrated_shell_test_app;
use super::super::northstar_dialogue::canonical_pages;
use super::super::northstar_ice_lair::event;
use super::super::winona::{at, continue_in_place, finish, press};
use super::fixture;

const REPORT: &str = "EVENT_MN_HEADWATERS_REPORT";
const OFFICER: &str = "MN_BEMIDJI_OFFICER";

fn capture(c: &mut VisibleShellController, stage: &str) {
    let directory = std::path::PathBuf::from(
        std::env::var_os("CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR")
            .expect("isolated comparison captures"),
    );
    std::fs::create_dir_all(&directory).expect("comparison capture directory");
    c.save(directory.join(format!("officer-{stage}.crystalsave")))
        .expect("actual comparison scene");
}

fn frame(app: &mut App, key: Option<KeyCode>) {
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.reset_all();
    if let Some(key) = key {
        keys.press(key);
    }
    app.update();
    assert_eq!(app.world().resource::<BevyRuntimeShell>().last_error, None);
}

#[test]
#[ignore = "declared Pine badge and Itasca notes; requires the staged comparison candidate"]
fn overhaul_officer_inspects_records_and_reacts_before_granting_report() {
    let (root, r, mut c) = fixture("MnBemidji", (21, 20), &["EVENT_MN_HEADWATERS_READING"]);
    let state = c.shell.shell.session_mut().state_mut();
    state.badges.johto[6] = true;
    state
        .flags
        .set_engine_flag("ENGINE_GLACIERBADGE", true)
        .expect("declared Pine badge");
    mark_runtime_snapshot_dirty(&mut c.shell);
    press(&mut c, GameButton::Up);
    assert!(!event(&c, REPORT));
    capture(&mut c, "before");
    let mut app = integrated_shell_test_app(c.shell);
    let mut inspection = false;
    for index in 0..180 {
        frame(
            &mut app,
            match index {
                0 => Some(KeyCode::KeyZ),
                1 => Some(KeyCode::ArrowLeft),
                _ => None,
            },
        );
        let shell = app.world().resource::<BevyRuntimeShell>();
        if let Some(emote) = &shell.visible_overworld_emote {
            assert_eq!(emote.object, OFFICER);
            assert_eq!(emote.emote, "EMOTE_QUESTION");
            inspection = true;
        }
        assert!(
            !shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(REPORT)
                .expect("unread report")
        );
    }
    assert!(
        inspection,
        "the officer's inspection must occupy visible frames"
    );
    let mut c = VisibleShellController {
        shell: app
            .world_mut()
            .remove_resource::<BevyRuntimeShell>()
            .expect("native 2D scene"),
    };
    at(&mut c, "MnBemidji", (21, 20));
    capture(&mut c, "comparison");
    let pages = canonical_pages("MS07", "MnBemidjiComparisonText");
    for (index, page) in pages.iter().enumerate() {
        let snapshot = c.snapshot().expect("officer's comparison page");
        assert_eq!(snapshot.visible_object_runtime_tiles[OFFICER].x, 21);
        assert_eq!(snapshot.visible_object_runtime_tiles[OFFICER].y, 19);
        assert_eq!(snapshot.visible_object_facings[OFFICER], Direction::Down);
        assert_eq!(
            snapshot.ui.text.expect("comparison").asm_text.as_ref(),
            Some(page)
        );
        assert!(!event(&c, REPORT));
        if index + 1 < pages.len() {
            press(&mut c, GameButton::A);
        }
    }
    let mut app = integrated_shell_test_app(c.shell);
    let mut reaction = false;
    for index in 0..120 {
        frame(
            &mut app,
            match index {
                0 => Some(KeyCode::KeyZ),
                1 => Some(KeyCode::ArrowLeft),
                _ => None,
            },
        );
        let shell = app.world().resource::<BevyRuntimeShell>();
        if let Some(emote) = &shell.visible_overworld_emote {
            assert_eq!(emote.object, OFFICER);
            assert_eq!(emote.emote, "EMOTE_SHOCK");
            assert!(
                !shell
                    .shell
                    .session()
                    .state()
                    .flags
                    .is_event_flag_set(REPORT)
                    .expect("reaction precedes report")
            );
            reaction = true;
        }
    }
    assert!(
        reaction,
        "the officer must visibly react before granting the report"
    );
    let mut c = VisibleShellController {
        shell: app
            .world_mut()
            .remove_resource::<BevyRuntimeShell>()
            .expect("report scene"),
    };
    at(&mut c, "MnBemidji", (21, 20));
    finish(&mut c);
    assert!(event(&c, REPORT));
    assert!(!event(&c, "EVENT_MN_PRAIRIE_NOTEBOOK"));
    capture(&mut c, "after");
    continue_in_place(
        &mut c,
        &root,
        &r,
        "overhaul-officer-report-without-optional-notebook",
    );
    assert!(event(&c, REPORT));
}
