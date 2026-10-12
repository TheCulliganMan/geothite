// The complete Celadon drawing owns an original modeled table and serving basket.
// Every basket/contents surface uses color samples inside that same source bowl.
fn append_cafe_basket(out:&mut SurfaceMeshData,center:[f32;2],scale:f32,
    swatches:[[f32;2];3]) {
    let point=|rx:f32,rz:f32,y:f32,angle:f32| {
        [center[0]+rx*scale*angle.cos(),y*scale,center[1]+rz*scale*angle.sin()]
    };
    let rings=[(3.9,5.2,16.1),(5.2,6.5,18.3),(4.6,5.9,18.3),(3.4,4.7,16.7)];
    for ring in 0..3 {for side in 0..16 {
        let a=side as f32*std::f32::consts::TAU/16.;
        let b=(side+1) as f32*std::f32::consts::TAU/16.;
        let (rx,rz,y)=rings[ring];let (nx,nz,ny)=rings[ring+1];
        let p=[point(rx,rz,y,a),point(nx,nz,ny,a),point(nx,nz,ny,b),point(rx,rz,y,b)];
        let normal=(Vec3::from_array(p[1])-Vec3::from_array(p[0]))
            .cross(Vec3::from_array(p[2])-Vec3::from_array(p[0])).normalize();
        let shade=if side%2==0{1.05}else{0.92};
        append_quad(out,p,normal.to_array(),[swatches[usize::from(ring==1)];4],
            crate::interior_models::authored_face_color([shade,shade,shade,1.],normal));
    }}
    for (ring,up) in [(0,false),(3,true)] {
        let (rx,rz,y)=rings[ring];
        for side in 0..16 {
            let a=side as f32*std::f32::consts::TAU/16.;
            let b=(side+1) as f32*std::f32::consts::TAU/16.;
            let mut p=[point(0.,0.,y,0.),point(rx,rz,y,a),point(rx,rz,y,b)];
            if up {p.swap(1,2);}
            append_cafe_triangle(out,p,swatches[0]);
        }
    }
    // Six small raised portions sit inside the basin rather than remaining a
    // large pixel picture printed onto the tabletop. Closed six-pole forms
    // keep the same source gold family with individually shaded facets.
    for (x,z) in [(-2.2,-2.5),(0.4,-2.5),(2.5,-0.1),(-2.5,0.4),(-0.1,0.2),(0.6,2.6)] {
        let c=Vec3::new(center[0]+x*scale,18.1*scale,center[1]+z*scale);
        let poles=[Vec3::X,Vec3::Z,-Vec3::X,-Vec3::Z,Vec3::Y,-Vec3::Y]
            .map(|v|(c+v*Vec3::new(1.6,1.2,1.7)*scale).to_array());
        for i in 0..4 {
            let next=(i+1)%4;
            append_cafe_triangle(out,[poles[4],poles[next],poles[i]],swatches[2]);
            append_cafe_triangle(out,[poles[5],poles[i],poles[next]],swatches[2]);
        }
    }
}
fn append_cafe_triangle(out:&mut SurfaceMeshData,p:[[f32;3];3],uv:[f32;2]) {
    let normal=(Vec3::from_array(p[1])-Vec3::from_array(p[0]))
        .cross(Vec3::from_array(p[2])-Vec3::from_array(p[0])).normalize();
    let first=out.positions.len() as u32;
    out.positions.extend(p);out.normals.extend([normal.to_array();3]);
    out.uvs.extend([uv;3]);out.colors.extend([
        crate::interior_models::authored_face_color([1.;4],normal);3]);
    out.indices.extend([first,first+1,first+2]);
}
