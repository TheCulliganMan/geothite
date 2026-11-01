//! Complete communications console: deck, recessed electronics, sides and controls.
use super::*;
const BLOCKS:[[u16;3];5]=[[0x3e,0x38,0x3f],[0x1e,0x28,0x1f],[0x18,0x25,0x19],
    [0x1c,0x24,0x1d],[0x20,0x2b,0x21]];
fn owns(_x:usize,y:usize)->bool {y<16}
pub(super) fn finish(mesh:&mut TerrainMeshData,frame:&VisualWorldFrame,cells:&[&VisualTile],
    g:&GridGeometry,profiles:Option<&crate::live_profiles::Document>) {
    if frame.map_id.as_ref()!=crate::pokecom::ADMIN_OFFICE
        || frame.source_map_size_core_tiles!=Some(bevy::prelude::UVec2::new(32,32))
        || cells.len()!=g.width*g.height || mesh.authored_cells.len()!=cells.len() {return;}
    let(x,y)=(48-frame.grid_origin.x,8-frame.grid_origin.y);
    if x<1 || y<0 || x as usize+12>g.width || y as usize+18>g.height {return;}
    let(x,y)=(x as usize,y as usize);let donor=y*g.width+x-1;let d=&cells[donor].source;
    if d.tileset_id.as_ref()!="pokecom_center" || d.metatile_id!=1 || d.tile_index!=17
        || d.subtile_column!=3 || d.subtile_row!=0 {return;}
    let mut fingerprint=0xcbf29ce484222325u64;
    for row in 0..18 {for column in 0..12 {
        let i=(y+row)*g.width+x+column;let s=&cells[i].source;
        if s.tileset_id.as_ref()!="pokecom_center" || s.metatile_id!=BLOCKS[row/4][column/4]
            || s.subtile_column as usize!=column%4 || s.subtile_row as usize!=row%4
            || mesh.authored_cells[i].is_some() {return;}
        fingerprint=(fingerprint^u64::from(s.tile_index)).wrapping_mul(0x100000001b3);
    }}
    if fingerprint!=0x37ab5aa1f36bcc24 {return;}
    if live::resolve(cells,g.width,g.height,&frame.map_id,profiles).iter().any(|p|
        p.indices(g.width).any(|i|i==donor || ((x..x+12).contains(&(i%g.width))
            && (y..y+18).contains(&(i/g.width))))) {return;}
    // The projected side/front/shadow ink is replaced as a complete drawing.
    // The apron receives floor, never object ownership or raised support.
    for out in [&mut mesh.textured,&mut mesh.solid,&mut mesh.animated_textured,&mut mesh.animated_solid] {
        out.indices=out.indices.chunks_exact(3).filter(|tri| {
            let p=tri.iter().map(|&i|Vec3::from_array(out.positions[i as usize])).sum::<Vec3>()/3.;
            let cx=(p.x-g.origin_x)/g.tile_width;let cy=(p.z-g.origin_z)/g.tile_height;
            !((x as f32..(x+12)as f32).contains(&cx) && (y as f32..(y+18)as f32).contains(&cy)
                && tri.iter().all(|&i|out.positions[i as usize][1].abs()<0.00001 && out.normals[i as usize][1]>0.9))
        }).flatten().copied().collect();
    }
    let floor=g.uv(donor%g.width,donor/g.width);
    for row in 0..18 {for column in 0..12 {append_top(&mut mesh.textured,
        g.bounds(x+column,y+row).into(),0.,floor);}}
    let scale=g.tile_height/8.;let(w,_,n,_)=g.bounds(x+2,y);
    let e=w+8.*g.tile_width;let shoulder=n+12.*g.tile_height;let control_start=shoulder+2.*g.tile_height;let front=control_start+2.*g.tile_height;
    let high=16.*scale;let low=8.*scale;let uv=g.uv(x+3,y+2);
    // This native blue field excludes the black electronic details.
    let paint=[uv.0+(uv.1-uv.0)*2.5/8.,uv.2+(uv.3-uv.2)*2.5/8.];
    let start=mesh.textured.positions.len();
    for row in 0..12 {for column in 2..10 {
        let height=if (5..7).contains(&column) && (2..11).contains(&row) {12.*scale}else{high};
        append_top(&mut mesh.textured,g.bounds(x+column,y+row).into(),height,g.uv(x+column,y+row));
    }}
    // Blue shoulders bridge the former projected front-course footprint.
    append_top(&mut mesh.textured,[w,e,shoulder,control_start],high,(paint[0],paint[0],paint[1],paint[1]));
    let outer_w=w-2.*g.tile_width;let outer_e=e+2.*g.tile_width;
    // The two side columns form beveled cheeks, meeting the native footprint
    // at ground level while keeping the original top-deck width.
    for row in 0..12 {for course in 0..2 {
        let(_,_,north,south)=g.bounds(x,y+row);let bottom=course as f32*8.*scale;let top=bottom+8.*scale;
        let l=g.uv(x+course,y+row);let west=outer_w+course as f32*g.tile_width;let east=west+g.tile_width;
        append_quad(&mut mesh.textured,[[west,bottom,north],[west,bottom,south],[east,top,south],[east,top,north]],
            [-std::f32::consts::FRAC_1_SQRT_2,std::f32::consts::FRAC_1_SQRT_2,0.],[[l.0,l.2],[l.0,l.3],[l.1,l.3],[l.1,l.2]],TEXTURED_SHADE);
        let r=g.uv(x+11-course,y+row);let east=outer_e-course as f32*g.tile_width;let west=east-g.tile_width;
        append_quad(&mut mesh.textured,[[east,bottom,south],[east,bottom,north],[west,top,north],[west,top,south]],
            [std::f32::consts::FRAC_1_SQRT_2,std::f32::consts::FRAC_1_SQRT_2,0.],[[r.1,r.3],[r.1,r.2],[r.0,r.2],[r.0,r.3]],TEXTURED_SHADE);
    }}
    for column in 2..10 {for row in 0..2 {
        let(left,right,_,_)=g.bounds(x+column,y+12+row);
        let top=g.uv(x+column,y+12+row);let face=g.uv(x+column,y+14+row);
        let north=control_start+row as f32*g.tile_height;let south=north+g.tile_height;
        let upper=high-row as f32*4.*scale;let lower=upper-4.*scale;
        append_quad(&mut mesh.textured,[[left,upper,north],[left,lower,south],[right,lower,south],[right,upper,north]],
            [0.,2./5f32.sqrt(),1./5f32.sqrt()],
            [[top.0,top.2],[top.0,top.3],[top.1,top.3],[top.1,top.2]],TEXTURED_SHADE);
        let upper=low-row as f32*4.*scale;let lower=upper-4.*scale;
        append_quad(&mut mesh.textured,[[left,lower,front],[right,lower,front],[right,upper,front],[left,upper,front]],
            [0.,0.,1.],[[face.0,face.3],[face.1,face.3],[face.1,face.2],[face.0,face.2]],TEXTURED_SHADE);
    }}
    append_quad(&mut mesh.textured,[[outer_e,0.,n],[outer_w,0.,n],[w,high,n],[e,high,n]],
        [0.,0.,-1.],[paint;4],TEXTURED_SHADE);
    append_quad(&mut mesh.textured,[[outer_w,0.,n],[outer_e,0.,n],[outer_e,0.,front],[outer_w,0.,front]],
        [0.,-1.,0.],[paint;4],TEXTURED_SHADE);
    for (north,south,upper,lower) in [(shoulder,control_start,high,high),(control_start,front,high,low)] {
        for q in [
            [[outer_w,0.,north],[outer_w,0.,south],[w,lower,south],[w,upper,north]],
            [[outer_e,0.,north],[e,upper,north],[e,lower,south],[outer_e,0.,south]],
        ] {
            triangle(&mut mesh.textured,[q[0],q[1],q[2]],paint);
            triangle(&mut mesh.textured,[q[0],q[2],q[3]],paint);
        }
    }
    triangle(&mut mesh.textured,[[outer_w,0.,front],[w,0.,front],[w,low,front]],paint);
    triangle(&mut mesh.textured,[[e,0.,front],[outer_e,0.,front],[e,low,front]],paint);
    // Closed shallow well for the source electronics; its surrounding deck
    // remains at sixteen, while its native circuit art sits four pixels below.
    let a=w+3.*g.tile_width;let b=a+2.*g.tile_width;
    let c=n+2.*g.tile_height;let d=c+9.*g.tile_height;let bottom=12.*scale;
    for(points,normal)in[
        ([[a,bottom,c],[a,bottom,d],[a,high,d],[a,high,c]],[1.,0.,0.]),
        ([[b,bottom,d],[b,bottom,c],[b,high,c],[b,high,d]],[-1.,0.,0.]),
        ([[b,bottom,c],[a,bottom,c],[a,high,c],[b,high,c]],[0.,0.,1.]),
        ([[a,bottom,d],[b,bottom,d],[b,high,d],[a,high,d]],[0.,0.,-1.]),
    ] {
        // Reverse the outer-body ordering for walls facing into the recess.
        append_quad(&mut mesh.textured,[points[1],points[0],points[3],points[2]],normal,[paint;4],TEXTURED_SHADE);
    }
    mesh.textured.cutaway_ranges.push(start..mesh.textured.positions.len());
    for row in 0..16 {for column in 0..12 {if owns(column,row) {
        mesh.authored_cells[(y+row)*g.width+x+column]=Some("pokecom:native-communications-console");
    }}}
}

fn triangle(out:&mut SurfaceMeshData,p:[[f32;3];3],paint:[f32;2]) {
    let[a,b,c]=p.map(Vec3::from_array);let normal=(b-a).cross(c-a).normalize().to_array();
    let base=out.positions.len()as u32;out.positions.extend(p);out.normals.extend([normal;3]);
    out.uvs.extend([paint;3]);out.colors.extend([TEXTURED_SHADE;3]);out.indices.extend([base,base+1,base+2]);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_pokecom_console_folds_controls_and_sides_closes_recess_and_preserves_apron() {
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
            !(48..60).contains(&(i%w)) || !(8..26).contains(&(i/w)))),"no other model owns this source assembly");
        let mut baseline=TerrainMeshData{footing_heights:vec![2.75;w*h],authored_cells:vec![None;w*h],..Default::default()};
        for y in 8..26 {for x in 48..60 {
            assert!(matches!(shape_for_source_on_map(&frame.map_id,&refs[y*w+x].source),CellShape::Flat),
                "the previous console is horizontal art, not another wall model");
            append_top(&mut baseline.textured,g.bounds(x,y).into(),0.,g.uv(x,y));
        }}
        let mut mesh=baseline.clone();finish(&mut mesh,&frame,&refs,&g,None);
        assert_eq!(mesh.footing_heights,baseline.footing_heights);
        assert_eq!(mesh.authored_cells.iter().filter(|&&p|p==Some("pokecom:native-communications-console")).count(),192);
        assert_eq!(mesh.textured.cutaway_ranges.len(),1);let body=mesh.textured.cutaway_ranges[0].clone();
        let(mut volume,mut floor,mut slots,mut controls,mut well,mut deck)=(0.,0.,0.,0.,0.,0.);
        for tri in mesh.textured.indices.chunks_exact(3) {
            let[a,b,c]=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
            let n=Vec3::from_array(mesh.textured.normals[tri[0]as usize]);let cross=(b-a).cross(c-a);
            assert!(cross.dot(n)>0.,"closed console and inward well winding {n:?}");
            let area=cross.length()*0.5;let center=(a+b+c)/3.;
            if body.contains(&(tri[0]as usize)) {volume+=a.dot(b.cross(c))/6.;}
            else {assert_eq!(n,Vec3::Y);assert!([a,b,c].iter().all(|p|p.y==0.));floor+=area;}
            if n==Vec3::Z && center.z==192. && (400.0..464.0).contains(&center.x) {slots+=area;}
            if n.y>0.8 && n.z>0.4 && (400.0..464.0).contains(&center.x) {controls+=area;}
            if n==Vec3::Y && center.y==12. {well+=area;}
            if n==Vec3::Y && center.y==16. {deck+=area;}
            if n.z>0.4 && center.z>=176. && (400.0..464.0).contains(&center.x) && body.contains(&(tri[0]as usize)) {
                let uv=tri.iter().map(|&i|bevy::prelude::Vec2::from_array(mesh.textured.uvs[i as usize])).sum::<bevy::prelude::Vec2>()/3.;
                let sx=(uv.x*w as f32)as usize;let sy=(uv.y*h as f32)as usize;
                assert!((50..58).contains(&sx));
                assert_eq!(sy,if n.y>0.8 {20+((16.-center.y)/4.).floor()as usize}
                    else{22+((8.-center.y)/4.).floor()as usize},"controls are on the sloping top; slots face forward");
            }
        }
        assert!((volume-461_312./3.).abs()<1.,"closed body less the recessed well: {volume}");
        assert_eq!(floor,216.*64.);assert_eq!(slots,512.);
        assert!((controls-64.*(320f32).sqrt()).abs()<0.01);
        assert_eq!(well,1152.);assert_eq!(deck,6016.);
        for y in 8..26 {for x in 48..60 {
            assert_eq!(mesh.authored_cells[y*w+x].is_some(),owns(x-48,y-8),"apron never earns object claims");
            let mut altered=frame.clone();altered.tiles[y*w+x].source.tile_index^=128;
            let mut out=baseline.clone();finish(&mut out,&altered,&altered.tiles.iter().collect::<Vec<_>>(),&g,None);
            assert_eq!(out,baseline,"a damaged native drawing retains the complete source assembly");
        }}
        let png=image::load_from_memory(&pack.runtime_files()["gfx/tilesets/pokecom_center.png"]).unwrap().to_luma8();
        assert_eq!(pack.data().tilesets["pokecom_center"].palette_map[143]&7,3);
        let bank_tiles=png.width()*png.height()/64/2;
        let source_tile=(143&127)+bank_tiles;
        assert_eq!(png.get_pixel(source_tile%16*8+2,source_tile/16*8+2)[0],170,
            "bank-one blue casing excludes black electronics");
        let mut reserved=baseline.clone();reserved.authored_cells[8*w+48]=Some("reserved");let mut out=reserved.clone();
        finish(&mut out,&frame,&refs,&g,None);assert_eq!(out,reserved);
        let mut altered=frame.clone();altered.tiles[8*w+47].source.tile_index=1;let mut out=baseline.clone();
        finish(&mut out,&altered,&altered.tiles.iter().collect::<Vec<_>>(),&g,None);assert_eq!(out,baseline);
        let custom:crate::live_profiles::Document=serde_json::from_str(r#"{"objects":[{"name":"Custom console","map":"PokecomCenterAdminOfficeMobile","tileset":"pokecom_center","metatile":40,"origin":[0,0],"tiles":[[182]],"ground":17,"top_pixels":0,"depth_pixels":8}]}"#).unwrap();
        let mut out=baseline.clone();finish(&mut out,&frame,&refs,&g,Some(&custom));assert_eq!(out,baseline);
        let mut combined=baseline.clone();super::super::pokecom_display::finish(&mut combined,&frame,&refs,&g,None);
        finish(&mut combined,&frame,&refs,&g,None);
        assert_eq!(combined.authored_cells.iter().filter(|&&v|v==Some("pokecom:native-communications-display")).count(),60);
        assert_eq!(combined.authored_cells.iter().filter(|&&v|v==Some("pokecom:native-communications-console")).count(),192);
        assert_eq!(combined.footing_heights,baseline.footing_heights);
    }
}
