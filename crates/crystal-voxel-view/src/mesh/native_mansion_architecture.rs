//! Source-complete Mansion north stairs and blue wall courses.
use super::*;

fn room(map:&str)->bool {matches!(map,"CeladonMansion1F"|"CeladonMansion2F"|"CeladonMansion3F")}

pub(super) fn stair(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->Option<bool> {
    if !room(map)||p.kind!=ModelKind::StairFlight||p.width!=2||p.height!=2
        ||p.column+2>g.width||p.row+2>g.height||cells.len()!=g.width*g.height {return None;}
    let (_,_,down)=crate::interior::mansion_stair_local(&cells[p.row*g.width+p.column].source)?;
    (p.base_pixels==if down {-16.}else{0.} &&p.depth_pixels==16.&&p.height_pixels==22.4
        &&p.indices(g.width).enumerate().all(|(j,i)|
            crate::interior::mansion_stair_local(&cells[i].source)==Some(((j%2)as u8,(j/2)as u8,down))))
        .then_some(down)
}

pub(super) fn candidates(map:&str,cells:&[&VisualTile],g:&GridGeometry,claimed:&mut[bool],out:&mut Vec<Placement>) {
    if !room(map){return;}
    let Some(ground)=ground_sample(cells,map,"mansion",1) else{return;};
    for group in grouped_flat_card_placements(cells,g,1,false,|s|
        crate::interior::mansion_stair_local(s).map(|(x,y,_)|(x,y,2,2))) {
        let (_,_,down)=crate::interior::mansion_stair_local(&cells[group.row*g.width+group.column].source).unwrap();
        let p=Placement{kind:ModelKind::StairFlight,column:group.column,row:group.row,width:2,height:2,
            ground,depth_pixels:16.,height_pixels:22.4,base_pixels:if down {-16.}else{0.},footing_pixels:None};
        if stair(map,&p,cells,g).is_none()||p.indices(g.width).any(|i|claimed[i]){continue;}
        for i in p.indices(g.width){claimed[i]=true;}out.push(p);
    }
    for group in grouped_flat_card_placements(cells,g,1,false,wall_local) {
        let p=Placement{kind:ModelKind::WallClinical,column:group.column,row:group.row,
            width:group.width,height:2,ground,depth_pixels:3.,height_pixels:32.,base_pixels:0.,footing_pixels:None};
        if p.indices(g.width).any(|i|claimed[i]) {continue;}
        for i in p.indices(g.width){claimed[i]=true;}out.push(p);
    }
    for group in grouped_flat_card_placements(cells,g,1,false,vertical_local) {
        let p=Placement{kind:ModelKind::WallClinical,column:group.column,row:group.row,
            width:2,height:group.height,ground,depth_pixels:group.height as f32*8.,
            height_pixels:32.,base_pixels:0.,footing_pixels:None};
        if p.indices(g.width).any(|i|claimed[i]) {continue;}
        for i in p.indices(g.width){claimed[i]=true;}out.push(p);
    }
}

fn vertical_local(s:&VisualTileSource)->Option<(u8,u8,usize,usize)> {
    if s.tileset_id.as_ref()!="mansion"||s.subtile_column<2||s.subtile_column>=4 {return None;}
    let drawing:&[[u16;2]]=match s.metatile_id {
        9=>&[[3,5],[0x14,0x15]],
        0x0a=>&[[0x0c,0x0f],[0x1c,0x0f],[0x0e,0x0f],[0x0e,0x0f]],
        0x0b=>&[[0x0e,0x0f];4],
        0x0c=>&[[0x0e,0x0f],[0x1e,0x1f]],
        0x0d=>&[[0x45,0x55],[0x0e,0x0f],[0x0e,0x0f],[0x0e,0x0f]],_=>return None,
    };
    let x=s.subtile_column-2;let y=s.subtile_row as usize;
    (drawing.get(y).is_some_and(|row|row[x as usize]==s.tile_index))
        .then_some((x,s.subtile_row,2,drawing.len()))
}

fn wall_local(s:&VisualTileSource)->Option<(u8,u8,usize,usize)> {
    if s.tileset_id.as_ref()!="mansion"||s.subtile_row>=2 {return None;}
    let (start,width,striped)=match s.metatile_id {
        4=>(0,4,false),5=>(2,2,false),6|0x0d|0x17=>(0,2,false),
        7|8|0x12=>(0,4,true),9|0x0a=>(0,2,true),0x11=>(2,2,true),_=>return None,
    };
    let x=s.subtile_column.checked_sub(start)?;
    if x as usize>=width {return None;}
    let tile=if striped {if s.subtile_row==0 {3}else{0x13}}else{0x12};
    (s.tile_index==tile).then_some((x,s.subtile_row,width,2))
}

pub(super) fn wall(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    if !room(map)||p.kind!=ModelKind::WallClinical||p.base_pixels!=0.
        ||p.column+p.width>g.width||p.row+p.height>g.height||cells.len()!=g.width*g.height{return false;}
    let block=cells[p.row*g.width+p.column].source.metatile_id;
    if p.width==2&&p.indices(g.width).enumerate().all(|(j,i)|cells[i].source.metatile_id==block
        &&vertical_local(&cells[i].source)==Some(((j%2)as u8,(j/2)as u8,2,p.height))) {return true;}
    if p.height!=2 {return false;}
    p.indices(g.width).enumerate().all(|(j,i)|cells[i].source.metatile_id==block
        &&wall_local(&cells[i].source)==Some(((j%p.width)as u8,(j/p.width)as u8,p.width,2)))
}

pub(super) fn pixels(down:bool)->[[usize;2];5] {
    if down {[[2,4],[2,4],[1,12],[7,8],[7,8]]}
    else {[[14,1],[14,1],[1,1],[9,5],[9,5]]}
}

pub(super) fn append_wall(mesh:&mut TerrainMeshData,g:&GridGeometry,p:&Placement,
    cells:&[&VisualTile],[w,e,n,s]:[f32;4]) {
    let (donor,pixel)=blue_donor(p,g,cells);
    let uv=g.uv(donor%g.width,donor/g.width);
    let blue=[uv.0+(uv.1-uv.0)*(pixel[0]as f32+0.5)/8.,
        uv.2+(uv.3-uv.2)*(pixel[1]as f32+0.5)/8.];
    let out=&mut mesh.textured;
    let first=out.positions.len();
    let indices=out.indices.len();
    architecture_box(out,[w,e,0.,32.*g.tile_height/8.,n,s],[1.;3]);
    out.uvs[first..].fill(blue);
    out.indices.drain(indices..indices+6);
    // Mansion's blue palette has no neutral middle tone. The cap is stone,
    // using the existing19/31 gray architectural finish in linear color.
    let gray=((19.0f32/31.+0.055)/1.055).powf(2.4);
    let h=32.*g.tile_height/8.;
    append_quad(&mut mesh.solid,[[w,h,n],[w,h,s],[e,h,s],[e,h,n]],
        Vec3::Y.to_array(),[[0.;2];4],[gray,gray,gray,1.]);
}

fn blue_donor(p:&Placement,g:&GridGeometry,cells:&[&VisualTile])->(usize,[usize;2]) {
    let first=p.row*g.width+p.column;
    if vertical_local(&cells[first].source).is_some() {
        if let Some(i)=cells.iter().position(|t|t.source.tileset_id.as_ref()=="mansion"&&t.source.tile_index==0x13) {
            return (i,[0,4]);
        }
        // A clipped straight side still owns a blue-highlight pixel.
        if let Some(i)=p.indices(g.width).find(|&i|cells[i].source.tile_index==0x0f) {return (i,[5,0]);}
    }
    if cells[first].source.tile_index==0x12 {(first,[0,2])}
        else {(first+g.width,[0,4])}
}

pub(super) fn append_stair(out:&mut SurfaceMeshData,map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry,
    [w,e,n,s]:[f32;4],selected:&crate::interior_models::Model) {
    let down=stair(map,p,cells,g).expect("complete Mansion flight");
    // Fit before rotation so unequal native tile axes retain correct normals.
    let bounds=if down {[n,s,-e,-w]}else{[-s,-n,w,e]};
    let first=out.positions.len();
    assert!(selected.append_native_stair(out,bounds,p.base_pixels*g.tile_height/8.,
        22.4*g.tile_height/8.,native_source_swatches(p,g,pixels(down))));
    let rotate=|v:[f32;3]|if down {[-v[2],v[1],v[0]]}else{[v[2],v[1],-v[0]]};
    for point in &mut out.positions[first..]{*point=rotate(*point);}
    for (normal,color) in out.normals[first..].iter_mut().zip(&mut out.colors[first..]) {
        *normal=rotate(*normal);
        *color=crate::interior_models::authored_face_color([1.;4],Vec3::from_array(*normal));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn packed_mansion_flights_and_blue_walls_have_exact_sources_rotated_bounds_and_support() {
        let explicit=std::env::var_os("CRYSTAL_RENDER_TEST_PACK");
        let path=explicit.as_ref().map(std::path::PathBuf::from).unwrap_or_else(||
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content-packs/text-tui.crystalpack"));
        assert!(explicit.is_none()||path.is_file());if !path.is_file(){return;}
        let pack=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let files=pack.pack().runtime_files();let layout=&files["data/tilesets/mansion_metatiles.bin"];
        let png=image::load_from_memory(&files["gfx/tilesets/mansion.png"]).unwrap().to_luma8();
        let palette=&pack.pack().data().tilesets["mansion"].palette_map;
        for map in ["CeladonMansion1F","CeladonMansion2F","CeladonMansion3F"] {
            let module=&pack.pack().data().maps[map];
            let (w,h)=(module.attributes.width as usize*4,module.attributes.height as usize*4);
            let tiles=(0..w*h).map(|i| {let x=i%w;let y=i/w;
                let block=module.blocks[y/4*(w/4)+x/4];
                VisualTile{column:x as u32,row:y as u32,texture:Default::default(),priority:false,animation_frames:None,
                    source:VisualTileSource{tileset_id:"mansion".into(),metatile_id:block,
                        subtile_column:(x%4)as u8,subtile_row:(y%4)as u8,
                        tile_index:layout[block as usize*16+y%4*4+x%4]as u16}}
            }).collect::<Vec<_>>();let refs=tiles.iter().collect::<Vec<_>>();
            for (tw,th) in [(8.,8.),(5.,9.),(11.,6.)] {
                let g=GridGeometry{width:w,height:h,tile_width:tw,tile_height:th,origin_x:3.,origin_z:-7.};
                let placements=resolve(map,&refs,&g,None);
                let flights=placements.iter().filter(|p|stair(map,p,&refs,&g).is_some()).collect::<Vec<_>>();
                assert_eq!(flights.len(),if map=="CeladonMansion1F"{2}else{4});
                let walls=placements.iter().filter(|p|wall(map,p,&refs,&g)).collect::<Vec<_>>();
                assert!(!walls.is_empty());
                for p in flights.into_iter().chain(walls) {
                    let down=stair(map,p,&refs,&g);
                    let mut mesh=TerrainMeshData{footing_heights:vec![2.75;w*h],..Default::default()};
                    let mut claims=vec![false;w*h];
                    assert!(super::super::append(&mut mesh,map,&refs,&g,p,&mut claims,[0,0]));
                    assert_eq!(claims.iter().filter(|&&v|v).count(),p.width*p.height);
                    assert_eq!(mesh.footing_heights,vec![2.75;w*h]);
                    if let Some(down)=down {
                        assert!(mesh.solid.indices.is_empty());
                        assert!(stair("MobileTradeRoom",p,&refs,&g).is_none());
                        let (w0,e0,n0,s0)=(g.origin_x+p.column as f32*tw,
                            g.origin_x+(p.column+2)as f32*tw,g.origin_z+p.row as f32*th,
                            g.origin_z+(p.row+2)as f32*th);
                        for v in &mesh.textured.positions[16..] {
                            assert!(v[0]>=w0-0.001&&v[0]<=e0+0.001&&v[2]>=n0-0.001&&v[2]<=s0+0.001);
                            assert!(v[1]>=p.base_pixels*th/8.-0.001&&v[1]<=(p.base_pixels+22.4)*th/8.+0.001);
                        }
                        for (pixel,value) in pixels(down).into_iter().zip([85,85,255,170,170]) {
                            let raw=refs[(p.row+pixel[1]/8)*w+p.column+pixel[0]/8].source.tile_index as usize;
                            assert_eq!(palette[raw],0);
                            assert_eq!(png.get_pixel((raw%16*8+pixel[0]%8)as u32,
                                (raw/16*8+pixel[1]%8)as u32).0[0],value);
                        }
                        let paint=native_source_swatches(p,&g,pixels(down));
                        assert!(mesh.textured.uvs[16..].iter().all(|uv|paint.contains(uv)));
                        for (j,i) in p.indices(w).enumerate() {
                            let expected=if down {(-16.+(j/2)as f32*8.,-8.+(j/2)as f32*8.)}
                                else{(16.-(j/2)as f32*8.,8.-(j/2)as f32*8.)};
                            assert_eq!(crate::interior::mansion_stair_shape(&refs[i].source),
                                Some(CellShape::RampNorth{north_height:expected.0,south_height:expected.1}));
                        }
                    }else{
                        assert!(mesh.textured.positions.iter().all(|v|v[1]>=0.&&v[1]<=32.*th/8.+0.001));
                        assert!(mesh.textured.positions.iter().any(|v|(v[1]-32.*th/8.).abs()<0.001));
                        for i in p.indices(w){assert_eq!(palette[refs[i].source.tile_index as usize],3);}
                        let (donor,pixel)=blue_donor(p,&g,&refs);
                        let uv=g.uv(donor%w,donor/w);
                        let blue=[uv.0+(uv.1-uv.0)*(pixel[0]as f32+0.5)/8.,
                            uv.2+(uv.3-uv.2)*(pixel[1]as f32+0.5)/8.];
                        let first=p.width*p.height*4;
                        assert!(mesh.textured.uvs[first..].iter().all(|uv|*uv==blue));
                        assert_eq!(mesh.solid.positions.len(),4);
                        assert!(mesh.solid.positions.iter().all(|v|(v[1]-32.*th/8.).abs()<0.001));
                        assert!(mesh.solid.colors.iter().all(|c|c[0]==c[1]&&c[1]==c[2]&&c[0]>0.&&c[0]<1.),
                            "gray cap cannot inherit the side's blue palette");
                        let raw=refs[donor].source.tile_index as usize;
                        assert_eq!(png.get_pixel((raw%16*8+pixel[0]%8)as u32,
                            (raw/16*8+pixel[1]%8)as u32).0[0],85);
                    }
                    for tri in mesh.textured.indices.chunks_exact(3) {
                        let a=Vec3::from_array(mesh.textured.positions[tri[0]as usize]);
                        let b=Vec3::from_array(mesh.textured.positions[tri[1]as usize]);
                        let c=Vec3::from_array(mesh.textured.positions[tri[2]as usize]);
                        let normal=Vec3::from_array(mesh.textured.normals[tri[0]as usize]);
                        assert!((b-a).cross(c-a).dot(normal)>=-0.001,"rotation retains winding");
                    }
                    for i in p.indices(w) {
                        let mut damaged=tiles.clone();damaged[i].source.tile_index=0xffff;
                        let changed=damaged.iter().collect::<Vec<_>>();
                        assert!(stair(map,p,&changed,&g).is_none()&&!wall(map,p,&changed,&g));
                    }
                }
            }
        }
    }
}
