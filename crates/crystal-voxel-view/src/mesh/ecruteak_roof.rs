//! Small closed ceramic courses for Ecruteak's intact, source-selected roofs.
use super::*;

pub(super) fn solid<const N: usize>(mesh: &mut SurfaceMeshData, rings: [[Vec3;N];2], colors: [[f32;3];3],
    origin: [f32;2], t: f32, painted: bool) {
    let center=rings.iter().flatten().copied().sum::<Vec3>()/(N*2) as f32;
    let mut faces=vec![(rings[0].to_vec(),colors[1]),(rings[1].to_vec(),colors[2])];
    for i in 0..N {let j=(i+1)%N;
        faces.push((vec![rings[0][i],rings[1][i],rings[1][j],rings[0][j]],colors[0]));
    }
    for (mut points,color) in faces {
        let normal=(points[1]-points[0]).cross(points[2]-points[0]);
        let midpoint=points.iter().copied().sum::<Vec3>()/points.len() as f32;
        if normal.dot(midpoint-center)<0. {points.reverse();}
        let first=mesh.positions.len();
        face(mesh,&points.iter().map(|p|p.to_array()).collect::<Vec<_>>(),color);
        if painted {for i in first..mesh.positions.len() {
            let p=mesh.positions[i];
            mesh.uvs[i]=[(p[0]-origin[0])/t,
                crate::scenery_finish::TOWN_PAINT_BASE+(p[2]-origin[1])/t];
        }}
    }
}

pub(super) fn append(mesh: &mut SurfaceMeshData, b: [f32;4], t: f32,
    eave: f32, ridge: f32, palette: Option<[[f32;4];3]>) {
    let [left,right,back,front]=b;let middle=(back+front)*0.5;
    let field=palette.map(|p|[p[0][0],p[0][1],p[0][2]]).unwrap_or(ROOF);
    let lip=palette.map(|p| {
        // The wide hall also exposes a pale roof border. Its brighter value
        // is not the ochre ceramic stripe. Choose the chromatic roof role
        // between the two lit swatches, retaining the live clock palette.
        let chroma=|c:[f32;4]| {
            let low=c[0].min(c[1]).min(c[2]);let high=c[0].max(c[1]).max(c[2]);
            (high-low)/high.max(0.0001)
        };
        let c=if chroma(p[1])>chroma(p[2]) {p[1]}else{p[2]};
        [c[0],c[1],c[2]]
    }).unwrap_or(EDGE);
    let painted=palette.is_some();let origin=[left,back];
    let columns=((right-left)/(t*0.65)).ceil() as usize;
    let step=(right-left)/columns as f32;
    let height=|z:f32|eave+(ridge-eave)*(1.-((z-middle)/((front-back)*0.5)).abs());
    for side in 0..2 {for row in 0..10 {
        // Run each slope from eave to ridge, so the visible lower lip always
        // covers the preceding course. Keep every part inside the old bounds.
        let outer=if side==0 {back}else{front};
        let inner=if side==0 {middle-t*0.075}else{middle+t*0.075};
        let a=outer+(inner-outer)*row as f32/10.;
        let z=outer+(inner-outer)*(row+1) as f32/10.;
        let (za,zb)=(a.min(z),a.max(z));
        let mut stops=vec![left];
        for col in 0..columns {
            let x=left+(col as f32+if row%2==0 {1.}else{0.5})*step;
            if x>left && x<right {stops.push(x);}
        }
        stops.push(right);
        for (col,x) in stops.windows(2).enumerate() {
            let x0=x[0]+t*0.006;let x1=x[1]-t*0.006;
            let profile=|z:f32| {
                let high=(height(z)+t*if (z-a).abs()<t*0.0001 {0.045}else{0.012}).min(ridge);
                // A shallow convex ceramic crown, wholly inside the former
                // tile envelope. Narrow half-tiles keep the same proportions.
                let drop=(t*0.035).min((x1-x0)*0.12);
                let crown=[(0.,1.),(0.12,0.36),(0.32,0.08),(0.5,0.),
                    (0.68,0.08),(0.88,0.36),(1.,1.)];
                std::array::from_fn::<_,9,_>(|i| {
                    if i<7 {let (u,d)=crown[i];Vec3::new(x0+(x1-x0)*u,high-drop*d,z)}
                    else {Vec3::new(if i==7 {x1}else{x0},high-t*0.065,z)}
                })
            };
            let pigment=field.map(|c|c*(0.96+((col+row*3)%5) as f32*0.02));
            let ends=if side==0 {[lip,pigment]}else{[pigment,lip]};
            solid(mesh,[profile(za),profile(zb)],[pigment,ends[0],ends[1]],origin,t,painted);
        }
    }}
    // Narrow, separately closed ridge pieces cover the reserved ridge gap.
    for cap in 0..columns {
        let x0=left+step*cap as f32+t*0.005;
        let x1=left+step*(cap+1) as f32-t*0.005;
        let ring=|x:f32| [Vec3::new(x,ridge-t*0.09,middle-t*0.10),
            Vec3::new(x,ridge-t*0.025,middle-t*0.07),Vec3::new(x,ridge,middle),
            Vec3::new(x,ridge-t*0.025,middle+t*0.07),Vec3::new(x,ridge-t*0.09,middle+t*0.10),
            Vec3::new(x,ridge-t*0.10,middle)];
        solid(mesh,[ring(x0),ring(x1)],[lip,field,field],origin,t,painted);
    }
    // Continuous closed backing seals course gaps and ridge joints without
    // covering the raised ceramic faces. It stays below the original slope.
    let deck=[Vec3::new(left,eave-t*0.08,back),Vec3::new(right,eave-t*0.08,back),
        Vec3::new(right,ridge-t*0.08,middle),Vec3::new(left,ridge-t*0.08,middle),
        Vec3::new(left,eave-t*0.08,front),Vec3::new(right,eave-t*0.08,front)];
    let center=deck.iter().copied().sum::<Vec3>()/6.;
    for ids in [vec![0,1,2,3],vec![3,2,5,4],vec![0,4,5,1],vec![0,3,4],vec![1,5,2]] {
        let first=mesh.positions.len();
        let mut points=ids.iter().map(|&i|deck[i]).collect::<Vec<_>>();
        if (points[1]-points[0]).cross(points[2]-points[0]).dot(points.iter().copied().sum::<Vec3>()/points.len() as f32-center)<0. {points.reverse();}
        face(mesh,&points.iter().map(|p|p.to_array()).collect::<Vec<_>>(),field.map(|v|v*0.65));
        if painted {for i in first..mesh.positions.len() {
            let p=mesh.positions[i];mesh.uvs[i]=[(p[0]-left)/t,
                crate::scenery_finish::TOWN_PAINT_BASE+(p[2]-back)/t];
        }}
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pale_source_roof_border_cannot_replace_the_ochre_ceramic_stripe() {
        for light in [[0.8,0.9,0.8,1.],[0.35,0.35,0.5,1.]] {
            let stripe=if light[0]>0.5 {[0.3,0.18,0.01,1.]}else{[0.15,0.06,0.22,1.]};
            let mut mesh=SurfaceMeshData::default();
            append(&mut mesh,[-64.,64.,-48.,48.],16.,43.2,67.2,
                Some([[0.04,0.04,0.04,1.],stripe,light]));
            let colored=mesh.colors.iter().filter(|c|(c[0]-c[2]).abs()>0.0001).collect::<Vec<_>>();
            assert!(!colored.is_empty());
            for c in colored {assert!((c[0]/c[2]-stripe[0]/stripe[2]).abs()<0.0001);}
        }
    }
    #[test]
    fn intact_ceramic_roof_is_closed_outward_and_keeps_its_original_envelope() {
        for t in [8.,16.] {for width in [8.,12.] {
            let b=[-width*t/2.,width*t/2.,-3.*t,3.*t];let mut mesh=SurfaceMeshData::default();
            append(&mut mesh,b,t,2.7*t,4.2*t,Some([[0.04,0.04,0.04,1.],[0.1,0.1,0.1,1.],[0.4,0.2,0.02,1.]]));
            assert!(mesh.indices.len()/3<14500);
            assert!(mesh.positions.iter().all(|p|p[0]>=b[0] && p[0]<=b[1]
                && p[2]>=b[2] && p[2]<=b[3] && p[1]<=4.2*t && p[1]>2.5*t));
            // Crowned tiles have32 triangles, ridge caps20 and the deck8.
            // Check each solid against its own center, including its underside.
            let tile_end=mesh.indices.len()-24;
            let caps=(((b[1]-b[0])/(t*0.65)).ceil() as usize)*60;
            let crown_end=tile_end-caps;
            assert_eq!(crown_end%96,0);
            assert_eq!((tile_end-crown_end)%60,0);
            let crown_normals=mesh.indices[..96].iter().filter_map(|&i| {
                let n=mesh.normals[i as usize];
                (n[1]>0.1).then_some((n[0]*1000.).round() as i32)
            }).collect::<std::collections::BTreeSet<_>>();
            assert!(crown_normals.len()>=6,"ceramic crown must resolve both curved shoulders");
            for ids in mesh.indices[..crown_end].chunks_exact(96)
                .chain(mesh.indices[crown_end..tile_end].chunks_exact(60))
                .chain(std::iter::once(&mesh.indices[tile_end..])) {
                let center=ids.iter().map(|&i|Vec3::from_array(mesh.positions[i as usize])).sum::<Vec3>()/ids.len() as f32;
                let mut volume=0.;
                let mut edges=std::collections::BTreeMap::new();
                for tri in ids.chunks_exact(3) {
                    let p=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
                    let n=(p[1]-p[0]).cross(p[2]-p[0]);
                    assert!(n.dot((p[0]+p[1]+p[2])/3.-center)>0.);
                    volume+=(p[0]-center).dot((p[1]-center).cross(p[2]-center))/6.;
                    for e in 0..3 {let mut edge=[p[e].to_array().map(f32::to_bits),p[(e+1)%3].to_array().map(f32::to_bits)];edge.sort();*edges.entry(edge).or_insert(0)+=1;}
                }
                assert!(volume>0.);
                assert!(edges.values().all(|&count|count==2),"every ceramic/deck solid must be closed");
            }
            assert!(mesh.uvs.iter().all(|&uv|crate::scenery_finish::town_paint_kind(uv)==Some(0)));
        }}
    }
}
