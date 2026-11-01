//! Original raised blue flowerbeds, colored only by their complete live drawing.
use super::*;

pub(super) fn append(
    out: &mut SurfaceMeshData, bounds: [f32;4], k: f32, rows: usize,
    uv: impl Fn([usize;2]) -> [f32;2],
) {
    let [w,e,n,s] = bounds;
    let pot_y = rows*8-8;
    let gold = [1,pot_y+2];
    let cream = [1,pot_y+1];
    let soil = [0,pot_y];
    // Closed body, inset soil, four substantial rim courses and a fine pale
    // front stripe retain the native yellow/cream planter silhouette.
    block(out,[w+k,e-k,n+k,s-k],0.,3.8*k,gold,&uv);
    block(out,[w+1.5*k,e-1.5*k,n+1.5*k,s-1.5*k],3.8*k,4.*k,soil,&uv);
    for b in [[w+0.5*k,e-0.5*k,n+0.5*k,n+1.5*k],
        [w+0.5*k,e-0.5*k,s-1.5*k,s-0.5*k],
        [w+0.5*k,w+1.5*k,n+1.5*k,s-1.5*k],
        [e-1.5*k,e-0.5*k,n+1.5*k,s-1.5*k]] {
        block(out,b,3.8*k,4.6*k,cream,&uv);
    }
    block(out,[w+k,e-k,s-1.05*k,s-0.95*k],1.2*k,1.8*k,cream,&uv);
    let courses = rows*2-1;
    for row in 0..courses {
        for column in 0..2 {
            let ordinal = row*2+column;
            let x = w+(e-w)*(0.29+column as f32*0.42)
                + if row%2==0 {0.25*k} else {-0.25*k};
            let z = n+3.5*k+(s-n-7.*k)*row as f32/(courses-1) as f32;
            let head = Vec3::new(x,(9.2+(ordinal%3) as f32*0.7)*k,z);
            let root = Vec3::new(x,4.*k,z);
            let mid = (head+root)*0.5;
            petal(out,[mid+Vec3::X*0.14*k,mid+Vec3::Z*0.14*k,
                mid-Vec3::X*0.14*k,mid-Vec3::Z*0.14*k,head,root],
                [11,3],&uv);
            // Broad paired leaves fill the bed below the blooms, rather than
            // leaving a large empty soil surface with a few tiny flowers.
            for sign in [-1.,1.] {
                let center = Vec3::new(x+sign*1.2*k,6.*k,z);
                petal(out,[center+Vec3::X*1.5*k,center+Vec3::Z*0.75*k,
                    center-Vec3::X*1.5*k,center-Vec3::Z*0.75*k,
                    center+Vec3::Y*0.7*k,center-Vec3::Y*0.3*k],
                    [10,3],&uv);
            }
            for i in 0..5 {
                let a = i as f32*std::f32::consts::TAU/5.+ordinal as f32*0.37;
                let along = Vec3::new(a.cos(),0.,a.sin());
                let across = Vec3::new(-a.sin(),0.,a.cos());
                let center = head+along*1.1*k;
                petal(out,[center+along*1.25*k,center+across*0.9*k,
                    center-along*1.25*k,center-across*0.9*k,
                    center+Vec3::Y*0.65*k,center-Vec3::Y*0.35*k],
                    if i%3==0 {[3,4]} else {[4,4]},&uv);
            }
            let center = head+Vec3::Y*0.45*k;
            petal(out,[center+Vec3::X*0.48*k,center+Vec3::Z*0.48*k,
                center-Vec3::X*0.48*k,center-Vec3::Z*0.48*k,
                center+Vec3::Y*0.4*k,center-Vec3::Y*0.3*k],[1,1],&uv);
        }
    }
}

fn triangle(out:&mut SurfaceMeshData, mut p:[Vec3;3], center:Vec3,
    pixel:[usize;2],uv:&impl Fn([usize;2])->[f32;2]) {
    let mut normal = (p[1]-p[0]).cross(p[2]-p[0]).normalize();
    if normal.dot((p[0]+p[1]+p[2])/3.-center)<0. {p.swap(1,2);normal=-normal;}
    let first=out.positions.len() as u32;
    out.positions.extend(p.map(|point|point.to_array()));
    out.normals.extend([normal.to_array();3]);
    out.uvs.extend([uv(pixel);3]);
    out.colors.extend([crate::interior_models::authored_face_color([1.;4],normal);3]);
    out.indices.extend([first,first+1,first+2]);
}
fn petal(out:&mut SurfaceMeshData,p:[Vec3;6],pixel:[usize;2],
    uv:&impl Fn([usize;2])->[f32;2]) {
    let center=p.into_iter().sum::<Vec3>()/6.;
    for ids in [[0,1,4],[1,2,4],[2,3,4],[3,0,4],
        [1,0,5],[2,1,5],[3,2,5],[0,3,5]] {
        triangle(out,ids.map(|i|p[i]),center,pixel,uv);
    }
}
fn block(out:&mut SurfaceMeshData,b:[f32;4],lo:f32,hi:f32,pixel:[usize;2],
    uv:&impl Fn([usize;2])->[f32;2]) {
    let [w,e,n,s]=b;
    let points=[[w,lo,n],[e,lo,n],[e,lo,s],[w,lo,s],
        [w,hi,n],[e,hi,n],[e,hi,s],[w,hi,s]].map(Vec3::from_array);
    let center=points.into_iter().sum::<Vec3>()/8.;
    for ids in [[0,1,2,3],[4,5,6,7],[0,1,5,4],
        [1,2,6,5],[2,3,7,6],[3,0,4,7]] {
        for t in [[ids[0],ids[1],ids[2]],[ids[0],ids[2],ids[3]]] {
            triangle(out,t.map(|i|points[i]),center,pixel,uv);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn celadon_blue_flowerbed_has_closed_parts_and_source_palette_inside_plot() {
        for rows in [2,4] {for k in [0.5,1.,2.] {
            let b=[-8.*k,8.*k,-8.*rows as f32*k,0.];
            let mut out=SurfaceMeshData::default();
            append(&mut out,b,k,rows,|[x,y]|[x as f32,y as f32]);
            assert_eq!(out.indices.len(),(7*12+9*8*2*(rows*2-1))*3);
            assert!(out.positions.iter().any(|p|p[1]>10.*k));
            for p in &out.positions {
                assert!(p.iter().all(|v|v.is_finite()));
                assert!(p[0]>=b[0]&&p[0]<=b[1]&&p[2]>=b[2]&&p[2]<=b[3]);
                assert!(p[1]>=0.&&p[1]<=12.*k);
            }
            for n in &out.normals {assert!((Vec3::from_array(*n).length()-1.).abs()<1e-5);}
            for i in (0..out.positions.len()).step_by(3) {
                let p=out.positions[i..i+3].iter().copied().map(Vec3::from_array).collect::<Vec<_>>();
                assert!((p[1]-p[0]).cross(p[2]-p[0]).dot(Vec3::from_array(out.normals[i]))>0.);
            }
            let mut first=0;
            for triangles in std::iter::repeat_n(12,7)
                .chain(std::iter::repeat_n(8,9*2*(rows*2-1))) {
                let mut edges=std::collections::BTreeMap::new();
                let mut volume=0.;
                for p in out.positions[first..first+triangles*3].chunks_exact(3) {
                    let v=p.iter().copied().map(Vec3::from_array).collect::<Vec<_>>();
                    volume+=v[0].dot(v[1].cross(v[2]))/6.;
                    let key=|p:[f32;3]|p.map(|v|(v/k*10000.).round() as i32);
                    for (a,b) in [(0,1),(1,2),(2,0)] {
                        let a=key(p[a]);let b=key(p[b]);
                        *edges.entry(if a<b {(a,b)} else {(b,a)}).or_insert(0)+=1;
                    }
                }
                assert!(volume>0.,"each authored planter, stem, leaf and petal is closed and outward");
                assert!(edges.values().all(|&n|n==2));
                first+=triangles*3;
            }
            for uv in &out.uvs {assert!(uv[0]>=0.&&uv[0]<16.&&uv[1]>=0.&&uv[1]<rows as f32*8.);}
            for pixel in [[3.,4.],[4.,4.],[1.,1.],[10.,3.],[11.,3.],
                [1.,(rows*8-6) as f32]] {assert!(out.uvs.contains(&pixel));}
        }}
    }
}
