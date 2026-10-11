//! Map-scoped wall art for Vermilion Gym.
//!
//! The Gym shares the Game Corner atlas. Complete bins and entrance guardians
//! have separate authored model selectors; this module keeps their flat baseline
//! and retains the north-panel baseline beneath separate complete model selectors.

use crystal_render_api::VisualTileSource;

use crate::profile::{CellShape, SolidKind};

pub(crate) const FLOOR_TILE: u16 = 0x01;

pub(crate) fn panel_local(source: &VisualTileSource) -> Option<(u8, u8, usize, usize)> {
    if source.tileset_id.as_ref() != "game_corner"
        || source.subtile_column >= 4
        || source.subtile_row >= 4
    {
        return None;
    }
    let right = match source.metatile_id {
        0x20 => false,
        0x21 => true,
        _ => return None,
    };
    let art = if right {
        [
            [0x49, 0x49, 0x49, 0x4a],
            [0x59, 0x59, 0x59, 0x5a],
            [0x4d, 0x4d, 0x4d, 0x5d],
            [0x5b, 0x5b, 0x5b, 0x5f],
        ]
    } else {
        [
            [0x48, 0x49, 0x49, 0x49],
            [0x58, 0x59, 0x59, 0x59],
            [0x5c, 0x4d, 0x4d, 0x4d],
            [0x5e, 0x5b, 0x5b, 0x5b],
        ]
    };
    (source.tile_index == art[source.subtile_row as usize][source.subtile_column as usize])
        .then_some((
            source.subtile_column + if right { 4 } else { 0 },
            source.subtile_row,
            8,
            4,
        ))
}

pub(crate) fn shape(map_id: &str, source: &VisualTileSource) -> Option<CellShape> {
    if map_id != "VermilionGym" || source.tileset_id.as_ref() != "game_corner" {
        return None;
    }
    // $1d and $1e contain complete two-column, four-row entrance guardians.
    // Their floor-height baseline preserves footing; the grouped selector
    // replaces only a complete, exact native drawing with authored geometry.
    let guardian = (source.metatile_id == 0x1d && source.subtile_column >= 2)
        || (source.metatile_id == 0x1e && source.subtile_column < 2);
    if guardian {
        return Some(CellShape::Flat);
    }
    // The complete $20/$21 pairs have separate framed-wall model selectors.
    // This baseline folds each original row once when that selection is rejected;
    // unknown or incomplete art must not earn authored-model coverage.
    if matches!(source.metatile_id, 0x20 | 0x21) {
        return Some(CellShape::FacadeBand {
            plane_subtile_row: 4,
            band_from_top: source.subtile_row,
            band_count: 4,
            ground_tile_index: FLOOR_TILE,
            solid: SolidKind::FlatCard,
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    fn source(metatile_id: u16, column: u8, row: u8) -> VisualTileSource {
        VisualTileSource {
            tileset_id: Arc::from("game_corner"),
            metatile_id,
            subtile_column: column,
            subtile_row: row,
            tile_index: 0,
        }
    }

    #[test]
    fn entrance_guardians_keep_a_flat_baseline_for_the_grouped_model() {
        for row in 0..4 {
            for column in 0..2 {
                assert_eq!(
                    shape("VermilionGym", &source(0x1d, column + 2, row)),
                    Some(CellShape::Flat)
                );
                assert_eq!(
                    shape("VermilionGym", &source(0x1e, column, row)),
                    Some(CellShape::Flat)
                );
            }
        }
    }

    #[test]
    fn shared_game_corner_atlas_does_not_claim_guardians_outside_the_gym() {
        assert_eq!(shape("GoldenrodGameCorner", &source(0x1d, 2, 0)), None);
    }

    #[test]
    fn north_panel_baseline_uses_each_native_row_once_on_a_flat_plane() {
        for row in 0..4 {
            assert_eq!(
                shape("VermilionGym", &source(0x20, 0, row)),
                Some(CellShape::FacadeBand {
                    plane_subtile_row: 4,
                    band_from_top: row,
                    band_count: 4,
                    ground_tile_index: FLOOR_TILE,
                    solid: SolidKind::FlatCard,
                })
            );
        }
    }
}
