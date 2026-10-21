//! Mountain relief inside permanent outdoor rock footprints. Accessible
//! mineral tops follow the same surface; paths, trees and caves keep their seats.
use super::*;
use bevy::math::Vec2;

const REACH: i32 = 6;
const RELIEF_SLOPE: f32 = 0.85;

fn source_point(position:[f32;3],uv:[f32;2],g:&GridGeometry,origin:[i32;2],
    regions:&[crystal_render_api::VisualSourceRegion])->Vec2 {
    super::mineral_coords::point(position,uv,g,origin,regions)
}

/// A raised object's old roof/card top does not close adjoining mineral.
/// Its standing ground still sets the lower end of that geological face.
pub(super) fn object_floor_height(source: &VisualTileSource, shape: CellShape,
    support: Option<f32>, tile_height: f32) -> Option<f32> {
    matches!(shape, CellShape::RaisedTop {solid,..} if solid != SolidKind::Bank)
        .then(||support.filter(|h|h.is_finite())
            .unwrap_or_else(||support_height(source,tile_height)))
}

pub(super) fn walking_surface(frame: &VisualWorldFrame, cell: bevy::prelude::UVec2) -> bool {
    let point = frame.grid_origin + cell.as_ivec2();
    enabled(&frame.map_id) && frame.terrace_datum.as_ref().is_some_and(|datum|
        datum.walkable_mineral.binary_search_by_key(&(point.y,point.x),|p|(p.y,p.x)).is_ok())
}

/// Within-cell contact on the same source grid as the mountain cap. Only
/// published walking mineral needs this correction; ordinary paths, water
/// and stairs retain their existing support policies.
pub(super) fn support_correction(frame: &VisualWorldFrame, cell: bevy::prelude::UVec2,
    foot: Vec2,resolved_heights:&[f32]) -> f32 {
    if !enabled(&frame.map_id) || frame.tiles.len() != (frame.grid_size.x*frame.grid_size.y) as usize {return 0.;}
    if !walking_surface(frame,cell) {return 0.;}
    let (width,height) = (frame.grid_size.x as i32,frame.grid_size.y as i32);
    let near_portal=(cell.y as i32-4..=cell.y as i32+4).any(|z|
        (cell.x as i32-4..=cell.x as i32+4).any(|x| {
            if x<0 || z<0 || x>=width || z>=height {return false;}
            let s=&frame.tiles[(z*width+x) as usize].source;
            s.subtile_row==2 && matches!(s.metatile_id,0x06|0x70|0x73)
                && matches!(s.tile_index,0x46|0x45|0x48)
        }));
    let resolved_shapes=near_portal.then(|| {
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let mut shapes=cells.iter().map(|tile|shape_for_source_on_map(
            frame.scenery_map_at(tile.column,tile.row),&tile.source)).collect::<Vec<_>>();
        crate::rock_platform::resolve_rock_platform_tiers(&cells,&mut shapes,width as usize);
        crate::elevation::resolve_authored_mountain_tiers(&mut shapes,width as usize,height as usize);
        shapes
    });
    let g=GridGeometry {width:width as usize,height:height as usize,
        tile_width:frame.tile_size.x,tile_height:frame.tile_size.y,origin_x:0.,origin_z:0.};
    let portal_roofs=if let Some(shapes)=&resolved_shapes {
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        super::mountain_portals::sites(&cells,shapes,&g)
    } else {Vec::new()};
    let source=|i:usize| {
        let tile=&frame.tiles[i];
        (&tile.source,resolved_shapes.as_ref().map_or_else(||
            shape_for_source_on_map(frame.scenery_map_at(tile.column,tile.row),&tile.source),|s|s[i]))
    };
    let heights=|i:usize|resolved_heights.get(i).copied().map(|height|
        height-datum_offset(&g,frame.grid_origin.to_array(),frame.terrace_datum.as_deref(),
            Vec2::new((i%g.width) as f32+0.5,(i/g.width) as f32+0.5)));
    let corner = |x: i32,z: i32| {
        let mut distance = REACH as f32;
        for dz in -REACH..REACH {for dx in -REACH..REACH {
            let (cx,cz) = (x+dx,z+dz);
            if cx<0 || cz<0 || cx>=width || cz>=height {continue;}
            let (s,shape)=source((cz*width+cx) as usize);
            if permanent(s,shape) {continue;}
            let px = (-(dx as f32)-1.).max(dx as f32).max(0.);
            let pz = (-(dz as f32)-1.).max(dz as f32).max(0.);
            distance = distance.min(px.hypot(pz));
        }}
        let point=Vec2::new(x as f32,z as f32);
        geometry_corner(point,distance,&g,frame.grid_origin.to_array(),&frame.source_regions,
            frame.terrace_datum.as_deref(),&portal_roofs,&source,&heights)
    };
    let (x,z) = (cell.x as i32,cell.y as i32);
    let [a,b,c,d] = [corner(x,z),corner(x+1,z),corner(x,z+1),corner(x+1,z+1)];
    let extent = frame.grid_size.as_vec2()*frame.tile_size;
    let local = Vec2::new((foot.x-frame.center.x+extent.x*0.5)/frame.tile_size.x,
        (frame.center.y+extent.y*0.5-foot.y)/frame.tile_size.y) - cell.as_vec2();
    let (u,v) = (local.x.clamp(0.,1.),local.y.clamp(0.,1.));
    let surface = if u<=v {a+(c-a)*v+(d-c)*u} else {a+(b-a)*u+(d-b)*v};
    surface-(a+d)*0.5
}

fn datum_offset(g:&GridGeometry,origin:[i32;2],datum:Option<&crystal_render_api::VisualTerraceDatum>,
    point:Vec2)->f32 {
    super::terrace_datum::sample(datum,point+Vec2::new(origin[0] as f32,origin[1] as f32))
        .unwrap_or(0.)*crate::profile::JUMP_LEDGE_HEIGHT*g.tile_height/SOURCE_TILE_HEIGHT
}

fn corner_floor<'a>(point:Vec2,g:&GridGeometry,
    source:&impl Fn(usize)->(&'a VisualTileSource,CellShape),heights:&impl Fn(usize)->Option<f32>)->f32 {
    let mut floor=0.0_f32;
    for z in point.y.floor() as i32-REACH..=point.y.floor() as i32+REACH {
    for x in point.x.floor() as i32-REACH..=point.x.floor() as i32+REACH {
        if x<0 || z<0 || x>=g.width as i32 || z>=g.height as i32 {continue;}
        let low=Vec2::new(x as f32,z as f32);
        let weight=(1.-point.distance(point.clamp(low,low+Vec2::ONE))/REACH as f32).clamp(0.,1.);
        if weight==0. {continue;}
        let i=z as usize*g.width+x as usize;
        let (s,shape)=source(i);
        if permanent(s,shape) {continue;}
        let ground=heights(i).filter(|h|h.is_finite()).unwrap_or_else(||
            object_floor_height(s,shape,None,g.tile_height)
                .unwrap_or_else(||shape.surface_height(g.tile_height)));
        floor=floor.max(ground*weight);
    }}
    floor
}

fn geometry_corner<'a>(point:Vec2,distance:f32,g:&GridGeometry,origin:[i32;2],
    regions:&[crystal_render_api::VisualSourceRegion],datum:Option<&crystal_render_api::VisualTerraceDatum>,
    portals:&[super::mountain_portals::Site],source:&impl Fn(usize)->(&'a VisualTileSource,CellShape),
    heights:&impl Fn(usize)->Option<f32>)->f32 {
    let mut structural=portals.iter().map(|p|p.body_rise(point,g.tile_height)).fold(0.,f32::max);
    let own_offset=datum_offset(g,origin,datum,point);
    for z in point.y.floor() as i32-4..=point.y.floor() as i32+4 {
        for x in point.x.floor() as i32-4..=point.x.floor() as i32+4 {
            if x<0 || z<0 || x>=g.width as i32 || z>=g.height as i32 {continue;}
            let (s,shape)=source(z as usize*g.width+x as usize);
            if s.tile_index!=0x14 || !matches!(shape,CellShape::Water
                |CellShape::PlaneAt {height:crate::profile::WATER_HEIGHT}) {continue;}
            let water=datum_offset(g,origin,datum,Vec2::new(x as f32+0.5,z as f32+0.5));
            if water<=0. {continue;}
            let low=Vec2::new(x as f32,z as f32);
            let weight=(1.-point.distance(point.clamp(low,low+Vec2::ONE))/4.).clamp(0.,1.);
            structural=structural.max((water-own_offset).max(0.)*weight*weight*(3.-2.*weight));
        }
    }
    let ground=|i:usize| {
        if portals.iter().any(|p|p.owns_cell(i%g.width,i/g.width)) {Some(0.)}
        else {heights(i)}
    };
    super::mountain_flanks::height(point,origin,regions,distance,g.tile_height)
        +corner_floor(point,g,source,&ground)+structural
}

pub(super) fn enabled(map: &str) -> bool {
    new_bark::ordinary_lawn_map(map)
        || matches!(map, "Route10North" | "Route10South" | "CinnabarIsland" | "BattleTowerOutside")
}

/// A body reveal can see beneath the closed mineral silhouette. Continue
/// nearby authenticated dry ground there, rather than exposing the horizon.
/// This hidden base supplies presentation only, never walking support.
pub(super) fn append_reveal_floor(mesh: &mut TerrainMeshData, map: &str,
    cells: &[&VisualTile], shapes: &[CellShape], g: &GridGeometry, origin: [i32;2]) {
    append_family_reveal_floor(mesh,map,cells,shapes,g,origin,false,&[]);
}
pub(super) fn append_reveal_floor_with_source_regions(mesh:&mut TerrainMeshData,map:&str,
    cells:&[&VisualTile],shapes:&[CellShape],g:&GridGeometry,origin:[i32;2],
    regions:&[crystal_render_api::VisualSourceRegion]) {
    append_family_reveal_floor(mesh,map,cells,shapes,g,origin,false,regions);
}

/// Kanto canyon backing runs after terraces, so its neighboring walking
/// support is final. Volcanic rock retains its separate source paint domain.
pub(super) fn append_kanto_reveal_floor(mesh:&mut TerrainMeshData,map:&str,
    cells:&[&VisualTile],shapes:&[CellShape],g:&GridGeometry,origin:[i32;2]) {
    append_family_reveal_floor(mesh,map,cells,shapes,g,origin,true,&[]);
}
pub(super) fn append_kanto_reveal_floor_with_source_regions(mesh:&mut TerrainMeshData,map:&str,
    cells:&[&VisualTile],shapes:&[CellShape],g:&GridGeometry,origin:[i32;2],
    regions:&[crystal_render_api::VisualSourceRegion]) {
    append_family_reveal_floor(mesh,map,cells,shapes,g,origin,true,regions);
}

fn append_family_reveal_floor(mesh:&mut TerrainMeshData,map:&str,
    cells:&[&VisualTile],shapes:&[CellShape],g:&GridGeometry,origin:[i32;2],kanto:bool,
    regions:&[crystal_render_api::VisualSourceRegion]) {
    if !enabled(map) || g.width==0 || g.height==0 || cells.len()!=g.width*g.height
        || shapes.len()!=cells.len()
        || !mesh.solid.uvs.iter().any(|uv|uv[0]< -9000. && uv[1]< -30000.) {return;}
    let eligible=cells.iter().enumerate().map(|(i,cell)| {
        let owned=mesh.authored_cells.get(i).copied().flatten()==Some("dungeon:connected-rock-terrain");
        let native_border=owned && matches!(shapes[i],
            CellShape::RaisedTop{height,solid:SolidKind::Bank}|CellShape::LedgeBand{height,..}
                if height>=crate::profile::MOUNTAIN_CLIFF_HEIGHT);
        (if kanto {cell.source.tileset_id.as_ref()=="kanto"}
            else {matches!(cell.source.tileset_id.as_ref(),"johto"|"johto_modern")})
            && (permanent(&cell.source,shapes[i]) || native_border)
            && matches!(mesh.authored_cells.get(i).copied().flatten(),
                None|Some("dungeon:connected-rock-terrain"))})
        .collect::<Vec<_>>();
    let mut donors=vec![None;cells.len()];
    let mut queue=std::collections::VecDeque::new();
    for (i,cell) in cells.iter().enumerate() {
        if !eligible[i] {continue;}
        let (x,z)=((i%g.width) as isize,(i/g.width) as isize);
        let mut donor=None;
        for dz in -2isize..=2 {for dx in -2isize..=2 {
            let (nx,nz)=(x+dx,z+dz);
            if nx<0 || nz<0 || nx>=g.width as isize || nz>=g.height as isize {continue;}
            let j=nz as usize*g.width+nx as usize;
            let source=&cells[j].source;
            let ground=if kanto {matches!(source.tile_index,0x23|0x39|0x2c)
                || (source.metatile_id==0x55 && source.tile_index==0x5b)}
                else {matches!(source.tile_index,0x05|0x06)};
            let claim=mesh.authored_cells.get(j).copied().flatten();
            let owned_paving=kanto && source.metatile_id==0x55 && source.tile_index==0x5b
                && claim==Some("outdoor/kanto-brick-path");
            if source.tileset_id!=cell.source.tileset_id || !ground
                || shapes[j]!=CellShape::Flat || (claim.is_some() && !owned_paving) {continue;}
            let distance=dx*dx+dz*dz;
            if donor.is_none_or(|(d,_)|distance<d) {donor=Some((distance,j));}
        }}
        if let Some((_,ground))=donor {donors[i]=Some(ground);queue.push_back(i);}
    }
    // A wide verified rock body can be revealed several cells from its edge.
    // Continue only its authenticated neighboring floor through that connected
    // body; never cross water, another atlas, or a separately claimed object.
    while let Some(i)=queue.pop_front() {
        let (x,z)=(i%g.width,i/g.width);
        for j in [x.checked_sub(1).map(|x|z*g.width+x),
            (x+1<g.width).then_some(i+1),z.checked_sub(1).map(|z|z*g.width+x),
            (z+1<g.height).then_some(i+g.width)].into_iter().flatten() {
            if eligible[j] && donors[j].is_none()
                && cells[j].source.tileset_id==cells[i].source.tileset_id {
                donors[j]=donors[i];queue.push_back(j);
            }
        }
    }
    let mut caps=Vec::new();
    let cap_vertices=mesh.mineral_cap_ranges.as_ref().map(|ranges| {
        let mut selected=vec![false;mesh.solid.positions.len()];
        for range in ranges {selected[range.clone()].fill(true);}
        selected
    });
    for indices in mesh.solid.indices.chunks_exact(3) {
        let ids=[indices[0] as usize,indices[1] as usize,indices[2] as usize];
        if cap_vertices.as_ref().is_some_and(|selected|ids.iter().any(|&i|!selected[i])) {continue;}
        if !ids.iter().all(|&i|mesh.solid.uvs.get(i)
            .is_some_and(|uv|uv[0]< -9000. && uv[1]< -30000.)) {continue;}
        if kanto && ids.iter().any(|&i|mesh.solid.uvs[i][1]< -50000.) {continue;}
        let p=ids.map(|i|Vec3::from_array(mesh.solid.positions[i]));
        if (p[1]-p[0]).cross(p[2]-p[0]).y<=0.0001 {continue;}
        // Source UVs retain the unsculpted cell identity. The actual positions
        // retain its rounded boundary, which this backing must never exceed.
        let uv=ids.map(|i|Vec2::from_array(mesh.solid.uvs[i])).into_iter().sum::<Vec2>()/3.;
        let center=p.into_iter().sum::<Vec3>()/3.;
        let point=source_point(center.to_array(),uv.to_array(),g,origin,regions);
        let (x,z)=(point.x.floor() as isize,point.y.floor() as isize);
        if x<0 || z<0 || x>=g.width as isize || z>=g.height as isize {continue;}
        let cell=z as usize*g.width+x as usize;
        let Some(ground)=donors[cell] else {continue;};
        // Continue the neighboring ground at its own seat. The mineral cell's
        // standing height is its cap; using that datum floats this backing
        // through the recessed wall as an exposed horizontal shelf.
        let support=mesh.footing_heights.get(ground).copied().filter(|h|h.is_finite()).unwrap_or(0.);
        let seat_bias=g.tile_height*0.01/SOURCE_TILE_HEIGHT;
        let floor=if support>0.02 {support-seat_bias}else{support};
        let clearance=g.tile_height/SOURCE_TILE_HEIGHT;
        let clip_height=floor+clearance;
        // Keep a strict interior gap. Clipping at the floor itself makes its
        // edge touch descending rock, exposing a pale line in the depth pass.
        // Require one source pixel of rock above this hidden floor. A cap
        // merely seating at walking height needs no backing through its rim.
        let mut clipped=Vec::with_capacity(4);
        for edge in 0..3 {
            let (a,b)=(p[edge],p[(edge+1)%3]);
            let (inside_a,inside_b)=(a.y>=clip_height,b.y>=clip_height);
            if inside_a {clipped.push(a);}
            if inside_a!=inside_b {clipped.push(a.lerp(b,(clip_height-a.y)/(b.y-a.y)));}
        }
        for i in 1..clipped.len().saturating_sub(1) {
            let triangle=[clipped[0],clipped[i],clipped[i+1]]
                .map(|p|[p.x,floor,p.z]);
            caps.push((ground,triangle));
        }
    }
    new_bark::append_rock_reveal_caps(&mut mesh.solid,map,cells,g,origin,&caps);
}

/// These native corner drawings contain mineral courses, not planted trees.
/// Match their actual ink so custom planted art keeps the existing flat seat.
pub(super) fn native_rock_corner(source: &VisualTileSource) -> bool {
    if !matches!(source.tileset_id.as_ref(), "johto" | "johto_modern")
        || source.subtile_column >= 4 || source.subtile_row >= 4 {return false;}
    let drawing = match source.metatile_id {
        0x6a => [0x3c,0x2b,0x2c,0x2c,0x2b,0x3b,0x3c,0x3c,
            0x3b,0x3b,0x3c,0x3c,0x3b,0x3b,0x3c,0x3c],
        0x6c => [0x3b,0x3b,0x3c,0x3c,0x3b,0x3b,0x3c,0x3c,
            0x3b,0x4b,0x4c,0x4c,0x4b,0x4c,0x4c,0x4c],
        _ => return false,
    };
    source.tile_index == drawing[source.subtile_row as usize*4+source.subtile_column as usize]
}

pub(super) fn permanent(source: &VisualTileSource, shape: CellShape) -> bool {
    // The complete native entrance producer owns these two-by-two mouths.
    // Leave a real opening in the relief field above their warp footing.
    if source.metatile_id == 0x73
        && source.subtile_row >= 2
        && ((source.tileset_id.as_ref() == "johto" && source.subtile_column < 2)
            || (source.tileset_id.as_ref() == "johto_modern" && source.subtile_column >= 2))
    {
        return false;
    }
    if source.tileset_id.as_ref() == "kanto"
        && source.metatile_id == 0x06
        && source.subtile_row >= 2
        && source.subtile_column < 2
    {
        return false;
    }
    let family = match source.tileset_id.as_ref() {
        "johto" | "johto_modern" => {
            source.metatile_id == 0x0a
                || ((0x68..=0x73).contains(&source.metatile_id)
                    && (!matches!(source.metatile_id, 0x6a | 0x6c)
                        || native_rock_corner(source)))
        }
        "kanto" => crate::kanto_cliff::kanto_cliff_shape(source).is_some_and(|native| {
            native.surface_height(SOURCE_TILE_HEIGHT) >= crate::kanto_cliff::KANTO_CLIFF_HEIGHT
        }),
        "battle_tower_outside" => crate::battle_tower::cliff_source(source),
        _ => false,
    };
    family
        && matches!(shape, CellShape::RaisedTop { solid: SolidKind::Bank, height }
        | CellShape::LedgeBand { height, .. } if height >= crate::profile::MOUNTAIN_CLIFF_HEIGHT)
}

struct Relief {
    width: usize,
    height: usize,
    peaks: Vec<f32>,
    geometry_peaks: Vec<f32>,
    #[cfg(test)]
    distances: Vec<f32>,
}

// Accessible mineral retains the source grid used by actor support. A shared
// corner field fades this constraint beyond walking plots, allowing broad
// inward slopes while preserving complete walking triangles.
fn walking_grid_weights(g: &GridGeometry, origin: [i32; 2],
    datum: Option<&crystal_render_api::VisualTerraceDatum>) -> Vec<f32> {
    let mut weights = vec![0.; (g.width + 1) * (g.height + 1)];
    let Some(datum) = datum else { return weights; };
    for plot in datum.walkable_mineral.iter() {
        constrain_grid_plot(&mut weights, g, plot.x - origin[0], plot.y - origin[1]);
    }
    weights
}

fn constrain_grid_plot(weights: &mut [f32], g: &GridGeometry, x: i32, z: i32) {
    for dz in -2..=3 { for dx in -2..=3 {
        let (cx, cz) = (x + dx, z + dz);
        if cx < 0 || cz < 0 || cx > g.width as i32 || cz > g.height as i32 { continue; }
        let distance = (-(dx as f32)).max(dx as f32 - 1.).max(0.)
            .hypot((-(dz as f32)).max(dz as f32 - 1.).max(0.));
        let t = (1. - distance / 2.).clamp(0., 1.);
        let weight = t * t * (3. - 2. * t);
        let i = cz as usize * (g.width + 1) + cx as usize;
        weights[i] = f32::max(weights[i], weight);
    }}
}

fn sample_grid(weights: &[f32], g: &GridGeometry, point: Vec2) -> f32 {
    if point.x < 0. || point.y < 0. || point.x > g.width as f32 || point.y > g.height as f32 {
        return 0.;
    }
    let (x, z) = ((point.x.floor() as usize).min(g.width - 1),
        (point.y.floor() as usize).min(g.height - 1));
    let (u, v) = (point.x - x as f32, point.y - z as f32);
    let i = z * (g.width + 1) + x;
    let a = weights[i] + (weights[i + 1] - weights[i]) * u;
    let b = weights[i + g.width + 1] + (weights[i + g.width + 2] - weights[i + g.width + 1]) * u;
    a + (b - a) * v
}

fn local_rock_top(cells: &[&VisualTile], shapes: &[CellShape], g: &GridGeometry,
    point: Vec2) -> Option<f32> {
    [Vec2::new(-0.0001,-0.0001),Vec2::new(0.0001,-0.0001),
        Vec2::new(-0.0001,0.0001),Vec2::new(0.0001,0.0001)].into_iter().filter_map(|delta| {
        let p=(point+delta).floor().as_ivec2();
        if p.x<0 || p.y<0 || p.x>=g.width as i32 || p.y>=g.height as i32 {return None;}
        let i=p.y as usize*g.width+p.x as usize;
        permanent(&cells[i].source,shapes[i]).then(||shapes[i].surface_height(g.tile_height))
    }).reduce(f32::max)
}

#[derive(Clone,Copy,Debug)]
struct MineralColumn {
    source:Vec2,
    top:f32,
    ridge:f32,
    floor:f32,
    cap:bool,
}

fn column_key(columns:&std::collections::HashMap<[i64;2],MineralColumn>,point:Vec2)->[i64;2] {
    let key=point.to_array().map(|v|(v*600.).round() as i64);
    if columns.contains_key(&key) {return key;}
    // World-registered float UVs can straddle a rounding boundary along one
    // chipped wall. Weld only neighboring subpixel columns, never separate
    // source lattice samples, and use one representative for the whole fit.
    let mut nearest=None::<(f32,[i64;2])>;
    for dz in -1..=1 {for dx in -1..=1 {
        let k=[key[0]+dx,key[1]+dz];
        let Some(c)=columns.get(&k) else {continue;};
        let distance=c.source.distance_squared(point);
        if distance<0.0025_f32.powi(2) && nearest.is_none_or(|(old,_)|distance<old) {
            nearest=Some((distance,k));
        }
    }}
    nearest.map_or(key,|(_,k)|k)
}

// Cap ownership preserves the original paint calculation. Wall-only source
// columns also need their complete extent; dividing each height ring by a
// nominal native course leaves chipped ribs beneath the geological surface.
fn mineral_columns(mesh: &TerrainMeshData, cells: &[&VisualTile], shapes: &[CellShape],
    g: &GridGeometry, origin: [i32; 2], datum: Option<&crystal_render_api::VisualTerraceDatum>,
    regions:&[crystal_render_api::VisualSourceRegion])
    -> std::collections::HashMap<[i64; 2], MineralColumn> {
    let mut columns = std::collections::HashMap::<[i64;2],MineralColumn>::new();
    let ranges=mesh.mineral_cap_ranges.as_deref().unwrap_or(&[]);
    let portal=|i:usize|mesh.mountain_portals.iter().any(|p|p.vertices.contains(&i));
    let terrain_offset = |point: Vec2| {
        super::terrace_datum::sample(datum, point + Vec2::new(origin[0] as f32, origin[1] as f32))
            .unwrap_or(0.) * crate::profile::JUMP_LEDGE_HEIGHT * g.tile_height / SOURCE_TILE_HEIGHT
    };
    let eligible = cells.iter().enumerate().map(|(i,c)|
        permanent(&c.source,shapes[i])).collect::<Vec<_>>();
    let mut region = vec![usize::MAX; cells.len()];
    let mut ridges = Vec::<f32>::new();
    for start in 0..cells.len() {
        if !eligible[start] || region[start] != usize::MAX { continue; }
        let id = ridges.len();
        ridges.push(0.);
        region[start] = id;
        let mut queue = std::collections::VecDeque::from([start]);
        while let Some(i) = queue.pop_front() {
            let (x,z) = (i % g.width,i / g.width);
            ridges[id] = ridges[id].max(shapes[i].surface_height(g.tile_height)
                + terrain_offset(Vec2::new(x as f32+0.5,z as f32+0.5)));
            for next in [x.checked_sub(1).map(|_|i-1),
                (x+1<g.width).then_some(i+1),
                z.checked_sub(1).map(|_|i-g.width),
                (z+1<g.height).then_some(i+g.width)].into_iter().flatten() {
                if eligible[next] && region[next] == usize::MAX {
                    region[next] = id;
                    queue.push_back(next);
                }
            }
        }
    }
    let touching = |point: Vec2| {
        [Vec2::new(-0.0001,-0.0001),Vec2::new(0.0001,-0.0001),
            Vec2::new(-0.0001,0.0001),Vec2::new(0.0001,0.0001)].map(|offset| {
            let c = (point+offset).floor().as_ivec2();
            (c.x>=0 && c.y>=0 && c.x<g.width as i32 && c.y<g.height as i32)
                .then(||c.y as usize*g.width+c.x as usize)
        })
    };
    // A protected upright edge can be taller than the nearby cap. Read the
    // actual geometry as well as the source datum, including its wall tops.
    for (i,(position,uv)) in mesh.solid.positions.iter().zip(&mesh.solid.uvs).enumerate() {
        if uv[0]>=-9000. {continue;}
        let point = source_point(*position,*uv,g,origin,regions);
        for j in touching(point).into_iter().flatten() {
            if region[j]!=usize::MAX {
                ridges[region[j]]=ridges[region[j]].max(position[1]+terrain_offset(point));
            }
        }
        let target=Vec2::new(g.origin_x+point.x*g.tile_width,g.origin_z+point.y*g.tile_height);
        if portal(i) || target.distance(Vec2::new(position[0],position[2]))>g.tile_width.max(g.tile_height)*2. {
            continue;
        }
        let key=column_key(&columns,point);
        let c=columns.entry(key).or_insert(MineralColumn {source:point,top:position[1],ridge:0.,floor:position[1],cap:false});
        c.top=c.top.max(position[1]);c.floor=c.floor.min(position[1]);
    }
    for i in ranges.iter().flat_map(|range|range.clone()) {
        if portal(i) {continue;}
        let point = source_point(mesh.solid.positions[i],mesh.solid.uvs[i],g,origin,regions);
        let mut upper = 0.0_f32;
        let mut floor = 0.0_f32;
        for j in touching(point).into_iter().flatten() {
            let height = shapes[j].surface_height(g.tile_height);
            if region[j]!=usize::MAX {upper=upper.max(ridges[region[j]]);}
            else {
                floor=floor.max(mesh.footing_heights.get(j).copied().filter(|h|h.is_finite())
                    .unwrap_or_else(||object_floor_height(&cells[j].source,shapes[j],None,g.tile_height)
                        .unwrap_or(height)));
            }
        }
        if upper<=0. {continue;}
        // Terraces are fitted later. Measure the minimum in final world
        // height, then remove this point's future offset exactly once.
        upper -= terrain_offset(point);
        let key=column_key(&columns,point);
        let current = mesh.solid.positions[i][1];
        let column=columns.entry(key).or_insert(MineralColumn {source:point,top:current,ridge:upper,floor,cap:true});
        if column.cap {
            column.top=column.top.min(current);
            column.ridge=column.ridge.max(upper);
            column.floor=column.floor.max(floor);
        } else {
            *column=MineralColumn {source:point,top:current,ridge:upper,floor,cap:true};
        }
    }
    columns
}

// Different native courses can meet at one source column. Each course needs
// its own original top for the wall fit: using the lower cap for both collapses
// the upper wall rings into a fan of triangles at the shared corner.
fn face_fit_tops(mesh:&TerrainMeshData,cells:&[&VisualTile],shapes:&[CellShape],
    g:&GridGeometry,origin:[i32;2],regions:&[crystal_render_api::VisualSourceRegion],
    columns:&std::collections::HashMap<[i64;2],MineralColumn>)->Vec<f32> {
    let mut owners=vec![u32::MAX;mesh.solid.positions.len()];
    for tri in mesh.solid.indices.chunks_exact(3) {
        let ids=[tri[0] as usize,tri[1] as usize,tri[2] as usize];
        if !ids.iter().all(|&i|mesh.solid.uvs[i][0]< -9000.) {continue;}
        let source=ids.map(|i|source_point(mesh.solid.positions[i],mesh.solid.uvs[i],g,origin,regions));
        let normal=Vec3::from_array(mesh.solid.normals[ids[0]]);
        let inside=if normal.y<0.9 {Vec2::new(normal.x,normal.z).normalize_or_zero()*0.01}
            else {Vec2::ZERO};
        let owner=(source.into_iter().sum::<Vec2>()/3.-inside).floor().as_ivec2();
        if owner.x<0 || owner.y<0 || owner.x>=g.width as i32 || owner.y>=g.height as i32 {continue;}
        let cell=owner.y as usize*g.width+owner.x as usize;
        if !permanent(&cells[cell].source,shapes[cell]) {continue;}
        // A resolved terrain tier can give neighboring source courses the
        // same nominal height. Their actual wall geometry still has separate
        // upper extents, so retain the face's source owner, not that height.
        for i in ids {if owners[i]==u32::MAX {owners[i]=cell as u32;}}
    }
    let mut maxima=std::collections::HashMap::<([i64;2],u32),f32>::new();
    for (i,&owner) in owners.iter().enumerate().filter(|(_,o)|**o!=u32::MAX) {
        let point=source_point(mesh.solid.positions[i],mesh.solid.uvs[i],g,origin,regions);
        let key=(column_key(columns,point),owner);
        let top=maxima.entry(key).or_insert(mesh.solid.positions[i][1]);
        *top=top.max(mesh.solid.positions[i][1]);
    }
    let mut tops=vec![0.;owners.len()];
    for (i,&owner) in owners.iter().enumerate().filter(|(_,o)|**o!=u32::MAX) {
        let point=source_point(mesh.solid.positions[i],mesh.solid.uvs[i],g,origin,regions);
        tops[i]=maxima[&(column_key(columns,point),owner)];
    }
    tops
}

impl Relief {
    #[cfg(test)]
    fn new(
        cells: &[&VisualTile],
        shapes: &[CellShape],
        g: &GridGeometry,
        origin: [i32; 2],
    ) -> Self {
        Self::with_datum(cells,shapes,g,origin,None)
    }

    #[cfg(test)]
    fn with_datum(cells:&[&VisualTile],shapes:&[CellShape],g:&GridGeometry,
        origin:[i32;2],datum:Option<&crystal_render_api::VisualTerraceDatum>)->Self {
        Self::with_ground(cells,shapes,g,origin,datum,&[],None)
    }

    fn with_ground(cells:&[&VisualTile],shapes:&[CellShape],g:&GridGeometry,
        origin:[i32;2],datum:Option<&crystal_render_api::VisualTerraceDatum>,
        regions:&[crystal_render_api::VisualSourceRegion],heights:Option<&[f32]>)->Self {
        let rock: Vec<_> = cells.iter().zip(shapes)
            .map(|(cell,&shape)|permanent(&cell.source,shape)).collect();
        let portal_roofs=super::mountain_portals::sites(cells,shapes,g);
        let mut peaks = vec![0.; (g.width + 1) * (g.height + 1)];
        let mut geometry_peaks=vec![0.;peaks.len()];
        #[cfg(test)]
        let mut distances=vec![0.;peaks.len()];
        let source=|i:usize|(&cells[i].source,shapes[i]);
        let footing=|i:usize|heights.and_then(|h|h.get(i).copied());
        for z in 0..=g.height {
            for x in 0..=g.width {
                // Distance to the nearest real opening, including paths, stairs,
                // portals and tree seats. A viewport cut is not a mountain edge.
                let mut distance = REACH as f32;
                for dz in -REACH..REACH {
                    for dx in -REACH..REACH {
                        let (cx, cz) = (x as i32 + dx, z as i32 + dz);
                        if cx < 0 || cz < 0 || cx >= g.width as i32 || cz >= g.height as i32 {
                            continue;
                        }
                        if rock[cz as usize * g.width + cx as usize] {
                            continue;
                        }
                        let px = (-(dx as f32) - 1.).max(dx as f32).max(0.);
                        let pz = (-(dz as f32) - 1.).max(dz as f32).max(0.);
                        distance = distance.min(px.hypot(pz));
                    }
                }
                // Intersecting mineral beds give the original broad slopes
                // and unequal ridges. Keep the current paint independent of
                // this restored shape and retain the real walking openings.
                // World coordinates preserve the sculpt through camera crops.
                let wx = x as f32 + origin[0] as f32;
                let wz = z as f32 + origin[1] as f32;
                let bed = super::cave_cliff::slab(wx * 0.105 + 17., wz * 0.105 - 31.);
                let ridge = 2.5 + bed.abs() * 3.;
                #[cfg(test)]
                {distances[z*(g.width+1)+x]=distance;}
                let hill=distance.min(ridge)*RELIEF_SLOPE*g.tile_height;
                let roof=portal_roofs.iter().map(|p|p.roof_rise(Vec2::new(x as f32,z as f32),g.tile_height))
                    .fold(0.,f32::max);
                peaks[z*(g.width+1)+x]=hill+roof;
                geometry_peaks[z*(g.width+1)+x]=geometry_corner(Vec2::new(x as f32,z as f32),
                    distance,g,origin,regions,datum,&portal_roofs,&source,&footing);
            }
        }
        Self {
            width: g.width,
            height: g.height,
            peaks,
            geometry_peaks,
            #[cfg(test)]
            distances,
        }
    }

    fn height(&self, x: f32, z: f32) -> f32 {
        self.sample(&self.peaks,x,z)
    }

    fn geometry_height(&self,x:f32,z:f32)->f32 {
        self.sample(&self.geometry_peaks,x,z)
    }

    #[cfg(test)]
    fn distance(&self,x:f32,z:f32)->f32 {
        self.sample(&self.distances,x,z)
    }

    fn sample(&self,field:&[f32],x:f32,z:f32)->f32 {
        if x < 0. || z < 0. || x > self.width as f32 || z > self.height as f32 {
            return 0.;
        }
        let cx = (x.floor() as usize).min(self.width - 1);
        let cz = (z.floor() as usize).min(self.height - 1);
        let (u, v) = (x - cx as f32, z - cz as f32);
        let i = cz * (self.width + 1) + cx;
        let [a, b, c, d] = [
            field[i],
            field[i + 1],
            field[i + self.width + 1],
            field[i + self.width + 2],
        ];
        // Follow the cap mesh's northwest/southeast diagonal so a mineral
        // bed remains one plane instead of acquiring extra crossing creases.
        if u <= v {
            a + (c - a) * v + (d - c) * u
        } else {
            a + (b - a) * u + (d - b) * v
        }
    }
}

pub(super) fn finish(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    origin: [i32; 2],
    datum: Option<&crystal_render_api::VisualTerraceDatum>,
) {
    finish_with_source_regions(mesh,map,cells,shapes,g,origin,datum,&[]);
}

pub(super) fn finish_with_source_regions(mesh:&mut TerrainMeshData,map:&str,
    cells:&[&VisualTile],shapes:&[CellShape],g:&GridGeometry,origin:[i32;2],
    datum:Option<&crystal_render_api::VisualTerraceDatum>,
    regions:&[crystal_render_api::VisualSourceRegion]) {
    if !enabled(map)
        || g.width == 0
        || g.height == 0
        || !cells
            .iter()
            .zip(shapes)
            .any(|(c, &s)| permanent(&c.source, s)
                || super::terrace_datum::mountain_ponds::plateau(&c.source))
    {
        return;
    }
    let relief = Relief::with_ground(cells,shapes,g,origin,datum,regions,Some(&mesh.footing_heights));
    let mut walking_grid = walking_grid_weights(g, origin, datum);
    for (i, cell) in cells.iter().enumerate() {
        let (x, z) = (i % g.width, i / g.width);
        let portal = mesh.authored_cells.get(i).is_some_and(|claim|
            matches!(claim, Some("johto/mountain_cave" | "outdoor/kanto_mountain_cave")));
        let elevated_water = cell.source.tile_index == 0x14
            && matches!(shapes[i], CellShape::Water | CellShape::PlaneAt {height: crate::profile::WATER_HEIGHT})
            && super::terrace_datum::sample(datum,
                Vec2::new(x as f32 + origin[0] as f32 + 0.5, z as f32 + origin[1] as f32 + 0.5))
                .is_some_and(|height| height > 0.);
        if portal || elevated_water {constrain_grid_plot(&mut walking_grid, g, x as i32, z as i32);}
    }
    let flanks = super::mountain_flanks::Field::new(cells, shapes, g, origin, regions);
    let columns = mineral_columns(mesh,cells,shapes,g,origin,datum,regions);
    let fit_tops=face_fit_tops(mesh,cells,shapes,g,origin,regions,&columns);
    let mut cap_vertices=vec![false;mesh.solid.positions.len()];
    if let Some(ranges)=&mesh.mineral_cap_ranges {
        for range in ranges {cap_vertices[range.clone()].fill(true);}
    }
    let portals=std::mem::take(&mut mesh.mountain_portals);
    let _trace=crate::loading_trace::span("mountain.source_alignment",|| {
        let mut samples=Vec::new();
        for (i,(p,uv)) in mesh.solid.positions.iter().zip(&mesh.solid.uvs).enumerate() {
            if uv[0]>=-9000. {continue;}
            let point=source_point(*p,*uv,g,origin,regions);
            let key=column_key(&columns,point);
            let Some(column)=columns.get(&key).filter(|c|c.cap) else {continue;};
            let target=Vec2::new(g.origin_x+point.x*g.tile_width,g.origin_z+point.y*g.tile_height);
            let distance=target.distance(Vec2::new(p[0],p[2]));
            if samples.len()<8 || samples.last().is_some_and(|s:&(f32,usize,Vec2,(f32,f32,f32))|distance>s.0) {
                samples.push((distance,i,target,(column.top,column.ridge,column.floor)));
                samples.sort_by(|a,b|b.0.total_cmp(&a.0));samples.truncate(8);
            }
        }
        serde_json::json!({"map":map,"origin":origin,"grid":[g.width,g.height],
            "geometry_origin":[g.origin_x,g.origin_z],"tile":[g.tile_width,g.tile_height],
            "largest":samples.into_iter().map(|(d,i,target,column)|serde_json::json!({
                "distance":d,"position":mesh.solid.positions[i],"uv":mesh.solid.uvs[i],
                "target":target.to_array(),"column":[column.0,column.1,column.2]})).collect::<Vec<_>>()})
    });
    let mut portal_index=0;
    for (i, (position, uv)) in mesh
        .solid
        .positions
        .iter_mut()
        .zip(&mesh.solid.uvs)
        .enumerate()
    {
        while portals.get(portal_index).is_some_and(|p|p.vertices.end<=i) {portal_index+=1;}
        if portals.get(portal_index).is_some_and(|p|p.vertices.contains(&i)) {continue;}
        if uv[0] >= -9000. {
            continue;
        }
        // Stone UVs retain the original source point through rim sculpting.
        // Use that point on both wall and cap to keep their seams joined.
        let point=source_point(*position,*uv,g,origin,regions);
        let (x,z)=(point.x,point.y);
        // Folded native cliff courses can extend their source footprint past
        // the exposed wall plane. Anchor every mineral toe at the real ground
        // instead of lifting the entire wall along with the mountain cap.
        let support = g.tile_height * crate::profile::MOUNTAIN_CLIFF_HEIGHT / SOURCE_TILE_HEIGHT;
        let before = position[1];
        let key=column_key(&columns,point);
        let column=columns.get(&key);
        let mut paint_rise=if column.is_some_and(|c|c.cap) {0.}
            else {relief.height(x,z)*(before/support).clamp(0.,1.)};
        if let Some(column) = column {
            let old_top=column.top;
            let native_top=column.ridge;
            let fit_top=if fit_tops[i]>0. {fit_tops[i]}else{old_top};
            let span=fit_top-column.floor;
            let local=local_rock_top(cells,shapes,g,point);
            let cap_like=!column.cap && span<0.0001 && local.is_some_and(|top|before>=top*0.75);
            let floor=if cap_like {0.} else {column.floor};
            let relief_height=relief.height(x,z);
            let filled_top=native_top+relief_height;
            let paint_floor=floor.min(old_top-g.tile_height/SOURCE_TILE_HEIGHT);
            let floor = floor.min(fit_top - g.tile_height / SOURCE_TILE_HEIGHT);
            // The complete body follows one geological surface. The old
            // regional course remains paint evidence, not a height target.
            let top=relief.geometry_height(column.source.x,column.source.y)
                .max(floor+g.tile_height/SOURCE_TILE_HEIGHT);
            if (before > floor || cap_vertices[i]) && (column.cap || span>0.0001 || cap_like) {
                let fraction = if cap_vertices[i] {1.}
                    else {((before - floor) / (fit_top - floor).max(0.0001)).clamp(0., 1.)};
                // Pigment remains independent of this silhouette correction.
                if column.cap {
                    let paint_fraction=((before-paint_floor)/(old_top-paint_floor).max(0.0001)).clamp(0.,1.);
                    paint_rise=(paint_floor+(filled_top-paint_floor)*paint_fraction-before).max(0.);
                }
                // One common horizontal surface keeps upper triangles ordered.
                // The earlier height-dependent bevel cannot be combined with
                // the filled cap directly: that can fold its corner triangles.
                // Add the inward geological slope to this shared source field.
                let constraint = sample_grid(&walking_grid, g, column.source);
                // Keep the rooted toe's rounding over its bottom sixth.
                // Compressing the old mid-height bevel into a low foothill
                // leaves a projecting shoulder beneath the new upper slope.
                let blend = (fraction*6.).clamp(0.,1.);
                let target=Vec2::new(g.origin_x+column.source.x*g.tile_width,
                    g.origin_z+column.source.y*g.tile_height);
                // Repeated mineral geometry can retain the donor plot's UV
                // registration. Only undo the bounded local shoulder sculpt;
                // never drag a placed face back across the map to that donor.
                if (column.cap || local.is_some())
                    && target.distance(Vec2::new(position[0],position[2]))<=g.tile_width.max(g.tile_height)*2. {
                    position[1]=floor+(top-floor)*fraction;
                    if !cap_like {
                        position[0]+=(target.x-position[0])*blend;
                        position[2]+=(target.y-position[2])*blend;
                        let inward=flanks.inset(column.source,g)
                            *super::mountain_flanks::slope(fraction)*(1.-constraint);
                        position[0]+=inward.x*g.tile_width;
                        position[2]+=inward.y*g.tile_height;
                    }
                }
            }
        }
        let rise = paint_rise;
        if map == "CinnabarIsland" {
            if let Some(color)=mesh.solid.colors.get_mut(i) {
                *color=[0.20,0.205,0.215,color[3]];
            }
        } else if rise > 0.
            && let Some(color) = mesh.solid.colors.get_mut(i)
        {
            let grain =
                super::cave_cliff::slab((x + origin[0] as f32) * 1.7, (z + origin[1] as f32) * 1.7);
            let brown = [
                0.34 + grain * 0.08,
                0.215 + grain * 0.055,
                0.105 + grain * 0.03,
            ];
            let weight = (rise / g.tile_height).min(1.);
            for c in 0..3 {
                color[c] += (brown[c] - color[c]) * weight;
            }
        }
    }
    // A portal is a closed volume, including its dark lining. Retain useful
    // native clearance, but give short source drawings standing headroom.
    // Fit the remaining rock above the intrados to the mountain. One monotone
    // mapping keeps all shared entrance edges joined and the threshold seated.
    let seat=g.tile_height*0.015;
    for portal in &portals {
        let point_at=|i:usize| {
            let p=mesh.solid.positions[i];
            if mesh.solid.uvs[i][0]< -9000. {
                source_point(p,mesh.solid.uvs[i],g,origin,regions)
            } else {Vec2::new((p[0]-g.origin_x)/g.tile_width,(p[2]-g.origin_z)/g.tile_height)}
        };
        let roof_at=|point:Vec2| {
            relief.geometry_height(point.x,point.y).max(portal.height)
        };
        let lowest_roof=portal.vertices.clone().map(|i|roof_at(point_at(i)))
            .fold(f32::INFINITY,f32::min);
        let opening=portal.height*0.90;
        // The modeled trainer is three tiles tall. Keep the authored width and
        // leave a little space overhead; low mountain roofs still bound it.
        let clearance=opening.max((g.tile_height*3.3).min(lowest_roof*0.90));
        for i in portal.vertices.clone() {
            let p=mesh.solid.positions[i];
            if p[1]<=seat {continue;}
            let point=if mesh.solid.uvs[i][0]< -9000. {
                source_point(p,mesh.solid.uvs[i],g,origin,regions)
            } else {Vec2::new((p[0]-g.origin_x)/g.tile_width,(p[2]-g.origin_z)/g.tile_height)};
            let target=roof_at(point);
            mesh.solid.positions[i][1]=if p[1]<=opening {
                seat+(p[1]-seat)*(clearance-seat)/(opening-seat)
            }
                else if p[1]>=portal.height {target+p[1]-portal.height}
                else {clearance+(target-clearance)*(p[1]-opening)/(portal.height-opening)};
        }
    }
    if let Some(datum) = datum {
        for (i,height) in mesh.footing_heights.iter_mut().enumerate() {
            let point = bevy::prelude::IVec2::new(origin[0]+(i%g.width) as i32,
                origin[1]+(i/g.width) as i32);
            if datum.walkable_mineral.binary_search_by_key(&(point.y,point.x),|p|(p.y,p.x)).is_err() {continue;}
            let local = Vec2::new((i%g.width) as f32+0.5,(i/g.width) as f32+0.5);
            let key=column_key(&columns,local);
            if columns.get(&key).is_some_and(|c|c.cap) {
                *height = relief.geometry_height(local.x,local.y);
            }
        }
    }
    if map == "CinnabarIsland" {
        // Keep the same rock geometry and world-registered texture field, but
        // retain the island's volcanic pigment through the shared shader.
        // Otherwise the canyon finish paints warm sediment over this gray.
        for uv in &mut mesh.solid.uvs {
            if uv[0] < -9000. {
                uv[1] -= if uv[1] < -30000. { 20000. } else { 40000. };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::Vec3Swizzles;

    #[test]
    fn connected_paint_bases_cannot_split_fitted_mountain_geometry_or_footing() {
        use bevy::prelude::{IVec2,UVec2};
        let (frame,g)=fixture("johto",0x0a);
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let shapes=cells.iter().map(|c|shape_for_source(&c.source)).collect::<Vec<_>>();
        let origin=[32,-24];
        let regions=[crystal_render_api::VisualSourceRegion {map_id:"neighbor".into(),
            origin:IVec2::from_array(origin),size:UVec2::new(12,12)}];
        let mut control=TerrainMeshData {footing_heights:shapes.iter()
            .map(|s|s.surface_height(g.tile_height)).collect(),authored_cells:vec![None;144],..Default::default()};
        for z in 2..10 {for x in 2..10 {
            let (w,e,n,s)=g.bounds(x,z);
            outdoor_stone::append_canyon(&mut control.solid,
                [[w,16.,n],[w,16.,s],[e,16.,s],[e,16.,n]],Vec3::Y.to_array(),&g,origin);
            for (exposed,a,b,normal) in [
                (x==2,[w,16.,s],[w,16.,n],Vec3::NEG_X),
                (x==9,[e,16.,n],[e,16.,s],Vec3::X),
                (z==2,[w,16.,n],[e,16.,n],Vec3::NEG_Z),
                (z==9,[e,16.,s],[w,16.,s],Vec3::Z),
            ] {
                if !exposed {continue;}
                let mut low_a=a;low_a[1]=0.;let mut low_b=b;low_b[1]=0.;
                outdoor_stone::append_canyon(&mut control.solid,[low_a,a,b,low_b],normal.to_array(),&g,origin);
            }
        }}
        let mut connected=control.clone();
        for (i,uv) in connected.solid.uvs.iter_mut().enumerate() {
            if (i/4)%2==0 {
                uv[0]+=origin[0] as f32*g.tile_width;
                uv[1]+=origin[1] as f32*g.tile_height;
            }
        }
        let datum=crystal_render_api::VisualTerraceDatum {
            origin:IVec2::from_array(origin),size:UVec2::new(12,12),
            levels:(0..144).map(|i| {
                let x=(i%12) as f32;
                [x*0.25,x*0.25,(x+1.)*0.25,(x+1.)*0.25]
            }).collect::<Vec<_>>().into(),
            walkable_mineral:(2..10).flat_map(|z|(2..10).map(move |x|
                IVec2::new(origin[0]+x,origin[1]+z))).collect::<Vec<_>>().into(),
            water_barriers:Default::default(),open_water:None,
        };
        for mesh in [&mut control,&mut connected] {
            outdoor_stone::finish_rock_geometry_with_source_regions(mesh,"Route44",&cells,&shapes,
                &g,origin,Some(&datum),&regions);
            super::super::terraces::finish_with_source_regions(mesh,"Route44",&cells,&g,
                origin,&[],Some(&datum),&regions);
        }
        assert_eq!(control.solid.indices,connected.solid.indices);
        for (a,b) in control.solid.positions.iter().zip(&connected.solid.positions) {
            assert!(Vec3::from_array(*a).distance(Vec3::from_array(*b))<0.002,
                "a connected material basis cannot change the fitted surface: {a:?} != {b:?}");
        }
        for (a,b) in control.footing_heights.iter().zip(&connected.footing_heights) {
            assert!((a-b).abs()<0.002,"paint registration cannot change walking support: {a} != {b}");
        }
        for (a,b) in control.solid.colors.iter().zip(&connected.solid.colors) {
            assert!(a.iter().zip(b).all(|(a,b)|(a-b).abs()<0.0001),
                "retain the existing source pigment: {a:?} != {b:?}");
        }
    }

    #[test]
    fn repeated_mineral_faces_keep_their_placement_when_source_uvs_are_reused() {
        let (frame,g)=fixture("johto",0x0a);
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let shapes=cells.iter().map(|c|shape_for_source(&c.source)).collect::<Vec<_>>();
        let mut mesh=TerrainMeshData::default();
        for z in 2..10 {for x in 2..10 {
            let (w,e,n,s)=g.bounds(x,z);
            outdoor_stone::append_canyon(&mut mesh.solid,
                [[w,16.,n],[w,16.,s],[e,16.,s],[e,16.,n]],Vec3::Y.to_array(),&g,[0,0]);
        }}
        // A retained repeated plot has placed geometry but donor coordinates
        // in its mineral material. Those UVs must not become placement data.
        for p in &mut mesh.solid.positions {p[0]+=g.tile_width*32.;}
        outdoor_stone::retain_mineral_cap_ownership(&mut mesh);
        let before=mesh.solid.positions.clone();
        finish(&mut mesh,"Route44",&cells,&shapes,&g,[0,0],None);
        for (a,b) in before.iter().zip(&mesh.solid.positions) {
            assert_eq!([a[0],a[2]],[b[0],b[2]],"a repeated plot must keep its placed footprint");
        }
    }

    #[test]
    fn walking_and_unreachable_rock_share_one_geology_and_keep_their_paint() {
        use bevy::prelude::{IVec2,UVec2};
        let (frame,g)=fixture("johto",0x0a);
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let shapes=cells.iter().map(|c|shape_for_source(&c.source)).collect::<Vec<_>>();
        let mut mesh=TerrainMeshData {authored_cells:vec![None;cells.len()],
            footing_heights:shapes.iter().map(|s|s.surface_height(g.tile_height)).collect(),
            ..Default::default()};
        for z in 2..10 {for x in 2..10 {
            let (w,e,n,s)=g.bounds(x,z);
            outdoor_stone::append_canyon(&mut mesh.solid,
                [[w,16.,n],[w,16.,s],[e,16.,s],[e,16.,n]],Vec3::Y.to_array(),&g,[0,0]);
            for (exposed,a,b,normal) in [
                (x==2,[w,16.,s],[w,16.,n],Vec3::NEG_X),
                (x==9,[e,16.,n],[e,16.,s],Vec3::X),
                (z==2,[w,16.,n],[e,16.,n],Vec3::NEG_Z),
                (z==9,[e,16.,s],[w,16.,s],Vec3::Z),
            ] {
                if !exposed {continue;}
                let mut low_a=a;low_a[1]=0.;let mut low_b=b;low_b[1]=0.;
                outdoor_stone::append_canyon(&mut mesh.solid,[low_a,a,b,low_b],normal.to_array(),&g,[0,0]);
            }
        }}
        outdoor_stone::sculpt_rock_rims_on_map(&mut mesh,"Route44",&cells,&shapes,&g,[0,0]);
        outdoor_stone::retain_mineral_cap_ownership(&mut mesh);
        let sculpted=mesh.clone();
        let mut walking=mesh.clone();
        let datum=crystal_render_api::VisualTerraceDatum {
            origin:IVec2::ZERO,size:UVec2::splat(12),levels:vec![[0.;4];144].into(),
            walkable_mineral:(2..10).flat_map(|z|(2..10).map(move |x|IVec2::new(x,z)))
                .collect::<Vec<_>>().into(),water_barriers:Default::default(),open_water:None,
        };
        finish(&mut mesh,"Route44",&cells,&shapes,&g,[0,0],None);
        finish(&mut walking,"Route44",&cells,&shapes,&g,[0,0],Some(&datum));
        let relief=Relief::new(&cells,&shapes,&g,[0,0]);
        let mut retained_shoulders=0;
        let mut lowered_margins=0;
        let mut shared=std::collections::HashMap::new();
        for (i,(after,before)) in mesh.solid.positions.iter().zip(&sculpted.solid.positions).enumerate() {
            assert!(Vec2::new(after[0]-before[0],after[2]-before[2]).length()<g.tile_width*1.62,
                "the inward slope remains bounded inside the mineral body");
            if before[1]<=0. {
                assert_eq!([after[0],after[2]],[before[0],before[2]],"rooted ground contacts stay fixed");
            }
            let full_height=walking.solid.positions[i][1];
            assert_eq!(after[1],full_height,"walking plots follow the same complete geological height field");
            let point=source_point(*before,sculpted.solid.uvs[i],&g,[0,0],&[]);
            if sculpted.solid.normals[i][1]>0.9 && relief.distance(point.x,point.y)<0.5
                && after[1]<g.tile_height {
                lowered_margins+=1;
            }
            let key=before.map(|v|(v*10000.).round() as i64);
            if let Some(previous)=shared.insert(key,*after) {
                assert!(Vec3::from_array(previous).distance(Vec3::from_array(*after))<0.001,
                    "cap and wall copies must stay joined");
            }
            if sculpted.solid.normals[i][1]>0.9 {
                let source=source_point(*before,sculpted.solid.uvs[i],&g,[0,0],&[])*8.;
                retained_shoulders+=usize::from(Vec2::new(after[0],after[2]).distance(source)>0.5);
            }
        }
        assert!(retained_shoulders>100,"exercise sculpted upper shoulders, not just a flat interior");
        assert!(lowered_margins>20,"lower the actual outer body, including walking plots");
        assert_eq!(mesh.footing_heights,sculpted.footing_heights);
        assert_eq!(mesh.authored_cells,sculpted.authored_cells);
        assert_eq!(mesh.solid.indices,sculpted.solid.indices);
        assert_eq!(mesh.solid.uvs,sculpted.solid.uvs);
        assert_eq!(mesh.solid.colors,walking.solid.colors);
    }

    #[test]
    fn walking_grid_constraint_is_complete_and_stable_through_a_source_crop() {
        use bevy::prelude::{IVec2,UVec2};
        let (_,g)=fixture("johto",0x0a);
        let datum=crystal_render_api::VisualTerraceDatum {
            origin:IVec2::ZERO,size:UVec2::splat(12),levels:vec![[0.;4];144].into(),
            walkable_mineral:vec![IVec2::new(5,5),IVec2::new(6,5)].into(),
            water_barriers:Default::default(),open_water:None,
        };
        let whole=walking_grid_weights(&g,[0,0],Some(&datum));
        let crop=GridGeometry {width:10,height:10,..g};
        let cropped=walking_grid_weights(&crop,[1,1],Some(&datum));
        for z in 0..=32 {for x in 0..=32 {
            let p=Vec2::new(x as f32/4.+2.,z as f32/4.+2.);
            let w=sample_grid(&whole,&g,p);
            assert!((w-sample_grid(&cropped,&crop,p-Vec2::ONE)).abs()<0.00001);
            assert!((0.0..=1.0).contains(&w));
            if (5.0..=7.0).contains(&p.x) && (5.0..=6.0).contains(&p.y) {
                assert_eq!(w,1.,"every point on accessible source triangles keeps its full support constraint");
            }
        }}
        assert_eq!(sample_grid(&whole,&g,Vec2::new(2.,2.)),0.,
            "distant unreachable shoulders must not be pulled straight");
    }

    #[test]
    fn a_distant_high_course_fills_the_center_without_doubling_the_local_outer_face() {
        use bevy::prelude::{IVec2,UVec2};
        let (frame,g)=fixture("johto",0x0a);
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let mut shapes=cells.iter().map(|c|shape_for_source(&c.source)).collect::<Vec<_>>();
        for z in 6..8 {for x in 6..8 {
            shapes[z*g.width+x]=CellShape::RaisedTop {height:32.,solid:SolidKind::Bank};
        }}
        let mut mesh=TerrainMeshData {authored_cells:vec![None;cells.len()],
            footing_heights:shapes.iter().map(|s|s.surface_height(g.tile_height)).collect(),
            ..Default::default()};
        for z in 2..10 {for x in 2..10 {
            let (w,e,n,s)=g.bounds(x,z);
            let top=shapes[z*g.width+x].surface_height(g.tile_height);
            outdoor_stone::append_canyon(&mut mesh.solid,
                [[w,top,n],[w,top,s],[e,top,s],[e,top,n]],Vec3::Y.to_array(),&g,[0,0]);
            for (exposed,a,b,normal) in [
                (x==2,[w,top,s],[w,top,n],Vec3::NEG_X),
                (x==9,[e,top,n],[e,top,s],Vec3::X),
                (z==2,[w,top,n],[e,top,n],Vec3::NEG_Z),
                (z==9,[e,top,s],[w,top,s],Vec3::Z),
            ] {
                if !exposed {continue;}
                let mut low_a=a;low_a[1]=0.;let mut low_b=b;low_b[1]=0.;
                outdoor_stone::append_canyon(&mut mesh.solid,[low_a,a,b,low_b],normal.to_array(),&g,[0,0]);
            }
        }}
        let original=mesh.clone();
        let mut walking=mesh.clone();
        let datum=crystal_render_api::VisualTerraceDatum {
            origin:IVec2::ZERO,size:UVec2::splat(12),levels:vec![[0.;4];144].into(),
            walkable_mineral:(2..10).flat_map(|z|(2..10).map(move |x|IVec2::new(x,z)))
                .collect::<Vec<_>>().into(),water_barriers:Default::default(),open_water:None,
        };
        outdoor_stone::finish_rock_geometry(&mut mesh,"Route44",&cells,&shapes,&g,[0,0],None);
        outdoor_stone::finish_rock_geometry(&mut walking,"Route44",&cells,&shapes,&g,[0,0],Some(&datum));
        let at=|p:[f32;3]|original.solid.positions.iter().zip(&original.solid.normals)
            .position(|(position,normal)|*position==p && normal[1]>0.9).unwrap();
        let edge=at([16.,16.,48.]);
        let center=at([48.,32.,48.]);
        assert!((0. ..8.).contains(&mesh.solid.positions[edge][1]),
            "the outer slope starts below half of its old native course");
        assert_eq!(mesh.solid.positions[edge][1],walking.solid.positions[edge][1],
            "a walking mark must not restore the old tall source shaft");
        assert_eq!(mesh.solid.positions[center][1],walking.solid.positions[center][1],
            "the walking datum follows the geological interior");
        assert!(mesh.solid.positions[center][1]>mesh.solid.positions[edge][1]+g.tile_height*2.);
        for i in 0..original.solid.positions.len() {
            assert_eq!(mesh.solid.colors[i],walking.solid.colors[i],
                "silhouette changes retain existing native pigment at vertex {i}");
            assert_eq!(mesh.solid.uvs[i],walking.solid.uvs[i],"native material coordinates remain at vertex {i}");
        }
        assert_eq!(mesh.solid.indices.len(),walking.solid.indices.len());
        for (a,b) in mesh.solid.indices.iter().zip(&walking.solid.indices) {assert_eq!(a,b);}
        assert_eq!(mesh.footing_heights,original.footing_heights);
        assert_eq!(mesh.authored_cells,original.authored_cells);
    }

    #[test]
    fn small_mountain_rises_from_low_foothills_without_using_an_adjacent_roof_as_ground() {
        for ground in [0.,crate::profile::JUMP_LEDGE_HEIGHT] {
            let (mut frame,g)=fixture("johto",0x0a);
            let rock=frame.tiles[2*g.width+2].source.clone();
            let flat=frame.tiles[0].source.clone();
            for tile in &mut frame.tiles {
                tile.source=if (4..8).contains(&tile.column) && (4..8).contains(&tile.row) {
                    rock.clone()
                } else {flat.clone()};
            }
            let cells=frame.tiles.iter().collect::<Vec<_>>();
            let mut shapes=cells.iter().map(|c|shape_for_source(&c.source)).collect::<Vec<_>>();
            for shape in &mut shapes {
                if !matches!(shape,CellShape::RaisedTop {solid:SolidKind::Bank,..}) {
                    *shape=CellShape::PlaneAt {height:ground};
                }
            }
            shapes[6*g.width+3]=CellShape::RaisedTop {height:96.,solid:SolidKind::Building};
            let mut mesh=TerrainMeshData {authored_cells:vec![None;cells.len()],
                footing_heights:shapes.iter().map(|s|s.surface_height(g.tile_height)).collect(),
                ..Default::default()};
            mesh.footing_heights[6*g.width+3]=ground;
            for z in 4..8 {for x in 4..8 {
                let (w,e,n,s)=g.bounds(x,z);
                outdoor_stone::append_canyon(&mut mesh.solid,
                    [[w,16.,n],[w,16.,s],[e,16.,s],[e,16.,n]],Vec3::Y.to_array(),&g,[0,0]);
                for (exposed,a,b,normal) in [
                    (x==4,[w,16.,s],[w,16.,n],Vec3::NEG_X),
                    (x==7,[e,16.,n],[e,16.,s],Vec3::X),
                    (z==4,[w,16.,n],[e,16.,n],Vec3::NEG_Z),
                    (z==7,[e,16.,s],[w,16.,s],Vec3::Z),
                ] {
                    if !exposed {continue;}
                    let mut low_a=a;low_a[1]=ground;let mut low_b=b;low_b[1]=ground;
                    outdoor_stone::append_canyon(&mut mesh.solid,[low_a,a,b,low_b],normal.to_array(),&g,[0,0]);
                }
            }}
            let original=mesh.clone();
            outdoor_stone::finish_rock_geometry(&mut mesh,"Route44",&cells,&shapes,&g,[0,0],None);
            let at=|x:f32|original.solid.positions.iter().zip(&original.solid.normals)
                .position(|(p,n)|*p==[x,16.,48.] && n[1]>0.9).unwrap();
            let edge=mesh.solid.positions[at(32.)][1];
            let shoulder=mesh.solid.positions[at(40.)][1];
            let center=mesh.solid.positions[at(48.)][1];
            assert!(edge>ground && edge<ground+(16.-ground)*0.4,
                "the foothill must use standing ground, not the neighboring roof: {ground}, {edge}");
            assert!(center>shoulder && shoulder>edge,
                "a small body rises continuously toward its middle: {edge}, {shoulder}, {center}");
            let mut profile=original.solid.positions.iter().zip(&original.solid.normals).enumerate()
                .filter(|(_, (p,n))|n[1]>0.9 && (p[2]-48.).abs()<0.001 && (32. ..=48.).contains(&p[0]))
                .map(|(i,(p,_))|(p[0],mesh.solid.positions[i][1])).collect::<Vec<_>>();
            profile.sort_by(|a,b|a.0.total_cmp(&b.0));
            for pair in profile.windows(2) {
                assert!(pair[1].1>=pair[0].1-0.001,"the foothill must not leave a lip or sunken strip: {pair:?}");
            }
            let expected=super::super::mountain_flanks::height(Vec2::new(6.,6.),[0,0],&[],2.,g.tile_height)
                +ground*(1.-2./REACH as f32);
            assert!((center-expected).abs()<0.001,
                "the complete body has one moderate summit, without a source shaft: {center}");
            for (before,after) in original.solid.positions.iter().zip(&mesh.solid.positions) {
                if before[1]==ground {assert_eq!(before,after,"ground contacts remain seated");}
            }
            assert_eq!(mesh.footing_heights,original.footing_heights);
            assert_eq!(mesh.authored_cells,original.authored_cells);
        }
    }

    #[test]
    fn distant_mineral_tags_cannot_pull_caps_across_the_map() {
        let (frame,g)=fixture("johto",0x0a);
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let shapes=cells.iter().map(|c|shape_for_source(&c.source)).collect::<Vec<_>>();
        for (canyon,origin) in [(false,[0,1800]),(true,[0,-1800])] {
            let mut mesh=TerrainMeshData::default();
            let append=if canyon {outdoor_stone::append_canyon}else{outdoor_stone::append};
            for z in 2..10 {for x in 2..10 {
                let (w,e,n,s)=g.bounds(x,z);
                append(&mut mesh.solid,[[w,16.,n],[w,16.,s],[e,16.,s],[e,16.,n]],
                    Vec3::Y.to_array(),&g,origin);
            }}
            outdoor_stone::sculpt_rock_rims_on_map(&mut mesh,"Route44",&cells,&shapes,&g,origin);
            outdoor_stone::retain_mineral_cap_ownership(&mut mesh);
            let before=mesh.solid.positions.clone();
            finish(&mut mesh,"Route44",&cells,&shapes,&g,origin,None);
            for (a,b) in before.iter().zip(&mesh.solid.positions) {
                assert!(Vec3::from_array(*a).xz().distance(Vec3::from_array(*b).xz())<g.tile_width*2.,
                    "material tag crossed into another map coordinate at {origin:?}: {a:?} -> {b:?}");
            }
            assert!(mesh.solid.positions.iter().any(|p|p[1]>g.tile_height*3.),
                "distant caps still receive their actual interior relief");
        }
    }

    #[test]
    fn mountain_mass_fills_the_eroded_cap_above_its_edge_and_keeps_ground_contacts() {
        for floor in [0., crate::profile::JUMP_LEDGE_HEIGHT] {
        for sloped_ground in [false,true] {
        let (mut frame, g) = fixture("johto", 0x0a);
        // A complete low cave needs a local roof shoulder. Its neighboring
        // accessible cap must retain the same triangle/footing agreement.
        for z in 0..2 {for x in 0..2 {
            frame.tiles[(8+z)*g.width+5+x].source=super::super::tests::source_with_tile(
                0x73,x as u8,2+z as u8,[[0x46,0x47],[0x56,0x57]][z][x]);
        }}
        let cells = frame.tiles.iter().collect::<Vec<_>>();
        let mut shapes = cells.iter().map(|c| shape_for_source(&c.source)).collect::<Vec<_>>();
        for (cell, shape) in cells.iter().zip(&mut shapes) {
            if !permanent(&cell.source, *shape) {
                *shape = CellShape::RaisedTop { height: floor, solid: SolidKind::Bank };
            }
        }
        let mut mesh = TerrainMeshData::default();
        mesh.footing_heights = shapes.iter().map(|s| s.surface_height(g.tile_height)).collect();
        for z in 2..10 { for x in 2..10 {
            if !permanent(&cells[z*g.width+x].source,shapes[z*g.width+x]) {continue;}
            let (w, e, n, s) = g.bounds(x, z);
            let top = shapes[z * g.width + x].surface_height(g.tile_height);
            outdoor_stone::append_canyon(&mut mesh.solid,
                [[w,top,n],[w,top,s],[e,top,s],[e,top,n]], [0.,1.,0.], &g, [0,0]);
            for (exposed, a, b, normal) in [
                (x == 2, [w,top,s], [w,top,n], [-1.,0.,0.]),
                (x == 9, [e,top,n], [e,top,s], [1.,0.,0.]),
                (z == 2, [w,top,n], [e,top,n], [0.,0.,-1.]),
                (z == 9, [e,top,s], [w,top,s], [0.,0.,1.]),
            ] {
                if !exposed { continue; }
                let mut a = a; a[1] = 32.;
                let mut b = b; b[1] = 32.;
                let mut low_a = a; low_a[1] = floor;
                let mut low_b = b; low_b[1] = floor;
                outdoor_stone::append_canyon(&mut mesh.solid,
                    [low_a,a,b,low_b], normal, &g, [0,0]);
            }
        }}
        let authored_positions=mesh.solid.positions.clone();
        outdoor_stone::sculpt_rock_rims_on_map(&mut mesh,"Route44",&cells,&shapes,&g,[0,0]);
        outdoor_stone::retain_mineral_cap_ownership(&mut mesh);
        let before = mesh.clone();
        let cap_indices = mesh.mineral_cap_ranges.clone().unwrap();
        let datum = sloped_ground.then(|| crystal_render_api::VisualTerraceDatum {
            origin: bevy::prelude::IVec2::ZERO,
            size: bevy::prelude::UVec2::new(g.width as u32,g.height as u32),
            levels: (0..g.width*g.height).map(|i| {
                [0.,0.,1.,1.].map(|dx| (4.-(i%g.width) as f32-dx).max(0.)*3.)
            }).collect::<Vec<_>>().into(),
            walkable_mineral: (2..10).flat_map(|z|(2..10).map(move |x|bevy::prelude::IVec2::new(x,z)))
                .collect::<Vec<_>>().into(),
            water_barriers: Default::default(), open_water: None,
        });
        let ridge = before.solid.positions.iter().zip(&before.solid.uvs).map(|(p,uv)| {
            let point = Vec2::new((-10000.-uv[0])/g.tile_width,(-40000.-uv[1])/g.tile_height);
            p[1] + super::super::terrace_datum::sample(datum.as_ref(),point).unwrap_or(0.)
                * crate::profile::JUMP_LEDGE_HEIGHT * g.tile_height / SOURCE_TILE_HEIGHT
        }).fold(0.0_f32,f32::max);
        assert!(cap_indices.iter().flat_map(|r| r.clone())
            .any(|i| mesh.solid.positions[i][1] < 16. - g.tile_height * 0.25),
            "fixture must contain the sunken top beneath the authored rim");
        finish(&mut mesh,"Route44",&cells,&shapes,&g,[0,0],datum.as_ref());
        for (before, after) in before.solid.positions.iter().zip(&mesh.solid.positions) {
            if before[1] == floor { assert_eq!(before, after, "the mountain toe stays grounded"); }
        }
        if datum.is_none() {assert_eq!(mesh.footing_heights, before.footing_heights);}
        assert_eq!(mesh.solid.indices, before.solid.indices);
        assert_eq!(mesh.solid.uvs, before.solid.uvs);
        if let Some(datum) = &datum {
            super::super::terraces::finish_with_datum(&mut mesh,"Route44",&cells,&g,[0,0],&[],Some(datum));
        }
        let relief=Relief::with_ground(&cells,&shapes,&g,[0,0],datum.as_ref(),&[],Some(&before.footing_heights));
        let mut final_caps=std::collections::HashMap::<[i64;2],f32>::new();
        for i in cap_indices.iter().flat_map(|r|r.clone()) {
            let point=source_point(before.solid.positions[i],before.solid.uvs[i],&g,[0,0],&[]);
            let key=point.to_array().map(|v|(v*600.).round() as i64);
            final_caps.entry(key).and_modify(|y|*y=y.max(mesh.solid.positions[i][1]))
                .or_insert(mesh.solid.positions[i][1]);
            let expected=relief.geometry_height(point.x,point.y)+datum_offset(&g,[0,0],datum.as_ref(),point);
            assert!((mesh.solid.positions[i][1]-expected).abs()<0.01,
                "the cap follows the complete geological surface, not the old source ridge {ridge}: {:?} != {expected}",mesh.solid.positions[i]);
        }
        for (i,normal) in before.solid.normals.iter().enumerate() {
            if normal[1]>0.9 {continue;}
            let point=source_point(before.solid.positions[i],before.solid.uvs[i],&g,[0,0],&[]);
            let key=point.to_array().map(|v|(v*600.).round() as i64);
            if let Some(top)=final_caps.get(&key) {
                assert!(mesh.solid.positions[i][1]<=*top+0.01,
                    "a lowered outer course cannot leave an upright ring above its cap");
            }
        }
        assert!(final_caps.iter().any(|(p,y)| {
            let point=Vec2::new(p[0] as f32/600.,p[1] as f32/600.);
            relief.distance(point.x,point.y)>=2.
                && *y-datum_offset(&g,[0,0],datum.as_ref(),point)>g.tile_height*3.
        }),"the filled body retains substantial interior relief above its ground datum");
        for triangle in mesh.solid.indices.chunks_exact(3) {for edge in 0..3 {
            let (a,b)=(triangle[edge] as usize,triangle[(edge+1)%3] as usize);
            if a>=authored_positions.len() || b>=authored_positions.len() {continue;}
            if (authored_positions[a][1]-authored_positions[b][1]).abs()>0.001 {continue;}
            let along=Vec3::from_array(authored_positions[a]).xz()
                .distance(Vec3::from_array(authored_positions[b]).xz());
            if along>g.tile_width*0.3 {continue;}
            assert!((mesh.solid.positions[a][1]-mesh.solid.positions[b][1]).abs()<g.tile_height*0.65,
                "one horizontal wall ring cannot split into alternating raised strips: {:?} -> {:?}",
                mesh.solid.positions[a],mesh.solid.positions[b]);
        }}
        if let Some(datum) = &datum {
            let mut published = frame.clone();
            published.map_id = "Route44".into();
            published.terrace_datum = Some(std::sync::Arc::new(datum.clone()));
            let mut cap_vertex = vec![false;mesh.solid.positions.len()];
            for range in &cap_indices {cap_vertex[range.clone()].fill(true);}
            for (x,z) in [(3,3),(5,6),(8,8)] {for (u,v) in [(0.1,0.2),(0.5,0.5),(0.8,0.7)] {
                let point = Vec2::new(x as f32+u,z as f32+v)*g.tile_width;
                let foot = Vec2::new(point.x-g.width as f32*g.tile_width*0.5,
                    g.height as f32*g.tile_height*0.5-point.y);
                let support = crate::footing::resolved_footing_height(&published,foot,&mesh.footing_heights).unwrap();
                let mut surface = None::<f32>;
                for tri in mesh.solid.indices.chunks_exact(3) {
                    if tri.iter().any(|&i|!cap_vertex[i as usize]) {continue;}
                    let [a,b,c] = [tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.solid.positions[i as usize]));
                    let (ab,ac,ap) = (b.xz()-a.xz(),c.xz()-a.xz(),point-a.xz());
                    let area = ab.perp_dot(ac);
                    if area.abs()<0.00001 {continue;}
                    let wb = ap.perp_dot(ac)/area;
                    let wc = ab.perp_dot(ap)/area;
                    if wb>=-0.0001 && wc>=-0.0001 && wb+wc<=1.0001 {
                        let height = a.y+(b.y-a.y)*wb+(c.y-a.y)*wc;
                        surface=Some(surface.map_or(height,|y|y.max(height)));
                    }
                }
                let surface=surface.expect("the walking point lies on a real mountain triangle");
                assert!((support-surface).abs()<0.01,"footing {support} must meet the raised surface {surface} at {point:?}");
            }}
            let ground_point = |x:f32,z:f32| Vec2::new(x*g.tile_width-g.width as f32*g.tile_width*0.5,
                g.height as f32*g.tile_height*0.5-z*g.tile_height);
            let from = ground_point(3.5,3.5);
            let to = ground_point(4.5,3.5);
            let mut actor = crystal_render_api::VisualActor {
                id: crystal_render_api::VisualActorId::Player, source_id:"chris".into(),
                model_palette:None,texture:Default::default(),center:from,size:Vec2::splat(16.),
                flip_x:false,above_priority:false,facing:Some(Vec2::X),
            };
            let mut traversal = crystal_render_api::VisualPlayerTraversal {
                map_id:published.map_id.clone(),step:Some(crystal_render_api::VisualTraversalStep {
                    from:from-Vec2::Y*g.tile_height,to:to-Vec2::Y*g.tile_height,
                    progress:0.,ledge_jump:false,
                }),
            };
            for t in [0.,0.25,0.5,0.75,1.] {
                actor.center=from.lerp(to,t);
                traversal.step.as_mut().unwrap().progress=t;
                let expected=crate::footing::resolved_footing_height(&published,actor.center,&mesh.footing_heights).unwrap();
                let (height,arc)=crate::footing::traversal_height(&published,&actor,&mesh.footing_heights,Some(&traversal)).unwrap();
                assert!((height-expected).abs()<0.001 && arc==0.,"ordinary mountain steps follow their real slope");
            }
            actor.center=from.lerp(to,0.5);
            traversal.step.as_mut().unwrap().progress=0.5;
            traversal.step.as_mut().unwrap().ledge_jump=true;
            assert!(crate::footing::traversal_height(&published,&actor,&mesh.footing_heights,Some(&traversal)).unwrap().1>0.,
                "an actual source ledge jump retains its arc");
        }
        }
        }
    }

    #[test]
    fn johto_reveal_ground_cannot_float_at_the_mountain_standing_height() {
        for family in ["johto", "johto_modern"] {
            for ground_height in [0., 7.] {
                let (mut frame, g) = fixture(family, 0x0a);
                for tile in &mut frame.tiles {
                    tile.source.tileset_id = family.into();
                }
                let cells = frame.tiles.iter().collect::<Vec<_>>();
                let shapes = cells.iter().map(|c| shape_for_source(&c.source)).collect::<Vec<_>>();
                let mut mesh = TerrainMeshData {
                    footing_heights: cells.iter().zip(&shapes).map(|(c, &s)| {
                        if permanent(&c.source, s) { 16. } else { ground_height }
                    }).collect(),
                    authored_cells: vec![None; cells.len()],
                    ..Default::default()
                };
                let cap = [[48.8, 32., 48.8], [48.8, 32., 55.2],
                    [55.2, 32., 55.2], [55.2, 32., 48.8]];
                append_quad_colors(&mut mesh.solid, cap, [0., 1., 0.],
                    cap.map(|p| [-10000. - p[0], -40000. - p[2]]), [[0.3, 0.2, 0.1, 1.]; 4]);
                outdoor_stone::retain_mineral_cap_ownership(&mut mesh);
                let original = mesh.clone();
                append_reveal_floor(&mut mesh, "CianwoodCity", &cells, &shapes, &g, [0, 0]);
                let floor = &mesh.solid.positions[original.solid.positions.len()..];
                assert_eq!(floor.len(), 6, "the wide mineral body still needs backing");
                let expected = if ground_height > 0.02 { ground_height - 0.01 } else { ground_height };
                assert!(floor.iter().all(|p| (p[1] - expected).abs() < 0.0001),
                    "hidden ground must stay at its donor's seat, below the weathered wall: {floor:?}");
                assert_eq!(&mesh.solid.positions[..original.solid.positions.len()], &original.solid.positions);
                assert_eq!(mesh.footing_heights, original.footing_heights);
                assert_eq!(mesh.authored_cells, original.authored_cells);
            }
        }
    }

    #[test]
    fn descending_caps_keep_their_hidden_floor_strictly_inside_the_rock() {
        let (frame,g)=fixture("johto",0x0a);
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let shapes=cells.iter().map(|c|shape_for_source(&c.source)).collect::<Vec<_>>();
        let mut mesh=TerrainMeshData {authored_cells:vec![None;cells.len()],
            footing_heights:vec![16.;cells.len()],..Default::default()};
        let p=[[48.8,15.5,48.8],[48.8,18.,55.2],[55.2,18.,55.2],[55.2,15.5,48.8]];
        append_quad_colors(&mut mesh.solid,p,[0.,1.,0.],
            p.map(|p|[-10000.-p[0],-40000.-p[2]]),[[0.3,0.2,0.1,1.];4]);
        outdoor_stone::retain_mineral_cap_ownership(&mut mesh);
        let before=mesh.clone();
        append_reveal_floor(&mut mesh,"CianwoodCity",&cells,&shapes,&g,[0,0]);
        let floor=mesh.solid.positions.iter().zip(&mesh.solid.uvs)
            .filter(|(_,uv)|uv[0]>87000. && uv[0]<90000.).collect::<Vec<_>>();
        assert!(!floor.is_empty());
        for (p,_) in floor {
            let rock=15.5+(p[2]-48.8)/(55.2-48.8)*2.5;
            assert!(rock>=16.9899,"the backing needs one source pixel inside the descending cap: {p:?}");
            assert!((p[1]-15.99).abs()<0.0001,"the actual hidden-floor height stays unchanged");
        }
        assert_eq!(&mesh.solid.positions[..before.solid.positions.len()],&before.solid.positions);
        assert_eq!(mesh.footing_heights,before.footing_heights);
    }

    #[test]
    fn weathered_upward_wall_facets_cannot_project_hidden_floor_shelves() {
        let (frame,g)=fixture("johto",0x0a);
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let shapes=cells.iter().map(|c|shape_for_source(&c.source)).collect::<Vec<_>>();
        let mut mesh=TerrainMeshData {authored_cells:vec![None;cells.len()],
            footing_heights:vec![16.;cells.len()],..Default::default()};
        let cap=[[48.8,32.,48.8],[48.8,32.,55.2],[55.2,32.,55.2],[55.2,32.,48.8]];
        append_quad_colors(&mut mesh.solid,cap,[0.,1.,0.],
            cap.map(|p|[-10000.-p[0],-40000.-p[2]]),[[0.3,0.2,0.1,1.];4]);
        let wall=[[56.5,20.,49.],[57.5,20.,49.],[57.5,32.,48.8],[56.5,32.,48.8]];
        let n=(Vec3::from_array(wall[1])-Vec3::from_array(wall[0]))
            .cross(Vec3::from_array(wall[3])-Vec3::from_array(wall[0])).normalize();
        assert!(n.y>0.,"fixture must reproduce an upward-facing wall facet");
        append_quad_colors(&mut mesh.solid,wall,n.to_array(),
            wall.map(|p|[-10000.-p[0],-40000.-p[2]]),[[0.3,0.2,0.1,1.];4]);
        outdoor_stone::retain_mineral_cap_ownership(&mut mesh);
        let before=mesh.clone();
        append_reveal_floor(&mut mesh,"CianwoodCity",&cells,&shapes,&g,[0,0]);
        let floor=mesh.solid.positions.iter().zip(&mesh.solid.uvs)
            .filter(|(_,uv)|uv[0]>87000. && uv[0]<90000.).collect::<Vec<_>>();
        assert_eq!(floor.len(),6,"only the true cap supplies its two backing triangles");
        assert!(floor.iter().all(|(p,_)|p[0]>=48.8 && p[0]<=55.2 && p[2]>=48.8 && p[2]<=55.2),
            "a vertical wall cannot extend the hidden floor beyond the original cap");
        assert_eq!(&mesh.solid.positions[..before.solid.positions.len()],&before.solid.positions);
        assert_eq!(mesh.footing_heights,before.footing_heights);
        assert_eq!(mesh.authored_cells,before.authored_cells);
    }

    #[test]
    fn native_corner_courses_continue_the_mountain_but_custom_planted_ink_keeps_its_seat() {
        for family in ["johto", "johto_modern"] {
            for (block,column,row,tile) in [(0x6a,1,1,0x3b),(0x6c,2,2,0x4c)] {
                let (mut frame,g)=fixture(family,0x0a);
                let index=6*g.width+6;
                let mut source=super::super::tests::source_with_tile(block,column,row,tile);
                source.tileset_id=family.into();
                assert!(native_rock_corner(&source));
                frame.tiles[index].source=source.clone();
                let cells=frame.tiles.iter().collect::<Vec<_>>();
                let shapes=cells.iter().map(|c|shape_for_source(&c.source)).collect::<Vec<_>>();
                let relief=Relief::new(&cells,&shapes,&g,[0,0]);
                assert!(relief.height(6.5,6.5)>g.tile_height*2.,
                    "a native corner cannot cut a flat shelf into a continuous mountain");
                let mut mesh=TerrainMeshData {footing_heights:vec![0.;cells.len()],
                    authored_cells:vec![None;cells.len()],..Default::default()};
                mesh.solid.positions=vec![[52.,16.,52.]];
                mesh.solid.uvs=vec![[-10052.,-40052.]];
                let before=mesh.clone();
                finish(&mut mesh,"CianwoodCity",&cells,&shapes,&g,[0,0],None);
                assert!(mesh.solid.positions[0][1]>16.);
                assert_eq!(mesh.footing_heights,before.footing_heights);
                assert_eq!(mesh.authored_cells,before.authored_cells);
                source.tile_index=0x20;
                assert!(!native_rock_corner(&source));
                assert!(!permanent(&source,shapes[index]),"custom planted art retains its exclusion");
            }
        }
    }

    #[test]
    fn reveal_floor_stays_inside_native_rock_without_changing_support_or_covering_water() {
        let (mut frame,g)=fixture("johto",0x0a);
        frame.tiles[6*12+6].source.metatile_id=0x6c;
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let shapes=cells.iter().map(|cell|shape_for_source(&cell.source)).collect::<Vec<_>>();
        let mut mesh=TerrainMeshData {authored_cells:vec![None;cells.len()],
            footing_heights:vec![16.;cells.len()],..Default::default()};
        mesh.authored_cells[6*12+6]=Some("dungeon:connected-rock-terrain");
        mesh.authored_cells[5*12+5]=Some("custom");
        // The actual mineral cap is inset from its source tile. Reusing raw
        // source rectangles would pass ownership checks but expose shelves.
        for z in 2..10 {for x in 2..10 {
            let (w,e,n,s)=g.bounds(x,z);
            let p=[[w+0.8,32.,n+0.8],[w+0.8,32.,s-0.8],
                [e-0.8,32.,s-0.8],[e-0.8,32.,n+0.8]];
            append_quad_colors(&mut mesh.solid,p,[0.,1.,0.],
                p.map(|p|[-10000.-p[0],-40000.-p[2]]),[[0.3,0.2,0.1,1.];4]);
        }}
        let original=mesh.clone();
        append_reveal_floor(&mut mesh,"BlackthornCity",&cells,&shapes,&g,[0,0]);
        assert!(mesh.solid.indices.len()>original.solid.indices.len(),
            "reveal needs real ground behind the mineral face");
        assert_eq!(mesh.footing_heights,original.footing_heights);
        assert_eq!(mesh.authored_cells,original.authored_cells);
        assert_eq!(&mesh.solid.positions[..original.solid.positions.len()],&original.solid.positions);
        assert_eq!(&mesh.solid.normals[..original.solid.normals.len()],&original.solid.normals);
        assert_eq!(&mesh.solid.indices[..original.solid.indices.len()],&original.solid.indices);
        let mut backed=std::collections::BTreeSet::new();
        for tri in mesh.solid.indices[original.solid.indices.len()..].chunks_exact(3) {
            let p=tri.iter().map(|&i|Vec3::from_array(mesh.solid.positions[i as usize])).sum::<Vec3>()/3.;
            assert!((p.y-15.99).abs()<0.0001);
            let (x,z)=((p.x/g.tile_width).floor() as usize,(p.z/g.tile_height).floor() as usize);
            let i=z*g.width+x;
            backed.insert(i);
            assert!(permanent(&cells[i].source,shapes[i]) || i==6*12+6,
                "hidden floor cannot extend onto open ground or water");
            let (w,e,n,s)=g.bounds(x,z);
            for &v in tri {
                let p=mesh.solid.positions[v as usize];
                assert!(p[0]>=w+0.799 && p[0]<=e-0.799 && p[2]>=n+0.799 && p[2]<=s-0.799,
                    "backing protrudes beyond the sculpted cap: {p:?}");
            }
        }
        assert!(backed.contains(&(6*12+6)),"a reveal at the center of a wide native formation needs backing too");
        assert!(!backed.contains(&(5*12+5)),"a custom object retains its floor ownership");
        let mut wet=frame.clone();
        for tile in &mut wet.tiles {
            if !permanent(&tile.source,shape_for_source(&tile.source)) {
                tile.source=super::super::tests::source_with_tile(0x34,0,0,0x14);
            }
        }
        let wet_cells=wet.tiles.iter().collect::<Vec<_>>();
        let wet_shapes=wet_cells.iter().map(|cell|shape_for_source(&cell.source)).collect::<Vec<_>>();
        let mut retained=original.clone();
        append_reveal_floor(&mut retained,"BlackthornCity",&wet_cells,&wet_shapes,&g,[0,0]);
        assert_eq!(retained.solid,original.solid,"water is never a dry-floor donor");
        let mut foreign=frame.clone();
        for tile in &mut foreign.tiles {
            if !permanent(&tile.source,shape_for_source(&tile.source)) {
                tile.source.tileset_id="johto_modern".into();
            }
        }
        let cells=foreign.tiles.iter().collect::<Vec<_>>();
        let shapes=cells.iter().map(|cell|shape_for_source(&cell.source)).collect::<Vec<_>>();
        let mut retained=original.clone();
        append_reveal_floor(&mut retained,"BlackthornCity",&cells,&shapes,&g,[0,0]);
        assert_eq!(retained.solid,original.solid,"a connected atlas cannot supply this rock's ground pigment");
    }

    #[test]
    fn production_reveal_backing_uses_the_final_raised_cap() {
        let (frame,g)=fixture("johto",0x0a);
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let shapes=cells.iter().map(|cell|shape_for_source(&cell.source)).collect::<Vec<_>>();
        let mut mesh=TerrainMeshData {authored_cells:vec![None;cells.len()],
            footing_heights:vec![16.;cells.len()],..Default::default()};
        let p=[[48.8,8.,48.8],[48.8,8.,55.2],[55.2,8.,55.2],[55.2,8.,48.8]];
        append_quad_colors(&mut mesh.solid,p,[0.,1.,0.],
            p.map(|p|[-10000.-p[0],-40000.-p[2]]),[[0.3,0.2,0.1,1.];4]);
        let support=mesh.footing_heights.clone();
        outdoor_stone::finish_rock_geometry(&mut mesh,"BlackthornCity",&cells,&shapes,&g,[0,0],None);
        assert!(mesh.solid.positions[..4].iter().all(|p|p[1]>16.),
            "fixture must reproduce relief rising above its walking support");
        let floor=mesh.solid.positions.iter().zip(&mesh.solid.uvs)
            .filter(|(_,uv)|(87000. ..90000.).contains(&uv[0])).collect::<Vec<_>>();
        assert!(!floor.is_empty(),"backing must be clipped against final relief, not the low initial cap");
        assert!(floor.iter().all(|(p,_)|(p[1]-15.99).abs()<0.0001));
        assert_eq!(mesh.footing_heights,support);
    }

    #[test]
    fn kanto_reveal_floor_uses_final_dry_support_inside_the_existing_cap_only() {
        let (mut frame,g)=fixture("kanto",0x2c);
        for tile in &mut frame.tiles {
            if tile.source.metatile_id==0x2c {tile.source.tile_index=0x11;}
            else {tile.source.tileset_id="kanto".into();tile.source.metatile_id=0x55;tile.source.tile_index=0x5b;}
        }
        let blocked=5*g.width+5;let wet=6*g.width+6;
        frame.tiles[wet].source.metatile_id=0x43;frame.tiles[wet].source.tile_index=0x14;
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let shapes=cells.iter().map(|c|shape_for_source(&c.source)).collect::<Vec<_>>();
        let mut mesh=TerrainMeshData {footing_heights:vec![7.;cells.len()],
            authored_cells:vec![Some("outdoor/kanto-brick-path");cells.len()],..Default::default()};
        for z in 2..10 {for x in 2..10 {
            let i=z*g.width+x;
            mesh.footing_heights[i]=16.;mesh.authored_cells[i]=Some("dungeon:connected-rock-terrain");
            let (w,e,n,s)=g.bounds(x,z);
            let p=[[w+0.8,32.,n+0.8],[w+0.8,32.,s-0.8],[e-0.8,32.,s-0.8],[e-0.8,32.,n+0.8]];
            append_quad_colors(&mut mesh.solid,p,[0.,1.,0.],
                p.map(|p|[-10000.-p[0],-40000.-p[2]]),[[0.3,0.2,0.1,1.];4]);
        }}
        mesh.authored_cells[blocked]=Some("custom");mesh.authored_cells[wet]=None;
        let original=mesh.clone();
        append_kanto_reveal_floor(&mut mesh,"Route22",&cells,&shapes,&g,[0,0]);
        assert!(mesh.solid.positions.len()>original.solid.positions.len());
        assert_eq!(mesh.footing_heights,original.footing_heights);
        assert_eq!(mesh.authored_cells,original.authored_cells);
        assert_eq!(&mesh.solid.positions[..original.solid.positions.len()],&original.solid.positions);
        assert_eq!(&mesh.solid.indices[..original.solid.indices.len()],&original.solid.indices);
        for tri in mesh.solid.indices[original.solid.indices.len()..].chunks_exact(3) {
            let p=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.solid.positions[i as usize]));
            assert!((p[1]-p[0]).cross(p[2]-p[0]).y>0.);
            for point in p {
                assert!((point.y-6.99).abs()<0.0001,"use the adjacent road, not the cliff's 16-unit standing datum");
                let x=(point.x/8.).floor()as usize;let z=(point.z/8.).floor()as usize;
                let cell=z*g.width+x;assert!(cell!=blocked&&cell!=wet);
                assert!(point.x>=x as f32*8.+0.799 && point.x<=(x+1)as f32*8.-0.799);
                assert!(point.z>=z as f32*8.+0.799 && point.z<=(z+1)as f32*8.-0.799);
            }
        }
        for fault in 0..3 {
            let mut source=frame.clone();let mut retained=original.clone();
            for (i,tile) in source.tiles.iter_mut().enumerate() {
                if (2..10).contains(&(i%g.width)) && (2..10).contains(&(i/g.width)) {continue;}
                match fault {
                    0=>{tile.source.metatile_id=0x43;tile.source.tile_index=0x14;},
                    1=>retained.authored_cells[i]=Some("custom"),
                    _=>tile.source.tileset_id="johto_modern".into(),
                }
            }
            let refs=source.tiles.iter().collect::<Vec<_>>();
            let shapes=refs.iter().map(|c|shape_for_source(&c.source)).collect::<Vec<_>>();
            append_kanto_reveal_floor(&mut retained,"Route22",&refs,&shapes,&g,[0,0]);
            assert_eq!(retained.solid,original.solid,"missing, wet or separately owned ground cannot supply backing");
        }
        let mut volcanic=original.clone();
        for uv in &mut volcanic.solid.uvs {uv[1]-=20000.;}
        let before=volcanic.solid.clone();
        append_kanto_reveal_floor(&mut volcanic,"CinnabarIsland",&cells,&shapes,&g,[0,0]);
        assert_eq!(volcanic.solid,before,"volcanic paint retains its separate domain");
    }

    #[test]
    fn volcanic_finish_preserves_relief_footing_and_texture_registration() {
        let (mut frame, g) = fixture("kanto", 0x2c);
        for tile in &mut frame.tiles {
            if tile.source.metatile_id == 0x2c { tile.source.tile_index = 0x11; }
        }
        let cells = frame.tiles.iter().collect::<Vec<_>>();
        let shapes = cells.iter().map(|c| shape_for_source(&c.source)).collect::<Vec<_>>();
        let mut original = TerrainMeshData::default();
        original.footing_heights = vec![7.; cells.len()];
        original.solid.positions = vec![[48.,16.,48.];3];
        original.solid.normals = vec![[0.,1.,0.];3];
        original.solid.colors = vec![[0.4,0.3,0.2,1.];3];
        original.solid.uvs = vec![[-10048.,-40048.],[-10048.,-20048.],crate::scenery_finish::SOURCE_PAINT_UV];
        let mut island = original.clone();
        let mut mainland = original.clone();
        finish(&mut island,"CinnabarIsland",&cells,&shapes,&g,[0,0],None);
        finish(&mut mainland,"LavenderTown",&cells,&shapes,&g,[0,0],None);
        assert_eq!(island.solid.positions,mainland.solid.positions);
        assert_eq!(island.footing_heights,original.footing_heights);
        assert_eq!(mainland.solid.uvs,original.solid.uvs);
        for i in 0..2 {
            assert_eq!(island.solid.uvs[i],[-10048.,-60048.]);
            assert!(island.solid.colors[i][2]>island.solid.colors[i][0],"volcanic stone keeps its cool ash tint");
        }
        assert_eq!(island.solid.uvs[2],original.solid.uvs[2]);
        assert_eq!(island.solid.colors[2],original.solid.colors[2]);
    }

    fn fixture(tileset: &str, metatile: u16) -> (VisualWorldFrame, GridGeometry) {
        let mut source = super::super::tests::source_with_tile(metatile, 1, 1, 0x3c);
        source.tileset_id = tileset.into();
        let flat = super::super::tests::source_with_tile(1, 0, 0, 5);
        let mut sources = vec![flat; 144];
        for z in 2..10 {
            for x in 2..10 {
                sources[z * 12 + x] = source.clone();
            }
        }
        (
            super::super::tests::frame(12, 12, sources),
            GridGeometry {
                width: 12,
                height: 12,
                tile_width: 8.,
                tile_height: 8.,
                origin_x: 0.,
                origin_z: 0.,
            },
        )
    }

    #[test]
    fn kanto_mountains_retain_their_ridges_without_changing_walk_support_or_water() {
        let (mut frame,g)=fixture("kanto",0x2c);
        for tile in &mut frame.tiles {if tile.source.metatile_id==0x2c {tile.source.tile_index=0x11;}}
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let shapes=cells.iter().map(|c|shape_for_source(&c.source)).collect::<Vec<_>>();
        let field=Relief::new(&cells,&shapes,&g,[0,0]);
        assert!(field.height(6.,6.)>g.tile_height*2.,"native mountain masses must retain their relief");
        let mut mesh=TerrainMeshData::default();
        mesh.solid.positions=vec![[48.,16.,48.],[40.,16.,48.],[48.,-2.,48.]];
        mesh.solid.uvs=vec![[-10048.,-40048.],[-10040.,-40048.],[0.5,0.5]];
        mesh.solid.colors=vec![[0.4,0.3,0.2,1.];3];
        mesh.footing_heights=vec![16.;144];
        let before=mesh.clone();
        finish(&mut mesh,"CinnabarIsland",&cells,&shapes,&g,[0,0],None);
        assert!(mesh.solid.positions[..2].iter().zip(&before.solid.positions).all(|(a,b)|a[1]>b[1]));
        assert_eq!(mesh.solid.positions[2],before.solid.positions[2],"water stays on its level plane");
        assert_eq!(mesh.solid.colors[2],before.solid.colors[2]);
        assert_eq!(mesh.solid.uvs[2],before.solid.uvs[2]);
        assert_eq!(mesh.footing_heights,before.footing_heights);
        assert!(mesh.solid.colors[..2].iter().all(|c|c[2]>c[0]));
    }

    #[test]
    fn walkable_mineral_joins_the_mountain_and_keeps_distant_ridges() {
        use bevy::prelude::IVec2;
        let (frame,g)=fixture("johto",0x0a);
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let shapes=cells.iter().map(|c|shape_for_source(&c.source)).collect::<Vec<_>>();
        let sources=cells.iter().enumerate().map(|(i,c)| {
            let p=IVec2::new((i%g.width)as i32,(i/g.width)as i32);
            let walking=(5..7).contains(&p.x) && (5..7).contains(&p.y);
            (p,c.source.clone(),walking || !permanent(&c.source,shapes[i]))
        }).collect::<Vec<_>>();
        let datum=source_terrace_datum("Route26",&[],&sources).unwrap();
        assert_eq!(datum.walkable_mineral.len(),4);
        let field=Relief::with_datum(&cells,&shapes,&g,[0,0],Some(&datum));
        for z in 5..7 {for x in 5..7 {for (u,v) in [(0.,0.),(0.25,0.75),(0.5,0.5),(1.,1.)] {
            assert!(field.height(x as f32+u,z as f32+v)>0.,"walking cap {x},{z} joins the mountain");
        }}}
        assert!(field.height(8.5,8.5)>g.tile_height,"inaccessible masses retain their mountain silhouette");
        let shift=IVec2::new(-31,72);
        let shifted=sources.iter().map(|(p,s,l)|(*p+shift,s.clone(),*l)).collect::<Vec<_>>();
        let shifted=source_terrace_datum("Route26",&[],&shifted).unwrap();
        let shifted=Relief::with_datum(&cells,&shapes,&g,shift.to_array(),Some(&shifted));
        assert!(shifted.height(5.5,5.5)>g.tile_height,"complete map coordinates retain the raised surface");
        let cropped=cells.chunks_exact(g.width).skip(1).take(10)
            .flat_map(|row|row[1..11].iter().copied()).collect::<Vec<_>>();
        let cropped_shapes=cropped.iter().map(|c|shape_for_source(&c.source)).collect::<Vec<_>>();
        let cropped_g=GridGeometry {width:10,height:10,..g};
        let cropped=Relief::with_datum(&cropped,&cropped_shapes,&cropped_g,[1,1],Some(&datum));
        assert!((cropped.height(4.5,4.5)-field.height(5.5,5.5)).abs()<0.0001,"a camera crop retains the same mountain surface");
        assert!((cropped.geometry_height(4.5,4.5)-field.geometry_height(5.5,5.5)).abs()<0.0001,
            "the complete geological body is stable through a camera crop");
        let mut mesh=TerrainMeshData::default();
        mesh.solid.positions=vec![[44.,16.,44.],[68.,16.,68.]];
        mesh.solid.uvs=vec![[-10044.,-40044.],[-10068.,-40068.]];
        mesh.footing_heights=shapes.iter().map(|s|s.surface_height(g.tile_height)).collect();
        let before=mesh.clone();
        finish(&mut mesh,"Route26",&cells,&shapes,&g,[0,0],Some(&datum));
        for (i,point) in [Vec2::new(5.5,5.5),Vec2::new(8.5,8.5)].into_iter().enumerate() {
            assert!((mesh.solid.positions[i][1]-field.geometry_height(point.x,point.y)).abs()<0.001,
                "walking and distant rock share the complete body profile");
        }
        assert!(mesh.solid.positions[0][1]>mesh.solid.positions[1][1]+g.tile_height*0.5,
            "the interior rises above the outer slope instead of retaining a level shaft");
        assert_eq!(mesh.footing_heights,before.footing_heights);
    }

    #[test]
    #[ignore = "requires a completed external CRYSTAL_RENDER_TEST_PACK"]
    fn external_directional_mountain_land_joins_the_raised_surface() {
        use bevy::prelude::IVec2;
        use crystal_core::world::collision::{describe_collision,Terrain,allows_ledge_direction};
        use crystal_core::world::map::Direction;
        let path=std::env::var("CRYSTAL_RENDER_TEST_PACK").unwrap();
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let pack=loaded.pack();
        let map=&pack.data().maps["Route26"];
        let atlas=&map.attributes.tileset_name;
        let layout=&pack.runtime_files()[&format!("data/tilesets/{atlas}_metatiles.bin")];
        let collision=pack.data().tileset_collision(atlas).unwrap();
        let (w,h)=(usize::from(map.attributes.width)*4,usize::from(map.attributes.height)*4);
        let mut sources=Vec::with_capacity(w*h);
        for y in 0..h {for x in 0..w {
            let block=map.blocks[y/4*(w/4)+x/4];
            let permission=collision.metatiles[block as usize].collision[(y%4/2)*2+x%4/2];
            let land=describe_collision(permission).terrain==Terrain::Land
                || [Direction::Up,Direction::Down,Direction::Left,Direction::Right].into_iter()
                    .any(|d|allows_ledge_direction(permission,d));
            sources.push((IVec2::new(x as i32,y as i32),VisualTileSource {
                tileset_id:atlas.as_str().into(),metatile_id:block,subtile_column:(x%4)as u8,
                subtile_row:(y%4)as u8,tile_index:layout[block as usize*16+(y%4)*4+x%4]as u16,
            },land));
        }}
        let region=TerraceSourceRegion {map:"Route26".into(),edges:Vec::new(),cells:sources.clone()};
        let datum=connected_source_terrace_datum("Route26",&[region],&[]).unwrap();
        let mut frame=super::super::tests::frame(w as u32,h as u32,sources.iter().map(|(_,s,_)|s.clone()).collect());
        frame.map_id="Route26".into();
        frame.terrace_datum=Some(datum.clone());
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let shapes=cells.iter().map(|c|shape_for_source(&c.source)).collect::<Vec<_>>();
        let g=GridGeometry {width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let field=Relief::with_datum(&cells,&shapes,&g,[0,0],Some(&datum));
        for (x,y) in [(9usize,28usize),(10,28)] {
            let block=map.blocks[y/2*(w/4)+x/2];
            assert_eq!(collision.metatiles[block as usize].collision[(y%2)*2+x%2],0xb2,
                "fixture must exercise the directional UP_WALL land permission");
            for dy in 0..2 {for dx in 0..2 {
                let p=IVec2::new((x*2+dx)as i32,(y*2+dy)as i32);
                assert!(datum.walkable_mineral.contains(&p),"missing authored walking cap {p}");
                for (u,v) in [(0.,0.),(0.5,0.5),(1.,1.)] {
                    assert!(field.height(p.x as f32+u,p.y as f32+v).is_finite(),"walking terrain remains a finite surface {p}");
                }
            }}
        }
        assert!(field.peaks.iter().any(|&p|p>g.tile_height*2.),"unwalkable mountain masses retain relief");
        let mut resolved=shapes.clone();
        crate::rock_platform::resolve_rock_platform_tiers(&cells,&mut resolved,w);
        crate::elevation::resolve_authored_mountain_tiers(&mut resolved,w,h);
        let mut mesh=TerrainMeshData {authored_cells:vec![None;cells.len()],
            footing_heights:cells.iter().map(|c|support_height(&c.source,g.tile_height)).collect(),
            ..Default::default()};
        let mut claimed=vec![false;cells.len()];
        super::super::mountain_portals::append(&mut mesh,&cells,&resolved,&g,&mut claimed,[0,0]);
        super::super::modeled_dungeons::append_terrain(&mut mesh,"Route26",&cells,&resolved,&g,&mut claimed,[0,0]);
        outdoor_stone::finish_rock_geometry(&mut mesh,"Route26",&cells,&resolved,&g,[0,0],Some(&datum));
        super::super::terraces::finish_with_datum(&mut mesh,"Route26",&cells,&g,[0,0],&[],Some(&datum));
        let mut cap=vec![false;mesh.solid.positions.len()];
        for range in mesh.mineral_cap_ranges.as_ref().unwrap() {cap[range.clone()].fill(true);}
        let mut checked=0;
        for (x,z) in [(18,56),(19,56),(20,56),(21,57)] {
            for (u,v) in [(0.15,0.3),(0.5,0.5),(0.8,0.65)] {
                let point=Vec2::new(x as f32+u,z as f32+v)*g.tile_width;
                let foot=Vec2::new(point.x-w as f32*g.tile_width*0.5,
                    h as f32*g.tile_height*0.5-point.y);
                let support=crate::footing::resolved_footing_height(&frame,foot,&mesh.footing_heights).unwrap();
                let mut surface=None::<f32>;
                for tri in mesh.solid.indices.chunks_exact(3) {
                    if tri.iter().any(|&i|!cap[i as usize]) {continue;}
                    let [a,b,c]=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.solid.positions[i as usize]));
                    let (ab,ac,ap)=(b.xz()-a.xz(),c.xz()-a.xz(),point-a.xz());
                    let area=ab.perp_dot(ac);
                    if area.abs()<0.00001 {continue;}
                    let (wb,wc)=(ap.perp_dot(ac)/area,ab.perp_dot(ap)/area);
                    if wb>=-0.0001 && wc>=-0.0001 && wb+wc<=1.0001 {
                        let height=a.y+(b.y-a.y)*wb+(c.y-a.y)*wc;
                        surface=Some(surface.map_or(height,|y|y.max(height)));
                    }
                }
                let surface=surface.expect("the real UP_WALL walking point has a mineral cap triangle");
                assert!((surface-support).abs()<0.02,
                    "actual Route26 footing {support} must meet its reshaped cap {surface} at {point:?}");
                checked+=1;
            }
        }
        assert_eq!(checked,12);
    }

    #[test]
    fn permanent_rock_grows_ridges_inside_its_footprint() {
        let (frame, g) = fixture("johto", 0x0a);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        let field = Relief::new(&cells, &shapes, &g, [0, 0]);
        assert!(field.height(6., 6.) > g.tile_height * 2.,
            "wide permanent rock should retain substantial relief");
        for z in 0..12 { for x in 0..12 {
            let h = field.height(x as f32, z as f32);
            for (dx, dz) in [(1., 0.), (0., 1.)] {
                let adjacent = field.height(x as f32 + dx, z as f32 + dz);
                assert!((adjacent-h).abs() <= g.tile_height * 1.85 + 0.001,
                    "the restored ridge must retain its bounded slope: {h} -> {adjacent}");
            }
        }}
        for x in 0..=12 {
            for z in [0., 2., 10., 12.] {
                assert_eq!(
                    field.height(x as f32, z),
                    0.,
                    "mountain spreads over its source edge"
                );
            }
        }
        // Shared cap/wall samples are independent of previous rock relief.
        let mut mesh = TerrainMeshData::default();
        mesh.solid.positions = vec![
            [48., 16., 48.],
            [47.9, 16., 48.1],
            [16., 0., 48.],
            [48., 0., 48.],
        ];
        mesh.solid.uvs = vec![
            [-10048., -20048.],
            [-10048., -40048.],
            [-10016., -20048.],
            [-10048., -20048.],
        ];
        mesh.solid.colors = vec![[0.1, 0.1, 0.1, 1.]; 4];
        mesh.footing_heights = vec![16.; 144];
        let support = mesh.footing_heights.clone();
        let original = mesh.solid.clone();
        finish(&mut mesh, "Route26", &cells, &shapes, &g, [0, 0],None);
        assert_eq!(mesh.solid.positions[0], mesh.solid.positions[1],
            "upper cap/wall samples must meet on the same source column");
        assert_eq!(
            mesh.solid.colors[0], mesh.solid.colors[1],
            "cap and wall must share the same mineral pigment"
        );
        assert_eq!(mesh.solid.positions[2], [16., 0., 48.]);
        assert_eq!(
            mesh.solid.positions[3],
            [48., 0., 48.],
            "the mountain toe must stay grounded"
        );
        assert_eq!(mesh.footing_heights, support);
        assert_eq!(mesh.solid.uvs, original.uvs);
        assert_eq!(mesh.solid.indices, original.indices);
    }

    #[test]
    fn wall_only_columns_fit_one_flank_and_leave_short_ground_banks_alone() {
        let (frame,g)=fixture("johto",0x0a);
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let shapes=cells.iter().map(|c|shape_for_source(&c.source)).collect::<Vec<_>>();
        let mut mesh=TerrainMeshData {authored_cells:vec![None;cells.len()],
            footing_heights:shapes.iter().map(|s|s.surface_height(g.tile_height)).collect(),
            ..Default::default()};
        // A diagonal wall has no owned cap samples. Its chipped height rings
        // also cross float-UV rounding boundaries, as real route corners do.
        outdoor_stone::append_canyon(&mut mesh.solid,
            [[34.,0.,34.],[34.,16.,34.],[41.,16.,43.],[41.,0.,43.]],
            Vec3::new(9.,0.,-7.).normalize().to_array(),&g,[0,0]);
        let wall_end=mesh.solid.positions.len();
        outdoor_stone::append_canyon(&mut mesh.solid,
            [[4.,0.,4.],[4.,6.,4.],[12.,6.,4.],[12.,0.,4.]],
            Vec3::NEG_Z.to_array(),&g,[0,0]);
        outdoor_stone::sculpt_rock_rims_on_map(&mut mesh,"Route44",&cells,&shapes,&g,[0,0]);
        outdoor_stone::retain_mineral_cap_ownership(&mut mesh);
        super::super::rock_bases::round_geometry(&mut mesh,"Route44",&cells,&shapes,&g,[0,0]);
        assert!(mesh.mineral_cap_ranges.as_ref().unwrap().is_empty());
        let before=mesh.clone();
        let field=Relief::with_ground(&cells,&shapes,&g,[0,0],None,&[],Some(&mesh.footing_heights));
        let fits=mineral_columns(&mesh,&cells,&shapes,&g,[0,0],None,&[]);
        finish(&mut mesh,"Route44",&cells,&shapes,&g,[0,0],None);
        let mut columns=Vec::<(Vec2,Vec<usize>)>::new();
        for i in 0..wall_end {
            let point=source_point(before.solid.positions[i],before.solid.uvs[i],&g,[0,0],&[]);
            if let Some((_,ids))=columns.iter_mut().find(|(p,_)|p.distance(point)<0.001) {ids.push(i);}
            else {columns.push((point,vec![i]));}
        }
        assert!(columns.len()>=7,"exercise several distinct wall columns");
        let mut fitted=0;
        for (point,ids) in columns {
            let top=*ids.iter().max_by(|&&a,&&b|
                before.solid.positions[a][1].total_cmp(&before.solid.positions[b][1])).unwrap();
            let old_high=before.solid.positions[top][1];
            let high=Vec3::from_array(mesh.solid.positions[top]);
            let root=Vec3::new(point.x*g.tile_width,0.,point.y*g.tile_height);
            assert!((high.y-field.geometry_height(point.x,point.y)).abs()<0.01,
                "a wall-only top must meet the complete mountain surface: {high:?} at {point:?}, expected {}, old {old_high}, fit {:?}",
                field.geometry_height(point.x,point.y),fits.get(&column_key(&fits,point)));
            for i in ids {
                let fraction=before.solid.positions[i][1]/old_high;
                if fraction==0. {
                    assert_eq!(mesh.solid.positions[i],before.solid.positions[i],"rooted contacts stay seated");
                } else if fraction>=1./6. {
                    let expected=root.lerp(high,fraction);
                    let actual=Vec3::from_array(mesh.solid.positions[i]);
                    assert!(actual.distance(expected)<0.01,
                        "upper height rings must share one flank instead of forming ribs: {actual:?} != {expected:?}");
                    fitted+=1;
                }
            }
        }
        assert!(fitted>40);
        assert_eq!(mesh.solid.positions[wall_end..],before.solid.positions[wall_end..],
            "a short bank outside permanent mountain must keep its original shape");
        assert_eq!(mesh.solid.colors[wall_end..],before.solid.colors[wall_end..]);
        assert_eq!(mesh.footing_heights,before.footing_heights);
        assert_eq!(mesh.solid.uvs,before.solid.uvs);
        assert_eq!(mesh.solid.indices,before.solid.indices);
    }

    #[test]
    fn unequal_native_courses_keep_distinct_wall_rows_at_their_shared_corner() {
        let (frame,g)=fixture("johto",0x0a);
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let mut shapes=cells.iter().map(|c|shape_for_source(&c.source)).collect::<Vec<_>>();
        // Nominal terrain tiers can agree while the two authored wall
        // courses still have different original upper extents.
        shapes[9*g.width+4]=CellShape::RaisedTop {height:32.,solid:SolidKind::Bank};
        shapes[9*g.width+5]=CellShape::RaisedTop {height:32.,solid:SolidKind::Bank};
        let mut mesh=TerrainMeshData {authored_cells:vec![None;cells.len()],
            footing_heights:shapes.iter().map(|s|s.surface_height(g.tile_height)).collect(),
            ..Default::default()};
        let mut walls=Vec::new();
        for (x,height) in [(4,16.),(5,32.)] {
            let (w,e,n,s)=g.bounds(x,9);
            let first=mesh.solid.positions.len();
            outdoor_stone::append_canyon(&mut mesh.solid,
                [[e,0.,s],[e,height,s],[w,height,s],[w,0.,s]],Vec3::Z.to_array(),&g,[0,0]);
            walls.push(first..mesh.solid.positions.len());
            outdoor_stone::append_canyon(&mut mesh.solid,
                [[w,height,n],[w,height,s],[e,height,s],[e,height,n]],Vec3::Y.to_array(),&g,[0,0]);
        }
        outdoor_stone::sculpt_rock_rims_on_map(&mut mesh,"Route44",&cells,&shapes,&g,[0,0]);
        outdoor_stone::retain_mineral_cap_ownership(&mut mesh);
        super::super::rock_bases::round_geometry(&mut mesh,"Route44",&cells,&shapes,&g,[0,0]);
        let before=mesh.clone();
        finish(&mut mesh,"Route44",&cells,&shapes,&g,[0,0],None);
        let mut ends=Vec::new();
        for wall in walls {
            let mut edge=wall.filter(|&i| {
                let p=source_point(before.solid.positions[i],before.solid.uvs[i],&g,[0,0],&[]);
                p.distance(Vec2::new(5.,10.))<0.001
            }).collect::<Vec<_>>();
            edge.sort_by(|&a,&b|before.solid.positions[a][1].total_cmp(&before.solid.positions[b][1]));
            edge.dedup_by(|a,b|(before.solid.positions[*a][1]-before.solid.positions[*b][1]).abs()<0.0001);
            assert!(edge.len()>10,"both native courses must exercise several height rings");
            for pair in edge.windows(2) {
                assert!(mesh.solid.positions[pair[1]][1]>mesh.solid.positions[pair[0]][1]+0.00001,
                    "a taller course must not collapse its upper rows into the lower course's cap");
            }
            let first=Vec3::from_array(mesh.solid.positions[edge[0]]);
            let last=Vec3::from_array(mesh.solid.positions[*edge.last().unwrap()]);
            let old_high=before.solid.positions[*edge.last().unwrap()][1];
            for i in edge {
                if before.solid.positions[i][1]/old_high<1./6. {continue;}
                let p=Vec3::from_array(mesh.solid.positions[i]);
                let along=(p.y-first.y)/(last.y-first.y);
                assert!(p.distance(first.lerp(last,along))<0.002,
                    "the shared edge must stay straight through both tessellations");
            }
            ends.push((first,last));
        }
        assert!(ends[0].0.distance(ends[1].0)<0.002 && ends[0].1.distance(ends[1].1)<0.002,
            "different source heights must meet at the same rooted toe and new mountain surface");
    }

    #[test]
    fn broad_summits_vary_without_repeating_a_distance_plateau() {
        let (seed, g) = fixture("johto", 0x0a);
        let rock = seed.tiles[6 * 12 + 6].source.clone();
        let flat = seed.tiles[0].source.clone();
        let frame = super::super::tests::frame(32, 32, (0..1024).map(|i| {
            if (2..30).contains(&(i % 32)) && (2..30).contains(&(i / 32)) {
                rock.clone()
            } else { flat.clone() }
        }).collect());
        let g = GridGeometry { width: 32, height: 32, ..g };
        let cells: Vec<_> = frame.tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        let field = Relief::new(&cells, &shapes, &g, [0, 0]);
        // All four samples have the same distance to the rectangular edge.
        // Compare across a whole formation: neighboring tiles belong to one
        // broad ridge, rather than four unrelated miniature summits.
        let heights = [(6., 6.), (26., 6.), (6., 26.), (26., 26.)]
            .map(|(x, z)| field.height(x, z));
        let low = heights.into_iter().fold(f32::INFINITY, f32::min);
        let high = heights.into_iter().fold(f32::NEG_INFINITY, f32::max);
        assert!(
            high - low > g.tile_height * 0.5,
            "summits repeat: {heights:?}"
        );
        assert!(enabled("Route10South") && enabled("Route10North") && enabled("CinnabarIsland"));
    }

    #[test]
    fn caves_jump_ledges_and_tree_seats_are_unchanged() {
        let mut jump = super::super::tests::source_with_tile(0x07, 1, 1, 0x2c);
        jump.tileset_id = "kanto".into();
        assert!(
            !permanent(
                &jump,
                CellShape::RaisedTop {
                    height: 48.,
                    solid: SolidKind::Bank
                }
            ),
            "a resolved upper terrace cannot turn a jump bank into a mountain"
        );
        for (tileset, block, map) in [
            ("johto", 0x0a, "DragonsDenB1F"),
            ("kanto", 0x2c, "RockTunnel1F"),
            ("johto", 0x0a, "UndergroundPath"),
            ("cave", 0x0a, "MountMoon"),
            ("dark_cave", 0x0a, "DarkCaveVioletEntrance"),
            ("johto", 0x4b, "Route26"),
            ("johto", 0x6a, "Route26"),
            ("johto_modern", 0x6c, "Route33"),
            ("kanto", 0x07, "Route1"),
        ] {
            let (frame, g) = fixture(tileset, block);
            let cells: Vec<_> = frame.tiles.iter().collect();
            let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
            let mut mesh = TerrainMeshData::default();
            mesh.solid.positions = vec![[48., 16., 48.]];
            mesh.solid.uvs = vec![[-10048., -20048.]];
            let before = mesh.clone();
            finish(&mut mesh, map, &cells, &shapes, &g, [0, 0],None);
            assert_eq!(mesh, before, "{tileset}/{block:02x} in {map}");
        }
    }

    #[test]
    fn mountain_openings_and_world_origin_survive_relief() {
        let (mut frame, g) = fixture("johto", 0x0a);
        for z in 5..7 {
            for x in 5..7 {
                frame.tiles[z * 12 + x].source =
                    super::super::tests::source_with_tile(0x73, (x - 5) as u8, (z - 3) as u8, 0x46);
            }
        }
        let cells: Vec<_> = frame.tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        let field = Relief::new(&cells, &shapes, &g, [0, 0]);
        assert_eq!(field.height(5.5, 5.5), 0., "cave entrance must stay open");
        // A cropped viewport with a translated origin samples the same mass.
        let original = &cells;
        let cropped: Vec<_> = (1..12)
            .flat_map(|z| (1..12).map(move |x| original[z * 12 + x]))
            .collect();
        let cropped_shapes: Vec<_> = cropped
            .iter()
            .map(|c| shape_for_source(&c.source))
            .collect();
        let cg = GridGeometry {
            width: 11,
            height: 11,
            ..g
        };
        let shifted = Relief::new(&cropped, &cropped_shapes, &cg, [1, 1]);
        for (x, z) in [(3.5, 3.5), (7.5, 7.5), (8.25, 4.5)] {
            assert!((field.height(x, z) - shifted.height(x - 1., z - 1.)).abs() < 0.0001);
        }
    }

    #[test]
    #[ignore = "requires a completed external CRYSTAL_RENDER_TEST_PACK"]
    fn external_route26_has_mountains_and_preserves_every_open_tile() {
        check_external_footprints(&["Route26"]);
    }

    #[test]
    #[ignore = "requires a completed external CRYSTAL_RENDER_TEST_PACK"]
    fn external_lavender_cinnabar_and_johto_keep_their_rock_footprints() {
        check_external_footprints(&[
            "Route10South",
            "LavenderTown",
            "CinnabarIsland",
            "CianwoodCity",
            "Route29",
        ]);
    }

    #[test]
    #[ignore = "requires a completed external CRYSTAL_RENDER_TEST_PACK"]
    fn external_named_outdoor_sites_keep_rock_relief_out_of_open_footprints() {
        check_external_footprints(&["RuinsOfAlphOutside","LakeOfRage","SilverCaveOutside","MountMoonSquare"]);
    }

    fn check_external_footprints(maps: &[&str]) {
        let path = std::env::var("CRYSTAL_RENDER_TEST_PACK").unwrap();
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let pack = loaded.pack();
        for &map in maps {
            assert!(enabled(map));
            let module = &pack.data().maps[map];
            let tileset = &module.attributes.tileset_name;
            let layout = &pack.runtime_files()[&format!("data/tilesets/{tileset}_metatiles.bin")];
            let width = module.attributes.width as usize * 4;
            let height = module.attributes.height as usize * 4;
            let sources = (0..height)
                .flat_map(|z| {
                    (0..width).map(move |x| {
                        let block = module.blocks[z / 4 * (width / 4) + x / 4];
                        VisualTileSource {
                            tileset_id: tileset.as_str().into(),
                            metatile_id: block,
                            subtile_column: (x % 4) as u8,
                            subtile_row: (z % 4) as u8,
                            tile_index: layout[block as usize * 16 + z % 4 * 4 + x % 4] as u16,
                        }
                    })
                })
                .collect();
            let frame = super::super::tests::frame(width as u32, height as u32, sources);
            let cells: Vec<_> = frame.tiles.iter().collect();
            let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
            let g = GridGeometry {
                width,
                height,
                tile_width: 16.,
                tile_height: 16.,
                origin_x: 0.,
                origin_z: 0.,
            };
            let field = Relief::new(&cells, &shapes, &g, [0, 0]);
            let portal_roofs=super::super::mountain_portals::sites(&cells,&shapes,&g);
            let mut peaks = 0;
            for (i, cell) in cells.iter().enumerate() {
                let x = (i % width) as f32;
                let z = (i / width) as f32;
                if permanent(&cell.source, shapes[i]) {
                    peaks += usize::from(field.height(x + 0.5, z + 0.5) > g.tile_height * 2.);
                } else {
                    // The mineral field also describes closed rock above a
                    // complete native cave. It is not that opening's walking
                    // floor. The portal regression tests actual path geometry
                    // and footing beneath this local roof on both ground levels.
                    let overhead=portal_roofs.iter().any(|roof|
                        [(0.,0.),(0.,1.),(1.,0.),(1.,1.)].into_iter().any(|(dx,dz)|
                            roof.roof_rise(Vec2::new(x+dx,z+dz),g.tile_height)>0.));
                    if overhead {continue;}
                    for (dx, dz) in [(0., 0.), (0.5, 0.5), (1., 1.)] {
                        assert_eq!(
                            field.height(x + dx, z + dz),
                            0.,
                            "{map}: relief enters open tile {x},{z}"
                        );
                    }
                }
            }
            if map == "Route26" || map == "CianwoodCity" {
                assert!(
                    peaks > 100,
                    "{map} should contain substantial mountain relief: {peaks}"
                );
            }
        }
    }
}
