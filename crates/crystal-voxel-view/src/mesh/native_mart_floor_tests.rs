//! Actual-pack floor coverage and texture phase through the production finisher.
use super::*;

#[test]
fn all_thirteen_actual_mart_floors_keep_every_source_pixel_on_the_flat_support_datum() {
    let path=std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external verified pack");
    let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(path).unwrap();
    let pack=loaded.pack();let art=&pack.runtime_files()["data/tilesets/mart_metatiles.bin"];
    let sheet=image::load_from_memory(&pack.runtime_files()["gfx/tilesets/mart.png"]).unwrap().to_rgba8();
    let tile=crate::mart::STANDARD_MART_FLOOR_TILE as u32;
    let x=tile%(sheet.width()/8)*8;let y=tile/(sheet.width()/8)*8;
    let shades=(0..64).map(|i|sheet.get_pixel(x+i%8,y+i/8).0).collect::<std::collections::BTreeSet<_>>();
    assert!(shades.len()>1,"the packed floor has a real fine pattern to preserve");
    for map in ["AzaleaMart","BlackthornMart","CherrygroveMart","EcruteakMart","OlivineMart","VioletMart",
        "CeruleanMart","FuchsiaMart","LavenderMart","PewterMart","SaffronMart","VermilionMart","ViridianMart"] {
        let data=&pack.data().maps[map];let (w,h)=(data.attributes.width as usize*4,data.attributes.height as usize*4);
        let sources=(0..w*h).map(|i|{let (x,y)=(i%w,i/w);let block=data.blocks[y/4*(w/4)+x/4];
            VisualTileSource{tileset_id:"mart".into(),metatile_id:block,subtile_column:(x%4)as u8,
                subtile_row:(y%4)as u8,tile_index:art[block as usize*16+y%4*4+x%4]as u16}
        }).collect::<Vec<_>>();
        let mut frame=crate::mesh::tests::frame(w as u32,h as u32,sources.clone());
        frame.map_id=map.into();frame.source_map_size_core_tiles=Some(bevy::prelude::UVec2::new(w as u32/2,h as u32/2));
        let mut images=Assets::<Image>::default();
        // Geometry/UV verification uses opaque art. The actual motif is read
        // above and production paired captures verify its palette and sampling.
        let texture=images.add(Image::new_fill(
            bevy::render::render_resource::Extent3d{width:8,height:8,depth_or_array_layers:1},
            bevy::render::render_resource::TextureDimension::D2,&[180,180,180,255],
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            bevy::asset::RenderAssetUsages::all()));
        for tile in &mut frame.tiles {tile.texture=texture.clone();}
        let built=crate::mesh::audit_terrain_mesh_with_images(&frame,&images).unwrap();
        let mut area=vec![0.;w*h];
        for tri in built.textured.indices.chunks_exact(3) {
            let points=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(built.textured.positions[i as usize]));
            if points.iter().any(|p|p.y.abs()>0.00001)||built.textured.normals[tri[0]as usize][1]<0.99 {continue;}
            let center=(points[0]+points[1]+points[2])/3.;
            let col=((center.x+w as f32*4.)/8.).floor()as usize;
            let row=((center.z+h as f32*4.)/8.).floor()as usize;
            if col>=w||row>=h||sources[row*w+col].tile_index!=crate::mart::STANDARD_MART_FLOOR_TILE {continue;}
            for (&i,p) in tri.iter().zip(points) {
                let expected=[(p.x+w as f32*4.)/(w as f32*8.),(p.z+h as f32*4.)/(h as f32*8.)];
                let uv=built.textured.uvs[i as usize];
                assert!((uv[0]-expected[0]).abs()<0.000001&&(uv[1]-expected[1]).abs()<0.000001,
                    "{map}: floor at {col},{row} must retain all source texels at native phase, got {uv:?} expected {expected:?}");
            }
            area[row*w+col]+=(points[1]-points[0]).cross(points[2]-points[0]).length()*0.5;
        }
        let mut count=0;
        for (i,s) in sources.iter().enumerate() {
            if s.tile_index!=crate::mart::STANDARD_MART_FLOOR_TILE {continue;}
            count+=1;
            assert!((area[i]-64.).abs()<0.0001,"{map}: source floor cell {i} must be covered exactly once, area={}",area[i]);
            assert_eq!(built.footing_heights[i],0.,"{map}: pattern never raises walking support");
            assert!(built.authored_cells[i].is_none(),"{map}: surface finishing does not claim a furnishing");
        }
        assert!(count>100,"{map}: inspect the whole actual floor, not a small synthetic patch");
    }
}
