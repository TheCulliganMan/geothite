//! Distance-driven human steps. Only the skin moves; the controller owns travel.
use crate::{open_people::PersonRig, new_bark_actors::character_meshes::CharacterKind};
use bevy::prelude::*;
use std::f32::consts::{PI, TAU};

pub(crate) const WALK_DISTANCE: f32 = 0.72;
pub(crate) const RUN_DISTANCE: f32 = 2.00;
pub(crate) const REFERENCE_HEIGHT: f32 = 1.38;

pub(crate) fn cycle_distance(run: f32) -> f32 {
    WALK_DISTANCE + (RUN_DISTANCE - WALK_DISTANCE) * run
}

pub(super) fn world(rig: &PersonRig, pose: &[Transform], node: usize) -> Mat4 {
    rig.parents[node]
        .map(|p| world(rig, pose, p))
        .unwrap_or(Mat4::IDENTITY)
        * pose[node].to_matrix()
}

pub(super) fn rotate_world(rig: &PersonRig, pose: &mut [Transform], node: usize, delta: Quat) {
    let parent = rig.parents[node]
        .map(|p| world(rig, pose, p).to_scale_rotation_translation().1)
        .unwrap_or(Quat::IDENTITY);
    pose[node].rotation = (parent.inverse() * delta * parent * pose[node].rotation).normalize();
}

#[derive(Clone, Copy)]
struct Leg {
    upper: usize,
    lower: usize,
    foot: usize,
    ankle: Vec3,
    rotation: Quat,
    lengths: Vec2,
    hip_offset: Vec3,
}

/// Cached neutral pose and anatomy. Sampling changes a fixed number of joints,
/// without mesh deformation on the CPU, allocations, or iterative IK.
pub(crate) struct HumanLocomotion {
    neutral: Vec<Transform>,
    hips: Vec3,
    legs: [Leg; 2],
    arms: [[usize; 3]; 2],
    style: Style,
    hands: crate::hand_pose::Hands,
}

#[derive(Clone, Copy)]
struct Style {
    swing: f32,
    elbow: f32,
    twist: f32,
    lean: f32,
}

impl Style {
    fn for_character(kind: CharacterKind) -> Self {
        use CharacterKind::*;
        let (swing, elbow, twist, lean) = match kind {
            Youngster | StandingYoungster | BugCatcher | GameboyKid | Twin | Bugsy =>
                (1.12, 0.90, 1.10, 0.00),
            Elder | Gramps | Granny | Pryce | Kurt | KurtOutside | Sage | Blaine
                | Gentleman | Pharmacist | FishingGuru => (0.72, 0.80, 0.65, 0.025),
            Mom | RedsMom | Daisy | Teacher | PokefanF | Nurse | Receptionist
                | LinkReceptionist | OldLinkReceptionist | Clerk | Officer
                | Beauty | Lass | KimonoGirl | Erika | Jasmine | Sabrina =>
                (0.76, 0.85, 0.72, -0.008),
            Biker | BlackBelt | Bruno | Chuck | Brock | Surge | Sailor | Captain
                | PokefanM | Outdoorsman | SwimmerGuy | SwimmerGirl =>
                (1.05, 1.10, 1.05, 0.008),
            Rival | Blue | Cal | CooltrainerM | CooltrainerF | Karen | Clair | Rocker
                | Janine | Koga | Lance | Rocket | RocketGirl =>
                (1.08, 1.05, 1.15, 0.012),
            Trainer | TrainerFemale | Red | Fisher | Elm | Oak | Scientist | Bill
                | SuperNerd | Morty | Falkner | Misty | GymGuide | UnusedGuy | Whitney
                | Will => (1., 1., 1., 0.),
        };
        Self { swing, elbow, twist, lean }
    }
}

/// Normalized left/right foot cycle: a level stance followed by a lifted return.
/// The Hermite return matches stance velocity at both joins, avoiding a kick
/// when the foot leaves or reaches the ground. Phase zero is left heel contact.
fn step(phase: f32, run: f32) -> (f32, f32, f32) {
    let stance = 0.54 + (0.22 - 0.54) * run;
    let cycle = cycle_distance(run);
    let span = cycle * stance;
    let p = (phase / TAU).rem_euclid(1.0);
    if p < stance {
        (span * 0.5 - cycle * p, 0., 0.)
    } else {
        let t = (p - stance) / (1. - stance);
        let smooth = t * t * (3. - 2. * t);
        let tangent = -cycle * (1. - stance);
        let z = -span * 0.5 + span * smooth + tangent * t * (1. - t) * (1. - 2. * t);
        let lift = (PI * t).sin().powi(2) * (0.055 + 0.095 * run);
        // A squared envelope also gives zero angular velocity at contact.
        let pitch = 0.32 * (TAU * t).sin() * (PI * t).sin().powi(2);
        (z, lift, pitch)
    }
}

impl HumanLocomotion {
    /// A weighted vertex is inside the union of its contributing joint boxes.
    /// Compute this once on import, including the new poses in GPU culling.
    pub(crate) fn bounds(rig: &PersonRig) -> (Vec3, Vec3) {
        let gait = Self::new(rig);
        let mut boxes = vec![
            (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY));
            rig.skin_nodes.len()
        ];
        for ((p, ids), weights) in rig.mesh.positions.iter().zip(&rig.ids).zip(&rig.weights) {
            let p = Vec3::from_array(*p);
            for i in 0..4 {
                if weights[i] > 0. {
                    let bounds = &mut boxes[ids[i] as usize];
                    bounds.0 = bounds.0.min(p);
                    bounds.1 = bounds.1.max(p);
                }
            }
        }
        let mut bounds = rig.bounds;
        let mut pose = rig.nodes.clone();
        let mut include = |pose: &[Transform]| {
            let inverse = world(rig, pose, rig.mesh_node).inverse();
            for (joint, &(min, max)) in boxes.iter().enumerate() {
                if !min.is_finite() { continue; }
                let matrix = inverse * world(rig, pose, rig.skin_nodes[joint]) * rig.inverse_binds[joint];
                for x in [min.x, max.x] {
                    for y in [min.y, max.y] {
                        for z in [min.z, max.z] {
                            let p = matrix.transform_point3(Vec3::new(x, y, z));
                            bounds.0 = bounds.0.min(p);
                            bounds.1 = bounds.1.max(p);
                        }
                    }
                }
            }
        };
        use CharacterKind::*;
        for kind in [Trainer, TrainerFemale, Youngster, Mom, Officer, Granny, Brock, KimonoGirl, Blue, Elm] {
            let gait = Self::for_character(rig, kind);
            for run in [0., 0.25, 0.5, 0.75, 1.] {
                for frame in 0..64 {
                    gait.sample(rig, frame as f32 / 64. * TAU, run, &mut pose);
                    include(&pose);
                }
            }
        }
        // Cover each manner and its widest motion without rebuilding the same
        // quiet envelope for every identity during first appearance loading.
        for kind in [Trainer, TrainerFemale, Mom, Youngster, Officer, Elm, Granny, Brock, KimonoGirl, Blue] {
            let idle = crate::human_idle::HumanIdle::new(rig, kind);
            for frame in 0..24 {
                idle.sample(rig, &gait, frame as f32 * 1.83, &mut pose);
                include(&pose);
            }
        }
        let margin = Vec3::splat(rig.height * 0.015);
        (bounds.0 - margin, bounds.1 + margin)
    }

    pub(crate) fn new(rig: &PersonRig) -> Self {
        Self::for_character(rig, CharacterKind::Trainer)
    }

    pub(crate) fn for_character(rig: &PersonRig, kind: CharacterKind) -> Self {
        // The supplied Idle is a combat crouch. Build locomotion from the
        // upright bind anatomy and relax the arms with our standing pose.
        let neutral = rig.nodes.clone();
        let point = |node| world(rig, &neutral, node).transform_point3(Vec3::ZERO);
        let hips = point(rig.skin_nodes[rig.hips_joint]);
        let legs = std::array::from_fn(|side| {
            let [upper, lower] = rig.leg_joints[side].map(|j| rig.skin_nodes[j]);
            let foot = *rig
                .skin_nodes
                .iter()
                .find(|&&n| rig.parents[n] == Some(lower))
                .expect("human shin requires a foot");
            Leg {
                upper,
                lower,
                foot,
                // Bind soles are level. Idle can already contain ankle rocking.
                ankle: Vec3::new(
                    hips.x + if side == 0 { 0.085 } else { -0.085 } * rig.height / REFERENCE_HEIGHT,
                    world(rig, &rig.nodes, foot).transform_point3(Vec3::ZERO).y,
                    hips.z,
                ),
                rotation: world(rig, &rig.nodes, foot)
                    .to_scale_rotation_translation()
                    .1,
                lengths: Vec2::new(
                    point(upper).distance(point(lower)),
                    point(lower).distance(point(foot)),
                ),
                hip_offset: point(upper) - hips,
            }
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
        let mut gait = Self {
            hips,
            neutral,
            legs,
            arms,
            style: Style::for_character(kind),
            hands: crate::hand_pose::Hands::new(rig),
        };
        let idle = crate::human_idle::HumanIdle::new(
            rig, kind,
        );
        let mut relaxed = gait.neutral.clone();
        idle.sample(rig, &gait, 0., &mut relaxed);
        gait.neutral = relaxed;
        gait
    }

    pub(crate) fn sample(&self, rig: &PersonRig, phase: f32, run: f32, pose: &mut [Transform]) {
        pose.copy_from_slice(&self.neutral);
        let phase = phase.rem_euclid(TAU);
        let run = run.clamp(0., 1.);
        let units = rig.height / REFERENCE_HEIGHT;
        let hips = rig.skin_nodes[rig.hips_joint];
        let targets = std::array::from_fn::<_, 2, _>(|side| {
            let (z, lift, pitch) = step(phase + side as f32 * PI, run);
            let leg = self.legs[side];
            // Compensate the pitched sole's lower edge, keeping the swing shoe
            // clear of the floor as well as the ankle itself.
            (
                leg.ankle + Vec3::new(0., lift + pitch.sin().abs() * 0.12, z) * units,
                pitch,
            )
        });
        // Keep the pelvis upright with a nearly extended supporting leg.
        // The small two-beat rise accompanies weight transfer, rather than
        // importing the source run's large root bounce into an ordinary walk.
        let bounce = (1. - (2. * phase).cos()) * (0.006 + 0.018 * run) * units;
        let mut desired =
            self.hips + Vec3::new(0.008 * units * phase.sin(), bounce, 0.);
        for (leg, (target, _)) in self.legs.iter().zip(targets) {
            let reach = (leg.lengths.x + leg.lengths.y) * 0.995;
            let horizontal = (target - desired - leg.hip_offset).xz().length_squared();
            let cap = target.y + (reach * reach - horizontal).max(0.).sqrt() - leg.hip_offset.y;
            // Smooth minimum avoids a pelvis velocity kink when support swaps.
            let softness = 0.012 * units;
            desired.y =
                0.5 * (desired.y + cap - ((desired.y - cap).powi(2) + softness * softness).sqrt());
        }
        let parent = rig.parents[hips]
            .map(|p| world(rig, pose, p))
            .unwrap_or(Mat4::IDENTITY);
        pose[hips].translation = parent.inverse().transform_point3(desired);
        let style = self.style;
        let twist = 0.045 * style.twist * phase.sin();
        let roll = (0.016 + 0.010 * run) * style.twist * phase.sin();
        rotate_world(rig, pose, hips, Quat::from_euler(EulerRot::YXZ, twist, 0., roll));
        let chest = rig.skin_nodes[rig.chest_joint];
        let lean = 0.04 + 0.13 * run + style.lean;
        rotate_world(
            rig,
            pose,
            chest,
            Quat::from_euler(EulerRot::YXZ, -twist * 1.6, lean, -roll * 1.6),
        );
        rotate_world(
            rig,
            pose,
            rig.skin_nodes[rig.head_joint],
            Quat::from_euler(EulerRot::YXZ, twist * 0.6,
                -lean + 0.006 * (2. * phase).sin(), roll * 0.6),
        );
        for side in 0..2 {
            let sign = if side == 0 { 1. } else { -1. };
            let [upper, lower, wrist] = self.arms[side];
            let arm_phase = phase + side as f32 * PI;
            // Running's short stance is asymmetric in time. A cosine swings
            // both arms through neutral while the legs are still far apart.
            // Oppose the actual foot trajectory so hands and legs stay in
            // counterbalance at every walk/run blend, including toe-off.
            let swing_span = 0.20 + 0.14 * run;
            let swing = step(arm_phase, run).0 / swing_span;
            let delayed = step(arm_phase - 0.35, run).0 / swing_span;
            rotate_world(
                rig,
                pose,
                upper,
                Quat::from_rotation_x(swing * (0.30 + 0.34 * run) * style.swing),
            );
            // The elbow folds into the forward swing and opens on return.
            // A small phase lag at the wrist carries the hand through reversal
            // instead of moving the entire arm as one rigid pendulum.
            let fold = (0.08 + 0.60 * run
                + (0.10 + 0.18 * run) * (0.5 - 0.5 * swing)) * style.elbow;
            rotate_world(rig, pose, lower, Quat::from_euler(EulerRot::YXZ,
                sign * 0.025 * arm_phase.sin(), -fold, 0.));
            rotate_world(rig, pose, wrist, Quat::from_rotation_x(
                (0.16 + 0.10 * run) * (delayed - swing)));
            let leg = self.legs[side];
            let (target, pitch) = targets[side];
            self.solve_leg(rig, pose, leg, target);
            let current = world(rig, pose, leg.foot).to_scale_rotation_translation().1;
            rotate_world(
                rig,
                pose,
                leg.foot,
                Quat::from_rotation_x(pitch) * leg.rotation * current.inverse(),
            );
        }
        self.hands.sample(0.25 + 0.75 * run, phase.sin(), pose);
    }

    /// Rotation blending alone shortens the leg's world-space reach and can
    /// push shoes under the floor while starting or stopping. Blend ankle
    /// contacts in world space, then solve the already blended skeleton.
    pub(crate) fn plant_blend(
        &self,
        rig: &PersonRig,
        idle: &[Transform],
        moving: &[Transform],
        weight: f32,
        pose: &mut [Transform],
    ) {
        for leg in self.legs {
            let a = world(rig, idle, leg.foot).to_scale_rotation_translation();
            let b = world(rig, moving, leg.foot).to_scale_rotation_translation();
            self.solve_leg(rig, pose, leg, a.2.lerp(b.2, weight));
            let current = world(rig, pose, leg.foot).to_scale_rotation_translation().1;
            rotate_world(
                rig,
                pose,
                leg.foot,
                a.1.slerp(b.1, weight) * current.inverse(),
            );
        }
    }

    /// Idle weight shifts keep both soles fixed, using the same anatomy and
    /// contact solver as the transition into a walk.
    pub(crate) fn plant_idle(&self, rig: &PersonRig, feet: &[(Vec3, Quat); 2], pose: &mut [Transform]) {
        for (leg, &(target, rotation)) in self.legs.iter().zip(feet) {
            self.solve_leg(rig, pose, *leg, target);
            let current = world(rig, pose, leg.foot).to_scale_rotation_translation().1;
            rotate_world(rig, pose, leg.foot, rotation * current.inverse());
        }
    }

    fn solve_leg(&self, rig: &PersonRig, pose: &mut [Transform], leg: Leg, target: Vec3) {
        let origin = world(rig, pose, leg.upper).transform_point3(Vec3::ZERO);
        let delta = target - origin;
        let length = delta.length().clamp(
            (leg.lengths.x - leg.lengths.y).abs() + 1e-5,
            leg.lengths.x + leg.lengths.y - 1e-5,
        );
        let axis = delta.normalize();
        let along =
            (leg.lengths.x.powi(2) - leg.lengths.y.powi(2) + length * length) / (2. * length);
        // Knees always bend toward the character's front, including at the
        // straightest part of stance. No iterative bend-direction flips.
        let pole = (Vec3::Z - axis * axis.z).normalize();
        let knee =
            origin + axis * along + pole * (leg.lengths.x.powi(2) - along * along).max(0.).sqrt();
        let current = world(rig, pose, leg.lower).transform_point3(Vec3::ZERO) - origin;
        rotate_world(
            rig,
            pose,
            leg.upper,
            Quat::from_rotation_arc(current.normalize(), (knee - origin).normalize()),
        );
        let knee = world(rig, pose, leg.lower).transform_point3(Vec3::ZERO);
        let current = world(rig, pose, leg.foot).transform_point3(Vec3::ZERO) - knee;
        rotate_world(
            rig,
            pose,
            leg.lower,
            Quat::from_rotation_arc(current.normalize(), (target - knee).normalize()),
        );
    }
}

#[cfg(any(test, feature = "people-review"))]
pub(crate) mod validation {
    use super::*;

    pub(crate) fn validate() {
        crate::hand_pose::validation::native_fingers_relax_and_close_without_wrist_or_skin_drift();
        crate::limb_surface::validation::supplied_limbs_preserve_landmarks_and_animate_without_seam_cracks();
        human_steps_match_ground_speed_and_join_without_velocity_pops();
        human_locomotion_keeps_contacts_bone_lengths_and_loop_seams_for_every_identity();
        human_gaits_keep_character_and_smooth_arm_followthrough();
        human_locomotion_preserves_shoe_floor_and_existing_skin_envelope();
    }

    #[cfg_attr(test, test)]
    #[cfg_attr(test, ignore = "requires supplied human GLBs")]
    fn human_gaits_keep_character_and_smooth_arm_followthrough() {
        let rig = crate::open_people::get("human-male").unwrap();
        let mut signatures = Vec::new();
        for kind in [CharacterKind::Trainer, CharacterKind::Youngster, CharacterKind::Officer,
            CharacterKind::Granny, CharacterKind::Brock, CharacterKind::Blue] {
            let gait = HumanLocomotion::for_character(rig, kind);
            let mut pose = rig.nodes.clone();
            let mut before = pose.clone();
            let mut after = pose.clone();
            let mut wrist_travel = 0_f32;
            let mut elbow_travel = 0_f32;
            for run in [0., 0.5, 1.] {
                for phase in [0., 0.22 * TAU, 0.54 * TAU, PI, TAU] {
                    gait.sample(rig, phase - 0.0001, run, &mut before);
                    gait.sample(rig, phase, run, &mut pose);
                    gait.sample(rig, phase + 0.0001, run, &mut after);
                    for n in 0..pose.len() {
                        assert!(before[n].rotation.angle_between(after[n].rotation) < 0.002,
                            "{kind:?} run={run} phase={phase} rotation pop at {n}");
                        let a = (pose[n].translation - before[n].translation) / 0.0001;
                        let b = (after[n].translation - pose[n].translation) / 0.0001;
                        assert!(a.distance(b) < 0.04,
                            "{kind:?} run={run} phase={phase} velocity pop at {n}: {a} / {b}");
                    }
                }
                gait.sample(rig, 0., run, &mut before);
                gait.sample(rig, PI, run, &mut after);
                let [upper, lower, wrist] = gait.arms[0];
                wrist_travel = wrist_travel.max(before[wrist].rotation.angle_between(after[wrist].rotation));
                elbow_travel = elbow_travel.max(before[lower].rotation.angle_between(after[lower].rotation));
                for frame in 0..64 {
                    gait.sample(rig, frame as f32 / 64. * TAU, run, &mut pose);
                    for (a, b) in [(upper, lower), (lower, wrist)] {
                        let length = world(rig, &pose, a).transform_point3(Vec3::ZERO)
                            .distance(world(rig, &pose, b).transform_point3(Vec3::ZERO));
                        let bind = world(rig, &rig.nodes, a).transform_point3(Vec3::ZERO)
                            .distance(world(rig, &rig.nodes, b).transform_point3(Vec3::ZERO));
                        assert!((length - bind).abs() < 1e-4, "{kind:?} stretched arm");
                    }
                }
            }
            assert!(wrist_travel > 0.04 && elbow_travel > 0.10,
                "{kind:?} rigid arm: wrist={wrist_travel}, elbow={elbow_travel}");
            gait.sample(rig, 0.7, 0., &mut pose);
            signatures.push(pose[gait.arms[0][0]].rotation);
        }
        for a in 0..signatures.len() {
            for b in a + 1..signatures.len() {
                assert!(signatures[a].angle_between(signatures[b]) > 0.008,
                    "character walks {a}/{b} lost their distinct shoulder poses");
            }
        }
    }

    #[cfg_attr(test, test)]
    fn human_steps_match_ground_speed_and_join_without_velocity_pops() {
        for run in [0., 0.25, 0.5, 0.75, 1.] {
            let stance = 0.54 + (0.22 - 0.54) * run;
            let distance = cycle_distance(run);
            for p in [0.02, 0.1, stance - 0.02] {
                let a = step(p * TAU, run);
                let b = step((p + 0.001) * TAU, run);
                assert!((b.0 - a.0 + distance * 0.001).abs() < 1e-6);
                assert_eq!((a.1, a.2), (0., 0.));
            }
            for join in [0., stance] {
                let h = 0.00001;
                let a = step((join - h) * TAU, run);
                let b = step(join * TAU, run);
                let c = step((join + h) * TAU, run);
                assert!(((b.0 - a.0) - (c.0 - b.0)).abs() < 2e-6);
                assert!((a.1 - c.1).abs() < 1e-6 && (a.2 - c.2).abs() < 1e-6);
            }
            assert!(step((stance + (1. - stance) * 0.5) * TAU, run).1 > 0.05);
        }
    }

    #[cfg_attr(test, test)]
    #[cfg_attr(test, ignore = "requires supplied human GLBs")]
    fn human_locomotion_keeps_contacts_bone_lengths_and_loop_seams_for_every_identity() {
        use crate::new_bark_actors::character_meshes::{CharacterKind, SOURCE_KINDS};
        let kinds = SOURCE_KINDS
            .iter()
            .map(|&(_, k)| k)
            .chain([CharacterKind::Outdoorsman]);
        for kind in kinds {
            let look = crate::npc_appearance::look(kind);
            let rig = crate::open_people::get(look.base).unwrap();
            let gait = HumanLocomotion::for_character(rig, kind);
            let mut pose = rig.nodes.clone();
            for run in [0., 0.5, 1.] {
                gait.sample(rig, 0., run, &mut pose);
                let start = pose.clone();
                gait.sample(rig, TAU, run, &mut pose);
                for (a, b) in start.iter().zip(&pose) {
                    assert!(
                        a.translation.distance(b.translation) < 1e-5
                            && a.rotation.abs_diff_eq(b.rotation, 1e-5)
                    );
                }
                for frame in 0..120 {
                    let phase = frame as f32 / 120. * TAU;
                    gait.sample(rig, phase, run, &mut pose);
                    assert!(pose.iter().all(|t| t.translation.is_finite()
                        && t.rotation.is_finite()
                        && (t.rotation.length_squared() - 1.).abs() < 1e-4));
                    for (side, leg) in gait.legs.iter().enumerate() {
                        let p = |n| world(rig, &pose, n).transform_point3(Vec3::ZERO);
                        if run == 0. && frame == side * 60 {
                            // Heel contact must use a long supporting leg,
                            // rather than preserving the combat Idle squat.
                            let extension = p(leg.upper).distance(p(leg.foot))
                                / (leg.lengths.x + leg.lengths.y);
                            assert!(extension > 0.98,
                                "{kind:?}: crouched walk contact ({extension})");
                        }
                        assert!((p(leg.upper).distance(p(leg.lower)) - leg.lengths.x).abs() < 1e-4);
                        assert!((p(leg.lower).distance(p(leg.foot)) - leg.lengths.y).abs() < 1e-4);
                        let (z, lift, pitch) = step(phase + side as f32 * PI, run);
                        let target = leg.ankle
                            + Vec3::new(0., lift + pitch.sin().abs() * 0.12, z) * rig.height
                                / REFERENCE_HEIGHT;
                        assert!(
                            p(leg.foot).distance(target) < 0.002,
                            "{kind:?} run={run} frame={frame} foot={side}: {:?} vs {target:?}",
                            p(leg.foot)
                        );
                    }
                    let feet = gait.legs.map(|leg|
                        world(rig, &pose, leg.foot).transform_point3(Vec3::ZERO).z);
                    let hands = gait.arms.map(|arm|
                        world(rig, &pose, arm[2]).transform_point3(Vec3::ZERO).z);
                    let units = rig.height / REFERENCE_HEIGHT;
                    // A scholar's asymmetric resting elbows can lead through
                    // the small crossing zone. At a clearly split stride the
                    // visible hands must oppose the legs, including in Run.
                    if (feet[0] - feet[1]).abs() > 0.22 * units {
                        assert!((feet[0] - feet[1]) * (hands[0] - hands[1]) < -0.002 * units * units,
                            "{kind:?} run={run} frame={frame} arms lost leg counterbalance: feet={feet:?}, hands={hands:?}");
                    }
                }
            }
        }
    }

    #[cfg_attr(test, test)]
    #[cfg_attr(test, ignore = "requires supplied human GLBs")]
    fn human_locomotion_preserves_shoe_floor_and_existing_skin_envelope() {
        for name in ["human-male", "human-female"] {
            let rig = crate::open_people::get(name).unwrap();
            let bounds = rig.locomotion_bounds();
            let gait = HumanLocomotion::new(rig);
            let mut pose = rig.nodes.clone();
            let mut moving = pose.clone();
            let mut idle = pose.clone();
            rig.sample("Idle", 0., &mut idle);
            crate::hand_pose::relax(rig, &mut idle);
            for run in [0., 0.5, 1.] {
                for weight in [1., 0.85, 0.55, 0.15] {
                    for frame in 0..60 {
                        let phase = frame as f32 / 60. * TAU;
                        gait.sample(rig, phase, run, &mut moving);
                        for ((p, a), b) in pose.iter_mut().zip(&idle).zip(&moving) {
                            *p = Transform {
                                translation: a.translation.lerp(b.translation, weight),
                                rotation: a.rotation.slerp(b.rotation, weight).normalize(),
                                scale: a.scale.lerp(b.scale, weight),
                            };
                        }
                        gait.plant_blend(rig, &idle, &moving, weight, &mut pose);
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
                                "{name} run={run} frame={frame} escapes culling bounds: {v:?}"
                            );
                            floor = floor.min(v.y);
                        }
                        assert!(
                            floor >= rig.floor - 0.01,
                            "{name} run={run} weight={weight} frame={frame} sole below floor: {floor} vs {}",
                            rig.floor
                        );
                        if run == 0. && weight == 1. {
                            assert!(
                                floor < rig.floor + 0.02,
                                "walking needs a grounded sole: {floor}"
                            );
                        }
                    }
                }
            }
        }
    }
}
