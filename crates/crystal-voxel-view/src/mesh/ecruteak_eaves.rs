//! Closed fascia, rake boards and exposed rafter tails under intact roofs.
use super::*;

fn beam(mesh: &mut SurfaceMeshData, a: Vec3, b: Vec3, width: f32, depth: f32,
    color: [f32;3], plot: [f32;4], t: f32, painted: bool) {
    let along=(b-a).normalize();
    let up=along.cross(Vec3::X).normalize();
    let cut=(t*0.008).min(width.min(depth)*0.2);
    let (x,y)=(width*0.5,depth*0.5);
    let profile=[(-x,-y+cut),(-x+cut,-y),(x-cut,-y),(x,-y+cut),
        (x,y-cut),(x-cut,y),(-x+cut,y),(-x,y-cut)];
    let rings=[a,b].map(|p|profile.map(|(x,y)|p+Vec3::X*x+up*y));
    let center=(a+b)*0.5;
    let mut faces=vec![rings[0].to_vec(),rings[1].to_vec()];
    for i in 0..8 {let j=(i+1)%8;faces.push(vec![rings[0][i],rings[1][i],rings[1][j],rings[0][j]]);}
    for mut points in faces {
        let normal=(points[1]-points[0]).cross(points[2]-points[0]);
        if normal.dot(points.iter().copied().sum::<Vec3>()/points.len() as f32-center)<0. {points.reverse();}
        let first=mesh.positions.len();
        face(mesh,&points.iter().map(|p|p.to_array()).collect::<Vec<_>>(),color);
        if painted {for i in first..mesh.positions.len() {
            let p=Vec3::from_array(mesh.positions[i]);
            mesh.uvs[i]=[(p.x-plot[0])/t,
                crate::scenery_finish::TOWN_TIMBER_BASE+(p-a).dot(along).max(0.)/t];
        }}
    }
}

pub(super) fn append(mesh: &mut SurfaceMeshData, roof: [f32;4], wall: [f32;4],
    t: f32, eave: f32, ridge: f32, color: [f32;3], painted: bool) {
    let [left,right,back,front]=roof;
    let middle=(back+front)*0.5;
    let height=|z:f32|eave+(ridge-eave)*(1.-((z-middle)/((front-back)*0.5)).abs());
    // Fascia closes the thin deck edge, entirely within the roof footprint.
    for z in [back,front-t*0.10] {
        timber::block(mesh,[left,right,eave-t*0.24,eave-t*0.08,z,z+t*0.10],
            color,t,roof,painted);
    }
    // Sloped rake boards support both gable edges below the ceramic backing.
    for x in [left+t*0.05,right-t*0.05] {for (a,b) in [(back,middle),(middle,front)] {
        beam(mesh,Vec3::new(x,height(a+t*0.08)-t*0.17,a+t*0.08),
            Vec3::new(x,height(b-t*0.08)-t*0.17,b-t*0.08),t*0.10,t*0.12,color,roof,t,painted);
    }}
    // Tails continue from the wall plate to the fascia; no new posts or props.
    let count=((right-left)/(t*0.65)).ceil() as usize;
    for i in 0..count {
        let x=left+(right-left)*(i as f32+0.5)/count as f32;
        for (a,b) in [(back+t*0.06,wall[2]+t*0.10),(wall[3]-t*0.10,front-t*0.06)] {
            if b<=a {continue;}
            beam(mesh,Vec3::new(x,height(a)-t*0.20,a),Vec3::new(x,height(b)-t*0.20,b),
                t*0.07,t*0.10,color,roof,t,painted);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ecruteak_eaves_are_closed_connected_to_wall_plates_and_inside_the_roof_footprint() {
        for t in [8.,16.] {for width in [8.,12.] {
            let roof=[-width*t/2.,width*t/2.,-3.*t,3.*t];
            let wall=[roof[0]+t*0.24,roof[1]-t*0.24,roof[2]+t*0.28,roof[3]-t*0.30];
            let mut mesh=SurfaceMeshData::default();
            append(&mut mesh,roof,wall,t,2.7*t,4.2*t,EDGE,true);
            assert!(mesh.indices.len()/3<1300);
            assert_eq!(mesh.indices.len()%84,0);
            for p in &mesh.positions {
                assert!(p[0]>=roof[0] && p[0]<=roof[1] && p[2]>=roof[2] && p[2]<=roof[3]);
                assert!(p[1]>=2.4*t && p[1]<4.2*t);
            }
            for ids in mesh.indices.chunks_exact(84) {
                let center=ids.iter().map(|&i|Vec3::from_array(mesh.positions[i as usize])).sum::<Vec3>()/ids.len() as f32;
                let mut edges=std::collections::BTreeMap::new();let mut volume=0.;
                for tri in ids.chunks_exact(3) {
                    let p=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
                    let n=(p[1]-p[0]).cross(p[2]-p[0]);
                    assert!(n.dot((p[0]+p[1]+p[2])/3.-center)>0.);
                    volume+=(p[0]-center).dot((p[1]-center).cross(p[2]-center))/6.;
                    for j in 0..3 {let mut edge=[p[j].to_array().map(f32::to_bits),p[(j+1)%3].to_array().map(f32::to_bits)];edge.sort();*edges.entry(edge).or_insert(0)+=1;}
                }
                assert!(volume>0.);assert!(edges.values().all(|&n|n==2));
            }
            for tail in mesh.indices[6*84..].chunks_exact(84) {
                assert!(tail.iter().any(|&i| {
                    let p=mesh.positions[i as usize];
                    p[0]>=wall[0] && p[0]<=wall[1] && p[2]>=wall[2] && p[2]<=wall[3]
                        && p[1]<2.7*t
                }),"every tail must intersect the existing wall plate");
            }
            assert!(mesh.uvs.iter().all(|&uv|crate::scenery_finish::town_timber_coordinates(uv).is_some()));
        }}
    }
}
