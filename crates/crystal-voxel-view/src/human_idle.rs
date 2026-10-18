//! Quiet standing poses, independent of the supplied crouched combat Idle.
//! Only the weighted skin moves; actor position and facing remain authoritative.
use crate::{
    human_locomotion::{HumanLocomotion, REFERENCE_HEIGHT, rotate_world, world},
    new_bark_actors::character_meshes::CharacterKind,
    open_people::PersonRig,
};
use bevy::prelude::*;
use std::f32::consts::TAU;

#[derive(Clone, Copy)]
struct Manner {
    period: f32,
    sway: f32,
    lean: f32,
    glance: f32,
    arm_space: f32,
    elbow: [f32; 2],
    stagger: f32,
}

fn manner(kind: CharacterKind) -> Manner {
    use CharacterKind::*;
    let mut m = Manner {
        period: 5.2,
        sway: 0.009,
        lean: 0.015,
        glance: 0.12,
        arm_space: 0.10,
        elbow: [0.13, 0.18],
        stagger: 0.018,
    };
    match kind {
        Youngster | StandingYoungster | BugCatcher | GameboyKid | Twin | Bugsy => {
            m.period = 3.7;
            m.sway = 0.017;
            m.glance = 0.22;
            m.elbow = [0.20, 0.11];
            m.stagger = 0.035;
        }
        Mom | RedsMom | Daisy | Teacher | PokefanF => {
            m.period = 5.8;
            m.lean = -0.018;
            m.glance = 0.15;
            m.elbow = [0.15, 0.26];
        }
        Nurse | Receptionist | LinkReceptionist | OldLinkReceptionist | Clerk | Officer => {
            m.period = 6.0;
            m.sway = 0.005;
            m.lean = 0.0;
            m.glance = 0.08;
            m.elbow = [0.16, 0.16];
            m.stagger = 0.0;
        }
        Elm | Oak | Scientist | Bill | SuperNerd | Pharmacist => {
            m.period = 5.5;
            m.lean = -0.025;
            m.glance = 0.19;
            m.elbow = [0.12, 0.34];
        }
        Elder | Gramps | Granny | Pryce | Kurt | KurtOutside | Sage | Blaine | Gentleman => {
            m.period = 6.6;
            m.sway = 0.006;
            m.lean = 0.025;
            m.glance = 0.09;
            m.elbow = [0.20, 0.24];
            m.stagger = 0.01;
        }
        Rival | Blue | Cal | CooltrainerM | CooltrainerF | Karen | Clair | Rocker => {
            m.period = 4.8;
            m.sway = 0.012;
            m.lean = 0.035;
            m.elbow = [0.24, 0.10];
            m.stagger = 0.045;
        }
        Biker | BlackBelt | Bruno | Chuck | Brock | Surge | Sailor | Captain | PokefanM
        | Outdoorsman => {
            m.period = 5.7;
            m.sway = 0.007;
            m.arm_space = 0.19;
            m.elbow = [0.13, 0.16];
            m.stagger = 0.025;
        }
        Beauty | Lass | KimonoGirl | Erika | Jasmine | Sabrina | Will | Morty => {
            m.period = 6.1;
            m.sway = 0.006;
            m.lean = -0.018;
            m.elbow = [0.14, 0.20];
            m.stagger = 0.012;
        }
        Trainer | TrainerFemale | Red | Fisher | FishingGuru | Falkner | Janine | Koga | Lance
        | Misty | GymGuide | Rocket | RocketGirl | SwimmerGirl | SwimmerGuy | UnusedGuy
        | Whitney => {}
    }
    m
}

pub(crate) struct HumanIdle {
    rest: Vec<Transform>,
    feet: [(Vec3, Quat); 2],
    arms: [[usize; 3]; 2],
    hips: Vec3,
    manner: Manner,
    phase: f32,
}

impl HumanIdle {
    pub(crate) fn new(rig: &PersonRig, kind: CharacterKind) -> Self {
        let rest = rig.nodes.clone();
        let hips = world(rig, &rest, rig.skin_nodes[rig.hips_joint]).transform_point3(Vec3::ZERO);
        let units = rig.height / REFERENCE_HEIGHT;
        let manner = manner(kind);
        let feet = std::array::from_fn(|side| {
            let shin = rig.skin_nodes[rig.leg_joints[side][1]];
            let foot = *rig
                .skin_nodes
                .iter()
                .find(|&&n| rig.parents[n] == Some(shin))
                .expect("human shin requires a foot");
            let (_, rotation, mut point) = world(rig, &rest, foot).to_scale_rotation_translation();
            let sign = if side == 0 { 1. } else { -1. };
            // Hip-width support, with a small character-specific stagger.
            // Preserve bind ankle height/orientation so the soles stay level.
            point.x = hips.x + sign * 0.085 * units;
            point.z = hips.z + sign * manner.stagger * units;
            (point, rotation)
        });
        let arms = std::array::from_fn(|side| {
            let hand = &rig.hand_joints[side];
            let wrist = hand
                .iter()
                .map(|&j| rig.skin_nodes[j])
                .find(|&n| {
                    rig.parents[n].is_none_or(|p| !hand.iter().any(|&j| rig.skin_nodes[j] == p))
                })
                .expect("human requires a wrist");
            let lower = rig.parents[wrist].expect("human requires a forearm");
            [
                rig.parents[lower].expect("human requires an upper arm"),
                lower,
                wrist,
            ]
        });
        // Stable source identity, not process-random hashing. Instance clocks
        // add their own offset in the world renderer, so crowds do not breathe
        // or look around in unison, including actors sharing a body rig.
        let key = crate::npc_appearance::look(kind).texture_key;
        let seed = key
            .bytes()
            .fold(0u32, |h, b| h.wrapping_mul(31).wrapping_add(b as u32));
        Self {
            rest,
            feet,
            arms,
            hips,
            manner,
            phase: (seed % 997) as f32 / 997. * TAU,
        }
    }

    pub(crate) fn sample(
        &self,
        rig: &PersonRig,
        gait: &HumanLocomotion,
        seconds: f32,
        pose: &mut [Transform],
    ) {
        pose.copy_from_slice(&self.rest);
        let m = self.manner;
        // Separate slow clocks give breathing, weight transfer and occasional
        // glances their own rhythm. No large bob, knee pumping or root travel.
        let breath = (seconds.rem_euclid(m.period) / m.period * TAU + self.phase).sin();
        let shift = (seconds.rem_euclid(13.0) / 13.0 * TAU + self.phase).sin();
        let glance_phase = seconds.rem_euclid(17.0) / 17.0 * TAU + self.phase;
        let glance = glance_phase.sin().powi(5) * m.glance;
        let units = rig.height / REFERENCE_HEIGHT;
        let hips = rig.skin_nodes[rig.hips_joint];
        let parent = rig.parents[hips]
            .map(|p| world(rig, pose, p))
            .unwrap_or(Mat4::IDENTITY);
        let offset = Vec3::new(
            (m.lean + m.sway * shift) * units,
            (-0.008 + 0.0015 * breath) * units,
            0.,
        );
        pose[hips].translation = parent.inverse().transform_point3(self.hips + offset);
        let chest = rig.skin_nodes[rig.chest_joint];
        rotate_world(
            rig,
            pose,
            chest,
            Quat::from_euler(
                EulerRot::YXZ,
                glance * 0.12,
                breath * 0.006,
                -offset.x / units * 0.28,
            ),
        );
        rotate_world(
            rig,
            pose,
            rig.skin_nodes[rig.head_joint],
            Quat::from_euler(
                EulerRot::YXZ,
                glance,
                -0.008 + breath * 0.004,
                m.lean * 0.30,
            ),
        );
        for (side, &[upper, lower, wrist]) in self.arms.iter().enumerate() {
            let sign = if side == 0 { 1. } else { -1. };
            let drift = breath * 0.009 + shift * 0.008;
            // Elbows hang below the shoulders and open hands rest beside the
            // thighs. Slight asymmetry conveys personality without a guard.
            aim(
                rig,
                pose,
                upper,
                lower,
                Vec3::new(sign * m.arm_space, -1., -0.025 + drift),
            );
            aim(
                rig,
                pose,
                lower,
                wrist,
                Vec3::new(sign * 0.018, -1., m.elbow[side] + drift),
            );
        }
        gait.plant_idle(rig, &self.feet, pose);
    }
}

fn aim(rig: &PersonRig, pose: &mut [Transform], joint: usize, child: usize, direction: Vec3) {
    let origin = world(rig, pose, joint).transform_point3(Vec3::ZERO);
    let current = world(rig, pose, child).transform_point3(Vec3::ZERO) - origin;
    rotate_world(
        rig,
        pose,
        joint,
        Quat::from_rotation_arc(current.normalize(), direction.normalize()),
    );
}

#[cfg(any(test, feature = "people-review"))]
pub(crate) mod validation {
    use super::*;

    pub(crate) fn validate() {
        relaxed_idle_keeps_every_identity_upright_grounded_and_continuous();
        relaxed_idle_skin_and_walk_transitions_stay_above_floor_and_inside_bounds();
    }

    #[cfg_attr(test, test)]
    #[cfg_attr(test, ignore = "requires supplied human rigs")]
    fn relaxed_idle_keeps_every_identity_upright_grounded_and_continuous() {
        use crate::new_bark_actors::character_meshes::SOURCE_KINDS;
        let mut signatures = std::collections::HashSet::new();
        for kind in SOURCE_KINDS
            .iter()
            .map(|&(_, k)| k)
            .chain([CharacterKind::Outdoorsman])
        {
            let rig = crate::open_people::get(crate::npc_appearance::look(kind).base).unwrap();
            let gait = HumanLocomotion::new(rig);
            let idle = HumanIdle::new(rig, kind);
            let mut pose = rig.nodes.clone();
            let mut next = pose.clone();
            let mut start = pose.clone();
            idle.sample(rig, &gait, 0., &mut start);
            let units = rig.height / REFERENCE_HEIGHT;
            // Different named characters must not receive an identical pose,
            // even when they share one rig and one general manner.
            signatures.insert(
                start
                    .iter()
                    .flat_map(|t| t.rotation.to_array().map(f32::to_bits))
                    .collect::<Vec<_>>(),
            );
            for frame in 0..96 {
                let seconds = frame as f32 * 0.43;
                idle.sample(rig, &gait, seconds, &mut pose);
                idle.sample(rig, &gait, seconds + 0.001, &mut next);
                let point = |node| world(rig, &pose, node).transform_point3(Vec3::ZERO);
                for ((t, b), n) in pose.iter().zip(&rig.nodes).zip(&next) {
                    assert!(t.translation.is_finite() && t.rotation.is_finite());
                    assert!((t.rotation.length_squared() - 1.).abs() < 1e-4);
                    assert_eq!(t.scale, b.scale, "idle must not stretch bones");
                    assert!(
                        t.rotation.angle_between(n.rotation) < 0.003,
                        "{kind:?}: idle velocity pop"
                    );
                    assert!(t.translation.distance(n.translation) < units * 0.0001);
                }
                for &[upper, lower, wrist] in &idle.arms {
                    assert!(point(upper).y > point(lower).y + 0.12 * units);
                    assert!(point(lower).y > point(wrist).y + 0.12 * units);
                    assert!(
                        point(wrist).y < point(rig.skin_nodes[rig.hips_joint]).y + 0.08 * units,
                        "{kind:?}: hands raised into a guard"
                    );
                }
                for (side, leg) in rig.leg_joints.iter().enumerate() {
                    let upper = rig.skin_nodes[leg[0]];
                    let lower = rig.skin_nodes[leg[1]];
                    let foot = *rig
                        .skin_nodes
                        .iter()
                        .find(|&&n| rig.parents[n] == Some(lower))
                        .unwrap();
                    assert!(
                        point(foot).distance(idle.feet[side].0) < 0.002,
                        "{kind:?}: planted foot moved at {seconds}: {:?} vs {:?}",
                        point(foot),
                        idle.feet[side].0
                    );
                    let length =
                        point(upper).distance(point(lower)) + point(lower).distance(point(foot));
                    assert!(
                        (point(upper).y - point(foot).y) / length > 0.95,
                        "{kind:?}: idle still crouches"
                    );
                }
            }
            assert!(
                start
                    .iter()
                    .zip(&pose)
                    .any(|(a, b)| a.rotation.angle_between(b.rotation) > 0.01),
                "{kind:?}: idle is frozen"
            );
            // Sampling is stateless: revisiting a time is bit exact.
            idle.sample(rig, &gait, 0., &mut pose);
            assert_eq!(start, pose);
            // Every independent clock wraps without a seam.
            for period in [idle.manner.period, 13., 17.] {
                idle.sample(rig, &gait, period - 0.001, &mut pose);
                idle.sample(rig, &gait, period + 0.001, &mut next);
                for (a, b) in pose.iter().zip(&next) {
                    assert!(
                        a.rotation.angle_between(b.rotation) < 0.003,
                        "{kind:?}: clock seam"
                    );
                }
            }
        }
        assert_eq!(
            signatures.len(),
            75,
            "each character needs an individual idle"
        );
    }

    #[cfg_attr(test, test)]
    #[cfg_attr(test, ignore = "requires supplied human rigs")]
    fn relaxed_idle_skin_and_walk_transitions_stay_above_floor_and_inside_bounds() {
        for kind in [
            CharacterKind::Trainer,
            CharacterKind::TrainerFemale,
            CharacterKind::Mom,
            CharacterKind::Youngster,
            CharacterKind::Officer,
            CharacterKind::Elm,
            CharacterKind::Granny,
            CharacterKind::Brock,
            CharacterKind::KimonoGirl,
            CharacterKind::Blue,
        ] {
            let rig = crate::open_people::get(crate::npc_appearance::look(kind).base).unwrap();
            let gait = HumanLocomotion::new(rig);
            let idle = HumanIdle::new(rig, kind);
            let bounds = rig.locomotion_bounds();
            let mut pose = rig.nodes.clone();
            let mut standing = pose.clone();
            let mut moving = pose.clone();
            for frame in 0..24 {
                let seconds = frame as f32 * 0.83;
                idle.sample(rig, &gait, seconds, &mut standing);
                crate::hand_pose::relax(rig, &mut standing);
                for run in [0., 1.] {
                    gait.sample(rig, frame as f32 / 24. * TAU, run, &mut moving);
                    crate::hand_pose::relax(rig, &mut moving);
                    for weight in [0., 0.15, 0.5, 0.85, 1.] {
                        for ((p, a), b) in pose.iter_mut().zip(&standing).zip(&moving) {
                            *p = Transform {
                                translation: a.translation.lerp(b.translation, weight),
                                rotation: a.rotation.slerp(b.rotation, weight).normalize(),
                                scale: a.scale.lerp(b.scale, weight),
                            };
                        }
                        if weight > 0. {
                            gait.plant_blend(rig, &standing, &moving, weight, &mut pose);
                        }
                        let inverse = world(rig, &pose, rig.mesh_node).inverse();
                        let palette = rig
                            .skin_nodes
                            .iter()
                            .zip(&rig.inverse_binds)
                            .map(|(&n, bind)| inverse * world(rig, &pose, n) * *bind)
                            .collect::<Vec<_>>();
                        let mut floor = f32::INFINITY;
                        for ((p, ids), weights) in
                            rig.mesh.positions.iter().zip(&rig.ids).zip(&rig.weights)
                        {
                            let p = Vec3::from_array(*p);
                            let v = (0..4)
                                .map(|i| palette[ids[i] as usize].transform_point3(p) * weights[i])
                                .sum::<Vec3>();
                            assert!(
                                v.cmpge(bounds.0 - Vec3::splat(0.01)).all()
                                    && v.cmple(bounds.1 + Vec3::splat(0.01)).all(),
                                "{kind:?}: idle/step escapes culling"
                            );
                            floor = floor.min(v.y);
                        }
                        assert!(
                            floor >= rig.floor - 0.01,
                            "{kind:?}: sole below floor at {seconds}, weight={weight}, run={run}: {floor} vs {}",
                            rig.floor
                        );
                        if weight == 0. {
                            assert!(
                                floor < rig.floor + 0.02,
                                "{kind:?}: idle floats above floor"
                            );
                        }
                    }
                }
            }
        }
    }
}
