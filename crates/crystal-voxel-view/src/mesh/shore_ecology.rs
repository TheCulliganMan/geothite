//! Sparse planted margins on verified outdoor water. Every accent stays in
//! its water cell; source paths, actor support and terrain ownership remain intact.
use super::*;
// A CPU terrain-fit hint. Solid-color shaders leave this marker unchanged.
pub(super) const ROOTED_WATER_UV:f32=86100.;

fn place_seed(map: &str) -> u32 {
    // These two presentations are the same physical park. Other locations
    // retain their own planting instead of repeating coordinate-based patches.
    let place = if map == "NationalParkBugContest" { "NationalPark" } else { map };
    place.bytes().fold(0x811c9dc5_u32, |seed, byte| {
        (seed ^ u32::from(byte)).wrapping_mul(0x01000193)
    })
}

fn noise(x: i32, z: i32, salt: u32) -> f32 {
    let mut n = (x as u32).wrapping_mul(0x9e3779b9)
        ^ (z as u32).wrapping_mul(0x85ebca6b) ^ salt.wrapping_mul(0x7feb352d);
    n ^= n >> 16; n = n.wrapping_mul(0x846ca68b); n ^= n >> 16;
    (n & 65535) as f32 / 65535.
}

fn face(mesh: &mut SurfaceMeshData, p: [Vec3;3], color: [f32;3]) {
    let normal = (p[1]-p[0]).cross(p[2]-p[0]).normalize_or_zero();
    if normal.length_squared() < 0.5 { return; }
    let i = mesh.positions.len() as u32;
    mesh.positions.extend(p.map(|p| p.to_array()));
    mesh.normals.extend([normal.to_array();3]);
    mesh.uvs.extend([[0.;2];3]);
    mesh.colors.extend([[color[0],color[1],color[2],1.];3]);
    mesh.indices.extend([i,i+1,i+2]);
    // Narrow foliage must remain visible from either side of the orbit.
    mesh.positions.extend([p[0].to_array(),p[2].to_array(),p[1].to_array()]);
    mesh.normals.extend([(-normal).to_array();3]);
    mesh.uvs.extend([[0.;2];3]); mesh.colors.extend([[color[0],color[1],color[2],1.];3]);
    mesh.indices.extend([i+3,i+4,i+5]);
}

fn blade(mesh: &mut SurfaceMeshData, root: Vec3, tip: Vec3, width: f32, turn: f32, color: [f32;3]) {
    let side = Vec3::new(turn.cos(),0.,turn.sin()) * width;
    let bend = root.lerp(tip,0.55) + side * 0.35;
    face(mesh,[root,bend-side,tip],color);
    face(mesh,[root,tip,bend+side],color);
}

fn patch(mesh: &mut SurfaceMeshData, center: Vec3, toward: Vec3, tile: f32, coord: [i32;2], freshwater: bool, seed: u32) {
    let [x,z] = coord; let tangent = Vec3::new(-toward.z,0.,toward.x);
    let variation = |salt| noise(x, z, seed ^ salt);
    for shoot in 0..9 {
        let along = (variation(shoot*4+1)-0.5) * tile * 0.52;
        let root = center + tangent*along - toward*(variation(shoot*4+2)*tile*0.12);
        let height = tile * (0.30 + variation(shoot*4+3)*0.65);
        let tip = root + Vec3::Y*height - toward*tile*0.08;
        blade(
            mesh,
            root,
            tip,
            tile * 0.013,
            variation(shoot * 4 + 4) * std::f32::consts::TAU,
            [0.12, 0.24, 0.055],
        );
        for leaf in 0..3 {
            let turn = variation(100+shoot*3+leaf)*std::f32::consts::TAU;
            let leaf_root = root + Vec3::Y*height*(0.15+leaf as f32*0.15);
            let leaf_tip = leaf_root + Vec3::new(turn.cos()*tile*0.13,height*0.25,turn.sin()*tile*0.13);
            blade(mesh,leaf_root,leaf_tip,tile*0.022,turn+1.57,[0.085,0.28,0.075]);
        }
        if freshwater && shoot%3 == 0 {
            // Russet seed heads punctuate the green silhouette.
            blade(mesh,tip-Vec3::Y*tile*0.14,tip+Vec3::Y*tile*0.015,
                tile*0.027,shoot as f32,[0.25,0.13,0.055]);
        }
    }
    if !freshwater { return; }
    for pad in 0..3 {
        let origin = center - toward*tile*(0.26+pad as f32*0.09)
            + tangent*tile*(pad as f32-1.)*0.14 + Vec3::Y*tile*0.008;
        let radius = tile*(0.07+variation(200+pad)*0.04);
        let turn = variation(210+pad)*std::f32::consts::TAU;
        // Leave a true radial notch in the leaf, rather than painting a disk.
        for segment in 0..11 {
            let angle = |s: u32| turn+0.22+(std::f32::consts::TAU-0.44)*s as f32/11.;
            let a=angle(segment);let b=angle(segment+1);
            face(mesh,[origin,origin+Vec3::new(b.cos()*radius,0.,b.sin()*radius),
                origin+Vec3::new(a.cos()*radius,0.,a.sin()*radius)],
                [0.055,0.20+pad as f32*0.025,0.085]);
        }
    }
}

pub(super) fn append(mesh: &mut TerrainMeshData, map: &str, cells: &[&VisualTile], shapes: &[CellShape],
    g: &GridGeometry, origin: [i32;2]) {
    if !crate::cave_water::outdoor(map) { return; }
    let seed = place_seed(map);
    let fresh = matches!(map,"LakeOfRage"|"Route42"|"Route43"|"Route44"|"IlexForest"|"NationalPark"|"NationalParkBugContest");
    for row in 0..g.height { for col in 0..g.width {
        let i=row*g.width+col;
        if shapes[i] != CellShape::Water || mesh.authored_cells[i].is_some()
            || !matches!(cells[i].source.tileset_id.as_ref(),"johto"|"johto_modern"|"kanto"|"forest"|"park") {continue;}
        let coord=[origin[0]+col as i32,origin[1]+row as i32];
        // Coherent groups and gaps, never a reed fence around the entire lake.
        if noise(coord[0].div_euclid(4),coord[1].div_euclid(4),seed ^ 400)>0.64
            || noise(coord[0],coord[1],seed ^ 401)>0.72 {continue;}
        for [dx,dz] in [[-1,0],[1,0],[0,-1],[0,1]] {
            let c=col as i32+dx;let r=row as i32+dz;
            if c<0||r<0||c>=g.width as i32||r>=g.height as i32 {continue;}
            let j=r as usize*g.width+c as usize;
            let land=&cells[j].source;
            let lawn = matches!((land.tileset_id.as_ref(),land.tile_index),
                ("johto"|"johto_modern"|"forest",0x05)|("kanto",0x2c)|("park",0x01));
            let natural_bank = matches!(land.tileset_id.as_ref(),"johto"|"johto_modern"|"kanto"|"forest")
                && matches!(shapes[j],CellShape::RaisedTop {solid:SolidKind::Bank,..}
                    | CellShape::LedgeBand {..} | CellShape::ShoreBand);
            if !(lawn && shapes[j] == CellShape::Flat || natural_bank)
                || mesh.authored_cells[j].is_some() {continue;}
            let (w,e,n,s)=g.bounds(col,row);let tile=g.tile_width.min(g.tile_height);
            let toward=Vec3::new(dx as f32,0.,dz as f32);
            let center=Vec3::new((w+e)*0.5,shapes[i].surface_height(g.tile_height),(n+s)*0.5)
                +toward*tile*0.30;
            let first=mesh.solid.positions.len();
            patch(&mut mesh.solid,center,toward,tile,coord,fresh,seed);
            mesh.solid.uvs[first..].fill([ROOTED_WATER_UV,0.]);
        }
    }}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accents_are_map_anchored_rooted_and_stay_in_the_water_cell() {
        for map in ["LakeOfRage", "Route42", "CherrygroveCity"] {
        let seed = place_seed(map);
        for direction in [Vec3::X,Vec3::NEG_X,Vec3::Z,Vec3::NEG_Z] {
            let mut a=SurfaceMeshData::default();let root=Vec3::new(16.,-2.,16.)+direction*32.*0.30;
            patch(&mut a,root,direction,32.,[24,18],true,seed);
            assert!(a.positions.iter().all(|p|p[0]>0.&&p[0]<32.&&p[2]>0.&&p[2]<32.&&p[1]>=-2.));
            assert!(a.positions.iter().any(|p|p[1]>10.));
            assert!(a.normals.iter().all(|n|(Vec3::from_array(*n).length()-1.).abs()<0.0001));
            let mut b=SurfaceMeshData::default();patch(&mut b,root,direction,32.,[24,18],true,seed);
            assert_eq!(a,b);
        }
        }
    }

    #[test]
    fn shoreline_accents_keep_footing_and_ownership_and_reject_paths_rooms_and_claimed_cells() {
        let g=GridGeometry { width:3,height:1,tile_width:32.,tile_height:32.,origin_x:0.,origin_z:0. };
        let tiles:Vec<_>=[0x05,0x14,0x05].into_iter().enumerate().map(|(column,tile)| VisualTile {
            column:column as u32,row:0,source:VisualTileSource {
                tileset_id:"johto".into(),metatile_id:0,subtile_column:0,subtile_row:0,tile_index:tile,
            },texture:Handle::default(),animation_frames:None,priority:false,
        }).collect();
        let cells:Vec<_>=tiles.iter().collect();let shapes=[CellShape::Flat,CellShape::Water,CellShape::Flat];
        let make=|| TerrainMeshData { authored_cells:vec![None;3],footing_heights:vec![0.,-8.,0.],..Default::default() };
        let mut model=make();let mut selected=None;
        for x in 0..40 { append(&mut model,"LakeOfRage",&cells,&shapes,&g,[x,18]);
            if !model.solid.positions.is_empty() { selected=Some(x);break; }
        }
        let x=selected.expect("a sparse eligible shoreline must receive accents");
        assert_eq!(model.footing_heights,[0.,-8.,0.]);assert_eq!(model.authored_cells,[None;3]);
        assert!(model.solid.positions.iter().all(|p|p[0]>32.&&p[0]<64.&&p[2]>0.&&p[2]<32.));
        let mut room=make();append(&mut room,"CeruleanGym",&cells,&shapes,&g,[x,18]);assert!(room.solid.positions.is_empty());
        let mut owned=make();owned.authored_cells[1]=Some("fixture/water-art");
        append(&mut owned,"LakeOfRage",&cells,&shapes,&g,[x,18]);assert!(owned.solid.positions.is_empty());
        let mut path_tiles=tiles.clone();path_tiles[0].source.tile_index=0x06;path_tiles[2].source.tile_index=0x06;
        let paths:Vec<_>=path_tiles.iter().collect();let mut path=make();
        append(&mut path,"LakeOfRage",&paths,&shapes,&g,[x,18]);assert!(path.solid.positions.is_empty());
    }
    #[test]
    fn elevated_pier_water_plants_remain_rooted_when_terrain_is_terraced() {
        let g=GridGeometry{width:3,height:1,tile_width:32.,tile_height:32.,origin_x:0.,origin_z:0.};
        let tiles=[5,0x14,5].into_iter().enumerate().map(|(x,t)|VisualTile{
            column:x as u32,row:0,source:VisualTileSource{tileset_id:"johto".into(),metatile_id:0,
                subtile_column:0,subtile_row:0,tile_index:t},texture:Handle::default(),animation_frames:None,priority:false}).collect::<Vec<_>>();
        let cells=tiles.iter().collect::<Vec<_>>();let shapes=[CellShape::Flat,CellShape::Water,CellShape::Flat];
        let mut mesh=TerrainMeshData{authored_cells:vec![None;3],footing_heights:vec![0.;3],..Default::default()};
        let mut selected=None;
        for x in 0..40 {append(&mut mesh,"Route32",&cells,&shapes,&g,[x,18]);
            if !mesh.solid.positions.is_empty() {selected=Some(x);break;}}
        let x=selected.unwrap();let old=mesh.solid.positions.clone();
        assert!(mesh.solid.uvs.iter().all(|uv|uv[0]==ROOTED_WATER_UV));
        let edge=crystal_render_api::VisualJumpLedge{start:bevy::math::IVec2::new(x,19),end:bevy::math::IVec2::new(x+3,19),descent:bevy::math::IVec2::Y};
        super::super::terraces::finish_on_map(&mut mesh,"Route32",&cells,&g,[x,18],&[edge]);
        assert!(mesh.footing_heights[0]>0.);
        assert_eq!(&mesh.solid.positions[..old.len()],old.as_slice());
    }

}
