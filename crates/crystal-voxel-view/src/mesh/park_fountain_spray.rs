//! Fine round streams break into droplets on their descending side.

use std::f32::consts::TAU;

use bevy::prelude::Vec3;

use crate::mesh::{SurfaceMeshData, append_quad_colors};

const SIDES: usize = 6;
const STEPS: usize = 24;

pub(super) fn append(
    mesh: &mut SurfaceMeshData,
    center: [f32; 2],
    radius: [f32; 2],
    water: f32,
    landing: f32,
    tile: f32,
    scale: f32,
) {
    let origin = Vec3::new(center[0], water + tile * 0.09, center[1]);
    // A slightly wandering vertical plume, with a rounded head and separate
    // falling beads. Every stream is a volume, visible from below and behind.
    let plume = |t: f32| {
        origin
            + Vec3::new(
                (t * 8.).sin() * tile * 0.015 * t,
                tile * 1.85 * scale * t,
                (t * 5.).sin() * tile * 0.012 * t,
            )
    };
    for step in 0..STEPS {
        let t0 = step as f32 / STEPS as f32;
        let t1 = (step + 1) as f32 / STEPS as f32;
        tube(
            mesh,
            [plume(t0), plume(t1)],
            [
                tile * 0.022 * scale * (1. - t0 * 0.60),
                tile * 0.022 * scale * (1. - t1 * 0.60),
            ],
            0.,
            [t0, t1],
        );
    }
    droplet(mesh, plume(1.), tile * 0.033 * scale, 0., 1.);
    for stream in 0..8 {
        let angle = stream as f32 / 8. * TAU + 0.17;
        let reach = Vec3::new(
            angle.cos() * radius[0] * 1.62,
            0.,
            angle.sin() * radius[1] * 1.62,
        );
        let start = origin + Vec3::Y * tile * 0.12 * scale;
        let end = Vec3::new(center[0], landing + tile * 0.01, center[1]) + reach;
        let height = tile * scale * (1.2 + 0.15 * (angle * 3.).sin());
        let point = |t: f32| start.lerp(end, t) + Vec3::Y * height * 4. * t * (1. - t);
        for step in 0..STEPS {
            let t0 = step as f32 / STEPS as f32;
            let t1 = (step + 1) as f32 / STEPS as f32;
            if step < 17 {
                tube(
                    mesh,
                    [point(t0), point(t1)],
                    [
                        tile * (0.014 - t0 * 0.006) * scale.sqrt(),
                        tile * (0.014 - t1 * 0.006) * scale.sqrt(),
                    ],
                    (stream + 1) as f32,
                    [t0, t1],
                );
            } else {
                let t = (step as f32 + 0.35) / STEPS as f32;
                droplet(
                    mesh,
                    point(t),
                    tile * (0.022 + 0.007 * (angle + t * 23.).sin()) * scale.sqrt(),
                    (stream + 1) as f32,
                    t,
                );
            }
        }
        // Small upward splash beads terminate the stream at the actual pool.
        for splash in 0..3 {
            let turn = angle + splash as f32 * 2.4;
            let p = end
                + Vec3::new(
                    turn.cos() * tile * 0.065,
                    tile * (0.04 + splash as f32 * 0.045) * scale,
                    turn.sin() * tile * 0.065,
                );
            droplet(
                mesh,
                p,
                tile * 0.018 * scale.sqrt(),
                (stream + 1) as f32,
                0.98,
            );
        }
    }
}

fn tube(
    mesh: &mut SurfaceMeshData,
    endpoints: [Vec3; 2],
    radii: [f32; 2],
    stream: f32,
    progress: [f32; 2],
) {
    let [a, b] = endpoints;
    let [r0, r1] = radii;
    let [t0, t1] = progress;
    let axis = (b - a).normalize();
    let across = axis
        .cross(if axis.y.abs() > 0.9 { Vec3::X } else { Vec3::Y })
        .normalize();
    let other = axis.cross(across).normalize();
    for side in 0..SIDES {
        let normal = |i: usize| {
            let angle = (i % SIDES) as f32 / SIDES as f32 * TAU;
            across * angle.cos() + other * angle.sin()
        };
        let u = normal(side);
        let v = normal(side + 1);
        let p = [a + u * r0, a + v * r0, b + v * r1, b + u * r1];
        let normals = [u, v, v, u];
        let colors = normals.map(|n| {
            let light = 0.72 + 0.28 * n.dot(Vec3::new(-0.5, 0.7, 0.5).normalize()).max(0.);
            [0.45 * light, 0.72 * light, 0.74 * light, 1.]
        });
        append_quad_colors(
            mesh,
            p.map(|p| p.to_array()),
            (u + v).normalize().to_array(),
            [
                [70000. + stream, t0],
                [70000. + stream, t0],
                [70000. + stream, t1],
                [70000. + stream, t1],
            ],
            colors,
        );
        let first = mesh.normals.len() - 4;
        mesh.normals[first..].copy_from_slice(&normals.map(|n| n.to_array()));
    }
}

fn droplet(mesh: &mut SurfaceMeshData, p: Vec3, radius: f32, stream: f32, t: f32) {
    const SIDES: usize = 8;
    const RINGS: usize = 6;
    // Rounded drops retain smooth highlights in a close first-person view.
    // The two poles are real closed fans, never needle-shaped diamonds.
    let ring = |latitude: usize, angle: f32| {
        let phi = latitude as f32 / RINGS as f32 * std::f32::consts::PI;
        let radial = if latitude == 0 || latitude == RINGS {
            0.
        } else {
            phi.sin()
        };
        let offset = Vec3::new(radial * angle.cos(), phi.cos() * 1.25, radial * angle.sin());
        let normal = Vec3::new(offset.x, phi.cos() / 1.25, offset.z).normalize();
        (p + offset * radius * 0.83, normal)
    };
    for latitude in 0..RINGS {
        for side in 0..SIDES {
            let a = side as f32 / SIDES as f32 * TAU;
            let b = (side + 1) as f32 / SIDES as f32 * TAU;
            let mut vertices = [
                ring(latitude, a),
                ring(latitude, b),
                ring(latitude + 1, b),
                ring(latitude + 1, a),
            ];
            if latitude == 0 {
                vertices = [vertices[0], vertices[2], vertices[3], vertices[0]];
            } else if latitude == RINGS - 1 {
                vertices[3] = vertices[0];
            }
            append_quad_colors(
                mesh,
                vertices.map(|(p, _)| p.to_array()),
                vertices[0].1.to_array(),
                [[70000. + stream, t]; 4],
                [[0.52, 0.78, 0.80, 1.]; 4],
            );
            let first = mesh.normals.len() - 4;
            mesh.normals[first..].copy_from_slice(&vertices.map(|(_, n)| n.to_array()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pond_jets_rise_and_land_outside_the_bowl_at_the_pond_datum() {
        let mut mesh = SurfaceMeshData::default();
        append(&mut mesh, [12., -8.], [7.04, 5.2], 3., -2., 8., 1.);
        assert!(
            mesh.positions
                .iter()
                .all(|p| Vec3::from_array(*p).is_finite() && p[1] > -2.1 && p[1] < 19.)
        );
        assert!(mesh.positions.iter().any(|p| p[1] > 18.));
        assert!(
            mesh.positions
                .iter()
                .any(|p| p[1] < -1. && (p[0] - 12.).abs() > 7.)
        );
        assert!(
            mesh.positions
                .iter()
                .all(|p| (p[0] - 12.).abs() < 12. && (p[2] + 8.).abs() < 9.)
        );
    }
}
