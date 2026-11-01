//! Whole source furniture owners exclude architectural backing and floor paint.
use super::*;

#[test]
fn actual_shared_ticket_vending_and_clinic_stock_keep_complete_separate_reveal_owners() {
    let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external verified pack")).unwrap();
    let pack=loaded.pack();let mut counts=std::collections::BTreeMap::<String,usize>::new();
    let mut rooms=std::collections::BTreeSet::new();
    for (map,data) in &pack.data().maps {
        let tileset=data.attributes.tileset_name.as_str();
        if !matches!(tileset,"mart"|"train_station"|"pokecenter") {continue;}
        let art=&pack.runtime_files()[&format!("data/tilesets/{tileset}_metatiles.bin")];
        let(w,h)=(data.attributes.width as usize*4,data.attributes.height as usize*4);
        let tiles=(0..w*h).map(|i|{let(x,y)=(i%w,i/w);let block=data.blocks[y/4*(w/4)+x/4];
            VisualTile{column:x as u32,row:y as u32,texture:Default::default(),priority:false,animation_frames:None,
                source:VisualTileSource{tileset_id:tileset.into(),metatile_id:block,subtile_column:(x%4)as u8,
                    subtile_row:(y%4)as u8,tile_index:art[block as usize*16+y%4*4+x%4]as u16}}
        }).collect::<Vec<_>>();let refs=tiles.iter().collect::<Vec<_>>();
        let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:-17.,origin_z:23.};
        let placements=resolve(map,&refs,&g,None);
        for p in &placements {
            let source=&refs[p.row*w+p.column].source;
            let (family,owners)=if native_station_ticket_source(map,&p,&refs,&g) {("station",4)}
                else if native_store_vending_source(map,&p,&refs,&g) {("beverage",1)}
                else if native_clinic::stock_source(map,&p,&refs,&g) {("clinic_stock",1)}
                else if reviewed_retail_room(map)&&p.kind==ModelKind::RetailShelf&&p.width==2&&p.height==4
                    &&source.tileset_id.as_ref()=="mart"&&source.tile_index==0x26 {("blue_aisle",1)}else{continue;};
            rooms.insert(map.clone());*counts.entry(family.into()).or_default()+=owners;
            if family=="blue_aisle"&&crate::mart::ground_tile_for_map(map)==crate::mart::STANDARD_MART_FLOOR_TILE {
                // Both $23/$28 variants must claim the entire authenticated
                // drawing. A changed cell cannot become a partial native case.
                assert!(matches!(source.metatile_id,0x23|0x28));
                for i in p.indices(w) {
                    let mut changed=tiles.clone();changed[i].source.tile_index^=1;
                    assert!(!resolve(map,&changed.iter().collect::<Vec<_>>(),&g,None).iter()
                        .any(|q|q.kind==p.kind&&q.column==p.column&&q.row==p.row),
                        "{map}: mutated case must retain source ownership");
                }
            }
            let mut mesh=TerrainMeshData{footing_heights:vec![3.25;w*h],..Default::default()};let mut claims=vec![false;w*h];
            assert!(append_with_tables(&mut mesh,map,&refs,&g,p,&mut claims,[0,0],&placements));
            assert_eq!(claims.iter().filter(|&&x|x).count(),p.width*p.height);
            assert_eq!(mesh.footing_heights,vec![3.25;w*h]);assert!(mesh.solid.indices.is_empty());
            let ranges=&mesh.textured.cutaway_ranges;assert_eq!(ranges.len(),owners,"{map}: {family}");
            assert_eq!(*ranges,mesh.depth_coverage_textured_ranges);
            assert!(ranges.windows(2).all(|r|r[0].end<=r[1].start));
            for range in ranges {assert!(!range.is_empty());assert!(mesh.textured.indices.chunks_exact(3)
                .any(|tri|tri.iter().all(|&i|range.contains(&(i as usize)))));}
            let mut architecture=0;
            for tri in mesh.textured.indices.chunks_exact(3) {
                let owned=tri.iter().filter(|&&i|ranges.iter().any(|r|r.contains(&(i as usize)))).count();
                assert!(owned==0||owned==3,"{map}: a triangle never crosses physical owners");
                let above=tri.iter().any(|&i|mesh.textured.positions[i as usize][1]>0.);
                if owned==0&&above {architecture+=1;assert_eq!(family,"clinic_stock","{map}: only backing remains above floor outside furniture");}
                if tri.iter().all(|&i|mesh.textured.positions[i as usize][1]==0.&&mesh.textured.normals[i as usize][1]>0.99) {
                    assert_eq!(owned,0,"{map}: the entire floor underlay stays opaque");
                }
                if owned==3 {assert!(ranges.iter().any(|r|tri.iter().all(|&i|r.contains(&(i as usize)))),"{map}: one complete physical owner per face");}
            }
            if family=="clinic_stock" {assert!(architecture>0,"Indigo's source wall backing must not fade with its stock");}
            let (opaque,_,mut groups,solid_groups)=crate::occluder_fade::prepare_linked(
                mesh.textured.clone(),mesh.solid.clone(),mesh.cutaway_links.clone());
            assert_eq!(groups.len(),owners,"{map}: physical owners survive real render grouping");
            assert!(solid_groups.is_empty()&&opaque.cutaway_ranges.is_empty()&&!opaque.indices.is_empty());
            crate::occluder_fade::use_source_depth_coverage_fade(&mut groups,&mesh.depth_coverage_textured_ranges);
            if family=="station" {
                let bounds=ranges.iter().map(|r|r.clone().map(|i|mesh.textured.positions[i][2])
                    .fold((f32::INFINITY,f32::NEG_INFINITY),|(a,b),z|(a.min(z),b.max(z)))).collect::<Vec<_>>();
                assert!(bounds.windows(2).all(|b|b[0].1<b[1].0),"station units own separated source modules, not an entire bank");
            }
        }
    }
    println!("actual shared owners {counts:?}; rooms={rooms:?}");
    assert_eq!(counts,[("beverage".into(),8),("blue_aisle".into(),48),
        ("clinic_stock".into(),2),("station".into(),4)].into_iter().collect());
    assert_eq!(rooms.len(),21);
}
