//! Closed mineral lips over the authoritative town terrace face.
use super::*;

/// Kanto's native earthen banks are exposed bedrock, not a retaining wall
/// assembled from upright pebble columns. Keep the authored support plane and
/// join heights while giving the face broad fractures and buried rim outcrops.
pub(super) fn append_kanto(mesh: &mut SurfaceMeshData, face: [[f32;3];4],
    normal: [f32;3], g: &GridGeometry, origin: [i32;2]) {
    let first = mesh.positions.len();
    if !new_bark::ledge_finish::append_bare_rock(mesh,face,normal,g,origin) {
        append_solid_quad(mesh,face,normal,[0.34,0.30,0.24,1.]);
    }
    for (p,uv) in mesh.positions[first..].iter().zip(&mut mesh.uvs[first..]) {
        // The positive mineral tag adds rock grain without enrolling these
        // already shaped faces in a second terrain elevation pass.
        *uv = [88000.+(p[0]-g.origin_x+origin[0] as f32*g.tile_width)/8.,
            -20000.-(p[2]-g.origin_z+origin[1] as f32*g.tile_height)/8.];
    }
}

pub(super) fn append(mesh: &mut SurfaceMeshData, face: [[f32;3];4],
    normal: [f32;3], g: &GridGeometry, origin: [i32;2]) {
    // The original plane seals the terrain behind the narrow stone joints.
    append_solid_quad(mesh,face,normal,[0.26,0.18,0.09,1.]);
    let p=face.map(Vec3::from_array);let outward=Vec3::from_array(normal);
    let width=(p[3]-p[0]).with_y(0.).length();
    let count=(width/(g.tile_width*0.30)).ceil().max(1.) as usize;
    let offset=Vec3::new(origin[0] as f32*g.tile_width-g.origin_x,0.,
        origin[1] as f32*g.tile_height-g.origin_z);
    let source=(p[0]+p[3])*0.5+offset;
    let seed=|salt:f32|(new_bark::meadow_noise(source.x/g.tile_width*3.17+salt,
        source.z/g.tile_height*2.71-salt*1.31)+0.5).clamp(0.,1.);
    let stop=|i:usize|if i==0 {0.}else if i==count {1.}else {
        (i as f32+(seed(i as f32*19.+31.)-0.5)*0.66)/count as f32
    };
    for stone in 0..count {
        let a=stop(stone); let b=stop(stone+1);
        let sample=seed(stone as f32*17.+59.);
        let light=super::super::outdoor_stone::canyon_color(source,Vec3::Y);
        let palette=[std::array::from_fn(|i|light[i]*0.96),
            std::array::from_fn(|i|light[i]*0.64)];
        super::super::pebbles::append(mesh,sample,palette,|local| {
            let u=a+(local.x+1.)*0.5*(b-a);
            let low=p[0].lerp(p[3],u); let high=p[1].lerp(p[2],u);
            let bottom=low.y-g.tile_height*0.06;
            let top=high.y.max(low.y);
            low.with_y(bottom+(local.y+1.)*0.5*(top-bottom))
                +outward*((local.z+1.)*0.5*0.17-0.03)*g.tile_width
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn kanto_rock_faces_keep_support_and_grain_stable_at_stair_returns() {
        let g=GridGeometry{width:4,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        for normal in [Vec3::X,Vec3::NEG_X,Vec3::Z,Vec3::NEG_Z] {
            let tangent=Vec3::Y.cross(normal);
            for heights in [[0.,6.],[6.,0.],[6.,6.]] {
                let a=Vec3::new(8.,0.,8.);let b=a+tangent*8.;
                let face=[a,a.with_y(heights[0]),b.with_y(heights[1]),b].map(|p|p.to_array());
                let mut mesh=SurfaceMeshData::default();
                append_kanto(&mut mesh,face,normal.to_array(),&g,[17,9]);
                // Original corners seal both the horizontal shelf and tapered
                // stair returns; relief is confined to the exposed side.
                for corner in face {assert!(mesh.positions.contains(&corner));}
                assert!(mesh.positions.iter().all(|p|p.iter().all(|v|v.is_finite())));
                assert!(mesh.positions.iter().any(|p| (Vec3::from_array(*p)-a).dot(normal)>0.1));
                let delta=Vec3::new(-8.,0.,8.);
                let mut shifted=SurfaceMeshData::default();
                append_kanto(&mut shifted,face.map(|p|(Vec3::from_array(p)+delta).to_array()),
                    normal.to_array(),&GridGeometry{origin_x:-8.,origin_z:8.,..g},[17,9]);
                assert_eq!(mesh.indices,shifted.indices);
                assert_eq!(mesh.uvs,shifted.uvs);
                for (a,b) in mesh.colors.iter().zip(&shifted.colors) {
                    assert!(a.iter().zip(b).all(|(a,b)|(a-b).abs()<0.0001));
                }
                for (a,b) in mesh.positions.iter().zip(&shifted.positions) {
                    assert!((Vec3::from_array(*a)+delta-Vec3::from_array(*b)).length()<0.0001);
                }
            }
        }
    }
    #[test]
    fn closed_lip_mineral_finish_stays_registered_when_the_viewport_moves() {
        let g=GridGeometry{width:4,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let face=[[8.,0.,8.],[8.,6.,8.],[0.,0.,8.],[0.,0.,8.]];
        let mut a=SurfaceMeshData::default();append(&mut a,face,[0.,0.,1.],&g,[17,9]);
        let shifted=face.map(|p|[p[0]-8.,p[1],p[2]+8.]);
        let other=GridGeometry{origin_x:-8.,origin_z:8.,..g};
        let mut b=SurfaceMeshData::default();append(&mut b,shifted,[0.,0.,1.],&other,[17,9]);
        assert_eq!(a.indices,b.indices);
        for (a,b) in a.positions.iter().zip(&b.positions) {
            assert!((a[0]-8.-b[0]).abs()<0.0001 && (a[1]-b[1]).abs()<0.0001
                && (a[2]+8.-b[2]).abs()<0.0001);
        }
        for (a,b) in a.colors.iter().zip(&b.colors) {
            assert!(a.iter().zip(b).all(|(a,b)|(a-b).abs()<0.0001));
        }
        for (a,b) in a.uvs.iter().zip(&b.uvs) {assert_eq!(a,b);}
        assert!(a.uvs[4..].iter().all(|uv| *uv == crate::scenery_finish::SOURCE_PAINT_UV));
    }
    #[test]
    fn closed_ledge_stones_keep_the_support_lip_and_bury_the_tapered_toe() {
        for t in [8.,16.] {for normal in [Vec3::X,Vec3::NEG_X,Vec3::Z,Vec3::NEG_Z] {
            let tangent=Vec3::Y.cross(normal);let g=GridGeometry{width:4,height:4,
                tile_width:t,tile_height:t,origin_x:0.,origin_z:0.};
            for heights in [[0.,t*0.75],[t*0.75,0.],[t*0.75,t*0.75]] {
                let a=Vec3::new(t,0.,t);let b=a+tangent*t;
                let face=[a,a.with_y(heights[0]),b.with_y(heights[1]),b].map(|p|p.to_array());
                let mut mesh=SurfaceMeshData::default();append(&mut mesh,face,normal.to_array(),&g,[0,0]);
                assert_eq!(mesh.indices.len(),6+4*126);
                for p in &mesh.positions[4..] {
                    let p=Vec3::from_array(*p);
                    assert!(p.y>=-t*0.0601 && p.y<=heights[0].max(heights[1])+0.0001);
                    let reach=(p-a).dot(normal);
                    assert!(reach>=-t*0.0301 && reach<=t*0.1401);
                }
                for ids in mesh.indices[6..].chunks_exact(126) {
                    let center=ids.iter().map(|&i|Vec3::from_array(mesh.positions[i as usize])).sum::<Vec3>()/ids.len() as f32;
                    let mut volume=0.;let mut edges=std::collections::BTreeMap::new();
                    for tri in ids.chunks_exact(3) {
                        let p=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
                        let cross=(p[1]-p[0]).cross(p[2]-p[0]);
                        assert!(cross.dot((p[0]+p[1]+p[2])/3.-center)>0.);
                        volume+=(p[0]-center).dot((p[1]-center).cross(p[2]-center))/6.;
                        for j in 0..3 {let mut e=[p[j].to_array().map(f32::to_bits),p[(j+1)%3].to_array().map(f32::to_bits)];e.sort();*edges.entry(e).or_insert(0)+=1;}
                    }
                    assert!(volume>0.);assert!(edges.values().all(|&n|n==2));
                }
            }
        }}
    }
}
