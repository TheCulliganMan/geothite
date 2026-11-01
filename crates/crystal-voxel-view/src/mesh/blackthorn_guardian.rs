//! Compact, original horned dragon carvings for Blackthorn's four-cell drawings.
//! The authenticated drawing supplies the blue stone; this never owns gameplay.
use crate::mesh::SurfaceMeshData;
use bevy::prelude::Vec3;

struct Sculptor<'a> {
    mesh: &'a mut SurfaceMeshData,
    origin: Vec3,
    scale: Vec3,
}
impl Sculptor<'_> {
    fn triangle(&mut self, p: [Vec3; 3], n: [Vec3; 3], uv: [f32; 2], tint: f32) {
        let mut p = p.map(|v| self.origin + v * self.scale);
        let mut n = n.map(|v| (v / self.scale).normalize());
        let cross = (p[1] - p[0]).cross(p[2] - p[0]);
        if cross.length_squared() < 1e-12 {
            return;
        }
        if cross.dot(n[0] + n[1] + n[2]) < 0. {
            p.swap(1, 2);
            n.swap(1, 2);
        }
        let first = self.mesh.positions.len() as u32;
        self.mesh.positions.extend(p.map(|v| v.to_array()));
        self.mesh.normals.extend(n.map(|v| v.to_array()));
        self.mesh.uvs.extend([uv; 3]);
        self.mesh.colors.extend(
            n.map(|v| crate::interior_models::authored_face_color([tint, tint, tint, 1.], v)),
        );
        self.mesh.indices.extend([first, first + 1, first + 2]);
    }
    fn oval(&mut self, center: [f32; 3], radius: [f32; 3], uv: [f32; 2], tint: f32) {
        let center = Vec3::from_array(center);
        let radius = Vec3::from_array(radius);
        let vertex = |latitude: usize, longitude: usize| {
            let a = std::f32::consts::PI * latitude as f32 / 8.;
            let b = std::f32::consts::TAU * longitude as f32 / 12.;
            let unit = Vec3::new(a.sin() * b.cos(), a.cos(), a.sin() * b.sin());
            (center + unit * radius, (unit / radius).normalize())
        };
        for y in 0..8 {
            for x in 0..12 {
                let corners = [
                    vertex(y, x),
                    vertex(y + 1, x),
                    vertex(y + 1, x + 1),
                    vertex(y, x + 1),
                ];
                for ids in [[0, 1, 2], [0, 2, 3]] {
                    self.triangle(
                        ids.map(|i| corners[i].0),
                        ids.map(|i| corners[i].1),
                        uv,
                        tint,
                    );
                }
            }
        }
    }
    fn horn(&mut self, base: Vec3, tip: Vec3, radius: f32, uv: [f32; 2]) {
        let axis = (tip - base).normalize();
        let u = axis.any_orthonormal_vector();
        let v = axis.cross(u);
        let ring = |i: usize| {
            base + radius
                * (u * (std::f32::consts::TAU * i as f32 / 8.).cos()
                    + v * (std::f32::consts::TAU * i as f32 / 8.).sin())
        };
        for i in 0..8 {
            let p = [ring(i), ring(i + 1), tip];
            let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize();
            self.triangle(p, [normal; 3], uv, 1.);
            self.triangle([base, ring(i + 1), ring(i)], [-axis; 3], uv, 1.);
        }
    }
}

pub(super) fn append(
    mesh: &mut SurfaceMeshData,
    b: [f32; 4],
    height: f32,
    stone: [f32; 2],
    light: [f32; 2],
) {
    let [w, e, n, s] = b;
    let mut sculptor = Sculptor {
        mesh,
        origin: Vec3::new((w + e) * 0.5, 0., (n + s) * 0.5),
        scale: Vec3::new(e - w, height, s - n),
    };
    sculptor.oval([0., 0.12, 0.], [0.36, 0.12, 0.30], stone, 0.85);
    sculptor.oval([0., 0.34, 0.], [0.34, 0.26, 0.28], stone, 1.);
    sculptor.oval([0., 0.68, 0.], [0.30, 0.25, 0.26], light, 1.);
    sculptor.oval([0., 0.58, 0.22], [0.21, 0.14, 0.15], stone, 1.);
    for sign in [-1., 1.] {
        sculptor.horn(
            Vec3::new(sign * 0.20, 0.76, -0.07),
            Vec3::new(sign * 0.30, 1., -0.13),
            0.075,
            light,
        );
        sculptor.oval([sign * 0.19, 0.17, 0.22], [0.10, 0.09, 0.12], light, 1.);
        sculptor.oval(
            [sign * 0.16, 0.72, 0.238],
            [0.045, 0.032, 0.02],
            stone,
            0.18,
        );
        sculptor.oval([sign * 0.16, 0.767, 0.222], [0.09, 0.026, 0.034], light, 1.);
    }
    sculptor.oval([0., 0.53, 0.362], [0.12, 0.016, 0.012], stone, 0.18);
}

/// Sprout's flower-headed guardian has a slender stem and broad leaf arms,
/// unlike the horned gym guardians. All six finishes remain live atlas samples.
pub(super) fn append_sprout(
    mesh: &mut SurfaceMeshData,
    b: [f32; 4],
    height: f32,
    sample: impl Fn([usize; 2]) -> [f32; 2],
) {
    let [w, e, n, s] = b;
    let mut sculptor = Sculptor {
        mesh,
        origin: Vec3::new((w + e) * 0.5, 0., (n + s) * 0.5),
        scale: Vec3::new(e - w, height, s - n),
    };
    let gold = sample([8, 6]);
    let gold_light = sample([8, 1]);
    let gold_dark = sample([6, 0]);
    sculptor.oval([0., 0.05, 0.], [0.43, 0.05, 0.32], sample([0, 24]), 1.);
    sculptor.oval([0., 0.14, 0.], [0.31, 0.10, 0.25], sample([8, 24]), 1.);
    sculptor.oval([0., 0.22, 0.], [0.36, 0.035, 0.28], sample([2, 26]), 1.);
    sculptor.horn(
        Vec3::new(0., 0.20, 0.),
        Vec3::new(0., 0.64, -0.015),
        0.035,
        gold,
    );
    for sign in [-1., 1.] {
        sculptor.oval([sign * 0.17, 0.45, -0.015], [0.18, 0.055, 0.10], gold, 1.);
    }
    sculptor.oval([0., 0.77, 0.], [0.28, 0.23, 0.26], gold, 1.);
    sculptor.oval([0., 0.91, 0.213], [0.06, 0.025, 0.014], gold_light, 1.);
    sculptor.oval([0., 0.76, 0.262], [0.09, 0.10, 0.018], gold_dark, 0.55);
    for sign in [-1., 1.] {
        sculptor.oval(
            [sign * 0.095, 0.87, 0.22],
            [0.025, 0.03, 0.018],
            gold_dark,
            0.55,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compact_guardians_keep_closed_outward_faces_live_stone_and_native_bounds() {
        for scale in [0.5, 1., 2.] {
            let mut mesh = SurfaceMeshData::default();
            append(
                &mut mesh,
                [-8. * scale, 8. * scale, -6. * scale, 6. * scale],
                16. * scale,
                [0.2, 0.3],
                [0.4, 0.5],
            );
            assert!(mesh.indices.len() / 3 < 3000);
            for (p, n) in mesh.positions.iter().zip(&mesh.normals) {
                assert!(
                    p[0].abs() <= 8. * scale
                        && p[2].abs() <= 6. * scale
                        && p[1] >= -0.0001
                        && p[1] <= 16. * scale + 0.0001
                );
                assert!((Vec3::from_array(*n).length() - 1.).abs() < 0.0001);
            }
            for tri in mesh.indices.chunks_exact(3) {
                let [a, b, c] =
                    [tri[0], tri[1], tri[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
                for &i in tri {
                    assert!(
                        (b - a)
                            .cross(c - a)
                            .dot(Vec3::from_array(mesh.normals[i as usize]))
                            > 0.
                    );
                }
            }
            assert!(
                mesh.uvs
                    .iter()
                    .all(|uv| *uv == [0.2, 0.3] || *uv == [0.4, 0.5])
            );
        }
    }
    #[test]
    fn sprout_flower_guardian_retains_all_six_live_palette_regions_and_floor_contact() {
        let mut mesh = SurfaceMeshData::default();
        append_sprout(&mut mesh, [-8., 8., -5., 5.], 24., |p| {
            [p[0] as f32 / 16., p[1] as f32 / 32.]
        });
        assert!(mesh.indices.len() / 3 < 3000);
        for p in [[8, 6], [8, 1], [6, 0], [8, 24], [2, 26], [0, 24]] {
            assert!(mesh.uvs.contains(&[p[0] as f32 / 16., p[1] as f32 / 32.]));
        }
        assert!(mesh.positions.iter().any(|p| p[1] == 0.));
        for (p, n) in mesh.positions.iter().zip(&mesh.normals) {
            assert!(p[0].abs() <= 8. && p[2].abs() <= 5. && p[1] >= 0. && p[1] <= 24.);
            assert!((Vec3::from_array(*n).length() - 1.).abs() < 0.0001);
        }
    }
}
