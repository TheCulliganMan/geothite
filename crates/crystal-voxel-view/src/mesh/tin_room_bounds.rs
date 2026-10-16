//! Tin Tower's outer block is blank source padding, outside the room floor.
use super::*;

pub(super) fn insets(frame: &VisualWorldFrame, cells: &[&VisualTile], g: &GridGeometry,
    width: i32, height: i32) -> [i32; 3] {
    if !super::super::super::modeled_dungeons::tin_tower_guardian_map(&frame.map_id)
        || cells.len() != g.width * g.height { return [0; 3]; }
    let mut counts = [0; 3];
    let mut valid = [true; 3];
    for (i, tile) in cells.iter().enumerate() {
        let x = frame.grid_origin.x + (i % g.width) as i32;
        let y = frame.grid_origin.y + (i / g.width) as i32;
        if x < 0 || y < 0 || x >= width || y >= height { continue; }
        for (edge, inside) in [x < 4, x >= width - 4, y >= height - 4].into_iter().enumerate() {
            if !inside { continue; }
            counts[edge] += 1;
            let s = &tile.source;
            valid[edge] &= s.tileset_id.as_ref() == "tower" && s.metatile_id == 1
                && s.tile_index == 1 && i32::from(s.subtile_column) == x % 4
                && i32::from(s.subtile_row) == y % 4;
        }
    }
    std::array::from_fn(|i| if valid[i] && counts[i] >= 16 { 4 } else { 0 })
}
