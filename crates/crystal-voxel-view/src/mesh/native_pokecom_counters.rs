//! Source counter courses: one horizontal top row over one vertical front row.
use super::*;

fn drawing(block:u16)->Option<[u16;8]> {
    Some(match block {
        8=>[19,53,52,52,36,36,36,36],
        9=>[12,12,52,52,36,36,36,36],
        10|26=>[52,52,52,52,36,36,36,36],
        16=>[12,12,19,53,36,36,36,36],
        59=>[12,12,52,22,36,36,36,149],
        _=>return None,
    })
}
pub(super) fn matches(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    if map!=crate::pokecom::ADMIN_OFFICE || p.kind!=ModelKind::ReceptionCounter
        || p.width!=4 || p.height!=2 || p.depth_pixels!=8. || p.height_pixels!=8.
        || p.base_pixels!=0. || p.column+4>g.width || p.row+2>g.height
        || cells.len()!=g.width*g.height {return false;}
    let block=cells[p.row*g.width+p.column].source.metatile_id;
    let Some(art)=drawing(block) else{return false;};
    p.indices(g.width).enumerate().all(|(j,i)| {
        let s=&cells[i].source;
        s.tileset_id.as_ref()=="pokecom_center" && s.metatile_id==block
            && s.subtile_column==(j%4)as u8 && s.subtile_row==(j/4)as u8
            && s.tile_index==art[j]
    })
}
pub(super) fn candidates(map:&str,cells:&[&VisualTile],g:&GridGeometry,
    claimed:&mut[bool],out:&mut Vec<Placement>) {
    if map!=crate::pokecom::ADMIN_OFFICE || g.width<4 || g.height<2 {return;}
    let Some(ground)=ground_sample(cells,map,"pokecom_center",0x11) else{return;};
    for row in 0..=g.height-2 {for column in 0..=g.width-4 {
        let p=Placement{kind:ModelKind::ReceptionCounter,column,row,width:4,height:2,
            ground,depth_pixels:8.,height_pixels:8.,base_pixels:0.,footing_pixels:None};
        if !matches(map,&p,cells,g) || p.indices(g.width).any(|i|claimed[i]) {continue;}
        for i in p.indices(g.width) {claimed[i]=true;}
        out.push(p);
    }}
    super::native_pokecom_returns::candidates(map,cells,g,claimed,out);
    super::native_pokecom_long_returns::candidates(map,cells,g,claimed,out);
}
pub(super) fn append(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,[w,e,n,s]:[f32;4],covered:Option<usize>) {
    let height=8.*g.tile_height/8.;let first=out.positions.len();let indices=out.indices.len();
    architecture_box(out,[w,e,0.,height,n,s],[1.;3]);
    let uv=g.uv(p.column,p.row+1);
    out.uvs[first..].fill([uv.0+(uv.1-uv.0)*4.5/8.,uv.2+(uv.3-uv.2)*4.5/8.]);
    // Replace the top and south front with their original independent rows.
    out.indices.drain(indices..indices+12);
    for x in 0..4 {
        let left=w+(e-w)*x as f32/4.;let right=w+(e-w)*(x+1)as f32/4.;
        let first=out.positions.len();
        append_top(out,[left,right,n,s],height,g.uv(p.column+x,p.row));
        if covered.is_some_and(|start|(start..start+2).contains(&x)) {
            out.uvs[first..].fill([uv.0+(uv.1-uv.0)*4.5/8.,uv.2+(uv.3-uv.2)*1.5/8.]);
        }
        let (u0,u1,v0,v1)=g.uv(p.column+x,p.row+1);
        append_quad(out,[[right,0.,s],[right,height,s],[left,height,s],[left,0.,s]],
            [0.,0.,1.],[[u1,v1],[u1,v0],[u0,v0],[u0,v1]],TEXTURED_SHADE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_pokecom_counter_courses_are_low_closed_atomic_and_leave_aprons_free() {
        let path=std::env::var_os("CRYSTAL_RENDER_TEST_PACK").map(std::path::PathBuf::from).unwrap();
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let pack=loaded.pack();let map=crate::pokecom::ADMIN_OFFICE;let data=&pack.data().maps[map];
        let art=&pack.runtime_files()["data/tilesets/pokecom_center_metatiles.bin"];
        let(w,h)=(data.attributes.width as usize*4,data.attributes.height as usize*4);
        let tiles=(0..w*h).map(|i| {let(x,y)=(i%w,i/w);let b=data.blocks[y/4*(w/4)+x/4];
            VisualTile{column:x as u32,row:y as u32,texture:Default::default(),priority:false,animation_frames:None,
                source:VisualTileSource{tileset_id:"pokecom_center".into(),metatile_id:b,
                    subtile_column:(x%4)as u8,subtile_row:(y%4)as u8,tile_index:art[b as usize*16+y%4*4+x%4]as u16}}
        }).collect::<Vec<_>>();
        let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let refs=tiles.iter().collect::<Vec<_>>();let all=resolve(map,&refs,&g,None);
        let counters=all.iter().filter(|p|matches(map,p,&refs,&g)).collect::<Vec<_>>();assert_eq!(counters.len(),10);
        for p in counters {
            let mut mesh=TerrainMeshData{footing_heights:vec![2.75;w*h],authored_cells:vec![None;w*h],..Default::default()};
            let mut claims=vec![false;w*h];assert!(super::super::append(&mut mesh,map,&refs,&g,p,&mut claims,[0,0]));
            assert_eq!(claims.iter().filter(|&&c|c).count(),8);
            assert_eq!(mesh.footing_heights,vec![2.75;w*h]);
            assert!((0..4).all(|x|!claims[(p.row+2)*w+p.column+x]),"source apron stays separate");
            let mut body=SurfaceMeshData::default();let south=(p.row+2)as f32*8.;
            append(&mut body,&g,p,[p.column as f32*8.,(p.column+4)as f32*8.,south-8.,south],None);
            assert!(body.positions.iter().all(|v|v[1]>=0.&&v[1]<=8.&&v[2]>=south-8.&&v[2]<=south));
            let center=Vec3::new((p.column+2)as f32*8.,4.,south-4.);
            for triangle in body.indices.chunks_exact(3) {
                let a=Vec3::from_array(body.positions[triangle[0]as usize]);
                let b=Vec3::from_array(body.positions[triangle[1]as usize]);
                let c=Vec3::from_array(body.positions[triangle[2]as usize]);
                assert!((b-a).cross(c-a).dot((a+b+c)/3.-center)>0.,"all closed faces point outward");
            }
            for index in p.indices(w) {
                let mut damaged=tiles.clone();damaged[index].source.tile_index^=0x80;let r=damaged.iter().collect::<Vec<_>>();
                assert!(!matches(map,p,&r,&g));
                assert!(!resolve(map,&r,&g,None).iter().any(|q|matches(map,q,&r,&g)&&q.column==p.column&&q.row==p.row));
                let mut reserved=vec![false;w*h];reserved[index]=true;let mut candidates_out=Vec::new();
                candidates(map,&refs,&g,&mut reserved,&mut candidates_out);
                assert!(!candidates_out.iter().any(|q|q.column==p.column&&q.row==p.row));
            }
            assert!(!matches("GoldenrodPokecenter1F",p,&refs,&g));
        }
    }
}
