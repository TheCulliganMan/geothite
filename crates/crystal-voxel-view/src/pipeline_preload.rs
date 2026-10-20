//! Specialize hidden destination materials against the live view before entry.
//! Uses Bevy's ordinary caches; no proxy draw, extra camera, or geometry upload.
use crate::{VoxelMaterial, occluder_fade};
use bevy::{
    asset::AssetId,
    material::{
        MaterialProperties, OpaqueRendererMethod, RenderPhaseType,
        key::{ErasedMaterialKey, ErasedMaterialPipelineKey, ErasedMeshPipelineKey},
    },
    mesh::MeshVertexBufferLayoutRef,
    pbr::{
        LightKeyCache, MeshPipelineKey, PreparedMaterial, PrepassFragmentShader,
        PrepassVertexShader, ViewKeyCache, ViewKeyPrepassCache, alpha_mode_pipeline_key,
    },
    prelude::*,
    render::{
        Extract, ExtractSchedule, Render, RenderApp, RenderSystems,
        erased_render_asset::ErasedRenderAssets,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        mesh::RenderMesh,
        render_asset::RenderAssets,
        render_resource::AsBindGroup,
        view::ExtractedView,
    },
};
use std::{
    collections::{HashSet, VecDeque},
    sync::Arc,
};

#[derive(Component, Clone, ExtractComponent)]
#[extract_app(RenderApp)]
pub(super) struct WarmupView;

/// Only the bounded staged destination may request speculative programs.
#[derive(Component)]
pub(super) struct HiddenDestination;

type Pair = (AssetId<Mesh>, AssetId<VoxelMaterial>, Option<(bool, bool)>);
struct FadeVariant {
    alpha: AlphaMode,
    key: ErasedMaterialKey,
}
const MAX_PENDING: usize = 4096;
const MAX_SIGNATURES: usize = 1024;
const PAIRS_PER_FRAME: usize = 64;
const PIPELINES_PER_FRAME: usize = 2;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Pass {
    Color,
    Shadow,
    CameraPrepass,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct Signature {
    key: ErasedMaterialPipelineKey,
    layout: MeshVertexBufferLayoutRef,
    pass: Pass,
}
struct Work {
    signature: Signature,
    properties: Arc<MaterialProperties>,
}
#[derive(Resource, Default)]
struct Queue {
    frame: u64,
    pending: VecDeque<(Pair, u64, Option<FadeVariant>)>,
    pairs: HashSet<Pair>,
    signatures: HashSet<Signature>,
    work: VecDeque<Work>,
}

pub(super) fn register(app: &mut App) {
    if std::env::var("CRYSTAL_DESTINATION_PIPELINES").as_deref() == Ok("off") {
        return;
    }
    if app.get_sub_app(RenderApp).is_none() {
        return;
    }
    app.add_plugins(ExtractComponentPlugin::<WarmupView>::default());
    app.sub_app_mut(RenderApp)
        .init_resource::<Queue>()
        .add_systems(ExtractSchedule, extract)
        .add_systems(
            Render,
            prepare
                .after(RenderSystems::Specialize)
                .before(RenderSystems::Queue),
        );
}

fn extract(
    mut queue: ResMut<Queue>,
    materials: Extract<Res<Assets<VoxelMaterial>>>,
    groups: Extract<Query<&occluder_fade::OccluderGroup>>,
    roots: Extract<Query<(), With<HiddenDestination>>>,
    parents: Extract<Query<&ChildOf>>,
    meshes: Extract<
        Query<
            (
                Entity,
                &Mesh3d,
                &MeshMaterial3d<VoxelMaterial>,
                &InheritedVisibility,
                Option<&ChildOf>,
            ),
            Or<(
                Changed<Mesh3d>,
                Changed<MeshMaterial3d<VoxelMaterial>>,
                Changed<InheritedVisibility>,
            )>,
        >,
    >,
) {
    queue.frame += 1;
    let expires = queue.frame + 600;
    for (entity, mesh, material, visibility, parent) in &meshes {
        if visibility.get() || queue.pending.len() >= MAX_PENDING {
            continue;
        }
        let mut cursor = entity;
        let mut staged = false;
        for _ in 0..32 {
            if roots.contains(cursor) {
                staged = true;
                break;
            }
            let Ok(parent) = parents.get(cursor) else {
                break;
            };
            cursor = parent.parent();
        }
        if !staged {
            continue;
        }
        let recipe = parent
            .and_then(|parent| groups.get(parent.parent()).ok())
            .map(occluder_fade::OccluderGroup::fade_recipe);
        let pair = (mesh.0.id(), material.0.id(), recipe);
        if queue.pairs.insert(pair) {
            let fade = recipe
                .and_then(|(coverage, depth)| {
                    materials
                        .get(&material.0)
                        .map(|source| occluder_fade::fade_material(source, coverage, depth))
                })
                .map(|mut material| {
                    // Compile the active reveal, whose nonzero opacity/body
                    // radius can select a different depth/aperture program.
                    material.extension.cutaway.fade.x = 0.5;
                    material.extension.cutaway.bottom_radius.w = 1.;
                    FadeVariant {
                        alpha: material.base.alpha_mode,
                        key: ErasedMaterialKey::new(material.bind_group_data()),
                    }
                });
            queue.pending.push_back((pair, expires, fade));
        }
    }
}

fn main_key(
    view: MeshPipelineKey,
    mesh: MeshPipelineKey,
    properties: &MaterialProperties,
) -> MeshPipelineKey {
    view | mesh
        | properties
            .mesh_pipeline_key_bits
            .downcast::<MeshPipelineKey>()
        | alpha_mode_pipeline_key(
            properties.alpha_mode,
            &Msaa::from_samples(view.msaa_samples()),
        )
}
fn shadow_key(
    light: MeshPipelineKey,
    mesh: MeshPipelineKey,
    properties: &MaterialProperties,
) -> MeshPipelineKey {
    let mut key = light | mesh;
    key.set(
        MeshPipelineKey::MAY_DISCARD,
        matches!(
            properties.alpha_mode,
            AlphaMode::Mask(_)
                | AlphaMode::Blend
                | AlphaMode::Premultiplied
                | AlphaMode::Add
                | AlphaMode::AlphaToCoverage
        ),
    );
    key.set(
        MeshPipelineKey::PREPASS_READS_MATERIAL,
        properties.get_shader(PrepassVertexShader).is_some()
            || properties.get_shader(PrepassFragmentShader).is_some(),
    );
    key
}

fn prepass_key(
    view: MeshPipelineKey,
    mesh: MeshPipelineKey,
    properties: &MaterialProperties,
) -> Option<MeshPipelineKey> {
    if !properties.prepass_enabled
        || properties.reads_view_transmission_texture
        || properties.render_method != OpaqueRendererMethod::Forward
        || !matches!(
            properties.alpha_mode,
            AlphaMode::Opaque | AlphaMode::Mask(_) | AlphaMode::AlphaToCoverage
        )
        || !view.intersects(
            MeshPipelineKey::DEPTH_PREPASS
                | MeshPipelineKey::NORMAL_PREPASS
                | MeshPipelineKey::MOTION_VECTOR_PREPASS,
        )
    {
        return None;
    }
    let mut key = view
        | mesh
        | alpha_mode_pipeline_key(
            properties.alpha_mode,
            &Msaa::from_samples(view.msaa_samples()),
        );
    key.set(
        MeshPipelineKey::PREPASS_READS_MATERIAL,
        properties.get_shader(PrepassVertexShader).is_some()
            || properties.get_shader(PrepassFragmentShader).is_some(),
    );
    Some(key)
}

fn fade_properties(source: &MaterialProperties, fade: FadeVariant) -> Arc<MaterialProperties> {
    Arc::new(MaterialProperties {
        alpha_mode: fade.alpha,
        material_key: fade.key,
        render_phase_type: if source.reads_view_transmission_texture {
            RenderPhaseType::Transmissive
        } else if matches!(fade.alpha, AlphaMode::Mask(_)) {
            RenderPhaseType::AlphaMask
        } else {
            RenderPhaseType::Transparent
        },
        render_method: source.render_method,
        mesh_pipeline_key_bits: source.mesh_pipeline_key_bits,
        depth_bias: source.depth_bias,
        reads_view_transmission_texture: source.reads_view_transmission_texture,
        material_layout: source.material_layout.clone(),
        draw_functions: source.draw_functions.clone(),
        shaders: source.shaders.clone(),
        bindless: source.bindless,
        base_specialize: source.base_specialize,
        prepass_specialize: source.prepass_specialize,
        user_specialize: source.user_specialize,
        shadows_enabled: source.shadows_enabled,
        prepass_enabled: source.prepass_enabled,
        oit_enabled: source.oit_enabled,
    })
}

fn prepare(world: &mut World) {
    world.resource_scope(|world, mut queue: Mut<Queue>| {
        if queue.pending.is_empty() && queue.work.is_empty() {
            return;
        }
        let view_entities: Vec<_> = world
            .query_filtered::<&ExtractedView, With<WarmupView>>()
            .iter(world)
            .map(|view| view.retained_view_entity)
            .collect();
        let cache = world.resource::<ViewKeyCache>();
        let views: HashSet<_> = view_entities
            .iter()
            .filter_map(|view| cache.get(view).copied())
            .collect();
        let prepass_cache = world.resource::<ViewKeyPrepassCache>();
        let prepasses: HashSet<_> = view_entities
            .iter()
            .filter_map(|view| prepass_cache.get(view).copied())
            .collect();
        // Directional shadow programs have no camera-position specialization.
        let mut lights: HashSet<_> = world
            .resource::<LightKeyCache>()
            .values()
            .copied()
            .collect();
        // The world's directional light can disable its shadow views indoors.
        // Keep its exact Bevy key even when only local shadow lights are active.
        lights.insert(
            MeshPipelineKey::DEPTH_PREPASS
                | MeshPipelineKey::VIEW_PROJECTION_ORTHOGRAPHIC
                | MeshPipelineKey::UNCLIPPED_DEPTH_ORTHO,
        );
        if views.is_empty() {
            return;
        }
        let attempts = queue.pending.len().min(PAIRS_PER_FRAME);
        for _ in 0..attempts {
            if queue.work.len() >= MAX_SIGNATURES {
                break;
            }
            let (pair, expires, fade) = queue.pending.pop_front().unwrap();
            if expires < queue.frame {
                queue.pairs.remove(&pair);
                continue;
            }
            let prepared = world
                .resource::<RenderAssets<RenderMesh>>()
                .get(pair.0)
                .zip(
                    world
                        .resource::<ErasedRenderAssets<PreparedMaterial>>()
                        .get(pair.1),
                )
                .map(|(mesh, material)| {
                    (
                        mesh.layout.clone(),
                        MeshPipelineKey::from_bits_retain(mesh.key_bits.bits()),
                        material.properties.clone(),
                    )
                });
            let Some((layout, mesh, properties)) = prepared else {
                queue.pending.push_back((pair, expires, fade));
                continue;
            };
            queue.pairs.remove(&pair);
            let variants = std::iter::once(properties.clone())
                .chain(fade.map(|fade| fade_properties(&properties, fade)));
            for properties in variants {
                let keys = views
                    .iter()
                    .map(|&view| (Pass::Color, main_key(view, mesh, &properties)))
                    .chain(
                        lights
                            .iter()
                            .filter(|_| properties.shadows_enabled)
                            .map(|&light| (Pass::Shadow, shadow_key(light, mesh, &properties))),
                    )
                    .chain(prepasses.iter().filter_map(|&view| {
                        prepass_key(view, mesh, &properties).map(|key| (Pass::CameraPrepass, key))
                    }));
                for (pass, mesh_key) in keys {
                    let signature = Signature {
                        layout: layout.clone(),
                        pass,
                        key: ErasedMaterialPipelineKey {
                            type_id: std::any::TypeId::of::<VoxelMaterial>(),
                            mesh_key: ErasedMeshPipelineKey::new(mesh_key),
                            material_key: properties.material_key.clone(),
                        },
                    };
                    if queue.signatures.insert(signature.clone()) {
                        queue.work.push_back(Work {
                            signature,
                            properties: properties.clone(),
                        });
                    }
                }
            }
        }
        for _ in 0..PIPELINES_PER_FRAME {
            let Some(work) = queue.work.pop_front() else {
                break;
            };
            let Signature { key, layout, pass } = &work.signature;
            let _trace = crate::loading_trace::span(
                "render.preload_material_pipeline",
                || serde_json::json!({"pass":format!("{pass:?}"),"frame":queue.frame}),
            );
            let result = if *pass != Pass::Color {
                work.properties
                    .prepass_specialize
                    .map(|specialize| specialize(world, key, layout, &work.properties))
            } else {
                work.properties
                    .base_specialize
                    .map(|specialize| specialize(world, key.clone(), layout, &work.properties))
            };
            if result.is_some_and(|result| result.is_err()) {
                // Never retry a failed speculative specialization every frame.
                // The ordinary visible path retains Bevy's normal diagnostics.
                continue;
            }
        }
        // Bevy owns the authoritative specialization cache. Bound our duplicate
        // suppression independently across long games and live profile changes.
        if queue.signatures.len() > MAX_SIGNATURES {
            queue.signatures = queue
                .work
                .iter()
                .map(|work| work.signature.clone())
                .collect();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{
        app::PluginsState,
        asset::load_internal_asset,
        camera::RenderTarget,
        pbr::{
            MaterialPipelineSpecializer, PrepassPipelineSpecializer,
            SpecializedMaterialPipelineCache, SpecializedPrepassMaterialPipelineCache,
            SpecializedShadowMaterialPipelineCache,
        },
        render::render_resource::{PipelineCache, SpecializedMeshPipelines, TextureFormat},
        window::{ExitCondition, WindowPlugin},
    };

    #[test]
    #[ignore = "requires a native GPU; compiles production hidden material and verifies visible cache reuse"]
    fn hidden_material_compiles_before_visibility_and_visible_draw_reuses_its_pipeline() {
        let mut app = App::new();
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                })
                .disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>(),
        );
        load_internal_asset!(
            app,
            crate::VOXEL_SURFACE_SHADER_HANDLE,
            "voxel_surface.wesl",
            Shader::from_wesl
        );
        load_internal_asset!(
            app,
            crate::GRASS_VERTEX_SHADER_HANDLE,
            "grass_vertex.wesl",
            Shader::from_wesl
        );
        app.add_plugins(MaterialPlugin::<VoxelMaterial>::default());
        register(&mut app);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
        while app.plugins_state() != PluginsState::Ready {
            assert!(std::time::Instant::now() < deadline);
            bevy::tasks::tick_global_task_pools_on_main_thread();
            std::thread::yield_now();
        }
        app.finish();
        app.cleanup();
        let target =
            app.world_mut()
                .resource_mut::<Assets<Image>>()
                .add(Image::new_target_texture(
                    64,
                    64,
                    TextureFormat::Rgba8UnormSrgb,
                    None,
                ));
        app.world_mut().spawn((
            Camera3d::default(),
            WarmupView,
            bevy::core_pipeline::prepass::DepthPrepass,
            bevy::core_pipeline::prepass::NormalPrepass,
            Msaa::Off,
            Camera::default(),
            bevy::render::view::Tonemapping::None,
            bevy::render::view::DebandDither::Disabled,
            bevy::camera::visibility::RenderLayers::from_layers(&[0, crate::VOXEL_RENDER_LAYER]),
            RenderTarget::Image(target.into()),
            Transform::from_xyz(0., 0., 5.).looking_at(Vec3::ZERO, Vec3::Y),
        ));
        let mut source = Mesh::from(Cuboid::default());
        source.insert_attribute(
            Mesh::ATTRIBUTE_COLOR,
            vec![[0.1, 0.4, 0.2, 1.]; source.count_vertices()],
        );
        let mesh = app.world_mut().resource_mut::<Assets<Mesh>>().add(source);
        let material = app
            .world_mut()
            .resource_mut::<Assets<VoxelMaterial>>()
            .add(crate::solid_terrain_material());
        let entity = app
            .world_mut()
            .spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material),
                Transform::default(),
                Visibility::Hidden,
                HiddenDestination,
            ))
            .id();
        let mut decoy = crate::solid_terrain_material();
        decoy.extension.smooth_depth_fade = true;
        let decoy_key = ErasedMaterialKey::new(decoy.bind_group_data());
        let decoy = app
            .world_mut()
            .resource_mut::<Assets<VoxelMaterial>>()
            .add(decoy);
        app.world_mut().spawn((
            Mesh3d(mesh),
            MeshMaterial3d(decoy),
            Transform::default(),
            Visibility::Hidden,
        ));
        app.world_mut().spawn((
            DirectionalLight {
                shadow_maps_enabled: true,
                ..default()
            },
            Transform::from_xyz(2., 4., 3.).looking_at(Vec3::ZERO, Vec3::Y),
        ));
        let main: bevy::render::sync_world::MainEntity = entity.into();
        let (signature, id, shadow_id, prepass_id) = loop {
            app.update();
            let render = app.sub_app(RenderApp).world();
            let warm = render.resource::<Queue>();
            let ready = warm
                .signatures
                .iter()
                .filter(|s| s.pass == Pass::Color)
                .find_map(|signature| {
                    let id = render
                        .resource::<SpecializedMeshPipelines<MaterialPipelineSpecializer>>()
                        .get_pipeline(&signature.key, &signature.layout)?;
                    render.resource::<PipelineCache>().get_render_pipeline(id)?;
                    Some((signature.clone(), id))
                });
            let shadow = warm
                .signatures
                .iter()
                .filter(|s| s.pass == Pass::Shadow)
                .find_map(|signature| {
                    let id = render
                        .resource::<SpecializedMeshPipelines<PrepassPipelineSpecializer>>()
                        .get_pipeline(&signature.key, &signature.layout)?;
                    render.resource::<PipelineCache>().get_render_pipeline(id)?;
                    Some(id)
                });
            let prepass = warm
                .signatures
                .iter()
                .filter(|s| s.pass == Pass::CameraPrepass)
                .find_map(|signature| {
                    let id = render
                        .resource::<SpecializedMeshPipelines<PrepassPipelineSpecializer>>()
                        .get_pipeline(&signature.key, &signature.layout)?;
                    render.resource::<PipelineCache>().get_render_pipeline(id)?;
                    Some(id)
                });
            if let (Some((signature, id)), Some(shadow), Some(prepass)) = (ready, shadow, prepass) {
                break (signature, id, shadow, prepass);
            }
            assert!(
                std::time::Instant::now() < deadline,
                "hidden production material did not compile"
            );
            std::thread::yield_now();
        };
        assert!(
            app.sub_app(RenderApp)
                .world()
                .resource::<Queue>()
                .signatures
                .iter()
                .all(|signature| signature.key.material_key != decoy_key),
            "hidden placeholder actors outside the staged destination must not compile"
        );
        assert!(
            !app.world()
                .get::<InheritedVisibility>(entity)
                .unwrap()
                .get()
        );
        assert!(
            !app.sub_app(RenderApp)
                .world()
                .resource::<SpecializedMaterialPipelineCache>()
                .values()
                .any(|cache| cache.contains_key(&main))
        );
        *app.world_mut().get_mut::<Visibility>(entity).unwrap() = Visibility::Visible;
        loop {
            app.update();
            let render = app.sub_app(RenderApp).world();
            let color_ready = render
                .resource::<SpecializedMaterialPipelineCache>()
                .values()
                .any(|cache| cache.get(&main) == Some(&id));
            let shadow_ready = render
                .resource::<SpecializedShadowMaterialPipelineCache>()
                .values()
                .any(|cache| cache.get(&main).is_some_and(|entry| entry.0 == shadow_id));
            let prepass_ready = render
                .resource::<SpecializedPrepassMaterialPipelineCache>()
                .values()
                .any(|cache| cache.get(&main).is_some_and(|entry| entry.1 == prepass_id));
            if color_ready && shadow_ready && prepass_ready {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "visible draw did not reuse warmed production pipeline"
            );
            std::thread::yield_now();
        }
        let render = app.sub_app(RenderApp).world();
        assert_eq!(
            render
                .resource::<SpecializedMeshPipelines<MaterialPipelineSpecializer>>()
                .get_pipeline(&signature.key, &signature.layout),
            Some(id)
        );
        assert!(
            render
                .resource::<PipelineCache>()
                .get_render_pipeline(id)
                .is_some()
        );

        // A hidden authored group also warms the exact future player-reveal
        // material without registering a speculative material or drawing it.
        let original = crate::solid_terrain_material();
        let mut future = occluder_fade::fade_material(&original, true, false);
        future.extension.cutaway.fade.x = 0.5;
        future.extension.cutaway.bottom_radius.w = 1.;
        let future_key = ErasedMaterialKey::new(future.bind_group_data());
        let original = app
            .world_mut()
            .resource_mut::<Assets<VoxelMaterial>>()
            .add(original);
        let surface = crate::mesh::SurfaceMeshData {
            positions: vec![[-1., -1., 0.], [1., -1., 0.], [1., 1., 0.], [-1., 1., 0.]],
            normals: vec![[0., 0., 1.]; 4],
            uvs: vec![[0.; 2]; 4],
            colors: vec![[0.1, 0.4, 0.2, 1.]; 4],
            indices: vec![0, 1, 2, 0, 2, 3],
            cutaway_ranges: vec![0..4],
            ..default()
        };
        let (_, mut prepared) = occluder_fade::prepare_grove(surface);
        occluder_fade::set_coverage_fade(&mut prepared, true);
        let root = app
            .world_mut()
            .spawn((Transform::default(), Visibility::Hidden, HiddenDestination))
            .id();
        let leaves = app
            .world_mut()
            .resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
                world.resource_scope(|world, mut materials: Mut<Assets<VoxelMaterial>>| {
                    occluder_fade::spawn_groups(
                        &mut world.commands(),
                        &mut meshes,
                        &mut materials,
                        root,
                        prepared,
                        original,
                        None,
                    )
                })
            });
        app.world_mut().flush();
        assert_eq!(leaves.len(), 1);
        let leaf: bevy::render::sync_world::MainEntity = leaves[0].into();
        let future_id = loop {
            app.update();
            let render = app.sub_app(RenderApp).world();
            let ready = render
                .resource::<Queue>()
                .signatures
                .iter()
                .filter(|s| s.pass == Pass::Color && s.key.material_key == future_key)
                .find_map(|signature| {
                    let id = render
                        .resource::<SpecializedMeshPipelines<MaterialPipelineSpecializer>>()
                        .get_pipeline(&signature.key, &signature.layout)?;
                    render.resource::<PipelineCache>().get_render_pipeline(id)?;
                    Some(id)
                });
            if let Some(id) = ready {
                break id;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "hidden player-reveal variant did not compile"
            );
            std::thread::yield_now();
        };
        assert!(
            !app.world()
                .get::<InheritedVisibility>(leaves[0])
                .unwrap()
                .get()
        );
        let future = app
            .world_mut()
            .resource_mut::<Assets<VoxelMaterial>>()
            .add(future);
        app.world_mut()
            .entity_mut(leaves[0])
            .insert(MeshMaterial3d(future));
        *app.world_mut().get_mut::<Visibility>(root).unwrap() = Visibility::Visible;
        loop {
            app.update();
            let render = app.sub_app(RenderApp).world();
            if render
                .resource::<SpecializedMaterialPipelineCache>()
                .values()
                .any(|cache| cache.get(&leaf) == Some(&future_id))
            {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "player-reveal draw did not reuse its precompiled pipeline"
            );
            std::thread::yield_now();
        }
    }
}
