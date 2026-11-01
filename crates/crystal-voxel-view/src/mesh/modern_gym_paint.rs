//! Live paint authenticated by the complete shared modern Gym drawing.
use super::*;
const BLOCKS:[(u16,[u16;16]);6]=[
 (0x18,[16,17,17,17,13,14,14,14,13,14,14,14,10,11,11,11]),
 (0x1f,[17,17,17,17,14,14,14,14,14,14,14,14,11,11,11,11]),
 (0x19,[17,17,17,18,14,14,14,15,14,14,14,15,11,11,11,12]),
 (0x1c,[26,27,27,27,26,38,38,27,26,27,27,27,1,2,2,2]),
 (0x1d,[35,36,27,27,27,27,27,27,55,56,27,27,57,58,2,2]),
 (0x1e,[27,27,27,28,27,38,38,28,27,27,27,28,2,2,2,22]),
];
const PANE:[&str;8]=["33333333","31111113","31001003","31001003",
    "31001003","31001003","31001003","33333333"];
fn drawing(x:usize,y:usize)->(u16,u16) {
    let row=y/4;
    let (block,art)=BLOCKS[row*3+x/4];(block,art[y%4*4+x%4])
}
pub(super) fn palette(cells:&[&VisualTile],g:&GridGeometry,p:BuildingPlacement,
    images:&TerrainImageSamples)->Option<[[f32;4];8]> {
    if p.width!=12 || p.height!=8 || p.roof_rows!=4
        || p.column+12>g.width || p.row+8>g.height {return None;}
    let mut glass=[None;4];let mut wall=std::collections::BTreeSet::new();
    for y in 0..8 {for x in 0..12 {
        let tile=*cells.get((p.row+y)*g.width+p.column+x)?;let s=&tile.source;
        let (block,index)=drawing(x,y);
        if s.tileset_id.as_ref()!="johto_modern" || s.metatile_id!=block || s.tile_index!=index
            || s.subtile_column as usize!=x%4 || s.subtile_row as usize!=y%4 {return None;}
        if index==0x1b {
            let pixels=tile_rgba(images,tile).ok()?;
            if pixels.len()!=256 {return None;}
            for pixel in pixels.chunks_exact(4) {
                if pixel[3]!=255 {return None;}
                wall.insert(<[u8;4]>::try_from(pixel).ok()?);
            }
        }
        if index!=0x26 {continue;}
        let pixels=tile_rgba(images,tile).ok()?;
        if pixels.len()!=256 {return None;}
        for (i,pixel) in pixels.chunks_exact(4).enumerate() {
            let color:[u8;4]=pixel.try_into().ok()?;
            let slot=&mut glass[(PANE[i/8].as_bytes()[i%8]-b'0')as usize];
            if color[3]!=255 || slot.is_some_and(|old|old!=color) {return None;}
            *slot=Some(color);
        }
    }}
    let [Some(pane),Some(trim),None,Some(ink)]=glass else {return None;};
    if pane==trim || pane==ink || trim==ink {return None;}
    let linear=|c:[u8;4]|std::array::from_fn(|i|if i==3 {1.}else {
        let s=c[i]as f32/255.;if s<=0.04045 {s/12.92}else{((s+0.055)/1.055).powf(2.4)}
    });
    // The Gym windows share the first two facade courses with its brickwork.
    // Sample only proven brick cells; yellow glass and dark name ink are not
    // masonry swatches even when they occupy that same horizontal band.
    if !(2..=4).contains(&wall.len()) {return None;}
    let mut wall=wall.into_iter().collect::<Vec<_>>();
    wall.sort_by_key(|c|u32::from(c[0])*2126+u32::from(c[1])*7152+u32::from(c[2])*722);
    let wall=[linear(wall[wall.len()-2]),linear(wall[wall.len()-1])];
    let roof=super::cabin_roof_palette(cells,g,p,images)?;
    let mix=|a:[f32;4],b:[f32;4],t:f32|std::array::from_fn(|i|a[i]*(1.-t)+b[i]*t);
    Some([wall[0],wall[1],mix(linear(pane),linear(trim),0.45),linear(pane),
        linear(ink),roof[0],roof[1],roof[2]])
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture()->(Vec<VisualTile>,GridGeometry,BuildingPlacement,TerrainImageSamples) {
        let g=GridGeometry {width:12,height:8,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let p=BuildingPlacement {column:0,row:0,width:12,height:8,roof_rows:4,ground_tile_index:6};
        let mut assets=Assets::<Image>::default();let mut images=TerrainImageSamples::default();
        let mut cells=Vec::new();
        for y in 0..8 {for x in 0..12 {
            let (block,index)=drawing(x,y);let texture=assets.add(Image::default());
            let colors=if index==0x26 {
                [[255,255,165,255],[222,255,222,255],[74,185,235,255],[49,49,49,255]]
            } else {[[222,255,222,255],[198,148,57,255],[165,123,24,255],[49,49,49,255]]};
            let pixels=(0..64).flat_map(|i|colors[if index==0x26 {
                (PANE[i/8].as_bytes()[i%8]-b'0')as usize
            }else if index==0x1b {if i%8<2 {1}else{2}}else {i%4}]).collect();
            images.pixels.insert(texture.id(),TileImageSample::Rgba(pixels));
            cells.push(VisualTile {column:x as u32,row:y as u32,texture,priority:false,animation_frames:None,
                source:VisualTileSource {tileset_id:"johto_modern".into(),metatile_id:block,tile_index:index,
                    subtile_column:(x%4)as u8,subtile_row:(y%4)as u8}});
        }}
        (cells,g,p,images)
    }
    #[test]
    fn complete_modern_gym_paint_tracks_live_art_and_rejects_altered_sources() {
        let (cells,g,p,images)=fixture();let refs=cells.iter().collect::<Vec<_>>();
        let day=palette(&refs,&g,p,&images).unwrap();
        let mut night=images.clone();for sample in night.pixels.values_mut() {
            if let TileImageSample::Rgba(p)=sample {for pixel in p.chunks_exact_mut(4) {pixel[0]/=2;pixel[1]/=2;}}
        }
        assert_ne!(palette(&refs,&g,p,&night).unwrap(),day);
        for i in 0..cells.len() {
            let mut altered=cells.clone();altered[i].source.tile_index^=1;
            assert!(palette(&altered.iter().collect::<Vec<_>>(),&g,p,&images).is_none());
        }
        for fault in 0..3 {
            let mut altered=cells.clone();let mut samples=images.clone();
            match fault {
                0=>altered[0].source.tileset_id="kanto".into(),
                1=>{samples.pixels.remove(&cells[50].texture.id());},
                _=>if let TileImageSample::Rgba(p)=samples.pixels.get_mut(&cells[5*12+1].texture.id()).unwrap(){p[3]=0;},
            }
            assert!(palette(&altered.iter().collect::<Vec<_>>(),&g,p,&samples).is_none());
        }
    }
}
