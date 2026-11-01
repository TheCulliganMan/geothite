use super::*;

#[test]
fn pokecom_floor_pattern_keeps_its_uvs_datum_and_custom_ownership() {
    let map = crate::pokecom::ADMIN_OFFICE;
    let g = GridGeometry { width:2, height:2, tile_width:8., tile_height:8., origin_x:0., origin_z:0. };
    let tiles = (0..4).map(|i| VisualTile { column:(i%2) as u32, row:(i/2) as u32,
        texture:Default::default(), animation_frames:None, priority:false,
        source:VisualTileSource { tileset_id:"pokecom_center".into(), metatile_id:1,
            subtile_column:(i%2) as u8, subtile_row:(i/2) as u8, tile_index:0x11 },
    }).collect::<Vec<_>>();
    let refs = tiles.iter().collect::<Vec<_>>();
    let mut original = TerrainMeshData { footing_heights:vec![3.25;4], authored_cells:vec![None;4], ..Default::default() };
    for tile in &tiles { append_top(&mut original.textured, g.bounds(tile.column as usize,tile.row as usize).into(),
        0.,g.uv(tile.column as usize,tile.row as usize)); }
    let mut mesh = original.clone();
    assert_eq!(finish_surfaces(&mut mesh,map,&refs,&g,[0,0]),4);
    assert_eq!(mesh.textured.positions,original.textured.positions,"no slabs, seams or enlarged pattern");
    assert_eq!(mesh.textured.uvs,original.textured.uvs);
    assert_eq!(mesh.textured.indices,original.textured.indices);
    assert!(mesh.solid.indices.is_empty());
    assert_eq!(mesh.footing_heights,original.footing_heights);
    assert_eq!(mesh.authored_cells,original.authored_cells);
    let custom = Document { objects:vec![Object {
        name:"User Pokecom floor".into(), map:Some(map.into()), maps:None,
        tileset:"pokecom_center".into(), metatile:1, metatiles:None, origin:[0,0],
        tiles:vec![vec![0x11;2];2], ground:0x11, top_pixels:0, depth_pixels:0.,
        footing_pixels:None, mask:Default::default(), parts:vec![],
    }], atmosphere:None };
    let mut owned = original.clone();
    assert_eq!(finish_surfaces_with_profiles(&mut owned,map,&refs,&g,[0,0],Some(&custom)),0);
    assert_eq!(owned,original);
}

#[test]
fn external_pokecom_floor_phases_cover_the_pattern_and_reject_black_void_equipment() {
    let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK").map(std::path::PathBuf::from)
        .expect("set CRYSTAL_RENDER_TEST_PACK to the completed external pack");
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
    let pack = loaded.pack();
    let map = crate::pokecom::ADMIN_OFFICE;
    let module = &pack.data().maps[map];
    let layout = &pack.runtime_files()["data/tilesets/pokecom_center_metatiles.bin"];
    let mut floor_count = 0;
    let mut void_count = 0;
    for &block in &module.blocks {
        for phase in 0..16 {
            let tile = u16::from(layout[block as usize*16+phase]);
            let mut s = VisualTileSource { tileset_id:"pokecom_center".into(), metatile_id:block,
                subtile_column:(phase%4) as u8, subtile_row:(phase/4) as u8, tile_index:tile };
            let is_floor = matches!(tile,1|0x11);
            assert_eq!(crate::pokecom::floor_source(map,&s),is_floor,"block{block:x} phase{phase}");
            if is_floor {
                floor_count += 1;
                assert_eq!(interior_floor(map,&s),Some(InteriorFloor::NativeRoom));
                assert_ne!(interior_floor("UnreviewedPokecomRoom",&s),Some(InteriorFloor::NativeRoom));
                s.tile_index ^= 0x80;
                assert!(!crate::pokecom::floor_source(map,&s));
            } else if tile == 0x3d {
                void_count += 1;
                assert_eq!(interior_floor(map,&s),None,"black void cannot become stone paving");
            }
        }
    }
    assert!(floor_count > 800 && void_count > 2000);
}
