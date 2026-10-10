//! Offline visual QA: the same GPU skins and costumes as the game, without a controller.
use crate::{
    VOXEL_RENDER_LAYER,
    new_bark_actors::character_meshes::{CharacterKind, SOURCE_KINDS},
    open_people_skin::PeopleGpu,
};
use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    prelude::*,
    render::{
        camera::ScalingMode,
        mesh::skinning::SkinnedMeshInverseBindposes,
        view::{RenderLayers, screenshot::ScreenshotManager},
    },
    window::PrimaryWindow,
};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

#[derive(Resource)]
struct Review {
    output: PathBuf,
    kinds: Vec<CharacterKind>,
    index: usize,
    roots: Vec<Entity>,
    gpu: PeopleGpu,
    frame: u32,
    pending: bool,
    result: Arc<Mutex<Option<Result<(), String>>>>,
}

pub fn run() {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .expect("supply an ignored review output directory");
    std::fs::create_dir_all(&output).expect("create review directory");
    let mut kinds: Vec<_> = SOURCE_KINDS.iter().map(|&(_, kind)| kind).collect();
    kinds.push(CharacterKind::Outdoorsman);
    if let Some(filter) = std::env::args().nth(2) {
        kinds.retain(|&kind| {
            filter
                .split(',')
                .any(|key| key == crate::npc_appearance::look(kind).texture_key)
        });
        assert!(!kinds.is_empty(), "filter selected no characters");
    }
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "NPC angle review".into(),
                resolution: (1600., 600.).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Review {
            output,
            kinds,
            index: 0,
            roots: vec![],
            gpu: default(),
            frame: 0,
            pending: false,
            result: default(),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, capture)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            projection: Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::Fixed {
                    width: 6.4,
                    height: 2.4,
                },
                near: 0.1,
                far: 100.,
                ..default()
            }),
            transform: Transform::from_xyz(0., 1.4, 8.)
                .looking_at(Vec3::new(0., 0.85, 0.), Vec3::Y),
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::srgb(0.87, 0.86, 0.82)),
                ..default()
            },
            tonemapping: Tonemapping::None,
            deband_dither: DebandDither::Disabled,
            ..default()
        },
        RenderLayers::layer(VOXEL_RENDER_LAYER),
    ));
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 3000.,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(180., 320., 220.).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        RenderLayers::layer(VOXEL_RENDER_LAYER),
    ));
}

#[allow(clippy::too_many_arguments)]
fn capture(
    mut commands: Commands,
    mut review: ResMut<Review>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut binds: ResMut<Assets<SkinnedMeshInverseBindposes>>,
    windows: Query<Entity, With<PrimaryWindow>>,
    mut screenshots: ResMut<ScreenshotManager>,
    mut exit: EventWriter<AppExit>,
) {
    let completed = review.result.lock().unwrap().take();
    if let Some(result) = completed {
        if let Err(error) = result {
            eprintln!("NPC capture failed: {error}");
            exit.send(AppExit::error());
            return;
        }
        for root in review.roots.drain(..) {
            commands.entity(root).despawn_recursive();
        }
        review.index += 1;
        review.pending = false;
        review.frame = 0;
    }
    if review.index == review.kinds.len() {
        exit.send(AppExit::Success);
        return;
    }
    if review.roots.is_empty() {
        let kind = review.kinds[review.index];
        let look = crate::npc_appearance::look(kind);
        let rig = crate::open_people::get(look.base)
            .expect("review requires the external rig")
            .clone();
        let scale = look.height / rig.height;
        for (i, yaw) in [
            0.,
            -std::f32::consts::FRAC_PI_4,
            -std::f32::consts::FRAC_PI_2,
            std::f32::consts::PI,
        ]
        .into_iter()
        .enumerate()
        {
            let root = commands
                .spawn(SpatialBundle {
                    transform: Transform::from_xyz(-2.4 + i as f32 * 1.6, 0., 0.)
                        .with_rotation(Quat::from_rotation_y(yaw))
                        .with_scale(Vec3::new(scale * look.width, scale, scale)),
                    ..default()
                })
                .id();
            review.gpu.spawn(
                look.base,
                kind,
                rig.clone(),
                root,
                false,
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut images,
                &mut binds,
            );
            review.roots.push(root);
        }
    }
    review.frame += 1;
    if review.pending || review.frame < 30 {
        return;
    }
    let key = crate::npc_appearance::look(review.kinds[review.index]).texture_key;
    let path = review.output.join(format!("{key}.png"));
    let completion = review.result.clone();
    if screenshots
        .take_screenshot(windows.single(), move |image| {
            let result = image
                .try_into_dynamic()
                .map_err(|e| e.to_string())
                .and_then(|im| im.to_rgb8().save(&path).map_err(|e| e.to_string()));
            *completion.lock().unwrap() = Some(result);
        })
        .is_ok()
    {
        review.pending = true;
    }
}
