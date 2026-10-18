//! Anatomical, world-only leg chains and level-foot inverse kinematics.
//! Canonical battle skins and clips are not rewritten.
use super::*;

#[derive(Debug, Clone)]
pub(crate) struct WalkLeg {
    hip: usize,
    knee: usize,
    foot: usize,
    a: Vec3,
    b: Vec3,
    c: Vec3,
    pole: Vec3,
    phase: f32,
}

fn world(rig: &SpeciesRig, pose: &[Transform], joint: usize) -> Mat4 {
    let local = pose[joint].to_matrix();
    rig.joints[joint]
        .parent
        .map_or(local, |p| world(rig, pose, p) * local)
}

fn crawl(name: &str) -> bool {
    matches!(
        name,
        "caterpie"
            | "weedle"
            | "paras"
            | "parasect"
            | "spinarak"
            | "ariados"
            | "shuckle"
            | "kabuto"
            | "krabby"
            | "kingler"
    )
}
fn words(name: &str, keys: &[&str]) -> bool {
    name.split_once(" / ")
        .map_or(name, |(category, _)| category)
        .split(|c: char| !c.is_ascii_alphabetic())
        .any(|w| keys.iter().any(|key| w.eq_ignore_ascii_case(key)))
}

fn support_center(rig: &SpeciesRig) -> Vec3 {
    let mut center = (rig.neutral_bounds.0 + rig.neutral_bounds.1) * 0.5;
    if crawl(rig.species.name()) || quadruped(rig.species.name()) {
        return center;
    }
    let hip_x = |suffix: &str| {
        rig.joints.iter().find(|joint| joint.parent == Some(0) && joint.name.ends_with(suffix))
            .map(|joint| joint.bind.translation.x)
    };
    let authored = (!matches!(rig.species, Species::Procedural(_)))
        .then(|| hip_x("/leg_left").zip(hip_x("/leg_right")))
        .flatten();
    let (left, right) = authored.unwrap_or_else(|| {
        // Earlier skins, including Pikachu, keep their haunches on the
        // presentation root. Complete named legs still supply bilateral
        // evidence; tiny toe marks and pads must not choose the axis.
        let mut left = f32::NEG_INFINITY;
        let mut right = f32::INFINITY;
        for part in &rig.anatomy {
            if role(&part.name) != Role::Leg || words(&part.name, &["foot", "feet", "toe", "toes", "claw", "claws", "talon", "paw", "hoof", "proleg", "shoe", "boot", "sole", "heel"]) {
                continue;
            }
            let (lo, hi) = point_bounds(rig.neutral.positions[part.vertices.clone()].iter().copied().map(Vec3::from_array));
            let x = (lo.x + hi.x) * 0.5;
            if x < 0. { left = left.max(x); }
            if x > 0. { right = right.min(x); }
        }
        (left, right)
    });
    if left.is_finite() && right.is_finite() {
        // A long one-sided tail or prop can put both real legs on the same
        // side of the silhouette center. Keep existing classification when
        // that center separates the authored hips; otherwise use their axis.
        if left < right && !(left < center.x && center.x < right) {
            center.x = (left + right) * 0.5;
        }
    }
    center
}

/// Complete named limbs are rebound together, including paint, feet and claws.
/// Each independent crawler leg receives its own chain; a row of insect feet
/// must never rotate as one long human shin.
pub(super) fn build(rig: &mut SpeciesRig) {
    let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
    let center = support_center(rig);
    let is_crawl = crawl(rig.species.name());
    let is_quad = quadruped(rig.species.name());
    let mut groups: std::collections::BTreeMap<usize, Vec<(Range<usize>, bool)>> =
        Default::default();
    let mut seeds = Vec::new();
    // An authored leg-root is stronger evidence than the bounding box.
    for part in &rig.anatomy {
        if rig.joints[rig.joint_indices[part.vertices.start][0] as usize]
            .name
            .contains("/crawl_tentacle_")
        {
            continue;
        }
        let (lo, hi) = point_bounds(
            rig.neutral.positions[part.vertices.clone()]
                .iter()
                .copied()
                .map(Vec3::from_array),
        );
        let r = role(&part.name);
        let owner = rig.joint_indices[part.vertices.start][0] as usize;
        let ankle_cuff =
            words(&part.name, &["cuff", "anklet"]) && hi.y < rig.neutral_bounds.0.y + height * 0.40;
        let support = ankle_cuff
            || r == Role::Leg
            || r == Role::Other && role(&rig.joints[owner].name) == Role::Leg
            || (words(&part.name, &["claw", "limb", "tentacle"])
                && !words(&part.name, &["hand", "arm", "pincer"])
                && hi.y < rig.neutral_bounds.0.y + height * 0.35);
        if !support {
            continue;
        }
        let c = (lo + hi) * 0.5;
        let foot = ankle_cuff
            || words(
                &part.name,
                &[
                    "foot", "feet", "toe", "toes", "claw", "claws", "talon", "paw", "hoof",
                    "proleg", "shoe", "boot", "sole", "heel",
                ],
            );
        let right = usize::from(c.x >= center.x);
        let rear =
            is_quad
                && (part
                    .name
                    .split_ascii_whitespace()
                    .any(|w| w.eq_ignore_ascii_case("rear") || w.eq_ignore_ascii_case("hind"))
                    || !part.name.split_ascii_whitespace().any(|w| {
                        w.eq_ignore_ascii_case("front") || w.eq_ignore_ascii_case("fore")
                    }) && c.z < center.z);
        let key = if is_crawl {
            // Preserve each existing articulated leg tree when available.
            let mut owner = rig.joint_indices[part.vertices.start][0] as usize;
            if role(&rig.joints[owner].name) == Role::Leg
                && !matches!(rig.species, Species::Procedural(_))
            {
                while let Some(parent) = rig.joints[owner].parent {
                    if role(&rig.joints[parent].name) != Role::Leg {
                        break;
                    }
                    owner = parent;
                }
                owner
            } else {
                rig.joints.len() + seeds.len()
            }
        } else {
            right + 2 * usize::from(rear)
        };
        seeds.push((c, key, part.vertices.clone(), foot));
    }
    // Toe paint/claws on the fixed root inherit the nearest complete leg.
    // Named leg/shank parts are preferred anchors to a tiny nail or toe tip.
    let anchors: Vec<_> = seeds
        .iter()
        .filter(|(_, _, _, foot)| !*foot)
        .map(|(p, k, _, _)| (*p, *k))
        .collect();
    for (center, key, vertices, foot) in &seeds {
        let key = if *foot && !anchors.is_empty() {
            anchors
                .iter()
                .filter(|(p, _)| p.x.signum() == center.x.signum())
                .min_by(|(a, _), (b, _)| {
                    a.distance_squared(*center)
                        .total_cmp(&b.distance_squared(*center))
                })
                .map_or(*key, |(_, key)| *key)
        } else {
            *key
        };
        groups
            .entry(key)
            .or_default()
            .push((vertices.clone(), *foot));
    }
    // The canonical skins deliberately keep resting supports fixed at the
    // presentation root. Walking IK articulates them without replaying idle
    // torso sway through planted feet.
    let mut legs = Vec::new();
    let mut support_vertices = vec![false; rig.neutral.positions.len()];
    for (order, parts) in groups.values().enumerate() {
        let (lo, hi) = point_bounds(
            parts
                .iter()
                .flat_map(|(range, _)| range.clone())
                .map(|v| Vec3::from_array(rig.neutral.positions[v])),
        );
        let foot_vertices: Vec<_> = parts
            .iter()
            .filter(|(_, foot)| *foot)
            .flat_map(|(range, _)| range.clone())
            .collect();
        let foot_points: Vec<_> = if foot_vertices.is_empty() {
            parts
                .iter()
                .flat_map(|(r, _)| r.clone())
                .filter(|&v| rig.neutral.positions[v][1] < lo.y + (hi.y - lo.y) * 0.18)
                .collect()
        } else {
            foot_vertices.clone()
        };
        if foot_points.is_empty() {
            continue;
        }
        let (flo, fhi) = point_bounds(
            foot_points
                .iter()
                .map(|&v| Vec3::from_array(rig.neutral.positions[v])),
        );
        let ankle = Vec3::new(
            (flo.x + fhi.x) * 0.5,
            (flo.y + fhi.y) * 0.5,
            (flo.z + fhi.z) * 0.5,
        );
        let hip = Vec3::new(
            (lo.x + hi.x) * 0.5,
            hi.y + if hi.y - fhi.y < height * 0.08 {
                height * 0.15
            } else {
                0.
            },
            (lo.z + hi.z) * 0.5,
        );
        let distance = hip.distance(ankle);
        if distance < height * 0.01 {
            continue;
        }
        let side = if ankle.x < center.x { -1. } else { 1. };
        let rear = ankle.z < center.z;
        let pole = if is_crawl {
            Vec3::X * side
        } else {
            Vec3::Z * if is_quad && !rear { -1. } else { 1. }
        };
        let axis = (ankle - hip).normalize();
        let bend = (pole - axis * pole.dot(axis)).normalize_or_zero();
        let knee = (hip + ankle) * 0.5 + bend * distance * 0.20;
        let parent = rig
            .crawl_segments
            .iter()
            .min_by(|a, b| {
                (a.center.z - hip.z)
                    .abs()
                    .total_cmp(&(b.center.z - hip.z).abs())
            })
            .map_or(0, |segment| segment.joint);
        let parent_bind = rig.joints[parent].inverse_bind.inverse();
        let index = rig.joints.len();
        for (name, pivot, ancestor, local) in [
            (
                "gait_hip",
                hip,
                parent,
                parent_bind.inverse().transform_point3(hip),
            ),
            ("gait_knee", knee, index, knee - hip),
            ("gait_foot", ankle, index + 1, ankle - knee),
        ] {
            rig.joints.push(Joint {
                name: format!("{}/{name}_{order}", rig.species.name()),
                parent: Some(ancestor),
                bind: Transform::from_translation(local),
                inverse_bind: Mat4::from_translation(-pivot),
            });
        }
        for (range, foot) in parts {
            for v in range.clone() {
                support_vertices[v] = true;
                let y = rig.neutral.positions[v][1];
                let (owner, other, w) = if *foot || y <= ankle.y + distance * 0.07 {
                    (index + 2, index + 2, 1.)
                } else {
                    let t = ((hip.y - y) / (hip.y - ankle.y).max(0.0001)).clamp(0., 1.);
                    if t < 0.5 {
                        (index, index + 1, (0.5 - t) * 2.)
                    } else {
                        (index + 1, index + 2, (1. - t) * 2.)
                    }
                };
                rig.joint_indices[v] = [owner as u16, other as u16, 0, 0];
                rig.joint_weights[v] = [w, 1. - w, 0., 0.];
            }
        }
        let phase = if !rig.crawl_segments.is_empty() {
            // Bilateral prolegs on one abdominal segment move together;
            // successive pairs follow the wave down the larva's body.
            -(rig
                .crawl_segments
                .iter()
                .position(|segment| segment.joint == parent)
                .unwrap() as f32)
                // The authored larvae have three support rows. A 0.12-cycle
                // offset can put every pair in the 0.40-cycle swing window;
                // quarter-cycle staggering keeps a support pair planted.
                * if matches!(rig.species,Species::Caterpie | Species::Weedle) {0.25} else {0.12}
        } else if is_crawl {
            order as f32 * 0.27
        } else {
            if (side > 0.) ^ rear && is_quad || side > 0. && !is_quad {
                0.5
            } else {
                0.
            }
        };
        legs.push(WalkLeg {
            hip: index,
            knee: index + 1,
            foot: index + 2,
            a: hip,
            b: knee,
            c: ankle,
            pole: bend,
            phase,
        });
    }
    rig.walk_cycle = legs
        .iter()
        .map(|leg| leg.a.distance(leg.b) + leg.b.distance(leg.c))
        .fold(f32::INFINITY, f32::min)
        * 0.75;
    if !rig.walk_cycle.is_finite() {
        rig.walk_cycle = height * 0.5;
    }
    rig.walk_legs = legs;
    rig.ground_vertices = (0..rig.neutral.positions.len())
        .filter(|&v| {
            support_vertices[v]
                || rig.neutral.positions[v][1] <= rig.neutral_bounds.0.y + height * 0.40
        })
        .collect();
}

fn step(phase: f32, cycle: f32) -> (f32, f32) {
    let stance = 0.60;
    let p = phase.rem_euclid(1.);
    let span = cycle * stance;
    if p < stance {
        (span * 0.5 - cycle * p, 0.)
    } else {
        let t = (p - stance) / (1. - stance);
        let smooth = t * t * (3. - 2. * t);
        let tangent = -cycle * (1. - stance);
        (
            -span * 0.5 + span * smooth + tangent * t * (1. - t) * (1. - 2. * t),
            (std::f32::consts::PI * t).sin().powi(2),
        )
    }
}

#[cfg(test)]
pub(super) fn assert_supports(rig: &SpeciesRig, pose: &[Transform]) {
    for leg in &rig.walk_legs {
        let hip = world(rig, pose, leg.hip).transform_point3(Vec3::ZERO);
        let knee = world(rig, pose, leg.knee).transform_point3(Vec3::ZERO);
        let foot_matrix = world(rig, pose, leg.foot);
        let foot = foot_matrix.transform_point3(Vec3::ZERO);
        assert!(
            (hip.distance(knee) - leg.a.distance(leg.b)).abs() < 0.00001,
            "{} upper leg stretched",
            rig.species.name()
        );
        assert!(
            (knee.distance(foot) - leg.b.distance(leg.c)).abs() < 0.00001,
            "{} lower leg stretched",
            rig.species.name()
        );
        assert!(
            foot_matrix
                .to_scale_rotation_translation()
                .1
                .angle_between(Quat::IDENTITY)
                < 0.001,
            "{} planted foot tipped",
            rig.species.name()
        );
    }
}

#[test]
fn pokemon_motion_stance_matches_travel_and_swing_closes_smoothly() {
    let cycle = 0.4;
    let dt = 0.0001;
    for p in [0.05, 0.2, 0.4, 0.55] {
        let (a, lift) = step(p, cycle);
        let (b, _) = step(p + dt, cycle);
        assert_eq!(lift, 0.);
        assert!(((b - a) / dt + cycle).abs() < 0.001);
    }
    for boundary in [0., 0.6, 1.] {
        let a = step(boundary - dt, cycle);
        let b = step(boundary, cycle);
        let c = step(boundary + dt, cycle);
        assert!((a.0 - b.0).abs() < cycle * dt * 1.1);
        assert!((c.0 - b.0).abs() < cycle * dt * 1.1);
        assert!(a.1.abs().max(c.1.abs()) < 0.00001);
    }
}

pub(super) fn sample(rig: &SpeciesRig, phase: f32, weight: f32, pose: &mut [Transform]) {
    let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
    for leg in &rig.walk_legs {
        let (z, lift) = step(phase / TAU + leg.phase, rig.walk_cycle);
        let target =
            leg.c + Vec3::new(0., lift * height.min(leg.a.distance(leg.c)) * 0.18, z) * weight;
        let parent = rig.joints[leg.hip].parent.unwrap();
        let parent_world = world(rig, pose, parent);
        let hip = parent_world.transform_point3(rig.joints[leg.hip].bind.translation);
        let upper = leg.a.distance(leg.b);
        let lower = leg.b.distance(leg.c);
        let direction = (target - hip).normalize_or_zero();
        let d = target
            .distance(hip)
            .clamp((upper - lower).abs() + 0.00001, (upper + lower) * 0.9999);
        let along = (upper * upper - lower * lower + d * d) / (2. * d);
        let bend = (leg.pole - direction * leg.pole.dot(direction)).normalize_or_zero();
        let knee = hip + direction * along + bend * (upper * upper - along * along).max(0.).sqrt();
        let foot = hip + direction * d;
        let upper_rotation =
            Quat::from_rotation_arc((leg.b - leg.a).normalize(), (knee - hip).normalize());
        let lower_rotation =
            Quat::from_rotation_arc((leg.c - leg.b).normalize(), (foot - knee).normalize());
        let parent_rotation = parent_world.to_scale_rotation_translation().1;
        pose[leg.hip].rotation = (parent_rotation.inverse() * upper_rotation).normalize();
        pose[leg.knee].rotation = (upper_rotation.inverse() * lower_rotation).normalize();
        pose[leg.foot].rotation = lower_rotation.inverse().normalize();
    }
    // Only the pose moves. Neither root travel nor the authoritative foot changes.
    // Cache support vertices; no geometry buffers are rebuilt during playback.
    if rig.joints.len() <= 256 && !rig.ground_vertices.is_empty() {
        let mut matrices = [Mat4::IDENTITY; 256];
        rig.skin_matrices_into(pose, &mut matrices[..rig.joints.len()])
            .unwrap();
        let floor = rig
            .ground_vertices
            .iter()
            .map(|&v| rig.skin_point(v, &matrices).y)
            .fold(f32::INFINITY, f32::min);
        if floor < 0. {
            pose[0].translation.y -= floor;
        }
    }
}

#[cfg(test)]
mod support_center_tests {
    use super::*;

    #[test]
    fn raichu_skin_support_center_uses_authored_hips_past_the_long_tail() {
        let rig = crate::species_rig::for_species("RAICHU").unwrap();
        let bounds = (rig.neutral_bounds.0 + rig.neutral_bounds.1) * 0.5;
        let hips: Vec<_> = rig.joints.iter().filter(|j| j.parent == Some(0) && (j.name.ends_with("/leg_left") || j.name.ends_with("/leg_right"))).map(|j| j.bind.translation.x).collect();
        assert_eq!(hips.len(), 2);
        assert!(hips.iter().all(|x| *x < bounds.x), "tail must expose the former one-leg classification");
        let center = support_center(rig);
        assert_eq!(center.x.to_bits(), ((hips[0] + hips[1]) * 0.5).to_bits());
        assert_eq!(center.y.to_bits(), bounds.y.to_bits());
        assert_eq!(center.z.to_bits(), bounds.z.to_bits());
    }

    #[test]
    fn raichu_skin_support_center_preserves_a_separating_silhouette_axis() {
        let mut rig = crate::species_rig::for_species("RAICHU").unwrap().clone();
        rig.neutral_bounds.1.x = -rig.neutral_bounds.0.x + 0.02;
        let bounds = (rig.neutral_bounds.0 + rig.neutral_bounds.1) * 0.5;
        assert_eq!(support_center(&rig).to_array().map(f32::to_bits), bounds.to_array().map(f32::to_bits));
        for name in ["BELLSPROUT", "GLOOM", "KABUTOPS", "PINECO"] {
            let rig = crate::species_rig::for_species(name).unwrap();
            let bounds = (rig.neutral_bounds.0 + rig.neutral_bounds.1) * 0.5;
            assert_eq!(support_center(rig).to_array().map(f32::to_bits), bounds.to_array().map(f32::to_bits), "{name} keeps its existing axis");
        }
    }

    #[test]
    fn raichu_skin_support_center_requires_bilateral_leg_evidence() {
        let mut rig = crate::species_rig::for_species("RAICHU").unwrap().clone();
        let right = rig.joints.iter_mut().find(|j| j.name.ends_with("/leg_right")).unwrap();
        right.parent = Some(1);
        rig.anatomy.retain(|part| role(&part.name) != Role::Leg || words(&part.name, &["foot", "feet", "toe", "toes"]));
        let bounds = (rig.neutral_bounds.0 + rig.neutral_bounds.1) * 0.5;
        assert_eq!(support_center(&rig).to_array().map(f32::to_bits), bounds.to_array().map(f32::to_bits));
    }

    #[test]
    fn raichu_skin_support_center_handles_pikachu_without_authored_leg_roots() {
        let rig = crate::species_rig::for_species("PIKACHU").unwrap();
        assert!(!rig.joints.iter().any(|j| j.name.ends_with("/leg_left") || j.name.ends_with("/leg_right")));
        let bounds = (rig.neutral_bounds.0 + rig.neutral_bounds.1) * 0.5;
        let haunches: Vec<_> = rig.anatomy.iter().filter(|p| p.name.starts_with("Haunch /")).map(|part| {
            let (lo, hi) = point_bounds(rig.neutral.positions[part.vertices.clone()].iter().copied().map(Vec3::from_array));
            (lo.x + hi.x) * 0.5
        }).collect();
        assert_eq!(haunches.len(), 2);
        assert!(haunches.iter().all(|x| *x < bounds.x));
        assert_eq!(support_center(rig).x.to_bits(), ((haunches[0] + haunches[1]) * 0.5).to_bits());
    }
}
