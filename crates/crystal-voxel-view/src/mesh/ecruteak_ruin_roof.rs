//! Broken ceramic courses and surviving charred rafters for the shared ruin.
use super::*;

fn missing(side: usize, row: usize, col: usize) -> bool {
    // Two irregular burned-through areas; their stepped rims expose the
    // framing and the existing dark interior below, rather than a roof decal.
    if side==1 {
        matches!((row,col),(2,5..=6)|(3,4..=7)|(4,3..=7)|(5,4..=8)|(6,5..=7))
    } else {
        matches!((row,col),(5,6..=7)|(6,5..=8)|(7,6..=8))
    }
}

pub(super) fn append(mesh: &mut SurfaceMeshData,b:[f32;4],t:f32,eave:f32,ridge:f32) {
    let [left,right,back,front]=b;let middle=(back+front)*0.5;
    let height=|z:f32|eave+(ridge-eave)*(1.-((z-middle)/((front-back)*0.5)).abs());
    let origin=[left,back];
    // Closed timber members remain below the original roof skin. Their
    // exposed portions cross the openings without filling them back in.
    for side in 0..2 {
        let (a,z)=if side==0 {(back,middle)}else{(middle,front)};
        for col in [2,5,8,10] {
            let x=left+(right-left)*(col as f32+0.5)/12.;
            let ring=|z:f32| {
                let y=height(z)-t*0.10;
                [Vec3::new(x-t*0.055,y,z),Vec3::new(x+t*0.055,y,z),
                 Vec3::new(x+t*0.055,y-t*0.12,z),Vec3::new(x-t*0.055,y-t*0.12,z)]
            };
            roof::solid(mesh,[ring(a+t*0.003),ring(z-t*0.003)],[[0.045,0.035,0.025];3],origin,t,false);
        }
        for row in 0..8 {for col in 0..=12 {
            if missing(side,row,col) {continue;}
            let za=a+(z-a)*row as f32/8.+t*0.006;
            let zb=a+(z-a)*(row+1) as f32/8.-t*0.006;
            let shift=if row%2==0 {0.}else{0.5};
            let xa=(left+(right-left)*(col as f32-shift)/12.).max(left)+t*0.006;
            let xb=(left+(right-left)*(col as f32+1.-shift)/12.).min(right)-t*0.006;
            if xb-xa<t*0.08 {continue;}
            let broken=[(row.saturating_sub(1),col),(row+1,col),
                (row,col.saturating_sub(1)),(row,col+1)].into_iter().any(|(r,c)|missing(side,r,c));
            let cut=(t*if broken {0.09+((row*7+col*3+side)%5) as f32*0.018}else{0.008})
                .min((xb-xa)*0.32);
            let outline=[(xa,za),(xb-cut,za),(xb,za+cut),(xb,zb),(xa+cut,zb),(xa,zb-cut)];
            let wear=t*((row*13+col*7+side)%5) as f32*0.005;
            let ring=|depth:f32|outline.map(|(x,z)|Vec3::new(x,height(z)-depth-wear,z));
            let value=0.83+((row*17+col*11+side*7)%13) as f32*0.024;
            let slate=[0.11,0.108,0.097].map(|v|v*value);
            let edge=[0.064,0.060,0.050];
            roof::solid(mesh,[ring(0.),ring(t*0.065)],[edge,slate,edge],origin,t,false);
        }}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn burned_courses_leave_real_openings_above_closed_framing_and_keep_the_roof_envelope() {
        for t in [8.,16.] {
            let b=[-3.9*t,3.9*t,-3.9*t,3.88*t];let eave=3.5*t;let ridge=3.95*t;
            let mut mesh=SurfaceMeshData::default();append(&mut mesh,b,t,eave,ridge);
            let middle=(b[2]+b[3])*0.5;
            let roof_height=|z:f32|eave+(ridge-eave)*(1.-((z-middle)/((b[3]-b[2])*0.5)).abs());
            for p in &mesh.positions {
                assert!(p[0]>=b[0] && p[0]<=b[1] && p[2]>=b[2] && p[2]<=b[3]);
                assert!(p[1]<=roof_height(p[2])+0.00001 && p[1]>=eave-t*0.23);
            }
            let mut edges=std::collections::BTreeMap::new();
            for tri in mesh.indices.chunks_exact(3) {
                let p=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
                let n=(p[1]-p[0]).cross(p[2]-p[0]);
                assert!(n.dot(Vec3::from_array(mesh.normals[tri[0] as usize]))>0.);
                for i in 0..3 {
                    let mut key=[p[i].to_array().map(f32::to_bits),p[(i+1)%3].to_array().map(f32::to_bits)];
                    key.sort();*edges.entry(key).or_insert(0)+=1;
                }
            }
            assert!(edges.values().all(|&n|n==2),"every surviving tile and rafter is closed");
            // Center of the large front opening, between surviving rafters.
            let q=Vec2::new(b[0]+(b[1]-b[0])*6.25/12.,middle+(b[3]-middle)*4.5/8.);
            for tri in mesh.indices.chunks_exact(3) {
                let p=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
                let [a,b,c]=p.map(|p|Vec2::new(p.x,p.z));
                let signs=[(b-a).perp_dot(q-a),(c-b).perp_dot(q-b),(a-c).perp_dot(q-c)];
                assert!(!(signs.iter().all(|v|*v>0.00001)||signs.iter().all(|v|*v< -0.00001)),"burn-through is a real aperture");
            }
        }
    }
}
