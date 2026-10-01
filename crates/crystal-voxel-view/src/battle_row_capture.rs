//! Optional capture reuse for the already gated, static modeled-row pilot.
//! Geometry still uses Bevy's ordinary PBR draw and output blit. A receipt is
//! published only after that actor draw succeeds and the frame is submitted.
use super::*;
use bevy::{
    asset::{AssetEvents, AssetId},
    core_pipeline::{
        blit::{BlitPipeline, BlitPipelineKey},
        core_3d::{
            Opaque3d,
            graph::{Core3d, Node3d},
        },
        tonemapping::DebandDither,
        upscaling::ViewUpscalingPipeline,
    },
    ecs::{
        query::{QueryItem, ROQueryItem},
        system::{SystemParam, SystemParamItem},
    },
    pbr::{
        DrawMesh, PreparedMaterial, RenderMaterialInstances, SetMaterialBindGroup,
        SetMeshBindGroup, SetMeshViewBindGroup, queue_material_meshes,
    },
    render::{
        Render, RenderApp, RenderSet,
        camera::ExtractedCamera,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        mesh::{GpuBufferInfo, GpuMesh},
        render_asset::{RenderAssets, prepare_assets},
        render_graph::{
            NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, RenderSubGraph,
            ViewNode, ViewNodeRunner,
        },
        render_phase::{
            AddRenderCommand, BinnedRenderPhaseType, DrawFunctions, PhaseItem, RenderCommand,
            RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewBinnedRenderPhases,
        },
        render_resource::{CachedRenderPipelineId, PipelineCache, SpecializedRenderPipelines},
        renderer::{RenderContext, render_system},
        texture::GpuImage,
        view::{ColorGrading, ViewTarget, VisibleEntities, WithMesh},
    },
};

#[path = "battle_row_capture_state.rs"]
mod state;
use state::{CaptureCache, CaptureTicket};

pub(super) struct BattleRowCapturePlugin;
impl Plugin for BattleRowCapturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActorCaptures>()
            .add_event::<AssetEvent<Mesh>>()
            .add_event::<AssetEvent<StandardMaterial>>()
            .add_event::<AssetEvent<Image>>()
            .add_event::<AssetEvent<Shader>>()
            // Visibility and camera projection update normally in PostUpdate.
            // Last sees the finished transforms AND same-frame asset events,
            // then disables only captures whose submitted image is reusable.
            .add_systems(Last, sync_actor_captures.after(AssetEvents))
            .add_plugins(ExtractComponentPlugin::<ActorCaptureRequest>::default());
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_render_command::<Opaque3d, CaptureActorDraw>()
                .add_render_graph_node::<ViewNodeRunner<CaptureOutputNode>>(
                    Core3d,
                    CaptureOutputLabel,
                )
                .add_render_graph_edges(Core3d, (Node3d::Upscaling, CaptureOutputLabel))
                .add_systems(
                    Render,
                    prepare_capture_output_guard.in_set(RenderSet::PrepareBindGroups),
                )
                .add_systems(
                    Render,
                    observe_actor_draws
                        .in_set(RenderSet::QueueMeshes)
                        .after(queue_material_meshes::<StandardMaterial>)
                        .after(prepare_assets::<PreparedMaterial<StandardMaterial>>)
                        .after(prepare_assets::<GpuImage>),
                )
                .add_systems(
                    Render,
                    finish_actor_captures
                        .in_set(RenderSet::Render)
                        .after(render_system),
                );
        }
    }
}

#[derive(PartialEq)]
struct CaptureKey {
    actor: Entity,
    mesh: AssetId<Mesh>,
    material: AssetId<StandardMaterial>,
    actor_pose: Mat4,
    camera_pose: Mat4,
    target: AssetId<Image>,
    projection: [f32; 4],
    // Camera3d, exposure, tonemapping, color grading and static deband settings.
    camera_changes: [u32; 5],
    light: (Entity, u32, Mat4),
    ambient: Option<(Color, f32)>,
    palette: (u8, BattleFlashMode),
    unlit: bool,
    render_size: UVec2,
    output_size: UVec2,
    msaa_samples: u32,
}

#[derive(Resource)]
pub(super) struct ActorCaptures {
    enabled: bool,
    actors: [CaptureCache<CaptureKey>; 2],
}
impl Default for ActorCaptures {
    fn default() -> Self {
        Self {
            // Separate opt-in until native capture/alpha/resize QA is green.
            enabled: cfg!(not(target_arch = "wasm32"))
                && std::env::var("CRYSTAL_BATTLE_ROW_PROTOTYPE").as_deref() == Ok("1")
                && std::env::var("CRYSTAL_BATTLE_ROW_CAPTURE_CACHE").as_deref() == Ok("1"),
            actors: std::array::from_fn(|_| CaptureCache::default()),
        }
    }
}

impl ActorCaptures {
    pub(super) fn prewarm_actor(&self, frame: &VisualBattleFrame, index: usize) -> bool {
        self.enabled
            && frame.active
            && !frame.use_source_scene
            && frame.battlers[index]
                .as_ref()
                .is_some_and(|actor| actor.visible && actor.allow_species_model && !actor.shiny)
    }
}

#[derive(Component, Clone, ExtractComponent)]
struct ActorCaptureRequest {
    actor: Entity,
    mesh: AssetId<Mesh>,
    material: AssetId<StandardMaterial>,
    target: AssetId<Image>,
    size: UVec2,
    ticket: CaptureTicket,
}

fn changed_asset<A: Asset>(event: &AssetEvent<A>) -> AssetId<A> {
    match event {
        AssetEvent::Added { id }
        | AssetEvent::Modified { id }
        | AssetEvent::Removed { id }
        | AssetEvent::Unused { id }
        | AssetEvent::LoadedWithDependencies { id } => *id,
    }
}

#[allow(clippy::too_many_arguments)]
#[derive(SystemParam)]
struct CaptureAssetEvents<'w, 's> {
    meshes: EventReader<'w, 's, AssetEvent<Mesh>>,
    materials: EventReader<'w, 's, AssetEvent<StandardMaterial>>,
    images: EventReader<'w, 's, AssetEvent<Image>>,
    shaders: EventReader<'w, 's, AssetEvent<Shader>>,
}

fn sync_actor_captures(
    mut commands: Commands,
    scene: Res<BattleScene>,
    mut status: ResMut<BattleViewStatus>,
    frame: Res<VisualBattleFrame>,
    mut captures: ResMut<ActorCaptures>,
    materials: Res<Assets<StandardMaterial>>,
    ambient: Option<Res<AmbientLight>>,
    msaa: Option<Res<Msaa>>,
    mut events: CaptureAssetEvents,
    actors: Query<(&GlobalTransform, &Visibility, &RenderLayers), With<BattleActor>>,
    lights: Query<(
        Entity,
        Ref<DirectionalLight>,
        &GlobalTransform,
        &InheritedVisibility,
        Option<&RenderLayers>,
    )>,
    other_lights: Query<
        (Option<&RenderLayers>, Has<bevy::pbr::LightProbe>),
        Or<(
            With<PointLight>,
            With<SpotLight>,
            With<bevy::pbr::LightProbe>,
        )>,
    >,
    unsupported_cameras: Query<
        (),
        Or<(
            With<bevy::render::view::GpuCulling>,
            With<bevy::core_pipeline::Skybox>,
            With<bevy::render::camera::TemporalJitter>,
            With<bevy::core_pipeline::experimental::taa::TemporalAntiAliasSettings>,
            With<bevy::core_pipeline::dof::DepthOfFieldSettings>,
            With<bevy::pbr::ScreenSpaceAmbientOcclusionSettings>,
            With<bevy::pbr::ScreenSpaceReflectionsSettings>,
            With<bevy::pbr::environment_map::EnvironmentMapLight>,
        )>,
    >,
    other_cameras: Query<&Camera, Without<BattleRowCamera>>,
    mut cameras: Query<
        (
            Entity,
            &BattleRowCamera,
            &mut Camera,
            &GlobalTransform,
            &Projection,
            &bevy::render::camera::CameraRenderGraph,
            Ref<Camera3d>,
            Ref<bevy::render::camera::Exposure>,
            Ref<Tonemapping>,
            Ref<ColorGrading>,
            Ref<DebandDither>,
            &RenderLayers,
            Option<&Fxaa>,
            Option<&FogSettings>,
        ),
        With<BattleRowCamera>,
    >,
) {
    let mut dirty = [false; 2];
    for event in events.meshes.read() {
        for (index, actor) in scene.actors.iter().enumerate() {
            dirty[index] |= actor
                .as_ref()
                .and_then(|actor| actor.mesh.as_ref())
                .is_some_and(|mesh| mesh.id() == changed_asset(event));
        }
    }
    for event in events.materials.read() {
        for (index, actor) in scene.actors.iter().enumerate() {
            dirty[index] |= actor
                .as_ref()
                .is_some_and(|actor| actor.material.id() == changed_asset(event));
        }
    }
    for event in events.images.read() {
        for (index, target) in scene.row_targets.iter().enumerate() {
            dirty[index] |= target.id() == changed_asset(event);
        }
    }
    if events.shaders.read().count() != 0 {
        dirty = [true; 2];
    }
    status.row_capture_ready = [false; 2];
    status.row_capture_pending = [false; 2];
    let cache_enabled = captures.enabled;
    for (
        entity,
        row,
        mut camera,
        camera_pose,
        projection,
        graph,
        camera_3d,
        exposure,
        tonemapping,
        grading,
        dither,
        layers,
        fxaa,
        fog,
    ) in &mut cameras
    {
        let index = row.0;
        let cache = &mut captures.actors[index];
        // PostUpdate activates supported actor targets during stable battle
        // presentation as well as rows. Normal idle/row transitions keep the
        // exact capture; F3, zero-size, fallback and retirement invalidate it.
        if !cache_enabled || !camera.is_active {
            cache.invalidate();
            commands.entity(entity).remove::<ActorCaptureRequest>();
            continue;
        }
        if dirty[index] {
            cache.invalidate();
        }
        let key = (|| {
            let actor = scene.actors[index].as_ref()?;
            let mesh = actor.mesh.as_ref()?; // Source cards keep rendering.
            let material = materials.get(&actor.material)?;
            let (pose, visibility, actor_layers) = actors.get(actor.entity).ok()?;
            let row_layers = RenderLayers::layer(BATTLE_ROW_LAYERS[index]);
            // Authored actors are opaque, untextured StandardMaterial meshes.
            // Temporal/extra postprocessing or external light configurations
            // retain continuous rendering until separately proved cacheable.
            if !actor.key.modeled
                // The exact-static-image cache is deliberately single-mesh.
                // Articulated anatomy remains live through all source rows.
                || actor.animated.is_some()
                || !frame.battlers[index]
                    .as_ref()
                    .is_some_and(|actor| actor.visible && actor.allow_species_model && !actor.shiny)
                || *visibility != Visibility::Visible
                || !actor_layers.intersects(&row_layers)
                || *layers != row_layers
                || **graph != Core3d.intern()
                || unsupported_cameras.contains(entity)
                || scene.row_targets[0].id() == scene.row_targets[1].id()
                || other_cameras.iter().any(|other| {
                    other.is_active
                        && matches!(&other.target, RenderTarget::Image(image)
                        if image.id() == scene.row_targets[index].id())
                })
                || camera.hdr
                || camera.viewport.is_some()
                || fxaa.is_some()
                || fog.is_some()
                || !matches!(camera.target, RenderTarget::Image(ref image) if image.id() == scene.row_targets[index].id())
                || !matches!(camera.clear_color, ClearColorConfig::Custom(color) if color == Color::NONE)
                || !matches!(camera.output_mode, bevy::render::camera::CameraOutputMode::Write {
                    blend_state: Some(bevy::render::render_resource::BlendState::REPLACE),
                    clear_color: ClearColorConfig::Custom(color),
                } if color == Color::NONE)
                || material.alpha_mode != AlphaMode::Opaque
                || material.base_color_texture.is_some()
                || material.emissive_texture.is_some()
                || material.metallic_roughness_texture.is_some()
                || material.normal_map_texture.is_some()
                || material.occlusion_texture.is_some()
                || material.depth_map.is_some()
                || material.specular_transmission != 0.0
                || material.diffuse_transmission != 0.0
                || other_lights
                    .iter()
                    // Bevy probes ignore RenderLayers and use camera frusta.
                    .any(|(layers, probe)| {
                        probe || layers.cloned().unwrap_or_default().intersects(&row_layers)
                    })
            {
                return None;
            }
            let mut relevant_lights = lights.iter().filter(|(_, _, _, visibility, layers)| {
                visibility.get() && layers.cloned().unwrap_or_default().intersects(&row_layers)
            });
            let (light_entity, light, light_pose, _, _) = relevant_lights.next()?;
            if relevant_lights.next().is_some() || light.shadows_enabled {
                return None;
            }
            let Projection::Perspective(projection) = projection else {
                return None;
            };
            Some(CaptureKey {
                actor: actor.entity,
                mesh: mesh.id(),
                material: actor.material.id(),
                actor_pose: pose.compute_matrix(),
                camera_pose: camera_pose.compute_matrix(),
                target: scene.row_targets[index].id(),
                projection: [
                    projection.fov,
                    projection.aspect_ratio,
                    projection.near,
                    projection.far,
                ],
                camera_changes: [
                    camera_3d.last_changed().get(),
                    exposure.last_changed().get(),
                    tonemapping.last_changed().get(),
                    grading.last_changed().get(),
                    dither.last_changed().get(),
                ],
                light: (
                    light_entity,
                    light.last_changed().get(),
                    light_pose.compute_matrix(),
                ),
                ambient: ambient
                    .as_ref()
                    .map(|light| (light.color, light.brightness)),
                palette: actor.palette_key,
                unlit: material.unlit,
                render_size: scene.row_sizes[index],
                output_size: status.output_size,
                msaa_samples: msaa.as_ref().map_or(1, |msaa| msaa.samples()),
            })
        })();
        let Some(key) = key else {
            // Unsupported settings retain the original live rendering while
            // a row is needed. Do not spend extra passes on idle source cards.
            if !battle_has_rows(&frame) {
                camera.is_active = false;
            }
            cache.invalidate();
            commands.entity(entity).remove::<ActorCaptureRequest>();
            continue;
        };
        let (actor, mesh, material) = (key.actor, key.mesh, key.material);
        if let Some(ticket) = cache.request(key) {
            status.row_capture_pending[index] = true;
            commands.entity(entity).insert(ActorCaptureRequest {
                actor,
                mesh,
                material,
                target: scene.row_targets[index].id(),
                size: scene.row_sizes[index],
                ticket,
            });
        } else {
            let ready = cache.settled();
            status.row_capture_ready[index] = ready;
            status.row_capture_pending[index] = !ready;
            camera.is_active = false;
            commands.entity(entity).remove::<ActorCaptureRequest>();
        }
    }
}

// Identical to Bevy 0.14 DrawMaterial<StandardMaterial>, followed by a receipt.
// The tuple stops on Failure, including asynchronous pipeline compilation,
// missing bindings/assets and not-yet-ready GPU mesh preprocessing.
type CaptureActorDraw = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetMaterialBindGroup<StandardMaterial, 2>,
    DrawMesh,
    RecordActorDraw,
);
struct RecordActorDraw;
impl RenderCommand<Opaque3d> for RecordActorDraw {
    type Param = ();
    type ViewQuery = &'static ActorCaptureRequest;
    type ItemQuery = ();
    fn render<'w>(
        item: &Opaque3d,
        request: ROQueryItem<'w, Self::ViewQuery>,
        _: Option<()>,
        _: SystemParamItem<'w, '_, Self::Param>,
        _: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if item.entity() == request.actor && !item.batch_range().is_empty() {
            request.ticket.mark_drawn();
        }
        RenderCommandResult::Success
    }
}

fn observe_actor_draws(
    views: Query<(Entity, &ActorCaptureRequest, &VisibleEntities), With<ExtractedCamera>>,
    mut phases: ResMut<ViewBinnedRenderPhases<Opaque3d>>,
    draws: Res<DrawFunctions<Opaque3d>>,
    meshes: Res<RenderAssets<GpuMesh>>,
    materials: Res<RenderAssets<PreparedMaterial<StandardMaterial>>>,
    material_instances: Res<RenderMaterialInstances<StandardMaterial>>,
    images: Res<RenderAssets<GpuImage>>,
) {
    for (view, request, visible) in &views {
        let (Some(phase), Some(mesh), Some(material), Some(image)) = (
            phases.get_mut(&view),
            meshes.get(request.mesh),
            materials.get(request.material),
            images.get(request.target),
        ) else {
            continue;
        };
        if visible.get::<WithMesh>() != [request.actor]
            || image.size != request.size
            || mesh.vertex_count == 0
            || matches!(mesh.buffer_info, GpuBufferInfo::Indexed { count: 0, .. })
            || material_instances.get(&request.actor) != Some(&request.material)
            || phase.batchable_mesh_keys.len() != 1
            || !phase.unbatchable_mesh_keys.is_empty()
            || !phase.non_mesh_items.is_empty()
        {
            continue;
        }
        let mut key = phase.batchable_mesh_keys[0].clone();
        if key.asset_id != request.mesh.untyped()
            || key.material_bind_group_id != Some(material.bind_group.id())
        {
            continue;
        }
        // This isolated view contains exactly its one modeled actor. Requeue
        // that same bin with the same geometry/material/pipeline, replacing
        // only the draw function. The arena and other views are untouched.
        key.draw_function = draws.read().id::<CaptureActorDraw>();
        phase.clear();
        phase.add(key, request.actor, BinnedRenderPhaseType::BatchableMesh);
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct CaptureOutputLabel;

#[derive(Component)]
struct CaptureOutputGuard(CachedRenderPipelineId);

fn prepare_capture_output_guard(
    mut commands: Commands,
    views: Query<(Entity, &ViewTarget), With<ActorCaptureRequest>>,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<BlitPipeline>>,
    blit: Res<BlitPipeline>,
) {
    for (entity, target) in &views {
        // Exactly the fixed row-camera key used by Bevy's UpscalingPlugin.
        // ViewUpscalingPipeline's presence alone does not prove compilation:
        // Bevy inserts it even when block_on_render_pipeline returns an error.
        let id = pipelines.specialize(
            &pipeline_cache,
            &blit,
            BlitPipelineKey {
                texture_format: target.out_texture_format(),
                blend_state: Some(bevy::render::render_resource::BlendState::REPLACE),
                samples: 1,
            },
        );
        commands.entity(entity).insert(CaptureOutputGuard(id));
    }
}

#[derive(Default)]
struct CaptureOutputNode;
impl ViewNode for CaptureOutputNode {
    type ViewQuery = (
        &'static ActorCaptureRequest,
        &'static CaptureOutputGuard,
        &'static ViewUpscalingPipeline,
        &'static ViewTarget,
        &'static ExtractedCamera,
    );

    fn run(
        &self,
        _: &mut RenderGraphContext,
        _: &mut RenderContext,
        (request, guard, _, target, camera): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        // This node runs after this view's output blit, never after an assumed
        // number of display frames. The fixed key must have been loaded and
        // the actual output view must be the current requested target.
        let pipeline = world.resource::<PipelineCache>();
        let images = world.resource::<RenderAssets<GpuImage>>();
        if camera.render_graph == Core3d.intern()
            && camera.viewport.is_none()
            && matches!(&camera.target,
                Some(bevy::render::camera::NormalizedRenderTarget::Image(image))
                    if image.id() == request.target)
            && camera.physical_target_size == Some(request.size)
            && matches!(camera.output_mode,
                bevy::render::camera::CameraOutputMode::Write {
                    blend_state: Some(bevy::render::render_resource::BlendState::REPLACE),
                    clear_color: ClearColorConfig::Custom(color),
                } if color == Color::NONE)
            && pipeline.get_render_pipeline(guard.0).is_some()
            && images.get(request.target).is_some_and(|image| {
                image.size == request.size && image.texture_view.id() == target.out_texture().id()
            })
        {
            request.ticket.mark_output();
        }
        Ok(())
    }
}

fn finish_actor_captures(requests: Query<&ActorCaptureRequest>) {
    // render_system returns only after the graph's command buffers submit;
    // errors panic instead. Draw + verified output are per-attempt receipts.
    for request in &requests {
        request.ticket.mark_submitted();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capture_app() -> App {
        let mut app = super::super::tests::headless_battle_app();
        app.add_plugins((
            bevy::transform::TransformPlugin,
            bevy::render::view::VisibilityPlugin,
        ));
        // MinimalPlugins has no AssetPlugin; publish exactly as AssetPlugin
        // does in production, before the cache's Last-stage decision.
        app.add_systems(
            Last,
            (
                Assets::<Mesh>::asset_events,
                Assets::<StandardMaterial>::asset_events,
                Assets::<Image>::asset_events,
                Assets::<Shader>::asset_events,
            )
                .in_set(AssetEvents),
        );
        app.world_mut().resource_mut::<ActorCaptures>().enabled = true;
        app.world_mut()
            .spawn((Window::default(), bevy::window::PrimaryWindow));
        app.world_mut().spawn(Camera2dBundle::default());
        let mut source = super::super::tests::source_test_frame(0xe4);
        source.battler_offsets = [Vec2::ZERO; 2];
        source.battler_rows[0] = Some(crystal_render_api::VisualBattleBattlerRows {
            source_y: Vec2::new(48.0, 64.0),
            bg_cleared: true,
        });
        let mut frame = app.world_mut().resource_mut::<VisualBattleFrame>();
        frame.source = Some(source);
        frame.battlers[1].as_mut().unwrap().species_id = Arc::from("GENGAR");
        app.update();
        app
    }

    fn active(app: &mut App) -> [bool; 2] {
        let world = app.world_mut();
        let mut result = [false; 2];
        for (row, camera) in world.query::<(&BattleRowCamera, &Camera)>().iter(world) {
            result[row.0] = camera.is_active;
        }
        result
    }

    // This simulates the receipt only. Native acceptance must verify that the
    // actual draw observer and output blit produce it, including cold starts.
    fn submit(app: &mut App) {
        let world = app.world_mut();
        let requests: Vec<_> = world
            .query::<&ActorCaptureRequest>()
            .iter(world)
            .cloned()
            .collect();
        assert!(
            (1..=2).contains(&requests.len()),
            "expected a bounded pending actor capture"
        );
        for request in requests {
            request.ticket.mark_drawn();
            request.ticket.mark_output();
            request.ticket.mark_submitted();
        }
    }

    #[test]
    fn pidgeotto_rows_remain_live_and_wing_motion_never_modifies_mesh_assets() {
        let mut app = capture_app();
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 30.0),
        ));
        app.world_mut().resource_mut::<VisualBattleFrame>().battlers[0]
            .as_mut()
            .unwrap()
            .species_id = Arc::from("PIDGEOTTO");
        app.update();
        let (entities, meshes) = {
            let actor = app.world().resource::<BattleScene>().actors[0]
                .as_ref()
                .unwrap();
            let wings = &actor.animated.as_ref().unwrap().wings;
            (
                [actor.entity, wings[0].entity, wings[1].entity],
                [
                    actor.mesh.as_ref().unwrap().id(),
                    wings[0].mesh.id(),
                    wings[1].mesh.id(),
                ],
            )
        };
        let mut reader = bevy::ecs::event::ManualEventReader::<AssetEvent<Mesh>>::default();
        reader.clear(app.world().resource::<Events<AssetEvent<Mesh>>>());
        for _ in 0..12 {
            app.update();
            let world = app.world_mut();
            assert!(
                world
                    .query::<&ActorCaptureRequest>()
                    .iter(world)
                    .all(|request| request.actor != entities[0])
            );
            for entity in entities {
                assert_eq!(
                    *world.get::<RenderLayers>(entity).unwrap(),
                    RenderLayers::layer(BATTLE_ROW_LAYERS[0])
                );
            }
            assert!(
                reader
                    .read(world.resource::<Events<AssetEvent<Mesh>>>())
                    .all(|event| !meshes.contains(&changed_asset(event)))
            );
            assert!(active(&mut app)[0]);
        }
    }

    #[test]
    fn capture_cache_reuses_images_while_source_rows_scroll_and_change_sides() {
        let mut app = capture_app();
        assert_eq!(active(&mut app), [true; 2]);
        for _ in 0..8 {
            app.update();
            assert_eq!(
                active(&mut app),
                [true; 2],
                "missing render readiness cannot time out"
            );
        }
        submit(&mut app);
        app.update();
        assert_eq!(active(&mut app), [false; 2]);
        let handles = app.world().resource::<BattleScene>().row_targets.clone();
        for tick in 1..100 {
            let mut frame = app.world_mut().resource_mut::<VisualBattleFrame>();
            let source = frame.source.as_mut().unwrap();
            source.frame = tick;
            source.line_x_offsets = Some([tick as i8 % 11; 95]);
            source.line_y_offsets = Some([tick as i8 % 5; 95]);
            source.screen_offset = Vec2::new(1.0, -2.0);
            source.battler_rows = [None; 2];
            let side = usize::from(tick % 2 == 0);
            source.battler_rows[side] = Some(crystal_render_api::VisualBattleBattlerRows {
                source_y: if side == 0 {
                    Vec2::new(48.0, 64.0)
                } else {
                    Vec2::new(40.0, 56.0)
                },
                bg_cleared: tick % 3 == 0,
            });
            app.update();
            assert_eq!(active(&mut app), [false; 2]);
            assert_eq!(app.world().resource::<BattleScene>().row_targets, handles);
            assert!(
                app.world()
                    .resource::<Assets<BattleCompositeMaterial>>()
                    .iter()
                    .next()
                    .unwrap()
                    .1
                    .source
                    .battler_rows[side]
                    .w
                    != 0.0
            );
        }
    }

    #[test]
    fn capture_cache_recaptures_only_changed_actor_palette_material_geometry_and_pose() {
        let mut app = capture_app();
        submit(&mut app);
        app.update();
        app.world_mut()
            .resource_mut::<VisualBattleFrame>()
            .source
            .as_mut()
            .unwrap()
            .battler_bgps[0] = 0xff;
        app.update();
        assert_eq!(active(&mut app), [true, false]);
        submit(&mut app);
        app.update();
        app.world_mut().insert_resource(BattleFlashMode::Reduced);
        app.update();
        assert_eq!(
            active(&mut app),
            [true; 2],
            "F4 changes palette material/vertex data"
        );
        submit(&mut app);
        app.update();
        let mesh = app.world().resource::<BattleScene>().actors[1]
            .as_ref()
            .unwrap()
            .mesh
            .clone()
            .unwrap();
        app.world_mut()
            .resource_mut::<Assets<Mesh>>()
            .get_mut(&mesh)
            .unwrap();
        app.update();
        assert_eq!(
            active(&mut app),
            [false, true],
            "same-handle mesh modification is dirty in this frame"
        );
        submit(&mut app);
        app.update();
        app.world_mut()
            .resource_mut::<VisualBattleFrame>()
            .source
            .as_mut()
            .unwrap()
            .battler_offsets[0] = Vec2::new(2.0, 0.0);
        app.update();
        assert_eq!(
            active(&mut app),
            [true, false],
            "actual actor displacement changes capture"
        );
        submit(&mut app);
        app.update();
        let material = app.world().resource::<BattleScene>().actors[1]
            .as_ref()
            .unwrap()
            .material
            .clone();
        app.world_mut()
            .resource_mut::<Assets<StandardMaterial>>()
            .get_mut(&material)
            .unwrap()
            .perceptual_roughness = 0.8;
        app.update();
        assert_eq!(active(&mut app), [false, true]);
    }

    #[test]
    fn capture_cache_invalidates_resize_f3_interruption_and_changed_appearance() {
        let mut app = capture_app();
        submit(&mut app);
        app.update();
        let targets = app.world().resource::<BattleScene>().row_targets.clone();
        app.world_mut()
            .resource_mut::<VisualBattleCanvas>()
            .physical_size = UVec2::new(400, 300);
        app.update();
        assert_eq!(active(&mut app), [true; 2]);
        assert_eq!(app.world().resource::<BattleScene>().row_targets, targets);
        submit(&mut app);
        app.update();
        for _ in 0..3 {
            app.world_mut().resource_mut::<VoxelViewSettings>().enabled = false;
            app.update();
            assert_eq!(active(&mut app), [false; 2]);
            app.world_mut().resource_mut::<VoxelViewSettings>().enabled = true;
            app.update();
            assert_eq!(active(&mut app), [true; 2]);
            submit(&mut app);
            app.update();
        }
        app.world_mut()
            .resource_mut::<VisualBattleFrame>()
            .use_source_scene = true;
        app.update();
        assert_eq!(active(&mut app), [false; 2]);
        app.world_mut()
            .resource_mut::<VisualBattleFrame>()
            .use_source_scene = false;
        app.update();
        assert_eq!(active(&mut app), [true; 2]);
        submit(&mut app);
        app.update();
        let before_layout = app.world().resource::<BattleSceneLayout>().clone();
        app.world_mut().resource_mut::<VisualBattleFrame>().battlers[0]
            .as_mut()
            .unwrap()
            .species_id = Arc::from("GENGAR");
        app.update();
        assert_ne!(*app.world().resource::<BattleSceneLayout>(), before_layout);
        assert_eq!(
            active(&mut app),
            [true; 2],
            "species changes can reframe the shared camera for both actors"
        );
    }

    #[test]
    fn capture_cache_invalidates_camera_and_lighting_and_keeps_unsupported_views_live() {
        let mut app = capture_app();
        submit(&mut app);
        app.update();
        {
            let world = app.world_mut();
            let mut cameras =
                world.query::<(&BattleRowCamera, &mut bevy::render::camera::Exposure)>();
            for (row, mut exposure) in cameras.iter_mut(world) {
                if row.0 == 0 {
                    exposure.ev100 += 1.0;
                }
            }
        }
        app.update();
        assert_eq!(active(&mut app), [true, false]);
        submit(&mut app);
        app.update();
        app.world_mut().insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 70.0,
        });
        app.update();
        assert_eq!(active(&mut app), [true; 2]);
        submit(&mut app);
        app.update();
        {
            let world = app.world_mut();
            for mut light in world.query::<&mut DirectionalLight>().iter_mut(world) {
                light.shadows_enabled = true;
            }
        }
        for _ in 0..4 {
            app.update();
            assert_eq!(active(&mut app), [true; 2]);
            let world = app.world_mut();
            assert_eq!(world.query::<&ActorCaptureRequest>().iter(world).count(), 0);
        }
    }

    #[test]
    fn captures_prewarm_before_rows_without_hiding_arena_or_advancing_source() {
        let mut app = capture_app();
        app.world_mut().resource_mut::<VisualBattleFrame>().source = None;
        app.update();
        // No render receipt was sent yet: a menu frame must really prewarm.
        assert_eq!(active(&mut app), [true; 2]);
        let original = app.world().resource::<VisualBattleFrame>().clone();
        {
            let world = app.world_mut();
            for (actor, layers) in world.query::<(&BattleActor, &RenderLayers)>().iter(world) {
                let _ = actor;
                assert!(layers.intersects(&RenderLayers::layer(BATTLE_LAYER)));
                assert!(
                    BATTLE_ROW_LAYERS
                        .iter()
                        .any(|layer| layers.intersects(&RenderLayers::layer(*layer)))
                );
            }
            assert_eq!(
                world
                    .query_filtered::<&Visibility, With<BattleRowCompositeSprite>>()
                    .single(world),
                &Visibility::Hidden
            );
        }
        let counts = (
            app.world().resource::<Assets<Mesh>>().len(),
            app.world().resource::<Assets<Image>>().len(),
            app.world().resource::<Assets<StandardMaterial>>().len(),
        );
        let targets = app.world().resource::<BattleScene>().row_targets.clone();
        submit(&mut app);
        app.update();
        assert_eq!(active(&mut app), [false; 2]);
        assert_eq!(
            app.world().resource::<BattleViewStatus>().row_capture_ready,
            [true; 2]
        );
        assert_eq!(*app.world().resource::<VisualBattleFrame>(), original);
        let mut source = super::super::tests::source_test_frame(0xe4);
        source.battler_offsets = [Vec2::ZERO; 2];
        source.battler_rows[0] = Some(crystal_render_api::VisualBattleBattlerRows {
            source_y: Vec2::new(48.0, 64.0),
            bg_cleared: true,
        });
        app.world_mut().resource_mut::<VisualBattleFrame>().source = Some(source.clone());
        app.update();
        assert_eq!(
            active(&mut app),
            [false; 2],
            "attack starts from proven prewarm"
        );
        assert_eq!(
            app.world().resource::<VisualBattleFrame>().source,
            Some(source)
        );
        for tick in 0..200 {
            if tick % 2 == 0 {
                app.world_mut().resource_mut::<VisualBattleFrame>().source = None;
            }
            app.update();
            assert_eq!(active(&mut app), [false; 2]);
            assert_eq!(app.world().resource::<BattleScene>().row_targets, targets);
            assert_eq!(
                (
                    app.world().resource::<Assets<Mesh>>().len(),
                    app.world().resource::<Assets<Image>>().len(),
                    app.world().resource::<Assets<StandardMaterial>>().len()
                ),
                counts
            );
        }
    }

    #[test]
    fn capture_palette_value_changes_with_the_same_bgp_are_not_stale() {
        let mut app = capture_app();
        app.world_mut()
            .resource_mut::<VisualBattleFrame>()
            .source
            .as_mut()
            .unwrap()
            .battler_bgps[0] = 0x1b;
        app.update();
        submit(&mut app);
        app.update();
        let mesh = app.world().resource::<BattleScene>().actors[0]
            .as_ref()
            .unwrap()
            .mesh
            .clone()
            .unwrap();
        let colors = app
            .world()
            .resource::<Assets<Mesh>>()
            .get(&mesh)
            .unwrap()
            .attribute(Mesh::ATTRIBUTE_COLOR)
            .unwrap()
            .clone();
        app.world_mut()
            .resource_mut::<VisualBattleFrame>()
            .source
            .as_mut()
            .unwrap()
            .battler_palettes[0] = [[0.9, 0.1, 0.3, 1.0]; 4];
        app.update();
        assert_eq!(active(&mut app), [true, false]);
        assert_ne!(
            app.world()
                .resource::<Assets<Mesh>>()
                .get(&mesh)
                .unwrap()
                .attribute(Mesh::ATTRIBUTE_COLOR)
                .unwrap()
                .get_bytes(),
            colors.get_bytes()
        );
    }

    #[test]
    fn capture_target_identity_and_camera_transform_both_invalidate() {
        let mut app = capture_app();
        let size = app.world().resource::<BattleScene>().row_sizes[0];
        let replacement = app
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .add(battle_row_target_image(size));
        app.update(); // Publish the new image event before switching identities.
        submit(&mut app);
        app.update();
        app.world_mut().resource_mut::<BattleScene>().row_targets[0] = replacement.clone();
        {
            let world = app.world_mut();
            for (row, mut camera) in world
                .query::<(&BattleRowCamera, &mut Camera)>()
                .iter_mut(world)
            {
                if row.0 == 0 {
                    camera.target = RenderTarget::Image(replacement.clone());
                }
            }
        }
        app.update();
        assert_eq!(
            active(&mut app),
            [true, false],
            "same-size existing image needs its own receipt"
        );
        submit(&mut app);
        app.update();
        let before_camera = app.world().resource::<BattleSceneLayout>().camera;
        app.world_mut()
            .resource_mut::<BattleSceneLayout>()
            .camera
            .translation
            .x += 1.0;
        app.update();
        let expected_camera = app.world().resource::<BattleSceneLayout>().camera;
        assert_ne!(expected_camera, before_camera);
        {
            let world = app.world_mut();
            assert!(
                world
                    .query_filtered::<&Transform, With<BattleRowCamera>>()
                    .iter(world)
                    .all(|pose| *pose == expected_camera)
            );
        }
        assert_eq!(
            active(&mut app),
            [true, true],
            "the shared camera reframes both actor targets"
        );
    }

    #[test]
    fn unsupported_appearances_do_not_prewarm_or_reuse_old_captures() {
        let mut app = capture_app();
        submit(&mut app);
        app.update();
        let before_layout = app.world().resource::<BattleSceneLayout>().clone();
        {
            let mut frame = app.world_mut().resource_mut::<VisualBattleFrame>();
            frame.source = None;
            frame.battlers[0].as_mut().unwrap().allow_species_model = false;
        }
        app.update();
        assert_ne!(*app.world().resource::<BattleSceneLayout>(), before_layout);
        assert_eq!(
            active(&mut app),
            [false, true],
            "unsupported actor stays off; the supported actor must recapture the new shared view"
        );
        submit(&mut app);
        app.update();
        assert_eq!(active(&mut app), [false; 2]);
        assert_eq!(
            app.world().resource::<BattleViewStatus>().row_capture_ready,
            [false, true]
        );
        app.world_mut().resource_mut::<VisualBattleFrame>().battlers[0]
            .as_mut()
            .unwrap()
            .allow_species_model = true;
        app.update();
        assert_eq!(
            active(&mut app),
            [true, true],
            "restoring the model restores framing for both participants"
        );
    }
}
