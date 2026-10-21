//! A small radial cache of the finished scalp, used only while fitting hair.

use crate::mesh::SurfaceMeshData;
use bevy::math::{Vec2, Vec3};
use std::f32::consts::TAU;

const ROWS: usize = 33;
const SIDES: usize = 64;

pub(crate) struct HairScalp {
    center: Vec3,
    half_height: f32,
    radii: Vec<f32>,
}

impl HairScalp {
    pub(crate) fn new(
        mesh: &SurfaceMeshData,
        joints: &[[u16; 4]],
        weights: &[[f32; 4]],
        head: usize,
        center: Vec3,
        half_height: f32,
    ) -> Self {
        let selected: Vec<_> = joints
            .iter()
            .zip(weights)
            .map(|(ids, weights)| {
                (0..4)
                    .filter(|&s| usize::from(ids[s]) == head)
                    .map(|s| weights[s])
                    .sum::<f32>()
                    > 0.5
            })
            .collect();
        let triangles: Vec<_> = mesh
            .indices
            .as_chunks::<3>()
            .0
            .iter()
            .filter(|face| face.iter().all(|&i| selected[i as usize]))
            .map(|face| {
                [face[0], face[1], face[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]))
            })
            .collect();
        let mut radii = Vec::with_capacity(ROWS * SIDES);
        let origin = Vec2::new(center.x, center.z);
        for row in 0..ROWS {
            let y = center.y + half_height * (0.20 + 0.74 * row as f32 / (ROWS - 1) as f32);
            let mut segments = Vec::new();
            for triangle in &triangles {
                let mut hits = [Vec2::ZERO; 3];
                let mut count = 0;
                for (a, b) in [(0, 1), (1, 2), (2, 0)] {
                    let dy = triangle[b].y - triangle[a].y;
                    if dy.abs() < 1e-8 {
                        continue;
                    }
                    let t = (y - triangle[a].y) / dy;
                    if !(0. ..=1.).contains(&t) {
                        continue;
                    }
                    let p = triangle[a].lerp(triangle[b], t);
                    let p = Vec2::new(p.x, p.z);
                    if hits[..count]
                        .iter()
                        .all(|old| old.distance_squared(p) > 1e-14)
                    {
                        hits[count] = p;
                        count += 1;
                    }
                }
                if count == 2 {
                    segments.push([hits[0], hits[1]]);
                }
            }
            for side in 0..SIDES {
                let angle = TAU * side as f32 / SIDES as f32;
                let direction = Vec2::new(angle.sin(), angle.cos());
                let mut radius = 0_f32;
                for segment in &segments {
                    let edge = segment[1] - segment[0];
                    let cross = direction.perp_dot(edge);
                    if cross.abs() < 1e-9 {
                        continue;
                    }
                    let delta = segment[0] - origin;
                    let r = delta.perp_dot(edge) / cross;
                    let t = delta.perp_dot(direction) / cross;
                    if r > 0. && (-1e-5..=1.00001).contains(&t) {
                        radius = radius.max(r);
                    }
                }
                radii.push(radius);
            }
        }
        // A painted scalp can retain tiny cage bumps. Dilate, then average
        // with the same neighbourhood: every averaged term still contains
        // the original centre sample, so the smooth envelope stays outside
        // that sample instead of exposing it again after smoothing.
        let mut dilated = radii.clone();
        for row in 0..ROWS {
            for col in 0..SIDES {
                for dy in -1_isize..=1 {
                    for dx in -1_isize..=1 {
                        let r = (row as isize + dy).clamp(0, ROWS as isize - 1) as usize;
                        let c = (col as isize + dx).rem_euclid(SIDES as isize) as usize;
                        dilated[row * SIDES + col] =
                            dilated[row * SIDES + col].max(radii[r * SIDES + c]);
                    }
                }
            }
        }
        for row in 0..ROWS {
            for col in 0..SIDES {
                let mut sum = 0.;
                for (dy, wy) in [(-1_isize, 1.), (0, 2.), (1, 1.)] {
                    for (dx, wx) in [(-1_isize, 1.), (0, 2.), (1, 1.)] {
                        let r = (row as isize + dy).clamp(0, ROWS as isize - 1) as usize;
                        let c = (col as isize + dx).rem_euclid(SIDES as isize) as usize;
                        sum += dilated[r * SIDES + c] * wx * wy;
                    }
                }
                radii[row * SIDES + col] = sum / 16.;
            }
        }
        Self {
            center,
            half_height,
            radii,
        }
    }

    pub(crate) fn fit(&self, point: Vec3) -> Vec3 {
        let height = (point.y - self.center.y) / self.half_height;
        if !(0.20..=0.94).contains(&height) {
            return point;
        }
        let radial = Vec2::new(point.x - self.center.x, point.z - self.center.z);
        let radius = radial.length();
        if radius < 1e-6 {
            return point;
        }
        let angle = radial.x.atan2(radial.y).rem_euclid(TAU) / TAU * SIDES as f32;
        let column = angle.floor() as usize % SIDES;
        let fraction = angle.fract();
        let row = ((height - 0.20) / 0.74 * (ROWS - 1) as f32).clamp(0., (ROWS - 1) as f32);
        let lower = (row.floor() as usize).min(ROWS - 2);
        let at = |r: usize| {
            self.radii[r * SIDES + column] * (1. - fraction)
                + self.radii[r * SIDES + (column + 1) % SIDES] * fraction
        };
        let occupied = at(lower) + (at(lower + 1) - at(lower)) * (row - lower as f32);
        if occupied <= 0. {
            return point;
        }
        let smooth = |v: f32| {
            let t = v.clamp(0., 1.);
            t * t * (3. - 2. * t)
        };
        let cover = smooth((0.94 - height) / 0.20) * smooth((height - 0.20) / 0.12);
        let wanted = occupied + 0.015;
        let maximum =
            (radius + wanted + ((radius - wanted).powi(2) + 0.004_f32.powi(2)).sqrt()) * 0.5;
        let delta = radial / radius * ((maximum - radius) * cover);
        point + Vec3::new(delta.x, 0., delta.y)
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::TAU;

    use bevy::math::Vec3;

    use super::{HairScalp, SIDES};
    use crate::mesh::SurfaceMeshData;

    #[test]
    fn occupied_scalp_pushes_through_a_smaller_guessed_curtain() {
        let mesh = SurfaceMeshData {
            positions: vec![
                [-0.4, 2.8, -0.5],
                [0.4, 2.8, -0.5],
                [0.4, 3.9, -0.5],
                [-0.4, 3.9, -0.5],
                [-0.4, 2.8, 0.5],
                [0.4, 2.8, 0.5],
                [0.4, 3.9, 0.5],
                [-0.4, 3.9, 0.5],
            ],
            indices: vec![
                0, 2, 1, 0, 3, 2, 4, 5, 6, 4, 6, 7, 0, 4, 7, 0, 7, 3, 1, 2, 6, 1, 6, 5, 3, 7, 6, 3,
                6, 2, 0, 1, 5, 0, 5, 4,
            ],
            ..SurfaceMeshData::default()
        };
        let scalp = HairScalp::new(
            &mesh,
            &[[0; 4]; 8],
            &[[1., 0., 0., 0.]; 8],
            0,
            Vec3::new(0., 3.18, 0.),
            0.58,
        );
        let original = Vec3::new(0.38, 3.18 + 0.70 * 0.58, 0.);
        let fitted = scalp.fit(original);
        assert!(original.x < 0.4, "the old proxy exposes the actual scalp");
        let neighbourhood_support = 0.4 / (TAU * 2. / SIDES as f32).cos();
        assert!(
            fitted.x >= 0.414 && fitted.x < neighbourhood_support + 0.016,
            "fit the actual surface with a small gap: {fitted:?}"
        );
        assert_eq!(fitted.y, original.y);
        assert_eq!(fitted.z, original.z);
        let buried_root = Vec3::new(0.1, 3.18 + 0.94 * 0.58, 0.);
        assert!(
            scalp.fit(buried_root).distance(buried_root) < 1e-6,
            "keep the top closing rim buried"
        );
    }
}
