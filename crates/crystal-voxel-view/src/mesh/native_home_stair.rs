//! Complete domestic/facility flights use their own stair drawing and palette.
use super::*;

pub(super) fn source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)
    ->Option<[[usize;2];5]> {
    if !(reviewed_native_room(map)||matches!(map,"WiseTriosRoom"|"TrainerHouseB1F"
        |"TeamRocketBaseB2F"|"TeamRocketBaseB3F"))||p.kind!=ModelKind::StairFlight||p.width!=2||p.height!=2
        ||p.column+2>g.width||p.row+2>g.height||cells.len()!=g.width*g.height
        ||p.height_pixels!=22.4||p.depth_pixels!=16. {return None;}
    let first=&cells[p.row*g.width+p.column].source;
    if !matches!(first.tileset_id.as_ref(),"players_house"|"players_room"|"house"|"traditional_house"|"facility"){return None;}
    let (x,y,kind)=house_stair_local(map,first)?;
    if x!=0||y!=0||p.base_pixels!=if kind==crate::players_house::StairKind::UpEast {0.}else{-16.}
        ||!p.indices(g.width).enumerate().all(|(j,i)|house_stair_local(map,&cells[i].source)
            ==Some(((j%2)as u8,(j/2)as u8,kind))){return None;}
    Some(match first.tileset_id.as_ref() {
        "house"=>[[1,0],[1,0],[13,2],[12,3],[12,3]],
        "traditional_house"=>[[2,1],[2,1],[13,2],[12,3],[12,3]],
        "facility" if kind==crate::players_house::StairKind::UpEast=>
            [[2,11],[2,11],[3,0],[1,2],[1,2]],
        // This descending drawing has no white tread pixels. Preserve its
        // darker source tones instead of borrowing white from the upper flight.
        "facility"=>[[1,0],[1,0],[2,2],[2,8],[2,8]],
        _ if kind==crate::players_house::StairKind::UpEast=>
            [[14,1],[14,1],[4,1],[12,6],[12,6]],
        _=>[[2,3],[2,7],[5,12],[10,8],[10,8]],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_home_flights_use_complete_own_source_paint_and_preserve_support_geometry() {
        let explicit=std::env::var_os("CRYSTAL_RENDER_TEST_PACK");
        let path=explicit.as_ref().map(std::path::PathBuf::from).unwrap_or_else(||
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content-packs/text-tui.crystalpack"));
        assert!(explicit.is_none()||path.is_file());if !path.is_file(){return;}
        let pack=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let mut maps=std::collections::BTreeMap::new();
        for (map,module) in &pack.pack().data().maps {
            let tileset=module.attributes.tileset_name.as_str();
            if !matches!(tileset,"players_house"|"players_room"|"house"|"traditional_house"|"facility"){continue;}
            let layout=&pack.pack().runtime_files()[&format!("data/tilesets/{tileset}_metatiles.bin")];
            let png=image::load_from_memory(&pack.pack().runtime_files()[&format!("gfx/tilesets/{tileset}.png")]).unwrap().to_luma8();
            let palette=&pack.pack().data().tilesets[tileset].palette_map;
            let (w,h)=(module.attributes.width as usize*4,module.attributes.height as usize*4);
            let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
            let tiles=(0..w*h).map(|i| {let x=i%w;let y=i/w;
                let block=module.blocks[y/4*(w/4)+x/4];
                VisualTile{column:x as u32,row:y as u32,texture:Default::default(),priority:false,animation_frames:None,
                    source:VisualTileSource{tileset_id:tileset.into(),metatile_id:block,subtile_column:(x%4)as u8,
                        subtile_row:(y%4)as u8,tile_index:layout[block as usize*16+y%4*4+x%4]as u16}}
            }).collect::<Vec<_>>();
            let refs=tiles.iter().collect::<Vec<_>>();
            for p in resolve(map,&refs,&g,None) {
                let Some(pixels)=source(map,&p,&refs,&g) else {continue;};
                *maps.entry(map.clone()).or_insert(0usize)+=1;
                assert!(source("UnreviewedHouse",&p,&refs,&g).is_none());
                let values=if tileset=="facility"&&p.base_pixels<0. {[85,85,170,170,170]}
                    else{[85,85,255,170,170]};
                for (pixel,value) in pixels.into_iter().zip(values) {
                    let raw=refs[(p.row+pixel[1]/8)*w+p.column+pixel[0]/8].source.tile_index as usize;
                    assert_eq!(palette[raw],match tileset {"players_room"=>0,"facility"=>6,_=>5});
                    assert_eq!(png.get_pixel((raw%16*8+pixel[0]%8)as u32,(raw/16*8+pixel[1]%8)as u32).0[0],value);
                }
                let mut native=TerrainMeshData{footing_heights:vec![2.75;w*h],..Default::default()};
                let mut claims=vec![false;w*h];
                assert!(super::super::append(&mut native,map,&refs,&g,&p,&mut claims,[0,0]));
                let mut ordinary=TerrainMeshData{footing_heights:vec![2.75;w*h],..Default::default()};
                assert!(super::super::append(&mut ordinary,"UnreviewedHouse",&refs,&g,&p,&mut vec![false;w*h],[0,0]));
                assert!(native.solid.indices.is_empty());assert!(!ordinary.solid.indices.is_empty());
                assert_eq!(claims.iter().filter(|&&v|v).count(),4);
                assert_eq!(native.footing_heights,ordinary.footing_heights);
                assert_eq!(native.footing_heights,vec![2.75;w*h]);
                for (j,i) in p.indices(w).enumerate() {
                    assert_eq!(shape_for_source_on_map(map,&refs[i].source),
                        CellShape::RampEast{west_height:p.base_pixels+(j%2)as f32*8.,
                            east_height:p.base_pixels+(j%2+1)as f32*8.},"{map}: exact flight support");
                    if tileset=="facility" {
                        let src=&refs[i].source;
                        let tile=if (src.subtile_column+src.subtile_row)%2==0 {1}else{0x26};
                        let donor=ground_sample(&refs,map,"facility",tile).unwrap();
                        let (u0,u1,v0,v1)=g.uv(donor%w,donor/w);
                        assert!(native.textured.uvs[j*4..j*4+4].iter().all(|uv|
                            [[u0,v0],[u0,v1],[u1,v0],[u1,v1]].contains(uv)),
                            "{map}: checker continues under the complete flight");
                    }
                }
                assert_eq!(native.textured.positions[16..],ordinary.solid.positions);
                assert_eq!(native.textured.normals[16..],ordinary.solid.normals);
                let swatches=native_source_swatches(&p,&g,pixels);
                assert!(native.textured.uvs[16..].iter().all(|uv|swatches.contains(uv)));
                for i in p.indices(w) {
                    let mut changed=tiles.clone();changed[i].source.tile_index=0xffff;
                    assert!(source(map,&p,&changed.iter().collect::<Vec<_>>(),&g).is_none());
                }
            }
        }
        for map in ["PlayersHouse1F","PlayersHouse2F","RedsHouse1F","RedsHouse2F",
            "CopycatsHouse1F","CopycatsHouse2F"] {assert_eq!(maps.get(map),Some(&1));}
        for (map,count) in [("TrainerHouse1F",1),("WiseTriosRoom",1),("TrainerHouseB1F",1),
            ("TeamRocketBaseB2F",5),("TeamRocketBaseB3F",4)] {assert_eq!(maps.get(map),Some(&count));}
        assert_eq!(maps.len(),11);
        println!("Native domestic/facility stair maps: {}",maps.keys().cloned().collect::<Vec<_>>().join(","));
    }
}
