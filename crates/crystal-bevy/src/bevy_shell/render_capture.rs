//! Automated captures share a GPU session and finish on readback, not a timer.
use super::*;
use std::sync::{Arc, Mutex};

type Completion = Arc<Mutex<Option<Result<(), String>>>>;

#[derive(Resource)]
struct Capture {
    path: PathBuf,
    second: Option<PathBuf>,
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
}

pub(super) fn install(
    app: &mut App,
    path: PathBuf,
    second: Option<PathBuf>,
    live: bool,
) -> Result<()> {
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
        path,
        second,
        frame: 0,
        requested: false,
        completion: default(),
        started: Instant::now(),
    })
    .add_systems(Update, capture);
    Ok(())
}

fn capture(
    mut capture: ResMut<Capture>,
    runtime: Res<BevyRuntimeShell>,
    walk: Option<Res<render_walk::RenderWalk>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    frame: Res<crystal_render_api::VisualWorldFrame>,
    status: Res<crystal_voxel_view::VoxelViewStatus>,
    mut settings: ResMut<crystal_voxel_view::VoxelViewSettings>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    mut screenshots: ResMut<ScreenshotManager>,
    mut exit: EventWriter<AppExit>,
) {
    if let Some(error) = runtime.render_test_error.as_ref() {
        eprintln!("capture rejected after runtime error: {error}");
        exit.send(AppExit::error());
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
                    exit.send(AppExit::error());
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
            exit.send(AppExit::error());
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
            settings.enabled = !settings.enabled;
        } else if capture.live {
            if let Err(error) = live_comparison(&mut capture) {
                eprintln!("review failed: {error:#}");
                exit.send(AppExit::error());
                return;
            }
            capture.idle = true;
            println!(
                "review ready in {:.2}s; session stays open — F6 to capture again",
                capture.started.elapsed().as_secs_f64()
            );
        } else {
            exit.send(AppExit::Success);
        }
        return;
    }
    // Bound startup/readback failures; ordinary walking has its own duration.
    if capture.started.elapsed().as_secs() > 60 && walk.is_none() {
        eprintln!("capture timed out: {:?}", status.inactive_reason);
        exit.send(AppExit::error());
        return;
    }
    capture.frame = capture.frame.saturating_add(1);
    // Preserve the established 30 active-frame GPU warmup, but drop the
    // unconditional 90-frame startup and 60-frame post-save waits.
    let settled = if settings.enabled {
        status.active && !status.profiles_pending && status.active_frames >= 30
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
    let Ok(window) = primary_window.get_single() else {
        return;
    };
    if capture.identity.is_empty() {
        capture.identity = format!("{}|{:?}|{:?}", frame.map_id, frame.center, settings.camera);
    }
    let path = capture.path.clone();
    let completion = capture.completion.clone();
    if screenshots
        .take_screenshot(window, move |image| {
            let result = image
                .try_into_dynamic()
                .map_err(|error| format!("decode GPU image: {error}"))
                .and_then(|image| {
                    image
                        .to_rgb8()
                        .save(&path)
                        .map_err(|error| error.to_string())
                });
            *completion.lock().unwrap() = Some(result);
        })
        .is_ok()
    {
        capture.requested = true;
    }
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
