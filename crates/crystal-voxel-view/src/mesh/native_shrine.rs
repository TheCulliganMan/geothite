//! The shrine's lit floor plots contain a small stand and a flat pool of light.
use super::*;

pub(super) fn dragon_pixels(asset:Asset,east:bool)->[[usize;2];4] {
    if asset==Asset::ShrineDragonMask {[[0,0],[1,0],[1,3],[5,12]]}
    else if east {[[0,0],[18,0],[17,0],[17,0]]}
    else {[[7,0],[0,0],[21,15],[21,15]]}
}

pub(super) fn append_wall(mesh:&mut TerrainMeshData,g:&GridGeometry,p:&Placement,cells:&[&VisualTile]) {
    // Fingerprinted wall cells occupy only the three north-wall source rows.
    // Stack them vertically instead of turning each into a full-height cube.
    let source_row=usize::from(cells[p.row*g.width+p.column].source.subtile_row);
    assert!(source_row<3&&p.width==1&&p.height==1);
    let (w,e,_,_)=g.bounds(p.column,p.row);let k=g.tile_height/8.;
    let front=g.origin_z+(p.row+3-source_row)as f32*g.tile_height;let back=front-3.*k;
    let low=(2-source_row)as f32*8.*k;let high=low+8.*k;
    let mut part=SurfaceMeshData::default();
    slab(&mut part,[w,e,back,front],low,high,[1.;4]);
    part.indices=part.indices.chunks_exact(3).filter(|tri|
        part.normals[tri[0]as usize][2]<0.9).flatten().copied().collect();
    let uv=source_texel_uv(g,p,[0,0]);part.uvs.fill(uv);
    for (color,normal) in part.colors.iter_mut().zip(&part.normals) {
        *color=crate::interior_models::authored_face_color([1.;4],Vec3::from_array(*normal));
    }
    let offset=mesh.textured.positions.len()as u32;
    mesh.textured.positions.extend(part.positions);mesh.textured.normals.extend(part.normals);
    mesh.textured.uvs.extend(part.uvs);mesh.textured.colors.extend(part.colors);
    mesh.textured.indices.extend(part.indices.into_iter().map(|i|i+offset));
    let uv=g.uv(p.column,p.row);
    append_quad(&mut mesh.textured,[[e,low,front],[e,high,front],[w,high,front],[w,low,front]],
        Vec3::Z.to_array(),[[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2],[uv.0,uv.3]],
        crate::interior_models::authored_face_color([1.;4],Vec3::Z));
}

fn triangle(out:&mut SurfaceMeshData,p:[Vec3;3],normal:Vec3,uv:[f32;2]) {
    let first=out.positions.len()as u32;
    out.positions.extend(p.map(|v|v.to_array()));
    out.normals.extend([normal.to_array();3]);out.uvs.extend([uv;3]);
    out.colors.extend([crate::interior_models::authored_face_color([1.;4],normal);3]);
    out.indices.extend([first,first+1,first+2]);
}

fn tube(out:&mut SurfaceMeshData,start:Vec3,end:Vec3,radii:[f32;2],uv:[f32;2]) {
    let delta=end-start;let length=delta.length();
    let rotation=bevy::prelude::Quat::from_rotation_arc(Vec3::Y,delta/length);
    let point=|angle:f32,r:f32,y:f32|start+rotation*Vec3::new(angle.cos()*r,y,angle.sin()*r);
    let sides=12;
    for j in 0..sides {
        let a=j as f32*std::f32::consts::TAU/sides as f32;
        let b=(j+1)as f32*std::f32::consts::TAU/sides as f32;
        let mid=(a+b)*0.5;
        let normal=rotation*Vec3::new(mid.cos(),(radii[0]-radii[1])/length,mid.sin()).normalize();
        let p=[point(a,radii[0],0.),point(a,radii[1],length),point(b,radii[1],length),point(b,radii[0],0.)];
        triangle(out,[p[0],p[1],p[2]],normal,uv);
        triangle(out,[p[0],p[2],p[3]],normal,uv);
        triangle(out,[start,p[0],p[3]],rotation*-Vec3::Y,uv);
        triangle(out,[end,p[2],p[1]],rotation*Vec3::Y,uv);
    }
}

pub(super) fn append_lit_fixture(mesh:&mut TerrainMeshData,g:&GridGeometry,p:&Placement) {
    let (w,_,n,_)=g.bounds(p.column,p.row);let k=g.tile_height/8.;let kx=g.tile_width/8.;
    // Split only at tile and silhouette boundaries. The small source stand's
    // projected ink is replaced by the same illuminated plank at row22;
    // every other texel, including the pool's outline and seams, stays exact.
    let xs=[0usize,8,12,16,20,24,32];let ys=[0usize,7,8,16,20,24,32];
    for xx in xs.windows(2) {for yy in ys.windows(2) {
        let [x0,x1]=[xx[0],xx[1]];let [y0,y1]=[yy[0],yy[1]];
        let mask=x0>=12&&x1<=20&&y0>=7&&y1<=20;
        let sample_y=if mask {22}else{y0};
        let uv=g.uv(p.column+x0/8,p.row+sample_y/8);
        let u=|x:usize|uv.0+(uv.1-uv.0)*(x-x0/8*8)as f32/8.;
        let v=|y:f32|uv.2+(uv.3-uv.2)*(y-(sample_y/8*8)as f32)/8.;
        let [v0,v1]=if mask {[v(22.5);2]}else{[v(y0 as f32),v(y1 as f32)]};
        append_quad(&mut mesh.textured,
            [[w+x0 as f32*kx,0.,n+y0 as f32*k],[w+x0 as f32*kx,0.,n+y1 as f32*k],
             [w+x1 as f32*kx,0.,n+y1 as f32*k],[w+x1 as f32*kx,0.,n+y0 as f32*k]],
            Vec3::Y.to_array(),[[u(x0),v0],[u(x0),v1],[u(x1),v1],[u(x1),v0]],[1.;4]);
    }}
    let dark=source_texel_uv(g,p,[14,12]);
    let shade=source_texel_uv(g,p,[15,19]);
    let light=source_texel_uv(g,p,[11,4]);
    let center=Vec3::new(w+16.*k,0.,n+19.*k);
    let at=|x:f32,y:f32,z:f32|center+Vec3::new(x,y,z)*k;
    let mut part=|a:[f32;3],b:[f32;3],r:[f32;2],paint|tube(&mut mesh.textured,
        at(a[0],a[1],a[2]),at(b[0],b[1],b[2]),r.map(|v|v*k),paint);
    part([0.,0.,0.],[0.,0.65,0.],[2.,1.3],dark);
    part([0.,0.65,0.],[0.,1.2,0.],[1.3,0.45],shade);
    part([0.,1.2,0.],[0.,8.,0.],[0.32,0.32],dark);
    for sign in [-1.,1.] {
        part([0.,5.8,0.],[sign*1.7,6.8,0.],[0.24,0.24],shade);
        part([sign*1.7,6.8,0.],[sign*2.5,8.4,0.],[0.24,0.24],dark);
    }
    for (x,y) in [(-2.5,8.4),(0.,8.8),(2.5,8.4)] {
        part([x,y,0.],[x,y+0.5,0.],[0.75,0.65],dark);
        part([x,y+0.5,0.],[x,y+2.,0.],[0.3,0.3],shade);
        part([x,y+2.,0.],[x,y+2.7,0.],[0.27,0.04],light);
    }
}
