//! A render-only arena. Presentation cues come from the production shell's
//! retained visible scene; this module has no runtime or battle-engine imports.
use crate::battle_layout::{BattleBody, BattleSceneLayout};
use crate::encounter_terrain::{self, EncounterTerrain};
use crate::{VoxelViewSettings, mesh::SurfaceMeshData};
use bevy::{
    asset::load_internal_asset,
    core_pipeline::{fxaa::Fxaa, tonemapping::Tonemapping},
    pbr::{CascadeShadowConfigBuilder, FogFalloff, FogSettings},
    prelude::*,
    reflect::TypePath,
    render::{
        camera::{CameraUpdateSystem, ClearColorConfig, RenderTarget},
        mesh::skinning::SkinnedMeshInverseBindposes,
        render_asset::RenderAssetUsages,
        render_resource::{
            AsBindGroup, Extent3d, ShaderRef, ShaderType, TextureDimension, TextureFormat,
            TextureUsages,
        },
        renderer::RenderAdapterInfo,
        texture::ImageSampler,
        view::RenderLayers,
    },
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
};
use crystal_render_api::{
    BattleCanvasExtract, BattleFlashMode, VisualBattleBattler, VisualBattleCanvas, VisualBattleCue,
    VisualBattleCueKind, VisualBattleEnvironment, VisualBattleFrame, VisualBattleSide,
    VisualBattleSourceFrame, WorldRenderSet,
};
use std::{
    collections::HashMap,
    f32::consts::{PI, TAU},
    sync::{Arc, OnceLock},
};

const BATTLE_LAYER: usize = 29;
// Both actor views are isolated only during the bounded source row phase.
const BATTLE_ROW_LAYERS: [usize; 2] = [27, 28];
// The arena image is drawn behind the native HUD in its existing pass.
// Layer 30 parks classic sprites, and 31 is the modeled overworld.
const BATTLE_COMPOSITE_LAYER: usize = 0;
const MODEL_SCALE: f32 = crate::battle_layout::WORLD_UNITS_PER_METER;
const FALLBACK_CARD_HEIGHT: f32 = 1.9;
const SOURCE_PIXEL_WORLD: f32 = crate::battle_layout::SOURCE_PIXEL_WORLD;
const BATTLE_CAMERA_FOV: f32 = crate::battle_layout::CAMERA_FOV;
const PARTICLES: usize = 32;

#[path = "battle_capture_bridge.rs"]
mod capture_bridge;
#[cfg(test)]
#[path = "battle_capture_tiles.rs"]
mod capture_tiles;
#[path = "battle_row_capture.rs"]
mod row_capture;
#[path = "battle_skinning.rs"]
mod skinning;

#[derive(Default)]
pub struct BattleViewPlugin;
impl Plugin for BattleViewPlugin {
    fn build(&self, app: &mut App) {
        if app.world().contains_resource::<AssetServer>() {
            app.add_plugins(Material2dPlugin::<BattleCompositeMaterial>::default());
            if let Some(render_app) = app.get_sub_app_mut(bevy::render::RenderApp) {
                // Bevy 0.14's Material2d plugin has no image preparation
                // dependency. A resized image must exist before rebinding it.
                render_app.add_systems(
                    bevy::render::Render,
                    battle_composite_images_ready
                        .in_set(bevy::render::RenderSet::PrepareAssets)
                        .after(
                            bevy::render::render_asset::prepare_assets::<
                                bevy::render::texture::GpuImage,
                            >,
                        )
                        .before(
                            bevy::render::render_asset::prepare_assets::<
                                bevy::sprite::PreparedMaterial2d<BattleCompositeMaterial>,
                            >,
                        ),
                );
            }
        } else {
            // Minimal headless renderer tests have no asset server/render app.
            app.init_resource::<Assets<BattleCompositeMaterial>>();
        }
        app.init_resource::<Assets<Shader>>()
            .init_resource::<Assets<SkinnedMeshInverseBindposes>>();
        load_internal_asset!(
            app,
            BATTLE_COMPOSITE_SHADER,
            "battle_composite.wgsl",
            Shader::from_wgsl
        );
        app.init_resource::<VisualBattleFrame>()
            .init_resource::<crystal_render_api::BattleLocationFrame>()
            .init_resource::<EncounterTerrain>()
            .init_resource::<BattleFlashMode>()
            .init_resource::<VisualBattleCanvas>()
            .init_resource::<BattleViewStatus>()
            .init_resource::<BattleScene>()
            .init_resource::<BattleSceneLayout>()
            .init_resource::<capture_bridge::BattleCaptureBridge>()
            .add_systems(Startup, setup_battle_scene)
            .add_systems(
                PostUpdate,
                trace_battle_visibility
                    .after(bevy::render::view::VisibilitySystems::CheckVisibility),
            )
            .add_systems(
                PostUpdate,
                sync_battle_target
                    .after(BattleCanvasExtract)
                    .before(CameraUpdateSystem)
                    // These views start inactive. Activating after AddClusters
                    // leaves their first render without light uniforms and
                    // SetMeshViewBindGroup panics on the missing view query.
                    .before(bevy::pbr::SimulationLightSystems::AddClusters),
            )
            .add_systems(
                PostUpdate,
                sync_battle_composite
                    .after(CameraUpdateSystem)
                    .before(bevy::transform::TransformSystem::TransformPropagate),
            )
            .add_systems(
                Update,
                (
                    toggle_battle_flash_mode,
                    encounter_terrain::prepare,
                    sync_battle_layout,
                    capture_bridge::select_capture_bridge,
                    sync_battle_scene,
                    encounter_terrain::sync,
                    skinning::sync,
                    sync_source_objects,
                    sync_modeled_source_effects,
                )
                    .chain()
                    .after(crate::toggle_voxel_view)
                    .after(crate::sync_voxel_view)
                    .in_set(WorldRenderSet::RenderSync),
            );
        app.add_plugins(row_capture::BattleRowCapturePlugin);
    }
}

/// Explicit diagnostics distinguish exact modeled species from source-art
/// fallback; support for an icon family never claims a modeled Pokédex entry.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct BattleViewStatus {
    pub active: bool,
    pub active_frames: u32,
    pub modeled_species: Vec<String>,
    pub source_art_species: Vec<String>,
    pub last_error: Option<String>,
    pub lighting: &'static str,
    pub quality: &'static str,
    pub software_renderer: bool,
    pub render_size: UVec2,
    pub output_size: UVec2,
    /// Current actor targets have a successful draw and output submission.
    pub row_capture_ready: [bool; 2],
    /// Actual capture requests still waiting for that receipt, never a timer.
    pub row_capture_pending: [bool; 2],
    pub capture_image_active: bool,
    pub capture_fallback_reason: Option<&'static str>,
}
#[derive(Component)]
struct BattleCamera;
#[derive(Component)]
struct BattleCompositeSprite;
#[derive(Component)]
struct BattleRowCompositeSprite;
#[derive(Component)]
struct BattleRowCamera(usize);

const BATTLE_COMPOSITE_SHADER: Handle<Shader> =
    Handle::weak_from_u128(0x519a630f_8cf7_4303_bb48_30dbdbefb74d);

#[derive(Clone, Debug, PartialEq, ShaderType)]
struct BattleCompositeUniform {
    // Vec4 gives each LCD row a 16-byte aligned uniform stride on WebGL too.
    rows: [Vec4; 96],
    background: Vec4,
    screen_offset: Vec4,
    source_to_view: [Vec4; 3],
    view_to_source: [Vec4; 3],
    // Source Y start/end, BG-cleared flag, active flag. No renderer clock.
    battler_rows: [Vec4; 2],
    actor_view_rects: [Vec4; 2],
    actor_source_rects: [Vec4; 2],
    // Shown tiles (zero hides), source tiles, image-enabled flag; no clock.
    capture_enemy: Vec4,
    capture_slot: Vec4,
}
impl Default for BattleCompositeUniform {
    fn default() -> Self {
        Self {
            rows: [Vec4::ZERO; 96],
            background: Vec4::ONE,
            screen_offset: Vec4::ZERO,
            source_to_view: [Vec4::X, Vec4::Y, Vec4::Z],
            view_to_source: [Vec4::X, Vec4::Y, Vec4::Z],
            battler_rows: [Vec4::ZERO; 2],
            actor_view_rects: [Vec4::ZERO; 2],
            actor_source_rects: [Vec4::ZERO; 2],
            capture_enemy: Vec4::ZERO,
            capture_slot: Vec4::ZERO,
        }
    }
}
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
struct BattleCompositeMaterial {
    #[uniform(0)]
    source: BattleCompositeUniform,
    #[texture(1)]
    #[sampler(2)]
    texture: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    player_rows: Handle<Image>,
    #[texture(5)]
    #[sampler(6)]
    enemy_rows: Handle<Image>,
}
impl Material2d for BattleCompositeMaterial {
    fn fragment_shader() -> ShaderRef {
        BATTLE_COMPOSITE_SHADER.into()
    }
}

fn battle_composite_uniform(
    layout: &BattleSceneLayout,
    frame: &VisualBattleFrame,
    mode: BattleFlashMode,
    viewport: Vec2,
) -> BattleCompositeUniform {
    let mut out = BattleCompositeUniform::default();
    if let Some(capture) = frame.capture.as_ref().filter(|capture| capture.presented)
        && let Some(enemy) = frame.battlers[1].as_ref()
    {
        out.capture_enemy = Vec4::new(
            f32::from(capture_bridge::picture_tiles(capture)),
            7.0,
            1.0,
            0.0,
        );
        out.capture_slot = Vec4::new(
            enemy.source_rect.min.x,
            enemy.source_rect.min.y,
            enemy.source_rect.max.x,
            enemy.source_rect.max.y,
        );
    }
    let Some(source) = &frame.source else {
        return out;
    };
    for row in 0..95 {
        out.rows[row].x = source
            .line_x_offsets
            .as_ref()
            .map_or(0.0, |lines| f32::from(lines[row]));
        out.rows[row].y = source
            .line_y_offsets
            .as_ref()
            .map_or(0.0, |lines| f32::from(lines[row]));
    }
    let (dark, light) = source_environment_palette(Some(source), mode);
    out.background = rgb(palette(frame.environment).sky)
        .mix(&Color::BLACK, dark)
        .mix(&Color::WHITE, light)
        .to_linear()
        .to_vec4();
    for (index, rows) in source.battler_rows.iter().enumerate() {
        if let Some(rows) = rows {
            out.battler_rows[index] = Vec4::new(
                rows.source_y.x,
                rows.source_y.y,
                if rows.bg_cleared { 1.0 } else { 0.0 },
                1.0,
            );
        }
    }
    // Presentation offsets are Y up, while the LCD sampler is Y down.
    let scrolling = source.screen_offset != Vec2::ZERO
        || out.rows.iter().any(|row| row.x != 0.0 || row.y != 0.0);
    out.screen_offset = Vec4::new(
        source.screen_offset.x,
        -source.screen_offset.y,
        if scrolling { 1.0 } else { 0.0 },
        0.0,
    );
    if scrolling || source.battler_rows.iter().any(Option::is_some) {
        let projection = source_plane_projection(layout, viewport);
        out.source_to_view = projection
            .to_cols_array_2d()
            .map(|column| Vec3::from_array(column).extend(0.0));
        out.view_to_source = projection
            .inverse()
            .to_cols_array_2d()
            .map(|column| Vec3::from_array(column).extend(0.0));
    }
    out
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum BattleRenderQuality {
    #[default]
    Automatic,
    Native,
    Balanced,
    Performance,
}
impl BattleRenderQuality {
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    fn from_name(name: &str) -> Self {
        match name {
            "balanced" => Self::Balanced,
            "performance" => Self::Performance,
            "native" => Self::Native,
            _ => Self::Automatic,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Automatic => "auto",
            Self::Native => "native",
            Self::Balanced => "balanced",
            Self::Performance => "performance",
        }
    }
    fn target_size(self, output: UVec2) -> UVec2 {
        let numerator = match self {
            Self::Automatic | Self::Native => 4,
            Self::Balanced => 3,
            Self::Performance => 2,
        };
        // Integer extents remain stable between actual resizes. This also
        // bounds unusually large/minimized window allocations.
        let width = (u64::from(output.x) * numerator / 4).max(1);
        let height = (u64::from(output.y) * numerator / 4).max(1);
        let divisor = width.max(height).max(4096);
        UVec2::new(
            (width * 4096 / divisor).max(1) as u32,
            (height * 4096 / divisor).max(1) as u32,
        )
    }
}

fn resolve_battle_profile(
    software_renderer: bool,
    requested_quality: BattleRenderQuality,
    requested_lighting: Option<&str>,
) -> (BattleRenderQuality, bool) {
    let quality = match requested_quality {
        BattleRenderQuality::Automatic if software_renderer => BattleRenderQuality::Balanced,
        BattleRenderQuality::Automatic => BattleRenderQuality::Native,
        explicit => explicit,
    };
    let vertex_lighting = match requested_lighting {
        Some("vertex") => true,
        Some("pbr") => false,
        _ => software_renderer,
    };
    (quality, vertex_lighting)
}

#[derive(Component)]
struct BattleLight;
#[derive(Component)]
struct BattleArena;
#[derive(Component)]
struct BattleContactShadow(VisualBattleSide);
#[derive(Component)]
struct BattleActor;
#[derive(Component)]
struct BattleParticle;
#[derive(Component)]
struct BattleBall;
#[derive(Component)]
struct BattleSourceObject(usize);
#[derive(Component)]
struct BattleModeledSourceEffect(usize);
#[derive(Clone, PartialEq)]
struct ActorKey {
    species: Arc<str>,
    party_index: Option<usize>,
    modeled: bool,
}
struct ActorInstance {
    entity: Entity,
    key: ActorKey,
    material: Handle<StandardMaterial>,
    mesh: Option<Handle<Mesh>>,
    palette_key: (u8, BattleFlashMode),
    palette_colors: Option<[[f32; 4]; 4]>,
    neutral_colors: Option<Vec<[f32; 4]>>,
    base_pose: Transform,
    source_rect: Rect,
    source_opaque_rect: Rect,
    projected_registration: Option<(Vec2, Transform, Rect)>,
    animated: Option<AnimatedActor>,
    skinned: Option<skinning::SkinnedActor>,
}

struct AnimatedPart {
    entity: Entity,
    mesh: Handle<Mesh>,
    neutral_colors: Option<Vec<[f32; 4]>>,
}

struct AnimatedActor {
    rig: &'static crate::pidgeotto_rig::PidgeottoRig,
    wings: [AnimatedPart; 2],
    elapsed: f32,
}

impl ActorInstance {
    fn is_articulated(&self) -> bool {
        self.animated.is_some() || self.skinned.is_some()
    }

    fn retire(
        self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) {
        commands.entity(self.entity).despawn_recursive();
        materials.remove(self.material.id());
        if let Some(mesh) = self.mesh {
            meshes.remove(mesh.id());
        }
        if let Some(animated) = self.animated {
            for part in animated.wings {
                meshes.remove(part.mesh.id());
            }
        }
    }
}

fn animated_species(species: &str) -> Option<&'static crate::pidgeotto_rig::PidgeottoRig> {
    species
        .eq_ignore_ascii_case("PIDGEOTTO")
        .then(crate::pidgeotto_rig::rig)
}

fn actor_mesh(
    data: &SurfaceMeshData,
    rotation: Quat,
    vertex_lighting: bool,
    meshes: &mut Assets<Mesh>,
) -> (Handle<Mesh>, Option<Vec<[f32; 4]>>) {
    let neutral = vertex_lighting.then(|| vertex_lit_colors(data, rotation));
    let mut mesh = data.clone().into_mesh();
    if let Some(colors) = &neutral {
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors.clone());
    }
    mesh.asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
    (meshes.add(mesh), neutral)
}

fn apply_actor_palette(
    mesh: &mut Mesh,
    data: &SurfaceMeshData,
    neutral: Option<&[[f32; 4]]>,
    source: Option<&VisualBattleSourceFrame>,
    side: usize,
    bgp: u8,
    flash_mode: BattleFlashMode,
) {
    let colors = data
        .colors
        .iter()
        .enumerate()
        .map(|(index, color)| {
            let neutral = neutral.map_or(*color, |colors| colors[index]);
            source.map_or(neutral, |source| {
                let mapped = source_model_color(
                    *color,
                    bgp,
                    &source.battler_palettes[side],
                    BattleFlashMode::Full,
                );
                source_lit_color(neutral, mapped, bgp, flash_mode)
            })
        })
        .collect::<Vec<_>>();
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
}

/// Species geometry is immutable. Scan it once when loading that species,
/// then reuse these bounds for every source-frontpic footprint and display frame.
#[derive(Clone, Copy, Debug, PartialEq)]
struct BattleModelBounds {
    min: Vec3,
    max: Vec3,
}
impl BattleModelBounds {
    fn from_surface(model: &SurfaceMeshData) -> Self {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for point in &model.positions {
            let point = Vec3::from_array(*point);
            min = min.min(point);
            max = max.max(point);
        }
        Self { min, max }
    }
}

#[derive(Resource, Default)]
struct BattleScene {
    arena: Option<Entity>,
    arena_key: Option<VisualBattleEnvironment>,
    arena_mesh: Option<Handle<Mesh>>,
    arena_neutral_colors: Vec<[f32; 4]>,
    surface_white_strength: Option<f32>,
    actors: [Option<ActorInstance>; 2],
    species_meshes: HashMap<Arc<str>, Option<Arc<SurfaceMeshData>>>,
    species_bounds: HashMap<Arc<str>, BattleModelBounds>,
    skin_inverse_binds: HashMap<&'static str, Handle<SkinnedMeshInverseBindposes>>,
    surface_material: Handle<StandardMaterial>,
    fallback_mesh: Handle<Mesh>,
    particle_mesh: Handle<Mesh>,
    particle_materials: Vec<Handle<StandardMaterial>>,
    particles: Vec<Entity>,
    ball: Option<Entity>,
    ball_mesh: Handle<Mesh>,
    ball_neutral_colors: Vec<[f32; 4]>,
    elapsed: f32,
    was_active: bool,
    source_effect_meshes: Vec<Handle<Mesh>>,
    vertex_lighting: bool,
    quality: BattleRenderQuality,
    software_renderer: bool,
    render_target: Option<Handle<Image>>,
    render_size: UVec2,
    row_targets: [Handle<Image>; 2],
    row_sizes: [UVec2; 2],
}
#[derive(Clone, serde::Deserialize)]
struct ArenaPalette {
    ground: [f32; 4],
    stage: [f32; 4],
    edge: [f32; 4],
    accent: [f32; 4],
    sky: [f32; 4],
}
fn palette(environment: VisualBattleEnvironment) -> &'static ArenaPalette {
    static PALETTES: OnceLock<HashMap<String, ArenaPalette>> = OnceLock::new();
    let all = PALETTES.get_or_init(|| {
        serde_json::from_str(include_str!("../../../art/battles/arena-palettes.json"))
            .expect("verified authored arena palette")
    });
    &all[match environment {
        VisualBattleEnvironment::Meadow => "meadow",
        VisualBattleEnvironment::Forest => "forest",
        VisualBattleEnvironment::Cave => "cave",
        VisualBattleEnvironment::Water => "water",
        VisualBattleEnvironment::Interior => "interior",
        VisualBattleEnvironment::Ice => "ice",
    }]
}
fn rgb(c: [f32; 4]) -> Color {
    Color::srgba(c[0], c[1], c[2], c[3])
}
fn side_position(side: VisualBattleSide) -> Vec3 {
    match side {
        VisualBattleSide::Player => Vec3::new(-2.1, 0.12, 1.5),
        VisualBattleSide::Enemy => Vec3::new(2.1, 0.12, -1.5),
    }
}
fn camera_pose() -> Transform {
    // Keep the immersive three-quarter framing fixed while an interpreter
    // sample is held. Only source screen offsets may move the battle image.
    Transform::from_xyz(7.8, 6.3, 11.6).looking_at(Vec3::new(0.0, 0.80, 0.0), Vec3::Y)
}
/// Prepare both immutable neutral bodies before fitting one shared camera.
fn sync_battle_layout(
    frame: Res<VisualBattleFrame>,
    canvas: Res<VisualBattleCanvas>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut scene: ResMut<BattleScene>,
    mut layout: ResMut<BattleSceneLayout>,
    mut scenery: ResMut<EncounterTerrain>,
    mut previous: Local<Option<([Option<BattleBody>; 2], Option<[Option<BattleBody>; 2]>, Vec2, (Option<u64>, bool))>>,
) {
    if !frame.active || frame.use_source_scene || frame.validate().is_err() {
        return;
    }
    let viewport = windows
        .get_single()
        .map_or(canvas.physical_size.as_vec2(), |window| {
            Vec2::new(
                window.physical_width() as f32,
                window.physical_height() as f32,
            )
        });
    let bodies = std::array::from_fn(|index| {
        let battler = frame.battlers[index].as_ref()?;
        if battler.allow_species_model
            && battler
                .pokedex_size_m
                .is_some_and(|v| v.is_finite() && v > 0.0)
        {
            if !scene.species_meshes.contains_key(&battler.species_id) {
                let mesh = crate::battle_species_models::mesh(&battler.species_id)
                    .or_else(|| {
                        crate::new_bark_actors::actor_props::battle_species_mesh(
                            &battler.species_id,
                        )
                    })
                    .map(Arc::new);
                if let Some(data) = &mesh {
                    scene.species_bounds.insert(
                        battler.species_id.clone(),
                        BattleModelBounds::from_surface(data),
                    );
                }
                scene
                    .species_meshes
                    .insert(battler.species_id.clone(), mesh);
            }
            if let Some(bounds) = scene.species_bounds.get(&battler.species_id) {
                if let Some(scale) = crate::battle_layout::model_scale(
                    &battler.species_id,
                    battler.pokedex_size_m,
                    bounds.min,
                    bounds.max,
                ) {
                    let mut body =
                        BattleBody::modeled(&battler.species_id, bounds.min, bounds.max, scale);
                    if let Some(rig) = animated_species(&battler.species_id) {
                        // Fit every wing pose once without changing the neutral
                        // height used for canonical size, feet or hit anchors.
                        body.visual_bounds = Some(rig.animated_bounds);
                    }
                    if let Some(rig) = crate::species_rig::for_species(&battler.species_id) {
                        body.visual_bounds = Some(rig.animated_bounds);
                    }
                    return Some(body);
                }
            }
        }
        Some(BattleBody::source_card(
            battler.texture_size.x / battler.texture_size.y,
        ))
    });
    let supported = scenery.supported_bodies(bodies,
        std::array::from_fn(|index| frame.battlers[index].as_ref().map(|b| b.species_id.as_ref())));
    let key = (bodies, supported, viewport, scenery.layout_key());
    if previous.as_ref() != Some(&key) {
        let anchored = scenery.anchors().and_then(|anchors| {
            let Some(supported) = supported else {
                scenery.reject_layout("battler lacks an authored neutral water stance");
                return None;
            };
            scenery.reject_layout("battlers do not fit the checked encounter anchors");
            BattleSceneLayout::for_anchored_bodies(supported, anchors, viewport)
                .map(|layout| (layout, supported))
        });
        let next = if let Some((mut anchored, supported)) = anchored {
            if scenery.constrain_layout(&mut anchored, supported, viewport) {
                anchored
            } else {
                BattleSceneLayout::for_bodies(bodies, viewport)
            }
        } else {
            BattleSceneLayout::for_bodies(bodies, viewport)
        };
        #[cfg(not(target_arch = "wasm32"))]
        if std::env::var_os("CRYSTAL_ENCOUNTER_TRACE").is_some() {
            eprintln!(
                "encounter terrain: generation={:?} accepted={} reason={:?} anchors={:?}",
                scenery.generation(),
                scenery.accepted(),
                scenery.reason(),
                scenery.anchors()
            );
        }
        layout.set_if_neq(next);
        *previous = Some(key);
    }
}

fn layout_actor_pose(
    battler: &VisualBattleBattler,
    bounds: Option<BattleModelBounds>,
    layout: &BattleSceneLayout,
) -> Transform {
    layout.body_poses[battler.side.index()]
        .unwrap_or_else(|| actor_pose(battler, &[], 0.0, bounds, None))
}

fn setup_battle_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut composite_materials: ResMut<Assets<BattleCompositeMaterial>>,
    mut scene: ResMut<BattleScene>,
    adapter: Option<Res<RenderAdapterInfo>>,
) {
    scene.software_renderer = adapter
        .as_ref()
        .is_some_and(|adapter| crate::software_renderer(&adapter.name));
    #[cfg(not(target_arch = "wasm32"))]
    let (requested_quality, requested_lighting) = (
        std::env::var("CRYSTAL_BATTLE_QUALITY")
            .ok()
            .map_or(scene.quality, |quality| {
                BattleRenderQuality::from_name(&quality)
            }),
        std::env::var("CRYSTAL_BATTLE_LIGHTING").ok(),
    );
    #[cfg(target_arch = "wasm32")]
    let (requested_quality, requested_lighting) = (scene.quality, None::<String>);
    // Adapter-based defaults apply to the production native and browser
    // renderers. Explicit native overrides remain useful for matched A/Bs.
    let (quality, vertex_lighting) = resolve_battle_profile(
        scene.software_renderer,
        requested_quality,
        requested_lighting.as_deref(),
    );
    scene.quality = quality;
    scene.vertex_lighting = vertex_lighting;
    scene.row_targets = std::array::from_fn(|_| images.add(battle_row_target_image(UVec2::ONE)));
    scene.row_sizes = [UVec2::ONE; 2];
    for index in 0..2 {
        commands.spawn((
            Camera3dBundle {
                camera: Camera {
                    order: -2,
                    target: RenderTarget::Image(scene.row_targets[index].clone()),
                    is_active: false,
                    clear_color: ClearColorConfig::Custom(Color::NONE),
                    output_mode: bevy::render::camera::CameraOutputMode::Write {
                        blend_state: Some(bevy::render::render_resource::BlendState::REPLACE),
                        clear_color: ClearColorConfig::Custom(Color::NONE),
                    },
                    ..default()
                },
                exposure: bevy::render::camera::Exposure { ev100: 12.0 },
                projection: Projection::Perspective(PerspectiveProjection {
                    fov: BATTLE_CAMERA_FOV,
                    near: 0.1,
                    far: 100.0,
                    ..default()
                }),
                transform: camera_pose(),
                tonemapping: Tonemapping::AcesFitted,
                ..default()
            },
            RenderLayers::layer(BATTLE_ROW_LAYERS[index]),
            BattleRowCamera(index),
        ));
    }
    {
        let image = images.add(battle_target_image(UVec2::ONE));
        scene.render_size = UVec2::ONE;
        scene.render_target = Some(image.clone());
        let material = composite_materials.add(BattleCompositeMaterial {
            source: BattleCompositeUniform::default(),
            texture: image,
            player_rows: scene.row_targets[0].clone(),
            enemy_rows: scene.row_targets[1].clone(),
        });
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(battle_composite_mesh(false)).into(),
                material: material.clone(),
                visibility: Visibility::Hidden,
                ..default()
            },
            RenderLayers::layer(BATTLE_COMPOSITE_LAYER),
            BattleCompositeSprite,
        ));
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(battle_composite_mesh(true)).into(),
                material,
                visibility: Visibility::Hidden,
                ..default()
            },
            RenderLayers::layer(BATTLE_COMPOSITE_LAYER),
            BattleRowCompositeSprite,
        ));
    }

    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                order: -1,
                target: scene
                    .render_target
                    .as_ref()
                    .map_or_else(RenderTarget::default, |image| {
                        RenderTarget::Image(image.clone())
                    }),
                is_active: false,
                clear_color: ClearColorConfig::Custom(rgb(palette(
                    VisualBattleEnvironment::Meadow,
                )
                .sky)),
                ..default()
            },
            exposure: bevy::render::camera::Exposure { ev100: 12.0 },
            projection: Projection::Perspective(PerspectiveProjection {
                fov: BATTLE_CAMERA_FOV,
                near: 0.1,
                far: 100.0,
                ..default()
            }),
            transform: camera_pose(),
            tonemapping: Tonemapping::AcesFitted,
            ..default()
        },
        RenderLayers::layer(BATTLE_LAYER),
        BattleCamera,
        FogSettings {
            color: rgb(palette(VisualBattleEnvironment::Meadow).sky),
            falloff: FogFalloff::Linear {
                start: 19.0,
                end: 39.0,
            },
            ..default()
        },
        Fxaa {
            enabled: true,
            ..default()
        },
    ));
    #[cfg(not(target_arch = "wasm32"))]
    let hardware_shadows = std::env::var("CRYSTAL_BATTLE_SHADOWS").is_ok_and(|value| value == "on");
    #[cfg(target_arch = "wasm32")]
    let hardware_shadows = false;
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 10_000.0,
                shadows_enabled: hardware_shadows,
                color: Color::srgb(1.0, 0.94, 0.82),
                shadow_depth_bias: 0.015,
                shadow_normal_bias: 0.6,
                ..default()
            },
            transform: Transform::from_xyz(6.0, 12.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
            cascade_shadow_config: CascadeShadowConfigBuilder {
                num_cascades: 1,
                maximum_distance: 40.0,
                first_cascade_far_bound: 40.0,
                ..default()
            }
            .build(),
            visibility: Visibility::Hidden,
            ..default()
        },
        RenderLayers::from_layers(&[BATTLE_LAYER, BATTLE_ROW_LAYERS[0], BATTLE_ROW_LAYERS[1]]),
        BattleLight,
    ));
    scene.surface_material = materials.add(StandardMaterial {
        perceptual_roughness: 0.96,
        reflectance: 0.1,
        unlit: scene.vertex_lighting,
        ..default()
    });
    scene.fallback_mesh = meshes.add(Rectangle::new(FALLBACK_CARD_HEIGHT, FALLBACK_CARD_HEIGHT));
    scene.particle_mesh = meshes.add(
        Sphere::new(1.0)
            .mesh()
            .ico(1)
            .expect("valid low-poly particle"),
    );
    for element in [
        "NORMAL", "FIRE", "WATER", "GRASS", "ELECTRIC", "ICE", "PSYCHIC", "POISON",
    ] {
        let color = element_color(element);
        let material = materials.add(StandardMaterial {
            base_color: rgb(color),
            unlit: true,
            ..default()
        });
        scene.particle_materials.push(material);
    }
    for _ in 0..PARTICLES {
        let entity = commands
            .spawn((
                PbrBundle {
                    mesh: scene.particle_mesh.clone(),
                    material: scene.particle_materials[0].clone(),
                    visibility: Visibility::Hidden,
                    ..default()
                },
                RenderLayers::layer(BATTLE_LAYER),
                BattleParticle,
            ))
            .id();
        scene.particles.push(entity);
    }
    let mut ball_mesh = capture_ball_mesh();
    if scene.vertex_lighting {
        ball_mesh.colors = vertex_lit_colors(&ball_mesh, Quat::IDENTITY);
        scene.ball_neutral_colors = ball_mesh.colors.clone();
    }
    let mut ball_mesh = ball_mesh.into_mesh();
    if scene.vertex_lighting {
        ball_mesh.asset_usage = bevy::render::render_asset::RenderAssetUsages::all();
    }
    scene.ball_mesh = meshes.add(ball_mesh);
    let ball = commands
        .spawn((
            PbrBundle {
                mesh: scene.ball_mesh.clone(),
                material: scene.surface_material.clone(),
                visibility: Visibility::Hidden,
                ..default()
            },
            RenderLayers::layer(BATTLE_LAYER),
            BattleBall,
        ))
        .id();
    scene.ball = Some(ball);
    let object_mesh = meshes.add(Rectangle::new(1.0, 1.0));
    for slot in 0..10 {
        commands.spawn((
            PbrBundle {
                mesh: object_mesh.clone(),
                material: materials.add(StandardMaterial {
                    alpha_mode: AlphaMode::Mask(0.1),
                    unlit: true,
                    cull_mode: None,
                    ..default()
                }),
                visibility: Visibility::Hidden,
                ..default()
            },
            RenderLayers::layer(BATTLE_LAYER),
            BattleSourceObject(slot),
            bevy::pbr::NotShadowCaster,
        ));
    }
    // A bounded companion pool adds depth to the currently visible source
    // objects. Its clock, lifetimes and positions remain the interpreter's.
    let mut beam = SurfaceMeshData::default();
    append_cylinder(&mut beam, Vec3::Y * -0.5, 1.0, 1.0, 16, [1.0; 4]);
    scene.source_effect_meshes = vec![
        meshes.add(source_wave_ring_mesh().into_mesh()),
        meshes.add(beam.into_mesh()),
        scene.particle_mesh.clone(),
    ];
    for slot in 0..10 {
        commands.spawn((
            PbrBundle {
                mesh: scene.source_effect_meshes[0].clone(),
                material: materials.add(StandardMaterial {
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    cull_mode: None,
                    ..default()
                }),
                visibility: Visibility::Hidden,
                ..default()
            },
            RenderLayers::layer(BATTLE_LAYER),
            BattleModeledSourceEffect(slot),
            bevy::pbr::NotShadowCaster,
            bevy::pbr::NotShadowReceiver,
        ));
    }
    let contact_mesh = meshes.add(contact_shadow_mesh().into_mesh());
    let contact_material = materials.add(StandardMaterial {
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });
    for side in [VisualBattleSide::Player, VisualBattleSide::Enemy] {
        commands.spawn((
            PbrBundle {
                mesh: contact_mesh.clone(),
                material: contact_material.clone(),
                visibility: Visibility::Hidden,
                ..default()
            },
            RenderLayers::layer(BATTLE_LAYER),
            BattleContactShadow(side),
            bevy::pbr::NotShadowCaster,
            bevy::pbr::NotShadowReceiver,
        ));
    }
}

fn battle_target_image(size: UVec2) -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::all(),
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    image.sampler = ImageSampler::linear();
    image
}

fn battle_composite_mesh(row_overlay: bool) -> Mesh {
    let mut mesh = Mesh::from(Rectangle::new(1.0, 1.0));
    // Per-quad vertex color selects the pass, retaining ONE stable material.
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        vec![[if row_overlay { 1.0 } else { 0.0 }, 0.0, 0.0, 1.0]; 4],
    );
    mesh
}

fn battle_row_target_image(size: UVec2) -> Image {
    let mut image = battle_target_image(size);
    image
        .data
        .chunks_exact_mut(4)
        .for_each(|pixel| pixel[3] = 0);
    image
}

fn sync_battle_target(
    layout: Res<BattleSceneLayout>,
    canvas: Res<VisualBattleCanvas>,
    frame: Res<VisualBattleFrame>,
    captures: Res<row_capture::ActorCaptures>,
    capture_bridge: Res<capture_bridge::BattleCaptureBridge>,
    mut scene: ResMut<BattleScene>,
    mut status: ResMut<BattleViewStatus>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<BattleCompositeMaterial>>,
    composites: Query<&Handle<BattleCompositeMaterial>, With<BattleCompositeSprite>>,
    mut battle_cameras: Query<
        (&mut Camera, &mut Projection),
        (With<BattleCamera>, Without<BattleRowCamera>),
    >,
    mut row_cameras: Query<
        (
            &BattleRowCamera,
            &mut Camera,
            &mut Projection,
            &mut Transform,
        ),
        Without<BattleCamera>,
    >,
) {
    let output = windows.get_single().map_or(UVec2::ZERO, |window| {
        UVec2::new(window.physical_width(), window.physical_height())
    });
    status.output_size = output;
    status.lighting = if scene.vertex_lighting {
        "vertex"
    } else {
        "pbr"
    };
    status.quality = scene.quality.name();
    status.software_renderer = scene.software_renderer;
    let canvas_pixels = if canvas.physical_size.min_element() > 0 {
        canvas.physical_size
    } else {
        output
    };
    status.render_size = scene.quality.target_size(canvas_pixels);
    let active = status.active && output.min_element() > 0;
    if !active && scene.render_target.is_some() {
        status.render_size = scene.render_size;
    }
    for (row, mut camera, mut projection, mut transform) in &mut row_cameras {
        transform.set_if_neq(layout.camera);
        // An unconditional mutable dereference marks Projection changed even
        // when the far plane is stable, rerunning Bevy's camera/frustum work.
        if matches!(&*projection, Projection::Perspective(p) if p.far != layout.far)
            && let Projection::Perspective(p) = &mut *projection
        {
            p.far = layout.far;
        }
        let index = row.0;
        // A settled image lease has no pending writer. Pin its GPU view and
        // dimensions through Hidden/EnterMon; do not clear or resize it here.
        if index == 1 && capture_bridge.image_active() {
            camera.is_active = false;
            continue;
        }
        let row_active = active
            && (battle_has_rows(&frame)
                || (captures.prewarm_actor(&frame, index)
                    && scene.actors[index]
                        .as_ref()
                        .is_some_and(|actor| actor.key.modeled && !actor.is_articulated())));
        camera.is_active = row_active;
        if row_active && scene.row_sizes[index] != status.render_size {
            let target = scene.row_targets[index].clone();
            images.insert(target.id(), battle_row_target_image(status.render_size));
            // Stable IDs get fresh GPU views. Invalidate the material binding,
            // even for a held frame or a zero-scroll first extraction tick.
            for handle in &composites {
                if let Some(material) = materials.get_mut(handle) {
                    if index == 0 {
                        material.player_rows = target.clone();
                    } else {
                        material.enemy_rows = target.clone();
                    }
                }
            }
            projection.set_changed();
            scene.row_sizes[index] = status.render_size;
        }
    }
    let Some(target) = scene.render_target.clone() else {
        return;
    };
    for (mut camera, _) in &mut battle_cameras {
        if camera.is_active != active {
            camera.is_active = active;
        }
    }
    if active && scene.render_size != status.render_size {
        // Replace only this target's image contents; the stable handle keeps
        // the composite sprite bound across resize/fullscreen transitions.
        images.insert(target.id(), battle_target_image(status.render_size));
        // Image replacement preserves its asset ID but creates a new GPU
        // texture view. Invalidate the composite bind group even when source
        // uniforms remain neutral; otherwise it samples the retired 1x1 sky.
        for handle in &composites {
            if let Some(material) = materials.get_mut(handle) {
                material.texture = target.clone();
            }
        }
        // Image asset events are flushed in Last, after camera dimensions are
        // updated. Marking Projection changed makes CameraUpdateSystem read
        // the new image extent this frame rather than rendering a stale size.
        for (_, mut projection) in &mut battle_cameras {
            projection.set_changed();
        }
        scene.render_size = status.render_size;
    }
}

fn battle_composite_pose(
    projection: &OrthographicProjection,
    camera: &Transform,
) -> (Vec2, Transform) {
    // Fill the actual window through the existing native HUD pass. Following
    // its camera avoids applying source screen movement a second time.
    let depth = -projection.far + (projection.far - projection.near) * 0.01;
    let size = projection.area.size();
    (
        size,
        Transform {
            translation: camera.transform_point(projection.area.center().extend(depth)),
            rotation: camera.rotation,
            scale: camera.scale * size.extend(1.0),
        },
    )
}

fn sync_battle_composite(
    layout: Res<BattleSceneLayout>,
    frame: Res<VisualBattleFrame>,
    mode: Res<BattleFlashMode>,
    mut materials: ResMut<Assets<BattleCompositeMaterial>>,
    status: Res<BattleViewStatus>,
    mut scene: ResMut<BattleScene>,
    cameras: Query<
        (
            &Camera,
            &OrthographicProjection,
            &Transform,
            Option<&RenderLayers>,
        ),
        (
            With<bevy::core_pipeline::core_2d::Camera2d>,
            Without<BattleCompositeSprite>,
            Without<BattleRowCompositeSprite>,
            Without<Parent>,
        ),
    >,
    mut sprites: Query<
        (
            &Handle<BattleCompositeMaterial>,
            &mut Transform,
            &mut Visibility,
            Option<&BattleRowCompositeSprite>,
        ),
        Or<(With<BattleCompositeSprite>, With<BattleRowCompositeSprite>)>,
    >,
) {
    let camera = cameras.iter().find(|(camera, _, _, layers)| {
        camera.is_active
            && matches!(camera.target, RenderTarget::Window(_))
            && layers.is_none_or(|layers| {
                layers.intersects(&RenderLayers::layer(BATTLE_COMPOSITE_LAYER))
            })
    });
    let mut source = battle_composite_uniform(&layout, &frame, *mode, status.output_size.as_vec2());
    register_battle_row_actors(
        &layout,
        &mut source,
        &mut scene,
        status.output_size.as_vec2(),
    );
    for (material, mut transform, mut visibility, row_overlay) in &mut sprites {
        let Some((_, projection, camera, _)) =
            camera.filter(|_| status.active && status.output_size.min_element() > 0)
        else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        let (_, mut pose) = battle_composite_pose(projection, camera);
        if row_overlay.is_some() {
            if !frame
                .source
                .as_ref()
                .is_some_and(|source| source.battler_rows.iter().any(Option::is_some))
            {
                visibility.set_if_neq(Visibility::Hidden);
                continue;
            }
            // The shell derives this from the same source priority decision
            // and slot as explicit OAM. Validation rejects different depths
            // within this shared row quad; future multi-depth rows need quads
            // of their own rather than silently taking the first row's order.
            pose.translation.z = frame
                .source
                .as_ref()
                .unwrap()
                .battler_rows
                .iter()
                .flatten()
                .next()
                .unwrap()
                .oam_depth;
        }
        if materials
            .get(material)
            .is_some_and(|material| material.source != source)
        {
            materials.get_mut(material).unwrap().source = source.clone();
        }
        transform.set_if_neq(pose);
        visibility.set_if_neq(Visibility::Visible);
    }
}

#[allow(clippy::too_many_arguments)]
fn sync_battle_scene(
    mut commands: Commands,
    frame: Res<VisualBattleFrame>,
    presentation: (
        Res<VoxelViewSettings>,
        Res<BattleSceneLayout>,
        Res<row_capture::ActorCaptures>,
        Res<capture_bridge::BattleCaptureBridge>,
        ResMut<Assets<SkinnedMeshInverseBindposes>>,
        Res<EncounterTerrain>,
    ),
    flash_mode: Res<BattleFlashMode>,
    time: Res<Time>,
    mut status: ResMut<BattleViewStatus>,
    mut scene: ResMut<BattleScene>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cameras: Query<
        (
            &mut Camera,
            &mut Transform,
            &mut FogSettings,
            &mut Projection,
        ),
        (
            With<BattleCamera>,
            Without<BattleActor>,
            Without<BattleParticle>,
            Without<BattleBall>,
        ),
    >,
    mut lights: Query<
        &mut Visibility,
        (
            With<BattleLight>,
            Without<BattleArena>,
            Without<BattleActor>,
            Without<BattleParticle>,
            Without<BattleBall>,
        ),
    >,
    mut arenas: Query<
        (&mut Visibility, &mut Transform),
        (
            With<BattleArena>,
            Without<BattleCamera>,
            Without<BattleLight>,
            Without<BattleActor>,
            Without<BattleParticle>,
            Without<BattleBall>,
        ),
    >,
    mut actors: Query<
        (&mut Transform, &mut Visibility, &mut RenderLayers),
        (
            With<BattleActor>,
            Without<BattleCamera>,
            Without<BattleLight>,
            Without<BattleArena>,
            Without<BattleParticle>,
            Without<BattleBall>,
        ),
    >,
    mut effects: Query<
        (
            &mut Transform,
            &mut Visibility,
            &mut Handle<StandardMaterial>,
        ),
        (
            With<BattleParticle>,
            Without<BattleCamera>,
            Without<BattleActor>,
            Without<BattleLight>,
            Without<BattleArena>,
            Without<BattleBall>,
        ),
    >,
    mut balls: Query<
        (&mut Transform, &mut Visibility),
        (
            With<BattleBall>,
            Without<BattleCamera>,
            Without<BattleActor>,
            Without<BattleLight>,
            Without<BattleArena>,
            Without<BattleParticle>,
        ),
    >,
    mut contact_shadows: Query<
        (&BattleContactShadow, &mut Transform, &mut Visibility),
        (
            Without<BattleCamera>,
            Without<BattleActor>,
            Without<BattleLight>,
            Without<BattleArena>,
            Without<BattleParticle>,
            Without<BattleBall>,
        ),
    >,
) {
    let (settings, layout, captures, capture_bridge, mut inverse_binds, scenery) = presentation;
    #[cfg(feature = "operation-trace")]
    let _span = bevy::log::info_span!("crystal_battle_render_sync").entered();
    let valid = frame.validate();
    let ground_clearance = scenery.permits_displacements(
        &layout,
        frame.source.as_ref().map_or([Vec2::ZERO; 2], |source| source.battler_offsets),
    );
    let active = settings.enabled
        && frame.active
        && !frame.use_source_scene
        && valid.is_ok()
        && capture_bridge.allows_scene()
        && ground_clearance;
    status.active = active;
    status.active_frames = if active {
        status.active_frames.saturating_add(1)
    } else {
        0
    };
    status.last_error = valid.err().map(str::to_owned);
    for (mut camera, mut transform, mut fog, mut projection) in &mut cameras {
        if camera.is_active != active {
            camera.is_active = active;
        }
        if active {
            transform.set_if_neq(layout.camera);
            if let Projection::Perspective(p) = &mut *projection {
                if p.far != layout.far {
                    p.far = layout.far;
                }
            }
            let (fog_start, fog_end) = scenery
                .fog_range()
                .unwrap_or((layout.distance + 10.0, layout.distance + 30.0));
            fog.falloff = FogFalloff::Linear {
                start: fog_start,
                end: fog_end,
            };
            let (dark, light) = source_environment_palette(frame.source.as_ref(), *flash_mode);
            let sky = rgb(palette(frame.environment).sky)
                .mix(&Color::BLACK, dark)
                .mix(&Color::WHITE, light);
            if !matches!(camera.clear_color, ClearColorConfig::Custom(color) if color == sky) {
                camera.clear_color = ClearColorConfig::Custom(sky);
            }
            if fog.color != sky {
                fog.color = sky;
            }
        }
    }
    for mut visibility in &mut lights {
        visibility.set_if_neq(if active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
    }
    for (mut visibility, mut transform) in &mut arenas {
        transform.scale = Vec3::new(layout.arena_scale, 1.0, layout.arena_scale);
        visibility.set_if_neq(if active && !scenery.accepted() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
    }
    if !active {
        for (_, mut visibility, _) in &mut actors {
            visibility.set_if_neq(Visibility::Hidden);
        }
        for (_, mut visibility, _) in &mut effects {
            visibility.set_if_neq(Visibility::Hidden);
        }
        for (_, mut visibility) in &mut balls {
            visibility.set_if_neq(Visibility::Hidden);
        }
        for (_, _, mut visibility) in &mut contact_shadows {
            visibility.set_if_neq(Visibility::Hidden);
        }
        scene.was_active = false;
        status.modeled_species.clear();
        status.source_art_species = if frame.active && (frame.use_source_scene || !ground_clearance) {
            frame
                .battlers
                .iter()
                .flatten()
                .map(|battler| battler.species_id.to_string())
                .collect()
        } else {
            Vec::new()
        };
        return;
    }
    if !scene.was_active {
        scene.elapsed = 0.0;
    }
    scene.was_active = true;
    scene.elapsed += time.delta_seconds().clamp(0.0, 0.1);
    let (dark, light) = source_environment_palette(frame.source.as_ref(), *flash_mode);
    let base_color = Color::srgb(1.0 - dark, 1.0 - dark, 1.0 - dark);
    let emissive = LinearRgba::new(light, light, light, 1.0);
    // Assets::get_mut emits Modified even if the value does not change. Avoid
    // re-extracting and preparing the same GPU material every display frame.
    if materials
        .get(&scene.surface_material)
        .is_some_and(|material| material.base_color != base_color || material.emissive != emissive)
    {
        let material = materials.get_mut(&scene.surface_material).unwrap();
        material.base_color = base_color;
        material.emissive = emissive;
    }
    if scene.arena_key != Some(frame.environment) {
        if let Some(entity) = scene.arena.take() {
            commands.entity(entity).despawn_recursive();
        }
        if let Some(mesh) = scene.arena_mesh.take() {
            meshes.remove(mesh.id());
        }
        let mut data = arena_mesh(frame.environment);
        if scene.vertex_lighting {
            data.colors = vertex_lit_colors(&data, Quat::IDENTITY);
            scene.arena_neutral_colors = data.colors.clone();
        }
        let mut mesh = data.into_mesh();
        if scene.vertex_lighting {
            mesh.asset_usage = bevy::render::render_asset::RenderAssetUsages::all();
        }
        let mesh = meshes.add(mesh);
        scene.arena = Some(
            commands
                .spawn((
                    PbrBundle {
                        mesh: mesh.clone(),
                        material: scene.surface_material.clone(),
                        transform: Transform::from_scale(Vec3::new(
                            layout.arena_scale,
                            1.0,
                            layout.arena_scale,
                        )),
                        visibility: if scenery.accepted() {
                            Visibility::Hidden
                        } else {
                            Visibility::Visible
                        },
                        ..default()
                    },
                    RenderLayers::layer(BATTLE_LAYER),
                    BattleArena,
                ))
                .id(),
        );
        scene.arena_mesh = Some(mesh);
        scene.arena_key = Some(frame.environment);
        scene.surface_white_strength = None;
    }
    if scene.vertex_lighting && scene.surface_white_strength != Some(light) {
        // Unlit materials deliberately skip emissive. Apply the BGP white
        // flash to these background surfaces explicitly, preserving both its
        // source cadence and reduced intensity. OBJ palettes remain separate.
        for (handle, neutral) in [
            (
                scene.arena_mesh.as_ref().unwrap(),
                &scene.arena_neutral_colors,
            ),
            (&scene.ball_mesh, &scene.ball_neutral_colors),
        ] {
            if let Some(mesh) = meshes.get_mut(handle) {
                mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, surface_white_colors(neutral, light));
            }
        }
        scene.surface_white_strength = Some(light);
    }
    status.modeled_species.clear();
    status.source_art_species.clear();
    for side in [VisualBattleSide::Player, VisualBattleSide::Enemy] {
        let index = side.index();
        let Some(battler) = frame.battlers[index].as_ref() else {
            if let Some(instance) = scene.actors[index].take() {
                instance.retire(&mut commands, &mut meshes, &mut materials);
            }
            continue;
        };
        if battler.allow_species_model && !scene.species_meshes.contains_key(&battler.species_id) {
            let mesh = crate::battle_species_models::mesh(&battler.species_id)
                .or_else(|| {
                    crate::new_bark_actors::actor_props::battle_species_mesh(&battler.species_id)
                })
                .map(Arc::new);
            if let Some(data) = &mesh {
                scene.species_bounds.insert(
                    battler.species_id.clone(),
                    BattleModelBounds::from_surface(data),
                );
            }
            scene
                .species_meshes
                .insert(battler.species_id.clone(), mesh);
        }
        let modeled_data = if battler.allow_species_model
            && battler
                .pokedex_size_m
                .is_some_and(|v| v.is_finite() && v > 0.0)
        {
            scene
                .species_meshes
                .get(&battler.species_id)
                .cloned()
                .flatten()
        } else {
            None
        };
        let modeled = modeled_data.is_some();
        let rig = modeled
            .then(|| animated_species(&battler.species_id))
            .flatten();
        let skin_rig = modeled
            .then(|| crate::species_rig::for_species(&battler.species_id))
            .flatten();
        let root_data = rig
            .map(|rig| &rig.groups[0].mesh)
            .or(modeled_data.as_deref());
        let modeled_bounds = modeled_data
            .as_ref()
            .and_then(|_| scene.species_bounds.get(&battler.species_id))
            .copied();
        let source_texture = frame
            .source
            .as_ref()
            .filter(|_| *flash_mode == BattleFlashMode::Full)
            .map(|source| &source.battler_textures[index])
            .filter(|texture| **texture != Handle::default())
            .unwrap_or(&battler.texture);
        let key = ActorKey {
            species: battler.species_id.clone(),
            party_index: battler.party_index,
            modeled,
        };
        if modeled {
            status.modeled_species.push(battler.species_id.to_string());
        } else {
            status
                .source_art_species
                .push(battler.species_id.to_string());
        }
        if scene.actors[index]
            .as_ref()
            .is_none_or(|instance| instance.key != key)
        {
            if let Some(instance) = scene.actors[index].take() {
                instance.retire(&mut commands, &mut meshes, &mut materials);
            }
            let material = materials.add(if modeled {
                StandardMaterial {
                    perceptual_roughness: 0.92,
                    reflectance: 0.15,
                    unlit: scene.vertex_lighting,
                    ..default()
                }
            } else {
                StandardMaterial {
                    base_color_texture: Some(source_texture.clone()),
                    alpha_mode: AlphaMode::Mask(0.1),
                    unlit: true,
                    cull_mode: None,
                    ..default()
                }
            });
            let base_pose = layout_actor_pose(battler, modeled_bounds, &layout);
            // Deferred entities cannot be reached by the actor query below
            // until the next update. Present this source offset on spawn too.
            let mut transform = base_pose;
            if let Some(source) = &frame.source {
                transform.translation += layout.source_displacement(source.battler_offsets[index]);
            }
            let (mesh, neutral_colors) = root_data.map_or((None, None), |data| {
                let (mesh, colors) =
                    actor_mesh(data, transform.rotation, scene.vertex_lighting, &mut meshes);
                (Some(mesh), colors)
            });
            let entity = commands
                .spawn((
                    PbrBundle {
                        mesh: mesh.clone().unwrap_or_else(|| scene.fallback_mesh.clone()),
                        material: material.clone(),
                        transform,
                        visibility: if battler.visible {
                            Visibility::Visible
                        } else {
                            Visibility::Hidden
                        },
                        ..default()
                    },
                    battle_actor_layers(
                        &frame,
                        index,
                        captures.prewarm_actor(&frame, index)
                            && modeled
                            && rig.is_none()
                            && skin_rig.is_none(),
                    ),
                    BattleActor,
                ))
                .id();
            let animated = rig.map(|rig| {
                let wings = std::array::from_fn(|wing| {
                    let (mesh, neutral_colors) = actor_mesh(
                        &rig.groups[wing + 1].mesh,
                        transform.rotation,
                        scene.vertex_lighting,
                        &mut meshes,
                    );
                    let child = commands
                        .spawn((
                            PbrBundle {
                                mesh: mesh.clone(),
                                material: material.clone(),
                                visibility: Visibility::Inherited,
                                ..default()
                            },
                            battle_actor_layers(&frame, index, false),
                            BattleActor,
                        ))
                        .id();
                    commands.entity(entity).add_child(child);
                    AnimatedPart {
                        entity: child,
                        mesh,
                        neutral_colors,
                    }
                });
                AnimatedActor {
                    rig,
                    wings,
                    elapsed: 0.0,
                }
            });
            let skinned = skin_rig.map(|rig| {
                skinning::spawn(
                    &mut commands,
                    entity,
                    meshes
                        .get_mut(mesh.as_ref().expect("modeled skin mesh"))
                        .expect("new skin mesh asset"),
                    &mut inverse_binds,
                    &mut scene.skin_inverse_binds,
                    rig,
                )
            });
            scene.actors[index] = Some(ActorInstance {
                entity,
                key,
                material,
                mesh,
                palette_key: (0xe4, *flash_mode),
                palette_colors: None,
                neutral_colors,
                base_pose,
                source_rect: battler.source_rect,
                source_opaque_rect: battler.source_opaque_rect,
                projected_registration: None,
                animated,
                skinned,
            });
        }
        let vertex_lighting = scene.vertex_lighting;
        if let Some(instance) = &mut scene.actors[index] {
            let base_pose = layout_actor_pose(battler, modeled_bounds, &layout);
            if instance.base_pose != base_pose {
                instance.base_pose = base_pose;
                instance.projected_registration = None;
            }
            if !modeled
                && materials.get(&instance.material).is_some_and(|material| {
                    material.base_color_texture.as_ref() != Some(source_texture)
                })
            {
                materials
                    .get_mut(&instance.material)
                    .unwrap()
                    .base_color_texture = Some(source_texture.clone());
            }
            let bgp = frame
                .source
                .as_ref()
                .map_or(0xe4, |source| source.battler_bgps[index]);
            // Neutral source frames preserve authored colors. Nonneutral
            // palette values can change without changing their BGP register.
            let palette_colors = frame
                .source
                .as_ref()
                .filter(|_| bgp != 0xe4)
                .map(|source| source.battler_palettes[index]);
            if instance.palette_key.0 != bgp
                || (bgp != 0xe4 && instance.palette_key.1 != *flash_mode)
                || instance.palette_colors != palette_colors
            {
                if let (Some(data), Some(handle)) = (root_data, instance.mesh.as_ref()) {
                    if let Some(mesh) = meshes.get_mut(handle) {
                        apply_actor_palette(
                            mesh,
                            data,
                            instance.neutral_colors.as_deref(),
                            frame.source.as_ref(),
                            index,
                            bgp,
                            *flash_mode,
                        );
                    }
                }
                if let Some(animated) = &instance.animated {
                    for (wing, part) in animated.wings.iter().enumerate() {
                        if let Some(mesh) = meshes.get_mut(&part.mesh) {
                            apply_actor_palette(
                                mesh,
                                &animated.rig.groups[wing + 1].mesh,
                                part.neutral_colors.as_deref(),
                                frame.source.as_ref(),
                                index,
                                bgp,
                                *flash_mode,
                            );
                        }
                    }
                }
                instance.palette_key = (bgp, *flash_mode);
                instance.palette_colors = palette_colors;
            }
            instance.palette_key = (bgp, *flash_mode);
            let unlit = vertex_lighting
                || !modeled
                || (bgp != 0xe4 && *flash_mode == BattleFlashMode::Full);
            if materials
                .get(&instance.material)
                .is_some_and(|material| material.unlit != unlit)
            {
                materials.get_mut(&instance.material).unwrap().unlit = unlit;
            }
            if instance.source_rect != battler.source_rect
                || instance.source_opaque_rect != battler.source_opaque_rect
            {
                instance.base_pose = layout_actor_pose(battler, modeled_bounds, &layout);
                instance.projected_registration = None;
                instance.source_rect = battler.source_rect;
                instance.source_opaque_rect = battler.source_opaque_rect;
            }
            if let Ok((mut transform, mut visibility, mut layers)) = actors.get_mut(instance.entity)
            {
                layers.set_if_neq(battle_actor_layers(
                    &frame,
                    index,
                    captures.prewarm_actor(&frame, index) && modeled && !instance.is_articulated(),
                ));
                let mut pose = instance.base_pose;
                if let Some(source) = &frame.source {
                    pose.translation += layout.source_displacement(source.battler_offsets[index]);
                }
                transform.set_if_neq(pose);
                visibility.set_if_neq(if battler.visible {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                });
            }
            if actors.contains(instance.entity) {
                if let Some(animated) = &mut instance.animated {
                    // Idle presentation owns its instance clock. It never
                    // reads/writes the authoritative move timeline or root.
                    if battler.visible {
                        animated.elapsed = (animated.elapsed + time.delta_seconds())
                            .rem_euclid(animated.rig.duration);
                    }
                    let poses = animated
                        .rig
                        .sample(animated.elapsed)
                        .expect("finite battle presentation clock");
                    for (wing, part) in animated.wings.iter().enumerate() {
                        if let Ok((mut transform, mut visibility, mut layers)) =
                            actors.get_mut(part.entity)
                        {
                            transform.set_if_neq(poses[wing + 1]);
                            visibility.set_if_neq(if battler.visible {
                                Visibility::Inherited
                            } else {
                                Visibility::Hidden
                            });
                            layers.set_if_neq(battle_actor_layers(&frame, index, false));
                        }
                    }
                }
            }
        }
    }
    for (shadow, mut transform, mut visibility) in &mut contact_shadows {
        if shadow.0 == VisualBattleSide::Enemy
            && capture_bridge.image_active()
            && frame
                .capture
                .as_ref()
                .is_some_and(|capture| capture_bridge::picture_tiles(capture) < 7)
        {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        }
        let Some(battler) = frame.battlers[shadow.0.index()]
            .as_ref()
            .filter(|b| b.visible)
        else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        let mut position = layout.origin(battler.side.index());
        if let Some(source) = &frame.source {
            position += layout.source_displacement(source.battler_offsets[battler.side.index()]);
        }
        let ground_y = if scenery.accepted() {
            layout.origin(battler.side.index()).y + 0.006
        } else {
            0.086
        };
        transform.translation = Vec3::new(position.x, ground_y, position.z);
        if let Some(actor) = scene.actors[shadow.0.index()].as_ref() {
            if let Some(bounds) = scene
                .species_bounds
                .get(&battler.species_id)
                .filter(|_| actor.key.modeled)
            {
                let extent = (bounds.max - bounds.min) * actor.base_pose.scale;
                let center = actor
                    .base_pose
                    .transform_point((bounds.min + bounds.max) * 0.5);
                transform.translation.x += center.x - actor.base_pose.translation.x;
                transform.translation.z += center.z - actor.base_pose.translation.z;
                transform.rotation = actor.base_pose.rotation;
                transform.scale = Vec3::new(extent.x * 0.48, 1.0, extent.z * 0.48);
            } else {
                transform.rotation = Quat::IDENTITY;
                transform.scale = Vec3::new(0.38, 1.0, 0.23) * MODEL_SCALE;
            }
        }
        visibility.set_if_neq(Visibility::Visible);
    }
    // HP loss is a HUD tween, not permission to invent impact sparks. Capture
    // and reveal sequences retain their actual source renderer until modeled.
    for (_, mut visibility, _) in &mut effects {
        visibility.set_if_neq(Visibility::Hidden);
    }
    for (_, mut visibility) in &mut balls {
        visibility.set_if_neq(Visibility::Hidden);
    }
}

/// Register source row identity to the sculpture's projected opaque footprint.
/// The source pixels label rows; they do not scale or squash authored geometry.
/// Camera projection scans immutable vertices once per actor/viewport, then caches.
fn register_battle_row_actors(
    layout: &BattleSceneLayout,
    uniform: &mut BattleCompositeUniform,
    scene: &mut BattleScene,
    viewport: Vec2,
) {
    if uniform.battler_rows.iter().all(|rows| rows.w == 0.0) && uniform.capture_enemy.z == 0.0 {
        return;
    }
    for index in 0..2 {
        let data = scene.actors[index]
            .as_ref()
            .and_then(|actor| scene.species_meshes.get(&actor.key.species))
            .cloned()
            .flatten();
        let Some(actor) = scene.actors[index].as_mut() else {
            continue;
        };
        let view_rect = if let Some((_size, _camera, bounds)) = actor
            .projected_registration
            .filter(|(size, camera, _)| *size == viewport && *camera == layout.camera)
        {
            bounds
        } else {
            let fallback = [
                [
                    -FALLBACK_CARD_HEIGHT * 0.5,
                    -FALLBACK_CARD_HEIGHT * 0.5,
                    0.0,
                ],
                [FALLBACK_CARD_HEIGHT * 0.5, -FALLBACK_CARD_HEIGHT * 0.5, 0.0],
                [-FALLBACK_CARD_HEIGHT * 0.5, FALLBACK_CARD_HEIGHT * 0.5, 0.0],
                [FALLBACK_CARD_HEIGHT * 0.5, FALLBACK_CARD_HEIGHT * 0.5, 0.0],
            ];
            // A single fixed envelope covers every wing pose. Row capture
            // remains live; only the source-to-view registration is cached.
            let animated_corners = actor.animated.as_ref().map(|animated| {
                let (min, max) = animated.rig.animated_bounds;
                std::array::from_fn::<_, 8, _>(|i| {
                    [
                        if i & 1 == 0 { min.x } else { max.x },
                        if i & 2 == 0 { min.y } else { max.y },
                        if i & 4 == 0 { min.z } else { max.z },
                    ]
                })
            });
            let points = animated_corners.as_ref().map_or_else(
                || {
                    data.as_ref()
                        .map_or(fallback.as_slice(), |data| data.positions.as_slice())
                },
                |corners| corners.as_slice(),
            );
            let bounds = projected_actor_footprint(&layout, points, actor.base_pose, viewport);
            actor.projected_registration = Some((viewport, layout.camera, bounds));
            bounds
        };
        let source_rect = if actor.key.modeled {
            actor.source_opaque_rect
        } else {
            actor.source_rect
        };
        uniform.actor_view_rects[index] = Vec4::new(
            view_rect.min.x,
            view_rect.min.y,
            view_rect.max.x,
            view_rect.max.y,
        );
        uniform.actor_source_rects[index] = Vec4::new(
            source_rect.min.x,
            source_rect.min.y,
            source_rect.max.x,
            source_rect.max.y,
        );
    }
}

fn projected_actor_footprint(
    layout: &BattleSceneLayout,
    points: &[[f32; 3]],
    pose: Transform,
    viewport: Vec2,
) -> Rect {
    let view = layout.camera.compute_matrix().inverse();
    let tangent = (BATTLE_CAMERA_FOV * 0.5).tan();
    let aspect = viewport.max(Vec2::ONE).x / viewport.max(Vec2::ONE).y;
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    for point in points {
        let point = view.transform_point3(pose.transform_point(Vec3::from_array(*point)));
        let uv = Vec2::new(
            0.5 + point.x / (-point.z * 2.0 * tangent * aspect),
            0.5 - point.y / (-point.z * 2.0 * tangent),
        );
        min = min.min(uv);
        max = max.max(uv);
    }
    Rect::from_corners(min, max)
}

fn battle_has_rows(frame: &VisualBattleFrame) -> bool {
    frame
        .source
        .as_ref()
        .is_some_and(|source| source.battler_rows.iter().any(Option::is_some))
}

fn battle_actor_layers(frame: &VisualBattleFrame, index: usize, prewarm: bool) -> RenderLayers {
    if index == 1
        && frame
            .capture
            .as_ref()
            .is_some_and(|capture| capture.presented)
    {
        // The immutable enemy image is composited once. Its original mesh
        // remains available for restoration and keeps its canonical transform.
        RenderLayers::layer(BATTLE_ROW_LAYERS[index])
    } else if battle_has_rows(frame) {
        RenderLayers::layer(BATTLE_ROW_LAYERS[index])
    } else if prewarm {
        // The arena keeps its ordinary actor during prewarm. Its private row
        // camera sees the same entity, so there is no duplicate mesh/entity.
        RenderLayers::from_layers(&[BATTLE_LAYER, BATTLE_ROW_LAYERS[index]])
    } else {
        RenderLayers::layer(BATTLE_LAYER)
    }
}

fn toggle_battle_flash_mode(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    settings: Res<VoxelViewSettings>,
    mut mode: ResMut<BattleFlashMode>,
) {
    if settings.allow_f3_toggle && keys.is_some_and(|keys| keys.just_pressed(KeyCode::F4)) {
        *mode = if *mode == BattleFlashMode::Full {
            BattleFlashMode::Reduced
        } else {
            BattleFlashMode::Full
        };
    }
}

pub(super) fn source_environment_palette(
    source: Option<&VisualBattleSourceFrame>,
    mode: BattleFlashMode,
) -> (f32, f32) {
    let Some(source) = source else {
        return (0.0, 0.0);
    };
    let strength = mode.palette_strength();
    let dark = f32::from(source.bgp & 3) / 3.0 * strength;
    let light = if source.bgp == 0 { strength } else { 0.0 };
    (dark, light)
}

/// Map authored model colors to the source palette's closest shade only while
/// its register is changed. Neutral frames retain every authored mesh color.
fn source_model_color(
    color: [f32; 4],
    bgp: u8,
    palette: &[[f32; 4]; 4],
    mode: BattleFlashMode,
) -> [f32; 4] {
    if bgp == 0xe4 {
        return color;
    }
    let shade = (0..4)
        .min_by(|a, b| {
            let distance = |index: usize| {
                (0..3)
                    .map(|channel| (palette[index][channel] - color[channel]).powi(2))
                    .sum::<f32>()
            };
            distance(*a).total_cmp(&distance(*b))
        })
        .unwrap_or(0);
    let mapped = palette[usize::from((bgp >> (shade * 2)) & 3)];
    let strength = mode.palette_strength();
    [
        color[0] + (mapped[0] - color[0]) * strength,
        color[1] + (mapped[1] - color[1]) * strength,
        color[2] + (mapped[2] - color[2]) * strength,
        color[3],
    ]
}

/// Static diffuse lighting keeps the authored volume legible without running
/// a rough-surface BRDF for every covered pixel on software renderers. The
/// source mesh and its normals remain unchanged, including during LCD waves.
fn vertex_lit_colors(data: &SurfaceMeshData, rotation: Quat) -> Vec<[f32; 4]> {
    let key = Vec3::new(6.0, 12.0, 8.0).normalize();
    let fill = Vec3::new(-5.0, 7.0, -3.0).normalize();
    data.colors
        .iter()
        .zip(&data.normals)
        .map(|(color, normal)| {
            let normal = rotation * Vec3::from_array(*normal);
            let diffuse = normal.dot(key).max(0.0) * 0.72;
            let fill = normal.dot(fill).max(0.0) * 0.16;
            [
                color[0] * (0.18 + diffuse + fill),
                color[1] * (0.20 + diffuse * 0.96 + fill),
                color[2] * (0.23 + diffuse * 0.88 + fill),
                color[3],
            ]
        })
        .collect()
}

fn source_lit_color(
    neutral: [f32; 4],
    mapped: [f32; 4],
    bgp: u8,
    mode: BattleFlashMode,
) -> [f32; 4] {
    if bgp == 0xe4 {
        return neutral;
    }
    if mode == BattleFlashMode::Full {
        return [mapped[0], mapped[1], mapped[2], neutral[3]];
    }
    let strength = mode.palette_strength();
    [
        neutral[0] + (mapped[0] - neutral[0]) * strength,
        neutral[1] + (mapped[1] - neutral[1]) * strength,
        neutral[2] + (mapped[2] - neutral[2]) * strength,
        neutral[3],
    ]
}

fn surface_white_colors(neutral: &[[f32; 4]], strength: f32) -> Vec<[f32; 4]> {
    neutral
        .iter()
        .map(|color| {
            [
                color[0] + (1.0 - color[0]) * strength,
                color[1] + (1.0 - color[1]) * strength,
                color[2] + (1.0 - color[2]) * strength,
                color[3],
            ]
        })
        .collect()
}

fn source_object_pose(layout: &BattleSceneLayout, center: Vec2, size: Vec2) -> Transform {
    layout.source_pose(center, size)
}

/// One camera-facing similarity used for original LCD scanlines and source OAM.
fn source_plane_projection(layout: &BattleSceneLayout, viewport: Vec2) -> Mat3 {
    layout.source_projection(viewport)
}

fn project_source_point(projection: Mat3, point: Vec2) -> Vec2 {
    let projected = projection * point.extend(1.0);
    projected.truncate() / projected.z
}

/// Raster coverage of the source OAM's projected corners, in viewport pixels.
/// Texture pixels must use `BattleSourceProjection`'s inverse mapping;
/// stretching a Sprite into this bounding rectangle is not a valid projection.
pub fn battle_source_overlay_rect(
    layout: &BattleSceneLayout,
    center: Vec2,
    size: Vec2,
    viewport: Vec2,
) -> Option<Rect> {
    let projection = crate::BattleSourceProjection::new(layout, viewport)?;
    let bounds = projection.coverage(Rect::from_center_size(center, size))?;
    Some(Rect::from_corners(
        bounds.min * viewport,
        bounds.max * viewport,
    ))
}

fn source_battler_displacement(offset: Vec2) -> Vec3 {
    let forward = (side_position(VisualBattleSide::Enemy)
        - side_position(VisualBattleSide::Player))
    .normalize();
    forward * offset.x * SOURCE_PIXEL_WORLD + Vec3::Y * offset.y * SOURCE_PIXEL_WORLD
}

fn sync_source_objects(
    layout: Res<BattleSceneLayout>,
    frame: Res<VisualBattleFrame>,
    status: Res<BattleViewStatus>,
    mode: Res<BattleFlashMode>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut objects: Query<(
        &BattleSourceObject,
        &mut Transform,
        &mut Visibility,
        &Handle<StandardMaterial>,
    )>,
) {
    for (slot, mut transform, mut visibility, material) in &mut objects {
        let Some(source) = frame.source.as_ref().filter(|_| status.active) else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        let Some(object) = source.objects.iter().find(|object| object.slot == slot.0) else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        *transform = source_object_pose(&layout, object.center, object.size);
        // Native source OAM is composited after this BG target. Retain its
        // cached pose/material, but never duplicate it inside the scrolled BG.
        visibility.set_if_neq(Visibility::Hidden);
        let texture = if *mode == BattleFlashMode::Reduced {
            &object.neutral_texture
        } else {
            &object.texture
        };
        let uv_transform = bevy::math::Affine2::from_scale_angle_translation(
            object.uv_rect.size(),
            0.0,
            object.uv_rect.min,
        );
        if materials.get(material).is_some_and(|material| {
            material.base_color_texture.as_ref() != Some(texture)
                || material.uv_transform != uv_transform
        }) {
            let material = materials.get_mut(material).unwrap();
            material.base_color_texture = Some(texture.clone());
            material.uv_transform = uv_transform;
        }
    }
}

/// All added volumes are keyed by the actual presented move ID and existing
/// live source objects. No dialogue parsing, predicted target state or timer.
fn modeled_source_effect_pose(
    layout: &BattleSceneLayout,
    frame: &VisualBattleFrame,
    slot: usize,
) -> Option<(usize, Transform, [f32; 4])> {
    let source = frame.source.as_ref()?;
    let cue = frame
        .cues
        .iter()
        .find(|cue| cue.kind == VisualBattleCueKind::Move)?;
    let direction = (layout.hit_anchors[cue.side.opposite().index()]
        - layout.hit_anchors[cue.side.index()])
    .normalize();
    let ring_rotation = Quat::from_rotation_arc(Vec3::Y, direction);
    let object = source.objects.iter().find(|object| object.slot == slot)?;
    let center = source_object_pose(&layout, object.center, object.size).translation;
    match (cue.move_id.as_ref(), object.object_id.as_ref()) {
        ("PSYCHIC_M", "BATTLE_ANIM_OBJ_WAVE") => {
            let radius = (object.size.max_element() * 0.025).max(0.20);
            Some((
                0,
                Transform::from_translation(center)
                    .with_rotation(ring_rotation)
                    .with_scale(Vec3::splat(radius)),
                [0.73, 0.31, 1.0, 0.82],
            ))
        }
        ("HYPER_BEAM", "BATTLE_ANIM_OBJ_BEAM") => {
            let length =
                object.size.x * layout.hit_anchors[0].distance(layout.hit_anchors[1]) / 84.0;
            let radius = (object.size.y * 0.013).clamp(0.08, 0.24);
            Some((
                1,
                Transform::from_translation(center)
                    .with_rotation(ring_rotation)
                    .with_scale(Vec3::new(radius, length, radius)),
                [1.0, 0.73, 0.12, 0.72],
            ))
        }
        ("HYPER_BEAM", "BATTLE_ANIM_OBJ_BEAM_TIP") => Some((
            2,
            Transform::from_translation(center)
                .with_scale(Vec3::splat(object.size.max_element() * 0.020)),
            [1.0, 0.92, 0.46, 0.78],
        )),
        _ => None,
    }
}

fn sync_modeled_source_effects(
    mut effects: Query<&mut Visibility, With<BattleModeledSourceEffect>>,
) {
    // Exact source OAM supplies the silhouette, palette, position and timing.
    // Companion Psychic/Hyper Beam volumes remain disabled until their source
    // projected shapes and colors are verified; they must not add decoration.
    for mut visibility in &mut effects {
        visibility.set_if_neq(Visibility::Hidden);
    }
}

fn source_wave_ring_mesh() -> SurfaceMeshData {
    let mut mesh = SurfaceMeshData::default();
    let point = |i: usize, j: usize| {
        let angle = i as f32 / 40.0 * TAU;
        let tube = j as f32 / 6.0 * TAU;
        Vec3::new(
            angle.cos() * (1.0 + 0.055 * tube.cos()),
            tube.sin() * 0.055,
            angle.sin() * (1.0 + 0.055 * tube.cos()),
        )
    };
    for i in 0..40 {
        for j in 0..6 {
            let [a, b, c, d] = [
                point(i, j),
                point(i + 1, j),
                point(i + 1, j + 1),
                point(i, j + 1),
            ];
            triangle(&mut mesh, a, b, c, [1.0; 4]);
            triangle(&mut mesh, a, c, d, [1.0; 4]);
        }
    }
    mesh
}

fn actor_pose(
    battler: &VisualBattleBattler,
    _cues: &[VisualBattleCue],
    _elapsed: f32,
    model: Option<BattleModelBounds>,
    source: Option<&VisualBattleSourceFrame>,
) -> Transform {
    let origin = side_position(battler.side);
    let mut transform = Transform::from_translation(origin);
    if let Some((bounds, scale)) = model.and_then(|bounds| {
        crate::battle_layout::model_scale(
            &battler.species_id,
            battler.pokedex_size_m,
            bounds.min,
            bounds.max,
        )
        .map(|scale| (bounds, scale))
    }) {
        // Preserve authored proportions and the original open three-quarter
        // staging. Source sprite footprints do not squash species geometry.
        let direction = (side_position(battler.side.opposite()) - origin).normalize();
        let camera_direction = (camera_pose().translation - origin) * Vec3::new(1.0, 0.0, 1.0);
        let facing = direction
            .lerp(camera_direction.normalize(), 0.40)
            .normalize();
        transform.rotation = Quat::from_rotation_y(facing.x.atan2(facing.z));
        transform.scale = Vec3::splat(scale);
        transform.translation.y -= bounds.min.y * scale;
    } else {
        transform.rotation = camera_pose().rotation;
        // Lift along the card's actual up vector, so the camera-facing bottom
        // stays on the platform even with a pitched camera.
        transform.translation += transform.rotation * Vec3::Y * (FALLBACK_CARD_HEIGHT * 0.5);
        transform.scale = Vec3::new(battler.texture_size.x / battler.texture_size.y, 1.0, 1.0);
    }
    if let Some(source) = source {
        transform.translation +=
            source_battler_displacement(source.battler_offsets[battler.side.index()]);
    }
    // Move/HP/faint/switch cues never invent lunges, rolls, recoil or shrink.
    // Source-only clipping and reveal sequences keep their native renderer.
    transform
}
fn element_index(element: &str) -> usize {
    match element {
        "FIRE" => 1,
        "WATER" => 2,
        "GRASS" | "BUG" => 3,
        "ELECTRIC" => 4,
        "ICE" => 5,
        "PSYCHIC" | "GHOST" | "DRAGON" => 6,
        "POISON" => 7,
        _ => 0,
    }
}
fn element_color(element: &str) -> [f32; 4] {
    match element_index(element) {
        1 => [1.0, 0.32, 0.055, 1.0],
        2 => [0.18, 0.67, 1.0, 1.0],
        3 => [0.47, 0.92, 0.22, 1.0],
        4 => [1.0, 0.88, 0.12, 1.0],
        5 => [0.59, 0.96, 1.0, 1.0],
        6 => [0.95, 0.43, 1.0, 1.0],
        7 => [0.62, 0.24, 0.8, 1.0],
        _ => [1.0, 0.85, 0.59, 1.0],
    }
}
fn particle_pose(cues: &[VisualBattleCue], i: usize, elapsed: f32) -> Option<(Vec3, f32, usize)> {
    let cue = cues
        .iter()
        .find(|cue| cue.kind == VisualBattleCueKind::Impact)
        .or_else(|| {
            cues.iter().find(|cue| {
                matches!(
                    cue.kind,
                    VisualBattleCueKind::Move | VisualBattleCueKind::SendOut
                )
            })
        })?;
    let angle = i as f32 * 2.399_963;
    let origin = side_position(cue.side) + Vec3::Y;
    match cue.kind {
        VisualBattleCueKind::Impact => {
            let phase = (elapsed * 3.0 + i as f32 / PARTICLES as f32).fract();
            let vector = Vec3::new(angle.cos(), 0.4 + (i % 4) as f32 * 0.25, angle.sin());
            Some((origin + vector * phase * 0.9, 0.065 * (1.0 - phase), 0))
        }
        VisualBattleCueKind::SendOut => {
            if cue.progress > 0.65 {
                return None;
            }
            let radius = cue.progress * 2.4;
            Some((
                origin
                    + Vec3::new(
                        angle.cos() * radius,
                        angle.sin() * radius,
                        (angle * 0.7).sin() * radius,
                    ),
                0.045,
                4,
            ))
        }
        VisualBattleCueKind::Move => {
            if cue.progress < 0.12 || cue.progress > 0.88 {
                return None;
            }
            let phase = ((cue.progress - 0.12) / 0.76 - i as f32 * 0.018).clamp(0.0, 1.0);
            if phase == 0.0 {
                return None;
            }
            if cue.damaging && element_index(&cue.element) != 0 {
                let target = side_position(cue.side.opposite()) + Vec3::Y;
                let drift = Vec3::new(angle.cos(), angle.sin(), (angle * 0.8).sin()) * 0.13;
                Some((
                    origin.lerp(target, phase) + drift,
                    0.07 + (i % 3) as f32 * 0.023,
                    element_index(&cue.element),
                ))
            } else if !cue.damaging {
                let a = angle + elapsed * 2.3;
                Some((
                    origin + Vec3::new(a.cos() * 0.55, phase * 1.4 - 0.5, a.sin() * 0.55),
                    0.055,
                    element_index(&cue.element),
                ))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn arena_mesh(environment: VisualBattleEnvironment) -> SurfaceMeshData {
    let p = palette(environment);
    let mut mesh = SurfaceMeshData::default();
    append_cylinder(
        &mut mesh,
        Vec3::new(0.0, -0.18, 0.0),
        24.0,
        0.16,
        96,
        p.ground,
    );
    append_cylinder(&mut mesh, Vec3::new(0.0, -0.02, 0.0), 5.4, 0.10, 64, p.edge);
    append_cylinder(
        &mut mesh,
        Vec3::new(0.0, 0.035, 0.0),
        5.2,
        0.035,
        64,
        p.stage,
    );
    for side in [VisualBattleSide::Player, VisualBattleSide::Enemy] {
        let center = side_position(side) - Vec3::Y * 0.036;
        append_ring(&mut mesh, center, 1.48, 1.53, p.accent);
        append_ring(&mut mesh, center, 1.56, 1.59, p.edge);
    }
    let interior = environment == VisualBattleEnvironment::Interior;
    for i in 0..28 {
        let angle = i as f32 / 28.0 * TAU;
        let radius = 9.5 + (i % 3) as f32 * 1.9;
        let center = Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius);
        if matches!(
            environment,
            VisualBattleEnvironment::Meadow | VisualBattleEnvironment::Forest
        ) {
            // Preserve the foreground sightline. Distant tree silhouettes give
            // the arena geographic depth without obscuring either battler.
            if center.z < 2.0 || center.x.abs() > 11.0 {
                let tree =
                    crate::new_bark_models::model(crate::new_bark_models::ModelKind::TreeLod);
                let scale = (3.5 + (i % 4) as f32 * 0.38) / (tree.max[1] - tree.min[1]);
                append_transformed(
                    &mut mesh,
                    &tree.surface_mesh(),
                    center,
                    Vec3::splat(scale),
                    Quat::from_rotation_y(angle),
                );
            }
            for j in 0..3 {
                let c = center + Vec3::new(j as f32 * 0.27, 0.0, 0.3);
                append_cone(
                    &mut mesh,
                    c,
                    0.12,
                    0.43 + (i % 3) as f32 * 0.08,
                    4,
                    p.ground,
                );
            }
        } else if interior {
            if i % 2 == 0 {
                append_cylinder(&mut mesh, center, 0.40, 3.5, 8, p.edge);
                append_cylinder(&mut mesh, center + Vec3::Y * 3.5, 0.58, 0.20, 8, p.accent);
            }
        } else if matches!(
            environment,
            VisualBattleEnvironment::Cave | VisualBattleEnvironment::Ice
        ) {
            if center.z < 4.0 || center.x.abs() > 8.0 {
                append_cone(
                    &mut mesh,
                    center,
                    1.1 + (i % 4) as f32 * 0.16,
                    2.0 + (i % 5) as f32 * 0.65,
                    5,
                    if i % 3 == 0 { p.edge } else { p.ground },
                );
                if environment == VisualBattleEnvironment::Ice {
                    append_cone(
                        &mut mesh,
                        center + Vec3::new(0.6, 0.0, 0.2),
                        0.25,
                        2.2,
                        5,
                        p.accent,
                    );
                }
            }
        } else if i % 2 == 0 {
            append_ring(&mut mesh, center + Vec3::Y * 0.01, 1.1, 1.12, p.accent);
        }
    }
    if interior {
        for i in -9..10 {
            for j in -9..10 {
                let center = Vec3::new(i as f32 * 1.4, 0.001, j as f32 * 1.4);
                if center.length() > 6.0 {
                    append_cylinder(
                        &mut mesh,
                        center,
                        0.52,
                        0.014,
                        4,
                        if (i + j) % 2 == 0 { p.stage } else { p.ground },
                    );
                }
            }
        }
    }
    mesh
}
fn append_transformed(
    out: &mut SurfaceMeshData,
    source: &SurfaceMeshData,
    translation: Vec3,
    scale: Vec3,
    rotation: Quat,
) {
    let base = out.positions.len() as u32;
    out.positions.extend(
        source
            .positions
            .iter()
            .map(|p| (translation + rotation * (Vec3::from_array(*p) * scale)).to_array()),
    );
    out.normals.extend(source.normals.iter().map(|n| {
        (rotation * (Vec3::from_array(*n) / scale))
            .normalize()
            .to_array()
    }));
    out.uvs.extend_from_slice(&source.uvs);
    out.colors.extend_from_slice(&source.colors);
    out.indices
        .extend(source.indices.iter().map(|index| base + index));
}
fn triangle(mesh: &mut SurfaceMeshData, a: Vec3, b: Vec3, c: Vec3, color: [f32; 4]) {
    let normal = (b - a).cross(c - a).normalize_or_zero();
    if normal == Vec3::ZERO {
        return;
    }
    let base = mesh.positions.len() as u32;
    for p in [a, b, c] {
        mesh.positions.push(p.to_array());
        mesh.normals.push(normal.to_array());
        mesh.uvs.push([0.0, 0.0]);
        mesh.colors.push(color);
    }
    mesh.indices.extend([base, base + 1, base + 2]);
}
fn append_cylinder(
    mesh: &mut SurfaceMeshData,
    center: Vec3,
    radius: f32,
    height: f32,
    segments: usize,
    color: [f32; 4],
) {
    for i in 0..segments {
        let a = i as f32 / segments as f32 * TAU;
        let b = (i + 1) as f32 / segments as f32 * TAU;
        let p = center + Vec3::new(a.cos() * radius, 0.0, a.sin() * radius);
        let q = center + Vec3::new(b.cos() * radius, 0.0, b.sin() * radius);
        let top = Vec3::Y * height;
        triangle(mesh, center + top, q + top, p + top, color);
        let side = [color[0] * 0.82, color[1] * 0.82, color[2] * 0.82, color[3]];
        triangle(mesh, p, q + top, q, side);
        triangle(mesh, p, p + top, q + top, side);
    }
}
fn append_cone(
    mesh: &mut SurfaceMeshData,
    center: Vec3,
    radius: f32,
    height: f32,
    segments: usize,
    color: [f32; 4],
) {
    for i in 0..segments {
        let a = i as f32 / segments as f32 * TAU;
        let b = (i + 1) as f32 / segments as f32 * TAU;
        triangle(
            mesh,
            center + Vec3::new(a.cos() * radius, 0.0, a.sin() * radius),
            center + Vec3::Y * height,
            center + Vec3::new(b.cos() * radius, 0.0, b.sin() * radius),
            color,
        );
    }
}
fn append_ring(mesh: &mut SurfaceMeshData, center: Vec3, inner: f32, outer: f32, color: [f32; 4]) {
    for i in 0..64 {
        let a = i as f32 / 64.0 * TAU;
        let b = (i + 1) as f32 / 64.0 * TAU;
        let p = Vec3::new(a.cos(), 0.0, a.sin());
        let q = Vec3::new(b.cos(), 0.0, b.sin());
        triangle(
            mesh,
            center + p * inner,
            center + q * outer,
            center + p * outer,
            color,
        );
        triangle(
            mesh,
            center + p * inner,
            center + q * inner,
            center + q * outer,
            color,
        );
    }
}
/// One cheap translucent fan with a soft radial falloff, shared by both sides.
/// Grounding follows only presented actor visibility/pose, including fainting.
fn contact_shadow_mesh() -> SurfaceMeshData {
    let mut mesh = SurfaceMeshData::default();
    for i in 0..48 {
        let a = i as f32 / 48.0 * TAU;
        let b = (i + 1) as f32 / 48.0 * TAU;
        let base = mesh.positions.len() as u32;
        for (point, alpha) in [
            (Vec3::ZERO, 0.40),
            (Vec3::new(b.cos(), 0.0, b.sin()), 0.0),
            (Vec3::new(a.cos(), 0.0, a.sin()), 0.0),
        ] {
            mesh.positions.push(point.to_array());
            mesh.normals.push(Vec3::Y.to_array());
            mesh.uvs.push([0.0, 0.0]);
            mesh.colors.push([0.015, 0.022, 0.012, alpha]);
        }
        mesh.indices.extend([base, base + 1, base + 2]);
    }
    mesh
}

fn capture_ball_mesh() -> SurfaceMeshData {
    let mut mesh = SurfaceMeshData::default();
    for row in 0..12 {
        for col in 0..24 {
            let vertex = |r: usize, c: usize| {
                let lat = r as f32 / 12.0 * PI;
                let lon = c as f32 / 24.0 * TAU;
                Vec3::new(lat.sin() * lon.cos(), lat.cos(), lat.sin() * lon.sin()) * 0.19
            };
            let color = if row < 5 {
                [0.82, 0.08, 0.12, 1.0]
            } else if row < 7 {
                [0.10, 0.12, 0.15, 1.0]
            } else {
                [0.94, 0.94, 0.88, 1.0]
            };
            let a = vertex(row, col);
            let b = vertex(row + 1, col);
            let c = vertex(row, col + 1);
            let d = vertex(row + 1, col + 1);
            if row > 0 {
                triangle(&mut mesh, a, c, b, color);
            }
            if row < 11 {
                triangle(&mut mesh, c, d, b, color);
            }
        }
    }
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pidgeotto_app(hz: u32) -> App {
        let mut app = headless_battle_app();
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / f64::from(hz)),
        ));
        let mut frame = app.world_mut().resource_mut::<VisualBattleFrame>();
        for battler in frame.battlers.iter_mut().flatten() {
            battler.species_id = Arc::from("PIDGEOTTO");
            battler.pokedex_size_m = Some(1.0922);
        }
        frame.source = Some(source_test_frame(0xe4));
        app
    }

    fn actor_parts(app: &App, side: usize) -> [(Entity, Handle<Mesh>); 3] {
        let actor = app.world().resource::<BattleScene>().actors[side]
            .as_ref()
            .unwrap();
        let wings = &actor.animated.as_ref().unwrap().wings;
        [
            (actor.entity, actor.mesh.clone().unwrap()),
            (wings[0].entity, wings[0].mesh.clone()),
            (wings[1].entity, wings[1].mesh.clone()),
        ]
    }

    #[test]
    fn pidgeotto_battle_clips_use_elapsed_time_without_changing_source_or_meshes() {
        let mut final_poses = Vec::new();
        for hz in [60, 30, 9] {
            let mut app = pidgeotto_app(hz);
            app.update();
            let source = app.world().resource::<VisualBattleFrame>().clone();
            let layout = app.world().resource::<BattleSceneLayout>().clone();
            let parts = actor_parts(&app, 0);
            let root = *app.world().get::<Transform>(parts[0].0).unwrap();
            let counts = (
                app.world().entities().len(),
                app.world().resource::<Assets<Mesh>>().len(),
            );
            let positions = parts.each_ref().map(|(_, handle)| {
                app.world()
                    .resource::<Assets<Mesh>>()
                    .get(handle)
                    .unwrap()
                    .attribute(Mesh::ATTRIBUTE_POSITION)
                    .unwrap()
                    .get_bytes()
                    .to_vec()
            });
            for _ in 0..hz {
                app.update();
            }
            assert_eq!(*app.world().resource::<VisualBattleFrame>(), source);
            assert_eq!(*app.world().resource::<BattleSceneLayout>(), layout);
            assert_eq!(*app.world().get::<Transform>(parts[0].0).unwrap(), root);
            assert_eq!(actor_parts(&app, 0), parts);
            assert_eq!(
                (
                    app.world().entities().len(),
                    app.world().resource::<Assets<Mesh>>().len()
                ),
                counts
            );
            for (i, (_, handle)) in parts.iter().enumerate() {
                assert_eq!(
                    app.world()
                        .resource::<Assets<Mesh>>()
                        .get(handle)
                        .unwrap()
                        .attribute(Mesh::ATTRIBUTE_POSITION)
                        .unwrap()
                        .get_bytes(),
                    positions[i]
                );
            }
            let pose = *app.world().get::<Transform>(parts[1].0).unwrap();
            assert_ne!(pose.rotation, Quat::IDENTITY);
            final_poses.push(pose);
        }
        for pose in final_poses.iter().skip(1) {
            assert!(pose.rotation.abs_diff_eq(final_poses[0].rotation, 0.00001));
            assert!(
                pose.translation
                    .abs_diff_eq(final_poses[0].translation, 0.00001)
            );
        }
    }

    #[test]
    fn pidgeotto_switch_and_visibility_keep_independent_clocks_and_retire_all_parts() {
        let mut app = pidgeotto_app(30);
        app.update();
        for _ in 0..7 {
            app.update();
        }
        let old = actor_parts(&app, 0);
        let other = actor_parts(&app, 1);
        let old_material = app.world().resource::<BattleScene>().actors[0]
            .as_ref()
            .unwrap()
            .material
            .clone();
        let other_time = app.world().resource::<BattleScene>().actors[1]
            .as_ref()
            .unwrap()
            .animated
            .as_ref()
            .unwrap()
            .elapsed;
        // A different individual of the same species starts a fresh clip.
        app.world_mut().resource_mut::<VisualBattleFrame>().battlers[0]
            .as_mut()
            .unwrap()
            .party_index = Some(3);
        app.update();
        for (entity, handle) in old {
            assert!(app.world().get_entity(entity).is_none());
            assert!(!app.world().resource::<Assets<Mesh>>().contains(handle.id()));
        }
        assert!(
            !app.world()
                .resource::<Assets<StandardMaterial>>()
                .contains(old_material.id())
        );
        assert_eq!(actor_parts(&app, 1), other);
        let scene = app.world().resource::<BattleScene>();
        assert_eq!(
            scene.actors[0]
                .as_ref()
                .unwrap()
                .animated
                .as_ref()
                .unwrap()
                .elapsed,
            0.0
        );
        assert!(
            scene.actors[1]
                .as_ref()
                .unwrap()
                .animated
                .as_ref()
                .unwrap()
                .elapsed
                > other_time
        );
        let parts = actor_parts(&app, 0);
        app.world_mut().resource_mut::<VoxelViewSettings>().enabled = false;
        app.update();
        for (entity, _) in &parts {
            assert_eq!(
                *app.world().get::<Visibility>(*entity).unwrap(),
                Visibility::Hidden
            );
        }
        app.world_mut().resource_mut::<VoxelViewSettings>().enabled = true;
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(parts[0].0).unwrap(),
            Visibility::Visible
        );
        for (entity, _) in &parts[1..] {
            assert_eq!(
                *app.world().get::<Visibility>(*entity).unwrap(),
                Visibility::Inherited
            );
        }
        app.world_mut().resource_mut::<VisualBattleFrame>().battlers[0] = None;
        app.update();
        for (entity, handle) in parts {
            assert!(app.world().get_entity(entity).is_none());
            assert!(!app.world().resource::<Assets<Mesh>>().contains(handle.id()));
        }
    }

    #[test]
    fn pidgeotto_source_palette_covers_every_part_without_touching_opponent() {
        let mut app = pidgeotto_app(30);
        app.update();
        let all = [actor_parts(&app, 0), actor_parts(&app, 1)];
        let colors = |app: &App, side: usize| {
            all[side].each_ref().map(|(_, handle)| {
                app.world()
                    .resource::<Assets<Mesh>>()
                    .get(handle)
                    .unwrap()
                    .attribute(Mesh::ATTRIBUTE_COLOR)
                    .unwrap()
                    .get_bytes()
                    .to_vec()
            })
        };
        let neutral = [colors(&app, 0), colors(&app, 1)];
        app.world_mut()
            .resource_mut::<VisualBattleFrame>()
            .source
            .as_mut()
            .unwrap()
            .battler_bgps[0] = 0x1b;
        let source = app.world().resource::<VisualBattleFrame>().clone();
        app.update();
        let full = colors(&app, 0);
        for i in 0..3 {
            assert_ne!(full[i], neutral[0][i]);
        }
        assert_eq!(colors(&app, 1), neutral[1]);
        app.world_mut().insert_resource(BattleFlashMode::Reduced);
        app.update();
        for i in 0..3 {
            assert_ne!(colors(&app, 0)[i], full[i]);
        }
        assert_eq!(*app.world().resource::<VisualBattleFrame>(), source);
        app.world_mut()
            .resource_mut::<VisualBattleFrame>()
            .source
            .as_mut()
            .unwrap()
            .battler_bgps[0] = 0xe4;
        app.update();
        assert_eq!(colors(&app, 0), neutral[0]);
    }

    #[test]
    fn pidgeotto_animated_envelope_preserves_neutral_size_ground_and_hit_anchor() {
        let rig = crate::pidgeotto_rig::rig();
        let bounds = BattleModelBounds::from_surface(&rig.neutral);
        let scale =
            crate::battle_layout::model_scale("PIDGEOTTO", Some(1.0922), bounds.min, bounds.max)
                .unwrap();
        let neutral = BattleBody::modeled("PIDGEOTTO", bounds.min, bounds.max, scale);
        let mut animated = neutral;
        animated.visual_bounds = Some(rig.animated_bounds);
        for viewport in [Vec2::new(1180.0, 812.0), Vec2::new(600.0, 1000.0)] {
            let before = BattleSceneLayout::for_bodies([Some(neutral), None], viewport);
            let after = BattleSceneLayout::for_bodies([Some(animated), None], viewport);
            let pose = after.body_poses[0].unwrap();
            assert_eq!(pose.scale, before.body_poses[0].unwrap().scale);
            assert_eq!(
                pose.translation.y,
                before.body_poses[0].unwrap().translation.y
            );
            assert_eq!(after.hit_anchors[0].y, before.hit_anchors[0].y);
            let corners = animated
                .corners(Transform::IDENTITY)
                .map(|point| point.to_array());
            let registration = projected_actor_footprint(&after, &corners, pose, viewport);
            for frame in 0..65 {
                let local = rig.sample(frame as f32 * rig.duration / 64.0).unwrap();
                for (group, local) in rig.groups.iter().zip(local) {
                    for vertex in &group.mesh.positions {
                        let point = local.transform_point(Vec3::from_array(*vertex)).to_array();
                        let projected = projected_actor_footprint(&after, &[point], pose, viewport);
                        assert!(
                            registration
                                .min
                                .cmple(projected.min + Vec2::splat(0.00001))
                                .all()
                        );
                        assert!(
                            registration
                                .max
                                .cmpge(projected.max - Vec2::splat(0.00001))
                                .all()
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn battle_profile_defaults_only_reduce_known_software_renderers() {
        assert_eq!(
            resolve_battle_profile(false, BattleRenderQuality::Automatic, None),
            (BattleRenderQuality::Native, false)
        );
        assert_eq!(
            resolve_battle_profile(true, BattleRenderQuality::Automatic, None),
            (BattleRenderQuality::Balanced, true)
        );
        assert_eq!(
            resolve_battle_profile(
                false,
                BattleRenderQuality::from_name("unknown"),
                Some("unknown")
            ),
            (BattleRenderQuality::Native, false)
        );
        assert_eq!(
            resolve_battle_profile(
                true,
                BattleRenderQuality::from_name("unknown"),
                Some("unknown")
            ),
            (BattleRenderQuality::Balanced, true)
        );
    }

    #[test]
    fn battle_profile_explicit_quality_and_lighting_override_independently() {
        for software in [false, true] {
            assert_eq!(
                resolve_battle_profile(software, BattleRenderQuality::Native, Some("pbr")),
                (BattleRenderQuality::Native, false)
            );
            assert_eq!(
                resolve_battle_profile(software, BattleRenderQuality::Balanced, Some("vertex")),
                (BattleRenderQuality::Balanced, true)
            );
            assert_eq!(
                resolve_battle_profile(software, BattleRenderQuality::Performance, None),
                (BattleRenderQuality::Performance, software)
            );
        }
        assert_eq!(
            resolve_battle_profile(false, BattleRenderQuality::Automatic, Some("vertex")),
            (BattleRenderQuality::Native, true)
        );
        assert_eq!(
            resolve_battle_profile(true, BattleRenderQuality::Automatic, Some("pbr")),
            (BattleRenderQuality::Balanced, false)
        );
    }
    #[test]
    fn scaled_battle_target_is_bounded_and_preserves_aspect() {
        assert_eq!(
            BattleRenderQuality::Balanced.target_size(UVec2::new(1180, 812)),
            UVec2::new(885, 609)
        );
        for output in [
            UVec2::ZERO,
            UVec2::ONE,
            UVec2::new(6016, 3384),
            UVec2::new(800, 8000),
        ] {
            let size = BattleRenderQuality::Balanced.target_size(output);
            assert!(size.min_element() > 0 && size.max_element() <= 4096);
            if output.min_element() >= 800 {
                let expected = output.x as f32 / output.y as f32;
                assert!((size.x as f32 / size.y as f32 - expected).abs() < 0.002);
            }
        }
    }

    #[test]
    fn battle_target_resize_and_f3_keep_native_hud_and_source_state() {
        use bevy::render::camera::{CameraProjection, ScalingMode};
        let mut app = headless_battle_app();
        app.world_mut().resource_mut::<BattleScene>().quality = BattleRenderQuality::Balanced;
        app.world_mut()
            .resource_mut::<VoxelViewSettings>()
            .allow_f3_toggle = true;
        app.init_resource::<ButtonInput<KeyCode>>()
            .add_systems(Update, crate::toggle_voxel_view);
        let window = app
            .world_mut()
            .spawn((
                Window {
                    resolution: bevy::window::WindowResolution::new(1180.0, 812.0),
                    ..default()
                },
                bevy::window::PrimaryWindow,
            ))
            .id();
        // Match the production root HUD camera, including its specialized
        // 2D depth range, layer, minimum LCD extent and compositor order.
        let mut hud_camera = Camera2dBundle::default();
        hud_camera.camera.order = 1;
        hud_camera.camera.clear_color = ClearColorConfig::None;
        hud_camera.projection.scaling_mode = ScalingMode::AutoMin {
            min_width: 640.0,
            min_height: 576.0,
        };
        hud_camera.projection.update(1180.0, 812.0);
        let camera_id = app
            .world_mut()
            .spawn((hud_camera, RenderLayers::layer(0)))
            .id();
        let hud_transform = Transform::from_xyz(310.0, 210.0, 3.2);
        let hud = app
            .world_mut()
            .spawn((
                SpriteBundle {
                    transform: hud_transform,
                    ..default()
                },
                RenderLayers::layer(0),
            ))
            .id();
        app.world_mut().resource_mut::<VisualBattleFrame>().source = Some(source_test_frame(0));
        let frame = app.world().resource::<VisualBattleFrame>().clone();
        app.update();
        let target = app
            .world()
            .resource::<BattleScene>()
            .render_target
            .clone()
            .unwrap();
        let world = app.world_mut();
        assert_eq!(
            world
                .query_filtered::<&Camera, With<bevy::core_pipeline::core_2d::Camera2d>>()
                .iter(world)
                .count(),
            1,
            "the native HUD camera must own the only full-window 2D pass"
        );
        let image_count = app.world().resource::<Assets<Image>>().len();
        assert_eq!(
            app.world().resource::<BattleViewStatus>().render_size,
            UVec2::new(885, 609)
        );
        for (width, height) in [(1600, 900), (812, 1180), (1180, 812)] {
            app.world_mut()
                .get_mut::<Window>(window)
                .unwrap()
                .resolution
                .set(width as f32, height as f32);
            app.world_mut()
                .get_mut::<OrthographicProjection>(camera_id)
                .unwrap()
                .update(width as f32, height as f32);
            app.world_mut()
                .get_mut::<Transform>(camera_id)
                .unwrap()
                .translation = Vec3::new(4.0, -2.0, 0.0);
            app.update();
            assert_eq!(
                app.world().resource::<BattleScene>().render_target.as_ref(),
                Some(&target)
            );
            assert_eq!(app.world().resource::<Assets<Image>>().len(), image_count);
            let status = app.world().resource::<BattleViewStatus>();
            assert_eq!(status.output_size, UVec2::new(width, height));
            assert_eq!(
                status.render_size,
                BattleRenderQuality::Balanced.target_size(status.output_size)
            );
            let image = app
                .world()
                .resource::<Assets<Image>>()
                .get(&target)
                .unwrap();
            assert_eq!(
                UVec2::new(
                    image.texture_descriptor.size.width,
                    image.texture_descriptor.size.height
                ),
                status.render_size
            );
            assert_eq!(app.world().get::<Transform>(hud), Some(&hud_transform));
            assert_eq!(*app.world().resource::<VisualBattleFrame>(), frame);
            let projection = app
                .world()
                .get::<OrthographicProjection>(camera_id)
                .unwrap()
                .clone();
            let camera_transform = *app.world().get::<Transform>(camera_id).unwrap();
            let world = app.world_mut();
            let (composite, visibility) = world
                .query_filtered::<(&Transform, &Visibility), With<BattleCompositeSprite>>()
                .single(world);
            assert_eq!(*visibility, Visibility::Visible);
            assert!(composite.translation.z < hud_transform.translation.z);
            let clip_from_sprite = projection.get_clip_from_view()
                * camera_transform.compute_matrix().inverse()
                * composite.compute_matrix();
            for (x, y) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                let half = Vec2::splat(0.5);
                let clip = clip_from_sprite * Vec4::new(x * half.x, y * half.y, 0.0, 1.0);
                let ndc = clip.truncate() / clip.w;
                assert!(
                    (ndc.x - x).abs() < 0.0001 && (ndc.y - y).abs() < 0.0001,
                    "resize and source screen movement must preserve the full-window arena"
                );
                assert!(
                    ndc.z > 0.0 && ndc.z < 1.0,
                    "composite must remain inside the real camera depth range"
                );
            }
        }
        let composite_layers = RenderLayers::layer(BATTLE_COMPOSITE_LAYER);
        assert!(composite_layers.intersects(&RenderLayers::layer(0)));
        for layer in [
            BATTLE_LAYER,
            crate::HIDDEN_CLASSIC_WORLD_RENDER_LAYER,
            crate::VOXEL_RENDER_LAYER,
        ] {
            assert!(!composite_layers.intersects(&RenderLayers::layer(layer)));
        }
        for expected in [false, true] {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::F3);
            app.update();
            assert_eq!(app.world().resource::<BattleViewStatus>().active, expected);
            let world = app.world_mut();
            assert!(
                world
                    .query_filtered::<&Camera, With<BattleCamera>>()
                    .iter(world)
                    .all(|camera| camera.is_active == expected)
            );
            assert!(world.get::<Camera>(camera_id).unwrap().is_active);
            assert!(
                world
                    .query_filtered::<&Visibility, With<BattleCompositeSprite>>()
                    .iter(world)
                    .all(|visibility| (*visibility == Visibility::Visible) == expected)
            );
            let mut keys = world.resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::F3);
            keys.clear();
        }
        app.world_mut().resource_mut::<VisualBattleFrame>().active = false;
        app.update();
        let world = app.world_mut();
        assert!(
            world
                .query_filtered::<&Camera, With<BattleCamera>>()
                .iter(world)
                .all(|camera| !camera.is_active)
        );
        assert_eq!(world.get::<Transform>(hud), Some(&hud_transform));
    }

    #[test]
    fn vertex_background_white_flashes_restore_without_touching_object_palettes() {
        let neutral = vec![[0.15, 0.35, 0.1, 1.0], [0.3, 0.4, 0.2, 0.5]];
        let mut source = source_test_frame(0xe4);
        let objects = source.objects.clone();
        for bgp in [0xe4, 0, 0xe4, 0xff, 0, 0xe4] {
            source.bgp = bgp;
            let (_, full) = source_environment_palette(Some(&source), BattleFlashMode::Full);
            let colors = surface_white_colors(&neutral, full);
            assert_eq!(
                colors,
                if bgp == 0 {
                    vec![[1.0; 4], [1.0, 1.0, 1.0, 0.5]]
                } else {
                    neutral.clone()
                }
            );
            let (_, reduced) = source_environment_palette(Some(&source), BattleFlashMode::Reduced);
            let reduced_colors = surface_white_colors(&neutral, reduced);
            if bgp == 0 {
                assert!(
                    reduced_colors[0][0] > neutral[0][0] && reduced_colors[0][0] < colors[0][0]
                );
            }
            assert_eq!(source.objects, objects);
        }
    }

    #[test]
    fn fallback_palette_frames_reuse_actor_entity_and_material() {
        let mut app = headless_battle_app();
        app.world_mut().resource_mut::<VisualBattleFrame>().source = Some(source_test_frame(0xe4));
        app.update();
        let original = {
            let scene = app.world().resource::<BattleScene>();
            let actor = scene.actors[VisualBattleSide::Enemy.index()]
                .as_ref()
                .unwrap();
            (actor.entity, actor.material.clone())
        };
        for image in 100..110 {
            let texture = Handle::weak_from_u128(image);
            app.world_mut()
                .resource_mut::<VisualBattleFrame>()
                .source
                .as_mut()
                .unwrap()
                .battler_textures[1] = texture.clone();
            app.update();
            let scene = app.world().resource::<BattleScene>();
            let actor = scene.actors[VisualBattleSide::Enemy.index()]
                .as_ref()
                .unwrap();
            assert_eq!((actor.entity, actor.material.clone()), original);
            assert_eq!(
                app.world()
                    .resource::<Assets<StandardMaterial>>()
                    .get(&actor.material)
                    .unwrap()
                    .base_color_texture,
                Some(texture)
            );
        }
    }

    #[test]
    fn source_motion_at_30_and_60_hz_preserves_clock_and_trajectory() {
        let mut endpoints = Vec::new();
        for hz in [30_u16, 60] {
            let mut app = headless_battle_app();
            app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_secs_f64(1.0 / f64::from(hz)),
            ));
            let mut last = None::<Vec3>;
            for sample in 0..=hz {
                let source_frame = sample * (60 / hz);
                let fraction = f32::from(source_frame) / 60.0;
                let mut source = source_test_frame(0xe4);
                source.frame = source_frame;
                source.objects[0].center =
                    // Displayed LCD positions have already removed hardware
                    // OAM (8,16); do not reintroduce it in this trajectory.
                    Vec2::new(40.0 + 84.0 * fraction, 72.0 - 40.0 * fraction);
                app.world_mut().resource_mut::<VisualBattleFrame>().source = Some(source);
                let before = app.world().resource::<VisualBattleFrame>().clone();
                app.update();
                assert_eq!(
                    *app.world().resource::<VisualBattleFrame>(),
                    before,
                    "display updates cannot advance source timing or palette events"
                );
                let world = app.world_mut();
                let position = world
                    .query::<(&BattleSourceObject, &Transform)>()
                    .iter(world)
                    .find(|(slot, _)| slot.0 == 0)
                    .unwrap()
                    .1
                    .translation;
                let anchors = world.resource::<BattleSceneLayout>().hit_anchors;
                let expected = anchors[0].lerp(anchors[1], fraction);
                assert!(position.distance(expected) < 0.0001);
                if let Some(last) = last {
                    let maximum_step = anchors[0].distance(anchors[1]) / f32::from(hz);
                    assert!(position.distance(last) <= maximum_step + 0.0001);
                }
                last = Some(position);
            }
            // A repeated presentation sample must hold its source position;
            // adding a display frame never fabricates another source tick.
            let before = app.world().resource::<VisualBattleFrame>().clone();
            app.update();
            assert_eq!(*app.world().resource::<VisualBattleFrame>(), before);
            endpoints.push(last.unwrap());
        }
        assert_eq!(endpoints[0], endpoints[1]);
    }

    #[test]
    fn vertex_lighting_preserves_geometry_and_full_source_palette_flashes() {
        let data = SurfaceMeshData {
            positions: vec![[0.0, 0.0, 0.0]; 2],
            normals: vec![[0.0, 1.0, 0.0], [0.0, -1.0, 0.0]],
            colors: vec![[0.8, 0.4, 0.2, 0.7]; 2],
            ..default()
        };
        let lit = vertex_lit_colors(&data, Quat::IDENTITY);
        assert!(lit[0][0] > lit[1][0]);
        assert_eq!(lit[0][3], 0.7);
        assert_eq!(data.positions, vec![[0.0, 0.0, 0.0]; 2]);
        let mapped = [1.0, 1.0, 1.0, 0.7];
        assert_eq!(
            source_lit_color(lit[0], mapped, 0xe4, BattleFlashMode::Full),
            lit[0]
        );
        assert_eq!(
            source_lit_color(lit[0], mapped, 0, BattleFlashMode::Full),
            mapped
        );
        let reduced = source_lit_color(lit[0], mapped, 0, BattleFlashMode::Reduced);
        assert!(reduced[0] > lit[0][0] && reduced[0] < mapped[0]);
        assert_eq!(reduced[3], 0.7);
    }

    #[test]
    fn stable_battle_frames_do_not_modify_material_assets() {
        let mut app = headless_battle_app();
        app.world_mut().resource_mut::<VisualBattleFrame>().source = Some(source_test_frame(0xe4));
        app.update();
        app.update();
        let changed = app
            .world()
            .get_resource_ref::<Assets<StandardMaterial>>()
            .unwrap()
            .last_changed();
        app.update();
        assert_eq!(
            app.world()
                .get_resource_ref::<Assets<StandardMaterial>>()
                .unwrap()
                .last_changed(),
            changed,
            "steady source frames must retain prepared material assets"
        );
    }

    #[test]
    fn move_cues_do_not_invent_battler_recoil() {
        let battler = VisualBattleBattler {
            side: VisualBattleSide::Enemy,
            species_id: Arc::from("RATTATA"),
            pokedex_size_m: Some(1.0),
            party_index: Some(0),
            texture: Handle::weak_from_u128(1),
            texture_size: Vec2::splat(56.0),
            source_rect: Rect::new(96.0, 0.0, 152.0, 56.0),
            source_opaque_rect: Rect::new(96.0, 0.0, 152.0, 56.0),
            visible: true,
            allow_species_model: true,
            shiny: false,
        };
        let cue = VisualBattleCue {
            kind: VisualBattleCueKind::Move,
            side: VisualBattleSide::Player,
            move_id: Arc::from("TACKLE"),
            element: Arc::from("NORMAL"),
            progress: 0.5,
            damaging: true,
        };
        assert_eq!(
            actor_pose(&battler, &[cue], 0.0, None, None).translation,
            side_position(battler.side)
                + camera_pose().rotation * Vec3::Y * (FALLBACK_CARD_HEIGHT * 0.5)
        );
    }
    #[test]
    fn hp_loss_cues_never_manufacture_particles_or_capture_choreography() {
        let mut app = headless_battle_app();
        app.world_mut()
            .resource_mut::<VisualBattleFrame>()
            .cues
            .push(VisualBattleCue {
                kind: VisualBattleCueKind::Impact,
                side: VisualBattleSide::Enemy,
                move_id: Arc::from("VISIBLE_HP_LOSS"),
                element: Arc::from("NORMAL"),
                progress: 0.5,
                damaging: true,
            });
        app.update();
        let world = app.world_mut();
        assert!(
            world
                .query_filtered::<&Visibility, Or<(With<BattleParticle>, With<BattleBall>)>>()
                .iter(world)
                .all(|v| *v == Visibility::Hidden)
        );
    }
    #[test]
    fn move_particles_do_not_invent_a_hit() {
        let cue = VisualBattleCue {
            kind: VisualBattleCueKind::Move,
            side: VisualBattleSide::Player,
            move_id: Arc::from("TACKLE"),
            element: Arc::from("NORMAL"),
            progress: 0.5,
            damaging: true,
        };
        assert!(particle_pose(&[cue], 0, 0.5).is_none());
    }
    #[test]
    fn faint_and_switch_poses_never_produce_invalid_transforms() {
        let battler = VisualBattleBattler {
            side: VisualBattleSide::Player,
            species_id: Arc::from("CHIKORITA"),
            pokedex_size_m: Some(1.0),
            party_index: Some(3),
            texture: Handle::weak_from_u128(1),
            texture_size: Vec2::splat(56.0),
            source_rect: Rect::new(96.0, 0.0, 152.0, 56.0),
            source_opaque_rect: Rect::new(96.0, 0.0, 152.0, 56.0),
            visible: true,
            allow_species_model: true,
            shiny: false,
        };
        for kind in [
            VisualBattleCueKind::Faint,
            VisualBattleCueKind::SendOut,
            VisualBattleCueKind::Withdraw,
        ] {
            for progress in [0.0, 0.5, 1.0] {
                let cue = VisualBattleCue {
                    kind,
                    side: battler.side,
                    move_id: Arc::from(""),
                    element: Arc::from(""),
                    progress,
                    damaging: false,
                };
                let pose = actor_pose(&battler, &[cue], 0.2, None, None);
                assert!(pose.translation.is_finite());
                assert!(pose.scale.min_element() > 0.0);
            }
        }
    }
    #[test]
    fn every_authored_arena_has_finite_grounded_geometry() {
        for environment in [
            VisualBattleEnvironment::Meadow,
            VisualBattleEnvironment::Forest,
            VisualBattleEnvironment::Cave,
            VisualBattleEnvironment::Water,
            VisualBattleEnvironment::Interior,
            VisualBattleEnvironment::Ice,
        ] {
            let mesh = arena_mesh(environment);
            assert!(mesh.positions.len() > 1000);
            assert_eq!(mesh.positions.len(), mesh.normals.len());
            assert_eq!(mesh.positions.len(), mesh.colors.len());
            assert!(mesh.positions.iter().flatten().all(|v| v.is_finite()));
            assert!(
                mesh.indices
                    .iter()
                    .all(|i| (*i as usize) < mesh.positions.len())
            );
        }
    }

    #[test]
    fn capture_ball_is_original_volume_with_outward_normals() {
        let mesh = capture_ball_mesh();
        assert!(mesh.indices.len() > 500);
        for (p, n) in mesh.positions.iter().zip(&mesh.normals) {
            assert!(Vec3::from_array(*p).dot(Vec3::from_array(*n)) > 0.0);
        }
    }

    pub(super) fn headless_battle_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<Image>>()
            .init_resource::<Assets<StandardMaterial>>()
            .insert_resource(VoxelViewSettings {
                enabled: true,
                ..Default::default()
            })
            // Mirror PBR's camera initialization order in headless tests.
            // Inactive actor cameras do not receive clusters at startup.
            .add_systems(
                PostUpdate,
                bevy::pbr::add_clusters.in_set(bevy::pbr::SimulationLightSystems::AddClusters),
            )
            .add_plugins(BattleViewPlugin);
        let battler = |side, species: &str| VisualBattleBattler {
            side,
            species_id: Arc::from(species),
            pokedex_size_m: Some(if species == "CYNDAQUIL" { 0.508 } else { 1.0 }),
            party_index: Some(side.index()),
            texture: Handle::weak_from_u128(20 + side.index() as u128),
            texture_size: Vec2::splat(56.0),
            source_rect: Rect::new(96.0, 0.0, 152.0, 56.0),
            source_opaque_rect: Rect::new(96.0, 0.0, 152.0, 56.0),
            visible: true,
            allow_species_model: true,
            shiny: false,
        };
        app.insert_resource(VisualBattleFrame {
            active: true,
            map_id: Arc::from("Route29"),
            battlers: [
                Some(battler(VisualBattleSide::Player, "CYNDAQUIL")),
                Some(battler(VisualBattleSide::Enemy, "UNAUTHORED_SPECIES")),
            ],
            ..Default::default()
        });
        app
    }

    #[test]
    fn unchanged_source_frame_holds_camera_and_idle_actor_poses() {
        let mut app = headless_battle_app();
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 30.0),
        ));
        app.world_mut().resource_mut::<VisualBattleFrame>().source = Some(source_test_frame(0xe4));
        app.update();
        let poses = |app: &mut App| {
            let world = app.world_mut();
            world
                .query_filtered::<(Entity, &Transform), Or<(With<BattleCamera>, With<BattleActor>)>>()
                .iter(world)
                .map(|(entity, pose)| (entity, *pose))
                .collect::<Vec<_>>()
        };
        let before = poses(&mut app);
        let source = app.world().resource::<VisualBattleFrame>().clone();
        for _ in 0..90 {
            app.update();
        }
        assert_eq!(
            poses(&mut app),
            before,
            "wall time cannot invent idle or camera movement"
        );
        assert_eq!(*app.world().resource::<VisualBattleFrame>(), source);
    }

    #[test]
    fn battle_scene_switches_modes_restores_world_and_bounds_entity_counts() {
        let mut app = headless_battle_app();
        app.update();
        assert!(app.world().resource::<BattleViewStatus>().active);
        assert_eq!(
            app.world().resource::<BattleViewStatus>().modeled_species,
            ["CYNDAQUIL"]
        );
        assert_eq!(
            app.world()
                .resource::<BattleViewStatus>()
                .source_art_species,
            ["UNAUTHORED_SPECIES"]
        );
        let count = app.world().entities().len();
        let mesh_count = app.world().resource::<Assets<Mesh>>().len();
        for _ in 0..20 {
            app.update();
        }
        assert_eq!(
            app.world().entities().len(),
            count,
            "idle must reuse the scene and bounded particle pool"
        );
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), mesh_count);
        app.world_mut().resource_mut::<VoxelViewSettings>().enabled = false;
        app.update();
        assert!(!app.world().resource::<BattleViewStatus>().active);
        {
            let world = app.world_mut();
            assert!(
                !world
                    .query_filtered::<&Camera, With<BattleCamera>>()
                    .single(world)
                    .is_active
            );
            assert!(
                world
                    .query_filtered::<&Visibility, With<BattleActor>>()
                    .iter(world)
                    .all(|v| *v == Visibility::Hidden)
            );
        }
        app.world_mut().resource_mut::<VoxelViewSettings>().enabled = true;
        app.update();
        assert!(app.world().resource::<BattleViewStatus>().active);
        *app.world_mut().resource_mut::<VisualBattleFrame>() = VisualBattleFrame::default();
        app.update();
        assert!(!app.world().resource::<BattleViewStatus>().active);
        let world = app.world_mut();
        assert!(
            !world
                .query_filtered::<&Camera, With<BattleCamera>>()
                .single(world)
                .is_active
        );
    }

    pub(super) fn source_test_frame(bgp: u8) -> VisualBattleSourceFrame {
        VisualBattleSourceFrame {
            frame: 17,
            bgp,
            battler_bgps: [bgp; 2],
            battler_palettes: [[
                [1.0, 1.0, 1.0, 1.0],
                [0.7, 0.7, 0.7, 1.0],
                [0.3, 0.3, 0.3, 1.0],
                [0.0, 0.0, 0.0, 1.0],
            ]; 2],
            battler_textures: [Handle::weak_from_u128(50), Handle::weak_from_u128(51)],
            battler_offsets: [Vec2::new(4.0, 0.0), Vec2::ZERO],
            screen_offset: Vec2::ZERO,
            line_x_offsets: None,
            line_y_offsets: None,
            battler_rows: [None; 2],
            objects: vec![crystal_render_api::VisualBattleSourceObject {
                slot: 0,
                object_id: Arc::from("PRESENTED_SOURCE_OBJECT"),
                texture: Handle::weak_from_u128(60),
                neutral_texture: Handle::weak_from_u128(61),
                center: Vec2::new(92.0, 68.0),
                size: Vec2::splat(16.0),
                uv_rect: Rect::from_corners(Vec2::ZERO, Vec2::ONE),
            }],
        }
    }

    #[test]
    fn source_palette_full_and_reduced_preserve_timing_and_limit_contrast() {
        let source = source_test_frame(0x1b);
        let dark = [0.0, 0.0, 0.0, 1.0];
        let palette = &source.battler_palettes[0];
        assert_eq!(
            source_model_color(dark, 0x1b, palette, BattleFlashMode::Full),
            [1.0; 4]
        );
        let reduced = source_model_color(dark, 0x1b, palette, BattleFlashMode::Reduced);
        assert_eq!(reduced, [0.12, 0.12, 0.12, 1.0]);
        assert_eq!(
            source_model_color(dark, 0xe4, palette, BattleFlashMode::Full),
            dark
        );
        assert_eq!(
            source_environment_palette(Some(&source), BattleFlashMode::Full),
            (1.0, 0.0)
        );
        assert_eq!(
            source_environment_palette(Some(&source), BattleFlashMode::Reduced),
            (0.12, 0.0)
        );
        let mut app = headless_battle_app();
        app.world_mut().resource_mut::<VisualBattleFrame>().source = Some(source);
        let before = app.world().resource::<VisualBattleFrame>().clone();
        app.update();
        let source_pose = |app: &mut App| {
            let world = app.world_mut();
            world
                .query::<(&BattleSourceObject, &Transform)>()
                .iter(world)
                .find(|(slot, _)| slot.0 == 0)
                .unwrap()
                .1
                .to_owned()
        };
        let full = source_pose(&mut app);
        app.world_mut().insert_resource(BattleFlashMode::Reduced);
        app.update();
        let reduced = source_pose(&mut app);
        assert_eq!(full.translation, reduced.translation);
        assert_eq!(full.scale, reduced.scale);
        assert_eq!(
            *app.world().resource::<VisualBattleFrame>(),
            before,
            "flash intensity cannot rewrite the source frame, cue phase, HP or command data"
        );
        let world = app.world_mut();
        let material = world
            .query::<(&BattleSourceObject, &Handle<StandardMaterial>)>()
            .iter(world)
            .find(|(slot, _)| slot.0 == 0)
            .unwrap()
            .1
            .clone();
        assert_eq!(
            world
                .resource::<Assets<StandardMaterial>>()
                .get(&material)
                .unwrap()
                .base_color_texture,
            Some(Handle::weak_from_u128(61)),
            "reduced mode retains OAM/frame but suppresses object palette cycling"
        );
    }

    #[test]
    fn psychic_and_beam_volumes_require_current_source_objects() {
        let layout = BattleSceneLayout::default();
        let mut frame = VisualBattleFrame {
            source: Some(source_test_frame(0xe4)),
            cues: vec![VisualBattleCue {
                kind: VisualBattleCueKind::Move,
                side: VisualBattleSide::Player,
                move_id: Arc::from("PSYCHIC_M"),
                element: Arc::from("PSYCHIC"),
                progress: 0.3,
                damaging: true,
            }],
            ..Default::default()
        };
        assert!(modeled_source_effect_pose(&layout, &frame, 0).is_none());
        frame.source.as_mut().unwrap().objects[0].object_id = Arc::from("BATTLE_ANIM_OBJ_WAVE");
        let ring = modeled_source_effect_pose(&layout, &frame, 0).unwrap();
        assert_eq!(ring.0, 0);
        assert!(ring.1.translation.is_finite());
        assert!(modeled_source_effect_pose(&layout, &frame, 1).is_none());
        assert!(modeled_source_effect_pose(&layout, &frame, 10).is_none());
        frame.source.as_mut().unwrap().line_x_offsets = Some([5; 0x5f]);
        assert!(modeled_source_effect_pose(&layout, &frame, 10).is_none());
        frame.cues[0].move_id = Arc::from("HYPER_BEAM");
        assert!(modeled_source_effect_pose(&layout, &frame, 0).is_none());
        assert!(modeled_source_effect_pose(&layout, &frame, 10).is_none());
        frame.source.as_mut().unwrap().objects[0].object_id = Arc::from("BATTLE_ANIM_OBJ_BEAM");
        let beam = modeled_source_effect_pose(&layout, &frame, 0).unwrap();
        assert_eq!(beam.0, 1);
        assert_eq!(beam.1.translation, ring.1.translation);
        frame.source.as_mut().unwrap().objects.clear();
        assert!(modeled_source_effect_pose(&layout, &frame, 0).is_none());
        frame.source = None;
        assert!((0..13).all(|slot| modeled_source_effect_pose(&layout, &frame, slot).is_none()));
    }

    #[test]
    fn modeled_source_effects_restore_on_interruption_and_reduce_only_intensity() {
        let mut app = headless_battle_app();
        {
            let mut frame = app.world_mut().resource_mut::<VisualBattleFrame>();
            frame.source = Some(source_test_frame(0xe4));
            frame.source.as_mut().unwrap().objects[0].object_id = Arc::from("BATTLE_ANIM_OBJ_WAVE");
            frame.source.as_mut().unwrap().line_x_offsets = Some([5; 0x5f]);
            frame.cues.push(VisualBattleCue {
                kind: VisualBattleCueKind::Move,
                side: VisualBattleSide::Player,
                move_id: Arc::from("PSYCHIC_M"),
                element: Arc::from("PSYCHIC"),
                progress: 0.3,
                damaging: true,
            });
        }
        app.update();
        let poses = |app: &mut App| {
            let mut query = app
                .world_mut()
                .query::<(&BattleModeledSourceEffect, &Transform, &Visibility)>();
            query
                .iter(app.world())
                .map(|(slot, pose, visibility)| (slot.0, *pose, *visibility))
                .collect::<Vec<_>>()
        };
        let full = poses(&mut app);
        assert_eq!(
            full.iter()
                .filter(|(_, _, visibility)| *visibility == Visibility::Visible)
                .count(),
            0,
            "native OAM stays outside the background row sampler"
        );
        app.world_mut().insert_resource(BattleFlashMode::Reduced);
        app.update();
        assert_eq!(poses(&mut app), full);
        app.world_mut().resource_mut::<VisualBattleFrame>().source = None;
        app.update();
        assert!(
            poses(&mut app)
                .iter()
                .all(|(_, _, visibility)| *visibility == Visibility::Hidden)
        );
    }

    #[test]
    fn source_effect_pools_stay_hidden_through_toggle_and_interruption() {
        let mut app = headless_battle_app();
        app.world_mut().resource_mut::<VisualBattleFrame>().source = Some(source_test_frame(0x1b));
        app.update();
        let counts = (
            app.world().entities().len(),
            app.world().resource::<Assets<Mesh>>().len(),
            app.world().resource::<Assets<StandardMaterial>>().len(),
        );
        for _ in 0..20 {
            app.update();
        }
        assert_eq!(
            (
                app.world().entities().len(),
                app.world().resource::<Assets<Mesh>>().len(),
                app.world().resource::<Assets<StandardMaterial>>().len()
            ),
            counts
        );
        let visible_count = |app: &mut App| {
            let world = app.world_mut();
            world
                .query_filtered::<&Visibility, With<BattleSourceObject>>()
                .iter(world)
                .filter(|visibility| **visibility == Visibility::Visible)
                .count()
        };
        assert_eq!(visible_count(&mut app), 0);
        app.world_mut().resource_mut::<VoxelViewSettings>().enabled = false;
        app.update();
        assert_eq!(
            visible_count(&mut app),
            0,
            "classic must not double-render source objects"
        );
        app.world_mut().resource_mut::<VoxelViewSettings>().enabled = true;
        app.update();
        assert_eq!(visible_count(&mut app), 0);
        app.world_mut().resource_mut::<VisualBattleFrame>().source = None;
        app.update();
        assert_eq!(visible_count(&mut app), 0);
        assert_eq!(
            app.world().resource::<BattleScene>().actors[0]
                .as_ref()
                .unwrap()
                .palette_key
                .0,
            0xe4
        );
        *app.world_mut().resource_mut::<VisualBattleFrame>() = VisualBattleFrame::default();
        app.update();
        assert_eq!(
            visible_count(&mut app),
            0,
            "capture/Dex/world interruptions retire all source effects"
        );
    }

    #[test]
    fn source_scroll_uniform_preserves_rows_axes_palette_and_intensity() {
        let layout = BattleSceneLayout::default();
        let mut frame = VisualBattleFrame {
            source: Some(source_test_frame(0xe4)),
            ..Default::default()
        };
        let source = frame.source.as_mut().unwrap();
        source.line_x_offsets = Some([5; 95]);
        source.line_y_offsets = Some([-2; 95]);
        source.screen_offset = Vec2::new(3.0, -4.0);
        let normal = battle_composite_uniform(
            &layout,
            &frame,
            BattleFlashMode::Full,
            Vec2::new(1180.0, 812.0),
        );
        assert!(
            normal.rows[..95]
                .iter()
                .all(|row| row.x == 5.0 && row.y == -2.0)
        );
        assert_eq!(
            normal.rows[95],
            Vec4::ZERO,
            "LCD rows 95 onward cannot inherit row scroll"
        );
        assert_eq!(normal.screen_offset, Vec4::new(3.0, 4.0, 1.0, 0.0));
        assert_eq!(
            normal.background,
            rgb(palette(frame.environment).sky).to_linear().to_vec4()
        );
        frame.source.as_mut().unwrap().bgp = 0xff;
        let full = battle_composite_uniform(
            &layout,
            &frame,
            BattleFlashMode::Full,
            Vec2::new(1180.0, 812.0),
        );
        let reduced = battle_composite_uniform(
            &layout,
            &frame,
            BattleFlashMode::Reduced,
            Vec2::new(1180.0, 812.0),
        );
        assert_eq!(full.rows, normal.rows);
        assert_eq!(reduced.rows, normal.rows);
        assert_eq!(full.background, Vec4::new(0.0, 0.0, 0.0, 1.0));
        assert!(reduced.background.x > 0.0 && reduced.background.x < 1.0);
        assert_eq!(
            battle_composite_uniform(
                &layout,
                &VisualBattleFrame::default(),
                BattleFlashMode::Full,
                Vec2::new(1180.0, 812.0)
            ),
            BattleCompositeUniform::default()
        );
    }

    #[test]
    fn scanline_sampler_keeps_models_and_retains_native_oam_outside_target() {
        let mut app = headless_battle_app();
        app.world_mut().resource_mut::<VisualBattleFrame>().source = Some(source_test_frame(0xe4));
        app.update();
        let source = app.world().resource::<VisualBattleFrame>().clone();
        let actor = app.world().resource::<BattleScene>().actors[0]
            .as_ref()
            .unwrap()
            .entity;
        let transform = *app.world().get::<Transform>(actor).unwrap();
        app.world_mut()
            .resource_mut::<VisualBattleFrame>()
            .source
            .as_mut()
            .unwrap()
            .line_y_offsets = Some([2; 95]);
        app.update();
        assert!(app.world().resource::<BattleViewStatus>().active);
        assert_eq!(
            app.world().get::<Transform>(actor),
            Some(&transform),
            "sampling must not bend model geometry"
        );
        assert_eq!(
            app.world().get::<Visibility>(actor),
            Some(&Visibility::Visible)
        );
        let world = app.world_mut();
        assert!(
            world
                .query_filtered::<&Visibility, With<BattleSourceObject>>()
                .iter(world)
                .all(|v| *v == Visibility::Hidden)
        );
        assert_eq!(
            world
                .query_filtered::<&Camera, With<BattleCamera>>()
                .iter(world)
                .count(),
            1
        );
        assert_eq!(
            world
                .resource::<VisualBattleFrame>()
                .source
                .as_ref()
                .unwrap()
                .frame,
            source.source.as_ref().unwrap().frame
        );
        app.world_mut()
            .resource_mut::<VisualBattleFrame>()
            .source
            .as_mut()
            .unwrap()
            .line_y_offsets = None;
        app.update();
        let world = app.world_mut();
        assert_eq!(
            world
                .query_filtered::<&Visibility, With<BattleSourceObject>>()
                .iter(world)
                .filter(|v| **v == Visibility::Visible)
                .count(),
            0
        );
    }

    #[test]
    fn stable_composite_reuses_material_and_resets_sampler_after_interruptions() {
        let mut app = headless_battle_app();
        app.world_mut()
            .spawn((Window::default(), bevy::window::PrimaryWindow));
        app.world_mut().spawn(Camera2dBundle::default());
        app.world_mut().resource_mut::<VisualBattleFrame>().source = Some(source_test_frame(0xe4));
        app.world_mut()
            .resource_mut::<VisualBattleFrame>()
            .source
            .as_mut()
            .unwrap()
            .line_x_offsets = Some([4; 95]);
        app.update();
        app.update();
        let changes = app
            .world()
            .get_resource_ref::<Assets<BattleCompositeMaterial>>()
            .unwrap()
            .last_changed();
        app.update();
        assert_eq!(
            app.world()
                .get_resource_ref::<Assets<BattleCompositeMaterial>>()
                .unwrap()
                .last_changed(),
            changes
        );
        app.world_mut().resource_mut::<VisualBattleFrame>().source = None;
        app.update();
        let materials = app.world().resource::<Assets<BattleCompositeMaterial>>();
        assert_eq!(materials.len(), 1);
        assert_eq!(
            materials.iter().next().unwrap().1.source,
            BattleCompositeUniform::default()
        );
    }

    #[test]
    fn changing_frontpic_footprints_reuse_cached_species_bounds() {
        let mut app = headless_battle_app();
        app.update();
        let cached = app.world().resource::<BattleScene>().species_bounds.clone();
        let actor = app.world().resource::<BattleScene>().actors[0]
            .as_ref()
            .unwrap()
            .entity;
        let mesh_count = app.world().resource::<Assets<Mesh>>().len();
        for width in [40.0, 42.0, 38.0, 40.0] {
            app.world_mut().resource_mut::<VisualBattleFrame>().battlers[0]
                .as_mut()
                .unwrap()
                .source_opaque_rect = Rect::new(20.0, 52.0, 20.0 + width, 96.0);
            app.update();
            let scene = app.world().resource::<BattleScene>();
            assert_eq!(scene.species_bounds, cached);
            assert_eq!(scene.actors[0].as_ref().unwrap().entity, actor);
            assert_eq!(app.world().resource::<Assets<Mesh>>().len(), mesh_count);
        }
    }

    #[test]
    fn offscreen_source_effects_stay_hidden_while_source_metadata_remains() {
        let mut app = headless_battle_app();
        for (move_id, object_id) in [
            ("PSYCHIC_M", "BATTLE_ANIM_OBJ_WAVE"),
            ("HYPER_BEAM", "BATTLE_ANIM_OBJ_BEAM"),
            ("HYPER_BEAM", "BATTLE_ANIM_OBJ_BEAM_TIP"),
        ] {
            let mut frame = app.world_mut().resource_mut::<VisualBattleFrame>();
            frame.source = Some(source_test_frame(0xe4));
            frame.source.as_mut().unwrap().objects[0].object_id = Arc::from(object_id);
            frame.cues = vec![VisualBattleCue {
                kind: VisualBattleCueKind::Move,
                side: VisualBattleSide::Player,
                move_id: Arc::from(move_id),
                element: Arc::from("NORMAL"),
                progress: 0.5,
                damaging: true,
            }];
            let source = frame.source.clone();
            drop(frame);
            app.update();
            let world = app.world_mut();
            assert!(
                world
                    .query_filtered::<&Visibility, With<BattleModeledSourceEffect>>()
                    .iter(world)
                    .all(|v| *v == Visibility::Hidden)
            );
            assert_eq!(
                world
                    .query_filtered::<&Visibility, With<BattleSourceObject>>()
                    .iter(world)
                    .filter(|v| **v == Visibility::Visible)
                    .count(),
                0
            );
            assert_eq!(world.resource::<VisualBattleFrame>().source, source);
        }
    }

    #[test]
    fn native_oam_metadata_stays_cached_without_entering_scrolled_bg_target() {
        let mut app = headless_battle_app();
        app.world_mut().resource_mut::<VisualBattleFrame>().source = Some(source_test_frame(0xe4));
        app.update();
        let count = app.world().resource::<Assets<StandardMaterial>>().len();
        for (offset, crop) in [
            (Vec2::new(3.0, -2.0), Rect::new(0.25, 0.0, 1.0, 0.75)),
            (Vec2::ZERO, Rect::new(0.0, 0.5, 0.75, 1.0)),
            (Vec2::new(-3.0, 2.0), Rect::new(0.0, 0.0, 1.0, 1.0)),
        ] {
            app.world_mut()
                .resource_mut::<VisualBattleFrame>()
                .source
                .as_mut()
                .unwrap()
                .screen_offset = offset;
            app.world_mut()
                .resource_mut::<VisualBattleFrame>()
                .source
                .as_mut()
                .unwrap()
                .objects[0]
                .uv_rect = crop;
            app.update();
            let world = app.world_mut();
            assert!(
                world
                    .query_filtered::<&Visibility, With<BattleSourceObject>>()
                    .iter(world)
                    .all(|visibility| *visibility == Visibility::Hidden)
            );
            let material = world
                .query::<(&BattleSourceObject, &Handle<StandardMaterial>)>()
                .iter(world)
                .find(|(slot, _)| slot.0 == 0)
                .map(|(_, material)| material.clone())
                .unwrap();
            let material = world
                .resource::<Assets<StandardMaterial>>()
                .get(&material)
                .unwrap();
            assert_eq!(
                material.base_color_texture,
                Some(Handle::weak_from_u128(60))
            );
            assert_eq!(material.uv_transform.transform_point2(Vec2::ZERO), crop.min);
            assert_eq!(material.uv_transform.transform_point2(Vec2::ONE), crop.max);
            assert_eq!(world.resource::<Assets<StandardMaterial>>().len(), count);
        }
    }

    #[test]
    fn projected_source_rows_match_oam_across_aspects_scroll_signs_and_modes() {
        let layout = BattleSceneLayout::default();
        for viewport in [
            Vec2::new(1180.0, 812.0),
            Vec2::new(1600.0, 900.0),
            Vec2::new(812.0, 1180.0),
        ] {
            let projection = source_plane_projection(&layout, viewport);
            let inverse = projection.inverse();
            for center in [
                Vec2::new(48.0, 88.0),
                Vec2::new(96.0, 72.0),
                Vec2::new(132.0, 48.0),
            ] {
                let uv = project_source_point(projection, center);
                assert!(
                    (project_source_point(inverse, uv) - center)
                        .abs()
                        .max_element()
                        < 0.002
                );
                let oam = battle_source_overlay_rect(&layout, center, Vec2::splat(16.0), viewport)
                    .unwrap();
                assert!(uv.cmpge(oam.min / viewport).all() && uv.cmple(oam.max / viewport).all());
                let source_rect = Rect::from_center_size(center, Vec2::splat(16.0));
                for corner in [
                    source_rect.min,
                    source_rect.max,
                    Vec2::new(source_rect.min.x, source_rect.max.y),
                    Vec2::new(source_rect.max.x, source_rect.min.y),
                ] {
                    let point = project_source_point(projection, corner) * viewport;
                    assert!(
                        point.cmpge(oam.min - Vec2::splat(0.001)).all()
                            && point.cmple(oam.max + Vec2::splat(0.001)).all()
                    );
                }
                for offset in [
                    Vec2::new(5.0, 0.0),
                    Vec2::new(-5.0, 0.0),
                    Vec2::new(0.0, 3.0),
                    Vec2::new(0.0, -3.0),
                    Vec2::ZERO,
                ] {
                    let sampled = project_source_point(
                        projection,
                        project_source_point(inverse, uv) + offset,
                    );
                    assert!(
                        (project_source_point(inverse, sampled) - center - offset)
                            .abs()
                            .max_element()
                            < 0.002
                    );
                }
            }
            let mut frame = VisualBattleFrame {
                source: Some(source_test_frame(0xe4)),
                ..default()
            };
            let original = frame.clone();
            let neutral =
                battle_composite_uniform(&layout, &frame, BattleFlashMode::Full, viewport);
            assert_eq!(
                neutral.screen_offset.z, 0.0,
                "zero register offsets bypass the sampler"
            );
            for value in [-3, 0, 3] {
                frame.source.as_mut().unwrap().line_y_offsets = Some([value; 95]);
                let full =
                    battle_composite_uniform(&layout, &frame, BattleFlashMode::Full, viewport);
                let reduced =
                    battle_composite_uniform(&layout, &frame, BattleFlashMode::Reduced, viewport);
                assert_eq!(full.rows, reduced.rows);
                assert_eq!(full.source_to_view, reduced.source_to_view);
                assert_eq!(full.view_to_source, reduced.view_to_source);
                assert_eq!(full.screen_offset.z, if value == 0 { 0.0 } else { 1.0 });
            }
            frame.source.as_mut().unwrap().line_y_offsets = None;
            assert_eq!(
                frame, original,
                "projection never advances or rewrites the source frame"
            );
            assert_eq!(
                battle_composite_uniform(&layout, &frame, BattleFlashMode::Full, viewport),
                neutral
            );
        }
    }

    #[test]
    fn source_objects_map_continuously_between_immersive_battlers() {
        let layout = BattleSceneLayout::default();
        // The bridge has removed the hardware OAM origin (8,16) once.
        // These are displayed LCD anchors, not raw hardware coordinates.
        let start = Vec2::new(40.0, 72.0);
        let end = Vec2::new(124.0, 32.0);
        for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let size = Vec2::new(16.0, 24.0);
            let pose = source_object_pose(&layout, start.lerp(end, t), size);
            assert!(
                pose.translation.distance(
                    side_position(VisualBattleSide::Player)
                        .lerp(side_position(VisualBattleSide::Enemy), t)
                        + Vec3::Y
                ) < 0.0001
            );
            assert_eq!(pose.scale.truncate(), size * SOURCE_PIXEL_WORLD);
            assert_eq!(pose.rotation, camera_pose().rotation);
            let rect = battle_source_overlay_rect(
                &layout,
                start.lerp(end, t),
                size,
                Vec2::new(1180.0, 812.0),
            )
            .unwrap();
            assert!(rect.min.is_finite() && rect.max.is_finite());
            assert!(rect.size().min_element() > 0.0);
        }
        let raw_hardware = start + Vec2::new(8.0, 16.0);
        assert!(
            source_object_pose(&layout, raw_hardware, Vec2::ONE)
                .translation
                .distance(side_position(VisualBattleSide::Player) + Vec3::Y)
                > 0.1,
            "raw OAM coordinates must not be mistaken for displayed pixels"
        );
        assert!(battle_source_overlay_rect(&layout, start, Vec2::ONE, Vec2::ZERO).is_none());
    }

    #[test]
    fn immersive_models_keep_authored_proportions_and_source_motion() {
        let data = SurfaceMeshData {
            positions: vec![[-0.2, 0.0, -0.1], [0.3, 1.2, 0.2]],
            ..Default::default()
        };
        let app = headless_battle_app();
        let mut battler = app.world().resource::<VisualBattleFrame>().battlers[0]
            .clone()
            .unwrap();
        battler.species_id = Arc::from("SUDOWOODO");
        battler.pokedex_size_m = Some(1.1938);
        for side in [VisualBattleSide::Player, VisualBattleSide::Enemy] {
            battler.side = side;
            let bounds = Some(BattleModelBounds::from_surface(&data));
            let neutral = actor_pose(&battler, &[], 0.0, bounds, None);
            assert_eq!(neutral.translation, side_position(side));
            assert_eq!(neutral.scale, Vec3::splat(neutral.scale.x));
            assert!((neutral.scale.y * 1.2 - 1.1938 * MODEL_SCALE).abs() < 0.00001);
            battler.source_opaque_rect = Rect::new(10.0, 20.0, 25.0, 100.0);
            assert_eq!(actor_pose(&battler, &[], 0.0, bounds, None), neutral);
            let mut source = source_test_frame(0xe4);
            source.battler_offsets[side.index()] = Vec2::new(4.0, -3.0);
            let moved = actor_pose(&battler, &[], 200.0, bounds, Some(&source));
            assert_eq!(
                moved.translation,
                neutral.translation + source_battler_displacement(Vec2::new(4.0, -3.0))
            );
            assert_eq!(moved.rotation, neutral.rotation);
            assert_eq!(moved.scale, neutral.scale);
        }
    }

    #[test]
    fn immersive_scene_restores_arena_camera_and_contact_shadows() {
        let mut app = headless_battle_app();
        app.world_mut().resource_mut::<VisualBattleFrame>().source = Some(source_test_frame(0xe4));
        app.update();
        let world = app.world_mut();
        let scene = world.resource::<BattleScene>();
        assert!(scene.arena.is_some() && scene.arena_mesh.is_some());
        let actor = scene.actors[0].as_ref().unwrap().entity;
        let scale = world.get::<Transform>(actor).unwrap().scale;
        assert_eq!(scale, Vec3::splat(scale.x));
        assert!((scale.y * 0.716729 - 0.508 * MODEL_SCALE).abs() < 0.00001);
        assert_eq!(
            world.get::<Transform>(actor).unwrap().translation,
            side_position(VisualBattleSide::Player)
                + source_battler_displacement(Vec2::new(4.0, 0.0))
        );
        assert!(
            world
                .query_filtered::<&Projection, With<BattleCamera>>()
                .iter(world)
                .all(|p| matches!(p, Projection::Perspective(_)))
        );
        assert_eq!(
            world
                .query_filtered::<&Visibility, With<BattleContactShadow>>()
                .iter(world)
                .filter(|v| **v == Visibility::Visible)
                .count(),
            2
        );
        world.resource_mut::<VoxelViewSettings>().enabled = false;
        app.update();
        let world = app.world_mut();
        assert!(
            world
                .query_filtered::<&Visibility, Or<(With<BattleArena>, With<BattleContactShadow>)>>()
                .iter(world)
                .all(|v| *v == Visibility::Hidden)
        );
    }

    #[test]
    fn native_and_scaled_quality_use_one_bounded_immersive_target() {
        for quality in [
            BattleRenderQuality::Native,
            BattleRenderQuality::Balanced,
            BattleRenderQuality::Performance,
        ] {
            let mut app = headless_battle_app();
            app.world_mut().resource_mut::<BattleScene>().quality = quality;
            app.world_mut()
                .spawn((Window::default(), bevy::window::PrimaryWindow));
            app.insert_resource(VisualBattleCanvas {
                size: Vec2::new(960.0, 864.0),
                physical_size: UVec2::new(960, 864),
            });
            app.update();
            let scene = app.world().resource::<BattleScene>();
            assert!(scene.render_target.is_some());
            assert_eq!(scene.render_size, quality.target_size(UVec2::new(960, 864)));
            let world = app.world_mut();
            assert_eq!(
                world
                    .query_filtered::<&Camera, With<BattleCamera>>()
                    .iter(world)
                    .count(),
                1
            );
            assert_eq!(
                world
                    .query_filtered::<&Mesh2dHandle, With<BattleCompositeSprite>>()
                    .iter(world)
                    .count(),
                1
            );
        }
    }

    #[test]
    fn source_only_sequence_restores_models_without_advancing_source() {
        let mut app = headless_battle_app();
        app.world_mut().resource_mut::<VisualBattleFrame>().source = Some(source_test_frame(0xe4));
        app.update();
        let counts = (
            app.world().entities().len(),
            app.world().resource::<Assets<Mesh>>().len(),
        );
        let saved = app.world().resource::<VisualBattleFrame>().clone();
        for source_only in [true, false, true, false] {
            app.world_mut()
                .resource_mut::<VisualBattleFrame>()
                .use_source_scene = source_only;
            app.update();
            let frame = app.world().resource::<VisualBattleFrame>();
            assert_eq!(frame.source, saved.source);
            assert_eq!(frame.cues, saved.cues);
            assert_eq!(
                app.world().resource::<BattleViewStatus>().active,
                !source_only
            );
            assert_eq!(
                (
                    app.world().entities().len(),
                    app.world().resource::<Assets<Mesh>>().len()
                ),
                counts
            );
        }
    }

    #[test]
    fn rejected_battle_frame_cannot_hide_the_classic_scene() {
        let mut app = headless_battle_app();
        app.world_mut().resource_mut::<VisualBattleFrame>().battlers[0]
            .as_mut()
            .unwrap()
            .texture = Handle::default();
        app.update();
        let status = app.world().resource::<BattleViewStatus>();
        assert!(!status.active);
        assert!(status.last_error.is_some());
    }

    #[test]
    fn target_resize_rebinds_neutral_composite_without_per_frame_asset_changes() {
        let mut app = headless_battle_app();
        let window = app
            .world_mut()
            .spawn((Window::default(), bevy::window::PrimaryWindow))
            .id();
        app.world_mut().spawn(Camera2dBundle::default());
        app.world_mut().resource_mut::<VisualBattleFrame>().source = None;
        app.update();
        app.update();
        let stable = app
            .world()
            .get_resource_ref::<Assets<BattleCompositeMaterial>>()
            .unwrap()
            .last_changed();
        app.update();
        assert_eq!(
            app.world()
                .get_resource_ref::<Assets<BattleCompositeMaterial>>()
                .unwrap()
                .last_changed(),
            stable
        );
        let material = app
            .world()
            .resource::<Assets<BattleCompositeMaterial>>()
            .iter()
            .next()
            .unwrap()
            .1
            .clone();
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .resolution
            .set(1200.0, 800.0);
        app.update();
        let resized = app
            .world()
            .get_resource_ref::<Assets<BattleCompositeMaterial>>()
            .unwrap()
            .last_changed();
        assert_ne!(
            resized, stable,
            "new GPU texture view needs a new bind group even with neutral uniforms"
        );
        let current = app
            .world()
            .resource::<Assets<BattleCompositeMaterial>>()
            .iter()
            .next()
            .unwrap()
            .1;
        assert_eq!(current.source, material.source);
        assert_eq!(
            current.texture, material.texture,
            "stable asset handles must not conceal GPU view replacement"
        );
        app.update();
        assert_eq!(
            app.world()
                .get_resource_ref::<Assets<BattleCompositeMaterial>>()
                .unwrap()
                .last_changed(),
            resized
        );
        assert_eq!(
            app.world()
                .resource::<Assets<BattleCompositeMaterial>>()
                .len(),
            1
        );
    }
    #[test]
    fn row_targets_keep_stable_projection_ticks_and_refresh_dirty_inputs() {
        let mut app = headless_battle_app();
        let window = app
            .world_mut()
            .spawn((Window::default(), bevy::window::PrimaryWindow))
            .id();
        let mut source = source_test_frame(0xe4);
        source.battler_rows[0] = Some(crystal_render_api::VisualBattleBattlerRows {
            source_y: Vec2::new(48.0, 64.0),
            bg_cleared: true,
            oam_depth: 3.45,
        });
        app.world_mut().resource_mut::<VisualBattleFrame>().source = Some(source);
        app.update();
        let projection_ticks = |app: &mut App| {
            let world = app.world_mut();
            let mut ticks = [0; 2];
            for (row, projection) in world
                .query::<(&BattleRowCamera, Ref<Projection>)>()
                .iter(world)
            {
                ticks[row.0] = projection.last_changed().get();
            }
            ticks
        };
        let stable = projection_ticks(&mut app);
        app.update();
        assert_eq!(
            projection_ticks(&mut app),
            stable,
            "unchanged actor targets must not force camera/frustum recomputation"
        );
        {
            let world = app.world_mut();
            for mut projection in world
                .query_filtered::<&mut Projection, With<BattleRowCamera>>()
                .iter_mut(world)
            {
                if let Projection::Perspective(p) = &mut *projection {
                    p.far = 0.5;
                }
            }
        }
        app.update();
        let far = app.world().resource::<BattleSceneLayout>().far;
        {
            let world = app.world_mut();
            assert!(world
                .query_filtered::<&Projection, With<BattleRowCamera>>()
                .iter(world)
                .all(|projection| matches!(projection, Projection::Perspective(p) if p.far == far)));
        }
        let restored = projection_ticks(&mut app);
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .resolution
            .set(1200.0, 800.0);
        app.update();
        let resized = projection_ticks(&mut app);
        assert!(
            resized.iter().zip(restored).all(|(new, old)| *new != old),
            "target resize must still invalidate both camera projections"
        );
        app.update();
        assert_eq!(projection_ticks(&mut app), resized);
    }

    #[test]
    fn row_targets_initialize_pbr_clusters_on_the_first_active_frame() {
        let mut app = headless_battle_app();
        app.world_mut()
            .spawn((Window::default(), bevy::window::PrimaryWindow));
        app.update();
        {
            let world = app.world_mut();
            let mut rows = world
                .query_filtered::<(&Camera, Option<&bevy::pbr::Clusters>), With<BattleRowCamera>>();
            assert_eq!(rows.iter(world).count(), 2);
            assert!(
                rows.iter(world)
                    .all(|(camera, clusters)| !camera.is_active && clusters.is_none())
            );
        }
        let mut source = source_test_frame(0xe4);
        source.battler_rows[0] = Some(crystal_render_api::VisualBattleBattlerRows {
            source_y: Vec2::new(48.0, 64.0),
            bg_cleared: true,
            oam_depth: 3.45,
        });
        app.world_mut().resource_mut::<VisualBattleFrame>().source = Some(source);
        app.update();
        let world = app.world_mut();
        assert!(
            world
                .query_filtered::<(&Camera, Option<&bevy::pbr::Clusters>), With<BattleRowCamera>>()
                .iter(world)
                .all(|(camera, clusters)| camera.is_active && clusters.is_some()),
            "a first-use view must be eligible for light uniform preparation in the same frame"
        );
    }

    #[test]
    fn row_targets_are_bounded_and_restore_after_f3_and_interruption() {
        let mut app = headless_battle_app();
        app.world_mut()
            .spawn((Window::default(), bevy::window::PrimaryWindow));
        app.world_mut().spawn(Camera2dBundle::default());
        let mut source = source_test_frame(0xe4);
        source.battler_rows[0] = Some(crystal_render_api::VisualBattleBattlerRows {
            source_y: Vec2::new(48.0, 64.0),
            bg_cleared: true,
            oam_depth: 3.9,
        });
        source.battler_offsets = [Vec2::ZERO; 2];
        source.line_y_offsets = Some([2; 95]);
        app.world_mut().resource_mut::<VisualBattleFrame>().source = Some(source);
        app.update();
        app.update();
        assert!(
            app.world()
                .resource::<BattleViewStatus>()
                .modeled_species
                .iter()
                .any(|species| species == "CYNDAQUIL")
        );
        {
            let world = app.world_mut();
            for (_, camera, fxaa, fog) in world
                .query::<(
                    &BattleRowCamera,
                    &Camera,
                    Option<&Fxaa>,
                    Option<&FogSettings>,
                )>()
                .iter(world)
            {
                assert!(!camera.hdr);
                assert!(
                    matches!(camera.clear_color, ClearColorConfig::Custom(color) if color == Color::NONE)
                );
                assert!(
                    matches!(camera.output_mode, bevy::render::camera::CameraOutputMode::Write {
                    blend_state: Some(bevy::render::render_resource::BlendState::REPLACE),
                    clear_color: ClearColorConfig::Custom(color),
                } if color == Color::NONE)
                );
                assert!(
                    fxaa.is_none() && fog.is_none(),
                    "actor targets must not inherit arena postprocessing"
                );
            }
            let image = battle_row_target_image(UVec2::new(8, 8));
            assert!(
                image
                    .data
                    .chunks_exact(4)
                    .all(|pixel| pixel == [0, 0, 0, 0])
            );
        }
        let counts = (
            app.world().entities().len(),
            app.world().resource::<Assets<Image>>().len(),
            app.world()
                .resource::<Assets<BattleCompositeMaterial>>()
                .len(),
            app.world().resource::<Assets<Mesh>>().len(),
        );
        {
            let world = app.world_mut();
            let overlay = world.query_filtered::<(&Handle<BattleCompositeMaterial>, &Transform, &Visibility), With<BattleRowCompositeSprite>>().single(world);
            assert_eq!(
                overlay.1.translation.z, 3.9,
                "row overlay must inherit the current source OAM depth"
            );
            assert_eq!(*overlay.2, Visibility::Visible);
            assert_eq!(world.resource::<Assets<BattleCompositeMaterial>>().len(), 1);
        }
        let handles = app.world().resource::<BattleScene>().row_targets.clone();
        let source = app.world().resource::<VisualBattleFrame>().source.clone();
        for enabled in [false, true, false, true] {
            app.world_mut().resource_mut::<VoxelViewSettings>().enabled = enabled;
            app.update();
            let world = app.world_mut();
            let mut cameras = world.query::<(&BattleRowCamera, &Camera)>();
            for (_, camera) in cameras.iter(world) {
                assert_eq!(
                    camera.is_active, enabled,
                    "both actors use registered source rows"
                );
            }
            assert_eq!(
                world.resource::<VisualBattleFrame>().source,
                source,
                "F3 must hold source tick and masks"
            );
            assert_eq!(world.resource::<BattleScene>().row_targets, handles);
        }
        app.world_mut().insert_resource(BattleFlashMode::Reduced);
        app.update();
        assert_eq!(app.world().resource::<VisualBattleFrame>().source, source);
        app.world_mut().resource_mut::<VisualBattleFrame>().source = None;
        app.update();
        let world = app.world_mut();
        assert!(
            world
                .query::<(&BattleRowCamera, &Camera)>()
                .iter(world)
                .all(|(_, camera)| !camera.is_active)
        );
        assert!(
            world
                .query_filtered::<&Visibility, With<BattleRowCompositeSprite>>()
                .iter(world)
                .all(|visibility| *visibility == Visibility::Hidden)
        );
        assert!(
            world
                .query_filtered::<&RenderLayers, With<BattleActor>>()
                .iter(world)
                .all(|layers| *layers == RenderLayers::layer(BATTLE_LAYER))
        );
        assert_eq!(
            (
                world.entities().len(),
                world.resource::<Assets<Image>>().len(),
                world.resource::<Assets<BattleCompositeMaterial>>().len(),
                world.resource::<Assets<Mesh>>().len()
            ),
            counts
        );
        assert_eq!(
            world
                .resource::<Assets<BattleCompositeMaterial>>()
                .iter()
                .next()
                .unwrap()
                .1
                .source,
            BattleCompositeUniform::default()
        );
    }

    #[test]
    fn row_target_first_use_and_resize_rebind_stable_handles() {
        let mut app = headless_battle_app();
        app.world_mut()
            .spawn((Window::default(), bevy::window::PrimaryWindow));
        app.world_mut().spawn(Camera2dBundle::default());
        app.update();
        app.update();
        app.world_mut().resource_mut::<VisualBattleFrame>().battlers[1]
            .as_mut()
            .unwrap()
            .species_id = Arc::from("GENGAR");
        let targets = app.world().resource::<BattleScene>().row_targets.clone();
        assert_eq!(
            app.world().resource::<BattleScene>().row_sizes,
            [UVec2::ONE; 2]
        );
        let mut source = source_test_frame(0xe4);
        source.battler_rows[1] = Some(crystal_render_api::VisualBattleBattlerRows {
            source_y: Vec2::new(40.0, 56.0),
            bg_cleared: false,
            oam_depth: 3.45,
        });
        source.battler_offsets = [Vec2::ZERO; 2];
        app.world_mut().resource_mut::<VisualBattleFrame>().source = Some(source);
        for size in [
            UVec2::new(640, 576),
            UVec2::new(940, 610),
            UVec2::new(640, 576),
        ] {
            app.world_mut()
                .resource_mut::<VisualBattleCanvas>()
                .physical_size = size;
            app.update();
            let world = app.world();
            let scene = world.resource::<BattleScene>();
            assert_eq!(scene.row_targets, targets);
            assert_eq!(scene.row_sizes[0], scene.render_size);
            assert_eq!(scene.row_sizes[1], scene.render_size);
            assert_eq!(
                world
                    .resource::<Assets<Image>>()
                    .get(&targets[1])
                    .unwrap()
                    .size(),
                scene.render_size
            );
            assert_eq!(world.resource::<Assets<BattleCompositeMaterial>>().len(), 1);
            let changed = world
                .get_resource_ref::<Assets<BattleCompositeMaterial>>()
                .unwrap()
                .last_changed();
            app.update();
            assert_eq!(
                app.world()
                    .get_resource_ref::<Assets<BattleCompositeMaterial>>()
                    .unwrap()
                    .last_changed(),
                changed,
                "held frames must not recreate material bindings"
            );
        }
    }

    #[test]
    fn row_uniforms_keep_source_bands_unwarped_and_reset() {
        let layout = BattleSceneLayout::default();
        let viewport = Vec2::new(1180.0, 812.0);
        let mut frame = VisualBattleFrame {
            source: Some(source_test_frame(0xe4)),
            ..default()
        };
        frame.source.as_mut().unwrap().battler_rows[0] =
            Some(crystal_render_api::VisualBattleBattlerRows {
                source_y: Vec2::new(48.0, 64.0),
                bg_cleared: true,
                oam_depth: 3.45,
            });
        let full = battle_composite_uniform(&layout, &frame, BattleFlashMode::Full, viewport);
        let reduced = battle_composite_uniform(&layout, &frame, BattleFlashMode::Reduced, viewport);
        assert_eq!(full.battler_rows, reduced.battler_rows);
        assert_eq!(full.battler_rows[0], Vec4::new(48.0, 64.0, 1.0, 1.0));
        assert_eq!(
            full.screen_offset.z, 0.0,
            "row extraction does not invent background movement"
        );
        assert_ne!(
            full.view_to_source,
            BattleCompositeUniform::default().view_to_source,
            "stationary extraction still needs the shared source-plane inverse"
        );
        frame.source.as_mut().unwrap().battler_rows = [None; 2];
        let neutral = battle_composite_uniform(&layout, &frame, BattleFlashMode::Full, viewport);
        assert_eq!(neutral.battler_rows, [Vec4::ZERO; 2]);
        assert_eq!(
            neutral.view_to_source,
            BattleCompositeUniform::default().view_to_source
        );
        frame.source = None;
        assert_eq!(
            battle_composite_uniform(&layout, &frame, BattleFlashMode::Full, viewport),
            BattleCompositeUniform::default()
        );
    }
    #[test]
    fn fallback_card_keeps_native_size_and_projected_bottom_footing() {
        let app = headless_battle_app();
        let frame = app.world().resource::<VisualBattleFrame>();
        let camera_from_world = camera_pose().compute_matrix().inverse();
        let screen = |point: Vec3| {
            let view = camera_from_world.transform_point3(point);
            Vec2::new(view.x / -view.z, view.y / -view.z)
        };
        for side in [VisualBattleSide::Player, VisualBattleSide::Enemy] {
            let mut battler = frame.battlers[0].as_ref().unwrap().clone();
            battler.side = side;
            battler.texture_size = Vec2::new(48.0, 56.0);
            let mut source = source_test_frame(0xe4);
            source.battler_offsets[side.index()] = Vec2::new(6.0, -3.0);
            for source in [None, Some(&source)] {
                let pose = actor_pose(&battler, &[], 0.0, None, source);
                let bottom = pose.transform_point(Vec3::new(0.0, -FALLBACK_CARD_HEIGHT * 0.5, 0.0));
                let expected = side_position(side)
                    + source.map_or(Vec3::ZERO, |source| {
                        source_battler_displacement(source.battler_offsets[side.index()])
                    });
                assert!(bottom.abs_diff_eq(expected, 0.00001));
                assert!(screen(bottom).abs_diff_eq(screen(expected), 0.00001));
                let top = pose.transform_point(Vec3::new(0.0, FALLBACK_CARD_HEIGHT * 0.5, 0.0));
                assert!((top.distance(bottom) - 1.9).abs() < 0.00001);
            }
        }
    }
    #[test]
    fn registered_rows_cover_actual_model_feet_without_changing_artistic_pose() {
        let mut app = headless_battle_app();
        app.world_mut()
            .spawn((Window::default(), bevy::window::PrimaryWindow));
        app.world_mut().spawn(Camera2dBundle::default());
        {
            let mut frame = app.world_mut().resource_mut::<VisualBattleFrame>();
            let player = frame.battlers[0].as_mut().unwrap();
            player.species_id = Arc::from("TOTODILE");
            player.source_rect = Rect::new(16.0, 48.0, 64.0, 96.0);
            player.source_opaque_rect = player.source_rect;
            let enemy = frame.battlers[1].as_mut().unwrap();
            enemy.species_id = Arc::from("GENGAR");
            enemy.source_rect = Rect::new(96.0, 0.0, 152.0, 56.0);
            enemy.source_opaque_rect = enemy.source_rect;
        }
        app.update();
        let original_poses: Vec<_> = {
            let world = app.world_mut();
            world
                .query_filtered::<&Transform, With<BattleActor>>()
                .iter(world)
                .copied()
                .collect()
        };
        let mut source = source_test_frame(0xe4);
        source.battler_offsets = [Vec2::ZERO; 2];
        source.battler_rows[1] = Some(crystal_render_api::VisualBattleBattlerRows {
            source_y: Vec2::new(40.0, 56.0),
            bg_cleared: true,
            oam_depth: 3.45,
        });
        let mut scx = [0; 95];
        scx[47..].fill(-8);
        source.line_x_offsets = Some(scx);
        app.world_mut().resource_mut::<VisualBattleFrame>().source = Some(source);
        app.update();
        let world = app.world_mut();
        let uniform = &world
            .resource::<Assets<BattleCompositeMaterial>>()
            .iter()
            .next()
            .unwrap()
            .1
            .source;
        let view = uniform.actor_view_rects[1];
        let original = uniform.actor_source_rects[1];
        let uv = Vec2::new((view.x + view.z) * 0.5, view.w - (view.w - view.y) * 0.001);
        let registered = original.truncate().truncate()
            + (uv - Vec2::new(view.x, view.y)) / Vec2::new(view.z - view.x, view.w - view.y)
                * Vec2::new(original.z - original.x, original.w - original.y);
        assert!(
            (40.0..56.0).contains(&registered.y),
            "the complete modeled feet belong to the extracted source row"
        );
        let inverse = Mat3::from_cols(
            uniform.view_to_source[0].truncate(),
            uniform.view_to_source[1].truncate(),
            uniform.view_to_source[2].truncate(),
        );
        let raw = inverse * uv.extend(1.0);
        assert!(
            raw.y / raw.z > 56.0,
            "this model exposes the raw-plane crop defect"
        );
        assert_eq!(
            world
                .query_filtered::<&Transform, With<BattleActor>>()
                .iter(world)
                .copied()
                .collect::<Vec<_>>(),
            original_poses,
            "row registration must not reshape or move the neutral sculpture"
        );
        let registration = world.resource::<BattleScene>().actors[1]
            .as_ref()
            .unwrap()
            .projected_registration;
        app.update();
        assert_eq!(
            app.world().resource::<BattleScene>().actors[1]
                .as_ref()
                .unwrap()
                .projected_registration,
            registration
        );
    }
}

/// Opt-in native GPU diagnostics. Headless scene tests cannot establish that
/// the real camera frustum admitted the meshes into its render view.
fn trace_battle_visibility(
    status: Res<BattleViewStatus>,
    cameras: Query<
        (
            &Camera,
            &GlobalTransform,
            &Projection,
            &bevy::render::view::VisibleEntities,
        ),
        With<BattleCamera>,
    >,
    objects: Query<
        (
            Entity,
            &GlobalTransform,
            &Visibility,
            &ViewVisibility,
            Option<&bevy::render::primitives::Aabb>,
        ),
        Or<(With<BattleActor>, With<BattleArena>)>,
    >,
    mut enabled: Local<Option<bool>>,
    mut emitted: Local<bool>,
) {
    let enabled = *enabled.get_or_insert_with(|| {
        std::env::var("CRYSTAL_BATTLE_VISIBILITY_TRACE").is_ok_and(|v| v == "1")
    });
    if !enabled || *emitted || status.active_frames < 60 {
        return;
    }
    *emitted = true;
    for (camera, transform, projection, visible) in &cameras {
        eprintln!(
            "battle_visibility camera active={} target={:?} viewport={:?} global={:?} projection={:?} meshes={:?}",
            camera.is_active,
            camera.physical_target_size(),
            camera.physical_viewport_size(),
            transform,
            projection,
            visible.get::<With<Handle<Mesh>>>()
        );
    }
    for (entity, transform, visibility, view, bounds) in &objects {
        eprintln!(
            "battle_visibility mesh {entity:?} global={transform:?} visibility={visibility:?} in_view={} bounds={bounds:?}",
            view.get()
        );
    }
}

// Ordering bridge for Material2d's image-dependent bind group preparation.
fn battle_composite_images_ready() {}
