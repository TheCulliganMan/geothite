//! Exact-source grouping for Crystal's animated cave waterfalls.

use std::collections::{HashSet, VecDeque};

use crystal_render_api::VisualTile;

const WATERFALL_METATILE: u16 = 0x2c;
const WATERFALL_TILE: u16 = 0x40;
pub(crate) const CASCADE_LIP: f32 = 0.10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WaterfallPlacement {
    pub column: usize,
    pub row: usize,
    pub width: usize,
    pub height: usize,
}

/// The upper basin sits below the rock plateau that encloses it. The
/// receiving pool uses the native cave floor, not the outdoor water recess.
fn upper_level(source: &crystal_render_api::VisualTileSource) -> f32 {
    // One source-cell recess clears the enclosing rock's sculpted relief.
    crate::cave::rock_height(source) - crate::profile::SOURCE_TILE_HEIGHT
}

pub(crate) fn tohjo_levels(cells: &[&VisualTile], width: usize, fall: WaterfallPlacement) -> [f32; 2] {
    let source = &cells[fall.row * width + fall.column].source;
    [upper_level(source), crate::profile::GROUND_HEIGHT]
}

/// A short level lip rolls into the falling sheet and flattens into the pool.
/// Terrain and rendered Surf support must sample the same longitudinal curve.
pub(crate) fn cascade_fraction(along: f32) -> f32 {
    let t = ((along - CASCADE_LIP) / (1.0 - 2.0 * CASCADE_LIP)).clamp(0.0, 1.0);
    1.0 - t * t * (3.0 - 2.0 * t)
}

/// Match the quarter-cell triangles used by the sheet while an actor moves
/// between tile centers. A waterfall ride must not jump between height rows.
pub(crate) fn support_correction(
    frame: &crystal_render_api::VisualWorldFrame,
    cell: bevy::prelude::UVec2,
    foot: bevy::prelude::Vec2,
    heights: &[f32],
) -> f32 {
    if frame.map_id.as_ref() != "TohjoFalls" { return 0.; }
    let tile = |x: u32, y: u32| {
        frame.tiles.get(y as usize * frame.grid_size.x as usize + x as usize)
            .filter(|t| t.column == x && t.row == y)
            .or_else(|| frame.tiles.iter().find(|t| t.column == x && t.row == y))
    };
    let is_fall = |x, y| tile(x, y).is_some_and(|t| {
        let s = &t.source;
        matches!(s.tileset_id.as_ref(), "cave" | "dark_cave")
            && s.metatile_id == WATERFALL_METATILE && s.tile_index == WATERFALL_TILE
    });
    if !is_fall(cell.x, cell.y) { return 0.; }
    let mut north = cell.y;
    let mut south = cell.y + 1;
    while north > 0 && is_fall(cell.x, north - 1) { north -= 1; }
    while south < frame.grid_size.y && is_fall(cell.x, south) { south += 1; }
    if north == 0 || south == frame.grid_size.y { return 0.; }
    let width = frame.grid_size.x as usize;
    let Some(&resolved) = heights.get(cell.y as usize * width + cell.x as usize) else { return 0.; };
    let scale = frame.tile_size.y / crate::profile::SOURCE_TILE_HEIGHT;
    let head = upper_level(&tile(cell.x, cell.y).unwrap().source) * scale;
    if heights.get((north - 1) as usize * width + cell.x as usize)
        .is_none_or(|&y| (y - head).abs() > 0.001) { return 0.; }
    let rows = (south - north) as f32;
    let center = (cell.y - north) as f32 + 0.5;
    if (resolved - head * cascade_fraction(center / rows)).abs() > 0.001 { return 0.; }
    let top = frame.center.y + frame.grid_size.y as f32 * frame.tile_size.y * 0.5;
    let along = ((top - foot.y) / frame.tile_size.y - north as f32).clamp(0., rows);
    let knot = (along * 4.).floor() / 4.;
    let a = cascade_fraction(knot / rows);
    let b = cascade_fraction((knot + 0.25) / rows);
    head * (a + (b - a) * (along - knot) * 4.) - resolved
}

pub(crate) fn waterfall_placements(
    cells: &[&VisualTile],
    width: usize,
    height: usize,
) -> Vec<WaterfallPlacement> {
    if cells.len() != width.saturating_mul(height) {
        return Vec::new();
    }
    let is_fall = |index: usize| {
        let source = &cells[index].source;
        matches!(source.tileset_id.as_ref(), "cave" | "dark_cave")
            && source.metatile_id == WATERFALL_METATILE
            && source.tile_index == WATERFALL_TILE
    };
    let mut visited = HashSet::new();
    let mut placements = Vec::new();
    for start in 0..cells.len() {
        if !is_fall(start) || !visited.insert(start) {
            continue;
        }
        let mut queue = VecDeque::from([start]);
        let mut component = vec![start];
        while let Some(index) = queue.pop_front() {
            let column = index % width;
            let row = index / width;
            for (next_column, next_row) in [
                (column.wrapping_sub(1), row),
                (column + 1, row),
                (column, row.wrapping_sub(1)),
                (column, row + 1),
            ] {
                if next_column >= width || next_row >= height {
                    continue;
                }
                let next = next_row * width + next_column;
                if is_fall(next) && visited.insert(next) {
                    component.push(next);
                    queue.push_back(next);
                }
            }
        }
        let min_column = component.iter().map(|index| index % width).min().unwrap();
        let max_column = component.iter().map(|index| index % width).max().unwrap();
        let min_row = component.iter().map(|index| index / width).min().unwrap();
        let max_row = component.iter().map(|index| index / width).max().unwrap();
        let placement_width = max_column - min_column + 1;
        let placement_height = max_row - min_row + 1;
        if component.len() == placement_width * placement_height {
            placements.push(WaterfallPlacement {
                column: min_column,
                row: min_row,
                width: placement_width,
                height: placement_height,
            });
        }
    }
    placements.sort_by_key(|placement| (placement.row, placement.column));
    placements
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bevy::prelude::{Handle, Image};
    use crystal_render_api::{VisualTile, VisualTileSource};

    use super::*;

    #[test]
    fn adjacent_cave_waterfall_blocks_form_one_rectangle() {
        let mut cells = Vec::new();
        for row in 0..8 {
            for column in 0..12 {
                cells.push(VisualTile {
                    animation_frames: None,
                    column,
                    row,
                    source: VisualTileSource {
                        tileset_id: Arc::from("cave"),
                        metatile_id: WATERFALL_METATILE,
                        subtile_column: (column % 4) as u8,
                        subtile_row: (row % 4) as u8,
                        tile_index: WATERFALL_TILE,
                    },
                    texture: Handle::<Image>::Uuid(bevy::asset::uuid::Uuid::from_u128(1), std::marker::PhantomData),
                    priority: false,
                });
            }
        }
        let ordered: Vec<_> = cells.iter().collect();
        assert_eq!(
            waterfall_placements(&ordered, 12, 8),
            vec![WaterfallPlacement {
                column: 0,
                row: 0,
                width: 12,
                height: 8,
            }]
        );
    }
}


/// Tohjo's upper pool is the connected native water north of a complete
/// cascade. It must be separated from the receiving pool by the cascade;
/// ambiguous/clipped layouts are never assigned an invented elevation.
pub(crate) fn tohjo_upstream_pool(map:&str,cells:&[&VisualTile],width:usize,height:usize,
    falls:&[WaterfallPlacement])->Vec<bool> {
    let mut upper=vec![false;cells.len()];
    if map!="TohjoFalls" || cells.len()!=width.saturating_mul(height) {return upper;}
    let pool=|i:usize|{let s=&cells[i].source;
        matches!(s.tileset_id.as_ref(),"cave"|"dark_cave") && s.tile_index==0x14
            && s.subtile_column<4 && s.subtile_row<4};
    for fall in falls {
        if fall.row==0 || fall.row+fall.height>=height {continue;}
        if !(fall.column..fall.column+fall.width).all(|x|
            pool((fall.row-1)*width+x) && pool((fall.row+fall.height)*width+x)) {continue;}
        let mut visited=HashSet::new();let mut queue=VecDeque::new();
        for x in fall.column..fall.column+fall.width {queue.push_back((fall.row-1)*width+x);}
        while let Some(i)=queue.pop_front() {
            if !visited.insert(i) {continue;}
            let x=i%width;let y=i/width;
            for (x,y) in [(x.wrapping_sub(1),y),(x+1,y),(x,y.wrapping_sub(1)),(x,y+1)] {
                if x<width && y<height && pool(y*width+x) && !visited.contains(&(y*width+x)) {queue.push_back(y*width+x);}
            }
        }
        if visited.iter().any(|i|i/width>=fall.row+fall.height) {continue;}
        for i in visited {upper[i]=true;}
    }
    upper
}
