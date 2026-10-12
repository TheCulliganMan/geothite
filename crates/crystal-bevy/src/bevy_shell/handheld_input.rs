//! Native controllers feed the same input-owner loops as the keyboard.
use super::*;
use std::collections::HashSet;

const DIRECTIONS: [KeyCode; 4] = [
    KeyCode::ArrowUp,
    KeyCode::ArrowDown,
    KeyCode::ArrowLeft,
    KeyCode::ArrowRight,
];
const STICK_PRESS: f32 = 0.55;
const STICK_RELEASE: f32 = 0.35;

#[derive(Resource, Default)]
pub(super) struct HandheldInput {
    pub(super) keys: ButtonInput<KeyCode>,
    controller_keys: ButtonInput<KeyCode>,
    active: Option<Entity>,
    direction: Option<KeyCode>,
    unfocused: bool,
    wait_for_neutral: bool,
}

pub(super) fn install(app: &mut App) {
    app.init_resource::<HandheldInput>()
        .add_systems(
            Update,
            gather_input
                .after(release_input_on_focus_loss)
                .after(apply_save_management)
                .after(apply_player_customization)
                .after(poll_multiplayer)
                .before(apply_keyboard_input),
        )
        .add_systems(Update, toggle_fullscreen.after(gather_input));
    #[cfg(feature = "fullscreen-scaling")]
    app.add_systems(Update, fit_initial_window.before(toggle_fullscreen));
    #[cfg(feature = "voxel-view")]
    app.add_systems(Update, controller_camera.after(gather_input));
}

fn stick_direction(x: f32, y: f32, previous: Option<KeyCode>) -> Option<KeyCode> {
    if !x.is_finite() || !y.is_finite() {
        return None;
    }
    if let Some(previous) = previous {
        let component = match previous {
            KeyCode::ArrowRight => x,
            KeyCode::ArrowLeft => -x,
            KeyCode::ArrowUp => y,
            KeyCode::ArrowDown => -y,
            _ => 0.0,
        };
        if component >= STICK_RELEASE && x.abs().max(y.abs()) <= component + 0.15 {
            return Some(previous);
        }
    }
    let horizontal = if x < 0.0 {
        KeyCode::ArrowLeft
    } else {
        KeyCode::ArrowRight
    };
    let vertical = if y < 0.0 {
        KeyCode::ArrowDown
    } else {
        KeyCode::ArrowUp
    };
    if x.abs().max(y.abs()) >= STICK_PRESS {
        return Some(if x.abs() == y.abs() && previous == Some(horizontal) {
            horizontal
        } else if y.abs() >= x.abs() {
            vertical
        } else {
            horizontal
        });
    }
    previous.filter(|key| match key {
        KeyCode::ArrowRight => x >= STICK_RELEASE,
        KeyCode::ArrowLeft => -x >= STICK_RELEASE,
        KeyCode::ArrowUp => y >= STICK_RELEASE,
        KeyCode::ArrowDown => -y >= STICK_RELEASE,
        _ => false,
    })
}

fn sample_pad(pad: &Gamepad, previous: Option<KeyCode>) -> Vec<KeyCode> {
    let down = |button| pad.pressed(button) || pad.just_pressed(button);
    let mut result = Vec::new();
    for (button, key) in [
        (GamepadButton::South, KeyCode::KeyZ),
        (GamepadButton::East, KeyCode::KeyX),
        (GamepadButton::Start, KeyCode::Enter),
        (GamepadButton::Select, KeyCode::ShiftRight),
        (GamepadButton::West, KeyCode::F11),
        (GamepadButton::North, KeyCode::F3),
        (GamepadButton::LeftTrigger, KeyCode::KeyQ),
        (GamepadButton::RightTrigger, KeyCode::KeyE),
        (GamepadButton::LeftTrigger2, KeyCode::PageDown),
        (GamepadButton::RightTrigger2, KeyCode::PageUp),
    ] {
        if down(button) {
            result.push(key);
        }
    }
    let mut digital = Vec::new();
    for (first, second, first_key, second_key) in [
        (
            GamepadButton::DPadUp,
            GamepadButton::DPadDown,
            KeyCode::ArrowUp,
            KeyCode::ArrowDown,
        ),
        (
            GamepadButton::DPadLeft,
            GamepadButton::DPadRight,
            KeyCode::ArrowLeft,
            KeyCode::ArrowRight,
        ),
    ] {
        if down(first) != down(second) {
            digital.push(if down(first) { first_key } else { second_key });
        }
    }
    let dpad_held = [
        GamepadButton::DPadUp,
        GamepadButton::DPadDown,
        GamepadButton::DPadLeft,
        GamepadButton::DPadRight,
    ]
    .into_iter()
    .any(down);
    let direction = if dpad_held {
        previous
            .filter(|key| digital.contains(key))
            .or_else(|| digital.first().copied())
    } else {
        stick_direction(
            pad.get(GamepadAxis::LeftStickX).unwrap_or(0.0),
            pad.get(GamepadAxis::LeftStickY).unwrap_or(0.0),
            previous,
        )
    };
    if let Some(direction) = direction {
        result.push(direction);
    }
    result
}

impl HandheldInput {
    fn update<'a>(
        &mut self,
        keyboard: &ButtonInput<KeyCode>,
        pads: impl IntoIterator<Item = (Entity, &'a Gamepad)>,
        focus: Option<bool>,
        blocked: bool,
    ) {
        if let Some(focused) = focus {
            self.unfocused = !focused;
            if !focused {
                self.wait_for_neutral = true;
                self.direction = None;
            }
        }
        let mut pads = pads.into_iter().collect::<Vec<_>>();
        pads.sort_by_key(|(id, _)| id.to_bits());
        if !self
            .active
            .is_some_and(|id| pads.iter().any(|(pad, _)| *pad == id))
        {
            self.active = None;
            self.direction = None;
        }
        let current = pads
            .iter()
            .find(|(id, _)| Some(*id) == self.active)
            .map(|(_, pad)| sample_pad(pad, self.direction))
            .unwrap_or_default();
        // Keep a held controller; an idle built-in pad must not steal a docked controller.
        if current.is_empty() {
            if let Some((id, _)) = pads
                .iter()
                .find(|(_, pad)| !sample_pad(pad, None).is_empty())
            {
                if self.active != Some(*id) {
                    self.direction = None;
                }
                self.active = Some(*id);
            }
        }
        let mut mapped = pads
            .iter()
            .find(|(id, _)| Some(*id) == self.active)
            .map(|(_, pad)| sample_pad(pad, self.direction))
            .unwrap_or_default();
        self.direction = mapped.iter().find(|key| DIRECTIONS.contains(key)).copied();
        if self.wait_for_neutral {
            // Require every connected pad to be released after an overlay/suspend.
            let neutral = pads
                .iter()
                .all(|(_, pad)| sample_pad(pad, self.direction).is_empty());
            if neutral && !self.unfocused && !blocked {
                self.wait_for_neutral = false;
            }
            mapped.clear();
        }
        if self.unfocused || blocked {
            self.wait_for_neutral = true;
            self.keys.reset_all();
            self.controller_keys.reset_all();
            return;
        }
        self.controller_keys.clear();
        let controller: HashSet<_> = mapped.iter().copied().collect();
        for key in self
            .controller_keys
            .get_pressed()
            .copied()
            .collect::<Vec<_>>()
        {
            if !controller.contains(&key) {
                self.controller_keys.release(key);
            }
        }
        for key in controller {
            self.controller_keys.press(key);
        }
        // A physical key and its controller equivalent are one logical held input.
        let mut desired: HashSet<_> = keyboard.get_pressed().copied().collect();
        let keyboard_direction = DIRECTIONS.iter().any(|key| keyboard.pressed(*key));
        desired.extend(
            mapped
                .into_iter()
                .filter(|key| !keyboard_direction || !DIRECTIONS.contains(key)),
        );
        self.keys.clear();
        for key in self.keys.get_pressed().copied().collect::<Vec<_>>() {
            if !desired.contains(&key) {
                self.keys.release(key);
            }
        }
        for key in desired {
            self.keys.press(key);
        }
        // Preserve physical taps whose press and release arrived in one frame.
        for key in keyboard.get_just_pressed().copied() {
            if !keyboard.pressed(key) && !self.keys.pressed(key) {
                self.keys.press(key);
                self.keys.release(key);
            }
        }
    }
}

pub(super) fn gather_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    pads: Query<(Entity, &Gamepad)>,
    mut focus: MessageReader<WindowFocused>,
    mut input: ResMut<HandheldInput>,
) {
    let events = focus.read().map(|event| event.focused).collect::<Vec<_>>();
    if events.iter().any(|focused| !focused) {
        input.wait_for_neutral = true;
        input.direction = None;
    }
    let focused = events.last().copied();
    input.update(
        &keyboard,
        pads.iter(),
        focused,
        customization_is_open() || save_manager_is_open(),
    );
}

#[cfg(feature = "fullscreen-scaling")]
fn fit_initial_window(
    mut fitted: Local<bool>,
    monitors: Query<&bevy::window::Monitor, With<bevy::window::PrimaryMonitor>>,
    mut windows: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
) {
    if *fitted {
        return;
    }
    let (Ok(monitor), Ok(mut window)) = (monitors.single(), windows.single_mut()) else {
        return;
    };
    if window.mode != bevy::window::WindowMode::Windowed {
        *fitted = true;
        return;
    }
    if monitor.physical_width == 0 || monitor.physical_height == 0 {
        return;
    }
    let dpi = window
        .resolution
        .scale_factor_override()
        .unwrap_or(monitor.scale_factor as f32);
    if !dpi.is_finite() || dpi <= 0.0 {
        return;
    }
    // Leave room for native decorations and the desktop taskbar on high-DPI handhelds.
    let available =
        (monitor.physical_size().as_vec2() / dpi - Vec2::new(48.0, 96.0)).max(Vec2::ONE);
    let requested = Vec2::new(window.width(), window.height());
    let size = requested.min(available);
    if size != requested {
        window.resolution.set(size.x, size.y);
    }
    *fitted = true;
}

fn toggle_fullscreen(
    input: Res<HandheldInput>,
    mut windows: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
) {
    if !input.keys.just_pressed(KeyCode::F11) {
        return;
    }
    if let Ok(mut window) = windows.single_mut() {
        window.mode = if window.mode == bevy::window::WindowMode::Windowed {
            bevy::window::WindowMode::BorderlessFullscreen(bevy::window::MonitorSelection::Current)
        } else {
            bevy::window::WindowMode::Windowed
        };
    }
}

#[cfg(feature = "voxel-view")]
fn controller_camera(
    input: Res<HandheldInput>,
    keyboard: Res<ButtonInput<KeyCode>>,
    frame: Option<Res<crystal_render_api::VisualWorldFrame>>,
    settings: Option<ResMut<crystal_voxel_view::VoxelViewSettings>>,
) {
    let (Some(frame), Some(mut settings)) = (frame, settings) else {
        return;
    };
    // Steam Input can expose both a native button and its keyboard alias.
    let pressed = |key| {
        input.controller_keys.just_pressed(key)
            && !keyboard.pressed(key)
            && !keyboard.just_pressed(key)
    };
    if settings.allow_f3_toggle && pressed(KeyCode::F3) {
        settings.enabled = !settings.enabled;
    }
    if !frame.active || !settings.enabled {
        return;
    }
    let mut camera = settings.camera;
    // Only controller edges: the voxel plugin already handles physical keyboard edges.
    // Handheld shoulders are not synthesized into the physical keyboard resource.
    if pressed(KeyCode::KeyQ) {
        camera.rotation_step -= 1.0;
    }
    if pressed(KeyCode::KeyE) {
        camera.rotation_step += 1.0;
    }
    if pressed(KeyCode::PageUp) {
        camera.zoom_step += 1.0;
    }
    if pressed(KeyCode::PageDown) {
        camera.zoom_step -= 1.0;
    }
    settings.camera =
        crystal_voxel_view::VoxelCameraControls::new(camera.zoom_step, camera.rotation_step);
}

#[cfg(test)]
mod tests {
    use super::*;
    fn id(index: u32) -> Entity {
        Entity::from_raw_u32(index).unwrap()
    }

    #[test]
    fn handheld_standard_buttons_dpad_precedence_and_running_chord() {
        let mut pad = Gamepad::default();
        for button in [
            GamepadButton::South,
            GamepadButton::East,
            GamepadButton::Start,
            GamepadButton::Select,
            GamepadButton::DPadRight,
        ] {
            pad.digital_mut().press(button);
        }
        pad.analog_mut().set(GamepadAxis::LeftStickY, 1.0);
        let sample = sample_pad(&pad, None);
        for key in [
            KeyCode::KeyZ,
            KeyCode::KeyX,
            KeyCode::Enter,
            KeyCode::ShiftRight,
            KeyCode::ArrowRight,
        ] {
            assert!(sample.contains(&key));
        }
        assert!(!sample.contains(&KeyCode::ArrowUp));
        pad.digital_mut().press(GamepadButton::DPadLeft);
        assert!(
            !sample_pad(&pad, None)
                .iter()
                .any(|key| DIRECTIONS.contains(key))
        );
    }

    #[test]
    fn handheld_stick_deadzone_hysteresis_diagonals_and_invalid_samples() {
        assert_eq!(stick_direction(0.3, 0.1, None), None);
        assert_eq!(stick_direction(0.8, 0.7, None), Some(KeyCode::ArrowRight));
        assert_eq!(
            stick_direction(0.4, 0.1, Some(KeyCode::ArrowRight)),
            Some(KeyCode::ArrowRight)
        );
        assert_eq!(stick_direction(0.3, 0.1, Some(KeyCode::ArrowRight)), None);
        assert_eq!(stick_direction(-0.8, 0.1, None), Some(KeyCode::ArrowLeft));
        assert_eq!(stick_direction(0.1, -0.8, None), Some(KeyCode::ArrowDown));
        assert_eq!(
            stick_direction(0.8, 0.8, Some(KeyCode::ArrowRight)),
            Some(KeyCode::ArrowRight)
        );
        assert_eq!(
            stick_direction(0.7, 0.71, Some(KeyCode::ArrowRight)),
            Some(KeyCode::ArrowRight)
        );
        assert_eq!(
            stick_direction(0.3, 0.8, Some(KeyCode::ArrowRight)),
            Some(KeyCode::ArrowUp)
        );
        assert_eq!(stick_direction(f32::NAN, 1.0, None), None);
        assert_eq!(stick_direction(1.0, f32::INFINITY, None), None);
    }

    #[test]
    fn handheld_keyboard_and_pad_share_edges_without_overwriting_keyboard() {
        let mut keyboard = ButtonInput::default();
        let mut pad = Gamepad::default();
        let mut input = HandheldInput::default();
        pad.digital_mut().press(GamepadButton::South);
        input.update(&keyboard, [(id(0), &pad)], None, false);
        assert!(input.keys.just_pressed(KeyCode::KeyZ));
        input.update(&keyboard, [(id(0), &pad)], None, false);
        assert!(!input.keys.just_pressed(KeyCode::KeyZ));
        keyboard.press(KeyCode::KeyZ);
        input.update(&keyboard, [(id(0), &pad)], None, false);
        assert!(!input.keys.just_pressed(KeyCode::KeyZ));
        keyboard.clear();
        pad.digital_mut().reset_all();
        input.update(&keyboard, [(id(0), &pad)], None, false);
        assert!(input.keys.pressed(KeyCode::KeyZ));
        assert!(!input.keys.just_released(KeyCode::KeyZ));
        assert!(keyboard.pressed(KeyCode::KeyZ));
        keyboard.release(KeyCode::KeyZ);
        input.update(&keyboard, [(id(0), &pad)], None, false);
        assert!(input.keys.just_released(KeyCode::KeyZ));
    }

    #[test]
    fn handheld_preserves_keyboard_and_controller_taps_completed_within_one_frame() {
        let mut keyboard = ButtonInput::default();
        let mut input = HandheldInput::default();
        keyboard.press(KeyCode::KeyZ);
        keyboard.release(KeyCode::KeyZ);
        input.update(&keyboard, [], None, false);
        assert!(input.keys.just_pressed(KeyCode::KeyZ));
        assert!(input.keys.just_released(KeyCode::KeyZ));
        assert!(!input.keys.pressed(KeyCode::KeyZ));
        keyboard.clear();
        input.update(&keyboard, [], None, false);
        assert!(!input.keys.just_pressed(KeyCode::KeyZ));
        let mut pad = Gamepad::default();
        pad.digital_mut().press(GamepadButton::South);
        pad.digital_mut().release(GamepadButton::South);
        input.update(&keyboard, [(id(0), &pad)], None, false);
        assert!(input.keys.just_pressed(KeyCode::KeyZ));
        pad.digital_mut().clear();
        input.update(&keyboard, [(id(0), &pad)], None, false);
        assert!(input.keys.just_released(KeyCode::KeyZ));
        assert!(!input.keys.pressed(KeyCode::KeyZ));
    }

    #[test]
    fn handheld_disconnect_and_docked_controller_handoff_release_inputs() {
        let keyboard = ButtonInput::default();
        let mut built_in = Gamepad::default();
        let mut external = Gamepad::default();
        let mut input = HandheldInput::default();
        built_in.digital_mut().press(GamepadButton::East);
        built_in.digital_mut().press(GamepadButton::DPadRight);
        input.update(
            &keyboard,
            [(id(0), &built_in), (id(1), &external)],
            None,
            false,
        );
        assert_eq!(input.active, Some(id(0)));
        input.update(&keyboard, [(id(1), &external)], None, false);
        assert!(input.keys.just_released(KeyCode::KeyX));
        assert!(input.keys.just_released(KeyCode::ArrowRight));
        external.digital_mut().press(GamepadButton::DPadUp);
        input.update(
            &keyboard,
            [(id(0), &Gamepad::default()), (id(1), &external)],
            None,
            false,
        );
        assert_eq!(input.active, Some(id(1)));
        assert!(input.keys.just_pressed(KeyCode::ArrowUp));
    }

    #[test]
    fn handheld_focus_and_external_overlay_require_release_before_resuming() {
        for external_overlay in [false, true] {
            let keyboard = ButtonInput::default();
            let mut pad = Gamepad::default();
            let mut input = HandheldInput::default();
            pad.digital_mut().press(GamepadButton::East);
            input.update(&keyboard, [(id(0), &pad)], None, false);
            input.update(
                &keyboard,
                [(id(0), &pad)],
                Some(external_overlay),
                external_overlay,
            );
            assert!(!input.keys.pressed(KeyCode::KeyX));
            input.update(&keyboard, [(id(0), &pad)], Some(true), false);
            assert!(!input.keys.pressed(KeyCode::KeyX));
            pad.digital_mut().reset_all();
            input.update(&keyboard, [(id(0), &pad)], None, false);
            pad.digital_mut().press(GamepadButton::East);
            input.update(&keyboard, [(id(0), &pad)], None, false);
            assert!(input.keys.just_pressed(KeyCode::KeyX));
        }
    }

    #[test]
    fn handheld_connection_messages_feed_production_input_and_release_on_disconnect() {
        use bevy::input::gamepad::{GamepadConnection, GamepadConnectionEvent};
        let mut app = App::new();
        app.add_plugins(bevy::input::InputPlugin)
            .add_message::<WindowFocused>();
        install(&mut app);
        let pad = app.world_mut().spawn_empty().id();
        app.world_mut().write_message(GamepadConnectionEvent::new(
            pad,
            GamepadConnection::Connected {
                name: "Steam Deck virtual gamepad".into(),
                vendor_id: None,
                product_id: None,
            },
        ));
        app.update();
        let mut gamepad = app.world_mut().get_mut::<Gamepad>(pad).unwrap();
        gamepad.digital_mut().press(GamepadButton::South);
        gamepad.digital_mut().press(GamepadButton::DPadRight);
        app.update();
        let input = app.world().resource::<HandheldInput>();
        assert!(input.keys.just_pressed(KeyCode::KeyZ));
        assert!(input.keys.just_pressed(KeyCode::ArrowRight));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyZ);
        app.update();
        assert!(
            !app.world()
                .resource::<HandheldInput>()
                .keys
                .just_pressed(KeyCode::KeyZ)
        );
        app.world_mut().write_message(GamepadConnectionEvent::new(
            pad,
            GamepadConnection::Disconnected,
        ));
        app.update();
        assert!(app.world().get::<Gamepad>(pad).is_none());
        let input = app.world().resource::<HandheldInput>();
        assert!(input.keys.just_released(KeyCode::ArrowRight));
        assert!(input.keys.pressed(KeyCode::KeyZ));
        assert!(!input.keys.just_released(KeyCode::KeyZ));
    }

    #[cfg(feature = "fullscreen-scaling")]
    #[test]
    fn handheld_initial_window_fits_high_dpi_display_and_keeps_user_resizes() {
        let mut app = App::new();
        app.add_systems(Update, fit_initial_window);
        app.world_mut().spawn((
            bevy::window::Monitor {
                name: Some("Handheld".into()),
                physical_width: 1920,
                physical_height: 1200,
                physical_position: IVec2::ZERO,
                refresh_rate_millihertz: Some(60000),
                scale_factor: 2.0,
                video_modes: Vec::new(),
            },
            bevy::window::PrimaryMonitor,
        ));
        let window = app
            .world_mut()
            .spawn((
                Window {
                    resolution: bevy::window::WindowResolution::new(1280, 800),
                    ..default()
                },
                bevy::window::PrimaryWindow,
            ))
            .id();
        app.update();
        let resized = app.world().get::<Window>(window).unwrap();
        assert_eq!(resized.width(), 912.0);
        assert_eq!(resized.height(), 504.0);
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .resolution
            .set(800.0, 450.0);
        app.update();
        assert_eq!(app.world().get::<Window>(window).unwrap().width(), 800.0);
        assert_eq!(app.world().get::<Window>(window).unwrap().height(), 450.0);
    }

    #[test]
    fn handheld_fullscreen_toggles_once_per_press() {
        let mut app = App::new();
        app.init_resource::<HandheldInput>()
            .add_systems(Update, toggle_fullscreen);
        let window = app
            .world_mut()
            .spawn((Window::default(), bevy::window::PrimaryWindow))
            .id();
        app.world_mut()
            .resource_mut::<HandheldInput>()
            .keys
            .press(KeyCode::F11);
        app.update();
        assert_eq!(
            app.world().get::<Window>(window).unwrap().mode,
            bevy::window::WindowMode::BorderlessFullscreen(bevy::window::MonitorSelection::Current)
        );
        app.world_mut().resource_mut::<HandheldInput>().keys.clear();
        app.update();
        assert_eq!(
            app.world().get::<Window>(window).unwrap().mode,
            bevy::window::WindowMode::BorderlessFullscreen(bevy::window::MonitorSelection::Current)
        );
        let mut input = app.world_mut().resource_mut::<HandheldInput>();
        input.keys.release(KeyCode::F11);
        input.keys.press(KeyCode::F11);
        app.update();
        assert_eq!(
            app.world().get::<Window>(window).unwrap().mode,
            bevy::window::WindowMode::Windowed
        );
    }
    #[cfg(feature = "voxel-view")]
    #[test]
    fn handheld_optional_view_toggle_and_camera_preserve_keyboard_ownership() {
        let mut app = App::new();
        app.init_resource::<HandheldInput>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<crystal_render_api::VisualWorldFrame>()
            .init_resource::<crystal_voxel_view::VoxelViewSettings>()
            .add_systems(Update, controller_camera);
        app.world_mut()
            .resource_mut::<HandheldInput>()
            .controller_keys
            .press(KeyCode::F3);
        app.update();
        assert!(
            app.world()
                .resource::<crystal_voxel_view::VoxelViewSettings>()
                .enabled
        );
        app.world_mut()
            .resource_mut::<HandheldInput>()
            .controller_keys
            .clear();
        app.world_mut()
            .resource_mut::<crystal_render_api::VisualWorldFrame>()
            .active = true;
        let before = app
            .world()
            .resource::<crystal_voxel_view::VoxelViewSettings>()
            .camera;
        // Physical keys are exclusively processed by the voxel plugin.
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyE);
        app.update();
        assert_eq!(
            app.world()
                .resource::<crystal_voxel_view::VoxelViewSettings>()
                .camera,
            before
        );
        app.world_mut()
            .resource_mut::<HandheldInput>()
            .controller_keys
            .press(KeyCode::KeyE);
        app.update();
        assert_eq!(
            app.world()
                .resource::<crystal_voxel_view::VoxelViewSettings>()
                .camera,
            before,
            "the physical keyboard plugin owns a simultaneous Steam Input alias"
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        let mut input = app.world_mut().resource_mut::<HandheldInput>();
        input.controller_keys.reset_all();
        input.controller_keys.press(KeyCode::KeyE);
        input.controller_keys.press(KeyCode::PageUp);
        app.update();
        let after = app
            .world()
            .resource::<crystal_voxel_view::VoxelViewSettings>()
            .camera;
        assert_eq!(after.rotation_step, before.rotation_step + 1.0);
        assert_eq!(after.zoom_step, before.zoom_step + 1.0);
        app.world_mut()
            .resource_mut::<HandheldInput>()
            .controller_keys
            .clear();
        app.update();
        assert_eq!(
            app.world()
                .resource::<crystal_voxel_view::VoxelViewSettings>()
                .camera,
            after
        );
    }
}
