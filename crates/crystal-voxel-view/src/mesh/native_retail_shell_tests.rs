//! Actual source layouts must have complete, grounded cabinet shells from every side.
use super::*;

#[test]
fn actual_retail_cabinets_have_closed_source_painted_backs_and_exposed_sides() {
    let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external verified pack")).unwrap();
    let pack=loaded.pack();let art=&pack.runtime_files()["data/tilesets/mart_metatiles.bin"];
    let mut counts=std::collections::BTreeMap::new();let mut vented=0;let mut unvented=0;
    for (map,data) in &pack.data().maps {
        if !reviewed_retail_room(map)||data.attributes.tileset_name!="mart" {continue;}
        let(w,h)=(data.attributes.width as usize*4,data.attributes.height as usize*4);
        let tiles=(0..w*h).map(|i|{let(x,y)=(i%w,i/w);let block=data.blocks[y/4*(w/4)+x/4];
            VisualTile{column:x as u32,row:y as u32,texture:Default::default(),priority:false,animation_frames:None,
                source:VisualTileSource{tileset_id:"mart".into(),metatile_id:block,subtile_column:(x%4)as u8,
                    subtile_row:(y%4)as u8,tile_index:art[block as usize*16+y%4*4+x%4]as u16}}
        }).collect::<Vec<_>>();let refs=tiles.iter().collect::<Vec<_>>();
        // Non-square world pixels and a translated origin also exercise the
        // production fitting, rather than checking one synthetic unit box.
        let g=GridGeometry{width:w,height:h,tile_width:12.,tile_height:6.,origin_x:-31.,origin_z:17.};
        for p in resolve(map,&refs,&g,None) {
            let source=&refs[p.row*w+p.column].source;
            if p.width!=2||p.height!=4||!matches!(p.kind,ModelKind::RetailShelf|ModelKind::RetailRefrigerator)
                ||!matches!((source.metatile_id,source.tile_index),(0x13,_)|(0x15|0x27|0x14,0x0c)) {continue;}
            *counts.entry(map.clone()).or_insert(0usize)+=1;
            if source.tile_index==0x0c {vented+=1;}else {unvented+=1;}
            let mut mesh=TerrainMeshData{footing_heights:vec![2.5;w*h],..Default::default()};let mut claims=vec![false;w*h];
            assert!(append(&mut mesh,map,&refs,&g,&p,&mut claims,[0,0]));
            assert_eq!(claims.iter().filter(|&&c|c).count(),8);
            assert_eq!(mesh.footing_heights,vec![2.5;w*h]);assert!(mesh.solid.indices.is_empty());
            assert_eq!(mesh.textured.cutaway_ranges.len(),1,"{map}: one whole cabinet reveal");
            assert_eq!(mesh.depth_coverage_textured_ranges,mesh.textured.cutaway_ranges);
            let reveal=mesh.textured.cutaway_ranges[0].clone();
            assert!(mesh.textured.indices.chunks_exact(3).all(|tri| {
                let above_floor=tri.iter().any(|&i|mesh.textured.positions[i as usize][1]>0.);
                !above_floor || tri.iter().all(|&i|reveal.contains(&(i as usize)))
            }),"{map}: every cabinet face shares its owner");
            assert!(mesh.textured.indices.chunks_exact(3).filter(|tri|tri.iter().all(|&i| {
                mesh.textured.positions[i as usize][1]==0. && mesh.textured.normals[i as usize][1]>0.99
            })).all(|tri|tri.iter().all(|&i|!reveal.contains(&(i as usize)))),"{map}: floor stays opaque");
            let left=g.origin_x+p.column as f32*g.tile_width;let right=left+p.width as f32*g.tile_width;
            let south=g.origin_z+(p.row+p.height)as f32*g.tile_height;let north=south-p.depth_pixels*g.tile_height/8.;
            let top=p.height_pixels*0.75*g.tile_height/8.;
            let mut area=[0.;4];let mut grounded=[false;4];let mut topped=[false;4];
            let uv=g.uv(p.column,p.row);let casing=[uv.0+(uv.1-uv.0)*1.5/8.,uv.2+(uv.3-uv.2)*1.5/8.];
            for tri in mesh.textured.indices.chunks_exact(3) {
                let ids=[tri[0],tri[1],tri[2]].map(|i|i as usize);
                let points=ids.map(|i|Vec3::from_array(mesh.textured.positions[i]));
                let normal=Vec3::from_array(mesh.textured.normals[ids[0]]);let cross=(points[1]-points[0]).cross(points[2]-points[0]);
                assert!(normal.is_finite()&&cross.dot(normal)>0.,"{map}: outward finite cabinet/floor faces");
                let side=if normal.z < -0.999 && points.iter().all(|p|(p.z-north).abs()<0.001) {Some((0,-cross.z))}
                    else if normal.x < -0.999 && points.iter().all(|p|(p.x-left).abs()<0.001) {Some((1,-cross.x))}
                    else if normal.x > 0.999 && points.iter().all(|p|(p.x-right).abs()<0.001) {Some((2,cross.x))}
                    else if normal.z>0.5 && points.iter().all(|p|p.z>south-1.) {Some((3,cross.z))}else {None};
                if let Some((side,projected))=side {
                    area[side]+=projected/2.;
                    grounded[side]|=points.iter().any(|p|p.y.abs()<0.001);
                    topped[side]|=points.iter().any(|p|(p.y-top).abs()<0.001);
                    if side<3 {for i in ids {assert_eq!(mesh.textured.uvs[i],casing,"{map}: casing uses its own source paint");}}
                }
                assert!(points.iter().all(|p|p.y<=top+0.001),"{map}: no extra header height");
            }
            let joined=if native_store_aisle_shelf_source(map,&p,&refs,&g) {native_store_shelf_neighbors(map,&p,&refs,&g)}else{[false;2]};
            let expected=[(right-left)*top,if joined[0]{0.}else{(south-north)*top},
                if joined[1]{0.}else{(south-north)*top},(right-left)*top];
            for side in 0..4 {assert!((area[side]-expected[side]).abs()<0.02,"{map}: complete projected area on side {side}: {} vs {}",area[side],expected[side]);
                if expected[side]>0. {assert!(grounded[side]&&topped[side],"{map}: shell meets floor and cap on side {side}");}
            }
        }
    }
    println!("actual cabinet counts: {counts:?}; vented={vented}, unvented={unvented}");
    assert_eq!(vented,115);assert_eq!(unvented,184);assert_eq!(counts.len(),19);
}
