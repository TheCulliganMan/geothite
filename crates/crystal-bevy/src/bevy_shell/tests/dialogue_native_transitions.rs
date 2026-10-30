//! Autonomous scene continuations must replace the rendered core text snapshot.
use std::path::PathBuf;

use bevy::prelude::{ButtonInput, Entity, KeyCode, With};
use crystal_core::input::GameButton;

use super::super::{
    BevyRuntimeShell, DialogGlyphMarker, GAME_TICK_SECONDS, RuntimeTickTimer,
    SceneDialogTextBoxBackgroundMarker, VisibleShellController, cached_runtime_snapshot,
    visible_field_dialog_pages, visible_field_dialogue_is_fully_revealed,
};
use super::integrated_shell_test_app;
use super::winona::{load, press};

#[test]
#[ignore = "requires the matching external pack and CRYSTAL_STORY_HARBOR_REVIEW_SAVE"]
fn overhaul_native_harbor_pause_reopens_a_visible_fresh_textbox_without_input() {
    let (root, runtime) = load();
    let save = PathBuf::from(
        std::env::var_os("CRYSTAL_STORY_HARBOR_REVIEW_SAVE")
            .expect("actual same-pack harbor approach checkpoint"),
    );
    let mut c = VisibleShellController::load_save(root, runtime, save, None)
        .expect("native harbor checkpoint");
    c.set_runtime_journal_enabled(false);
    let initial = c.snapshot().expect("actual harbor approach");
    assert_eq!(initial.overworld.map_name, "MnDuluthHarborWorkshop");
    assert_eq!(
        (initial.overworld.tile.x, initial.overworld.tile.y),
        (10, 5)
    );
    press(&mut c, GameButton::Right);
    press(&mut c, GameButton::A);
    let source = c.shell.shell.presentation_snapshot().expect("first speech");
    assert_eq!(
        source.ui.text.as_ref().expect("first speech").label,
        "MN_HARBOR_RECEIVERText"
    );
    let pages = visible_field_dialog_pages(&source, &c.shell).expect("first authored pages");
    for _ in 1..pages.len() {
        press(&mut c, GameButton::A);
    }
    assert_eq!(
        c.shell
            .field_text_reveal
            .as_ref()
            .expect("last page")
            .page_index
            + 1,
        pages.len()
    );

    let mut app = integrated_shell_test_app(c.shell);
    app.insert_resource(RuntimeTickTimer::new(f64::from(GAME_TICK_SECONDS)));
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyZ);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::KeyZ);
    // This minimal app omits InputPlugin's per-frame edge clearing. A released
    // key must not remain a fresh A press throughout the unheld continuation.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear_just_pressed(KeyCode::KeyZ);
    app.update();
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert!(shell.visible_script_delay_frames.is_some());
        let closed = cached_runtime_snapshot(&mut shell).expect("rendered pause snapshot");
        assert!(
            !closed.ui.text_window_open,
            "the authored pause closes the box"
        );
    }

    let expected = "MnOverhaulHarborEvidenceText";
    let mut ready = false;
    for _ in 0..512 {
        app.update();
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert!(shell.last_error.is_none(), "{:?}", shell.last_error);
        let live = shell
            .shell
            .presentation_snapshot()
            .expect("native continuation");
        if live
            .ui
            .text
            .as_ref()
            .is_some_and(|text| text.label == expected)
            && visible_field_dialogue_is_fully_revealed(shell, &live)
        {
            ready = true;
            break;
        }
    }
    assert!(
        ready,
        "the next speech prints through ordinary unheld frames"
    );
    app.update();
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        let cached = cached_runtime_snapshot(&mut shell).expect("renderer snapshot");
        assert!(cached.ui.text_window_open);
        assert_eq!(
            cached.ui.text.as_ref().expect("rendered new speech").label,
            expected
        );
        assert_eq!(
            shell
                .field_text_reveal
                .as_ref()
                .expect("first new page")
                .page_index,
            0
        );
        assert!(
            !shell
                .shell
                .session()
                .state()
                .flags
                .is_script_flag_set("EVENT_MN_LIGHTHOUSE_SIGNAL")
                .expect("harbor reward identity"),
            "later evidence and directions still need their acknowledgments"
        );
    }
    let mut boxes = app
        .world_mut()
        .query_filtered::<Entity, With<SceneDialogTextBoxBackgroundMarker>>();
    assert_eq!(boxes.iter(app.world()).count(), 1);
    let mut glyphs = app
        .world_mut()
        .query_filtered::<Entity, With<DialogGlyphMarker>>();
    assert!(
        glyphs.iter(app.world()).count() > 0,
        "the new page has rendered glyphs"
    );
}
