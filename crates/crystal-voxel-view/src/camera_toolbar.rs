//! Native presentation controls; no gameplay input is generated here.
use crate::{VoxelCameraControls, VoxelViewSettings, VoxelViewStatus, VoxelWorldCamera};
use bevy::prelude::*;

#[derive(Component)]
struct Toolbar;

#[derive(Component)]
struct ToolbarPanel;

#[derive(Resource, Default)]
struct ToolbarState {
    expanded: bool,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum Action {
    Toggle,
    ZoomOut,
    Reset,
    ZoomIn,
    Left,
    Right,
    Up,
    Down,
    FirstPerson,
}

#[derive(Component)]
struct Label(Action);

#[derive(Resource, Default)]
struct PreviousOrbit(Option<VoxelCameraControls>);

pub(super) fn register(app: &mut App) {
    app.init_resource::<PreviousOrbit>()
        .init_resource::<ToolbarState>()
        .add_systems(Startup, setup.after(crate::setup_voxel_view))
        .add_systems(
            Update,
            (
                input.before(crate::sync_voxel_view),
                update.after(crate::sync_voxel_view),
            ),
        );
}

fn setup(mut commands: Commands, cameras: Query<Entity, With<VoxelWorldCamera>>) {
    let Ok(camera) = cameras.single() else {
        return;
    };
    commands
        .spawn((
            Toolbar,
            UiTargetCamera(camera),
            GlobalZIndex(100),
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                top: px(12),
                right: px(12),
                max_width: percent(90),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                column_gap: px(4),
                row_gap: px(4),
                padding: UiRect::all(px(6)),
                ..default()
            },
        ))
        .with_children(|bar| {
            bar.spawn((
                Interaction::None,
                bevy::ui::FocusPolicy::Block,
                Action::Toggle,
                Node { min_width: px(44), height: px(44),
                    padding: UiRect::horizontal(px(10)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center, ..default() },
                BackgroundColor(Color::srgb(0.10, 0.14, 0.19)),
            )).with_children(|button| {
                button.spawn((Text::new("Camera"), TextFont::from_font_size(14.),
                    TextColor(Color::WHITE)));
            });
            bar.spawn((ToolbarPanel,
                Node { display: Display::None, max_width: percent(100),
                    flex_wrap: FlexWrap::Wrap, justify_content: JustifyContent::FlexEnd,
                    column_gap: px(4), row_gap: px(4), padding: UiRect::all(px(6)),
                    ..default() },
                BackgroundColor(Color::srgba(0.04, 0.06, 0.09, 0.88)),
            )).with_children(|bar| {
            for (action, label) in [
                (Action::ZoomOut, "-"),
                (Action::Reset, "100%"),
                (Action::ZoomIn, "+"),
                (Action::Left, "Rotate left"),
                (Action::Right, "Rotate right"),
                (Action::Up, "Tilt up"),
                (Action::Down, "Tilt down"),
                (Action::FirstPerson, "First person"),
            ] {
                bar.spawn((
                    Interaction::None,
                    bevy::ui::FocusPolicy::Block,
                    action,
                    Node {
                        min_width: px(44),
                        height: px(44),
                        padding: UiRect::horizontal(px(10)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.10, 0.14, 0.19)),
                ))
                .with_children(|button| {
                    button.spawn((
                        Label(action),
                        Text::new(label),
                        TextFont::from_font_size(14.),
                        TextColor(Color::WHITE),
                    ));
                });
            }
            });
        });
}

fn apply(action: Action, camera: &mut VoxelCameraControls, orbit: &mut PreviousOrbit) {
    match action {
        Action::Toggle => {},
        Action::ZoomOut => camera.zoom_step = (camera.zoom_step - 0.5).max(0.),
        Action::ZoomIn => camera.zoom_step = (camera.zoom_step + 0.5).min(5.),
        Action::Left => camera.rotation_step = (camera.rotation_step - 1.).rem_euclid(8.),
        Action::Right => camera.rotation_step = (camera.rotation_step + 1.).rem_euclid(8.),
        Action::Up => {
            camera.pitch_degrees =
                (camera.pitch_degrees - 10.).max(if camera.first_person { -85. } else { 5. })
        }
        Action::Down => camera.pitch_degrees = (camera.pitch_degrees + 10.).min(85.),
        Action::Reset => {
            *camera = VoxelCameraControls::default();
            orbit.0 = None;
        }
        Action::FirstPerson => {
            if camera.first_person {
                *camera = orbit.0.take().unwrap_or_default();
            } else {
                orbit.0 = Some(*camera);
                camera.first_person = true;
                camera.pitch_degrees = 0.;
            }
        }
    }
}

fn input(
    status: Res<VoxelViewStatus>,
    mut settings: ResMut<VoxelViewSettings>,
    mut orbit: ResMut<PreviousOrbit>,
    mut toolbar: ResMut<ToolbarState>,
    scroll: Option<Res<bevy::input::mouse::AccumulatedMouseScroll>>,
    pinch: Option<Res<Messages<bevy::input::gestures::PinchGesture>>>,
    mut pinch_cursor: Local<bevy::ecs::message::MessageCursor<bevy::input::gestures::PinchGesture>>,
    mut buttons: Query<(&Interaction, &Action, &mut BackgroundColor), Changed<Interaction>>,
) {
    if status.active && settings.enabled {
        if let Some(scroll) = scroll {
            let scale = match scroll.unit {
                bevy::input::mouse::MouseScrollUnit::Line => 0.5,
                bevy::input::mouse::MouseScrollUnit::Pixel => 0.005,
            };
            if scroll.delta.x.is_finite() && scroll.delta.x != 0.0 {
                settings.camera.steer(-scroll.delta.x * scale * 0.2, 0.);
            }
            if scroll.delta.y.is_finite() && scroll.delta.y != 0.0 {
                settings.camera.zoom_step = (settings.camera.zoom_step
                    + (scroll.delta.y * scale).clamp(-1., 1.))
                .clamp(0., 5.);
            }
        }
    }
    if let Some(pinch) = pinch {
        for event in pinch_cursor.read(&pinch) {
            if status.active && settings.enabled && event.0.is_finite() {
                settings.camera.zoom_step =
                    (settings.camera.zoom_step + event.0 * 4.).clamp(0., 5.);
            }
        }
    }
    for (interaction, action, mut color) in &mut buttons {
        color.0 = match interaction {
            Interaction::None => Color::srgb(0.10, 0.14, 0.19),
            _ => Color::srgb(0.15, 0.28, 0.36),
        };
        if status.active && settings.enabled && *interaction == Interaction::Pressed {
            if *action == Action::Toggle {
                toolbar.expanded = !toolbar.expanded;
            } else {
                apply(*action, &mut settings.camera, &mut orbit);
            }
        }
    }
}

fn update(
    settings: Res<VoxelViewSettings>,
    status: Res<VoxelViewStatus>,
    toolbar: Res<ToolbarState>,
    mut roots: Query<&mut Visibility, With<Toolbar>>,
    mut panels: Query<&mut Node, With<ToolbarPanel>>,
    mut labels: Query<(&Label, &mut Text)>,
) {
    let visible = if settings.enabled && status.active {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut visibility in &mut roots {
        if *visibility != visible {
            *visibility = visible;
        }
    }
    let display = if toolbar.expanded { Display::Flex } else { Display::None };
    for mut panel in &mut panels {
        if panel.display != display { panel.display = display; }
    }
    if !settings.is_changed() {
        return;
    }
    for (label, mut text) in &mut labels {
        let value = match label.0 {
            Action::Reset => format!("{}%", (75. + settings.camera.zoom_step * 25.).round()),
            Action::FirstPerson => if settings.camera.first_person {
                "Orbit view"
            } else {
                "First person"
            }
            .into(),
            _ => continue,
        };
        if text.0 != value {
            text.0 = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_camera_toggle_changes_only_presentation_and_respects_world_visibility() {
        let mut app = App::new();
        app.init_resource::<VoxelViewSettings>().init_resource::<VoxelViewStatus>()
            .init_resource::<PreviousOrbit>().init_resource::<ToolbarState>()
            .add_systems(Update, (input, update).chain());
        app.world_mut().resource_mut::<VoxelViewSettings>().enabled = true;
        let camera = app.world().resource::<VoxelViewSettings>().camera;
        let panel = app.world_mut().spawn((ToolbarPanel, Node::default())).id();
        let button = app.world_mut().spawn((Action::Toggle, Interaction::Pressed,
            BackgroundColor::default())).id();
        app.update();
        assert!(!app.world().resource::<ToolbarState>().expanded);
        assert_eq!(app.world().get::<Node>(panel).unwrap().display, Display::None);
        app.world_mut().resource_mut::<VoxelViewStatus>().active = true;
        for expanded in [true, false] {
            *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::None;
            app.update();
            *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::Pressed;
            app.update();
            assert_eq!(app.world().resource::<ToolbarState>().expanded, expanded);
            assert_eq!(app.world().get::<Node>(panel).unwrap().display,
                if expanded {Display::Flex} else {Display::None});
            assert_eq!(app.world().resource::<VoxelViewSettings>().camera, camera);
        }
    }

    #[test]
    fn clicks_change_the_shared_camera_only_while_world_is_visible() {
        let mut app = App::new();
        app.init_resource::<VoxelViewSettings>()
            .init_resource::<VoxelViewStatus>()
            .init_resource::<PreviousOrbit>()
            .init_resource::<ToolbarState>()
            .add_systems(Update, input);
        app.world_mut().resource_mut::<VoxelViewSettings>().enabled = true;
        let button = app
            .world_mut()
            .spawn((
                Action::FirstPerson,
                Interaction::Pressed,
                BackgroundColor::default(),
            ))
            .id();
        app.update();
        assert!(
            !app.world()
                .resource::<VoxelViewSettings>()
                .camera
                .first_person
        );
        app.world_mut().resource_mut::<VoxelViewStatus>().active = true;
        *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::None;
        app.update();
        *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::Pressed;
        app.update();
        assert!(
            app.world()
                .resource::<VoxelViewSettings>()
                .camera
                .first_person
        );
        assert_eq!(
            app.world()
                .resource::<VoxelViewSettings>()
                .camera
                .pitch_degrees,
            0.
        );
        *app.world_mut().get_mut::<Action>(button).unwrap() = Action::Reset;
        *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::None;
        app.update();
        *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::Pressed;
        app.update();
        assert_eq!(
            app.world().resource::<VoxelViewSettings>().camera,
            VoxelCameraControls::default()
        );
    }
}
