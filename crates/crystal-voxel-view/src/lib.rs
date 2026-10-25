#![forbid(unsafe_code)]

//! Optional, presentation-only voxel view for Crystal's Bevy shell.

use bevy::camera::primitives::MeshAabb;

#[cfg(not(target_arch = "wasm32"))]
mod camera_gestures;
mod day_cycle;
mod city_lights;
#[cfg(test)]
mod material_borrow_tests;
mod sky;
mod regional_atmosphere;
mod summit_vista;
pub use summit_vista::{SummitBackdrop, SummitBackdropMap, SummitBackdropStatus, summit_preload_interior, summit_preload_foreground_ready};
mod environment_lighting;
mod render_quality;
pub use render_quality::configure_route_shadow_filter;
mod texture_filtering;
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
mod pokemon_appearance;
mod pokemon_follow;
pub use pokemon_follow::pokemon_following_distance;
mod species_rig;
mod battle_tower;
mod battle_view;
mod model_storage;
mod shared_model_geometry;
mod open_models;
mod open_people;
mod open_people_skin;
mod human_locomotion;
mod human_idle;
mod bicycle;
mod seating;
#[cfg(feature = "people-review")]
mod people_review;
#[cfg(feature = "people-review")]
pub use people_review::run as run_people_review;
mod npc_appearance;
mod npc_uniform;
mod hair_curtain;
mod hair_paint;
mod hair_scalp;
mod hair_shapes;
mod npc_textures;
mod browser_memory;
#[cfg(target_arch = "wasm32")]
mod mesh_buffer_retirement;
pub mod loading_trace;
#[cfg(feature = "terrain-worker")]
pub mod terrain_worker;
#[cfg(feature = "actor-worker")]
pub mod actor_worker;
mod shared_groves;
mod forest_instances;
pub use forest_instances::configure_lazy_tree_entities_diagnostics;
pub use occluder_fade::configure_tree_vertex_weld_diagnostics;
pub use shared_groves::configure_grove_collision_diagnostics;
mod grove_stream;

mod destination_preload;
mod world_stream;
mod world_stream_camera_trace;
pub use world_stream::{WorldStreamStatus, WorldStreamTarget};
#[cfg(not(target_arch = "wasm32"))]
mod pipeline_preload;
pub use destination_preload::{DestinationPreload, supported as destination_preload_supported};

mod painted_torso;
mod bare_body;
pub use npc_textures::normalize_npc_texture;
pub use open_people::npc_texture_names;
pub use open_people::{install_open_people_bundle, PEOPLE as OPEN_PEOPLE};
mod scenery_surfaces;
mod scenery_finish;
mod grass_blades;
mod grass_rustle;
mod garment_fit;
mod garment_body;
mod tailored_trousers;
#[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
mod garment_drape_trial;
mod limb_surface;
mod arm_skin;
mod arm_deformation;
mod arm_surface;
mod hand_surface;
mod hand_digits;
mod handwear;
mod hand_pose;
mod head_surface;
mod face_volume;
mod people_mesh;
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
mod scenery_variants;
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
mod border_water;
mod cave_lighting;
mod source_color;
mod water_surface;
mod water_ripples;
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
mod outdoor_wind;
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
mod terrain_upload;
mod terrain_publication;
mod terrain_disposal;
#[cfg(not(target_arch = "wasm32"))]
mod gpu_retirement;
mod terrain_animation;
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
mod environment_fx;
#[cfg(not(target_arch = "wasm32"))]
mod render_metrics;
#[cfg(not(target_arch = "wasm32"))]
mod shadow_bounds;
#[cfg(not(target_arch = "wasm32"))]
mod shadow_fit;
mod shadow_receivers;
pub use shadow_receivers::configure_shadow_receiver_diagnostics;
#[cfg(not(target_arch = "wasm32"))]
mod gpu_culling;
#[cfg(not(target_arch = "wasm32"))]
mod cluster_qa;
mod compact_vertices;
mod wise_trios;

use std::collections::{HashMap, HashSet};

#[cfg(not(target_arch = "wasm32"))]
use bevy::tasks::{AsyncComputeTaskPool, futures_lite::future};
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
};
mod healing_effects;
pub use healing_effects::{native_clinic_healing_map, native_clinic_healing_supported};
use crystal_render_api::{VisualActor, VisualActorId, VisualWorldFrame, WorldRenderSet};

pub use camera::{CAMERA_PITCH_DEGREES, VoxelCameraControls, VoxelCameraPose, camera_pose};
pub use footing::{
    actor_foot, footing_height, resolved_footing_height, tile_at_visual_point,
    visual_point_to_voxel,
};
pub use mesh::{
    CellCoverageKind, SurfaceMeshData, TerrainImageSamples, TerrainMeshError, audit_cell_coverage,
    audit_cell_coverage_on_map, audit_terrain_mesh_with_images, build_terrain_mesh, jump_changes_floor_level,
    build_terrain_mesh_with_images, build_terrain_mesh_with_samples,
    source_terrace_datum, TerraceSourceRegion, connected_source_terrace_datum,
};
pub use cave_water::outdoor as outdoor_water_map;
pub use profile::{
    COMPACT_BUILDING_HEIGHT, CellShape, GROUND_HEIGHT, LARGE_BUILDING_HEIGHT, MAX_PROFILE_HEIGHT,
    MIN_PROFILE_HEIGHT, SOURCE_TILE_HEIGHT, SolidKind, WATER_HEIGHT, shape_for_source,
    support_height, supports_frame_profile,
};

/// Count the production window emitters in a complete outdoor map mesh, before
/// selecting the nearest visible sources. Interior maps have no exterior lights.
pub fn audit_city_window_sources(map: &str, mesh: &SurfaceMeshData) -> Option<usize> {
    city_lights::enabled(map).then(|| city_lights::collect(mesh).len())
}

/// Exact resolved actor-source coverage for diagnostics. This does not accept
/// script tokens, infer identities from an object slot, or guess species.
pub fn authored_actor_source(source: &str) -> Option<&'static str> {
    new_bark_actors::authored_source(source)
}

/// Exact normal-palette battle mesh availability, excluding generic icons.
pub fn has_authored_battle_species(species: &str) -> bool {
    battle_species_models::supported_species().any(|id| id.eq_ignore_ascii_case(species))
}

/// Parking layer used to keep the classic overworld out of every active
/// camera while the user has manually selected 2.5D.
pub const HIDDEN_CLASSIC_WORLD_RENDER_LAYER: usize = 30;

const VOXEL_RENDER_LAYER: usize = 31;
/// Host UI camera compositing over the world. Its main-buffer format follows
/// the active world camera; its tonemapping and effects remain host-owned.
#[derive(Component)]
pub struct WorldOverlayCamera;
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
        browser_memory::register(app);
        #[cfg(target_arch = "wasm32")]
        mesh_buffer_retirement::register(app);
        loading_trace::register(app);
        #[cfg(not(target_arch = "wasm32"))]
        pipeline_preload::register(app);

        #[cfg(not(target_arch = "wasm32"))]
        render_metrics::register(app);
        #[cfg(not(target_arch = "wasm32"))]
        shadow_bounds::register(app);
        #[cfg(not(target_arch = "wasm32"))]
        shadow_fit::register(app);
        shadow_receivers::register(app);
        #[cfg(not(target_arch = "wasm32"))]
        gpu_culling::register(app);
        #[cfg(not(target_arch = "wasm32"))]
        cluster_qa::register(app);
        load_internal_asset!(
            app,
            VOXEL_SURFACE_SHADER_HANDLE,
            "voxel_surface.wesl",
            Shader::from_wesl
        );
        load_internal_asset!(
            app,
            GRASS_VERTEX_SHADER_HANDLE,
            "grass_vertex.wesl",
            Shader::from_wesl
        );
        // Native visual review can iterate the same source shader without
        // recompiling the game. Shipped clients use the embedded shader.
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(path) = std::env::var_os("CRYSTAL_VOXEL_SHADER_REVIEW") {
            let source = std::fs::read_to_string(path)
                .expect("read explicitly selected voxel surface review shader");
            let mut shaders = app.world_mut().resource_mut::<Assets<Shader>>();
            let mut shader = shaders.get_mut(&VOXEL_SURFACE_SHADER_HANDLE)
                .expect("embedded voxel surface shader is registered");
            *shader = Shader::from_wesl(source, shader.path.clone());
        }
        app.add_plugins(MaterialPlugin::<VoxelMaterial>::default());
        #[cfg(not(target_arch = "wasm32"))]
        {
            app.add_systems(Update, native_model_camera.in_set(VoxelCameraInput).before(sync_voxel_view));
            camera_gestures::register(app);
        }
        load_internal_asset!(
            app,
            SILHOUETTE_SHADER_HANDLE,
            "occlusion_silhouette.wesl",
            Shader::from_wesl
        );
        app.add_plugins(MaterialPlugin::<OcclusionSilhouetteMaterial>::default());
        sky::register(app);
        summit_vista::register(app);
        environment_lighting::register(app);
        render_quality::register(app);
        app.init_resource::<new_bark_actors::ModeledActors>()
            .add_systems(
                Update,
                new_bark_actors::sync
                    .after(sync_voxel_view)
                    .in_set(WorldRenderSet::RenderSync),
            )
            .init_resource::<VoxelViewSettings>()
            .init_resource::<VoxelViewStatus>()
            .init_resource::<NativeHealingEffectsStatus>()
            .init_resource::<live_profiles::LiveProfiles>()
            .add_systems(Update, live_profiles::reload.before(sync_voxel_view))
            .init_resource::<ActorScreenHeads>()
            .init_resource::<VoxelScene>()
            .init_resource::<TerrainRevisionCache>()
            .init_resource::<TerrainBuildQueue>()
            .init_resource::<DestinationPreload>()
            .init_resource::<WorldStreamStatus>()
            .init_resource::<destination_preload::PreparedDestination>()
            .init_resource::<destination_preload::PreparedWorld>()
            .add_systems(Update, world_stream::update_interest.after(sync_voxel_view)
                .in_set(WorldRenderSet::RenderSync))
            .add_systems(Update, destination_preload::prepare.after(world_stream::update_interest)
                .in_set(WorldRenderSet::RenderSync))
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
            .add_systems(Update, city_lights::sync.after(new_bark_actors::sync)
                .after(sync_interior_cutaway).in_set(WorldRenderSet::RenderSync))
            .add_systems(Update, terrain_animation::sync.after(sync_voxel_view)
                .before(sync_cave_water_phases).in_set(WorldRenderSet::RenderSync))
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
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Update, new_bark_actors::prepare_destination_people
            .after(new_bark_actors::sync).in_set(WorldRenderSet::RenderSync));
        #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
        garment_drape_trial::live::register(app);
        grass_rustle::register(app);
        outdoor_wind::register(app);
        app.add_systems(Update, flower::animate.after(sync_voxel_view)
            .in_set(WorldRenderSet::RenderSync));
        occluder_fade::register(app);
        terrain_upload::register(app);
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

/// Camera input settles before the shell projects field movement into the world.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct VoxelCameraInput;

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

pub use terrain_tracking::connected_frame_shift;
pub use forest_instances::configure_understory_density;
/// Decorative plant density, independent of full-detail authored models.
pub fn understory_full_detail_enabled() -> bool { forest_instances::dense_understory() }

/// Report the existing scenery-detail selection for read-only diagnostics.
pub fn scenery_full_detail_enabled() -> bool {
    new_bark_models::scenery_full_detail()
}

/// Observable presentation state for the developer location tester. Gameplay
/// never reads this resource.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct NativeHealingEffectsStatus { pub consumed:bool }

#[derive(Resource,Default,Clone,Debug,PartialEq,Eq)]
pub struct VoxelViewStatus {
    pub active: bool,
    pub active_frames: u32,
    /// Completed or pending geometry builds; useful for movement profiling.
    pub terrain_builds: u64,
    /// Map whose completed geometry is currently installed in the scene.
    pub terrain_map: Option<String>,
    /// The desired terrain revision has not reached the scene yet.
    pub terrain_pending: bool,
    /// Authored connected acreage remains visible while its new scene builds.
    pub connected_terrain: bool,
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
    source_regions: std::sync::Arc<[crystal_render_api::VisualSourceRegion]>,
    source_map_size_core_tiles: Option<UVec2>,
    boundary_warps: std::sync::Arc<[IVec2]>,
    jump_ledges: std::sync::Arc<[crystal_render_api::VisualJumpLedge]>,
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
            source_regions: frame.source_regions.clone(),
            source_map_size_core_tiles: frame.source_map_size_core_tiles,
            boundary_warps: frame.boundary_warps.clone(),
            jump_ledges: frame.jump_ledges.clone(),
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
    uploads: terrain_upload::Pending,
    city_emitters: Vec<city_lights::Emitter>,
    forest_library: std::sync::Arc<forest_instances::Library>,
    groves: Option<grove_stream::Stream>,
    painted_stone: Option<Handle<Image>>,
    painted_bedrock: Option<Handle<Image>>,
    built_frame: Option<VisualWorldFrame>,
    // Exact profile document consumed by the completed build, never a desired revision.
    built_profiles: Option<std::sync::Arc<live_profiles::Document>>,
    // Original compositor handle at build submission; built_frame owns its atlas copy.
    built_source_texture: Option<Handle<Image>>,
    // Live compositor authenticated against the completed preload's cells.
    // Keep this separate from its original submission receipt above.
    verified_source_texture: Option<Handle<Image>>,
    // Exact current bounds for the two mutable animated material domains.
    built_animated_bounds: [Option<bevy::camera::primitives::Aabb>; 2],
    built_footing_heights: Vec<f32>,
    built_animated_pillars: Vec<[usize; 2]>,
    built_train_doors: bool,
    built_train_open: Option<u32>,
    footing_origin: Option<IVec2>,
    footing_map: Option<std::sync::Arc<str>>,
    connected_proof: Option<ConnectedTerrainProof>,
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
    animated_solid_material: Option<Handle<VoxelMaterial>>,
    grass_material: Option<Handle<VoxelMaterial>>,
    footing_heights: Vec<f32>,
}

#[derive(Resource, Default)]
struct TerrainBuildQueue {
    started: u64,
    speculative: bool,
    defer_to_world_preload: bool,
    publication: Option<terrain_publication::Pending>,
    // Let asset removal/render extraction retire the previous map before a
    // synchronous browser build allocates the replacement scene.
    retirement_frames: u8,
    key: Option<TerrainCacheKey>,
    #[cfg(not(target_arch = "wasm32"))]
    task: Option<Task<TerrainBuildResult>>,
    #[cfg(not(target_arch = "wasm32"))]
    painted: painted_stone::NativePreparation,
    #[cfg(target_arch = "wasm32")]
    pending: Option<BrowserTerrainBuild>,
}

#[cfg(target_arch = "wasm32")]
struct BrowserTerrainBuild {
    key: TerrainCacheKey,
    frame: VisualWorldFrame,
    source_texture: Handle<Image>,
    profiles: std::sync::Arc<live_profiles::Document>,
    work: Option<BrowserTerrainWork>,
}
#[cfg(target_arch = "wasm32")]
enum BrowserTerrainWork {
    Synchronous(Result<mesh::PendingRuntimeTerrain, TerrainMeshError>),
    #[cfg(feature = "terrain-worker-browser")]
    Worker { ticket: terrain_worker::browser::Ticket, samples: TerrainImageSamples },
}
#[cfg(target_arch = "wasm32")]
impl BrowserTerrainBuild {
    fn complete(self, terrain: Result<BuiltTerrain, TerrainMeshError>) -> TerrainBuildResult {
        TerrainBuildResult { key: self.key, frame: self.frame,
            source_texture: self.source_texture, profiles: self.profiles, terrain }
    }
}

struct TerrainBuildResult {
    key: TerrainCacheKey,
    frame: VisualWorldFrame,
    source_texture: Handle<Image>,
    profiles: std::sync::Arc<live_profiles::Document>,
    terrain: Result<BuiltTerrain, TerrainMeshError>,
    #[cfg(not(target_arch = "wasm32"))]
    water: Option<water_surface::PreparedImages>,
}

fn prepare_runtime_terrain(
    runtime: mesh::RuntimeTerrainBuild,
    build_frame: &VisualWorldFrame,
) -> BuiltTerrain {
    let _trace = loading_trace::span("region.prepare_render_meshes", || serde_json::json!({"map": build_frame.map_id.as_ref()}));
    let mesh::RuntimeTerrainBuild {
        mut terrain,
        grove_groups,
        grove_opaque,
        streamed_groves,
    } = runtime;
    // Runtime groves arrive prepared; a revision publishes only once complete.
    // Whole-object translucency can expose every original join.
    // Keep these triangles in their owning wall rather than applying
    // the old capsule's camera-dependent hidden-face selection.
    terrain.reveal_join_batches.clear();
    let reveal_join_batches = Vec::new();
    let animated_textured_mesh = std::mem::take(&mut terrain.animated_textured).into_mesh();
    let animated_solid_mesh = std::mem::take(&mut terrain.animated_solid).into_mesh();
    let animated_pillars = std::mem::take(&mut terrain.animated_pillars);
    let train_doors = terrain.animated_train_doors;
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
    let footing_heights = std::mem::take(&mut terrain.footing_heights);
    let city_emitters = if city_lights::enabled(&build_frame.map_id) {
        city_lights::collect(&terrain.solid)
    } else { Vec::new() };
    let (textured, solid, mut fade_textured_groups, mut fade_solid_groups) = {
        let _trace = loading_trace::span("region.prepare_linked", || serde_json::json!({
            "map":build_frame.map_id.as_ref(),
            "textured_triangles":terrain.textured.indices.len()/3,
            "solid_triangles":terrain.solid.indices.len()/3,
        }));
        occluder_fade::prepare_linked_with_layered_alpha(terrain.textured, terrain.solid,
            terrain.cutaway_links, &terrain.layered_alpha_textured_ranges)
    };
    occluder_fade::use_source_depth_coverage_fade(&mut fade_textured_groups,
        &terrain.depth_coverage_textured_ranges);
    occluder_fade::use_source_depth_coverage_fade(&mut fade_solid_groups,
        &terrain.depth_coverage_solid_ranges);
    occluder_fade::use_camera_walls(&mut fade_solid_groups, &terrain.camera_wall_solid_ranges);
    occluder_fade::use_camera_walls(&mut fade_textured_groups, &terrain.camera_wall_textured_ranges);
    // Retain the obstructing building as a smooth faint form, with depth
    // ordering and an authoritative player aperture keeping the trainer clear.
    occluder_fade::use_building_reveal(&mut fade_solid_groups, &terrain.coverage_fade_ranges);
    // Grove ownership is independent of linked wall/label anchors.
    fade_solid_groups.extend(grove_groups);
    let source_bounds = authentic_mesh_bounds(&build_frame);
    let mut scenery = Vec::new();
    let mut scenery_fades = Vec::new();
    for (index, part) in std::mem::take(&mut terrain.scenery).into_iter().enumerate() {
        let _part_trace = loading_trace::span("region.prepare_scenery_part", || serde_json::json!({
            "map":build_frame.map_id.as_ref(),"index":index,
            "vertices":part.mesh.positions.len(),"triangles":part.mesh.indices.len()/3,
            "cutaway_ranges":part.mesh.cutaway_ranges.len(),
        }));
        for (domain, data) in terrain_batches::separate_authentic_domains(part.mesh, source_bounds)
        {
            let (opaque, groups) = occluder_fade::prepare(data);
            if !groups.is_empty() {
                scenery_fades.push((groups, part.texture.clone(), domain));
            }
            if !opaque.indices.is_empty() {
                scenery.extend(
                    terrain_batches::prepare_with_authentic_bounds(
                        opaque,
                        build_frame.tile_size,
                        source_bounds,
                    )
                    .into_iter()
                    .map(|batch| (batch, part.texture.clone())),
                );
            }
        }
    }
    let prepare_surfaces = if cave_enclosure.is_some() {
        terrain_batches::prepare_enclosed_cave
    } else {
        terrain_batches::prepare_with_authentic_bounds
    };
    let textured_meshes = {
        let _trace = loading_trace::span("region.prepare_textured_surfaces", || serde_json::json!({
            "map":build_frame.map_id.as_ref(),"triangles":textured.indices.len()/3,
        }));
        prepare_surfaces(textured, build_frame.tile_size, source_bounds)
    };
    let mut solid_meshes = {
        let _trace = loading_trace::span("region.prepare_solid_surfaces", || serde_json::json!({
            "map":build_frame.map_id.as_ref(),"triangles":solid.indices.len()/3,
        }));
        prepare_surfaces(solid, build_frame.tile_size, source_bounds)
    };
    {
        let _trace = loading_trace::span("region.prepare_grove_opaque", || serde_json::json!({
            "map":build_frame.map_id.as_ref(),"parts":grove_opaque.len(),
        }));
        for opaque in grove_opaque {
            solid_meshes.extend(prepare_surfaces(
                opaque,
                build_frame.tile_size,
                source_bounds,
            ));
        }
    }
    BuiltTerrain {
        city_emitters,
        streamed_groves,
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
        train_doors,
    }
}

struct BuiltTerrain {
    city_emitters: Vec<city_lights::Emitter>,
    streamed_groves: Option<std::sync::Arc<mesh::StreamedGroves>>,
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
    train_doors: bool,
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
fn lighthouse_room(map:&str)->bool {
    matches!(map,"OlivineLighthouse1F"|"OlivineLighthouse2F"|"OlivineLighthouse3F"
        |"OlivineLighthouse4F"|"OlivineLighthouse5F"|"OlivineLighthouse6F")
}
fn bind_preloaded_compositor(frame: &VisualWorldFrame, cache: &mut TerrainRevisionCache) {
    cache.verified_source_texture = if cache.built_source_texture.as_ref() == Some(&frame.map_texture) {
        None
    } else {
        cache.built_frame.as_ref().filter(|built|
            built.validate().is_ok() && frame.validate().is_ok()
                && destination_preload::compatible(built, frame))
            .map(|_| frame.map_texture.clone())
    };
}

struct ConnectedTerrainProof {
    key: TerrainCacheKey,
    datum: Option<std::sync::Arc<crystal_render_api::VisualTerraceDatum>>,
    valid: bool,
}
fn retain_connected_terrain(frame: &VisualWorldFrame, cache: &mut TerrainRevisionCache) -> bool {
    let Some(built) = cache.built_frame.as_ref().filter(|built|
        cache.key.is_some() && built.map_id != frame.map_id) else { return false; };
    let key = TerrainCacheKey::from_frame(frame);
    let same_datum = |old: &Option<std::sync::Arc<crystal_render_api::VisualTerraceDatum>>| match (old, &frame.terrace_datum) {
        (None,None) => true,
        (Some(a),Some(b)) => std::sync::Arc::ptr_eq(a,b),
        _ => false,
    };
    if let Some(proof) = cache.connected_proof.as_ref().filter(|proof|
        proof.key == key && same_datum(&proof.datum)) { return proof.valid; }
    let valid = terrain_tracking::connected_frame_shift(built, frame).is_ok();
    cache.connected_proof = Some(ConnectedTerrainProof { key, datum: frame.terrace_datum.clone(), valid });
    valid
}

fn retain_lighthouse_room(frame:&VisualWorldFrame,cache:&TerrainRevisionCache)->bool {
    cache.key.is_some() && lighthouse_room(&frame.map_id)
        && cache.built_frame.as_ref().is_some_and(|old|old.map_id!=frame.map_id&&lighthouse_room(&old.map_id)
            // Only retain a completed mesh's immutable atlas, never the host's
            // reusable image while it is being uploaded for the next room.
            && cache.built_source_texture.as_ref().is_some_and(|source|*source!=old.map_texture))
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
    device: Option<Res<bevy::render::renderer::RenderDevice>>,
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
    #[cfg(not(target_arch = "wasm32"))]
    commands.entity(camera).insert(pipeline_preload::WarmupView);
    if TREE_DEPTH_PREPASS.get().copied().unwrap_or(false) {
        commands.entity(camera).insert(bevy::core_pipeline::prepass::DepthPrepass);
    }
    render_quality::configure_camera(&mut commands, camera, adapter.as_deref(), device.as_deref());
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
    presentation: (
        Res<VoxelViewSettings>,
        Res<live_profiles::LiveProfiles>,
        Query<
            (&DirectionalLight, &Transform),
            (
                With<VoxelSun>,
                Without<VoxelWorldCamera>,
                Without<VoxelTerrain>,
                Without<VoxelActorCard>,
            ),
        >,
        ResMut<NativeHealingEffectsStatus>,
        Option<Res<crystal_render_api::VisualPlayerTraversal>>,
        Option<ResMut<DestinationPreload>>,
        Option<ResMut<destination_preload::PreparedDestination>>,
        Option<ResMut<destination_preload::PreparedWorld>>,
        Option<ResMut<WorldStreamStatus>>,
    ),
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
    let (
        settings,
        profiles,
        sun,
        mut healing_status,
        traversal,
        mut preload,
        mut prepared_destination,
        mut prepared_world,
        mut streaming,
    ) = presentation;
    let previous_root = destination_preload::roots(&terrain_cache).next();
    let previous_build_pending = terrain_builds.key.is_some();
    status.connected_terrain = false;
    if settings.enabled && frame.active {
        if let (Some(world), Some(streaming)) = (prepared_world.as_mut(), streaming.as_mut()) {
            destination_preload::activate_world(
                &frame,
                &profiles,
                world,
                streaming,
                &mut terrain_cache,
                &mut terrain_builds,
                &mut commands,
            );
        }
        if let (Some(request), Some(destination)) =
            (preload.as_mut(), prepared_destination.as_mut())
        {
            destination_preload::activate(
                &frame,
                &profiles,
                destination,
                request,
                &mut terrain_cache,
                &mut terrain_builds,
                &mut commands,
            );
        }
    }
    let _trace = loading_trace::span(
        "frame.sync_3d",
        || serde_json::json!({"map": frame.map_id.as_ref(), "enabled": settings.enabled}),
    );
    status.profiles_pending = terrain_cache
        .key
        .as_ref()
        .is_none_or(|key| key.profiles_revision != profiles.revision);
    if status.profiles_pending {
        status.active_frames = 0;
    }
    if status.active
        && previous_root == destination_preload::roots(&terrain_cache).next()
        && !status.terrain_pending
        && !profiles.is_changed()
        && !status.profiles_pending
        && !frame.is_changed()
        && !settings.is_changed()
        && terrain_builds.key.is_none()
        && terrain_cache.groves.is_none()
    {
        // The published frame is retained and Bevy change detection proves
        // there is no camera, actor, texture, or terrain work to synchronize.
        // Avoid validating and profile-scanning the complete terrain grid on
        // every vsynced host frame while the game is visually idle.
        status.active_frames = status.active_frames.saturating_add(1);
        return;
    }
    healing_status.consumed = false;
    let holding_lighthouse = retain_lighthouse_room(&frame, &terrain_cache);
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
    } else if !images.contains(&frame.map_texture) && !holding_lighthouse {
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
    // Until terrain sync aligns support to this frame, only the inactive
    // output path may use a pose. Resolve the moving camera afterwards.
    let mut pose = settings.camera.pose(frame.viewport_size);
    let valid = failure.is_none();
    let output_ready = if valid {
        // Check ownership without publishing a pose from the previous support
        // grid. A held lighthouse keeps its complete old room's camera active.
        scene.camera.is_some_and(|entity| {
            cameras.get_mut(entity).is_ok_and(|(mut camera, _, _)| {
                if holding_lighthouse {
                    camera.is_active = true;
                }
                true
            })
        })
    } else {
        set_output_active(&scene, false, &mut cameras, pose)
    };
    if valid && !output_ready {
        let failure = "voxel world camera is unavailable";
        status.active = false;
        status.terrain_pending = false;
        status.active_frames = 0;
        status.inactive_reason = Some(failure.to_owned());
        report_inactive_change(&mut last_failure, failure);
        return;
    }
    if !valid {
        #[cfg(not(target_arch = "wasm32"))]
        {
            terrain_builds.painted.cancel();
        }
        #[cfg(target_arch = "wasm32")]
        {
            terrain_builds.pending = None;
            terrain_builds.key = None;
        }
        let failure = failure.unwrap_or("unknown reason");
        status.active = false;
        status.terrain_pending = false;
        status.active_frames = 0;
        status.inactive_reason = Some(failure.to_owned());
        report_inactive_change(&mut last_failure, failure);
        return;
    }

    {
        terrain_builds.speculative = false;
        terrain_builds.defer_to_world_preload = terrain_cache.built_frame.is_some()
            && prepared_world.as_mut().is_some_and(|world|
                destination_preload::pending_world_covers(&frame, &profiles, world));
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
        Err(TerrainSyncError::SourceTextureUnavailable) if holding_lighthouse => {
            TerrainSyncState::Pending
        }
        Err(error) => {
            let failure = format!("terrain sync failed: {error:?}");
            status.active = false;
            status.terrain_pending = false;
            status.active_frames = 0;
            status.inactive_reason = Some(failure.clone());
            report_inactive_change(&mut last_failure, &failure);
            set_output_active(&scene, false, &mut cameras, pose);
            return;
        }
    };
    status.connected_terrain = retain_connected_terrain(&frame, &mut terrain_cache);
    if !holding_lighthouse || matches!(terrain_state, TerrainSyncState::Ready) {
        let _trace = loading_trace::span(
            "frame.resolve_world_camera",
            || serde_json::json!({"map":frame.map_id.as_ref()}),
        );
        // Alignment also changes during same-map scrolling while a replacement
        // is pending. Sampling the previous grid produced a one-frame height
        // jump followed by a correction without any player movement.
        pose = settings.camera.world_pose_with_traversal(
            &frame,
            &terrain_cache.footing_heights,
            traversal.as_deref(),
        );
        if previous_root != destination_preload::roots(&terrain_cache).next()
            || (previous_build_pending && terrain_builds.key.is_none())
        {
            let old_eye = scene.camera.and_then(|entity| {
                cameras.get_mut(entity).ok().map(|(_, _, transform)| transform.translation)
            });
            world_stream_camera_trace::publish(&frame, &settings, &terrain_cache,
                traversal.as_deref(), old_eye, pose);
        }
        if !holding_lighthouse {
            set_output_active(&scene, true, &mut cameras, pose);
        }
    }
    status.terrain_builds = terrain_builds.started;
    let applied_map = terrain_cache.key.as_ref().map(|key| key.map_id.as_ref());
    if status.terrain_map.as_deref() != applied_map {
        status.terrain_map = applied_map.map(str::to_owned);
    }
    status.terrain_pending = matches!(terrain_state, TerrainSyncState::Pending);
    if matches!(terrain_state, TerrainSyncState::Ready) {
        // The pose already observes support published or aligned in this call.
        let projection = cameras
            .iter()
            .find_map(|(_, projection, _)| match projection {
                Projection::Perspective(p) => Some(Mat4::perspective_rh(
                    pose.vertical_fov_radians,
                    p.aspect_ratio,
                    pose.near,
                    pose.far,
                )),
                _ => None,
            });
        if let Some(projection) = projection {
            let view_projection = projection * Mat4::look_at_rh(pose.eye, pose.target, Vec3::Y);
            let sun = sun.iter().find_map(|(light, transform)| {
                (light.shadow_maps_enabled && light.illuminance > 0.)
                    .then(|| -transform.forward().as_vec3())
            });
            status.terrain_pending |= grove_stream::sync(
                &frame,
                view_projection,
                sun,
                &mut terrain_cache,
                &mut commands,
                &mut meshes,
                &mut materials,
            );
        }
    }
    if status.terrain_pending {
        status.active_frames = 0;
    }
    if retain_lighthouse_room(&frame, &terrain_cache) && status.terrain_pending {
        // Keep the previous actors with their room until geometry, atlas and
        // support for the destination can be published together.
        status.active = true;
        status.inactive_reason = None;
        return;
    }
    if holding_lighthouse {
        set_output_active(&scene, true, &mut cameras, pose);
    }
    if must_wait_for_initial_terrain(terrain_state, &terrain_cache) {
        // Initial activation has no authored mesh to present yet. Keep the
        // manually selected 2.5D presentation inactive until the first build
        // completes; the classic world remains parked on its hidden layer.
        // Subsequent revisions keep the last complete terrain alive while its
        // replacement builds.
        status.active = false;
        status.active_frames = 0;
        status.inactive_reason = Some("building authored terrain".to_owned());
        set_output_active(&scene, false, &mut cameras, pose);
        return;
    }

    let Some(actor_quad) = scene.actor_quad.as_ref() else {
        status.active = false;
        status.terrain_pending = false;
        status.active_frames = 0;
        status.inactive_reason = Some("actor card mesh is unavailable".to_owned());
        report_inactive_change(&mut last_failure, "actor card mesh is unavailable");
        set_output_active(&scene, false, &mut cameras, pose);
        return;
    };
    let healing_allowed = frame.actors.iter().any(healing_effects::is_healing)
        && healing_effects::profile_allows(&frame, &profiles.document);
    if let Err(error) = sync_actor_cards(
        &frame,
        &terrain_cache.footing_heights,
        pose,
        healing_allowed,
        actor_quad,
        &mut commands,
        &mut actor_cache,
        &images,
        &mut materials,
        &mut actor_entities,
    ) {
        let failure = format!("actor sync failed: {error:?}");
        status.active = false;
        status.terrain_pending = false;
        status.active_frames = 0;
        status.inactive_reason = Some(failure.clone());
        report_inactive_change(&mut last_failure, &failure);
        set_output_active(&scene, false, &mut cameras, pose);
        return;
    }
    if let Some(entity) = actor_cache.entities.get(&VisualActorId::Player) {
        commands
            .entity(*entity)
            .insert(if settings.camera.first_person {
                Visibility::Hidden
            } else {
                Visibility::Visible
            });
    }
    healing_status.consumed =
        healing_allowed && frame.actors.iter().any(healing_effects::is_healing);
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
    time: Res<Time>,
    frame: Res<VisualWorldFrame>,
    phases: Option<Res<crystal_render_api::VisualTileAnimationFrame>>,
    status: Res<VoxelViewStatus>,
    mut cache: ResMut<TerrainRevisionCache>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
    mut applied: Local<Option<(Option<TerrainCacheKey>, Option<bevy::asset::AssetId<Image>>, bevy::asset::AssetId<Image>)>>,
    mut ripples: Local<water_ripples::WaterRipples>,
) {
    if !status.active || !frame.active {
        *applied = None;
        *ripples = default();
        return;
    }
    // Rooted sea stones use the same map coordinates as the water around
    // them. Solid materials need that scale and retained-grid translation,
    // while X stays zero so their geometry never enters water optics.
    // Coordinate updates must not wait for an animation phase to advance.
    if let (Some(built), Some(handle)) = (cache.built_frame.as_ref(), cache.solid_material.as_ref()) {
        let mut registration = cave_water::retained_registration(built, &frame);
        registration.x = 0.;
        if materials.get(handle).is_some_and(|m|m.extension.water_enabled != registration) {
            materials.get_mut(handle).unwrap().extension.water_enabled = registration;
        }
    }
    let Some(phases) = phases else { return; };
    let material_handle = cache.textured_material.clone();
    let mask = material_handle.as_ref()
        .and_then(|handle| materials.get(handle))
        .and_then(|material| material.extension.water_mask.clone());
    let Some(atlas) = cache.built_frame.as_ref().map(|built| built.map_texture.id()) else { return; };
    let next = (cache.key.clone(), mask.as_ref().map(Handle::id), atlas);
    // Source animation is sparse. Scanning every retained water tile and its
    // mask on every display refresh cannot advance an authenticated phase.
    // A new terrain/atlas/mask must still receive the current phase once.
    let new_surface = applied.as_ref() != Some(&next);
    let refresh_source = new_surface || phases.is_changed();
    if !refresh_source { return; }
    if phases.map_id != frame.map_id { *applied = None; return; }
    *applied = Some(next);
    let built = cache.built_frame.as_mut().unwrap();
    cave_water::refresh_phases(built, &phases, &mut images);
    if let Some(mask) = mask {
        let (changed, steps) = cave_water::refresh_mask_with_steps(built, &mask, &mut images);
        let palette=if (changed || refresh_source)
            && (cave_water::outdoor_source_palette(built) || cave_water::native_cave_source(built)) {
            // A missing/dark replacement must clear an earlier lit swatch too.
            Some(cave_water::native_palette(built,&images).unwrap_or([Vec4::ZERO;2]))
        } else {None};
        let ripple_height = if changed || new_surface {
            let shore = material_handle.as_ref()
                .and_then(|handle| materials.get(handle))
                .and_then(|material| material.extension.water_shore.clone());
            let active = shore.and_then(|shore| images.get_mut(&shore))
                .is_some_and(|mut shore| ripples.update(built, water_surface::contact(&frame), steps, &mut shore));
            Some(if active { water_ripples::HEIGHT_RANGE } else { 0.0 })
        } else { None };
        if changed || palette.is_some() || ripple_height.is_some() {
            // Re-uploading an Image replaces its GPU texture. Refresh every
            // material that binds it, including cloned cave occluder groups.
            let users: Vec<_> = materials.iter().filter_map(|(id, material)| {
                (material.extension.water_mask.as_ref() == Some(&mask)).then_some(id)
            }).collect();
            for id in users {
                if let Some(mut material) = materials.get_mut(id) {
                    if changed {
                        material.extension.water_motion = water_surface::advance_motion(
                            material.extension.water_motion, steps);
                        material.extension.water_phase = water_surface::blend_clock(
                            material.extension.water_phase, time.elapsed_secs_wrapped(), steps);
                    }
                    if let Some(palette)=palette {
                        material.extension.water_deep=palette[0];
                        material.extension.water_foam=palette[1];
                    }
                    if let Some(height) = ripple_height {
                        material.extension.water_motion.z = height;
                    }
                }
            }
        }
    }
    let registration = cave_water::retained_registration(built, &frame);
    let contact = water_surface::contact(&frame);
    if let Some(handle) = material_handle {
        if materials.get(&handle).is_some_and(|material|
            material.extension.water_contact != contact) {
            materials.get_mut(&handle).unwrap().extension.water_contact = contact;
        }
        if materials.get(&handle).is_some_and(|material|
            material.extension.water_enabled.x > 0.5 && material.extension.water_enabled != registration)
        {
            materials.get_mut(&handle).unwrap().extension.water_enabled = registration;
        }
    }
}

fn sync_voxel_atmosphere(
    frame: Res<VisualWorldFrame>,
    lighting: Option<Res<crystal_render_api::VisualWorldLighting>>,
    clock: Option<Res<crystal_render_api::VisualWorldClock>>,
    profiles: Res<live_profiles::LiveProfiles>,
    settings: Res<VoxelViewSettings>,
    adapter: Option<Res<RenderAdapterInfo>>,
    mut lights: Query<(&mut DirectionalLight, &mut bevy::light::CascadeShadowConfig, Option<&mut Transform>), With<VoxelSun>>,
    mut cameras: Query<(&mut DistanceFog, &mut Fxaa, Option<&mut Camera>), With<VoxelWorldCamera>>,
    mut ambient: Option<ResMut<GlobalAmbientLight>>,
    mut woodland: Local<(Option<(TerrainCacheKey, bool)>, f32)>,
) {
    let source_key = (TerrainCacheKey::from_frame(&frame), frame.active);
    if woodland.0.as_ref() != Some(&source_key) {
        woodland.1 = sky::woodland_strength(&frame);
        woodland.0 = Some(source_key);
    }
    let cycle = clock.as_ref().and_then(|clock| clock.seconds_of_day())
        .filter(|_| sky::outdoors(&frame))
        .map(day_cycle::DayCycle::at_seconds);
    let unlit_map = lighting.as_ref().is_some_and(|period|
        **period == crystal_render_api::VisualWorldLighting::Dark);
    let horizon = if unlit_map { Color::BLACK }
        else { cycle.map_or_else(|| voxel_clear_color(&frame), |cycle| cycle.horizon) };
    let regional = regional_atmosphere::sample(&frame);
    let woodland_haze = if settings.enabled && !unlit_map {
        // Routes need clear landmarks. Retain only a faint distant veil;
        // dedicated forests keep their authored atmosphere.
        woodland.1 * regional.map_or_else(
            || if frame.map_id.starts_with("Route") { 0.12 } else { 1.0 },
            |regional| regional.haze_scale)
    } else { 0.0 };
    let fog_horizon = if woodland_haze > 0.0 {
        Color::from(horizon.to_linear().mix(
            &sky::woodland_color(horizon).to_linear(), woodland_haze))
    } else { horizon };
    // Atmosphere owns the clear color, including the clock-only updates that
    // do not rebuild terrain or move the camera. Preserve idle change ticks.
    for (_, _, camera) in &mut cameras {
        if let Some(mut camera) = camera {
            if !matches!(camera.clear_color, ClearColorConfig::Custom(color) if color == fog_horizon) {
                camera.clear_color = ClearColorConfig::Custom(fog_horizon);
            }
        }
    }
    if !frame.is_changed() && !profiles.is_changed() && !settings.is_changed()
        && !clock.as_ref().is_some_and(|clock| clock.is_changed())
        && !lighting.as_ref().is_some_and(|period| period.is_changed()) {
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
    let atmosphere = atmosphere.map(|a| (a.start_tiles, a.end_tiles, a.opacity))
        // Match the battle camera's distance-fog treatment: keep the player
        // and nearby scenery clear, then dissolve the finite forest edge.
        .or_else(|| (woodland_haze > 0.0).then_some(
            regional.map_or_else(
                || if frame.map_id.starts_with("Route") { (16.0, 40.0, woodland_haze) }
                    else { (8.0, 22.0, woodland_haze) },
                |regional| (regional.fog_start, regional.fog_end, woodland_haze))));
    let cavern_lit = cave && frame.active && settings.enabled;
    let dark = cavern_lit && frame.tiles.first().is_some_and(|tile| tile.source.tileset_id.as_ref() == "dark_cave");
    // These rooms retain their live cave palette, including Rock Tunnel's
    // authored darkness. Extra dimming hides the floor/rock distinction a
    // second time. Broad fill exposes that pigment; reduced direct light
    // and the source face shading still give the enclosing rock its depth.
    let source_palette_cave = cavern_lit
        && frame.tiles.first().is_some_and(|tile|
            matches!(tile.source.tileset_id.as_ref(), "cave" | "dark_cave"))
        && (matches!(frame.map_id.as_ref(), "DiglettsCave" | "MountMoon"
            | "RockTunnel1F" | "RockTunnelB1F" | "TohjoFalls" | "VictoryRoad")
            // Flash publishes the actual lit Night palette. Its pigment already
            // carries the cave's darkness; do not dim it a second time. A real
            // Dark publication or an unknown producer retains the old energy.
            || lighting.as_ref().is_some_and(|period|
                **period != crystal_render_api::VisualWorldLighting::Dark));
    // Forest uses a fixed night source palette, even at noon. Its paint already
    // carries that cool temperature; a second blue scene light erases foliage,
    // earth and actor separation. Filtered neutral fill preserves that paint.
    let woodland = frame.map_id.as_ref() == "IlexForest"
        && frame.tiles.first().is_some_and(|tile| tile.source.tileset_id.as_ref() == "forest");
    let (ambient_gain, sun_gain) = if woodland { (0.90, 0.90) }
        else if source_palette_cave { (1.0, 0.65) }
        else if dark { (0.36, 0.28) }
        else if cavern_lit { (0.60, 0.48) } else { (1.0, 1.0) };
    // Cave swatches, including Flash's Night palette, already carry their
    // temperature. Neutral illumination preserves that source hue; ordinary
    // outdoor night still receives its actual blue light.
    let burned_timber = frame.map_id.as_ref()=="BurnedTower1F"
        && frame.tiles.first().is_some_and(|tile|tile.source.tileset_id.as_ref()=="tower");
    let period = if cave || woodland || burned_timber { crystal_render_api::VisualWorldLighting::Day }
        else { lighting.as_ref().map(|v| **v).unwrap_or_default() };
    let temperature = match period {
        crystal_render_api::VisualWorldLighting::Morning => [1.18, 1.03, 0.70],
        crystal_render_api::VisualWorldLighting::Day => [1.0, 1.0, 1.0],
        crystal_render_api::VisualWorldLighting::Night => [0.40, 0.55, 2.0],
        crystal_render_api::VisualWorldLighting::Dark => [1.0, 1.0, 1.0],
    };
    // Temperature is the primary palette shift. The latest source comparison
    // also requires a darker night, with enough fill to retain scenery detail.
    let luminance = temperature[0] * 0.2126 + temperature[1] * 0.7152 + temperature[2] * 0.0722;
    let period_color = Color::linear_rgb(temperature[0] / luminance,
        temperature[1] / luminance, temperature[2] / luminance);
    let period_gain = if period == crystal_render_api::VisualWorldLighting::Night { 0.50 } else { 1.0 };
    if let Some(ambient) = ambient.as_mut() {
        let brightness = GlobalAmbientLight::default().brightness * ambient_gain
            * cycle.map_or(period_gain, |cycle| cycle.ambient_gain);
        if ambient.brightness != brightness { ambient.brightness = brightness; }
        let color = cycle.map_or(period_color, |cycle| cycle.ambient);
        if ambient.color != color { ambient.color = color; }
    }
    // CPU rasterizers pay for the entire broad terrain halo again in the
    // shadow pass. Authored contact shading already grounds these scenes;
    // retain dynamic shadows on hardware and use that baked support on known
    // software drivers. This changes rendering only, never world state.
    let software = adapter
        .as_ref()
        .is_some_and(|adapter| software_renderer(&adapter.name));
    let mut shadows = !new_bark_models::supports_map(&frame.map_id) || !software;
    // The miniature world's roofs and leaf cards produce oversized moon
    // silhouettes. Retain authored contact shading at night; daylight keeps
    // actual geometry shadows, with low-energy contrast eased in the shader.
    if cycle.is_some_and(|cycle| cycle.direct_gain <= 0.42) { shadows = false; }
    #[cfg(not(target_arch = "wasm32"))]
    {
        match std::env::var("CRYSTAL_MODELED_SHADOWS").as_deref() {
            Ok("on") => shadows = true,
            Ok("off") => shadows = false,
            _ => {}
        }
    }
    for (mut light, mut cascade, transform) in &mut lights {
        if let Some(mut transform) = transform {
            let direction = cycle.map_or(Vec3::new(180.0, 320.0, 220.0), |cycle| cycle.direction);
            let desired = Transform::from_translation(direction).looking_at(Vec3::ZERO, Vec3::Y);
            if *transform != desired { *transform = desired; }
        }
        let illuminance = 3_000.0 * sun_gain * cycle.map_or(period_gain, |cycle| cycle.direct_gain);
        if light.illuminance != illuminance { light.illuminance = illuminance; }
        let regional_color = if environment_fx::frozen(&frame) {
            Color::srgb(0.68, 0.85, 1.0)
        } else if woodland {
            Color::srgb(1.0, 0.95, 0.82)
        } else if let Some(regional) = regional {
            regional.sun_tint
        } else if matches!(frame.map_id.as_ref(), "Route29" | "NewBarkTown") {
            Color::srgb(1.0, 0.94, 0.82)
        } else if environment_fx::coastal(&frame) {
            Color::srgb(0.89, 0.96, 1.0)
        } else { Color::WHITE };
        let color = if let Some(cycle) = cycle {
            // Keep the authored regional daylight at noon; the clock adds
            // dawn/moon temperature rather than erasing coastal/ice lighting.
            let base = Color::WHITE.mix(&regional_color, cycle.regional_tint_gain).to_linear();
            let tint = cycle.direct.to_linear();
            Color::linear_rgb(base.red * tint.red, base.green * tint.green, base.blue * tint.blue)
        } else if period != crystal_render_api::VisualWorldLighting::Day {
            period_color
        } else { regional_color };
        if light.color != color { light.color = color; }
        if light.shadow_maps_enabled != shadows { light.shadow_maps_enabled = shadows; }
        let desired_cascade = if render_quality::requested().materials() {
            render_quality::shadow_config(frame.viewport_size, settings.camera, render_quality::effective(adapter.as_deref()))
        } else if matches!(frame.map_id.as_ref(), "Route29" | "Route26" | "NewBarkTown"
            | "NationalPark" | "NationalParkBugContest") {
            // Resolve leaves near the player rather than spending a single
            // shadow map on the camera's entire distant terrain halo.
            CascadeShadowConfigBuilder {
                // WebGL supports one cascade; tighten that one around the
                // visible scene instead of requesting unsupported cascades.
                num_cascades: if cfg!(target_arch = "wasm32") { 1 } else { 3 },
                maximum_distance: if cfg!(target_arch = "wasm32") { 1024.0 } else { 2048.0 },
                first_cascade_far_bound: 256.0,
                ..default()
            }.build()
        } else {
            CascadeShadowConfigBuilder {
                num_cascades: 1,
                maximum_distance: 4096.0,
                first_cascade_far_bound: 4096.0,
                ..default()
            }.build()
        };
        // Source animation changes the frame resource, but does not change
        // sun coverage. Preserve Bevy's change tracking on those idle updates.
        if cascade.bounds != desired_cascade.bounds
            || cascade.minimum_distance != desired_cascade.minimum_distance
            || cascade.overlap_proportion != desired_cascade.overlap_proportion
        { *cascade = desired_cascade; }
        let bias = if render_quality::requested().materials() { 1.0 } else { 2.5 };
        if light.shadow_normal_bias != bias { light.shadow_normal_bias = bias; }
    }
    for (mut fog, mut fxaa, _) in &mut cameras {
        let enabled = frame.active
            && settings.enabled
            && (cave || new_bark_models::supports_map(&frame.map_id))
            && !render_quality::effective(adapter.as_deref()).smaa();
        if fxaa.enabled != enabled { fxaa.enabled = enabled; }
        if let Some((start_tiles, end_tiles, opacity)) = atmosphere {
            let pose = settings.camera.pose(frame.viewport_size);
            let target_distance = pose.eye.distance(pose.target);
            let color = fog_horizon.with_alpha(opacity);
            if fog.color != color { fog.color = color; }
            let start = target_distance + start_tiles * frame.tile_size.y;
            let end = target_distance + end_tiles * frame.tile_size.y;
            if !matches!(&fog.falloff, FogFalloff::Linear {start: old_start, end: old_end}
                if *old_start == start && *old_end == end)
            { fog.falloff = FogFalloff::Linear { start, end }; }
        } else {
            if fog.color != Color::NONE { fog.color = Color::NONE; }
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
    if tileset.is_some_and(|tileset|interior::open_air_roof(&frame.map_id,tileset)) {
        Color::srgb(0.72, 0.83, 0.78)
    } else if tileset == Some("game_corner") {
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

// Native small-room builds can start while the previous room's GPU assets
// retire. Keep the memory fence for large/uncertain scenes and synchronous
// browser allocation, including department-store exits into Goldenrod.
fn terrain_retirement_frames(previous: Option<&VisualWorldFrame>, next: &VisualWorldFrame) -> u8 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let small_room=|frame:&VisualWorldFrame| {
            frame.source_map_size_core_tiles.is_some_and(|size|
                size.x>0 && size.y>0 && size.x<=16 && size.y<=16)
                && frame.tiles.first().is_some_and(|tile|
                    interior::has_back_wall(&tile.source.tileset_id))
        };
        if previous.is_some_and(small_room) && small_room(next) {return 0;}
    }
    #[cfg(target_arch = "wasm32")]
    let _=(previous,next);
    2
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
    // Preserve the resource tick until this retained check actually publishes.
    materials: &mut ResMut<Assets<VoxelMaterial>>,
    terrain_entities: &mut Query<(&mut Visibility, &mut Transform), VoxelTerrainFilter>,
) -> Result<TerrainSyncState, TerrainSyncError> {
    let _trace=loading_trace::span("region.sync_terrain", || serde_json::json!({"map":frame.map_id.as_ref()}));
    let mut next_key = TerrainCacheKey::from_frame(frame);
    next_key.profiles_revision = profiles.revision;
    #[cfg(not(target_arch = "wasm32"))]
    if cache.key.as_ref() != Some(&next_key)
        && (cache.painted_stone.is_none() || cache.painted_bedrock.is_none()) {
        // Immutable images may finish while a retired map is being removed or
        // the independent terrain task is still building its CPU geometry.
        builds.painted.start();
    }
    if let Some(publication) = builds.publication.as_mut() {
        if publication.matches(frame, profiles.revision) {
            builds.key = Some(next_key.clone());
        } else {
            terrain_publication::discard(&mut builds.publication, commands);
            builds.key = None;
        }
    }
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
            builds.pending = None;
        }
        builds.key = None;
    }
    // A proven connection advances the logical key while the immutable mesh
    // keeps its original source basis. Subsequent scrolling must retain that
    // geometry during replacement just as it does inside one authored map.
    let retained_logical_map = cache
        .key
        .as_ref()
        .is_some_and(|key| key.map_id == frame.map_id)
        && cache
            .built_frame
            .as_ref()
            .is_some_and(|built| built.connected_source_shift(frame).is_some());
    if cache
        .built_frame
        .as_ref()
        .is_some_and(|built| built.map_id != frame.map_id)
        && !retained_logical_map
        && !retain_lighthouse_room(frame,cache)
        && !(cache.key.as_ref().is_some_and(|key| key.profiles_revision == profiles.revision)
            && retain_connected_terrain(frame, cache))
    {
        let retirement=terrain_retirement_frames(cache.built_frame.as_ref(),frame);
        let _retirement = loading_trace::span("world.window_retire", || {
            serde_json::json!({
                "map": frame.map_id.as_ref(), "origin": frame.grid_origin.to_array(),
                "resident_map": cache.built_frame.as_ref().map(|old| old.map_id.as_ref()),
                "resident_origin": cache.built_frame.as_ref().map(|old| old.grid_origin.to_array()),
                "reason": cache.built_frame.as_ref().and_then(|old|
                    terrain_tracking::connected_frame_shift(old, frame).err()),
            })
        });
        let mut retired = std::mem::take(cache);
        cache.uploads.require_gpu(retired.uploads.gpu_required());
        cache.painted_stone = retired.painted_stone.take();
        cache.painted_bedrock = retired.painted_bedrock.take();
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
        terrain_disposal::retire(retired);
        builds.retirement_frames = retirement;
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
    if !builds.defer_to_world_preload
        && should_start_terrain_build(cache.key.as_ref(), builds.key.as_ref(), &next_key) {
        let _coverage = loading_trace::span("world.coverage_miss", || serde_json::json!({
            "map": frame.map_id.as_ref(),
            "origin": frame.grid_origin.to_array(),
            "resident_map": cache.built_frame.as_ref().map(|old| old.map_id.as_ref()),
            "resident_origin": cache.built_frame.as_ref().map(|old| old.grid_origin.to_array()),
            "connected_rejection": cache.built_frame.as_ref().filter(|old| old.map_id != frame.map_id)
                .and_then(|old| terrain_tracking::connected_frame_shift(old, frame).err()),
            "roots": ([cache.instances_root, cache.textured_entity, cache.solid_entity,
                cache.animated_textured_entity, cache.animated_solid_entity]
                .into_iter().flatten().map(|entity| entity.to_bits()).collect::<Vec<_>>()),
        }));
        #[cfg(all(target_arch = "wasm32", feature = "terrain-worker-browser"))]
        {
            let availability = terrain_worker::browser::availability();
            if availability == terrain_worker::browser::Availability::Initializing
                || (builds.speculative
                    && availability != terrain_worker::browser::Availability::Ready)
            {
                // Disabled or failed workers leave speculation deferred. Only
                // foreground demand may recover by meshing on the frame thread.
                return Ok(TerrainSyncState::Pending);
            }
        }
        #[cfg(all(target_arch = "wasm32", feature = "terrain-worker-browser"))]
        if terrain_worker::browser::busy() {
            let _scheduling = loading_trace::span("worker.busy", || {
                serde_json::json!({"map": frame.map_id.as_ref(), "speculative": builds.speculative})
            });
            if builds.speculative {
                // The other preload lane owns the single worker. Defer before
                // copying the atlas or serializing another request.
                return Ok(TerrainSyncState::Pending);
            }
            // A doorway or exhausted foreground window has priority. The old
            // ticket will observe supersession without synchronous fallback.
            terrain_worker::browser::cancel_pending();
            if terrain_worker::browser::busy() {
                // A failed pool lane can reduce capacity before its old
                // tickets have been polled. Defer instead of turning that
                // scheduling pressure into a synchronous recovery build.
                return Ok(TerrainSyncState::Pending);
            }
        }
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
        #[cfg(not(target_arch = "wasm32"))]
        let build = async move {
            #[cfg(feature = "operation-trace")]
            let _span = bevy::log::info_span!("crystal_terrain_build").entered();
            // SurfaceMeshData -> Bevy Mesh conversion walks and moves every
            // vertex/index buffer. Keep that work on the compute task too;
            // doing it when polling the completed build caused a deterministic
            // 30-45 ms main-thread hitch several seconds into 2.5D movement.
            let terrain = mesh::begin_runtime_terrain_mesh_with_profiles(
                &build_frame,
                &samples,
                &profile_document,
            )
            .map(|terrain| prepare_runtime_terrain(terrain.into_streamed(), &build_frame));
            let water = (terrain.is_ok() && !std::env::var("CRYSTAL_WATER_IMAGE_WARMUP")
                .is_ok_and(|value| matches!(value.as_str(), "0" | "off" | "false")))
                .then(|| water_surface::PreparedImages::for_frame(&build_frame));
            TerrainBuildResult {
                water,
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
            #[cfg(feature = "terrain-worker-browser")]
            let ticket = if terrain_worker::browser::availability() == terrain_worker::browser::Availability::Ready {
                match terrain_worker::browser::submit(&build_frame, &samples, &profile_document) {
                    Ok(ticket) => Some(ticket),
                    Err(error) => {
                        terrain_worker::browser::fallback(&error);
                        if builds.speculative {
                            builds.key = None;
                            return Ok(TerrainSyncState::Pending);
                        }
                        None
                    }
                }
            } else { None };
            #[cfg(feature = "terrain-worker-browser")]
            let work = if let Some(ticket) = ticket {
                BrowserTerrainWork::Worker { ticket, samples }
            } else {
                BrowserTerrainWork::Synchronous(mesh::begin_runtime_terrain_mesh_with_profiles(
                    &build_frame, &samples, &profile_document))
            };
            #[cfg(not(feature = "terrain-worker-browser"))]
            let work = BrowserTerrainWork::Synchronous(mesh::begin_runtime_terrain_mesh_with_profiles(
                &build_frame, &samples, &profile_document));
            builds.pending = Some(BrowserTerrainBuild {
                key: build_key, frame: build_frame, source_texture,
                profiles: profile_document, work: Some(work),
            });
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    if (cache.painted_stone.is_none() || cache.painted_bedrock.is_none()) && builds.painted.started() {
        let Some((stone, bedrock)) = builds.painted.poll(images) else {
            // Keep the completed geometry owned by its task until every
            // publication prerequisite is ready. Never decode on main here.
            return Ok(TerrainSyncState::Pending);
        };
        cache.painted_stone = Some(stone);
        cache.painted_bedrock = Some(bedrock);
    }

    #[cfg(not(target_arch = "wasm32"))]
    let completed = builds
        .task
        .as_mut()
        .and_then(|task| future::block_on(future::poll_once(task)));
    #[cfg(target_arch = "wasm32")]
    let completed = if let Some(mut pending) = builds.pending.take() {
        match pending.work.take().expect("browser build owns its work") {
            BrowserTerrainWork::Synchronous(terrain) => {
                let terrain = terrain.map(|terrain|
                    prepare_runtime_terrain(terrain.into_streamed(), &pending.frame));
                Some(pending.complete(terrain))
            }
            #[cfg(feature = "terrain-worker-browser")]
            BrowserTerrainWork::Worker { ticket, samples } => {
                match terrain_worker::browser::poll(&ticket) {
                    terrain_worker::browser::Poll::Pending => {
                        pending.work = Some(BrowserTerrainWork::Worker { ticket, samples });
                        builds.pending = Some(pending);
                        None
                    }
                    terrain_worker::browser::Poll::Ready(terrain) => Some(pending.complete(Ok(terrain))),
                    terrain_worker::browser::Poll::Superseded => {
                        builds.key = None;
                        None
                    }
                    terrain_worker::browser::Poll::Failed(error) => {
                        terrain_worker::browser::fallback(&error);
                        if builds.speculative {
                            // A failed lookahead must never move meshing onto
                            // the frame thread. Foreground recovery owns fallback.
                            builds.key = None;
                            return Ok(TerrainSyncState::Pending);
                        }
                        let _trace = loading_trace::span("worker.synchronous_fallback", || serde_json::json!({"error":error}));
                        let terrain = mesh::begin_runtime_terrain_mesh_with_profiles(
                            &pending.frame, &samples, &pending.profiles)
                            .map(|terrain| prepare_runtime_terrain(terrain.into_streamed(), &pending.frame));
                        Some(pending.complete(terrain))
                    }
                }
            }
        }
    } else { None };
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
            #[cfg(not(target_arch = "wasm32"))]
            let water = completed.water;
            #[cfg(target_arch = "wasm32")]
            let water = None;
            let publishing = !builds.speculative && cache.uploads.gpu_required();
            let target = if publishing {
                builds.publication = Some(terrain_publication::Pending::new(cache));
                builds.key = Some(next_key.clone());
                &mut builds.publication.as_mut().expect("new publication is present").cache
            } else {
                &mut *cache
            };
            apply_built_terrain(
                &completed.frame,
                next_key.clone(),
                terrain,
                commands,
                target,
                meshes,
                materials,
                images,
                water,
                builds.speculative || publishing,
            )?;
            target.built_source_texture = Some(completed.source_texture);
            target.built_profiles = Some(completed.profiles);
            if !publishing { bind_preloaded_compositor(frame, cache); }
            if profile_changed {
                println!(
                    "geometry profile mesh applied: revision {}",
                    next_key.profiles_revision
                );
            }
        } else {
            terrain_disposal::retire(completed);
        }
    }

    if terrain_publication::advance(&mut builds.publication, cache, meshes, commands) {
        builds.key = None;
        cache.key = Some(next_key.clone());
        bind_preloaded_compositor(frame, cache);
    }
    cache.uploads.advance(meshes);
    let ready = cache.key.as_ref() == Some(&next_key) && cache.uploads.ready();
    if ready {
        if let Some(built) = &mut cache.built_frame {
            if built.terrain_revision != frame.terrain_revision {
                let geometry_changed = terrain_tracking::refresh_animation(built, frame, images)
                    .map_err(|()| TerrainSyncError::SourceTextureUnavailable)?;
                if geometry_changed {
                    let (textured, solid) = mesh::build_animated_scene(built, images, &cache.built_animated_pillars,
                        cache.built_train_doors.then(||mesh::station_train_door_openness(frame)))
                        .map_err(TerrainSyncError::Mesh)?;
                    cache.built_animated_bounds = [textured.get_aabb(), solid.get_aabb()];
                    update_mesh_asset(meshes, &mut cache.animated_textured_mesh, retain_animated_mesh_cpu(textured));
                    update_mesh_asset(meshes, &mut cache.animated_solid_mesh, retain_animated_mesh_cpu(solid));
                }
            }
        }
    }
    if !ready&&retain_lighthouse_room(frame,cache) {return Ok(TerrainSyncState::Pending);}
    if cache.footing_origin != Some(frame.grid_origin)
        || cache.footing_map.as_ref() != Some(&frame.map_id) {
        if let Some(built) = &cache.built_frame {
            terrain_tracking::align_footings(
                built,
                frame,
                &cache.built_footing_heights,
                &mut cache.footing_heights,
            );
            cache.footing_origin = Some(frame.grid_origin);
            cache.footing_map = Some(frame.map_id.clone());
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
    prepared_water: Option<water_surface::PreparedImages>,
    staged: bool,
) -> Result<(), TerrainSyncError> {
    let _trace = loading_trace::span("region.publish_assets", || serde_json::json!({"map": frame.map_id.as_ref(), "origin": frame.grid_origin.to_array()}));
    let existing_meshes = staged.then(|| meshes.ids().collect::<std::collections::HashSet<_>>());
    let publish_setup = loading_trace::span("region.publish_setup", || serde_json::json!({}));
    let gpu_required = cache.uploads.gpu_required();
    cache.uploads = terrain_upload::Pending::default();
    cache.uploads.require_gpu(gpu_required);
    cache.city_emitters = terrain.city_emitters;
    cache.footing_origin = None;
    cache.footing_map = None;
    cache.connected_proof = None;
    cache.groves = terrain.streamed_groves.map(|source| grove_stream::Stream::new(source, cache.forest_library.clone()));
    cache.built_frame = Some(frame.clone());
    cache.built_profiles = None;
    cache.built_source_texture = Some(frame.map_texture.clone());
    cache.verified_source_texture = None;
    cache.built_animated_bounds = [terrain.animated_textured_mesh.get_aabb(),
        terrain.animated_solid_mesh.get_aabb()];
    cache.built_footing_heights = terrain.footing_heights;
    cache.built_animated_pillars = terrain.animated_pillars;
    cache.built_train_doors = terrain.train_doors;
    cache.built_train_open = None;

    drop(publish_setup);
    let publish_animated = loading_trace::span("region.publish_animated_meshes", || serde_json::json!({}));
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
    drop(publish_animated);
    let publish_materials = loading_trace::span("region.publish_materials_and_images", || serde_json::json!({}));
    let textured_material_handle = if let Some(handle) = cache.textured_material.as_ref() {
        sync_terrain_texture(materials, handle, &frame.map_texture)?;
        handle.clone()
    } else {
        let handle = materials.add(textured_terrain_material(frame.map_texture.clone()));
        cache.textured_material = Some(handle.clone());
        handle
    };
    let water_surface::PreparedImages { mask, shore } = prepared_water
        .unwrap_or_else(|| water_surface::PreparedImages::for_frame(frame));
    let water_mask = mask.map(|mask| add_terrain_image(images, mask, "water_mask"));
    let water_shore = shore.map(|shore| add_terrain_image(images, shore, "water_shore"));
    if let Some(mut material) = materials.get_mut(&textured_material_handle) {
        material.extension.water_enabled = if water_mask.is_some() {
            cave_water::registration(frame)
        } else {
            Vec4::ZERO
        };
        material.extension.water_mask = water_mask;
        material.extension.water_shore = water_shore;
        material.extension.water_bounds = water_surface::bounds(frame);
        material.extension.water_style = water_surface::style(frame);
        material.extension.water_phase = Vec4::ZERO;
        material.extension.water_motion.y = material.extension.water_motion.x;
        material.extension.water_motion.z = 0.0;
        material.extension.water_contact = water_surface::contact(frame);
        let palette = {
            let _trace = loading_trace::span("region.prepare_water_palette", || serde_json::json!({}));
            cave_water::native_palette(frame, images).unwrap_or([Vec4::ZERO; 2])
        };
        material.extension.water_deep=palette[0];
        material.extension.water_foam=palette[1];
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

    if cache.painted_stone.is_none() {
        cache.painted_stone = Some(add_terrain_image(images,
            painted_stone::image(), "painted_stone"));
    }
    if cache.painted_bedrock.is_none() {
        cache.painted_bedrock = Some(add_terrain_image(images,
            painted_stone::bedrock_image(), "painted_bedrock"));
    }
    let painted_stone = cache.painted_stone.as_ref().unwrap().clone();
    let painted_bedrock = cache.painted_bedrock.as_ref().unwrap().clone();
    if let Some(mut material) = materials.get_mut(&solid_material_handle) {
        material.extension.painted_stone = Some(painted_stone);
        material.extension.painted_bedrock = Some(painted_bedrock);
        let mut registration = cave_water::registration(frame);
        registration.x = 0.;
        material.extension.water_enabled = registration;
    }
    let cave_reveal = if interior_cutaway::is_cave(frame) {
        let dark = frame.tiles.first().is_some_and(|tile| tile.source.tileset_id.as_ref() == "dark_cave")
            && std::env::var_os("CRYSTAL_CAVE_GEOMETRY_REVIEW").is_none();
        // The reveal policy also reaches untextured rock materials, which do
        // not carry the live-water registration used by the surface shader.
        Vec4::new(if frame.map_id.as_ref()=="TohjoFalls" {2.0} else {1.0},
            if dark { 1.0 } else { 0.0 }, 0.0, 0.0)
    } else if cave_water::outdoor(frame.map_id.as_ref()) && frame.tiles.first().is_some_and(|tile|
        matches!(tile.source.tileset_id.as_ref(), "johto" | "johto_modern" | "kanto"
            | "forest" | "park" | "battle_tower_outside" | "port")) {
        // The existing environment vector's Z channel selects exterior
        // daylight; value2 gives ordinary towns stronger cast-shade grounding.
        // X/Y retain cave reveal and darkness ownership; forest stays value1.
        let town = frame.tiles.first().is_some_and(|tile|
            scenery_finish::town_shadow_fill(&frame.map_id, &tile.source.tileset_id));
        Vec4::new(0., 0., if town { 2. } else { 1. }, 0.)
    } else { Vec4::ZERO };
    for handle in [&textured_material_handle, &solid_material_handle] {
        if let Some(mut material) = materials.get_mut(handle) {
            material.extension.cave_reveal = cave_reveal;
        }
    }
    let animated_solid_material_handle = flower::animated_material(
        &mut cache.animated_solid_material, &solid_material_handle, materials,
    );
    let grass_material_handle = grass_rustle::material(
        &mut cache.grass_material, &solid_material_handle, materials,
    );

    drop(publish_materials);
    let publish_instances = loading_trace::span("region.publish_instances_and_scenery", || serde_json::json!({}));
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
        material.extension.cave_reveal = Vec4::new(cave_reveal.x.max(1.), cave_reveal.y, 0., 1.);
        material.extension.painted_bedrock = cache.painted_bedrock.clone();
        let child = commands.spawn(((Mesh3d(meshes.add(mesh)),
MeshMaterial3d(materials.add(material)),), RenderLayers::layer(VOXEL_RENDER_LAYER), encounter_terrain::SyntheticTerrainApron)).id();
        commands.entity(root).add_child(child);
    }
    if let Some((mesh, source)) = terrain.background {
        use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
        let mut image = {
            let _trace = loading_trace::span("region.prepare_background_image", || serde_json::json!({}));
            images.get(&source).ok_or(TerrainSyncError::SourceTextureUnavailable)?.clone()
        };
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            ..ImageSamplerDescriptor::nearest()
        });
        image.asset_usage = bevy::asset::RenderAssetUsages::RENDER_WORLD;
        let mut material = textured_terrain_material(add_terrain_image(images, image, "background"));
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
            let mut material = textured_terrain_material(add_terrain_image(images, texture.image(), "scenery"));
            material.extension.cave_reveal = cave_reveal;
            material.base.alpha_mode = texture.alpha_mode();
            material.base.double_sided = texture.double_sided;
            if texture.double_sided { material.base.cull_mode = None; }
            materials.add(material)
        }).clone();
        let mesh = meshes.add(batch.mesh);
        let mut child = commands.spawn(((Mesh3d(mesh.clone()),
MeshMaterial3d(handle),), RenderLayers::layer(VOXEL_RENDER_LAYER), batch.authentic_domain));
        if let Some(bounds) = batch.coverage { child.insert(terrain_batches::TerrainBatchCoverage { mesh: mesh.id(), sight_bounds: batch.sight_coverage.unwrap_or_else(|| bounds.clone()), grass_bounds: batch.grass_coverage.unwrap_or_default(), bounds }); }
        if let Some(bounds) = batch.bounds { child.insert(bounds); }
        let child=child.id();
        commands.entity(root).add_child(child);
    }
    for (groups, texture, domain) in terrain.scenery_fades {
        let key = std::sync::Arc::as_ptr(&texture) as usize;
        let handle = scenery_materials.entry(key).or_insert_with(|| {
            let mut material = textured_terrain_material(add_terrain_image(images, texture.image(), "scenery"));
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

    drop(publish_instances);
    let _publish_batches = loading_trace::span("region.publish_static_batches", || serde_json::json!({"textured":terrain.textured_meshes.len(),"solid":terrain.solid_meshes.len()}));
    if cache.animated_textured_entity.is_none() {
        cache.animated_textured_entity = Some(spawn_terrain_entity(
            commands,
            animated_textured,
            textured_material_handle.clone(),
            frame,
        ));
    }
    if let Some(entity) = cache.animated_solid_entity {
        commands.entity(entity).insert(MeshMaterial3d(animated_solid_material_handle));
    } else {
        cache.animated_solid_entity = Some(spawn_terrain_entity(
            commands,
            animated_solid,
            animated_solid_material_handle,
            frame,
        ));
    }
    replace_static_terrain_batches(
        commands,
        meshes,
        &mut cache.textured_entity,
        terrain.textured_meshes,
        textured_material_handle,
        None,
        frame,
        staged.then_some(&mut cache.uploads),
    );
    replace_static_terrain_batches(
        commands,
        meshes,
        &mut cache.solid_entity,
        terrain.solid_meshes,
        solid_material_handle,
        Some(grass_material_handle),
        frame,
        staged.then_some(&mut cache.uploads),
    );

    if let Some(existing) = existing_meshes {
        cache.uploads.capture_added(meshes, &existing);
        let receipt = cache.uploads.seal();
        commands.entity(root).insert(receipt);
    }

    cache.key = Some(key);
    Ok(())
}

// Image preparation remains separate from Assets insertion in the waterfall.
// This helper receives the already-owned CPU result and changes no sampler or pixels.
fn add_terrain_image(images: &mut Assets<Image>, image: Image, kind: &'static str) -> Handle<Image> {
    let _trace = loading_trace::span("region.publish_image_asset", || serde_json::json!({
        "kind":kind,"width":image.width(),"height":image.height(),
        "bytes":image.data.as_ref().map_or(0, Vec::len),
    }));
    images.add(image)
}

/// Static roots retain the same scrolling/visibility contract as the previous
/// combined meshes. Only their children draw. Replacing the complete hierarchy
/// also replaces every AABB, so no bound from an earlier map/revision survives.
#[allow(clippy::too_many_arguments)]
fn replace_static_terrain_batches(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    cached_root: &mut Option<Entity>,
    batches: Vec<terrain_batches::PreparedBatch>,
    material: Handle<VoxelMaterial>,
    grass_material: Option<Handle<VoxelMaterial>>,
    frame: &VisualWorldFrame,
    mut uploads: Option<&mut terrain_upload::Pending>,
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
        rooted_grass,
        bounds,
        authentic_domain,
        coverage,
        sight_coverage,
        grass_coverage,
    } in batches {
        let batch_material = grass_material.as_ref().filter(|_| rooted_grass)
            .unwrap_or(&material).clone();
        let mesh = if let Some(uploads) = uploads.as_deref_mut() {
            uploads.enqueue(meshes, mesh)
        } else {
            meshes.add(mesh)
        };
        let mut child = commands.spawn((
            (Mesh3d(mesh.clone()),
MeshMaterial3d(batch_material),
Visibility::Inherited,),
            RenderLayers::layer(VOXEL_RENDER_LAYER),
            authentic_domain,
        ));
        if let Some(bounds) = coverage {
            child.insert(terrain_batches::TerrainBatchCoverage { mesh: mesh.id(), sight_bounds: sight_coverage.unwrap_or_else(|| bounds.clone()), grass_bounds: grass_coverage.unwrap_or_default(), bounds });
        }
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

const GRASS_VERTEX_SHADER_HANDLE: Handle<Shader> =
    Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(0x2fbd_8b33_1fe1_487b_b4e4_c368af150abb), std::marker::PhantomData);

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
#[bind_group_data(VoxelSurfaceKey)]
// Pack surface values into one binding to fit WebGL2 fragment uniform limits.
struct VoxelSurface {
    #[uniform(100)]
    cutaway: interior_cutaway::CutawayUniform,
    #[texture(101)]
    #[sampler(102)]
    water_mask: Option<Handle<Image>>,
    #[uniform(100)]
    water_enabled: Vec4,
    #[uniform(100)]
    cave_reveal: Vec4,
    #[texture(105)]
    #[sampler(106)]
    painted_stone: Option<Handle<Image>>,
    #[texture(107)]
    #[sampler(108)]
    painted_bedrock: Option<Handle<Image>>,
    #[uniform(100)]
    water_deep: Vec4,
    #[uniform(100)]
    water_foam: Vec4,
    #[texture(103)]
    #[sampler(104)]
    water_shore: Option<Handle<Image>>,
    #[uniform(100)]
    water_bounds: Vec4,
    #[uniform(100)]
    water_style: Vec4,
    #[uniform(100)]
    water_phase: Vec4,
    #[uniform(100)]
    water_motion: Vec4,
    #[uniform(100)]
    water_contact: Vec4,
    #[uniform(100)]
    cave_exits: [Vec4; 4],
    #[uniform(100)]
    cave_sky: Vec4,
    #[uniform(100)]
    city_night: Vec4,
    #[uniform(100)]
    city_windows: [Vec4;city_lights::MAX_LIGHTS],
    #[uniform(100)]
    city_window_planes: [Vec4;city_lights::MAX_LIGHTS],
    /// Battle-only clearance around the complete bodies; zero in the overworld.
    #[uniform(100)]
    battle_clearance: [Vec4; 2],
    #[uniform(100)]
    battle_clearance_shape: [Vec4; 2],
    /// World-space foot contact (XZ center, ground Y, footprint radius).
    #[uniform(100)]
    battle_grass_contact: [Vec4; 2],
    /// Footprint X axis in world XZ, then local contact radii.
    #[uniform(100)]
    battle_contact_shape: [Vec4; 2],
    #[texture(109)]
    #[sampler(110)]
    battle_grass_mask: Option<Handle<Image>>,
    #[uniform(100)]
    battle_grass_mask_info: Vec4,
    #[uniform(111)]
    grass_rustle: grass_rustle::GrassUniform,
    // CPU-only constructor proof; no new GPU uniform/binding. Current opacity
    // and mesh attributes are checked again before any camera depth write.
    depth_prepass_palette: bool,
    // CPU-only pipeline choice for closed authored layered-alpha objects.
    smooth_depth_fade: bool,
}
// Opaque palette surfaces must not carry the fragment-discard instructions
// used only by their separate fading material. In particular, dense crowns
// can then use the GPU's ordinary early depth rejection. Continuous fade
// animation retains its pipeline.
static LEGACY_SURFACE_DISCARD: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
static TREE_DEPTH_PREPASS: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

/// Startup-only selective-tree depth QA. Defaults off while exact image and
/// hardware timing comparisons establish whether the extra pass is worthwhile.
pub fn configure_tree_depth_prepass_diagnostics(enabled: bool) -> Result<(), String> {
    TREE_DEPTH_PREPASS.set(enabled)
        .map_err(|_| "tree depth prepass diagnostics already configured".to_owned())
}


/// Startup-only QA control: retain the original uniform-guarded discard
/// instructions for a same-binary GPU timing and visual comparison.
pub fn configure_surface_discard_diagnostics(legacy: bool) -> Result<(), String> {
    LEGACY_SURFACE_DISCARD.set(legacy)
        .map_err(|_| "surface discard diagnostics already configured".to_owned())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct VoxelSurfaceKey {
    source_alpha_cutoff: bool,
    coverage_fade: bool,
    depth_prepass_palette: bool,
    smooth_depth_fade: bool,
    material_lighting: bool,
}
impl From<&VoxelSurface> for VoxelSurfaceKey {
    fn from(surface: &VoxelSurface) -> Self {
        Self::for_mode(surface, LEGACY_SURFACE_DISCARD.get().copied().unwrap_or(false))
    }
}
impl VoxelSurfaceKey {
    fn for_mode(surface: &VoxelSurface, legacy: bool) -> Self {
        Self {
            source_alpha_cutoff: legacy || surface.cutaway.fade.w > 0.0,
            coverage_fade: legacy || surface.cutaway.fade.w != 0.0,
            depth_prepass_palette: TREE_DEPTH_PREPASS.get().copied().unwrap_or(false)
                && surface.depth_prepass_palette && surface.cutaway.fade.x == 0.0,
            smooth_depth_fade: surface.smooth_depth_fade,
            material_lighting: render_quality::requested().materials(),
        }
    }
}
impl MaterialExtension for VoxelSurface {
    fn vertex_shader() -> ShaderRef {
        GRASS_VERTEX_SHADER_HANDLE.into()
    }

    fn prepass_vertex_shader() -> ShaderRef {
        GRASS_VERTEX_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> ShaderRef {
        VOXEL_SURFACE_SHADER_HANDLE.into()
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        key: bevy::pbr::MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let normal_prepass = key.mesh_key.contains(bevy::pbr::MeshPipelineKey::NORMAL_PREPASS)
            && descriptor.vertex.shader_defs.iter().any(|def| matches!(def,
                bevy::shader::ShaderDefVal::Bool(name, true) if name == "PREPASS_PIPELINE"));
        if let Some(fragment) = descriptor.fragment.as_mut() {
            // StandardMaterial intentionally omits normals for unlit paint.
            // Our contact pass needs actual surface normals and the same
            // source cutouts/camera apertures as the visible pass. Shadow and
            // depth-only pipelines retain their existing shaders.
            if normal_prepass { fragment.shader = VOXEL_SURFACE_SHADER_HANDLE; }
            let hdr = fragment.targets.iter().flatten().any(|target|
                target.format == bevy::render::render_resource::TextureFormat::Rgba16Float);
            fragment.shader_defs.push(bevy::shader::ShaderDefVal::Bool("VOXEL_HDR".into(), hdr));
            fragment.shader_defs.push(bevy::shader::ShaderDefVal::Bool(
                "VOXEL_MATERIAL_LIGHTING".into(), key.bind_group_data.material_lighting));
            fragment.shader_defs.push(bevy::shader::ShaderDefVal::Bool(
                "VOXEL_SOURCE_ALPHA_CUTOFF".into(), key.bind_group_data.source_alpha_cutoff));
            fragment.shader_defs.push(bevy::shader::ShaderDefVal::Bool(
                "VOXEL_COVERAGE_FADE".into(), key.bind_group_data.coverage_fade));
            fragment.shader_defs.push(bevy::shader::ShaderDefVal::Bool(
                "VOXEL_SMOOTH_DEPTH_FADE".into(), key.bind_group_data.smooth_depth_fade));
        }
        configure_tree_depth_prepass_pipeline(
            descriptor, key.mesh_key, key.bind_group_data, layout,
            TREE_DEPTH_PREPASS.get().copied().unwrap_or(false),
        );
        configure_smooth_depth_fade_pipeline(descriptor,key.mesh_key,key.bind_group_data.smooth_depth_fade);
        if std::env::var_os("CRYSTAL_VOXEL_PIPELINE_TRACE").is_some() {
            eprintln!("voxel pipeline: key={:?} fragment_defs={:?}", key.mesh_key,
                descriptor.fragment.as_ref().map(|fragment| &fragment.shader_defs));
        }
        Ok(())
    }
}

fn depth_prepass_palette_base(base: &StandardMaterial) -> bool {
    base.unlit && matches!(base.alpha_mode, AlphaMode::Opaque)
        && base.base_color_texture.is_none() && base.normal_map_texture.is_none()
        && base.depth_map.is_none() && base.emissive_texture.is_none()
        && base.metallic_roughness_texture.is_none() && base.occlusion_texture.is_none()
}

fn voxel_material(base: StandardMaterial) -> VoxelMaterial {
    let extension = VoxelSurface {
        depth_prepass_palette: depth_prepass_palette_base(&base),
        ..default()
    };
    ExtendedMaterial { base, extension }
}

/// Closed layered alpha keeps smooth opacity while rejecting incorrectly
/// ordered hidden faces. Opaque, mask, shadow and camera-prepass paths retain
/// their original depth contract; this choice adds no shader binding/uniform.
fn configure_smooth_depth_fade_pipeline(descriptor:&mut RenderPipelineDescriptor,
    mesh_key:bevy::pbr::MeshPipelineKey,enabled:bool) {
    use bevy::pbr::MeshPipelineKey as Key;
    let prepass=descriptor.vertex.shader_defs.iter().any(|def|matches!(def,
        bevy::shader::ShaderDefVal::Bool(name,true) if name=="PREPASS_PIPELINE"));
    if enabled&&!prepass&&mesh_key.intersection(Key::BLEND_RESERVED_BITS)==Key::BLEND_ALPHA {
        if let Some(depth)=descriptor.depth_stencil.as_mut(){depth.depth_write_enabled=Some(true);}
    }
}

/// Prepass and main forward pipelines both carry DEPTH_PREPASS. Only Bevy's
/// actual prepass descriptor has PREPASS_PIPELINE; shadow descriptors also
/// carry explicit VIEW_PROJECTION bits (directional also has UNCLIPPED_DEPTH_ORTHO).
/// The camera-prepass view key carries no projection bits, even for perspective
/// cameras; suppress depth only in that ordinary camera pass.
fn configure_tree_depth_prepass_pipeline(
    descriptor: &mut RenderPipelineDescriptor,
    mesh_key: bevy::pbr::MeshPipelineKey,
    surface: VoxelSurfaceKey,
    layout: &MeshVertexBufferLayoutRef,
    enabled: bool,
) {
    use bevy::pbr::MeshPipelineKey as Key;
    if !enabled || !mesh_key.contains(Key::DEPTH_PREPASS)
        || mesh_key.intersects(Key::VIEW_PROJECTION_RESERVED_BITS | Key::UNCLIPPED_DEPTH_ORTHO | Key::NORMAL_PREPASS
            | Key::MOTION_VECTOR_PREPASS | Key::DEFERRED_PREPASS)
        || !descriptor.vertex.shader_defs.iter().any(|def| matches!(def,
            bevy::shader::ShaderDefVal::Bool(name, true) if name == "PREPASS_PIPELINE")) {
        return;
    }
    // Runtime tree prototypes omit UVs only after proving all source UVs are
    // zero and the owning palette material cannot consume them. The main
    // vertex shader bends only UV-tagged grass; these static meshes use the
    // same position_world_to_clip expression as Bevy's stock prepass vertex.
    // UV-bearing cave walls/grass and every fading/masked object write no depth
    // here, so their forward-only apertures and alpha retain full control.
    let eligible = surface.depth_prepass_palette
        && !surface.source_alpha_cutoff && !surface.coverage_fade
        && mesh_key.intersection(Key::BLEND_RESERVED_BITS) == Key::BLEND_OPAQUE
        && !mesh_key.intersects(Key::MAY_DISCARD | Key::MORPH_TARGETS | Key::VISIBILITY_RANGE_DITHER)
        && layout.0.contains(Mesh::ATTRIBUTE_POSITION)
        && !layout.0.contains(Mesh::ATTRIBUTE_UV_0)
        && !layout.0.contains(Mesh::ATTRIBUTE_UV_1)
        && !layout.0.contains(Mesh::ATTRIBUTE_JOINT_INDEX)
        && !layout.0.contains(Mesh::ATTRIBUTE_JOINT_WEIGHT);
    if !eligible {
        if let Some(depth) = descriptor.depth_stencil.as_mut() {
            depth.depth_write_enabled = Some(false);
            depth.depth_compare = Some(CompareFunction::Never);
        }
    }
}

/// Publish the resolved player support/capsule to the whole-object fade system.
/// It uses the current propagated camera and cached authored triangles.
fn sync_interior_cutaway(
    frame: Res<VisualWorldFrame>,
    lighting: Option<Res<crystal_render_api::VisualWorldLighting>>,
    status: Res<VoxelViewStatus>,
    cache: Res<TerrainRevisionCache>,
    images: Res<Assets<Image>>,
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
                let foot = footing::actor_ground_point(&frame, actor);
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
    let dark = lighting.as_ref().is_some_and(|period|
        **period == crystal_render_api::VisualWorldLighting::Dark);
    let exits = if status.active && !dark { cave_lighting::exits(&frame) } else { [Vec4::ZERO; 4] };
    let sky = if dark { Vec4::ZERO } else { cave_lighting::sky_tint(&frame, &images) };
    // Update terrain and actors together, so entering the daylight spill also
    // lights the trainer. Exits are cleared on every non-cave frame.
    let light_changed: Vec<_> = materials.iter().filter_map(|(id, material)|
        (material.extension.cave_exits != exits || material.extension.cave_sky != sky
            || material.extension.cave_reveal.y != if dark { 1.0 } else { 0.0 }).then_some(id)).collect();
    for id in light_changed {
        if let Some(mut material) = materials.get_mut(id) {
            material.extension.cave_exits = exits;
            material.extension.cave_sky = sky;
            material.extension.cave_reveal.y = if dark { 1.0 } else { 0.0 };
        }
    }
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
    for handle in [Some(handle), cache.animated_solid_material.as_ref(), cache.grass_material.as_ref()].into_iter().flatten() {
        if materials.get(handle).is_some_and(|material| material.extension.cutaway != next) {
            if let Some(mut material) = materials.get_mut(handle) {
                material.extension.cutaway = next;
            }
        }
    }
}

#[cfg(test)]
mod surf_defaults_tests;

#[allow(clippy::too_many_arguments)]
fn sync_actor_cards(
    frame: &VisualWorldFrame,
    footing_heights: &[f32],
    pose: VoxelCameraPose,
    healing_allowed:bool,
    actor_quad: &Handle<Mesh>,
    commands: &mut Commands,
    cache: &mut ActorIdCache,
    images: &Assets<Image>,
    materials: &mut ResMut<Assets<VoxelMaterial>>,
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
        // Surf has one 3D presentation: the trainer on the actual user's
        // canonical body. Neither the source avatar nor a mount sprite is a
        // substitute while identity/artwork is unavailable.
        if new_bark_actors::is_surf_avatar(actor) || actor.id == VisualActorId::SurfMount { continue; }
        // Native blades consume this event; the source sprite belongs to 2D.
        if grass_rustle::is_rustle(actor) { continue; }
        if healing_effects::is_healing(actor)&&!healing_allowed {continue;}
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
    if !status.active || settings.camera.first_person || new_bark_actors::has_modeled_player(&frame)
        || frame.actors.iter().any(new_bark_actors::is_surf_avatar) {
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
        .filter(|actor| !new_bark_actors::is_surf_avatar(actor))
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
    materials: &mut ResMut<Assets<VoxelMaterial>>,
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
    if healing_effects::is_healing(actor) {
        return healing_effects::transform(frame,actor);
    }
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

    include!("world_stream_camera_tests.rs");

    #[test]
    fn rooted_sea_stone_registration_tracks_retained_grid_without_animation_steps() {
        let built = VisualWorldFrame { active: true, map_id: "Route19".into(),
            tile_size: Vec2::splat(32.), grid_size: UVec2::new(44, 40),
            grid_origin: IVec2::new(-12, 20), center: Vec2::new(48., -64.), ..default() };
        let mut materials = Assets::<VoxelMaterial>::default();
        let handle = materials.add(solid_terrain_material());
        let mut app = App::new();
        app.insert_resource(Time::<()>::default()).insert_resource(built.clone())
            .insert_resource(VoxelViewStatus { active: true, ..default() })
            .insert_resource(TerrainRevisionCache { built_frame: Some(built.clone()),
                solid_material: Some(handle.clone()), ..default() })
            .insert_resource(Assets::<Image>::default()).insert_resource(materials)
            .add_systems(Update, sync_cave_water_phases);
        for offset in [Vec2::ZERO, Vec2::new(80., -48.), Vec2::new(-32., 96.)] {
            app.world_mut().resource_mut::<VisualWorldFrame>().center = built.center + offset;
            app.update();
            let frame = app.world().resource::<VisualWorldFrame>();
            let expected = cave_water::retained_registration(&built, frame);
            let material = app.world().resource::<Assets<VoxelMaterial>>().get(&handle).unwrap();
            assert_eq!(material.extension.water_enabled, Vec4::new(0., expected.y, expected.z, expected.w));
            assert_eq!(material.extension.water_enabled.y, 32.);
            assert_eq!(material.extension.water_phase, Vec4::ZERO);
            assert!(material.extension.water_mask.is_none());
            assert_eq!(frame.tiles, built.tiles, "material coordinates never rewrite source scenery");
        }
    }

    #[test]
    fn outdoor_water_palette_refreshes_without_advancing_the_animation_mask() {
        use bevy::{asset::RenderAssetUsages,render::render_resource::{Extent3d,TextureDimension}};
        for (map,family,block,flag,blue) in [
            ("OlivineCity","johto",0x35,3.,0.48),
            ("DarkCaveVioletEntrance","dark_cave",0x2c,1.,1.),
            ("UnionCave1F","cave",0x2c,1.,1.),
        ] {
        let mut images=Assets::<Image>::default();
        let pixels=|blue:u8|(0..64).flat_map(|i|if i%8==0 {[255,255,255,255]}else {[0,0,blue,255]}).collect();
        let water=images.add(Image::new(Extent3d{width:8,height:8,depth_or_array_layers:1},
            TextureDimension::D2,pixels(255),TextureFormat::Rgba8UnormSrgb,RenderAssetUsages::MAIN_WORLD));
        let atlas=images.add(images.get(&water).unwrap().clone());
        let tile=crystal_render_api::VisualTile{column:0,row:0,texture:water.clone(),priority:false,
            animation_frames:Some(vec![water.clone(),water.clone()].into()),
            source:crystal_render_api::VisualTileSource{tileset_id:family.into(),metatile_id:block,
                subtile_column:0,subtile_row:0,tile_index:0x14}};
        let frame=VisualWorldFrame{active:true,map_id:map.into(),grid_size:UVec2::ONE,
            tile_size:Vec2::splat(8.),map_texture:atlas.clone(),tiles:vec![tile],..default()};
        let mask=images.add(cave_water::mask(&frame).unwrap());
        let mask_data=images.get(&mask).unwrap().data.clone();
        let mut material=textured_terrain_material(atlas);
        material.extension.water_mask=Some(mask.clone());
        material.extension.water_enabled=cave_water::registration(&frame);
        let mut materials=Assets::<VoxelMaterial>::default();
        let clone=materials.add(material.clone());let handle=materials.add(material);
        let mut app=App::new();
        app.insert_resource(Time::<()>::default()).insert_resource(frame.clone())
            .insert_resource(crystal_render_api::VisualTileAnimationFrame{map_id:frame.map_id.clone(),
                grid_origin:frame.grid_origin,tiles:frame.tiles.clone()})
            .insert_resource(VoxelViewStatus{active:true,..default()})
            .insert_resource(TerrainRevisionCache{built_frame:Some(frame.clone()),
                textured_material:Some(handle.clone()),..default()})
            .insert_resource(images).insert_resource(materials).add_systems(Update,sync_cave_water_phases);
        app.update();
        let before=app.world().resource::<Assets<VoxelMaterial>>().get(&handle).unwrap().extension.water_deep;
        let phase=app.world().resource::<Assets<VoxelMaterial>>().get(&handle).unwrap().extension.water_phase;
        assert_eq!(before,if family=="johto" {Vec4::new(0.016,0.16,blue,flag)} else {Vec4::new(0.,0.,blue,flag)});
        app.world_mut().resource_mut::<Assets<Image>>().get_mut(&water).unwrap().data=Some(pixels(99));
        // The source republishes its palette while retaining the same phase.
        app.world_mut().resource_mut::<crystal_render_api::VisualTileAnimationFrame>().tiles=frame.tiles.clone();
        app.update();
        let materials=app.world().resource::<Assets<VoxelMaterial>>();
        let after=&materials.get(&handle).unwrap().extension;
        assert!(after.water_deep.z<before.z*0.2 && after.water_deep.w==flag);
        assert_eq!(after.water_phase,phase);
        assert_eq!(app.world().resource::<Assets<Image>>().get(&mask).unwrap().data,mask_data);
        assert_eq!(*app.world().resource::<VisualWorldFrame>(),frame);
        assert_eq!(materials.get(&clone).unwrap().extension.water_deep,after.water_deep);
        // Missing/dark source paint must clear both the main and retained
        // occluder materials, without advancing flow or changing the mask.
        app.world_mut().resource_mut::<Assets<Image>>().get_mut(&water).unwrap().data=Some([0,0,0,255].repeat(64));
        app.world_mut().resource_mut::<crystal_render_api::VisualTileAnimationFrame>().tiles=frame.tiles.clone();
        app.update();
        let materials=app.world().resource::<Assets<VoxelMaterial>>();
        for id in [&handle,&clone] {
            let ext=&materials.get(id).unwrap().extension;
            assert_eq!(ext.water_deep.w,if family=="johto" {-2.} else {0.});
            assert_eq!(ext.water_phase,phase);
        }
        assert_eq!(app.world().resource::<Assets<Image>>().get(&mask).unwrap().data,mask_data);
        assert_eq!(*app.world().resource::<VisualWorldFrame>(),frame);
        }
    }

    #[test]
    fn smooth_depth_alpha_only_changes_the_opted_in_forward_blend_pipeline() {
        use bevy::pbr::MeshPipelineKey as Key;
        for (prepass,key,enabled,expected) in [
            (false,Key::BLEND_ALPHA,true,true),
            (false,Key::BLEND_ALPHA,false,false),
            (false,Key::NONE,true,false),
            (false,Key::MAY_DISCARD,true,false),
            (true,Key::BLEND_ALPHA,true,false),
            (true,Key::DEPTH_PREPASS|Key::VIEW_PROJECTION_ORTHOGRAPHIC,true,false),
        ] {
            let mut descriptor=depth_qa_descriptor(prepass);
            descriptor.depth_stencil.as_mut().unwrap().depth_write_enabled=Some(false);
            let mut original=descriptor.clone();
            configure_smooth_depth_fade_pipeline(&mut descriptor,key,enabled);
            original.depth_stencil.as_mut().unwrap().depth_write_enabled=Some(expected);
            assert_eq!(descriptor,original,"all other pipeline fields and passes stay exact");
        }
    }

    #[test]
    fn surface_discard_specialization_retains_source_cutoffs_and_fade_animation() {
        let mut surface = VoxelSurface::default();
        assert_eq!(VoxelSurfaceKey::for_mode(&surface, false), VoxelSurfaceKey {
            source_alpha_cutoff: false, coverage_fade: false, depth_prepass_palette: false, smooth_depth_fade:false, material_lighting:render_quality::requested().materials(),
        });
        for cutoff in [-1.0, 0.0, 0.63, f32::NAN] {
            surface.cutaway.fade.w = cutoff;
            let key = VoxelSurfaceKey::for_mode(&surface, false);
            assert_eq!(key.source_alpha_cutoff, cutoff > 0.0);
            assert_eq!(key.coverage_fade, cutoff != 0.0);
            for fade in [0.0, 0.25, 1.0] {
                surface.cutaway.fade.x = fade;
                assert_eq!(key, VoxelSurfaceKey::for_mode(&surface, false));
                assert_eq!(VoxelSurfaceKey::for_mode(&surface, true), VoxelSurfaceKey {
                    source_alpha_cutoff: true, coverage_fade: true, depth_prepass_palette: false, smooth_depth_fade:false, material_lighting:render_quality::requested().materials(),
                });
            }
        }
    }

    fn depth_qa_descriptor(prepass: bool) -> RenderPipelineDescriptor {
        let mut descriptor = RenderPipelineDescriptor {
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(CompareFunction::GreaterEqual),
                stencil: StencilState::default(), bias: DepthBiasState::default(),
            }),
            ..default()
        };
        if prepass {
            descriptor.vertex.shader_defs.push("PREPASS_PIPELINE".into());
        }
        descriptor
    }

    fn depth_qa_layout(uvs: bool, skin: bool) -> MeshVertexBufferLayoutRef {
        let mut mesh = Mesh::from(Cuboid::default());
        if !uvs { mesh.remove_attribute(Mesh::ATTRIBUTE_UV_0); }
        if skin {
            mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, vec![[1.0, 0.0, 0.0, 0.0]; mesh.count_vertices()]);
        }
        mesh.get_mesh_vertex_buffer_layout(&mut bevy::mesh::MeshVertexBufferLayouts::default())
    }

    #[test]
    fn selective_tree_depth_qa_preserves_forward_shadow_and_other_prepass_descriptors() {
        use bevy::pbr::MeshPipelineKey as Key;
        let layout = depth_qa_layout(false, false);
        let palette = VoxelSurfaceKey { source_alpha_cutoff: false,
            coverage_fade: false, depth_prepass_palette: true, smooth_depth_fade:false, material_lighting:render_quality::requested().materials(),
            };
        for (prepass, mesh_key, enabled) in [
            (false, Key::DEPTH_PREPASS, true), // Actual forward view also has this bit.
            (true, Key::DEPTH_PREPASS | Key::VIEW_PROJECTION_ORTHOGRAPHIC | Key::UNCLIPPED_DEPTH_ORTHO, true),
            (true, Key::DEPTH_PREPASS | Key::VIEW_PROJECTION_PERSPECTIVE, true), // Point/spot shadows.
            (true, Key::DEPTH_PREPASS | Key::NORMAL_PREPASS, true),
            (true, Key::DEPTH_PREPASS | Key::MOTION_VECTOR_PREPASS, true),
            (true, Key::DEPTH_PREPASS | Key::DEFERRED_PREPASS, true),
            (true, Key::DEPTH_PREPASS, false), // Existing native opt-in prepass is unaffected.
            (true, Key::NONE, true),
            (true, Key::DEPTH_PREPASS, true), // Eligible static palette tree keeps exact depth state.
        ] {
            let mut descriptor = depth_qa_descriptor(prepass);
            let original = descriptor.clone();
            configure_tree_depth_prepass_pipeline(&mut descriptor, mesh_key, palette, &layout, enabled);
            assert_eq!(descriptor, original, "forward/shadow/other pass descriptors must remain byte-for-byte equivalent");
        }
    }

    #[test]
    fn selective_tree_depth_qa_never_occludes_forward_cutaway_masked_or_deformed_surfaces() {
        use bevy::pbr::MeshPipelineKey as Key;
        let tree = depth_qa_layout(false, false);
        let tagged = depth_qa_layout(true, false);
        let skinned = depth_qa_layout(false, true);
        let palette = VoxelSurfaceKey { source_alpha_cutoff: false,
            coverage_fade: false, depth_prepass_palette: true, smooth_depth_fade:false, material_lighting:render_quality::requested().materials(),
            };
        let base = Key::DEPTH_PREPASS;
        for (surface, key, layout) in [
            (palette, base, &tagged), // Cave/grass tags and texture coordinates retained.
            (palette, base, &skinned),
            (palette, base | Key::MORPH_TARGETS, &tree),
            (palette, base | Key::MAY_DISCARD, &tree),
            (palette, base | Key::VISIBILITY_RANGE_DITHER, &tree),
            (palette, base | Key::BLEND_ALPHA, &tree),
            (VoxelSurfaceKey { source_alpha_cutoff: true, ..palette }, base, &tree),
            (VoxelSurfaceKey { coverage_fade: true, ..palette }, base, &tree),
            (VoxelSurfaceKey { depth_prepass_palette: false, ..palette }, base, &tree),
        ] {
            let mut descriptor = depth_qa_descriptor(true);
            let original = descriptor.clone();
            configure_tree_depth_prepass_pipeline(&mut descriptor, key, surface, layout, true);
            let depth = descriptor.depth_stencil.as_ref().unwrap();
            assert_eq!(depth.depth_write_enabled, Some(false));
            assert_eq!(depth.depth_compare, Some(CompareFunction::Never));
            descriptor.depth_stencil = original.depth_stencil.clone();
            assert_eq!(descriptor, original, "only camera-prepass depth state may change");
        }
    }

    #[test]
    fn tree_depth_constructor_contract_rejects_textures_alpha_and_lit_materials() {
        let palette = solid_terrain_material();
        assert!(palette.extension.depth_prepass_palette);
        for base in [
            StandardMaterial { unlit: false, ..palette.base.clone() },
            StandardMaterial { alpha_mode: AlphaMode::Blend, ..palette.base.clone() },
            StandardMaterial { alpha_mode: AlphaMode::Mask(0.5), ..palette.base.clone() },
            StandardMaterial { base_color_texture: Some(Handle::default()), ..palette.base.clone() },
            StandardMaterial { normal_map_texture: Some(Handle::default()), ..palette.base.clone() },
            StandardMaterial { depth_map: Some(Handle::default()), ..palette.base.clone() },
        ] {
            assert!(!voxel_material(base).extension.depth_prepass_palette);
        }
    }

    #[test]
    fn source_palette_period_changes_lighting_without_rebuilding_terrain() {
        use crystal_render_api::VisualWorldLighting;
        let mut app = App::new();
        app.init_resource::<VisualWorldFrame>()
            .init_resource::<VisualWorldLighting>()
            .init_resource::<live_profiles::LiveProfiles>()
            .init_resource::<VoxelViewSettings>()
            .init_resource::<GlobalAmbientLight>()
            .add_systems(Update, sync_voxel_atmosphere);
        let sun = app.world_mut().spawn((DirectionalLight::default(),
            bevy::light::CascadeShadowConfig::default(), VoxelSun)).id();
        app.update();
        let day = app.world().get::<DirectionalLight>(sun).unwrap().illuminance;
        let ambient = app.world().resource::<GlobalAmbientLight>().brightness;
        let frame = app.world().resource::<VisualWorldFrame>().clone();
        *app.world_mut().resource_mut::<VisualWorldLighting>() = VisualWorldLighting::Night;
        app.update();
        assert_eq!(app.world().get::<DirectionalLight>(sun).unwrap().illuminance, day * 0.50);
        assert_eq!(app.world().resource::<GlobalAmbientLight>().brightness, ambient * 0.50);
        let night = app.world().get::<DirectionalLight>(sun).unwrap().color.to_linear();
        assert!(night.blue > night.red && night.blue > night.green);
        assert_eq!(*app.world().resource::<VisualWorldFrame>(), frame);
        *app.world_mut().resource_mut::<VisualWorldLighting>() = VisualWorldLighting::Morning;
        app.update();
        assert_eq!(app.world().get::<DirectionalLight>(sun).unwrap().illuminance, day);
        let morning = app.world().get::<DirectionalLight>(sun).unwrap().color.to_linear();
        assert!(morning.red > morning.green && morning.green > morning.blue);
        *app.world_mut().resource_mut::<VisualWorldLighting>() = VisualWorldLighting::Day;
        app.update();
        assert_eq!(app.world().get::<DirectionalLight>(sun).unwrap().illuminance, day);
        assert_eq!(app.world().resource::<GlobalAmbientLight>().brightness, ambient);
        assert_eq!(VisualWorldLighting::from_source_palette("indoor"), VisualWorldLighting::Day);
        assert_eq!(VisualWorldLighting::from_source_palette("darkness"), VisualWorldLighting::Dark);
        assert_eq!(VisualWorldLighting::from_source_palette("dark"), VisualWorldLighting::Dark);
    }

    #[test]
    fn atmosphere_animation_updates_do_not_dirty_unchanged_light_or_shadow_settings() {
        #[derive(Resource, Default)]
        struct Changes(usize);
        fn count_changes(
            changed: Query<(), Or<(Changed<DirectionalLight>,
                Changed<bevy::light::CascadeShadowConfig>, Changed<DistanceFog>, Changed<Fxaa>)>>,
            ambient: Res<GlobalAmbientLight>,
            mut count: ResMut<Changes>,
        ) {
            count.0 += changed.iter().count() + usize::from(ambient.is_changed());
        }
        let mut app = App::new();
        app.init_resource::<VisualWorldFrame>()
            .init_resource::<live_profiles::LiveProfiles>()
            .init_resource::<VoxelViewSettings>()
            .init_resource::<GlobalAmbientLight>()
            .init_resource::<Changes>()
            .add_systems(Update, (sync_voxel_atmosphere, count_changes).chain());
        app.world_mut().resource_mut::<VoxelViewSettings>().enabled = true;
        {
            let mut frame = app.world_mut().resource_mut::<VisualWorldFrame>();
            frame.active = true;
            frame.map_id = "NewBarkTown".into();
        }
        let sun = app.world_mut().spawn((DirectionalLight::default(),
            bevy::light::CascadeShadowConfig::default(), VoxelSun)).id();
        app.world_mut().spawn((DistanceFog::default(), Fxaa::default(), VoxelWorldCamera));
        app.update();
        assert!(app.world().resource::<Changes>().0 > 0);
        app.world_mut().resource_mut::<Changes>().0 = 0;
        for _ in 0..8 {
            // Force source-frame changes like live atlas animation, without
            // changing any atmosphere inputs or the authored rendering values.
            app.world_mut().resource_mut::<VisualWorldFrame>().terrain_revision += 1;
            app.update();
        }
        assert_eq!(app.world().resource::<Changes>().0, 0);
        assert_eq!(app.world().get::<DirectionalLight>(sun).unwrap().illuminance, 3000.0);
        assert_eq!(app.world().get::<bevy::light::CascadeShadowConfig>(sun).unwrap().bounds.len(),
            if cfg!(target_arch = "wasm32") { 1 } else { 2 });
        app.world_mut().resource_mut::<VisualWorldFrame>().map_id = "GoldenrodCity".into();
        app.update();
        assert!(app.world().resource::<Changes>().0 > 0,
            "actual map lighting changes must still reach Bevy");
        assert_eq!(app.world().get::<bevy::light::CascadeShadowConfig>(sun).unwrap().bounds.len(),
            if cfg!(target_arch = "wasm32") {1} else {2});
    }

    #[test]
    fn burned_tower_fixed_palette_does_not_receive_second_night_tint() {
        use crystal_render_api::{VisualTile,VisualTileSource,VisualWorldLighting};
        let mut app=App::new();app.init_resource::<VisualWorldFrame>()
            .init_resource::<VisualWorldLighting>().init_resource::<live_profiles::LiveProfiles>()
            .init_resource::<VoxelViewSettings>().init_resource::<GlobalAmbientLight>()
            .add_systems(Update,sync_voxel_atmosphere);
        app.world_mut().resource_mut::<VoxelViewSettings>().enabled=true;
        *app.world_mut().resource_mut::<VisualWorldLighting>()=VisualWorldLighting::Night;
        let mut frame=app.world_mut().resource_mut::<VisualWorldFrame>();frame.active=true;
        frame.map_id="BurnedTower1F".into();frame.tiles=vec![VisualTile{column:0,row:0,priority:false,
            animation_frames:None,texture:Default::default(),source:VisualTileSource{
                tileset_id:"tower".into(),metatile_id:9,tile_index:2,subtile_column:0,subtile_row:0}}];drop(frame);
        let sun=app.world_mut().spawn((DirectionalLight::default(),bevy::light::CascadeShadowConfig::default(),VoxelSun)).id();
        let before=app.world().resource::<VisualWorldFrame>().clone();app.update();
        assert_eq!(app.world().resource::<GlobalAmbientLight>().color.to_linear(),Color::WHITE.to_linear());
        assert_eq!(app.world().get::<DirectionalLight>(sun).unwrap().color.to_linear(),Color::WHITE.to_linear());
        assert_eq!(app.world().get::<DirectionalLight>(sun).unwrap().illuminance,3000.);
        assert_eq!(*app.world().resource::<VisualWorldFrame>(),before);
        assert_eq!(*app.world().resource::<VisualWorldLighting>(),VisualWorldLighting::Night);
        app.world_mut().resource_mut::<VisualWorldFrame>().map_id="Route29".into();app.update();
        let light=app.world().get::<DirectionalLight>(sun).unwrap().color.to_linear();assert!(light.blue>light.red);
    }
    #[test]
    fn fixed_forest_palette_does_not_receive_a_second_blue_night_light() {
        use crystal_render_api::{VisualTile, VisualTileSource, VisualWorldLighting};
        let mut app = App::new();
        app.init_resource::<VisualWorldFrame>()
            .init_resource::<VisualWorldLighting>()
            .init_resource::<live_profiles::LiveProfiles>()
            .init_resource::<VoxelViewSettings>()
            .init_resource::<GlobalAmbientLight>()
            .add_systems(Update, sync_voxel_atmosphere);
        app.world_mut().resource_mut::<VoxelViewSettings>().enabled = true;
        *app.world_mut().resource_mut::<VisualWorldLighting>() = VisualWorldLighting::Night;
        let mut frame = app.world_mut().resource_mut::<VisualWorldFrame>();
        frame.active = true;
        frame.map_id = "IlexForest".into();
        frame.tiles = vec![VisualTile { column: 0, row: 0, priority: false,
            animation_frames: None, texture: Default::default(),
            source: VisualTileSource { tileset_id: "forest".into(), metatile_id: 0x01,
                tile_index: 0x05, subtile_column: 0, subtile_row: 0 } }];
        drop(frame);
        let sun = app.world_mut().spawn((DirectionalLight::default(),
            bevy::light::CascadeShadowConfig::default(), VoxelSun)).id();
        let before = app.world().resource::<VisualWorldFrame>().clone();
        app.update();
        assert_eq!(app.world().resource::<GlobalAmbientLight>().color.to_linear(), Color::WHITE.to_linear());
        assert!((app.world().resource::<GlobalAmbientLight>().brightness - 72.).abs() < 0.001);
        let light = app.world().get::<DirectionalLight>(sun).unwrap();
        assert_eq!(light.illuminance, 2700.);
        assert!(light.color.to_linear().red > light.color.to_linear().blue);
        assert_eq!(*app.world().resource::<VisualWorldFrame>(), before);
        assert_eq!(*app.world().resource::<VisualWorldLighting>(), VisualWorldLighting::Night);
        // Ordinary outdoor night still has its actual blue light and energy.
        app.world_mut().resource_mut::<VisualWorldFrame>().map_id = "Route29".into();
        app.update();
        let light = app.world().get::<DirectionalLight>(sun).unwrap();
        assert_eq!(light.illuminance, 1500.);
        assert!(light.color.to_linear().blue > light.color.to_linear().red);
    }

    #[test]
    fn flash_lit_caves_use_published_pigment_without_changing_darkness_or_source_state() {
        use crystal_render_api::{VisualTile, VisualTileSource, VisualWorldLighting};
        let mut app=App::new();
        app.init_resource::<VisualWorldFrame>().init_resource::<VisualWorldLighting>()
            .init_resource::<live_profiles::LiveProfiles>().init_resource::<VoxelViewSettings>()
            .init_resource::<GlobalAmbientLight>().add_systems(Update,sync_voxel_atmosphere);
        app.world_mut().resource_mut::<VoxelViewSettings>().enabled=true;
        let sun=app.world_mut().spawn((DirectionalLight::default(),
            bevy::light::CascadeShadowConfig::default(),VoxelSun)).id();
        for (map,family,unlit_ambient,unlit_sun) in [
            ("DarkCaveVioletEntrance","dark_cave",28.8,840.),
            ("WhirlIslandB1F","cave",48.,1440.),
            ("SilverCaveRoom1","dark_cave",28.8,840.),
        ] {
            *app.world_mut().resource_mut::<VisualWorldFrame>()=VisualWorldFrame {
                active:true,map_id:map.into(),tiles:vec![VisualTile {column:0,row:0,
                    priority:false,animation_frames:None,texture:Default::default(),
                    source:VisualTileSource {tileset_id:family.into(),metatile_id:2,
                        subtile_column:0,subtile_row:0,tile_index:0x16}}],..default()};
            let source=app.world().resource::<VisualWorldFrame>().clone();
            for (period,ambient,direct) in [
                (VisualWorldLighting::Dark,unlit_ambient,unlit_sun),
                (VisualWorldLighting::Night,80.,1950.),
                (VisualWorldLighting::Day,80.,1950.),
                (VisualWorldLighting::Dark,unlit_ambient,unlit_sun),
            ] {
                *app.world_mut().resource_mut::<VisualWorldLighting>()=period;
                app.update();
                let fill=app.world().resource::<GlobalAmbientLight>();
                let direct_light=app.world().get::<DirectionalLight>(sun).unwrap();
                assert!((fill.brightness-ambient).abs()<0.001,"{map}/{period:?}");
                assert!((direct_light.illuminance-direct).abs()<0.001,"{map}/{period:?}");
                assert_eq!(fill.color.to_linear(),Color::WHITE.to_linear());
                assert_eq!(direct_light.color.to_linear(),Color::WHITE.to_linear());
                assert_eq!(*app.world().resource::<VisualWorldFrame>(),source);
                assert_eq!(*app.world().resource::<VisualWorldLighting>(),period);
            }
        }
    }

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
            ("RockTunnelB1F", "dark_cave", 80.0, 1950.0),
            ("RockTunnel1F", "dark_cave", 80.0, 1950.0),
            ("DiglettsCave", "cave", 80.0, 1950.0),
            ("MountMoon", "cave", 80.0, 1950.0),
            ("TohjoFalls", "cave", 80.0, 1950.0),
            ("VictoryRoad", "cave", 80.0, 1950.0),
            ("RockTunnel1F", "house", 80.0, 3000.0),
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
            assert!((app.world().resource::<GlobalAmbientLight>().brightness - ambient).abs() < 0.001, "ambient in {map}/{tileset}");
            assert!((app.world().get::<DirectionalLight>(sun).unwrap().illuminance - direct).abs() < 0.001, "direct in {map}/{tileset}");
        }
    }

    #[test]
    fn shadow_detail_tracks_the_camera_across_outdoor_and_cave_maps() {
        let mut app = App::new();
        app.init_resource::<VisualWorldFrame>()
            .init_resource::<live_profiles::LiveProfiles>()
            .init_resource::<VoxelViewSettings>()
            .add_systems(Update, sync_voxel_atmosphere);
        let sun = app.world_mut().spawn((DirectionalLight::default(),
            bevy::light::CascadeShadowConfig::default(), VoxelSun)).id();
        for map in ["Route29", "Route26", "NewBarkTown", "NationalPark", "NationalParkBugContest",
            "GoldenrodCity", "UnionCave1F", "Route29"] {
            app.world_mut().resource_mut::<VisualWorldFrame>().map_id = map.into();
            app.update();
            let world = app.world();
            let cascade = world.get::<bevy::light::CascadeShadowConfig>(sun).unwrap();
            let light = world.get::<DirectionalLight>(sun).unwrap();
            assert_eq!(cascade.bounds.len(), if cfg!(target_arch = "wasm32") {1} else {2});
            assert!(*cascade.bounds.last().unwrap() < 1024.0,
                "near-player shadow detail must not regress to a 4096-unit halo in {map}");
            assert!(cascade.bounds.windows(2).all(|pair|pair[0]<pair[1]));
            assert_eq!(light.shadow_normal_bias, 1.0);
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
        for (map, tileset, reveal, daylight) in [
            ("DarkCaveVioletEntrance", "dark_cave", true, false),
            ("UnionCave1F", "cave", true, false),
            ("IcePath1F", "ice_path", true, false), ("HallOfFame", "ice_path", false, false),
            ("NewBarkTown", "johto", false, true), ("Route29", "johto", false, true),
            ("Route26", "johto", false, true), ("Route36", "johto", false, true),
            ("Route38", "johto_modern", false, true), ("GoldenrodCity", "johto", false, true),
            ("BlackthornCity", "johto", false, true), ("CherrygroveCity", "johto", false, true),
            ("NationalPark", "park", false, true), ("IlexForest", "forest", false, true),
            ("NewBarkTown", "forest", false, true),
            ("CeruleanCity", "kanto", false, true), ("OlivinePort", "port", false, true),
            ("PlayersHouse1F", "house", false, false), ("Route36", "house", false, false),
            ("Route36", "cave", true, false), ("NewBarkTown", "johto", false, true)] {
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
            let mut grass_mesh = actor_quad_mesh();
            grass_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[84000.0, 0.5]; 4]);
            let terrain = BuiltTerrain {
                city_emitters: Vec::new(),
                streamed_groves: None,
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
                solid_meshes: vec![actor_quad_mesh().into(), grass_mesh.into()],
                animated_textured_mesh: actor_quad_mesh(),
                animated_solid_mesh: actor_quad_mesh(),
                animated_pillars: Vec::new(), train_doors: false,
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
                None,
                false,
            )
            .unwrap();
            for handle in [cache.textured_material.as_ref().unwrap(), cache.solid_material.as_ref().unwrap(),
                cache.animated_solid_material.as_ref().unwrap(), cache.grass_material.as_ref().unwrap()] {
                let environment = materials.get(handle).unwrap().extension.cave_reveal;
                assert_eq!(environment.x > 0.5, reveal);
                assert_eq!(environment.y > 0.5, tileset == "dark_cave");
                assert_eq!(environment.z > 0.5, daylight, "surface light in {map}/{tileset}");
                let town_fill = matches!(map, "NewBarkTown" | "GoldenrodCity"
                    | "BlackthornCity" | "CherrygroveCity" | "CeruleanCity")
                    && tileset != "forest";
                assert_eq!(environment.z, if town_fill { 2. } else if daylight { 1. } else { 0. },
                    "town shadow fill in {map}/{tileset}");
            }
            assert_eq!(materials.get(&actor_material).unwrap().extension.cave_reveal, Vec4::ZERO);
            queue.apply(&mut world);
            let solid_children = world.get::<Children>(cache.solid_entity.unwrap()).unwrap();
            assert_eq!(solid_children.len(), 2);
            assert_eq!(&world.get::<MeshMaterial3d<VoxelMaterial>>(solid_children[0]).unwrap().0,
                cache.solid_material.as_ref().unwrap());
            assert_eq!(&world.get::<MeshMaterial3d<VoxelMaterial>>(solid_children[1]).unwrap().0,
                cache.grass_material.as_ref().unwrap());
            assert_ne!(cache.grass_material, cache.solid_material);
            let animated = world.get::<MeshMaterial3d<VoxelMaterial>>(
                cache.animated_solid_entity.unwrap()).unwrap();
            assert_eq!(Some(&animated.0), cache.animated_solid_material.as_ref());
            assert_ne!(Some(&animated.0), cache.solid_material.as_ref(),
                "flower animation must not invalidate static terrain in {map}");
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
                        None,
                        &frame,
                        None,
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

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_water_images_publish_captured_phase_and_reject_stale_results() {
        let pool = bevy::tasks::TaskPoolBuilder::new().num_threads(1).build();
        for stale in [false,true] {
            let mut app = App::new();
            app.insert_resource(Assets::<Image>::default())
                .insert_resource(Assets::<Mesh>::default())
                .insert_resource(Assets::<VoxelMaterial>::default());
            let mut images = app.world_mut().resource_mut::<Assets<Image>>();
            let atlas = images.add(Image::default());
            let first = images.add(Image::default());
            let second = images.add(Image::default());
            let stone = images.add(Image::default());
            let bedrock = images.add(Image::default());
            drop(images);
            let captured = VisualWorldFrame {
                active: true, map_id: "UnionCave1F".into(), grid_size: UVec2::ONE,
                tile_size: Vec2::splat(8.),map_texture: atlas.clone(),
                tiles: vec![crystal_render_api::VisualTile {
                    column:0,row:0,texture:first.clone(),priority:false,
                    animation_frames:Some(vec![first,second.clone()].into()),
                    source:crystal_render_api::VisualTileSource {tileset_id:"cave".into(),
                        metatile_id:0x2c,subtile_column:0,subtile_row:0,tile_index:0x14},
                }], ..default()
            };
            let mut live = captured.clone();
            live.tiles[0].texture=second;
            let key = TerrainCacheKey::from_frame(&live);
            let mut completed_key = key.clone();
            if stale { completed_key.profiles_revision+=1; }
            let water = water_surface::PreparedImages::for_frame(&captured);
            let expected = water.mask.as_ref().unwrap().data.clone();
            let terrain = BuiltTerrain {
                city_emitters: Vec::new(),
                streamed_groves:None,scenery:Vec::new(),scenery_fades:Vec::new(),
                background:None,cave_enclosure:None,instances:Vec::new(),
                reveal_join_batches:Vec::new(),fade_textured_groups:Vec::new(),
                fade_solid_groups:Vec::new(),footing_heights:vec![0.],
                textured_meshes:Vec::new(),solid_meshes:Vec::new(),
                animated_textured_mesh:mesh::SurfaceMeshData::default().into_mesh(),
                animated_solid_mesh:mesh::SurfaceMeshData::default().into_mesh(),animated_pillars:Vec::new(),train_doors:false,
            };
            let mut builds = TerrainBuildQueue {key:Some(key),..default()};
            builds.task=Some(pool.spawn(async move {TerrainBuildResult {
                key:completed_key,frame:captured,source_texture:atlas,
                profiles:live_profiles::LiveProfiles::default().document,terrain:Ok(terrain),water:Some(water),
            }}));
            app.insert_resource(live).insert_resource(builds)
                .insert_resource(TerrainRevisionCache {painted_stone:Some(stone),painted_bedrock:Some(bedrock),..default()})
                .add_systems(Update,|frame:Res<VisualWorldFrame>,mut commands:Commands,
                    mut cache:ResMut<TerrainRevisionCache>,mut builds:ResMut<TerrainBuildQueue>,
                    mut images:ResMut<Assets<Image>>,mut meshes:ResMut<Assets<Mesh>>,
                    mut materials:ResMut<Assets<VoxelMaterial>>,
                    mut entities:Query<(&mut Visibility,&mut Transform),VoxelTerrainFilter>| {
                    sync_terrain(&frame,&live_profiles::LiveProfiles::default(),&mut commands,
                        &mut cache,&mut builds,&mut images,&mut meshes,&mut materials,&mut entities).unwrap();
                });
            let deadline=std::time::Instant::now()+std::time::Duration::from_secs(5);
            while app.world().resource::<TerrainBuildQueue>().task.is_some() {
                assert!(std::time::Instant::now()<deadline);
                app.update();std::thread::yield_now();
            }
            let cache=app.world().resource::<TerrainRevisionCache>();
            if stale {
                assert!(cache.built_frame.is_none() && cache.key.is_none());
                assert_eq!(app.world().resource::<Assets<Image>>().len(),5,"stale images never published");
                assert!(app.world().resource::<Assets<VoxelMaterial>>().is_empty());
            } else {
                let material=app.world().resource::<Assets<VoxelMaterial>>()
                    .get(cache.textured_material.as_ref().unwrap()).unwrap();
                let image=app.world().resource::<Assets<Image>>()
                    .get(material.extension.water_mask.as_ref().unwrap()).unwrap();
                assert_eq!(image.data,expected,"publish captured phase, never newer live phase");
                assert_eq!(image.data.as_ref().unwrap()[2],0);
                assert_eq!(app.world().resource::<Assets<Image>>().len(),7);
                assert!(cache.key.is_some());
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_painted_prerequisite_holds_atomic_publish_and_reuses_map_handles() {
        use std::sync::{Arc, atomic::{AtomicBool,Ordering}};
        let pool = bevy::tasks::TaskPoolBuilder::new().num_threads(2).build();
        let gate = Arc::new(AtomicBool::new(false));
        let job_gate = gate.clone();
        let mut app = App::new();
        app.insert_resource(Assets::<Image>::default())
            .insert_resource(Assets::<Mesh>::default())
            .insert_resource(Assets::<VoxelMaterial>::default())
            .init_resource::<TerrainRevisionCache>();
        let atlas = app.world_mut().resource_mut::<Assets<Image>>().add(Image::default());
        let frame = VisualWorldFrame { active: true, map_id: "PaintedPrerequisite".into(),
            map_texture: atlas.clone(), ..default() };
        let key = TerrainCacheKey::from_frame(&frame);
        let build_frame = frame.clone();
        let terrain = BuiltTerrain {
            city_emitters: Vec::new(),
            streamed_groves: None, scenery: Vec::new(), scenery_fades: Vec::new(),
            background: None, cave_enclosure: None, instances: Vec::new(),
            reveal_join_batches: Vec::new(), fade_textured_groups: Vec::new(),
            fade_solid_groups: Vec::new(), footing_heights: Vec::new(),
            textured_meshes: Vec::new(), solid_meshes: Vec::new(),
            animated_textured_mesh: mesh::SurfaceMeshData::default().into_mesh(),
            animated_solid_mesh: mesh::SurfaceMeshData::default().into_mesh(),
            animated_pillars: Vec::new(), train_doors: false,
        };
        let mut builds = TerrainBuildQueue { key: Some(key.clone()), ..default() };
        builds.task = Some(pool.spawn(async move { TerrainBuildResult {
            key, frame: build_frame, source_texture: atlas,
            profiles: live_profiles::LiveProfiles::default().document,
            terrain: Ok(terrain),
            water: None,
        }}));
        builds.painted = painted_stone::NativePreparation::from_task(pool.spawn(async move {
            while !job_gate.load(Ordering::SeqCst) {
                bevy::tasks::futures_lite::future::yield_now().await;
            }
            painted_stone::PreparedImages::decode()
        }));
        app.insert_resource(frame).insert_resource(builds)
            .add_systems(Update, |frame: Res<VisualWorldFrame>, mut commands: Commands,
                mut cache: ResMut<TerrainRevisionCache>, mut builds: ResMut<TerrainBuildQueue>,
                mut images: ResMut<Assets<Image>>, mut meshes: ResMut<Assets<Mesh>>,
                mut materials: ResMut<Assets<VoxelMaterial>>,
                mut entities: Query<(&mut Visibility,&mut Transform),VoxelTerrainFilter>| {
                sync_terrain(&frame,&live_profiles::LiveProfiles::default(),&mut commands,
                    &mut cache,&mut builds,&mut images,&mut meshes,&mut materials,&mut entities).unwrap();
            });
        app.update();
        assert!(app.world().resource::<TerrainRevisionCache>().key.is_none());
        assert!(app.world().resource::<TerrainBuildQueue>().task.is_some(),"geometry not consumed before images");
        assert_eq!(app.world().resource::<Assets<Image>>().len(),1);
        assert!(app.world().resource::<Assets<VoxelMaterial>>().is_empty());
        gate.store(true,Ordering::SeqCst);
        let deadline = std::time::Instant::now()+std::time::Duration::from_secs(10);
        while app.world().resource::<TerrainRevisionCache>().key.is_none() {
            assert!(std::time::Instant::now()<deadline,"native publication did not finish");
            app.update();
            std::thread::yield_now();
        }
        let cache = app.world().resource::<TerrainRevisionCache>();
        let handles = (cache.painted_stone.clone(),cache.painted_bedrock.clone());
        assert!(handles.0.is_some() && handles.1.is_some());
        assert!(!app.world().resource::<TerrainBuildQueue>().painted.pending());
        assert_eq!(app.world().resource::<Assets<Image>>().len(),3);
        app.world_mut().resource_mut::<VisualWorldFrame>().map_id = "PaintedNextMap".into();
        app.update();
        let cache = app.world().resource::<TerrainRevisionCache>();
        assert_eq!((cache.painted_stone.clone(),cache.painted_bedrock.clone()),handles);
        assert!(!app.world().resource::<TerrainBuildQueue>().painted.pending());
        assert_eq!(app.world().resource::<Assets<Image>>().len(),3);
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

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn small_native_room_transition_starts_replacement_in_the_first_update() {
        use crystal_render_api::{VisualTile,VisualTileSource};
        let room=|map:&str,tileset:&str|VisualWorldFrame {
            active:true,map_id:map.into(),source_map_size_core_tiles:Some(UVec2::splat(8)),
            tiles:vec![VisualTile {column:0,row:0,texture:default(),animation_frames:None,priority:false,
                source:VisualTileSource {tileset_id:tileset.into(),metatile_id:0,subtile_column:0,
                    subtile_row:0,tile_index:0}}],..default()
        };
        let old=room("PlayersHouse2F","players_room");
        let next=room("PlayersHouse1F","players_house");
        let mut app=App::new();
        app.init_resource::<Assets<Image>>().init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<VoxelMaterial>>().init_resource::<TerrainBuildQueue>();
        let root=app.world_mut().spawn((VoxelTerrain,Visibility::Visible,Transform::IDENTITY)).id();
        app.insert_resource(TerrainRevisionCache {key:Some(TerrainCacheKey::from_frame(&old)),
            built_frame:Some(old),instances_root:Some(root),painted_stone:Some(default()),
            painted_bedrock:Some(default()),..default()}).insert_resource(next.clone())
            .add_systems(Update,|frame:Res<VisualWorldFrame>,mut commands:Commands,
                mut cache:ResMut<TerrainRevisionCache>,mut builds:ResMut<TerrainBuildQueue>,
                mut images:ResMut<Assets<Image>>,mut meshes:ResMut<Assets<Mesh>>,
                mut materials:ResMut<Assets<VoxelMaterial>>,
                mut entities:Query<(&mut Visibility,&mut Transform),VoxelTerrainFilter>| {
                // Deliberately absent source atlas proves the first update
                // reaches actual replacement submission, rather than a wait.
                assert_eq!(sync_terrain(&frame,&live_profiles::LiveProfiles::default(),&mut commands,
                    &mut cache,&mut builds,&mut images,&mut meshes,&mut materials,&mut entities),
                    Err(TerrainSyncError::SourceTextureUnavailable));
            });
        app.update();
        assert_eq!(app.world().resource::<TerrainBuildQueue>().started,1);
        assert!(app.world().get_entity(root).is_err());
        let old=room("PlayersHouse2F","players_room");
        let mut uncertain=next.clone();uncertain.source_map_size_core_tiles=None;
        assert_eq!(terrain_retirement_frames(Some(&old),&uncertain),2);
        let mut large=next.clone();large.source_map_size_core_tiles=Some(UVec2::splat(32));
        assert_eq!(terrain_retirement_frames(Some(&old),&large),2);
        let outside=room("NewBarkTown","johto");
        assert_eq!(terrain_retirement_frames(Some(&old),&outside),2);
        assert_eq!(terrain_retirement_frames(Some(&outside),&next),2);
    }

    #[test]
    fn department_store_exit_retires_old_terrain_before_allocating_goldenrod() {
        let mut old = VisualWorldFrame { map_id: "GoldenrodDeptStore1F".into(), ..default() };
        let next = VisualWorldFrame { active: true, map_id: "GoldenrodCity".into(), ..default() };
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()))
            .init_asset::<Image>().init_asset::<Mesh>().init_asset::<VoxelMaterial>();
        let atlas = app.world_mut().resource_mut::<Assets<Image>>().add(Image::default());
        let atlas_id = atlas.id();
        let material = app.world_mut().resource_mut::<Assets<VoxelMaterial>>()
            .add(textured_terrain_material(atlas.clone()));
        let material_id = material.id();
        old.map_texture = atlas;
        let mesh = app.world_mut().resource_mut::<Assets<Mesh>>()
            .add(Mesh::from(Rectangle::new(8., 8.)));
        let mesh_id = mesh.id();
        let root = app.world_mut().spawn((VoxelTerrain, Visibility::Visible, Transform::IDENTITY)).id();
        let child = app.world_mut().spawn((ChildOf(root), Visibility::Inherited, Transform::IDENTITY,
            Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()))).id();
        app.insert_resource(TerrainRevisionCache {
            key: Some(TerrainCacheKey::from_frame(&old)), built_frame: Some(old),
            instances_root: Some(root), built_footing_heights: vec![1.; 64], footing_heights: vec![1.; 64],
            animated_solid_mesh: Some(mesh), solid_material: Some(material),
            ..default()
        }).insert_resource(next).init_resource::<TerrainBuildQueue>()
            .add_systems(Update, move |frame: Res<VisualWorldFrame>, mut commands: Commands,
                mut cache: ResMut<TerrainRevisionCache>, mut builds: ResMut<TerrainBuildQueue>,
                mut images: ResMut<Assets<Image>>, mut meshes: ResMut<Assets<Mesh>>,
                mut materials: ResMut<Assets<VoxelMaterial>>,
                mut entities: Query<(&mut Visibility, &mut Transform), VoxelTerrainFilter>, mut calls: Local<u8>| {
                if *calls == 2 {
                    // Use real strong handles and Bevy's asset tracking, including
                    // the material's atlas dependency. Check before allocating.
                    assert!(meshes.get(mesh_id).is_none());
                    assert!(materials.get(material_id).is_none());
                    assert!(images.get(atlas_id).is_none());
                    assert_eq!(builds.started, 0);
                }
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
    fn lighthouse_transition_keeps_complete_room_camera_and_actors_during_upload() {
        use crystal_render_api::{VisualTile,VisualTileSource};
        let mut app=App::new();
        app.init_resource::<Assets<Image>>().init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<VoxelMaterial>>().init_resource::<live_profiles::LiveProfiles>()
            .init_resource::<VoxelViewSettings>().init_resource::<VoxelViewStatus>()
            .init_resource::<NativeHealingEffectsStatus>().init_resource::<TerrainBuildQueue>()
            .init_resource::<ActorIdCache>();
        app.world_mut().resource_mut::<VoxelViewSettings>().enabled=true;
        let atlas=app.world_mut().resource_mut::<Assets<Image>>().add(Image::default());
        let source=Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(942),std::marker::PhantomData);
        let old=VisualWorldFrame{active:true,map_id:"OlivineLighthouse1F".into(),map_texture:atlas.clone(),
            grid_size:UVec2::ONE,viewport_size:Vec2::splat(8.),tile_size:Vec2::splat(8.),
            tiles:vec![VisualTile{column:0,row:0,texture:atlas,priority:false,animation_frames:None,
                source:VisualTileSource{tileset_id:"lighthouse".into(),metatile_id:0x27,tile_index:0x2e,
                    subtile_column:0,subtile_row:0}}],..default()};
        let mut live=old.clone();live.map_id="OlivineLighthouse2F".into();live.map_texture=source.clone();
        live.center=Vec2::new(200.,100.);live.grid_origin=IVec2::new(20,10);
        let old_transform=Transform::from_xyz(7.,8.,9.);
        let terrain=app.world_mut().spawn((VoxelTerrain,Visibility::Visible,old_transform)).id();
        let camera=app.world_mut().spawn((Camera::default(),Projection::default(),old_transform,VoxelWorldCamera)).id();
        let actor=app.world_mut().spawn((VoxelActorCard,Visibility::Visible,old_transform,
            Mesh3d::default(),MeshMaterial3d::<VoxelMaterial>::default())).id();
        app.insert_resource(TerrainRevisionCache{key:Some(TerrainCacheKey::from_frame(&old)),built_frame:Some(old),
            built_source_texture:Some(source),textured_entity:Some(terrain),footing_heights:vec![3.],
            painted_stone:Some(Handle::default()),painted_bedrock:Some(Handle::default()),..default()});
        app.insert_resource(VoxelScene{camera:Some(camera),..default()}).insert_resource(live)
            .add_systems(Update,sync_voxel_view);
        for _ in 0..3 {
            app.update();
            let status=app.world().resource::<VoxelViewStatus>();assert!(status.active,"{:?}",status.inactive_reason);assert!(status.terrain_pending);
            assert_eq!(status.terrain_map.as_deref(),Some("OlivineLighthouse1F"));
            assert_eq!(*app.world().get::<Visibility>(terrain).unwrap(),Visibility::Visible);
            assert!(app.world().get::<Camera>(camera).unwrap().is_active);
            for entity in [terrain,camera,actor] {assert_eq!(*app.world().get::<Transform>(entity).unwrap(),old_transform);}
            assert_eq!(app.world().resource::<TerrainRevisionCache>().footing_heights,vec![3.]);
        }
        app.world_mut().resource_mut::<VisualWorldFrame>().active = false;
        app.update();
        let status = app.world().resource::<VoxelViewStatus>();
        assert!(!status.active);
        assert!(!status.terrain_pending, "inactive output must not leave a stale loading fence");
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
                        Ok(TerrainSyncState::Pending));
                });
            app.update();
            let cache = app.world().resource::<TerrainRevisionCache>();
            assert!(cache.built_frame.is_none());
            assert!(must_wait_for_initial_terrain(
                TerrainSyncState::Pending,
                cache
            ));
            assert!(app.world().get_entity(entity).is_err(),"other room changes still retire the previous geometry");
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
        frame.map_id="CeladonMansionRoof".into();frame.tiles[0].source.tileset_id="mansion".into();
        assert_eq!(voxel_clear_color(&frame),Color::srgb(0.72,0.83,0.78));
        frame.map_id="CeladonMansionRoofHouse".into();
        assert_eq!(voxel_clear_color(&frame),Color::srgb(0.055,0.050,0.045));
    }

    #[test]
    fn anchored_retained_geometry_stays_fixed_through_grid_and_subtile_motion() {
        let raw=VisualWorldFrame {active:true, center:Vec2::new(5.,7.),
            grid_origin:IVec2::new(4,5),tile_size:Vec2::splat(8.),
            viewport_size:Vec2::new(160.,144.),grid_size:UVec2::new(20,18),..default()};
        let mut anchored=raw.clone();anchored.anchor_to_map();
        let cache=TerrainRevisionCache {key:Some(TerrainCacheKey::from_frame(&anchored)),
            built_frame:Some(anchored.clone()),..default()};
        let old_cache=TerrainRevisionCache {key:Some(TerrainCacheKey::from_frame(&raw)),
            built_frame:Some(raw.clone()),..default()};
        let fixed=terrain_transform(&anchored);
        #[derive(Component)] struct StaticDescendant;
        #[derive(Resource,Default)] struct Changes(Vec<usize>);
        let mut app=App::new();
        app.add_plugins((MinimalPlugins,bevy::transform::TransformPlugin))
            .init_resource::<Changes>().add_systems(PostUpdate,
                (|query:Query<(),(With<StaticDescendant>,Changed<GlobalTransform>)>,
                    mut changes:ResMut<Changes>| changes.0.push(query.iter().count()))
                .after(bevy::transform::TransformSystems::Propagate));
        let root_entity=app.world_mut().spawn(fixed).id();
        let child=app.world_mut().spawn((Transform::from_xyz(3.,9.,12.),StaticDescendant)).id();
        app.world_mut().entity_mut(root_entity).add_child(child);
        let camera_entity=app.world_mut().spawn(Transform::default()).id();
        app.update();
        for n in 0..128 {
            let mut live=raw.clone();
            live.grid_origin.x+=n/16;
            live.center.x+=(n/16) as f32*8.-n as f32*0.5;
            let old_root=retained_terrain_transform(&live,&old_cache).unwrap();
            let old_pose=VoxelCameraControls::default().world_pose(&live,&[]);
            let shift=live.anchor_to_map();let delta=Vec3::new(shift.x,0.,-shift.y);
            let root=retained_terrain_transform(&live,&cache).unwrap();
            assert_eq!(root,fixed,"static root must not move at subtiles or grid boundaries");
            assert!(root.translation.abs_diff_eq(old_root.translation+delta,0.0001));
            let pose=VoxelCameraControls::default().world_pose(&live,&[]);
            assert!(pose.eye.abs_diff_eq(old_pose.eye+delta,0.0001));
            assert!(pose.target.abs_diff_eq(old_pose.target+delta,0.0001));
            app.world_mut().get_mut::<Transform>(root_entity).unwrap().set_if_neq(root);
            *app.world_mut().get_mut::<Transform>(camera_entity).unwrap()=pose.transform();
            app.update();
            assert_eq!(app.world().resource::<Changes>().0.last(),Some(&0),
                "camera movement must not dirty retained mesh descendants");
        }
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
        cache.key = Some(TerrainCacheKey { source_regions: Default::default(),
            map_id: Default::default(),
            source_map_size_core_tiles: None,
            jump_ledges: Default::default(),
            boundary_warps: Default::default(),
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
        let key = |revision| TerrainCacheKey { source_regions: Default::default(),
            map_id: Default::default(),
            source_map_size_core_tiles: None,
            jump_ledges: Default::default(),
            boundary_warps: Default::default(),
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
    fn first_ready_frame_uses_newly_published_camera_support() {
        use crystal_render_api::{VisualTile, VisualTileSource};
        let mut app = App::new();
        app.init_resource::<Assets<Image>>().init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<VoxelMaterial>>().init_resource::<live_profiles::LiveProfiles>()
            .init_resource::<VoxelViewSettings>().init_resource::<VoxelViewStatus>()
            .init_resource::<NativeHealingEffectsStatus>().init_resource::<TerrainBuildQueue>()
            .init_resource::<ActorIdCache>();
        app.world_mut().resource_mut::<VoxelViewSettings>().enabled = true;
        let atlas = app.world_mut().resource_mut::<Assets<Image>>().add(Image::default());
        let frame = VisualWorldFrame {
            active: true, map_id: "NewBarkTown".into(), map_texture: atlas.clone(),
            grid_size: UVec2::splat(2), viewport_size: Vec2::splat(16.), tile_size: Vec2::splat(8.),
            tiles: (0..4).map(|i| VisualTile {
                column: i % 2, row: i / 2, texture: atlas.clone(), priority: false, animation_frames: None,
                source: VisualTileSource { tileset_id: "johto".into(), metatile_id: 1, tile_index: 1,
                    subtile_column: (i % 2) as u8, subtile_row: (i / 2) as u8 },
            }).collect(),
            actors: vec![VisualActor { id: VisualActorId::Player, source_id: "player".into(),
                model_palette: None, texture: atlas, center: Vec2::ZERO, size: Vec2::splat(16.),
                flip_x: false, above_priority: false, facing: None }], ..default()
        };
        let expected = VoxelCameraControls::default().world_pose(&frame, &[7.; 4]).transform();
        let camera = app.world_mut().spawn((Camera::default(), Projection::default(),
            Transform::IDENTITY, VoxelWorldCamera)).id();
        let actor_quad = app.world_mut().resource_mut::<Assets<Mesh>>()
            .add(Mesh::from(Rectangle::new(1., 1.)));
        app.insert_resource(TerrainRevisionCache {
            key: Some(TerrainCacheKey::from_frame(&frame)), built_frame: Some(frame.clone()),
            built_footing_heights: vec![7.; 4], ..default()
        }).insert_resource(VoxelScene { camera: Some(camera), actor_quad: Some(actor_quad) })
            .insert_resource(frame).add_systems(Update, sync_voxel_view);
        app.update();
        let status = app.world().resource::<VoxelViewStatus>();
        assert!(status.active, "{:?}", status.inactive_reason);
        assert_eq!(app.world().resource::<TerrainRevisionCache>().footing_heights, vec![7.; 4]);
        assert_eq!(*app.world().get::<Transform>(camera).unwrap(), expected);
        app.update();
        assert_eq!(*app.world().get::<Transform>(camera).unwrap(), expected);
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
            actors: vec![VisualActor { model_palette: None, id: VisualActorId::Player, source_id: "player".into(),
                texture: Handle::default(), center: Vec2::new(0.0, 8.0),
                size: Vec2::splat(16.0), flip_x: false, above_priority: false, facing: None }],
            ..default()
        };
        let mut app = App::new();
        app.init_resource::<Assets<Image>>()
            .init_resource::<crystal_render_api::VisualWorldLighting>()
            .insert_resource(frame).insert_resource(materials)
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
                assert_eq!(capsule.bottom_radius.truncate(), Vec3::new(x, height + 0.04, -8.0));
                assert_eq!(capsule.top_feather.y, height + 0.04 + 21.6);
            }
            assert_eq!(assets.get(&fade).unwrap().extension.cutaway.fade, Vec4::new(0.7,0.2,0.3,0.4));
            assert_eq!(assets.get(&actor).unwrap().extension.cutaway, actor_original);
            assert_eq!(app.world().resource::<TerrainRevisionCache>().footing_heights, vec![height; 16]);
        }
        // The same live palette transition used by Flash hides terrain and
        // disables doorway fill without changing actor geometry or support.
        app.world_mut().resource_mut::<VisualWorldFrame>().tiles = (0..4).map(|i|
            crystal_render_api::VisualTile {
                column: i % 2, row: i / 2, texture: Handle::default(),
                animation_frames: None, priority: false,
                source: crystal_render_api::VisualTileSource {
                    tileset_id: "cave".into(), metatile_id: 0x24,
                    subtile_column: 2 + (i % 2) as u8,
                    subtile_row: 2 + (i / 2) as u8, tile_index: 0x01,
                },
            }).collect();
        app.update();
        assert!(app.world().resource::<Assets<VoxelMaterial>>().get(&solid).unwrap()
            .extension.cave_exits[0].w > 0.0);
        *app.world_mut().resource_mut::<crystal_render_api::VisualWorldLighting>() =
            crystal_render_api::VisualWorldLighting::Dark;
        app.update();
        let assets = app.world().resource::<Assets<VoxelMaterial>>();
        assert_eq!(assets.get(&solid).unwrap().extension.cave_reveal.y, 1.0);
        assert_eq!(assets.get(&solid).unwrap().extension.cave_exits, [Vec4::ZERO; 4]);
        assert_eq!(assets.get(&actor).unwrap().extension.cave_reveal.x, 0.0);
        assert_eq!(assets.get(&actor).unwrap().extension.cutaway, actor_original);
        *app.world_mut().resource_mut::<crystal_render_api::VisualWorldLighting>() =
            crystal_render_api::VisualWorldLighting::Night;
        app.update();
        assert_eq!(app.world().resource::<Assets<VoxelMaterial>>().get(&solid).unwrap()
            .extension.cave_reveal.y, 0.0);
        assert!(app.world().resource::<Assets<VoxelMaterial>>().get(&solid).unwrap()
            .extension.cave_exits[0].w > 0.0);
        app.world_mut().resource_mut::<VoxelViewStatus>().active = false;
        app.update();
        assert_eq!(app.world().resource::<Assets<VoxelMaterial>>().get(&fade).unwrap()
            .extension.cutaway.bottom_radius.w, 0.0);
    }

    #[test]
    fn player_depth_pull_clears_same_level_terrain_at_forty_five_degrees() {
        let player = VisualActor { model_palette: None,
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
        let player = VisualActor { model_palette: None,
            id: VisualActorId::Player,
            source_id: "player".into(),
            texture: texture.clone(),
            center: Vec2::ZERO,
            size: Vec2::splat(16.0),
            flip_x: false,
            above_priority: false,
            facing: None,
        };
        let remote = VisualActor { model_palette: None,
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
        let mut player = VisualActor { model_palette: None,
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
                .init_resource::<Time>()
                .add_systems(Update, native_model_camera);
            app.world_mut().resource_mut::<Time>()
                .advance_by(std::time::Duration::from_secs_f32(0.1));
            {
                let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
                keys.press(KeyCode::KeyE);
                keys.press(KeyCode::PageUp);
            }
            app.update();
            let moved = app.world().resource::<VoxelViewSettings>().camera;
            assert_eq!(moved.zoom_step, 1., "{map}");
            assert!((moved.rotation_step * std::f32::consts::FRAC_PI_4 - 0.25).abs() < 1e-6, "{map}");

            // Holding rotates continuously; zoom still consumes only the edge.
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().clear();
            app.update();
            let held = app.world().resource::<VoxelViewSettings>().camera;
            assert_eq!(held.zoom_step, moved.zoom_step);
            assert!((held.rotation_step * std::f32::consts::FRAC_PI_4 - 0.5).abs() < 1e-6);

            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyQ);
            app.update();
            assert_eq!(app.world().resource::<VoxelViewSettings>().camera, held,
                "opposite rotation keys cancel");

            app.world_mut().resource_mut::<VisualWorldFrame>().active = false;
            {
                let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
                keys.press(KeyCode::KeyQ);
                keys.press(KeyCode::PageDown);
            }
            app.update();
            assert_eq!(app.world().resource::<VoxelViewSettings>().camera, held);
        }
    }
}

/// Native modeled-world controls; browser camera controls remain authoritative there.
#[cfg(not(target_arch = "wasm32"))]
fn native_model_camera(
    keyboard: Res<ButtonInput<KeyCode>>,
    frame: Res<VisualWorldFrame>,
    mut settings: ResMut<VoxelViewSettings>,
    time: Option<Res<Time>>,
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
    let orbit = i32::from(keyboard.pressed(KeyCode::KeyE))
        - i32::from(keyboard.pressed(KeyCode::KeyQ));
    let dt = time.as_ref().map_or(0.0, |time| time.delta_secs().min(0.1));
    camera.steer(orbit as f32 * dt * 2.5, 0.0);
    if keyboard.just_pressed(KeyCode::PageUp) {
        camera.zoom_step += 1.0;
    }
    if keyboard.just_pressed(KeyCode::PageDown) {
        camera.zoom_step -= 1.0;
    }
    let bounded = VoxelCameraControls::new(camera.zoom_step, camera.rotation_step);
    camera.zoom_step = bounded.zoom_step;
    camera.rotation_step = bounded.rotation_step;
    if settings.camera != camera {
        settings.camera = camera;
    }
}
