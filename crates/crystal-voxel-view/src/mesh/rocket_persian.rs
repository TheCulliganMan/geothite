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
    let point=|row:usize,side:usize| {
        let a=std::f32::consts::PI*row as f32/10.;
        let b=std::f32::consts::TAU*side as f32/20.;
        center+Vec3::new(a.sin()*b.cos(),a.cos(),a.sin()*b.sin())*r
    };
    for row in 0..10 {for side in 0..20 {
        let [a,b,c,d]=[point(row,side),point(row+1,side),point(row+1,side+1),point(row,side+1)];
        let outside=(a+b+c+d)*0.25-center;
        if row!=9 {triangle(out,[a,b,c],outside,uv);}
        if row!=0 {triangle(out,[a,c,d],outside,uv);}
    }}
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
    // Haunches, upright chest and two separate straight forelegs/paws.
    ellipsoid(0.,15.1,-0.6,3.,3.8,2.6,paint[0]);
    for sign in [-1.,1.] {
        ellipsoid(sign*2.8,13.6,-0.2,1.7,2.,2.,paint[0]);
        ellipsoid(sign*1.3,14.2,1.6,0.75,2.5,0.75,paint[0]);
        ellipsoid(sign*1.3,11.9,2.,1.,0.5,1.3,paint[0]);
    }
    ellipsoid(0.,18.9,0.25,2.1,3.2,1.8,paint[0]);
    // Broad feline cheeks, muzzle and narrow forward-facing eyes.
    ellipsoid(0.,23.3,0.5,3.05,2.7,2.05,paint[0]);
    for sign in [-1.,1.] {
        ellipsoid(sign*0.85,22.5,2.15,0.95,0.6,0.65,paint[0]);
        ellipsoid(sign*1.2,24.,2.27,0.65,0.30,0.15,paint[1]);
        ellipsoid(sign*1.2,24.,2.40,0.12,0.25,0.08,paint[2]);
    }
    ellipsoid(0.,23.,2.65,0.35,0.23,0.18,paint[1]);
    ellipsoid(0.,25.1,2.05,0.35,0.45,0.16,paint[1]);
    for sign in [-1.,1.] {
        ear(out,[at(sign*1.1,24.8,0.),at(sign*3.,24.8,0.8),
            at(sign*2.8,28.,0.),at(sign*2.4,24.8,-0.8)],[paint[0],paint[1]]);
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
