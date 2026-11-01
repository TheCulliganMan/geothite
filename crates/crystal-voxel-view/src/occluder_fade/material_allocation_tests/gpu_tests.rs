//! First-frame rendering of a material created by the production fade system.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use bevy::app::{App, PluginGroup, PluginsState};
use bevy::asset::{Assets, Handle, load_internal_asset};
use bevy::camera::visibility::{RenderLayers, ViewVisibility, Visibility};
use bevy::camera::{Camera, Camera3d, ClearColor, RenderTarget};
use bevy::color::Color;
use bevy::core_pipeline::core_3d::Transparent3d;
use bevy::core_pipeline::prepass::{DepthPrepass, NormalPrepass};
use bevy::ecs::change_detection::Mut;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::ChildOf;
use bevy::image::Image;
use bevy::math::Vec3;
use bevy::mesh::Mesh;
use bevy::pbr::{MaterialPlugin, PreparedMaterial};
use bevy::prelude::{DefaultPlugins, Msaa, On};
use bevy::render::RenderApp;
use bevy::render::erased_render_asset::ErasedRenderAssets;
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
use bevy::render::render_phase::ViewSortedRenderPhases;
use bevy::render::render_resource::{PipelineCache, TextureFormat};
use bevy::render::sync_world::MainEntity;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::render::view::{DebandDither, Tonemapping};
use bevy::shader::Shader;
use bevy::time::TimeUpdateStrategy;
use bevy::transform::components::Transform;
use bevy::window::{ExitCondition, WindowPlugin};

use crate::occluder_fade::{self, OccluderGroup};
use crate::{TerrainRevisionCache, VoxelMaterial, VoxelViewStatus};

fn finish_capture(app: &mut App, image: &Handle<Image>, deadline: Instant) -> Image {
    let result = Arc::new(Mutex::new(None));
    let completion = Arc::clone(&result);
    app.world_mut()
        .spawn(Screenshot::image(image.clone()))
        .observe(move |event: On<ScreenshotCaptured>| {
            *completion.lock().expect("screenshot result lock") = Some(event.image.clone());
        });
    await_capture(app, &result, deadline)
}

fn await_capture(app: &mut App, result: &Mutex<Option<Image>>, deadline: Instant) -> Image {
    loop {
        if let Some(image) = result.lock().expect("screenshot result lock").take() {
            return image;
        }
        assert!(
            Instant::now() < deadline,
            "offscreen screenshot did not complete"
        );
        app.update();
        std::thread::yield_now();
    }
}

fn rgba_pixels(image: Image) -> Vec<u8> {
    image
        .try_into_dynamic()
        .expect("decode actual GPU readback")
        .to_rgba8()
        .into_raw()
}

fn occupied_pixels(pixels: &[u8]) -> usize {
    pixels
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|pixel| pixel[..3].iter().any(|&value| value > 8))
        .count()
}

#[test]
#[ignore = "requires a native GPU; checks the first late blend bind group, queue and actual pixels"]
fn regional_material_first_fade_renders_with_the_prepared_pipeline() {
    let library = crate::forest_instances::Library::default();
    let combined = library
        .prepare(
            [-16.0, 16.0, -16.0, 16.0],
            0.0,
            16.0,
            [4, 8],
            [false; 4],
            crate::foliage::GroveFamily::JohtoPine,
        )
        .into_iter()
        .find(|group| group.opaque_leaf.is_some() && group.overlay.is_none())
        .expect("production canopy must exercise late sorted-mesh upload");
    for mut recipe in [super::single_leaf_recipe(), combined] {
        // Exercise the bank-conforming path that retains its sorted children.
        // The nonregional control therefore keeps the original eager material.
        recipe.runtime_lazy_sorted = false;
        first_fade_case(recipe);
    }
}

fn first_fade_case(recipe: occluder_fade::PreparedGroup) {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..Default::default()
            })
            .disable::<PipelinedRenderingPlugin>()
            .disable::<bevy::log::LogPlugin>(),
    )
    .add_plugins(MaterialPlugin::<VoxelMaterial>::default())
    .init_resource::<TerrainRevisionCache>()
    .insert_resource(VoxelViewStatus {
        active: true,
        ..Default::default()
    })
    .insert_resource(ClearColor(Color::BLACK))
    .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        1.0 / 60.0,
    )));
    load_internal_asset!(
        app,
        crate::VOXEL_SURFACE_SHADER_HANDLE,
        "../../voxel_surface.wesl",
        Shader::from_wesl
    );
    load_internal_asset!(
        app,
        crate::GRASS_VERTEX_SHADER_HANDLE,
        "../../grass_vertex.wesl",
        Shader::from_wesl
    );
    occluder_fade::register(&mut app);
    crate::pipeline_preload::register(&mut app);
    let deadline = Instant::now() + Duration::from_secs(60);
    while app.plugins_state() != PluginsState::Ready {
        assert!(Instant::now() < deadline, "GPU plugins did not initialize");
        bevy::tasks::tick_global_task_pools_on_main_thread();
        std::thread::yield_now();
    }
    app.finish();
    app.cleanup();
    let (low, high) = occluder_fade::prepared_bounds(std::slice::from_ref(&recipe))
        .expect("authentic plant bounds");
    let center = (low + high) * 0.5;
    let radius = (high - low).max_element().max(1.0);
    let target = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::new_target_texture(
            64,
            64,
            TextureFormat::Rgba8UnormSrgb,
            None,
        ));
    app.world_mut().spawn((
        Camera3d::default(),
        Camera::default(),
        crate::VoxelWorldCamera,
        crate::pipeline_preload::WarmupView,
        DepthPrepass,
        NormalPrepass,
        Msaa::Off,
        Tonemapping::None,
        DebandDither::Disabled,
        RenderLayers::from_layers(&[0, crate::VOXEL_RENDER_LAYER]),
        RenderTarget::Image(target.clone().into()),
        Transform::from_translation(center + Vec3::new(0.0, radius * 1.5, radius * 3.0))
            .looking_at(center, Vec3::Y),
    ));
    let root = app
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Visibility::Hidden,
            crate::pipeline_preload::HiddenDestination,
        ))
        .id();
    let eager_root = app
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Visibility::Hidden,
            crate::pipeline_preload::HiddenDestination,
        ))
        .id();
    let opaque = app
        .world_mut()
        .resource_mut::<Assets<VoxelMaterial>>()
        .add(crate::solid_terrain_material());
    let (leaves, eager_leaves) =
        app.world_mut()
            .resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
                world.resource_scope(|world, mut materials: Mut<Assets<VoxelMaterial>>| {
                    let leaves = occluder_fade::spawn_region_groups(
                        &mut world.commands(),
                        &mut meshes,
                        &mut materials,
                        root,
                        vec![recipe.clone()],
                        opaque.clone(),
                    );
                    // Ordinary single-leaf groups preserve the pre-change eager blend
                    // policy, providing an identical source/material control here.
                    let eager = occluder_fade::spawn_groups(
                        &mut world.commands(),
                        &mut meshes,
                        &mut materials,
                        eager_root,
                        vec![recipe],
                        opaque,
                        None,
                    );
                    (leaves, eager)
                })
            });
    assert_eq!(leaves.len(), eager_leaves.len());
    assert!(!leaves.is_empty());
    app.world_mut().flush();
    let leaf = leaves
        .iter()
        .copied()
        .find(|&entity| app.world().get::<bevy::mesh::Mesh3d>(entity).is_some())
        .expect("initial opaque mesh");
    let fade_leaf = leaves
        .iter()
        .copied()
        .find(|&entity| {
            app.world()
                .get::<occluder_fade::SortedObjectLeaf>(entity)
                .is_some()
        })
        .unwrap_or(leaf);
    let group: Entity = app
        .world()
        .get::<ChildOf>(leaf)
        .expect("regional leaf parent")
        .parent();
    let eager_group = app
        .world()
        .get::<ChildOf>(eager_leaves[0])
        .expect("eager leaf parent")
        .parent();
    let mut settled = 0;
    let mut frames = 0;
    loop {
        assert!(
            Instant::now() < deadline,
            "hidden fade pipelines did not prepare"
        );
        app.update();
        frames += 1;
        let waiting = app
            .sub_app(RenderApp)
            .world()
            .resource::<PipelineCache>()
            .waiting_pipelines()
            .next()
            .is_some();
        settled = if waiting { 0 } else { settled + 1 };
        if frames >= 120 && settled >= 16 {
            break;
        }
        std::thread::yield_now();
    }
    assert!(
        !app.world()
            .get::<OccluderGroup>(group)
            .expect("clear owner")
            .transparent_ready
    );
    *app.world_mut()
        .get_mut::<Visibility>(root)
        .expect("staged root") = Visibility::Inherited;
    for _ in 0..4 {
        app.update();
    }
    assert!(
        app.world()
            .get::<ViewVisibility>(leaf)
            .expect("visible source mesh")
            .get()
    );
    let opaque_image = rgba_pixels(finish_capture(&mut app, &target, deadline));
    let opaque_pixels = occupied_pixels(&opaque_image);
    assert!(
        opaque_pixels >= 8,
        "the actual source plant must be drawn before fading"
    );
    let capture = Arc::new(Mutex::new(None));
    let completion = Arc::clone(&capture);
    app.world_mut()
        .spawn(Screenshot::image(target.clone()))
        .observe(move |event: On<ScreenshotCaptured>| {
            *completion.lock().expect("first fade screenshot lock") = Some(event.image.clone());
        });
    app.world_mut()
        .get_mut::<OccluderGroup>(group)
        .expect("first fade owner")
        .fade
        .opacity = 0.5;
    app.update();
    // This is the very first update that creates the blend. No render warmup
    // or retry is allowed between allocation and these preparation assertions.
    let blend = app
        .world()
        .get::<OccluderGroup>(group)
        .expect("allocated blend owner")
        .transparent
        .id();
    let render = app.sub_app(RenderApp).world();
    assert!(
        render
            .resource::<ErasedRenderAssets<PreparedMaterial>>()
            .get(blend)
            .is_some(),
        "the late blend bind group must be prepared in its first render frame"
    );
    let main: MainEntity = fade_leaf.into();
    let phases = render.resource::<ViewSortedRenderPhases<Transparent3d>>();
    let item = phases
        .values()
        .flat_map(|phase| phase.items.values())
        .find(|item| item.entity.1 == main)
        .expect("first fade must enter the real transparent draw queue");
    assert!(
        render
            .resource::<PipelineCache>()
            .get_render_pipeline(item.pipeline)
            .is_some(),
        "first fade must reuse an already prepared hardware pipeline"
    );
    let first_fade = rgba_pixels(await_capture(&mut app, &capture, deadline));
    assert!(
        occupied_pixels(&first_fade) >= 8,
        "actual first-fade pixels must survive late material allocation"
    );
    *app.world_mut()
        .get_mut::<Visibility>(root)
        .expect("regional root") = Visibility::Hidden;
    *app.world_mut()
        .get_mut::<Visibility>(eager_root)
        .expect("eager root") = Visibility::Inherited;
    app.world_mut()
        .get_mut::<OccluderGroup>(eager_group)
        .expect("eager control owner")
        .fade
        .opacity = 0.5;
    let eager_fade = rgba_pixels(finish_capture(&mut app, &target, deadline));
    assert_eq!(
        first_fade, eager_fade,
        "late allocation must preserve every first-fade RGBA pixel from the eager control"
    );
}
