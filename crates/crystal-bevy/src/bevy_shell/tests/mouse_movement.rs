fn mouse_buttons(app: &mut App, left: bool, right: bool) {
    let mut buttons = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
    buttons.reset_all();
    if left {
        buttons.press(MouseButton::Left);
    }
    if right {
        buttons.press(MouseButton::Right);
    }
}

#[test]
fn natural_movement_mouse_chord_requires_both_buttons_and_stops_on_either_release() {
    for first in [MouseButton::Left, MouseButton::Right] {
        let mut app = app();
        app.init_resource::<ButtonInput<MouseButton>>();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(first);
        app.update();
        assert_eq!(
            app.world()
                .resource::<BevyRuntimeShell>()
                .natural_movement
                .position
                .offset,
            [0.; 2]
        );
        mouse_buttons(&mut app, true, true);
        app.update();
        let offset = app
            .world()
            .resource::<BevyRuntimeShell>()
            .natural_movement
            .position
            .offset;
        assert_eq!(offset, [0., -0.125]);
        mouse_buttons(
            &mut app,
            first == MouseButton::Left,
            first == MouseButton::Right,
        );
        app.update();
        assert_eq!(
            app.world()
                .resource::<BevyRuntimeShell>()
                .natural_movement
                .position
                .offset,
            offset
        );
        // A chord released between gameplay ticks must not become a queued step.
        app.world_mut()
            .resource_mut::<RuntimeTickTimer>()
            .step_seconds = 1.;
        mouse_buttons(&mut app, true, true);
        app.update();
        mouse_buttons(&mut app, false, false);
        app.world_mut()
            .resource_mut::<RuntimeTickTimer>()
            .step_seconds = 0.;
        app.update();
        assert_eq!(
            app.world()
                .resource::<BevyRuntimeShell>()
                .natural_movement
                .position
                .offset,
            offset
        );
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn natural_movement_mouse_chord_steers_in_the_same_frame() {
    let mut app = app();
    app.add_message::<WindowFocused>()
        .insert_resource(crystal_render_api::VisualWorldFrame {
            active: true,
            ..default()
        })
        .insert_resource(bevy::input::mouse::AccumulatedMouseMotion {
            delta: Vec2::new(100., 0.),
        })
        .init_resource::<ButtonInput<MouseButton>>();
    handheld_input::install(&mut app);
    mouse_buttons(&mut app, true, true);
    app.update();
    let camera = app
        .world()
        .resource::<crystal_voxel_view::VoxelViewSettings>()
        .camera;
    assert_ne!(camera.rotation_step, 0.);
    let forward = camera.pose(Vec2::new(160., 144.)).transform().forward();
    let expected = Vec2::new(forward.x, forward.z).normalize() * 0.125;
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(shell.last_error, None);
    assert!(Vec2::from_array(shell.natural_movement.position.offset).abs_diff_eq(expected, 1e-6));
}

#[test]
fn natural_movement_mouse_chord_strafes_without_increasing_speed() {
    let mut app = app();
    app.init_resource::<ButtonInput<MouseButton>>();
    mouse_buttons(&mut app, true, true);
    key(&mut app, KeyCode::KeyD, true);
    app.update();
    let offset = Vec2::from_array(
        app.world()
            .resource::<BevyRuntimeShell>()
            .natural_movement
            .position
            .offset,
    );
    assert!(offset.abs_diff_eq(Vec2::new(1., -1.).normalize() * 0.125, 1e-6));
}

#[test]
fn natural_movement_mouse_chord_is_disabled_in_2d_and_rearms_after_release() {
    let mut app = app();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.world_mut()
        .resource_mut::<crystal_voxel_view::VoxelViewSettings>()
        .enabled = false;
    mouse_buttons(&mut app, true, true);
    app.update();
    assert_eq!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .natural_movement
            .position
            .offset,
        [0.; 2]
    );
    app.world_mut()
        .resource_mut::<crystal_voxel_view::VoxelViewSettings>()
        .enabled = true;
    app.update();
    assert_eq!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .natural_movement
            .position
            .offset,
        [0.; 2]
    );
    mouse_buttons(&mut app, false, false);
    app.update();
    mouse_buttons(&mut app, true, true);
    app.update();
    assert_eq!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .natural_movement
            .position
            .offset,
        [0., -0.125]
    );
}

#[test]
fn natural_movement_mouse_chord_respects_ui_and_focus_loss() {
    let mut app = app();
    app.init_resource::<ButtonInput<MouseButton>>()
        .add_message::<WindowFocused>()
        .add_systems(
            Update,
            release_input_on_focus_loss.before(apply_keyboard_input),
        );
    let ui = app
        .world_mut()
        .spawn(bevy::ui::DeprecatedInteraction::Hovered)
        .id();
    mouse_buttons(&mut app, true, true);
    app.update();
    assert_eq!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .natural_movement
            .position
            .offset,
        [0.; 2]
    );
    app.world_mut().despawn(ui);
    app.update();
    assert_eq!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .natural_movement
            .position
            .offset,
        [0.; 2]
    );
    mouse_buttons(&mut app, false, false);
    app.update();
    mouse_buttons(&mut app, true, true);
    app.update();
    let offset = app
        .world()
        .resource::<BevyRuntimeShell>()
        .natural_movement
        .position
        .offset;
    assert_eq!(offset, [0., -0.125]);
    app.world_mut()
        .resource_mut::<Messages<WindowFocused>>()
        .write(WindowFocused {
            window: Entity::PLACEHOLDER,
            focused: false,
        });
    app.update();
    assert_eq!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .natural_movement
            .position
            .offset,
        offset
    );
    assert!(
        !app.world()
            .resource::<ButtonInput<MouseButton>>()
            .pressed(MouseButton::Left)
    );
}

#[test]
fn natural_movement_mouse_chord_does_not_navigate_the_start_menu() {
    let mut app = app();
    app.init_resource::<ButtonInput<MouseButton>>();
    key(&mut app, KeyCode::Enter, true);
    app.update();
    key(&mut app, KeyCode::Enter, false);
    let cursor = app
        .world()
        .resource::<BevyRuntimeShell>()
        .start_menu_cursor
        .clone();
    assert!(cursor.is_some());
    mouse_buttons(&mut app, true, true);
    for _ in 0..10 {
        app.update();
    }
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(shell.start_menu_cursor, cursor);
    assert_eq!(shell.natural_movement.position.offset, [0.; 2]);
}

#[test]
fn natural_movement_mouse_chord_crosses_authoritative_cells_and_stops_at_walls() {
    let mut app = app();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.world_mut()
        .resource_mut::<crystal_voxel_view::VoxelViewSettings>()
        .camera
        .rotation_step = 6.;
    let start = app
        .world()
        .resource::<BevyRuntimeShell>()
        .shell
        .session()
        .overworld()
        .player
        .tile;
    mouse_buttons(&mut app, true, true);
    for _ in 0..80 {
        app.update();
    }
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(shell.last_error, None);
    let tile = shell.shell.session().overworld().player.tile;
    assert!(tile.x > start.x);
    assert_eq!(tile.y, start.y);
    let offset = shell.natural_movement.position.offset;
    for _ in 0..20 {
        app.update();
    }
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(shell.shell.session().overworld().player.tile, tile);
    assert!(
        Vec2::from_array(shell.natural_movement.position.offset)
            .abs_diff_eq(Vec2::from_array(offset), 1e-6)
    );
}

#[test]
fn natural_movement_mouse_chord_waits_for_release_after_window_refocus() {
    let mut app = app();
    app.init_resource::<ButtonInput<MouseButton>>();
    let window = app
        .world_mut()
        .spawn((
            Window {
                focused: false,
                ..default()
            },
            bevy::window::PrimaryWindow,
        ))
        .id();
    mouse_buttons(&mut app, true, true);
    app.update();
    assert_eq!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .natural_movement
            .position
            .offset,
        [0.; 2]
    );
    app.world_mut()
        .get_mut::<Window>(window)
        .expect("primary window")
        .focused = true;
    app.update();
    assert_eq!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .natural_movement
            .position
            .offset,
        [0.; 2]
    );
    mouse_buttons(&mut app, false, false);
    app.update();
    mouse_buttons(&mut app, true, true);
    app.update();
    assert_eq!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .natural_movement
            .position
            .offset,
        [0., -0.125]
    );
}
