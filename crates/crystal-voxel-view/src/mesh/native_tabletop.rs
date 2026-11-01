//! Project the complete native house tabletop onto the selected model's top.
//! The final source row is its perspective apron, not more tabletop.
use super::*;

fn clip(poly:Vec<[f32;3]>,axis:usize,plane:f32,keep_above:bool)->Vec<[f32;3]> {
    let mut out=Vec::new();
    if poly.is_empty(){return out;}
    let mut a=*poly.last().unwrap();
    for b in poly {
        let inside=|v:[f32;3]|if keep_above {v[axis]>=plane}else{v[axis]<=plane};
        if inside(a)!=inside(b) {
            let t=(plane-a[axis])/(b[axis]-a[axis]);
            let mut v=std::array::from_fn(|i|a[i]+(b[i]-a[i])*t);
            v[axis]=plane;out.push(v);
        }
        if inside(b){out.push(b);}
        a=b;
    }
    out
}

pub(super) fn append(mesh:&mut SurfaceMeshData,first_index:usize,g:&GridGeometry,
    p:&Placement,bounds:[f32;4],centerpiece:bool) {
    let [w,e,n,s]=bounds;
    let source_rows=p.height-1;
    let height=p.height_pixels*g.tile_height/8.;
    let indices=mesh.indices.split_off(first_index);
    for ids in indices.chunks_exact(3) {
        if !ids.iter().all(|&i|mesh.normals[i as usize][1]>0.999
            &&mesh.positions[i as usize][1]>height*0.95) {
            mesh.indices.extend_from_slice(ids);continue;
        }
        let triangle:Vec<_>=ids.iter().map(|&i|mesh.positions[i as usize]).collect();
        let normal=mesh.normals[ids[0]as usize];let color=mesh.colors[ids[0]as usize];
        for y in 0..source_rows {for x in 0..p.width {
            let left=w+(e-w)*x as f32/p.width as f32;
            let right=w+(e-w)*(x+1)as f32/p.width as f32;
            let north=n+(s-n)*y as f32/source_rows as f32;
            let south=n+(s-n)*(y+1)as f32/source_rows as f32;
            let mut poly=triangle.clone();
            for (axis,plane,above) in [(0,left,true),(0,right,false),(2,north,true),(2,south,false)] {
                poly=clip(poly,axis,plane,above);
            }
            let uv=g.uv(p.column+x,p.row+y);
            for j in 1..poly.len().saturating_sub(1) {
                let points=[poly[0],poly[j],poly[j+1]];
                let [a,b,c]=points.map(Vec3::from_array);
                if (b-a).cross(c-a).length_squared()<1e-12 {continue;}
                for v in points {
                    mesh.indices.push(mesh.positions.len()as u32);
                    mesh.positions.push(v);mesh.normals.push(normal);mesh.colors.push(color);
                    mesh.uvs.push(if centerpiece&&(3..5).contains(&x)&&(2..5).contains(&y) {
                        // The complete pot/foliage drawing is now a volume.
                        // Restore the adjacent uninterrupted native gold field.
                        native_source_swatches(p,g,[[17,9]])[0]
                    } else {[uv.0+(uv.1-uv.0)*(v[0]-left)/(right-left),
                        uv.2+(uv.3-uv.2)*(v[2]-north)/(south-north)]});
                }
            }
        }}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn projection_preserves_top_area_sides_winding_and_native_tile_boundaries() {
        for (width,rows) in [(4,2),(4,3),(4,4),(6,2),(8,7)] {
        for scale in [0.5,1.,2.] {
            let g=GridGeometry{width:16,height:16,tile_width:8.*scale,tile_height:8.*scale,
                origin_x:0.,origin_z:0.};
            let p=Placement{kind:ModelKind::DiningTable,column:3,row:2,width,height:rows+1,
                ground:0,depth_pixels:32.,height_pixels:10.,base_pixels:0.,footing_pixels:None};
            let b=[-11.*scale,(-11.+width as f32*8.)*scale,7.*scale,39.*scale];
            let mut mesh=SurfaceMeshData::default();
            architecture_box(&mut mesh,[b[0],b[1],0.,10.*scale,b[2],b[3]],[1.;3]);
            let old=mesh.clone();append(&mut mesh,0,&g,&p,b,false);
            let sides=|m:&SurfaceMeshData|m.indices.chunks_exact(3).filter(|ids|
                m.normals[ids[0]as usize][1]<0.99).map(|ids|ids.to_vec()).collect::<Vec<_>>();
            assert_eq!(sides(&mesh),sides(&old));
            let mut area=0.;let mut tiles=std::collections::BTreeSet::new();
            for ids in mesh.indices.chunks_exact(3).filter(|ids|mesh.normals[ids[0]as usize][1]>0.99) {
                let [a,b,c]=[ids[0],ids[1],ids[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
                let cross=(b-a).cross(c-a);assert!(cross.y>0.);area+=cross.length()/2.;
                let center=(a+b+c)/3.;
                let x=((center.x+11.*scale)/(8.*scale)).floor()as usize;
                let y=((center.z-7.*scale)/(32.*scale/rows as f32)).floor()as usize;
                tiles.insert((x,y));let uv=g.uv(p.column+x,p.row+y);
                for &i in ids {let v=mesh.uvs[i as usize];
                    assert!(v[0]>=uv.0-1e-6&&v[0]<=uv.1+1e-6&&v[1]>=uv.2-1e-6&&v[1]<=uv.3+1e-6);
                }
            }
            assert!((area-width as f32*256.*scale*scale).abs()<0.01);
            assert_eq!(tiles.len(),width*rows);
        }
        }
    }
}
