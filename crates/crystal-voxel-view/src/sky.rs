//! Camera-centered procedural sky. No textures, lights, or gameplay state.
use crate::{VOXEL_RENDER_LAYER, VoxelViewSettings, VoxelWorldCamera, day_cycle::DayCycle};
use bevy::{
    asset::load_internal_asset,
    light::{NotShadowCaster, NotShadowReceiver},
    mesh::MeshVertexBufferLayoutRef,
    pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin},
    prelude::*,
    render::render_resource::{
        AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};
use crystal_render_api::{VisualWorldClock, VisualWorldFrame, VisualWorldLighting};

const SHADER: Handle<Shader> = Handle::Uuid(
    bevy::asset::uuid::Uuid::from_u128(0x4367_dc38_77ad_4e42_b18d_688e_a65f_ba90),
    std::marker::PhantomData,
);
#[derive(Component)]
struct SkyDome;

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
struct SkyMaterial {
    #[uniform(0)]
    center: Vec4,
    #[uniform(0)]
    zenith: Vec4,
    #[uniform(0)]
    horizon: Vec4,
    #[uniform(0)]
    cloud: Vec4,
    #[uniform(0)]
    sun: Vec4,
    #[uniform(0)]
    parameters: Vec4,
    #[uniform(0)]
    woodland: Vec4,
    #[uniform(0)]
    woodland_sectors: [Vec4; 16],
}
impl Material for SkyMaterial {
    // The dome paints the background without writing depth. It cannot supply
    // scene geometry to SSAO, a depth prepass or a shadow map.
    fn enable_prepass() -> bool { false }
    fn enable_shadows() -> bool { false }

    fn fragment_shader() -> ShaderRef {
        SHADER.into()
    }
    fn specialize(
        _: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _: &MeshVertexBufferLayoutRef,
        _: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // Both sides remain visible, including camera rigs close to map edges.
        descriptor.primitive.cull_mode = None;
        if let Some(depth) = descriptor.depth_stencil.as_mut() {
            depth.depth_write_enabled = Some(false);
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct Palette {
    zenith: Vec3,
    cloud: Vec3,
    stars: f32,
    daylight: f32,
}
impl Palette {
    fn at(second: u32) -> Self {
        let night = Self {
            zenith: Vec3::new(0.009, 0.018, 0.055),
            cloud: Vec3::new(0.13, 0.17, 0.27),
            stars: 1.0,
            daylight: 0.0,
        };
        let dawn = Self {
            zenith: Vec3::new(0.27, 0.48, 0.72),
            cloud: Vec3::new(0.98, 0.83, 0.65),
            stars: 0.0,
            daylight: 1.0,
        };
        let day = Self {
            zenith: Vec3::new(0.17, 0.42, 0.75),
            cloud: Vec3::new(0.97, 0.98, 1.0),
            stars: 0.0,
            daylight: 1.0,
        };
        let dusk = Self {
            zenith: Vec3::new(0.20, 0.24, 0.47),
            cloud: Vec3::new(0.94, 0.62, 0.45),
            stars: 0.10,
            daylight: 0.65,
        };
        let stops = [
            (0.0, night),
            (4.0, night),
            (6.0, dawn),
            (9.0, day),
            (15.0, day),
            (18.0, dusk),
            (20.0, night),
            (24.0, night),
        ];
        let hour = (second % 86_400) as f32 / 3600.0;
        let pair = stops.windows(2).find(|pair| hour < pair[1].0).unwrap();
        let t = (hour - pair[0].0) / (pair[1].0 - pair[0].0);
        let t = t * t * (3.0 - 2.0 * t);
        let (a, b) = (pair[0].1, pair[1].1);
        Self {
            zenith: a.zenith.lerp(b.zenith, t),
            cloud: a.cloud.lerp(b.cloud, t),
            stars: a.stars + (b.stars - a.stars) * t,
            daylight: a.daylight + (b.daylight - a.daylight) * t,
        }
    }
}
fn linear(rgb: Vec3) -> Vec4 {
    Color::srgb(rgb.x, rgb.y, rgb.z).to_linear().to_vec4()
}

/// Use the real grove drawings to select woodland haze. Open water, rooftops
/// and enclosed maps retain their ordinary sky, without a map-name allowlist.
pub(super) fn woodland_strength(frame: &VisualWorldFrame) -> f32 {
    if !frame.active || !outdoors(frame) || frame.tiles.is_empty() {
        return 0.0;
    }
    let mut trees = 0;
    for tile in &frame.tiles {
        let shape = crate::profile::shape_for_source(&tile.source);
        // Camera fog affects the whole scene. Keep every water-bearing frame
        // on its existing atmosphere; directional sky haze can still fill
        // the wooded shore without touching the sea or a pond.
        if shape == crate::profile::CellShape::Water { return 0.0; }
        trees += usize::from(is_tree(shape));
    }
    coverage_strength(trees as f32 / frame.tiles.len() as f32)
}

fn is_tree(shape: crate::profile::CellShape) -> bool {
    matches!(shape,
        crate::profile::CellShape::FacadeBand { solid: crate::profile::SolidKind::Tree, .. }
            | crate::profile::CellShape::RaisedTop { solid: crate::profile::SolidKind::Tree, .. }
    )
}

fn coverage_strength(coverage: f32) -> f32 {
    let t = ((coverage - 0.12) / 0.28).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Sample the farthest source band in 64 world directions. Near water cannot
/// erase a forest behind it; water reaching the horizon vetoes its sector and
/// both neighbors so interpolation cannot leak woodland color onto the sea.
#[cfg(test)]
fn woodland_sectors(frame: &VisualWorldFrame, eye: Vec3) -> [Vec4; 16] {
    woodland_sectors_with_scratch(frame, eye, &mut Vec::new())
}

#[derive(Default)]
struct WoodlandCache {
    key: Option<(std::sync::Arc<str>, u64, IVec2, UVec2, Vec2, bool, Vec2)>,
    sectors: [Vec4; 16],
    samples: Vec<(usize, f32)>,
}
impl WoodlandCache {
    fn update(&mut self, frame: &VisualWorldFrame, eye: Vec3) -> [Vec4; 16] {
        // Source-sector samples are viewport-relative; the live camera may
        // now use stable source-map coordinates.
        let mut eye = eye - Vec3::new(frame.view_center.x, 0., -frame.view_center.y);
        // Distant sky sectors need source-cell precision, not subpixel camera
        // interpolation. Keep the dome itself smooth while avoiding thousands
        // of atan2/sqrt samples on every walking frame.
        let cell = frame.tile_size.max(Vec2::ONE);
        let horizontal = (Vec2::new(eye.x, eye.z) / cell).round() * cell;
        eye.x = horizontal.x;
        eye.z = horizontal.y;
        let key = (frame.map_id.clone(), frame.terrain_revision, frame.grid_origin,
            frame.grid_size, frame.tile_size, frame.active, Vec2::new(eye.x, eye.z));
        if self.key.as_ref() != Some(&key) {
            self.sectors = woodland_sectors_with_scratch(frame, eye, &mut self.samples);
            self.key = Some(key);
        }
        self.sectors
    }
}

fn woodland_sectors_with_scratch(frame: &VisualWorldFrame, eye: Vec3,
    samples: &mut Vec<(usize, f32)>) -> [Vec4; 16] {
    if !frame.active || !outdoors(frame) { return [Vec4::ZERO; 16]; }
    let mut total = [0usize; 64];
    let mut trees = [0usize; 64];
    let mut water = [false; 64];
    let mut far = [0.0_f32; 64];
    let size = frame.grid_size.as_vec2() * frame.tile_size;
    let direction = |tile: &crystal_render_api::VisualTile| {
        let point = (Vec2::new(tile.column as f32, tile.row as f32) + Vec2::splat(0.5))
            * frame.tile_size - size * 0.5;
        let delta = point - Vec2::new(eye.x, eye.z);
        let sector = (delta.y.atan2(delta.x).rem_euclid(std::f32::consts::TAU)
            / std::f32::consts::TAU * 64.) as usize % 64;
        (sector, delta.length())
    };
    samples.clear();
    for tile in &frame.tiles {
        let (sector, distance) = direction(tile);
        far[sector] = far[sector].max(distance);
        samples.push((sector, distance));
    }
    for (tile, &(sector, distance)) in frame.tiles.iter().zip(samples.iter()) {
        if distance < far[sector] * 0.8
            || distance < frame.tile_size.max_element() * 8. { continue; }
        let shape = crate::profile::shape_for_source(&tile.source);
        water[sector] |= shape == crate::profile::CellShape::Water;
        total[sector] += 1;
        trees[sector] += usize::from(is_tree(shape));
    }
    let sectors: [f32; 64] = std::array::from_fn(|i| {
        if water[i] || water[(i+63)%64] || water[(i+1)%64] || total[i] == 0 { 0. }
        else { coverage_strength(trees[i] as f32 / total[i] as f32) }
    });
    std::array::from_fn(|i| Vec4::from_array(std::array::from_fn(|j| sectors[i*4+j])))
}

/// The sky and distance fog must meet at the same shade between trunks.
/// Derive its energy from the live horizon so dusk and night stay dark.
pub(super) fn woodland_color(horizon: Color) -> Color {
    let rgb = horizon.to_linear().to_vec4().truncate();
    let energy = rgb.dot(Vec3::new(0.2126, 0.7152, 0.0722));
    Color::linear_rgb(energy * 0.20, energy * 0.30, energy * 0.26)
}
pub(super) fn environment_colors(second: u32) -> [Vec3; 3] {
    let palette = Palette::at(second);
    let cycle = DayCycle::at_seconds(second);
    let upper = linear(palette.zenith.lerp(palette.cloud, 0.18)).truncate();
    let horizon = cycle.horizon.to_linear().to_vec4().truncate();
    let ground = linear(Vec3::new(0.22, 0.20, 0.16)).truncate() * cycle.ambient_gain;
    [upper, horizon, ground]
}
fn material(second: u32, center: Vec3, elapsed: f32) -> SkyMaterial {
    let palette = Palette::at(second);
    let cycle = DayCycle::at_seconds(second);
    SkyMaterial {
        center: center.extend(0.0),
        zenith: linear(palette.zenith),
        horizon: cycle.horizon.to_linear().to_vec4(),
        cloud: linear(palette.cloud),
        sun: cycle.direction.normalize().extend(0.0),
        parameters: Vec4::new(elapsed * 0.003, palette.stars, palette.daylight, 0.0),
        woodland: woodland_color(cycle.horizon).to_linear().to_vec4(),
        woodland_sectors: [Vec4::ZERO; 16],
    }
}
pub(super) fn register(app: &mut App) {
    load_internal_asset!(app, SHADER, "sky.wesl", Shader::from_wesl);
    app.add_plugins(MaterialPlugin::<SkyMaterial>::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            sync.after(crate::sync_voxel_view)
                .after(crate::sync_voxel_atmosphere),
        );
}
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SkyMaterial>>,
) {
    commands.spawn((
        SkyDome,
        Mesh3d(meshes.add(Sphere::new(3000.0).mesh().uv(64, 32))),
        MeshMaterial3d(materials.add(material(12 * 3600, Vec3::ZERO, 0.0))),
        Transform::default(),
        Visibility::Hidden,
        bevy::camera::visibility::RenderLayers::layer(VOXEL_RENDER_LAYER),
        NotShadowCaster,
        NotShadowReceiver,
    ));
}
pub(super) fn outdoors(frame: &VisualWorldFrame) -> bool {
    !crate::interior_cutaway::is_cave(frame)
        && frame
            .tiles
            .first()
            .is_some_and(|tile| !crate::interior::has_back_wall(&tile.source.tileset_id)
                || crate::interior::open_air_roof(&frame.map_id,&tile.source.tileset_id))
}
fn sync(
    frame: Res<VisualWorldFrame>,
    settings: Res<VoxelViewSettings>,
    clock: Option<Res<VisualWorldClock>>,
    period: Option<Res<VisualWorldLighting>>,
    time: Res<Time>,
    cameras: Query<Ref<Transform>, (With<VoxelWorldCamera>, Without<SkyDome>)>,
    mut domes: Query<
        (
            &mut Transform,
            &mut Visibility,
            &MeshMaterial3d<SkyMaterial>,
        ),
        (With<SkyDome>, Without<VoxelWorldCamera>),
    >,
    mut materials: ResMut<Assets<SkyMaterial>>,
    mut woodland: Local<WoodlandCache>,
) {
    let visible = frame.active
        && settings.enabled
        && outdoors(&frame)
        && !matches!(period.as_deref(), Some(VisualWorldLighting::Dark));
    let second = clock
        .and_then(|clock| clock.seconds_of_day())
        .unwrap_or_else(|| match period.as_deref().copied().unwrap_or_default() {
            VisualWorldLighting::Morning => 6 * 3600,
            VisualWorldLighting::Night | VisualWorldLighting::Dark => 0,
            VisualWorldLighting::Day => 12 * 3600,
        });
    let Some(camera) = cameras.iter().next() else {
        return;
    };
    // Actor movement republishes VisualWorldFrame; it does not change the
    // horizon's source geometry. Camera height/rotation do not change sectors.
    let sectors = if visible { woodland.update(&frame, camera.translation) }
        else { [Vec4::ZERO; 16] };
    for (mut transform, mut visibility, handle) in &mut domes {
        let next = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *visibility != next {
            *visibility = next;
        }
        if !visible {
            continue;
        }
        if transform.translation != camera.translation {
            transform.translation = camera.translation;
        }
        let scale=if crate::summit_vista::enabled(&frame) {
            Vec3::splat(5000.*crate::summit_vista::world_units(&frame)/3000.)
        } else {Vec3::ONE};
        if transform.scale!=scale {transform.scale=scale;}
        if let Some(mut sky) = materials.get_mut(&handle.0) {
            *sky = material(second, camera.translation, time.elapsed_secs_wrapped());
            sky.woodland_sectors = sectors;
            sky.parameters.w = if crate::summit_vista::enabled(&frame) { 1.0 } else { 0.0 };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn grove_frame() -> VisualWorldFrame {
        let tile = crystal_render_api::VisualTile {
            column: 0, row: 0, priority: false, texture: Handle::default(),
            animation_frames: None,
            source: crystal_render_api::VisualTileSource {
                tileset_id: "johto_modern".into(), metatile_id: 5,
                subtile_column: 0, subtile_row: 0, tile_index: 0x1e,
            },
        };
        VisualWorldFrame {
            active: true, map_id: "AzaleaTown".into(),
            viewport_size: Vec2::new(160., 144.), tile_size: Vec2::splat(8.),
            grid_size: UVec2::splat(40),
            tiles: (0..1600).map(|i| {
                let mut tile = tile.clone();
                tile.column = i % 40;
                tile.row = i / 40;
                tile
            }).collect(), ..Default::default()
        }
    }

    #[test]
    fn map_anchoring_preserves_woodland_sector_selection() {
        let frame=grove_frame();
        let mut anchored=frame.clone();let shift=anchored.anchor_to_map();
        let delta=Vec3::new(shift.x,0.,-shift.y);
        let mut before=WoodlandCache::default();let mut after=WoodlandCache::default();
        for eye in [Vec3::ZERO,Vec3::new(-40.,50.,32.)] {
            assert_eq!(before.update(&frame,eye),after.update(&anchored,eye+delta));
        }
    }

    #[test]
    fn woodland_cache_reuses_actor_only_frames_and_rechecks_source_and_camera_changes() {
        let mut frame = grove_frame();
        let mut cache = WoodlandCache::default();
        assert_eq!(cache.update(&frame, Vec3::ZERO), woodland_sectors(&frame, Vec3::ZERO));
        let key = cache.key.clone();
        // An actor-only publication and a vertical camera change keep the same
        // horizontal source panorama and retain the completed sample storage.
        frame.center += Vec2::ONE;
        assert_eq!(cache.update(&frame, Vec3::Y * 50.), woodland_sectors(&frame, Vec3::ZERO));
        assert_eq!(cache.key, key);
        assert_eq!(cache.update(&frame, Vec3::new(1., 0., -1.)), woodland_sectors(&frame, Vec3::ZERO));
        assert_eq!(cache.key, key, "subcell walking must retain the panorama samples");
        for tile in &mut frame.tiles {
            if tile.column >= 30 { tile.source.tile_index = 0x14; tile.source.metatile_id = 0; }
        }
        frame.terrain_revision += 1;
        let coast = cache.update(&frame, Vec3::ZERO);
        assert_eq!(coast, woodland_sectors(&frame, Vec3::ZERO));
        assert_eq!(coast[0][0], 0.);
        let eye = Vec3::new(-40., 0., 32.);
        assert_eq!(cache.update(&frame, eye), woodland_sectors(&frame, eye));
        frame.grid_origin += IVec2::ONE;
        cache.update(&frame, eye);
        assert_eq!(cache.key.as_ref().unwrap().2, frame.grid_origin);
        frame.active = false;
        assert_eq!(cache.update(&frame, eye), [Vec4::ZERO;16]);
    }

    #[test]
    fn woodland_haze_requires_source_trees_and_keeps_open_sky_and_rooms_clear() {
        let mut frame = grove_frame();
        assert_eq!(woodland_strength(&frame), 1.0);
        for tile in &mut frame.tiles[160..] {
            tile.source.metatile_id = 0;
            tile.source.tile_index = 5;
        }
        assert_eq!(woodland_strength(&frame), 0.0, "a few garden trees are not forest");
        for tile in &mut frame.tiles {
            tile.source.tileset_id = "house".into();
        }
        assert_eq!(woodland_strength(&frame), 0.0);
        frame = grove_frame();
        frame.active = false;
        assert_eq!(woodland_strength(&frame), 0.0);
    }

    #[test]
    fn wooded_coast_preserves_ocean_sky_and_disables_automatic_camera_fog() {
        let mut frame = grove_frame();
        assert!(woodland_sectors(&frame, Vec3::ZERO).iter().all(|v| *v == Vec4::ONE));
        for tile in &mut frame.tiles {
            if tile.column >= 30 {
                tile.source.metatile_id = 0;
                tile.source.tile_index = 0x14;
            }
        }
        assert_eq!(woodland_strength(&frame), 0.,
            "nearby forest must never enable scene-wide fog over water");
        for eye in [Vec3::ZERO, Vec3::new(-40., 0., 32.)] {
            let sectors = woodland_sectors(&frame, eye);
            assert_eq!(sectors[0][0], 0., "east-facing sea retains its exact existing sky");
            assert_eq!(sectors[15][3], 0., "wraparound interpolation cannot tint the sea");
            assert!(sectors[8][0] > 0.9, "the western forest still fills its trunk gaps");
        }
        for tile in &mut frame.tiles {
            tile.source.metatile_id = 0;
            tile.source.tile_index = 0x14;
        }
        assert_eq!(woodland_sectors(&frame, Vec3::ZERO), [Vec4::ZERO;16]);
    }

    #[test]
    fn nearby_pond_does_not_erase_the_distant_forest_sky() {
        let mut frame = grove_frame();
        for tile in &mut frame.tiles {
            if (26..=28).contains(&tile.column) && (18..=21).contains(&tile.row) {
                tile.source.metatile_id = 0;
                tile.source.tile_index = 0x14;
            }
        }
        assert_eq!(woodland_strength(&frame), 0., "pond water still excludes camera fog");
        assert!(woodland_sectors(&frame, Vec3::ZERO)[0][0] > 0.9,
            "forest beyond a near pond still owns its horizon");
    }

    #[test]
    fn forest_fog_matches_the_sky_tracks_night_and_clears_on_room_entry() {
        use bevy::{anti_alias::fxaa::Fxaa, pbr::{DistanceFog, FogFalloff}};
        let mut app = App::new();
        app.insert_resource(grove_frame())
            .insert_resource(VoxelViewSettings { enabled: true, ..Default::default() })
            .insert_resource(VisualWorldClock { minute_of_day: Some(720), second_of_minute: 0 })
            .init_resource::<crate::live_profiles::LiveProfiles>()
            .add_systems(Update, crate::sync_voxel_atmosphere);
        let entity = app.world_mut().spawn((VoxelWorldCamera, Camera::default(),
            DistanceFog::default(), Fxaa::default())).id();
        app.update();
        let day = app.world().get::<DistanceFog>(entity).unwrap().color;
        assert!(day.to_linear().to_vec4().distance(
            material(12 * 3600, Vec3::ZERO, 0.).woodland) < 0.000001);
        let pose = app.world().resource::<VoxelViewSettings>().camera.pose(Vec2::new(160.,144.));
        let target_distance = pose.eye.distance(pose.target);
        assert!(matches!(app.world().get::<DistanceFog>(entity).unwrap().falloff,
            FogFalloff::Linear { start, end }
            if start == target_distance + 64. && end == target_distance + 176.));
        app.world_mut().resource_mut::<VisualWorldClock>().minute_of_day = Some(0);
        app.update();
        let night = app.world().get::<DistanceFog>(entity).unwrap().color;
        assert!(night.to_linear().to_vec4().truncate().length() < day.to_linear().to_vec4().truncate().length());
        {
            let mut frame = app.world_mut().resource_mut::<VisualWorldFrame>();
            frame.tiles[0].source.metatile_id = 0;
            frame.tiles[0].source.tile_index = 0x14;
            frame.terrain_revision += 1;
        }
        app.update();
        assert_eq!(app.world().get::<DistanceFog>(entity).unwrap().color, Color::NONE);
        assert!(matches!(app.world().get::<Camera>(entity).unwrap().clear_color,
            bevy::camera::ClearColorConfig::Custom(color) if color == DayCycle::at(0).horizon));
        {
            let mut frame = app.world_mut().resource_mut::<VisualWorldFrame>();
            for tile in &mut frame.tiles { tile.source.tileset_id = "house".into(); }
            frame.terrain_revision += 1;
        }
        app.update();
        assert_eq!(app.world().get::<DistanceFog>(entity).unwrap().color, Color::NONE);
    }

    #[test]
    fn route_haze_is_faint_and_starts_beyond_nearby_landmarks() {
        use bevy::{anti_alias::fxaa::Fxaa, pbr::{DistanceFog, FogFalloff}};
        for map in ["Route26", "Route29"] {
            let mut frame = grove_frame();
            frame.map_id = map.into();
            let mut app = App::new();
            app.insert_resource(frame)
                .insert_resource(VoxelViewSettings { enabled: true, ..Default::default() })
                .init_resource::<crate::live_profiles::LiveProfiles>()
                .add_systems(Update, crate::sync_voxel_atmosphere);
            let camera = app.world_mut().spawn((DistanceFog::default(), Fxaa::default(), VoxelWorldCamera)).id();
            app.update();
            let fog = app.world().get::<DistanceFog>(camera).unwrap();
            assert!(fog.color.alpha() <= 0.12);
            let pose = app.world().resource::<VoxelViewSettings>().camera.pose(Vec2::new(160.,144.));
            let distance = pose.eye.distance(pose.target);
            assert!(matches!(fog.falloff, FogFalloff::Linear { start, end }
                if start == distance + 128. && end == distance + 320.));
        }
    }

    #[test]
    fn skybox_is_continuous_and_stars_fade_before_daylight() {
        assert_eq!(Palette::at(0).stars, 1.0);
        assert_eq!(Palette::at(6 * 3600).stars, 0.0);
        assert!(Palette::at(19 * 3600).stars > Palette::at(18 * 3600).stars);
        for second in 1..=86_400 {
            let (a, b) = (Palette::at(second - 1), Palette::at(second));
            assert!(a.zenith.distance(b.zenith) < 0.001);
            assert!((a.stars - b.stars).abs() < 0.001);
        }
    }
    #[test]
    fn skybox_follows_camera_and_hides_in_caves_and_interiors() {
        let mut app = App::new();
        app.init_resource::<VisualWorldFrame>()
            .init_resource::<VisualWorldClock>()
            .init_resource::<Time>()
            .init_resource::<Assets<SkyMaterial>>()
            .insert_resource(VoxelViewSettings {
                enabled: true,
                ..Default::default()
            })
            .add_systems(Update, sync);
        let handle = app
            .world_mut()
            .resource_mut::<Assets<SkyMaterial>>()
            .add(material(0, Vec3::ZERO, 0.0));
        let camera = app
            .world_mut()
            .spawn((VoxelWorldCamera, Transform::from_xyz(20.0, 30.0, 40.0)))
            .id();
        let dome = app
            .world_mut()
            .spawn((
                SkyDome,
                Transform::default(),
                Visibility::Hidden,
                MeshMaterial3d(handle.clone()),
            ))
            .id();
        {
            let mut frame = app.world_mut().resource_mut::<VisualWorldFrame>();
            frame.active = true;
            frame.tiles = vec![crystal_render_api::VisualTile {
                column: 0,
                row: 0,
                priority: false,
                texture: Handle::default(),
                animation_frames: None,
                source: crystal_render_api::VisualTileSource {
                    tileset_id: "johto".into(),
                    metatile_id: 0,
                    subtile_column: 0,
                    subtile_row: 0,
                    tile_index: 0,
                },
            }];
        }
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(dome).unwrap(),
            Visibility::Visible
        );
        assert_eq!(
            app.world().get::<Transform>(dome).unwrap().translation,
            app.world().get::<Transform>(camera).unwrap().translation
        );
        {
            let mut frame = app.world_mut().resource_mut::<VisualWorldFrame>();
            frame.tiles[0].source.tileset_id = "cave".into();
        }
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(dome).unwrap(),
            Visibility::Hidden
        );
        {
            let mut frame = app.world_mut().resource_mut::<VisualWorldFrame>();
            frame.tiles[0].source.tileset_id = "house".into();
        }
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(dome).unwrap(),
            Visibility::Hidden
        );
        for (map,atlas) in [("CeladonMansionRoof","mansion"),
            ("GoldenrodDeptStoreRoof","mart"),("TinTowerRoof","tower")] {
            {
                let mut frame=app.world_mut().resource_mut::<VisualWorldFrame>();
                frame.map_id=map.into();frame.tiles[0].source.tileset_id=atlas.into();
            }
            app.update();
            assert_eq!(*app.world().get::<Visibility>(dome).unwrap(),Visibility::Visible,"{map}");
        }
        {
            let mut frame=app.world_mut().resource_mut::<VisualWorldFrame>();
            frame.map_id="CeladonMansionRoofHouse".into();frame.tiles[0].source.tileset_id="mansion".into();
        }
        app.update();
        assert_eq!(*app.world().get::<Visibility>(dome).unwrap(),Visibility::Hidden);
    }
}
