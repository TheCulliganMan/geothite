//! Two contiguous source cushions on one supported department-store bench.
use super::*;
pub(super) fn is_map(map:&str)->bool {
    matches!(map,"CeladonDeptStore1F"|"CeladonDeptStore5F"|"CeladonDeptStore6F"
        |"GoldenrodDeptStore1F"|"GoldenrodDeptStore5F"|"GoldenrodDeptStore6F")
}
fn drawing(map:&str,col:usize,row:usize,cells:&[&VisualTile],g:&GridGeometry)->bool {
    if !is_map(map)||g.width.checked_mul(g.height)!=Some(cells.len())||col+2>g.width||row+4>g.height {return false;}
    let first=&cells[row*g.width+col].source;
    if first.tileset_id.as_ref()!="mart"||first.subtile_row!=0
        ||!matches!((first.metatile_id,first.subtile_column),(7,0)|(0x20,2)) {return false;}
    let art=[[0x22,0x23],[0x32,0x33],[0x24,0x25],[0x34,0x35]];
    (0..8).all(|i|{let s=&cells[(row+i/2)*g.width+col+i%2].source;
        s.tileset_id.as_ref()=="mart"&&s.metatile_id==first.metatile_id
            &&s.subtile_column==first.subtile_column+(i%2)as u8&&s.subtile_row==(i/2)as u8&&s.tile_index==art[i/2][i%2]})
}
pub(super) fn matches(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    p.kind==ModelKind::Cushion&&p.width==2&&p.height==4&&p.depth_pixels==26.
        &&p.height_pixels==7.&&p.base_pixels==0.&&p.footing_pixels.is_none()
        &&drawing(map,p.column,p.row,cells,g)
}
pub(super) fn candidates(map:&str,cells:&[&VisualTile],g:&GridGeometry,claimed:&mut[bool],out:&mut Vec<Placement>) {
    if !is_map(map)||g.width<2||g.height<4||claimed.len()!=cells.len(){return;}
    let Some(ground)=ground_sample(cells,map,"mart",1)else{return;};
    for row in 0..=g.height-4 {for column in 0..=g.width-2 {
        if !drawing(map,column,row,cells,g){continue;}
        let p=Placement{kind:ModelKind::Cushion,column,row,width:2,height:4,ground,
            depth_pixels:26.,height_pixels:7.,base_pixels:0.,footing_pixels:None};
        if p.indices(g.width).any(|i|claimed[i]){continue;}
        for i in p.indices(g.width){claimed[i]=true;}out.push(p);
    }}
}
pub(super) fn seat_rects(map:&str,p:&Placement,s:&VisualTileSource)->Option<[[f32;4];2]> {
    if !is_map(map)||p.kind!=ModelKind::Cushion||p.width!=2||p.height!=4||p.depth_pixels!=26.
        ||s.tileset_id.as_ref()!="mart"||s.subtile_row!=0||!matches!((s.metatile_id,s.subtile_column),(7,0)|(0x20,2)){return None;}
    Some([0.,13./8.].map(|offset|[p.column as f32,p.row as f32+offset,2.,13./8.]))
}
pub(super) fn append(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,b:[f32;4]) {
    let [w,e,n,s]=b;let sy=g.tile_height/8.;let sx=g.tile_width/8.;
    let paint=native_source_swatches(p,g,[[1,1],[3,3],[0,1],[3,27],[3,30]]);
    for i in 0..2 {
        let mut pad=SurfaceMeshData::default();
        let bounds=[w+sx,e-sx,n+(i as f32*13.+0.1)*sy,n+((i+1)as f32*13.-0.1)*sy];
        assert!(model(ModelKind::Cushion).append_native_gift_fixture(&mut pad,bounds,5.5*sy,1.5*sy,
            [paint[0],paint[1],paint[0],paint[2]]));
        let cx=(bounds[0]+bounds[1])*0.5;let cz=(bounds[2]+bounds[3])*0.5;
        // The canonical cushion button is an isolated central component. The
        // source bench has plain pads; preserve its piping and broad faces.
        let faces=pad.indices.chunks_exact(3).filter(|t|!t.iter().all(|&j| {
            let v=pad.positions[j as usize];(v[0]-cx).abs()<(bounds[1]-bounds[0])*0.075
                &&(v[2]-cz).abs()<(bounds[3]-bounds[2])*0.075&&v[1]>6.7*sy
        })).collect::<Vec<_>>();
        let top=faces.iter().flat_map(|t|t.iter()).map(|&j|pad.positions[j as usize][1]).fold(5.5*sy,f32::max);
        let scale=1.5*sy/(top-5.5*sy);let mut remap=vec![None;pad.positions.len()];
        for tri in faces {for &j in tri {
            let index=*remap[j as usize].get_or_insert_with(|| {
                let index=out.positions.len()as u32;let mut v=pad.positions[j as usize];
                v[1]=5.5*sy+(v[1]-5.5*sy)*scale;
                let normal=Vec3::from_array(pad.normals[j as usize])*Vec3::new(1.,scale.recip(),1.);
                let normal=normal.normalize();out.positions.push(v);out.normals.push(normal.to_array());
                out.uvs.push(pad.uvs[j as usize]);out.colors.push(crate::interior_models::authored_face_color([1.;4],normal));index
            });out.indices.push(index);
        }}
    }
    let first=out.positions.len();
    architecture_box(out,[w+sx,e-sx,2.*sy,5.5*sy,n+0.5*sy,s-0.5*sy],[1.;3]);
    for i in first..out.positions.len(){out.uvs[i]=paint[3];}
    for x in [w+sx,e-2.4*sx]{for z in [n+sy,s-2.4*sy]{
        let first=out.positions.len();architecture_box(out,[x,x+1.4*sx,0.,2.2*sy,z,z+1.4*sy],[1.;3]);
        for i in first..out.positions.len(){out.uvs[i]=if out.normals[i][2]>0.99{paint[4]}else{paint[2]};}
    }}
}

#[cfg(test)]
mod tests {
    use super::*;
    fn top_at(mesh:&SurfaceMeshData,x:f32,z:f32)->f32 {
        mesh.indices.chunks_exact(3).filter_map(|t|{
            if mesh.normals[t[0]as usize][1]<0.99{return None;}
            let [a,b,c]=[t[0],t[1],t[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
            let side=|a:Vec3,b:Vec3|(b.x-a.x)*(z-a.z)-(b.z-a.z)*(x-a.x);
            let signs=[side(a,b),side(b,c),side(c,a)];
            (signs.iter().all(|s|*s>=-0.0001)||signs.iter().all(|s|*s<=0.0001)).then_some(a.y)
        }).fold(f32::NEG_INFINITY,f32::max)
    }
    #[test]
    fn bench_guards_respect_every_source_cell_reservation_and_unequal_axis_crop() {
        for (block,phase) in [(7,0),(0x20,2)] {
            let g=GridGeometry{width:4,height:6,tile_width:12.,tile_height:4.,origin_x:-12.,origin_z:7.};
            let mut tiles=(0..24).map(|i|VisualTile{column:(i%4)as u32,row:(i/4)as u32,
                texture:Default::default(),priority:false,animation_frames:None,
                source:VisualTileSource{tileset_id:"mart".into(),metatile_id:4,
                    subtile_column:(i%4)as u8,subtile_row:(i/4%4)as u8,tile_index:1}}).collect::<Vec<_>>();
            for (y,row)in [[0x22,0x23],[0x32,0x33],[0x24,0x25],[0x34,0x35]].iter().enumerate(){for x in 0..2{
                tiles[y*4+x].source=VisualTileSource{tileset_id:"mart".into(),metatile_id:block,
                    subtile_column:phase+x as u8,subtile_row:y as u8,tile_index:row[x]};
            }}
            let refs=tiles.iter().collect::<Vec<_>>();let mut claims=vec![false;24];let mut found=Vec::new();
            candidates("CeladonDeptStore6F",&refs,&g,&mut claims,&mut found);assert_eq!(found.len(),1);let p=&found[0];
            assert!(matches("CeladonDeptStore6F",p,&refs,&g));
            let mut mesh=TerrainMeshData{footing_heights:vec![2.5;24],..Default::default()};
            assert!(super::super::append(&mut mesh,"CeladonDeptStore6F",&refs,&g,p,&mut vec![false;24],[-4,9]));
            assert_eq!(mesh.footing_heights,vec![2.5;24]);
            for v in mesh.textured.positions.iter().filter(|v|v[1]>0.){
                assert!(v[0]>=-12.&&v[0]<=12.&&v[2]>=7.&&v[2]<=20.&&v[1]<=3.5001,
                    "folded physical bounds follow both axes, not the 32-pixel source height");
            }
            for i in p.indices(4){
                let mut reserved=vec![false;24];reserved[i]=true;let before=reserved.clone();let mut output=Vec::new();
                candidates("CeladonDeptStore6F",&refs,&g,&mut reserved,&mut output);
                assert!(output.is_empty());assert_eq!(reserved,before);
                for field in 0..5{
                    let mut changed=tiles.clone();let s=&mut changed[i].source;
                    match field {0=>s.tile_index^=1,1=>s.metatile_id^=1,2=>s.subtile_column^=1,
                        3=>s.subtile_row^=1,_=>s.tileset_id="house".into()}
                    let mut reserved=vec![false;24];let mut output=Vec::new();
                    candidates("CeladonDeptStore6F",&changed.iter().collect::<Vec<_>>(),&g,&mut reserved,&mut output);
                    assert!(output.is_empty());assert!(reserved.iter().all(|&v|!v));
                }
            }
            for map in ["UnknownStore","CeladonDeptStore2F"]{
                let mut output=Vec::new();candidates(map,&refs,&g,&mut vec![false;24],&mut output);assert!(output.is_empty());
                assert!(seat_rects(map,p,&refs[0].source).is_none());
            }
            let mut missing=tiles.clone();for t in &mut missing{if t.source.tile_index==1{t.source.tile_index=0x81;}}
            let mut output=Vec::new();candidates("CeladonDeptStore6F",&missing.iter().collect::<Vec<_>>(),&g,&mut vec![false;24],&mut output);
            assert!(output.is_empty());
        }
    }
    #[test]
    fn all_actual_store_benches_have_two_pads_one_connected_base_and_four_grounded_feet() {
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(
            std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external verified pack")).unwrap();
        let pack=loaded.pack();let art=&pack.runtime_files()["data/tilesets/mart_metatiles.bin"];
        for city in ["Celadon","Goldenrod"]{for (floor,expected)in[(1,4),(5,2),(6,4)]{
            let map=format!("{city}DeptStore{floor}F");let data=&pack.data().maps[&map];
            let(w,h)=(data.attributes.width as usize*4,data.attributes.height as usize*4);
            let tiles=(0..w*h).map(|i|{let(x,y)=(i%w,i/w);let block=data.blocks[y/4*(w/4)+x/4];
                VisualTile{column:x as u32,row:y as u32,texture:Default::default(),priority:false,animation_frames:None,
                    source:VisualTileSource{tileset_id:"mart".into(),metatile_id:block,subtile_column:(x%4)as u8,
                        subtile_row:(y%4)as u8,tile_index:art[block as usize*16+y%4*4+x%4]as u16}}
            }).collect::<Vec<_>>();let refs=tiles.iter().collect::<Vec<_>>();
            let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
            let all=super::super::resolve(&map,&refs,&g,None);
            let benches=all.iter().filter(|p|p.kind==ModelKind::Cushion).collect::<Vec<_>>();assert_eq!(benches.len(),expected,"{map}");
            let mut frame=crate::mesh::tests::frame(w as u32,h as u32,tiles.iter().map(|t|t.source.clone()).collect());
            frame.map_id=map.as_str().into();let seats=crate::mesh::seating_surfaces(&frame,None);
            assert_eq!(seats.len(),expected*2,"{map}: each physical pad remains independently seatable");
            for p in benches {
                assert!(matches(&map,p,&refs,&g));
                let mut mesh=TerrainMeshData{footing_heights:vec![2.5;w*h],..Default::default()};let mut claims=vec![false;w*h];
                assert!(super::super::append(&mut mesh,&map,&refs,&g,p,&mut claims,[0,0]));
                assert_eq!(claims.iter().filter(|&&x|x).count(),8);assert_eq!(mesh.footing_heights,vec![2.5;w*h]);
                let(n,x)=(p.row as f32*8.,p.column as f32*8.+8.);
                for z in [n+6.5,n+19.5]{assert!((top_at(&mesh.textured,x,z)-7.).abs()<0.0001,"{map}: both pad tops at the established seat datum");}
                assert!((top_at(&mesh.textured,x,n+13.)-5.5).abs()<0.0001,"{map}: common frame closes the narrow pad seam");
                assert_eq!(top_at(&mesh.textured,x,n+29.),0.,"{map}: perspective front folds upright, not extra physical bench depth");
                let mut feet_area=0.;
                for t in mesh.textured.indices.chunks_exact(3){
                    let[a,b,c]=[t[0],t[1],t[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
                    assert!((b-a).cross(c-a).dot(Vec3::from_array(mesh.textured.normals[t[0]as usize]))>0.);
                    if [a,b,c].iter().all(|v|v.y==0.)&&mesh.textured.normals[t[0]as usize][1]< -0.99 {feet_area+=(b-a).cross(c-a).length()*0.5;}
                    for v in [a,b,c]{if v.y>0.{assert!(v.z>=n&&v.z<=n+26.&&v.x>=x-8.&&v.x<=x+8.);}}
                }
                assert!((feet_area-4.*1.4*1.4).abs()<0.001,"{map}: four small grounded feet, not two independent stool frames");
                let rects=seat_rects(&map,p,&refs[p.row*w+p.column].source).unwrap();
                for rect in rects{
                    let seat=seats.iter().find(|s|s.rect==rect).expect("production seat extractor uses each pad");
                    assert_eq!(seat.height,7.);assert!(seat.yaw.is_none());
                    let point=frame.center+bevy::prelude::Vec2::new(
                        (rect[0]+rect[2]*0.5-w as f32*0.5)*8.,(h as f32*0.5-rect[1]-rect[3]*0.5)*8.);
                    assert_eq!(seat.target(&frame,point,0.),Some(Vec3::new(0.,7.,0.)));
                }
                for i in p.indices(w){let mut claimed=vec![false;w*h];claimed[i]=true;let before=claimed.clone();let mut output=Vec::new();
                    candidates(&map,&refs,&g,&mut claimed,&mut output);
                    assert!(!output.iter().any(|q|q.column==p.column&&q.row==p.row));
                    assert!(claimed[i]&&before[i]);
                }
            }
        }}
    }
}
