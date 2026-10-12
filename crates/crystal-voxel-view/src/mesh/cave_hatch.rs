//! Upright cave ladders rooted on the authored walking surface.
use super::*;

fn beam(out: &mut SurfaceMeshData, a: Vec3, b: Vec3, width: f32, depth: f32, color: [f32; 4]) {
    let axis = (b - a).normalize();
    let u = if axis.y.abs() > 0.9 {
        Vec3::X
    } else {
        axis.cross(Vec3::Y).normalize()
    };
    let v = axis.cross(u).normalize();
    let ring = [
        [-0.5, -0.25],
        [-0.25, -0.5],
        [0.25, -0.5],
        [0.5, -0.25],
        [0.5, 0.25],
        [0.25, 0.5],
        [-0.25, 0.5],
        [-0.5, 0.25],
    ];
    let point = |center: Vec3, i: usize| center + u * ring[i][0] * width + v * ring[i][1] * depth;
    for i in 0..8 {
        let j = (i + 1) % 8;
        let p = [point(a, i), point(a, j), point(b, j), point(b, i)];
        let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize();
        let shade = 0.84 + 0.16 * normal.dot(Vec3::new(-0.4, 0.7, 0.6)).max(0.);
        append_solid_quad(
            out,
            p.map(|p| p.to_array()),
            normal.to_array(),
            [color[0] * shade, color[1] * shade, color[2] * shade, 1.],
        );
        for (normal, p) in [
            (-axis, [a, point(a, j), point(a, i)]),
            (axis, [b, point(b, i), point(b, j)]),
        ] {
            let first = out.positions.len() as u32;
            out.positions.extend(p.map(|p| p.to_array()));
            out.normals.extend([normal.to_array(); 3]);
            out.uvs.extend([[0.; 2]; 3]);
            out.colors.extend([color; 3]);
            out.indices.extend([first, first + 1, first + 2]);
        }
    }
}

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    g: &GridGeometry,
    p: &Placement,
    dark: bool,
    ice: bool,
    _origin: [i32; 2],
) {
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let unit = g.tile_height / SOURCE_TILE_HEIGHT;
    let z = |y: f32| n + (9. - 7. * y / 35.) * unit;
    let rails = if dark {
        [0.34, 0.25, 0.15, 1.]
    } else if ice {
        [0.47, 0.34, 0.20, 1.]
    } else {
        [0.44, 0.29, 0.12, 1.]
    };
    let rungs = [0.60, 0.43, 0.23, 1.];
    let left = w + 3.2 * unit;
    let right = w + 12.8 * unit;
    for x in [left, right] {
        beam(
            &mut mesh.solid,
            Vec3::new(x, 0.14 * unit, z(0.14)),
            Vec3::new(x, 34.86 * unit, z(34.86)),
            1.1 * unit,
            1.4 * unit,
            rails,
        );
    }
    for i in 0..11 {
        let y = 2.5 + i as f32 * 3.;
        beam(
            &mut mesh.solid,
            Vec3::new(left, y * unit, z(y)),
            Vec3::new(right, y * unit, z(y)),
            0.9 * unit,
            0.8 * unit,
            rungs,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beveled_ladder_beams_have_closed_caps_and_outward_faces() {
        for (a, b) in [
            (Vec3::new(3.2, 0.14, 9.), Vec3::new(3.2, 34.86, 2.)),
            (Vec3::new(3.2, 2.5, 8.5), Vec3::new(12.8, 2.5, 8.5)),
        ] {
            let mut mesh = SurfaceMeshData::default();
            beam(&mut mesh, a, b, 1.1, 1.4, [0.4, 0.3, 0.2, 1.]);
            assert_eq!(mesh.indices.len(), 96);
            let center = (a + b) * 0.5;
            for triangle in mesh.indices.chunks_exact(3) {
                let p = [triangle[0], triangle[1], triangle[2]]
                    .map(|i| Vec3::from_array(mesh.positions[i as usize]));
                let geometric = (p[1] - p[0]).cross(p[2] - p[0]).normalize();
                let authored = Vec3::from_array(mesh.normals[triangle[0] as usize]);
                assert!(geometric.dot(authored) > 0.999);
                assert!(authored.dot((p[0] + p[1] + p[2]) / 3. - center) > 0.);
            }
        }
    }
}


/// Native warp drawings are viewed from above, with no authored upright
/// ladder flight. A closed shallow volume retains their actual four source
/// tiles and floor footprint instead of obscuring the room with tall rails.
pub(super) fn append_native_warp(out:&mut SurfaceMeshData,g:&GridGeometry,
    p:&Placement,datum:f32) {
    let (w,_,n,_)=g.bounds(p.column,p.row);
    let e=w+2.*g.tile_width;let s=n+2.*g.tile_height;
    let top=datum+1.25*g.tile_height/SOURCE_TILE_HEIGHT;
    let first=out.positions.len();let indices=out.indices.len();
    super::super::modeled_interiors::architecture_box(out,[w,e,datum,top,n,s],[1.;3]);
    // Side faces use a texel of this opening's source edge. No brown/yellow
    // wood material or foreign cave/floor palette is introduced.
    let uv=g.uv(p.column,p.row);
    out.uvs[first..].fill([(uv.0+uv.1)*0.5,uv.2+(uv.3-uv.2)*7.5/8.]);
    out.indices.drain(indices..indices+6);
    for y in 0..2 {for x in 0..2 {
        append_top(out,g.bounds(p.column+x,p.row+y).into(),top,
            g.uv(p.column+x,p.row+y));
    }}
}
