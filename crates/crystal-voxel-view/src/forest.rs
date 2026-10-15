//! Presentation-only shape identities for Crystal's Forest tileset.

use crystal_render_api::VisualTileSource;

use crate::profile::{CellShape, LedgeFace, SolidKind, JUMP_LEDGE_HEIGHT};

const FOREST_TILESET: &str = "forest";
const FOREST_GROUND_TILE: u16 = 0x05;

/// Exact dry samples of the forest pond rim verified against the local pack.
/// The water and unknown art retain their original roles. Only presentation
/// changes: the bank top stays at zero and its lip uses the existing water drop.
pub(crate) fn forest_shore_shape(source: &VisualTileSource) -> Option<CellShape> {
    if source.tileset_id.as_ref() != FOREST_TILESET
        || source.subtile_column >= 4
        || source.subtile_row >= 4
    {
        return None;
    }
    let tile = match (
        source.metatile_id,
        source.subtile_column,
        source.subtile_row,
    ) {
        (0x03, _, 0) | (0x07, 1..=3, 0) | (0x0d, 0..=2, 0) => 0x2a,
        (0x07, 0, 0) => 0x09,
        (0x07, 0, 1..=3) | (0x0a, 0, _) => 0x1b,
        (0x0b, 3, _) | (0x0d, 3, 1..=3) => 0x19,
        (0x0d, 3, 0) => 0x0b,
        _ => return None,
    };
    (source.tile_index == tile).then_some(CellShape::ShoreBand)
}

/// Resolve the connected `$18-$1c` Forest jump-ledge drawing.
///
/// The U-shaped drawing has authored west/east side courses and one south
/// lip. Cells enclosed by those courses are its raised cap. This is visual
/// source classification only; gameplay collision and hop permissions remain
/// authoritative in crystal-core.
pub(crate) fn forest_ledge_shape(source: &VisualTileSource) -> Option<CellShape> {
    if source.tileset_id.as_ref() != FOREST_TILESET {
        return None;
    }
    let raised = || CellShape::RaisedTop {
        height: JUMP_LEDGE_HEIGHT,
        solid: SolidKind::Bank,
    };
    let band = |face, plane_subtile| CellShape::LedgeBand {
        face,
        plane_subtile,
        band_from_top: 0,
        band_count: 1,
        top_tile_index: FOREST_GROUND_TILE,
        height: JUMP_LEDGE_HEIGHT,
    };
    match source.metatile_id {
        0x18 => Some(if source.subtile_column == 0 {
            band(LedgeFace::West, 0)
        } else {
            raised()
        }),
        0x19 => Some(if source.subtile_row == 3 {
            band(LedgeFace::South, 4)
        } else if source.subtile_column == 0 {
            band(LedgeFace::West, 0)
        } else {
            raised()
        }),
        0x1a => Some(if source.subtile_row == 3 {
            band(LedgeFace::South, 4)
        } else {
            raised()
        }),
        0x1b => Some(if source.subtile_row == 3 {
            band(LedgeFace::South, 4)
        } else if source.subtile_column == 3 {
            band(LedgeFace::East, 4)
        } else {
            raised()
        }),
        0x1c if source.subtile_column < 2 => Some(if source.subtile_row == 3 {
            band(LedgeFace::South, 4)
        } else {
            raised()
        }),
        0x1c if source.subtile_row < 2 => Some(raised()),
        0x1c => {
            // The opening occupies only the south half of this metatile.
            // Its north half is the upper plateau; two 8px source rows make
            // the short transition down to the lower path.
            let step = JUMP_LEDGE_HEIGHT / 2.0;
            let ramp_row = source.subtile_row - 2;
            let north_height = JUMP_LEDGE_HEIGHT - f32::from(ramp_row) * step;
            Some(CellShape::RampNorth {
                north_height,
                south_height: north_height - step,
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    fn source(metatile_id: u16, column: u8, row: u8) -> VisualTileSource {
        VisualTileSource {
            tileset_id: Arc::from(FOREST_TILESET),
            metatile_id,
            subtile_column: column,
            subtile_row: row,
            tile_index: FOREST_GROUND_TILE,
        }
    }

    #[test]
    fn pond_rim_requires_exact_source_samples_and_keeps_water_unclaimed() {
        for (block, x, y, tile) in [
            (0x03, 2, 0, 0x2a),
            (0x07, 0, 0, 0x09),
            (0x07, 0, 2, 0x1b),
            (0x0a, 0, 3, 0x1b),
            (0x0b, 3, 1, 0x19),
            (0x0d, 3, 0, 0x0b),
            (0x0d, 2, 0, 0x2a),
        ] {
            let mut s = source(block, x, y);
            s.tile_index = tile;
            assert_eq!(forest_shore_shape(&s), Some(CellShape::ShoreBand));
            assert_eq!(crate::profile::shape_for_source(&s), CellShape::ShoreBand);
            s.tile_index = 0x14;
            assert_eq!(forest_shore_shape(&s), None);
            assert_eq!(crate::profile::shape_for_source(&s), CellShape::Water);
            s.tile_index = tile;
            s.tileset_id = "johto".into();
            assert_eq!(forest_shore_shape(&s), None);
        }
        let mut s = source(0x03, 2, 1);
        s.tile_index = 0x2a;
        assert_eq!(
            forest_shore_shape(&s),
            None,
            "same byte outside the rim is untouched"
        );
    }

    #[test]
    fn forest_u_ledge_separates_cap_and_directional_courses() {
        assert!(matches!(
            forest_ledge_shape(&source(0x19, 1, 1)),
            Some(CellShape::RaisedTop { height, .. }) if height == JUMP_LEDGE_HEIGHT
        ));
        assert!(matches!(
            forest_ledge_shape(&source(0x19, 0, 1)),
            Some(CellShape::LedgeBand {
                face: LedgeFace::West,
                ..
            })
        ));
        assert!(matches!(
            forest_ledge_shape(&source(0x1a, 2, 3)),
            Some(CellShape::LedgeBand {
                face: LedgeFace::South,
                ..
            })
        ));
        assert!(matches!(
            forest_ledge_shape(&source(0x1b, 3, 1)),
            Some(CellShape::LedgeBand {
                face: LedgeFace::East,
                ..
            })
        ));
        assert_eq!(
            forest_ledge_shape(&source(0x1c, 2, 1)),
            Some(CellShape::RaisedTop {
                height: JUMP_LEDGE_HEIGHT,
                solid: SolidKind::Bank,
            })
        );
        assert_eq!(
            forest_ledge_shape(&source(0x1c, 2, 2)),
            Some(CellShape::RampNorth {
                north_height: JUMP_LEDGE_HEIGHT,
                south_height: JUMP_LEDGE_HEIGHT / 2.0,
            })
        );
    }
}
