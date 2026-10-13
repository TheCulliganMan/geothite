//! GPU regression for independent source textures and sizes. No game pack needed.
use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::window::WindowResolution;
use std::sync::{Arc, Mutex};

#[path = "../src/bevy_shell/source_sprite_renderer.rs"]
mod source_sprite_renderer;

#[derive(Resource, Clone, Default)]
struct ResultState(Arc<Mutex<Option<Result<(), String>>>>);

fn main() -> anyhow::Result<()> {
    let result = ResultState::default();
    let mut app = App::new();
    app.insert_resource(result.clone()).add_plugins(
        DefaultPlugins
            .set(bevy::render::RenderPlugin {
                synchronous_pipeline_compilation: true,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Source sprite GPU regression".into(),
                    resolution: WindowResolution::new(640, 480),
                    ..default()
                }),
                ..default()
            }),
    );
    source_sprite_renderer::install(&mut app);
    app.add_systems(Startup, setup)
        .add_systems(Update, capture)
        .run();
    let result = result
        .0
        .lock()
        .unwrap()
        .take()
        .unwrap_or_else(|| Err("no GPU readback".into()));
    result.map_err(anyhow::Error::msg)?;
    println!(
        "PASS: all 40 independent source sprite textures retain their GPU positions and colors"
    );
    Ok(())
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
    commands.spawn(Camera2d);
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
}

fn capture(
    mut commands: Commands,
    mut frame: Local<u32>,
    result: Res<ResultState>,
    mut exit: MessageWriter<AppExit>,
) {
    *frame += 1;
    if *frame == 120 {
        let result = result.0.clone();
        commands.spawn(Screenshot::primary_window()).observe(
            move |capture: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
                let image = capture.image.clone().try_into_dynamic().unwrap().to_rgba8();
                let mut errors = Vec::new();
                for index in 0..40 {
                    let x = (50 + index % 10 * 60) * image.width() / 640;
                    let y = (60 + index / 10 * 100) * image.height() / 480;
                    let actual = image.get_pixel(x, y).0;
                    let expected = color(index);
                    if actual.iter().zip(expected).any(|(&a, b)| a.abs_diff(b) > 2) {
                        errors.push(format!("sprite {index}: {actual:?}, expected {expected:?}"));
                    }
                }
                if let Some(path) = std::env::args().nth(1) {
                    image.save(path).unwrap();
                }
                *result.lock().unwrap() = Some(if errors.is_empty() {
                    Ok(())
                } else {
                    Err(errors.join("\n"))
                });
                exit.write(AppExit::Success);
            },
        );
    }
    if *frame == 3600 {
        *result.0.lock().unwrap() = Some(Err("GPU readback timed out".into()));
        exit.write(AppExit::Success);
    }
}
