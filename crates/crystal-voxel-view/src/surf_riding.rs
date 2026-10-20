//! Fit the rider to the actual carrying surface, then retain that contact as
//! the Pokémon animates. All coordinates remain local to its existing skin.
use super::*;
use crate::species_rig::SpeciesRig;
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Copy, Debug)]
pub(super) struct SurfSeat {
    pub point: Vec3,
    pub half_width: f32,
    /// Body-local water plane. The carrying body floats partly submerged;
    /// its lowest vertex is not a walking foot placed above the water.
    pub waterline: f32,
    vertices: [usize; 3],
    weights: Vec3,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::species_rig::{self, Species};

    #[test]
    fn surf_seats_use_body_surfaces_and_follow_the_animated_skin() {
        for species in [
            Species::Totodile,
            Species::Pikachu,
            Species::Lapras,
            Species::Gyarados,
            Species::Procedural("goldeen"),
        ] {
            let rig = species_rig::motion::world_rig(species_rig::rig(species));
            let seat = SurfSeat::for_rig(rig);
            if species == Species::Gyarados {
                let original = SurfSeat::fit(species_rig::rig(species));
                assert!(seat.point.abs_diff_eq(original.point, 0.0001),
                    "world body joints changed the neutral rider seat");
                assert!((seat.waterline-original.waterline).abs() < 0.0001);
            }
            assert!(seat.point.is_finite() && seat.half_width.is_finite());
            assert!(
                seat.point.y > 0.1 && seat.point.y < rig.neutral_bounds.1.y * 0.85,
                "{} must carry the trainer on its body, got {:?}",
                species.name(),
                seat.point
            );
            assert!(seat.half_width >= 0.12);
            let mut pose = rig.joints.iter().map(|j| j.bind).collect::<Vec<_>>();
            let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            assert!(
                seat.animated_point(rig, &matrices)
                    .abs_diff_eq(seat.point, 0.0001)
            );
            for tick in 0..32 {
                rig.sample_swimming(tick as f32 / 16., tick as f32 * std::f32::consts::TAU / 32.,
                    1., &mut pose);
                // The rider must inherit swimming lift as well as joint motion.
                pose[0].translation += Vec3::new(0.02, 0.16, -0.01);
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                let contact = seat.animated_point(rig, &matrices);
                assert!(contact.is_finite());
                assert!(contact.distance(seat.point) > 0.08);
            }
        }
    }
}

impl SurfSeat {
    pub fn for_rig(rig: &'static SpeciesRig) -> Self {
        static SEATS: OnceLock<Mutex<HashMap<&'static str, SurfSeat>>> = OnceLock::new();
        *SEATS
            .get_or_init(Default::default)
            .lock()
            .unwrap()
            .entry(rig.species.name())
            .or_insert_with(|| Self::fit(rig))
    }

    fn fit(rig: &SpeciesRig) -> Self {
        // Head, ears, tails and shell spikes are not seats. Joint ownership
        // also handles continuous sculptures with no separate torso primitive.
        let body_weight = |v: usize| {
            rig.joint_indices[v]
                .iter()
                .zip(rig.joint_weights[v])
                .filter(|(j, _)| {
                    **j == 0
                        || matches!(
                            rig.joints[**j as usize].name.rsplit('/').next(),
                            Some("body" | "torso" | "shell")
                        )
                        || rig.joints[**j as usize].name.rsplit('/').next()
                            .is_some_and(|name| name.starts_with("swim_wave_"))
                })
                .map(|(_, weight)| weight)
                .sum::<f32>()
        };
        let body_part =
            |name: &str| matches!(name.split(" / ").next(), Some("Body" | "Shell" | "Torso"));
        let has_body_parts = rig.anatomy.iter().any(|part| body_part(&part.name));
        let mut carrying_surface = vec![!has_body_parts; rig.neutral.positions.len()];
        for part in &rig.anatomy {
            if body_part(&part.name) {
                carrying_surface[part.vertices.clone()].fill(true);
            }
        }
        let points = &rig.neutral.positions;
        let body = (0..points.len())
            .filter(|&v| carrying_surface[v] && body_weight(v) >= 0.7)
            .collect::<Vec<_>>();
        let bounds = body.iter().map(|&v| Vec3::from_array(points[v])).fold(
            (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
            |(low, high), p| (low.min(p), high.max(p)),
        );
        let (low, high) = if body.is_empty() {
            rig.neutral_bounds
        } else {
            bounds
        };
        let target = Vec2::new((low.x + high.x) * 0.5, low.z + (high.z - low.z) * 0.28);
        let mut best: Option<Self> = None;
        for part in &rig.anatomy {
            if has_body_parts && !body_part(&part.name) {
                continue;
            }
            if part.name.to_ascii_lowercase().contains("spike") {
                continue;
            }
            for triangle in rig.neutral.indices[part.indices.clone()].chunks_exact(3) {
                let vertices = [
                    triangle[0] as usize,
                    triangle[1] as usize,
                    triangle[2] as usize,
                ];
                let [a, b, c] = vertices.map(|v| Vec3::from_array(points[v]));
                let u = Vec2::new(b.x - a.x, b.z - a.z);
                let v = Vec2::new(c.x - a.x, c.z - a.z);
                let d = u.perp_dot(v);
                if d.abs() < 1e-8 {
                    continue;
                }
                let q = target - Vec2::new(a.x, a.z);
                let wb = q.perp_dot(v) / d;
                let wc = u.perp_dot(q) / d;
                let weights = Vec3::new(1. - wb - wc, wb, wc);
                if weights.min_element() < -0.0001 {
                    continue;
                }
                let point = a * weights.x + b * weights.y + c * weights.z;
                let support = Vec3::from_array(vertices.map(body_weight)).dot(weights);
                if support < 0.5 || point.y < low.y + (high.y - low.y) * 0.32 {
                    continue;
                }
                if best.is_none_or(|seat| point.y > seat.point.y) {
                    best = Some(Self {
                        point,
                        vertices,
                        weights,
                        half_width: 0.,
                        waterline: 0.,
                    });
                }
            }
        }
        // Unusual open/segmented bodies may have no triangle at the center.
        // Use their nearest upper body vertex, preserving a real skin contact.
        let mut seat = best.unwrap_or_else(|| {
            let v = body
                .iter()
                .copied()
                .chain((body.is_empty()).then_some(0))
                .filter(|&v| points[v][1] >= low.y + (high.y - low.y) * 0.65)
                .min_by(|&a, &b| {
                    let score = |v: usize| {
                        let p = Vec3::from_array(points[v]);
                        Vec2::new(p.x, p.z).distance_squared(target) - p.y * 0.04
                    };
                    score(a).total_cmp(&score(b))
                })
                .unwrap_or_else(|| body.first().copied().unwrap_or(0));
            Self {
                point: Vec3::from_array(points[v]),
                vertices: [v; 3],
                weights: Vec3::X,
                half_width: 0.,
                waterline: 0.,
            }
        });
        seat.half_width = body
            .iter()
            .map(|&v| Vec3::from_array(points[v]))
            .filter(|p| (p.z - seat.point.z).abs() < (high.z - low.z) * 0.2)
            .map(|p| (p.x - seat.point.x).abs())
            .fold(0.12, f32::max);
        // Use the same authenticated torso/shell as the carrying surface.
        // Heads, fins, legs and spikes must not set immersion depth. Keep the
        // upper back and its seated rider above the plane on unusual bodies.
        seat.waterline = (low.y + (high.y - low.y) * 0.40)
            .min(seat.point.y - (high.y - low.y) * 0.10).max(0.);
        seat
    }

    pub fn animated_point(&self, rig: &SpeciesRig, matrices: &[Mat4]) -> Vec3 {
        let [a, b, c] = self.vertices.map(|v| rig.skin_point(v, matrices));
        a * self.weights.x + b * self.weights.y + c * self.weights.z
    }
}
