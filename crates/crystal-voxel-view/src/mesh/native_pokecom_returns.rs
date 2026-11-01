//! Service-counter returns and the blue controls sitting at their corners.
use super::*;

fn side_placement(column:usize,row:usize,ground:usize)->Placement {
    Placement{kind:ModelKind::ReceptionCounter,column,row,width:1,height:2,ground,
        depth_pixels:32.,height_pixels:8.,base_pixels:0.,footing_pixels:None}
}
fn context(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->Option<(Placement,Placement)> {
    if map!=crate::pokecom::ADMIN_OFFICE || p.column>=g.width || p.row+5>g.height
        || cells.len()!=g.width*g.height {return None;}
    let origin=&cells[p.row*g.width+p.column].source;
    let (block,offset,footer_block)=match (origin.metatile_id,origin.subtile_column) {
        (4,0)=>(4,0,8), (12,3)=>(12,3,16), _=>return None,
    };
    let left=p.column.checked_sub(offset)?;
    if left+4>g.width || !(0..2).all(|y| {
        let s=&cells[(p.row+y)*g.width+p.column].source;
        s.tileset_id.as_ref()=="pokecom_center" && s.metatile_id==block
            &&s.subtile_column==offset as u8&&s.subtile_row==(y+1)as u8&&s.tile_index==177
    }) {return None;}
    let panel_column=left+if offset==0 {0}else{2};
    if !(0..2).all(|x| {
        let s=&cells[(p.row+2)*g.width+panel_column+x].source;
        s.tileset_id.as_ref()=="pokecom_center"&&s.metatile_id==block
            &&s.subtile_column==(panel_column-left+x)as u8&&s.subtile_row==3
            &&s.tile_index==[3,37][x]
    }) {return None;}
    let footer=Placement{kind:ModelKind::ReceptionCounter,column:left,row:p.row+3,width:4,height:2,
        depth_pixels:8.,height_pixels:8.,base_pixels:0.,footing_pixels:None,..p.clone()};
    if cells[footer.row*g.width+footer.column].source.metatile_id!=footer_block
        || !native_pokecom_counters::matches(map,&footer,cells,g) {return None;}
    let panel=Placement{kind:ModelKind::HealingConsole,column:panel_column,row:p.row+2,width:2,height:1,
        depth_pixels:6.,height_pixels:6.,base_pixels:8.,footing_pixels:None,..p.clone()};
    Some((footer,panel))
}
pub(super) fn side(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    p.kind==ModelKind::ReceptionCounter && p.width==1 && p.height==2
        &&p.depth_pixels==32.&&p.height_pixels==8.&&p.base_pixels==0.
        &&context(map,p,cells,g).is_some()
}
fn panel_context(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->Option<(Placement,Placement)> {
    if p.kind!=ModelKind::HealingConsole||p.width!=2||p.height!=1||p.base_pixels!=8.
        ||p.height_pixels!=6.||p.depth_pixels!=6.||p.row<2||p.column>=g.width||p.row>=g.height
        ||cells.len()!=g.width*g.height {return None;}
    let block=cells[p.row*g.width+p.column].source.metatile_id;
    let column=p.column+usize::from(block==12);
    let side=side_placement(column,p.row-2,p.ground);
    let(footer,expected)=context(map,&side,cells,g)?;
    (expected.column==p.column&&expected.row==p.row).then_some((side,footer))
}
pub(super) fn panel(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    panel_context(map,p,cells,g).is_some()
}
pub(super) fn candidates(map:&str,cells:&[&VisualTile],g:&GridGeometry,claimed:&mut[bool],out:&mut Vec<Placement>) {
    if map!=crate::pokecom::ADMIN_OFFICE||g.width==0||g.height<5 {return;}
    let Some(ground)=ground_sample(cells,map,"pokecom_center",0x11)else{return;};
    for row in 0..=g.height-5 {for column in 0..g.width {
        let side=side_placement(column,row,ground);
        let Some((footer,panel))=context(map,&side,cells,g)else{continue;};
        if !out.iter().any(|p|p.column==footer.column&&p.row==footer.row
            &&native_pokecom_counters::matches(map,p,cells,g))
            ||side.indices(g.width).chain(panel.indices(g.width)).any(|i|claimed[i]) {continue;}
        for i in side.indices(g.width).chain(panel.indices(g.width)) {claimed[i]=true;}
        out.extend([side,panel]);
    }}
}
pub(super) fn covered_columns(map:&str,counter:&Placement,cells:&[&VisualTile],g:&GridGeometry,
    furnishings:&[Placement])->Option<usize> {
    furnishings.iter().find_map(|p| {
        let(_,footer)=panel_context(map,p,cells,g)?;
        (footer.column==counter.column&&footer.row==counter.row).then(||p.column-counter.column)
    })
}
pub(super) fn append_side(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,[w,e,n,s]:[f32;4]) {
    let first=out.positions.len();let scale=g.tile_height/8.;
    architecture_box(out,[w+g.tile_width/8.,e,0.,8.*scale,n,s],[1.;3]);
    let uv=g.uv(p.column,p.row);
    out.uvs[first..].fill([uv.0+(uv.1-uv.0)*1.5/8.,uv.2+(uv.3-uv.2)*4.5/8.]);
    let indices=out.indices.len();
    // Remove the original top's six indices without disturbing the side faces.
    out.indices.drain(indices-36..indices-30);
    let top=out.positions.len();
    append_top(out,[w+g.tile_width/8.,e,n,s],8.*scale,
        (uv.0+(uv.1-uv.0)/8.,uv.1,uv.2,uv.3));
    // The source strip repeats longitudinally; use its central cream pigment
    // on the continuous cap rather than stretching one 8px tile down the run.
    out.uvs[top..].fill([uv.0+(uv.1-uv.0)*4.5/8.,uv.2+(uv.3-uv.2)*4.5/8.]);
}
pub(super) fn append_panel(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,[w,e,_,s]:[f32;4]) {
    let scale=g.tile_height/8.;let cx=(w+e)*0.5;
    let(w,e,n,s)=(cx-6.*g.tile_width/8.,cx+6.*g.tile_width/8.,s-7.*scale,s-scale);
    let first=out.positions.len();let indices=out.indices.len();
    architecture_box(out,[w,e,8.*scale,14.*scale,n,s],[1.;3]);
    let uv=g.uv(p.column,p.row);
    out.uvs[first..].fill([uv.0+(uv.1-uv.0)*2.5/8.,uv.2+(uv.3-uv.2)*1.5/8.]);
    for i in first..out.positions.len() {
        if out.positions[i][1]>13.*scale {
            out.positions[i][1]=14.*scale-4.*scale*(out.positions[i][2]-n)/(s-n);
        }
        if out.normals[i][1]>0.5 {out.normals[i]=Vec3::new(0.,1.,4.*scale/(s-n)).normalize().to_array();}
    }
    out.indices.drain(indices..indices+6);
    let point=|x:f32,z:f32|[w+(e-w)*x,14.*scale-4.*scale*z,n+(s-n)*z];
    let normal=Vec3::new(0.,1.,4.*scale/(s-n)).normalize().to_array();
    for y in 0..2 {for x in 0..2 {
        let u=g.uv(p.column+x,p.row+y);let(x0,x1,z0,z1)=(x as f32/2.,(x+1)as f32/2.,y as f32/2.,(y+1)as f32/2.);
        append_quad(out,[point(x0,z0),point(x0,z1),point(x1,z1),point(x1,z0)],normal,
            [[u.0,u.2],[u.0,u.3],[u.1,u.3],[u.1,u.2]],TEXTURED_SHADE);
    }}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_pokecom_returns_join_both_counter_corners_and_own_complete_controls() {
        let path=std::env::var_os("CRYSTAL_RENDER_TEST_PACK").map(std::path::PathBuf::from).unwrap();
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let pack=loaded.pack();let map=crate::pokecom::ADMIN_OFFICE;let data=&pack.data().maps[map];
        let art=&pack.runtime_files()["data/tilesets/pokecom_center_metatiles.bin"];
        let(w,h)=(data.attributes.width as usize*4,data.attributes.height as usize*4);
        let tiles=(0..w*h).map(|i|{let(x,y)=(i%w,i/w);let b=data.blocks[y/4*(w/4)+x/4];
            VisualTile{column:x as u32,row:y as u32,texture:Default::default(),animation_frames:None,priority:false,
                source:VisualTileSource{tileset_id:"pokecom_center".into(),metatile_id:b,
                    subtile_column:(x%4)as u8,subtile_row:(y%4)as u8,tile_index:art[b as usize*16+y%4*4+x%4]as u16}}
        }).collect::<Vec<_>>();
        let refs=tiles.iter().collect::<Vec<_>>();let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let all=resolve(map,&refs,&g,None);
        let sides=all.iter().filter(|p|side(map,p,&refs,&g)).collect::<Vec<_>>();
        assert_eq!(sides.len(),2);assert_eq!(all.iter().filter(|p|panel(map,p,&refs,&g)).count(),2);
        assert_eq!(pack.data().tilesets["pokecom_center"].palette_map[177],9,"return uses bank one, not the raw PNG index");
        for p in sides {
            let(footer,control)=context(map,p,&refs,&g).unwrap();
            let cap=all.iter().find(|q|q.column==control.column&&q.row==control.row&&panel(map,q,&refs,&g)).unwrap();
            assert_eq!(covered_columns(map,&footer,&refs,&g,&all),Some(cap.column-footer.column));
            assert_eq!(covered_columns(map,&footer,&refs,&g,&[]),None,"fallback keeps the original corner drawing");
            let mut mesh=TerrainMeshData{footing_heights:vec![2.75;w*h],authored_cells:vec![None;w*h],..Default::default()};
            let mut claims=vec![false;w*h];
            for piece in [&footer,p,cap] {assert!(append_with_tables(&mut mesh,map,&refs,&g,piece,&mut claims,[0,0],&all));}
            assert_eq!(claims.iter().filter(|&&v|v).count(),12);
            assert_eq!(mesh.footing_heights,vec![2.75;w*h]);
            assert!(mesh.textured.positions.iter().all(|v|v.iter().all(|c|c.is_finite())));
            let mut body=SurfaceMeshData::default();let west=p.column as f32*8.;let end=(p.row+4)as f32*8.;
            append_side(&mut body,&g,p,[west,west+8.,p.row as f32*8.,end]);
            assert!(body.positions.iter().any(|v|v[2]==end&&v[1]==8.));
            assert_eq!(end,(footer.row+1)as f32*8.,"return meets the transverse back without an overlapping front face");
            let mut tilted=SurfaceMeshData::default();let west=cap.column as f32*8.;
            let front=(footer.row+2)as f32*8.;
            append_panel(&mut tilted,&g,cap,[west,west+16.,front-6.,front]);
            assert!(tilted.positions.iter().all(|v|v[1]>=8.&&v[1]<=14.&&v[2]>=front-7.&&v[2]<=front-1.));
            for triangle in tilted.indices.chunks_exact(3) {
                let i=triangle[0]as usize;let a=Vec3::from_array(tilted.positions[i]);
                let b=Vec3::from_array(tilted.positions[triangle[1]as usize]);
                let c=Vec3::from_array(tilted.positions[triangle[2]as usize]);
                assert!((b-a).cross(c-a).dot(Vec3::from_array(tilted.normals[i]))>0.,"control faces retain outward winding");
            }
            let owned=p.indices(w).chain(cap.indices(w)).collect::<Vec<_>>();
            for &i in &owned {
                let mut reserved=vec![false;w*h];reserved[i]=true;
                let mut out=all.iter().filter(|q|native_pokecom_counters::matches(map,q,&refs,&g)).cloned().collect::<Vec<_>>();
                candidates(map,&refs,&g,&mut reserved,&mut out);
                assert!(!out.iter().any(|q|q.column==p.column&&q.row==p.row));
                assert!(!out.iter().any(|q|q.column==cap.column&&q.row==cap.row));
            }
            for i in owned.into_iter().chain(footer.indices(w)) {
                let mut damaged=tiles.clone();damaged[i].source.tile_index^=0x80;let r=damaged.iter().collect::<Vec<_>>();
                assert!(!side(map,p,&r,&g));assert!(!panel(map,cap,&r,&g));
            }
            assert!(!side("GoldenrodPokecenter1F",p,&refs,&g));
        }
    }
}
