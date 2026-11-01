//! The original clock's introductory text must advance and remain readable.
use bevy::prelude::{Assets, Image};

use super::super::{
    GAME_TICK_SECONDS, RuntimeTickTimer, SOURCE_TILE_SIZE, TIME_SET_TEXTBOX_Y, VisibleTimeSetNext,
    VisibleTimeSetPhase, load_time_set_frame, open_visible_time_set_screen, shell_render_key,
};
use super::{core_modular_title_shell_for_test, integrated_shell_test_app};

#[test]
#[ignore = "requires the external core text pack and native font/window art"]
fn overhaul_clock_idle_frames_finish_the_wake_text_at_the_production_tick_rate() {
    let mut shell = core_modular_title_shell_for_test();
    shell.splash_screen = None;
    shell.intro_screen = None;
    shell.title_menu = None;
    open_visible_time_set_screen(&mut shell, VisibleTimeSetNext::OakIntro)
        .expect("native clock entry");
    let mut app = integrated_shell_test_app(shell);
    app.insert_resource(RuntimeTickTimer::new(f64::from(GAME_TICK_SECONDS)));
    for _ in 0..300 {
        app.update();
    }
    let shell = app.world().resource::<super::super::BevyRuntimeShell>();
    assert!(
        shell.last_error.is_none(),
        "clock render error: {:?}",
        shell.last_error
    );
    let clock = shell
        .pending_time_set
        .as_ref()
        .expect("clock still owns input");
    assert_eq!(clock.phase, VisibleTimeSetPhase::WakeDialogue);
    let page = &clock.wake_pages[0];
    assert_eq!(
        clock.visible_chars,
        page.chars().count(),
        "idle clock text: {clock:?}"
    );
    let rendered = app.world().resource::<super::super::RenderedViewport>();
    assert_eq!(rendered.shell_render_key, Some(shell_render_key(shell)));
}

#[test]
#[ignore = "requires the external core text pack and native font/window art"]
fn overhaul_clock_sleep_ellipses_have_visible_native_glyph_pixels() {
    let mut shell = core_modular_title_shell_for_test();
    open_visible_time_set_screen(&mut shell, VisibleTimeSetNext::OakIntro).expect("native clock");
    let clock = shell.pending_time_set.as_mut().expect("clock text");
    clock.phase = VisibleTimeSetPhase::WakeDialogue;
    let mut images = Assets::<Image>::default();
    assert_eq!(TIME_SET_TEXTBOX_Y * SOURCE_TILE_SIZE, 96);
    for index in 0..clock.wake_pages.len() {
        clock.wake_index = index;
        clock.visible_chars = clock.wake_pages[index].chars().count();
        let frame = load_time_set_frame(&shell.asset_root, clock, &mut images)
            .expect("native clock pixels");
        let image = images.get(&frame.handle).expect("clock image");
        let pixels = image.data.as_ref().expect("CPU pixels");
        let dark = (104..136)
            .flat_map(|y| (8..152).map(move |x| (y * 160 + x) * 4))
            .filter(|index| pixels[*index..*index + 3].iter().all(|value| *value < 32))
            .count();
        assert!(
            dark >= 12,
            "the wake page cannot be blank: {:?}; dark={dark}",
            clock.wake_pages[index]
        );
    }
}
