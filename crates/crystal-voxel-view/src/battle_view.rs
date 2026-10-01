//! A render-only arena. Presentation cues come from the production shell's
//! retained visible scene; this module has no runtime or battle-engine imports.
use crate::{VoxelViewSettings, mesh::SurfaceMeshData};
use bevy::{
    asset::load_internal_asset,
    core_pipeline::{fxaa::Fxaa, tonemapping::Tonemapping},
    pbr::{CascadeShadowConfigBuilder, FogFalloff, FogSettings},
    prelude::*,
    reflect::TypePath,
    render::{
        camera::{CameraUpdateSystem, ClearColorConfig, RenderTarget},
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
// The arena image is drawn behind the native HUD in its existing pass.
// Layer 30 parks classic sprites, and 31 is the modeled overworld.
const BATTLE_COMPOSITE_LAYER: usize = 0;
const MODEL_SCALE: f32 = 2.65;
const FALLBACK_CARD_HEIGHT: f32 = 1.9;
const SOURCE_PIXEL_WORLD: f32 = 0.045;
const BATTLE_CAMERA_FOV: f32 = 0.58;
const PARTICLES: usize = 32;

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
        app.init_resource::<Assets<Shader>>();
        load_internal_asset!(
            app,
            BATTLE_COMPOSITE_SHADER,
            "battle_composite.wgsl",
            Shader::from_wgsl
        );
        app.init_resource::<VisualBattleFrame>()
            .init_resource::<BattleFlashMode>()
            .init_resource::<VisualBattleCanvas>()
            .init_resource::<BattleViewStatus>()
            .init_resource::<BattleScene>()
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
                    .before(CameraUpdateSystem),
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
                    sync_battle_scene,
                    sync_source_objects,
                    sync_modeled_source_effects,
                )
                    .chain()
                    .after(crate::toggle_voxel_view)
                    .in_set(WorldRenderSet::RenderSync),
            );
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
}
#[derive(Component)]
struct BattleCamera;
#[derive(Component)]
struct BattleCompositeSprite;

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
}
impl Default for BattleCompositeUniform {
    fn default() -> Self {
        Self {
            rows: [Vec4::ZERO; 96],
            background: Vec4::ONE,
            screen_offset: Vec4::ZERO,
            source_to_view: [Vec4::X, Vec4::Y, Vec4::Z],
            view_to_source: [Vec4::X, Vec4::Y, Vec4::Z],
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
}
impl Material2d for BattleCompositeMaterial {
    fn fragment_shader() -> ShaderRef {
        BATTLE_COMPOSITE_SHADER.into()
    }
}

fn battle_composite_uniform(
    frame: &VisualBattleFrame,
    mode: BattleFlashMode,
    viewport: Vec2,
) -> BattleCompositeUniform {
    let mut out = BattleCompositeUniform::default();
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
    // Presentation offsets are Y up, while the LCD sampler is Y down.
    let scrolling = source.screen_offset != Vec2::ZERO
        || out.rows.iter().any(|row| row.x != 0.0 || row.y != 0.0);
    out.screen_offset = Vec4::new(
        source.screen_offset.x,
        -source.screen_offset.y,
        if scrolling { 1.0 } else { 0.0 },
        0.0,
    );
    if scrolling {
        let projection = source_plane_projection(viewport);
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
    modeled: bool,
}
struct ActorInstance {
    entity: Entity,
    key: ActorKey,
    material: Handle<StandardMaterial>,
    mesh: Option<Handle<Mesh>>,
    palette_key: (u8, BattleFlashMode),
    neutral_colors: Option<Vec<[f32; 4]>>,
    base_pose: Transform,
    source_rect: Rect,
    source_opaque_rect: Rect,
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
    {
        let image = images.add(battle_target_image(UVec2::ONE));
        scene.render_size = UVec2::ONE;
        scene.render_target = Some(image.clone());
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(Rectangle::new(1.0, 1.0)).into(),
                material: composite_materials.add(BattleCompositeMaterial {
                    source: BattleCompositeUniform::default(),
                    texture: image,
                }),
                visibility: Visibility::Hidden,
                ..default()
            },
            RenderLayers::layer(BATTLE_COMPOSITE_LAYER),
            BattleCompositeSprite,
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
        RenderLayers::layer(BATTLE_LAYER),
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

fn sync_battle_target(
    canvas: Res<VisualBattleCanvas>,
    mut scene: ResMut<BattleScene>,
    mut status: ResMut<BattleViewStatus>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<BattleCompositeMaterial>>,
    composites: Query<&Handle<BattleCompositeMaterial>, With<BattleCompositeSprite>>,
    mut battle_cameras: Query<(&mut Camera, &mut Projection), With<BattleCamera>>,
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
    frame: Res<VisualBattleFrame>,
    mode: Res<BattleFlashMode>,
    mut materials: ResMut<Assets<BattleCompositeMaterial>>,
    status: Res<BattleViewStatus>,
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
            Without<Parent>,
        ),
    >,
    mut sprites: Query<
        (
            &Handle<BattleCompositeMaterial>,
            &mut Transform,
            &mut Visibility,
        ),
        With<BattleCompositeSprite>,
    >,
) {
    let camera = cameras.iter().find(|(camera, _, _, layers)| {
        camera.is_active
            && matches!(camera.target, RenderTarget::Window(_))
            && layers.is_none_or(|layers| {
                layers.intersects(&RenderLayers::layer(BATTLE_COMPOSITE_LAYER))
            })
    });
    for (material, mut transform, mut visibility) in &mut sprites {
        let Some((_, projection, camera, _)) =
            camera.filter(|_| status.active && status.output_size.min_element() > 0)
        else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        let (_, pose) = battle_composite_pose(projection, camera);
        let source = battle_composite_uniform(&frame, *mode, status.output_size.as_vec2());
        if materials
            .get(material)
            .is_some_and(|material| material.source != source)
        {
            materials.get_mut(material).unwrap().source = source;
        }
        transform.set_if_neq(pose);
        visibility.set_if_neq(Visibility::Visible);
    }
}

#[allow(clippy::too_many_arguments)]
fn sync_battle_scene(
    mut commands: Commands,
    frame: Res<VisualBattleFrame>,
    settings: Res<VoxelViewSettings>,
    flash_mode: Res<BattleFlashMode>,
    time: Res<Time>,
    mut status: ResMut<BattleViewStatus>,
    mut scene: ResMut<BattleScene>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cameras: Query<
        (&mut Camera, &mut Transform, &mut FogSettings),
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
        &mut Visibility,
        (
            With<BattleArena>,
            Without<BattleLight>,
            Without<BattleActor>,
            Without<BattleParticle>,
            Without<BattleBall>,
        ),
    >,
    mut actors: Query<
        (&mut Transform, &mut Visibility),
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
    #[cfg(feature = "operation-trace")]
    let _span = bevy::log::info_span!("crystal_battle_render_sync").entered();
    let valid = frame.validate();
    let active = settings.enabled && frame.active && !frame.use_source_scene && valid.is_ok();
    status.active = active;
    status.active_frames = if active {
        status.active_frames.saturating_add(1)
    } else {
        0
    };
    status.last_error = valid.err().map(str::to_owned);
    for (mut camera, mut transform, mut fog) in &mut cameras {
        if camera.is_active != active {
            camera.is_active = active;
        }
        if active {
            *transform = camera_pose();
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
    for mut visibility in &mut arenas {
        visibility.set_if_neq(if active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
    }
    if !active {
        for (_, mut visibility) in &mut actors {
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
        status.source_art_species = if frame.active && frame.use_source_scene {
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
                commands.entity(instance.entity).despawn_recursive();
                materials.remove(instance.material.id());
                if let Some(mesh) = instance.mesh {
                    meshes.remove(mesh.id());
                }
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
        let modeled_data = if battler.allow_species_model {
            scene
                .species_meshes
                .get(&battler.species_id)
                .cloned()
                .flatten()
        } else {
            None
        };
        let modeled = modeled_data.is_some();
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
                commands.entity(instance.entity).despawn_recursive();
                materials.remove(instance.material.id());
                if let Some(mesh) = instance.mesh {
                    meshes.remove(mesh.id());
                }
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
            let base_pose = actor_pose(battler, &frame.cues, scene.elapsed, modeled_bounds, None);
            // Deferred entities cannot be reached by the actor query below
            // until the next update. Present this source offset on spawn too.
            let mut transform = base_pose;
            if let Some(source) = &frame.source {
                transform.translation += source_battler_displacement(source.battler_offsets[index]);
            }
            let neutral_colors = modeled_data
                .as_ref()
                .filter(|_| scene.vertex_lighting)
                .map(|data| vertex_lit_colors(data, transform.rotation));
            let mesh = modeled_data.as_ref().map(|data| {
                let mut mesh = data.as_ref().clone().into_mesh();
                if let Some(colors) = &neutral_colors {
                    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors.clone());
                }
                mesh.asset_usage = bevy::render::render_asset::RenderAssetUsages::MAIN_WORLD
                    | bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD;
                meshes.add(mesh)
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
                    RenderLayers::layer(BATTLE_LAYER),
                    BattleActor,
                ))
                .id();
            scene.actors[index] = Some(ActorInstance {
                entity,
                key,
                material,
                mesh,
                palette_key: (0xe4, *flash_mode),
                neutral_colors,
                base_pose,
                source_rect: battler.source_rect,
                source_opaque_rect: battler.source_opaque_rect,
            });
        }
        let vertex_lighting = scene.vertex_lighting;
        if let Some(instance) = &mut scene.actors[index] {
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
            if instance.palette_key != (bgp, *flash_mode) {
                if let (Some(data), Some(mesh)) = (modeled_data.as_ref(), instance.mesh.as_ref()) {
                    if let Some(mesh) = meshes.get_mut(mesh) {
                        let colors = data
                            .colors
                            .iter()
                            .enumerate()
                            .map(|(index, color)| {
                                let neutral = instance
                                    .neutral_colors
                                    .as_ref()
                                    .map_or(*color, |colors| colors[index]);
                                frame.source.as_ref().map_or(neutral, |source| {
                                    let mapped = source_model_color(
                                        *color,
                                        bgp,
                                        &source.battler_palettes[battler.side.index()],
                                        BattleFlashMode::Full,
                                    );
                                    source_lit_color(neutral, mapped, bgp, *flash_mode)
                                })
                            })
                            .collect::<Vec<_>>();
                        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
                    }
                }
                instance.palette_key = (bgp, *flash_mode);
            }
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
                instance.base_pose = actor_pose(battler, &[], 0.0, modeled_bounds, None);
                instance.source_rect = battler.source_rect;
                instance.source_opaque_rect = battler.source_opaque_rect;
            }
            if let Ok((mut transform, mut visibility)) = actors.get_mut(instance.entity) {
                let mut pose = instance.base_pose;
                if let Some(source) = &frame.source {
                    pose.translation += source_battler_displacement(source.battler_offsets[index]);
                }
                transform.set_if_neq(pose);
                visibility.set_if_neq(if battler.visible {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                });
            }
        }
    }
    for (shadow, mut transform, mut visibility) in &mut contact_shadows {
        let Some(battler) = frame.battlers[shadow.0.index()]
            .as_ref()
            .filter(|b| b.visible)
        else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        let mut position = side_position(battler.side);
        if let Some(source) = &frame.source {
            position += source_battler_displacement(source.battler_offsets[battler.side.index()]);
        }
        transform.translation = Vec3::new(position.x, 0.086, position.z);
        transform.scale = Vec3::new(0.38, 1.0, 0.23) * MODEL_SCALE;
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

fn source_environment_palette(
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

fn source_object_pose(center: Vec2, size: Vec2) -> Transform {
    let across = (center.x - 48.0) / 84.0;
    let baseline_y = 88.0 - across * 40.0;
    let position = side_position(VisualBattleSide::Player)
        .lerp(side_position(VisualBattleSide::Enemy), across)
        + Vec3::Y * (1.0 + (baseline_y - center.y) * SOURCE_PIXEL_WORLD);
    Transform::from_translation(position)
        .with_rotation(camera_pose().rotation)
        .with_scale((size * SOURCE_PIXEL_WORLD).extend(1.0))
}

/// Homography from original LCD pixels to normalized viewport coordinates.
/// Both OAM placement and BG scanline sampling use this same attack plane;
/// source SCX/SCY values are never interpreted as window pixels.
fn source_plane_projection(viewport: Vec2) -> Mat3 {
    let viewport = viewport.max(Vec2::ONE);
    let camera = camera_pose().compute_matrix().inverse();
    let origin = source_object_pose(Vec2::ZERO, Vec2::ZERO).translation;
    let dx = source_object_pose(Vec2::X, Vec2::ZERO).translation - origin;
    let dy = source_object_pose(Vec2::Y, Vec2::ZERO).translation - origin;
    let tangent = (BATTLE_CAMERA_FOV * 0.5).tan();
    let aspect = viewport.x / viewport.y;
    let homogeneous = |view: Vec3| {
        Vec3::new(
            view.x / (2.0 * tangent * aspect) - view.z * 0.5,
            -view.y / (2.0 * tangent) - view.z * 0.5,
            -view.z,
        )
    };
    Mat3::from_cols(
        homogeneous(camera.transform_vector3(dx)),
        homogeneous(camera.transform_vector3(dy)),
        homogeneous(camera.transform_point3(origin)),
    )
}

fn project_source_point(projection: Mat3, point: Vec2) -> Vec2 {
    let projected = projection * point.extend(1.0);
    projected.truncate() / projected.z
}

/// Project the original source OAM plane into the immersive viewport. The
/// caller draws these exact source textures in the native HUD pass, after BG
/// row scrolling. Returned coordinates are pixels from the viewport's top left.
pub fn battle_source_overlay_rect(center: Vec2, size: Vec2, viewport: Vec2) -> Option<Rect> {
    if !viewport.is_finite() || viewport.min_element() <= 0.0 {
        return None;
    }
    let pose = source_object_pose(center, size);
    let camera = camera_pose();
    let view = camera
        .compute_matrix()
        .inverse()
        .transform_point3(pose.translation);
    let depth = -view.z;
    if depth <= 0.1 {
        return None;
    }
    let world_height = 2.0 * depth * (BATTLE_CAMERA_FOV * 0.5).tan();
    let pixels_per_world = viewport.y / world_height;
    let projected_center =
        project_source_point(source_plane_projection(viewport), center) * viewport;
    let half = pose.scale.truncate() * pixels_per_world * 0.5;
    Some(Rect::from_corners(
        projected_center - half,
        projected_center + half,
    ))
}

fn source_battler_displacement(offset: Vec2) -> Vec3 {
    let forward = (side_position(VisualBattleSide::Enemy)
        - side_position(VisualBattleSide::Player))
    .normalize();
    forward * offset.x * SOURCE_PIXEL_WORLD + Vec3::Y * offset.y * SOURCE_PIXEL_WORLD
}

fn sync_source_objects(
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
        *transform = source_object_pose(object.center, object.size);
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
    frame: &VisualBattleFrame,
    slot: usize,
) -> Option<(usize, Transform, [f32; 4])> {
    let source = frame.source.as_ref()?;
    let cue = frame
        .cues
        .iter()
        .find(|cue| cue.kind == VisualBattleCueKind::Move)?;
    let direction = (side_position(cue.side.opposite()) - side_position(cue.side)).normalize();
    let ring_rotation = Quat::from_rotation_arc(Vec3::Y, direction);
    let object = source.objects.iter().find(|object| object.slot == slot)?;
    let center = source_object_pose(object.center, object.size).translation;
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
            let length = object.size.x
                * side_position(VisualBattleSide::Player)
                    .distance(side_position(VisualBattleSide::Enemy))
                / 84.0;
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
    if model.is_some() {
        // Preserve authored proportions and the original open three-quarter
        // staging. Source sprite footprints do not squash species geometry.
        let direction = (side_position(battler.side.opposite()) - origin).normalize();
        let camera_direction = (camera_pose().translation - origin) * Vec3::new(1.0, 0.0, 1.0);
        let facing = direction
            .lerp(camera_direction.normalize(), 0.40)
            .normalize();
        transform.rotation = Quat::from_rotation_y(facing.x.atan2(facing.z));
        transform.scale = Vec3::splat(MODEL_SCALE);
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
                    Vec2::new(48.0 + 84.0 * fraction, 88.0 - 40.0 * fraction);
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
                let expected = side_position(VisualBattleSide::Player)
                    .lerp(side_position(VisualBattleSide::Enemy), fraction)
                    + Vec3::Y;
                assert!(position.distance(expected) < 0.0001);
                if let Some(last) = last {
                    let maximum_step = side_position(VisualBattleSide::Player)
                        .distance(side_position(VisualBattleSide::Enemy))
                        / f32::from(hz);
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

    fn headless_battle_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<Image>>()
            .init_resource::<Assets<StandardMaterial>>()
            .insert_resource(VoxelViewSettings {
                enabled: true,
                ..Default::default()
            })
            .add_plugins(BattleViewPlugin);
        let battler = |side, species: &str| VisualBattleBattler {
            side,
            species_id: Arc::from(species),
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

    fn source_test_frame(bgp: u8) -> VisualBattleSourceFrame {
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
        assert!(modeled_source_effect_pose(&frame, 0).is_none());
        frame.source.as_mut().unwrap().objects[0].object_id = Arc::from("BATTLE_ANIM_OBJ_WAVE");
        let ring = modeled_source_effect_pose(&frame, 0).unwrap();
        assert_eq!(ring.0, 0);
        assert!(ring.1.translation.is_finite());
        assert!(modeled_source_effect_pose(&frame, 1).is_none());
        assert!(modeled_source_effect_pose(&frame, 10).is_none());
        frame.source.as_mut().unwrap().line_x_offsets = Some([5; 0x5f]);
        assert!(modeled_source_effect_pose(&frame, 10).is_none());
        frame.cues[0].move_id = Arc::from("HYPER_BEAM");
        assert!(modeled_source_effect_pose(&frame, 0).is_none());
        assert!(modeled_source_effect_pose(&frame, 10).is_none());
        frame.source.as_mut().unwrap().objects[0].object_id = Arc::from("BATTLE_ANIM_OBJ_BEAM");
        let beam = modeled_source_effect_pose(&frame, 0).unwrap();
        assert_eq!(beam.0, 1);
        assert_eq!(beam.1.translation, ring.1.translation);
        frame.source.as_mut().unwrap().objects.clear();
        assert!(modeled_source_effect_pose(&frame, 0).is_none());
        frame.source = None;
        assert!((0..13).all(|slot| modeled_source_effect_pose(&frame, slot).is_none()));
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
        let mut frame = VisualBattleFrame {
            source: Some(source_test_frame(0xe4)),
            ..Default::default()
        };
        let source = frame.source.as_mut().unwrap();
        source.line_x_offsets = Some([5; 95]);
        source.line_y_offsets = Some([-2; 95]);
        source.screen_offset = Vec2::new(3.0, -4.0);
        let normal =
            battle_composite_uniform(&frame, BattleFlashMode::Full, Vec2::new(1180.0, 812.0));
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
        let full =
            battle_composite_uniform(&frame, BattleFlashMode::Full, Vec2::new(1180.0, 812.0));
        let reduced =
            battle_composite_uniform(&frame, BattleFlashMode::Reduced, Vec2::new(1180.0, 812.0));
        assert_eq!(full.rows, normal.rows);
        assert_eq!(reduced.rows, normal.rows);
        assert_eq!(full.background, Vec4::new(0.0, 0.0, 0.0, 1.0));
        assert!(reduced.background.x > 0.0 && reduced.background.x < 1.0);
        assert_eq!(
            battle_composite_uniform(
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
        for viewport in [
            Vec2::new(1180.0, 812.0),
            Vec2::new(1600.0, 900.0),
            Vec2::new(812.0, 1180.0),
        ] {
            let projection = source_plane_projection(viewport);
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
                let oam = battle_source_overlay_rect(center, Vec2::splat(16.0), viewport).unwrap();
                assert!((oam.center() / viewport - uv).abs().max_element() < 0.00001);
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
            let neutral = battle_composite_uniform(&frame, BattleFlashMode::Full, viewport);
            assert_eq!(
                neutral.screen_offset.z, 0.0,
                "zero register offsets bypass the sampler"
            );
            for value in [-3, 0, 3] {
                frame.source.as_mut().unwrap().line_y_offsets = Some([value; 95]);
                let full = battle_composite_uniform(&frame, BattleFlashMode::Full, viewport);
                let reduced = battle_composite_uniform(&frame, BattleFlashMode::Reduced, viewport);
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
                battle_composite_uniform(&frame, BattleFlashMode::Full, viewport),
                neutral
            );
        }
    }

    #[test]
    fn source_objects_map_continuously_between_immersive_battlers() {
        let start = Vec2::new(48.0, 88.0);
        let end = Vec2::new(132.0, 48.0);
        for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let size = Vec2::new(16.0, 24.0);
            let pose = source_object_pose(start.lerp(end, t), size);
            assert!(
                pose.translation.distance(
                    side_position(VisualBattleSide::Player)
                        .lerp(side_position(VisualBattleSide::Enemy), t)
                        + Vec3::Y
                ) < 0.0001
            );
            assert_eq!(pose.scale.truncate(), size * SOURCE_PIXEL_WORLD);
            assert_eq!(pose.rotation, camera_pose().rotation);
            let rect =
                battle_source_overlay_rect(start.lerp(end, t), size, Vec2::new(1180.0, 812.0))
                    .unwrap();
            assert!(rect.min.is_finite() && rect.max.is_finite());
            assert!(rect.size().min_element() > 0.0);
        }
        assert!(battle_source_overlay_rect(start, Vec2::ONE, Vec2::ZERO).is_none());
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
        for side in [VisualBattleSide::Player, VisualBattleSide::Enemy] {
            battler.side = side;
            let bounds = Some(BattleModelBounds::from_surface(&data));
            let neutral = actor_pose(&battler, &[], 0.0, bounds, None);
            assert_eq!(neutral.translation, side_position(side));
            assert_eq!(neutral.scale, Vec3::splat(MODEL_SCALE));
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
        assert_eq!(
            world.get::<Transform>(actor).unwrap().scale,
            Vec3::splat(MODEL_SCALE)
        );
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
