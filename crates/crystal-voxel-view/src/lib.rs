#![forbid(unsafe_code)]

//! Optional, presentation-only voxel view for Crystal's Bevy shell.

mod azalea_gym;
mod barn;
mod battle_tower;
mod building_catalog;
mod building_style;
mod cafe;
mod camera;
mod live_profiles;
mod casino;
mod cave;
mod celadon_gym;
mod cerulean_gym;
mod cut_tree;
mod dance_theater;
mod elevation;
mod elite_four_room;
mod facility;
mod facility_divider;
mod flower;
mod flower_shop;
mod footing;
mod forest;
mod fuchsia_gym;
mod gate;
mod goldenrod_underground;
mod grass;
mod hall_of_fame;
mod house;
mod ice_path;
mod interior;
mod johto_fence;
mod kanto_cliff;
mod kanto_post;
mod lab;
mod mart;
mod mesh;
mod modern_route;
mod olivine_gym;
mod park;
mod players_house;
mod pokecenter;
mod pokecom;
mod port;
mod power_plant;
mod profile;
mod rock_platform;
mod rocket_base;
mod ruins_of_alph;
mod saffron_gym;
mod ship;
mod sign;
mod terrain_tracking;
mod tower;
mod train_station;
mod underground_boundary;
mod underground_path;
mod vermilion;
mod vermilion_gym;
mod violet_gym;
mod viridian_gym;
mod warehouse;
mod waterfall;
mod wise_trios;

use std::collections::{HashMap, HashSet};

#[cfg(not(target_arch = "wasm32"))]
use bevy::tasks::AsyncComputeTaskPool;
#[cfg(not(target_arch = "wasm32"))]
use bevy::tasks::Task;
use bevy::{
    asset::{AssetId, load_internal_asset},
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    pbr::{
        CascadeShadowConfigBuilder, DirectionalLightShadowMap, ExtendedMaterial, FogFalloff, FogSettings, Material,
        MaterialExtension, MaterialPipeline, MaterialPipelineKey, MaterialPlugin,
    },
    prelude::*,
    render::{
        camera::{ClearColorConfig, OrthographicProjection, Projection, RenderTarget, ScalingMode},
        mesh::MeshVertexBufferLayoutRef,
        render_resource::{
            AsBindGroup, CompareFunction, DepthStencilState, Face, RenderPipelineDescriptor,
            ShaderRef, SpecializedMeshPipelineError,
        },
        view::RenderLayers,
    },
    tasks::futures_lite::future,
};
use crystal_render_api::{VisualActor, VisualActorId, VisualWorldFrame, WorldRenderSet};

pub use camera::{CAMERA_PITCH_DEGREES, VoxelCameraControls, VoxelCameraPose, camera_pose};
pub use footing::{
    actor_foot, footing_height, resolved_footing_height, tile_at_visual_point,
    visual_point_to_voxel,
};
pub use mesh::{
    CellCoverageKind, SurfaceMeshData, TerrainImageSamples, TerrainMeshError, audit_cell_coverage,
    audit_cell_coverage_on_map, build_terrain_mesh, build_terrain_mesh_with_images,
    build_terrain_mesh_with_samples,
};
pub use profile::{
    COMPACT_BUILDING_HEIGHT, CellShape, GROUND_HEIGHT, LARGE_BUILDING_HEIGHT, MAX_PROFILE_HEIGHT,
    MIN_PROFILE_HEIGHT, SOURCE_TILE_HEIGHT, SolidKind, WATER_HEIGHT, shape_for_source,
    support_height, supports_frame_profile,
};

/// Parking layer used to keep the classic overworld out of every active
/// camera while the user has manually selected 2.5D.
pub const HIDDEN_CLASSIC_WORLD_RENDER_LAYER: usize = 30;

const VOXEL_RENDER_LAYER: usize = 31;
/// Logical LCD grid used only to derive source-pixel camera scale. The
/// optional terrain grid may extend beyond it in every direction.
pub const EXPECTED_GRID_SIZE: UVec2 = UVec2::new(20, 18);
const ACTOR_BASE_CAMERA_PULL: f32 = 6.0;
const ACTOR_CARD_HEIGHT: f32 = 16.0;
const ACTOR_FOOT_ANCHOR: f32 = 8.0;
const MIN_PULL_SINE: f32 = 0.2;
const ABOVE_PRIORITY_EXTRA_PULL: f32 = 0.70;
const SILHOUETTE_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x7a88_22d4_d773_4874_94d3_c20a_0dcc_f21a);
#[derive(Clone, Copy, Debug, Default)]
pub struct VoxelViewPlugin;

impl Plugin for VoxelViewPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            VOXEL_SURFACE_SHADER_HANDLE,
            "voxel_surface.wgsl",
            Shader::from_wgsl
        );
        app.add_plugins(MaterialPlugin::<VoxelMaterial>::default());
        load_internal_asset!(
            app,
            SILHOUETTE_SHADER_HANDLE,
            "occlusion_silhouette.wgsl",
            Shader::from_wgsl
        );
        app.add_plugins(MaterialPlugin::<OcclusionSilhouetteMaterial>::default());
        app.init_resource::<VoxelViewSettings>()
            .init_resource::<VoxelViewStatus>()
            .init_resource::<live_profiles::LiveProfiles>()
            .add_systems(Update, live_profiles::reload.before(sync_voxel_view))
            .init_resource::<ActorScreenHeads>()
            .init_resource::<VoxelScene>()
            .init_resource::<TerrainRevisionCache>()
            .init_resource::<TerrainBuildQueue>()
            .init_resource::<ActorIdCache>()
            .init_resource::<PlayerSilhouetteCache>()
            .add_systems(PostUpdate, project_actor_heads.after(bevy::transform::TransformSystem::TransformPropagate).in_set(ActorHeadProjection))
            .add_systems(Startup, setup_voxel_view)
            .add_systems(Update, toggle_voxel_view.before(sync_voxel_view))
            .add_systems(Update, sync_voxel_view.in_set(WorldRenderSet::RenderSync))
            .add_systems(Update, sync_voxel_atmosphere.after(sync_voxel_view))
            .add_systems(
                Update,
                sync_player_silhouette_system
                    .after(sync_voxel_view)
                    .in_set(WorldRenderSet::RenderSync),
            );
    }
}

/// Runtime presentation switch used by the location tester and optional-mod
/// builds. It cannot affect simulation because only the renderer reads it.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct VoxelViewSettings {
    pub enabled: bool,
    pub allow_f3_toggle: bool,
    pub camera: VoxelCameraControls,
}

/// Normalized canvas positions for browser overlays, derived from rendered cards.
#[derive(Resource, Default)]
pub struct ActorScreenHeads(pub HashMap<VisualActorId, Vec2>);
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActorHeadProjection;

fn project_actor_heads(
    status: Res<VoxelViewStatus>, cache: Res<ActorIdCache>,
    cameras: Query<(&Camera, &GlobalTransform), With<VoxelWorldCamera>>,
    cards: Query<&GlobalTransform, With<VoxelActorCard>>,
    mut heads: ResMut<ActorScreenHeads>,
) {
    heads.0.clear();
    if !status.active { return; }
    let Ok((camera, camera_transform)) = cameras.get_single() else { return; };
    let (Some(size), Some(rect)) = (camera.logical_target_size(), camera.logical_viewport_rect()) else { return; };
    for (id, entity) in &cache.entities {
        if !matches!(id, VisualActorId::Player | VisualActorId::RemotePlayer(_)) { continue; }
        let Ok(transform) = cards.get(*entity) else { continue; };
        if let Some(point) = camera.world_to_viewport(camera_transform, transform.transform_point(Vec3::Y)) {
            let normalized = (point + rect.min) / size;
            if normalized.is_finite() && normalized.cmpge(Vec2::ZERO).all() && normalized.cmple(Vec2::ONE).all() {
                heads.0.insert(*id, normalized);
            }
        }
    }
}

/// Observable presentation state for the developer location tester. Gameplay
/// never reads this resource.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct VoxelViewStatus {
    pub active: bool,
    pub active_frames: u32,
    /// Completed or pending geometry builds; useful for movement profiling.
    pub terrain_builds: u64,
    /// True until the current profile revision has reached the GPU scene.
    pub profiles_pending: bool,
    pub inactive_reason: Option<String>,
}

impl Default for VoxelViewSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            allow_f3_toggle: true,
            camera: VoxelCameraControls::default(),
        }
    }
}

fn toggle_voxel_view(keyboard: Res<ButtonInput<KeyCode>>, mut settings: ResMut<VoxelViewSettings>) {
    if settings.allow_f3_toggle && keyboard.just_pressed(KeyCode::F3) {
        settings.enabled = !settings.enabled;
    }
}

#[derive(Component)]
struct VoxelWorldCamera;

#[derive(Component)]
struct VoxelTerrain;

#[derive(Component)]
struct VoxelActorCard;

#[derive(Component)]
struct VoxelActorSilhouette;

type VoxelWorldCameraFilter = (
    With<VoxelWorldCamera>,
    Without<VoxelTerrain>,
    Without<VoxelActorCard>,
);
type VoxelTerrainFilter = (With<VoxelTerrain>, Without<VoxelActorCard>);

#[derive(Resource, Default)]
struct VoxelScene {
    camera: Option<Entity>,
    actor_quad: Option<Handle<Mesh>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TerrainCacheKey {
    map_id: std::sync::Arc<str>,
    grid_origin: IVec2,
    revision: u64,
    profiles_revision: u64,
    viewport_bits: [u32; 2],
    tile_bits: [u32; 2],
    grid_size: UVec2,
}

impl TerrainCacheKey {
    fn from_frame(frame: &VisualWorldFrame) -> Self {
        Self {
            map_id: frame.map_id.clone(),
            grid_origin: frame.grid_origin,
            revision: frame.terrain_revision,
            profiles_revision: 0,
            viewport_bits: [
                frame.viewport_size.x.to_bits(),
                frame.viewport_size.y.to_bits(),
            ],
            tile_bits: [frame.tile_size.x.to_bits(), frame.tile_size.y.to_bits()],
            grid_size: frame.grid_size,
        }
    }
}

#[derive(Resource, Default)]
struct TerrainRevisionCache {
    built_frame: Option<VisualWorldFrame>,
    built_footing_heights: Vec<f32>,
    footing_origin: Option<IVec2>,
    key: Option<TerrainCacheKey>,
    instances_root: Option<Entity>,
    textured_entity: Option<Entity>,
    solid_entity: Option<Entity>,
    animated_textured_entity: Option<Entity>,
    animated_solid_entity: Option<Entity>,
    textured_mesh: Option<Handle<Mesh>>,
    solid_mesh: Option<Handle<Mesh>>,
    animated_textured_mesh: Option<Handle<Mesh>>,
    animated_solid_mesh: Option<Handle<Mesh>>,
    textured_material: Option<Handle<VoxelMaterial>>,
    solid_material: Option<Handle<VoxelMaterial>>,
    footing_heights: Vec<f32>,
}

#[derive(Resource, Default)]
struct TerrainBuildQueue {
    started: u64,
    key: Option<TerrainCacheKey>,
    #[cfg(not(target_arch = "wasm32"))]
    task: Option<Task<TerrainBuildResult>>,
    #[cfg(target_arch = "wasm32")]
    completed: Option<TerrainBuildResult>,
}

struct TerrainBuildResult {
    key: TerrainCacheKey,
    frame: VisualWorldFrame,
    terrain: Result<BuiltTerrain, TerrainMeshError>,
}

struct BuiltTerrain {
    background: Option<(Mesh, Handle<Image>)>,
    instances: Vec<(Mesh, Vec<[f32; 3]>)>,
    footing_heights: Vec<f32>,
    textured_mesh: Mesh,
    solid_mesh: Mesh,
    animated_textured_mesh: Mesh,
    animated_solid_mesh: Mesh,
}

fn should_start_terrain_build(
    cached_key: Option<&TerrainCacheKey>,
    queued_key: Option<&TerrainCacheKey>,
    next_key: &TerrainCacheKey,
) -> bool {
    cached_key != Some(next_key) && queued_key.is_none()
}

fn must_validate_static_frame(
    validated_key: Option<&TerrainCacheKey>,
    frame: &VisualWorldFrame,
) -> bool {
    validated_key != Some(&TerrainCacheKey::from_frame(frame))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TerrainSyncState {
    Ready,
    Pending,
}

fn must_wait_for_initial_terrain(state: TerrainSyncState, cache: &TerrainRevisionCache) -> bool {
    state == TerrainSyncState::Pending && cache.key.is_none()
}

struct ActorTextureAssets {
    material: Handle<VoxelMaterial>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ActorMaterialKey {
    texture: AssetId<Image>,
    remote_player: bool,
}

#[derive(Resource, Default)]
struct ActorIdCache {
    entities: HashMap<VisualActorId, Entity>,
    textures: HashMap<ActorMaterialKey, ActorTextureAssets>,
}

#[derive(Resource, Default)]
struct PlayerSilhouetteCache {
    entity: Option<Entity>,
    texture: Option<AssetId<Image>>,
    material: Option<Handle<OcclusionSilhouetteMaterial>>,
}

#[derive(Asset, AsBindGroup, TypePath, Clone, Debug)]
struct OcclusionSilhouetteMaterial {
    #[texture(0)]
    #[sampler(1)]
    texture: Handle<Image>,
    #[uniform(2)]
    color: LinearRgba,
}

impl Material for OcclusionSilhouetteMaterial {
    fn fragment_shader() -> ShaderRef {
        SILHOUETTE_SHADER_HANDLE.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(depth_stencil) = descriptor.depth_stencil.as_mut() {
            // Bevy's 3D camera uses reversed depth. Strict Less draws the
            // silhouette only where a closer terrain depth already exists;
            // equal-depth pixels from the normal player card remain untouched.
            configure_silhouette_depth(depth_stencil);
        }
        Ok(())
    }
}

fn configure_silhouette_depth(depth_stencil: &mut DepthStencilState) {
    depth_stencil.depth_compare = CompareFunction::Less;
    depth_stencil.depth_write_enabled = false;
}

#[allow(clippy::too_many_arguments)]
fn setup_voxel_view(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut scene: ResMut<VoxelScene>,
) {
    // Face colors retain their authored shade; this light supplies only
    // the scene's depth-tested cast-shadow visibility.
    commands.insert_resource(DirectionalLightShadowMap { size: 2048 });
    let initial_viewport = Vec2::new(160.0, 144.0);
    let pose = camera_pose(initial_viewport);
    let camera = commands
        .spawn((
            Camera3dBundle {
                camera: Camera {
                    order: 0,
                    is_active: false,
                    target: RenderTarget::Window(bevy::window::WindowRef::Primary),
                    // Full-map terrain extends beyond the LCD crop. Clear the
                    // finite horizon to sky so the faithful 2D viewport can
                    // never leak through as a vertical backdrop behind it.
                    clear_color: ClearColorConfig::Custom(Color::srgb(0.72, 0.83, 0.78)),
                    ..default()
                },
                projection: Projection::Orthographic(OrthographicProjection {
                    near: 0.1,
                    far: 4096.0,
                    scaling_mode: ScalingMode::Fixed {
                        width: initial_viewport.x,
                        height: initial_viewport.y,
                    },
                    ..default()
                }),
                transform: pose.transform(),
                tonemapping: Tonemapping::None,
                deband_dither: DebandDither::Disabled,
                ..default()
            },
            RenderLayers::layer(VOXEL_RENDER_LAYER),
            FogSettings {
                color: Color::NONE,
                directional_light_color: Color::NONE,
                ..default()
            },
            VoxelWorldCamera,
        ))
        .id();
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: Color::WHITE,
                illuminance: 3_000.0,
                shadows_enabled: true,
                shadow_depth_bias: 0.1,
                shadow_normal_bias: 2.5,
            },
            cascade_shadow_config: CascadeShadowConfigBuilder {
                num_cascades: 1,
                maximum_distance: 4096.0,
                first_cascade_far_bound: 4096.0,
                ..default()
            }
            .build(),
            // A southeast light gives the small world readable northwest cast
            // shadows while keeping the source sprites' front faces bright.
            // Voxel world coordinates are +X east and +Z south. Put the sun
            // in that actual quadrant so the camera-facing facade receives
            // direct light, matching the reference renderer's south/east
            // face model. The previous negative X/Z position lit every
            // house from behind and made its mapped windows and door nearly
            // black even though their source pixels were correct.
            transform: Transform::from_xyz(180.0, 320.0, 220.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        RenderLayers::layer(VOXEL_RENDER_LAYER),
    ));

    scene.camera = Some(camera);
    scene.actor_quad = Some(meshes.add(actor_quad_mesh()));
}

#[allow(clippy::too_many_arguments)]
fn sync_voxel_view(
    frame: Res<VisualWorldFrame>,
    presentation: (Res<VoxelViewSettings>, Res<live_profiles::LiveProfiles>),
    mut status: ResMut<VoxelViewStatus>,
    mut last_failure: Local<Option<String>>,
    mut validated_frame_key: Local<Option<TerrainCacheKey>>,
    mut commands: Commands,
    scene: Res<VoxelScene>,
    mut terrain_cache: ResMut<TerrainRevisionCache>,
    mut terrain_builds: ResMut<TerrainBuildQueue>,
    mut actor_cache: ResMut<ActorIdCache>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
    mut cameras: Query<(&mut Camera, &mut Projection, &mut Transform), VoxelWorldCameraFilter>,
    mut terrain_entities: Query<(&mut Visibility, &mut Transform), VoxelTerrainFilter>,
    mut actor_entities: Query<
        (
            &mut Transform,
            &mut Visibility,
            &mut Handle<Mesh>,
            &mut Handle<VoxelMaterial>,
        ),
        (
            With<VoxelActorCard>,
            Without<VoxelActorSilhouette>,
            Without<VoxelWorldCamera>,
            Without<VoxelTerrain>,
        ),
    >,
) {
    let (settings, profiles) = presentation;
    status.profiles_pending = terrain_cache.key.as_ref()
        .is_none_or(|key| key.profiles_revision != profiles.revision);
    if status.profiles_pending { status.active_frames = 0; }
    if status.active
        && !profiles.is_changed()
        && !status.profiles_pending
        && !frame.is_changed()
        && !settings.is_changed()
        && terrain_builds.key.is_none()
    {
        // The published frame is retained and Bevy change detection proves
        // there is no camera, actor, texture, or terrain work to synchronize.
        // Avoid validating and profile-scanning the complete terrain grid on
        // every vsynced host frame while the game is visually idle.
        status.active_frames = status.active_frames.saturating_add(1);
        return;
    }
    let failure = if !settings.enabled {
        Some("disabled")
    } else if !frame.active {
        Some("waiting for an active world frame")
    } else if must_validate_static_frame(validated_frame_key.as_ref(), &frame)
        && frame.validate().is_err()
    {
        Some("world frame validation failed")
    } else if !supports_frame_profile(&frame) {
        Some("world frame is not supported by the visual profile")
    } else if !images.contains(&frame.map_texture) {
        Some("composed map texture is unavailable")
    } else {
        None
    };
    if failure.is_none() {
        // Terrain geometry and tile identities are immutable for a published
        // revision. Movement republishes only center/actor transforms, so do
        // not rebuild the validation hash set and scan thousands of terrain
        // cells on every display refresh.
        *validated_frame_key = Some(TerrainCacheKey::from_frame(&frame));
    }
    let pose = settings.camera.pose(frame.viewport_size);
    let valid = failure.is_none();
    let output_ready = set_output_active(&scene, valid, &mut cameras, &frame, pose);
    if valid && !output_ready {
        let failure = "voxel world camera is unavailable";
        status.active = false;
        status.active_frames = 0;
        status.inactive_reason = Some(failure.to_owned());
        report_inactive_change(&mut last_failure, failure);
        return;
    }
    if !valid {
        let failure = failure.unwrap_or("unknown reason");
        status.active = false;
        status.active_frames = 0;
        status.inactive_reason = Some(failure.to_owned());
        report_inactive_change(&mut last_failure, failure);
        return;
    }

    let terrain_state = match sync_terrain(
        &frame,
        &profiles,
        &mut commands,
        &mut terrain_cache,
        &mut terrain_builds,
        &mut images,
        &mut meshes,
        &mut materials,
        &mut terrain_entities,
    ) {
        Ok(state) => state,
        Err(error) => {
            let failure = format!("terrain sync failed: {error:?}");
            status.active = false;
            status.active_frames = 0;
            status.inactive_reason = Some(failure.clone());
            report_inactive_change(&mut last_failure, &failure);
            set_output_active(&scene, false, &mut cameras, &frame, pose);
            return;
        }
    };
    status.terrain_builds = terrain_builds.started;
    if must_wait_for_initial_terrain(terrain_state, &terrain_cache) {
        // Initial activation has no authored mesh to present yet. Keep the
        // manually selected 2.5D presentation inactive until the first build
        // completes; the classic world remains parked on its hidden layer.
        // Subsequent revisions keep the last complete terrain alive while its
        // replacement builds.
        status.active = false;
        status.active_frames = 0;
        status.inactive_reason = Some("building authored terrain".to_owned());
        set_output_active(&scene, false, &mut cameras, &frame, pose);
        return;
    }

    let Some(actor_quad) = scene.actor_quad.as_ref() else {
        status.active = false;
        status.active_frames = 0;
        status.inactive_reason = Some("actor card mesh is unavailable".to_owned());
        report_inactive_change(&mut last_failure, "actor card mesh is unavailable");
        set_output_active(&scene, false, &mut cameras, &frame, pose);
        return;
    };
    if let Err(error) = sync_actor_cards(
        &frame,
        &terrain_cache.footing_heights,
        pose,
        actor_quad,
        &mut commands,
        &mut actor_cache,
        &images,
        &mut materials,
        &mut actor_entities,
    ) {
        let failure = format!("actor sync failed: {error:?}");
        status.active = false;
        status.active_frames = 0;
        status.inactive_reason = Some(failure.clone());
        report_inactive_change(&mut last_failure, &failure);
        set_output_active(&scene, false, &mut cameras, &frame, pose);
        return;
    }
    status.active = true;
    status.active_frames = status.active_frames.saturating_add(1);
    status.inactive_reason = None;
    last_failure.take();
}

fn report_inactive_change(last_failure: &mut Option<String>, failure: &str) {
    if last_failure.as_deref() != Some(failure) {
        if failure != "disabled" {
            bevy::log::warn!("optional 2.5D renderer is inactive: {failure}");
        }
        *last_failure = Some(failure.to_owned());
    }
}

fn set_output_active(
    scene: &VoxelScene,
    active: bool,
    cameras: &mut Query<(&mut Camera, &mut Projection, &mut Transform), VoxelWorldCameraFilter>,
    frame: &VisualWorldFrame,
    pose: VoxelCameraPose,
) -> bool {
    let mut camera_ready = false;
    if let Some(camera_entity) = scene.camera
        && let Ok((mut camera, mut projection, mut transform)) = cameras.get_mut(camera_entity)
    {
        camera_ready = true;
        // The direct world camera is active only for a complete validated
        // frame; otherwise the untouched classic layer remains authoritative.
        if camera.is_active != active {
            camera.is_active = active;
        }
        if active {
            let next_clear_color = voxel_clear_color(frame);
            let clear_color_changed = !matches!(
                camera.clear_color,
                ClearColorConfig::Custom(current) if current == next_clear_color
            );
            if clear_color_changed {
                camera.clear_color = ClearColorConfig::Custom(next_clear_color);
            }
            let next_transform = pose.transform();
            if *transform != next_transform {
                *transform = next_transform;
            }
            if voxel_projection_needs_update(&projection, pose) {
                *projection = Projection::Perspective(PerspectiveProjection {
                    fov: pose.vertical_fov_radians,
                    near: pose.near,
                    far: pose.far,
                    ..default()
                });
            }
        }
    }
    camera_ready
}

fn voxel_projection_needs_update(projection: &Projection, pose: VoxelCameraPose) -> bool {
    match projection {
        Projection::Perspective(current) => {
            current.fov != pose.vertical_fov_radians
                || current.near != pose.near
                || current.far != pose.far
        }
        Projection::Orthographic(_) => true,
    }
}

fn sync_voxel_atmosphere(
    frame: Res<VisualWorldFrame>,
    profiles: Res<live_profiles::LiveProfiles>,
    settings: Res<VoxelViewSettings>,
    mut cameras: Query<&mut FogSettings, With<VoxelWorldCamera>>,
) {
    if !frame.is_changed() && !profiles.is_changed() && !settings.is_changed() {
        return;
    }
    let atmosphere = profiles.document.atmosphere.as_ref().filter(|atmosphere| {
        frame.active && settings.enabled
            && atmosphere.maps.iter().any(|map| map == frame.map_id.as_ref())
    });
    for mut fog in &mut cameras {
        if let Some(atmosphere) = atmosphere {
            let pose = settings.camera.pose(frame.viewport_size);
            let target_distance = pose.eye.distance(pose.target);
            fog.color = voxel_clear_color(&frame).with_alpha(atmosphere.opacity);
            fog.falloff = FogFalloff::Linear {
                start: target_distance + atmosphere.start_tiles * frame.tile_size.y,
                end: target_distance + atmosphere.end_tiles * frame.tile_size.y,
            };
        } else {
            fog.color = Color::NONE;
        }
    }
}

fn voxel_clear_color(frame: &VisualWorldFrame) -> Color {
    let tileset = frame
        .tiles
        .first()
        .map(|tile| tile.source.tileset_id.as_ref());
    if tileset == Some("game_corner") {
        // The casino wall backs onto unlit interior void, not an outdoor
        // horizon. This also matches the black source course above the wall.
        Color::srgb(0.10, 0.09, 0.08)
    } else if matches!(tileset, Some("cave" | "dark_cave")) {
        // A cave is an enclosed room. Exposed space beyond its authored rock
        // boundary is unlit void, never the outdoor horizon color.
        Color::srgb(0.035, 0.025, 0.065)
    } else if tileset.is_some_and(crate::interior::has_back_wall) {
        // Authored wall courses define the room. Anything beyond their finite
        // edges is neutral void; never synthesize a full-width gray backdrop.
        Color::srgb(0.055, 0.050, 0.045)
    } else {
        Color::srgb(0.72, 0.83, 0.78)
    }
}

#[allow(clippy::too_many_arguments)]
fn sync_terrain(
    frame: &VisualWorldFrame,
    profiles: &live_profiles::LiveProfiles,
    commands: &mut Commands,
    cache: &mut TerrainRevisionCache,
    builds: &mut TerrainBuildQueue,
    images: &mut Assets<Image>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<VoxelMaterial>,
    terrain_entities: &mut Query<(&mut Visibility, &mut Transform), VoxelTerrainFilter>,
) -> Result<TerrainSyncState, TerrainSyncError> {
    let mut next_key = TerrainCacheKey::from_frame(frame);
    next_key.profiles_revision = profiles.revision;
    if builds
        .key
        .as_ref()
        .is_some_and(|key| key.map_id != frame.map_id || key.profiles_revision != profiles.revision)
    {
        #[cfg(not(target_arch = "wasm32"))]
        {
            builds.task = None;
        }
        #[cfg(target_arch = "wasm32")]
        {
            builds.completed = None;
        }
        builds.key = None;
    }
    if cache
        .built_frame
        .as_ref()
        .is_some_and(|built| built.map_id != frame.map_id)
    {
        cache.key = None;
        cache.built_frame = None;
        for entity in [
            cache.instances_root,
            cache.textured_entity,
            cache.solid_entity,
            cache.animated_textured_entity,
            cache.animated_solid_entity,
        ]
        .into_iter()
        .flatten()
        {
            if let Ok((mut visibility, _)) = terrain_entities.get_mut(entity) {
                *visibility = Visibility::Hidden;
            }
        }
    }
    if cache.key.as_ref() != Some(&next_key)
        && cache.key.as_ref().is_some_and(|key| key.profiles_revision == profiles.revision)
        && cache
            .built_frame
            .as_ref()
            .is_some_and(|built| terrain_tracking::can_reuse(built, frame))
    {
        cache.key = Some(next_key.clone());
    }
    if should_start_terrain_build(cache.key.as_ref(), builds.key.as_ref(), &next_key) {
        builds.started += 1;
        let mut build_frame = frame.clone();
        // The host overwrites its viewport image during every scroll. Own an
        // immutable copy for the lifetime of this mesh, including async work.
        let atlas = images
            .get(&frame.map_texture)
            .ok_or(TerrainSyncError::SourceTextureUnavailable)?
            .clone();
        build_frame.map_texture = images.add(atlas);
        let samples = TerrainImageSamples::capture(frame, images);
        let build_key = next_key.clone();
        let profile_document = profiles.document.clone();
        builds.key = Some(next_key.clone());
        let build = async move {
            #[cfg(feature = "operation-trace")]
            let _span = bevy::log::info_span!("crystal_terrain_build").entered();
            // SurfaceMeshData -> Bevy Mesh conversion walks and moves every
            // vertex/index buffer. Keep that work on the compute task too;
            // doing it when polling the completed build caused a deterministic
            // 30-45 ms main-thread hitch several seconds into 2.5D movement.
            let terrain = mesh::build_instanced_terrain_mesh_with_profiles(&build_frame, &samples, &profile_document)
                .map(|mut terrain| {
                    let animated_textured_mesh =
                        std::mem::take(&mut terrain.animated_textured).into_mesh();
                    let animated_solid_mesh =
                        std::mem::take(&mut terrain.animated_solid).into_mesh();
                    let background = terrain
                        .background
                        .take()
                        .map(|b| (b.mesh.into_mesh(), b.texture));
                    let instances = std::mem::take(&mut terrain.tree_instances)
                        .into_iter()
                        .map(|group| (group.mesh.into_mesh(), group.origins))
                        .collect();
                    let footing_heights = terrain.footing_heights.clone();
                    let (textured_mesh, solid_mesh) = terrain.into_meshes();
                    BuiltTerrain {
                        background,
                        instances,
                        footing_heights,
                        textured_mesh,
                        solid_mesh,
                        animated_textured_mesh,
                        animated_solid_mesh,
                    }
                });
            TerrainBuildResult {
                key: build_key,
                frame: build_frame,
                terrain,
            }
        };
        #[cfg(not(target_arch = "wasm32"))]
        {
            builds.task = Some(AsyncComputeTaskPool::get().spawn(build));
        }
        #[cfg(target_arch = "wasm32")]
        {
            builds.completed = Some(future::block_on(build));
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    let completed = builds
        .task
        .as_mut()
        .and_then(|task| future::block_on(future::poll_once(task)));
    #[cfg(target_arch = "wasm32")]
    let completed = builds.completed.take();
    if let Some(completed) = completed {
        #[cfg(not(target_arch = "wasm32"))]
        {
            builds.task = None;
        }
        builds.key = None;
        if completed.key == next_key || (completed.key.profiles_revision == profiles.revision
            && terrain_tracking::can_reuse(&completed.frame, frame)) {
            let terrain = completed.terrain.map_err(TerrainSyncError::Mesh)?;
            let profile_changed = cache.key.as_ref()
                .is_none_or(|key| key.profiles_revision != next_key.profiles_revision);
            apply_built_terrain(
                &completed.frame,
                next_key.clone(),
                terrain,
                commands,
                cache,
                meshes,
                materials,
                images,
            )?;
            if profile_changed {
                println!("geometry profile mesh applied: revision {}", next_key.profiles_revision);
            }
        }
    }

    let ready = cache.key.as_ref() == Some(&next_key);
    if ready {
        if let Some(built) = &mut cache.built_frame {
            if built.terrain_revision != frame.terrain_revision {
                let flowers_changed = terrain_tracking::refresh_animation(built, frame, images)
                    .map_err(|()| TerrainSyncError::SourceTextureUnavailable)?;
                if flowers_changed {
                    let (textured, solid) = mesh::build_animated_flowers(built, images)
                        .map_err(TerrainSyncError::Mesh)?;
                    update_mesh_asset(meshes, &mut cache.animated_textured_mesh, textured);
                    update_mesh_asset(meshes, &mut cache.animated_solid_mesh, solid);
                }
            }
        }
    }
    if cache.footing_origin != Some(frame.grid_origin) {
        if let Some(built) = &cache.built_frame {
            terrain_tracking::align_footings(
                built,
                frame,
                &cache.built_footing_heights,
                &mut cache.footing_heights,
            );
            cache.footing_origin = Some(frame.grid_origin);
        }
    }
    // Geometry and atlas keep their built grid origin; only camera motion
    // transforms them while a replacement is pending.
    if let Some(live_transform) = retained_terrain_transform(frame, cache) {
        for entity in [
            cache.instances_root,
            cache.textured_entity,
            cache.solid_entity,
            cache.animated_textured_entity,
            cache.animated_solid_entity,
        ]
        .into_iter()
        .flatten()
        {
            if let Ok((mut visibility, mut transform)) = terrain_entities.get_mut(entity) {
                if *visibility != Visibility::Visible {
                    *visibility = Visibility::Visible;
                }
                if *transform != live_transform {
                    *transform = live_transform;
                }
            }
        }
    }
    Ok(if ready {
        TerrainSyncState::Ready
    } else {
        TerrainSyncState::Pending
    })
}

#[allow(clippy::too_many_arguments)]
fn apply_built_terrain(
    frame: &VisualWorldFrame,
    key: TerrainCacheKey,
    terrain: BuiltTerrain,
    commands: &mut Commands,
    cache: &mut TerrainRevisionCache,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<VoxelMaterial>,
    images: &mut Assets<Image>,
) -> Result<(), TerrainSyncError> {
    cache.footing_origin = None;
    cache.built_frame = Some(frame.clone());
    cache.built_footing_heights = terrain.footing_heights;

    let textured_mesh_handle =
        update_mesh_asset(meshes, &mut cache.textured_mesh, terrain.textured_mesh);
    let solid_mesh_handle = update_mesh_asset(meshes, &mut cache.solid_mesh, terrain.solid_mesh);
    let animated_textured = update_mesh_asset(
        meshes,
        &mut cache.animated_textured_mesh,
        terrain.animated_textured_mesh,
    );
    let animated_solid = update_mesh_asset(
        meshes,
        &mut cache.animated_solid_mesh,
        terrain.animated_solid_mesh,
    );
    let textured_material_handle = if let Some(handle) = cache.textured_material.as_ref() {
        sync_terrain_texture(materials, handle, &frame.map_texture)?;
        handle.clone()
    } else {
        let handle = materials.add(textured_terrain_material(frame.map_texture.clone()));
        cache.textured_material = Some(handle.clone());
        handle
    };
    let solid_material_handle = if let Some(handle) = cache.solid_material.as_ref() {
        if materials.get(handle).is_none() {
            return Err(TerrainSyncError::CachedSolidMaterialUnavailable);
        }
        handle.clone()
    } else {
        let handle = materials.add(solid_terrain_material());
        cache.solid_material = Some(handle.clone());
        handle
    };

    if let Some(root) = cache.instances_root.take() {
        commands.entity(root).despawn_recursive();
    }
    let root = commands
        .spawn((
            SpatialBundle {
                transform: terrain_transform(frame),
                ..default()
            },
            VoxelTerrain,
        ))
        .id();
    if let Some((mesh, source)) = terrain.background {
        use bevy::render::texture::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
        let mut image = images
            .get(&source)
            .ok_or(TerrainSyncError::SourceTextureUnavailable)?
            .clone();
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            ..ImageSamplerDescriptor::nearest()
        });
        image.asset_usage = bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD;
        let material = materials.add(textured_terrain_material(images.add(image)));
        let child = commands
            .spawn((
                MaterialMeshBundle::<VoxelMaterial> {
                    mesh: meshes.add(mesh),
                    material,
                    ..default()
                },
                RenderLayers::layer(VOXEL_RENDER_LAYER),
            ))
            .id();
        commands.entity(root).add_child(child);
    }
    for (mesh, origins) in terrain.instances {
        let mesh = meshes.add(mesh);
        for origin in origins {
            let child = commands
                .spawn((
                    MaterialMeshBundle::<VoxelMaterial> {
                        mesh: mesh.clone(),
                        material: textured_material_handle.clone(),
                        transform: Transform::from_translation(Vec3::from_array(origin)),
                        ..default()
                    },
                    RenderLayers::layer(VOXEL_RENDER_LAYER),
                ))
                .id();
            commands.entity(root).add_child(child);
        }
    }
    cache.instances_root = Some(root);

    if cache.animated_textured_entity.is_none() {
        cache.animated_textured_entity = Some(spawn_terrain_entity(
            commands,
            animated_textured,
            textured_material_handle.clone(),
            frame,
        ));
    }
    if cache.animated_solid_entity.is_none() {
        cache.animated_solid_entity = Some(spawn_terrain_entity(
            commands,
            animated_solid,
            solid_material_handle.clone(),
            frame,
        ));
    }
    if cache.textured_entity.is_none() {
        cache.textured_entity = Some(spawn_terrain_entity(
            commands,
            textured_mesh_handle,
            textured_material_handle,
            frame,
        ));
    }
    if cache.solid_entity.is_none() {
        cache.solid_entity = Some(spawn_terrain_entity(
            commands,
            solid_mesh_handle,
            solid_material_handle,
            frame,
        ));
    }

    cache.key = Some(key);
    Ok(())
}

fn sync_terrain_texture(
    materials: &mut Assets<VoxelMaterial>,
    handle: &Handle<VoxelMaterial>,
    texture: &Handle<Image>,
) -> Result<(), TerrainSyncError> {
    let material = materials
        .get(handle)
        .ok_or(TerrainSyncError::CachedTexturedMaterialUnavailable)?;
    // Assets::get_mut emits Modified even if its caller never writes. Read
    // first so camera/actor movement cannot re-upload an unchanged material.
    if material.base.base_color_texture.as_ref() != Some(texture) {
        materials
            .get_mut(handle)
            .ok_or(TerrainSyncError::CachedTexturedMaterialUnavailable)?
            .base
            .base_color_texture = Some(texture.clone());
    }
    Ok(())
}

fn update_mesh_asset(
    meshes: &mut Assets<Mesh>,
    cache: &mut Option<Handle<Mesh>>,
    mesh: Mesh,
) -> Handle<Mesh> {
    if let Some(handle) = cache.as_ref() {
        // Render-world-only extraction takes the CPU asset. Keep the strong
        // handle stable and insert the next complete revision under its ID.
        meshes.insert(handle.id(), mesh);
        handle.clone()
    } else {
        let handle = meshes.add(mesh);
        *cache = Some(handle.clone());
        handle
    }
}

fn spawn_terrain_entity(
    commands: &mut Commands,
    mesh: Handle<Mesh>,
    material: Handle<VoxelMaterial>,
    frame: &VisualWorldFrame,
) -> Entity {
    commands
        .spawn((
            MaterialMeshBundle::<VoxelMaterial> {
                mesh,
                material,
                transform: terrain_transform(frame),
                ..default()
            },
            RenderLayers::layer(VOXEL_RENDER_LAYER),
            VoxelTerrain,
        ))
        .id()
}

fn textured_terrain_material(texture: Handle<Image>) -> VoxelMaterial {
    voxel_material(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(texture),
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        unlit: true,
        alpha_mode: AlphaMode::Opaque,
        cull_mode: Some(Face::Back),
        ..default()
    })
}

fn solid_terrain_material() -> VoxelMaterial {
    voxel_material(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: None,
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        unlit: true,
        alpha_mode: AlphaMode::Opaque,
        cull_mode: Some(Face::Back),
        ..default()
    })
}

// All targets use the same palette-preserving surface and real shadow pass.
type VoxelMaterial = ExtendedMaterial<StandardMaterial, VoxelSurface>;
const VOXEL_SURFACE_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0xf1d7_662b_5b61_49b2_a221_6f89195c8542);

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
struct VoxelSurface {}
impl MaterialExtension for VoxelSurface {
    fn fragment_shader() -> ShaderRef {
        VOXEL_SURFACE_SHADER_HANDLE.into()
    }
}

fn voxel_material(base: StandardMaterial) -> VoxelMaterial {
    ExtendedMaterial {
        base,
        extension: VoxelSurface {},
    }
}

#[allow(clippy::too_many_arguments)]
fn sync_actor_cards(
    frame: &VisualWorldFrame,
    footing_heights: &[f32],
    pose: VoxelCameraPose,
    actor_quad: &Handle<Mesh>,
    commands: &mut Commands,
    cache: &mut ActorIdCache,
    images: &Assets<Image>,
    materials: &mut Assets<VoxelMaterial>,
    actor_entities: &mut Query<
        (
            &mut Transform,
            &mut Visibility,
            &mut Handle<Mesh>,
            &mut Handle<VoxelMaterial>,
        ),
        (
            With<VoxelActorCard>,
            Without<VoxelActorSilhouette>,
            Without<VoxelWorldCamera>,
            Without<VoxelTerrain>,
        ),
    >,
) -> Result<(), ActorSyncError> {
    // The player is mandatory. An NPC whose foot lies outside the published
    // terrain halo is simply clipped, exactly like the classic sprite pass;
    // retiring the whole optional world here caused a 2D flash while walking.
    let mut prepared = Vec::with_capacity(frame.actors.len());
    for actor in &frame.actors {
        if !images.contains(&actor.texture) {
            return Err(ActorSyncError::TextureUnavailable(actor.id));
        }
        let Some(transform) = actor_transform(frame, actor, footing_heights, pose) else {
            if actor.id == VisualActorId::Player {
                return Err(ActorSyncError::FootingUnavailable(actor.id));
            }
            continue;
        };
        let mesh = actor_quad.clone();
        prepared.push((actor, transform, mesh));
    }

    let visible_ids: HashSet<_> = prepared.iter().map(|(actor, _, _)| actor.id).collect();
    let stale_ids: Vec<_> = cache
        .entities
        .keys()
        .copied()
        .filter(|id| !visible_ids.contains(id))
        .collect();
    for id in stale_ids {
        if let Some(entity) = cache.entities.remove(&id) {
            commands.entity(entity).despawn();
        }
    }

    let mut used_textures = HashSet::with_capacity(frame.actors.len());
    for (actor, transform, mesh) in prepared {
        used_textures.insert(actor_material_key(actor));
        let material = actor_material(actor, cache, materials);

        let existing = cache.entities.get(&actor.id).copied();
        if let Some(entity) = existing
            && let Ok((
                mut current_transform,
                mut visibility,
                mut current_mesh,
                mut current_material,
            )) = actor_entities.get_mut(entity)
        {
            if *current_transform != transform {
                *current_transform = transform;
            }
            if *visibility != Visibility::Visible {
                *visibility = Visibility::Visible;
            }
            if *current_mesh != mesh {
                *current_mesh = mesh;
            }
            if *current_material != material {
                *current_material = material;
            }
            continue;
        }

        let entity = commands
            .spawn((
                MaterialMeshBundle::<VoxelMaterial> {
                    mesh,
                    material,
                    transform,
                    ..default()
                },
                RenderLayers::layer(VOXEL_RENDER_LAYER),
                VoxelActorCard,
            ))
            .id();
        cache.entities.insert(actor.id, entity);
    }

    let unused_texture_ids: Vec<_> = cache
        .textures
        .keys()
        .copied()
        .filter(|id| !used_textures.contains(id))
        .collect();
    for id in unused_texture_ids {
        if let Some(assets) = cache.textures.remove(&id) {
            materials.remove(assets.material.id());
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn sync_player_silhouette_system(
    frame: Res<VisualWorldFrame>,
    settings: Res<VoxelViewSettings>,
    status: Res<VoxelViewStatus>,
    scene: Res<VoxelScene>,
    terrain_cache: Res<TerrainRevisionCache>,
    mut commands: Commands,
    mut cache: ResMut<PlayerSilhouetteCache>,
    mut materials: ResMut<Assets<OcclusionSilhouetteMaterial>>,
    mut entities: Query<
        (
            &mut Transform,
            &mut Visibility,
            &mut Handle<OcclusionSilhouetteMaterial>,
        ),
        (
            With<VoxelActorSilhouette>,
            Without<VoxelActorCard>,
            Without<VoxelWorldCamera>,
            Without<VoxelTerrain>,
        ),
    >,
) {
    let Some(actor_quad) = scene.actor_quad.as_ref() else {
        return;
    };
    if !status.active {
        if let Some(entity) = cache.entity
            && let Ok((_, mut visibility, _)) = entities.get_mut(entity)
        {
            *visibility = Visibility::Hidden;
        }
        return;
    }
    sync_player_silhouette(
        &frame,
        &terrain_cache.footing_heights,
        settings.camera.pose(frame.viewport_size),
        actor_quad,
        &mut commands,
        &mut cache,
        &mut materials,
        &mut entities,
    );
}

fn sync_player_silhouette(
    frame: &VisualWorldFrame,
    footing_heights: &[f32],
    pose: VoxelCameraPose,
    actor_quad: &Handle<Mesh>,
    commands: &mut Commands,
    cache: &mut PlayerSilhouetteCache,
    materials: &mut Assets<OcclusionSilhouetteMaterial>,
    entities: &mut Query<
        (
            &mut Transform,
            &mut Visibility,
            &mut Handle<OcclusionSilhouetteMaterial>,
        ),
        (
            With<VoxelActorSilhouette>,
            Without<VoxelActorCard>,
            Without<VoxelWorldCamera>,
            Without<VoxelTerrain>,
        ),
    >,
) {
    let Some(player) = frame
        .actors
        .iter()
        .find(|actor| actor.id == VisualActorId::Player)
    else {
        if let Some(entity) = cache.entity.take() {
            commands.entity(entity).despawn();
        }
        if let Some(material) = cache.material.take() {
            materials.remove(material.id());
        }
        cache.texture = None;
        return;
    };
    let Some(transform) = actor_transform(frame, player, footing_heights, pose) else {
        return;
    };
    let texture_id = player.texture.id();
    let material = if cache.texture == Some(texture_id) {
        cache.material.clone()
    } else {
        if let Some(previous) = cache.material.take() {
            materials.remove(previous.id());
        }
        let material = materials.add(OcclusionSilhouetteMaterial {
            texture: player.texture.clone(),
            color: LinearRgba::new(1.0, 1.0, 1.0, 0.82),
        });
        cache.texture = Some(texture_id);
        cache.material = Some(material.clone());
        Some(material)
    };
    let Some(material) = material else {
        return;
    };

    if let Some(entity) = cache.entity
        && let Ok((mut current_transform, mut visibility, mut current_material)) =
            entities.get_mut(entity)
    {
        if *current_transform != transform {
            *current_transform = transform;
        }
        if *visibility != Visibility::Visible {
            *visibility = Visibility::Visible;
        }
        if *current_material != material {
            *current_material = material;
        }
        return;
    }
    cache.entity = Some(
        commands
            .spawn((
                MaterialMeshBundle {
                    mesh: actor_quad.clone(),
                    material,
                    transform,
                    ..default()
                },
                RenderLayers::layer(VOXEL_RENDER_LAYER),
                VoxelActorSilhouette,
            ))
            .id(),
    );
}

fn actor_material(
    actor: &VisualActor,
    cache: &mut ActorIdCache,
    materials: &mut Assets<VoxelMaterial>,
) -> Handle<VoxelMaterial> {
    let key = actor_material_key(actor);
    if let Some(assets) = cache.textures.get(&key) {
        return assets.material.clone();
    }
    let (base_color, alpha_mode) = if key.remote_player {
        (Color::srgba(0.48, 0.88, 1.0, 0.62), AlphaMode::Blend)
    } else {
        (Color::WHITE, AlphaMode::Mask(0.5))
    };
    let material = materials.add(voxel_material(StandardMaterial {
        base_color,
        base_color_texture: Some(actor.texture.clone()),
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        unlit: true,
        alpha_mode,
        cull_mode: None,
        ..default()
    }));
    cache.textures.insert(
        key,
        ActorTextureAssets {
            material: material.clone(),
        },
    );
    material
}

fn actor_material_key(actor: &VisualActor) -> ActorMaterialKey {
    ActorMaterialKey {
        texture: actor.texture.id(),
        remote_player: matches!(actor.id, VisualActorId::RemotePlayer(_)),
    }
}

fn actor_transform(
    frame: &VisualWorldFrame,
    actor: &VisualActor,
    footing_heights: &[f32],
    pose: VoxelCameraPose,
) -> Option<Transform> {
    let foot = actor_foot(actor);
    let height = resolved_footing_height(frame, foot, footing_heights)?;
    let mut position = visual_point_to_voxel(foot, height + 0.05);
    let rotation = if matches!(actor.id, VisualActorId::Object(_) | VisualActorId::Player) {
        // Directional character artwork supplies the visible side. Keep the card
        // vertical and its bottom pivot on the footing, with no camera pull.
        let toward_eye = pose.eye - pose.target;
        Quat::from_rotation_y(toward_eye.x.atan2(toward_eye.z))
    } else {
        let profile_scale = frame.tile_size.y / SOURCE_TILE_HEIGHT;
        let camera_pull = actor_camera_pull(actor, CAMERA_PITCH_DEGREES.to_radians()) * profile_scale;
        position += (pose.eye - pose.target).normalize_or_zero() * camera_pull;
        camera::card_rotation_toward_camera(pose)
    };
    let mut transform = Transform::from_translation(position).with_rotation(rotation);
    transform.scale = Vec3::new(
        if actor.flip_x {
            -actor.size.x
        } else {
            actor.size.x
        },
        if matches!(actor.id, VisualActorId::Object(_) | VisualActorId::Player) {
            // Preserve authored front-on proportions under the fixed terrain
            // pitch without leaning the sprite or lifting its bottom pivot.
            actor.size.y / CAMERA_PITCH_DEGREES.to_radians().cos()
        } else {
            actor.size.y
        },
        1.0,
    );
    Some(transform)
}

fn actor_camera_pull(actor: &VisualActor, pitch_radians: f32) -> f32 {
    let sine = pitch_radians.sin().max(MIN_PULL_SINE);
    let lean_overlap = (ACTOR_CARD_HEIGHT * pitch_radians.cos() - ACTOR_FOOT_ANCHOR).max(0.0);
    let normal_pull = ACTOR_BASE_CAMERA_PULL + lean_overlap / sine;
    normal_pull
        + if actor.above_priority {
            ABOVE_PRIORITY_EXTRA_PULL
        } else {
            0.0
        }
}

fn terrain_transform(frame: &VisualWorldFrame) -> Transform {
    Transform::from_xyz(frame.center.x, 0.0, -frame.center.y)
}

fn retained_terrain_transform(
    frame: &VisualWorldFrame,
    cache: &TerrainRevisionCache,
) -> Option<Transform> {
    cache.key.as_ref()?;
    let mut transform = terrain_transform(frame);
    if let Some(built) = &cache.built_frame {
        let offset = terrain_tracking::offset(built, frame);
        transform.translation += Vec3::new(offset.x, 0.0, offset.y);
    }
    Some(transform)
}

fn actor_quad_mesh() -> Mesh {
    use bevy::render::{
        mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology,
    };

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [-0.5, 0.0, 0.0],
            [0.5, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [-0.5, 1.0, 0.0],
        ],
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; 4]);
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
    );
    mesh.insert_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]));
    mesh
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TerrainSyncError {
    SourceTextureUnavailable,
    Mesh(TerrainMeshError),
    CachedTexturedMaterialUnavailable,
    CachedSolidMaterialUnavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActorSyncError {
    TextureUnavailable(VisualActorId),
    FootingUnavailable(VisualActorId),
}

#[cfg(test)]
mod renderer_tests {
    use bevy::render::render_resource::{DepthBiasState, StencilState, TextureFormat};

    use super::*;

    #[test]
    fn tree_instances_share_meshes_and_replacement_removes_the_old_hierarchy() {
        let mut world = World::new();
        let mut images = Assets::<Image>::default();
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<VoxelMaterial>::default();
        let mut cache = TerrainRevisionCache::default();
        let frame = VisualWorldFrame::default();
        let mut previous = Vec::new();
        for _ in 0..3 {
            let mut queue = bevy::ecs::world::CommandQueue::default();
            let mut commands = Commands::new(&mut queue, &world);
            let terrain = BuiltTerrain {
                background: None,
                instances: vec![(actor_quad_mesh(), vec![[0.0, 0.0, 0.0], [32.0, 4.0, 16.0]])],
                footing_heights: Vec::new(),
                textured_mesh: actor_quad_mesh(),
                solid_mesh: actor_quad_mesh(),
                animated_textured_mesh: actor_quad_mesh(),
                animated_solid_mesh: actor_quad_mesh(),
            };
            apply_built_terrain(
                &frame,
                TerrainCacheKey::from_frame(&frame),
                terrain,
                &mut commands,
                &mut cache,
                &mut meshes,
                &mut materials,
                &mut images,
            )
            .unwrap();
            queue.apply(&mut world);
            for old in previous.drain(..) {
                assert!(world.get_entity(old).is_none());
            }
            let root = cache.instances_root.unwrap();
            let children = world.get::<Children>(root).unwrap();
            assert_eq!(children.len(), 2);
            assert_eq!(
                world.get::<Handle<Mesh>>(children[0]),
                world.get::<Handle<Mesh>>(children[1])
            );
            assert_eq!(
                world.get::<Transform>(children[1]).unwrap().translation,
                Vec3::new(32.0, 4.0, 16.0)
            );
            previous.extend(children.iter().copied());
            previous.push(root);
        }
    }

    #[test]
    fn terrain_mesh_transfers_to_render_world_without_a_retained_cpu_copy() {
        let mesh = mesh::SurfaceMeshData::default().into_mesh();
        assert_eq!(
            mesh.asset_usage,
            bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD
        );
    }

    #[test]
    fn terrain_mesh_replacement_preserves_handle_after_render_world_takes_ownership() {
        let mut meshes = Assets::<Mesh>::default();
        let original = meshes.add(mesh::SurfaceMeshData::default().into_mesh());
        let mut cached = Some(original.clone());
        meshes.remove(original.id()); // Bevy's RENDER_WORLD extraction moves it out.
        let replacement = actor_quad_mesh();
        let vertices = replacement.get_vertex_buffer_data();
        let indices = replacement.indices().unwrap().iter().collect::<Vec<_>>();
        let handle = update_mesh_asset(&mut meshes, &mut cached, replacement);
        assert_eq!(handle.id(), original.id());
        let actual = meshes.get(&handle).unwrap();
        assert_eq!(actual.get_vertex_buffer_data(), vertices);
        assert_eq!(
            actual.indices().unwrap().iter().collect::<Vec<_>>(),
            indices
        );
    }

    fn sync_cached_terrain_for_test(
        frame: Res<VisualWorldFrame>,
        mut commands: Commands,
        mut cache: ResMut<TerrainRevisionCache>,
        mut builds: ResMut<TerrainBuildQueue>,
        mut images: ResMut<Assets<Image>>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<VoxelMaterial>>,
        mut entities: Query<(&mut Visibility, &mut Transform), VoxelTerrainFilter>,
    ) {
        assert_eq!(
            sync_terrain(
                &frame,
                &live_profiles::LiveProfiles::default(),
                &mut commands,
                &mut cache,
                &mut builds,
                &mut images,
                &mut meshes,
                &mut materials,
                &mut entities
            ),
            Ok(TerrainSyncState::Ready)
        );
    }

    #[test]
    fn retained_terrain_never_rebinds_to_the_mutable_live_atlas() {
        let mut app = App::new();
        let frame = VisualWorldFrame::default();
        let mut materials = Assets::<VoxelMaterial>::default();
        let material = materials.add(textured_terrain_material(frame.map_texture.clone()));
        let entity = app
            .world_mut()
            .spawn((VoxelTerrain, Visibility::Visible, terrain_transform(&frame)))
            .id();
        app.insert_resource(TerrainRevisionCache {
            key: Some(TerrainCacheKey::from_frame(&frame)),
            textured_material: Some(material.clone()),
            textured_entity: Some(entity),
            ..default()
        })
        .insert_resource(frame)
        .insert_resource(materials)
        .init_resource::<TerrainBuildQueue>()
        .init_resource::<Assets<Image>>()
        .init_resource::<Assets<Mesh>>()
        .add_event::<AssetEvent<VoxelMaterial>>()
        .add_systems(
            Update,
            (
                sync_cached_terrain_for_test,
                Assets::<VoxelMaterial>::asset_events,
            )
                .chain(),
        );
        app.update();
        app.world_mut()
            .resource_mut::<Events<AssetEvent<VoxelMaterial>>>()
            .clear();
        app.world_mut().clear_trackers();

        // Camera interpolation requires a terrain transform update, but must
        // not trigger material extraction and a GPU bind-group rebuild.
        app.world_mut().resource_mut::<VisualWorldFrame>().center.x += 1.0;
        app.update();
        assert_eq!(
            app.world()
                .resource::<Events<AssetEvent<VoxelMaterial>>>()
                .len(),
            0
        );
        app.world_mut().clear_trackers();
        app.update();
        assert!(
            !app.world()
                .entity(entity)
                .get_ref::<Transform>()
                .unwrap()
                .is_changed()
        );
        assert!(
            !app.world()
                .entity(entity)
                .get_ref::<Visibility>()
                .unwrap()
                .is_changed()
        );

        let replacement = Handle::weak_from_u128(999);
        app.world_mut()
            .resource_mut::<VisualWorldFrame>()
            .map_texture = replacement.clone();
        app.update();
        assert_eq!(
            app.world()
                .resource::<Assets<VoxelMaterial>>()
                .get(&material)
                .unwrap()
                .base
                .base_color_texture,
            Some(Handle::default())
        );
        assert_eq!(
            app.world()
                .resource::<Events<AssetEvent<VoxelMaterial>>>()
                .len(),
            0
        );
    }

    #[test]
    fn room_transition_hides_old_geometry_before_new_atlas_is_available() {
        for (old_map, new_map) in [
            ("PlayersHouse2F", "PlayersHouse1F"),
            ("PlayersHouse1F", "NewBarkTown"),
        ] {
            let mut app = App::new();
            let old = VisualWorldFrame {
                map_id: old_map.into(),
                ..default()
            };
            let mut live = old.clone();
            live.map_id = new_map.into();
            // Same revision and image handle, as happens while the host uploads
            // a new room into its reusable viewport texture.
            assert_ne!(
                TerrainCacheKey::from_frame(&old),
                TerrainCacheKey::from_frame(&live)
            );
            let entity = app
                .world_mut()
                .spawn((VoxelTerrain, Visibility::Visible, Transform::default()))
                .id();
            app.insert_resource(TerrainRevisionCache {
                key: Some(TerrainCacheKey::from_frame(&old)),
                built_frame: Some(old),
                textured_entity: Some(entity),
                ..default()
            }).insert_resource(live)
                .init_resource::<TerrainBuildQueue>()
                .init_resource::<Assets<Image>>()
                .init_resource::<Assets<Mesh>>()
                .init_resource::<Assets<VoxelMaterial>>()
                .add_systems(Update, |frame: Res<VisualWorldFrame>, mut commands: Commands,
                    mut cache: ResMut<TerrainRevisionCache>, mut builds: ResMut<TerrainBuildQueue>,
                    mut images: ResMut<Assets<Image>>, mut meshes: ResMut<Assets<Mesh>>,
                    mut materials: ResMut<Assets<VoxelMaterial>>,
                    mut entities: Query<(&mut Visibility, &mut Transform), VoxelTerrainFilter>| {
                    assert_eq!(sync_terrain(&frame, &live_profiles::LiveProfiles::default(), &mut commands, &mut cache, &mut builds,
                        &mut images, &mut meshes, &mut materials, &mut entities),
                        Err(TerrainSyncError::SourceTextureUnavailable));
                });
            app.update();
            let cache = app.world().resource::<TerrainRevisionCache>();
            assert!(cache.built_frame.is_none());
            assert!(must_wait_for_initial_terrain(
                TerrainSyncState::Pending,
                cache
            ));
            assert_eq!(
                *app.world().get::<Visibility>(entity).unwrap(),
                Visibility::Hidden
            );
        }
    }

    #[test]
    fn silhouette_pass_reads_only_strictly_closer_reverse_depth() {
        let mut depth = DepthStencilState {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: CompareFunction::GreaterEqual,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        };

        configure_silhouette_depth(&mut depth);

        assert_eq!(depth.depth_compare, CompareFunction::Less);
        assert!(!depth.depth_write_enabled);
    }

    #[test]
    fn optional_view_remains_disabled_by_default() {
        assert!(!VoxelViewSettings::default().enabled);
    }

    #[test]
    fn cave_clear_color_is_enclosed_void_not_outdoor_horizon() {
        let mut frame = VisualWorldFrame::default();
        frame.tiles.push(crystal_render_api::VisualTile {
            column: 0,
            row: 0,
            source: crystal_render_api::VisualTileSource {
                tileset_id: "cave".into(),
                metatile_id: 0,
                subtile_column: 0,
                subtile_row: 0,
                tile_index: 0,
            },
            texture: Handle::default(),
            priority: false,
        });

        assert_eq!(voxel_clear_color(&frame), Color::srgb(0.035, 0.025, 0.065));

        frame.tiles[0].source.tileset_id = "house".into();
        assert_eq!(
            voxel_clear_color(&frame),
            Color::srgb(0.055, 0.050, 0.045),
            "enclosed ordinary interiors must not inherit the outdoor horizon"
        );
    }

    #[test]
    fn retained_geometry_preserves_map_position_across_viewport_step() {
        let built = VisualWorldFrame {
            center: Vec2::new(5.0, 7.0),
            tile_size: Vec2::splat(8.0),
            ..default()
        };
        let cache = TerrainRevisionCache {
            key: Some(TerrainCacheKey::from_frame(&built)),
            built_frame: Some(built.clone()),
            ..default()
        };
        let mut live = built.clone();
        live.grid_origin.x += 2;
        live.center.x += 16.0;
        assert_eq!(
            retained_terrain_transform(&live, &cache).unwrap(),
            terrain_transform(&built),
            "scroll origin rebasing must not drag the old objects two tiles"
        );
        live.center.x -= 3.0;
        assert_eq!(
            retained_terrain_transform(&live, &cache)
                .unwrap()
                .translation
                .x,
            2.0,
            "subtile camera interpolation must remain fluid"
        );
    }

    #[test]
    fn terrain_rebuild_keeps_the_last_complete_voxel_frame_visible() {
        let mut cache = TerrainRevisionCache::default();
        assert!(must_wait_for_initial_terrain(
            TerrainSyncState::Pending,
            &cache
        ));
        cache.key = Some(TerrainCacheKey {
            map_id: Default::default(),
            grid_origin: IVec2::ZERO,
            revision: 1,
            profiles_revision: 0,
            viewport_bits: [160.0_f32.to_bits(), 144.0_f32.to_bits()],
            tile_bits: [8.0_f32.to_bits(), 8.0_f32.to_bits()],
            grid_size: UVec2::new(20, 18),
        });
        assert!(!must_wait_for_initial_terrain(
            TerrainSyncState::Pending,
            &cache
        ));
        assert!(!must_wait_for_initial_terrain(
            TerrainSyncState::Ready,
            &cache
        ));

        let frame = VisualWorldFrame {
            center: Vec2::new(24.0, -12.0),
            ..default()
        };
        let retained = retained_terrain_transform(&frame, &cache)
            .expect("a pending replacement keeps the completed terrain posed");
        assert_eq!(retained.translation, Vec3::new(24.0, 0.0, 12.0));
    }

    #[test]
    fn movement_coalesces_terrain_rebuilds_while_one_is_in_flight() {
        let key = |revision| TerrainCacheKey {
            map_id: Default::default(),
            grid_origin: IVec2::ZERO,
            revision,
            profiles_revision: 0,
            viewport_bits: [160.0_f32.to_bits(), 144.0_f32.to_bits()],
            tile_bits: [8.0_f32.to_bits(), 8.0_f32.to_bits()],
            grid_size: UVec2::new(84, 82),
        };
        let cached = key(1);
        let queued = key(2);
        let latest_movement = key(3);

        assert!(should_start_terrain_build(Some(&cached), None, &queued));
        assert!(
            !should_start_terrain_build(Some(&cached), Some(&queued), &latest_movement),
            "a newer walking viewport must not replace work already running on the compute pool"
        );
    }

    #[test]
    fn transform_only_frames_do_not_rescan_the_static_terrain_grid() {
        let mut frame = VisualWorldFrame {
            terrain_revision: 42,
            grid_origin: bevy::prelude::IVec2::ZERO,
            viewport_size: Vec2::new(160.0, 144.0),
            tile_size: Vec2::splat(8.0),
            grid_size: UVec2::new(84, 82),
            ..default()
        };
        let validated = TerrainCacheKey::from_frame(&frame);

        frame.center = Vec2::new(3.5, -1.25);
        assert!(
            !must_validate_static_frame(Some(&validated), &frame),
            "camera interpolation must not rescan unchanged terrain"
        );
        frame.terrain_revision += 1;
        assert!(must_validate_static_frame(Some(&validated), &frame));
    }

    #[test]
    fn movement_does_not_reset_an_unchanged_camera_projection() {
        let pose = camera_pose(Vec2::new(640.0, 576.0));
        let projection = Projection::Perspective(PerspectiveProjection {
            fov: pose.vertical_fov_radians,
            near: pose.near,
            far: pose.far,
            // The window system owns this runtime-computed field. Movement
            // must not replace it with PerspectiveProjection::default().
            aspect_ratio: 640.0 / 576.0,
        });

        assert!(!voxel_projection_needs_update(&projection, pose));
    }

    #[test]
    fn player_depth_pull_clears_same_level_terrain_at_forty_five_degrees() {
        let player = VisualActor {
            id: VisualActorId::Player,
            source_id: "player".into(),
            texture: Handle::weak_from_u128(1),
            center: Vec2::ZERO,
            size: Vec2::splat(16.0),
            flip_x: false,
            above_priority: false,
        };
        let pull = actor_camera_pull(&player, 45.0_f32.to_radians());
        let expected = ACTOR_BASE_CAMERA_PULL
            + (ACTOR_CARD_HEIGHT * 45.0_f32.to_radians().cos() - ACTOR_FOOT_ANCHOR)
                / 45.0_f32.to_radians().sin();
        assert!((pull - expected).abs() < 0.001);
        assert!(pull > 10.0 && pull < 11.0);
    }

    #[test]
    fn remote_player_uses_a_distinct_material_without_changing_its_footing() {
        let texture = Handle::weak_from_u128(1);
        let player = VisualActor {
            id: VisualActorId::Player,
            source_id: "player".into(),
            texture: texture.clone(),
            center: Vec2::ZERO,
            size: Vec2::splat(16.0),
            flip_x: false,
            above_priority: false,
        };
        let remote = VisualActor {
            id: VisualActorId::RemotePlayer(7),
            source_id: "remote_player".into(),
            texture,
            ..player.clone()
        };
        assert_ne!(actor_material_key(&player), actor_material_key(&remote));

        let frame = VisualWorldFrame {
            viewport_size: Vec2::splat(16.0),
            tile_size: Vec2::splat(8.0),
            grid_size: UVec2::new(2, 2),
            ..default()
        };
        let heights = vec![0.0; 4];
        assert_eq!(
            actor_transform(&frame, &player, &heights, camera_pose(frame.viewport_size)),
            actor_transform(&frame, &remote, &heights, camera_pose(frame.viewport_size)),
            "remote players must use the same world-to-voxel transform as the local player"
        );
    }

    #[test]
    fn scrolling_frame_keeps_actor_locked_to_retained_terrain() {
        let mut frame = VisualWorldFrame {
            center: Vec2::ZERO,
            viewport_size: Vec2::splat(16.0),
            tile_size: Vec2::splat(8.0),
            grid_size: UVec2::new(2, 2),
            ..default()
        };
        let mut player = VisualActor {
            id: VisualActorId::Player,
            source_id: "player".into(),
            texture: Handle::weak_from_u128(1),
            center: Vec2::ZERO,
            size: Vec2::splat(16.0),
            flip_x: false,
            above_priority: false,
        };
        let heights = vec![0.0; 4];
        let actor_before =
            actor_transform(&frame, &player, &heights, camera_pose(frame.viewport_size))
                .expect("player has footing before scroll");
        let terrain_before = terrain_transform(&frame);

        let scroll = Vec2::new(6.0, -3.0);
        frame.center += scroll;
        player.center += scroll;
        let actor_after =
            actor_transform(&frame, &player, &heights, camera_pose(frame.viewport_size))
                .expect("player has footing during scroll");
        let terrain_after = terrain_transform(&frame);

        assert_eq!(
            actor_after.translation - terrain_after.translation,
            actor_before.translation - terrain_before.translation,
            "actor and retained terrain must consume the identical live scroll"
        );
    }

    #[test]
    fn live_animation_texture_does_not_invalidate_terrain_geometry() {
        let mut first = VisualWorldFrame {
            terrain_revision: 42,
            grid_origin: bevy::prelude::IVec2::ZERO,
            map_texture: Handle::weak_from_u128(1),
            viewport_size: Vec2::new(160.0, 144.0),
            tile_size: Vec2::splat(8.0),
            grid_size: UVec2::new(20, 18),
            ..default()
        };
        let first_key = TerrainCacheKey::from_frame(&first);
        first.map_texture = Handle::weak_from_u128(2);
        assert_eq!(first_key, TerrainCacheKey::from_frame(&first));
    }
}
