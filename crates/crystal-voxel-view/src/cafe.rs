//! Map-scoped round café tables from Crystal's shared Game Corner atlas.

use crystal_render_api::VisualTileSource;

use crate::profile::{CellShape, SolidKind};

const FLOOR_TILE: u16 = 0x01;
pub(crate) const TABLETOP_BOTTOM: f32 = 14.0;
pub(crate) const TABLETOP_THICKNESS: f32 = 2.0;
pub(crate) const SERVICE_COUNTER_HEIGHT: f32 = 8.0;
/// Dining height above the floor, independent of the tall 2D table drawing.
pub(crate) const NATIVE_TABLE_HEIGHT: f32 = 8.0;

pub(crate) fn is_cafe_map(map_id: &str) -> bool {
    matches!(
        map_id,
        "CeladonCafe" | "OlivineCafe" | "SafariZoneMainOffice"
    )
}

/// Exact Celadon table drawing; an edited or cropped block cannot invent furniture.
pub(crate) fn celadon_table_local(map:&str,s:&VisualTileSource)->Option<(u8,u8)> {
    const ART:[u16;16]=[0x1c,0x20,0x21,0x1d,0x1c,0x30,0x31,0x1d,
        0x3e,0x50,0x50,0x3f,0xa8,0x4e,0x4f,0xa9];
    if map!="CeladonCafe" || s.tileset_id.as_ref()!="game_corner"
        || s.subtile_column>=4 || s.subtile_row>=4 {return None;}
    if matches!(s.metatile_id,0x2c|0x2d) && s.subtile_row==3
        && s.tile_index==[0x2e,0x0d,0x0d,0x2f][s.subtile_column as usize] {
        return Some((s.subtile_column,0));
    }
    (s.metatile_id==0x2e
        && s.tile_index==ART[s.subtile_row as usize*4+s.subtile_column as usize])
        .then_some((s.subtile_column,s.subtile_row+1))
}

/// Olivine's undecorated table is a complete four-by-four drawing.
pub(crate) fn olivine_table_local(map:&str,s:&VisualTileSource)->Option<(u8,u8)> {
    const ART:[[u16;4];4]=[[0x2e,0x0d,0x0d,0x2f],[0x1c,0x0c,0x0c,0x1d],
        [0x3e,0x50,0x50,0x3f],[0xa8,0x4e,0x4f,0xa9]];
    if map!="OlivineCafe" || s.tileset_id.as_ref()!="game_corner" || s.metatile_id!=0x15
        ||s.subtile_column>=4||s.subtile_row>=4 {return None;}
    (s.tile_index==ART[s.subtile_row as usize][s.subtile_column as usize])
        .then_some((s.subtile_column,s.subtile_row))
}

pub(crate) fn olivine_plant_local(map:&str,s:&VisualTileSource)->Option<(u8,u8)> {
    const ART:[[u16;2];4]=[[0x19,0x2c],[0x29,0x3c],[0x39,0x0f],[0x24,0x25]];
    if map!="OlivineCafe" || s.tileset_id.as_ref()!="game_corner" || s.metatile_id!=0x2a
        ||s.subtile_column>=2||s.subtile_row>=4 {return None;}
    (s.tile_index==ART[s.subtile_row as usize][s.subtile_column as usize])
        .then_some((s.subtile_column,s.subtile_row))
}

/// The front counter has two complete stools in the upper half of block19.
pub(crate) fn celadon_counter_stool_local(map:&str,s:&VisualTileSource)->Option<(u8,u8)> {
    if !matches!(map,"CeladonCafe"|"OlivineCafe") || s.tileset_id.as_ref()!="game_corner" || s.metatile_id!=0x19
        || s.subtile_column>=4 || s.subtile_row>=2 {return None;}
    let x=s.subtile_column%2;let y=s.subtile_row;
    (s.tile_index==[[0x0a,0x0b],[0x1a,0x1b]][y as usize][x as usize]).then_some((x,y))
}

/// Each café table has its own authenticated drawing. The Safari office keeps
/// its legacy relief until its complete furnishing is modeled separately.
pub(crate) fn is_table(map_id: &str, source: &VisualTileSource) -> bool {
    if source.tileset_id.as_ref() != "game_corner" {
        return false;
    }
    match map_id {
        "CeladonCafe" => celadon_table_local(map_id,source).is_some(),
        "OlivineCafe" => olivine_table_local(map_id,source).is_some(),
        "SafariZoneMainOffice" => source.metatile_id == 0x15,
        _ => false,
    }
}

/// Exact source cells forming the cafés' long service counter. The vertical
/// arm is depth visible in the original plan view, not a stack of wall bands;
/// the complete C-shaped footprint rises by one native tile course.
fn is_service_counter(map_id: &str, source: &VisualTileSource) -> bool {
    if !is_cafe_map(map_id) || source.tileset_id.as_ref() != "game_corner" {
        return false;
    }
    match source.metatile_id {
        // Rounded north end and the straight north-south arm.
        0x13 => source.subtile_column < 2 && source.subtile_row >= 1,
        0x16 => source.subtile_column < 2,
        // The turn continues south across the lower half of $17/$18.
        0x17 => source.subtile_column < 2 || source.subtile_row >= 2,
        0x18 => source.subtile_row >= 2,
        _ => false,
    }
}

pub(crate) fn shape(map_id: &str, source: &VisualTileSource) -> Option<CellShape> {
    if is_table(map_id, source) {
        if matches!(map_id,"CeladonCafe"|"OlivineCafe") {
            // The complete authored table owns this drawing atomically. An
            // incomplete source stays on its original plane, without a guessed pedestal.
            return Some(CellShape::PlaneAt{height:0.});
        }
        return Some(CellShape::FloatingRelief {
            height: TABLETOP_THICKNESS,
            ground_tile_index: FLOOR_TILE,
            base_height: TABLETOP_BOTTOM,
        });
    }
    is_service_counter(map_id, source).then_some(CellShape::RaisedTop {
        height: SERVICE_COUNTER_HEIGHT,
        solid: SolidKind::Prop,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    fn source_at(metatile_id: u16, column: u8, row: u8) -> VisualTileSource {
        VisualTileSource {
            tileset_id: Arc::from("game_corner"),
            metatile_id,
            subtile_column: column,
            subtile_row: row,
            tile_index: if metatile_id==0x15 {0x2e}else{0x1c},
        }
    }

    fn source(metatile_id: u16) -> VisualTileSource {
        source_at(metatile_id, 0, 0)
    }

    #[test]
    fn each_cafe_promotes_only_its_complete_round_table_block() {
        let expected = Some(CellShape::FloatingRelief {
            height: TABLETOP_THICKNESS,
            ground_tile_index: FLOOR_TILE,
            base_height: TABLETOP_BOTTOM,
        });
        assert_eq!(shape("CeladonCafe", &source(0x2e)), Some(CellShape::PlaneAt{height:0.}));
        assert_eq!(shape("OlivineCafe", &source(0x15)), Some(CellShape::PlaneAt{height:0.}));
        assert_eq!(shape("SafariZoneMainOffice", &source(0x15)), expected);
        assert_eq!(shape("CeladonCafe", &source(0x15)), None);
        assert_eq!(shape("OlivineCafe", &source(0x2e)), None);
    }

    #[test]
    fn shared_atlas_maps_do_not_inherit_cafe_tables() {
        assert!(is_cafe_map("CeladonCafe"));
        assert!(is_cafe_map("OlivineCafe"));
        assert!(is_cafe_map("SafariZoneMainOffice"));
        assert!(!is_cafe_map("GoldenrodGameCorner"));
        assert_eq!(shape("GoldenrodGameCorner", &source(0x2e)), None);
        assert_eq!(shape("VermilionGym", &source(0x15)), None);
    }

    #[test]
    fn service_counter_raises_only_its_authored_c_footprint() {
        let counter = Some(CellShape::RaisedTop {
            height: SERVICE_COUNTER_HEIGHT,
            solid: SolidKind::Prop,
        });
        for row in 0..4 {
            for column in 0..4 {
                assert_eq!(
                    shape("CeladonCafe", &source_at(0x16, column, row)),
                    (column < 2).then_some(counter.expect("copy shape"))
                );
                assert_eq!(
                    shape("CeladonCafe", &source_at(0x18, column, row)),
                    (row >= 2).then_some(counter.expect("copy shape"))
                );
            }
        }
        assert_eq!(shape("CeladonCafe", &source_at(0x13, 0, 0)), None);
        assert_eq!(shape("CeladonCafe", &source_at(0x13, 0, 1)), counter);
        assert_eq!(shape("CeladonCafe", &source_at(0x17, 3, 0)), None);
        assert_eq!(shape("CeladonCafe", &source_at(0x17, 3, 2)), counter);
    }

    #[test]
    fn other_shared_atlas_maps_do_not_inherit_the_cafe_counter() {
        let counter_cell = source_at(0x16, 0, 0);
        assert_eq!(shape("GoldenrodGameCorner", &counter_cell), None);
        assert_eq!(shape("VermilionGym", &counter_cell), None);
    }
}
