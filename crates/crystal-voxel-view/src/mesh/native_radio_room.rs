//! Complete Radio Tower wall and plant drawings, with native room finishes.
use super::*;

pub(super) fn is_map(map:&str)->bool {
    matches!(map,"RadioTower1F"|"RadioTower2F"|"RadioTower3F"|"RadioTower4F"|"RadioTower5F"|"LavRadioTower1F")
}

pub(super) fn matches(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    if !is_map(map)||p.width!=2||p.base_pixels!=0.||cells.len()!=g.width*g.height
        ||p.column+2>g.width||p.row+p.height>g.height {return false;}
    if p.kind==ModelKind::PottedPlant&&p.height==8 {
        return topiary_source(map,p,cells,g);
    }
    let first=&cells[p.row*g.width+p.column].source;
    if p.kind==ModelKind::PictureFrame&&wall_donor(cells,g,p.row).is_none(){return false;}
    let (origin,art):([u8;2],&[u16])=match (p.kind,p.height,first.metatile_id,first.subtile_column) {
        (ModelKind::WallAcoustic,1,0x11,0|2)=>([first.subtile_column,0],&[0x11,0x11]),
        (ModelKind::WallAcoustic,1,0x24,0)=>([0,0],&[0x11,0x11]),
        (ModelKind::WallAcoustic,1,0x12,0)=>([0,0],&[0x11,0x11]),
        (ModelKind::WallAcoustic,2,2|3|4|0x0d|0x31,0)=>([0,0],&[0x11,0x11,0x10,0x10]),
        (ModelKind::WallAcoustic,2,4|0x0c|0x12|0x1d,2)=>([2,0],&[0x11,0x11,0x10,0x10]),
        (ModelKind::PictureFrame,2,2,2)=>([2,0],&[3,4,0x13,0x14]),
        (ModelKind::PottedPlant,4,0x0b,0|2)
            =>([first.subtile_column,0],&[0x46,0x47,0x48,0x49,0x44,0x45,0x4a,0x4b]),
        (ModelKind::PottedPlant,4,0x31,2)
            =>([2,0],&[0x46,0x47,0x48,0x49,0x44,0x45,0x4a,0x4b]),
        (ModelKind::PottedPlant,4,0x2f,0|2)
            =>([first.subtile_column,0],&[0x80,0x81,0x82,0x83,0x44,0x45,0x4a,0x4b]),
        (ModelKind::PottedPlant,4,0x37,2)
            =>([2,0],&[0x80,0x81,0x82,0x83,0x44,0x45,0x4a,0x4b]),
        (ModelKind::PottedPlant,4,0x3c,0)
            =>([0,0],&[0x80,0x81,0x82,0x83,0x44,0x45,0x4a,0x4b]),
        (ModelKind::PottedPlant,4,0x1f,0)
            =>([0,0],&[0x88,0x89,0x82,0x83,0x44,0x45,0x4a,0x4b]),
        _=>return false,
    };
    p.indices(g.width).enumerate().all(|(j,i)| {
        let s=&cells[i].source;
        s.tileset_id.as_ref()=="radio_tower"&&s.metatile_id==first.metatile_id
            &&s.subtile_column==origin[0]+(j%2)as u8&&s.subtile_row==origin[1]+(j/2)as u8
            &&s.tile_index==art[j]
    })
}

// The tall west-edge topiary is one drawing crossing two source blocks.
// Authenticate all sixteen cells before claiming it, including the lower pot.
fn topiary_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    if map!="RadioTower2F"||p.width!=2||p.height!=8
        ||p.column+2>g.width||p.row+8>g.height||cells.len()!=g.width*g.height {return false;}
    let art=[0x80,0x81,0x82,0x83,0x84,0x85,0x86,0x87,
        0x88,0x89,0x82,0x83,0x44,0x45,0x4a,0x4b];
    p.indices(g.width).enumerate().all(|(j,i)| {
        let s=&cells[i].source;let y=j/2;
        s.tileset_id.as_ref()=="radio_tower"&&s.metatile_id==(if y<4 {0x30}else{0x1f})
            &&s.subtile_column==(j%2)as u8&&s.subtile_row==(y%4)as u8
            &&s.tile_index==art[j]
    })
}

pub(super) fn candidates(map:&str,cells:&[&VisualTile],g:&GridGeometry,claimed:&mut[bool],out:&mut Vec<Placement>) {
    if !is_map(map)||g.width<2||cells.len()!=g.width*g.height||claimed.len()!=cells.len(){return;}
    let Some(ground)=ground_sample(cells,map,"radio_tower",1)else{return;};
    for (kind,height,depth,rise) in [(ModelKind::PottedPlant,8,12.,56.),(ModelKind::WallAcoustic,2,3.,32.),(ModelKind::WallAcoustic,1,3.,32.),
        (ModelKind::PictureFrame,2,3.,32.),(ModelKind::PottedPlant,4,12.,24.)] {
        if height>g.height {continue;}
        for row in 0..=g.height-height {for column in 0..=g.width-2 {
            let p=Placement{kind,column,row,width:2,height,ground,depth_pixels:depth,height_pixels:rise,
                base_pixels:0.,footing_pixels:None};
            if p.indices(g.width).any(|i|claimed[i])||!matches(map,&p,cells,g){continue;}
            for i in p.indices(g.width){claimed[i]=true;}out.push(p);
        }}
    }
}

fn wall_donor(cells:&[&VisualTile],g:&GridGeometry,row:usize)->Option<usize> {
    (row*g.width..(row+1)*g.width).find(|&i| {
        let s=&cells[i].source;
        s.tileset_id.as_ref()=="radio_tower"&&s.subtile_row==0&&s.tile_index==0x11
            &&matches!(s.metatile_id,2|3|4|0x0c|0x0d|0x11|0x12|0x1d|0x24|0x31)
    })
}

fn backing(out:&mut SurfaceMeshData,g:&GridGeometry,donor:usize,b:[f32;4]) {
    let uv=g.uv(donor%g.width,donor/g.width);let scale=g.tile_height/8.;
    let first=out.positions.len();
    architecture_box(out,[b[0],b[1],0.,32.*scale,b[2],b[3]],[1.;3]);
    out.uvs[first..].fill([uv.0+(uv.1-uv.0)*0.5/8.,uv.2+(uv.3-uv.2)*0.5/8.]);
}

pub(super) fn bookcase_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    if !is_map(map)||p.kind!=ModelKind::Bookcase||p.width!=2||p.height!=4
        ||p.column+2>g.width||p.row+4>g.height||cells.len()!=g.width*g.height {return false;}
    let first=&cells[p.row*g.width+p.column].source;
    if !matches!((first.metatile_id,first.subtile_column),(0x0a,0|2)|(0x24,2)){return false;}
    p.indices(g.width).enumerate().all(|(j,i)| {
        let s=&cells[i].source;
        s.tileset_id.as_ref()=="radio_tower"&&s.metatile_id==first.metatile_id
            &&s.subtile_column==first.subtile_column+(j%2)as u8&&s.subtile_row==(j/2)as u8
            &&s.tile_index==[0x30,0x31,0x1a,0x1b,0x1a,0x1b,0x0a,0x0b][j]
    })
}

pub(super) fn chair_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    if !is_map(map)||p.kind!=ModelKind::Chair||p.width!=2||p.height!=2
        ||p.column+2>g.width||p.row+2>g.height||cells.len()!=g.width*g.height{return false;}
    let first=&cells[p.row*g.width+p.column].source;
    if !matches!((first.metatile_id,first.subtile_column),(0x11,0|2)|(0x24,0)){return false;}
    p.indices(g.width).enumerate().all(|(j,i)| {
        let s=&cells[i].source;
        s.tileset_id.as_ref()=="radio_tower"&&s.metatile_id==first.metatile_id
            &&s.subtile_column==first.subtile_column+(j%2)as u8&&s.subtile_row==1+(j/2)as u8
            &&s.tile_index==[0x52,if first.metatile_id==0x11&&first.subtile_column==2 {0x34}else{0x53},0x50,0x51][j]
    })
}

pub(super) fn stool_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    if !is_map(map)||p.kind!=ModelKind::ArcadeStool||p.width!=2||p.height!=2||p.base_pixels!=0.
        ||p.column+2>g.width||p.row+2>g.height||cells.len()!=g.width*g.height{return false;}
    let first=&cells[p.row*g.width+p.column].source;
    if !matches!(first.metatile_id,0x14|0x2c|0x2d|0x39|0x3a){return false;}
    p.indices(g.width).enumerate().all(|(j,i)| {
        let s=&cells[i].source;
        s.tileset_id.as_ref()=="radio_tower"&&s.metatile_id==first.metatile_id
            &&s.subtile_column==(j%2)as u8&&s.subtile_row==(j/2)as u8
            &&s.tile_index==[0x2c,0x2d,0x3c,0x3d][j]
    })
}

pub(super) fn append_bookcase_backing(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,
    cells:&[&VisualTile],origin:[i32;2],map:&str) {
    if p.row as i32+origin[1]!=0||!bookcase_source(map,p,cells,g){return;}
    if let Some(donor)=wall_donor(cells,g,p.row) {
        let (w,_,_,s)=g.bounds(p.column,p.row+1);let scale=g.tile_height/8.;
        backing(out,g,donor,[w,w+2.*g.tile_width,s-3.*scale,s]);
    }
}

pub(super) fn append(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,cells:&[&VisualTile],
    b:[f32;4],selected:&crate::interior_models::Model,origin:[i32;2]) {
    let [w,e,_,s]=b;let scale=g.tile_height/8.;
    if p.kind==ModelKind::PottedPlant {
        if p.height==8 {
            assert!(selected.append_native_facility_topiary(out,b,scale,
                native_source_swatches(p,g,[[3,51],[2,51],[4,51],[3,57],[3,58],[4,58]])));
        } else {
            assert!(selected.append_native_facility_plant(out,b,p.height_pixels*scale,
                // The common lower crown provides stable green swatches even
                // when the upper source slots are supplied by the live renderer.
                native_source_swatches(p,g,[[3,19],[2,19],[4,19],[3,25],[3,26],[4,26]])));
        }
        // Plants on the north perimeter stand in front of a continuous wall.
        // Interior plants keep their own free-standing footprint.
        if p.height==4&&p.row as i32+origin[1]==0 {
            if let Some(donor)=wall_donor(cells,g,p.row) {
                let south=g.origin_z+(p.row+2)as f32*g.tile_height;
                let west=g.bounds(p.column,p.row).0;
                backing(out,g,donor,[west,west+2.*g.tile_width,south-3.*scale,south]);
            }
        }
    } else if let Some(donor)=wall_donor(cells,g,p.row) {
        backing(out,g,donor,b);
        if p.kind==ModelKind::PictureFrame {
            // Keep the complete sign at its native16px size, high on the wall.
            for y in 0..2 {for x in 0..2 {
                let uv=g.uv(p.column+x,p.row+y);
                let a=w+x as f32*(e-w)/2.;let z=a+(e-w)/2.;
                let top=(31.-y as f32*8.)*scale;let bottom=top-8.*scale;
                append_quad(out,[[z,bottom,s+0.01*scale],[z,top,s+0.01*scale],
                    [a,top,s+0.01*scale],[a,bottom,s+0.01*scale]],Vec3::Z.to_array(),
                    [[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2],[uv.0,uv.3]],TEXTURED_SHADE);
            }}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_radio_rooms_keep_complete_plants_wall_backing_floor_and_custom_ownership() {
        let explicit=std::env::var_os("CRYSTAL_RENDER_TEST_PACK");
        let path=explicit.as_ref().map(std::path::PathBuf::from).unwrap_or_else(||
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content-packs/text-tui.crystalpack"));
        assert!(explicit.is_none()||path.is_file());if !path.is_file(){return;}
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();let pack=loaded.pack();
        let art=&pack.runtime_files()["data/tilesets/radio_tower_metatiles.bin"];
        for name in ["RadioTower1F","RadioTower2F","RadioTower3F","RadioTower4F","RadioTower5F","LavRadioTower1F"] {
            let data=&pack.data().maps[name];let (w,h)=(data.attributes.width as usize*4,data.attributes.height as usize*4);
            let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
            let t=(0..w*h).map(|i| {
                let (x,y)=(i%w,i/w);let block=data.blocks[y/4*(w/4)+x/4];
                VisualTile{column:x as u32,row:y as u32,texture:Default::default(),priority:false,animation_frames:None,
                    source:VisualTileSource{tileset_id:"radio_tower".into(),metatile_id:block,
                        subtile_column:(x%4)as u8,subtile_row:(y%4)as u8,
                        tile_index:art[block as usize*16+y%4*4+x%4]as u16}}
            }).collect::<Vec<_>>();
            let refs=t.iter().collect::<Vec<_>>();let p=resolve(name,&refs,&g,None);
            let plants=p.iter().filter(|p|p.kind==ModelKind::PottedPlant&&matches(name,p,&refs,&g)).collect::<Vec<_>>();
            let expected=data.blocks.iter().map(|&b|match b {0x0b|0x2f=>2,0x31|0x1f|0x37|0x3c=>1,_=>0}).sum::<usize>();
            assert_eq!(plants.len(),expected,"{name} must preserve every full plant drawing");
            if name=="RadioTower2F" {
                let topiary=plants.iter().find(|p|p.height==8).expect("cross-block topiary");
                assert_eq!((topiary.column,topiary.row,topiary.height_pixels),(0,8,56.));
                assert!(!p.iter().any(|q|q.height==4&&q.column==0&&q.row==12),
                    "one topiary must own its lower pot, without a duplicate plant");
                for bad in topiary.indices(w) {
                    let mut altered=t.clone();altered[bad].source.tile_index^=0x80;
                    let refs=altered.iter().collect::<Vec<_>>();
                    assert!(!resolve(name,&refs,&g,None).iter().any(|q|q.height==8&&q.kind==ModelKind::PottedPlant));
                }
                // Production pads short rooms to the LCD minimum and can
                // include surrounding cells; source identity must survive that.
                let pg=GridGeometry{width:w+4,height:h+4,..g};
                let mut padded=vec![t[0].clone();pg.width*pg.height];
                for tile in &mut padded {tile.source.metatile_id=u16::MAX;}
                for y in 0..h {for x in 0..w {padded[(y+2)*pg.width+x+2]=t[y*w+x].clone();}}
                let pref=padded.iter().collect::<Vec<_>>();
                let mut shifted=(*topiary).clone();shifted.column+=2;shifted.row+=2;
                assert!(matches(name,&shifted,&pref,&pg),"padded live frame retains full drawing");
                assert!(!matches("RadioTower1F",&shifted,&pref,&pg));
                let mut custom_claimed=vec![false;w*h];custom_claimed[topiary.row*w]=true;
                let mut candidate=Vec::new();
                candidates(name,&refs,&g,&mut custom_claimed,&mut candidate);
                assert!(!candidate.iter().any(|q|q.height==8),"custom ownership wins");
                let mut topiary_mesh=TerrainMeshData{footing_heights:vec![2.5;w*h],..Default::default()};
                assert!(super::super::append(&mut topiary_mesh,name,&refs,&g,topiary,&mut vec![false;w*h],[0,0]));
                assert!(topiary_mesh.textured.positions.iter().any(|v|v[1]>50.));
                assert_eq!(topiary_mesh.footing_heights,vec![2.5;w*h]);
            }
            assert!(!p.iter().any(|p|p.kind==ModelKind::ArcadeStool&&refs[p.row*w+p.column].source.metatile_id==0x0b));
            let mut claimed=vec![false;w*h];let mut mesh=TerrainMeshData{footing_heights:vec![2.5;w*h],..Default::default()};
            for piece in p.iter().filter(|p|matches(name,p,&refs,&g)) {
                assert!(super::super::append(&mut mesh,name,&refs,&g,piece,&mut claimed,[0,0]));
                assert!(piece.indices(w).all(|i|claimed[i]));
            }
            assert_eq!(mesh.footing_heights,vec![2.5;w*h]);
            assert!(mesh.solid.indices.is_empty());
            assert!(mesh.textured.positions.iter().any(|v|v[1]>=32.));
            assert!(mesh.textured.normals.iter().all(|n|(Vec3::from_array(*n).length()-1.).abs()<0.0001));
            for picture in p.iter().filter(|p|p.kind==ModelKind::PictureFrame&&matches(name,p,&refs,&g)) {
                let mut sign=TerrainMeshData{footing_heights:vec![2.5;w*h],..Default::default()};
                assert!(super::super::append(&mut sign,name,&refs,&g,picture,&mut vec![false;w*h],[0,0]));
                for y in 0..2 {for x in 0..2 {
                    let uv=g.uv(picture.column+x,picture.row+y);
                    let count=sign.textured.indices.chunks_exact(3).filter(|tri| {
                        let q=[tri[0],tri[1],tri[2]].map(|i|sign.textured.uvs[i as usize]);
                        let area=(q[1][0]-q[0][0])*(q[2][1]-q[0][1])-(q[1][1]-q[0][1])*(q[2][0]-q[0][0]);
                        area.abs()>0.0000001&&tri.iter().all(|&i| {
                            let i=i as usize;let q=sign.textured.uvs[i];let v=sign.textured.positions[i];
                            sign.textured.normals[i][2]>0.5&&v[1]>=15.&&v[1]<=31.
                                &&q[0]>=uv.0&&q[0]<=uv.1&&q[1]>=uv.2&&q[1]<=uv.3
                        })
                    }).count();assert_eq!(count,2,"native radio art must not get a second generic overlay");
                }}
                assert_eq!(sign.footing_heights,vec![2.5;w*h]);
            }
            let seat_count=data.blocks.iter().map(|&b|match b {0x11=>2,0x24=>1,_=>0}).sum::<usize>();
            assert_eq!(p.iter().filter(|p|chair_source(name,p,&refs,&g)).count(),seat_count,
                "both native seat variants must stay in front of the rear wall");
            for piece in p.iter().filter(|p|chair_source(name,p,&refs,&g)) {
                let mut seat=TerrainMeshData{footing_heights:vec![2.5;w*h],..Default::default()};
                assert!(super::super::append(&mut seat,name,&refs,&g,piece,&mut vec![false;w*h],[0,0]));
                assert!(seat.textured.positions.iter().any(|v|v[1]>0.));
                let wall=g.bounds(piece.column,piece.row).3;
                assert!(seat.textured.positions.iter().filter(|v|v[1]>0.).all(|v|v[2]>=wall-0.001),"rear seat stays in front of its wall");
                let white=native_source_swatches(piece,&g,[[4,2]])[0];
                assert!(seat.textured.positions.iter().zip(&seat.textured.normals).zip(&seat.textured.uvs)
                    .filter(|((v,n),_)|v[1]>0.&&n[1]>0.5).all(|(_,uv)|*uv==white),
                    "white seat tops must not sample the black source outline");
                assert_eq!(seat.footing_heights,vec![2.5;w*h]);
            }
            for piece in p.iter().filter(|p|stool_source(name,p,&refs,&g)) {
                let mut stool=TerrainMeshData{footing_heights:vec![2.5;w*h],..Default::default()};
                assert!(super::super::append(&mut stool,name,&refs,&g,piece,&mut vec![false;w*h],[0,0]));
                assert!(stool.solid.indices.is_empty());assert_eq!(stool.footing_heights,vec![2.5;w*h]);
                let first=piece.indices(w).count()*4;let paint=native_source_swatches(piece,&g,[[8,5],[3,9]]);
                for (n,uv) in stool.textured.normals[first..].iter().zip(&stool.textured.uvs[first..]) {
                    assert_eq!(*uv,paint[usize::from(n[1]<=0.5)]);
                }
            }
            for piece in plants.iter().filter(|p|p.row==0) {
                let west=g.bounds(piece.column,0).0;let south=g.bounds(piece.column,3).3;
                let mut geometry=SurfaceMeshData::default();
                append(&mut geometry,&g,piece,&refs,[west+0.36,west+16.-0.36,south-12.,south],
                    crate::interior_models::selected_model(ModelKind::PottedPlant).unwrap(),[0,0]);
                let top=geometry.positions.iter().filter(|v|v[1]==32.).collect::<Vec<_>>();
                assert_eq!(top.iter().map(|v|v[0]).fold(f32::INFINITY,f32::min),west);
                assert_eq!(top.iter().map(|v|v[0]).fold(f32::NEG_INFINITY,f32::max),west+16.,
                    "plant insets cannot create wall seams");
            }
            for piece in p.iter().filter(|p|p.row==0&&bookcase_source(name,p,&refs,&g)) {
                let mut backing=SurfaceMeshData::default();
                append_bookcase_backing(&mut backing,&g,piece,&refs,[0,0],name);
                assert!(backing.positions.iter().any(|v|v[1]==32.));
                let south=g.bounds(piece.column,1).3;
                assert!(backing.positions.iter().all(|v|v[2]<=south));
                let mut interior=SurfaceMeshData::default();
                append_bookcase_backing(&mut interior,&g,piece,&refs,[0,4],name);
                assert!(interior.indices.is_empty(),"interior cabinets cannot invent perimeter walls");
            }
            for warp in &data.events.warps {for dy in 0..2 {for dx in 0..2 {
                assert!(!claimed[(warp.y as usize*2+dy)*w+warp.x as usize*2+dx],"{name} warp remains unclaimed");
            }}}
            let mut floor=TerrainMeshData{footing_heights:vec![0.;w*h],authored_cells:vec![None;w*h],..Default::default()};
            let mut count=0;
            for (i,tile) in t.iter().enumerate() {
                if interior_floor(name,&tile.source)==Some(InteriorFloor::NativeRoom) {
                    append_top(&mut floor.textured,g.bounds(i%w,i/w).into(),0.,g.uv(i%w,i/w));count+=1;
                }
            }
            assert!(count>50);
            assert_eq!(finish_surfaces_excluding(&mut floor,name,&refs,&g,[0,0],&vec![false;w*h]),count);
            assert!(floor.solid.indices.is_empty());assert_eq!(floor.textured.indices.len(),count*6);
            for &i in &floor.textured.indices {let v=floor.textured.positions[i as usize];let uv=floor.textured.uvs[i as usize];
                assert_eq!(v[1],0.);assert!((uv[0]-v[0]/(w as f32*8.)).abs()<0.0001);
                assert!((uv[1]-v[2]/(h as f32*8.)).abs()<0.0001);
            }
            assert!(resolve("UnreviewedRadioLobby",&refs,&g,None).iter().all(|p|!matches("UnreviewedRadioLobby",p,&refs,&g)));
            if let Some(plant)=plants.iter().find(|p|p.height==4) {
                let i=plant.row*w+plant.column;let source=&t[i].source;
                let object:crate::live_profiles::Object=serde_json::from_value(serde_json::json!({
                    "name":"Custom radio lobby plant","map":name,"tileset":"radio_tower",
                    "metatile":source.metatile_id,"origin":[source.subtile_column,source.subtile_row],
                    "tiles":(0..plant.height).map(|y|(0..2).map(|x|t[i+y*w+x].source.tile_index).collect::<Vec<_>>()).collect::<Vec<_>>(),
                    "ground":1,"top_pixels":0,"depth_pixels":12
                })).unwrap();
                let custom=Document{objects:vec![object],atmosphere:None};
                assert!(!resolve(name,&refs,&g,Some(&custom)).iter().any(|p|p.indices(w).any(|j|plant.indices(w).any(|k|j==k))));
                for bad in plant.indices(w) {
                    let mut altered=t.clone();altered[bad].source.tile_index^=0x80;
                    let refs=altered.iter().collect::<Vec<_>>();
                    assert!(!resolve(name,&refs,&g,None).iter().any(|p|p.kind==ModelKind::PottedPlant&&p.column==plant.column&&p.row==plant.row));
                }
            }
        }
    }
}
