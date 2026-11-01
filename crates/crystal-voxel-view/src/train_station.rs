//! Authored, presentation-only fixtures from Crystal's Magnet Train stations.

use crystal_render_api::VisualTileSource;

pub(crate) const FLOOR_TILE: u16 = 0x3d;

pub(crate) fn supports_map(map_id: &str) -> bool {
    matches!(
        map_id,
        "GoldenrodMagnetTrainStation"
            | "SaffronMagnetTrainStation"
            | "MnDetroitLakesStation"
            | "MnStCloudStation"
            | "MnStPaulUnionDepot"
            | "MnWinonaStation"
    )
}

/// Blocks $01 and $02 contain separate 16x16 waiting-room chairs in row
/// and column arrangements. Their plots remain independent of physical size.
pub(crate) fn seat_local(map_id: &str, source: &VisualTileSource) -> Option<(u8, u8)> {
    if !supports_map(map_id) || source.tileset_id.as_ref() != "train_station"
        || source.subtile_column>=4 || source.subtile_row>=4 {return None;}
    match source.metatile_id {
        0x01 if source.subtile_row<2=>{},
        0x02 if source.subtile_column<2=>{},
        _=>return None,
    }
    let x=source.subtile_column%2;let y=source.subtile_row%2;
    const DRAWING:[[u16;2];2]=[[0x40,0x41],[0x42,0x43]];
    (source.tile_index==DRAWING[y as usize][x as usize]).then_some((x,y))
}

/// The left half of $0c is one 16x32 station planter. Its repeated middle
/// foliage is part of the single authored drawing, not evidence for stacked
/// cubes, so retain it as one masked upright card over the station floor.
pub(crate) fn planter_local(map_id: &str, source: &VisualTileSource) -> Option<(u8, u8)> {
    if !supports_map(map_id)
        || source.tileset_id.as_ref() != "train_station"
        || source.metatile_id != 0x0c
        || source.subtile_column >= 2
    {
        return None;
    }
    const DRAWING: [[u16; 2]; 4] = [[0x50, 0x51], [0x52, 0x53], [0x52, 0x53], [0x54, 0x55]];
    (source.tile_index
        == DRAWING[usize::from(source.subtile_row)][usize::from(source.subtile_column)])
    .then_some((source.subtile_column, source.subtile_row))
}

/// Whole native planter variants, including the paired and shorter drawings
/// used by Saffron. Floor halves are never part of a plant claim.
pub(crate) fn planter_drawing_local(map: &str,s: &VisualTileSource)->Option<(u8,u8,usize,usize)> {
    if !supports_map(map) || s.tileset_id.as_ref()!="train_station"
        || s.subtile_column>=4 || s.subtile_row>=4 {return None;}
    let (origin,height)=match s.metatile_id {
        0x0c=>(0,4),0x0e=>(2,4),0x20=>(s.subtile_column/2*2,4),
        0x2f=>(s.subtile_column/2*2,2),0x33=>(0,3),_=>return None,
    };
    if s.subtile_column<origin || s.subtile_column>=origin+2 || s.subtile_row as usize>=height {return None;}
    let x=s.subtile_column-origin;let y=s.subtile_row;
    let expected=if y==0 {[0x50,0x51]}else if y as usize==height-1 {[0x54,0x55]}else {[0x52,0x53]};
    (s.tile_index==expected[x as usize]).then_some((x,y,2,height))
}

/// Goldenrod's four ticket boxes form one complete east-wall bank across
/// blocks $2c/$2d. Preserve the entire drawing, including its bottom cap.
pub(crate) fn ticket_box_local(map:&str,s:&VisualTileSource)->Option<(u8,u8,usize,usize)> {
    if map!="GoldenrodMagnetTrainStation" || s.tileset_id.as_ref()!="train_station"
        || !(2..=3).contains(&s.subtile_column) || s.subtile_row>=4 {return None;}
    let y=match s.metatile_id {0x2c=>s.subtile_row,0x2d=>s.subtile_row+4,_=>return None};
    let x=s.subtile_column-2;
    let expected=match y {6=>[0x46,0x82],7=>[0x47,0x83],_=>if y%2==0 {[0x44,0x80]}else{[0x45,0x81]}};
    (s.tile_index==expected[x as usize]).then_some((x,y,2,8))
}

pub(crate) fn wall_local(map:&str,s:&VisualTileSource)->Option<(u8,u8,usize,usize)> {
    if !supports_map(map) || s.tileset_id.as_ref()!="train_station"
        || s.subtile_column>=4 || s.subtile_row>=4 {return None;}
    let x=s.subtile_column;let y=s.subtile_row;
    let expected=match s.metatile_id {
        0x0a=>[0x37,0x38,0x30,0x30][x as usize],
        0x16=>0x30,
        0x0b=>if y==3 {0x33}else{0x3e},
        _=>return None,
    };
    (s.tile_index==expected).then_some((x,if s.metatile_id==0x0b {y+4}else{y},4,8))
}

pub(crate) fn railing_local(map:&str,s:&VisualTileSource)->Option<(u8,u8,usize,usize)> {
    if !supports_map(map) || s.tileset_id.as_ref()!="train_station" || !matches!(s.metatile_id,6|8)
        || s.subtile_column>=4 || !(2..=3).contains(&s.subtile_row) {return None;}
    if s.metatile_id==8 && s.subtile_column<2 {return None;}
    (s.tile_index==if s.subtile_row==2 {8}else{0x18})
        .then_some((s.subtile_column-if s.metatile_id==8 {2}else{0},s.subtile_row-2,
            if s.metatile_id==8 {2}else{4},2))
}

/// Blocks $07 and $08 each carry the same narrow 16x32 station gate on their
/// left half. Preserve each occurrence as its own thin face-on fixture; the
/// right halves are unrelated floor/rail art and must not be claimed.
pub(crate) fn gate_local(map_id: &str, source: &VisualTileSource) -> Option<(u8, u8)> {
    if !supports_map(map_id)
        || source.tileset_id.as_ref() != "train_station"
        || !matches!(source.metatile_id, 0x07 | 0x08)
        || source.subtile_column >= 2
    {
        return None;
    }
    const DRAWING: [[u16; 2]; 4] = [[0x35, 0x36], [0x37, 0x38], [0x39, 0x3a], [0x3b, 0x3c]];
    (source.tile_index
        == DRAWING[usize::from(source.subtile_row)][usize::from(source.subtile_column)])
    .then_some((source.subtile_column, source.subtile_row))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    fn source(
        map_tileset: &str,
        metatile: u16,
        column: u8,
        row: u8,
        tile: u16,
    ) -> VisualTileSource {
        VisualTileSource {
            tileset_id: Arc::from(map_tileset),
            metatile_id: metatile,
            subtile_column: column,
            subtile_row: row,
            tile_index: tile,
        }
    }

    #[test]
    fn each_half_of_block_one_is_one_complete_seat() {
        for half in 0..2 {
            for row in 0..2 {
                for column in 0..2 {
                    let drawing = [[0x40, 0x41], [0x42, 0x43]];
                    assert_eq!(
                        seat_local(
                            "GoldenrodMagnetTrainStation",
                            &source(
                                "train_station",
                                0x01,
                                half * 2 + column,
                                row,
                                drawing[usize::from(row)][usize::from(column)],
                            ),
                        ),
                        Some((column, row))
                    );
                }
            }
        }
    }

    #[test]
    fn block_two_keeps_two_independent_column_chairs_and_rejects_floor_half() {
        for chair in 0..2 {for y in 0..2 {for x in 0..2 {
            let tile=[[0x40,0x41],[0x42,0x43]][y][x];
            assert_eq!(seat_local("SaffronMagnetTrainStation",&source("train_station",2,x as u8,(chair*2+y)as u8,tile)),Some((x as u8,y as u8)));
        }}}
        assert_eq!(seat_local("SaffronMagnetTrainStation",&source("train_station",2,2,0,0x3d)),None);
    }

    #[test]
    fn seat_identity_is_map_tileset_and_art_scoped() {
        let seat = source("train_station", 0x01, 0, 0, 0x40);
        assert_eq!(seat_local("SaffronMagnetTrainStation", &seat), Some((0, 0)));
        assert_eq!(seat_local("GoldenrodCity", &seat), None);
        assert_eq!(
            seat_local(
                "GoldenrodMagnetTrainStation",
                &source("train_station", 0x03, 0, 0, 0x40),
            ),
            None
        );
        assert_eq!(
            seat_local(
                "GoldenrodMagnetTrainStation",
                &source("mart", 0x01, 0, 0, 0x40),
            ),
            None
        );
    }

    #[test]
    fn planter_keeps_one_complete_two_by_four_drawing() {
        let drawing = [[0x50, 0x51], [0x52, 0x53], [0x52, 0x53], [0x54, 0x55]];
        for row in 0..4 {
            for column in 0..2 {
                assert_eq!(
                    planter_local(
                        "GoldenrodMagnetTrainStation",
                        &source(
                            "train_station",
                            0x0c,
                            column,
                            row,
                            drawing[usize::from(row)][usize::from(column)],
                        ),
                    ),
                    Some((column, row))
                );
            }
        }
        assert_eq!(
            planter_local(
                "GoldenrodMagnetTrainStation",
                &source("train_station", 0x0c, 2, 0, 0x3d),
            ),
            None
        );
    }

    #[test]
    fn each_station_gate_is_one_scoped_two_by_four_card() {
        let drawing = [[0x35, 0x36], [0x37, 0x38], [0x39, 0x3a], [0x3b, 0x3c]];
        for metatile in [0x07, 0x08] {
            for row in 0..4 {
                for column in 0..2 {
                    assert_eq!(
                        gate_local(
                            "SaffronMagnetTrainStation",
                            &source(
                                "train_station",
                                metatile,
                                column,
                                row,
                                drawing[usize::from(row)][usize::from(column)],
                            ),
                        ),
                        Some((column, row))
                    );
                }
            }
        }
        assert_eq!(
            gate_local(
                "GoldenrodMagnetTrainStation",
                &source("train_station", 0x07, 2, 0, 0x3e),
            ),
            None
        );
    }
}
