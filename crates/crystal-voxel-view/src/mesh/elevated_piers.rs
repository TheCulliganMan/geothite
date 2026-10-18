//! Raised walking decks. Source water remains beneath the suspended structure.
use super::*;
use bevy::prelude::Vec2;
const HEIGHT: f32 = crate::port::PIER_HEIGHT;
// Sixteen 8px source cells give an eight-walking-tile shore approach. Share this
// distance between graph propagation and height interpolation so the mesh and
// actor support rise at the same gradual rate, including around corners.
const SHORE_APPROACH_CELLS: u8 = 16;

fn cerulean(s: &VisualTileSource) -> bool {
    if s.tileset_id.as_ref() != "port" || s.subtile_column >= 4 || s.subtile_row >= 4 {
        return false;
    }
    let j = s.subtile_row as usize * 4 + s.subtile_column as usize;
    let art = match s.metatile_id {
        0x0c => [20, 20, 20, 20, 20, 20, 20, 20, 5, 5, 5, 5, 5, 5, 5, 5],
        0x14 => [34, 34, 5, 5, 20, 20, 5, 5, 20, 20, 5, 5, 20, 20, 5, 5],
        0x26 => [5; 16],
        0x27 => [5, 5, 5, 5, 5, 5, 5, 5, 34, 34, 34, 34, 20, 20, 20, 20],
        0x29 => [5, 5, 5, 5, 5, 5, 5, 5, 34, 34, 5, 5, 20, 20, 5, 5],
        0x2d => [20, 20, 5, 5, 20, 20, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5],
        0x32 => [6, 7, 5, 5, 22, 23, 5, 5, 8, 9, 5, 5, 24, 25, 5, 5],
        0x33 => [5, 5, 5, 5, 5, 5, 5, 5, 12, 13, 13, 21, 28, 29, 29, 14],
        0x34 => [5, 5, 34, 34, 5, 5, 20, 20, 5, 5, 20, 20, 5, 5, 20, 20],
        0x39 => [11, 11, 11, 11, 20, 20, 20, 20, 5, 5, 5, 5, 5, 5, 5, 5],
        _ => return false,
    };
    s.tile_index == art[j] && (s.tile_index == 5 || s.metatile_id == 0x32 && s.subtile_column < 2)
}
fn route_deck(map: &str, s: &VisualTileSource) -> bool {
    if s.subtile_column >= 4 || s.subtile_row >= 4 {
        return false;
    }
    match (map, s.tileset_id.as_ref(), s.metatile_id, s.tile_index) {
        // This complete native deck identity also occurs in connected-map
        // scenery. Its geometry cannot depend on which side of the seam the
        // player occupies. Other Kanto pier drawings remain context-specific.
        (_, "kanto", 5, 0x4b) => true,
        ("Route12", "kanto", 0x79, _) => {
            let j = s.subtile_row as usize * 4 + s.subtile_column as usize;
            s.tile_index
                == [
                    48, 57, 48, 57, 57, 48, 57, 48, 48, 57, 70, 71, 57, 48, 86, 87,
                ][j]
        }
        ("Route12", "kanto", 0x74, 0x2c | 3) => {
            let j = s.subtile_row as usize * 4 + s.subtile_column as usize;
            s.tile_index == [44, 44, 44, 44, 44, 3, 44, 3, 3, 44, 3, 44, 44, 44, 44, 44][j]
        }
        ("Route26" | "Route27" | "Route32", "johto", 6, 7) => true,
        _ => false,
    }
}
/// Used by both actor support and geometry, in source-pixel units.
fn deck(map: &str, s: &VisualTileSource) -> bool {
    if map == "CeruleanGym" {
        cerulean(s)
    } else {
        route_deck(map, s)
    }
}
fn is_land(s: &VisualTileSource) -> bool {
    if s.subtile_column >= 4 || s.subtile_row >= 4 {
        return false;
    }
    match (s.tileset_id.as_ref(), s.metatile_id, s.tile_index) {
        ("kanto", 1, 0x23 | 0x39) => {
            let j = s.subtile_row as usize * 4 + s.subtile_column as usize;
            s.tile_index == if matches!(j, 4 | 14) { 0x39 } else { 0x23 }
        }
        ("kanto", 0x55, 0x5b) => true,
        ("kanto", 0x77, 0x39) if s.subtile_row < 2 => true,
        // Tall grass still owns ground: native block 03 uses glyph 04.
        ("johto", 1, 6) | ("johto", 2, 5) | ("johto", 3, 4) => true,
        _ => false,
    }
}

pub(super) fn has_piers(map: &str) -> bool {
    matches!(
        map,
        "CeruleanGym"
            | "CeruleanCity"
            | "Route12"
            | "Route13"
            | "Route20"
            | "Route21"
            | "Route24"
            | "Route25"
            | "Route26"
            | "Route27"
            | "Route32"
    )
}
pub(super) fn level_surface(map: &str, s: &VisualTileSource) -> bool {
    deck(map,s) || (has_piers(map) && (entry(map,s) || face(map,s)
        || (s.tile_index == 0x14
            && matches!(s.tileset_id.as_ref(), "kanto" | "johto" | "port"))))
}
pub(super) fn structural_edge(map: &str, s: &VisualTileSource) -> bool {
    face(map, s)
}
pub(super) fn walkway(map: &str, s: &VisualTileSource) -> bool {
    deck(map, s) || entry(map, s)
}
pub(super) fn water_under_deck(map: &str, s: &VisualTileSource, north: Option<&VisualTileSource>) -> bool {
    (deck(map, s) && !garden(map, s))
        || edge_below_deck(map, s, north)
}

// The edge drawing alone is ambiguous (it also appears in city scenery).
// Require its real deck immediately north before recognizing a connected
// bridge edge; never extend the host's standalone face classification.
fn edge_below_deck(map: &str, s: &VisualTileSource, north: Option<&VisualTileSource>) -> bool {
    let connected = s.tileset_id.as_ref() == "kanto" && s.metatile_id == 0x78
        && s.tile_index == 0x31 && s.subtile_row == 0 && s.subtile_column < 4;
    (face(map,s) || connected) && north.is_some_and(|source| deck(map,source))
}
pub(super) fn entry_levels(s: &VisualTileSource) -> Option<[f32; 2]> {
    if s.tileset_id.as_ref() != "port"
        || s.metatile_id != 0x33
        || s.subtile_column >= 4
        || s.subtile_row >= 4
    {
        return None;
    }
    let art = [5, 5, 5, 5, 5, 5, 5, 5, 12, 13, 13, 21, 28, 29, 29, 14];
    if s.tile_index != art[s.subtile_row as usize * 4 + s.subtile_column as usize] {
        return None;
    }
    Some([
        HEIGHT * (1. - s.subtile_row as f32 / 4.),
        HEIGHT * (1. - (s.subtile_row as f32 + 1.) / 4.),
    ])
}
fn garden(map: &str, s: &VisualTileSource) -> bool {
    map == "Route12"
        && s.tileset_id.as_ref() == "kanto"
        && matches!(s.metatile_id, 0x74 | 0x79)
        && route_deck(map, s)
}
pub(super) fn garden_flower_base(map: &str, s: &VisualTileSource) -> Option<f32> {
    (garden(map, s) && s.metatile_id == 0x74 && s.tile_index == 3).then_some(HEIGHT + 0.02)
}
fn garden_ground<'a>(
    cells: &[&'a VisualTile],
    g: &GridGeometry,
    i: usize,
) -> Option<&'a VisualTile> {
    let tile = cells[i];
    let source = &tile.source;
    if source.metatile_id != 0x74 || source.tile_index != 3 {
        return Some(tile);
    }
    // Flowers have a different palette. Their grass comes from the exact
    // native ground cell in this same metatile, never from petal highlights.
    let bx = (i % g.width) as isize - source.subtile_column as isize;
    let by = (i / g.width) as isize - source.subtile_row as isize;
    for y in 0..4 {
        for x in 0..4 {
            let cx = bx + x;
            let cy = by + y;
            if cx < 0 || cy < 0 || cx >= g.width as isize || cy >= g.height as isize {
                continue;
            }
            let t = cells[cy as usize * g.width + cx as usize];
            let s = &t.source;
            if s.tileset_id == source.tileset_id
                && s.metatile_id == 0x74
                && s.tile_index == 0x2c
                && s.subtile_column == x as u8
                && s.subtile_row == y as u8
            {
                return Some(t);
            }
        }
    }
    None
}

fn planting_pigment(tile:&VisualTile,images:&TerrainImageSamples)->Option<[f32;4]> {
    let rgba=tile_rgba(images,tile).ok()?;
    if rgba.len()!=256 || rgba.chunks_exact(4).any(|p|p[3]!=255) {return None;}
    let mut colors=std::collections::BTreeMap::<[u8;3],usize>::new();
    for p in rgba.chunks_exact(4) {
        *colors.entry([p[0],p[1],p[2]]).or_default()+=1;
    }
    if !(1..=4).contains(&colors.len()) {return None;}
    // Native night grass is blue: a green-channel test silently lost its
    // live pigment. The dominant opaque ground pigment owns both exposures.
    let (rgb,_)=colors.into_iter().max_by_key(|(_,count)|*count)?;
    let native=rgb.map(|v| {
        let s=v as f32/255.;
        if s<=0.04045 {s/12.92}else{((s+0.055)/1.055).powf(2.4)}
    });
    let energy=native[0]*0.2126+native[1]*0.7152+native[2]*0.0722;
    let exposure=(energy/0.65).clamp(0.,1.25);
    let organic=[0.19,0.29,0.075];
    Some([organic[0]*exposure*0.84+native[0]*0.16,
        organic[1]*exposure*0.84+native[1]*0.16,
        organic[2]*exposure*0.84+native[2]*0.16,1.])
}
// The mixed landing also includes shrub cells. Keep their previous pigment;
// walkable cells have their own live paving treatment.
fn landing_pigment(tile:&VisualTile,images:&TerrainImageSamples)->Option<[f32;4]> {
    let rgba=tile_rgba(images,tile).ok()?;
    let mut colors=std::collections::BTreeMap::<[u8;3],usize>::new();
    for p in rgba.chunks_exact(4).filter(|p|p[1]>p[0] && p[1]>p[2] && p[3]==255) {
        *colors.entry([p[0],p[1],p[2]]).or_default()+=1;
    }
    let (rgb,_)=colors.into_iter().max_by_key(|(_,count)|*count)?;
    let c=rgb.map(|v| {let s=v as f32/255.;
        if s<=0.04045 {s/12.92}else{((s+0.055)/1.055).powf(2.4)}});
    Some([c[0],c[1],c[2],1.])
}
fn landing_paving_pigment(tile:&VisualTile,images:&TerrainImageSamples)->Option<[f32;4]> {
    let rgba=tile_rgba(images,tile).ok()?;
    if rgba.len()!=256 || rgba.chunks_exact(4).any(|p|p[3]!=255) {return None;}
    let mut c=[0.,0.,0.,1.];
    for p in rgba.chunks_exact(4) {for i in 0..3 {
        let s=p[i] as f32/255.;
        c[i]+=(if s<=0.04045 {s/12.92}else{((s+0.055)/1.055).powf(2.4)})/64.;
    }}
    Some(c)
}
fn landing_ground<'a>(cells:&[&'a VisualTile],g:&GridGeometry,i:usize)->Option<&'a VisualTile> {
    let source=&cells[i].source;
    let bx=(i%g.width) as isize-source.subtile_column as isize;
    let by=(i/g.width) as isize-source.subtile_row as isize;
    for y in 0..4 {for x in 0..4 {
        let (cx,cy)=(bx+x,by+y);
        if cx<0 || cy<0 || cx>=g.width as isize || cy>=g.height as isize {continue;}
        let tile=cells[cy as usize*g.width+cx as usize];
        let s=&tile.source;
        if s.tileset_id==source.tileset_id && s.metatile_id==0x79 && s.tile_index==0x30
            && s.subtile_column as isize==x && s.subtile_row as isize==y {return Some(tile);}
    }}
    None
}
// A short brick strip enclosed by authenticated dry ground on opposite sides is a
// ground-supported causeway, not a suspended deck. Require actual native
// ground at both contacts; clipped/missing context keeps the pier fallback.
fn grounded_path(map:&str,cells:&[&VisualTile],g:&GridGeometry,i:usize)->bool {
    let s=&cells[i].source;
    if !route_deck(map,s) || s.tileset_id.as_ref()!="kanto" || s.metatile_id!=5 {return false;}
    let contact=|dx:isize,dz:isize| {
        for step in 1..=4 {
            let x=(i%g.width)as isize+dx*step;
            let z=(i/g.width)as isize+dz*step;
            if x<0||z<0||x>=g.width as isize||z>=g.height as isize {return false;}
            let s=&cells[z as usize*g.width+x as usize].source;
            if s.tileset_id.as_ref()=="kanto" && s.metatile_id==0x31 && s.tile_index==0x39
                && s.subtile_column<4 && s.subtile_row<4 {return true;}
            if !route_deck(map,s) {return false;}
        }
        false
    };
    (contact(-1,0)&&contact(1,0)) || (contact(0,-1)&&contact(0,1))
}

// Opposing water contacts prove a real crossing, even when its connected
// land approaches also contain short brick strips enclosed by dry terrain.
fn crosses_water(map:&str,cells:&[&VisualTile],g:&GridGeometry,i:usize)->bool {
    let contact=|dx:isize,dy:isize| {
        for step in 1..=g.width.max(g.height) as isize {
            let x=(i%g.width)as isize+dx*step;
            let y=(i/g.width)as isize+dy*step;
            if x<0||y<0||x>=g.width as isize||y>=g.height as isize {return false;}
            let s=&cells[y as usize*g.width+x as usize].source;
            if crate::cave_water::standing_water_source(s) {return true;}
            if !route_deck(map,s) {return false;}
        }
        false
    };
    (contact(-1,0)&&contact(1,0)) || (contact(0,-1)&&contact(0,1))
}

struct PierLevels {
    width: usize,
    values: Vec<f32>,
}
impl PierLevels {
    fn new(
        map: &str,
        cells: &[&VisualTile],
        g: &GridGeometry,
        field: Option<&terraces::Field<'_>>,
    ) -> Self {
        let width = g.width + 1;
        let count = width * (g.height + 1);
        let decks = cells
            .iter()
            .map(|t| deck(map, &t.source))
            .collect::<Vec<_>>();
        let cell = |x: isize, y: isize| {
            if x < 0 || y < 0 || x >= g.width as isize || y >= g.height as isize {
                None
            } else {
                Some(y as usize * g.width + x as usize)
            }
        };
        let has_deck = |x: isize, y: isize| cell(x, y).is_some_and(|i| decks[i]);
        let mut distance = vec![u8::MAX; count];
        let mut targets = vec![0f32; count];
        let mut islands = vec![false; count];
        let mut eligible = vec![false; count];
        let mut queue = std::collections::VecDeque::new();
        for y in 0..=g.height {
            for x in 0..=g.width {
                let v = y * width + x;
                let incident = [(-1, -1), (0, -1), (-1, 0), (0, 0)]
                    .map(|(dx, dy)| cell(x as isize + dx, y as isize + dy));
                eligible[v] = incident.iter().flatten().any(|&i| decks[i]);
                islands[v] = incident
                    .iter()
                    .flatten()
                    .any(|&i| garden(map, &cells[i].source));
                if !eligible[v] || islands[v] {
                    continue;
                }
                let mut shore = None::<f32>;
                for i in incident
                    .into_iter()
                    .flatten()
                    .filter(|&i| !decks[i] && is_land(&cells[i].source))
                {
                    let contact = Vec2::new(
                        g.origin_x + x as f32 * g.tile_width,
                        g.origin_z + y as f32 * g.tile_height,
                    );
                    let center = Vec2::new(
                        g.origin_x + (i % g.width) as f32 * g.tile_width + g.tile_width * 0.5,
                        g.origin_z + (i / g.width) as f32 * g.tile_height + g.tile_height * 0.5,
                    );
                    let height = field.map_or(0., |f| {
                        f.height(contact + (center - contact).normalize_or_zero() * 0.001)
                    }) * 8.
                        / g.tile_height;
                    shore = Some(shore.map_or(height, |h| h.max(height)));
                }
                if let Some(shore) = shore {
                    distance[v] = 0;
                    targets[v] = shore;
                    queue.push_back(v);
                }
            }
        }
        // Follow the actual deck graph around bends; scanning straight lines
        // alone makes a steep seam on the inside of a corner.
        while let Some(v) = queue.pop_front() {
            if distance[v] >= SHORE_APPROACH_CELLS {
                continue;
            }
            let x = (v % width) as isize;
            let y = (v / width) as isize;
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let nx = x + dx;
                let ny = y + dy;
                if nx < 0 || ny < 0 || nx > g.width as isize || ny > g.height as isize {
                    continue;
                }
                let next = ny as usize * width + nx as usize;
                let connected = if dx != 0 {
                    let left = x.min(nx);
                    has_deck(left, y) || has_deck(left, y - 1)
                } else {
                    let top = y.min(ny);
                    has_deck(x, top) || has_deck(x - 1, top)
                };
                if !eligible[next] || islands[next] || !connected {
                    continue;
                }
                let d = distance[v] + 1;
                if d < distance[next] || (d == distance[next] && targets[v] > targets[next]) {
                    distance[next] = d;
                    targets[next] = targets[v];
                    queue.push_back(next);
                }
            }
        }
        // Raised garden islands keep their flat soil surface. Ease the long
        // shore grade into those fixed-height edges instead of leaving a lip.
        let mut garden_floor = vec![0.; count];
        for v in 0..count {
            if islands[v] {
                garden_floor[v] = HEIGHT;
                queue.push_back(v);
            }
        }
        while let Some(v) = queue.pop_front() {
            let next_height = garden_floor[v] - HEIGHT / 4.;
            if next_height <= 0. {
                continue;
            }
            let x = (v % width) as isize;
            let y = (v / width) as isize;
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let nx = x + dx;
                let ny = y + dy;
                if nx < 0 || ny < 0 || nx > g.width as isize || ny > g.height as isize {
                    continue;
                }
                let next = ny as usize * width + nx as usize;
                let connected = if dx != 0 {
                    let left = x.min(nx);
                    has_deck(left, y) || has_deck(left, y - 1)
                } else {
                    let top = y.min(ny);
                    has_deck(x, top) || has_deck(x - 1, top)
                };
                if eligible[next] && connected && next_height > garden_floor[next] {
                    garden_floor[next] = next_height;
                    queue.push_back(next);
                }
            }
        }
        // Ground-supported strips meet their actual land datum along the
        // whole width. They must not dip under a cave landing or bow upward.
        let mut ground_cells=(0..cells.len()).map(|i|grounded_path(map,cells,g,i)).collect::<Vec<_>>();
        let mut ground_queue=ground_cells.iter().enumerate().filter_map(|(i,&yes)|yes.then_some(i))
            .collect::<std::collections::VecDeque<_>>();
        while let Some(i)=ground_queue.pop_front() {
            let (x,y)=((i%g.width)as isize,(i/g.width)as isize);
            for (dx,dy) in [(-1,0),(1,0),(0,-1),(0,1)] {
                let Some(j)=cell(x+dx,y+dy) else {continue;};
                let s=&cells[j].source;
                if !ground_cells[j] && decks[j] && s.tileset_id.as_ref()=="kanto" && s.metatile_id==5 {
                    ground_cells[j]=true;ground_queue.push_back(j);
                }
            }
        }
        let mut water_queue=ground_cells.iter().enumerate()
            .filter_map(|(i,&yes)|(yes&&crosses_water(map,cells,g,i)).then_some(i))
            .collect::<std::collections::VecDeque<_>>();
        while let Some(i)=water_queue.pop_front() {
            if !ground_cells[i] {continue;}
            ground_cells[i]=false;
            let (x,y)=((i%g.width)as isize,(i/g.width)as isize);
            for (dx,dy) in [(-1,0),(1,0),(0,-1),(0,1)] {
                if let Some(j)=cell(x+dx,y+dy) {if ground_cells[j] {water_queue.push_back(j);}}
            }
        }
        let mut grounded=vec![None::<f32>;count];
        for i in 0..cells.len() {
            if !ground_cells[i] {continue;}
            let x=i%g.width;let y=i/g.width;
            for (dx,dy) in [(0,0),(0,1),(1,1),(1,0)] {
                let point=Vec2::new(g.origin_x+(x+dx)as f32*g.tile_width,
                    g.origin_z+(y+dy)as f32*g.tile_height);
                grounded[(y+dy)*width+x+dx]=Some(field.map_or(0.,|f|f.height(point))*8./g.tile_height);
            }
        }
        let values = (0..count)
            .map(|i| {
                if let Some(land)=grounded[i] {return land;}
                let position=Vec2::new(g.origin_x+(i%width)as f32*g.tile_width,
                    g.origin_z+(i/width)as f32*g.tile_height);
                let river=field.map_or(0.,|f|f.water_surface_offset(position))*8./g.tile_height;
                let deck_height=HEIGHT+river;
                if distance[i] > SHORE_APPROACH_CELLS {
                    deck_height
                } else {
                    let t = distance[i] as f32 / SHORE_APPROACH_CELLS as f32;
                    (targets[i] * (1. - t) + deck_height * t).max(garden_floor[i])
                }
            })
            .collect();
        Self { width, values }
    }
}
fn heights(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    i: usize,
    levels: &PierLevels,
) -> [f32; 4] {
    if map == "CeruleanGym" {
        if let Some([n, s]) = entry_levels(&cells[i].source) {
            return [n, s, s, n];
        }
    }
    let v = (i / g.width) * levels.width + i % g.width;
    [
        levels.values[v],
        levels.values[v + levels.width],
        levels.values[v + levels.width + 1],
        levels.values[v + 1],
    ]
}

pub(super) fn support(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    field: Option<&terraces::Field<'_>>,
) {
    let levels = PierLevels::new(map, cells, g, field);
    for (i, tile) in cells.iter().enumerate() {
        if entry(map, &tile.source) {
            let [n, s] = entry_levels(&tile.source).unwrap();
            mesh.footing_heights[i] = (n + s) * 0.5 * g.tile_height / 8.;
        }
        if deck(map, &tile.source) {
            mesh.footing_heights[i] =
                heights(map, cells, g, i, &levels).iter().sum::<f32>() * 0.25 * g.tile_height / 8.;
        }
    }
}
fn entry(map: &str, s: &VisualTileSource) -> bool {
    if map != "CeruleanGym"
        || s.tileset_id.as_ref() != "port"
        || s.metatile_id != 0x33
        || s.subtile_column >= 4
        || !(2..4).contains(&s.subtile_row)
    {
        return false;
    }
    s.tile_index
        == [[12, 13, 13, 21], [28, 29, 29, 14]][s.subtile_row as usize - 2]
            [s.subtile_column as usize]
}
fn face(map: &str, s: &VisualTileSource) -> bool {
    if s.subtile_column >= 4 || s.subtile_row >= 4 {
        return false;
    }
    match (
        map,
        s.tileset_id.as_ref(),
        s.metatile_id,
        s.tile_index,
        s.subtile_row,
    ) {
        ("CeruleanCity" | "Route12" | "Route13" | "Route20" | "Route21" | "Route24" | "Route25", "kanto", 0x78, 0x31, 0) => true,
        ("Route26" | "Route27" | "Route32", "johto", 0x43, 0x49, 0) => true,
        ("CeruleanGym", "port", 0x08 | 0x14 | 0x34, 0x22, 0) => true,
        ("CeruleanGym", "port", 0x27 | 0x29, 0x22, 2) => true,
        _ => false,
    }
}
fn beam(out: &mut SurfaceMeshData, b: [f32; 4], low: f32, high: f32) {
    let [w, e, n, s] = b;
    for (p, normal) in [
        (
            [[w, high, n], [w, high, s], [e, high, s], [e, high, n]],
            [0., 1., 0.],
        ),
        (
            [[w, low, n], [e, low, n], [e, low, s], [w, low, s]],
            [0., -1., 0.],
        ),
        (
            [[e, low, n], [w, low, n], [w, high, n], [e, high, n]],
            [0., 0., -1.],
        ),
        (
            [[w, low, s], [e, low, s], [e, high, s], [w, high, s]],
            [0., 0., 1.],
        ),
        (
            [[w, low, n], [w, low, s], [w, high, s], [w, high, n]],
            [-1., 0., 0.],
        ),
        (
            [[e, low, s], [e, low, n], [e, high, n], [e, high, s]],
            [1., 0., 0.],
        ),
    ] {
        append_quad(out, p, normal, [[0.; 2]; 4], [0.47, 0.34, 0.19, 1.]);
    }
}
pub(super) fn append(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    claimed: &mut [bool],
    images: Option<&TerrainImageSamples>,
    field: Option<&terraces::Field<'_>>,
    map_origin: [i32;2],
) {
    let scale = g.tile_height / 8.;
    let levels = PierLevels::new(map, cells, g, field);
    let water = cells.iter().position(|t| {
        t.source.tile_index == 0x14
            && matches!(t.source.tileset_id.as_ref(), "port" | "johto" | "kanto")
    });
    for (i, tile) in cells.iter().enumerate() {
        let source = &tile.source;
        let (w, e, n, s) = g.bounds(i % g.width, i / g.width);
        if garden(map, source) {
            let first = mesh.solid.positions.len();
            beam(&mut mesh.solid, [w, e, n, s], -6. * scale, HEIGHT * scale);
            mesh.solid.colors[first..].fill([0.25, 0.16, 0.075, 1.]);
            let pigment=(source.metatile_id==0x74).then(||images.and_then(|images|
                garden_ground(cells,g,i).and_then(|ground|planting_pigment(ground,images)))).flatten();
            let paving=(source.metatile_id==0x79 && matches!(source.tile_index,0x30|0x39))
                .then(||images.and_then(|images|landing_ground(cells,g,i)
                    .and_then(|ground|landing_paving_pigment(ground,images)))).flatten();
            let green=pigment.or(paving).or_else(||images.and_then(|images|
                garden_ground(cells,g,i).and_then(|ground|landing_pigment(ground,images))))
                .unwrap_or([0.23,0.40,0.07,1.]);
            let top_first=mesh.solid.positions.len();
            append_solid_quad(
                &mut mesh.solid,
                [
                    [w, (HEIGHT + 0.01) * scale, n],
                    [w, (HEIGHT + 0.01) * scale, s],
                    [e, (HEIGHT + 0.01) * scale, s],
                    [e, (HEIGHT + 0.01) * scale, n],
                ],
                [0., 1., 0.],
                green,
            );
            if pigment.is_some() || paving.is_some() {
                // Reuse the retained granular ground material with live
                // pigment. Coordinates stay anchored while the viewport moves.
                for k in top_first..mesh.solid.positions.len() {
                    let p=mesh.solid.positions[k];
                    let x=map_origin[0] as f32+tile.column as f32+(p[0]-w)/g.tile_width;
                    let y=map_origin[1] as f32+tile.row as f32+(p[2]-n)/g.tile_height;
                    mesh.solid.uvs[k]=if paving.is_some() {
                        [crate::scenery_finish::PAVING_GRAIN_BASE[0]+x,
                            crate::scenery_finish::PAVING_GRAIN_BASE[1]-y]
                    } else {[78000.+x,-80000.+y]};
                }
            }
            if source.metatile_id == 0x74 && source.tile_index == 3 {
                assert!(modeled_exteriors::append_flower(
                    &mut mesh.animated_solid,
                    map,
                    tile,
                    g,
                    (HEIGHT + 0.02) * scale,
                    [0, 0],
                    images,
                ));
            }
            if !claimed[i] {
                mesh.authored_cells[i] = Some("outdoor/pier-ground-island");
                claimed[i] = true;
            }
            continue;
        }
        if claimed[i] {
            if route_deck(map, source) {
                beam(
                    &mut mesh.solid,
                    [w, e, n, s],
                    (HEIGHT - 2.) * scale,
                    (HEIGHT - 0.02) * scale,
                );
            }
            continue;
        }
        if edge_below_deck(map, source, i.checked_sub(g.width).map(|j| &cells[j].source)) {
            let Some(water) = water else {
                continue;
            };
            append_top(
                &mut mesh.textured,
                [w, e, n, s],
                crate::profile::WATER_HEIGHT * scale,
                g.uv(water % g.width, water / g.width),
            );
            let deck = heights(map, cells, g, i - g.width, &levels);
            let [west, east] = [deck[1] * scale, deck[2] * scale];
            let first = mesh.solid.positions.len();
            beam(
                &mut mesh.solid,
                [w, e, n, n + 1.2 * scale],
                (HEIGHT - 3.) * scale,
                HEIGHT * scale,
            );
            let slope = (east - west) / (e - w);
            for (point, normal) in mesh.solid.positions[first..]
                .iter_mut()
                .zip(&mut mesh.solid.normals[first..])
            {
                point[1] += west - HEIGHT * scale + (point[0] - w) * slope;
                *normal = Vec3::new(normal[0] - normal[1] * slope, normal[1], normal[2])
                    .normalize()
                    .to_array();
            }
            if source.subtile_column % 2 == 0 {
                let high = west + 2.5 * scale * slope - 3. * scale;
                beam(
                    &mut mesh.solid,
                    [w + scale, w + 2.5 * scale, n, n + 1.5 * scale],
                    -6. * scale,
                    high,
                );
            }
            mesh.authored_cells[i] = Some("outdoor/elevated-pier-edge");
            claimed[i] = true;
            continue;
        }
        if entry(map, source) {
            let [high, low] = entry_levels(source).unwrap().map(|h| h * scale);
            let (u0, u1, v0, v1) = g.uv(i % g.width, i / g.width);
            let p = [[w, high, n], [w, low, s], [e, low, s], [e, high, n]];
            append_quad(
                &mut mesh.textured,
                p,
                Vec3::new(0., 1., HEIGHT / 32.).normalize().to_array(),
                [[u0, v0], [u0, v1], [u1, v1], [u1, v0]],
                [1.; 4],
            );
            mesh.authored_cells[i] = Some("interior/pier-entry-ramp");
            claimed[i] = true;
            continue;
        }
        if !route_deck(map, source) {
            continue;
        }
        if let Some(water) = water {
            append_top(
                &mut mesh.textured,
                [w, e, n, s],
                crate::profile::WATER_HEIGHT * scale,
                g.uv(water % g.width, water / g.width),
            );
        }
        let h = heights(map, cells, g, i, &levels).map(|y| y * scale);
        let p = [[w, h[0], n], [w, h[1], s], [e, h[2], s], [e, h[3], n]];
        let normal = (Vec3::from_array(p[1]) - Vec3::from_array(p[0]))
            .cross(Vec3::from_array(p[2]) - Vec3::from_array(p[0]))
            .normalize()
            .to_array();
        let (u0, u1, v0, v1) = g.uv(i % g.width, i / g.width);
        let native = matches!(
            (source.tileset_id.as_ref(), source.tile_index),
            ("kanto", 0x4b) | ("johto", 7)
        ) && images.is_some_and(|images| {
            kanto_paving::raised_brick_deck(&mut mesh.solid, g, i, tile, images, h)
        });
        if !native {
            append_quad(
                &mut mesh.textured,
                p,
                normal,
                [[u0, v0], [u0, v1], [u1, v1], [u1, v0]],
                [1.; 4],
            );
        }
        // A shallow frame follows the approach slope and leaves air below.
        for j in 0..4 {
            let k = (j + 1) % 4;
            let mut low_j = p[j];
            low_j[1] -= 2. * scale;
            let mut low_k = p[k];
            low_k[1] -= 2. * scale;
            let face = [low_j, low_k, p[k], p[j]];
            let normal = (Vec3::from_array(face[1]) - Vec3::from_array(face[0]))
                .cross(Vec3::from_array(face[2]) - Vec3::from_array(face[0]))
                .normalize()
                .to_array();
            let (dx,dz)=[(-1,0),(0,1),(1,0),(0,-1)][j];
            let (nx,nz)=((i%g.width)as isize+dx,(i/g.width)as isize+dz);
            let continues=nx>=0 && nz>=0 && nx<g.width as isize && nz<g.height as isize
                && deck(map,&cells[nz as usize*g.width+nx as usize].source);
            // Interior source-cell seams are not exposed structural edges.
            // Beveling them would push bright grout through the paved deck.
            if !(native && !continues && images.is_some_and(|images|kanto_paving::raised_brick_side(
                &mut mesh.solid,tile,images,face,normal,g.tile_width.min(g.tile_height)))) {
                append_quad(&mut mesh.solid,face,normal,[[0.;2];4],[0.47,0.34,0.19,1.]);
            }
        }
        let bottom = [p[3], p[2], p[1], p[0]].map(|mut v| {
            v[1] -= 2. * scale;
            v
        });
        let normal = (Vec3::from_array(bottom[1]) - Vec3::from_array(bottom[0]))
            .cross(Vec3::from_array(bottom[2]) - Vec3::from_array(bottom[0]))
            .normalize()
            .to_array();
        append_quad(
            &mut mesh.solid,
            bottom,
            normal,
            [[0.; 2]; 4],
            [0.34, 0.24, 0.13, 1.],
        );
        {
            let x = (i % g.width) as isize;
            let y = (i / g.width) as isize;
            for (dx, dy, bounds, phase) in [
                (
                    -1,
                    0,
                    [
                        w + 0.5 * scale,
                        w + 2. * scale,
                        n + 3. * scale,
                        n + 4.5 * scale,
                    ],
                    source.subtile_row,
                ),
                (
                    1,
                    0,
                    [
                        e - 2. * scale,
                        e - 0.5 * scale,
                        n + 3. * scale,
                        n + 4.5 * scale,
                    ],
                    source.subtile_row,
                ),
                (
                    0,
                    -1,
                    [
                        w + 3. * scale,
                        w + 4.5 * scale,
                        n + 0.5 * scale,
                        n + 2. * scale,
                    ],
                    source.subtile_column,
                ),
                (
                    0,
                    1,
                    [
                        w + 3. * scale,
                        w + 4.5 * scale,
                        s - 2. * scale,
                        s - 0.5 * scale,
                    ],
                    source.subtile_column,
                ),
            ] {
                let cx = x + dx;
                let cy = y + dy;
                if phase % 2 != 0
                    || cx < 0
                    || cy < 0
                    || cx >= g.width as isize
                    || cy >= g.height as isize
                {
                    continue;
                }
                let neighbor = &cells[cy as usize * g.width + cx as usize].source;
                if !deck(map, neighbor) && !is_land(neighbor) {
                    let at = |x: f32, z: f32| {
                        let x = (x - w) / (e - w);
                        let z = (z - n) / (s - n);
                        (h[0] * (1. - x) + h[3] * x) * (1. - z) + (h[1] * (1. - x) + h[2] * x) * z
                    };
                    let top = [
                        at(bounds[0], bounds[2]),
                        at(bounds[0], bounds[3]),
                        at(bounds[1], bounds[2]),
                        at(bounds[1], bounds[3]),
                    ]
                    .into_iter()
                    .fold(f32::NEG_INFINITY, f32::max)
                        - 2. * scale;
                    if top > crate::profile::WATER_HEIGHT * scale {
                        beam(&mut mesh.solid, bounds, -6. * scale, top + 0.01 * scale);
                    }
                }
            }
        }
        mesh.authored_cells[i] = Some("outdoor/elevated-pier");
        claimed[i] = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::IVec2;
    #[test]
    fn inland_river_bridge_keeps_its_clearance_above_the_shared_water_plane() {
        let mut frame=super::super::tests::frame(4,20,(0..80).map(|i| {
            let mut s=super::super::tests::source_with_tile(5,(i%4)as u8,(i/4%4)as u8,0x4b);
            s.tileset_id="kanto".into();s
        }).collect());
        frame.map_id="CeruleanCity".into();
        let g=GridGeometry {width:4,height:20,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let datum=crystal_render_api::VisualTerraceDatum {origin:bevy::prelude::IVec2::ZERO,
            size:bevy::prelude::UVec2::new(4,20),levels:vec![[2.;4];80].into(),water_barriers:Default::default(),
            open_water:Some(crystal_render_api::VisualOpenWaterDatum {level:2.,
                cells:(0..80).map(|i|bevy::prelude::IVec2::new(i%4,i/4)).collect()})};
        let field=terraces::Field::new(&[],&g,[0,0]).with_datum(Some(&datum));
        let levels=PierLevels::new("CeruleanCity",&cells,&g,Some(&field));
        assert!(levels.values.iter().all(|&v|(v-(HEIGHT+12.)).abs()<0.001),
            "raising the river must raise the suspended deck, never flood it");
        let mut closed=datum.clone();closed.open_water=None;
        let field=terraces::Field::new(&[],&g,[0,0]).with_datum(Some(&closed));
        let closed_levels=PierLevels::new("CeruleanCity",&cells,&g,Some(&field));
        assert_eq!(closed_levels.values,levels.values,
            "complete connected source maps cannot drop the bridge back to sea level");
    }

    #[test]
    fn connected_kanto_bridge_matches_its_native_route_without_promoting_city_edges() {
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(
            std::path::PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap())).unwrap();
        let pack=loaded.pack();
        let art=&pack.runtime_files()["data/tilesets/kanto_metatiles.bin"];
        assert_eq!(&art[5*16..6*16],&[0x4b;16],"exact shared native deck drawing");
        let data=&pack.data().maps["Route21"];
        let w=data.attributes.width as usize*4;
        let h=data.attributes.height as usize*4;
        let tiles=(0..w*h).map(|i| {
            let x=i%w;let y=i/w;let block=data.blocks[y/4*(w/4)+x/4];
            VisualTile {column:x as u32,row:y as u32,texture:Default::default(),priority:false,
                animation_frames:None,source:VisualTileSource {tileset_id:"kanto".into(),
                    metatile_id:block,subtile_column:(x%4) as u8,subtile_row:(y%4) as u8,
                    tile_index:art[block as usize*16+y%4*4+x%4] as u16}}
        }).collect::<Vec<_>>();
        let cells=tiles.iter().collect::<Vec<_>>();
        let g=GridGeometry {width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let build=|map| {
            let mut mesh=TerrainMeshData {footing_heights:vec![0.;w*h],authored_cells:vec![None;w*h],..Default::default()};
            support(&mut mesh,map,&cells,&g,None);
            append(&mut mesh,map,&cells,&g,&mut vec![false;w*h],None,None,[0,0]);
            mesh
        };
        let native=build("Route21");let connected=build("CinnabarIsland");
        assert!(native.authored_cells.iter().any(|c|*c==Some("outdoor/elevated-pier")));
        assert!(native.authored_cells.iter().any(|c|*c==Some("outdoor/elevated-pier-edge")));
        assert_eq!(connected.footing_heights,native.footing_heights);
        assert_eq!(connected.authored_cells,native.authored_cells);
        assert_eq!(connected.solid.positions,native.solid.positions);
        assert_eq!(connected.solid.normals,native.solid.normals);
        assert_eq!(connected.solid.indices,native.solid.indices);
        assert_eq!(connected.textured.positions,native.textured.positions,"water stays below the suspended deck");
        for tile in &tiles {
            let s=&tile.source;
            if deck("Route21",s) {assert!(level_surface("CinnabarIsland",s));}
            if s.metatile_id==0x78 && s.subtile_row==0 && s.tile_index==0x31 {
                assert!(!level_surface("CeladonCity",s));
                assert!(!structural_edge("CeladonCity",s));
                assert!(!water_under_deck("CeladonCity",s,None));
                let mut unrelated=s.clone();unrelated.metatile_id=1;
                assert!(!water_under_deck("CeladonCity",s,Some(&unrelated)));
            }
        }
        let edge=tiles.iter().find(|t|t.source.metatile_id==0x78 && t.source.subtile_row==0).unwrap();
        let g=GridGeometry {width:1,height:1,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut mesh=TerrainMeshData {footing_heights:vec![0.],authored_cells:vec![None],..Default::default()};
        append(&mut mesh,"CeladonCity",&[edge],&g,&mut vec![false],None,None,[0,0]);
        assert!(mesh.solid.positions.is_empty() && mesh.textured.positions.is_empty());
        assert_eq!(mesh.authored_cells,vec![None]);
    }
    #[test]
    fn raised_planting_keeps_geometry_and_accepts_native_blue_night_ground() {
        let tile=VisualTile {column:0,row:0,texture:Default::default(),priority:false,
            animation_frames:None,source:VisualTileSource {tileset_id:"kanto".into(),
                metatile_id:0x74,subtile_column:0,subtile_row:0,tile_index:0x2c}};
        let cells=vec![&tile];
        let mut previous:Option<TerrainMeshData>=None;
        let mut paint=Vec::new();
        for (rgb,origin) in [([156,222,0],0.),([57,99,156],-16.)] {
            let mut images=TerrainImageSamples::default();
            images.pixels.insert(tile.texture.id(),TileImageSample::Rgba(
                [rgb[0],rgb[1],rgb[2],255].repeat(64)));
            let color=planting_pigment(&tile,&images).expect("uniform native ground is valid paint");
            paint.push(color);
            let g=GridGeometry{width:1,height:1,tile_width:8.,tile_height:8.,origin_x:origin,origin_z:origin};
            let mut mesh=TerrainMeshData{footing_heights:vec![HEIGHT],authored_cells:vec![None],..Default::default()};
            append(&mut mesh,"Route12",&cells,&g,&mut vec![false],Some(&images),None,[24,36]);
            assert_eq!(mesh.footing_heights,vec![HEIGHT]);
            assert_eq!(mesh.solid.positions.len(),28,"existing box and top quad only");
            assert_eq!(mesh.solid.positions[24..].iter().map(|p|p[1]).collect::<Vec<_>>(),vec![HEIGHT+0.01;4]);
            if let Some(before)=previous {
                assert_eq!(mesh.solid.indices,before.solid.indices);
                assert_eq!(mesh.solid.normals,before.solid.normals);
                assert_eq!(mesh.solid.uvs,before.solid.uvs,"scrolling cannot slide planting grain");
                for (a,b) in mesh.solid.positions.iter().zip(&before.solid.positions) {
                    assert_eq!([a[0]-origin,a[1],a[2]-origin],*b);
                }
            }
            previous=Some(mesh);
            images.pixels.insert(tile.texture.id(),TileImageSample::Rgba([57,99,156,0].repeat(64)));
            assert!(planting_pigment(&tile,&images).is_none(),"transparent art supplies no ground pigment");
        }
        assert!(paint[0][1]<0.7,"planting must not become a lime-colored light panel");
        assert!(paint[1][2]>paint[1][0],"blue night pigment cannot fall back to daytime green");
        assert!(paint[1][1]<paint[0][1]);
        let mut landing=tile.clone();landing.source.metatile_id=0x79;landing.source.tile_index=0x30;
        let mut images=TerrainImageSamples::default();
        images.pixels.insert(landing.texture.id(),TileImageSample::Rgba([220,220,220,255].repeat(64)));
        let g=GridGeometry{width:1,height:1,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut mesh=TerrainMeshData{footing_heights:vec![HEIGHT],authored_cells:vec![None],..Default::default()};
        append(&mut mesh,"Route12",&[&landing],&g,&mut vec![false],Some(&images),None,[24,36]);
        assert!(mesh.solid.uvs[24..].iter().all(|uv|crate::scenery_finish::paving_coordinates(*uv).is_some()),
            "the mixed landing receives paving grain instead of becoming a planting bed");
        images.pixels.insert(landing.texture.id(),TileImageSample::Rgba([90,100,150,255].repeat(64)));
        let dark=landing_paving_pigment(&landing,&images).unwrap();
        assert!(dark[2]>dark[1] && dark[1]>dark[0],"native blue-gray night paving cannot become green");
        let mut patterned=landing.clone();patterned.column=1;patterned.source.subtile_column=1;
        patterned.source.tile_index=0x39;
        patterned.texture=bevy::asset::Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(99123),std::marker::PhantomData);
        images.pixels.insert(patterned.texture.id(),TileImageSample::Rgba([220,220,220,255].repeat(64)));
        let g=GridGeometry{width:2,..g};
        let mut mesh=TerrainMeshData{footing_heights:vec![HEIGHT;2],authored_cells:vec![None;2],..Default::default()};
        append(&mut mesh,"Route12",&[&landing,&patterned],&g,&mut vec![false;2],Some(&images),None,[24,36]);
        assert_eq!(mesh.solid.colors[24..28],mesh.solid.colors[52..56],
            "different patterned source tiles share one continuous landing background");
    }
    #[test]
    fn native_tall_grass_bank_meets_the_pier_grade_without_a_height_gap() {
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(
            std::path::PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap())).unwrap();
        let art=&loaded.pack().runtime_files()["data/tilesets/johto_metatiles.bin"];
        assert_eq!(&art[3*16..4*16],&[4;16],"native tall-grass contact identity");
        let g=GridGeometry {width:20,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let tiles=(0..80).map(|i|VisualTile {column:(i%20) as u32,row:(i/20) as u32,
            texture:Default::default(),priority:false,animation_frames:None,
            source:VisualTileSource {tileset_id:"johto".into(),metatile_id:if i%20<4 {3}else{6},
                subtile_column:(i%4) as u8,subtile_row:(i/20) as u8,
                tile_index:if i%20<4 {4}else{7}}}).collect::<Vec<_>>();
        let cells=tiles.iter().collect::<Vec<_>>();
        let edges=[crystal_render_api::VisualJumpLedge {start:IVec2::new(0,4),end:IVec2::new(20,4),descent:IVec2::Y}];
        let field=terraces::Field::with_source(&edges,&g,[0,0],&cells);
        let shore=field.height(Vec2::new(31.999,12.));assert!(shore>HEIGHT);
        let levels=PierLevels::new("Route32",&cells,&g,Some(&field));
        let h=heights("Route32",&cells,&g,24,&levels);
        assert!((h[0]-shore).abs()<0.001 && (h[1]-shore).abs()<0.001,"deck starts on the native bank");
        assert!((h[3]-h[0]).abs()<=(shore-HEIGHT).abs()/16.+0.001);
        let next=heights("Route32",&cells,&g,25,&levels);
        assert_eq!([h[3],h[2]],[next[0],next[1]],"the approach has no internal step");
        let mut mesh=TerrainMeshData {footing_heights:vec![0.;80],authored_cells:vec![None;80],..Default::default()};
        support(&mut mesh,"Route32",&cells,&g,Some(&field));
        assert!((mesh.footing_heights[24]-h.iter().sum::<f32>()/4.).abs()<0.001);
        append(&mut mesh,"Route32",&cells,&g,&mut vec![false;80],None,Some(&field),[0,0]);
        let before=mesh.textured.positions.clone();let footing=mesh.footing_heights.clone();
        terraces::finish_on_map(&mut mesh,"Route32",&cells,&g,[0,0],&edges);
        assert_eq!(mesh.textured.positions,before,"water and fitted deck are not lifted twice");
        for i in 0..80 {if i%20>=4 {assert_eq!(mesh.footing_heights[i],footing[i]);}}
        let mut unknown=tiles.clone();for tile in &mut unknown {if tile.source.metatile_id==3 {tile.source.tile_index=3;}}
        let cells=unknown.iter().collect::<Vec<_>>();
        let levels=PierLevels::new("Route32",&cells,&g,Some(&field));
        assert_eq!(heights("Route32",&cells,&g,24,&levels),[HEIGHT;4],"unproven art is not a land contact");
    }
    #[test]
    fn elevated_pier_ramp_meets_raised_native_paving_without_a_step() {
        let g = GridGeometry {
            width: 20,
            height: 4,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let tiles = (0..80)
            .map(|i| VisualTile {
                column: (i % 20) as u32,
                row: (i / 20) as u32,
                texture: Default::default(),
                priority: false,
                animation_frames: None,
                source: VisualTileSource {
                    tileset_id: "kanto".into(),
                    metatile_id: if i % 20 < 4 { 0x55 } else { 5 },
                    subtile_column: (i % 4) as u8,
                    subtile_row: (i / 20) as u8,
                    tile_index: if i % 20 < 4 { 0x5b } else { 0x4b },
                },
            })
            .collect::<Vec<_>>();
        let cells = tiles.iter().collect::<Vec<_>>();
        let edges = [crystal_render_api::VisualJumpLedge {
            start: IVec2::new(0, 4),
            end: IVec2::new(20, 4),
            descent: IVec2::Y,
        }];
        let field = terraces::Field::with_source(&edges, &g, [0, 0], &cells);
        let shore = field.height(Vec2::new(31.999, 12.));
        assert!(shore > HEIGHT);
        let levels = PierLevels::new("Route25", &cells, &g, Some(&field));
        let h = heights("Route25", &cells, &g, 24, &levels);
        assert!((h[0] - shore).abs() < 0.001 && (h[1] - shore).abs() < 0.001);
        assert!((h[3] - h[0]).abs() <= 1. && (h[2] - h[1]).abs() <= 1.);
        let next = heights("Route25", &cells, &g, 25, &levels);
        assert_eq!([h[3], h[2]], [next[0], next[1]]);
        // Eight source cells are halfway along the longer shore approach.
        // Sixteen cells reach the deck with no step at either end.
        let halfway = heights("Route25", &cells, &g, 32, &levels);
        assert!((halfway[0] - (shore + HEIGHT) * 0.5).abs() < 0.001);
        let end = heights("Route25", &cells, &g, 39, &levels);
        assert!((end[3] - HEIGHT).abs() < 0.001);
        assert!((h[3] - h[0]).abs() <= (shore - HEIGHT).abs() / 16. + 0.001);
        let mut mesh = TerrainMeshData {
            footing_heights: vec![0.; 80],
            authored_cells: vec![None; 80],
            ..Default::default()
        };
        support(&mut mesh, "Route25", &cells, &g, Some(&field));
        let height = mesh.footing_heights[24];
        assert!(height > HEIGHT && height < shore);
        let mut claimed = vec![false; 80];
        append(
            &mut mesh,
            "Route25",
            &cells,
            &g,
            &mut claimed,
            None,
            Some(&field),
            [0,0],
        );
        let old = mesh.solid.positions.clone();
        terraces::finish_on_map(&mut mesh, "Route25", &cells, &g, [0, 0], &edges);
        assert_eq!(mesh.footing_heights[24], height);
        assert_eq!(&mesh.solid.positions[..old.len()], old.as_slice());
    }
    #[test]
    fn source_brick_bevels_at_shore_contact_keep_their_already_fitted_pier_height() {
        let g=GridGeometry {width:16,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut assets=Assets::<Image>::default();let texture=assets.add(Image::default());
        let rows=["00000000","11111110","11111110","00000000",
            "11101111","11101111","00000000","01111110"];
        let rgba=rows.iter().flat_map(|row|row.bytes().flat_map(|b|
            if b==b'0' {[222,255,222,255]} else {[255,156,198,255]})).collect();
        let mut images=TerrainImageSamples::default();
        images.pixels.insert(texture.id(),TileImageSample::Rgba(rgba));
        for (map,family,land_block,land_tile,deck_block,deck_tile) in [
            ("Route25","kanto",0x55,0x5b,5,0x4b),
            ("Route27","johto",1,6,6,7)] {
            let tiles=(0..64).map(|i|VisualTile {
                column:(i%16)as u32,row:(i/16)as u32,texture:texture.clone(),
                priority:false,animation_frames:None,
                source:VisualTileSource {tileset_id:family.into(),
                    metatile_id:if i%16<4 {land_block}else{deck_block},
                    tile_index:if i%16<4 {land_tile}else{deck_tile},
                    subtile_column:(i%4)as u8,subtile_row:(i/16)as u8}
            }).collect::<Vec<_>>();
            let cells=tiles.iter().collect::<Vec<_>>();
            let edges=[crystal_render_api::VisualJumpLedge {start:IVec2::new(0,4),
                end:IVec2::new(16,4),descent:IVec2::Y}];
            let field=terraces::Field::with_source(&edges,&g,[0,0],&cells);
            let mut mesh=TerrainMeshData {footing_heights:vec![0.;64],
                authored_cells:vec![None;64],..Default::default()};
            support(&mut mesh,map,&cells,&g,Some(&field));
            append(&mut mesh,map,&cells,&g,&mut vec![false;64],Some(&images),Some(&field),[0,0]);
            let before=mesh.clone();
            assert!(before.solid.positions.len()>2000,"exercise modeled brick bevels");
            terraces::finish_on_map(&mut mesh,map,&cells,&g,[0,0],&edges);
            for (i,(after,before)) in mesh.solid.positions.iter().zip(&before.solid.positions).enumerate() {
                assert_eq!(after,before,"{map}: land must not lift fitted pier vertex {i} a second time");
            }
            assert_eq!(&mesh.solid.normals[..before.solid.normals.len()],&before.solid.normals);
            assert_eq!(&mesh.solid.colors[..before.solid.colors.len()],&before.solid.colors);
            assert_eq!(&mesh.solid.uvs[..before.solid.uvs.len()],&before.solid.uvs);
            for i in 0..64 {if i%16>=4 {assert_eq!(mesh.footing_heights[i],before.footing_heights[i]);}}
        }
    }

    #[test]
    fn external_elevated_piers_share_deck_footing_and_leave_water_and_rims_low() {
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(
            std::path::PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap()),
        )
        .unwrap();
        let pack = loaded.pack();
        for map in [
            "CeruleanGym",
            "CeruleanCity",
            "Route12",
            "Route13",
            "Route20",
            "Route21",
            "Route24",
            "Route25",
            "Route26",
            "Route27",
            "Route32",
        ] {
            let data = &pack.data().maps[map];
            let ts = pack.data().map_tileset_name(map).unwrap();
            let art = &pack.runtime_files()[&format!("data/tilesets/{ts}_metatiles.bin")];
            let w = data.attributes.width as usize * 4;
            let h = data.attributes.height as usize * 4;
            let tiles = (0..w * h)
                .map(|i| {
                    let x = i % w;
                    let y = i / w;
                    let block = data.blocks[y / 4 * (w / 4) + x / 4];
                    VisualTile {
                        column: x as u32,
                        row: y as u32,
                        texture: Default::default(),
                        priority: false,
                        animation_frames: None,
                        source: VisualTileSource {
                            tileset_id: ts.into(),
                            metatile_id: block,
                            subtile_column: (x % 4) as u8,
                            subtile_row: (y % 4) as u8,
                            tile_index: art[block as usize * 16 + y % 4 * 4 + x % 4] as u16,
                        },
                    }
                })
                .collect::<Vec<_>>();
            let cells = tiles.iter().collect::<Vec<_>>();
            let g = GridGeometry {
                width: w,
                height: h,
                tile_width: 8.,
                tile_height: 8.,
                origin_x: 0.,
                origin_z: 0.,
            };
            let levels = PierLevels::new(map, &cells, &g, None);
            let mut mesh = TerrainMeshData {
                footing_heights: vec![0.; w * h],
                authored_cells: vec![None; w * h],
                ..Default::default()
            };
            if map == "Route12" {
                for (i, tile) in cells.iter().enumerate() {
                    if tile.source.metatile_id == 0x74 && tile.source.tile_index == 3 {
                        let ground = garden_ground(&cells, &g, i).unwrap();
                        assert_eq!(ground.source.tile_index, 0x2c);
                        assert_eq!(
                            ground.column as i32 - ground.source.subtile_column as i32,
                            tile.column as i32 - tile.source.subtile_column as i32
                        );
                        assert_eq!(
                            ground.row as i32 - ground.source.subtile_row as i32,
                            tile.row as i32 - tile.source.subtile_row as i32
                        );
                    }
                }
            }
            support(&mut mesh, map, &cells, &g, None);
            let raised = cells
                .iter()
                .enumerate()
                .filter(|(_, t)| deck(map, &t.source))
                .map(|(i, _)| i)
                .collect::<Vec<_>>();
            assert!(raised.len() > 20, "{map}: genuine external pier drawing");
            for &i in &raised {
                let h = heights(map, &cells, &g, i, &levels);
                for (a, b) in [(0, 1), (1, 2), (2, 3), (3, 0)] {
                    assert!(
                        (h[a] - h[b]).abs() <= HEIGHT / 4. + 0.0001,
                        "{map}: shore grade must remain gradual"
                    );
                }
                if i % w + 1 < w && deck(map, &cells[i + 1].source) {
                    let east = heights(map, &cells, &g, i + 1, &levels);
                    assert_eq!([h[3], h[2]], [east[0], east[1]], "{map}: east join");
                }
                if i + w < cells.len() && deck(map, &cells[i + w].source) {
                    let south = heights(map, &cells, &g, i + w, &levels);
                    assert_eq!([h[1], h[2]], [south[0], south[3]], "{map}: south join");
                }
            }
            if map=="Route20" {
                assert_eq!(raised.iter().filter(|&&i|mesh.footing_heights[i]==0.).count(),32,
                    "the complete Seafoam cave approach stays at sea-level ground");
                assert_eq!(raised.iter().filter(|&&i|mesh.footing_heights[i]==HEIGHT).count(),32,
                    "the separate offshore bridge retains its raised deck");
            }
            for &i in &raised {
                assert!(mesh.footing_heights[i] <= HEIGHT);
                assert!(if map=="Route20" {mesh.footing_heights[i]>=0.} else {mesh.footing_heights[i]>0.},
                    "{map}: ordinary crossings retain their raised support");
            }
            for (i, tile) in cells.iter().enumerate() {
                if !deck(map, &tile.source) && !entry(map, &tile.source) {
                    assert_eq!(
                        mesh.footing_heights[i], 0.,
                        "{map}: water and gray rim remain low"
                    );
                }
            }
            let mut claimed = vec![false; w * h];
            append(&mut mesh, map, &cells, &g, &mut claimed, None, None,[0,0]);
            assert!(
                mesh.solid.positions.iter().any(|p| p[1] < 0.),
                "{map}: pilings extend into water"
            );
            assert!(
                mesh.solid.positions.iter().any(|p| p[1] >= HEIGHT - 0.03),
                "{map}: raised beam"
            );
            if map == "CeruleanGym" {
                let x = 8;
                assert_eq!(mesh.footing_heights[29 * w + x], HEIGHT * 0.625);
                assert_eq!(mesh.footing_heights[30 * w + x], HEIGHT * 0.375);
                assert_eq!(mesh.footing_heights[31 * w + x], HEIGHT * 0.125);
            } else {
                assert!(mesh.textured.positions.iter().any(|p| p[1] == HEIGHT));
                assert!(
                    mesh.textured
                        .positions
                        .iter()
                        .any(|p| p[1] == crate::profile::WATER_HEIGHT)
                );
            }
            for (i, tile) in cells.iter().enumerate() {
                if mesh.authored_cells[i] != Some("outdoor/elevated-pier-edge") {
                    continue;
                }
                let (w, e, n, _) = g.bounds(i % g.width, i / g.width);
                let edge = heights(map, &cells, &g, i - g.width, &levels);
                for (x, expected) in [(w, edge[1]), (e, edge[2])] {
                    let actual = mesh
                        .solid
                        .positions
                        .iter()
                        .filter(|p| (p[0] - x).abs() < 0.0001 && (p[2] - n - 1.2).abs() < 0.0001)
                        .map(|p| p[1])
                        .fold(f32::NEG_INFINITY, f32::max);
                    assert!(
                        (actual - expected).abs() < 0.0001,
                        "{map}: the timber face must follow the ramp instead of floating above it"
                    );
                }
                let _ = tile;
            }
            let old_support = mesh.footing_heights.clone();
            let old_solid = mesh.solid.positions.clone();
            let old_indices = mesh.solid.indices.len();
            let old_textured = mesh.textured.positions.clone();
            let old_flowers = mesh.animated_solid.positions.clone();
            let edge = crystal_render_api::VisualJumpLedge {
                start: IVec2::new(0, h as i32 / 2),
                end: IVec2::new(w as i32, h as i32 / 2),
                descent: IVec2::Y,
            };
            terraces::finish_on_map(&mut mesh, map, &cells, &g, [0, 0], &[edge]);
            for tri in mesh.solid.indices[old_indices..].chunks_exact(3) {
                let p = tri
                    .iter()
                    .map(|i| Vec3::from_array(mesh.solid.positions[*i as usize]))
                    .collect::<Vec<_>>();
                let cross = (p[1] - p[0]).cross(p[2] - p[0]);
                if cross.length() > 0.00001 {
                    assert!(
                        cross.dot(Vec3::from_array(mesh.solid.normals[tri[0] as usize])) > 0.,
                        "{map}: shore backing faces the water"
                    );
                }
            }
            for &i in &raised {
                assert_eq!(
                    mesh.footing_heights[i], old_support[i],
                    "{map}: terrace must not bend a pier"
                );
            }
            assert_eq!(
                &mesh.solid.positions[..old_solid.len()],
                old_solid.as_slice(),
                "{map}: deck/piling geometry stays level"
            );
            assert_eq!(
                &mesh.textured.positions[..old_textured.len()],
                old_textured.as_slice(),
                "{map}: level water and ramps"
            );
            assert_eq!(
                &mesh.animated_solid.positions[..old_flowers.len()],
                old_flowers.as_slice(),
                "{map}: roots stay in soil"
            );
            if map == "Route12" {
                assert!(!old_flowers.is_empty());
                assert!(old_flowers.iter().any(|v| v[1] > HEIGHT + 1.));
                assert!(
                    mesh.authored_cells
                        .iter()
                        .any(|label| *label == Some("outdoor/pier-ground-island"))
                );
            }
            for &i in &raised {
                for field in 0..5 {
                    let mut changed = tiles[i].source.clone();
                    match field {
                        0 => changed.tileset_id = "custom".into(),
                        1 => changed.metatile_id ^= 0x80,
                        2 => changed.tile_index ^= 0x80,
                        3 => changed.subtile_column = 4,
                        _ => changed.subtile_row = 4,
                    }
                    assert!(
                        !deck(map, &changed),
                        "{map}: custom source must not acquire raised support"
                    );
                }
            }
        }
    }
}
