//! Native blue courses folded below Karen's unchanged platform datum.
use super::*;

#[derive(Clone,Debug,PartialEq,Eq)]
pub(super) struct Paint {
    courses:[usize;2],
    void:usize,
}
pub(super) fn paint(plan:&source::Plan,cells:&[&VisualTile],width:usize)->Option<Paint> {
    let first=*plan.cells.iter().find(|&&i| {
        let s=&cells[i].source;
        s.metatile_id==0x19 && s.subtile_row==2 && s.tile_index==0x13
            && plan.cells.contains(&(i+width))
            && cells.get(i+width).is_some_and(|t|t.source.metatile_id==0x19
                && t.source.subtile_row==3 && t.source.tile_index==0x12)
    })?;
    let void=*plan.cells.iter().find(|&&i|cells[i].source.metatile_id==0x17
        && cells[i].source.tile_index==0)?;
    Some(Paint {courses:[first,first+width],void})
}
pub(super) fn append_geometry(mesh:&mut SurfaceMeshData,g:&GridGeometry,
    plan:&source::Plan,paint:&Paint) {
    let unit=g.tile_height/SOURCE_TILE_HEIGHT;
    let depth=DEPTH_PIXELS*unit;
    let void=g.uv(paint.void%g.width,paint.void/g.width);
    let black=[(void.0+void.1)*0.5,(void.2+void.3)*0.5];
    for &i in &plan.cells {
        let (w,e,n,s)=g.bounds(i%g.width,i/g.width);
        append_quad(mesh,[[w,-depth,n],[w,-depth,s],[e,-depth,s],[e,-depth,n]],
            [0.,1.,0.],[black;4],TEXTURED_SHADE);
    }
    let mut masks=vec![0u8;g.width*g.height];
    for edge in &plan.edges {if edge.rim {masks[edge.cell]|=1<<edge.side;}}
    for edge in &plan.edges {
        let (w,e,n,s)=g.bounds(edge.cell%g.width,edge.cell/g.width);
        let mask=masks[edge.cell];
        let iw=w+if mask&8!=0 {g.tile_width*0.095}else{0.};
        let ie=e-if mask&2!=0 {g.tile_width*0.095}else{0.};
        let inn=n+if mask&1!=0 {g.tile_height*0.095}else{0.};
        let ins=s-if mask&4!=0 {g.tile_height*0.095}else{0.};
        let (a,b,ia,ib,normal)=match edge.side {
            0=>([e,n],[w,n],[ie,inn],[iw,inn],[0.,0.,1.]),
            1=>([e,s],[e,n],[ie,ins],[ie,inn],[-1.,0.,0.]),
            2=>([w,s],[e,s],[iw,ins],[ie,ins],[0.,0.,-1.]),
            _=>([w,n],[w,s],[iw,inn],[iw,ins],[1.,0.,0.]),
        };
        if edge.rim {
            let uv=g.uv(paint.courses[0]%g.width,paint.courses[0]/g.width);
            let v=uv.2+(uv.3-uv.2)*0.5/8.;
            append_quad(mesh,[[a[0],0.,a[1]],[b[0],0.,b[1]],
                [ib[0],0.,ib[1]],[ia[0],0.,ia[1]]],[0.,1.,0.],
                [[uv.0,v],[uv.1,v],[uv.1,v],[uv.0,v]],TEXTURED_SHADE);
        }
        let (a,b)=if edge.rim {(ia,ib)}else{(a,b)};
        for row in 0..3 {
            let top=-(row as f32)*8.*unit;let bottom=top-8.*unit;
            let uv=if edge.rim && row<2 {
                let source=paint.courses[row];let uv=g.uv(source%g.width,source/g.width);
                [[uv.0,uv.2],[uv.1,uv.2],[uv.1,uv.3],[uv.0,uv.3]]
            }else{[black;4]};
            append_quad(mesh,[[a[0],top,a[1]],[b[0],top,b[1]],
                [b[0],bottom,b[1]],[a[0],bottom,a[1]]],normal,uv,TEXTURED_SHADE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_karen_native_recesses_preserve_platform_support_and_source_guards() {
        let path=std::path::PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap());
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(path).unwrap();
        let pack=loaded.pack();let map=&pack.data().maps["KarensRoom"];
        let art=&pack.runtime_files()["data/tilesets/elite_four_room_metatiles.bin"];
        let (w,h)=(map.attributes.width as usize*4,map.attributes.height as usize*4);
        let tiles=(0..w*h).map(|i| {
            let (x,y)=(i%w,i/w);let block=map.blocks[y/4*(w/4)+x/4];
            VisualTile {column:x as u32,row:y as u32,priority:false,animation_frames:None,texture:Default::default(),
                source:VisualTileSource {tileset_id:"elite_four_room".into(),metatile_id:block,
                    subtile_column:(x%4)as u8,subtile_row:(y%4)as u8,
                    tile_index:art[block as usize*16+y%4*4+x%4]as u16}}
        }).collect::<Vec<_>>();
        let g=GridGeometry {width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let cells=tiles.iter().collect::<Vec<_>>();let reserved=vec![false;w*h];
        let p=resolve("KarensRoom",&cells,&g,None,&reserved).unwrap();
        let expected=tiles.iter().enumerate().filter(|(_,t)|t.source.metatile_id==0x17
            ||(t.source.metatile_id==0x19 && t.source.subtile_row>=2))
            .map(|(i,_)|i).collect::<Vec<_>>();
        assert_eq!(p.indices().collect::<Vec<_>>(),expected);
        assert_eq!(expected.len(),160);
        assert!(p.plan.edges.iter().any(|edge|edge.rim));
        let mut mesh=TerrainMeshData {authored_cells:vec![None;w*h],footing_heights:vec![2.75;w*h],..Default::default()};
        let mut claimed=reserved.clone();assert!(append(&mut mesh,&cells,&g,&p,&mut claimed));
        assert!(mesh.solid.positions.is_empty());
        assert_eq!(mesh.footing_heights,vec![2.75;w*h]);
        assert_eq!(claimed.iter().filter(|&&v|v).count(),expected.len());
        for (i,cell) in mesh.authored_cells.iter().enumerate() {
            assert_eq!(*cell,expected.contains(&i).then_some("league:karen-recess-depth"));
        }
        let mut bottom_area=0.;let mut rim_area=0.;
        for tri in mesh.textured.indices.chunks_exact(3) {
            let points=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
            let normal=Vec3::from_array(mesh.textured.normals[tri[0]as usize]);
            let area=(points[1]-points[0]).cross(points[2]-points[0]);
            assert!(area.dot(normal)>0.,"all native recess faces point into the void");
            assert!(points.iter().all(|p|(-24.0..=0.0).contains(&p.y)));
            if normal==Vec3::Y {
                if points[0].y==-24. {bottom_area+=area.length()*0.5;}
                else {
                    rim_area+=area.length()*0.5;
                    let center=points.iter().copied().sum::<Vec3>()/3.;
                    let i=(center.z/8.)as usize*w+(center.x/8.)as usize;
                    assert!(expected.contains(&i),"coping stays inside blocked source cells");
                }
            }
        }
        assert_eq!(bottom_area,expected.len()as f32*64.);
        assert!(rim_area>0. && rim_area<expected.len()as f32*64.*0.2,
            "coping cannot become a replacement level-zero floor");
        let paint=p.native.as_ref().unwrap();
        let black=g.uv(paint.void%w,paint.void/w);
        let mut allowed=vec![[(black.0+black.1)*0.5,(black.2+black.3)*0.5]];
        for index in paint.courses {
            let uv=g.uv(index%w,index/w);
            allowed.extend([[uv.0,uv.2],[uv.1,uv.2],[uv.1,uv.3],[uv.0,uv.3]]);
            if index==paint.courses[0] {
                let v=uv.2+(uv.3-uv.2)*0.5/8.;allowed.extend([[uv.0,v],[uv.1,v]]);
            }
        }
        assert!(mesh.textured.uvs.iter().all(|uv|allowed.contains(uv)),
            "every course and coping samples only native blue lip art or native black void");
        assert!(mesh.textured.colors.iter().all(|&c|c==TEXTURED_SHADE));
        for index in paint.courses {
            let mut altered=tiles.clone();altered[index].source.tile_index^=1;
            let mut target=TerrainMeshData::default();let mut claimed=reserved.clone();
            assert!(!append(&mut target,&altered.iter().collect::<Vec<_>>(),&g,&p,&mut claimed));
            assert!(target.textured.positions.is_empty() && claimed==reserved);
        }
        assert!(resolve("WillsRoom",&cells,&g,None,&reserved).is_none());
        let custom:Document=serde_json::from_str(r#"{"objects":[{"name":"Custom recess","map":"KarensRoom","tileset":"elite_four_room","metatile":23,"origin":[0,0],"tiles":[[0]],"ground":1,"top_pixels":0,"depth_pixels":8}]}"#).unwrap();
        assert!(resolve("KarensRoom",&cells,&g,Some(&custom),&reserved).is_none(),
            "custom void ownership also removes the native black donor; keep the source edge drawing");
        let mut reserved=reserved.clone();reserved[paint.courses[0]]=true;
        let remaining=resolve("KarensRoom",&cells,&g,None,&reserved).unwrap();
        assert_eq!(remaining.indices().count(),152,
            "a reserved cell protects its complete eight-cell lip, retaining unrelated recesses");
    }
}
