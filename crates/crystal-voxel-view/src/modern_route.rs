//! Authored Johto-modern route scenery made from mixed-role metatiles.
//!
//! Route 34's Day-Care block combines compact trees, a sign, and fence art in
//! one 4x4 source drawing. Resolve those objects by their stable source cells
//! instead of assigning one voxel role to the complete metatile.

use crystal_render_api::VisualTileSource;

use crate::profile::{CellShape, LedgeFace, MOUNTAIN_CLIFF_HEIGHT, SolidKind};

const TILESET: &str = "johto_modern";
const GROUND: u16 = 0x06;

pub(crate) fn modern_route_shape(source: &VisualTileSource) -> Option<CellShape> {
    if source.tileset_id.as_ref() != TILESET {
        return None;
    }

    // These hop-down drawings are shared byte-for-byte with the Johto atlas.
    // Reuse its established profile rather than leaving modern-route ledges
    // painted on the floor. Mixed fence/corner variants remain separate.
    if matches!(source.metatile_id, 0x4b | 0x50..=0x53 | 0x56 | 0x57) {
        let mut shared = source.clone();
        shared.tileset_id = "johto".into();
        return Some(crate::profile::shape_for_source(&shared));
    }

    // The modern atlas also owns the mountain family used on Route33.
    // Resolve its mineral glyphs before generic modern-city decoration.
    // Its corner grass and southeastern cave opening remain separate art.
    if (0x68..=0x73).contains(&source.metatile_id) {
        let face = match source.tile_index {
            0x3b => Some((
                LedgeFace::West,
                0,
                1_u8.saturating_sub(source.subtile_column),
            )),
            0x3d => Some((
                LedgeFace::East,
                4,
                source.subtile_column.saturating_sub(2).min(1),
            )),
            0x4b | 0x4c | 0x4d => Some((
                LedgeFace::South,
                4,
                source.subtile_row.saturating_sub(2).min(1),
            )),
            _ => None,
        };
        if let Some((face, plane_subtile, band_from_top)) = face {
            return Some(CellShape::LedgeBand {
                face,
                plane_subtile,
                band_from_top,
                band_count: 2,
                top_tile_index: 0x3c,
                height: MOUNTAIN_CLIFF_HEIGHT,
            });
        }
        return Some(if matches!(source.tile_index, 0x2b | 0x2c | 0x2d | 0x3c) {
            CellShape::RaisedTop {
                height: MOUNTAIN_CLIFF_HEIGHT,
                solid: SolidKind::Bank,
            }
        } else {
            CellShape::Flat
        });
    }

    // Day-Care west plot: two independent 16x16 headbutt-tree drawings.
    if source.metatile_id == 0x3d && source.subtile_row < 2 {
        return Some(CellShape::FacadeBand {
            plane_subtile_row: 2,
            band_from_top: source.subtile_row,
            band_count: 2,
            ground_tile_index: GROUND,
            solid: SolidKind::Tree,
        });
    }

    // The sign occupies the southwest 2x2 drawing beneath those trees.
    if source.metatile_id == 0x3d && source.subtile_column < 2 && source.subtile_row >= 2 {
        return Some(CellShape::FacadeBand {
            plane_subtile_row: 4,
            band_from_top: source.subtile_row - 2,
            band_count: 2,
            ground_tile_index: GROUND,
            solid: SolidKind::Prop,
        });
    }

    // The north half of $40-$42 is one complete horizontal fence drawing.
    // Corner blocks replace an end cell of the lower rail with the vertical
    // post art; it still belongs to the same two-band silhouette. Treating
    // these cells as generic props made the rail shallow and left the post
    // continuation painted flat on the ground.
    if matches!(source.metatile_id, 0x40..=0x42) && source.subtile_row < 2 {
        return Some(CellShape::FacadeBand {
            plane_subtile_row: 2,
            band_from_top: source.subtile_row,
            band_count: 2,
            ground_tile_index: GROUND,
            solid: SolidKind::Fence,
        });
    }

    // $5a/$59 are the upper/lower courses of Route 34's horizontal fence.
    // Fold them together once; they occur in straight runs and mixed corners.
    if matches!(source.metatile_id, 0x3d | 0x48 | 0x49 | 0x4a)
        && matches!(source.tile_index, 0x5a | 0x59)
    {
        return Some(CellShape::FacadeBand {
            plane_subtile_row: 4,
            band_from_top: u8::from(source.tile_index == 0x59),
            band_count: 2,
            ground_tile_index: GROUND,
            solid: SolidKind::Fence,
        });
    }

    // $4a is one north-south fence post repeated through the containing
    // block. Each source cell stands independently instead of forming a
    // fence-textured wall or remaining painted on the ground.
    if matches!(source.metatile_id, 0x40 | 0x42 | 0x44 | 0x46 | 0x48 | 0x4a)
        && source.tile_index == 0x4a
    {
        return Some(CellShape::FacadeBand {
            plane_subtile_row: source.subtile_row + 1,
            band_from_top: 0,
            band_count: 1,
            ground_tile_index: GROUND,
            solid: SolidKind::Fence,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    fn source(metatile_id: u16, column: u8, row: u8, tile_index: u16) -> VisualTileSource {
        VisualTileSource {
            tileset_id: Arc::from(TILESET),
            metatile_id,
            subtile_column: column,
            subtile_row: row,
            tile_index,
        }
    }

    #[test]
    fn modern_route_hop_down_edge_uses_the_shared_shallow_height() {
        let top = modern_route_shape(&source(0x57, 1, 1, 0x05));
        let face = modern_route_shape(&source(0x57, 1, 3, 0x4c));
        assert!(matches!(
            top,
            Some(CellShape::RaisedTop {
                height: crate::profile::JUMP_LEDGE_HEIGHT,
                solid: SolidKind::Bank
            })
        ));
        assert!(matches!(
            face,
            Some(CellShape::LedgeBand {
                height: crate::profile::JUMP_LEDGE_HEIGHT,
                face: LedgeFace::South,
                ..
            })
        ));
    }

    #[test]
    fn modern_mountain_sides_caps_and_cave_opening_have_distinct_roles() {
        assert!(matches!(
            modern_route_shape(&source(0x71, 2, 2, 0x3c)),
            Some(CellShape::RaisedTop {
                solid: SolidKind::Bank,
                height: MOUNTAIN_CLIFF_HEIGHT
            })
        ));
        assert!(matches!(
            modern_route_shape(&source(0x68, 0, 1, 0x3b)),
            Some(CellShape::LedgeBand {
                face: LedgeFace::West,
                ..
            })
        ));
        assert!(matches!(
            modern_route_shape(&source(0x72, 1, 3, 0x4c)),
            Some(CellShape::LedgeBand {
                face: LedgeFace::South,
                ..
            })
        ));
        for (x, y, tile) in [(2, 2, 0x45), (3, 2, 0x22), (2, 3, 0x27), (3, 3, 0x28)] {
            assert_eq!(
                modern_route_shape(&source(0x73, x, y, tile)),
                Some(CellShape::Flat)
            );
        }
        assert_eq!(
            modern_route_shape(&source(0x6a, 0, 0, 0x05)),
            Some(CellShape::Flat)
        );
    }

    #[test]
    fn daycare_mixed_block_separates_tree_sign_and_fence_roles() {
        assert!(matches!(
            modern_route_shape(&source(0x3d, 0, 0, 0x1e)),
            Some(CellShape::FacadeBand {
                solid: SolidKind::Tree,
                ..
            })
        ));
        assert!(matches!(
            modern_route_shape(&source(0x3d, 0, 2, 0x4e)),
            Some(CellShape::FacadeBand {
                solid: SolidKind::Prop,
                band_count: 2,
                ..
            })
        ));
        assert!(matches!(
            modern_route_shape(&source(0x3d, 2, 2, 0x5a)),
            Some(CellShape::FacadeBand {
                solid: SolidKind::Fence,
                band_from_top: 0,
                ..
            })
        ));
        assert!(matches!(
            modern_route_shape(&source(0x3d, 2, 3, 0x59)),
            Some(CellShape::FacadeBand {
                solid: SolidKind::Fence,
                band_from_top: 1,
                ..
            })
        ));
    }

    #[test]
    fn vertical_route_fence_cells_are_independent_upright_posts() {
        assert!(matches!(
            modern_route_shape(&source(0x44, 0, 2, 0x4a)),
            Some(CellShape::FacadeBand {
                plane_subtile_row: 3,
                band_count: 1,
                ..
            })
        ));
    }

    #[test]
    fn goldenrod_fence_caps_and_corner_posts_share_the_two_band_rail() {
        for metatile_id in 0x40..=0x42 {
            assert!(matches!(
                modern_route_shape(&source(metatile_id, 0, 0, 0x5a)),
                Some(CellShape::FacadeBand {
                    plane_subtile_row: 2,
                    band_from_top: 0,
                    band_count: 2,
                    solid: SolidKind::Fence,
                    ..
                })
            ));
        }
        assert!(matches!(
            modern_route_shape(&source(0x40, 0, 1, 0x4a)),
            Some(CellShape::FacadeBand {
                band_from_top: 1,
                solid: SolidKind::Fence,
                ..
            })
        ));
        assert!(matches!(
            modern_route_shape(&source(0x40, 0, 2, 0x4a)),
            Some(CellShape::FacadeBand {
                plane_subtile_row: 3,
                band_count: 1,
                solid: SolidKind::Fence,
                ..
            })
        ));
    }
}
