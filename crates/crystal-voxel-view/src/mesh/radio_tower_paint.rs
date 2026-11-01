//! Authenticated live pigment for the shared broadcast tower.
use super::*;

const BLOCKS: [(u16,[u16;16]);6] = [
    (0x25,[80,33,29,83,80,38,38,83,80,33,29,33,80,38,38,38]),
    (0x26,[46,86,66,67,29,48,51,83,29,33,29,83,38,38,38,83]),
    (0x29,[80,33,29,33,80,38,38,38,80,33,29,33,80,38,38,38]),
    (0x2a,[29,33,29,83,38,38,38,83,29,33,29,83,38,38,38,83]),
    (0x2d,[80,33,29,33,80,83,53,53,80,83,53,53,32,37,6,6]),
    (0x2e,[29,33,29,83,80,33,29,83,80,33,29,83,32,33,29,37]),
];
fn ink(tile:u16)->Option<[&'static str;8]> {
    match tile {
        0x50=>Some(["31233131","31232331","31233331","31233331",
            "31233231","31323331","31302331","31230231"]),
        0x26=>Some(["33333333","31111113","31001003","31001003",
            "31001003","31001003","31001003","33333333"]),
        _=>None,
    }
}
pub(super) fn palette(cells:&[&VisualTile],g:&GridGeometry,p:BuildingPlacement,
    images:&TerrainImageSamples)->Option<[[f32;4];6]> {
    if p.width!=8 || p.height!=12 || p.column+8>g.width || p.row+12>g.height {return None;}
    let mut wall=[None;4];let mut glass=[None;4];
    for y in 0..12 {for x in 0..8 {
        let tile=cells.get((p.row+y)*g.width+p.column+x)?;let s=&tile.source;
        let (block,drawing)=BLOCKS[y/4*2+x/4];
        if s.tileset_id.as_ref()!="johto_modern" || s.metatile_id!=block
            || s.subtile_column as usize!=x%4 || s.subtile_row as usize!=y%4
            || s.tile_index!=drawing[y%4*4+x%4] {return None;}
        let Some(pattern)=ink(s.tile_index) else {continue;};
        let rgba=tile_rgba(images,tile).ok()?;
        if rgba.len()!=256 {return None;}
        let slots=if s.tile_index==0x50 {&mut wall}else{&mut glass};
        for (i,pixel) in rgba.chunks_exact(4).enumerate() {
            let color:[u8;4]=pixel.try_into().ok()?;
            let index=(pattern[i/8].as_bytes()[i%8]-b'0')as usize;
            if color[3]!=255 || slots[index].is_some_and(|c|c!=color) {return None;}
            slots[index]=Some(color);
        }
    }}
    let [Some(w0),Some(w1),Some(w2),Some(w3)]=wall else {return None;};
    let [Some(g0),Some(g1),None,Some(g3)]=glass else {return None;};
    let wall=[w0,w1,w2,w3];
    if (0..4).any(|i|(i+1..4).any(|j|wall[i]==wall[j])) || g0==g1 || g0==g3 || g1==g3 {return None;}
    let linear=|c:[u8;4]|[0,1,2,3].map(|i|if i==3 {1.} else {
        let s=f32::from(c[i])/255.;if s<=0.04045 {s/12.92} else {((s+0.055)/1.055).powf(2.4)}
    });
    let mix=|a:[f32;4],b:[f32;4],t:f32|std::array::from_fn(|i|a[i]*(1.-t)+b[i]*t);
    let dark=mix(linear(w3),linear(w2),0.20);
    let pane=mix(linear(g0),linear(g1),0.45);
    Some([dark,linear(w2),linear(w1),pane,linear(g0),mix(pane,linear(g3),0.35)])
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture()->(Vec<VisualTile>,GridGeometry,BuildingPlacement,TerrainImageSamples) {
        let g=GridGeometry {width:8,height:12,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut images=TerrainImageSamples::default();let mut assets=Assets::<Image>::default();
        let mut cells=Vec::new();
        for y in 0..12 {for x in 0..8 {
            let (block,drawing)=BLOCKS[y/4*2+x/4];let tile=drawing[y%4*4+x%4];
            let texture=assets.add(Image::default());
            if let Some(pattern)=ink(tile) {
                let colors=if tile==0x50 {[[222,255,222,255],[156,173,156,255],[99,115,99,255],[49,49,49,255]]}
                    else {[[255,255,165,255],[222,255,222,255],[99,115,99,255],[49,49,49,255]]};
                let rgba=pattern.iter().flat_map(|r|r.bytes().flat_map(|b|colors[(b-b'0')as usize])).collect();
                images.pixels.insert(texture.id(),TileImageSample::Rgba(rgba));
            }
            cells.push(VisualTile {column:x as u32,row:y as u32,texture,priority:false,animation_frames:None,
                source:VisualTileSource {tileset_id:"johto_modern".into(),metatile_id:block,
                    subtile_column:(x%4)as u8,subtile_row:(y%4)as u8,tile_index:tile}});
        }}
        let p=BuildingPlacement {column:0,row:0,width:8,height:12,roof_rows:4,ground_tile_index:6};
        (cells,g,p,images)
    }
    #[test]
    fn source_broadcast_facade_has_four_upper_window_rows() {
        let (cells,g,_,_)=fixture();
        let rows=(0..8).map(|y|(0..8).filter(|x|
            cells[y*g.width+x].source.tile_index==0x26).count()).collect::<Vec<_>>();
        assert_eq!(rows,vec![0,2,0,6,0,6,0,6],
            "three complete native rows and a partial upper row above the entrance storey");
    }

    #[test]
    fn native_broadcast_palette_requires_complete_drawing_and_exact_opaque_ink() {
        let (cells,g,p,images)=fixture();let refs=cells.iter().collect::<Vec<_>>();
        let day=palette(&refs,&g,p,&images).unwrap();assert!(day[0][0]<0.1);
        let mut night=images.clone();for sample in night.pixels.values_mut() {
            if let TileImageSample::Rgba(p)=sample {for c in p.chunks_exact_mut(4) {c[0]/=2;c[1]/=2;}}
        }
        assert_ne!(palette(&refs,&g,p,&night).unwrap(),day);
        for fault in 0..4 {
            let mut altered=cells.clone();let mut samples=images.clone();
            match fault {
                0=>altered[44].source.tile_index^=1,
                1=>for c in &mut altered {c.source.tileset_id="kanto".into();},
                _=>if let TileImageSample::Rgba(p)=samples.pixels.get_mut(&cells[0].texture.id()).unwrap(){p[if fault==2 {0}else{3}]^=1;},
            }
            assert!(palette(&altered.iter().collect::<Vec<_>>(),&g,p,&samples).is_none());
        }
    }
    #[test]
    fn broadcast_paint_preserves_shared_geometry_and_antenna_light_roles() {
        let (cells,g,p,images)=fixture();let paint=palette(&cells.iter().collect::<Vec<_>>(),&g,p,&images).unwrap();
        let m=model(Kind::RadioTower);let mut mesh=SurfaceMeshData::default();
        m.append_fitted(&mut mesh,[0.,64.,0.,96.],0.,16.,Some(24.));let old=mesh.clone();
        m.recolor_fitted_radio_tower(&mut mesh,0,paint);
        assert_eq!(mesh.positions,old.positions);assert_eq!(mesh.normals,old.normals);assert_eq!(mesh.indices,old.indices);
        let mut changed=0;let mut lamp=0;
        for i in 0..mesh.positions.len() {
            let material=m.vertex_material(i);
            if mesh.colors[i]!=old.colors[i] {
                changed+=1;
                if matches!(material,"plaster_light"|"plaster"|"stone"|"stone_dark") {
                    assert_eq!(crate::scenery_finish::town_paint_kind(mesh.uvs[i]),Some(3));
                }else{assert_eq!(mesh.uvs[i],crate::scenery_finish::SOURCE_PAINT_UV);}
            }
            if material=="coral" || material=="yellow" || material.starts_with("stone") && m.surface.positions[i][1]>7.22 {
                assert_eq!(mesh.colors[i],old.colors[i]);lamp+=usize::from(material=="coral");
            }
        }
        assert!(changed>1000 && lamp>0);
        crate::city_lights::mark(&mut mesh,0,m,Kind::RadioTower);
        assert!(mesh.uvs.iter().any(|uv|crate::city_lights::source_window_tag(*uv)));
        assert!(mesh.uvs.iter().any(|uv|uv[1]==crate::city_lights::UV_BASE+3.));
    }
}
