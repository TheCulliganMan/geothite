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
}
impl Material for SkyMaterial {
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
fn outdoors(frame: &VisualWorldFrame) -> bool {
    !crate::interior_cutaway::is_cave(frame)
        && frame
            .tiles
            .first()
            .is_some_and(|tile| !crate::interior::has_back_wall(&tile.source.tileset_id))
}
fn sync(
    frame: Res<VisualWorldFrame>,
    settings: Res<VoxelViewSettings>,
    clock: Option<Res<VisualWorldClock>>,
    period: Option<Res<VisualWorldLighting>>,
    time: Res<Time>,
    cameras: Query<&Transform, (With<VoxelWorldCamera>, Without<SkyDome>)>,
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
        if let Some(mut sky) = materials.get_mut(&handle.0) {
            *sky = material(second, camera.translation, time.elapsed_secs_wrapped());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    }
}
