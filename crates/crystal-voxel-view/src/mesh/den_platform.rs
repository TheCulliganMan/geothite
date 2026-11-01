//! Rounded, closed rims for source-complete Dragon's Den rock platforms.
use super::*;
use bevy::math::Vec2;
use std::collections::{BTreeMap, HashSet};

pub(super) fn cells(cells: &[&VisualTile], g: &GridGeometry) -> HashSet<usize> {
    let drawing = [0x2b,0x2c,0x2c,0x2d,0x3b,0x3c,0x3c,0x3d,
        0x3b,0x3c,0x3c,0x3d,0x4b,0x4c,0x4c,0x4d];
    let mut result = HashSet::new();
    for row in 0..g.height.saturating_sub(3) { for col in 0..g.width.saturating_sub(3) {
        if (0..16).all(|j| {
            let s=&cells[(row+j/4)*g.width+col+j%4].source;
            s.tileset_id.as_ref()=="johto" && s.metatile_id==0x0a
                && s.subtile_column as usize==j%4 && s.subtile_row as usize==j/4
                && s.tile_index==drawing[j]
        }) {result.extend((0..16).map(|j|(row+j/4)*g.width+col+j%4));}
    }}
    result
}

fn key(p: Vec3) -> [i32;3] {p.to_array().map(|v|(v*10000.).round() as i32)}

fn distance(p: Vec3, edges: &[(Vec3,Vec3)]) -> f32 {
    edges.iter().map(|&(a,b)| {
        let a=Vec2::new(a.x,a.z); let b=Vec2::new(b.x,b.z);
        let p=Vec2::new(p.x,p.z); let ab=b-a;
        let t=((p-a).dot(ab)/ab.length_squared()).clamp(0.,1.);
        (p-a-ab*t).length()
    }).fold(f32::INFINITY,f32::min)
}

fn rounded(p: Vec3, edges: &[(Vec3,Vec3)], joins: &[(Vec3,Vec3)], radius: f32) -> Vec3 {
    let d=distance(p,edges).min(radius);
    // Circular cross-section meets the vertical face tangentially and settles
    // onto the unchanged cap before reaching a walking cell's centre.
    let t=(distance(p,joins)/radius).clamp(0.,1.);
    let blend=t*t*(3.-2.*t);
    let drop=radius-(radius*radius-(radius-d).powi(2)).max(0.).sqrt();
    Vec3::new(p.x,p.y-drop*blend,p.z)
}

fn triangle(out: &mut SurfaceMeshData,p: [Vec3;3],g:&GridGeometry,origin:[i32;2],cap:bool) {
    let cross=(p[1]-p[0]).cross(p[2]-p[0]);
    if cross.length_squared()<1e-8 {return;}
    let n=cross.normalize();let first=out.positions.len() as u32;
    for point in p {
        out.positions.push(point.to_array());out.normals.push(n.to_array());
        out.uvs.push(if cap {cave_cliff::floor_uv(point,g,origin,false)}
            else {cave_cliff::stone_uv(point,g,origin,false)});
        out.colors.push(cave_cliff::pigment([0.34,0.345,0.31,1.],point,g,origin,false));
    }
    out.indices.extend([first,first+1,first+2]);
}

fn cap(out:&mut SurfaceMeshData,p:[Vec3;3],edges:&[(Vec3,Vec3)],joins:&[(Vec3,Vec3)],g:&GridGeometry,origin:[i32;2],depth:u8) {
    if depth==0 {
        triangle(out,p.map(|p|rounded(p,edges,joins,g.tile_width*0.25)),g,origin,true);
        return;
    }
    let [a,b,c]=p;let ab=(a+b)*0.5;let bc=(b+c)*0.5;let ca=(c+a)*0.5;
    for tri in [[a,ab,ca],[ab,b,bc],[ca,bc,c],[ab,bc,ca]] {cap(out,tri,edges,joins,g,origin,depth-1);}
}

fn underside(out:&mut SurfaceMeshData,p:[Vec3;3],g:&GridGeometry,origin:[i32;2],depth:u8) {
    if depth==0 {triangle(out,p,g,origin,false);return;}
    let [a,b,c]=p;let ab=(a+b)*0.5;let bc=(b+c)*0.5;let ca=(c+a)*0.5;
    for tri in [[a,ab,ca],[ab,b,bc],[ca,bc,c],[ab,bc,ca]] {underside(out,tri,g,origin,depth-1);}
}

pub(super) fn append(out:&mut SurfaceMeshData,caps:&[[Vec3;3]],footing:&[f32],g:&GridGeometry,origin:[i32;2]) {
    if caps.is_empty(){return;}
    let mut counts=BTreeMap::new();
    let mut real_caps:Vec<_>=caps.iter().filter(|tri|
        (tri[1]-tri[0]).cross(tri[2]-tri[0]).length_squared()>1e-8).copied().collect();
    for tri in &real_caps {for (a,b) in [(0,1),(1,2),(2,0)] {
        let mut k=[key(tri[a]),key(tri[b])];k.sort();
        let entry=counts.entry(k).or_insert((0,tri[a],tri[b]));entry.0+=1;
    }}
    // The generic platform cap clips a corner whenever the adjoining source
    // drawing differs. Complete that corner when an adjoining real bank
    // has the same raised datum. Its straight face otherwise ends at a
    // clipped cap, leaving a narrow slot through the connected formation.
    // Corners surrounded by lower water retain their authored chamfer.
    for &(_,a,b) in counts.values().filter(|&&(count,_,_)|count==1) {
        let dx=b.x-a.x;let dz=b.z-a.z;
        if dx.abs()<1e-4 || dz.abs()<1e-4 {continue;}
        let outward=(b-a).cross(Vec3::Y).normalize();let mid=(a+b)*0.5;
        let x=((mid.x-g.origin_x)/g.tile_width).floor() as i32;
        let z=((mid.z-g.origin_z)/g.tile_height).floor() as i32;
        let high=|x:i32,z:i32|x>=0 && z>=0 && x<g.width as i32 && z<g.height as i32
            && footing.get(z as usize*g.width+x as usize).is_some_and(|&h|h>=a.y-1e-4);
        if high(x+outward.x.signum() as i32,z) || high(x,z+outward.z.signum() as i32) {
            let corner=Vec3::new(if outward.x<0. {a.x.min(b.x)} else {a.x.max(b.x)},a.y,
                if outward.z<0. {a.z.min(b.z)} else {a.z.max(b.z)});
            real_caps.push([a,corner,b]);
        }
    }
    counts.clear();
    for tri in &real_caps {for (a,b) in [(0,1),(1,2),(2,0)] {
        let mut k=[key(tri[a]),key(tri[b])];k.sort();
        let entry=counts.entry(k).or_insert((0,tri[a],tri[b]));entry.0+=1;
    }}
    // Unknown viewport continuations stay flat and do not grow a cut face.
    let at_viewport=|a:Vec3,b:Vec3| {
        let east=g.origin_x+g.width as f32*g.tile_width;
        let south=g.origin_z+g.height as f32*g.tile_height;
        [(a.x,b.x,g.origin_x),(a.x,b.x,east),(a.z,b.z,g.origin_z),(a.z,b.z,south)]
            .iter().any(|&(a,b,line)|(a-line).abs()<1e-4 && (b-line).abs()<1e-4)
    };
    let cell=|p:Vec3| {
        let x=((p.x-g.origin_x)/g.tile_width).floor() as i32;
        let z=((p.z-g.origin_z)/g.tile_height).floor() as i32;
        (x>=0 && z>=0 && x<g.width as i32 && z<g.height as i32)
            .then(||z as usize*g.width+x as usize)
    };
    let joins_ground=|a:Vec3,b:Vec3| {
        let outward=(b-a).cross(Vec3::Y).normalize();let mid=(a+b)*0.5;
        let inside=cell(mid-outward*0.02);let outside=cell(mid+outward*0.02);
        // A clipped diagonal stays inside one source cell, whose centre is
        // high ground. That datum does not prove ground beyond the diagonal.
        outside!=inside && outside.and_then(|i|footing.get(i))
            .is_some_and(|&height|height>=a.y-1e-4)
    };
    let (joins,edges):(Vec<_>,Vec<_>)=counts.values().filter(|&&(count,_,_)|count==1)
        .map(|&(_,a,b)|(a,b)).partition(|&(a,b)|at_viewport(a,b) || joins_ground(a,b));
    let bottom=crate::profile::WATER_HEIGHT*g.tile_height/SOURCE_TILE_HEIGHT;
    for tri in real_caps {
        cap(out,tri,&edges,&joins,g,origin,3);
        // Seal the submerged underside using the original authored contour.
        underside(out,[tri[2],tri[1],tri[0]].map(|p|Vec3::new(p.x,bottom,p.z)),g,origin,3);
    }
    for &(a,b) in &edges {for j in 0..8 {
        let start=a.lerp(b,j as f32/8.);let end=a.lerp(b,(j+1) as f32/8.);
        let (a,b)=(start,end);
        let a=rounded(a,&edges,&joins,g.tile_width*0.25);let b=rounded(b,&edges,&joins,g.tile_width*0.25);
        let low_a=Vec3::new(a.x,bottom,a.z);let low_b=Vec3::new(b.x,bottom,b.z);
        // Boundary direction follows upward cap winding: the rock is left of
        // this edge, so the reversed side winding faces outward.
        triangle(out,[a,low_a,low_b],g,origin,false);
        triangle(out,[a,low_b,b],g,origin,false);
    }}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn den_rounded_platform_keeps_centres_and_seals_the_waterline() {
        let g=GridGeometry{width:6,height:6,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut raw=SurfaceMeshData::default();
        for z in 1..5 {for x in 1..5 {
            let (w,e,n,s)=g.bounds(x,z);
            append_rock_platform_top(&mut raw,[w,e,n,s],16.,g.uv(x,z),
                (x-1) as u8,(z-1) as u8,[true;4],[1.;4]);
        }}
        let caps:Vec<[Vec3;3]>=raw.indices.chunks_exact(3).map(|i|
            std::array::from_fn(|j|Vec3::from_array(raw.positions[i[j] as usize]))).collect();
        let original=caps.clone();let mut result=SurfaceMeshData::default();
        append(&mut result,&caps,&vec![0.;36],&g,[0,0]);
        assert_eq!(caps,original);
        assert!(result.positions.iter().all(|p|p[0]>=8. && p[0]<=40. && p[2]>=8. && p[2]<=40.
            && p[1]>=-2. && p[1]<=16.));
        assert!(result.positions.iter().any(|p|p[0]==24. && p[2]==24. && p[1]==16.));
        assert!(result.positions.iter().any(|p|p[0]==8. && p[1]==14.));
        assert!(result.positions.iter().any(|p|p[0]==9. && p[1]>15. && p[1]<16.));
        assert!(result.normals.iter().any(|n|n[1]>0.2 && n[1]<0.9));
        let mut edges=BTreeMap::new();
        for ids in result.indices.chunks_exact(3) {
            for (a,b) in [(0,1),(1,2),(2,0)] {
                let mut edge=[key(Vec3::from_array(result.positions[ids[a] as usize])),
                    key(Vec3::from_array(result.positions[ids[b] as usize]))];edge.sort();
                *edges.entry(edge).or_insert(0)+=1;
            }
        }
        assert!(edges.values().all(|&count|count==2),"the cap, rim, wall and submerged base must share closed edges");
        let mut footing=vec![0.;36];footing[..6].fill(16.);
        let mut joined=SurfaceMeshData::default();append(&mut joined,&caps,&footing,&g,[0,0]);
        assert!(joined.positions.contains(&[8.,16.,8.])
            && joined.positions.contains(&[40.,16.,8.]),
            "a single adjoining raised bank must close both clipped join corners");
        assert!(joined.positions.iter().zip(&joined.normals)
            .filter(|(p,n)|p[2]==8. && n[1]>0.).all(|(p,_)|p[1]==16.),
            "the cap must remain level along a shared higher bank");
        assert!(!joined.indices.chunks_exact(3).any(|tri|
            tri.iter().all(|&i|joined.positions[i as usize][2]==8.)
            && tri.iter().any(|&i|joined.positions[i as usize][1]<0.)),
            "a shared bank cannot acquire a new exposed wall");
        for z in 0..6 {footing[z*6]=16.;}
        let mut corner=SurfaceMeshData::default();append(&mut corner,&caps,&footing,&g,[0,0]);
        assert!(corner.positions.contains(&[8.,16.,8.]),
            "two adjoining raised banks must close the clipped inside corner");
    }
}
