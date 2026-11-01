//! The standard Mart's continuous return and its separate mechanical register.
use super::*;

fn standard_mart(map:&str)->bool {
    matches!(map,"AzaleaMart"|"BlackthornMart"|"CherrygroveMart"|"EcruteakMart"
        |"OlivineMart"|"VioletMart"|"CeruleanMart"|"FuchsiaMart"|"LavenderMart"
        |"PewterMart"|"SaffronMart"|"VermilionMart"|"ViridianMart")
}
fn drawing(map:&str,column:usize,row:usize,cells:&[&VisualTile],g:&GridGeometry)->bool {
    if !standard_mart(map)||g.width.checked_mul(g.height)!=Some(cells.len())
        ||column+2>g.width||row<2||row+8>g.height {return false;}
    // The wall header authenticates the back join but is not claimed. The
    // last row is the south face, folded upright at the preceding top edge.
    let tiles=[[0x11,0x11],[0x2a,0x2b],[0x3e,0x3f],[0x3e,0x3f],
        [0x20,0x21],[0x30,0x31],[0x3e,0x3f],[0x3e,0x3f],[0x1e,0x2f],[0x1a,0x19]];
    (0..20).all(|j| {
        let y=j/2;let x=j%2;let s=&cells[(row-2+y)*g.width+column+x].source;
        s.tileset_id.as_ref()=="mart"&&s.metatile_id==[0x27,0x22,0x29][y/4]
            &&s.subtile_column==x as u8&&s.subtile_row==(y%4)as u8&&s.tile_index==tiles[y][x]
    })
}
pub(super) fn matches(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    p.kind==ModelKind::ReceptionCounter&&p.width==2&&p.height==8
        &&p.depth_pixels==56.&&p.height_pixels==8.&&p.base_pixels==0.
        &&drawing(map,p.column,p.row,cells,g)
}
pub(super) fn append_candidates(map:&str,cells:&[&VisualTile],g:&GridGeometry,
    claimed:&mut[bool],out:&mut Vec<Placement>) {
    if !standard_mart(map)||g.width<2||g.height<10||claimed.len()!=cells.len() {return;}
    let Some(ground)=ground_sample(cells,map,"mart",crate::mart::ground_tile_for_map(map)) else{return;};
    for row in 2..=g.height-8 {for column in 0..=g.width-2 {
        if !drawing(map,column,row,cells,g) {continue;}
        let p=Placement{kind:ModelKind::ReceptionCounter,column,row,width:2,height:8,
            ground,depth_pixels:56.,height_pixels:8.,base_pixels:0.,footing_pixels:None};
        if p.indices(g.width).any(|i|claimed[i]) {continue;}
        for i in p.indices(g.width) {claimed[i]=true;}
        out.push(p);
    }}
}
fn mart_counter_fascia_uv_from_placement(g:&GridGeometry,p:&Placement)->Option<[f32;3]> {
    let (u0,u1,v0,v1)=g.uv(p.column,p.row+p.height-1);
    Some([u0+(u1-u0)*3.5/8.,v0+(v1-v0)*0.5/8.,v0+(v1-v0)*7.5/8.])
}
pub(super) fn append(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,
    selected:&crate::interior_models::Model,b:[f32;4]) {
    let [w,e,n,s]=b;let sy=g.tile_height/8.;
    let swatches=mart_counter_swatches(g,p.row*g.width+p.column,0x3e).unwrap();
    // The drawers face the clerk on the west; one full cap joins the entire
    // return, without stretching the register drawing into cabinet panels.
    let mut body=SurfaceMeshData::default();
    assert!(selected.append_native_store_counter(&mut body,[n,s,-e,-w],7.*sy,swatches,true));
    // The canonical body adapter retains unused cap vertices for its material
    // roles. Copy only indexed body vertices, so their bounds cannot include
    // an invisible second countertop above the joined worktop.
    let body_first=out.positions.len();
    let mut remap=vec![None;body.positions.len()];
    for triangle in body.indices.chunks_exact(3) {
        let [a,b,c]=[triangle[0],triangle[1],triangle[2]].map(|i|Vec3::from_array(body.positions[i as usize]));
        if (b-a).cross(c-a).length_squared()==0. {continue;}
        for &index in triangle {
        let i=index as usize;
        let index=*remap[i].get_or_insert_with(|| {
            let index=out.positions.len()as u32;
            let v=body.positions[i];let normal=body.normals[i];
            let normal=[-normal[2],normal[1],normal[0]];
            out.positions.push([-v[2],v[1],v[0]]);out.normals.push(normal);
            out.uvs.push(body.uvs[i]);
            out.colors.push(crate::interior_models::authored_face_color([1.;4],Vec3::from_array(normal)));
            index
        });
        out.indices.push(index);
    }}
    paint_mart_counter_fascia(out,body_first,mart_counter_fascia_uv_from_placement(g,p),7.*sy);
    let first=out.positions.len();architecture_box(out,[w,e,7.*sy,8.*sy,n,s],[1.;3]);
    for i in first..out.positions.len() {out.uvs[i]=swatches[if out.normals[i][1]>0.5 {3}else{4}];}
    let register_north=g.origin_z+(p.row+2)as f32*g.tile_height;
    let first=out.positions.len();
    department_store::append_cash_register(out,[w,e,register_north,register_north+2.*g.tile_height],sy);
    // The canonical register supplies geometry only. Its four finishes come
    // from this actual source register, including the keys and receipt hood.
    let colors=native_source_swatches(p,g,[[0,16],[4,17],[1,16],[2,16]]);
    for i in first..out.positions.len() {
        let color=out.colors[i];let gray=(color[0]+color[1]+color[2])/3.;
        out.uvs[i]=colors[if gray<0.24 {0}else if gray<0.45 {1}else if gray<0.8 {2}else{3}];
        out.colors[i]=crate::interior_models::authored_face_color([1.;4],Vec3::from_array(out.normals[i]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ray_height(surface:&SurfaceMeshData,x:f32,z:f32)->f32 {
        surface.indices.chunks_exact(3).filter_map(|t| {
            let [a,b,c]=[t[0],t[1],t[2]].map(|i|Vec3::from_array(surface.positions[i as usize]));
            if surface.normals[t[0]as usize][1]<0.99 {return None;}
            let side=|a:Vec3,b:Vec3|(b.x-a.x)*(z-a.z)-(b.z-a.z)*(x-a.x);
            let signs=[side(a,b),side(b,c),side(c,a)];
            (signs.iter().all(|s|*s>=-0.0001)||signs.iter().all(|s|*s<=0.0001)).then_some(a.y)
        }).fold(f32::NEG_INFINITY,f32::max)
    }
    fn west_face_uv(mesh:&SurfaceMeshData,y:f32,z:f32)->Option<[f32;2]> {
        mesh.indices.chunks_exact(3).filter_map(|t| {
            if mesh.normals[t[0]as usize][0]> -0.99 {return None;}
            let [a,b,c]=[t[0],t[1],t[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
            let denominator=(b.z-c.z)*(a.y-c.y)+(c.y-b.y)*(a.z-c.z);
            if denominator.abs()<0.000001 {return None;}
            let u=((b.z-c.z)*(y-c.y)+(c.y-b.y)*(z-c.z))/denominator;
            let v=((c.z-a.z)*(y-c.y)+(a.y-c.y)*(z-c.z))/denominator;
            let w=1.-u-v;
            if [u,v,w].iter().any(|&x|x< -0.00001) {return None;}
            let uv=std::array::from_fn(|axis|u*mesh.uvs[t[0]as usize][axis]
                +v*mesh.uvs[t[1]as usize][axis]+w*mesh.uvs[t[2]as usize][axis]);
            Some((u*a.x+v*b.x+w*c.x,uv))
        }).min_by(|a,b|a.0.total_cmp(&b.0)).map(|(_,uv)|uv)
    }
    #[test]
    fn fascia_paint_preserves_geometry_caps_and_existing_unknown_source_fallback() {
        let selected=crate::interior_models::model(ModelKind::ReceptionCounter);
        let mut mesh=SurfaceMeshData::default();
        selected.append_native_store_counter(&mut mesh,[0.,16.,0.,32.],7.,[[0.2,0.3];5],true);
        let before=mesh.clone();
        paint_mart_counter_fascia(&mut mesh,0,None,7.);
        assert_eq!(mesh.uvs,before.uvs);
        paint_mart_counter_fascia(&mut mesh,0,Some([0.5,0.1,0.9]),7.);
        assert_eq!(mesh.positions,before.positions);assert_eq!(mesh.indices,before.indices);
        assert_eq!(mesh.normals,before.normals);assert_eq!(mesh.colors,before.colors);
        for i in 0..mesh.positions.len() {
            if mesh.normals[i][1].abs()>=0.99||mesh.positions[i][1]>7.0001 {
                assert_eq!(mesh.uvs[i],before.uvs[i],"cap/horizontal geometry keeps its existing finish");
            }
        }
    }
    #[test]
    fn actual_mart_checkouts_join_the_wall_and_end_with_a_seated_clerk_facing_register() {
        let path=std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external verified pack");
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(path).unwrap();
        let pack=loaded.pack();let art=&pack.runtime_files()["data/tilesets/mart_metatiles.bin"];
        let maps=pack.data().maps.keys().filter(|map|standard_mart(map)).collect::<Vec<_>>();
        assert_eq!(maps.len(),13);
        for map in maps {
            let data=&pack.data().maps[map];let (w,h)=(data.attributes.width as usize*4,data.attributes.height as usize*4);
            let tiles=(0..w*h).map(|i|{let (x,y)=(i%w,i/w);let block=data.blocks[y/4*(w/4)+x/4];
                VisualTile{column:x as u32,row:y as u32,texture:Default::default(),priority:false,animation_frames:None,
                    source:VisualTileSource{tileset_id:"mart".into(),metatile_id:block,subtile_column:(x%4)as u8,
                        subtile_row:(y%4)as u8,tile_index:art[block as usize*16+y%4*4+x%4]as u16}}
            }).collect::<Vec<_>>();let refs=tiles.iter().collect::<Vec<_>>();
            let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
            let placements=super::super::resolve(map,&refs,&g,None);
            let checkouts=placements.iter().filter(|p|matches(map,p,&refs,&g)).collect::<Vec<_>>();
            assert_eq!(checkouts.len(),1,"{map}: complete return, not only register/end fragments");let p=checkouts[0];
            assert_eq!((p.column,p.row),(4,2));
            let mut mesh=TerrainMeshData{footing_heights:vec![2.5;w*h],authored_cells:vec![None;w*h],..Default::default()};
            let mut claims=vec![false;w*h];assert!(super::super::append(&mut mesh,map,&refs,&g,p,&mut claims,[0,0]));
            assert_eq!(claims.iter().filter(|&&v|v).count(),16);
            assert_eq!(mesh.footing_heights,vec![2.5;w*h]);
            // Read visible UVs by ray interpolation across independently
            // overlapping body panels, rather than checking assigned vertices.
            // These are the six inner horizontal source courses (top to base).
            let sheet=image::load_from_memory(&pack.runtime_files()["gfx/tilesets/mart.png"]).unwrap().to_rgba8();
            for (source_row,expected_gray) in [(1,170),(2,85),(3,0),(4,85),(5,85),(6,85)] {
                let uv=west_face_uv(&mesh.textured,7.-source_row as f32,44.).expect("continuous exposed body face");
                let atlas_x=uv[0]*w as f32*8.;let atlas_y=uv[1]*h as f32*8.;
                assert_eq!((atlas_x/8.)as usize,p.column,"fascia samples its actual front strip");
                assert_eq!((atlas_y/8.)as usize,p.row+p.height-1);
                let pixel_y=(atlas_y%8.).floor()as u32;
                assert_eq!(pixel_y,source_row,"{map}: courses must stay horizontal across body frames and panels");
                assert_eq!(sheet.get_pixel((0x1a%(sheet.width()/8))*8+3,
                    (0x1a/(sheet.width()/8))*8+pixel_y)[0],expected_gray,
                    "{map}: source gold bands/dark separators must not become solid dark panels");
            }
            assert_eq!(claims,(0..w*h).map(|i|p.indices(w).any(|j|i==j)).collect::<Vec<_>>());
            let (west,north,south)=(32.,16.,72.);
            for y in 0..112 {let z=north+0.25+y as f32*0.5;
                assert!((ray_height(&mesh.textured,west+1.,z)-8.).abs()<0.0001,"{map}: continuous worktop at {z}");
            }
            let raised=mesh.textured.positions.iter().filter(|v|v[1]>8.0001).collect::<Vec<_>>();assert!(!raised.is_empty());
            assert!(raised.iter().all(|v|v[0]>west&&v[0]<west+16.&&v[2]>32.&&v[2]<48.&&v[1]<11.2));
            let base=mesh.textured.positions.iter().filter(|v|v[1]>=8.&&v[2]>32.&&v[2]<48.).map(|v|v[1]).fold(f32::INFINITY,f32::min);
            assert!((base-8.).abs()<0.0001,"register rests on the source worktop");
            // The vacated perspective/front row retains its floor underlay.
            // Only raised furniture must fit the folded physical footprint.
            assert!(mesh.textured.positions.iter().filter(|v|v[1]>0.0001)
                .all(|v|v[0]>=west-0.001&&v[0]<=west+16.001&&v[2]>=north-0.001&&v[2]<=south+0.001));
            assert!(mesh.textured.positions.iter().zip(&mesh.textured.normals).any(|(v,n)|
                v[1]>9.&&v[2]>32.&&v[2]<48.&&n[0]< -0.05&&n[1]>0.5),
                "{map}: raised keypad slopes toward the clerk on the west");
            for tri in mesh.textured.indices.chunks_exact(3) {
                let [a,b,c]=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
                assert!((b-a).cross(c-a).dot(Vec3::from_array(mesh.textured.normals[tri[0]as usize]))>0.,
                    "{map}: winding at {a:?} {b:?} {c:?}, normal={:?}",mesh.textured.normals[tri[0]as usize]);
            }
            for row in p.row-2..p.row+p.height {for x in 0..2 {
                let mut changed=tiles.clone();changed[row*w+p.column+x].source.tile_index^=1;
                assert!(!matches(map,p,&changed.iter().collect::<Vec<_>>(),&g),"{map}: altered drawing must reject the joined adapter");
            }}
            for i in p.indices(w) {
                let mut reserved=vec![false;w*h];reserved[i]=true;let mut output=Vec::new();
                append_candidates(map,&refs,&g,&mut reserved,&mut output);assert!(output.is_empty());
            }
            assert!(!matches("CeladonDeptStore1F",p,&refs,&g));assert!(!matches("CustomMart",p,&refs,&g));
        }
    }
    #[test]
    fn actual_store_fascia_finds_the_shared_front_in_all_eight_source_layouts() {
        let path=std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external verified pack");
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(path).unwrap();
        let pack=loaded.pack();let art=&pack.runtime_files()["data/tilesets/mart_metatiles.bin"];
        for city in ["Celadon","Goldenrod"] {for floor in 1..=4 {
            let map=format!("{city}DeptStore{floor}F");let data=&pack.data().maps[&map];
            let (w,h)=(data.attributes.width as usize*4,data.attributes.height as usize*4);
            let tiles=(0..w*h).map(|i|{let (x,y)=(i%w,i/w);let block=data.blocks[y/4*(w/4)+x/4];
                VisualTile{column:x as u32,row:y as u32,texture:Default::default(),priority:false,animation_frames:None,
                    source:VisualTileSource{tileset_id:"mart".into(),metatile_id:block,subtile_column:(x%4)as u8,
                        subtile_row:(y%4)as u8,tile_index:art[block as usize*16+y%4*4+x%4]as u16}}
            }).collect::<Vec<_>>();
            let refs=tiles.iter().collect::<Vec<_>>();
            let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
            let [u,top,bottom]=mart_counter_fascia_uv(&refs,&g).expect("every actual counter layout needs its native front courses");
            let x=(u*w as f32)as usize;let y=(top*h as f32)as usize;
            assert_eq!(tiles[y*w+x].source.tile_index,0x1a,"{map}: actual source front, not a dark top-edge swatch");
            assert!((bottom-top-7./(8.*h as f32)).abs()<0.000001);
            let mut foreign=tiles.clone();for t in &mut foreign {t.source.tileset_id="pokecenter".into();}
            assert!(mart_counter_fascia_uv(&foreign.iter().collect::<Vec<_>>(),&g).is_none());
        }}
    }
    #[test]
    fn actual_store_front_strips_join_returns_through_the_final_finisher() {
        let path=std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external verified pack");
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(path).unwrap();
        let pack=loaded.pack();let art=&pack.runtime_files()["data/tilesets/mart_metatiles.bin"];
        for (map,columns) in [("CeladonDeptStore1F",[18,24]),("GoldenrodDeptStore1F",[18,24]),
            ("CeladonDeptStore3F",[10,16]),("GoldenrodDeptStore3F",[10,16])] {
            let data=&pack.data().maps[map];let (w,h)=(data.attributes.width as usize*4,data.attributes.height as usize*4);
            let sources=(0..w*h).map(|i|{let (x,y)=(i%w,i/w);let block=data.blocks[y/4*(w/4)+x/4];
                VisualTileSource{tileset_id:"mart".into(),metatile_id:block,subtile_column:(x%4)as u8,
                    subtile_row:(y%4)as u8,tile_index:art[block as usize*16+y%4*4+x%4]as u16}
            }).collect();
            let mut frame=crate::mesh::tests::frame(w as u32,h as u32,sources);
            frame.map_id=map.into();frame.source_map_size_core_tiles=Some(bevy::prelude::UVec2::new(w as u32/2,h as u32/2));
            let mut images=Assets::<Image>::default();
            let texture=images.add(Image::new_fill(
                bevy::render::render_resource::Extent3d{width:8,height:8,depth_or_array_layers:1},
                bevy::render::render_resource::TextureDimension::D2,&[180,180,180,255],
                bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                bevy::asset::RenderAssetUsages::all()));
            for tile in &mut frame.tiles {tile.texture=texture.clone();}
            let built=crate::mesh::audit_terrain_mesh_with_images(&frame,&images).unwrap();
            for column in columns {for step in 0..64 {
                // Sweep from the rear return to the complete front strip;
                // neither perspective-face rows nor separate adapters may
                // create a trench in the actual eight-pixel worktop.
                let x=column as f32*8.+1.-w as f32*4.;
                let z=16.25+step as f32*0.5-h as f32*4.;
                assert!((ray_height(&built.textured,x,z)-8.).abs()<0.0001,
                    "{map}: joined cap missing at source column {column}, depth {}",16.25+step as f32*0.5);
            }}
        }
    }

}
