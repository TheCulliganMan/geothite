//! The B1F alarm's complete native drawing: a gold seated cat on a gray plinth.
use super::*;
use crate::mesh::modeled_interiors::architecture_box;

// Gold body/shadow/eyes and gray pedestal/highlight/engraving, all sampled
// inside this fixture's eight authenticated source cells.
const PIXELS: [[usize;2];6] = [[3,1],[4,0],[0,0],[3,16],[2,16],[1,16]];

fn triangle(out:&mut SurfaceMeshData,mut p:[Vec3;3],outside:Vec3,uv:[f32;2]) {
    let mut normal=(p[1]-p[0]).cross(p[2]-p[0]);
    if normal.length_squared()<1e-12 {return;}
    if normal.dot(outside)<0. {p.swap(1,2);normal=-normal;}
    normal=normal.normalize();let first=out.positions.len()as u32;
    out.positions.extend(p.map(|v|v.to_array()));out.normals.extend([normal.to_array();3]);
    out.uvs.extend([uv;3]);out.colors.extend([
        crate::interior_models::authored_face_color([1.;4],normal);3]);
    out.indices.extend([first,first+1,first+2]);
}

fn oval(out:&mut SurfaceMeshData,center:Vec3,r:Vec3,uv:[f32;2]) {
    let first=out.positions.len();
    const ROWS:usize=12;
    const SIDES:usize=24;
    let point=|row:usize,side:usize| {
        let a=std::f32::consts::PI*row as f32/ROWS as f32;
        let b=std::f32::consts::TAU*side as f32/SIDES as f32;
        center+Vec3::new(a.sin()*b.cos(),a.cos(),a.sin()*b.sin())*r
    };
    for row in 0..ROWS {for side in 0..SIDES {
        let [a,b,c,d]=[point(row,side),point(row+1,side),point(row+1,side+1),point(row,side+1)];
        let outside=(a+b+c+d)*0.25-center;
        if row!=ROWS-1 {triangle(out,[a,b,c],outside,uv);}
        if row!=0 {triangle(out,[a,c,d],outside,uv);}
    }}
    // Analytic ellipsoid normals retain the curve across triangle corners,
    // including the nonuniform native footprint/height fit. Pedestal, ears
    // and other deliberately hard faces keep the triangle helper's normals.
    for i in first..out.positions.len() {
        let normal=((Vec3::from_array(out.positions[i])-center)/(r*r)).normalize();
        out.normals[i]=normal.to_array();
        out.colors[i]=crate::interior_models::authored_face_color([1.;4],normal);
    }
}

fn ear(out:&mut SurfaceMeshData,p:[Vec3;4],paint:[[f32;2];2]) {
    let center=p.into_iter().sum::<Vec3>()*0.25;
    for (j,[a,b,c]) in [[0,1,2],[0,3,1],[0,2,3],[1,3,2]].into_iter().enumerate() {
        triangle(out,[p[a],p[b],p[c]],(p[a]+p[b]+p[c])/3.-center,paint[usize::from(j==0)]);
    }
}

pub(super) fn append(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,b:[f32;4]) {
    let paint=PIXELS.map(|pixel| {
        let uv=g.uv(p.column+pixel[0]/8,p.row+pixel[1]/8);
        [uv.0+(uv.1-uv.0)*((pixel[0]%8)as f32+0.5)/8.,
         uv.2+(uv.3-uv.2)*((pixel[1]%8)as f32+0.5)/8.]
    });
    append_shape(out,b,p.rise_pixels*g.tile_height/8.,paint);
}

pub(super) fn append_wall_backing(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,
    cells:&[&VisualTile])->bool {
    if !matches!(p.form,Form::RocketPersian)||p.width!=2||p.height!=4
        ||p.column+2>g.width||p.row+4>g.height||cells.len()!=g.width*g.height {return false;}
    let art=[[0x45,0x46],[0x55,0x56],[0x09,0x19],[0x30,0x31]];
    if !p.indices(g.width).enumerate().all(|(j,i)| {
        let s=&cells[i].source;
        s.tileset_id.as_ref()=="underground"&&s.metatile_id==4
            &&s.subtile_column as usize==j%2&&s.subtile_row as usize==j/2
            &&s.tile_index==art[j/2][j%2]
    }) {return false;}
    // Only a verified adjoining two-row horizontal wall establishes backing.
    // An isolated/custom statue drawing must not invent a room wall.
    let joins=|x:usize| {
        let a=&cells[p.row*g.width+x].source;
        let b=&cells[(p.row+1)*g.width+x].source;
        a.tileset_id.as_ref()=="underground"&&b.tileset_id.as_ref()=="underground"
            &&crate::rocket_base::is_upper_face(a)&&crate::rocket_base::is_lower_face(b)
    };
    let left=p.column.checked_sub(1).filter(|&x|joins(x));
    let right=(p.column+2<g.width).then_some(p.column+2).filter(|&x|joins(x));
    let Some(donor)=left.or(right) else{return false;};
    let texel=|row,x:f32,y:f32| {
        let uv=g.uv(donor,row);
        [uv.0+(uv.1-uv.0)*(x+0.5)/8.,uv.2+(uv.3-uv.2)*(y+0.5)/8.]
    };
    let(w,_,n,_)=g.bounds(p.column,p.row);
    super::append_native_rocket_wall(out,[w,w+2.*g.tile_width,n,n+2.*g.tile_height],
        crate::rocket_base::WALL_HEIGHT*g.tile_height/8.,
        [true,right.is_none(),true,left.is_none()],
        texel(p.row,0.,0.),texel(p.row+1,3.,3.));
    true
}

fn append_shape(out:&mut SurfaceMeshData,[w,e,n,s]:[f32;4],height:f32,paint:[[f32;2];6]) {
    // Keep the established 16 x 8 footprint and 28px total height. Anatomy is
    // fitted as one sculpture; no source claim, support or alarm state changes.
    let k=Vec3::new((e-w)/16.,height/28.,(s-n)/8.);
    let at=|x,y,z|Vec3::new((w+e)*0.5,0.,(n+s)*0.5)+Vec3::new(x,y,z)*k;
    let mut block=|bounds:[f32;6],uv| {
        let [x0,x1,y0,y1,z0,z1]=bounds;let first=out.positions.len();
        architecture_box(out,[at(x0,y0,z0).x,at(x1,y1,z1).x,
            y0*k.y,y1*k.y,at(x0,y0,z0).z,at(x1,y1,z1).z],[1.;3]);
        out.uvs[first..].fill(uv);
    };
    block([-5.8,5.8,0.,1.5,-3.8,3.8],paint[5]);
    block([-5.1,5.1,1.5,10.2,-3.2,3.2],paint[3]);
    block([-5.6,5.6,10.2,11.5,-3.6,3.6],paint[4]);
    // Recessed dark plaque and its raised gray inner face.
    block([-3.8,3.8,4.,8.,3.2,3.28],paint[5]);
    block([-3.1,3.1,4.6,7.4,3.28,3.34],paint[3]);
    let mut ellipsoid=|x,y,z,rx,ry,rz,uv|oval(out,at(x,y,z),Vec3::new(rx,ry,rz)*k,uv);
    // Seated haunches blend into shoulders rather than a thin cylindrical
    // neck under a spherical head. Forelegs remain separate down to the paws.
    ellipsoid(0.,15.7,-0.6,3.3,4.2,2.5,paint[0]);
    for sign in [-1.,1.] {
        ellipsoid(sign*2.9,13.7,-0.2,1.8,2.1,2.,paint[0]);
        ellipsoid(sign*1.25,14.5,1.7,0.85,2.65,0.8,paint[0]);
        ellipsoid(sign*1.25,11.9,2.,1.,0.5,1.3,paint[0]);
    }
    ellipsoid(0.,18.7,0.,2.7,3.4,2.1,paint[0]);
    // A low broad skull, cheek planes and small muzzle keep the feline face
    // distinct from the previous round head and two protruding muzzle balls.
    ellipsoid(0.,23.8,0.15,3.3,2.25,2.1,paint[0]);
    for sign in [-1.,1.] {
        ellipsoid(sign*1.3,22.8,1.55,1.35,0.85,0.9,paint[0]);
        ellipsoid(sign*0.57,22.3,2.25,0.65,0.38,0.56,paint[0]);
    }
    ellipsoid(0.,21.9,2.,1.1,0.42,0.8,paint[0]);
    // Embossed almond lids have a shallow lower arc and a lifted outer
    // corner. The narrow vertical pupils sit on the face, not on round eyes.
    let face_z=|x:f32,y:f32|0.15+2.1*(1.-(x/3.3).powi(2)
        -((y-23.8)/2.25).powi(2)).max(0.).sqrt();
    for sign in [-1.,1.] {
        let center=at(sign*1.3,24.05,face_z(sign*1.3,24.05)+0.075);
        for segment in 0..24 {
            let edge=|i:usize| {
                let angle=std::f32::consts::TAU*i as f32/24.;
                let x=angle.cos()*0.82;
                let y=angle.sin()*if angle.sin()>0. {0.24}else{0.11};
                let (x,y)=(sign*(1.3+x),24.05+y+x*0.14);
                at(x,y,face_z(x,y)+0.015)
            };
            triangle(out,[center,edge(segment),edge(segment+1)],Vec3::Z,paint[1]);
        }
        oval(out,at(sign*1.3,24.05,2.16),Vec3::new(0.10,0.19,0.045)*k,paint[2]);
    }
    // Chiselled nose and forehead jewel are metal relief in the same source
    // gold/shadow palette; the statue does not borrow the living cat's colors.
    ear(out,[at(-0.34,22.56,2.59),at(0.34,22.56,2.59),
        at(0.,22.27,2.9),at(0.,22.45,2.45)],[paint[1];2]);
    let jewel=[[0.,25.55],[0.38,24.94],[0.,24.40],[-0.38,24.94]]
        .map(|[x,y]|at(x,y,face_z(x,y)+0.015));
    for i in 0..4 {
        triangle(out,[jewel[i],jewel[(i+1)%4],at(0.,24.97,2.36)],Vec3::Z,
            paint[if i<2 {1}else{0}]);
    }
    for sign in [-1.,1.] {
        ear(out,[at(sign*1.25,24.8,0.),at(sign*3.1,24.6,0.8),
            at(sign*2.8,28.,0.),at(sign*2.4,24.8,-0.8)],[paint[0],paint[1]]);
        // Three fine raised whisker ridges are attached to each cheek. They
        // remain inside the established sculpture/plinth footprint.
        for (root_y,end_y) in [(22.65,23.05),(22.45,22.5),(22.25,21.95)] {
            let a=at(sign*1.45,root_y,2.36);let b=at(sign*3.75,end_y,2.35);
            let tangent=(b-a).normalize();let side=tangent.cross(Vec3::Z).normalize();
            let depth=tangent.cross(side);
            for segment in 0..8 {
                let ring=|center:Vec3,radius:f32,i:usize| {
                    let angle=std::f32::consts::TAU*i as f32/8.;
                    center+(side*angle.cos()+depth*angle.sin())*radius
                };
                let [q0,q1,q2,q3]=[ring(a,0.07*k.y,segment),ring(b,0.025*k.y,segment),
                    ring(b,0.025*k.y,segment+1),ring(a,0.07*k.y,segment+1)];
                let outside=(q0+q1+q2+q3)*0.25-(a+b)*0.5;
                triangle(out,[q0,q1,q2],outside,paint[0]);
                triangle(out,[q0,q2,q3],outside,paint[0]);
                triangle(out,[b,q2,q1],tangent,paint[0]);
                triangle(out,[a,q0,q3],-tangent,paint[0]);
            }
        }
    }
    // A single connected, capped curl rests beside the haunch.
    let path=[[2.5,12.3,-1.5],[4.,12.,-1.],[5.,12.,0.],[5.3,12.5,1.3],
        [5.,13.2,2.2],[4.2,13.5,2.4],[3.7,13.2,2.]];
    let centers=path.map(Vec3::from_array);
    let rings:Vec<[Vec3;16]>=centers.iter().enumerate().map(|(i,&center)| {
        let tangent=(centers[(i+1).min(centers.len()-1)]-centers[i.saturating_sub(1)]).normalize();
        let side=tangent.cross(Vec3::Y).normalize();let up=tangent.cross(side);
        std::array::from_fn(|j| {
            let angle=j as f32*std::f32::consts::TAU/16.;
            let v=center+(side*angle.cos()+up*angle.sin())*0.43;
            at(v.x,v.y,v.z)
        })
    }).collect();
    for i in 0..rings.len()-1 {for j in 0..16 {
        let next=(j+1)%16;let [a,b,c,d]=[rings[i][j],rings[i+1][j],rings[i+1][next],rings[i][next]];
        let center=(centers[i]+centers[i+1])*0.5;
        let outside=(a+b+c+d)*0.25-at(center.x,center.y,center.z);
        triangle(out,[a,b,c],outside,paint[0]);triangle(out,[a,c,d],outside,paint[0]);
    }}
    for (i,other) in [(0,1),(rings.len()-1,rings.len()-2)] {
        let c=centers[i];let center=at(c.x,c.y,c.z);let delta=(centers[i]-centers[other])*k;
        for j in 0..16 {triangle(out,[center,rings[i][j],rings[i][(j+1)%16]],delta,paint[0]);}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wall_backing_closes_only_the_verified_course_behind_the_sensor() {
        let g=GridGeometry{width:6,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut tiles=(0..24).map(|i|VisualTile {
            column:(i%6)as u32,row:(i/6)as u32,texture:Default::default(),
            animation_frames:None,priority:false,
            source:VisualTileSource{tileset_id:"underground".into(),metatile_id:1,
                subtile_column:(i%6%4)as u8,subtile_row:(i/6)as u8,tile_index:0x10},
        }).collect::<Vec<_>>();
        let art=[[0x45,0x46],[0x55,0x56],[0x09,0x19],[0x30,0x31]];
        for y in 0..4 {for x in 0..2 {
            let s=&mut tiles[y*6+x+2].source;s.metatile_id=4;
            s.subtile_column=x as u8;s.subtile_row=y as u8;s.tile_index=art[y][x];
        }}
        let p=Placement{column:2,row:0,width:2,height:4,ground:0,form:Form::RocketPersian,
            rise_pixels:28.,depth_pixels:8.,front_rows:4.};
        assert!(!append_wall_backing(&mut SurfaceMeshData::default(),&g,&p,
            &tiles.iter().collect::<Vec<_>>()),"freestanding statue cannot infer a wall");
        for x in [1,4] {tiles[x].source.tile_index=4;tiles[6+x].source.tile_index=0x14;}
        let mut mesh=SurfaceMeshData::default();
        assert!(append_wall_backing(&mut mesh,&g,&p,&tiles.iter().collect::<Vec<_>>()));
        let mut front_area=0.;
        for tri in mesh.indices.chunks_exact(3) {
            let [a,b,c]=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
            let normal=Vec3::from_array(mesh.normals[tri[0]as usize]);
            if normal.z>0.9 {
                assert_eq!([a.z,b.z,c.z],[16.;3]);
                front_area+=(b-a).cross(c-a).z/2.;
            }
        }
        assert!((front_area-16.*32.).abs()<0.001,"ceiling-to-floor wall behind statue without a gap");
        assert!(mesh.positions.iter().all(|v|(16.0..=32.0).contains(&v[0])
            &&(0.0..=32.0).contains(&v[1])&&(0.0..=16.0).contains(&v[2])),
            "backing stays behind the pedestal and inside its own source columns");
        for i in p.indices(6) {
            let mut altered=tiles.clone();altered[i].source.tile_index^=0x80;
            let mut rejected=SurfaceMeshData::default();
            assert!(!append_wall_backing(&mut rejected,&g,&p,&altered.iter().collect::<Vec<_>>()));
            assert!(rejected.positions.is_empty());
        }
        let mut surface=TerrainMeshData{footing_heights:vec![72.;24],..Default::default()};
        super::super::append(&mut surface,&g,&p,&tiles.iter().collect::<Vec<_>>(),[0,0]);
        assert_eq!(surface.footing_heights,vec![72.;24]);
        assert_eq!(p.indices(6).count(),8,"statue ownership stays eight cells");
    }
    #[test]
    fn external_rocket_persian_backing_closes_all_five_security_statues() {
        let path=std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack required");
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(std::path::PathBuf::from(path)).unwrap();
        let pack=loaded.pack();let data=&pack.data().maps["TeamRocketBaseB1F"];
        let art=&pack.runtime_files()["data/tilesets/underground_metatiles.bin"];
        let(w,h)=(data.attributes.width as usize*4,data.attributes.height as usize*4);
        let tiles=(0..w*h).map(|i| {
            let(x,y)=(i%w,i/w);let block=data.blocks[y/4*(w/4)+x/4];
            VisualTile{column:x as u32,row:y as u32,texture:Default::default(),
                animation_frames:None,priority:false,
                source:VisualTileSource{tileset_id:"underground".into(),metatile_id:block,
                    subtile_column:(x%4)as u8,subtile_row:(y%4)as u8,
                    tile_index:art[block as usize*16+y%4*4+x%4]as u16}}
        }).collect::<Vec<_>>();
        let cells=tiles.iter().collect::<Vec<_>>();
        let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let placements=super::super::resolve("TeamRocketBaseB1F",&cells,&g,None);
        let statues=placements.iter().filter(|p|matches!(p.form,Form::RocketPersian)).collect::<Vec<_>>();
        assert_eq!(statues.len(),5,"all source security sensors must be accounted for");
        let mut mesh=TerrainMeshData{footing_heights:vec![7.5;w*h],..Default::default()};
        for p in statues {
            let mut backing=SurfaceMeshData::default();
            assert!(append_wall_backing(&mut backing,&g,p,&cells),
                "source statue at {},{} must retain its wall",p.column,p.row);
            let front=(p.row+2)as f32*8.;let mut area=0.;
            for tri in backing.indices.chunks_exact(3) {
                if backing.normals[tri[0]as usize][2]<0.9 {continue;}
                let[a,b,c]=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(backing.positions[i as usize]));
                assert_eq!([a.z,b.z,c.z],[front;3]);area+=(b-a).cross(c-a).z/2.;
            }
            assert!((area-512.).abs()<0.001,"complete ceiling-height source backing");
            super::super::append(&mut mesh,&g,p,&cells,[0,0]);
        }
        assert_eq!(mesh.footing_heights,vec![7.5;w*h]);
    }
    #[test]
    fn feline_curves_keep_analytic_normals_under_nonuniform_fitting() {
        let center=Vec3::new(3.,7.,-2.);let radius=Vec3::new(2.,5.,0.75);
        let mut mesh=SurfaceMeshData::default();
        oval(&mut mesh,center,radius,[0.2,0.3]);
        assert!(mesh.indices.len()/3>=500,"adequate curved silhouette resolution");
        for (p,n) in mesh.positions.iter().zip(&mesh.normals) {
            let gradient=(Vec3::from_array(*p)-center)/(radius*radius);
            let n=Vec3::from_array(*n);
            assert!(n.is_finite()&&(n.length_squared()-1.).abs()<0.00001);
            assert!(n.dot(gradient.normalize())>0.99999);
        }
        assert!(mesh.normals.chunks_exact(3).any(|n|n[0]!=n[1]));
        let mut sculpture=SurfaceMeshData::default();
        append_shape(&mut sculpture,[-8.,8.,-4.,4.],28.,[[0.2,0.3];6]);
        assert!(sculpture.normals.chunks_exact(3).any(|n|n[0]==n[1]&&n[0]==n[2]),
            "hard pedestal/ear faces must not inherit body smoothing");
    }
    #[test]
    fn seated_cat_keeps_pedestal_bounds_and_outward_finite_faces() {
        for height in [14.,28.,56.] {
            let mut out=SurfaceMeshData::default();
            append_shape(&mut out,[-8.,8.,-4.,4.],height,[[0.2,0.3];6]);
            assert!(out.positions.iter().any(|v|v[1]==height));
            assert!(out.positions.iter().all(|v|v.iter().all(|f|f.is_finite())
                &&v[0]>=-8.&&v[0]<=8.&&v[1]>=0.&&v[1]<=height&&v[2]>=-4.&&v[2]<=4.));
            for ids in out.indices.chunks_exact(3) {
                let [a,b,c]=[ids[0],ids[1],ids[2]].map(|i|Vec3::from_array(out.positions[i as usize]));
                assert!((b-a).cross(c-a).dot(Vec3::from_array(out.normals[ids[0]as usize]))>0.);
            }
        }
    }
}
