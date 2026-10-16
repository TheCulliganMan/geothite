//! The complete large-table drawing: a flared pot and rounded overlapping crown.
use super::*;

// All anchors belong to the six native centerpiece tiles, not the room floor.
pub(super) const PIXELS:[[usize;2];7]=[
    [32,20],[32,22],[26,22],[31,16],[29,36],[25,34],[30,30],
];

fn triangle(out:&mut SurfaceMeshData,mut p:[Vec3;3],outside:Vec3,uv:[f32;2]) {
    let mut normal=(p[1]-p[0]).cross(p[2]-p[0]);
    if normal.length_squared()<1e-12{return;}
    normal=normal.normalize();
    if normal.dot(outside)<0. {p.swap(1,2);normal=-normal;}
    let first=out.positions.len()as u32;
    out.positions.extend(p.map(|v|v.to_array()));out.normals.extend([normal.to_array();3]);
    out.uvs.extend([uv;3]);
    out.colors.extend([crate::interior_models::authored_face_color([1.;4],normal);3]);
    out.indices.extend([first,first+1,first+2]);
}

fn petal(out:&mut SurfaceMeshData,center:Vec3,radii:Vec3,rotation:bevy::prelude::Quat,
    paint:[[f32;2];3]) {
    let point=|ring:usize,side:usize| {
        let latitude=std::f32::consts::PI*ring as f32/8.;
        let angle=std::f32::consts::TAU*side as f32/16.;
        center+rotation*(Vec3::new(latitude.sin()*angle.cos(),latitude.cos(),
            latitude.sin()*angle.sin())*radii)
    };
    for ring in 0..8 {for side in 0..16 {
        let a=point(ring,side);let b=point(ring+1,side);
        let c=point(ring+1,side+1);let d=point(ring,side+1);
        let outside=(a+b+c+d)*0.25-center;
        let normal=rotation.inverse()*outside;
        let uv=paint[if normal.y>radii.y*0.4 {0}else if normal.y< -radii.y*0.4 {2}else{1}];
        if ring!=7 {triangle(out,[a,b,c],outside,uv);}
        if ring!=0 {triangle(out,[a,c,d],outside,uv);}
    }}
}

fn pot(out:&mut SurfaceMeshData,center:Vec3,scale:f32,paint:[[f32;2];7]) {
    // Closed bottom, gently flared body, rolled lip, recessed soil and inner wall.
    let profile=[(0.,0.),(3.6,0.),(4.,0.5),(4.7,6.1),(5.2,6.4),
        (5.3,7.),(5.,7.5),(4.5,7.4),(4.4,6.6),(0.,6.6)];
    let point=|j:usize,i:usize| {
        let a=std::f32::consts::TAU*i as f32/24.;let (r,y)=profile[j];
        center+Vec3::new(r*a.cos(),y,r*a.sin())*scale
    };
    for j in 0..profile.len()-1 {for i in 0..24 {
        let a=point(j,i);let b=point(j+1,i);
        let c=point(j+1,i+1);let d=point(j,i+1);
        let outside=if j==0 {-Vec3::Y}else if j==profile.len()-2 {Vec3::Y}
            else {let v=(a+b+c+d)*0.25-center;
                Vec3::new(v.x,if (4..7).contains(&j){scale}else{0.},v.z)
                    *if j>=7 {-1.}else{1.}};
        let uv=paint[if j==profile.len()-2 {6}else if j==7 {3}
            else if (4..7).contains(&j){5}else{4}];
        triangle(out,[a,b,c],outside,uv);triangle(out,[a,c,d],outside,uv);
    }}
}

pub(super) fn append(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,b:[f32;4]) {
    let paint=native_source_swatches(p,g,PIXELS);
    let [w,e,n,s]=b;let scale=g.tile_height/8.;
    let base=Vec3::new((w+e)*0.5,p.height_pixels*scale,n+(s-n)*4.5/7.);
    pot(out,base,scale,paint);
    // The native dark mass fills the pot mouth and joins both upper lobes.
    // Broad overlapping leaves avoid an empty rim with isolated exposed stems.
    for leaf in 0..7 {
        let angle=std::f32::consts::TAU*leaf as f32/7.;
        let direction=Vec3::new(angle.cos(),0.,angle.sin());
        let axis=(direction*0.6+Vec3::Y*0.8).normalize();
        petal(out,base+(direction*2.+Vec3::Y*9.2)*scale,
            Vec3::new(1.35,3.1,0.9)*scale,
            bevy::prelude::Quat::from_rotation_arc(Vec3::Y,axis),
            [paint[1],paint[2],paint[3]]);
    }
    // Two asymmetric rounded crowns follow the upper and lower source lobes.
    for (offset,radius,yaw) in [(Vec3::new(1.5,14.2,-0.9),4.5,0.3),
        (Vec3::new(-2.5,11.3,1.1),3.3,0.8)] {
        let center=base+offset*scale;
        let root=base+Vec3::Y*6.5*scale;let delta=center-root;
        petal(out,(root+center)*0.5,Vec3::new(0.23*scale,delta.length()*0.5+0.1*scale,0.23*scale),
            bevy::prelude::Quat::from_rotation_arc(Vec3::Y,delta.normalize()),[paint[2];3]);
        petal(out,center,Vec3::splat(radius*0.65*scale),bevy::prelude::Quat::IDENTITY,
            [paint[0],paint[1],paint[2]]);
        for tier in 0..2 {for leaf in 0..7 {
            let angle=std::f32::consts::TAU*leaf as f32/7.+yaw+tier as f32*0.4;
            let direction=Vec3::new(angle.cos(),0.,angle.sin());
            let r=radius*(if tier==0 {0.56}else{0.29});
            let at=center+(direction*r+Vec3::Y*(tier as f32*1.05-0.4))*scale;
            let rotation=bevy::prelude::Quat::from_rotation_y(-angle)
                *bevy::prelude::Quat::from_rotation_z(-0.45);
            petal(out,at,Vec3::new(radius*0.58,radius*0.48,radius*0.38)*scale,
                rotation,[paint[0],paint[1],paint[2]]);
        }}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn curved_pot_and_crowns_are_closed_positive_volumes_with_outward_normals() {
        for scale in [0.5,1.,2.] {
            for is_pot in [false,true] {
                let mut out=SurfaceMeshData::default();
                if is_pot {pot(&mut out,Vec3::ZERO,scale,[[0.1,0.2];7]);}
                else {petal(&mut out,Vec3::ZERO,Vec3::new(2.,1.,1.5)*scale,
                    bevy::prelude::Quat::from_rotation_z(0.4),[[0.1,0.2];3]);}
                let mut volume=0.;let mut edges=std::collections::BTreeMap::new();
                let key=|v:Vec3|v.to_array().map(|f|(f/scale*10000.).round()as i32);
                for ids in out.indices.chunks_exact(3) {
                    let [a,b,c]=[ids[0],ids[1],ids[2]].map(|i|Vec3::from_array(out.positions[i as usize]));
                    let normal=Vec3::from_array(out.normals[ids[0]as usize]);
                    assert!((b-a).cross(c-a).dot(normal)>0.);
                    volume+=a.dot(b.cross(c))/6.;
                    for (a,b) in [(a,b),(b,c),(c,a)] {
                        let (a,b)=(key(a),key(b));
                        let (edge,direction)=if a<b {((a,b),1i32)}else{((b,a),-1)};
                        let entry=edges.entry(edge).or_insert((0,0));entry.0+=1;entry.1+=direction;
                    }
                }
                assert!(volume>scale.powi(3));
                assert!(edges.values().all(|&(count,sum)|count==2&&sum==0));
            }
        }
    }
}
