//! Automated captures share a GPU session and finish on readback, not a timer.
use super::*;
// Location-test startup is also available in local browser builds.
use bevy::platform::time::Instant;
use std::sync::{Arc, Mutex};

type Completion = Arc<Mutex<Option<Result<(), String>>>>;

#[derive(Resource)]
struct Capture {
    path: PathBuf,
    second: Option<PathBuf>,
    same_second_view: bool,
    frame: u32,
    requested: bool,
    completion: Completion,
    started: Instant,
    live: bool,
    idle: bool,
    first_path: PathBuf,
    second_path: Option<PathBuf>,
    request_path: PathBuf,
    previous: Option<(String, image::RgbImage)>,
    identity: String,
    image_target_mode: bool,
    image_target: Option<bevy::camera::ImageRenderTarget>,
    image_size: UVec2,
}

pub(super) fn install(
    app: &mut App,
    path: PathBuf,
    second: Option<PathBuf>,
    live: bool,
) -> Result<()> {
    let same_second_view = match std::env::var("CRYSTAL_RENDER_TEST_SECOND_VIEW") {
        Err(std::env::VarError::NotPresent) => false,
        Ok(value) if value == "toggle" => false,
        Ok(value) if value == "same" => true,
        _ => anyhow::bail!("CRYSTAL_RENDER_TEST_SECOND_VIEW must be toggle or same"),
    };
    anyhow::ensure!(!same_second_view || !live,
        "same-view temporal captures require one-shot verification mode");
    let image_target_mode = match std::env::var("CRYSTAL_RENDER_TEST_CAPTURE_TARGET") {
        Err(std::env::VarError::NotPresent) => false,
        Ok(value) if value == "window" => false,
        Ok(value) if value == "image" => true,
        _ => anyhow::bail!("CRYSTAL_RENDER_TEST_CAPTURE_TARGET must be window or image"),
    };
    anyhow::ensure!(
        !image_target_mode || !live,
        "image-target capture is a one-shot verification mode; live review requires window capture"
    );
    for path in std::iter::once(&path).chain(second.iter()) {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        // A failed capture must never validate a previous run's image.
        match std::fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    let request_path = path.with_extension("request");
    if live {
        println!(
            "live review: F6 captures again; external trigger: {} (empty, or zoom and orbit numbers)",
            request_path.display()
        );
    }
    app.insert_resource(Capture {
        live,
        idle: false,
        first_path: path.clone(),
        second_path: second.clone(),
        request_path,
        previous: None,
        identity: String::new(),
        image_target_mode,
        image_target: None,
        image_size: UVec2::ZERO,
        path,
        second,
        same_second_view,
        frame: 0,
        requested: false,
        completion: default(),
        started: Instant::now(),
    })
    .add_systems(Update, (prepare_image_target, capture).chain());
    Ok(())
}

// Explicit verification target, never a retry after a failed window capture.
// Keep production cameras, render order, viewport and projection. Other image
// targets (including battle rows) retain their existing ownership.
fn prepare_image_target(
    mut capture: ResMut<Capture>,
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
    mut targets: Query<(Entity, &mut bevy::camera::RenderTarget, Has<Camera2d>), With<Camera>>,
    mut commands: Commands,
) {
    if !capture.image_target_mode {
        return;
    }
    let Ok((window_entity, window)) = windows.single() else {
        return;
    };
    let size = UVec2::new(window.physical_width(), window.physical_height());
    if size.x == 0 || size.y == 0 {
        return;
    }
    let scale_factor = window.resolution.scale_factor();
    let previous = capture.image_target.clone();
    if capture.image_target.is_none()
        || capture.image_size != size
        || capture
            .image_target
            .as_ref()
            .is_some_and(|target| target.scale_factor != scale_factor)
    {
        let handle = images.add(Image::new_target_texture(
            size.x,
            size.y,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            None,
        ));
        capture.image_target = Some(bevy::camera::ImageRenderTarget {
            handle,
            scale_factor,
        });
        capture.image_size = size;
        capture.frame = 0;
        println!(
            "render-test explicit image target: {}x{} scale {}",
            size.x, size.y, scale_factor
        );
    }
    let Some(image) = capture.image_target.as_ref() else {
        return;
    };
    for (entity, mut target, is_hud_camera) in &mut targets {
        let selected = match &*target {
            bevy::camera::RenderTarget::Window(bevy::window::WindowRef::Primary) => true,
            bevy::camera::RenderTarget::Window(bevy::window::WindowRef::Entity(entity)) => {
                *entity == window_entity
            }
            bevy::camera::RenderTarget::Image(current) => previous
                .as_ref()
                .is_some_and(|old| old.handle == current.handle),
            _ => false,
        };
        if selected
            && !matches!(&*target,
            bevy::camera::RenderTarget::Image(current) if current == image)
        {
            *target = bevy::camera::RenderTarget::Image(image.clone());
            if is_hud_camera {
                commands.entity(entity).insert(crystal_voxel_view::BattleCompositeImageCamera);
            }
        }
    }
}

fn capture(
    mut capture: ResMut<Capture>,
    runtime: Res<BevyRuntimeShell>,
    walk: Option<Res<render_walk::RenderWalk>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    frame: Res<crystal_render_api::VisualWorldFrame>,
    status: Res<crystal_voxel_view::VoxelViewStatus>,
    battle_status: Res<crystal_voxel_view::BattleViewStatus>,
    battle_frame: Res<crystal_render_api::VisualBattleFrame>,
    mut settings: ResMut<crystal_voxel_view::VoxelViewSettings>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    mut commands: Commands,
    mut exit: MessageWriter<AppExit>,
) {
    if let Some(error) = runtime.render_test_error.as_ref() {
        eprintln!("capture rejected after runtime error: {error}");
        exit.write(AppExit::error());
        return;
    }
    if capture.idle {
        let external = match std::fs::read_to_string(&capture.request_path) {
            Ok(value) => {
                let _ = std::fs::remove_file(&capture.request_path);
                Some(value)
            }
            Err(_) => None,
        };
        if !keyboard.just_pressed(KeyCode::F6) && external.is_none() {
            return;
        }
        if let Some(value) = external.filter(|value| !value.trim().is_empty()) {
            match parse_camera_request(&value) {
                Ok(camera) => settings.camera = camera,
                Err(error) => {
                    eprintln!("review request: {error}");
                    return;
                }
            }
        }
        capture.path = capture.first_path.clone();
        capture.second = capture.second_path.clone();
        capture.requested = false;
        capture.frame = 0;
        capture.started = Instant::now();
        capture.identity.clear();
        capture.idle = false;
        for path in std::iter::once(&capture.first_path).chain(capture.second_path.iter()) {
            if let Err(error) = std::fs::remove_file(path) {
                if error.kind() != std::io::ErrorKind::NotFound {
                    eprintln!("cannot clear previous capture: {error}");
                    exit.write(AppExit::error());
                    return;
                }
            }
        }
        settings.enabled = false;
    }
    let completed = capture.completion.lock().unwrap().take();
    if let Some(result) = completed {
        if let Err(error) = result {
            eprintln!("capture failed: {error}");
            exit.write(AppExit::error());
            return;
        }
        println!(
            "wrote {} in {:.2}s",
            capture.path.display(),
            capture.started.elapsed().as_secs_f64()
        );
        if let Some(second) = capture.second.take() {
            capture.path = second;
            capture.frame = 0;
            capture.requested = false;
            if !capture.same_second_view {
                settings.enabled = !settings.enabled;
            }
        } else if capture.live {
            if let Err(error) = live_comparison(&mut capture) {
                eprintln!("review failed: {error:#}");
                exit.write(AppExit::error());
                return;
            }
            capture.idle = true;
            println!(
                "review ready in {:.2}s; session stays open — F6 to capture again",
                capture.started.elapsed().as_secs_f64()
            );
        } else {
            exit.write(AppExit::Success);
        }
        return;
    }
    // Bound startup/readback failures; ordinary walking has its own duration.
    if capture.started.elapsed().as_secs() > 60 && walk.is_none() {
        eprintln!("capture timed out: {:?}", status.inactive_reason);
        exit.write(AppExit::error());
        return;
    }
    capture.frame = capture.frame.saturating_add(1);
    // Preserve the established 30 active-frame GPU warmup, but drop the
    // unconditional 90-frame startup and 60-frame post-save waits.
    let settled = if settings.enabled {
        (status.active && !status.profiles_pending && status.active_frames >= 30)
            || (battle_status.active && battle_status.active_frames >= 30)
    } else {
        status.inactive_reason.as_deref() == Some("disabled")
    };
    if capture.requested
        || capture.frame < 12
        || !settled
        || walk.as_ref().is_some_and(|walk| !walk.settled())
    {
        return;
    }
    let screenshot = if capture.image_target_mode {
        let Some(target) = capture.image_target.as_ref() else {
            return;
        };
        // The screenshot's normalized target must also retain the scale.
        // Screenshot::image(handle) creates a distinct scale-1 target.
        Screenshot(bevy::camera::RenderTarget::Image(target.clone()))
    } else {
        let Ok(window) = primary_window.single() else {
            return;
        };
        Screenshot::window(window)
    };
    if capture.identity.is_empty() {
        capture.identity = if battle_status.active {
            format!(
                "battle|{}|modeled={:?}|source={:?}",
                battle_frame.map_id,
                battle_status.modeled_species,
                battle_status.source_art_species
            )
        } else {
            format!("{}|{:?}|{:?}", frame.map_id, frame.center, settings.camera)
        };
    }
    let path = capture.path.clone();
    let completion = capture.completion.clone();
    commands
        .spawn(screenshot)
        .observe(move |event: On<ScreenshotCaptured>| {
            let result = event
                .image
                .clone()
                .try_into_dynamic()
                .map_err(|error| format!("decode GPU image: {error}"))
                .and_then(|image| {
                    image
                        .to_rgb8()
                        .save(&path)
                        .map_err(|error| error.to_string())
                });
            *completion.lock().unwrap() = Some(result);
        });
    capture.requested = true;
}

fn parse_camera_request(value: &str) -> Result<crystal_voxel_view::VoxelCameraControls> {
    let values = value
        .split_whitespace()
        .map(str::parse::<f32>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    anyhow::ensure!(
        values.len() == 2 && values.iter().all(|value| value.is_finite()),
        "expected finite zoom and orbit, e.g. 1.0 0.5"
    );
    Ok(crystal_voxel_view::VoxelCameraControls::new(
        values[0], values[1],
    ))
}

fn live_comparison(capture: &mut Capture) -> Result<()> {
    validate_render_test_screenshot(&capture.first_path)?;
    let second = capture
        .second_path
        .as_ref()
        .context("live review needs paired views")?;
    validate_render_test_screenshot(second)?;
    let current = image::open(second)?.to_rgb8();
    let reference = image::open(&capture.first_path)?.to_rgb8();
    let previous = capture
        .previous
        .as_ref()
        .filter(|(identity, _)| identity == &capture.identity);
    let mut panels = vec![&reference];
    if let Some((_, previous)) = previous {
        panels.push(previous);
    }
    panels.push(&current);
    let mut sheet = image::RgbImage::new(640 * panels.len() as u32, 576);
    for (index, panel) in panels.iter().enumerate() {
        let panel = image::imageops::resize(*panel, 640, 576, image::imageops::FilterType::Nearest);
        image::imageops::replace(&mut sheet, &panel, index as i64 * 640, 0);
    }
    let path = capture.first_path.with_extension("compare.png");
    sheet.save(&path)?;
    println!(
        "review {} — 2D reference / {}current 2.5D",
        path.display(),
        if previous.is_some() {
            "previous 2.5D / "
        } else {
            ""
        }
    );
    capture.previous = Some((capture.identity.clone(), current));
    Ok(())
}

#[cfg(test)]
mod image_target_tests {
    use super::*;
    use bevy::{
        camera::RenderTarget,
        window::{WindowRef, WindowResolution},
    };

    fn fixture(enabled: bool) -> (App, Entity, Entity, Entity, Entity, Handle<Image>) {
        let mut app = App::new();
        app.init_resource::<Assets<Image>>()
            .insert_resource(Capture {
                path: PathBuf::new(),
                second: None,
                same_second_view: false,
                frame: 0,
                requested: false,
                completion: default(),
                started: Instant::now(),
                live: false,
                idle: false,
                first_path: PathBuf::new(),
                second_path: None,
                request_path: PathBuf::new(),
                previous: None,
                identity: String::new(),
                image_target_mode: enabled,
                image_target: None,
                image_size: UVec2::ZERO,
            })
            .add_systems(Update, prepare_image_target);
        let window = app
            .world_mut()
            .spawn((
                Window {
                    resolution: WindowResolution::new(640, 576).with_scale_factor_override(2.),
                    ..default()
                },
                PrimaryWindow,
            ))
            .id();
        let secondary = app.world_mut().spawn(Window::default()).id();
        let primary_camera = app
            .world_mut()
            .spawn((
                Camera2d,
                Camera {
                    order: 3,
                    viewport: Some(bevy::camera::Viewport {
                        physical_position: UVec2::new(16, 24),
                        physical_size: UVec2::new(320, 288),
                        ..default()
                    }),
                    ..default()
                },
            ))
            .id();
        let secondary_camera = app
            .world_mut()
            .spawn((Camera2d, RenderTarget::Window(WindowRef::Entity(secondary))))
            .id();
        let owned_image =
            app.world_mut()
                .resource_mut::<Assets<Image>>()
                .add(Image::new_target_texture(
                    32,
                    32,
                    bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                    None,
                ));
        let image_camera = app
            .world_mut()
            .spawn((Camera2d, RenderTarget::Image(owned_image.clone().into())))
            .id();
        (
            app,
            window,
            primary_camera,
            secondary_camera,
            image_camera,
            owned_image,
        )
    }

    #[test]
    fn explicit_image_capture_preserves_viewport_scale_and_other_target_ownership() {
        let (mut app, _, camera, secondary, image_camera, owned) = fixture(true);
        app.update();
        let capture = app.world().resource::<Capture>();
        let target = capture.image_target.as_ref().unwrap();
        assert_eq!(target.scale_factor, 2.);
        assert_eq!(capture.image_size, UVec2::new(640, 576));
        assert!(matches!(app.world().get::<RenderTarget>(camera),
            Some(RenderTarget::Image(image)) if image == target));
        assert!(app.world().get::<crystal_voxel_view::BattleCompositeImageCamera>(camera).is_some());
        assert!(app.world().get::<crystal_voxel_view::BattleCompositeImageCamera>(secondary).is_none());
        assert!(app.world().get::<crystal_voxel_view::BattleCompositeImageCamera>(image_camera).is_none());
        let camera = app.world().get::<Camera>(camera).unwrap();
        assert_eq!(camera.order, 3);
        assert_eq!(
            camera.viewport.as_ref().unwrap().physical_position,
            UVec2::new(16, 24)
        );
        assert_eq!(
            camera.viewport.as_ref().unwrap().physical_size,
            UVec2::new(320, 288)
        );
        assert!(matches!(
            app.world().get::<RenderTarget>(secondary),
            Some(RenderTarget::Window(WindowRef::Entity(_)))
        ));
        assert!(matches!(app.world().get::<RenderTarget>(image_camera),
            Some(RenderTarget::Image(image)) if image.handle == owned && image.scale_factor == 1.));
    }

    #[test]
    fn explicit_image_capture_resize_updates_only_its_owned_target() {
        let (mut app, window, camera, _, image_camera, owned) = fixture(true);
        app.update();
        let old = app
            .world()
            .resource::<Capture>()
            .image_target
            .as_ref()
            .unwrap()
            .handle
            .clone();
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .resolution
            .set_physical_resolution(800, 600);
        app.update();
        let capture = app.world().resource::<Capture>();
        let target = capture.image_target.as_ref().unwrap();
        assert_ne!(target.handle, old);
        assert_eq!(capture.image_size, UVec2::new(800, 600));
        assert!(matches!(app.world().get::<RenderTarget>(camera),
            Some(RenderTarget::Image(image)) if image == target));
        assert!(matches!(app.world().get::<RenderTarget>(image_camera),
            Some(RenderTarget::Image(image)) if image.handle == owned));
    }

    #[test]
    fn ordinary_window_capture_does_not_create_or_redirect_an_image_target() {
        let (mut app, _, camera, _, _, _) = fixture(false);
        app.update();
        assert!(app.world().resource::<Capture>().image_target.is_none());
        assert!(matches!(
            app.world().get::<RenderTarget>(camera),
            Some(RenderTarget::Window(WindowRef::Primary))
        ));
    }
}
