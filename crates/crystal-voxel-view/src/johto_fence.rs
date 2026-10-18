//! Reusable fence drawings from Crystal's base Johto outdoor atlas.

use crystal_render_api::VisualTileSource;

use crate::profile::{CellShape, SolidKind};

const TILESET: &str = "johto";
const GROUND_TILE: u16 = 0x06;

/// Fold the exact horizontal rail courses, including corner blocks. Vertical
/// stripes are owned by the grouped north/south rail mesher.
pub(crate) fn johto_fence_shape(source: &VisualTileSource) -> Option<CellShape> {
    let first_row = if source.tileset_id.as_ref()=="battle_tower_outside" {
        match source.metatile_id {
            0x20 => 2,
            0x2d | 0x2f if (2..4).contains(&source.subtile_column) => 2,
            0x0d if source.subtile_column<2 => 2,
            _=>return None,
        }
    } else if source.tileset_id.as_ref()==TILESET {match source.metatile_id {
        0x40 | 0x41 | 0x42 => 0,
        0x48 | 0x49 | 0x4a => 2,
        // Port approach: two outer rail pairs flank the two native warps.
        0x0c if source.subtile_column < 2 => 2,
        0x0d if (2..4).contains(&source.subtile_column) => 2,
        _ => return None,
    }} else {return None;};
    let band_from_top = match (source.subtile_row.checked_sub(first_row)?, source.tile_index) {
        (0, 0x5a) => 0,
        (1, 0x59) => 1,
        _ => return None,
    };
    Some(CellShape::FacadeBand {
        plane_subtile_row: first_row + 2,
        band_from_top,
        band_count: 2,
        ground_tile_index: GROUND_TILE,
        solid: SolidKind::Fence,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn tower_rails_share_the_fence_bands_and_leave_the_warp_channels_clear() {
        for (block,cols) in [(0x2d,2..4),(0x0d,0..2),(0x2f,2..4),(0x20,0..4)] {
            for col in 0..4 {for (row,tile) in [(2,0x5a),(3,0x59)] {
                let mut s=source(block,row,tile);s.tileset_id="battle_tower_outside".into();s.subtile_column=col;
                assert_eq!(johto_fence_shape(&s).is_some(),cols.contains(&col));
                s.tile_index^=1;assert!(johto_fence_shape(&s).is_none());
            }}
        }
        for col in 0..4 {
            let mut s=source(0x0c,2,0x40);s.tileset_id="battle_tower_outside".into();s.subtile_column=col;
            assert!(johto_fence_shape(&s).is_none());
        }
    }

    fn source(metatile: u16, row: u8, tile: u16) -> VisualTileSource {
        VisualTileSource {
            tileset_id: Arc::from(TILESET),
            metatile_id: metatile,
            subtile_column: 0,
            subtile_row: row,
            tile_index: tile,
        }
    }

    #[test]
    fn port_rails_require_their_outer_columns_and_native_courses() {
        for block in [0x0c, 0x0d] {
            for column in 0..4 {
                for (row,tile) in [(2,0x5a),(3,0x59)] {
                    let mut s = source(block,row,tile);
                    s.subtile_column = column;
                    let owns = (block==0x0c && column<2) || (block==0x0d && column>=2);
                    assert_eq!(johto_fence_shape(&s).is_some(),owns);
                    s.tile_index ^= 1;
                    assert_eq!(johto_fence_shape(&s),None);
                    s.tile_index = tile;
                    s.subtile_row = 1;
                    assert_eq!(johto_fence_shape(&s),None);
                    s.subtile_row = row;
                    s.tileset_id = Arc::from("johto_modern");
                    assert_eq!(johto_fence_shape(&s),None);
                }
            }
        }
    }

    #[test]
    fn block_49_folds_only_its_two_native_fence_courses() {
        assert_eq!(
            johto_fence_shape(&source(0x49, 2, 0x5a)),
            Some(CellShape::FacadeBand {
                plane_subtile_row: 4,
                band_from_top: 0,
                band_count: 2,
                ground_tile_index: GROUND_TILE,
                solid: SolidKind::Fence,
            })
        );
        assert_eq!(
            johto_fence_shape(&source(0x49, 3, 0x59)),
            Some(CellShape::FacadeBand {
                plane_subtile_row: 4,
                band_from_top: 1,
                band_count: 2,
                ground_tile_index: GROUND_TILE,
                solid: SolidKind::Fence,
            })
        );
        assert_eq!(johto_fence_shape(&source(0x49, 1, 0x06)), None);
        assert_eq!(johto_fence_shape(&source(0x48, 2, 0x5a)),
            johto_fence_shape(&source(0x49, 2, 0x5a)));
    }
}
