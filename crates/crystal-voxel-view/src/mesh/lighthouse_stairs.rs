//! Complete native stone flights. Warp plots retain their original support.
use super::*;

fn local(s:&VisualTileSource)->Option<(u8,u8)> {
    if s.tileset_id.as_ref()!="lighthouse" || !(2..4).contains(&s.subtile_column)
        || !(2..4).contains(&s.subtile_row) {return None;}
    let art=match s.metatile_id {0x31=>[[0x27,0x28],[0x37,0x38]],
        0x3a=>[[0x29,0x2a],[0x39,0x3a]],_=>return None};
    let (x,y)=(s.subtile_column-2,s.subtile_row-2);
    (s.tile_index==art[y as usize][x as usize]).then_some((x,y))
}
pub(super) fn resolve_into(r:&mut Resolver<'_>) {
    if !lighthouse_map(r.map) {return;}
    let Some(ground)=ground(r.map,r.cells,"lighthouse",0x2e) else{return;};
    let groups=grouped_flat_card_placements(r.cells,r.g,0x2e,false,|s|local(s).map(|(x,y)|(x,y,2,2)));
    for p in groups {
        if !phase_consistent(r.cells,r.g,p) {continue;}
        r.add(Placement{column:p.column,row:p.row,width:2,height:2,ground,
            form:Form::LighthouseStair{down:r.cells[p.row*r.g.width+p.column].source.metatile_id==0x31},
            rise_pixels:16.,depth_pixels:16.,front_rows:2.});
    }
}
pub(super) fn append(mesh:&mut TerrainMeshData,g:&GridGeometry,p:&Placement,down:bool) {
    let (w,_,n,_)=g.bounds(p.column,p.row);let k=g.tile_height/8.;
    let e=w+2.*g.tile_width;let s=n+2.*g.tile_height;
    let sample=|pixel:[usize;2]| {
        let uv=g.uv(p.column+pixel[0]/8,p.row+pixel[1]/8);
        [uv.0+(uv.1-uv.0)*((pixel[0]%8)as f32+0.5)/8.,
            uv.2+(uv.3-uv.2)*((pixel[1]%8)as f32+0.5)/8.]
    };
    let [stone,edge,ink]=if down {[[12,2],[1,0],[0,0]]}
        else{[[0,0],[1,2],[7,3]]}.map(sample);
    let slab=|out:&mut SurfaceMeshData,b,lo,hi,uv| {
        let start=out.positions.len();rooms::slab(out,b,lo,hi,[1.;4]);out.uvs[start..].fill(uv);
    };
    if down {
        slab(&mut mesh.textured,[w,e,n,s],-17.*k,-16.*k,ink);
        // Closed shaft cheeks with an open west approach to the first tread.
        for b in [[w,e,n,n+k],[w,e,s-k,s],[e-k,e,n+k,s-k]] {
            slab(&mut mesh.textured,b,-16.*k,k,edge);
        }
    } else {
        let uv=g.uv(p.ground%g.width,p.ground/g.width);
        for i in p.indices(g.width) {append_top(&mut mesh.textured,g.bounds(i%g.width,i/g.width).into(),0.,uv);}
    }
    let left=if down {w}else{w+k};let right=e-k;
    for step in 0..6 {
        let x0=left+(right-left)*step as f32/6.;let x1=left+(right-left)*(step+1)as f32/6.;
        let top=if down {-16.*k*(step+1)as f32/6.}else{16.*k*(step+1)as f32/6.};
        let base=if down {-17.*k}else{0.};
        slab(&mut mesh.textured,[x0,x1,n+k,s-k],base,top,stone);
    }
    if !down {
        for b in [[w,e,n,n+k],[w,e,s-k,s]] {
            slab(&mut mesh.textured,b,0.,16.*k,edge);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_lighthouse_stairs_match_all_fourteen_warps_and_keep_support() {
        let path=std::env::var_os("CRYSTAL_RENDER_TEST_PACK").map(std::path::PathBuf::from)
            .unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content-packs/text-tui.crystalpack"));
        if !path.is_file(){return;}
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();let pack=loaded.pack();
        let art=&pack.runtime_files()["data/tilesets/lighthouse_metatiles.bin"];
        let png=image::load_from_memory(&pack.runtime_files()["gfx/tilesets/lighthouse.png"]).unwrap().to_luma8();
        let mut total=0;
        for level in 1..=6 {
            let name=format!("OlivineLighthouse{level}F");let map=&pack.data().maps[&name];
            let (w,h)=(map.attributes.width as usize*4,map.attributes.height as usize*4);
            let tiles=(0..w*h).map(|i| {
                let (x,y)=(i%w,i/w);let b=map.blocks[y/4*(w/4)+x/4];
                VisualTile{column:x as u32,row:y as u32,texture:Default::default(),priority:false,animation_frames:None,
                    source:VisualTileSource{tileset_id:"lighthouse".into(),metatile_id:b,subtile_column:(x%4)as u8,
                        subtile_row:(y%4)as u8,tile_index:art[b as usize*16+y%4*4+x%4]as u16}}
            }).collect::<Vec<_>>();
            let refs=tiles.iter().collect::<Vec<_>>();let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
            let placements=super::super::resolve(&name,&refs,&g,None);
            let stairs=placements.iter().filter(|p|matches!(p.form,Form::LighthouseStair{..})).collect::<Vec<_>>();
            assert_eq!(stairs.len(),[1,2,3,4,3,1][level-1]);total+=stairs.len();
            for warp in &map.events.warps {
                let i=warp.y as usize*2*w+warp.x as usize*2;
                assert_eq!(stairs.iter().any(|p|p.indices(w).any(|j|i==j)),local(&tiles[i].source).is_some(),"{name}: floor/pit/carpet isn't a stair");
            }
            for p in stairs {
                let Form::LighthouseStair{down}=p.form else{unreachable!()};
                let mut mesh=TerrainMeshData{footing_heights:vec![4.5;w*h],..Default::default()};
                append(&mut mesh,&g,p,down);assert_eq!(mesh.footing_heights,vec![4.5;w*h]);
                assert!(mesh.solid.positions.is_empty());assert!(!mesh.textured.positions.is_empty());
                let samples=if down {[[12,2],[1,0],[0,0]]}else{[[0,0],[1,2],[7,3]]};
                for (pixel,value) in samples.into_iter().zip([170,85,0]) {
                    let s=&tiles[(p.row+pixel[1]/8)*w+p.column+pixel[0]/8].source;
                    assert_eq!(png.get_pixel((s.tile_index as u32%16)*8+(pixel[0]%8)as u32,
                        (s.tile_index as u32/16)*8+(pixel[1]%8)as u32)[0],value);
                }
                assert_eq!(mesh.textured.positions.iter().any(|p|p[1]<0.),down);
                for i in p.indices(w) {
                    let mut changed=tiles.clone();changed[i].source.tile_index^=1;
                    assert!(!super::super::resolve(&name,&changed.iter().collect::<Vec<_>>(),&g,None).iter()
                        .any(|q|matches!(q.form,Form::LighthouseStair{..})&&q.column==p.column&&q.row==p.row));
                }
            }
        }
        assert_eq!(total,14);
    }
}
