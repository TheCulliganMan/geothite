//! The complete native communications display, above its own closed plinth.
use super::*;
const ART:[[u16;10];6]=[
    [134,166,166,166,166,166,166,166,166,135],
    [183,142,142,142,142,142,142,142,142,182],
    [183,142,142,142,142,142,142,142,142,182],
    [183,142,142,142,142,142,142,142,142,182],
    [183,142,142,142,142,142,142,142,142,182],
    [150,167,167,167,167,167,167,167,167,151],
];
fn local(frame:&VisualWorldFrame,g:&GridGeometry,x:i32,y:i32)->Option<(usize,usize)> {
    let(x,y)=(x-frame.grid_origin.x,y-frame.grid_origin.y);
    (x>=0 && y>=0 && (x as usize)<g.width && (y as usize)<g.height).then_some((x as usize,y as usize))
}
pub(super) fn finish(mesh:&mut TerrainMeshData,frame:&VisualWorldFrame,cells:&[&VisualTile],
    g:&GridGeometry,profiles:Option<&crate::live_profiles::Document>) {
    if frame.map_id.as_ref()!=crate::pokecom::ADMIN_OFFICE
        || frame.source_map_size_core_tiles!=Some(bevy::prelude::UVec2::new(32,32))
        || g.width*g.height!=cells.len() || mesh.authored_cells.len()!=cells.len() {return;}
    let Some((x,y))=local(frame,g,49,2) else{return;};
    let Some((dx,dy))=local(frame,g,47,8) else{return;};
    if x+10>g.width || y+6>g.height {return;}
    let donor=dy*g.width+dx;let d=&cells[donor].source;
    if d.tileset_id.as_ref()!="pokecom_center" || d.metatile_id!=1 || d.tile_index!=17
        || d.subtile_column!=3 || d.subtile_row!=0 {return;}
    for row in 0..6 {for column in 0..10 {
        let i=(y+row)*g.width+x+column;let s=&cells[i].source;
        let ax=49+column;let ay=2+row;
        let block=if ay<4 {[0x26,0x2a,0x27]}else{[0x2f,0x29,0x23]}[(ax-48)/4];
        if s.tileset_id.as_ref()!="pokecom_center" || s.metatile_id!=block || s.tile_index!=ART[row][column]
            || s.subtile_column as usize!=ax%4 || s.subtile_row as usize!=ay%4
            || mesh.authored_cells[i].is_some() {return;}
    }}
    if live::resolve(cells,g.width,g.height,&frame.map_id,profiles).iter().any(|p|
        p.indices(g.width).any(|i|i==donor || ((x..x+10).contains(&(i%g.width))
            && (y..y+6).contains(&(i/g.width))))) {return;}
    // Remove only the original upward-facing picture. Other architecture and
    // the floor around the display remain independent of its cutaway group.
    for out in [&mut mesh.textured,&mut mesh.solid,&mut mesh.animated_textured,&mut mesh.animated_solid] {
        let keep=out.indices.chunks_exact(3).filter(|tri| {
            let p=tri.iter().map(|&i|Vec3::from_array(out.positions[i as usize])).sum::<Vec3>()/3.;
            let cx=(p.x-g.origin_x)/g.tile_width;let cy=(p.z-g.origin_z)/g.tile_height;
            !((x as f32..(x+10)as f32).contains(&cx) && (y as f32..(y+6)as f32).contains(&cy)
                && tri.iter().all(|&i|out.positions[i as usize][1].abs()<0.00001 && out.normals[i as usize][1]>0.9))
        }).flatten().copied().collect();
        out.indices=keep;
    }
    let floor=g.uv(dx,dy);
    // The first two source rows were projected over the rear wall. They do
    // not authorize new floor behind its source-row-four boundary.
    for row in 2..6 {for column in 0..10 {
        let(w,e,n,s)=g.bounds(x+column,y+row);
        append_top(&mut mesh.textured,[w,e,n,s],0.,floor);
    }}
    let scale=g.tile_height/8.;let(w,_,n,_)=g.bounds(x,y);
    let e=w+10.*g.tile_width;let s=n+6.*g.tile_height;let low=16.*scale;let high=64.*scale;
    let uv=g.uv(x,y);let paint=[uv.0+(uv.1-uv.0)*2.5/8.,uv.2+(uv.3-uv.2)*2.5/8.];
    let start=mesh.textured.positions.len();
    // Base and display are closed individually and fade together. Original
    // screen/frame/bezel pixels are kept in their native phase and aspect.
    closed_box(&mut mesh.textured,[w,e,s-8.*scale,s],0.,low,paint,None);
    closed_box(&mut mesh.textured,[w,e,s-4.*scale,s],low,high,paint,Some([0.,0.,1.]));
    for row in 0..6 {for column in 0..10 {
        let(left,right,_,_)=g.bounds(x+column,y+row);let uv=g.uv(x+column,y+row);
        let top=high-row as f32*8.*scale;let bottom=top-8.*scale;
        append_quad(&mut mesh.textured,[[left,bottom,s],[right,bottom,s],[right,top,s],[left,top,s]],
            [0.,0.,1.],[[uv.0,uv.3],[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2]],TEXTURED_SHADE);
        mesh.authored_cells[(y+row)*g.width+x+column]=Some("pokecom:native-communications-display");
    }}
    mesh.textured.cutaway_ranges.push(start..mesh.textured.positions.len());
}
fn closed_box(out:&mut SurfaceMeshData,b:[f32;4],low:f32,high:f32,paint:[f32;2],omit:Option<[f32;3]>) {
    let[w,e,n,s]=b;
    for(points,normal)in[
        ([[w,high,n],[w,high,s],[e,high,s],[e,high,n]],[0.,1.,0.]),
        ([[w,low,n],[e,low,n],[e,low,s],[w,low,s]],[0.,-1.,0.]),
        ([[e,low,n],[w,low,n],[w,high,n],[e,high,n]],[0.,0.,-1.]),
        ([[w,low,s],[e,low,s],[e,high,s],[w,high,s]],[0.,0.,1.]),
        ([[w,low,n],[w,low,s],[w,high,s],[w,high,n]],[-1.,0.,0.]),
        ([[e,low,s],[e,low,n],[e,high,n],[e,high,s]],[1.,0.,0.]),
    ] {if omit!=Some(normal) {append_quad(out,points,normal,[paint;4],TEXTURED_SHADE);}}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_pokecom_display_preserves_native_frame_closed_volume_and_room_side_floor() {
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(
            std::path::PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap())).unwrap();
        let pack=loaded.pack();let map=&pack.data().maps[crate::pokecom::ADMIN_OFFICE];
        let art=&pack.runtime_files()["data/tilesets/pokecom_center_metatiles.bin"];
        let(w,h)=(map.attributes.width as usize*4,map.attributes.height as usize*4);
        let sources=(0..w*h).map(|i|{let(x,y)=(i%w,i/w);let b=map.blocks[y/4*(w/4)+x/4];
            VisualTileSource{tileset_id:"pokecom_center".into(),metatile_id:b,
                subtile_column:(x%4)as u8,subtile_row:(y%4)as u8,
                tile_index:art[b as usize*16+y%4*4+x%4]as u16}}).collect();
        let mut frame=super::super::tests::frame(w as u32,h as u32,sources);
        frame.map_id=crate::pokecom::ADMIN_OFFICE.into();frame.grid_origin=bevy::prelude::IVec2::ZERO;
        frame.source_map_size_core_tiles=Some(bevy::prelude::UVec2::new(32,32));
        let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let refs=frame.tiles.iter().collect::<Vec<_>>();
        let owners=super::super::modeled_interiors::resolve(&frame.map_id,&refs,&g,None);
        assert!(owners.iter().all(|p|p.indices(w).all(|i|
            !(49..59).contains(&(i%w)) || !(2..8).contains(&(i/w)))),"existing furniture must retain its source cells");
        let mut baseline=TerrainMeshData{footing_heights:vec![2.75;w*h],authored_cells:vec![None;w*h],..Default::default()};
        for y in 2..8 {for x in 49..59 {append_top(&mut baseline.textured,g.bounds(x,y).into(),0.,g.uv(x,y));}}
        let mut mesh=baseline.clone();finish(&mut mesh,&frame,&refs,&g,None);
        assert_eq!(mesh.footing_heights,baseline.footing_heights);
        assert_eq!(mesh.authored_cells.iter().filter(|&&p|p==Some("pokecom:native-communications-display")).count(),60);
        assert_eq!(mesh.textured.cutaway_ranges.len(),1);
        let body=mesh.textured.cutaway_ranges[0].clone();let mut volume=0.;let mut face=0.;let mut floor=0.;
        for tri in mesh.textured.indices.chunks_exact(3) {
            let[a,b,c]=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
            let n=Vec3::from_array(mesh.textured.normals[tri[0]as usize]);
            let cross=(b-a).cross(c-a);assert!(cross.dot(n)>0.,"all six sides keep outward winding");
            if body.contains(&(tri[0]as usize)) {volume+=a.dot(b.cross(c))/6.;}
            if n==Vec3::Z && a.z==64. && b.z==64. && c.z==64. && [a,b,c].iter().all(|p|p.y>=16.) {
                face+=cross.length()*0.5;
                let uv=tri.iter().map(|&i|bevy::prelude::Vec2::from_array(mesh.textured.uvs[i as usize])).sum::<bevy::prelude::Vec2>()/3.;
                let x=(uv.x*w as f32)as usize;let y=(uv.y*h as f32)as usize;
                assert!((49..59).contains(&x) && (2..8).contains(&y),"native screen/frame/bezel UVs only");
            }
            if n==Vec3::Y && [a,b,c].iter().all(|p|p.y==0.) {
                assert!(tri.iter().all(|&i|!body.contains(&(i as usize))),"floor does not fade with appliance");
                floor+=cross.length()*0.5;
            }
        }
        assert!((volume-(80.*8.*16.+80.*48.*4.)).abs()<1.,"both closed bodies: {volume}");
        assert_eq!(face,80.*48.);assert_eq!(floor,80.*32.);
        assert!(mesh.textured.positions.iter().enumerate().filter(|(i,_)|body.contains(i))
            .all(|(_,p)|p[2]>=56.),"the physical display and plinth stay inside the rear wall at z=32");
        assert_eq!(mesh.textured.positions.iter().map(|p|p[1]).fold(0.,f32::max),64.);
        let png=image::load_from_memory(&pack.runtime_files()["gfx/tilesets/pokecom_center.png"]).unwrap().to_luma8();
        let source_tile=(134&127)+png.width()*png.height()/64/2;
        assert_eq!(png.get_pixel(source_tile%16*8+2,source_tile/16*8+2)[0],170);
        assert_eq!(pack.data().tilesets["pokecom_center"].palette_map[134]&7,3,"blue chassis paint");
        for y in 2..8 {for x in 49..59 {
            let mut altered=frame.clone();altered.tiles[y*w+x].source.tile_index^=128;
            let mut out=baseline.clone();finish(&mut out,&altered,&altered.tiles.iter().collect::<Vec<_>>(),&g,None);
            assert_eq!(out,baseline,"incomplete source must retain the original complete picture");
        }}
        let mut altered=frame.clone();altered.tiles[8*w+47].source.tile_index=1;
        let mut out=baseline.clone();finish(&mut out,&altered,&altered.tiles.iter().collect::<Vec<_>>(),&g,None);
        assert_eq!(out,baseline,"missing floor donor rejects without partial writes");
        let mut reserved=baseline.clone();reserved.authored_cells[2*w+49]=Some("reserved");let mut out=reserved.clone();
        finish(&mut out,&frame,&refs,&g,None);assert_eq!(out,reserved);
        let custom:crate::live_profiles::Document=serde_json::from_str(r#"{"objects":[{"name":"Custom display","map":"PokecomCenterAdminOfficeMobile","tileset":"pokecom_center","metatile":41,"origin":[0,0],"tiles":[[142]],"ground":17,"top_pixels":0,"depth_pixels":8}]}"#).unwrap();
        let mut out=baseline.clone();finish(&mut out,&frame,&refs,&g,Some(&custom));assert_eq!(out,baseline);
        let mut other=frame.clone();other.map_id="GoldenrodPokecenter1F".into();let mut out=baseline.clone();
        finish(&mut out,&other,&refs,&g,None);assert_eq!(out,baseline);
    }
}
