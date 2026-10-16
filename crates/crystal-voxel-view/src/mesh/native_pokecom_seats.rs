//! Compact upholstered lobby seats, independently owned in each source half.
use super::*;

pub(super) fn matches(map: &str, p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    p.kind == ModelKind::Chair && p.width == 2 && p.height == 2
        && p.column + 2 <= g.width && p.row + 2 <= g.height
        && cells.len() == g.width * g.height
        && p.indices(g.width).enumerate().all(|(j, i)|
            crate::pokecom::chair_local(map, &cells[i].source) == Some(((j % 2) as u8, (j / 2) as u8)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_pokecom_seats_keep_twelve_compact_source_cushions_and_independent_claims() {
        let path=std::env::var_os("CRYSTAL_RENDER_TEST_PACK").map(std::path::PathBuf::from)
            .expect("set CRYSTAL_RENDER_TEST_PACK to the completed external pack");
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let pack=loaded.pack();let map=crate::pokecom::ADMIN_OFFICE;
        let module=&pack.data().maps[map];
        let layout=&pack.runtime_files()["data/tilesets/pokecom_center_metatiles.bin"];
        let (w,h)=(module.attributes.width as usize*4,module.attributes.height as usize*4);
        let tiles=(0..w*h).map(|i| {
            let (x,y)=(i%w,i/w);let b=module.blocks[y/4*(w/4)+x/4];
            VisualTile {column:x as u32,row:y as u32,texture:Default::default(),
                animation_frames:None,priority:false,source:VisualTileSource {
                    tileset_id:"pokecom_center".into(),metatile_id:b,subtile_column:(x%4)as u8,
                    subtile_row:(y%4)as u8,tile_index:u16::from(layout[b as usize*16+y%4*4+x%4]),
                }}
        }).collect::<Vec<_>>();
        let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let refs=tiles.iter().collect::<Vec<_>>();
        let placements=resolve(map,&refs,&g,None);
        let seats=placements.iter().filter(|p|matches(map,p,&refs,&g)).collect::<Vec<_>>();
        assert_eq!(seats.len(),12);
        let png=image::load_from_memory(&pack.runtime_files()["gfx/tilesets/pokecom_center.png"])
            .unwrap().to_luma8();
        for p in seats {
            assert_eq!((p.depth_pixels,p.height_pixels),(8.,7.));
            assert_eq!(p.seat_height_pixels(map,&tiles[p.row*w+p.column].source),Some(7.));
            let mut mesh=TerrainMeshData{footing_heights:vec![3.25;w*h],authored_cells:vec![None;w*h],..Default::default()};
            let mut claimed=vec![false;w*h];
            assert!(append(&mut mesh,map,&refs,&g,p,&mut claimed,[0,0]));
            assert_eq!(claimed.iter().filter(|&&v|v).count(),4,"each cushion owns only its four source cells");
            assert!(p.indices(w).all(|i|claimed[i]));
            assert_eq!(mesh.footing_heights,vec![3.25;w*h]);
            assert!(mesh.solid.indices.is_empty(),"seat has no generic beige paint");
            assert_eq!(mesh.textured.positions.iter().map(|v|v[1]).fold(0.,f32::max),7.);
            assert!(mesh.textured.positions.iter().all(|v|v[1]>=0.));
            let pigments=mesh.textured.uvs.iter().map(|uv| {
                let fx=uv[0]*w as f32;let fy=uv[1]*h as f32;
                let tile=&tiles[(fy.floor()as usize).min(h-1)*w+(fx.floor()as usize).min(w-1)].source;
                png.get_pixel((tile.tile_index as u32%16)*8+((fx.fract()*8.)as u32).min(7),
                    (tile.tile_index as u32/16)*8+((fy.fract()*8.)as u32).min(7))[0]
            }).collect::<std::collections::BTreeSet<_>>();
            assert!(pigments.contains(&85)&&pigments.contains(&170)&&pigments.contains(&255),
                "live leg, orange body and light cushion pigments: {pigments:?}");
            for index in p.indices(w) {
                let mut changed=tiles.clone();changed[index].source.tile_index^=0x80;
                let changed_refs=changed.iter().collect::<Vec<_>>();
                assert!(!matches(map,p,&changed_refs,&g));
                assert!(!resolve(map,&changed_refs,&g,None).iter().any(|q|
                    q.kind==ModelKind::Chair&&q.column==p.column&&q.row==p.row));
                let mut reserved=vec![false;w*h];reserved[index]=true;
                let mut rejected=TerrainMeshData::default();
                assert!(!append(&mut rejected,map,&refs,&g,p,&mut reserved,[0,0]));
                assert!(rejected.textured.indices.is_empty());
            }
            assert!(!matches("GoldenrodPokecenter1F",p,&refs,&g));
        }
    }
}
