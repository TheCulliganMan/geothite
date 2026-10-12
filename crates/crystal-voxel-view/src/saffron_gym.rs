//! Exact Saffron Gym wall and planter presentation roles.

use crystal_render_api::VisualTileSource;

use crate::profile::{CellShape, SolidKind};

pub(crate) const FLOOR_TILE: u16 = 0x10;

pub(crate) fn shape(map_id: &str, source: &VisualTileSource) -> Option<CellShape> {
    if map_id != "SaffronGym"
        || source.tileset_id.as_ref() != "underground"
        || source.metatile_id != 0x09
        || source.subtile_column >= 4
        || source.subtile_row >= 2
    {
        return None;
    }
    Some(CellShape::FacadeBand {
        plane_subtile_row: 2,
        band_from_top: source.subtile_row,
        band_count: 2,
        ground_tile_index: FLOOR_TILE,
        solid: SolidKind::FlatCard,
    })
}

/// Exact ordinary paving cells; warp panels, carpet and planter art are excluded.
pub(crate) fn floor_source(s: &VisualTileSource) -> bool {
    if s.tileset_id.as_ref() != "underground" || s.tile_index != FLOOR_TILE
        || s.subtile_column>=4 || s.subtile_row>=4 {return false;}
    let mask:u16=match s.metatile_id {
        0x08=>0xcc00,0x09=>0xff00,0x0a=>0x3300,0x0c|0x36=>0xcccc,
        0x0d=>0xffff,0x0e=>0x3333,0x0f=>0x33ff,0x24=>0x00ff,
        _=>return false,
    };
    mask & (1<<(s.subtile_row*4+s.subtile_column)) != 0
}

/// Complete horizontal wall module, distinct from corridor junctions.
pub(crate) fn north_wall_local(s: &VisualTileSource) -> Option<(u8,u8,usize,usize)> {
    if s.tileset_id.as_ref() != "underground" || s.metatile_id != 0x09
        || s.subtile_column >= 4 || s.subtile_row >= 2 { return None; }
    (s.tile_index == [0x04,0x14][s.subtile_row as usize])
        .then_some((s.subtile_column,s.subtile_row,4,2))
}

pub(crate) fn planter_local(source: &VisualTileSource) -> Option<(u8, u8)> {
    if source.tileset_id.as_ref() != "underground"
        || source.metatile_id != 0x36
        || source.subtile_column >= 2
        || source.subtile_row >= 4
    {
        return None;
    }
    Some((source.subtile_column, source.subtile_row))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    fn source(block: u16, column: u8, row: u8) -> VisualTileSource {
        VisualTileSource {
            tileset_id: Arc::from("underground"),
            metatile_id: block,
            subtile_column: column,
            subtile_row: row,
            tile_index: 0,
        }
    }

    #[test]
    fn saffron_floor_masks_keep_only_native_diamonds() {
        for (block,mask) in [(0x08,0xcc00u16),(0x09,0xff00),(0x0a,0x3300),
            (0x0c,0xcccc),(0x0d,0xffff),(0x0e,0x3333),(0x0f,0x33ff),(0x24,0x00ff),(0x36,0xcccc)] {
            for y in 0..4 {for x in 0..4 {
                let mut s=source(block,x,y);s.tile_index=FLOOR_TILE;
                assert_eq!(floor_source(&s),mask&(1<<(y*4+x))!=0);
                s.tile_index^=0x80;assert!(!floor_source(&s));
            }}
        }
        for (x,y) in [(4,0),(255,0),(0,4),(0,255)] {
            let mut s=source(0x0d,x,y);s.tile_index=FLOOR_TILE;assert!(!floor_source(&s));
        }
        let mut s=source(0,0,0);s.tile_index=FLOOR_TILE;assert!(!floor_source(&s));
        s.metatile_id=0x0d;s.tileset_id="mart".into();assert!(!floor_source(&s));
    }
    #[test]
    fn straight_wall_is_exactly_two_native_rows() {
        for row in 0..2 {
            assert!(matches!(
                shape("SaffronGym", &source(0x09, 0, row)),
                Some(CellShape::FacadeBand {
                    band_count: 2,
                    solid: SolidKind::FlatCard,
                    ..
                })
            ));
        }
        assert_eq!(shape("SaffronGym", &source(0x09, 0, 2)), None);
        assert_eq!(shape("SilphCo1F", &source(0x09, 0, 0)), None);
    }

    #[test]
    fn saffron_blue_band_requires_each_exact_native_wall_phase() {
        for row in 0..2 { for column in 0..4 {
            let mut s=source(0x09,column,row);s.tile_index=[0x04,0x14][row as usize];
            assert_eq!(north_wall_local(&s),Some((column,row,4,2)));
            s.tile_index^=1;assert_eq!(north_wall_local(&s),None);
        }}
        for (column,row) in [(4,0),(255,0),(0,2),(0,255)] {
            let mut s=source(0x09,column,row);s.tile_index=0x04;
            assert_eq!(north_wall_local(&s),None);
        }
        let mut s=source(0x08,0,0);s.tile_index=0x04;
        assert_eq!(north_wall_local(&s),None);
    }
    #[test]
    fn malformed_saffron_phases_cannot_claim_wall_or_statue_cells() {
        for column in [4,127,255] {
            assert_eq!(shape("SaffronGym", &source(0x09,column,0)),None);
            assert_eq!(planter_local(&source(0x36,column,0)),None);
        }
        for row in [4,127,255] {
            assert_eq!(planter_local(&source(0x36,0,row)),None);
        }
    }
    #[test]
    fn planter_keeps_one_complete_two_by_four_drawing() {
        for row in 0..4 {
            for column in 0..2 {
                assert_eq!(
                    planter_local(&source(0x36, column, row)),
                    Some((column, row))
                );
            }
        }
        assert_eq!(planter_local(&source(0x36, 2, 0)), None);
    }
}
