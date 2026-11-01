//! Closed timber at the authored wall/floor seam; radios remain floor objects.
//! Tile0x50 in the lower source rows is a walkable wooden floor strip. Only
//! the upper rows of blocks01/08 are WALL; never fold that floor up the wall.
use super::*;

pub(super) fn is_map(map:&str)->bool {
    traditional_room::floor_map(map)&&map!="DanceTheater"
}
pub(super) fn source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    is_map(map)&&matches!(native_gift_shop::source(map,p,cells,g),
        Some(ModelKind::WallTraditional|ModelKind::RadioWall))
}
fn timber(out:&mut SurfaceMeshData,g:&GridGeometry,west:f32,width:usize,front:f32,uv:(f32,f32,f32,f32)) {
    let scale=g.tile_height/8.;let height=32.*scale;
    let sample=|x:f32,y:f32|[uv.0+(uv.1-uv.0)*(x+0.5)/8.,uv.2+(uv.3-uv.2)*(y+0.5)/8.];
    let field=sample(2.,0.);let edge=sample(1.,0.);let joint=sample(0.,0.);let grain=sample(3.,1.);
    for x in 0..width {
        let w=west+x as f32*g.tile_width;let e=w+g.tile_width;
        let first=out.positions.len();let index=out.indices.len();
        architecture_box(out,[w,e,0.,height,front-3.*scale,front],[1.;3]);
        out.uvs[first..].fill(field);out.indices.drain(index+6..index+12);
        let xs=[0.,0.025,0.35,0.45,0.95,0.975,1.];let ys=[0.,0.10,0.16,0.60,0.66,1.];
        native_source_surface_patches(out,&xs,&ys,
            |x,y|[w+(e-w)*x,height*(1.-y),front-if x==0.975 {0.2*scale}else{0.}],
            |x,y|if x>0.95 {joint}else if x<0.025 {edge}
                else if x>0.35&&x<0.45&&((y>0.10&&y<0.16)||(y>0.60&&y<0.66)) {grain}else{field},
            |_,_|1.,true);
    }
}
pub(super) fn append(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement) {
    let front=g.origin_z+(p.row+2)as f32*g.tile_height;
    let west=g.origin_x+p.column as f32*g.tile_width;
    timber(out,g,west,p.width,front,g.uv(p.column,p.row));
    if p.kind==ModelKind::RadioWall {
        let scale=g.tile_height/8.;let south=g.origin_z+(p.row+4)as f32*g.tile_height;
        let radio=crate::interior_models::selected_model(ModelKind::Radio).expect("validated native radio");
        assert!(radio.append_native_home_fixture(out,
            [west,west+2.*g.tile_width,south-7.*scale,south],0.,12.*scale,
            native_source_swatches(p,g,[[4,22],[1,24],[1,22],[2,22]])));
    }
}
pub(super) fn perimeter_template(out:&mut SurfaceMeshData,map:&str,cells:&[&VisualTile],
    g:&GridGeometry,placements:&[Placement])->Option<usize> {
    if let Some(p)=placements.iter().filter(|p|source(map,p,cells,g)).min_by_key(|p|(p.row,p.column)) {
        // Keep the timber field without repeating the radio below it.
        timber(out,g,g.origin_x+p.column as f32*g.tile_width,2,
            g.origin_z+(p.row+2)as f32*g.tile_height,g.uv(p.column,p.row));
        return Some(p.row);
    }
    let (blocks,_)=binding(map)?;let width=blocks.len()*4;
    for row in 0..g.height.saturating_sub(3) {for column in 0..g.width.saturating_sub(width-1) {
        if !backing_source(map,cells,g,column,row,&[]) {continue;}
        let donor=(0..4).flat_map(|y|(0..width).map(move|x|(row+y)*g.width+column+x))
            .find(|&i|cells[i].source.tile_index==0x11)?;
        timber(out,g,g.origin_x+column as f32*g.tile_width,2,
            g.origin_z+(row+2)as f32*g.tile_height,g.uv(donor%g.width,donor/g.width));
        return Some(row);
    }}
    None
}
fn binding(map:&str)->Option<(&'static[u16],u64)> {
    match map {
        "EcruteakItemfinderHouse"|"EcruteakLugiaSpeechHouse"|"VioletKylesHouse"|"VioletNicknameSpeechHouse"
            =>Some((&[0x13,1,8,0x29],0xd6d889249c4bd66d)),
        "MahoganyMart1F"=>Some((&[1,8,8,0x13],0x5c3a5e247cc09a54)),
        "KurtsHouse"=>Some((&[0x0f,0x1a,0x13,0x23,0x14,0x29,0x31,0x32],0xddcc22218f20284d)),
        _=>None,
    }
}
pub(super) fn backing_source(map:&str,cells:&[&VisualTile],g:&GridGeometry,column:usize,row:usize,
    excluded:&[bool])->bool {
    let Some((blocks,expected))=binding(map) else{return false;};let width=blocks.len()*4;
    if column+width>g.width||row+4>g.height||g.width.checked_mul(g.height)!=Some(cells.len())
        ||(!excluded.is_empty()&&excluded.len()!=cells.len()) {return false;}
    let mut hash=0xcbf29ce484222325u64;
    for y in 0..4 {for x in 0..width {
        let i=(row+y)*g.width+column+x;let s=&cells[i].source;
        if excluded.get(i)==Some(&true)||s.tileset_id.as_ref()!="traditional_house"
            ||s.metatile_id!=blocks[x/4]||s.subtile_column as usize!=x%4||s.subtile_row as usize!=y {return false;}
        for byte in [(s.metatile_id&255)as u8,(s.metatile_id>>8)as u8,s.subtile_column,
            s.subtile_row,(s.tile_index&255)as u8,(s.tile_index>>8)as u8] {
            hash=(hash^u64::from(byte)).wrapping_mul(0x100000001b3);
        }
    }}
    hash==expected
}
pub(super) fn append_backing(mesh:&mut TerrainMeshData,map:&str,cells:&[&VisualTile],g:&GridGeometry,
    origin:[i32;2],excluded:&[bool]) {
    let Some((blocks,_))=binding(map) else{return;};let width=blocks.len()*4;
    if crate::interior_models::selected_model(ModelKind::WallTraditional).is_none() {return;}
    for row in 0..g.height.saturating_sub(3) {for column in 0..g.width.saturating_sub(width-1) {
        if origin[0]+column as i32!=0||origin[1]+row as i32!=0
            ||!backing_source(map,cells,g,column,row,excluded) {continue;}
        let Some(donor)=(0..4).flat_map(|y|(0..width).map(move|x|(row+y)*g.width+column+x))
            .find(|&i|cells[i].source.tile_index==0x11) else{continue;};
        // Native wall profiles already close their own columns. This fills
        // only authenticated gaps behind the clock/cabinet/craft fixtures.
        for x in 0..width {
            if matches!(blocks[x/4],1|8) {continue;}
            timber(&mut mesh.textured,g,g.origin_x+(column+x)as f32*g.tile_width,1,
                g.origin_z+(row+2)as f32*g.tile_height,g.uv(donor%g.width,donor/g.width));
        }
    }}
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tile(x:usize,y:usize,block:u16,art:u16)->VisualTile {
        VisualTile{column:x as u32,row:y as u32,texture:Default::default(),animation_frames:None,
            priority:false,source:VisualTileSource{tileset_id:"traditional_house".into(),metatile_id:block,
                subtile_column:(x%4)as u8,subtile_row:(y%4)as u8,tile_index:art}}
    }
    fn geometry(width:usize,height:usize)->GridGeometry {
        GridGeometry{width,height,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.}
    }
    #[test]
    fn timber_wall_stays_at_row_two_and_radio_is_separate_without_changing_footing() {
        for map in ["EcruteakItemfinderHouse","EcruteakLugiaSpeechHouse","VioletKylesHouse",
            "VioletNicknameSpeechHouse","MahoganyMart1F","KurtsHouse"] {
            for name in ["Traditional house plain north wall","Traditional house north wall with radio"] {
                let object=built_in_profiles().objects.iter().find(|o|o.name==name).unwrap();
                let g=geometry(6,6);
                let mut tiles=(0..36).map(|i|tile(i%6,i/6,4,
                    [[0x44,0x45,0x45,0x46],[0x54,0x55,0x55,0x56],
                     [0x45,0x46,0x44,0x45],[0x55,0x56,0x54,0x55]][i/6%4][i%6%4])).collect::<Vec<_>>();
                for y in 0..4 {for x in 0..4 {tiles[y*6+x]=tile(x,y,object.metatile,object.tiles[y][x]);}}
                let cells=tiles.iter().collect::<Vec<_>>();
                let p=resolve(map,&cells,&g,None).into_iter().find(|p|
                    matches!(p.kind,ModelKind::WallTraditional|ModelKind::RadioWall)).unwrap();
                assert!(source(map,&p,&cells,&g));
                let mut mesh=TerrainMeshData{footing_heights:vec![3.25;36],authored_cells:vec![None;36],..Default::default()};
                let mut claimed=vec![false;36];
                assert!(super::super::append(&mut mesh,map,&cells,&g,&p,&mut claimed,[0,0]));
                assert_eq!(claimed.iter().filter(|&&v|v).count(),16);
                assert_eq!(mesh.footing_heights,vec![3.25;36]);assert!(mesh.solid.positions.is_empty());
                assert!(mesh.textured.positions.iter().any(|v|v[1]==32.&&v[2]==16.));
                // Source collisions01/08 leave the lower half RADIO/FLOOR
                // or FLOOR/FLOOR. Keep its native wood bands flat; interpreting
                // these bands as wall rails would erase a real walking strip.
                let donor=p.row*g.width+p.column+2+2*g.width;
                assert_eq!(cells[donor].source.tile_index,crate::house::TRADITIONAL_HOUSE_FLOOR_TILE);
                let floor=g.uv(p.column+2,p.row+2);
                let expected_uvs=[[floor.0,floor.2],[floor.0,floor.3],
                    [floor.1,floor.3],[floor.1,floor.2]];
                for (cell,i) in p.indices(g.width).filter(|i|i/g.width>=p.row+2).enumerate() {
                    let (w,e,n,south)=g.bounds(i%g.width,i/g.width);
                    assert_eq!(&mesh.textured.positions[cell*4..cell*4+4],
                        &[[w,0.,n],[w,0.,south],[e,0.,south],[e,0.,n]]);
                    assert_eq!(&mesh.textured.uvs[cell*4..cell*4+4],&expected_uvs,
                        "native horizontal wood bands must remain on the ground");
                }
                for v in &mesh.textured.positions {
                    if v[1]>0.&&v[2]>16.+1e-5 {
                        assert_eq!(p.kind,ModelKind::RadioWall);
                        assert!(v[0]<=16.+1e-5&&v[1]<=12.+1e-5,
                            "only the radio may occupy the lower floor half");
                    }
                }
                let mut custom=object.clone();custom.name="Custom timber wall".into();
                let doc=Document{objects:vec![custom],atmosphere:None};
                assert!(!resolve(map,&cells,&g,Some(&doc)).iter().any(|p|source(map,p,&cells,&g)));
                for i in 0..16 {let mut bad=tiles.clone();bad[(i/4)*6+i%4].source.tile_index^=1;
                    assert!(!source(map,&p,&bad.iter().collect::<Vec<_>>(),&g));}
            }
        }
    }
    fn course(blocks:&[u16])->(Vec<VisualTile>,GridGeometry) {
        let art=|b|match b {
            0x13=>[35,36,17,17,41,42,35,36,57,58,62,63,82,83,24,25],
            1=>[17,17,17,17,17,17,17,17,10,11,80,80,26,27,80,80],
            8=>[17,17,17,17,17,17,17,17,80,80,80,80,80,80,80,80],
            0x29=>[17,17,35,36,35,36,30,31,8,9,46,47,82,83,24,25],_=>unreachable!(),
        };
        let g=geometry(blocks.len()*4,4);
        let tiles=(0..g.width*g.height).map(|i| {
            let x=i%g.width;let y=i/g.width;let b=blocks[x/4];tile(x,y,b,art(b)[y*4+x%4])
        }).collect();(tiles,g)
    }
    #[test]
    fn backing_requires_all_source_art_and_reserves_no_extra_floor_or_custom_cells() {
        for (map,blocks) in [("EcruteakItemfinderHouse",&[0x13,1,8,0x29][..]),
            ("MahoganyMart1F",&[1,8,8,0x13][..])] {
            let (tiles,g)=course(blocks);let cells=tiles.iter().collect::<Vec<_>>();
            assert!(backing_source(map,&cells,&g,0,0,&[]));
            assert!(!backing_source("UnreviewedTraditionalHouse",&cells,&g,0,0,&[]));
            let mut mesh=TerrainMeshData{footing_heights:vec![1.25;64],authored_cells:vec![None;64],..Default::default()};
            append_backing(&mut mesh,map,&cells,&g,[0,0],&[]);
            assert!(!mesh.textured.positions.is_empty());
            assert!(mesh.textured.positions.iter().all(|p|p[2]<=16.));
            assert_eq!(mesh.footing_heights,vec![1.25;64]);assert_eq!(mesh.authored_cells,vec![None;64]);
            for i in 0..64 {
                let mut changed=tiles.clone();changed[i].source.tile_index^=1;
                assert!(!backing_source(map,&changed.iter().collect::<Vec<_>>(),&g,0,0,&[]));
                let mut excluded=vec![false;64];excluded[i]=true;
                assert!(!backing_source(map,&cells,&g,0,0,&excluded));
            }
            let mut shifted=TerrainMeshData::default();append_backing(&mut shifted,map,&cells,&g,[1,0],&[]);
            assert!(shifted.textured.positions.is_empty());
        }
    }
}
