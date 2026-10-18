//! Quiet physical timber for authenticated player-home walls and their backing.
//! Source texels choose the live wood pigment; grain marks are not enlarged holes.
use super::*;
use super::modeled_interiors::architecture_box;

pub(super) fn append(out:&mut SurfaceMeshData,g:&GridGeometry,b:[f32;4],base:f32,top:f32,donor:[usize;2]) {
    let [w,e,n,s]=b;
    let uv=g.uv(donor[0],donor[1]);
    let field=[uv.0+(uv.1-uv.0)*1.5/8.,uv.2+(uv.3-uv.2)*0.5/8.];
    let first=out.positions.len();let indices=out.indices.len();
    architecture_box(out,[w,e,base,top,n,s],[1.;3]);
    out.uvs[first..].fill(field);
    out.indices.drain(indices+6..indices+12);
    let mut left=w;
    while left<e-0.0001 {
        let right=(left+g.tile_width).min(e);
        let xs=[0.,0.015,0.035,0.06,0.94,0.965,0.985,1.];
        let ys=[0.,0.004,0.009,0.015,0.985,0.991,0.996,1.];
        let point=|x:f32,y:f32| {
            let edge=(x.min(1.-x)/0.06).min(y.min(1.-y)/0.015).clamp(0.,1.);
            let round=(1.-(1.-edge)*(1.-edge)).sqrt();
            let px=left+(right-left)*x;let py=top-(top-base)*y;
            // The front's outer boundary must meet the remaining five box
            // faces exactly. Interior plank joints may recess; floor, crown
            // and end contacts stay flush instead of exposing a dark slit.
            let contact=((px-w).min(e-px).min(py-base).min(top-py)
                /(g.tile_height*0.04)).clamp(0.,1.);
            [px,py,s-(1.-round)*g.tile_height*0.025*contact]
        };
        super::modeled_interiors::native_source_surface_patches(out,&xs,&ys,point,|_,_|field,|x,y| {
            let seam=if x<0.015||x>0.985 {0.88}else{1.};
            let grain=crate::scenery_finish::timber_grain([left+(right-left)*x,top-(top-base)*y,s]);
            seam*(1.+0.018*grain)
        },true);
        left=right;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn home_timber_has_one_continuous_front_and_retains_closed_bounds() {
        let g=GridGeometry {width:8,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut mesh=SurfaceMeshData::default();
        append(&mut mesh,&g,[0.,32.,13.,16.],0.,32.,[0,0]);
        let uv=g.uv(0,0);let pigment=[uv.0+(uv.1-uv.0)*1.5/8.,uv.2+(uv.3-uv.2)*0.5/8.];
        assert!(mesh.uvs.iter().all(|&v|v==pigment),"no repeated bitmap grain or bright stretched marks");
        let mut area=0.;
        for tri in mesh.indices.chunks_exact(3) {
            let p=[0,1,2].map(|j|Vec3::from_array(mesh.positions[tri[j]as usize]));
            let normal=Vec3::from_array(mesh.normals[tri[0]as usize]);
            let cross=(p[1]-p[0]).cross(p[2]-p[0]);
            assert!(cross.dot(normal)>0.);
            if normal.z>0. {
                area+=cross.z/2.;
                for v in &p {
                    if v.x==0.||v.x==32.||v.y==0.||v.y==32. {
                        assert_eq!(v.z,16.,"floor/crown/end contacts must meet the closed body");
                    }
                }
            }
        }
        assert!((area-32.*32.).abs()<0.01,"joined front without overlaps or holes: {area}");
        assert!(mesh.positions.iter().all(|p|(0.0..=32.0).contains(&p[0])&&(0.0..=32.0).contains(&p[1])&&(13.0..=16.0).contains(&p[2])));
        assert!(mesh.positions.iter().any(|p|p[2]>15.7&&p[2]<16.));
    }
}
