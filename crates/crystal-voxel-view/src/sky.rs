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

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug, PartialEq)]
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

/// Distance fog on actual woodland geometry retains the live horizon energy.
/// It does not paint vegetation or opaque sectors onto the sky.
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
    fn sky_palette_does_not_depend_on_forest_coverage_water_or_viewport_origin() {
        let mut app=App::new();
        app.insert_resource(grove_frame())
            .insert_resource(VoxelViewSettings {enabled:true,..Default::default()})
            .insert_resource(VisualWorldClock {minute_of_day:Some(720),second_of_minute:0})
            .init_resource::<Time>()
            .init_resource::<Assets<SkyMaterial>>()
            .add_systems(Update,sync);
        let eye=Vec3::new(20.,30.,40.);
        app.world_mut().spawn((VoxelWorldCamera,Transform::from_translation(eye)));
        let handle=app.world_mut().resource_mut::<Assets<SkyMaterial>>()
            .add(material(0,Vec3::ZERO,0.));
        let dome=app.world_mut().spawn((SkyDome,Transform::default(),Visibility::Hidden,
            MeshMaterial3d(handle.clone()))).id();
        let reference=material(12*3600,eye,0.);
        for scene in 0..4 {
            {
                let mut frame=app.world_mut().resource_mut::<VisualWorldFrame>();
                frame.grid_origin+=IVec2::new(4,-4);
                frame.terrain_revision+=1;
                for tile in &mut frame.tiles {
                    if (scene==1 && tile.column>=30) || scene==2 {
                        tile.source.metatile_id=0;
                        tile.source.tile_index=0x14;
                    } else if scene==3 {
                        tile.source.metatile_id=0;
                        tile.source.tile_index=0x01;
                    }
                }
            }
            app.update();
            assert_eq!(*app.world().get::<Visibility>(dome).unwrap(),Visibility::Visible);
            let actual=app.world().resource::<Assets<SkyMaterial>>().get(&handle).unwrap();
            assert_eq!(actual,&reference,
                "trees must form the forest silhouette; the sky cannot synthesize dark backdrop sectors");
        }
    }

    #[test]
    fn woodland_distance_fog_requires_source_trees_and_excludes_rooms() {
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
    fn forest_distance_fog_tracks_night_and_clears_on_room_entry() {
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
            woodland_color(DayCycle::at_seconds(12 * 3600).horizon).to_linear().to_vec4()) < 0.000001);
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
