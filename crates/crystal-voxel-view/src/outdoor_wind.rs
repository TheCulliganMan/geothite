//! Bounded decorative gusts over authenticated woodland and coastal water.
//! No gameplay state, particle accumulation, or terrain rebuilds.
use super::*;
use crate::profile::{CellShape, SolidKind, shape_for_source_on_map};
use bevy::light::{NotShadowCaster, NotShadowReceiver};

#[derive(Component)]
struct WindParticle(usize);
#[derive(Resource, Default)]
struct WindSites {
    key: Option<TerrainCacheKey>,
    leaves: Vec<Vec3>,
    gusts: Vec<Vec3>,
    nearby: [Vec<Vec3>; 2],
    offset: Option<(Vec3, Vec2)>,
}

pub(super) fn register(app: &mut App) {
    app.init_resource::<WindSites>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            sync.after(sync_voxel_view)
                .in_set(WorldRenderSet::RenderSync),
        );
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let leaf = meshes.add(environment_fx::leaf_mesh());
    // A short, tapered curved ribbon reads as a passing breeze rather than rain.
    let mut ribbon = mesh::SurfaceMeshData::default();
    for i in 0..12 {
        let x = i as f32 / 11.;
        let z = (x * std::f32::consts::PI).sin() * 0.7;
        let width = (x * std::f32::consts::PI).sin() * 0.26;
        for side in [-1., 1.] {
            ribbon.positions.push([x * 7. - 3.5, 0., z + side * width]);
            ribbon.normals.push([0., 1., 0.]);
            ribbon.uvs.push([0., 0.]);
            ribbon.colors.push([1.; 4]);
        }
        if i < 11 {
            let a = i * 2;
            ribbon
                .indices
                .extend([a, a + 2, a + 1, a + 1, a + 2, a + 3]);
        }
    }
    let wind = meshes.add(ribbon.into_mesh());
    let handles: Vec<_> = [
        Color::srgb(0.38, 0.49, 0.16),
        Color::srgb(0.57, 0.48, 0.22),
        Color::srgba(0.80, 0.92, 0.91, 0.38),
    ]
    .into_iter()
    .map(|color| {
        materials.add(StandardMaterial {
            base_color: color,
            cull_mode: None,
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 1.,
            unlit: true,
            ..default()
        })
    })
    .collect();
    for i in 0..32 {
        commands.spawn((
            WindParticle(i),
            Mesh3d(if i < 24 { leaf.clone() } else { wind.clone() }),
            MeshMaterial3d(handles[if i < 24 { i % 2 } else { 2 }].clone()),
            Transform::default(),
            Visibility::Hidden,
            NotShadowCaster,
            NotShadowReceiver,
            RenderLayers::layer(VOXEL_RENDER_LAYER),
        ));
    }
}

fn collect_sites(frame: &VisualWorldFrame, heights: &[f32]) -> (Vec<Vec3>, Vec<Vec3>) {
    let wooded = frame.map_id.starts_with("Route")
        || matches!(frame.map_id.as_ref(), "IlexForest" | "ViridianForest");
    let mut leaves = Vec::new();
    let mut sea = Vec::new();
    let mut woodland_clearings = Vec::new();
    let unit = frame.tile_size.min_element() / 8.;
    let Some(bounds) = authentic_mesh_bounds(frame) else {
        return (leaves, sea);
    };
    for tile in &frame.tiles {
        if !matches!(
            tile.source.tileset_id.as_ref(),
            "johto" | "johto_modern" | "kanto" | "forest" | "park"
        ) {
            continue;
        }
        let p = (Vec2::new(tile.column as f32, tile.row as f32) + Vec2::splat(0.5)
            - frame.grid_size.as_vec2() * 0.5)
            * frame.tile_size;
        if p.x < bounds.min.x || p.x >= bounds.max.x || p.y < bounds.min.y || p.y >= bounds.max.y {
            continue;
        }
        let shape = shape_for_source_on_map(&frame.map_id, &tile.source);
        let base = heights
            .get(tile.row as usize * frame.grid_size.x as usize + tile.column as usize)
            .copied()
            .unwrap_or_else(|| shape.surface_height(frame.tile_size.y));
        if wooded
            && matches!(
                shape,
                CellShape::RaisedTop {
                    solid: SolidKind::Tree,
                    ..
                } | CellShape::Cutout {
                    solid: SolidKind::Tree,
                    ..
                } | CellShape::FacadeBand {
                    solid: SolidKind::Tree,
                    ..
                }
            )
        {
            leaves.push(Vec3::new(p.x, base + frame.tile_size.y * 4.5, p.y));
        }
        if wooded && shape == CellShape::Flat {
            woodland_clearings.push(Vec3::new(p.x, base + 2.5 * unit, p.y));
        }
        if environment_fx::coastal(frame) && shape == CellShape::Water {
            sea.push(Vec3::new(p.x, base + 1.4 * unit, p.y));
        }
    }
    if sea.is_empty() && !leaves.is_empty() {
        sea = woodland_clearings;
    }
    (leaves, sea)
}

fn sync(
    frame: Res<VisualWorldFrame>,
    status: Res<VoxelViewStatus>,
    cache: Res<TerrainRevisionCache>,
    time: Res<Time>,
    mut sites: ResMut<WindSites>,
    mut particles: Query<(&WindParticle, &mut Transform, &mut Visibility)>,
) {
    if !status.active || !frame.active {
        for (_, _, mut visible) in &mut particles {
            *visible = Visibility::Hidden;
        }
        return;
    }
    let source = cache
        .built_frame
        .as_ref()
        .filter(|built| built.map_id == frame.map_id)
        .unwrap_or(&frame);
    let key = TerrainCacheKey::from_frame(source);
    if sites.key.as_ref() != Some(&key) {
        (sites.leaves, sites.gusts) = collect_sites(source, &cache.built_footing_heights);
        sites.key = Some(key);
        sites.offset = None;
    }
    let t = time.elapsed_secs();
    let offset = retained_terrain_transform(&frame, &cache)
        .unwrap_or_else(|| terrain_transform(&frame))
        .translation;
    let bounds = authentic_mesh_bounds(source);
    let unit = frame.tile_size.min_element() / 8.;
    if sites.offset != Some((offset, frame.view_center)) {
        // Spend the bounded pool near the camera, including on long routes.
        let radius = frame.tile_size.min_element() * 7.;
        let nearby = |points: &[Vec3]| {
            points
                .iter()
                .copied()
                .filter(|p| {
                    let world = *p + offset;
                    (Vec2::new(world.x, -world.z) - frame.view_center).length_squared() < radius * radius
                })
                .collect()
        };
        sites.nearby = [nearby(&sites.leaves), nearby(&sites.gusts)];
        sites.offset = Some((offset, frame.view_center));
        #[cfg(not(target_arch = "wasm32"))]
        if std::env::var_os("CRYSTAL_WIND_TRACE").is_some() {
            eprintln!(
                "wind map={} tiles={} unit={} sites={}/{} nearby={}/{} offset={:?} clock={}",
                frame.map_id,
                frame.tiles.len(),
                unit,
                sites.leaves.len(),
                sites.gusts.len(),
                sites.nearby[0].len(),
                sites.nearby[1].len(),
                offset,
                t
            );
        }
    }
    for (particle, mut transform, mut visible) in &mut particles {
        let i = particle.0;
        let sources = &sites.nearby[usize::from(i >= 24)];
        let cycle = (t + (i % 8) as f32 * 0.31) / 17.;
        let age = cycle.fract() * 17.;
        let life = if i < 24 { 5.5 } else { 6.0 };
        if sources.is_empty() || age > life {
            *visible = Visibility::Hidden;
            continue;
        }
        let site = sources[(i * 37 + cycle.floor() as usize * 13) % sources.len()];
        let progress = age / life;
        let fade = (progress * std::f32::consts::PI).sin();
        let flutter = age * 3.2 + i as f32;
        transform.translation = offset
            + site
            + Vec3::new(
                (progress - 0.5) * 14.,
                if i < 24 {
                    -progress * 15. + flutter.sin() * 0.7
                } else {
                    0.
                },
                (progress - 0.5) * 5. + flutter.sin() * 0.9,
            ) * unit;
        let local = transform.translation - offset;
        if bounds.is_none_or(|b| {
            local.x < b.min.x || local.x >= b.max.x || local.z < b.min.y || local.z >= b.max.y
        }) {
            *visible = Visibility::Hidden;
            continue;
        }
        transform.rotation = if i < 24 {
            Quat::from_euler(EulerRot::XYZ, flutter, age * 1.1, age * 0.7)
        } else {
            Quat::from_rotation_y(-0.3)
        };
        transform.scale = Vec3::splat(fade * unit * if i < 24 { 0.85 } else { 1. });
        *visible = Visibility::Visible;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame(map: &str, tile: u16) -> VisualWorldFrame {
        VisualWorldFrame {
            active: true,
            map_id: map.into(),
            tile_size: Vec2::splat(8.),
            grid_size: UVec2::ONE,
            source_map_size_core_tiles: Some(UVec2::ONE),
            tiles: vec![crystal_render_api::VisualTile {
                column: 0,
                row: 0,
                priority: false,
                texture: default(),
                animation_frames: None,
                source: crystal_render_api::VisualTileSource {
                    tileset_id: "johto_modern".into(),
                    metatile_id: 5,
                    subtile_column: 0,
                    subtile_row: 0,
                    tile_index: tile,
                },
            }],
            ..default()
        }
    }
    #[test]
    fn gust_sites_require_native_trees_or_coastal_water_within_the_source_map() {
        let mut trees = frame("Route35", 0x25);
        assert!(!collect_sites(&trees, &[]).0.is_empty());
        assert_eq!(
            collect_sites(&trees, &[64.]).0[0].y - collect_sites(&trees, &[]).0[0].y,
            64.,
            "leaves follow raised native ground"
        );
        trees.map_id = "GoldenrodCity".into();
        assert!(collect_sites(&trees, &[]).0.is_empty());
        let mut sea = frame("Route40", 0x14);
        sea.tiles[0].source.metatile_id = 0;
        assert!(!collect_sites(&sea, &[]).1.is_empty());
        sea.map_id = "Route35".into();
        assert!(
            collect_sites(&sea, &[]).1.is_empty(),
            "inland pond is not sea"
        );
        sea.map_id = "Route40".into();
        sea.grid_origin = IVec2::splat(-10);
        assert!(
            collect_sites(&sea, &[]).1.is_empty(),
            "padded map border is not sea"
        );
        sea.grid_origin = IVec2::ZERO;
        sea.tiles[0].source.tileset_id = "house".into();
        assert!(
            collect_sites(&sea, &[]).1.is_empty(),
            "interiors never emit gusts"
        );
    }
    #[test]
    fn disabling_3d_hides_every_retained_particle() {
        let mut app = App::new();
        app.insert_resource(frame("Route40", 0x14))
            .init_resource::<VoxelViewStatus>()
            .init_resource::<Time>()
            .init_resource::<WindSites>()
            .init_resource::<TerrainRevisionCache>()
            .add_systems(Update, sync);
        let entity = app
            .world_mut()
            .spawn((WindParticle(0), Transform::default(), Visibility::Visible))
            .id();
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(entity).unwrap(),
            Visibility::Hidden
        );
    }
}
