//! Authored National Park fountain presentation.

use std::collections::BTreeSet;

use crystal_render_api::{VisualTile, VisualTileSource};

use crate::profile::CellShape;

const PARK_TILESET: &str = "park";
const FOUNTAIN_METATILE: u16 = 0x3f;
const PARK_WATER_TILE: u16 = 0x14;
// The inward tree transition block `$27` supplies the green ground underneath
// the park's outer `$06` tree mass. Selecting a tile from plaza block `$01`
// uses the same graphics under a pale palette and creates visible holes.
pub(crate) const PARK_TREE_GROUND_METATILE: u16 = 0x27;
pub(crate) const PARK_TREE_GROUND_TILE: u16 = 0x01;
const LARGE_TREE_METATILES: [u16; 2] = [0x00, 0x06];
const BENCH_METATILE: u16 = 0x0e;
pub(crate) const PARK_PLAZA_GROUND_TILE: u16 = 0x00;

/// Exposed masonry in the native plaza and pool-border drawings. Props,
/// spray and decorative tile reuse do not inherit the paving material.
pub(crate) fn masonry_ground(source: &VisualTileSource) -> bool {
    if source.tileset_id.as_ref() != PARK_TILESET
        || source.subtile_column >= 4 || source.subtile_row >= 4 {
        return false;
    }
    let masks = match source.metatile_id {
        0x01 => [0xa5a5, 0x5050, 0x0a0a, 0],
        0x08 => [0x0001, 0, 0, 0x113e],
        0x09 => [0x0008, 0, 0, 0x88c7],
        0x0a => [0x00a5, 0x0050, 0x000a, 0],
        0x0b => [0x2121, 0x1010, 0x0202, 0],
        0x0c => [0x1000, 0, 0, 0xe311],
        0x0d => [0x8000, 0, 0, 0x7c88],
        0x0e => [0xa000, 0x5000, 0, 0],
        0x0f | 0x15 => [0xa584, 0x5040, 0x0a08, 0],
        0x17 => [0x21a5, 0x1050, 0x020a, 0],
        0x2f => [0xa501, 0x5010, 0x0a00, 0],
        0x34 => [0, 0, 0, 0x0001],
        0x35 => [0, 0, 0, 0x0008],
        0x36 => [0, 0, 0, 0x1000],
        0x37 => [0, 0, 0, 0x8000],
        0x38 => [0, 0, 0, 0x1001],
        0x39 => [0, 0, 0, 0x8008],
        0x3a => [0, 0, 0, 0xf11f],
        0x3b => [0, 0, 0, 0xf88f],
        _ => return false,
    };
    let mask = match source.tile_index {
        0x00 => masks[0], 0x06 => masks[1],
        0x16 => masks[2], 0x15 => masks[3],
        _ => return false,
    };
    mask & (1 << (source.subtile_row * 4 + source.subtile_column)) != 0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FountainPlacement {
    pub column: usize,
    pub row: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HoopFenceKind {
    Light,
    Dark,
}

pub(crate) fn fountain_placements(
    cells: &[&VisualTile],
    width: usize,
    height: usize,
) -> Vec<FountainPlacement> {
    let mut origins = BTreeSet::new();
    for tile in cells {
        if tile.source.tileset_id.as_ref() != PARK_TILESET
            || tile.source.metatile_id != FOUNTAIN_METATILE
        {
            continue;
        }
        let column = tile.column as isize - tile.source.subtile_column as isize;
        let row = tile.row as isize - tile.source.subtile_row as isize;
        if column < 0 || row < 0 || column as usize + 4 > width || row as usize + 4 > height {
            continue;
        }
        let complete = (0..4).all(|local_row| {
            (0..4).all(|local_column| {
                let cell =
                    cells[(row as usize + local_row) * width + column as usize + local_column];
                cell.source.tileset_id.as_ref() == PARK_TILESET
                    && cell.source.metatile_id == FOUNTAIN_METATILE
                    && cell.source.subtile_column as usize == local_column
                    && cell.source.subtile_row as usize == local_row
            })
        });
        if complete {
            origins.insert(FountainPlacement {
                column: column as usize,
                row: row as usize,
            });
        }
    }
    origins.into_iter().collect()
}

/// Park uses the same animated `$14` water identity as the outdoor tilesets,
/// but owns a separate atlas. Recess that exact identity and leave every
/// non-water cell unchanged; the animated fountain requires a grouped mask
/// and must not be guessed by the generic per-tile relief path.
pub(crate) fn park_shape(source: &VisualTileSource) -> Option<CellShape> {
    if source.tileset_id.as_ref() != PARK_TILESET {
        return None;
    }
    (source.tile_index == PARK_WATER_TILE).then_some(CellShape::Water)
}

/// Native blocks `$00` (the map border) and `$06` have the same complete 32x32
/// tree drawing. Both need one modeled owner; retaining the border as flat art
/// leaves a bright pixel band behind the mature interior trees at low angles.
pub(crate) fn large_tree_local(source: &VisualTileSource) -> Option<(u8, u8)> {
    if source.tileset_id.as_ref() != PARK_TILESET
        || !LARGE_TREE_METATILES.contains(&source.metatile_id)
        || source.subtile_column >= 4
        || source.subtile_row >= 4
    {
        return None;
    }
    let expected = u16::from(source.subtile_row) * 0x10 + 0x0c + u16::from(source.subtile_column);
    (source.tile_index == expected).then_some((source.subtile_column, source.subtile_row))
}

/// The pale arches and gold posts enclosing the park trees are fences, not
/// clipped foliage. North and south drawings reverse the overlap of the posts
/// and hoop. Each is one complete 16x16 drawing on native lawn backing.
pub(crate) fn hoop_fence_local(source: &VisualTileSource) -> Option<(HoopFenceKind, u8, u8)> {
    if source.tileset_id.as_ref() != PARK_TILESET {
        return None;
    }
    match source.tile_index {
        0x25 => Some((HoopFenceKind::Light, 0, 0)),
        0x26 => Some((HoopFenceKind::Light, 1, 0)),
        0x35 => Some((HoopFenceKind::Light, 0, 1)),
        0x36 => Some((HoopFenceKind::Light, 1, 1)),
        0x23 => Some((HoopFenceKind::Dark, 0, 0)),
        0x24 => Some((HoopFenceKind::Dark, 1, 0)),
        0x33 => Some((HoopFenceKind::Dark, 0, 1)),
        0x34 => Some((HoopFenceKind::Dark, 1, 1)),
        _ => None,
    }
}

/// Block `$0e` contains a complete 32x24 bench drawing: two slat rows and
/// the lower feet/shadow row. Its final row is ordinary plaza backing.
pub(crate) fn bench_local(source: &VisualTileSource) -> Option<(u8, u8)> {
    if source.tileset_id.as_ref() != PARK_TILESET
        || source.metatile_id != BENCH_METATILE
        || source.subtile_row >= 3
        || source.subtile_column >= 4
    {
        return None;
    }
    let expected = 0x07 + u16::from(source.subtile_row) * 0x10 + u16::from(source.subtile_column);
    (source.tile_index == expected).then_some((source.subtile_column, source.subtile_row))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bevy::prelude::{Handle, Image};
    use crystal_render_api::{VisualTile, VisualTileSource};

    use super::*;

    fn source(metatile_id: u16, tile_index: u16) -> VisualTileSource {
        VisualTileSource {
            tileset_id: Arc::from(PARK_TILESET),
            metatile_id,
            subtile_column: 0,
            subtile_row: 0,
            tile_index,
        }
    }

    #[test]
    fn plaza_and_pool_masonry_exclude_props_and_water() {
        for (block, drawing) in [
            (0x01, [0,0x16,0,0x16, 6,0,6,0, 0,0x16,0,0x16, 6,0,6,0]),
            (0x08, [0,0x15,0x15,0x15, 0x15,0x15,0x14,0x14,
                0x15,0x14,0x14,0x14, 0x15,0x14,0x14,0x14]),
            (0x0f, [0x5a,0x5b,0,0x16, 0x13,0x82,6,0,
                0,0x16,0,0x16, 6,0,6,0]),
        ] {
            for (i, tile) in drawing.into_iter().enumerate() {
                let mut s = source(block, tile);
                s.subtile_column = (i % 4) as u8;
                s.subtile_row = (i / 4) as u8;
                assert_eq!(masonry_ground(&s), matches!(tile, 0|6|0x16|0x15));
                s.tile_index = 0x14;
                assert!(!masonry_ground(&s));
                s.tileset_id = Arc::from("johto");
                s.tile_index = tile;
                assert!(!masonry_ground(&s));
            }
        }
        assert!(!masonry_ground(&source(0x06, 0)));
        assert!(!masonry_ground(&source(0x3f, 0x5f)));
        let mut misplaced = source(0x01, 6);
        assert!(!masonry_ground(&misplaced));
        misplaced.subtile_row = 4;
        assert!(!masonry_ground(&misplaced));
    }

    #[test]
    fn fountain_water_recesses_without_extruding_animation_cells() {
        assert_eq!(
            park_shape(&source(0x3f, PARK_WATER_TILE)),
            Some(CellShape::Water)
        );
        assert_eq!(park_shape(&source(0x3f, 0x5f)), None);
        assert_eq!(park_shape(&source(0x3f, 0x80)), None);
    }

    #[test]
    fn ordinary_park_water_is_recessed_but_other_art_is_not_guessed() {
        assert_eq!(
            park_shape(&source(0x31, PARK_WATER_TILE)),
            Some(CellShape::Water)
        );
        assert_eq!(park_shape(&source(0x31, 0x41)), None);
    }

    #[test]
    fn complete_fountain_metatile_produces_one_grouped_placement() {
        let tiles = (0..16)
            .map(|index| VisualTile {
                animation_frames: None,
                column: (index % 4) as u32,
                row: (index / 4) as u32,
                source: VisualTileSource {
                    tileset_id: Arc::from(PARK_TILESET),
                    metatile_id: FOUNTAIN_METATILE,
                    subtile_column: (index % 4) as u8,
                    subtile_row: (index / 4) as u8,
                    tile_index: PARK_WATER_TILE,
                },
                texture: Handle::<Image>::Uuid(bevy::asset::uuid::Uuid::from_u128(index as u128 + 1), std::marker::PhantomData),
                priority: false,
            })
            .collect::<Vec<_>>();
        let ordered = tiles.iter().collect::<Vec<_>>();
        assert_eq!(
            fountain_placements(&ordered, 4, 4),
            vec![FountainPlacement { column: 0, row: 0 }]
        );
        assert!(fountain_placements(&ordered[..12], 4, 3).is_empty());
    }

    #[test]
    fn large_park_tree_is_one_exact_four_by_four_drawing() {
        for metatile in [0x00, 0x06] {
            for row in 0..4 {
                for column in 0..4 {
                    let mut tile = source(metatile, row * 0x10 + 0x0c + column);
                    tile.subtile_column = column as u8;
                    tile.subtile_row = row as u8;
                    assert_eq!(large_tree_local(&tile), Some((column as u8, row as u8)));
                }
            }
        }
        assert_eq!(large_tree_local(&source(0x00, 0x00)), None);
        assert_eq!(large_tree_local(&source(0x06, 0x00)), None);
        assert_eq!(large_tree_local(&source(0x05, 0x0c)), None);
    }

    #[test]
    fn park_hoop_fences_are_two_complete_two_by_two_drawings() {
        for (kind, drawing) in [
            (HoopFenceKind::Light, [0x25, 0x26, 0x35, 0x36]),
            (HoopFenceKind::Dark, [0x23, 0x24, 0x33, 0x34]),
        ] {
            for (tile, local) in drawing.into_iter().zip([(0, 0), (1, 0), (0, 1), (1, 1)]) {
                assert_eq!(
                    hoop_fence_local(&source(0x11, tile)),
                    Some((kind, local.0, local.1))
                );
            }
        }
        assert_eq!(hoop_fence_local(&source(0x11, 0x05)), None);
        let mut johto = source(0x11, 0x25);
        johto.tileset_id = Arc::from("johto");
        assert_eq!(hoop_fence_local(&johto), None);
    }

    #[test]
    fn park_bench_includes_its_feet_but_excludes_plaza_backing() {
        for row in 0..3 {
            for column in 0..4 {
                let mut tile = source(0x0e, 0x07 + row * 0x10 + column);
                tile.subtile_column = column as u8;
                tile.subtile_row = row as u8;
                assert_eq!(bench_local(&tile), Some((column as u8, row as u8)));
            }
        }
        let mut floor = source(0x0e, 0x06);
        floor.subtile_row = 3;
        assert_eq!(bench_local(&floor), None);
        assert_eq!(bench_local(&source(0x0f, 0x07)), None);
    }
}
