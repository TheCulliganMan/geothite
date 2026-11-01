//! Authored animated-flower presentation identities.

use crystal_render_api::VisualTileSource;

use crate::profile::{CellShape, SolidKind};

const FLOWER_GROUNDS: [(&str, u16); 5] = [
    ("johto", 0x05),
    ("johto_modern", 0x05),
    ("forest", 0x05),
    ("park", 0x01),
    ("kanto", 0x2c),
];

pub(crate) const SOURCE_BLOOM_TAG: f32 = 86200.;

pub(crate) fn flower_shape(source: &VisualTileSource) -> Option<CellShape> {
    if source.tile_index != 0x03 {
        return None;
    }
    let ground = FLOWER_GROUNDS.iter().find_map(|(tileset, ground)| {
        (source.tileset_id.as_ref() == *tileset).then_some(*ground)
    })?;
    Some(CellShape::Cutout {
        ground_tile_index: ground,
        solid: SolidKind::Flower,
    })
}

/// Mark only the appended bloom geometry. UV height keeps the root planted;
/// solid palette flowers do not sample the source animation atlas.
pub(crate) fn mark_sway(mesh: &mut crate::mesh::SurfaceMeshData, start: usize, base: f32) {
    for (position, uv) in mesh.positions[start..].iter().zip(&mut mesh.uvs[start..]) {
        *uv = [86000.0, (position[1] - base).max(0.0)];
    }
}

/// Animated solids share the exact terrain finish without sharing its asset
/// identity. Bevy dirties every mesh using a changed material; the blossom
/// clock must not repeatedly invalidate static terrain and grove instances.
pub(super) fn animated_material(
    cached: &mut Option<bevy::prelude::Handle<crate::VoxelMaterial>>,
    source: &bevy::prelude::Handle<crate::VoxelMaterial>,
    materials: &mut bevy::prelude::Assets<crate::VoxelMaterial>,
) -> bevy::prelude::Handle<crate::VoxelMaterial> {
    let material = materials.get(source).expect("published solid terrain material").clone();
    if let Some(handle) = cached {
        *materials.get_mut(&*handle).expect("retained animated solid material") = material;
        return handle.clone();
    }
    let handle = materials.add(material);
    *cached = Some(handle.clone());
    handle
}

/// Share the unchanged blossom clock and phase with every vertex pass, using
/// only the material owned by the mutable animated solid mesh.
pub(super) fn animate(
    frame: bevy::prelude::Res<crystal_render_api::VisualWorldFrame>,
    time: bevy::prelude::Res<bevy::prelude::Time>,
    status: bevy::prelude::Res<crate::VoxelViewStatus>,
    cache: bevy::prelude::Res<crate::TerrainRevisionCache>,
    mut materials: bevy::prelude::ResMut<bevy::prelude::Assets<crate::VoxelMaterial>>,
) {
    if !status.active || cache.built_animated_bounds[1].is_none() { return; }
    if let Some(mut material) = cache.animated_solid_material.as_ref().and_then(|h| materials.get_mut(h)) {
        material.extension.grass_rustle.height_scale.y = time.elapsed_secs();
        material.extension.grass_rustle.height_scale.z = -frame.view_center.x;
        material.extension.grass_rustle.height_scale.w = frame.view_center.y;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn animated_material_retains_its_identity_and_refreshes_the_exact_finish() {
        use bevy::prelude::*;
        let mut materials = Assets::<crate::VoxelMaterial>::default();
        let source = materials.add(crate::solid_terrain_material());
        let mut cached = None;
        let animated = animated_material(&mut cached, &source, &mut materials);
        assert_ne!(animated, source);
        {
            let mut material = materials.get_mut(&source).unwrap();
            material.base.base_color = Color::srgb(0.2, 0.4, 0.7);
            material.extension.cave_reveal = Vec4::new(1., 0., 0., 0.);
            material.extension.grass_rustle.height_scale = Vec4::new(0.75, 12., 24., 36.);
            material.extension.cutaway = crate::interior_cutaway::CutawayUniform::for_player(
                Vec3::new(8., 16., 24.), 32.).unwrap();
        }
        assert_eq!(animated_material(&mut cached, &source, &mut materials), animated);
        assert_eq!(materials.len(), 2, "geometry revisions must reuse the animated material");
        let source = materials.get(&source).unwrap();
        let animated = materials.get(&animated).unwrap();
        assert_eq!(animated.base.base_color, source.base.base_color);
        assert_eq!(animated.extension.cave_reveal, source.extension.cave_reveal);
        assert_eq!(animated.extension.grass_rustle, source.extension.grass_rustle);
        assert_eq!(animated.extension.cutaway, source.extension.cutaway);
    }

    #[test]
    fn blossom_clock_never_marks_the_shared_static_material_modified() {
        use bevy::{asset::{AssetEvent, AssetPlugin}, prelude::*};
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default())).init_asset::<crate::VoxelMaterial>();
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1. / 60.)));
        let (source, animated) = {
            let mut materials = app.world_mut().resource_mut::<Assets<crate::VoxelMaterial>>();
            let source = materials.add(crate::solid_terrain_material());
            let animated = animated_material(&mut None, &source, &mut materials);
            (source, animated)
        };
        let frame = crystal_render_api::VisualWorldFrame {
            active: true, view_center: Vec2::new(80., -72.), ..default()
        };
        app.insert_resource(frame)
            .insert_resource(crate::VoxelViewStatus { active: true, ..default() })
            .insert_resource(crate::TerrainRevisionCache {
                solid_material: Some(source.clone()), animated_solid_material: Some(animated.clone()),
                built_animated_bounds: [None, Some(bevy::camera::primitives::Aabb::default())],
                ..default()
            })
            .add_systems(Update, animate);
        app.update();
        app.world_mut().resource_mut::<Messages<AssetEvent<crate::VoxelMaterial>>>().clear();
        for _ in 0..64 {
            app.update();
            let events = app.world_mut().resource_mut::<Messages<AssetEvent<crate::VoxelMaterial>>>()
                .drain().collect::<Vec<_>>();
            assert!(events.iter().any(|event| event.is_modified(&animated)));
            assert!(!events.iter().any(|event| event.is_modified(&source)),
                "the animation clock must not dirty every static mesh using the shared finish");
            let materials = app.world().resource::<Assets<crate::VoxelMaterial>>();
            let phase = materials.get(&animated).unwrap().extension.grass_rustle.height_scale;
            assert_eq!(phase.y, app.world().resource::<Time>().elapsed_secs());
            assert_eq!((phase.z, phase.w), (-80., -72.));
            assert_eq!(materials.get(&source).unwrap().extension.grass_rustle.height_scale, Vec4::ZERO);
        }
    }

    #[test]
    fn every_flower_animation_tileset_uses_its_authored_ground() {
        for (tileset, ground) in FLOWER_GROUNDS {
            let source = VisualTileSource {
                tileset_id: Arc::from(tileset),
                metatile_id: 0,
                subtile_column: 0,
                subtile_row: 0,
                tile_index: 0x03,
            };
            assert_eq!(
                flower_shape(&source),
                Some(CellShape::Cutout {
                    ground_tile_index: ground,
                    solid: SolidKind::Flower,
                })
            );
        }
    }

    #[test]
    fn non_animated_tileset_reusing_slot_three_stays_unprofiled() {
        let source = VisualTileSource {
            tileset_id: Arc::from("house"),
            metatile_id: 0,
            subtile_column: 0,
            subtile_row: 0,
            tile_index: 0x03,
        };
        assert_eq!(flower_shape(&source), None);
    }
}
