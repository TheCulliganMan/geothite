//! A single validated GPU readback before opt-in 3D timing, never during it.
use super::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use std::sync::{
    Mutex,
    atomic::{AtomicU64, Ordering},
};

const DRAIN_FRAMES: usize = 30;

#[derive(Resource, Clone, Default)]
struct GpuCompletions {
    submitted: Arc<AtomicU64>,
    completed: Arc<AtomicU64>,
    poll_errors: Arc<AtomicU64>,
}

// Runs after the render graph and its final readback commands have submitted.
// This observes completed work, not GPU timestamp duration. Poll never waits.
fn track_gpu_completion(
    counts: Res<GpuCompletions>,
    queue: Res<bevy::render::renderer::RenderQueue>,
    device: Res<bevy::render::renderer::RenderDevice>,
) {
    counts.submitted.fetch_add(1, Ordering::Relaxed);
    let completed = counts.completed.clone();
    queue.on_submitted_work_done(move || {
        completed.fetch_add(1, Ordering::Relaxed);
    });
    if device
        .poll(bevy::render::render_resource::PollType::Poll)
        .is_err()
    {
        counts.poll_errors.fetch_add(1, Ordering::Relaxed);
    }
}

fn report_gpu_completion(
    validation: Res<TimingValidation>,
    counts: Res<GpuCompletions>,
    mut start: Local<Option<(Instant, u64, u64)>>,
    mut exit: MessageReader<AppExit>,
) {
    if validation.ready() && start.is_none() {
        *start = Some((
            Instant::now(),
            counts.submitted.load(Ordering::Relaxed),
            counts.completed.load(Ordering::Relaxed),
        ));
    }
    if exit.read().next().is_none() {
        return;
    }
    let Some((started, submitted_start, completed_start)) = *start else {
        return;
    };
    let elapsed = started.elapsed().as_secs_f64();
    let submitted_end = counts.submitted.load(Ordering::Relaxed);
    let completed_end = counts.completed.load(Ordering::Relaxed);
    let completed = completed_end.saturating_sub(completed_start);
    let receipt = format!(
        "observation_seconds={elapsed:.6}\nsubmitted_start={submitted_start}\nsubmitted_end={submitted_end}\ncompleted_start={completed_start}\ncompleted_end={completed_end}\ncompleted_in_interval={completed}\ncompleted_per_second={:.3}\noutstanding_start={}\noutstanding_end={}\npoll_errors={}\nmeasurement=queue-completion callbacks; not GPU timestamp duration or display-presented FPS\n",
        completed as f64 / elapsed.max(f64::MIN_POSITIVE),
        submitted_start.saturating_sub(completed_start),
        submitted_end.saturating_sub(completed_end),
        counts.poll_errors.load(Ordering::Relaxed)
    );
    if let Err(error) = std::fs::write(
        validation.path.with_file_name("gpu-completions.txt"),
        receipt,
    ) {
        eprintln!("GPU completion receipt failed: {error}");
    }
}

// A headless image-target benchmark can use the same update policy as normal
// focused play without forging focus or changing the fixed game tick.
fn apply_update_policy(settings: &mut WinitSettings, policy: &str) -> Result<()> {
    match policy {
        "production" => {}
        "focused" => settings.unfocused_mode = settings.focused_mode,
        _ => anyhow::bail!("CRYSTAL_RENDER_TEST_UPDATE_POLICY must be production or focused"),
    }
    Ok(())
}

#[derive(Resource)]
pub(super) struct TimingValidation {
    started: Instant,
    path: PathBuf,
    completion: Arc<Mutex<Option<std::result::Result<(), String>>>>,
    requested: bool,
    accepted: bool,
    drain_frames: usize,
    image_target_mode: bool,
    image_target: Option<bevy::camera::ImageRenderTarget>,
    image_size: UVec2,
}

impl TimingValidation {
    pub(super) fn ready(&self) -> bool {
        self.accepted && self.drain_frames >= DRAIN_FRAMES
    }
}

pub(super) fn install(app: &mut App, directory: &Path) -> Result<()> {
    let image_target_mode = match std::env::var("CRYSTAL_RENDER_TEST_CAPTURE_TARGET") {
        Err(std::env::VarError::NotPresent) => false,
        Ok(value) if value == "window" => false,
        Ok(value) if value == "image" => true,
        _ => anyhow::bail!("CRYSTAL_RENDER_TEST_CAPTURE_TARGET must be window or image"),
    };
    let policy = match std::env::var("CRYSTAL_RENDER_TEST_UPDATE_POLICY") {
        Err(std::env::VarError::NotPresent) => "production".to_owned(),
        Ok(value) => value,
        Err(error) => return Err(error.into()),
    };
    apply_update_policy(
        &mut app.world_mut().resource_mut::<WinitSettings>(),
        &policy,
    )?;
    std::fs::write(
        directory.join("update-policy.txt"),
        format!("requested_update_policy={policy}\nfixed_game_tick_seconds={GAME_TICK_SECONDS}\n"),
    )?;
    let completions = GpuCompletions::default();
    let render = app
        .get_sub_app_mut(bevy::render::RenderApp)
        .context("validated timing requires the GPU render application")?;
    render.insert_resource(completions.clone()).add_systems(
        bevy::render::Render,
        track_gpu_completion.in_set(bevy::render::RenderSystems::Cleanup),
    );
    app.insert_resource(completions)
        .add_systems(PostUpdate, report_gpu_completion);
    app.insert_resource(TimingValidation {
        started: Instant::now(),
        path: directory.join("validated-warmup.png"),
        completion: default(),
        requested: false,
        accepted: false,
        drain_frames: 0,
        image_target_mode,
        image_target: None,
        image_size: UVec2::ZERO,
    })
    .add_systems(
        Update,
        (prepare_image_target, validate)
            .chain()
            .after(crystal_render_api::WorldRenderSet::RenderSync),
    );
    Ok(())
}

fn prepare_image_target(
    mut validation: ResMut<TimingValidation>,
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
    mut targets: Query<(Entity, &mut bevy::camera::RenderTarget, Has<Camera2d>), With<Camera>>,
    mut commands: Commands,
    mut exit: MessageWriter<AppExit>,
) {
    if !validation.image_target_mode {
        return;
    }
    let TimingValidation {
        image_target,
        image_size,
        ..
    } = &mut *validation;
    if super::render_capture::prepare_image_target_for_cameras(
        image_target,
        image_size,
        &windows,
        &mut images,
        &mut targets,
        &mut commands,
    ) {
        if validation.accepted {
            eprintln!("3D timing rejected: GPU target size or scale changed after validation");
            exit.write(AppExit::error());
        }
        validation.drain_frames = 0;
    }
}

fn selected_screenshot(
    validation: &TimingValidation,
    window: Option<Entity>,
) -> Option<Screenshot> {
    if validation.image_target_mode {
        validation
            .image_target
            .as_ref()
            .map(|target| Screenshot(bevy::camera::RenderTarget::Image(target.clone())))
    } else {
        window.map(Screenshot::window)
    }
}

fn validate(
    mut validation: ResMut<TimingValidation>,
    status: Res<crystal_voxel_view::VoxelViewStatus>,
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    settings: Res<WinitSettings>,
    mut commands: Commands,
    mut exit: MessageWriter<AppExit>,
) {
    if validation.ready() {
        return;
    }
    if validation.started.elapsed() > Duration::from_secs(180) {
        eprintln!("3D timing rejected: no validated rendered scene within 180 seconds");
        exit.write(AppExit::error());
        return;
    }
    let completed = validation.completion.lock().unwrap().take();
    if let Some(result) = completed {
        match result.and_then(|()| {
            validate_render_test_screenshot(&validation.path).map_err(|error| error.to_string())
        }) {
            Ok(()) => {
                validation.accepted = true;
                validation.drain_frames = 0;
                let result = validation.path.with_extension("txt");
                let target = if validation.image_target_mode {
                    "image"
                } else {
                    "window"
                };
                let focused = windows.single().ok().map(|(_, window)| window.focused);
                let receipt = format!(
                    "validated_gpu_readback=true\nreadback=before timing\npost_readback_drain_frames=30\ntarget={target}\nwindow_focused={focused:?}\nfocused_update_mode={:?}\nunfocused_update_mode={:?}\n",
                    settings.focused_mode, settings.unfocused_mode
                );
                if let Err(error) = std::fs::write(result, receipt) {
                    eprintln!("3D timing validation output failed: {error}");
                    exit.write(AppExit::error());
                    return;
                }
            }
            Err(error) => {
                eprintln!("3D timing warmup rejected: {error}");
                validation.requested = false;
                validation.drain_frames = 0;
            }
        }
    }
    if validation.accepted {
        validation.drain_frames += 1;
        return;
    }
    validation.drain_frames += 1;
    if validation.requested
        || validation.drain_frames < DRAIN_FRAMES
        || !status.active
        || status.profiles_pending
        || status.active_frames < 30
    {
        return;
    }
    let Some(screenshot) =
        selected_screenshot(&validation, windows.single().ok().map(|(entity, _)| entity))
    else {
        return;
    };
    let path = validation.path.clone();
    let completion = validation.completion.clone();
    commands
        .spawn(screenshot)
        .observe(move |event: On<ScreenshotCaptured>| {
            let result = event
                .image
                .clone()
                .try_into_dynamic()
                .map_err(|error| format!("decode GPU readback: {error}"))
                .and_then(|image| {
                    image
                        .to_rgb8()
                        .save(&path)
                        .map_err(|error| error.to_string())
                });
            *completion.lock().unwrap() = Some(result);
        });
    validation.requested = true;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gpu_receipt_counts_only_the_ready_interval_and_retains_outstanding_work() {
        let directory =
            std::env::temp_dir().join(format!("geothite-gpu-receipt-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();
        let counts = GpuCompletions::default();
        counts.submitted.store(5, Ordering::Relaxed);
        counts.completed.store(3, Ordering::Relaxed);
        let mut app = App::new();
        app.add_message::<AppExit>()
            .insert_resource(counts.clone())
            .insert_resource(TimingValidation {
                started: Instant::now(),
                path: directory.join("validated-warmup.png"),
                completion: default(),
                requested: true,
                accepted: false,
                drain_frames: DRAIN_FRAMES,
                image_target_mode: true,
                image_target: None,
                image_size: UVec2::ZERO,
            })
            .add_systems(PostUpdate, report_gpu_completion);
        app.update();
        app.world_mut().resource_mut::<TimingValidation>().accepted = true;
        app.update();
        counts.submitted.store(10, Ordering::Relaxed);
        counts.completed.store(8, Ordering::Relaxed);
        app.world_mut().write_message(AppExit::Success);
        app.update();
        let receipt = std::fs::read_to_string(directory.join("gpu-completions.txt")).unwrap();
        assert!(receipt.contains("submitted_start=5\n"));
        assert!(receipt.contains("completed_start=3\n"));
        assert!(receipt.contains("completed_in_interval=5\n"));
        assert!(receipt.contains("outstanding_start=2\n"));
        assert!(receipt.contains("outstanding_end=2\n"));
        assert!(receipt.contains("poll_errors=0\n"));
        std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn focused_policy_changes_only_the_opt_in_unfocused_update_mode() {
        let mut settings = low_power_unfocused_game_winit_settings();
        let production = low_power_unfocused_game_winit_settings();
        apply_update_policy(&mut settings, "production").unwrap();
        assert_eq!(settings.unfocused_mode, production.unfocused_mode);
        apply_update_policy(&mut settings, "focused").unwrap();
        assert_eq!(settings.focused_mode, production.focused_mode);
        assert_eq!(settings.unfocused_mode, production.focused_mode);
        assert!(apply_update_policy(&mut settings, "invalid").is_err());
    }
    #[test]
    fn warmup_uses_the_real_capture_verifier_to_reject_blank_pixels() {
        let path = std::env::temp_dir().join(format!(
            "geothite-timing-warmup-{}.png",
            uuid::Uuid::new_v4()
        ));
        image::RgbImage::from_pixel(768, 576, image::Rgb([100, 100, 100]))
            .save(&path)
            .unwrap();
        assert!(validate_render_test_screenshot(&path).is_err());
        let mut image = image::RgbImage::new(768, 576);
        let colors = [[0, 0, 0], [255, 255, 255], [0, 255, 0], [255, 0, 0]];
        for (x, _, pixel) in image.enumerate_pixels_mut() {
            *pixel = image::Rgb(colors[x as usize % colors.len()]);
        }
        image.save(&path).unwrap();
        assert!(validate_render_test_screenshot(&path).is_ok());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn timing_requires_an_accepted_readback_and_a_completed_drain() {
        let mut validation = TimingValidation {
            started: Instant::now(),
            path: default(),
            completion: default(),
            requested: true,
            accepted: false,
            drain_frames: 100,
            image_target_mode: false,
            image_target: None,
            image_size: UVec2::ZERO,
        };
        assert!(
            !validation.ready(),
            "updates cannot substitute for rendered evidence"
        );
        validation.accepted = true;
        validation.drain_frames = 0;
        assert!(
            !validation.ready(),
            "readback/encoding latency must stay outside timing"
        );
        validation.drain_frames = DRAIN_FRAMES;
        assert!(validation.ready());
    }

    #[test]
    fn timing_readback_retains_image_scale_and_never_substitutes_a_window() {
        let window = Entity::from_bits(1);
        let mut validation = TimingValidation {
            started: Instant::now(),
            path: default(),
            completion: default(),
            requested: false,
            accepted: false,
            drain_frames: 0,
            image_target_mode: true,
            image_target: None,
            image_size: UVec2::ZERO,
        };
        assert!(selected_screenshot(&validation, Some(window)).is_none());
        let target = bevy::camera::ImageRenderTarget {
            handle: Handle::default(),
            scale_factor: 2.0,
        };
        validation.image_target = Some(target.clone());
        assert!(matches!(selected_screenshot(&validation, None).unwrap().0,
            bevy::camera::RenderTarget::Image(actual) if actual == target));
        validation.image_target_mode = false;
        assert!(
            matches!(selected_screenshot(&validation, Some(window)).unwrap().0,
            bevy::camera::RenderTarget::Window(bevy::window::WindowRef::Entity(actual)) if actual == window)
        );
    }
}
