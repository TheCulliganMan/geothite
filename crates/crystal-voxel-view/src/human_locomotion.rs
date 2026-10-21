//! Distance-driven human steps. Only the skin moves; the controller owns travel.
use crate::{open_people::PersonRig, new_bark_actors::character_meshes::CharacterKind};
use bevy::prelude::*;
use std::f32::consts::{PI, TAU};
use std::sync::Arc;

pub(crate) const WALK_DISTANCE: f32 = 0.72;
pub(crate) const RUN_DISTANCE: f32 = 2.00;
pub(crate) const REFERENCE_HEIGHT: f32 = 1.38;

pub(crate) fn cycle_distance(run: f32) -> f32 {
    WALK_DISTANCE + (RUN_DISTANCE - WALK_DISTANCE) * run
}

/// The distance publisher and the posed feet must use the same character
/// stride. Shorter steps increase cadence; they never change actor travel.
pub(crate) fn stride_scale(kind: CharacterKind) -> f32 {
    Style::for_character(kind).stride
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

/// Narrow the stance without advancing the shoes. The ankle joint sits near
/// the heel: centering that joint under the hips puts the sole too far forward
/// and makes the entire cast stand back on their heels.
pub(super) fn stance_ankle(rig: &PersonRig, hips: Vec3, foot: usize, sign: f32) -> Vec3 {
    let mut point = world(rig, &rig.nodes, foot).w_axis.truncate();
    point.x = hips.x + sign * 0.085 * rig.height / REFERENCE_HEIGHT;
    if heel_balance_baseline() {
        point.z = hips.z;
    }
    point
}

fn heel_balance_baseline() -> bool {
    #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
    { return std::env::var_os("CRYSTAL_PEOPLE_REVIEW_HEEL_BALANCE_BASELINE").is_some(); }
    #[cfg(not(all(feature = "people-review", not(target_arch = "wasm32"))))]
    { false }
}

#[derive(Clone)]
struct Sole {
    min: Vec3,
    max: Vec3,
    heel: Vec3,
    toe: Vec3,
    profile: Vec<Vec2>,
    rolls: Vec<Roll>,
}

#[derive(Clone, Copy)]
struct Roll { from: f32, to: f32, point: Vec2, z: f32 }

/// Derived once from the finished uploaded skin, including adapted footwear.
/// Worker-restored meshes use this same derivation; no new wire fields.
pub(crate) struct ShoeSupport([Sole; 2]);

impl ShoeSupport {
    pub(crate) fn from_surface(rig: &PersonRig, positions: &[[f32; 3]],
        ids: &[[u16; 4]], weights: &[[f32; 4]]) -> Self {
        let feet = rig.leg_joints.map(|leg| {
            let shin = rig.skin_nodes[leg[1]];
            *rig.skin_nodes.iter().find(|&&n| rig.parents[n] == Some(shin)).unwrap()
        });
        let sides: Vec<_> = rig.skin_nodes.iter().map(|&node| {
            let mut current = Some(node);
            while let Some(n) = current {
                if let Some(side) = feet.iter().position(|&foot| foot == n) { return Some(side); }
                current = rig.parents[n];
            }
            None
        }).collect();
        let mesh_world = world(rig, &rig.nodes, rig.mesh_node);
        let mut points = [Vec::new(), Vec::new()];
        for ((p, js), ws) in positions.iter().zip(ids).zip(weights) {
            let mut attached = [0.; 2];
            for i in 0..4 {
                if let Some(Some(side)) = sides.get(js[i] as usize) { attached[*side] += ws[i]; }
            }
            for side in 0..2 {
                if attached[side] > 0.999 {
                    points[side].push(mesh_world.transform_point3(Vec3::from_array(*p)));
                }
            }
        }
        Self(std::array::from_fn(|side| {
            let ankle = world(rig, &rig.nodes, feet[side]).w_axis.truncate();
            Sole::new(&points[side], ankle)
        }))
    }
}

impl Sole {
    fn new(points: &[Vec3], ankle: Vec3) -> Self {
        assert!(points.len() >= 4, "human requires weighted soles");
        let min = points.iter().copied().fold(Vec3::splat(f32::INFINITY), Vec3::min) - ankle;
        let max = points.iter().copied().fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max) - ankle;
        let bottom: Vec<_> = points.iter().copied().map(|p| p - ankle)
            .filter(|p| p.y <= min.y + 1e-5).collect();
        let heel = *bottom.iter().min_by(|a, b| a.z.total_cmp(&b.z)).unwrap();
        let toe = *bottom.iter().max_by(|a, b| a.z.total_cmp(&b.z)).unwrap();
        let mut sorted: Vec<_> = points.iter().map(|p| (*p - ankle).yz()).collect();
        sorted.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
        sorted.dedup();
        let chain = |iter: Box<dyn Iterator<Item = Vec2>>| {
            let mut hull = Vec::<Vec2>::new();
            for p in iter {
                while hull.len() >= 2 {
                    let a = hull[hull.len()-2].as_dvec2();
                    let b = hull[hull.len()-1].as_dvec2();
                    if (b-a).perp_dot(p.as_dvec2()-b) > 1e-12 { break; }
                    hull.pop();
                }
                hull.push(p);
            }
            hull
        };
        let mut lower = chain(Box::new(sorted.clone().into_iter()));
        let mut upper = chain(Box::new(sorted.into_iter().rev()));
        lower.pop(); upper.pop(); lower.extend(upper); let profile = lower;
        assert!(profile.len() >= 3);
        let mut cuts = vec![-1.55_f32, 0., 1.55];
        for (a, b) in profile.iter().zip(profile.iter().cycle().skip(1)).take(profile.len()) {
            let edge = *b - *a;
            let mut angle = edge.x.atan2(edge.y);
            while angle > PI * 0.5 { angle -= PI; }
            while angle < -PI * 0.5 { angle += PI; }
            if angle.abs() > 1e-6 && angle.abs() < 1.55 { cuts.push(angle); }
        }
        cuts.sort_by(f32::total_cmp); cuts.dedup_by(|a,b| (*a-*b).abs() < 1e-6);
        let mut rolls: Vec<_> = cuts.windows(2).map(|p| {
            let angle = (p[0]+p[1])*0.5;
            let (s,c) = angle.sin_cos();
            let point = *profile.iter().min_by(|a,b|
                (c*a.x-s*a.y).total_cmp(&(c*b.x-s*b.y))).unwrap();
            Roll { from:p[0], to:p[1], point, z:0. }
        }).collect();
        let value = |point: Vec2, angle: f32| {
            let (s,c) = angle.sin_cos(); point.y - (s*point.x+c*point.y)
        };
        // Integrate rotations across bevels. Changing the supporting vertex
        // must not jump the ankle or slide the active contact along the floor.
        let mut previous = 0.;
        for roll in rolls.iter_mut().filter(|r| r.from >= 0.) {
            roll.z = previous - value(roll.point,roll.from);
            previous = value(roll.point,roll.to) + roll.z;
        }
        previous = 0.;
        for roll in rolls.iter_mut().rev().filter(|r| r.to <= 0.) {
            roll.z = previous - value(roll.point,roll.to);
            previous = value(roll.point,roll.from) + roll.z;
        }
        Self { min,max,heel,toe,profile,rolls }
    }
    /// Rotate about the contacting heel or toe, rather than about an ankle
    /// suspended above the floor. The contact keeps its ground position.
    fn ankle_offset(&self, pitch: f32) -> Vec2 {
        let (s, c) = pitch.sin_cos();
        let roll = self.rolls.iter().find(|r| pitch >= r.from && pitch <= r.to)
            .expect("human foot pitch exceeds support profile");
        let p = roll.point;
        Vec2::new(self.min.y - (c*p.x-s*p.y), p.y - (s*p.x+c*p.y) + roll.z)
    }

    fn minimum_height(&self, rotation: Quat) -> f32 {
        let center = (self.min + self.max) * 0.5;
        let half = (self.max - self.min) * 0.5;
        let vertical = rotation.inverse() * Vec3::Y;
        if vertical.x.abs() < 1e-5 {
            return self.profile.iter().map(|p| vertical.y*p.x+vertical.z*p.y)
                .fold(f32::INFINITY,f32::min);
        }
        (rotation * center).y - vertical.abs().dot(half)
    }
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
    support: Arc<ShoeSupport>,
    neutral: Vec<Transform>,
    hips: Vec3,
    legs: [Leg; 2],
    arms: [[usize; 3]; 2],
    elbow_hinges: [Vec3; 2],
    style: Style,
    hands: crate::hand_pose::Hands,
}

#[derive(Clone, Copy)]
struct Style {
    swing: f32,
    elbow: f32,
    twist: f32,
    lean: f32,
    stride: f32,
    lift: f32,
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
        let (stride, lift) = match kind {
            Elder | Gramps | Granny | Pryce | Kurt | KurtOutside | Sage | Blaine
                | Gentleman | Pharmacist | FishingGuru => (0.82, 0.80),
            Mom | RedsMom | Daisy | Teacher | PokefanF | Nurse | Receptionist
                | LinkReceptionist | OldLinkReceptionist | Clerk | Officer
                | Beauty | Lass | KimonoGirl | Erika | Jasmine | Sabrina => (0.90, 0.88),
            _ => (1., 1.),
        };
        let mut style = Self { swing, elbow, twist, lean, stride, lift };
        #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
        if std::env::var_os("CRYSTAL_PEOPLE_REVIEW_GARMENT_GAIT_BASELINE").is_some() {
            return style;
        }
        let look = crate::npc_appearance::look(kind);
        // A closed skirt cannot accompany the same long, high return step
        // as trousers. Keep travel distance authoritative: the publisher uses
        // this same stride factor to increase cadence for the shorter steps.
        // Falkner's intended cut has separate trouser legs; its interim skirt
        // must not determine his eventual motion style.
        if kind != Falkner && look.costume != "hakama" {
            if let Some(hem) = crate::garment_body::hem(look) {
                let (stride, lift, swing) = if hem < 0.5 {
                    (0.78, 0.50, 0.88)
                } else {
                    (0.92, 0.72, 0.94)
                };
                style.stride *= stride;
                style.lift *= lift;
                style.swing *= swing;
            }
        }
        style
    }

    fn step(self, phase: f32, run: f32) -> (f32, f32, f32) {
        let (z, lift, pitch) = step(phase, run);
        // Toe-off has to articulate the shoe even beneath a long garment.
        // Garment lift scales the return height, rather than pinning the heel.
        (z * self.stride, lift * self.lift,
            if heel_balance_baseline() { pitch * self.lift } else { pitch })
    }
}

/// Normalized left/right contact cycle: heel, flat sole, toe-off, lifted return.
/// The Hermite return matches stance velocity at both joins, avoiding a kick
/// when the foot leaves or reaches the ground. Phase zero is left heel contact.
fn step(phase: f32, run: f32) -> (f32, f32, f32) {
    let stance = 0.54 + (0.22 - 0.54) * run;
    let cycle = cycle_distance(run);
    let span = cycle * stance;
    let p = (phase / TAU).rem_euclid(1.0);
    let smoothstep = |t: f32| { let t = t.clamp(0., 1.); t * t * (3. - 2. * t) };
    let heel = -0.12;
    let toe = 0.42 + 0.12 * run;
    if p < stance {
        let contact = p / stance;
        let pitch = if heel_balance_baseline() { 0. } else {
            heel * (1. - smoothstep(contact / 0.18))
                + toe * smoothstep((contact - 0.45) / 0.55)
        };
        (span * 0.5 - cycle * p, 0., pitch)
    } else {
        let t = (p - stance) / (1. - stance);
        let smooth = t * t * (3. - 2. * t);
        let tangent = -cycle * (1. - stance);
        let z = -span * 0.5 + span * smooth + tangent * t * (1. - t) * (1. - 2. * t);
        let lift = (PI * t).sin().powi(2) * (0.055 + 0.095 * run);
        // A squared envelope also gives zero angular velocity at contact.
        let pitch = if heel_balance_baseline() {
            0.32 * (TAU * t).sin() * (PI * t).sin().powi(2)
        } else {
            // Bring the shoe level in mid-flight, then prepare the heel for
            // contact. Both pivot changes have zero angular velocity.
            toe * (1. - smoothstep(t / 0.55))
                + heel * smoothstep((t - 0.65) / 0.35)
        };
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
        let support = Arc::new(ShoeSupport::from_surface(rig,
            &rig.mesh.positions, &rig.ids, &rig.weights));
        Self::with_support(rig, kind, support)
    }

    pub(crate) fn with_support(rig: &PersonRig, kind: CharacterKind,
        support: Arc<ShoeSupport>) -> Self {
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
                ankle: stance_ankle(rig, hips, foot, if side == 0 { 1. } else { -1. }),
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
            support,
            hips,
            neutral,
            legs,
            arms,
            elbow_hinges: [Vec3::ZERO; 2],
            style: Style::for_character(kind),
            hands: crate::hand_pose::Hands::new(rig),
        };
        let idle = crate::human_idle::HumanIdle::new(
            rig, kind,
        );
        let mut relaxed = gait.neutral.clone();
        idle.sample(rig, &gait, 0., &mut relaxed);
        gait.neutral = relaxed;
        // Store each elbow plane in the upper arm's frame. Carrying this
        // anatomical axis through the shoulder is cheaper than rebuilding
        // three world-space joint positions for every elbow every frame.
        gait.elbow_hinges = gait.arms.map(|[upper, lower, wrist]| {
            let inverse = world(rig, &gait.neutral, upper).inverse();
            let elbow = inverse.transform_point3(world(rig, &gait.neutral, lower).w_axis.truncate());
            let hand = inverse.transform_point3(world(rig, &gait.neutral, wrist).w_axis.truncate());
            elbow.cross(hand - elbow).normalize()
        });
        gait
    }

    pub(crate) fn sample(&self, rig: &PersonRig, phase: f32, run: f32, pose: &mut [Transform]) {
        pose.copy_from_slice(&self.neutral);
        let phase = phase.rem_euclid(TAU);
        let run = run.clamp(0., 1.);
        let units = rig.height / REFERENCE_HEIGHT;
        let hips = rig.skin_nodes[rig.hips_joint];
        let targets = std::array::from_fn::<_, 2, _>(|side|
            self.foot_target(self.legs[side], side, phase + side as f32 * PI, run, units));
        // Keep the pelvis upright with a nearly extended supporting leg.
        // The small two-beat rise accompanies weight transfer, rather than
        // importing the source run's large root bounce into an ordinary walk.
        let bounce = (1. - (2. * phase).cos()) * (0.006 + 0.018 * run)
            * self.style.lift * units;
        // Let heel contact extend the supporting leg instead of retaining a
        // permanently lowered pelvis. The reach cap still prevents stretching.
        let rise = if heel_balance_baseline() { 0. } else { 0.02 * units };
        let desired = self.fit_hips_to_feet(
            self.hips + Vec3::new(0.008 * units * phase.sin(), rise + bounce, 0.),
            targets.map(|(point, _)| point), 0.012 * units);
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
        #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
        let world_axis_elbows = std::env::var_os("CRYSTAL_PEOPLE_REVIEW_ELBOW_WORLD_AXIS").is_some();
        #[cfg(not(all(feature = "people-review", not(target_arch = "wasm32"))))]
        let world_axis_elbows = false;
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
            // Each standing manner supplies its own elbow plane. Flex around
            // that plane's hinge, rather than turning every forearm around
            // world X: a relaxed upper-arm swivel must not become elbow twist
            // when the actor starts moving.
            if world_axis_elbows {
                rotate_world(rig, pose, lower, Quat::from_euler(EulerRot::YXZ,
                    sign * 0.025 * arm_phase.sin(), -fold, 0.));
            } else {
                let hinge = world(rig, pose, upper)
                    .transform_vector3(self.elbow_hinges[side]).normalize();
                rotate_world(rig, pose, lower, Quat::from_axis_angle(hinge, fold));
                let forearm = world(rig, pose, lower)
                    .transform_vector3(pose[wrist].translation).normalize();
                rotate_world(rig, pose, lower, Quat::from_axis_angle(forearm,
                    -sign * 0.025 * arm_phase.sin()));
            }
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
        self.hands.sample(0.10 + 0.90 * run, phase.sin(), pose);
    }

    fn foot_target(&self, leg: Leg, side: usize, phase: f32, run: f32, units: f32) -> (Vec3, f32) {
        let (z, lift, pitch) = self.style.step(phase, run);
        let offset = if heel_balance_baseline() {
            Vec2::new(pitch.sin().abs() * 0.12 * units, 0.)
        } else { self.support.0[side].ankle_offset(pitch) };
        (leg.ankle + Vec3::new(0., lift * units + offset.x, z * units + offset.y), pitch)
    }

    fn fit_hips_to_feet(&self, mut desired: Vec3, targets: [Vec3; 2], softness: f32) -> Vec3 {
        for (leg, target) in self.legs.iter().zip(targets) {
            let reach = (leg.lengths.x + leg.lengths.y) * 0.995;
            let horizontal = (target - desired - leg.hip_offset).xz().length_squared();
            let cap = target.y + (reach * reach - horizontal).max(0.).sqrt() - leg.hip_offset.y;
            // Smooth minimum avoids a pelvis velocity kink when support swaps.
            desired.y =
                0.5 * (desired.y + cap - ((desired.y - cap).powi(2) + softness * softness).sqrt());
        }
        desired
    }

    pub(super) fn idle_hips(&self, desired: Vec3, feet: &[(Vec3, Quat); 2], units: f32) -> Vec3 {
        if heel_balance_baseline() { return desired; }
        self.fit_hips_to_feet(desired, feet.map(|(point, _)| point), 0.003 * units)
    }

    #[cfg(any(test, feature = "people-review"))]
    pub(super) fn sole_contacts(&self, rig: &PersonRig, pose: &[Transform], side: usize) -> [Vec3; 2] {
        let leg = self.legs[side];
        let foot = world(rig, pose, leg.foot);
        [self.support.0[side].heel, self.support.0[side].toe].map(|p|
            foot.transform_point3(leg.rotation.inverse() * p))
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
        for (side, leg) in self.legs.into_iter().enumerate() {
            let a = world(rig, idle, leg.foot).to_scale_rotation_translation();
            let b = world(rig, moving, leg.foot).to_scale_rotation_translation();
            let rotation = a.1.slerp(b.1, weight);
            let mut target = a.2.lerp(b.2, weight);
            if !heel_balance_baseline() {
                // Interpolating ankle heights does not preserve a rolled
                // sole's floor clearance. Reapply contact after blending.
                let sole = &self.support.0[side];
                let floor = leg.ankle.y + sole.min.y;
                let bottom = sole.minimum_height(rotation * leg.rotation.inverse());
                target.y = target.y.max(floor - bottom);
            }
            self.solve_leg(rig, pose, leg, target);
            let current = world(rig, pose, leg.foot).to_scale_rotation_translation().1;
            rotate_world(
                rig,
                pose,
                leg.foot,
                rotation * current.inverse(),
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
        sole_roll_preserves_ground_contacts_and_smooth_pivot_changes();
        beveled_soles_roll_without_gaps_or_contact_sliding();
        elbows_flex_in_their_moving_hinge_planes_for_every_identity();
        crate::hand_pose::validation::native_fingers_relax_and_close_without_wrist_or_skin_drift();
        crate::limb_surface::validation::supplied_limbs_preserve_landmarks_and_animate_without_seam_cracks();
        human_steps_match_ground_speed_and_join_without_velocity_pops();
        human_locomotion_keeps_contacts_bone_lengths_and_loop_seams_for_every_identity();
        human_gaits_keep_character_and_smooth_arm_followthrough();
        human_locomotion_preserves_shoe_floor_and_existing_skin_envelope();
        crate::new_bark_actors::validate_human_cadence();
    }

    #[cfg_attr(test, test)]
    fn beveled_soles_roll_without_gaps_or_contact_sliding() {
        let points: Vec<_> = [-0.04, 0.04].into_iter().flat_map(|x|
            [(-0.07,-0.04),(-0.07,0.12),(-0.06,0.16),(0.02,0.16),(0.02,-0.04)]
                .into_iter().map(move |(y,z)| Vec3::new(x,y,z))).collect();
        let sole = Sole::new(&points,Vec3::ZERO);
        let mut changed_contact = false;
        for frame in 0..128 {
            let pitch = -0.12 + 0.66 * frame as f32 / 127.;
            let offset = sole.ankle_offset(pitch);
            let rotation = Quat::from_rotation_x(pitch);
            let floor = points.iter().map(|p| (rotation * *p).y + offset.x)
                .fold(f32::INFINITY,f32::min);
            assert!((floor-sole.min.y).abs() < 2e-6,"beveled shoe lost ground contact");
            let roll = sole.rolls.iter().find(|r| pitch>=r.from && pitch<=r.to).unwrap();
            changed_contact |= roll.point.x > sole.min.y + 0.005;
            let h = 1e-5;
            if pitch-h>roll.from && pitch+h<roll.to {
                let contact = |a: f32| {
                    let (s,c)=a.sin_cos();
                    sole.ankle_offset(a).y + s*roll.point.x + c*roll.point.y
                };
                assert!((contact(pitch+h)-contact(pitch-h)).abs()<2e-6,
                    "active shoe contact slides during roll");
            }
        }
        assert!(changed_contact,"fixture must exercise a raised toe bevel");
        for roll in &sole.rolls {
            if !(-0.12..0.54).contains(&roll.to) {continue;}
            assert!(sole.ankle_offset(roll.to-1e-6)
                .distance(sole.ankle_offset(roll.to+1e-6))<3e-6,
                "shoe support change jumps the ankle");
        }
        let pitch=0.5;
        let (s,c)=f32::sin_cos(pitch);
        let box_floor=c*sole.min.y-s*sole.max.z;
        let exact_floor=sole.minimum_height(Quat::from_rotation_x(pitch));
        assert!(exact_floor-box_floor>0.005,"fixture must reject the old floating box support");
    }

    #[cfg_attr(test, test)]
    fn sole_roll_preserves_ground_contacts_and_smooth_pivot_changes() {
        let points: Vec<_> = [-0.04, 0.04].into_iter().flat_map(|x|
            [-0.07, 0.02].into_iter().flat_map(move |y|
                [-0.04, 0.16].into_iter().map(move |z| Vec3::new(x,y,z)))).collect();
        let sole = Sole::new(&points, Vec3::ZERO);
        for pitch in [-0.12, -0.06, 0., 0.2, 0.42, 0.54] {
            let offset = sole.ankle_offset(pitch);
            let rotation = Quat::from_rotation_x(pitch);
            for z in [sole.min.z, sole.max.z] {
                let p = rotation * Vec3::new(0., sole.min.y, z)
                    + Vec3::new(0., offset.x, offset.y);
                assert!(p.y >= sole.min.y - 1e-6, "rolled sole penetrates ground");
                if z == if pitch >= 0. { sole.max.z } else { sole.min.z } {
                    assert!((p.y - sole.min.y).abs() < 1e-6);
                    assert!((p.z - z).abs() < 1e-6, "contact slid during ankle rotation");
                }
            }
        }
        for run in [0., 0.25, 0.5, 0.75, 1.] {
            let stance = 0.54 + (0.22 - 0.54) * run;
            for p in [0., stance * 0.18, stance * 0.45, stance,
                stance + (1. - stance) * 0.55, stance + (1. - stance) * 0.65] {
                let sample = |p: f32| {
                    let (z, lift, pitch) = step(p * TAU, run);
                    let offset = sole.ankle_offset(pitch);
                    Vec3::new(z + offset.y, lift + offset.x, pitch)
                };
                let h = 1e-4;
                let a = sample(p - h); let b = sample(p); let c = sample(p + h);
                assert!(((b - a) - (c - b)).length() < 2e-5,
                    "foot pivot velocity pop, run={run} p={p}");
            }
        }
    }

    #[cfg_attr(test, test)]
    #[cfg_attr(test, ignore = "requires supplied human GLBs")]
    fn elbows_flex_in_their_moving_hinge_planes_for_every_identity() {
        use crate::new_bark_actors::character_meshes::SOURCE_KINDS;
        let mut poses = 0;
        let mut maximum_plane_error = 0_f32;
        for kind in SOURCE_KINDS.iter().map(|&(_, kind)| kind).chain([CharacterKind::Outdoorsman]) {
            let rig = crate::open_people::get(crate::npc_appearance::look(kind).base).unwrap();
            let gait = HumanLocomotion::for_character(rig, kind);
            let mut pose = rig.nodes.clone();
            let local_bones = |pose: &[Transform], [upper, lower, wrist]: [usize; 3]| {
                let inverse = world(rig, pose, upper).inverse();
                let elbow = inverse.transform_point3(world(rig, pose, lower).w_axis.truncate());
                let hand = inverse.transform_point3(world(rig, pose, wrist).w_axis.truncate());
                (elbow.normalize(), (hand - elbow).normalize())
            };
            let rest = gait.arms.map(|arm| {
                let (a, b) = local_bones(&gait.neutral, arm);
                (a.cross(b).normalize(), a.dot(b).clamp(-1., 1.).acos())
            });
            for run in [0., 0.25, 0.5, 0.75, 1.] {
                for frame in 0..32 {
                    gait.sample(rig, frame as f32 / 32. * TAU, run, &mut pose);
                    for (side, arm) in gait.arms.iter().copied().enumerate() {
                        let (a, b) = local_bones(&pose, arm);
                        let hinge = a.cross(b).normalize();
                        let error = 1. - hinge.dot(rest[side].0);
                        assert!(hinge.is_finite() && error < 2e-5,
                            "{kind:?}: elbow left its hinge plane, side={side} run={run} frame={frame} error={error}");
                        assert!(a.dot(b).clamp(-1., 1.).acos() > rest[side].1 + 0.04,
                            "{kind:?}: walking elbow failed to flex, side={side} run={run} frame={frame}");
                        maximum_plane_error = maximum_plane_error.max(error);
                    }
                    poses += 1;
                }
            }
        }
        assert_eq!(poses, 12000);
        eprintln!("Elbow hinge checks: {poses} poses across all 75 people; maximum plane error={maximum_plane_error}");
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
                assert_eq!(a.1, 0.);
                assert!((-0.12..=0.54).contains(&a.2));
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
            for leg in gait.legs {
                let bind_ankle = world(rig, &rig.nodes, leg.foot).w_axis.truncate();
                assert!((leg.ankle.z - bind_ankle.z).abs() < 1e-6,
                    "{kind:?}: shoe advanced from its anatomical standing offset");
            }
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
                        let (target, _) = gait.foot_target(*leg, side, phase + side as f32 * PI,
                            run, rig.height / REFERENCE_HEIGHT);
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
