//! Complete native Kanto enclosure: four rail runs and shared stone posts.
use super::*;

const BLOCKS:[[u16;2];2]=[[0x49,0x48],[0x44,0x45]];
const ART:[[u16;16];4]=[
    [0x21,0x20,0x20,0x20,0x10,0x23,0x23,0x23,0x10,0x23,0x23,0x23,0x10,0x23,0x23,0x21],
    [0x20,0x20,0x20,0x21,0x23,0x23,0x23,0x10,0x23,0x23,0x23,0x10,0x21,0x23,0x23,0x10],
    [0x10,0x23,0x23,0x21,0x10,0x23,0x23,0x23,0x10,0x23,0x23,0x23,0x21,0x20,0x20,0x20],
    [0x21,0x23,0x23,0x10,0x23,0x23,0x23,0x10,0x23,0x23,0x23,0x10,0x20,0x20,0x20,0x21],
];
fn art(x:usize,y:usize)->u16 {ART[y/4*2+x/4][y%4*4+x%4]}
fn matches(cells:&[&VisualTile],g:&GridGeometry,x:usize,y:usize)->bool {
    x+8<=g.width && y+8<=g.height && (0..8).all(|dy|(0..8).all(|dx| {
        let s=&cells[(y+dy)*g.width+x+dx].source;
        s.tileset_id.as_ref()=="kanto" && s.metatile_id==BLOCKS[dy/4][dx/4]
            && s.subtile_column as usize==dx%4 && s.subtile_row as usize==dy%4
            && s.tile_index==art(dx,dy)
    }))
}
fn stone_palette(images:&TerrainImageSamples,tile:&VisualTile)->Option<[[f32;4];3]> {
    let rgba=tile_rgba(images,tile).ok()?;
    if rgba.len()!=256 || rgba.chunks_exact(4).any(|p|p[3]!=255) {return None;}
    fence_palette(images,&[tile])
}

pub(super) fn append(mesh:&mut TerrainMeshData,cells:&[&VisualTile],shapes:&[CellShape],
    g:&GridGeometry,claimed:&mut [bool],images:Option<&TerrainImageSamples>) {
    let Some(images)=images else{return;};
    for y in 0..g.height.saturating_sub(7) {for x in 0..g.width.saturating_sub(7) {
        if !matches(cells,g,x,y) || !clear(claimed,g,[x,y,8,8]) {continue;}
        let sample=(y+1)*g.width+x+1;
        if !matches!(shapes[sample],CellShape::Flat|CellShape::PlaneAt{height:0.}) {continue;}
        let mut posts=Vec::new();let mut valid=true;
        for dy in 0..8 {for dx in 0..8 {
            let tile=cells[(y+dy)*g.width+x+dx];
            if art(dx,dy)!=0x23 && !tile_rgba(images,tile).is_ok_and(|p|
                p.len()==256 && p.chunks_exact(4).all(|p|p[3]==255)) {valid=false;}
            if art(dx,dy)==0x21 {
                if let Some(paint)=stone_palette(images,tile) {posts.push((dx,dy,paint));}
                else {valid=false;}
            }
        }}
        let Some(palette)=fence_palette(images,&[cells[y*g.width+x+1],cells[(y+1)*g.width+x]]) else {continue;};
        if !valid {continue;}
        let t=g.tile_height;let width=g.tile_width;
        let mut local=SurfaceMeshData::default();
        let gray=palette[1];let rail=[gray[0],gray[1],gray[2]];
        // One closed pair of rails per side, fitted into the stone shoulders.
        // Long runs avoid coincident surfaces at every source-tile join.
        for height in [0.28,0.58] {
            for z in [0.5*t,7.5*t] {
                kanto_craft::block(&mut local,[0.82*width,7.18*width,
                    (height-0.055)*t,(height+0.055)*t,z-t*0.055,z+t*0.055],t*0.015,rail,3.,t);
            }
            for xx in [0.5*width,7.5*width] {
                kanto_craft::block(&mut local,[xx-width*0.055,xx+width*0.055,
                    (height-0.055)*t,(height+0.055)*t,0.82*t,7.18*t],t*0.015,rail,3.,t);
            }
        }
        // The native round stone is shared by the corners and centre pile.
        // Reuse the irregular boulder, rather than a square-footed dock bollard.
        let stone=crate::exterior_models::model(crate::exterior_models::Kind::ShoreBoulder);
        for (dx,dy,paint) in posts {
            let bounds=[(dx as f32+0.05)*width,(dx as f32+0.95)*width,
                (dy as f32+0.05)*t,(dy as f32+0.95)*t];
            let first=local.positions.len();
            stone.append_fitted(&mut local,bounds,0.,t*0.78/(stone.max[1]-stone.min[1]),None);
            for i in first..local.positions.len() {
                let ny=local.normals[i][1];
                let color=paint[if ny>0.55 {2}else if ny < -0.25 {0}else {1}];
                let gain=0.72+0.22*ny.max(0.);
                local.colors[i]=[color[0]*gain,color[1]*gain,color[2]*gain,1.];
                // Solid meshes carry sampled linear pigment, not atlas UVs.
                local.uvs[i]=crate::scenery_finish::SOURCE_PAINT_UV;
            }
        }
        let offset=g.bounds(x,y);let first=mesh.solid.positions.len();
        append_surface(&mut mesh.solid,&local);
        for p in &mut mesh.solid.positions[first..] {p[0]+=offset.0;p[2]+=offset.2;}
        for dy in 0..8 {for dx in 0..8 {
            if art(dx,dy)==0x23 {continue;}
            floor(mesh,cells,shapes,g,[x+dx,y+dy,1,1],sample,claimed);
            mark_authored_rect(mesh,g,[x+dx,y+dy,1,1],"world-exterior/kanto-yard-fence");
        }}
    }}
}

fn append_surface(out:&mut SurfaceMeshData,from:&SurfaceMeshData) {
    let base=out.positions.len() as u32;
    out.positions.extend_from_slice(&from.positions);out.normals.extend_from_slice(&from.normals);
    out.colors.extend_from_slice(&from.colors);out.uvs.extend_from_slice(&from.uvs);
    out.indices.extend(from.indices.iter().map(|i|i+base));
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture()->(Vec<VisualTile>,GridGeometry,TerrainImageSamples) {
        let mut assets=bevy::prelude::Assets::<bevy::prelude::Image>::default();
        let texture=assets.add(bevy::prelude::Image::default());
        let tiles=(0..64).map(|i|VisualTile {column:(i%8) as u32,row:(i/8) as u32,
            texture:texture.clone(),priority:false,animation_frames:None,
            source:VisualTileSource {tileset_id:"kanto".into(),metatile_id:BLOCKS[i/8/4][i%8/4],
                subtile_column:(i%4) as u8,subtile_row:(i/8%4) as u8,tile_index:art(i%8,i/8)}}).collect();
        let g=GridGeometry {width:8,height:8,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut images=TerrainImageSamples::default();
        images.pixels.insert(texture.id(),TileImageSample::Rgba(
            [[20,20,20,255],[75,75,75,255],[140,140,140,255],[222,255,222,255]].repeat(16).into_iter().flatten().collect()));
        (tiles,g,images)
    }
    #[test]
    fn complete_yard_gets_closed_rails_and_eight_shared_stones_without_changing_ground_support() {
        let (tiles,mut g,images)=fixture();let cells=tiles.iter().collect::<Vec<_>>();
        let shapes=vec![CellShape::Flat;64];
        for t in [8.,16.] {g.tile_width=t;g.tile_height=t;
            let mut mesh=TerrainMeshData {footing_heights:vec![3.25;64],authored_cells:vec![None;64],..Default::default()};
            let mut claimed=vec![false;64];append(&mut mesh,&cells,&shapes,&g,&mut claimed,Some(&images));
            assert_eq!(claimed.iter().filter(|&&v|v).count(),32);
            assert_eq!(mesh.footing_heights,vec![3.25;64]);
            assert!(mesh.solid.positions.iter().all(|p|p[0]>=0. && p[0]<=8.*t && p[2]>=0. && p[2]<=8.*t && p[1]>=0. && p[1]<=t));
            assert!(mesh.solid.positions.iter().any(|p|p[1]>t*0.77));
            assert!(mesh.solid.positions.iter().all(|p|p[1]<=t*0.781));
            assert!(mesh.solid.indices.len()/3<8000);
            for tri in mesh.solid.indices.chunks_exact(3) {
                let p=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.solid.positions[i as usize]));
                assert!((p[1]-p[0]).cross(p[2]-p[0]).dot(Vec3::from_array(mesh.solid.normals[tri[0] as usize]))>0.);
            }
            let count=crate::exterior_models::model(crate::exterior_models::Kind::ShoreBoulder)
                .surface.positions.len()*8;
            let stone_start=mesh.solid.positions.len()-count;
            assert!(mesh.solid.uvs[stone_start..].iter().all(|&uv|
                uv==crate::scenery_finish::SOURCE_PAINT_UV));
            assert!(mesh.solid.colors[stone_start..].iter().all(|c|c[0]<0.9));
            assert!(mesh.solid.colors[stone_start..].iter().any(|c|c[1]>c[0]*1.1),
                "stone highlights retain the native mint pigment");
            let original=mesh.clone();append(&mut mesh,&cells,&shapes,&g,&mut claimed,Some(&images));assert_eq!(mesh,original);
        }
    }
    #[test]
    fn cropped_changed_foreign_claimed_or_unpainted_yards_keep_the_native_drawing() {
        let (tiles,g,images)=fixture();let shapes=vec![CellShape::Flat;64];
        for index in 0..64 {for change in 0..4 {
            let mut bad=tiles.clone();
            match change {0=>bad[index].source.tile_index^=1,1=>bad[index].source.metatile_id^=1,
                2=>bad[index].source.subtile_column^=1,_=>bad[index].source.tileset_id="johto".into()}
            let mut mesh=TerrainMeshData::default();let mut claimed=vec![false;64];
            append(&mut mesh,&bad.iter().collect::<Vec<_>>(),&shapes,&g,&mut claimed,Some(&images));
            assert!(mesh.solid.positions.is_empty() && claimed.iter().all(|v|!*v));
        }}
        for missing in [true,false] {
            let mut mesh=TerrainMeshData::default();let mut claimed=vec![false;64];
            if !missing {claimed[0]=true;}
            let before=claimed.clone();
            append(&mut mesh,&tiles.iter().collect::<Vec<_>>(),&shapes,&g,&mut claimed,if missing {None}else{Some(&images)});
            assert!(mesh.solid.positions.is_empty());assert_eq!(claimed,before);
        }
    }
}
