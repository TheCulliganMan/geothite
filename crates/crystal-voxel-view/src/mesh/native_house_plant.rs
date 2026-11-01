//! Complete house plant drawings: narrow green blades and their own gold pot.
use super::*;

pub(super) const PIXELS:[[usize;2];6]=[[7,0],[2,0],[0,0],[4,25],[4,25],[6,23]];

pub(super) fn source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    if !reviewed_native_room(map)||p.kind!=ModelKind::PottedPlant||p.width!=2||p.height!=4
        ||p.base_pixels!=0.||g.width.checked_mul(g.height)!=Some(cells.len())
        ||p.column+2>g.width||p.row+4>g.height {return false;}
    let first=&cells[p.row*g.width+p.column].source;
    let origin=match first.metatile_id {6=>0,7=>2,_=>return false};
    p.indices(g.width).enumerate().all(|(j,i)| {let s=&cells[i].source;
        s.tileset_id.as_ref()=="house"&&s.metatile_id==first.metatile_id
            &&s.subtile_column==origin+(j%2)as u8&&s.subtile_row==(j/2)as u8
            &&s.tile_index==[10,11,8,9,26,27,24,25][j]
    })
}

fn face(out:&mut SurfaceMeshData,mut p:[Vec3;4],outside:Vec3,uv:[f32;2]) {
    let mut normal=(p[1]-p[0]).cross(p[2]-p[0]).normalize();
    if normal.dot(outside)<0. {p.swap(1,3);normal=-normal;}
    append_quad(out,p.map(|v|v.to_array()),normal.to_array(),[uv;4],
        crate::interior_models::authored_face_color([1.;4],normal));
}

fn blade(out:&mut SurfaceMeshData,root:Vec3,tip:Vec3,curve:Vec3,width:f32,paint:[[f32;2];3]) {
    let axis=(tip-root).normalize();let across=Vec3::new(-axis.z,0.,axis.x).normalize();
    let vertical=across.cross(axis).normalize();
    let mut rings=Vec::new();let mut centers=Vec::new();
    for (j,t) in [0.,1./3.,2./3.,1.].into_iter().enumerate() {
        let center=root*(1.-t)*(1.-t)+curve*(2.*t*(1.-t))+tip*t*t;
        let half=width*[0.08,0.5,0.36,0.025][j];let depth=width*0.065*[0.5,1.,0.7,0.02][j];
        centers.push(center);
        rings.push([center-across*half,center+vertical*depth,center+across*half,center-vertical*depth]);
    }
    face(out,rings[0],-axis,paint[1]);face(out,rings[3],axis,paint[0]);
    for j in 0..3 {for side in 0..4 {
        let next=(side+1)%4;
        let p=[rings[j][side],rings[j+1][side],rings[j+1][next],rings[j][next]];
        let middle=(centers[j]+centers[j+1])*0.5;
        let outside=(p[0]+p[1]+p[2]+p[3])*0.25-middle;
        face(out,p,outside,paint[if side==0 {2}else if side==3 {1}else{0}]);
    }}
}

pub(super) fn append(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,b:[f32;4],selected:&crate::interior_models::Model) {
    let height=p.height_pixels*g.tile_height/8.;let scale=height/23.;
    let paint=native_source_swatches(p,g,PIXELS);
    assert!(selected.append_native_plant_pot(out,b,height,paint));
    let [w,e,n,s]=b;let cx=(w+e)*0.5;let cz=(n+s)*0.5;
    let first=out.positions.len();
    architecture_box(out,[cx-0.23*scale,cx+0.23*scale,height*0.40,height*0.84,
        cz-0.23*scale,cz+0.23*scale],[1.;3]);
    out.uvs[first..].fill(source_texel(p,g,[8,17]));
    let rx=((e-w)*0.42).min(6.4*scale);let rz=((s-n)*0.42).min(5.*scale);
    for tier in 0..3 {for leaf in 0..9 {
        let angle=leaf as f32*std::f32::consts::TAU/9.+tier as f32*0.53;
        let direction=Vec3::new(angle.cos()*rx,0.,angle.sin()*rz);
        let root=Vec3::new(cx,(21.3-tier as f32*1.5)*scale,cz);
        let tip=Vec3::new(cx,(17.-tier as f32*1.8)*scale,cz)+direction;
        let curve=root+direction*0.62+Vec3::Y*1.1*scale;
        blade(out,root,tip,curve,1.0*scale,[paint[0],paint[1],paint[2]]);
    }}
    for leaf in 0..3 {
        let angle=leaf as f32*std::f32::consts::TAU/3.;
        let root=Vec3::new(cx,18.5*scale,cz);
        let tip=Vec3::new(cx+angle.cos()*2.*scale,22.99*scale,cz+angle.sin()*2.*scale);
        blade(out,root,tip,(root+tip)*0.5+Vec3::Y*scale,0.85*scale,[paint[0],paint[1],paint[2]]);
    }
}

fn source_texel(p:&Placement,g:&GridGeometry,pixel:[usize;2])->[f32;2] {
    native_source_swatches(p,g,[pixel])[0]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn slender_blades_have_closed_positive_volume_and_consistent_normals() {
        for scale in [0.5,1.,2.] {
            let mut mesh=SurfaceMeshData::default();
            blade(&mut mesh,Vec3::ZERO,Vec3::new(6.,-4.,0.)*scale,
                Vec3::new(3.,2.,0.)*scale,0.9*scale,[[0.1,0.2],[0.3,0.4],[0.5,0.6]]);
            let mut volume=0.;
            for tri in mesh.indices.chunks_exact(3) {
                let [a,b,c]=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
                assert!((b-a).cross(c-a).dot(Vec3::from_array(mesh.normals[tri[0]as usize]))>0.);
                volume+=a.dot(b.cross(c))/6.;
            }
            assert!(volume>0.01*scale.powi(3));
            assert!(mesh.normals.iter().all(|n|(Vec3::from_array(*n).length()-1.).abs()<0.0001));
        }
    }
    #[test]
    fn external_house_plants_use_own_green_gold_pixels_complete_claims_and_custom_ownership() {
        let explicit=std::env::var_os("CRYSTAL_RENDER_TEST_PACK");
        let path=explicit.as_ref().map(std::path::PathBuf::from).unwrap_or_else(||
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content-packs/text-tui.crystalpack"));
        assert!(explicit.is_none()||path.is_file());if !path.is_file(){return;}
        let pack=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let layout=&pack.pack().runtime_files()["data/tilesets/house_metatiles.bin"];
        let image=image::load_from_memory(&pack.pack().runtime_files()["gfx/tilesets/house.png"]).unwrap().to_luma8();
        let palette=&pack.pack().data().tilesets["house"].palette_map;
        let colors=std::str::from_utf8(&pack.pack().runtime_files()["gfx/tilesets/house.pal"]).unwrap().lines()
            .filter_map(|line|line.trim().strip_prefix("RGB ")).map(|line|line.split(',')
                .map(|v|v.trim().parse::<u8>().unwrap()).collect::<Vec<_>>()).collect::<Vec<_>>();
        let mut maps=std::collections::BTreeSet::new();let mut count=0;
        for (map,m) in &pack.pack().data().maps {
            if m.attributes.tileset_name!="house"{continue;}
            let (w,h)=(m.attributes.width as usize*4,m.attributes.height as usize*4);
            let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
            let mut tiles=Vec::new();
            for y in 0..h {for x in 0..w {
                let block=m.blocks[y/4*(w/4)+x/4];
                tiles.push(VisualTile{column:x as u32,row:y as u32,texture:Default::default(),priority:false,animation_frames:None,
                    source:VisualTileSource{tileset_id:"house".into(),metatile_id:block,subtile_column:(x%4)as u8,
                        subtile_row:(y%4)as u8,tile_index:layout[block as usize*16+y%4*4+x%4]as u16}});
            }}
            let refs=tiles.iter().collect::<Vec<_>>();
            for p in resolve(map,&refs,&g,None).iter().filter(|p|source(map,p,&refs,&g)) {
                maps.insert(map.clone());count+=1;
                assert!(!source("UnknownHouse",p,&refs,&g));
                for (j,pixel) in PIXELS.into_iter().enumerate() {
                    let raw=refs[(p.row+pixel[1]/8)*w+p.column+pixel[0]/8].source.tile_index as usize;
                    let packed=palette[raw];assert_eq!(packed&7,if j<3{2}else{6});
                    let total=image.width()/8*(image.height()/8);
                    let index=if packed&8!=0 {(raw&127)+total as usize/2}else{raw};
                    let gray=image.get_pixel((index%16*8+pixel[0]%8)as u32,(index/16*8+pixel[1]%8)as u32).0[0];
                    assert_eq!(gray,[85,0,170,85,85,0][j]);
                    assert_eq!(colors[usize::from(packed&7)*4+usize::from((255-gray)/85)],
                        [[9,13,0],[7,7,7],[15,20,1],[16,13,3],[16,13,3],[7,7,7]][j],
                        "pot/leaf pigment, not the surrounding pink checkerboard");
                }
                let mut mesh=TerrainMeshData{footing_heights:vec![2.75;w*h],..Default::default()};
                let mut claimed=vec![false;w*h];
                assert!(super::super::append(&mut mesh,map,&refs,&g,p,&mut claimed,[0,0]));
                assert!(mesh.solid.indices.is_empty());assert_eq!(mesh.footing_heights,vec![2.75;w*h]);
                assert_eq!(claimed.iter().filter(|&&v|v).count(),8);
                assert!(mesh.textured.positions.iter().all(|v|v[1]>=0.&&v[1]<=p.height_pixels+0.001));
                for i in p.indices(w) {
                    let mut changed=tiles.clone();changed[i].source.tile_index=0xffff;
                    assert!(!source(map,p,&changed.iter().collect::<Vec<_>>(),&g));
                }
                let first=&refs[p.row*w+p.column].source;
                let object=Object{name:"User house plant".into(),tileset:"house".into(),map:Some(map.clone()),maps:None,
                    metatile:first.metatile_id,metatiles:None,origin:[first.subtile_column,0],
                    tiles:vec![vec![10,11],vec![8,9],vec![26,27],vec![24,25]],ground:1,top_pixels:0,
                    depth_pixels:0.,footing_pixels:None,mask:Default::default(),parts:vec![]};
                assert!(!resolve(map,&refs,&g,Some(&Document{objects:vec![object],atmosphere:None})).iter().any(|q|
                    q.kind==p.kind&&q.column==p.column&&q.row==p.row));
            }
        }
        assert!(maps.contains("BillsHouse")&&maps.contains("DayCare")&&maps.contains("TrainerHouse1F"));
        println!("Native house plant coverage: {count} placements in {} maps: {}",maps.len(),maps.into_iter().collect::<Vec<_>>().join(","));
    }
}
