//! Exact grouped boulder and entrance-statue drawings for Olivine Gym.

use crystal_render_api::VisualTileSource;

pub(crate) const GROUND_TILE: u16 = 0x53;

pub(crate) fn statue_local(map_id: &str, source: &VisualTileSource) -> Option<(u8, u8)> {
    if map_id != "OlivineGym" || source.tileset_id.as_ref() != "champions_room"
        || source.subtile_column >= 4 || source.subtile_row >= 4 {
        return None;
    }
    let local_column = match source.metatile_id {
        0x1a if source.subtile_column < 2 => source.subtile_column,
        0x1b if source.subtile_column >= 2 => source.subtile_column - 2,
        _ => return None,
    };
    let expected = [[0x2e, 0x2f], [0x3e, 0x3f], [0x4e, 0x4f], [0x5e, 0x5f]]
        [usize::from(source.subtile_row)][usize::from(local_column)];
    (source.tile_index == expected).then_some((local_column, source.subtile_row))
}

pub(crate) fn boulder_local(source: &VisualTileSource) -> Option<(u8, u8)> {
    if source.tileset_id.as_ref() != "champions_room"
        || source.subtile_column >= 4 || source.subtile_row >= 4
    {
        return None;
    }
    let mask:u16=match source.metatile_id {
        0x11=>0x33ff,0x13=>0xccff,0x16=>0xff00,
        0x18|0x26=>0x3333,0x19|0x25=>0xcccc,0x2a=>0xffff,
        _=>return None,
    };
    if mask & (1<<(source.subtile_row*4+source.subtile_column))==0 { return None; }
    let local_column = source.subtile_column % 2;
    let local_row = source.subtile_row % 2;
    let expected = match (local_column, local_row) {
        (0, 0) => 0x46,
        (1, 0) => 0x47,
        (0, 1) => 0x56,
        (1, 1) => 0x57,
        _ => unreachable!(),
    };
    (source.tile_index == expected).then_some((local_column, local_row))
}

/// Complete pink perimeter strips; the terminal blocks end in a separate cap row.
pub(crate) fn perimeter_local(s: &VisualTileSource) -> Option<(u8,u8,usize,usize)> {
    if s.tileset_id.as_ref() != "champions_room" || s.subtile_column>=4 || s.subtile_row>=4 {
        return None;
    }
    let x=match s.metatile_id {
        0x24|0x25 if s.subtile_column<2=>s.subtile_column,
        0x26|0x27 if s.subtile_column>=2=>s.subtile_column-2,
        _=>return None,
    };
    let tile=if matches!(s.metatile_id,0x24|0x27) && s.subtile_row==3 {
        [0x1a,0x1b][x as usize]
    } else {[0x0a,0x0b][x as usize]};
    (s.tile_index==tile).then_some((x,s.subtile_row,2,4))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    fn source(block: u16, column: u8, row: u8, tile: u16) -> VisualTileSource {
        VisualTileSource {
            tileset_id: Arc::from("champions_room"),
            metatile_id: block,
            subtile_column: column,
            subtile_row: row,
            tile_index: tile,
        }
    }

    #[test]
    fn olivine_perimeter_requires_exact_strip_and_terminal_phase() {
        for (block,offset) in [(0x24,0),(0x25,0),(0x26,2),(0x27,2)] {
            for y in 0..4 {for x in 0..2 {
                let tile=if matches!(block,0x24|0x27) && y==3 {0x1a+x as u16} else {0x0a+x as u16};
                let mut s=source(block,x+offset,y,tile);
                assert_eq!(perimeter_local(&s),Some((x,y,2,4)));
                s.tile_index^=0x80;assert_eq!(perimeter_local(&s),None);
            }}
            for (x,y) in [(4,0),(255,0),(0,4),(0,255)] {
                assert_eq!(perimeter_local(&source(block,x,y,0x0a)),None);
            }
        }
        assert_eq!(perimeter_local(&source(0x25,2,0,0x0a)),None);
        assert_eq!(perimeter_local(&source(0x26,0,0,0x0a)),None);
        assert_eq!(perimeter_local(&source(0x23,0,0,0x0a)),None);
    }
    #[test]
    fn source_phase_rejects_rocks_in_native_floor_and_invalid_statue_rows() {
        for (x,y,tile) in [(2,2,0x46),(3,2,0x47),(2,3,0x56),(3,3,0x57)] {
            assert_eq!(boulder_local(&source(0x11,x,y,tile)),None);
        }
        assert_eq!(boulder_local(&source(0x25,0,0,0x46)),None,"pillar half is not a rock plot");
        assert_eq!(boulder_local(&source(0x11,4,0,0x46)),None);
        assert_eq!(statue_local("OlivineGym",&source(0x1a,0,4,0x2e)),None);
    }

    #[test]
    fn only_complete_native_boulder_cells_are_grouped() {
        assert_eq!(boulder_local(&source(0x11, 0, 0, 0x46)), Some((0, 0)));
        assert_eq!(boulder_local(&source(0x11, 1, 0, 0x47)), Some((1, 0)));
        assert_eq!(boulder_local(&source(0x11, 0, 1, 0x56)), Some((0, 1)));
        assert_eq!(boulder_local(&source(0x11, 1, 1, 0x57)), Some((1, 1)));
        assert_eq!(boulder_local(&source(0x11, 0, 0, 0x53)), None);
        assert_eq!(boulder_local(&source(0x10, 0, 0, 0x46)), None);
    }

    #[test]
    fn split_entrance_blocks_resolve_complete_two_by_four_statues() {
        let drawing = [[0x2e, 0x2f], [0x3e, 0x3f], [0x4e, 0x4f], [0x5e, 0x5f]];
        for (block, offset) in [(0x1a, 0), (0x1b, 2)] {
            for row in 0..4 {
                for column in 0..2 {
                    let cell = source(
                        block,
                        column + offset,
                        row,
                        drawing[row as usize][column as usize],
                    );
                    assert_eq!(statue_local("OlivineGym", &cell), Some((column, row)));
                    assert_eq!(statue_local("ChampionsRoom", &cell), None);
                }
            }
        }
    }
}
