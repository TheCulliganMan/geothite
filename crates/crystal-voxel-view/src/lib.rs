#![forbid(unsafe_code)]

//! Optional, presentation-only voxel view for Crystal's Bevy shell.

use bevy::camera::primitives::MeshAabb;

mod azalea_gym;
mod painted_stone;
mod battle_layout;
mod encounter_terrain;
mod battle_source_projection;
pub use battle_source_projection::{
    BattleSourceBodyRegistration, BattleSourceBodyRegistrations, BattleSourceProjection,
};
mod barn;
mod battle_species_models;
mod pidgeotto_rig;
mod species_rig;
mod battle_tower;
mod battle_view;
mod model_storage;
mod shared_model_geometry;
mod open_models;
mod open_people;
mod open_people_skin;
mod bicycle;
#[cfg(feature = "people-review")]
mod people_review;
#[cfg(feature = "people-review")]
pub use people_review::run as run_people_review;
mod npc_appearance;
mod hair_shapes;
mod npc_textures;
mod painted_torso;
mod bare_body;
pub use npc_textures::normalize_npc_texture;
pub use open_people::npc_texture_names;
pub use open_people::{install_open_people_bundle, PEOPLE as OPEN_PEOPLE};
mod scenery_surfaces;
mod scenery_finish;
mod grass_blades;
mod garment_fit;
mod garment_body;
mod hand_surface;
mod hand_digits;
mod hand_pose;
mod head_surface;
#[cfg(test)]
mod bald_scalp;
mod footwear;
mod uniform_caps;
mod uniform_details;
mod kimono_obi;
mod hair_flower;
mod headband;
mod cape;
mod kimono_sleeves;
mod moustache;
mod beads;
mod apricorn_basket;
mod apron;
mod robe_drape;
mod roof_tiles;
mod foliage;
mod understory;
pub use open_models::install_open_model_bundle;
pub use battle_layout::{BattleSceneLayout, BattleUiBounds};
pub use battle_view::{BattleCompositeImageCamera, BattleViewPlugin, BattleViewStatus, battle_source_overlay_rect};
mod building_catalog;
mod building_style;
mod cafe;
mod camera;
mod casino;
mod cave;
mod cave_water;
mod celadon_gym;
mod cerulean_gym;
mod cut_tree;
mod dance_theater;
mod dungeon_models;
mod elevation;
mod elite_four_room;
mod exterior_models;
mod facility;
mod facility_divider;
mod flower;
mod flower_shop;
mod footing;
mod forest;
mod fuchsia_gym;
mod gate;
mod gate_counter_models;
mod outdoor_sign_models;
mod cable_club_models;
mod goldenrod_underground;
mod grass;
mod hall_of_fame;
mod house;
mod ice_path;
mod interior;
mod interior_cutaway;
mod occluder_fade;
mod maze_reveal_batches;
mod interior_models;
mod johto_fence;
mod kanto_cliff;
mod kanto_post;
mod lab;
mod live_profiles;
mod mart;
mod mesh;
mod modern_route;
mod new_bark_actors;
mod new_bark_models;
mod olivine_gym;
mod park;
mod park_scenery_models;
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
mod terrain_batches;
mod tower;
mod train_station;
mod train_station_models;
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
    asset::{AssetId, load_internal_asset, RenderAssetUsages},
    camera::{ClearColorConfig, OrthographicProjection, Projection, ScalingMode, visibility::RenderLayers},
    light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    mesh::MeshVertexBufferLayoutRef,
    shader::ShaderRef,
    anti_alias::fxaa::Fxaa,

    pbr::{
        ExtendedMaterial, FogFalloff,
        DistanceFog, Material, MaterialExtension, MaterialPipeline, MaterialPipelineKey,
        MaterialPlugin,
    },
    prelude::*,
    render::{
        render_resource::{
            AsBindGroup, CompareFunction, DepthStencilState, Face, RenderPipelineDescriptor,
            SpecializedMeshPipelineError,
        },
        renderer::RenderAdapterInfo,
        view::{DebandDither, Tonemapping},
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
    audit_cell_coverage_on_map, audit_terrain_mesh_with_images, build_terrain_mesh,
    build_terrain_mesh_with_images, build_terrain_mesh_with_samples,
};
pub use profile::{
    COMPACT_BUILDING_HEIGHT, CellShape, GROUND_HEIGHT, LARGE_BUILDING_HEIGHT, MAX_PROFILE_HEIGHT,
    MIN_PROFILE_HEIGHT, SOURCE_TILE_HEIGHT, SolidKind, WATER_HEIGHT, shape_for_source,
    support_height, supports_frame_profile,
};

/// Exact resolved actor-source coverage for diagnostics. This does not accept
/// script tokens, infer identities from an object slot, or guess species.
pub fn authored_actor_source(source: &str) -> Option<&'static str> {
    new_bark_actors::authored_source(source)
}

/// Exact normal-palette battle mesh availability, excluding generic icons.
pub fn has_authored_battle_species(species: &str) -> bool {
    battle_species_models::supported_species().any(|id| id.eq_ignore_ascii_case(species))
        || new_bark_actors::actor_props::battle_species_kind(species).is_some()
}

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
    Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(0x7a88_22d4_d773_4874_94d3_c20a_0dcc_f21a), std::marker::PhantomData);
#[derive(Clone, Copy, Debug, Default)]
pub struct VoxelViewPlugin;

impl Plugin for VoxelViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BattleViewPlugin);
        load_internal_asset!(
            app,
            VOXEL_SURFACE_SHADER_HANDLE,
            "voxel_surface.wesl",
            Shader::from_wesl
        );
        app.add_plugins(MaterialPlugin::<VoxelMaterial>::default());
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Update, native_model_camera.before(sync_voxel_view));
        load_internal_asset!(
            app,
            SILHOUETTE_SHADER_HANDLE,
            "occlusion_silhouette.wesl",
            Shader::from_wesl
        );
        app.add_plugins(MaterialPlugin::<OcclusionSilhouetteMaterial>::default());
        app.init_resource::<new_bark_actors::ModeledActors>()
            .add_systems(
                Update,
                new_bark_actors::sync
                    .after(sync_voxel_view)
                    .in_set(WorldRenderSet::RenderSync),
            )
            .init_resource::<VoxelViewSettings>()
            .init_resource::<VoxelViewStatus>()
            .init_resource::<live_profiles::LiveProfiles>()
            .add_systems(Update, live_profiles::reload.before(sync_voxel_view))
            .init_resource::<ActorScreenHeads>()
            .init_resource::<VoxelScene>()
            .init_resource::<TerrainRevisionCache>()
            .init_resource::<TerrainBuildQueue>()
            .init_resource::<ActorIdCache>()
            .init_resource::<PlayerSilhouetteCache>()
            .add_systems(
                PostUpdate,
                project_actor_heads
                    .after(bevy::transform::TransformSystems::Propagate)
                    .in_set(ActorHeadProjection),
            )
            .add_systems(Startup, setup_voxel_view)
            .add_systems(Update, toggle_voxel_view.before(sync_voxel_view))
            .add_systems(Update, sync_voxel_view.in_set(WorldRenderSet::RenderSync))
            .add_systems(Update, sync_voxel_atmosphere.after(sync_voxel_view))
            .add_systems(Update, sync_cave_water_phases.after(sync_voxel_view).in_set(WorldRenderSet::RenderSync))
            .add_systems(
                Update,
                sync_interior_cutaway
                    .after(sync_voxel_view)
                    .in_set(WorldRenderSet::RenderSync),
            )
            .add_systems(
                Update,
                sync_player_silhouette_system
                    .after(sync_voxel_view)
                    .in_set(WorldRenderSet::RenderSync),
            );
        occluder_fade::register(app);
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
    status: Res<VoxelViewStatus>,
    cache: Res<ActorIdCache>,
    cameras: Query<(&Camera, &GlobalTransform), With<VoxelWorldCamera>>,
    cards: Query<&GlobalTransform, With<VoxelActorCard>>,
    mut heads: ResMut<ActorScreenHeads>,
) {
    heads.0.clear();
    if !status.active {
        return;
    }
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };
    let (Some(size), Some(rect)) = (camera.logical_target_size(), camera.logical_viewport_rect())
    else {
        return;
    };
    for (id, entity) in &cache.entities {
        if !matches!(id, VisualActorId::Player | VisualActorId::RemotePlayer(_)) {
            continue;
        }
        let Ok(transform) = cards.get(*entity) else {
            continue;
        };
        if let Ok(point) =
            camera.world_to_viewport(camera_transform, transform.transform_point(Vec3::Y))
        {
            let normalized = (point + rect.min) / size;
            if normalized.is_finite()
                && normalized.cmpge(Vec2::ZERO).all()
                && normalized.cmple(Vec2::ONE).all()
            {
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
struct VoxelSun;

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
    source_map_size_core_tiles: Option<UVec2>,
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
            source_map_size_core_tiles: frame.source_map_size_core_tiles,
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
    painted_stone: Option<Handle<Image>>,
    painted_bedrock: Option<Handle<Image>>,
    built_frame: Option<VisualWorldFrame>,
    // Exact profile document consumed by the completed build, never a desired revision.
    built_profiles: Option<std::sync::Arc<live_profiles::Document>>,
    // Original compositor handle at build submission; built_frame owns its atlas copy.
    built_source_texture: Option<Handle<Image>>,
    // Exact current bounds for the two mutable animated material domains.
    built_animated_bounds: [Option<bevy::camera::primitives::Aabb>; 2],
    built_footing_heights: Vec<f32>,
    built_animated_pillars: Vec<[usize; 2]>,
    footing_origin: Option<IVec2>,
    key: Option<TerrainCacheKey>,
    instances_root: Option<Entity>,
    textured_entity: Option<Entity>,
    solid_entity: Option<Entity>,
    animated_textured_entity: Option<Entity>,
    animated_solid_entity: Option<Entity>,
    animated_textured_mesh: Option<Handle<Mesh>>,
    animated_solid_mesh: Option<Handle<Mesh>>,
    textured_material: Option<Handle<VoxelMaterial>>,
    solid_material: Option<Handle<VoxelMaterial>>,
    footing_heights: Vec<f32>,
}

#[derive(Resource, Default)]
struct TerrainBuildQueue {
    started: u64,
    // Let asset removal/render extraction retire the previous map before a
    // synchronous browser build allocates the replacement scene.
    retirement_frames: u8,
    key: Option<TerrainCacheKey>,
    #[cfg(not(target_arch = "wasm32"))]
    task: Option<Task<TerrainBuildResult>>,
    #[cfg(target_arch = "wasm32")]
    completed: Option<TerrainBuildResult>,
}

struct TerrainBuildResult {
    key: TerrainCacheKey,
    frame: VisualWorldFrame,
    source_texture: Handle<Image>,
    profiles: std::sync::Arc<live_profiles::Document>,
    terrain: Result<BuiltTerrain, TerrainMeshError>,
}

struct BuiltTerrain {
    scenery_fades: Vec<(Vec<occluder_fade::PreparedGroup>, std::sync::Arc<crate::scenery_surfaces::SceneryTexture>, terrain_batches::AuthenticDomain)>,
    scenery: Vec<(terrain_batches::PreparedBatch, std::sync::Arc<crate::scenery_surfaces::SceneryTexture>)>,
    background: Option<(Mesh, Handle<Image>)>,
    cave_enclosure: Option<Mesh>,
    instances: Vec<(Mesh, Vec<[f32; 3]>)>,
    reveal_join_batches: Vec<(Mesh, maze_reveal_batches::MazeRevealBatch)>,
    fade_textured_groups: Vec<occluder_fade::PreparedGroup>,
    fade_solid_groups: Vec<occluder_fade::PreparedGroup>,
    footing_heights: Vec<f32>,
    textured_meshes: Vec<terrain_batches::PreparedBatch>,
    solid_meshes: Vec<terrain_batches::PreparedBatch>,
    animated_textured_mesh: Mesh,
    animated_solid_mesh: Mesh,
    animated_pillars: Vec<[usize; 2]>,
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
        _pipeline: &MaterialPipeline,
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
    depth_stencil.depth_compare = Some(CompareFunction::Less);
    depth_stencil.depth_write_enabled = Some(false);
}

#[allow(clippy::too_many_arguments)]
fn setup_voxel_view(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut scene: ResMut<VoxelScene>,
    adapter: Option<Res<RenderAdapterInfo>>,
) {
    #[cfg(not(target_arch = "wasm32"))]
    if std::env::var_os("CRYSTAL_SCENERY_METRICS").is_some() {
        if let Some(adapter) = adapter.as_ref() {
            eprintln!(
                "modeled renderer adapter: name={} device={:?} backend={:?}",
                adapter.name, adapter.device_type, adapter.backend
            );
        }
    }
    // Face colors retain their authored shade; this light supplies only
    // the scene's depth-tested cast-shadow visibility.
    #[cfg(not(target_arch = "wasm32"))]
    let shadow_size = match std::env::var("CRYSTAL_MODELED_SHADOW_RESOLUTION").as_deref() {
        Ok("512") => 512,
        Ok("1024") => 1024,
        _ => 2048,
    };
    #[cfg(target_arch = "wasm32")]
    let shadow_size = 2048;
    commands.insert_resource(DirectionalLightShadowMap { size: shadow_size });
    let initial_viewport = Vec2::new(160.0, 144.0);
    let pose = camera_pose(initial_viewport);
    let camera = commands
        .spawn((

// Full-map terrain extends beyond the LCD crop. Clear the
// finite horizon to sky so the faithful 2D viewport can
// never leak through as a vertical backdrop behind it.
(Camera3d::default(), Msaa::Off,
Camera {
                    order: 0,
                    is_active: false,




                    clear_color: ClearColorConfig::Custom(Color::srgb(0.72, 0.83, 0.78)),
                    ..default()
                },
Projection::Orthographic(OrthographicProjection {
                    near: 0.1,
                    far: 4096.0,
                    scaling_mode: ScalingMode::Fixed {
                        width: initial_viewport.x,
                        height: initial_viewport.y,
                    },
                    ..OrthographicProjection::default_3d()
                }),
pose.transform(),
Tonemapping::None,
DebandDither::Disabled,),
            RenderLayers::layer(VOXEL_RENDER_LAYER),
            DistanceFog {
                color: Color::NONE,
                directional_light_color: Color::NONE,
                ..default()
            },
            // Smooth modeled silhouette edges without changing the faithful
            // 2D camera or its nearest-neighbor sprite textures.
            Fxaa {
                enabled: false,
                ..default()
            },
            VoxelWorldCamera,
        ))
        .id();
    commands.spawn((

// A southeast light gives the small world readable northwest cast
// shadows while keeping the source sprites' front faces bright.
// Voxel world coordinates are +X east and +Z south. Put the sun
// in that actual quadrant so the camera-facing facade receives
// direct light, matching the reference renderer's south/east
// face model. The previous negative X/Z position lit every
// house from behind and made its mapped windows and door nearly
// black even though their source pixels were correct.
(DirectionalLight {
                color: Color::WHITE,
                illuminance: 3_000.0,
                shadow_maps_enabled: true,
                shadow_depth_bias: 0.1,
                shadow_normal_bias: 2.5,
                ..default()
            },
CascadeShadowConfigBuilder {
                num_cascades: 1,
                maximum_distance: 4096.0,
                first_cascade_far_bound: 4096.0,
                ..default()
            }
            .build(),
Transform::from_xyz(180.0, 320.0, 220.0).looking_at(Vec3::ZERO, Vec3::Y),),
        RenderLayers::layer(VOXEL_RENDER_LAYER),
        VoxelSun,
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
            &mut Mesh3d,
            &mut MeshMaterial3d<VoxelMaterial>,
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
    status.profiles_pending = terrain_cache
        .key
        .as_ref()
        .is_none_or(|key| key.profiles_revision != profiles.revision);
    if status.profiles_pending {
        status.active_frames = 0;
    }
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
        Projection::Orthographic(_) | Projection::Custom(_) => true,
    }
}

// Phase changes update the retained atlas without invalidating async geometry.
fn sync_cave_water_phases(
    frame: Res<VisualWorldFrame>,
    phases: Option<Res<crystal_render_api::VisualTileAnimationFrame>>,
    status: Res<VoxelViewStatus>,
    mut cache: ResMut<TerrainRevisionCache>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
) {
    if !status.active || !frame.active {
        return;
    }
    let Some(phases) = phases else { return; };
    let material_handle = cache.textured_material.clone();
    let mask = material_handle.as_ref()
        .and_then(|handle| materials.get(handle))
        .and_then(|material| material.extension.water_mask.clone());
    let Some(built) = cache.built_frame.as_mut() else { return; };
    if phases.map_id != frame.map_id {
        return;
    }
    cave_water::refresh_phases(built, &phases, &mut images);
    if let Some(mask) = mask {
        if cave_water::refresh_mask_phases(built, &mask, &mut images) {
            // Re-uploading an Image replaces its GPU texture. Refresh every
            // material that binds it, including cloned cave occluder groups.
            let users: Vec<_> = materials.iter().filter_map(|(id, material)| {
                (material.extension.water_mask.as_ref() == Some(&mask)).then_some(id)
            }).collect();
            for id in users {
                let _ = materials.get_mut(id);
            }
        }
    }
    let registration = cave_water::retained_registration(built, &frame);
    if let Some(handle) = material_handle {
        if materials.get(&handle).is_some_and(|material|
            material.extension.water_enabled.x > 0.5 && material.extension.water_enabled != registration)
        {
            materials.get_mut(&handle).unwrap().extension.water_enabled = registration;
        }
    }
}

fn sync_voxel_atmosphere(
    frame: Res<VisualWorldFrame>,
    profiles: Res<live_profiles::LiveProfiles>,
    settings: Res<VoxelViewSettings>,
    adapter: Option<Res<RenderAdapterInfo>>,
    mut lights: Query<(&mut DirectionalLight, &mut bevy::light::CascadeShadowConfig), With<VoxelSun>>,
    mut cameras: Query<(&mut DistanceFog, &mut Fxaa), With<VoxelWorldCamera>>,
    mut ambient: Option<ResMut<GlobalAmbientLight>>,
) {
    if !frame.is_changed() && !profiles.is_changed() && !settings.is_changed() {
        return;
    }
    let atmosphere = profiles.document.atmosphere.as_ref().filter(|atmosphere| {
        frame.active
            && settings.enabled
            && atmosphere
                .maps
                .iter()
                .any(|map| map == frame.map_id.as_ref())
    });
    let cave = interior_cutaway::is_cave(&frame);
    // Caverns reduce actual light energy uniformly. Darkness is not a fog
    // overlay or a player-centered mask; explicit authored fog still applies.
    let atmosphere = atmosphere.map(|a| (a.start_tiles, a.end_tiles, a.opacity));
    let cavern_lit = cave && frame.active && settings.enabled;
    let dark = cavern_lit && frame.tiles.first().is_some_and(|tile| tile.source.tileset_id.as_ref() == "dark_cave");
    let (ambient_gain, sun_gain) = if dark { (0.36, 0.28) }
        else if cavern_lit { (0.60, 0.48) } else { (1.0, 1.0) };
    if let Some(ambient) = ambient.as_mut() {
        ambient.brightness = GlobalAmbientLight::default().brightness * ambient_gain;
    }
    // CPU rasterizers pay for the entire broad terrain halo again in the
    // shadow pass. Authored contact shading already grounds these scenes;
    // retain dynamic shadows on hardware and use that baked support on known
    // software drivers. This changes rendering only, never world state.
    let software = adapter
        .as_ref()
        .is_some_and(|adapter| software_renderer(&adapter.name));
    let mut shadows = !new_bark_models::supports_map(&frame.map_id) || !software;
    #[cfg(not(target_arch = "wasm32"))]
    {
        match std::env::var("CRYSTAL_MODELED_SHADOWS").as_deref() {
            Ok("on") => shadows = true,
            Ok("off") => shadows = false,
            _ => {}
        }
    }
    for (mut light, mut cascade) in &mut lights {
        light.illuminance = 3_000.0 * sun_gain;
        light.shadow_maps_enabled = shadows;
        if matches!(frame.map_id.as_ref(), "Route29" | "Route26" | "NewBarkTown") {
            // Resolve leaves near the player rather than spending a single
            // shadow map on the camera's entire distant terrain halo.
            *cascade = CascadeShadowConfigBuilder {
                // WebGL supports one cascade; tighten that one around the
                // visible scene instead of requesting unsupported cascades.
                num_cascades: if cfg!(target_arch = "wasm32") { 1 } else { 3 },
                maximum_distance: if cfg!(target_arch = "wasm32") { 1024.0 } else { 2048.0 },
                first_cascade_far_bound: 256.0,
                ..default()
            }.build();
            light.shadow_normal_bias = 2.5;
        } else {
            *cascade = CascadeShadowConfigBuilder {
                num_cascades: 1,
                maximum_distance: 4096.0,
                first_cascade_far_bound: 4096.0,
                ..default()
            }.build();
            light.shadow_normal_bias = 2.5;
        }
    }
    for (mut fog, mut fxaa) in &mut cameras {
        fxaa.enabled = frame.active
            && settings.enabled
            && (cave || new_bark_models::supports_map(&frame.map_id));
        if let Some((start_tiles, end_tiles, opacity)) = atmosphere {
            let pose = settings.camera.pose(frame.viewport_size);
            let target_distance = pose.eye.distance(pose.target);
            fog.color = voxel_clear_color(&frame).with_alpha(opacity);
            fog.falloff = FogFalloff::Linear {
                start: target_distance + start_tiles * frame.tile_size.y,
                end: target_distance + end_tiles * frame.tile_size.y,
            };
        } else {
            fog.color = Color::NONE;
        }
    }
}

fn software_renderer(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    ["llvmpipe", "softpipe", "swiftshader", "software rasterizer"]
        .iter()
        .any(|label| name.contains(label))
}

#[cfg(test)]
#[test]
fn modeled_software_shadow_profile_does_not_match_hardware_adapters() {
    assert!(software_renderer("llvmpipe (LLVM 19, 256 bits)"));
    assert!(software_renderer("Google SwiftShader"));
    assert!(!software_renderer("Apple M3"));
    assert!(!software_renderer("NVIDIA RTX 4080"));
    assert!(!software_renderer("Intel Iris Xe"));
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
    } else if tileset == Some("ice_path") && frame.map_id.as_ref() != "HallOfFame" {
        Color::srgb(0.025, 0.045, 0.075)
    } else if interior_cutaway::is_cave(frame) {
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
        let retired = std::mem::take(cache);
        cache.painted_stone = retired.painted_stone;
        cache.painted_bedrock = retired.painted_bedrock;
        for entity in [
            retired.instances_root,
            retired.textured_entity,
            retired.solid_entity,
            retired.animated_textured_entity,
            retired.animated_solid_entity,
        ]
        .into_iter()
        .flatten()
        {
            commands.entity(entity).despawn();
        }
        builds.retirement_frames = 2;
    }
    if builds.retirement_frames > 0 {
        builds.retirement_frames -= 1;
        return Ok(TerrainSyncState::Pending);
    }
    if cache.key.as_ref() != Some(&next_key)
        && cache
            .key
            .as_ref()
            .is_some_and(|key| key.profiles_revision == profiles.revision)
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
        let source_texture = frame.map_texture.clone();
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
            let terrain = mesh::build_instanced_terrain_mesh_with_profiles(
                &build_frame,
                &samples,
                &profile_document,
            )
            .map(|mut terrain| {
                // The public mesh stays complete until this explicit runtime
                // partition. Both pieces are uploaded only with the revision.
                // Whole-object translucency can expose every original join.
                // Keep these triangles in their owning wall rather than applying
                // the old capsule's camera-dependent hidden-face selection.
                terrain.reveal_join_batches.clear();
                let reveal_join_batches = Vec::new();
                let animated_textured_mesh =
                    std::mem::take(&mut terrain.animated_textured).into_mesh();
                let animated_solid_mesh = std::mem::take(&mut terrain.animated_solid).into_mesh();
                let animated_pillars = std::mem::take(&mut terrain.animated_pillars);
                let enclosure = std::mem::take(&mut terrain.cave_enclosure);
                let cave_enclosure = (!enclosure.indices.is_empty()).then(|| enclosure.into_mesh());
                let background = terrain
                    .background
                    .take()
                    .map(|b| (b.mesh.into_mesh(), b.texture));
                let instances = std::mem::take(&mut terrain.tree_instances)
                    .into_iter()
                    .map(|group| (group.mesh.into_mesh(), group.origins))
                    .collect();
                let footing_heights = terrain.footing_heights.clone();
                let (textured, solid, fade_textured_groups, fade_solid_groups) =
                    occluder_fade::prepare_linked(terrain.textured, terrain.solid, terrain.cutaway_links);
                let source_bounds = authentic_mesh_bounds(&build_frame);
                let mut scenery = Vec::new();
                let mut scenery_fades = Vec::new();
                for part in std::mem::take(&mut terrain.scenery) {
                    for (domain, data) in terrain_batches::separate_authentic_domains(part.mesh, source_bounds) {
                        let (opaque, groups) = occluder_fade::prepare(data);
                        if !groups.is_empty() { scenery_fades.push((groups, part.texture.clone(), domain)); }
                        if !opaque.indices.is_empty() {
                            scenery.extend(terrain_batches::prepare_with_authentic_bounds(opaque, build_frame.tile_size, source_bounds)
                                .into_iter().map(|batch| (batch, part.texture.clone())));
                        }
                    }
                }
                let textured_meshes = terrain_batches::prepare_with_authentic_bounds(
                    textured,
                    build_frame.tile_size,
                    source_bounds,
                );
                let solid_meshes = terrain_batches::prepare_with_authentic_bounds(
                    solid,
                    build_frame.tile_size,
                    source_bounds,
                );
                BuiltTerrain {
                    scenery,
                    scenery_fades,
                    background,
                    cave_enclosure,
                    instances,
                    reveal_join_batches,
                    fade_textured_groups,
                    fade_solid_groups,
                    footing_heights,
                    textured_meshes,
                    solid_meshes,
                    animated_textured_mesh,
                    animated_solid_mesh,
                    animated_pillars,
                }
            });
            TerrainBuildResult {
                key: build_key,
                frame: build_frame,
                source_texture,
                profiles: profile_document,
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
        if completed.key == next_key
            || (completed.key.profiles_revision == profiles.revision
                && terrain_tracking::can_reuse(&completed.frame, frame))
        {
            let terrain = completed.terrain.map_err(TerrainSyncError::Mesh)?;
            let profile_changed = cache
                .key
                .as_ref()
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
            cache.built_source_texture = Some(completed.source_texture);
            cache.built_profiles = Some(completed.profiles);
            if profile_changed {
                println!(
                    "geometry profile mesh applied: revision {}",
                    next_key.profiles_revision
                );
            }
        }
    }

    let ready = cache.key.as_ref() == Some(&next_key);
    if ready {
        if let Some(built) = &mut cache.built_frame {
            if built.terrain_revision != frame.terrain_revision {
                let geometry_changed = terrain_tracking::refresh_animation(built, frame, images)
                    .map_err(|()| TerrainSyncError::SourceTextureUnavailable)?;
                if geometry_changed {
                    let (textured, solid) = mesh::build_animated_flowers(built, images, &cache.built_animated_pillars)
                        .map_err(TerrainSyncError::Mesh)?;
                    cache.built_animated_bounds = [textured.get_aabb(), solid.get_aabb()];
                    update_mesh_asset(meshes, &mut cache.animated_textured_mesh, retain_animated_mesh_cpu(textured));
                    update_mesh_asset(meshes, &mut cache.animated_solid_mesh, retain_animated_mesh_cpu(solid));
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
            } else {
                // A completed asynchronous build queued these roots earlier in
                // this system. Set their live scroll offset before Commands
                // materializes them, rather than drawing the built pose once.
                commands.entity(entity).insert((Visibility::Visible, live_transform));
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
    cache.built_profiles = None;
    cache.built_source_texture = Some(frame.map_texture.clone());
    cache.built_animated_bounds = [terrain.animated_textured_mesh.get_aabb(),
        terrain.animated_solid_mesh.get_aabb()];
    cache.built_footing_heights = terrain.footing_heights;
    cache.built_animated_pillars = terrain.animated_pillars;

    let animated_textured = update_mesh_asset(
        meshes,
        &mut cache.animated_textured_mesh,
        retain_animated_mesh_cpu(terrain.animated_textured_mesh),
    );
    let animated_solid = update_mesh_asset(
        meshes,
        &mut cache.animated_solid_mesh,
        retain_animated_mesh_cpu(terrain.animated_solid_mesh),
    );
    let textured_material_handle = if let Some(handle) = cache.textured_material.as_ref() {
        sync_terrain_texture(materials, handle, &frame.map_texture)?;
        handle.clone()
    } else {
        let handle = materials.add(textured_terrain_material(frame.map_texture.clone()));
        cache.textured_material = Some(handle.clone());
        handle
    };
    let water_mask = cave_water::mask(frame).map(|mask| images.add(mask));
    if let Some(mut material) = materials.get_mut(&textured_material_handle) {
        material.extension.water_enabled = if water_mask.is_some() {
            cave_water::registration(frame)
        } else {
            Vec4::ZERO
        };
        material.extension.water_mask = water_mask;
    }
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

    let painted_stone = cache.painted_stone.get_or_insert_with(|| images.add(painted_stone::image())).clone();
    let painted_bedrock = cache.painted_bedrock.get_or_insert_with(|| images.add(painted_stone::bedrock_image())).clone();
    if let Some(mut material) = materials.get_mut(&solid_material_handle) {
        material.extension.painted_stone = Some(painted_stone);
        material.extension.painted_bedrock = Some(painted_bedrock);
    }
    let cave_reveal = if interior_cutaway::is_cave(frame) {
        let dark = frame.tiles.first().is_some_and(|tile| tile.source.tileset_id.as_ref() == "dark_cave")
            && std::env::var_os("CRYSTAL_CAVE_GEOMETRY_REVIEW").is_none();
        Vec4::new(1.0, if dark { 1.0 } else { 0.0 }, 0.0, 0.0)
    } else if matches!(frame.map_id.as_ref(), "NewBarkTown" | "Route29" | "Route26") {
        // The existing environment vector's Z channel selects exterior
        // daylight; X/Y retain cave reveal and darkness ownership.
        Vec4::Z
    } else { Vec4::ZERO };
    for handle in [&textured_material_handle, &solid_material_handle] {
        if let Some(mut material) = materials.get_mut(handle) {
            material.extension.cave_reveal = cave_reveal;
        }
    }

    if let Some(root) = cache.instances_root.take() {
        commands.entity(root).despawn();
    }
    let root = commands
        .spawn((
            (Visibility::default(),
terrain_transform(frame),),
            VoxelTerrain,
        ))
        .id();
    if let Some(mesh) = terrain.cave_enclosure {
        let mut material = solid_terrain_material();
        // Outer rock keeps its mass. Only the broad player aperture reveals
        // the room through it; whole-front-wall fading belongs to inner walls.
        material.extension.cave_reveal = Vec4::new(1., cave_reveal.y, 0., 1.);
        material.extension.painted_bedrock = cache.painted_bedrock.clone();
        let child = commands.spawn(((Mesh3d(meshes.add(mesh)),
MeshMaterial3d(materials.add(material)),), RenderLayers::layer(VOXEL_RENDER_LAYER), encounter_terrain::SyntheticTerrainApron)).id();
        commands.entity(root).add_child(child);
    }
    if let Some((mesh, source)) = terrain.background {
        use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
        let mut image = images
            .get(&source)
            .ok_or(TerrainSyncError::SourceTextureUnavailable)?
            .clone();
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            ..ImageSamplerDescriptor::nearest()
        });
        image.asset_usage = bevy::asset::RenderAssetUsages::RENDER_WORLD;
        let mut material = textured_terrain_material(images.add(image));
        material.extension.cave_reveal = cave_reveal;
        let material = materials.add(material);
        let child = commands
            .spawn((
                (Mesh3d(meshes.add(mesh)),
MeshMaterial3d(material),),
                RenderLayers::layer(VOXEL_RENDER_LAYER),
                encounter_terrain::SyntheticTerrainApron,
            ))
            .id();
        commands.entity(root).add_child(child);
    }
    for (mesh, batch) in terrain.reveal_join_batches {
        let child = commands.spawn((

// Absolute built-grid coordinates are unchanged. Identity
// local transform inherits the same retained scrolling root.
(Mesh3d(meshes.add(mesh)),
MeshMaterial3d(solid_material_handle.clone()),
Visibility::Inherited,),
            RenderLayers::layer(VOXEL_RENDER_LAYER),
            batch,
        )).id();
        commands.entity(root).add_child(child);
    }
    let mut scenery_materials = HashMap::new();
    for (batch, texture) in terrain.scenery {
        let key = std::sync::Arc::as_ptr(&texture) as usize;
        let handle = scenery_materials.entry(key).or_insert_with(|| {
            let mut material = textured_terrain_material(images.add(texture.image()));
            material.extension.cave_reveal = cave_reveal;
            material.base.alpha_mode = texture.alpha_mode();
            material.base.double_sided = texture.double_sided;
            if texture.double_sided { material.base.cull_mode = None; }
            materials.add(material)
        }).clone();
        let mut child = commands.spawn(((Mesh3d(meshes.add(batch.mesh)),
MeshMaterial3d(handle),), RenderLayers::layer(VOXEL_RENDER_LAYER), batch.authentic_domain));
        if let Some(bounds) = batch.bounds { child.insert(bounds); }
        let child=child.id();
        commands.entity(root).add_child(child);
    }
    for (groups, texture, domain) in terrain.scenery_fades {
        let key = std::sync::Arc::as_ptr(&texture) as usize;
        let handle = scenery_materials.entry(key).or_insert_with(|| {
            let mut material = textured_terrain_material(images.add(texture.image()));
            material.extension.cave_reveal = cave_reveal;
            material.base.alpha_mode = texture.alpha_mode();
            material.base.double_sided = texture.double_sided;
            if texture.double_sided { material.base.cull_mode = None; }
            materials.add(material)
        }).clone();
        let leaves = occluder_fade::spawn_groups(commands, meshes, materials, root, groups, handle, None);
        for leaf in leaves { commands.entity(leaf).insert(domain); }
    }
    for (mesh, origins) in terrain.instances {
        let mesh = meshes.add(mesh);
        for origin in origins {
            let child = commands
                .spawn((
                    (Mesh3d(mesh.clone()),
MeshMaterial3d(textured_material_handle.clone()),
Transform::from_translation(Vec3::from_array(origin)),),
                    RenderLayers::layer(VOXEL_RENDER_LAYER),
                ))
                .id();
            commands.entity(root).add_child(child);
        }
    }
    occluder_fade::spawn_groups(commands, meshes, materials, root,
        terrain.fade_textured_groups, textured_material_handle.clone(), None);
    occluder_fade::spawn_groups(commands, meshes, materials, root,
        terrain.fade_solid_groups, solid_material_handle.clone(), Some(textured_material_handle.clone()));
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
    replace_static_terrain_batches(
        commands,
        meshes,
        &mut cache.textured_entity,
        terrain.textured_meshes,
        textured_material_handle,
        frame,
    );
    replace_static_terrain_batches(
        commands,
        meshes,
        &mut cache.solid_entity,
        terrain.solid_meshes,
        solid_material_handle,
        frame,
    );

    cache.key = Some(key);
    Ok(())
}

/// Static roots retain the same scrolling/visibility contract as the previous
/// combined meshes. Only their children draw. Replacing the complete hierarchy
/// also replaces every AABB, so no bound from an earlier map/revision survives.
fn replace_static_terrain_batches(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    cached_root: &mut Option<Entity>,
    batches: Vec<terrain_batches::PreparedBatch>,
    material: Handle<VoxelMaterial>,
    frame: &VisualWorldFrame,
) {
    if let Some(root) = cached_root.take() {
        commands.entity(root).despawn();
    }
    let root = commands
        .spawn((
            (Visibility::default(),
terrain_transform(frame),),
            VoxelTerrain,
        ))
        .id();
    for terrain_batches::PreparedBatch {
        mesh,
        bounds,
        authentic_domain,
    } in batches {
        let mut child = commands.spawn((
            (Mesh3d(meshes.add(mesh)),
MeshMaterial3d(material.clone()),
Visibility::Inherited,),
            RenderLayers::layer(VOXEL_RENDER_LAYER),
            authentic_domain,
        ));
        if let Some(bounds) = bounds {
            child.insert(bounds);
        }
        let child = child.id();
        commands.entity(root).add_child(child);
    }
    *cached_root = Some(root);
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

// Only the two mutable flower domains retain CPU data. A checked encounter
// takes one exact private copy; all ordinary static terrain stays render-only.
fn retain_animated_mesh_cpu(mut mesh: Mesh) -> Mesh {
    mesh.asset_usage = bevy::asset::RenderAssetUsages::default();
    mesh
}

fn update_mesh_asset(
    meshes: &mut Assets<Mesh>,
    cache: &mut Option<Handle<Mesh>>,
    mesh: Mesh,
) -> Handle<Mesh> {
    if let Some(handle) = cache.as_ref() {
        // Render-world-only extraction takes the CPU asset. Keep the strong
        // handle stable and insert the next complete revision under its ID.
        meshes.insert(handle.id(), mesh).expect("retained mesh handle remains live");
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
            (Mesh3d(mesh),
MeshMaterial3d(material),
terrain_transform(frame),),
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
    Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(0xf1d7_662b_5b61_49b2_a221_6f89195c8542), std::marker::PhantomData);

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
struct VoxelSurface {
    #[uniform(100)]
    cutaway: interior_cutaway::CutawayUniform,
    #[texture(101)]
    #[sampler(102)]
    water_mask: Option<Handle<Image>>,
    #[uniform(103)]
    water_enabled: Vec4,
    #[uniform(104)]
    cave_reveal: Vec4,
    #[texture(105)]
    #[sampler(106)]
    painted_stone: Option<Handle<Image>>,
    #[texture(107)]
    #[sampler(108)]
    painted_bedrock: Option<Handle<Image>>,
}
impl MaterialExtension for VoxelSurface {
    fn fragment_shader() -> ShaderRef {
        VOXEL_SURFACE_SHADER_HANDLE.into()
    }
}

fn voxel_material(base: StandardMaterial) -> VoxelMaterial {
    ExtendedMaterial {
        base,
        extension: VoxelSurface::default(),
    }
}

/// Publish the resolved player support/capsule to the whole-object fade system.
/// It uses the current propagated camera and cached authored triangles.
fn sync_interior_cutaway(
    frame: Res<VisualWorldFrame>,
    status: Res<VoxelViewStatus>,
    cache: Res<TerrainRevisionCache>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
) {
    let Some(handle) = cache.solid_material.as_ref() else {
        return;
    };
    let next = if status.active
        && interior_cutaway::diagnostic_mode() != interior_cutaway::DiagnosticMode::ZeroRadius
    {
        frame
            .actors
            .iter()
            .find(|actor| actor.id == VisualActorId::Player)
            .and_then(|actor| {
                let foot = actor_foot(actor);
                let height = resolved_footing_height(&frame, foot, &cache.footing_heights)?;
                interior_cutaway::CutawayUniform::for_player(
                    visual_point_to_voxel(foot, height + 0.04),
                    frame.tile_size.y * 2.0,
                )
            })
            .unwrap_or_default()
    } else {
        interior_cutaway::CutawayUniform::default()
    };
    // Terrain groups clone the base material. Only cave terrain clones whose
    // capsule moved need an update; actor materials keep reveal disabled.
    let changed: Vec<_> = materials.iter().filter_map(|(id, material)| {
        (material.extension.cave_reveal.x > 0.5
            && (material.extension.cutaway.bottom_radius != next.bottom_radius
                || material.extension.cutaway.top_feather != next.top_feather)).then_some(id)
    }).collect();
    for id in changed {
        if let Some(mut material) = materials.get_mut(id) {
            material.extension.cutaway.bottom_radius = next.bottom_radius;
            material.extension.cutaway.top_feather = next.top_feather;
        }
    }
    // Avoid marking a material changed while the published player is idle.
    if materials
        .get(handle)
        .is_some_and(|material| material.extension.cutaway != next)
    {
        if let Some(mut material) = materials.get_mut(handle) {
            material.extension.cutaway = next;
        }
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
            &mut Mesh3d,
            &mut MeshMaterial3d<VoxelMaterial>,
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
            if current_mesh.0 != mesh {
                current_mesh.0 = mesh;
            }
            if current_material.0 != material {
                current_material.0 = material;
            }
            continue;
        }

        let entity = commands
            .spawn((
                (Mesh3d(mesh),
MeshMaterial3d(material),
transform,),
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
            &mut MeshMaterial3d<OcclusionSilhouetteMaterial>,
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
    // A card silhouette would be occluded by the player's own 3D body and
    // incorrectly paint the old pixel sprite across its front.
    if !status.active || new_bark_actors::has_modeled_player(&frame) {
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
            &mut MeshMaterial3d<OcclusionSilhouetteMaterial>,
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
        if current_material.0 != material {
            current_material.0 = material;
        }
        return;
    }
    cache.entity = Some(
        commands
            .spawn((
                (Mesh3d(actor_quad.clone()),
MeshMaterial3d(material),
transform,),
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
    let rotation = if matches!(
        actor.id,
        VisualActorId::Object(_) | VisualActorId::Player | VisualActorId::RemotePlayer(_)
    ) {
        // Directional character artwork supplies the visible side. Keep the card
        // vertical and its bottom pivot on the footing, with no camera pull.
        let toward_eye = pose.eye - pose.target;
        Quat::from_rotation_y(toward_eye.x.atan2(toward_eye.z))
    } else {
        let profile_scale = frame.tile_size.y / SOURCE_TILE_HEIGHT;
        let camera_pull =
            actor_camera_pull(actor, CAMERA_PITCH_DEGREES.to_radians()) * profile_scale;
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
        if matches!(
            actor.id,
            VisualActorId::Object(_) | VisualActorId::Player | VisualActorId::RemotePlayer(_)
        ) {
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

/// The intersection of actual source acreage and the built grid, in the mesh's
/// local X/Z coordinates. Screen center/camera movement cannot change it.
fn authentic_mesh_bounds(frame: &VisualWorldFrame) -> Option<terrain_batches::AuthenticBounds> {
    let source = frame.source_map_size_core_tiles?;
    if source.min_element() == 0
        || !frame.tile_size.is_finite()
        || frame.tile_size.min_element() <= 0.0
    {
        return None;
    }
    let start = frame.grid_origin.as_vec2();
    let size = frame.grid_size.as_vec2();
    let lo = start.max(Vec2::ZERO);
    let hi = (start + size).min(source.as_vec2() * 2.0);
    if hi.cmple(lo).any() {
        return None;
    }
    let origin = -size * frame.tile_size * 0.5;
    let min = origin + (lo - start) * frame.tile_size;
    let max = origin + (hi - start) * frame.tile_size;
    (min.is_finite() && max.is_finite())
        .then_some(terrain_batches::AuthenticBounds { min, max })
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
        mesh::Indices, render_resource::PrimitiveTopology,
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
    fn cavern_lights_dim_real_energy_and_restore_when_leaving() {
        let mut app = App::new();
        app.init_resource::<VisualWorldFrame>()
            .init_resource::<live_profiles::LiveProfiles>()
            .init_resource::<VoxelViewSettings>()
            .init_resource::<GlobalAmbientLight>()
            .add_systems(Update, sync_voxel_atmosphere);
        app.world_mut().resource_mut::<VoxelViewSettings>().enabled = true;
        let sun = app.world_mut().spawn((DirectionalLight::default(),
            bevy::light::CascadeShadowConfig::default(), VoxelSun)).id();
        for (map, tileset, ambient, direct) in [
            ("UnionCave1F", "cave", 48.0, 1440.0),
            ("IcePath1F", "ice_path", 48.0, 1440.0),
            ("DragonsDenB1F", "johto", 48.0, 1440.0),
            ("DarkCaveVioletEntrance", "dark_cave", 28.8, 840.0),
            ("Route29", "johto", 80.0, 3000.0),
        ] {
            let mut frame = app.world_mut().resource_mut::<VisualWorldFrame>();
            frame.active = true;
            frame.map_id = map.into();
            frame.tiles = vec![crystal_render_api::VisualTile { column: 0, row: 0, priority: false,
                animation_frames: None, texture: Default::default(),
                source: crystal_render_api::VisualTileSource { tileset_id: tileset.into(), metatile_id: 0x02,
                    tile_index: 0x16, subtile_column: 0, subtile_row: 0 } }];
            drop(frame);
            app.update();
            assert!((app.world().resource::<GlobalAmbientLight>().brightness - ambient).abs() < 0.001);
            assert!((app.world().get::<DirectionalLight>(sun).unwrap().illuminance - direct).abs() < 0.001);
        }
    }

    #[test]
    fn grove_shadow_detail_resets_when_leaving_the_daylight_maps() {
        let mut app = App::new();
        app.init_resource::<VisualWorldFrame>()
            .init_resource::<live_profiles::LiveProfiles>()
            .init_resource::<VoxelViewSettings>()
            .add_systems(Update, sync_voxel_atmosphere);
        let sun = app.world_mut().spawn((DirectionalLight::default(),
            bevy::light::CascadeShadowConfig::default(), VoxelSun)).id();
        for map in ["Route29", "Route26", "NewBarkTown", "GoldenrodCity", "UnionCave1F", "Route29"] {
            app.world_mut().resource_mut::<VisualWorldFrame>().map_id = map.into();
            app.update();
            let world = app.world();
            let cascade = world.get::<bevy::light::CascadeShadowConfig>(sun).unwrap();
            let light = world.get::<DirectionalLight>(sun).unwrap();
            let grove = matches!(map, "Route29" | "Route26" | "NewBarkTown");
            assert_eq!(cascade.bounds.len(), if grove && !cfg!(target_arch = "wasm32") {3} else {1});
            let far = if grove {
                if cfg!(target_arch = "wasm32") {1024.} else {2048.}
            } else {4096.};
            assert!((*cascade.bounds.last().unwrap() - far).abs() < 0.001);
            assert_eq!(light.shadow_normal_bias, 2.5);
        }
    }

    #[test]
    fn authentic_source_bounds_use_core_tiles_and_ignore_camera_center() {
        let mut frame = VisualWorldFrame {
            source_map_size_core_tiles: Some(UVec2::new(4, 3)),
            grid_origin: IVec2::new(-2, -3),
            grid_size: UVec2::splat(10),
            tile_size: Vec2::splat(32.0),
            ..default()
        };
        let bounds = authentic_mesh_bounds(&frame).unwrap();
        assert_eq!(bounds.min, Vec2::new(-96.0, -64.0));
        assert_eq!(bounds.max, Vec2::new(160.0, 128.0));
        frame.center = Vec2::new(800.0, -1400.0);
        let moved = authentic_mesh_bounds(&frame).unwrap();
        assert_eq!(moved.min, bounds.min);
        assert_eq!(moved.max, bounds.max);
        let key = TerrainCacheKey::from_frame(&frame);
        frame.source_map_size_core_tiles = Some(UVec2::new(4, 4));
        assert_ne!(TerrainCacheKey::from_frame(&frame), key);
        frame.grid_origin = IVec2::splat(100);
        assert!(authentic_mesh_bounds(&frame).is_none());
        frame.grid_origin = IVec2::ZERO;
        frame.source_map_size_core_tiles = None;
        assert!(authentic_mesh_bounds(&frame).is_none());
    }

    #[test]
    fn tree_instances_share_meshes_and_replacement_removes_the_old_hierarchy() {
        let mut world = World::new();
        let mut images = Assets::<Image>::default();
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<VoxelMaterial>::default();
        let mut cache = TerrainRevisionCache::default();
        let actor_material = materials.add(solid_terrain_material());
        let mut previous = Vec::new();
        for (map, tileset, reveal) in [("DarkCaveVioletEntrance", "dark_cave", true),
            ("UnionCave1F", "cave", true),
            ("IcePath1F", "ice_path", true), ("HallOfFame", "ice_path", false),
            ("NewBarkTown", "johto", false), ("Route29", "johto", false), ("Route26", "johto", false),
            ("GoldenrodCity", "johto", false)] {
            let frame = VisualWorldFrame {
                map_id: map.into(),
                tiles: vec![crystal_render_api::VisualTile {
                    column: 0, row: 0, texture: Handle::default(), animation_frames: None,
                    priority: false, source: crystal_render_api::VisualTileSource {
                        tileset_id: tileset.into(), metatile_id: 0,
                        subtile_column: 0, subtile_row: 0, tile_index: 0,
                    },
                }], ..default()
            };
            let mut queue = bevy::ecs::world::CommandQueue::default();
            let mut commands = Commands::new(&mut queue, &world);
            let terrain = BuiltTerrain {
                scenery: Vec::new(),
                scenery_fades: Vec::new(),
                background: None,
                cave_enclosure: None,
                instances: vec![(actor_quad_mesh(), vec![[0.0, 0.0, 0.0], [32.0, 4.0, 16.0]])],
                reveal_join_batches: Vec::new(),
                fade_textured_groups: Vec::new(),
                fade_solid_groups: Vec::new(),
                footing_heights: Vec::new(),
                textured_meshes: vec![actor_quad_mesh().into()],
                solid_meshes: vec![actor_quad_mesh().into()],
                animated_textured_mesh: actor_quad_mesh(),
                animated_solid_mesh: actor_quad_mesh(),
                animated_pillars: Vec::new(),
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
            for handle in [cache.textured_material.as_ref().unwrap(), cache.solid_material.as_ref().unwrap()] {
                let environment = materials.get(handle).unwrap().extension.cave_reveal;
                assert_eq!(environment.x > 0.5, reveal);
                assert_eq!(environment.y > 0.5, tileset == "dark_cave");
                assert_eq!(environment.z > 0.5, matches!(map, "NewBarkTown" | "Route29" | "Route26"));
            }
            assert_eq!(materials.get(&actor_material).unwrap().extension.cave_reveal, Vec4::ZERO);
            queue.apply(&mut world);
            for old in previous.drain(..) {
                assert!(world.get_entity(old).is_err());
            }
            let root = cache.instances_root.unwrap();
            let children = world.get::<Children>(root).unwrap();
            assert_eq!(children.len(), 2);
            assert_eq!(
                world.get::<Mesh3d>(children[0]),
                world.get::<Mesh3d>(children[1])
            );
            assert_eq!(
                world.get::<Transform>(children[1]).unwrap().translation,
                Vec3::new(32.0, 4.0, 16.0)
            );
            previous.extend(children.iter());
            previous.push(root);
        }
    }

    #[test]
    fn static_batches_replace_bounds_and_inherit_scroll_and_transition_visibility() {
        use bevy::camera::primitives::Aabb;
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::transform::TransformPlugin,
            bevy::camera::visibility::VisibilityPlugin,
        ))
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<bevy::mesh::skinning::SkinnedMeshInverseBindposes>>();
        let material = Handle::<VoxelMaterial>::Uuid(
            bevy::asset::uuid::Uuid::from_u128(314159),
            std::marker::PhantomData,
        );
        let mut root = None;
        let mut old_entities = Vec::new();
        for revision in 0..3 {
            let frame = VisualWorldFrame {
                center: Vec2::new(revision as f32 * 31.0, -17.0),
                ..default()
            };
            let mut first = actor_quad_mesh();
            let shift = revision as f32 * 100.0;
            first.insert_attribute(
                Mesh::ATTRIBUTE_POSITION,
                vec![
                    [shift - 0.5, 0.0, 0.0],
                    [shift + 0.5, 0.0, 0.0],
                    [shift + 0.5, 1.0, 0.0],
                    [shift - 0.5, 1.0, 0.0],
                ],
            );
            let expected_bounds = first.get_aabb().unwrap();
            app.world_mut()
                .resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
                    let mut queue = bevy::ecs::world::CommandQueue::default();
                    let mut commands = Commands::new(&mut queue, world);
                    replace_static_terrain_batches(
                        &mut commands,
                        &mut meshes,
                        &mut root,
                        vec![first.into(), actor_quad_mesh().into()],
                        material.clone(),
                        &frame,
                    );
                    queue.apply(world);
                });
            app.update();
            for old in old_entities.drain(..) {
                assert!(app.world().get_entity(old).is_err());
            }
            let root = root.unwrap();
            assert!(app.world().get::<Mesh3d>(root).is_none());
            assert_eq!(
                *app.world().get::<Transform>(root).unwrap(),
                terrain_transform(&frame)
            );
            let children = app.world().get::<Children>(root).unwrap().to_vec();
            assert_eq!(children.len(), 2);
            assert_eq!(
                *app.world().get::<Aabb>(children[0]).unwrap(),
                expected_bounds
            );
            for &child in &children {
                assert_eq!(
                    app.world()
                        .get::<MeshMaterial3d<VoxelMaterial>>(child)
                        .map(|material| &material.0),
                    Some(&material)
                );
                assert_eq!(
                    *app.world().get::<Transform>(child).unwrap(),
                    Transform::IDENTITY
                );
                assert_eq!(
                    *app.world().get::<Visibility>(child).unwrap(),
                    Visibility::Inherited
                );
                assert_eq!(
                    app.world().get::<GlobalTransform>(child),
                    app.world().get::<GlobalTransform>(root)
                );
                assert!(app.world().get::<InheritedVisibility>(child).unwrap().get());
            }
            let handles = children
                .iter()
                .map(|&child| app.world().get::<Mesh3d>(child).unwrap().clone())
                .collect::<Vec<_>>();
            let count = app.world().resource::<Assets<Mesh>>().len();
            app.world_mut()
                .get_mut::<Transform>(root)
                .unwrap()
                .translation
                .x += 11.0;
            *app.world_mut().get_mut::<Visibility>(root).unwrap() = Visibility::Hidden;
            app.update();
            for &child in &children {
                assert!(!app.world().get::<InheritedVisibility>(child).unwrap().get());
                assert_eq!(
                    app.world().get::<GlobalTransform>(child),
                    app.world().get::<GlobalTransform>(root)
                );
            }
            *app.world_mut().get_mut::<Visibility>(root).unwrap() = Visibility::Visible;
            app.update();
            app.world_mut().clear_trackers();
            app.update();
            assert_eq!(app.world().resource::<Assets<Mesh>>().len(), count);
            for (&child, handle) in children.iter().zip(handles) {
                assert!(app.world().get::<InheritedVisibility>(child).unwrap().get());
                assert_eq!(app.world().get::<Mesh3d>(child), Some(&handle));
                assert!(
                    !app.world()
                        .entity(child)
                        .get_ref::<Mesh3d>()
                        .unwrap()
                        .is_changed()
                );
            }
            assert_eq!(
                app.world_mut()
                    .query_filtered::<Entity, With<VoxelTerrain>>()
                    .iter(app.world())
                    .count(),
                1
            );
            assert_eq!(
                app.world_mut()
                    .query::<&Mesh3d>()
                    .iter(app.world())
                    .count(),
                2
            );
            old_entities.extend(children);
            old_entities.push(root);
        }
    }

    #[test]
    fn terrain_mesh_transfers_to_render_world_without_a_retained_cpu_copy() {
        let mesh = mesh::SurfaceMeshData::default().into_mesh();
        assert_eq!(
            mesh.asset_usage,
            bevy::asset::RenderAssetUsages::RENDER_WORLD
        );
    }

    #[test]
    fn terrain_mesh_replacement_preserves_handle_after_render_world_takes_ownership() {
        let mut meshes = Assets::<Mesh>::default();
        let original = meshes.add(mesh::SurfaceMeshData::default().into_mesh());
        let mut cached = Some(original.clone());
        meshes.remove(original.id()); // Bevy's RENDER_WORLD extraction moves it out.
        let replacement = actor_quad_mesh();
        let vertices = replacement.create_packed_vertex_buffer_data();
        let indices = replacement.indices().unwrap().iter().collect::<Vec<_>>();
        let handle = update_mesh_asset(&mut meshes, &mut cached, replacement);
        assert_eq!(handle.id(), original.id());
        let actual = meshes.get(&handle).unwrap();
        assert_eq!(actual.create_packed_vertex_buffer_data(), vertices);
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
    fn department_store_exit_retires_old_terrain_before_allocating_goldenrod() {
        let old = VisualWorldFrame { map_id: "GoldenrodDeptStore1F".into(), ..default() };
        let next = VisualWorldFrame { active: true, map_id: "GoldenrodCity".into(), ..default() };
        let mut app = App::new();
        let root = app.world_mut().spawn((VoxelTerrain, Visibility::Visible, Transform::IDENTITY)).id();
        let child = app.world_mut().spawn((ChildOf(root), Visibility::Inherited, Transform::IDENTITY)).id();
        app.insert_resource(TerrainRevisionCache {
            key: Some(TerrainCacheKey::from_frame(&old)), built_frame: Some(old),
            instances_root: Some(root), built_footing_heights: vec![1.; 64], footing_heights: vec![1.; 64],
            animated_solid_mesh: Some(Handle::default()), solid_material: Some(Handle::default()),
            ..default()
        }).insert_resource(next).init_resource::<TerrainBuildQueue>()
            .init_resource::<Assets<Image>>().init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<VoxelMaterial>>()
            .add_systems(Update, |frame: Res<VisualWorldFrame>, mut commands: Commands,
                mut cache: ResMut<TerrainRevisionCache>, mut builds: ResMut<TerrainBuildQueue>,
                mut images: ResMut<Assets<Image>>, mut meshes: ResMut<Assets<Mesh>>,
                mut materials: ResMut<Assets<VoxelMaterial>>,
                mut entities: Query<(&mut Visibility, &mut Transform), VoxelTerrainFilter>, mut calls: Local<u8>| {
                let result = sync_terrain(&frame, &live_profiles::LiveProfiles::default(), &mut commands,
                    &mut cache, &mut builds, &mut images, &mut meshes, &mut materials, &mut entities);
                if *calls < 2 { assert_eq!(result, Ok(TerrainSyncState::Pending)); }
                else { assert_eq!(result, Err(TerrainSyncError::SourceTextureUnavailable)); }
                *calls += 1;
            });
        app.update();
        assert!(app.world().get_entity(root).is_err());
        assert!(app.world().get_entity(child).is_err());
        let cache = app.world().resource::<TerrainRevisionCache>();
        assert!(cache.built_frame.is_none() && cache.key.is_none() && cache.instances_root.is_none());
        assert!(cache.animated_solid_mesh.is_none() && cache.solid_material.is_none());
        assert!(cache.built_footing_heights.is_empty() && cache.footing_heights.is_empty());
        assert_eq!(app.world().resource::<TerrainBuildQueue>().started, 0);
        app.update();
        assert_eq!(app.world().resource::<TerrainBuildQueue>().started, 0);
        // Only the third update reaches the replacement atlas/build path.
        app.update();
        assert_eq!(app.world().resource::<TerrainBuildQueue>().started, 1);
    }

    #[test]
    fn newly_queued_terrain_roots_receive_live_scroll_before_the_first_draw() {
        let built = VisualWorldFrame {
            center: Vec2::new(-8.0, 4.0),
            grid_origin: IVec2::new(10, 20),
            tile_size: Vec2::splat(8.0),
            ..default()
        };
        let live = VisualWorldFrame {
            center: Vec2::new(3.5, -1.25),
            grid_origin: IVec2::new(12, 19),
            ..built.clone()
        };
        let offset = terrain_tracking::offset(&built, &live);
        let mut expected = terrain_transform(&live);
        expected.translation += Vec3::new(offset.x, 0.0, offset.y);
        let mut app = App::new();
        app.insert_resource(TerrainRevisionCache {
            key: Some(TerrainCacheKey::from_frame(&live)),
            built_frame: Some(built),
            footing_origin: Some(live.grid_origin),
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
                let root = commands.spawn((VoxelTerrain, (terrain_transform(cache.built_frame.as_ref().unwrap()),
Visibility::Hidden,))).id();
                cache.solid_entity = Some(root);
                assert_eq!(sync_terrain(&frame, &live_profiles::LiveProfiles::default(),
                    &mut commands, &mut cache, &mut builds, &mut images, &mut meshes,
                    &mut materials, &mut entities), Ok(TerrainSyncState::Ready));
            });
        app.update();
        let root = app
            .world()
            .resource::<TerrainRevisionCache>()
            .solid_entity
            .unwrap();
        assert_eq!(*app.world().get::<Transform>(root).unwrap(), expected);
        assert_eq!(
            *app.world().get::<Visibility>(root).unwrap(),
            Visibility::Visible
        );
    }

    #[test]
    fn retained_terrain_never_rebinds_to_the_mutable_live_atlas() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default())).init_asset::<VoxelMaterial>();
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
        .add_systems(Update, sync_cached_terrain_for_test);
        app.update();
        app.world_mut()
            .resource_mut::<Messages<AssetEvent<VoxelMaterial>>>()
            .clear();
        app.world_mut().clear_trackers();

        // Camera interpolation requires a terrain transform update, but must
        // not trigger material extraction and a GPU bind-group rebuild.
        app.world_mut().resource_mut::<VisualWorldFrame>().center.x += 1.0;
        app.update();
        assert_eq!(
            app.world()
                .resource::<Messages<AssetEvent<VoxelMaterial>>>()
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

        let replacement = Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(999), std::marker::PhantomData);
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
                .resource::<Messages<AssetEvent<VoxelMaterial>>>()
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
            depth_write_enabled: Some(true),
            depth_compare: Some(CompareFunction::GreaterEqual),
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        };

        configure_silhouette_depth(&mut depth);

        assert_eq!(depth.depth_compare, Some(CompareFunction::Less));
        assert_eq!(depth.depth_write_enabled, Some(false));
    }

    #[test]
    fn optional_view_remains_disabled_by_default() {
        assert!(!VoxelViewSettings::default().enabled);
    }

    #[test]
    fn cave_clear_color_is_enclosed_void_not_outdoor_horizon() {
        let mut frame = VisualWorldFrame::default();
        frame.tiles.push(crystal_render_api::VisualTile {
            animation_frames: None,
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
        frame.tiles[0].source.tileset_id = "johto".into();
        frame.map_id = "DragonsDenB1F".into();
        assert_eq!(voxel_clear_color(&frame), Color::srgb(0.035, 0.025, 0.065));
        frame.map_id = "Route45".into();
        assert_eq!(voxel_clear_color(&frame), Color::srgb(0.72, 0.83, 0.78));
        frame.map_id = "".into();

        frame.tiles[0].source.tileset_id = "ice_path".into();
        assert_eq!(voxel_clear_color(&frame), Color::srgb(0.025, 0.045, 0.075));
        frame.map_id = "HallOfFame".into();
        assert_ne!(voxel_clear_color(&frame), Color::srgb(0.025, 0.045, 0.075));
        frame.map_id = "".into();

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
            source_map_size_core_tiles: None,
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
            source_map_size_core_tiles: None,
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
            ..default()
        });

        assert!(!voxel_projection_needs_update(&projection, pose));
    }

    #[test]
    fn cave_reveal_tracks_supported_player_and_preserves_actor_and_fade_materials() {
        let mut materials = Assets::<VoxelMaterial>::default();
        let mut terrain = solid_terrain_material();
        terrain.extension.cave_reveal = Vec4::X;
        let solid = materials.add(terrain.clone());
        terrain.extension.cutaway.fade = Vec4::new(0.7, 0.2, 0.3, 0.4);
        let fade = materials.add(terrain);
        let actor = materials.add(solid_terrain_material());
        let actor_original = materials.get(&actor).unwrap().extension.cutaway;
        let frame = VisualWorldFrame {
            active: true, grid_size: UVec2::splat(4), tile_size: Vec2::splat(8.0),
            viewport_size: Vec2::splat(32.0),
            actors: vec![VisualActor { id: VisualActorId::Player, source_id: "player".into(),
                texture: Handle::default(), center: Vec2::new(0.0, 8.0),
                size: Vec2::splat(16.0), flip_x: false, above_priority: false, facing: None }],
            ..default()
        };
        let mut app = App::new();
        app.insert_resource(frame).insert_resource(materials)
            .insert_resource(VoxelViewStatus { active: true, ..default() })
            .insert_resource(TerrainRevisionCache { solid_material: Some(solid.clone()),
                footing_heights: vec![6.0; 16], ..default() })
            .add_systems(Update, sync_interior_cutaway);
        for (x, height) in [(0.0, 6.0), (8.0, 12.0)] {
            app.world_mut().resource_mut::<VisualWorldFrame>().actors[0].center.x = x;
            app.world_mut().resource_mut::<TerrainRevisionCache>().footing_heights.fill(height);
            app.update();
            let assets = app.world().resource::<Assets<VoxelMaterial>>();
            for handle in [&solid, &fade] {
                let capsule = assets.get(handle).unwrap().extension.cutaway;
                assert_eq!(capsule.bottom_radius.truncate(), Vec3::new(x, height + 0.04, 0.0));
                assert_eq!(capsule.top_feather.y, height + 0.04 + 21.6);
            }
            assert_eq!(assets.get(&fade).unwrap().extension.cutaway.fade, Vec4::new(0.7,0.2,0.3,0.4));
            assert_eq!(assets.get(&actor).unwrap().extension.cutaway, actor_original);
            assert_eq!(app.world().resource::<TerrainRevisionCache>().footing_heights, vec![height; 16]);
        }
        app.world_mut().resource_mut::<VoxelViewStatus>().active = false;
        app.update();
        assert_eq!(app.world().resource::<Assets<VoxelMaterial>>().get(&fade).unwrap()
            .extension.cutaway.bottom_radius.w, 0.0);
    }

    #[test]
    fn player_depth_pull_clears_same_level_terrain_at_forty_five_degrees() {
        let player = VisualActor {
            id: VisualActorId::Player,
            source_id: "player".into(),
            texture: Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(1), std::marker::PhantomData),
            center: Vec2::ZERO,
            size: Vec2::splat(16.0),
            flip_x: false,
            above_priority: false,
            facing: None,
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
        let texture = Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(1), std::marker::PhantomData);
        let player = VisualActor {
            id: VisualActorId::Player,
            source_id: "player".into(),
            texture: texture.clone(),
            center: Vec2::ZERO,
            size: Vec2::splat(16.0),
            flip_x: false,
            above_priority: false,
            facing: None,
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
            texture: Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(1), std::marker::PhantomData),
            center: Vec2::ZERO,
            size: Vec2::splat(16.0),
            flip_x: false,
            above_priority: false,
            facing: None,
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
            map_texture: Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(1), std::marker::PhantomData),
            viewport_size: Vec2::new(160.0, 144.0),
            tile_size: Vec2::splat(8.0),
            grid_size: UVec2::new(20, 18),
            ..default()
        };
        let first_key = TerrainCacheKey::from_frame(&first);
        first.map_texture = Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(2), std::marker::PhantomData);
        assert_eq!(first_key, TerrainCacheKey::from_frame(&first));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_camera_inputs_work_across_modeled_maps_and_stop_when_world_is_hidden() {
        for map in ["NewBarkTown", "FastShipB1F", "ViridianGym", "PowerPlant"] {
            let mut app = App::new();
            app.insert_resource(ButtonInput::<KeyCode>::default())
                .insert_resource(VisualWorldFrame {
                    active: true,
                    map_id: map.into(),
                    ..default()
                })
                .insert_resource(VoxelViewSettings {
                    enabled: true,
                    camera: VoxelCameraControls::new(0., 0.),
                    ..default()
                })
                .add_systems(Update, native_model_camera);
            {
                let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
                keys.press(KeyCode::KeyE);
                keys.press(KeyCode::PageUp);
            }
            app.update();
            let moved = app.world().resource::<VoxelViewSettings>().camera;
            assert_eq!(moved, VoxelCameraControls::new(1., 1.), "{map}");

            // Holding a key is not another orbit step. The input plugin clears
            // these edge flags between frames in the real application.
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().clear();
            app.update();
            assert_eq!(app.world().resource::<VoxelViewSettings>().camera, moved);

            app.world_mut().resource_mut::<VisualWorldFrame>().active = false;
            {
                let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
                keys.press(KeyCode::KeyQ);
                keys.press(KeyCode::PageDown);
            }
            app.update();
            assert_eq!(app.world().resource::<VoxelViewSettings>().camera, moved);
        }
    }
}

/// Native modeled-world controls; browser camera controls remain authoritative there.
#[cfg(not(target_arch = "wasm32"))]
fn native_model_camera(
    keyboard: Res<ButtonInput<KeyCode>>,
    frame: Res<VisualWorldFrame>,
    mut settings: ResMut<VoxelViewSettings>,
) {
    if !settings.enabled || !frame.active {
        return;
    }
    static TRACE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if *TRACE.get_or_init(|| std::env::var_os("CRYSTAL_MODEL_INPUT_TRACE").is_some())
        && (keyboard.get_just_pressed().next().is_some()
            || keyboard.get_just_released().next().is_some())
    {
        bevy::log::info!(
            "model input pressed={:?} just_pressed={:?} released={:?}",
            keyboard.get_pressed().collect::<Vec<_>>(),
            keyboard.get_just_pressed().collect::<Vec<_>>(),
            keyboard.get_just_released().collect::<Vec<_>>()
        );
    }
    let mut camera = settings.camera;
    if keyboard.just_pressed(KeyCode::KeyQ) {
        camera.rotation_step -= 1.0;
    }
    if keyboard.just_pressed(KeyCode::KeyE) {
        camera.rotation_step += 1.0;
    }
    if keyboard.just_pressed(KeyCode::PageUp) {
        camera.zoom_step += 1.0;
    }
    if keyboard.just_pressed(KeyCode::PageDown) {
        camera.zoom_step -= 1.0;
    }
    let camera = VoxelCameraControls::new(camera.zoom_step, camera.rotation_step);
    if settings.camera != camera {
        settings.camera = camera;
    }
}
