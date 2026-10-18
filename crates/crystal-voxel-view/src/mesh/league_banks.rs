//! Closed blue bank walls from complete native pool drawings.
use super::*;
struct Bank {x:usize,y:usize,ground:[usize;4]}
fn resolve(map:&str,cells:&[&VisualTile],g:&GridGeometry,blocked:&[bool])->Vec<Bank> {
    let liquid=super::league_liquids::mask(map,cells,g,blocked);
    let mut banks=Vec::new();
    for y in 1..g.height.saturating_sub(3) {for x in 0..g.width.saturating_sub(3) {
        let s=&cells[y*g.width+x].source;
        if s.subtile_column!=0 || s.subtile_row!=0
            || s.metatile_id!=if map=="WillsRoom" {0x1f}else{0x2c}
            || !(0..4).all(|dx|liquid[(y+3)*g.width+x+dx]) {continue;}
        let ground=std::array::from_fn(|dx|(y-1)*g.width+x+dx);
        if !ground.iter().enumerate().all(|(dx,&i)| {
            let s=&cells[i].source;
            !blocked[i] && s.tileset_id.as_ref()=="elite_four_room" && s.tile_index==0x52
                && matches!(s.metatile_id,0x34|0x38) && s.subtile_row==3 && s.subtile_column as usize==dx
        }) {continue;}
        banks.push(Bank {x,y,ground});
    }}
    banks
}
pub(super) fn finish(mesh:&mut TerrainMeshData,frame:&VisualWorldFrame,cells:&[&VisualTile],
    g:&GridGeometry,profiles:Option<&crate::live_profiles::Document>) {
    let map=frame.map_id.as_ref();
    if !matches!(map,"WillsRoom"|"BrunosRoom") || g.width.checked_mul(g.height)!=Some(cells.len())
        || g.width==0 || mesh.authored_cells.len()!=cells.len() {return;}
    let mut blocked=mesh.authored_cells.iter().map(Option::is_some).collect::<Vec<_>>();
    for p in live::resolve(cells,g.width,g.height,map,profiles) {for i in p.indices(g.width) {blocked[i]=true;}}
    let banks=resolve(map,cells,g,&blocked);let mut owned=vec![false;cells.len()];
    for p in &banks {for y in 0..2 {for x in 0..4 {owned[(p.y+y)*g.width+p.x+x]=true;}}}
    // Fold only the original horizontal bank paint. The pool, adjacent floor,
    // apron and their retained animation UVs remain independent.
    for surface in [&mut mesh.textured,&mut mesh.solid,&mut mesh.animated_textured,&mut mesh.animated_solid] {
        let mut keep=Vec::with_capacity(surface.indices.len());
        for tri in surface.indices.chunks_exact(3) {
            let p=tri.iter().map(|&i|Vec3::from_array(surface.positions[i as usize])).sum::<Vec3>()/3.;
            let (x,y)=(((p.x-g.origin_x)/g.tile_width).floor()as i32,((p.z-g.origin_z)/g.tile_height).floor()as i32);
            let fold=x>=0 && y>=0 && (x as usize)<g.width && (y as usize)<g.height
                && owned[y as usize*g.width+x as usize] && tri.iter().all(|&i|
                    surface.positions[i as usize][1].abs()<0.00001 && surface.normals[i as usize][1]>0.9);
            if !fold {keep.extend_from_slice(tri);}
        }
        surface.indices=keep;
    }
    let unit=g.tile_height/SOURCE_TILE_HEIGHT;
    for p in banks {
        let (w,_,n,_)=g.bounds(p.x,p.y);let e=w+4.*g.tile_width;let s=n+2.*g.tile_height;
        let back=s-4.*unit;let height=16.*unit;
        for row in 0..2 {for column in 0..4 {
            let (west,east,north,south)=g.bounds(p.x+column,p.y+row);
            let end=south;let source=p.ground[column];let uv=g.uv(source%g.width,source/g.width);
            if end>north {let v=uv.2+(uv.3-uv.2)*(end-north)/(south-north);
                append_quad(&mut mesh.textured,[[west,0.,north],[west,0.,end],[east,0.,end],[east,0.,north]],
                    [0.,1.,0.],[[uv.0,uv.2],[uv.0,v],[uv.1,v],[uv.1,uv.2]],TEXTURED_SHADE);
            }
        }}
        let start=mesh.textured.positions.len();
        for row in 0..2 {for column in 0..4 {
            let (west,east,_,_)=g.bounds(p.x+column,p.y+row);let uv=g.uv(p.x+column,p.y+row);
            let top=height-row as f32*8.*unit;let bottom=top-8.*unit;
            append_quad(&mut mesh.textured,[[west,bottom,s],[east,bottom,s],[east,top,s],[west,top,s]],
                [0.,0.,1.],[[uv.0,uv.3],[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2]],TEXTURED_SHADE);
        }}
        let uv=g.uv(p.x,p.y);
        let top=[uv.0+(uv.1-uv.0)*2.5/8.,uv.2+(uv.3-uv.2)*0.5/8.];
        let side=[top[0],uv.2+(uv.3-uv.2)*2.5/8.];
        for (points,normal,paint) in [
            ([[w,height,back],[w,height,s],[e,height,s],[e,height,back]],[0.,1.,0.],top),
            ([[w,0.,back],[e,0.,back],[e,0.,s],[w,0.,s]],[0.,-1.,0.],side),
            ([[e,0.,back],[w,0.,back],[w,height,back],[e,height,back]],[0.,0.,-1.],side),
            ([[w,0.,back],[w,0.,s],[w,height,s],[w,height,back]],[-1.,0.,0.],side),
            ([[e,0.,s],[e,0.,back],[e,height,back],[e,height,s]],[1.,0.,0.],side),
        ] {append_quad(&mut mesh.textured,points,normal,[paint;4],TEXTURED_SHADE);}
        mesh.textured.cutaway_ranges.push(start..mesh.textured.positions.len());
        for row in 0..2 {for column in 0..4 {
            mesh.authored_cells[(p.y+row)*g.width+p.x+column]=Some("league:native-pool-bank");
        }}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::Vec2;
    #[test]
    fn external_league_banks_fold_eight_source_rows_into_four_closed_native_walls() {
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(
            std::path::PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap())).unwrap();
        let pack=loaded.pack();let art=&pack.runtime_files()["data/tilesets/elite_four_room_metatiles.bin"];
        for (map,block,tile) in [("WillsRoom",0x1f,0x13),("BrunosRoom",0x2c,0x39)] {
            let data=&pack.data().maps[map];let (w,h)=(data.attributes.width as usize*4,data.attributes.height as usize*4);
            let tiles=(0..w*h).map(|i| {
                let (x,y)=(i%w,i/w);let block=data.blocks[y/4*(w/4)+x/4];
                VisualTile {column:x as u32,row:y as u32,priority:false,animation_frames:None,texture:Default::default(),
                    source:VisualTileSource {tileset_id:"elite_four_room".into(),metatile_id:block,
                        subtile_column:(x%4)as u8,subtile_row:(y%4)as u8,
                        tile_index:art[block as usize*16+y%4*4+x%4]as u16}}
            }).collect::<Vec<_>>();
            let cells=tiles.iter().collect::<Vec<_>>();
            let g=GridGeometry {width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
            let banks=resolve(map,&cells,&g,&vec![false;w*h]);assert_eq!(banks.len(),4);
            assert_eq!(banks.iter().map(|p|(p.x,p.y)).collect::<Vec<_>>(),vec![(0,12),(16,12),(4,20),(12,20)]);
            let mut frame=super::super::tests::frame(w as u32,h as u32,tiles.iter().map(|t|t.source.clone()).collect());
            frame.map_id=map.into();
            let mut baseline=TerrainMeshData {footing_heights:vec![2.75;w*h],authored_cells:vec![None;w*h],..Default::default()};
            for y in 0..h {for x in 0..w {append_top(&mut baseline.textured,g.bounds(x,y).into(),0.,g.uv(x,y));}}
            super::super::league_liquids::finish(&mut baseline,&frame,&cells,&g,None);
            let mut out=baseline.clone();finish(&mut out,&frame,&cells,&g,None);
            assert_eq!(out.footing_heights,baseline.footing_heights);
            assert_eq!(&out.textured.uvs[..baseline.textured.uvs.len()],baseline.textured.uvs.as_slice());
            assert_eq!(out.authored_cells.iter().filter(|&&v|v==Some("league:native-pool-bank")).count(),32);
            assert_eq!(out.textured.cutaway_ranges.len(),4);
            for range in &out.textured.cutaway_ranges {
                let mut volume=0.;let mut front_area=0.;
                for tri in out.textured.indices.chunks_exact(3).filter(|tri|tri.iter().all(|&i|range.contains(&(i as usize)))) {
                    let p=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(out.textured.positions[i as usize]));
                    let normal=Vec3::from_array(out.textured.normals[tri[0]as usize]);
                    let area=(p[1]-p[0]).cross(p[2]-p[0]);assert!(area.dot(normal)>0.);
                    volume+=p[0].dot(p[1].cross(p[2]))/6.;
                    if normal==Vec3::Z {
                        front_area+=area.length()*0.5;
                        let uv=tri.iter().map(|&i|Vec2::from_array(out.textured.uvs[i as usize])).sum::<Vec2>()/3.;
                        assert_eq!(cells[(uv.y*h as f32)as usize*w+(uv.x*w as f32)as usize].source.tile_index,tile);
                    }
                }
                assert!((volume-32.*4.*16.).abs()<0.1,"the native bank is a closed body");
                assert_eq!(front_area,32.*16.);
            }
            let (opaque,groups)=crate::occluder_fade::prepare(out.textured.clone());
            assert_eq!(groups.len(),4);assert!(!opaque.positions.is_empty(),"floor and pool backing remain opaque");
            let mut backing=0.;
            for tri in opaque.indices.chunks_exact(3) {
                let p=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(opaque.positions[i as usize]));
                let center=p.iter().copied().sum::<Vec3>()/3.;
                let i=(center.z/8.)as usize*w+(center.x/8.)as usize;
                if i<out.authored_cells.len() && out.authored_cells[i]==Some("league:native-pool-bank")
                    && p.iter().all(|p|p.y==0.) && opaque.normals[tri[0]as usize][1]>0.9 {
                    backing+=(p[1]-p[0]).cross(p[2]-p[0]).length()*0.5;
                }
            }
            assert_eq!(backing,32.*64.,"fading a wall must reveal a complete native floor, without a black strip");
            for index in [banks[0].y*w+banks[0].x,banks[0].ground[0]] {
                let mut changed=tiles.clone();changed[index].source.tile_index^=1;
                assert_eq!(resolve(map,&changed.iter().collect::<Vec<_>>(),&g,&vec![false;w*h]).len(),3);
            }
            let custom:crate::live_profiles::Document=serde_json::from_value(serde_json::json!({"objects":[{
                "name":"Custom bank","map":map,"tileset":"elite_four_room","metatile":block,"origin":[0,0],
                "tiles":[[tile]],"ground":82,"top_pixels":0,"depth_pixels":8}]})).unwrap();
            let mut protected=baseline.clone();finish(&mut protected,&frame,&cells,&g,Some(&custom));
            assert_eq!(protected,baseline,"live overrides retain the complete bank drawing");
        }
    }
}
