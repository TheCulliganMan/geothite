//! Rooted broadleaf forest trees, distinct from Johto pines and park gardens.
use super::*;

fn leaf(mesh:&mut SurfaceMeshData,center:Vec3,axis:Vec3,normal:Vec3,length:f32,tint:[f32;3]) {
    let side=axis.cross(normal).normalize();
    let outline=[(-1.,0.),(-0.5,0.7),(0.55,0.65),(1.,0.),(0.55,-0.65),(-0.5,-0.7)];
    let mut points=[center;8];
    for (i,(along,across)) in outline.into_iter().enumerate() {
        points[i]=center+axis*along*length+side*across*length*0.38;
    }
    points[6]=center+normal*length*0.12;
    points[7]=center-normal*length*0.025;
    let start=mesh.positions.len() as u32;
    let mut normals=[Vec3::ZERO;8];
    for i in 0..6 {for apex in [6,7] {
        let mut ids=[apex,i,(i+1)%6];
        let mut n=(points[ids[1]]-points[ids[0]]).cross(points[ids[2]]-points[ids[0]]);
        let face=(points[ids[0]]+points[ids[1]]+points[ids[2]])/3.;
        if n.dot(face-center)<0. {ids.swap(1,2);n=-n;}
        mesh.indices.extend(ids.map(|i|start+i as u32));
        for i in ids {normals[i]+=n;}
    }}
    for (point,normal) in points.into_iter().zip(normals) {
        let n=normal.normalize();
        let sun=n.dot(Vec3::new(0.45,0.8,0.55).normalize()).max(0.);
        mesh.positions.push(point.to_array());mesh.normals.push(n.to_array());mesh.uvs.push([0.;2]);
        // Ilex's source atlas keeps its cool forest palette even at noon.
        mesh.colors.push([(0.012+sun*0.035)*tint[0],(0.10+sun*0.14)*tint[1],
            (0.16+sun*0.16)*tint[2],1.]);
    }
}

fn branch(mesh:&mut SurfaceMeshData,root:Vec3,tip:Vec3,radius:f32) {
    let first=mesh.colors.len();
    append_branch(mesh,root,tip,radius);
    for color in &mut mesh.colors[first..] {
        color[0]*=0.15/0.24;color[1]*=0.085/0.15;color[2]*=0.18/0.075;
    }
}

pub(super) fn append(mesh:&mut TerrainMeshData,bounds:[f32;4],base:f32,tile:f32,
    coordinates:[i32;2],neighbors:[bool;4]) {
    let first=mesh.solid.positions.len();
    let spacing=(bounds[1]-bounds[0]).min(bounds[3]-bounds[2]);
    let v=variation(coordinates[0],coordinates[1]);
    let offset=grove_trunk_offset(coordinates,spacing);
    let center=Vec3::new((bounds[0]+bounds[1])*0.5+offset.x,base,(bounds[2]+bounds[3])*0.5+offset.y);
    let sparse=neighbors.iter().all(|&n|n)
        && (coordinates[0].rem_euclid(4)!=0 || coordinates[1].rem_euclid(4)!=0);
    if !sparse {
        let height=tile*(6.1+v.height*0.5);
        let phase=v.turn+v.height*5.31;
        let lean=Vec3::new(phase.cos(),0.,phase.sin())*spacing*0.08;
        let bend=Vec3::new(-phase.sin(),0.,phase.cos())*spacing*0.055;
        let stem=|t:f32|center+Vec3::Y*height*t+lean*t*t
            +bend*t*(t*std::f32::consts::PI*1.4).sin();
        let trunk_point=|t:f32,a:f32| {
            let radial=Vec3::new(a.cos(),0.,a.sin());
            let radius=spacing*0.21*(1.-t*0.38)
                *(1.+(1.-t).powi(8)*(0.40+0.12*(a*5.+phase).cos()))
                *(1.+0.065*(a*9.+phase+t*1.7).sin());
            stem(t*0.7)+radial*radius
        };
        let start=mesh.solid.positions.len() as u32;
        // A substantial continuous bole, rooted buttresses and fluted bark.
        for row in 0..=10 {for ring in 0..=16 {
            let t=row as f32/10.;let a=(ring%16) as f32*std::f32::consts::TAU/16.;
            let vertical=trunk_point(t+0.001,a)-trunk_point(t-0.001,a);
            let around=trunk_point(t,a+0.002)-trunk_point(t,a-0.002);
            let normal=vertical.cross(around).normalize();
            mesh.solid.positions.push(trunk_point(t,a).to_array());
            mesh.solid.normals.push(normal.to_array());mesh.solid.uvs.push([0.;2]);
            let grain=0.84+0.12*(a*9.+phase+t*1.1).cos()+0.035*(a*17.-t*2.3).sin();
            mesh.solid.colors.push([0.15*grain,0.085*grain,0.18*grain,1.]);
        }}
        for row in 0..10 {for ring in 0..16 {
            let a=start+row*17+ring;let b=a+17;
            mesh.solid.indices.extend([a,b,a+1,a+1,b,b+1]);
        }}
        // Unequal spreading limbs support overlapping leaf layers, not a cone
        // or one closed crown shell. Broad foliage leaves the trunk readable.
        for bough in 0..[6,9,8][v.index] {
            let turn=phase+bough as f32*2.399963;
            let direction=Vec3::new(turn.cos(),0.,turn.sin());
            let across=Vec3::new(-direction.z,0.,direction.x);
            let root=stem(0.38+bough as f32*0.038);
            let tip=stem(0.64+bough as f32*0.027)+direction*spacing*(0.6+0.08*turn.sin());
            branch(&mut mesh.solid,root,tip,spacing*0.043);
            for fork in [-1.,0.,1.] {
                let end=tip+across*fork*spacing*0.22+direction*spacing*0.19;
                branch(&mut mesh.solid,root.lerp(tip,0.62),end,spacing*0.017);
                for layer in 0..3 {for spray in 0..4 {
                    let seed=turn+fork+layer as f32*1.9+spray as f32*2.399963;
                    let axis=Vec3::new(seed.cos(),0.07*(seed*1.7).sin(),seed.sin()).normalize();
                    let point=end+axis*spacing*(0.12+0.07*layer as f32)
                        +Vec3::Y*tile*(layer as f32-1.)*0.16;
                    leaf(&mut mesh.solid,point,axis,(Vec3::Y+axis*0.18).normalize(),
                        spacing*(0.18+0.035*(seed*1.3).sin()),v.tint);
                }}
            }
        }
        // Keep the existing authored source plot and its readable crown margin.
        let limits=[bounds[0]-spacing*0.75,bounds[1]+spacing*0.75,
            bounds[2]-spacing*0.75,bounds[3]+spacing*0.75];
        let mut fit=1f32;
        for p in &mesh.solid.positions[first..] {
            for (axis,origin,low,high) in [(0,center.x,limits[0],limits[1]),(2,center.z,limits[2],limits[3])] {
                let delta=p[axis]-origin;
                if delta>0. {fit=fit.min((high-origin)/delta);}
                if delta<0. {fit=fit.min((low-origin)/delta);}
            }
        }
        for i in first..mesh.solid.positions.len() {
            let p=&mut mesh.solid.positions[i];
            p[0]=center.x+(p[0]-center.x)*fit;p[2]=center.z+(p[2]-center.z)*fit;
            let n=Vec3::from_array(mesh.solid.normals[i]);
            mesh.solid.normals[i]=Vec3::new(n.x/fit,n.y,n.z/fit).normalize().to_array();
        }
    }
    crate::understory::append(&mut mesh.solid,bounds,base,tile,coordinates);
    mesh.solid.cutaway_ranges.push(first..mesh.solid.positions.len());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ancient_leaves_are_closed_broad_surfaces_with_outward_winding() {
        let mut mesh=SurfaceMeshData::default();
        leaf(&mut mesh,Vec3::ZERO,Vec3::X,Vec3::Y,1.,[1.;3]);
        assert_eq!(mesh.positions.len(),8);assert_eq!(mesh.indices.len(),36);
        let mut edges=std::collections::BTreeMap::new();
        for ids in mesh.indices.chunks_exact(3) {
            let p=[ids[0],ids[1],ids[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
            assert!((p[1]-p[0]).cross(p[2]-p[0]).dot((p[0]+p[1]+p[2])/3.)>0.);
            for (a,b) in [(ids[0],ids[1]),(ids[1],ids[2]),(ids[2],ids[0])] {
                *edges.entry((a.min(b),a.max(b))).or_insert(0)+=1;
            }
        }
        assert!(edges.values().all(|&count|count==2));
        assert!(mesh.normals.iter().all(|n|Vec3::from_array(*n).is_finite()));
    }
    #[test]
    fn ancient_forest_keeps_source_roots_bounds_and_distinct_geometry() {
        for coordinates in [[0,0],[4,8],[-7,3],[6,8]] {for neighbors in [[false;4],[true;4]] {
            let mut forest=TerrainMeshData::default();
            append(&mut forest,[0.,32.,0.,32.],3.,16.,coordinates,neighbors);
            assert_eq!(forest.solid.cutaway_ranges,vec![0..forest.solid.positions.len()]);
            assert!(forest.solid.positions.len()<u16::MAX as usize);
            assert!(forest.solid.positions.iter().all(|p|p.iter().all(|v|v.is_finite())
                && p[0]>=-24.001 && p[0]<=56.001 && p[2]>=-24.001 && p[2]<=56.001
                && p[1]>=2.9 && p[1]<3.+16.*9.));
            let mut pine=TerrainMeshData::default();
            super::super::pine::append(&mut pine,[0.,32.,0.,32.],3.,16.,coordinates,neighbors);
            if neighbors!=[true;4] || coordinates==[0,0] || coordinates==[4,8] {
                assert_ne!(forest.solid.positions,pine.solid.positions);
            }
            let mut repeat=TerrainMeshData::default();
            append(&mut repeat,[0.,32.,0.,32.],3.,16.,coordinates,neighbors);
            assert_eq!(forest.solid,repeat.solid);
        }}
    }
}
