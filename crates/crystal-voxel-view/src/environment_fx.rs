//! Retained, presentation-only 3D weather. Source tiles locate emitters; no
//! collision, scripts, terrain textures or gameplay state are modified.
use super::*;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

pub(super) struct EnvironmentFxPlugin;
impl Plugin for EnvironmentFxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Emitters>()
            .add_systems(Startup, setup)
            .add_systems(Update, animate.after(sync_voxel_view));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Leaf,
    Snow,
    Mist,
    Frost,
}
impl Kind {
    fn count(self) -> usize {
        match self {
            Self::Leaf => 40,
            Self::Snow => 48,
            Self::Mist => 16,
            Self::Frost => 20,
        }
    }
}
#[derive(Component)]
struct Particle {
    kind: Kind,
    slot: usize,
}
#[derive(Resource, Default)]
struct Emitters {
    key: Option<TerrainCacheKey>,
    leaves: Vec<Vec3>,
    water: Vec<Vec3>,
    ice: Vec<Vec3>,
    frost: Vec<Vec3>,
    visible: [Vec<Vec3>; 4],
    view_offset: Option<Vec3>,
}

pub(super) fn frozen(frame: &VisualWorldFrame) -> bool {
    interior_cutaway::is_cave(frame)
        && (frame.map_id.starts_with("IcePath") || frame.map_id.starts_with("SeafoamIslands"))
}
pub(super) fn coastal(frame: &VisualWorldFrame) -> bool {
    coastal_map(&frame.map_id)
}
pub(crate) fn coastal_map(map:&str)->bool {
    matches!(
        map,
        "NewBarkTown"
            | "OlivineCity"
            | "CianwoodCity"
            | "CherrygroveCity"
            | "Route19"
            | "Route20"
            | "Route21"
            | "Route26"
            | "Route27"
            | "Route34"
            | "Route40"
            | "Route41"
            | "WhirlIslandNW"
            | "WhirlIslandNE"
            | "WhirlIslandSW"
            | "WhirlIslandSE"
    )
}

fn rebuild(frame: &VisualWorldFrame, emitters: &mut Emitters) {
    emitters.leaves.clear();
    emitters.water.clear();
    emitters.ice.clear();
    emitters.frost.clear();
    let size = frame.grid_size.as_vec2() * frame.tile_size;
    let bounds = authentic_mesh_bounds(frame);
    for tile in &frame.tiles {
        let p = Vec3::new(
            (tile.column as f32 + 0.5) * frame.tile_size.x - size.x * 0.5,
            0.0,
            (tile.row as f32 + 0.5) * frame.tile_size.y - size.y * 0.5,
        );
        if bounds.is_some_and(|b| p.x < b.min.x || p.x > b.max.x || p.z < b.min.y || p.z > b.max.y)
        {
            continue;
        }
        let shape = shape_for_source(&tile.source);
        if !interior_cutaway::is_cave(frame)
            && matches!(
                tile.source.tileset_id.as_ref(),
                "johto" | "johto_modern" | "kanto" | "forest" | "park"
            )
            && matches!(
                shape,
                CellShape::Cutout {
                    solid: SolidKind::Tree,
                    ..
                } | CellShape::RaisedTop {
                    solid: SolidKind::Tree,
                    ..
                } | CellShape::FacadeBand {
                    solid: SolidKind::Tree,
                    ..
                }
            )
        {
            emitters.leaves.push(p);
        }
        if coastal(frame) && shape == CellShape::Water {
            emitters.water.push(p);
        }
        if frozen(frame) {
            if tile.source.tileset_id.as_ref() == "ice_path"
                && tile.source.tile_index == ice_path::SMOOTH_ICE_TILE
                || frame.map_id.starts_with("SeafoamIslands")
                    && tile.source.tileset_id.as_ref() == "cave"
                    && tile.source.tile_index == 0x16
            {
                emitters.ice.push(p);
            }
            if ice_path::edge_rock_local(&tile.source).is_some_and(|(_, x, y)| x == 0 && y == 0) {
                emitters.frost.push(p + Vec3::Y * 8.0);
            }
            if frame.map_id.starts_with("SeafoamIslands")
                && cave::small_rock_local(&tile.source) == Some((0, 0))
            {
                emitters.frost.push(p + Vec3::Y * cave::CAVE_ROCK_HEIGHT);
            }
        }
    }
    emitters.key = Some(TerrainCacheKey::from_frame(frame));
    emitters.view_offset = None;
}

fn update_visible(emitters: &mut Emitters, offset: Vec3) {
    if emitters.view_offset == Some(offset) {
        return;
    }
    for (visible, anchors) in emitters.visible.iter_mut().zip([
        &emitters.leaves,
        &emitters.ice,
        &emitters.water,
        &emitters.frost,
    ]) {
        visible.clear();
        visible.extend(anchors.iter().copied().filter(|p| {
            // The world camera looks at X/Z zero. Terrain translates during
            // scrolling; its own center is not the visible camera focus.
            let world = *p + offset;
            Vec2::new(world.x, world.z).length_squared() <= 220.0 * 220.0
        }));
    }
    emitters.view_offset = Some(offset);
}

// Soft, uneven density, with zero alpha at every edge. Built in memory once;
// no generated image or content-pack artifact is written to disk.
fn mist_pixels() -> Vec<u8> {
    let mut pixels = Vec::with_capacity(64 * 64 * 4);
    for y in 0..64 {
        for x in 0..64 {
            let p = Vec2::new(x as f32, y as f32) / 63.0 * 2.0 - Vec2::ONE;
            let edge = (1.0 - p.length_squared()).max(0.0);
            let billow = 0.62
                + 0.20 * (p.x * 10.0 + p.y * 7.0).sin()
                + 0.18 * (p.x * 17.0 - p.y * 13.0).cos();
            pixels.extend_from_slice(&[220, 238, 245, (edge.powi(3) * billow * 66.0) as u8]);
        }
    }
    pixels
}
pub(super) fn leaf_mesh() -> Mesh {
    // A folded, pointed lamina rather than a flat quad or green sphere.
    let positions = vec![
        [-1.0, 0.0, 0.0],
        [-0.4, 0.0, 0.65],
        [0.0, 0.22, 0.0],
        [0.4, 0.0, 0.65],
        [1.0, 0.0, 0.0],
        [0.4, 0.0, -0.65],
        [-0.4, 0.0, -0.65],
    ];
    let uvs: Vec<[f32; 2]> = positions
        .iter()
        .map(|p| [(p[0] + 1.0) * 0.5, (p[2] / 0.65 + 1.0) * 0.5])
        .collect();
    let mut mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::mesh::Indices::U32(vec![
        0, 1, 2, 1, 3, 2, 3, 4, 2, 4, 5, 2, 5, 6, 2, 6, 0, 2,
    ]));
    mesh.compute_smooth_normals();
    mesh
}
fn leaf_pixels() -> Vec<u8> {
    let mut pixels = Vec::with_capacity(32 * 32 * 4);
    for y in 0..32 {
        for x in 0..32 {
            let u = x as f32 / 31.0;
            let v = y as f32 / 31.0 - 0.5;
            let vein = v.abs() < 0.028 || ((u * 5.0 - v.abs() * 2.0).fract() - 0.5).abs() < 0.035;
            let grain = ((x * 17 + y * 31) % 11) as f32;
            pixels.extend_from_slice(&[
                if vein {
                    160
                } else {
                    (94.0 + 35.0 * u + grain) as u8
                },
                if vein { 171 } else { (134.0 + grain) as u8 },
                if vein { 58 } else { 38 },
                255,
            ]);
        }
    }
    pixels
}
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let mist = images.add(Image::new(
        Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        mist_pixels(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ));
    let leaf = images.add(Image::new(
        Extent3d {
            width: 32,
            height: 32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        leaf_pixels(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ));
    for kind in [Kind::Leaf, Kind::Snow, Kind::Mist, Kind::Frost] {
        let mesh = meshes.add(match kind {
            Kind::Leaf => leaf_mesh(),
            Kind::Snow => Mesh::from(Sphere::new(0.45).mesh().uv(6, 4)),
            Kind::Mist => Mesh::from(Rectangle::new(1.0, 1.0)),
            Kind::Frost => Mesh::from(
                Cone {
                    radius: 0.6,
                    height: 4.0,
                }
                .mesh()
                .resolution(6),
            ),
        });
        let material = materials.add(StandardMaterial {
            base_color: match kind {
                Kind::Leaf => Color::WHITE,
                Kind::Snow => Color::srgb(0.82, 0.94, 1.0),
                Kind::Mist => Color::WHITE,
                Kind::Frost => Color::srgb(0.48, 0.79, 0.92),
            },
            base_color_texture: match kind {
                Kind::Mist => Some(mist.clone()),
                Kind::Leaf => Some(leaf.clone()),
                _ => None,
            },
            alpha_mode: if kind == Kind::Mist {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            },
            perceptual_roughness: if kind == Kind::Frost { 0.18 } else { 0.85 },
            metallic: if kind == Kind::Frost { 0.25 } else { 0.0 },
            unlit: kind == Kind::Mist,
            double_sided: true,
            cull_mode: None,
            ..default()
        });
        for slot in 0..kind.count() {
            commands.spawn((
                Particle { kind, slot },
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::default(),
                Visibility::Hidden,
                RenderLayers::layer(VOXEL_RENDER_LAYER),
                NotShadowCaster,
                NotShadowReceiver,
            ));
        }
    }
}

fn pose(kind: Kind, slot: usize, anchor: Vec3, t: f32, camera: Quat) -> Transform {
    let seed = slot as f32 * 2.399963;
    let phase = (t * match kind {
        Kind::Leaf => 0.13,
        Kind::Snow => 0.10,
        _ => 0.07,
    } + seed.fract())
    .fract();
    let envelope = (phase * std::f32::consts::PI).sin();
    let mut transform = Transform::from_translation(anchor);
    match kind {
        Kind::Leaf => {
            transform.translation += Vec3::new(
                (phase - 0.5) * 38.0 + (t + seed).sin() * 3.0,
                3.0 + (1.0 - phase) * 32.0,
                (seed + t * 0.7).cos() * 8.0,
            );
            transform.rotation = Quat::from_euler(EulerRot::XYZ, t * 2.0 + seed, seed + t, t * 1.4);
            transform.scale = Vec3::new(1.4, 0.8, 0.9) * envelope;
        }
        Kind::Snow => {
            transform.translation += Vec3::new(
                (seed + t * 0.8).sin() * 7.0,
                2.0 + (1.0 - phase) * 34.0,
                (seed + t * 0.6).cos() * 5.0,
            );
            transform.scale = Vec3::splat(envelope * (0.5 + seed.sin().abs() * 0.5));
        }
        Kind::Mist => {
            transform.translation +=
                Vec3::new((phase - 0.5) * 16.0, 3.0 + (seed + t * 0.3).sin(), 0.0);
            transform.rotation = camera;
            transform.scale = Vec3::new(32.0, 12.0, 1.0) * envelope;
        }
        Kind::Frost => {
            transform.rotation = Quat::from_rotation_z(0.3 * seed.sin());
            transform.scale = Vec3::splat(0.8 + seed.sin().abs() * 0.7);
        }
    }
    transform
}

fn animate(
    frame: Res<VisualWorldFrame>,
    settings: Res<VoxelViewSettings>,
    status: Res<VoxelViewStatus>,
    time: Res<Time>,
    mut emitters: ResMut<Emitters>,
    mut particles: Query<(&Particle, &mut Transform, &mut Visibility)>,
) {
    let active = frame.active && settings.enabled && status.active;
    if active && emitters.key.as_ref() != Some(&TerrainCacheKey::from_frame(&frame)) {
        rebuild(&frame, &mut emitters);
    }
    let camera = settings
        .camera
        .pose(frame.viewport_size)
        .transform()
        .rotation;
    let offset = terrain_transform(&frame).translation;
    if active {
        update_visible(&mut emitters, offset);
    }
    for (particle, mut transform, mut visibility) in &mut particles {
        let anchors = match particle.kind {
            Kind::Leaf => &emitters.visible[0],
            Kind::Snow => &emitters.visible[1],
            Kind::Mist => &emitters.visible[2],
            Kind::Frost => &emitters.visible[3],
        };
        if !active || anchors.is_empty() {
            *visibility = Visibility::Hidden;
            continue;
        }
        // Spread the whole bounded pool across nearby source emitters, even
        // when a long route contains thousands of distant tree cells.
        let anchor = anchors[particle.slot * anchors.len() / particle.kind.count()] + offset;
        *visibility = Visibility::Visible;
        *transform = pose(
            particle.kind,
            particle.slot,
            anchor,
            time.elapsed_secs(),
            camera,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crystal_render_api::{VisualTile, VisualTileSource};
    fn fixture(map: &str, tileset: &str, index: u16) -> VisualWorldFrame {
        VisualWorldFrame {
            active: true,
            map_id: map.into(),
            grid_size: UVec2::new(2, 2),
            tile_size: Vec2::splat(8.0),
            viewport_size: Vec2::splat(16.0),
            source_map_size_core_tiles: Some(UVec2::ONE),
            tiles: (0..4)
                .map(|i| VisualTile {
                    column: i % 2,
                    row: i / 2,
                    priority: false,
                    animation_frames: None,
                    texture: default(),
                    source: VisualTileSource {
                        tileset_id: tileset.into(),
                        tile_index: index,
                        metatile_id: 0x05,
                        subtile_column: 0,
                        subtile_row: 0,
                    },
                })
                .collect(),
            ..default()
        }
    }
    #[test]
    fn environment_pool_retains_assets_animates_and_hides_when_3d_stops() {
        let frame = fixture("IcePath1F", "ice_path", ice_path::SMOOTH_ICE_TILE);
        let mut app = App::new();
        app.insert_resource(frame)
            .insert_resource(VoxelViewSettings {
                enabled: true,
                ..default()
            })
            .insert_resource(VoxelViewStatus {
                active: true,
                ..default()
            })
            .init_resource::<Time>()
            .init_resource::<Emitters>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<Assets<Image>>()
            .add_systems(Startup, setup)
            .add_systems(Update, animate);
        app.update();
        let assets = (
            app.world().resource::<Assets<Mesh>>().len(),
            app.world().resource::<Assets<StandardMaterial>>().len(),
            app.world().resource::<Assets<Image>>().len(),
        );
        let mut query = app
            .world_mut()
            .query::<(&Particle, &Transform, &Visibility, &RenderLayers)>();
        let before: Vec<_> = query
            .iter(app.world())
            .filter(|(p, _, _, _)| p.kind == Kind::Snow)
            .map(|(_, t, v, layers)| {
                assert_eq!(layers, &RenderLayers::layer(VOXEL_RENDER_LAYER));
                assert_eq!(*v, Visibility::Visible);
                t.translation
            })
            .collect();
        assert_eq!(before.len(), 48);
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs(1));
        app.update();
        let after: Vec<_> = query
            .iter(app.world())
            .filter(|(p, _, _, _)| p.kind == Kind::Snow)
            .map(|(_, t, _, _)| t.translation)
            .collect();
        assert_ne!(before, after);
        app.world_mut().resource_mut::<VisualWorldFrame>().center.x = 300.0;
        app.update();
        assert!(
            query
                .iter(app.world())
                .all(|(_, _, v, _)| *v == Visibility::Hidden),
            "scrolling terrain away from the camera must hide its emitters"
        );
        app.world_mut().resource_mut::<VisualWorldFrame>().center.x = 0.0;
        app.update();
        assert_eq!(
            query
                .iter(app.world())
                .filter(|(_, _, v, _)| **v == Visibility::Visible)
                .count(),
            48
        );
        assert_eq!(
            assets,
            (
                app.world().resource::<Assets<Mesh>>().len(),
                app.world().resource::<Assets<StandardMaterial>>().len(),
                app.world().resource::<Assets<Image>>().len()
            )
        );
        app.world_mut().resource_mut::<VoxelViewSettings>().enabled = false;
        app.update();
        assert!(
            query
                .iter(app.world())
                .all(|(_, _, v, _)| *v == Visibility::Hidden)
        );
        app.world_mut().resource_mut::<VoxelViewSettings>().enabled = true;
        *app.world_mut().resource_mut::<VisualWorldFrame>() =
            fixture("PlayersHouse1F", "house", 0x14);
        app.update();
        assert!(
            query
                .iter(app.world())
                .all(|(_, _, v, _)| *v == Visibility::Hidden)
        );
        assert_eq!(query.iter(app.world()).count(), 124);
    }
    #[test]
    fn weather_uses_source_roles_and_excludes_padded_terrain_and_ceremonial_ice() {
        let mut emitters = Emitters::default();
        let mut frame = fixture("Route40", "johto", 0x14);
        rebuild(&frame, &mut emitters);
        assert_eq!(emitters.water.len(), 4);
        frame.grid_origin = IVec2::new(-1, -1);
        rebuild(&frame, &mut emitters);
        assert_eq!(emitters.water.len(), 1, "padding cannot emit ocean spray");
        frame = fixture("HallOfFame", "ice_path", ice_path::SMOOTH_ICE_TILE);
        rebuild(&frame, &mut emitters);
        assert!(emitters.ice.is_empty());
        for map in ["NewBarkTown", "Route29"] {
            frame = fixture(map, "johto", 0x26);
            rebuild(&frame, &mut emitters);
            assert!(
                !emitters.leaves.is_empty(),
                "reference map must emit leaves: {map}"
            );
            assert!(emitters.ice.is_empty() && emitters.water.is_empty());
        }
    }
    #[test]
    fn mist_texture_has_soft_edges_and_uneven_density() {
        let pixels = mist_pixels();
        assert_eq!(pixels.len(), 64 * 64 * 4);
        for y in 0..64 {
            for x in 0..64 {
                if x == 0 || y == 0 || x == 63 || y == 63 {
                    assert_eq!(pixels[(y * 64 + x) * 4 + 3], 0);
                }
            }
        }
        assert!(pixels.chunks_exact(4).any(|p| p[3] > 30));
    }

    #[test]
    fn light_palette_restores_without_automatic_haze() {
        let mut app = App::new();
        app.insert_resource(fixture("NewBarkTown", "johto", 0x26))
            .insert_resource(VoxelViewSettings {
                enabled: true,
                ..default()
            })
            .init_resource::<live_profiles::LiveProfiles>()
            .add_systems(Update, sync_voxel_atmosphere);
        let sun = app
            .world_mut()
            .spawn((
                DirectionalLight::default(),
                bevy::light::CascadeShadowConfig::default(),
                VoxelSun,
            ))
            .id();
        let camera = app
            .world_mut()
            .spawn((DistanceFog::default(), Fxaa::default(), VoxelWorldCamera))
            .id();
        app.update();
        assert_eq!(
            app.world().get::<DirectionalLight>(sun).unwrap().color,
            Color::srgb(1.0, 0.94, 0.82)
        );
        assert_eq!(
            app.world().get::<DistanceFog>(camera).unwrap().color,
            Color::NONE
        );
        *app.world_mut().resource_mut::<VisualWorldFrame>() =
            fixture("IcePath1F", "ice_path", ice_path::SMOOTH_ICE_TILE);
        app.update();
        assert_eq!(
            app.world().get::<DirectionalLight>(sun).unwrap().color,
            Color::srgb(0.68, 0.85, 1.0)
        );
        *app.world_mut().resource_mut::<VisualWorldFrame>() =
            fixture("PlayersHouse1F", "house", 0x14);
        app.update();
        assert_eq!(
            app.world().get::<DirectionalLight>(sun).unwrap().color,
            Color::WHITE
        );
        assert_eq!(
            app.world().get::<DistanceFog>(camera).unwrap().color,
            Color::NONE
        );
        *app.world_mut().resource_mut::<VisualWorldFrame>() = fixture("Route29", "johto", 0x26);
        app.world_mut().resource_mut::<VoxelViewSettings>().enabled = false;
        app.update();
        assert_eq!(
            app.world().get::<DistanceFog>(camera).unwrap().color,
            Color::NONE
        );
    }
}
