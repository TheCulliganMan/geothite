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
    fn jelly_surf_uses_the_bell_and_submerges_tendrils_instead_of_seating_on_pinned_tips() {
        for name in ["TENTACOOL", "TENTACRUEL"] {
            let rig = species_rig::motion::world_rig(species_rig::for_species(name).unwrap());
            let seat = SurfSeat::for_rig(rig);
            let bell = rig.anatomy.iter().find(|part| part.name.starts_with("Bell /")).unwrap();
            assert!(seat.vertices.iter().all(|v| bell.vertices.contains(v)),
                "{name} rider is not carried by the actual bell");
            let low = bell.vertices.clone().map(|v|rig.neutral.positions[v][1])
                .fold(f32::INFINITY,f32::min);
            let high = bell.vertices.clone().map(|v|rig.neutral.positions[v][1])
                .fold(f32::NEG_INFINITY,f32::max);
            assert!(seat.point.y > low+(high-low)*0.65, "{name} seat is below the upper bell");
            assert!(seat.waterline > low && seat.waterline < seat.point.y,
                "{name} must immerse the lower bell, leaving its carrying surface above water");
            let tendrils: Vec<_> = rig.anatomy.iter().filter(|part|
                part.name.starts_with("Tentacle /")).collect();
            assert!(!tendrils.is_empty());
            let mut pose = vec![Transform::IDENTITY;rig.joints.len()];
            let mut matrices = vec![Mat4::IDENTITY;rig.joints.len()];
            for tick in 0..64 {
                rig.sample_swimming(tick as f32/32.,tick as f32*std::f32::consts::TAU/64.,1.,&mut pose);
                rig.skin_matrices_into(&pose,&mut matrices).unwrap();
                for part in &tendrils {
                    let tip = part.vertices.clone().map(|v|rig.skin_point(v,&matrices).y)
                        .fold(f32::INFINITY,f32::min);
                    assert!(tip < seat.waterline, "{name} {} left the water at phase {tick}",part.name);
                }
                assert!(seat.animated_point(rig,&matrices).y > seat.waterline,
                    "{name} animated carrying surface sank");
            }
        }
    }

    #[test]
    fn octillery_is_carried_by_the_upper_mantle_instead_of_rooted_arm_bases() {
        let rig = species_rig::motion::world_rig(species_rig::rig(Species::Octillery));
        let seat = SurfSeat::for_rig(rig);
        let body = rig.anatomy.iter().find(|part|part.name.starts_with("Body /")).unwrap();
        let low = body.vertices.clone().map(|v|rig.neutral.positions[v][1])
            .fold(f32::INFINITY,f32::min);
        let high = body.vertices.clone().map(|v|rig.neutral.positions[v][1])
            .fold(f32::NEG_INFINITY,f32::max);
        assert!(seat.point.y > low+(high-low)*0.65,
            "Octillery rider must sit on the upper mantle: {:?}",seat.point);
        assert!(seat.vertices.iter().all(|v|body.vertices.contains(v)));
        let root = body.vertices.clone().filter(|&v|rig.joint_indices[v].iter()
            .zip(rig.joint_weights[v]).any(|(&joint,w)|joint==0 && w>=0.7))
            .map(|v|rig.neutral.positions[v][1]).fold(f32::NEG_INFINITY,f32::max);
        assert!(seat.waterline > root && seat.waterline < seat.point.y,
            "arm bases must be submerged while the upper mantle carries the rider");
    }

    #[test]
    fn star_surf_carries_above_the_continuous_body_instead_of_inside_its_lower_points() {
        for name in ["STARYU", "STARMIE"] {
            let rig = species_rig::motion::world_rig(species_rig::for_species(name).unwrap());
            let seat = SurfSeat::for_rig(rig);
            let body:Vec<_> = rig.anatomy.iter().filter(|part|matches!(
                part.name.split(" / ").next(),Some("Body"|"Front star"|"Back star"|"Core")))
                .flat_map(|part|part.vertices.clone()).collect();
            assert!(seat.vertices.iter().all(|v|body.contains(v)),"{name} seat left its body");
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];
            let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];
            for tick in 0..64 {
                rig.sample_swimming(tick as f32/32.,tick as f32*std::f32::consts::TAU/64.,1.,&mut pose);
                rig.skin_matrices_into(&pose,&mut matrices).unwrap();
                let (low,high) = body.iter().map(|&v|rig.skin_point(v,&matrices)).fold(
                    (Vec3::splat(f32::INFINITY),Vec3::splat(f32::NEG_INFINITY)),
                    |(lo,hi),p|(lo.min(p),hi.max(p)));
                let span=high-low;
                assert!(span.x.max(span.z)>span.y*2.,"{name} must swim face-up");
                let contact=seat.animated_point(rig,&matrices);
                assert!(contact.y>seat.waterline,"{name} rider contact submerged");
                assert!(low.y<seat.waterline,"{name} body floats clear of water");
            }
        }
    }

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
        // A jelly's bell is its buoyant body. Its root-bound distal tendrils
        // must never become the fallback carrying surface or water datum.
        let jelly = matches!(rig.species.name(), "tentacool" | "tentacruel");
        let star = matches!(rig.species.name(), "staryu" | "starmie");
        // Head, ears, tails and shell spikes are not seats. Joint ownership
        // also handles continuous sculptures with no separate torso primitive.
        let body_weight = |v: usize| {
            // The continuous star is the body, including its articulating
            // points. Restrict by the authored body parts below, not by their
            // root-pinned distal tips or the separate jewel assembly.
            if star { return 1.; }
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
                        || (jelly && rig.joints[**j as usize].name.ends_with("/bell"))
                        || (rig.species.name() == "octillery"
                            && rig.joints[**j as usize].name.ends_with("/mantle"))
                })
                .map(|(_, weight)| weight)
                .sum::<f32>()
        };
        let body_part = |name: &str| {
            let category = name.split(" / ").next();
            matches!(category, Some("Body" | "Shell" | "Torso"))
                || (jelly && category == Some("Bell"))
                || (star && matches!(category,Some("Front star"|"Back star")))
        };
        let has_body_parts = rig.anatomy.iter().any(|part| body_part(&part.name));
        let mut carrying_surface = vec![!has_body_parts; rig.neutral.positions.len()];
        for part in &rig.anatomy {
            if body_part(&part.name) {
                carrying_surface[part.vertices.clone()].fill(true);
            }
        }
        // Water stars change orientation, so both the support triangle and
        // immersion datum must be fitted in that same water pose. Retain the
        // original vertex IDs/barycentric weights for animated rider contact.
        let water_points;
        let points = if star {
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];
            let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];
            rig.sample_swimming(0.,0.,0.,&mut pose);
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            water_points=(0..rig.neutral.positions.len()).map(|v|
                rig.skin_point(v,&matrices).to_array()).collect::<Vec<_>>();
            &water_points
        } else { &rig.neutral.positions };
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
        let target = Vec2::new((low.x + high.x) * 0.5,
            if star { (low.z+high.z)*0.5 } else { low.z + (high.z - low.z) * 0.28 });
        let mut best: Option<Self> = None;
        for part in &rig.anatomy {
            // A water star carries at its central hard core. The raised gem
            // sets rider contact, while the star body still sets buoyancy.
            if has_body_parts && !body_part(&part.name)
                && !(star && part.name.starts_with("Core /")) {
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
