//! Upright volume for the complete champion-room north drawing.
//! Called only after lance_terraces authenticates the course and custom scope.
use super::*;

pub(super) fn finish(mesh: &mut TerrainMeshData, frame: &VisualWorldFrame,
    cells: &[&VisualTile], g: &GridGeometry) {
    let origin = frame.grid_origin;
    let scale = g.tile_height / 8.;
    let at = |x: f32, y: f32, z: f32| [g.origin_x + (x-origin.x as f32)*g.tile_width,
        y*scale, g.origin_z + (z-origin.y as f32)*g.tile_height];
    let uv = |x: f32, y: f32| {
        let x=x.clamp(0.,19.999);let y=y.clamp(0.,3.999);
        let u=g.uv((x.floor() as i32-origin.x) as usize,
            (y.floor() as i32-origin.y) as usize);
        [u.0+(u.1-u.0)*x.fract(),u.2+(u.3-u.2)*y.fract()]
    };
    let paint=uv(3.5,0.5);
    let front_uv=|x:f32,y:f32|uv(x,(48.-y)/8.);
    let out=&mut mesh.textured;
    // Retire only the old horizontal art at the platform datum. Existing
    // upright scenery, cutaway ranges, source claims and support stay intact.
    let old=std::mem::take(&mut out.indices);
    for tri in old.chunks_exact(3) {
        let p=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(out.positions[i as usize]));
        let c=(p[0]+p[1]+p[2])/3.;
        let x=(c.x-g.origin_x)/g.tile_width+origin.x as f32;
        let z=(c.z-g.origin_z)/g.tile_height+origin.y as f32;
        let flat=tri.iter().all(|&i|out.normals[i as usize][1]>0.99)
            &&(c.y-16.*scale).abs()<0.5*scale
            &&(0.0..20.0).contains(&x)&&(0.0..4.0).contains(&z);
        if !flat {out.indices.extend_from_slice(tri);}
    }
    let open=cells[(-origin.y) as usize*g.width+(8-origin.x) as usize].source.metatile_id==0x0b;
    // Ground ends at the actual warp tile, not at the back of the folded
    // drawing. A closed door has only its wall-depth threshold. The open
    // recess includes the warp tile and dark source-painted returns, so no
    // unsupported pale shelf sticks out behind the north wall in side views.
    let rear=if open {2.}else{3.};
    let floor=g.uv((5-origin.x) as usize,(5-origin.y) as usize);
    append_quad(out,[at(8.,16.,rear),at(8.,16.,4.),at(12.,16.,4.),at(12.,16.,rear)],
        [0.,1.,0.],[[floor.0,floor.2];4],TEXTURED_SHADE);
    let start=out.positions.len();
    if open {
        let dark=uv(9.5,2.5);
        for (points,normal) in [
            ([at(8.,16.,2.),at(12.,16.,2.),at(12.,48.,2.),at(8.,48.,2.)],[0.,0.,1.]),
            ([at(8.,16.,4.),at(8.,16.,2.),at(8.,48.,2.),at(8.,48.,4.)],[1.,0.,0.]),
            ([at(12.,16.,2.),at(12.,16.,4.),at(12.,48.,4.),at(12.,48.,2.)],[-1.,0.,0.]),
            ([at(8.,48.,4.),at(8.,48.,2.),at(12.,48.,2.),at(12.,48.,4.)],[0.,-1.,0.]),
        ] {append_quad(out,points,normal,[dark;4],TEXTURED_SHADE);}
    }
    for (west,east) in [(0.,8.),(12.,20.)] {
        let first=out.positions.len();let indices=out.indices.len();
        let a=at(west,16.,3.);let b=at(east,48.,4.);
        super::super::modeled_interiors::architecture_box(out,
            [a[0],b[0],a[1],b[1],a[2],b[2]],[1.;3]);
        out.uvs[first..].fill(paint);
        out.indices.drain(indices+6..indices+12);
        for x in west as usize..east as usize {for row in 0..4 {
            let top=48.-row as f32*8.;let bottom=top-8.;
            // Gold post art wraps the column instead of also appearing as a
            // flat duplicate on the backing behind it.
            let painted=x<6||x>=14;
            let tex=if painted {[uv(x as f32,row as f32+0.999),uv(x as f32+0.999,row as f32+0.999),
                uv(x as f32+0.999,row as f32),uv(x as f32,row as f32)]}else{[paint;4]};
            append_quad(out,[at(x as f32,bottom,4.),at(x as f32+1.,bottom,4.),
                at(x as f32+1.,top,4.),at(x as f32,top,4.)],[0.,0.,1.],tex,TEXTURED_SHADE);
        }}
    }
    // An elliptical soffit gives a real aperture, not a dark door painted on
    // a blocking quad. Its full four-cell width clears both source warp lanes.
    for segment in 0..32 {
        let x0=8.+4.*segment as f32/32.;let x1=8.+4.*(segment+1) as f32/32.;
        let arch=|x:f32|36.+8.*(1.-((x-10.)/2.).powi(2)).max(0.).sqrt();
        let y0=arch(x0);let y1=arch(x1);
        append_quad(out,[at(x0,y0,4.),at(x1,y1,4.),at(x1,48.,4.),at(x0,48.,4.)],
            [0.,0.,1.],[front_uv(x0,y0),front_uv(x1,y1),front_uv(x1,48.),front_uv(x0,48.)],TEXTURED_SHADE);
        append_quad(out,[at(x0,48.,3.),at(x1,48.,3.),at(x1,y1,3.),at(x0,y0,3.)],
            [0.,0.,-1.],[paint;4],TEXTURED_SHADE);
        let normal=Vec3::new(y1-y0,-(x1-x0)*g.tile_width/scale,0.).normalize().to_array();
        append_quad(out,[at(x0,y0,3.),at(x1,y1,3.),at(x1,y1,4.),at(x0,y0,4.)],
            normal,[paint;4],TEXTURED_SHADE);
        if !open {
            append_quad(out,[at(x0,16.,4.),at(x1,16.,4.),at(x1,y1,4.),at(x0,y0,4.)],
                [0.,0.,1.],[front_uv(x0,16.),front_uv(x1,16.),front_uv(x1,y1),front_uv(x0,y0)],TEXTURED_SHADE);
            append_quad(out,[at(x0,y0,3.),at(x1,y1,3.),at(x1,16.,3.),at(x0,16.,3.)],
                [0.,0.,-1.],[paint;4],TEXTURED_SHADE);
        }
    }
    append_quad(out,[at(8.,48.,3.),at(8.,48.,4.),at(12.,48.,4.),at(12.,48.,3.)],
        [0.,1.,0.],[paint;4],TEXTURED_SHADE);
    for center in [7.,13.] {
        for segment in 0..32 {for row in 0..4 {
            let angle=|i:usize|std::f32::consts::TAU*i as f32/32.;
            let a=angle(segment);let b=angle(segment+1);
            let top=48.-row as f32*8.;let bottom=top-8.;
            let radius=|y:f32|if y<=18.||y>=46. {0.98}else{0.86};
            let point=|theta:f32,y:f32|at(center+radius(y)*theta.cos(),y,4.+radius(y)*0.8*theta.sin());
            let tex=|theta:f32,y:f32| {
                if theta.sin()>=-0.001 {uv(center-1.+(theta.cos()+1.).min(1.999), (48.-y)/8.)}
                else {uv(center-0.5,1.5)}
            };
            let first=out.positions.len();
            append_quad(out,[point(a,bottom),point(a,top),point(b,top),point(b,bottom)],
                [0.,0.,1.],[tex(a,bottom),tex(a,top),tex(b,top),tex(b,bottom)],TEXTURED_SHADE);
            // Parametric ellipse normals remain smooth with non-square grids.
            for (i,theta) in [a,a,b,b].into_iter().enumerate() {
                let slope=(radius(top)-radius(bottom))/(top-bottom);
                let n=Vec3::new(theta.cos()/g.tile_width,-slope/scale,theta.sin()/(g.tile_height*0.8)).normalize();
                out.normals[first+i]=n.to_array();
            }
        }}
        for (y,normal) in [(16.,[0.,-1.,0.]),(48.,[0.,1.,0.])] {
            for segment in 0..32 {
                let a=std::f32::consts::TAU*segment as f32/32.;let b=std::f32::consts::TAU*(segment+1) as f32/32.;
                let mut p=[at(center,y,4.),at(center+0.98*a.cos(),y,4.+0.98*0.8*a.sin()),at(center+0.98*b.cos(),y,4.+0.98*0.8*b.sin())];
                if y==48. {p.swap(1,2);}
                let first=out.positions.len() as u32;
                out.positions.extend(p);out.normals.extend([normal;3]);out.uvs.extend([uv(center-0.5,0.5);3]);
                out.colors.extend([TEXTURED_SHADE;3]);out.indices.extend([first,first+1,first+2]);
            }
        }
    }
    out.cutaway_ranges.push(start..out.positions.len());
    if mesh.authored_cells.len()==cells.len() {
        for y in 0..4 {for x in 0..20 {
            mesh.authored_cells[(y-origin.y) as usize*g.width+(x-origin.x) as usize]=Some("lance:north-portal");
        }}
    }
}
