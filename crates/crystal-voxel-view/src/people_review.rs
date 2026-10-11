//! Offline visual QA: the same GPU skins and costumes as the game, without a controller.
use bevy::camera::visibility::RenderLayers;
use crate::{
    VOXEL_RENDER_LAYER,
    new_bark_actors::character_meshes::{CharacterKind, SOURCE_KINDS},
    open_people_skin::{PeopleGpu, Person},
    new_bark_actors::ModeledActor,
};
use bevy::camera::ScalingMode;
use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    prelude::*,
    render::{

        mesh::skinning::SkinnedMeshInverseBindposes,
        view::screenshot::{Screenshot, ScreenshotCaptured},
    },
    window::PrimaryWindow,
    winit::{UpdateMode, WinitSettings},
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
    people: Vec<Person>,
    run_phase: Option<f32>,
    head_scale: Option<f32>,
    hand_scale: Option<f32>,
    unlit: bool,
    gpu: PeopleGpu,
    frame: u32,
    pending: bool,
    blank_attempts: u32,
    result: Arc<Mutex<Option<Result<CaptureOutcome, String>>>>,
}
enum CaptureOutcome { Saved, Blank }

#[derive(Resource)]
struct ReviewTarget(Option<Handle<Image>>);

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
    let run_phase = std::env::args().nth(3).map(|value| {
        let phase: f32 = value.parse().expect("run phase must be a number from 0 to 1");
        assert!(phase.is_finite() && (0.0..=1.0).contains(&phase), "run phase must be from 0 to 1");
        phase * std::f32::consts::TAU
    });
    // Export precisely the production paintings, never a legacy source face.
    if std::env::var_os("CRYSTAL_PEOPLE_REVIEW_EXPORT_ATLASES").is_some() {
        let atlases = output.join("atlases");
        std::fs::create_dir_all(&atlases).expect("create atlas review directory");
        for &kind in &kinds {
            let look = crate::npc_appearance::look(kind);
            let painting = crate::npc_textures::get(look.texture_key)
                .expect("required production character painting");
            let size = painting.size;
            let pixels = &painting.rgba;
            image::save_buffer(atlases.join(format!("{}.png", look.texture_key)),
                &pixels, size.0, size.1, image::ColorType::Rgba8).expect("write review atlas");
        }
    }
    let mut app = App::new();
    app
        // Offline captures must wait for their render pipelines; counting
        // update frames alone can finish before asynchronous compilation.
        .add_plugins(DefaultPlugins.set(bevy::render::RenderPlugin {
            synchronous_pipeline_compilation: true,
            ..default()
        }).set(WindowPlugin {
            primary_window: Some(Window {
                title: "NPC angle review".into(),
                resolution: (1600, 600).into(),
                ..default()
            }),
            ..default()
        }))
        // Screenshot batches must keep settling and reading back when another
        // window has focus; this setting belongs only to the offline reviewer.
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::Continuous,
            unfocused_mode: UpdateMode::Continuous,
        })
        .insert_resource(Review {
            output,
            kinds,
            index: 0,
            roots: vec![],
            people: vec![],
            run_phase,
            head_scale: std::env::var("CRYSTAL_PEOPLE_REVIEW_HEAD_SCALE").ok().map(|value| {
                let scale: f32 = value.parse().expect("head scale must be a number");
                assert!(scale.is_finite() && (0.5..=1.0).contains(&scale),
                    "review head scale must be from 0.5 to 1.0");
                scale
            }),
            hand_scale: std::env::var("CRYSTAL_PEOPLE_REVIEW_HAND_SCALE").ok().map(|value| {
                let scale: f32 = value.parse().expect("hand scale must be a number");
                assert!(scale.is_finite() && (0.5..=1.0).contains(&scale),
                    "review hand scale must be from 0.5 to 1.0");
                scale
            }),
            unlit: std::env::var_os("CRYSTAL_PEOPLE_REVIEW_UNLIT").is_some(),
            gpu: default(),
            frame: 0,
            pending: false,
            blank_attempts: 0,
            result: default(),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, capture);
    if std::env::var_os("CRYSTAL_PEOPLE_REVIEW_TRACE").is_some() {
        app.get_sub_app_mut(bevy::render::RenderApp)
            .expect("review requires the render app")
            .add_systems(bevy::render::Render,
                trace_render_state.in_set(bevy::render::RenderSystems::PostCleanup));
    }
    app.run();
}

/// One bounded diagnostic snapshot from the render world, opt-in and confined
/// to the offline reviewer. It distinguishes missing camera/output extraction
/// from unfinished or failed pipelines without tracing gameplay or TUI paths.
fn trace_render_state(
    mut frame: Local<u32>,
    cameras: Query<(&bevy::render::camera::ExtractedCamera, Option<&bevy::render::view::ViewTarget>)>,
    pipelines: Res<bevy::render::render_resource::PipelineCache>,
) {
    *frame += 1;
    if *frame != 30 { return; }
    use bevy::render::render_resource::CachedPipelineState;
    let mut counts = [0; 4];
    for pipeline in pipelines.pipelines() {
        counts[match &pipeline.state {
            CachedPipelineState::Queued => 0,
            CachedPipelineState::Creating(_) => 1,
            CachedPipelineState::Ok(_) => 2,
            CachedPipelineState::Err(_) => 3,
        }] += 1;
        if let CachedPipelineState::Err(error) = &pipeline.state {
            eprintln!("NPC review pipeline error: {error:?}");
        }
    }
    eprintln!("NPC review pipelines queued={} creating={} ready={} errors={}; cameras={}",
        counts[0], counts[1], counts[2], counts[3], cameras.iter().count());
    for (camera, view) in &cameras {
        eprintln!("NPC review camera target={:?} size={:?} viewport={:?} view={} output={}",
            camera.target, camera.physical_target_size, camera.physical_viewport_size,
            view.is_some(), view.is_some_and(|view| view.out_texture().is_some()));
    }
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // Diagnostic alternative to the window swapchain. It uses the same
    // camera, meshes and materials, and never changes production rendering.
    let image = std::env::var_os("CRYSTAL_PEOPLE_REVIEW_IMAGE_TARGET").map(|_| {
        images.add(Image::new_target_texture(1600, 600,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb, None))
    });
    let target = image.clone().map(|handle| bevy::camera::RenderTarget::Image(handle.into()))
        .unwrap_or_default();
    commands.insert_resource(ReviewTarget(image));
    commands.spawn((
        (Camera3d::default(), Msaa::Off,
Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::Fixed {
                    width: 6.4,
                    height: 2.4,
                },
                near: 0.1,
                far: 100.,
                ..OrthographicProjection::default_3d()
            }),
Transform::from_xyz(0., 1.4, 8.)
                .looking_at(Vec3::new(0., 0.85, 0.), Vec3::Y),
Camera {
                clear_color: ClearColorConfig::Custom(Color::srgb(0.87, 0.86, 0.82)),
                ..default()
            },
Tonemapping::None,
DebandDither::Disabled,),
        RenderLayers::layer(VOXEL_RENDER_LAYER),
        target,
    ));
    commands.spawn((
        (DirectionalLight {
                illuminance: 3000.,
                shadow_maps_enabled: true,
                ..default()
            },
Transform::from_xyz(180., 320., 220.).looking_at(Vec3::ZERO, Vec3::Y),),
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
    target: Res<ReviewTarget>,
    mut transforms: Query<&mut Transform, With<ModeledActor>>,

    mut exit: MessageWriter<AppExit>,
) {
    let completed = review.result.lock().unwrap().take();
    if let Some(result) = completed {
        match result {
            Ok(CaptureOutcome::Blank) if review.blank_attempts < 60 => {
                review.blank_attempts += 1;
                review.pending = false;
                review.frame = 0;
                return;
            }
            Ok(CaptureOutcome::Blank) => {
                eprintln!("NPC capture failed: renderer remained blank after 60 retries");
                exit.write(AppExit::error());
                return;
            }
            Err(error) => {
                eprintln!("NPC capture failed: {error}");
                exit.write(AppExit::error());
                return;
            }
            Ok(CaptureOutcome::Saved) => {}
        }
        for root in review.roots.drain(..) {
            commands.entity(root).despawn();
        }
        review.people.clear();
        review.index += 1;
        review.pending = false;
        review.frame = 0;
        review.blank_attempts = 0;
    }
    if review.index == review.kinds.len() {
        exit.write(AppExit::Success);
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
                .spawn((Visibility::default(),
Transform::from_xyz(-2.4 + i as f32 * 1.6, 0., 0.)
                        .with_rotation(Quat::from_rotation_y(yaw))
                        .with_scale(Vec3::new(scale * look.width, scale, scale)),))
                .id();
            let person = review.gpu.spawn(
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
            review.people.push(person);
            review.roots.push(root);
        }
        // Albedo-only diagnostic separates painting/UV defects from lighting.
        // Production materials remain unchanged outside this offline reviewer.
        if review.unlit {
            for (_, material) in materials.iter_mut() {
                material.unlit = true;
            }
        }
    }
    let run_phase = review.run_phase;
    let head_scale = review.head_scale;
    let hand_scale = review.hand_scale;
    if run_phase.is_some() || head_scale.is_some() || hand_scale.is_some() {
        for person in &mut review.people {
            // Resample before every experiment so scale never compounds across
            // frames, including idle captures with an overridden head size.
            person.update(run_phase.unwrap_or(0.),
                if run_phase.is_some() { 1. } else { 0. }, 0.0, &mut transforms);
            if let Some(scale) = head_scale {
                person.review_head_scale(scale, &mut transforms);
            }
            if let Some(scale) = hand_scale {
                person.review_hand_scale(scale, &mut transforms);
            }
        }
    }
    review.frame += 1;
    if review.pending || review.frame < 30 {
        return;
    }
    let key = crate::npc_appearance::look(review.kinds[review.index]).texture_key;
    let path = review.output.join(format!("{key}.png"));
    let completion = review.result.clone();
    let screenshot = if let Some(image) = &target.0 {
        Screenshot::image(image.clone())
    } else {
        let Ok(window) = windows.single() else { return; };
        Screenshot::window(window)
    };
    commands.spawn(screenshot).observe(move |event: On<ScreenshotCaptured>| {
            let result = event.image.clone()
                .try_into_dynamic()
                .map_err(|e| e.to_string())
                .and_then(|im| {
                    let pixels = im.to_rgb8();
                    // Reject black frames and a lone camera clear color alike.
                    // This only filters unusable flat captures; all model
                    // fitting and art-direction checks still require review.
                    let first = pixels.get_pixel(0, 0).0;
                    if pixels.pixels().all(|pixel| pixel.0 == first) {
                        return Ok(CaptureOutcome::Blank);
                    }
                    pixels.save(&path).map_err(|e| e.to_string())?;
                    Ok(CaptureOutcome::Saved)
                });
            *completion.lock().unwrap() = Some(result);
        });
    review.pending = true;
}
