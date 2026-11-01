//! Closed, chamfered timber members inside their existing authored bounds.
use super::*;

pub(super) fn block(mesh: &mut SurfaceMeshData, b: [f32;6], color: [f32;3],
    t: f32, plot: [f32;4], painted: bool) {
    let low=[b[0],b[2],b[4]];let high=[b[1],b[3],b[5]];
    let span=std::array::from_fn::<_,3,_>(|a|high[a]-low[a]);
    if span.iter().any(|&v|v<=0.) {super::block(mesh,b,color);return;}
    let axis=(0..3).max_by(|&a,&b|span[a].total_cmp(&span[b])).unwrap();
    let cross=(0..3).filter(|&a|a!=axis).collect::<Vec<_>>();
    let cut=(t*0.008).min(span[cross[0]].min(span[cross[1]])*0.2);
    let a=low[cross[0]];let z=low[cross[1]];let e=high[cross[0]];let s=high[cross[1]];
    let outline=[(a+cut,z),(e-cut,z),(e,z+cut),(e,s-cut),
        (e-cut,s),(a+cut,s),(a,s-cut),(a,z+cut)];
    let mut points=Vec::new();
    for end in [low[axis],high[axis]] {for &(u,v) in &outline {
        let mut p=[0.;3];p[axis]=end;p[cross[0]]=u;p[cross[1]]=v;points.push(Vec3::from_array(p));
    }}
    let center=(Vec3::from_array(low)+Vec3::from_array(high))*0.5;
    let mut faces=vec![(0..8).collect::<Vec<_>>(),(8..16).collect::<Vec<_>>()];
    for i in 0..8 {faces.push(vec![i,(i+1)%8,(i+1)%8+8,i+8]);}
    for (face_index,ids) in faces.into_iter().enumerate() {
        let mut p=ids.iter().map(|&i|points[i]).collect::<Vec<_>>();
        let n=(p[1]-p[0]).cross(p[2]-p[0]);
        let middle=p.iter().copied().sum::<Vec3>()/p.len() as f32;
        if n.dot(middle-center)<0. {p.reverse();}
        let first=mesh.positions.len();
        face(mesh,&p.iter().map(|v|v.to_array()).collect::<Vec<_>>(),color);
        if painted {for i in first..mesh.positions.len() {
            let p=mesh.positions[i];
            let local=[(p[0]-plot[0])/t,p[1]/t,(p[2]-plot[2])/t];
            let across=if matches!(face_index,0|1|2|3|6|7) {cross[0]}else{cross[1]};
            mesh.uvs[i]=[local[across],crate::scenery_finish::TOWN_TIMBER_BASE+local[axis]];
        }}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chamfered_members_are_closed_outward_and_preserve_all_three_axis_bounds() {
        for b in [[0.,8.,0.,1.,0.,1.],[0.,1.,0.,8.,0.,1.],[0.,1.,0.,1.,0.,8.]] {
            let mut m=SurfaceMeshData::default();block(&mut m,b,EDGE,1.,[0.,8.,0.,8.],true);
            let low=[b[0],b[2],b[4]];let high=[b[1],b[3],b[5]];
            for a in 0..3 {
                assert_eq!(m.positions.iter().map(|p|p[a]).fold(f32::INFINITY,f32::min),low[a]);
                assert_eq!(m.positions.iter().map(|p|p[a]).fold(f32::NEG_INFINITY,f32::max),high[a]);
            }
            let center=(Vec3::from_array(low)+Vec3::from_array(high))*0.5;
            let mut edges=std::collections::BTreeMap::new();let mut volume=0.;
            for ids in m.indices.chunks_exact(3) {
                let p=[ids[0],ids[1],ids[2]].map(|i|Vec3::from_array(m.positions[i as usize]));
                assert!((p[1]-p[0]).cross(p[2]-p[0]).dot((p[0]+p[1]+p[2])/3.-center)>0.);
                volume+=(p[0]-center).dot((p[1]-center).cross(p[2]-center))/6.;
                for i in 0..3 {let mut key=[p[i].to_array().map(f32::to_bits),p[(i+1)%3].to_array().map(f32::to_bits)];key.sort();*edges.entry(key).or_insert(0)+=1;}
            }
            assert!(volume>0.);assert!(edges.values().all(|&n|n==2));
            assert_eq!(m.indices.len()/3,28);
            assert!(m.uvs.iter().all(|&uv|crate::scenery_finish::town_timber_coordinates(uv).is_some()));
        }
    }

    #[test]
    fn timber_grain_coordinates_follow_members_when_the_scene_origin_moves() {
        let mut a=SurfaceMeshData::default();let mut b=SurfaceMeshData::default();
        block(&mut a,[1.,1.2,0.2,3.,1.,1.12],EDGE,1.,[0.,8.,0.,8.],true);
        block(&mut b,[129.,129.2,0.2,3.,-63.,-62.88],EDGE,1.,[128.,136.,-64.,-56.],true);
        for (a,b) in a.uvs.iter().zip(&b.uvs) {
            assert!((a[0]-b[0]).abs()<0.00002 && (a[1]-b[1]).abs()<0.0002);
        }
        for (a,b) in a.normals.iter().zip(&b.normals) {
            assert!(Vec3::from_array(*a).distance(Vec3::from_array(*b))<0.001);
        }
    }
}
