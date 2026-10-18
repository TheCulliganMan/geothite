//! Roofs retain their real deck; border-fill cells reveal the outdoor sky.
use super::*;

pub(super) fn void_cells(frame:&VisualWorldFrame,g:&GridGeometry,cells:&[&VisualTile],
    images:Option<&TerrainImageSamples>)->Vec<usize> {
    let Some(first)=cells.first() else {return Vec::new();};
    let atlas=first.source.tileset_id.as_ref();
    let Some(size)=frame.source_map_size_core_tiles else {return Vec::new();};
    if !crate::interior::open_air_roof(&frame.map_id,atlas) || g.width==0
        || g.height==0 || cells.len()!=g.width*g.height || size.x==0 || size.y==0 {
        return Vec::new();
    }
    let mut selected=vec![false;cells.len()];
    for (i,tile) in cells.iter().enumerate() {
        let x=(i%g.width) as i64+i64::from(frame.grid_origin.x);
        let z=(i/g.width) as i64+i64::from(frame.grid_origin.y);
        // Only the host's filler outside this finite rooftop is suppressed.
        // Foreign atlas cells remain untouched rather than discarding neighbors.
        if tile.source.tileset_id.as_ref()==atlas
            && (x<0 || z<0 || x>=i64::from(size.x)*2 || z>=i64::from(size.y)*2) {
            selected[i]=true;
        }
    }
    if frame.map_id.as_ref()=="TinTowerRoof" && let Some(images)=images {
        // Tin Tower also has complete blank sky blocks inside its map bounds.
        // Require all phases and one opaque uniform live pigment; a changed
        // drawing or real deck must never disappear on the strength of its color.
        for y in 0..g.height.saturating_sub(3) {for x in 0..g.width.saturating_sub(3) {
            let indices=std::array::from_fn::<_,16,_>(|j|(y+j/4)*g.width+x+j%4);
            let mut paint=None;
            let blank=indices.iter().enumerate().all(|(j,&i)| {
                let tile=cells[i];let s=&tile.source;
                if s.tileset_id.as_ref()!="tower" || s.metatile_id!=1 || s.tile_index!=1
                    || s.subtile_column as usize!=j%4 || s.subtile_row as usize!=j/4 {return false;}
                let Ok(rgba)=tile_rgba(images,tile) else {return false;};
                if rgba.len()!=256 {return false;}
                let color:[u8;4]=rgba[..4].try_into().unwrap();
                if color[3]!=255 || rgba.chunks_exact(4).any(|p|p!=color)
                    || paint.is_some_and(|old|old!=color) {return false;}
                paint=Some(color);true
            });
            if blank {for i in indices {selected[i]=true;}}
        }}
    }
    selected.into_iter().enumerate().filter_map(|(i,sky)|sky.then_some(i)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sky_border_respects_real_roof_bounds_and_keeps_rooms_and_foreign_sources() {
        let mut source=super::super::tests::source_with_tile(7,0,0,5);
        source.tileset_id="mansion".into();
        let mut frame=super::super::tests::frame(8,8,vec![source;64]);
        frame.map_id="CeladonMansionRoof".into();
        frame.source_map_size_core_tiles=Some(bevy::math::UVec2::new(2,2));
        frame.grid_origin=bevy::math::IVec2::new(-2,-2);
        let g=GridGeometry {width:8,height:8,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let selected=void_cells(&frame,&g,&frame.tiles.iter().collect::<Vec<_>>(),None);
        assert_eq!(selected.len(),48);
        assert!(selected.iter().all(|i|!((2..6).contains(&(i%8))&&(2..6).contains(&(i/8)))));
        frame.tiles[0].source.tileset_id="mart".into();
        assert!(void_cells(&frame,&g,&frame.tiles.iter().collect::<Vec<_>>(),None).is_empty(),
            "an incompatible host atlas cannot classify a roof");
        frame.tiles[0].source.tileset_id="mansion".into();
        frame.tiles[1].source.tileset_id="johto".into();
        assert!(!void_cells(&frame,&g,&frame.tiles.iter().collect::<Vec<_>>(),None).contains(&1));
        frame.map_id="CeladonMansion1F".into();
        assert!(void_cells(&frame,&g,&frame.tiles.iter().collect::<Vec<_>>(),None).is_empty());
        frame.map_id="CeladonMansionRoof".into();frame.source_map_size_core_tiles=None;
        assert!(void_cells(&frame,&g,&frame.tiles.iter().collect::<Vec<_>>(),None).is_empty());
    }

    #[test]
    fn tower_sky_requires_complete_uniform_source_and_never_consumes_deck_art() {
        let sources=(0..16).map(|j| {
            let mut s=super::super::tests::source_with_tile(1,(j%4)as u8,(j/4)as u8,1);
            s.tileset_id="tower".into();s
        }).collect();
        let mut frame=super::super::tests::frame(4,4,sources);
        frame.map_id="TinTowerRoof".into();
        frame.source_map_size_core_tiles=Some(bevy::math::UVec2::new(2,2));
        let g=GridGeometry{width:4,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut images=TerrainImageSamples::default();
        for tile in &frame.tiles {
            images.pixels.insert(tile.texture.id(),TileImageSample::Rgba([65,65,255,255].repeat(64)));
        }
        assert_eq!(void_cells(&frame,&g,&frame.tiles.iter().collect::<Vec<_>>(),Some(&images)).len(),16);
        for fault in 0..4 {
            let mut changed=frame.clone();let mut art=images.clone();
            match fault {
                0=>changed.tiles[0].source.tile_index=2,
                1=>changed.tiles[0].source.subtile_column=1,
                2=>if let TileImageSample::Rgba(p)=art.pixels.get_mut(&frame.tiles[0].texture.id()).unwrap(){p[0]^=1;},
                _=>if let TileImageSample::Rgba(p)=art.pixels.get_mut(&frame.tiles[0].texture.id()).unwrap(){p[3]=0;},
            }
            assert!(void_cells(&changed,&g,&changed.tiles.iter().collect::<Vec<_>>(),Some(&art)).is_empty());
        }
        assert!(void_cells(&frame,&g,&frame.tiles.iter().collect::<Vec<_>>(),None).is_empty());
        frame.map_id="TinTower1F".into();
        assert!(void_cells(&frame,&g,&frame.tiles.iter().collect::<Vec<_>>(),Some(&images)).is_empty());
    }
}
