//! Low, warm timber edging; all geometry stays within the native border cells.
use super::*;

pub(super) fn append(mesh: &mut SurfaceMeshData, b: [f32; 4], u: f32) {
    let [w, e, n, s] = b;
    let a = u * 0.28;
    let c = u * 0.62;
    beam(
        mesh,
        [w + a, e - a, n + a, n + c],
        u * 0.56,
        u,
        [0.46, 0.33, 0.17],
    );
    beam(
        mesh,
        [w + a, e - a, s - c, s - a],
        u * 0.56,
        u,
        [0.46, 0.33, 0.17],
    );
    beam(
        mesh,
        [w + a, w + c, n + c, s - c],
        u * 0.56,
        u,
        [0.46, 0.33, 0.17],
    );
    beam(
        mesh,
        [e - c, e - a, n + c, s - c],
        u * 0.56,
        u,
        [0.46, 0.33, 0.17],
    );
    for x in [w + u * 0.45, e - u * 0.45] {
        for z in [n + u * 0.45, s - u * 0.45] {
            beam(
                mesh,
                [x - u * 0.24, x + u * 0.24, z - u * 0.24, z + u * 0.24],
                u * 0.70,
                u,
                [0.50, 0.37, 0.19],
            );
        }
    }
}

fn beam(mesh: &mut SurfaceMeshData, b: [f32; 4], h: f32, u: f32, pigment: [f32; 3]) {
    let [w, e, n, s] = b;
    let bevel = (u * 0.045).min((e - w) * 0.2).min((s - n) * 0.2);
    let rectangle = |inset: f32, y: f32| {
        [
            [w + inset, y, n + inset],
            [w + inset, y, s - inset],
            [e - inset, y, s - inset],
            [e - inset, y, n + inset],
        ]
    };
    let bottom = rectangle(bevel, 0.);
    let lower = rectangle(0., bevel);
    let upper = rectangle(0., h - bevel);
    let top = rectangle(bevel, h);
    // Long panels receive restrained continuous grain, rather than repeated
    // source pixels or independent random colors on each neighboring segment.
    for (a, b) in [(bottom, lower), (lower, upper), (upper, top)] {
        for face in 0..4 {
            let next = (face + 1) % 4;
            strip(mesh, [a[face], a[next], b[next], b[face]], u, pigment);
        }
    }
    strip(mesh, top, u, pigment);
}

fn strip(mesh: &mut SurfaceMeshData, mut p: [[f32; 3]; 4], u: f32, pigment: [f32; 3]) {
    if (Vec3::from_array(p[1]) - Vec3::from_array(p[0])).length_squared()
        < (Vec3::from_array(p[3]) - Vec3::from_array(p[0])).length_squared()
    {
        p.rotate_left(1);
    }
    let a = Vec3::from_array(p[0]);
    let b = Vec3::from_array(p[1]);
    let c = Vec3::from_array(p[3]);
    let normal = (b - a).cross(c - a).normalize();
    let along = b - a;
    let count = (along.length() / (u * 0.5)).ceil().max(1.) as usize;
    let upper = Vec3::from_array(p[2]);
    let sample = |t: f32, v: f32| (a.lerp(b, t).lerp(c.lerp(upper, t), v)).to_array();
    for i in 0..count {
        let t = i as f32 / count as f32;
        let next = (i + 1) as f32 / count as f32;
        let points = [
            sample(t, 0.),
            sample(next, 0.),
            sample(next, 1.),
            sample(t, 1.),
        ];
        let colors = points.map(|p| {
            let grain = 1.
                + 0.018 * ((p[0] * 0.31 + p[2] * 0.23) / u).sin()
                + 0.012 * ((p[0] * 0.77 - p[2] * 0.61) / u).sin();
            let shade = 0.86 + 0.22 * normal.y.max(0.);
            [
                pigment[0] * grain * shade,
                pigment[1] * grain * shade,
                pigment[2] * grain * shade,
                1.,
            ]
        });
        append_quad_colors(mesh, points, normal.to_array(), [[0.; 2]; 4], colors);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn park_flowerbed_edge_is_floor_rooted_open_and_finite() {
        for width in [8., 12., 20.] {
            let mut m = SurfaceMeshData::default();
            let b = [-32., -32. + width * 8., -24., 8.];
            append(&mut m, b, 8.);
            assert!(m.positions.iter().all(|p| p.iter().all(|v| v.is_finite())
                && p[0] >= b[0]
                && p[0] <= b[1]
                && p[2] >= b[2]
                && p[2] <= b[3]
                && p[1] >= 0.
                && p[1] <= 5.61));
            assert!(m.positions.iter().any(|p| p[1] == 0.));
            assert!(
                m.positions.iter().all(|p| p[0] <= b[0] + 8.
                    || p[0] >= b[1] - 8.
                    || p[2] <= b[2] + 8.
                    || p[2] >= b[3] - 8.),
                "flower interior must remain empty"
            );
            assert!(m.normals.iter().all(|p| p.iter().all(|v| v.is_finite())
                && (Vec3::from_array(*p).length() - 1.).abs() < 0.001));
            assert!(
                m.colors
                    .iter()
                    .all(|c| c.iter().all(|v| v.is_finite() && *v >= 0. && *v <= 1.))
            );
            for t in m.indices.chunks_exact(3) {
                let a = Vec3::from_array(m.positions[t[0] as usize]);
                let b = Vec3::from_array(m.positions[t[1] as usize]);
                let c = Vec3::from_array(m.positions[t[2] as usize]);
                assert!((b - a).cross(c - a).length() > 0.0001);
            }
        }
    }
}
