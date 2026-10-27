//! Offscreen GPU regression for independent source textures and tint transitions.
//! No game pack, window, or saved image is required.
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::asset::RenderAssetUsages;
use bevy::camera::RenderTarget;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::render::{ExtractSchedule, RenderApp};
use bevy::sprite_render::ExtractedSprites;
use bevy::window::ExitCondition;

#[path = "../src/bevy_shell/source_sprite_renderer.rs"]
mod source_sprite_renderer;

const ALPHAS: [f32; 5] = [0.0, 1.0 / 255.0, 0.5, 1.0, 0.0];

#[derive(Resource, Clone, Default)]
struct ResultState(Arc<Mutex<Option<Result<(), String>>>>);

#[derive(Resource, Clone, Default)]
struct ExtractedCount(Arc<AtomicUsize>);

#[derive(Resource)]
struct CaptureTarget(Handle<Image>);

#[derive(Resource, Default)]
struct CaptureState {
    phase: usize,
    pending: bool,
    settled: u32,
}

#[derive(Component)]
struct TintOverlay;

fn main() -> anyhow::Result<()> {
    let result = ResultState::default();
    let count = ExtractedCount::default();
    let mut app = App::new();
    app.insert_resource(result.clone()).insert_resource(count.clone())
        .init_resource::<CaptureState>()
        .add_plugins(
        DefaultPlugins
            .build()
                .disable::<bevy::winit::WinitPlugin>()
                .disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>()
                .set(bevy::render::RenderPlugin {
                synchronous_pipeline_compilation: true,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                ..default()
            }),
    ).add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_millis(1)));
    source_sprite_renderer::install(&mut app);
    app.sub_app_mut(RenderApp)
        .insert_resource(count)
        .add_systems(
            ExtractSchedule,
            count_extracted.after(source_sprite_renderer::extract),
        );
    app.add_systems(Startup, setup)
        .add_systems(Update, capture)
        .run();
    let result = result
        .0
        .lock()
        .expect("GPU result lock")
        .take()
        .unwrap_or_else(|| Err("no GPU readback".into()));
    result.map_err(anyhow::Error::msg)?;
    println!(
        "PASS: 40 source textures and five tint phases have expected extraction counts and GPU colors"
    );
    Ok(())
}

fn count_extracted(sprites: Res<ExtractedSprites>, count: Res<ExtractedCount>) {
    count.0.store(sprites.sprites.len(), Ordering::Relaxed);
}

fn color(index: u32) -> [u8; 4] {
    [
        (index * 71 % 256) as u8,
        (index * 113 % 256) as u8,
        (index * 29 % 256) as u8,
        255,
    ]
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut target = Image::new_target_texture(640, 480, TextureFormat::Rgba8UnormSrgb, None);
    target.texture_descriptor.usage |= TextureUsages::COPY_SRC;
    let target = images.add(target);
    commands.spawn((
        Camera2d,
        Msaa::Off,
        RenderTarget::Image(target.clone().into()),
    ));
    commands.insert_resource(CaptureTarget(target));
    for index in 0..40 {
        let mut image = Image::new(
            Extent3d {
                width: 8,
                height: 8,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            color(index).repeat(64),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        image.sampler = ImageSampler::nearest();
        commands.spawn((
            Sprite {
                image: images.add(image),
                custom_size: Some(Vec2::new(24.0 + index as f32, 24.0)),
                ..default()
            },
            Transform::from_xyz(
                -270.0 + (index % 10) as f32 * 60.0,
                180.0 - (index / 10) as f32 * 100.0,
                0.0,
            ),
        ));
    }
commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 0.0, 0.0, ALPHAS[0]),
            custom_size: Some(Vec2::new(640.0, 480.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 100.0),
        TintOverlay,
    ));
}

fn expected_color(index: u32, alpha: f32) -> [u8; 4] {
    // The direct sprite pipeline blends linear-light samples into sRGB pixels.
    let source = color(index);
    let source = Color::srgb_u8(source[0], source[1], source[2]).to_linear();
    let blended = Color::linear_rgb(
        source.red * (1.0 - alpha) + alpha,
        source.green * (1.0 - alpha),
        source.blue * (1.0 - alpha),
    )
    .to_srgba();
    [
        (blended.red * 255.0).round() as u8,
        (blended.green * 255.0).round() as u8,
        (blended.blue * 255.0).round() as u8,
        255,
    ]
}

fn capture(
    mut commands: Commands,
    mut frame: Local<u32>,
    mut state: ResMut<CaptureState>,
    target: Res<CaptureTarget>,
    result: Res<ResultState>,
    mut exit: MessageWriter<AppExit>,
) {
    *frame += 1;
    if *frame == 3600 {
        *result.0.lock().expect("GPU result lock") = Some(Err("GPU readback timed out".into()));
        exit.write(AppExit::error());
        return;
    }
    if state.pending {
        return;
    }
    state.settled += 1;
    if state.settled < 30 {
        return;
    }
    let phase = state.phase;
    state.pending = true;
        commands.spawn(Screenshot::image(target.0.clone())).observe(
            move |capture: On<ScreenshotCaptured>, mut state: ResMut<CaptureState>,
              count: Res<ExtractedCount>,
              result: Res<ResultState>,
              mut overlays: Query<&mut Sprite, With<TintOverlay>>,
              mut exit: MessageWriter<AppExit>| {
                let checked = (|| -> Result<(), String> {
                let path = std::env::args().nth(1).map(std::path::PathBuf::from);
                if let Some(path) = &path {
                    crystal_bevy::capture_archive::archive_existing(path)
                        .map_err(|error| format!("{error:#}"))?;
                    crystal_bevy::capture_archive::archive_gpu_readback(path, &capture.image)
                        .map_err(|error| format!("{error:#}"))?;
                }
                let original = capture
                    .image
                    .clone()
                    .try_into_dynamic()
                    .map_err(|error| error.to_string())?;
                if let Some(path) = &path {
                    crystal_bevy::capture_archive::archive_capture_image(
                        path,
                        &format!("phase-{phase}-original"),
                        &original,
                    )
                    .map_err(|error| format!("{error:#}"))?;
                }
                let image = original.to_rgba8();
                if let Some(path) = &path {
                    crystal_bevy::capture_archive::archive_capture_image(
                        path,
                        &format!("phase-{phase}-rgba"),
                        &image::DynamicImage::ImageRgba8(image.clone()),
                    )
                    .map_err(|error| format!("{error:#}"))?;
                }
                // Archive requested captures before visual assertions so a
                // rejected phase remains available for diagnosis and reuse.
                let alpha = ALPHAS[phase];
                let expected_count = 40 + usize::from(alpha != 0.0);
                let actual_count = count.0.load(Ordering::Relaxed);
                if actual_count != expected_count {
                    return Err(format!(
                        "phase {phase}: extracted {actual_count} sprites, expected {expected_count}"
                    ));
                }
                for index in 0..40 {
                    let x = (50 + index % 10 * 60) * image.width() / 640;
                    let y = (60 + index / 10 * 100) * image.height() / 480;
                    let actual = image.get_pixel(x, y).0;
                    let expected = expected_color(index, alpha);
                    if actual.iter().zip(expected).any(|(&a, b)| a.abs_diff(b) > 2) {
                        return Err(format!(
                            "phase {phase}, sprite {index}: {actual:?}, expected {expected:?}"
                        ));
                    }
                }
                if phase + 1 == ALPHAS.len()
                    && let Some(path) = &path
                {
                    crystal_bevy::capture_archive::write_capture_png(
                        path,
                        &image::DynamicImage::ImageRgba8(image),
                    )
                    .map_err(|error| format!("{error:#}"))?;
                }
                Ok(())
            })();
            if let Err(error)=checked {
        *result.0.lock().expect("GPU result lock") = Some(Err(error));
                exit.write(AppExit::error());
                return;
            }
            state.phase += 1;
            if state.phase == ALPHAS.len() {
                *result.0.lock().expect("GPU result lock") = Some(Ok(()));
        exit.write(AppExit::Success);
    }
else {
                overlays.single_mut().expect("one tint overlay").color =
                    Color::srgba(1.0, 0.0, 0.0, ALPHAS[state.phase]);
                state.pending = false;
                state.settled = 0;
            }
},
    );
}
