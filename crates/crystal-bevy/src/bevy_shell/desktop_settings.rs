//! Native presentation controls never synthesize gameplay input or edit saves.
use super::*;
use std::{path::PathBuf, time::Instant};
mod macos;
mod state;
mod store;
use state::{Action, Camera, Preferences};

#[derive(Resource)]
pub(super) struct DesktopSettings {
    state: Preferences,
    path: Option<PathBuf>,
    changed: Option<Instant>,
}

impl DesktopSettings {
    pub(super) fn gain(&self) -> f32 {
        self.state.gain()
    }

    fn flush(&mut self) {
        if self.changed.take().is_some()
            && let Some(path) = &self.path
            && let Err(error) = store::save(path, self.state)
        {
            bevy::log::warn!("save desktop settings: {error}");
        }
    }
}

impl Drop for DesktopSettings {
    fn drop(&mut self) {
        self.flush();
    }
}

struct MenuHost {
    native: Option<macos::NativeMenus>,
    mirrored: Preferences,
}

pub(super) fn install(app: &mut App) {
    let path = std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .map(|home| home.join("Library/Application Support/Geothite/settings.json"));
    let mut prefs = path
        .as_ref()
        .map(|path| store::load(path))
        .transpose()
        .unwrap_or_else(|error| {
            bevy::log::warn!("load desktop settings: {error}");
            None
        })
        .unwrap_or_default();
    if !app
        .world()
        .resource::<renderer_policy::RendererPolicy>()
        .voxel_allowed
    {
        prefs.view_3d = false;
    }
    #[cfg(feature = "voxel-view")]
    if let Some(mut view) = app
        .world_mut()
        .get_resource_mut::<crystal_voxel_view::VoxelViewSettings>()
    {
        view.enabled = prefs.view_3d;
        view.camera = camera_controls(prefs.camera);
    }
    let mut windows = app
        .world_mut()
        .query_filtered::<&mut Window, With<bevy::window::PrimaryWindow>>();
    if let Ok(mut window) = windows.single_mut(app.world_mut()) {
        apply_fullscreen(&mut window, prefs.fullscreen);
    }
    app.insert_resource(DesktopSettings {
        state: prefs,
        path,
        changed: None,
    })
    .insert_non_send(MenuHost {
        native: None,
        mirrored: prefs,
    })
    .add_systems(
        Update,
        apply_actions
            .before(play_pending_audio)
            .before(crystal_render_api::WorldRenderSet::PresentationExtract),
    )
    .add_systems(PostUpdate, sync_actual_settings);
}

#[cfg(feature = "voxel-view")]
fn camera_controls(camera: Camera) -> crystal_voxel_view::VoxelCameraControls {
    crystal_voxel_view::VoxelCameraControls::with_pose(
        camera.zoom,
        camera.rotation,
        camera.pitch,
        camera.first_person,
    )
}

fn apply_fullscreen(window: &mut Window, fullscreen: bool) {
    window.mode = if fullscreen {
        bevy::window::WindowMode::BorderlessFullscreen(bevy::window::MonitorSelection::Current)
    } else {
        bevy::window::WindowMode::Windowed
    };
}

fn apply_actions(
    mut prefs: ResMut<DesktopSettings>,
    policy: Res<renderer_policy::RendererPolicy>,
    _host: NonSendMut<MenuHost>,
    frame: Option<Res<crystal_render_api::VisualWorldFrame>>,
    #[cfg(feature = "voxel-view")] mut view: Option<ResMut<crystal_voxel_view::VoxelViewSettings>>,
    mut windows: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
) {
    let facing = frame
        .as_ref()
        .and_then(|frame| {
            frame
                .actors
                .iter()
                .find(|actor| actor.id == crystal_render_api::VisualActorId::Player)
                .and_then(|actor| actor.facing)
        })
        .unwrap_or(Vec2::NEG_Y)
        .to_array();
    for action in macos::take_actions() {
        if !policy.voxel_allowed
            && matches!(
                action,
                Action::Toggle3d
                    | Action::ToggleFirstPerson
                    | Action::Camera(_)
                    | Action::ResetCamera
            )
        {
            continue;
        }
        prefs.state.apply(action, facing);
        prefs.state.view_3d &= policy.voxel_allowed;
        #[cfg(feature = "voxel-view")]
        if let Some(view) = view.as_mut() {
            if matches!(
                action,
                Action::Toggle3d | Action::ToggleFirstPerson | Action::Camera(_) | Action::Reset
            ) {
                view.enabled = prefs.state.view_3d;
            }
            if matches!(
                action,
                Action::ToggleFirstPerson | Action::Camera(_) | Action::ResetCamera | Action::Reset
            ) {
                view.camera = camera_controls(prefs.state.camera);
            }
        }
        if matches!(action, Action::Fullscreen | Action::Reset)
            && let Ok(mut window) = windows.single_mut()
        {
            apply_fullscreen(&mut window, prefs.state.fullscreen);
        }
    }
}

fn sync_actual_settings(
    mut prefs: ResMut<DesktopSettings>,
    policy: Res<renderer_policy::RendererPolicy>,
    mut host: NonSendMut<MenuHost>,
    #[cfg(feature = "voxel-view")] view: Option<Res<crystal_voxel_view::VoxelViewSettings>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    // Read actual presentation resources so keyboard, mouse and controller
    // changes stay reflected in the menus and survive a normal app restart.
    #[cfg(feature = "voxel-view")]
    if let Some(view) = view {
        let camera = Camera {
            zoom: view.camera.zoom_step,
            rotation: view.camera.rotation_step,
            pitch: view.camera.pitch_degrees,
            first_person: view.camera.first_person,
        };
        if prefs.state.view_3d != view.enabled || prefs.state.camera != camera {
            prefs.state.view_3d = view.enabled;
            prefs.state.camera = camera;
        }
    }
    if let Ok(window) = windows.single() {
        let fullscreen = window.mode != bevy::window::WindowMode::Windowed;
        if prefs.state.fullscreen != fullscreen {
            prefs.state.fullscreen = fullscreen;
        }
    }
    let created = host.native.is_none() && {
        if let Ok(window) = windows.single() {
            host.native = macos::NativeMenus::new(&window.title);
        }
        host.native.is_some()
    };
    if created || host.mirrored != prefs.state {
        if let Some(native) = &host.native {
            native.sync(
                prefs.state,
                cfg!(feature = "voxel-view") && policy.voxel_allowed,
            );
        }
    }
    if host.mirrored != prefs.state {
        if std::env::var_os("CRYSTAL_SETTINGS_TRACE").is_some() {
            eprintln!("desktop settings: {:?}", prefs.state);
        }
        host.mirrored = prefs.state;
        prefs.changed = Some(Instant::now());
    }
    // Dragging/looking can change camera pose every frame. Debounce disk writes.
    if prefs
        .changed
        .is_some_and(|time| time.elapsed() >= Duration::from_millis(350))
    {
        prefs.flush();
    }
}
