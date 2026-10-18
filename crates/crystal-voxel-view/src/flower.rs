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

/// Share the blossom clock with the depth/shadow vertex passes, whose view
/// bindings deliberately omit Bevy's forward globals. Only the retained solid
/// terrain material owns blooms, so other scene materials remain unchanged.
pub(super) fn animate(
    time: bevy::prelude::Res<bevy::prelude::Time>,
    status: bevy::prelude::Res<crate::VoxelViewStatus>,
    cache: bevy::prelude::Res<crate::TerrainRevisionCache>,
    mut materials: bevy::prelude::ResMut<bevy::prelude::Assets<crate::VoxelMaterial>>,
) {
    if !status.active || cache.built_animated_bounds[1].is_none() { return; }
    if let Some(mut material) = cache.solid_material.as_ref().and_then(|h| materials.get_mut(h)) {
        material.extension.grass_rustle.height_scale.y = time.elapsed_secs();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

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
