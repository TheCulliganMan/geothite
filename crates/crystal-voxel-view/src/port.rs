//! Exact presentation roles for Crystal's Vermilion Port tileset.

use crystal_render_api::VisualTileSource;

use crate::profile::{CellShape, GROUND_HEIGHT, SolidKind};

pub(crate) const SHIP_HEIGHT: f32 = 28.0;
pub(crate) const PIER_HEIGHT: f32 = 4.0;

pub(crate) fn harbor_map(map:&str)->bool {
    matches!(map,"OlivinePort"|"VermilionPort")
}

/// Shared signature of the complete eight-block ferry drawing. Optical water
/// backing and the model resolver must agree before replacing this rectangle.
pub(crate) fn ferry_source(source: &VisualTileSource, x: usize, y: usize) -> bool {
    x < 16 && y < 8 && source.tileset_id.as_ref() == "port"
        && source.metatile_id == 0x18 + (y / 4 * 4 + x / 4) as u16
        && source.subtile_column as usize == x % 4
        && source.subtile_row as usize == y % 4
        && ferry_anchor(x, y).is_none_or(|tile| source.tile_index == tile)
}

pub(crate) fn ferry_anchor(x: usize, y: usize) -> Option<u16> {
    match (x, y) {
        (0, 2) | (0, 4) => Some(0x2b),
        (6, 0) => Some(0x1a), (8, 1) => Some(0x26), (12, 1) => Some(0x2a),
        (4, 4) => Some(0x46), (8, 4) => Some(0x49), (12, 4) => Some(0x4d),
        (1, 6) => Some(0x58), (7, 6) => Some(0x5c), (11, 6) => Some(0x5e),
        (12, 6) => Some(0x5f), (3, 3) => Some(0x33), (7, 3) => Some(0x3e),
        (11, 3) => Some(0x42), (14, 3) => Some(0x45),
        _ => None,
    }
}
/// The same complete four-tile drawing owns both the modeled mooring post
/// and its continuous water backing. Ship artwork keeps its separate role.
pub(crate) fn bollard_local(s:&VisualTileSource)->Option<(u8,u8)> {
    if s.tileset_id.as_ref()!="port" || s.subtile_column>=4 || s.subtile_row>=4
        || matches!(s.metatile_id,0x18..=0x1f) {return None;}
    let (x,y)=(s.subtile_column%2,s.subtile_row%2);
    (s.tile_index==[[0x01,0x02],[0x11,0x12]][y as usize][x as usize]).then_some((x,y))
}


pub(crate) fn cerulean_floor_source(map: &str, source: &VisualTileSource) -> bool {
    if map != "CeruleanGym" || source.tileset_id.as_ref() != "port"
        || source.tile_index != 0x0a || source.subtile_column >= 4 || source.subtile_row >= 4 {
        return false;
    }
    let mask: u16 = match source.metatile_id {
        0x2b => 0xff00,
        0x2e | 0x37 => 0x3333,
        0x2f | 0x38 => 0xcccc,
        0x31 => 0xffff,
        _ => return false,
    };
    mask & (1 << (source.subtile_row * 4 + source.subtile_column)) != 0
}

pub(crate) fn port_shape(source: &VisualTileSource) -> Option<CellShape> {
    if source.tileset_id.as_ref() != "port" {
        return None;
    }
    if source.tile_index == 0x14 {
        return Some(CellShape::Water);
    }
    // Atlas trim fallback. Authored Cerulean piers use PIER_HEIGHT for both
    // their modeled decks and actor support; the gray rim remains ground.
    if matches!(source.metatile_id, 0x2e..=0x31 | 0x36..=0x39)
        && matches!(source.tile_index, 0x0a | 0x0b | 0x22)
    {
        return Some(CellShape::Relief {
            height: 3.0,
            ground_tile_index: 0x14,
            base_height: GROUND_HEIGHT,
        });
    }
    if (0x18..=0x1f).contains(&source.metatile_id) {
        return Some(CellShape::RaisedTop {
            height: SHIP_HEIGHT,
            solid: SolidKind::Ship,
        });
    }
    // Each repeated blue barrier is one 2x2 face-on drawing. Stand that exact
    // drawing at the south edge of its own cell with zero geometric depth,
    // matching the flat-card tree treatment. It must never become pixel
    // relief or a volume: both make the repeated art read as barrels/cages.
    if matches!(source.tile_index, 0x01 | 0x02 | 0x11 | 0x12) {
        let group_row = source.subtile_row / 2;
        return Some(CellShape::FacadeBand {
            plane_subtile_row: (group_row + 1) * 2,
            band_from_top: source.subtile_row % 2,
            band_count: 2,
            ground_tile_index: 0x14,
            solid: SolidKind::FlatCard,
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    fn source(metatile_id: u16, tile_index: u16) -> VisualTileSource {
        VisualTileSource {
            tileset_id: Arc::from("port"),
            metatile_id,
            subtile_column: 0,
            subtile_row: 0,
            tile_index,
        }
    }

    #[test]
    fn ship_blocks_preserve_water_holes_instead_of_becoming_a_rectangle() {
        assert_eq!(port_shape(&source(0x18, 0x14)), Some(CellShape::Water));
        assert_eq!(
            port_shape(&source(0x18, 0x2b)),
            Some(CellShape::RaisedTop {
                height: SHIP_HEIGHT,
                solid: SolidKind::Ship,
            })
        );
    }

    #[test]
    fn gangway_and_quay_use_the_safe_flat_baseline() {
        assert_eq!(port_shape(&source(0x11, 0x31)), None);
        assert_eq!(port_shape(&source(0x04, 0x05)), None);
    }

    #[test]
    fn repeated_quay_barrier_is_a_zero_depth_two_band_card() {
        assert_eq!(
            port_shape(&source(0x01, 0x01)),
            Some(CellShape::FacadeBand {
                plane_subtile_row: 2,
                band_from_top: 0,
                band_count: 2,
                ground_tile_index: 0x14,
                solid: SolidKind::FlatCard,
            })
        );
        let mut bottom = source(0x01, 0x12);
        bottom.subtile_row = 1;
        assert_eq!(
            port_shape(&bottom),
            Some(CellShape::FacadeBand {
                plane_subtile_row: 2,
                band_from_top: 1,
                band_count: 2,
                ground_tile_index: 0x14,
                solid: SolidKind::FlatCard,
            })
        );
    }

    #[test]
    fn atlas_bridge_trim_fallback_stays_distinct_from_ship_geometry() {
        assert!(matches!(
            port_shape(&source(0x30, 0x0a)),
            Some(CellShape::Relief {
                ground_tile_index: 0x14,
                ..
            })
        ));
    }
}
