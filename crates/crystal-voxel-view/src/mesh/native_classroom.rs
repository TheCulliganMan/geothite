//! Earl's complete desk, chair and lectern drawings, with their own palettes.
use super::*;

const DESK:[u16;8]=[0x2a,0x2b,0x2b,0x2c,0x3a,0x3b,0x3b,0x3c];
const CHAIR:[u16;4]=[0x40,0x41,0x50,0x51];
const LECTERN:[u16;4]=[5,7,0x13,0x14];
const WALL:[u16;36]=[
    0x34,0x2d,0x2e,0x2e,0x2e,0x2e,0x2f,0x34,0x34,0x34,0x34,0x34,
    0x34,0x3d,0x3e,0x3e,0x3e,0x3e,0x3f,0x34,0x34,0x34,0x34,0x34,
    0x34,0x34,0x34,0x34,0x34,0x34,0x34,0x34,0x34,0x34,0x34,0x34,
];

pub(super) fn matches(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    if map!="EarlsPokemonAcademy"||g.width.checked_mul(g.height)!=Some(cells.len())
        ||p.width>g.width||p.height>g.height||p.column>g.width-p.width||p.row>g.height-p.height {return false;}
    let first=&cells[p.row*g.width+p.column].source;
    if (p.kind,p.width,p.height)==(ModelKind::WallPanel,12,3) {
        return p.indices(g.width).enumerate().all(|(j,i)| {
            let s=&cells[i].source;let x=j%12;let y=j/12;
            s.tileset_id.as_ref()=="lab"&&s.metatile_id==0x11+(x/4)as u16
                &&s.subtile_column==(x%4)as u8&&s.subtile_row==y as u8&&s.tile_index==WALL[j]
        });
    }
    let (block,origin,art):(u16,[u8;2],&[u16])=match (p.kind,p.width,p.height) {
        (ModelKind::DiningTable,4,2)=>(0x16,[0,0],&DESK),
        (ModelKind::LinkSeat,2,2) if matches!(first.subtile_column,0|2)=>(0x16,[first.subtile_column,2],&CHAIR),
        (ModelKind::Cabinet,2,2)=>(0x15,[2,0],&LECTERN),
        _=>return false,
    };
    p.indices(g.width).enumerate().all(|(j,i)| {let s=&cells[i].source;
        s.tileset_id.as_ref()=="lab"&&s.metatile_id==block&&s.subtile_column==origin[0]+(j%p.width)as u8
            &&s.subtile_row==origin[1]+(j/p.width)as u8&&s.tile_index==art[j]
    })
}

pub(super) fn candidates(map:&str,cells:&[&VisualTile],g:&GridGeometry,claimed:&mut [bool],out:&mut Vec<Placement>) {
    if map!="EarlsPokemonAcademy" {return;}
    let Some(ground)=ground_sample(cells,map,"lab",0x24) else{return;};
    for (kind,width,rows,depth,height) in [(ModelKind::DiningTable,4,2,8.,8.),
        (ModelKind::LinkSeat,2,2,8.,11.),(ModelKind::Cabinet,2,2,8.,8.),
        (ModelKind::WallPanel,12,3,3.,32.)] {
        if width>g.width||g.height<rows {continue;}
        for row in 0..=g.height-rows {for column in 0..=g.width-width {
            let p=Placement{kind,column,row,width,height:rows,ground,depth_pixels:depth,height_pixels:height,
                base_pixels:0.,footing_pixels:None};
            if p.indices(g.width).any(|i|claimed[i])||!matches(map,&p,cells,g) {continue;}
            for i in p.indices(g.width){claimed[i]=true;}out.push(p);
        }}
    }
}

pub(super) fn materials(kind:ModelKind,selected:&crate::interior_models::Model)->bool {
    match kind {ModelKind::DiningTable=>selected.has_native_dining_table_materials(),
        ModelKind::LinkSeat=>selected.has_native_link_seat_materials(),
        ModelKind::Cabinet=>selected.has_native_home_fixture_materials(),
        ModelKind::WallPanel=>true,_=>false}
}

pub(super) fn append(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,b:[f32;4],selected:&crate::interior_models::Model) {
    let scale=g.tile_height/8.;let [w,e,n,s]=b;
    match p.kind {
        ModelKind::WallPanel=>{
            let first=out.positions.len();let indices=out.indices.len();
            architecture_box(out,[w,e,0.,32.*scale,n,s],[1.;3]);
            out.uvs[first..].fill(native_source_swatches(p,g,[[0,16]])[0]);
            out.indices.drain(indices+6..indices+12);
            // Keep one source-sized blackboard near the ceiling, with an
            // additional plain wall course below it.
            for y in 0..3 {for x in 0..12 {
                let uv=g.uv(p.column+x,p.row+y);
                let a=w+(e-w)*x as f32/12.;let b=w+(e-w)*(x+1)as f32/12.;
                let bottom=(3-y)as f32*8.*scale;let top=bottom+8.*scale;
                append_quad(out,[[b,bottom,s],[b,top,s],[a,top,s],[a,bottom,s]],Vec3::Z.to_array(),
                    [[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2],[uv.0,uv.3]],
                    crate::interior_models::authored_face_color([1.;4],Vec3::Z));
            }}
            let paint=native_source_swatches(p,g,[[0,16]])[0];
            append_quad(out,[[e,0.,s],[e,8.*scale,s],[w,8.*scale,s],[w,0.,s]],
                Vec3::Z.to_array(),[paint;4],crate::interior_models::authored_face_color([1.;4],Vec3::Z));
        },
        ModelKind::DiningTable=>{
            let paint=native_source_swatches(p,g,[[1,2],[1,7]]);
            assert!(selected.append_native_dining_table(out,b,8.*scale,paint));
            // The drawing includes a storage apron beneath the desktop.
            let first=out.positions.len();let indices=out.indices.len();
            architecture_box(out,[w+0.5*scale,e-0.5*scale,scale,7.*scale,s-scale,s],[1.;3]);
            out.uvs[first..].fill(paint[1]);out.indices.drain(indices+6..indices+12);
            for x in 0..4 {
                let uv=g.uv(p.column+x,p.row+1);let a=w+(e-w)*x as f32/4.;let b=w+(e-w)*(x+1)as f32/4.;
                append_quad(out,[[b,scale,s],[b,7.*scale,s],[a,7.*scale,s],[a,scale,s]],Vec3::Z.to_array(),
                    [[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2],[uv.0,uv.3]],
                    crate::interior_models::authored_face_color([1.;4],Vec3::Z));
            }
        },
        ModelKind::LinkSeat=>{
            append_school_chair(out,b,scale,native_source_swatches(p,g,[[0,0],[3,2],[4,0],[3,3]]));
        },
        ModelKind::Cabinet=>assert!(selected.append_native_home_fixture_with_footprint(out,b,0.,8.*scale,
            native_source_swatches(p,g,[[1,8],[2,2],[2,2],[1,1]]))),
        _=>unreachable!("complete classroom drawing"),
    }
}

fn chair_triangle(out:&mut SurfaceMeshData,mut points:[Vec3;3],outside:Vec3,uv:[f32;2]) {
    let mut normal=(points[1]-points[0]).cross(points[2]-points[0]);
    if normal.length_squared()<1e-12 {return;}
    if normal.dot(outside)<0. {points.swap(1,2);normal=-normal;}
    normal=normal.normalize();let first=out.positions.len()as u32;
    out.positions.extend(points.map(|p|p.to_array()));out.normals.extend([normal.to_array();3]);
    out.uvs.extend([uv;3]);out.colors.extend([crate::interior_models::authored_face_color([1.;4],normal);3]);
    out.indices.extend([first,first+1,first+2]);
}

fn chair_panel(out:&mut SurfaceMeshData,rings:[[Vec3;8];2],paint:[[f32;2];2]) {
    let centers=rings.map(|ring|ring.into_iter().sum::<Vec3>()/8.);
    let center=(centers[0]+centers[1])*0.5;
    for j in 0..8 {
        let next=(j+1)%8;let [a,b,c,d]=[rings[0][j],rings[1][j],rings[1][next],rings[0][next]];
        let outside=(a+b+c+d)*0.25-center;
        chair_triangle(out,[a,b,c],outside,paint[1]);chair_triangle(out,[a,c,d],outside,paint[1]);
        for side in 0..2 {
            chair_triangle(out,[centers[side],rings[side][j],rings[side][next]],
                centers[side]-centers[1-side],paint[0]);
        }
    }
}

fn chair_rod(out:&mut SurfaceMeshData,a:Vec3,b:Vec3,radius:f32,uv:[f32;2]) {
    let delta=b-a;let axis=delta.normalize();
    let rotation=bevy::prelude::Quat::from_rotation_arc(Vec3::Y,axis);
    let ring=|center:Vec3|std::array::from_fn::<_,8,_>(|i| {
        let angle=i as f32*std::f32::consts::TAU/8.;
        center+rotation*Vec3::new(angle.cos()*radius,0.,angle.sin()*radius)
    });
    let rings=[ring(a),ring(b)];
    for j in 0..8 {let next=(j+1)%8;
        let [p,q,r,s]=[rings[0][j],rings[1][j],rings[1][next],rings[0][next]];
        let outside=(p+q+r+s)*0.25-(a+b)*0.5;
        chair_triangle(out,[p,q,r],outside,uv);chair_triangle(out,[p,r,s],outside,uv);
        chair_triangle(out,[a,p,s],-axis,uv);chair_triangle(out,[b,r,q],axis,uv);
    }
}

fn append_school_chair(out:&mut SurfaceMeshData,[w,e,n,s]:[f32;4],scale:f32,paint:[[f32;2];4]) {
    // Compact school furniture: one shallow molded seat, a rounded back and
    // four splayed metal legs. The former office armchair was stretched16px
    // deep, producing long rails and outsized armrests beside an8px desk.
    let center=Vec3::new((w+e)*0.5,0.,(n+s)*0.5);
    let at=|x,y,z|center+Vec3::new(x*(e-w)/8.,y*scale,z*(s-n)/8.);
    let outline=[[-3.,-3.5],[-3.7,-2.8],[-3.7,2.8],[-3.,3.5],
        [3.,3.5],[3.7,2.8],[3.7,-2.8],[3.,-3.5]];
    chair_panel(out,[outline.map(|[x,z]|at(x,4.5,z)),outline.map(|[x,z]|at(x,5.2,z))],
        [paint[0],paint[1]]);
    let back=[[-3.1,6.],[-3.7,6.6],[-3.7,9.8],[-3.1,10.4],
        [3.1,10.4],[3.7,9.8],[3.7,6.6],[3.1,6.]];
    chair_panel(out,[back.map(|[x,y]|at(x,y,2.6+(y-5.)*0.08)),
        back.map(|[x,y]|at(x,y,3.1+(y-5.)*0.08))],[paint[0],paint[1]]);
    for x in [-1.,1.] {for z in [-1.,1.] {
        chair_rod(out,at(x*3.1,0.22,z*2.9),at(x*2.7,4.6,z*2.5),0.23*scale,paint[2]);
        chair_rod(out,at(x*3.1,0.,z*2.9),at(x*3.1,0.4,z*2.9),0.27*scale,paint[3]);
    }
        chair_rod(out,at(x*2.7,4.6,2.5),at(x*2.7,9.5,3.2),0.20*scale,paint[2]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::Vec2;
    #[test]
    fn external_classroom_has_all_desks_chairs_lectern_and_preserves_custom_ownership() {
        let explicit=std::env::var_os("CRYSTAL_RENDER_TEST_PACK");
        let path=explicit.as_ref().map(std::path::PathBuf::from).unwrap_or_else(||
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content-packs/text-tui.crystalpack"));
        assert!(explicit.is_none()||path.is_file());if !path.is_file(){return;}
        let pack=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let module=&pack.pack().data().maps["EarlsPokemonAcademy"];
        let layout=&pack.pack().runtime_files()["data/tilesets/lab_metatiles.bin"];
        let (w,h)=(module.attributes.width as usize*4,module.attributes.height as usize*4);
        let mut tiles=Vec::new();
        for y in 0..h {for x in 0..w {
            let block=module.blocks[y/4*(w/4)+x/4];
            tiles.push(VisualTile{column:x as u32,row:y as u32,texture:Default::default(),priority:false,animation_frames:None,
                source:VisualTileSource{tileset_id:"lab".into(),metatile_id:block,subtile_column:(x%4)as u8,
                    subtile_row:(y%4)as u8,tile_index:layout[block as usize*16+y%4*4+x%4]as u16}});
        }}
        let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let refs=tiles.iter().collect::<Vec<_>>();
        let placements=resolve("EarlsPokemonAcademy",&refs,&g,None).into_iter()
            .filter(|p|matches("EarlsPokemonAcademy",p,&refs,&g)).collect::<Vec<_>>();
        for (kind,count) in [(ModelKind::DiningTable,8),(ModelKind::LinkSeat,16),(ModelKind::Cabinet,1),(ModelKind::WallPanel,1)] {
            assert_eq!(placements.iter().filter(|p|p.kind==kind).count(),count,"{kind:?}");
        }
        assert_eq!(traditional_tabletop_height("EarlsPokemonAcademy",&refs,&g,None,5,9),Some(8.),"book rests on actual first desktop");
        let frame=crystal_render_api::VisualWorldFrame{active:true,map_id:"EarlsPokemonAcademy".into(),
            grid_size:bevy::prelude::UVec2::new(w as u32,h as u32),tile_size:Vec2::splat(8.),tiles:tiles.clone(),..Default::default()};
        let seats=crate::mesh::seating_surfaces(&frame,None);
        assert_eq!(seats.len(),16);
        assert!(seats.iter().all(|seat|seat.height>0.&&seat.height<11.&&seat.yaw==Some(std::f32::consts::PI)));
        for (x,y) in [(2,5),(3,11),(4,11),(4,7)] {
            let foot=Vec2::new(((x*2+1)as f32-w as f32*0.5)*8.,(h as f32*0.5-((y+1)*2)as f32)*8.);
            assert!(seats.iter().any(|seat|seat.target(&frame,foot,0.).is_some()),"student ({x},{y}) has actual seat support");
        }
        for p in placements {
            assert!(!matches("ElmsLab",&p,&refs,&g));
            let mut mesh=TerrainMeshData{footing_heights:vec![2.75;tiles.len()],..Default::default()};
            let mut claimed=vec![false;tiles.len()];
            assert!(super::super::append(&mut mesh,"EarlsPokemonAcademy",&refs,&g,&p,&mut claimed,[0,0]));
            assert!(mesh.solid.indices.is_empty());assert_eq!(mesh.footing_heights,vec![2.75;tiles.len()]);
            assert_eq!(claimed.iter().filter(|&&x|x).count(),p.width*p.height);
            assert!(mesh.textured.positions.iter().all(|v|v[1]>=0.&&v[1]<=p.height_pixels+0.001));
            if p.kind==ModelKind::WallPanel {
                let north=mesh.textured.positions.iter().filter(|v|v[1]>0.).map(|v|v[2]).fold(f32::INFINITY,f32::min);
                let south=mesh.textured.positions.iter().filter(|v|v[1]>0.).map(|v|v[2]).fold(f32::NEG_INFINITY,f32::max);
                assert!((south-north-3.).abs()<0.001,"wall has a thin physical depth");
                for y in 0..3 {for x in 0..12 {
                    let uv=g.uv(p.column+x,p.row+y);
                    let expected=[[(x+1+p.column)as f32*8.,(3-y)as f32*8.,south],
                        [(x+1+p.column)as f32*8.,(4-y)as f32*8.,south],
                        [(x+p.column)as f32*8.,(4-y)as f32*8.,south],
                        [(x+p.column)as f32*8.,(3-y)as f32*8.,south]];
                    let tex=[[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2],[uv.0,uv.3]];
                    assert!(mesh.textured.positions.windows(4).zip(mesh.textured.uvs.windows(4))
                        .any(|(positions,uvs)|positions==expected&&uvs==tex),"upright source cell ({x},{y})");
                }}
            }
            for tri in mesh.textured.indices.chunks_exact(3) {
                let [a,b,c]=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
                let normal=Vec3::from_array(mesh.textured.normals[tri[0]as usize]);
                assert!((b-a).cross(c-a).dot(normal)>0.);
            }
            for i in p.indices(w) {
                let mut changed=tiles.clone();changed[i].source.tile_index=0xffff;
                assert!(!matches("EarlsPokemonAcademy",&p,&changed.iter().collect::<Vec<_>>(),&g));
            }
            let source=&refs[p.row*w+p.column].source;
            let object=Object{name:"User classroom furniture".into(),tileset:"lab".into(),map:Some("EarlsPokemonAcademy".into()),
                maps:None,metatile:source.metatile_id,
                metatiles:if p.kind==ModelKind::WallPanel {Some(vec![vec![0x11,0x12,0x13]])}else{None},
                origin:[source.subtile_column,source.subtile_row],
                tiles:(0..p.height).map(|y|(0..p.width).map(|x|refs[(p.row+y)*w+p.column+x].source.tile_index).collect()).collect(),
                ground:0x24,top_pixels:0,depth_pixels:0.,footing_pixels:None,mask:Default::default(),parts:vec![]};
            let custom=Document{objects:vec![object],atmosphere:None};
            assert!(!resolve("EarlsPokemonAcademy",&refs,&g,Some(&custom)).iter().any(|q|
                q.kind==p.kind&&q.column==p.column&&q.row==p.row),"custom drawing owns its source cells");
            if p.kind==ModelKind::DiningTable {
                assert_eq!(traditional_tabletop_height("EarlsPokemonAcademy",&refs,&g,Some(&custom),p.column+1,p.row+1),None);
            }
        }
    }
}
