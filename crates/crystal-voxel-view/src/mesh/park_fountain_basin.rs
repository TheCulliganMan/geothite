//! A hollow stone basin inside the existing source-derived oval footprint.
use super::*;

pub(super) fn append(
    mesh: &mut SurfaceMeshData,
    center: [f32; 2],
    radius: [f32; 2],
    base: f32,
    top: f32,
    tile: f32,
) {
    const SEGMENTS: usize = 32;
    let bevel = tile * 0.045;
    let water = top - tile * 0.125;
    let point = |angle: f32, scale: f32, y: f32| {
        [
            center[0] + angle.cos() * radius[0] * scale,
            y,
            center[1] + angle.sin() * radius[1] * scale,
        ]
    };
    // Outer wall, worn bevel, coping and inner wall share exact ring samples.
    let rings = [
        (1., base),
        (1., top - bevel),
        (0.96, top),
        (0.78, top),
        (0.78, water),
    ];
    for segment in 0..SEGMENTS {
        let a = segment as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
        let b = ((segment + 1) % SEGMENTS) as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
        for pair in rings.windows(2) {
            let p = [
                point(a, pair[0].0, pair[0].1),
                point(a, pair[1].0, pair[1].1),
                point(b, pair[1].0, pair[1].1),
                point(b, pair[0].0, pair[0].1),
            ];
            let normal = (Vec3::from_array(p[1]) - Vec3::from_array(p[0]))
                .cross(Vec3::from_array(p[2]) - Vec3::from_array(p[0]))
                .normalize();
            let tone = 0.96 + 0.025 * (a * 3. + 0.7).sin();
            let light = 0.80 + 0.20 * normal.dot(Vec3::new(0.45, 0.80, 0.55).normalize()).max(0.);
            append_solid_quad(
                mesh,
                p,
                normal.to_array(),
                [
                    0.48 * tone * light,
                    0.45 * tone * light,
                    0.35 * tone * light,
                    1.,
                ],
            );
        }
        // Terrain finishers require four-vertex faces. As with append_polygon,
        // the fan's second triangle repeats the center deliberately.
        let center_point = [center[0], water, center[1]];
        append_solid_quad(
            mesh,
            [
                center_point,
                point(b, 0.78, water),
                point(a, 0.78, water),
                center_point,
            ],
            [0., 1., 0.],
            [0.025, 0.16, 0.19, 1.],
        );
        // Bowl ripples use bowl-relative coordinates: its raised water is a
        // separate surface from the pond, but stays anchored while scrolling.
        let start = mesh.uvs.len()-4;
        for i in start..mesh.uvs.len() {
            let p = mesh.positions[i];
            mesh.uvs[i] = [50000.+(p[0]-center[0])/tile,
                -50000.+(p[2]-center[1])/tile];
        }
    }
}

/// Fine water threads rise above the hollow bowl and fall inside its water
/// surface. The source spray tiles are replaced only with a complete fixture.
pub(super) fn append_spray(
    mesh: &mut SurfaceMeshData,
    center: [f32; 2],
    radius: [f32; 2],
    water: f32,
    tile: f32,
) {
    let tube = |mesh: &mut SurfaceMeshData, a: Vec3, b: Vec3, r: f32, tint: [f32; 4]| {
        let axis = (b-a).normalize();
        let across = axis.cross(if axis.y.abs() > 0.9 { Vec3::X } else { Vec3::Y }).normalize();
        let other = axis.cross(across).normalize();
        for side in 0..6 {
            let normal = |i: usize| {
                let angle = (i % 6) as f32 / 6. * std::f32::consts::TAU;
                across * angle.cos() + other * angle.sin()
            };
            let u = normal(side);
            let v = normal(side+1);
            let p = [a+u*r, a+v*r, b+v*r, b+u*r];
            append_solid_quad(mesh, p.map(|p| p.to_array()),
                (u+v).normalize().to_array(), tint);
            let start = mesh.uvs.len()-4;
            mesh.uvs[start..].copy_from_slice(&[[70000.,0.],[70000.,0.],[70000.,1.],[70000.,1.]]);
        }
    };
    let center = Vec3::new(center[0], water, center[1]);
    tube(mesh, center, center + Vec3::Y * tile * 0.92, tile * 0.012,
        [0.64, 0.82, 0.82, 1.]);
    for stream in 0..8 {
        let angle = stream as f32 / 8. * std::f32::consts::TAU + 0.17;
        let reach = Vec3::new(angle.cos()*radius[0]*0.63, 0., angle.sin()*radius[1]*0.63);
        let point = |step: usize| {
            let t = step as f32 / 12.;
            center + reach * t + Vec3::Y * tile * (0.16*(1.-t) + 2.5*t*(1.-t))
        };
        let across = Vec3::new(-reach.z, 0., reach.x).normalize();
        let rings: Vec<_> = (0..=12).map(|step| {
            let t = step as f32 / 12.;
            let axis = (reach + Vec3::Y * tile * (-0.16 + 2.5*(1.-2.*t))).normalize();
            let other = axis.cross(across).normalize();
            let normals: [Vec3; 6] = std::array::from_fn(|side| {
                let turn = side as f32 / 6. * std::f32::consts::TAU;
                across*turn.cos() + other*turn.sin()
            });
            let r = tile * (0.010 - t*0.003);
            (normals.map(|n| point(step)+n*r), normals)
        }).collect();
        for (step, pair) in rings.windows(2).enumerate() {
            for side in 0..6 {
                let next = (side+1)%6;
                let p = [pair[0].0[side], pair[0].0[next], pair[1].0[next], pair[1].0[side]];
                let normal = (pair[0].1[side]+pair[0].1[next]
                    +pair[1].1[side]+pair[1].1[next]).normalize();
                append_solid_quad(mesh, p.map(|p| p.to_array()), normal.to_array(),
                    [0.52, 0.75, 0.77, 1.]);
                let start = mesh.uvs.len()-4;
                let tag = 70001. + stream as f32;
                let a = step as f32 / 12.;
                let b = (step+1) as f32 / 12.;
                mesh.uvs[start..].copy_from_slice(&[[tag,a],[tag,a],[tag,b],[tag,b]]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn park_spray_rises_above_the_basin_and_lands_inside_it() {
        let mut mesh = SurfaceMeshData::default();
        append_spray(&mut mesh, [12., -8.], [7.04, 5.2], 3., 8.);
        assert!(mesh.positions.iter().all(|p| p.iter().all(|v| v.is_finite())
            && p[1] > 2.9 && p[1] < 10.5
            && ((p[0]-12.)/7.04).powi(2)+((p[2]+8.)/5.2).powi(2) < 0.6));
        assert!(mesh.positions.iter().any(|p| p[1] > 10.));
        assert!(mesh.normals.iter().all(|n| (Vec3::from_array(*n).length()-1.).abs()<0.001));
        assert!(mesh.indices.iter().all(|&i| (i as usize) < mesh.positions.len()));
        for p in mesh.positions.chunks_exact(4) {
            let [a,b,c] = [p[0],p[1],p[2]].map(Vec3::from_array);
            assert!((b-a).cross(c-a).length_squared()>1e-10);
        }
    }
    #[test]
    fn park_basin_has_a_recessed_center_and_closed_shared_rings() {
        let mut mesh = SurfaceMeshData::default();
        append(&mut mesh, [0., 0.], [5.92, 4.], -2., 3., 8.);
        assert!(
            mesh.positions
                .iter()
                .all(|p| p.iter().all(|v| v.is_finite())
                    && p[1] >= -2.
                    && p[1] <= 3.
                    && (p[0] / 5.92).powi(2) + (p[2] / 4.).powi(2) <= 1.0001)
        );
        assert!(
            mesh.positions
                .iter()
                .filter(|p| p[0] == 0. && p[2] == 0.)
                .all(|p| p[1] == 2.0)
        );
        assert!(
            mesh.normals
                .iter()
                .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 0.001)
        );
        assert_eq!(mesh.positions.len() % 4, 0);
        for t in mesh.indices.chunks_exact(6) {
            let [a, b, c] =
                [t[0], t[1], t[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
            assert!((b - a).cross(c - a).length_squared() > 0.00001);
        }
    }
}
