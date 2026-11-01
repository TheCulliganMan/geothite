//! Larvae crawl with a travelling body wave and independent prolegs.
use super::*;

#[derive(Debug, Clone)]
pub(crate) struct CrawlSegment {
    pub joint: usize,
    pub center: Vec3,
    phase: f32,
    tentacle: bool,
}

pub(super) fn build(rig: &mut SpeciesRig) {
    if matches!(rig.species.name(), "omanyte" | "omastar") {
        if matches!(rig.species, Species::Omanyte | Species::Omastar) {
            // The canonical skins already own ten actual root-cap hinges.
            // Reuse those bindings instead of cutting each arm onto a new
            // world-only joint and discarding its authored battle motion.
            rig.crawl_segments = rig.joints.iter().enumerate().filter_map(|(joint, bind)| {
                bind.name.rsplit('/').next().unwrap().strip_prefix("crawl_tentacle_")
                    .map(|_| CrawlSegment {
                        joint,
                        center: bind.inverse_bind.inverse().transform_point3(Vec3::ZERO),
                        phase: joint as f32 * 0.8,
                        tentacle: true,
                    })
            }).collect();
            return;
        }
        for part in rig
            .anatomy
            .iter()
            .filter(|p| p.name.starts_with("Tentacle /"))
        {
            let (lo, hi) = point_bounds(
                rig.neutral.positions[part.vertices.clone()]
                    .iter()
                    .copied()
                    .map(Vec3::from_array),
            );
            // Tentacles sweep along the ground. A virtual vertical human shin
            // bends these curled meshes through the floor.
            let center = Vec3::new((lo.x + hi.x) * 0.25, hi.y, (lo.z + hi.z) * 0.25);
            let joint = rig.joints.len();
            let body = rig.joints[1]
                .inverse_bind
                .inverse()
                .transform_point3(Vec3::ZERO);
            rig.joints.push(Joint {
                name: format!("{}/crawl_tentacle_{joint}", rig.species.name()),
                parent: Some(1),
                bind: Transform::from_translation(center - body),
                inverse_bind: Mat4::from_translation(-center),
            });
            for v in part.vertices.clone() {
                rig.joint_indices[v] = [joint as u16, 0, 0, 0];
                rig.joint_weights[v] = [1., 0., 0., 0.];
            }
            rig.crawl_segments.push(CrawlSegment {
                joint,
                center,
                phase: joint as f32 * 0.8,
                tentacle: true,
            });
        }
        return;
    }
    if !matches!(rig.species.name(), "caterpie" | "weedle") {
        return;
    }
    if matches!(rig.species, Species::Caterpie | Species::Weedle) {
        // Authored continuous skins already blend their body and pigment
        // through these joints. Reuse them without cutting or rebinding the
        // body, so the cream/green material seam remains exactly shared.
        let mut segments: Vec<_> = rig.joints.iter().enumerate().filter_map(|(joint, bind)| {
            bind.name.rsplit('/').next().unwrap().starts_with("segment_").then(|| {
                let center=bind.inverse_bind.inverse().transform_point3(Vec3::ZERO);
                (joint,center)
            })
        }).collect();
        segments.sort_by(|(_,a),(_,b)|b.z.total_cmp(&a.z));
        rig.crawl_segments=segments.into_iter().enumerate().map(|(order,(joint,center))|CrawlSegment {
            joint,center,phase:order as f32*0.75,tentacle:false,
        }).collect();
        return;
    }
    let mut parts: Vec<_> = rig
        .anatomy
        .iter()
        .filter(|p| {
            p.name
                .to_ascii_lowercase()
                .starts_with("articulated body segment")
        })
        .map(|p| {
            let (lo, hi) = point_bounds(
                rig.neutral.positions[p.vertices.clone()]
                    .iter()
                    .copied()
                    .map(Vec3::from_array),
            );
            (p.vertices.clone(), (lo + hi) * 0.5)
        })
        .collect();
    parts.sort_by(|(_, a), (_, b)| b.z.total_cmp(&a.z));
    for (order, (vertices, center)) in parts.into_iter().enumerate() {
        let joint = rig.joints.len();
        rig.joints.push(Joint {
            name: format!("{}/crawl_segment_{order}", rig.species.name()),
            parent: Some(0),
            bind: Transform::from_translation(center),
            inverse_bind: Mat4::from_translation(-center),
        });
        for v in vertices {
            rig.joint_indices[v] = [joint as u16, 0, 0, 0];
            rig.joint_weights[v] = [1., 0., 0., 0.];
        }
        rig.crawl_segments.push(CrawlSegment {
            joint,
            center,
            phase: order as f32 * 0.75,
            tentacle: false,
        });
    }
    if rig.crawl_segments.is_empty() {
        return;
    }
    // The complete head, eyes and antennae remain one assembly at the neck.
    let head = 2;
    let pivot = rig.joints[head]
        .inverse_bind
        .inverse()
        .transform_point3(Vec3::ZERO);
    rig.joints[head].parent = Some(0);
    rig.joints[head].bind.translation = pivot;
    // Pigment and the terminal sting follow their actual body segment.
    for part in &rig.anatomy {
        let category = part.name.to_ascii_lowercase();
        if !category.starts_with("yellow side ring") && !category.starts_with("tail sting") {
            continue;
        }
        let (lo, hi) = point_bounds(
            rig.neutral.positions[part.vertices.clone()]
                .iter()
                .copied()
                .map(Vec3::from_array),
        );
        let z = (lo.z + hi.z) * 0.5;
        let owner = rig
            .crawl_segments
            .iter()
            .min_by(|a, b| (a.center.z - z).abs().total_cmp(&(b.center.z - z).abs()))
            .unwrap()
            .joint;
        for v in part.vertices.clone() {
            rig.joint_indices[v] = [owner as u16, 0, 0, 0];
            rig.joint_weights[v] = [1., 0., 0., 0.];
        }
    }
}

pub(super) fn sample(rig: &SpeciesRig, phase: f32, weight: f32, pose: &mut [Transform]) {
    let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
    for segment in &rig.crawl_segments {
        let wave = phase - segment.phase;
        if segment.tentacle {
            let stroke = Quat::from_rotation_y(wave.sin() * 0.08 * weight);
            pose[segment.joint].rotation = if matches!(rig.species, Species::Omanyte | Species::Omastar) {
                (pose[segment.joint].rotation * stroke).normalize()
            } else { stroke };
            continue;
        }
        // Small compression and arching preserve the overlapping segment
        // connections. Proleg IK plants supports underneath this body wave.
        // The continuous authored skins have tiny prolegs: a large arch
        // exhausts their IK reach and pulls resting pads above the ground.
        let arch=if matches!(rig.species,Species::Caterpie | Species::Weedle) {0.003} else {0.018};
        pose[segment.joint].translation += Vec3::new(
            0.,
            height * arch * (1. - wave.cos()),
            height * 0.008 * wave.sin(),
        ) * weight;
    }
    // The authored head is already a child of the front body joint.
    if let Some(front) = rig.crawl_segments.first().filter(|s| !s.tentacle
        && !matches!(rig.species, Species::Caterpie | Species::Weedle)) {
        pose[2].translation += pose[front.joint].translation - front.center;
    }
}
