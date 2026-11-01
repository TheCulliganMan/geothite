//! Shallow native pools. Atlas slots/animation and authoritative support stay intact.
use super::*;

fn drawing(map:&str,block:u16)->Option<[[u16;4];4]> {
    Some(match (map,block) {
        ("WillsRoom",0x1f)=>[[0x13;4],[0x13;4],[0x50;4],[0x24;4]],
        ("WillsRoom",0x24)=>[[0x0e,0x5d,0x0e,0x0e],[0x0e,0x5a,0x5c,0x0e],
            [0x5c,0x0e,0x5d,0x5a],[0x0e,0x0e,0x5a,0x0e]],
        ("BrunosRoom",0x2b)=>[[2,2,0x38,2],[0x38,2,0x5b,2],[2;4],[2,0x5b,2,2]],
        ("BrunosRoom",0x2c)=>[[0x39;4],[0x39;4],[2,2,2,0x5b],[2,0x38,2,2]],
        _=>return None,
    })
}
fn liquid_row(map:&str,block:u16,row:usize)->bool {
    match (map,block) {
        ("WillsRoom",0x1f)=>row>=2,
        ("BrunosRoom",0x2c)=>row>=2,
        _=>true,
    }
}
pub(super) fn mask(map:&str,cells:&[&VisualTile],g:&GridGeometry,blocked:&[bool])->Vec<bool> {
    let mut out=vec![false;cells.len()];
    for y in 0..g.height.saturating_sub(3) {for x in 0..g.width.saturating_sub(3) {
        let block=cells[y*g.width+x].source.metatile_id;
        let Some(art)=drawing(map,block)else{continue;};
        if !(0..4).all(|dy|(0..4).all(|dx| {
            let i=(y+dy)*g.width+x+dx;let s=&cells[i].source;
            !blocked[i] && s.tileset_id.as_ref()=="elite_four_room" && s.metatile_id==block
                && s.subtile_column as usize==dx && s.subtile_row as usize==dy && s.tile_index==art[dy][dx]
        })) {continue;}
        for dy in 0..4 {if liquid_row(map,block,dy) {for dx in 0..4 {out[(y+dy)*g.width+x+dx]=true;}}}
    }}
    out
}
pub(super) fn finish(mesh:&mut TerrainMeshData,frame:&VisualWorldFrame,cells:&[&VisualTile],
    g:&GridGeometry,profiles:Option<&crate::live_profiles::Document>) {
    let map=frame.map_id.as_ref();
    if !matches!(map,"WillsRoom"|"BrunosRoom") || g.width.checked_mul(g.height)!=Some(cells.len())
        || g.width==0 || mesh.authored_cells.len()!=cells.len() {return;}
    let mut blocked=mesh.authored_cells.iter().map(Option::is_some).collect::<Vec<_>>();
    for p in live::resolve(cells,g.width,g.height,map,profiles) {for i in p.indices(g.width) {blocked[i]=true;}}
    let native=mask(map,cells,g,&blocked);
    let unit=g.tile_height/SOURCE_TILE_HEIGHT;
    // Water shares the short shoreline drop used throughout the world.
    // The separately authored lava keeps its existing recess.
    let depth=if map=="WillsRoom" {-crate::profile::WATER_HEIGHT}else{16.};
    let (bank_block,bank_tile)=if map=="WillsRoom" {(0x1f,0x13)}else{(0x2c,0x39)};
    let top=-depth*unit;let base=top-2.*unit;
    let blue=cells.iter().position(|t|t.source.metatile_id==bank_block
        && t.source.tile_index==bank_tile);
    let mut lowered=vec![false;cells.len()];
    for surface in [&mut mesh.textured,&mut mesh.solid,&mut mesh.animated_textured,&mut mesh.animated_solid] {
        let mut vertices=vec![false;surface.positions.len()];
        for tri in surface.indices.chunks_exact(3) {
            if !tri.iter().all(|&i|surface.positions[i as usize][1].abs()<0.00001
                && surface.normals[i as usize][1]>0.9) {continue;}
            let p=tri.iter().map(|&i|Vec3::from_array(surface.positions[i as usize])).sum::<Vec3>()/3.;
            let (x,y)=(((p.x-g.origin_x)/g.tile_width).floor()as i32,
                ((p.z-g.origin_z)/g.tile_height).floor()as i32);
            if x<0 || y<0 || x as usize>=g.width || y as usize>=g.height {continue;}
            let i=y as usize*g.width+x as usize;if !native[i] {continue;}
            lowered[i]=true;for &i in tri {vertices[i as usize]=true;}
        }
        for (p,lower) in surface.positions.iter_mut().zip(vertices) {if lower {p[1]=top;}}
    }
    for (i,&lower) in lowered.iter().enumerate() {
        if !lower {continue;}
        let (x,y)=(i%g.width,i/g.width);let (w,e,n,s)=g.bounds(x,y);let uv=g.uv(x,y);
        append_quad(&mut mesh.textured,[[w,base,n],[e,base,n],[e,base,s],[w,base,s]],
            [0.,-1.,0.],[[uv.0,uv.2],[uv.1,uv.2],[uv.1,uv.3],[uv.0,uv.3]],TEXTURED_SHADE);
        let neighbors=[y.checked_sub(1).map(|y|y*g.width+x),(x+1<g.width).then_some(i+1),
            (y+1<g.height).then_some(i+g.width),x.checked_sub(1).map(|x|y*g.width+x)];
        for (side,other) in neighbors.into_iter().enumerate() {
            if other.is_some_and(|j|lowered[j]) {continue;}
            // The complete bank drawing supplies its own two-course face.
            // Do not put a second coplanar rim over that blue source art.
            if other.is_some_and(|j| {
                let s=&cells[j].source;
                s.metatile_id==bank_block && s.subtile_row<2 && s.tile_index==bank_tile
            }) {continue;}
            let donor=blue.unwrap_or_else(||other.unwrap_or(i));let uv=g.uv(donor%g.width,donor/g.width);
            let paint=[uv.0+(uv.1-uv.0)*2.5/8.,uv.2+(uv.3-uv.2)*2.5/8.];
            let (a,b,normal)=match side {
                0=>([e,n],[w,n],[0.,0.,1.]),1=>([e,s],[e,n],[-1.,0.,0.]),
                2=>([w,s],[e,s],[0.,0.,-1.]),_=>([w,n],[w,s],[1.,0.,0.]),
            };
            append_quad(&mut mesh.textured,[[a[0],0.,a[1]],[b[0],0.,b[1]],
                [b[0],base,b[1]],[a[0],base,a[1]]],normal,[paint;4],TEXTURED_SHADE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_league_pools_keep_native_atlas_animation_slots_and_authoritative_support() {
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(
            std::path::PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap())).unwrap();
        let pack=loaded.pack();let art=&pack.runtime_files()["data/tilesets/elite_four_room_metatiles.bin"];
        for (map,pure,count,edge_count) in [("WillsRoom",0x24,160,32),("BrunosRoom",0x2b,160,32)] {
            let depth=if map=="WillsRoom" {-crate::profile::WATER_HEIGHT}else{16.};
            let data=&pack.data().maps[map];let (w,h)=(data.attributes.width as usize*4,data.attributes.height as usize*4);
            let tiles=(0..w*h).map(|i| {
                let (x,y)=(i%w,i/w);let block=data.blocks[y/4*(w/4)+x/4];
                VisualTile {column:x as u32,row:y as u32,priority:false,animation_frames:None,texture:Default::default(),
                    source:VisualTileSource {tileset_id:"elite_four_room".into(),metatile_id:block,
                        subtile_column:(x%4)as u8,subtile_row:(y%4)as u8,
                        tile_index:art[block as usize*16+y%4*4+x%4]as u16}}
            }).collect::<Vec<_>>();
            let g=GridGeometry {width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
            let cells=tiles.iter().collect::<Vec<_>>();let eligible=mask(map,&cells,&g,&vec![false;w*h]);
            assert_eq!(eligible.iter().filter(|&&v|v).count(),count);
            let mut baseline=TerrainMeshData {footing_heights:vec![2.75;w*h],authored_cells:vec![None;w*h],..Default::default()};
            for y in 0..h {for x in 0..w {append_top(&mut baseline.textured,g.bounds(x,y).into(),0.,g.uv(x,y));}}
            let mut frame=super::super::tests::frame(w as u32,h as u32,tiles.iter().map(|t|t.source.clone()).collect());
            frame.map_id=map.into();
            let mut out=baseline.clone();finish(&mut out,&frame,&cells,&g,None);
            assert_eq!(&out.textured.uvs[..baseline.textured.uvs.len()],baseline.textured.uvs.as_slice(),
                "native atlas/animation bindings are never redirected");
            assert_eq!(out.footing_heights,baseline.footing_heights);
            assert_eq!(out.authored_cells,baseline.authored_cells,"a pool finish never earns object claims");
            for (i,&liquid) in eligible.iter().enumerate() {for v in 0..4 {
                assert_eq!(out.textured.positions[i*4+v][1],if liquid {-depth}else{0.});
            }}
            let mut bottom_area=0.;
            for tri in out.textured.indices[baseline.textured.indices.len()..].chunks_exact(3) {
                let p=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(out.textured.positions[i as usize]));
                let normal=Vec3::from_array(out.textured.normals[tri[0]as usize]);
                let area=(p[1]-p[0]).cross(p[2]-p[0]);assert!(area.dot(normal)>0.);
                assert!(p.iter().all(|p|(-depth-2.0..=0.0).contains(&p.y)));
                if normal==Vec3::NEG_Y {bottom_area+=area.length()*0.5;}
            }
            assert_eq!(bottom_area,count as f32*64.);
            let first=cells.iter().position(|t|t.source.metatile_id==pure
                && t.source.subtile_column==0 && t.source.subtile_row==0).unwrap();
            let mut altered=tiles.clone();altered[first].source.tile_index^=1;
            assert_eq!(mask(map,&altered.iter().collect::<Vec<_>>(),&g,&vec![false;w*h])
                .iter().filter(|&&v|v).count(),count-16,"an edited source rejects its whole drawing");
            let mut reserved=baseline.clone();reserved.authored_cells[first]=Some("custom:pool");
            finish(&mut reserved,&frame,&cells,&g,None);
            assert_eq!(reserved.textured.positions[..w*h*4].iter().filter(|p|p[1]==-depth).count(),(count-16)*4);
            let tile=if map=="WillsRoom" {14}else{2};
            let custom:crate::live_profiles::Document=serde_json::from_value(serde_json::json!({"objects":[{
                "name":"Custom pool","map":map,"tileset":"elite_four_room","metatile":pure,"origin":[0,0],
                "tiles":[[tile]],"ground":1,"top_pixels":0,"depth_pixels":8}]})).unwrap();
            let mut out=baseline.clone();finish(&mut out,&frame,&cells,&g,Some(&custom));
            assert_eq!(out.textured.positions[..w*h*4].iter().filter(|p|p[1]==-depth).count(),edge_count*4);
            frame.map_id="KogasRoom".into();let mut out=baseline.clone();finish(&mut out,&frame,&cells,&g,None);
            assert_eq!(out,baseline);
        }
    }
}
