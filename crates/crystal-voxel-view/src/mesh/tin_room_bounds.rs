//! Verified blank source padding remains outside the physical room floor.
use super::*;

/// Stair and teleport warps stay inside the enclosure. Only the two ground
/// floor door tiles open through the perimeter.
pub(super) fn opens_perimeter(map: &str, warp: IVec2) -> bool {
    if super::super::super::modeled_dungeons::tin_tower_guardian_map(map) {
        return map == "TinTower1F" && warp.y == 15 && matches!(warp.x, 9 | 10);
    }
    if matches!(map,"SproutTower1F"|"SproutTower2F"|"SproutTower3F") {
        return map == "SproutTower1F" && warp.y == 15 && matches!(warp.x, 9 | 10);
    }
    true
}

fn sprout_upper_insets(frame:&VisualWorldFrame,cells:&[&VisualTile],g:&GridGeometry)->[i32;3] {
    // The third-floor room is flanked by four cells of source lattice and four
    // cells of black padding. That field is outside its enclosure, not a gap
    // between an outer wall and the actual upper room's timber corners.
    let mut valid=[true;2];let mut counts=[[0;2];2];
    for (i,tile) in cells.iter().enumerate() {
        let x=frame.grid_origin.x+(i%g.width)as i32;
        let y=frame.grid_origin.y+(i/g.width)as i32;
        if !(0..40).contains(&x)||!(0..32).contains(&y) {continue;}
        let side=if x<8 {0}else if x>=32 {1}else{continue};
        let black=x<4||x>=36;counts[side][usize::from(!black)]+=1;
        let block=if black {1}else if side==1&&matches!(y/4,4|5) {0x2a}else{0x0b};
        let art=if black {1}else if block==0x0b&&x%4==0 {0x1a}else{0x1b};
        let s=&tile.source;
        valid[side]&=s.tileset_id.as_ref()=="tower"&&s.metatile_id==block&&s.tile_index==art
            &&i32::from(s.subtile_column)==x%4&&i32::from(s.subtile_row)==y%4;
    }
    [if valid[0]&&counts[0].iter().all(|&n|n>=16){8}else{0},
     if valid[1]&&counts[1].iter().all(|&n|n>=16){8}else{0},0]
}

pub(super) fn insets(frame: &VisualWorldFrame, cells: &[&VisualTile], g: &GridGeometry,
    width: i32, height: i32) -> [i32; 3] {
    if cells.len() != g.width * g.height { return [0; 3]; }
    if matches!(frame.map_id.as_ref(),"SproutTower1F"|"SproutTower2F"|"SproutTower3F") {
        if (width,height)!=(40,32) {return [0;3];}
        if frame.map_id.as_ref()=="SproutTower3F" {return sprout_upper_insets(frame,cells,g);}
    }
    if frame.map_id.as_ref()=="DayCare"&&(width,height)!=(20,16) {return [0;3];}
    let (atlas, block, tile_index, edges) =
        if super::super::super::modeled_dungeons::tin_tower_guardian_map(&frame.map_id)
            ||matches!(frame.map_id.as_ref(),"SproutTower1F"|"SproutTower2F") {
            ("tower", 1, 1, [true; 3])
        } else if frame.map_id.as_ref()=="DayCare" {
            ("house",0x0a,0x10,[false,true,false])
        } else if frame.map_id.as_ref() == "CopycatsHouse2F" {
            ("players_house", 4, 0x44, [false, true, false])
        } else { return [0; 3]; };
    let mut counts = [0; 3];
    let mut valid = [true; 3];
    for (i, tile) in cells.iter().enumerate() {
        let x = frame.grid_origin.x + (i % g.width) as i32;
        let y = frame.grid_origin.y + (i / g.width) as i32;
        if x < 0 || y < 0 || x >= width || y >= height { continue; }
        for (edge, inside) in [x < 4, x >= width - 4, y >= height - 4].into_iter().enumerate() {
            if !inside || !edges[edge] { continue; }
            counts[edge] += 1;
            let s = &tile.source;
            valid[edge] &= s.tileset_id.as_ref() == atlas && s.metatile_id == block
                && s.tile_index == tile_index && i32::from(s.subtile_column) == x % 4
                && i32::from(s.subtile_row) == y % 4;
        }
    }
    std::array::from_fn(|i| if valid[i] && counts[i] >= 16 { 4 } else { 0 })
}
