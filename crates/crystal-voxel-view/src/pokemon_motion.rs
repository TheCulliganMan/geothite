//! Render-only articulation for the complete species catalogue. Existing GLB
//! clips stay authoritative; named source parts supply joints for older meshes.
use super::*;
use std::f32::consts::TAU;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Locomotion {
    Walk,
    Fly,
    Glide,
    Float,
}

pub(crate) fn locomotion(name: &str) -> Locomotion {
    // Runtime rig identities are canonical lowercase; public review selectors
    // may also be uppercase.
    let lower;
    let name = if name.bytes().any(|b| b.is_ascii_uppercase()) {
        lower = name.to_ascii_lowercase();
        lower.as_str()
    } else {
        name
    };
    match name {
        "cloyster" | "shellder" | "mew" | "gastly" | "haunter" | "misdreavus" | "magnemite"
        | "magneton" | "geodude" | "koffing" | "weezing" | "porygon" | "porygon2" | "tentacool"
        | "tentacruel" | "horsea" | "seadra" | "kingdra" | "magikarp" | "gyarados" | "goldeen"
        | "seaking" | "chinchou" | "lanturn" | "remoraid" | "qwilfish" | "lapras" | "staryu"
        | "starmie" => Locomotion::Float,
        "gligar" | "mantine" | "jumpluff" | "hoppip" | "skiploom" => Locomotion::Glide,
        "pidgey" | "pidgeotto" | "pidgeot" | "spearow" | "fearow" | "zubat" | "golbat"
        | "crobat" | "butterfree" | "beedrill" | "venomoth" | "aerodactyl" | "charizard"
        | "articuno" | "zapdos" | "moltres" | "dragonite" | "noctowl" | "ledyba" | "ledian"
        | "yanma" | "togetic" | "murkrow" | "skarmory" | "lugia" | "ho_oh" | "celebi" => {
            Locomotion::Fly
        }
        name if name.starts_with("unown") => Locomotion::Float,
        _ => Locomotion::Walk,
    }
}

/// Swimming locomotion does not imply hovering during a battle. Lapras's
/// authored belly is seated; its flippers and neck supply the swimming motion.
pub(crate) fn battle_hovers(name: &str) -> bool {
    locomotion(name) == Locomotion::Float && !name.eq_ignore_ascii_case("lapras")
}

/// A hovering battler keeps its clearance throughout idle, attack and recoil.
/// Source cue time still controls the authored rotations.
pub(crate) fn battle_float(rig: &SpeciesRig, pose: &mut [Transform]) {
    if battle_hovers(rig.species.name()) {
        pose[0].translation.y += (rig.neutral_bounds.1.y - rig.neutral_bounds.0.y) * 0.16;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Role {
    Body,
    Head,
    Arm,
    Leg,
    Wing,
    Tail,
    Other,
}

fn role(name: &str) -> Role {
    // Anatomy is "Category / description"; skeletons are "species/joint".
    // Descriptive prose and species names never determine skin ownership.
    let name = if let Some((category, _)) = name.split_once(" / ") {
        category
    } else {
        name.rsplit('/').next().unwrap_or(name)
    };
    let has = |candidates: &[&str]| {
        name.split(|c: char| !c.is_ascii_alphabetic()).any(|word| {
            candidates
                .iter()
                .any(|candidate| word.eq_ignore_ascii_case(candidate))
        })
    };
    if has(&["tail"]) {
        Role::Tail
    } else if has(&["wing", "wings", "primary", "secondary", "covert"])
        || has(&["flight"]) && has(&["feather", "feathers"])
    {
        Role::Wing
    } else if has(&[
        "leg", "legs", "proleg", "foot", "feet", "foreleg", "hindleg", "hind", "thigh", "shin",
        "shank", "haunch", "hock", "toe", "toes", "talon", "ankle", "knee", "calf", "paw", "shoe",
        "boot", "sole", "heel", "paws", "hoof", "hooves",
    ]) || has(&["fore", "rear"]) && has(&["left", "right"])
    {
        Role::Leg
    } else if has(&[
        "arm", "arms", "forearm", "hand", "hands", "palm", "finger", "fingers", "elbow", "pincer",
        "claw", "claws", "wrist", "shoulder",
    ]) {
        Role::Arm
    } else if has(&[
        "head",
        "cranium",
        "face",
        "eye",
        "eyes",
        "pupil",
        "glint",
        "mask",
        "ear",
        "ears",
        "bill",
        "beak",
        "jaw",
        "mandible",
        "mouth",
        "tongue",
        "crown",
        "snout",
        "muzzle",
        "horn",
        "horns",
        "crest",
        "antenna",
        "whisker",
        "lip",
        "teeth",
        "fang",
        "tooth",
        "nose",
        "cheek",
        "iris",
        "sclera",
        "nostril",
        "catchlight",
        "eyelid",
        "forehead",
        "brow",
        "helmet",
        "beard",
    ]) {
        Role::Head
    } else if has(&[
        "body", "torso", "pelvis", "chest", "abdomen", "thorax", "mantle", "hull", "core", "shell",
    ]) {
        Role::Body
    } else {
        Role::Other
    }
}

fn quadruped(name: &str) -> bool {
    matches!(
        name,
        "bulbasaur"
            | "ivysaur"
            | "venusaur"
            | "rattata"
            | "raticate"
            | "vulpix"
            | "ninetales"
            | "nidoran_f"
            | "nidoran_m"
            | "nidorina"
            | "nidorino"
            | "tauros"
            | "mareep"
            | "meowth"
            | "persian"
            | "growlithe"
            | "arcanine"
            | "ponyta"
            | "rapidash"
            | "slowpoke"
            | "rhyhorn"
            | "eevee"
            | "vaporeon"
            | "jolteon"
            | "flareon"
            | "chikorita"
            | "bayleef"
            | "meganium"
            | "cyndaquil"
            | "quilava"
            | "furret"
            | "espeon"
            | "umbreon"
            | "girafarig"
            | "swinub"
            | "piloswine"
            | "houndour"
            | "houndoom"
            | "phanpy"
            | "donphan"
            | "stantler"
            | "raikou"
            | "entei"
            | "suicune"
    )
}

const NAMES: [&str; 12] = [
    "root",
    "body",
    "head",
    "arm_left",
    "arm_right",
    "fore_left",
    "fore_right",
    "rear_left",
    "rear_right",
    "wing_left",
    "wing_right",
    "tail",
];

pub(crate) fn from_mesh(
    name: &'static str,
    neutral: SurfaceMeshData,
    anatomy: Vec<Anatomy>,
) -> SpeciesRig {
    let neutral_bounds = point_bounds(neutral.positions.iter().copied().map(Vec3::from_array));
    let (min, max) = neutral_bounds;
    let size = max - min;
    let center = (min + max) * 0.5;
    let mut pivots = [
        Vec3::ZERO,
        center,
        Vec3::new(center.x, min.y + size.y * 0.72, center.z),
        Vec3::new(center.x - size.x * 0.2, min.y + size.y * 0.58, center.z),
        Vec3::new(center.x + size.x * 0.2, min.y + size.y * 0.58, center.z),
        Vec3::new(
            center.x - size.x * 0.2,
            min.y + size.y * 0.24,
            center.z + size.z * 0.2,
        ),
        Vec3::new(
            center.x + size.x * 0.2,
            min.y + size.y * 0.24,
            center.z + size.z * 0.2,
        ),
        Vec3::new(
            center.x - size.x * 0.2,
            min.y + size.y * 0.24,
            center.z - size.z * 0.2,
        ),
        Vec3::new(
            center.x + size.x * 0.2,
            min.y + size.y * 0.24,
            center.z - size.z * 0.2,
        ),
        Vec3::new(center.x - size.x * 0.15, min.y + size.y * 0.60, center.z),
        Vec3::new(center.x + size.x * 0.15, min.y + size.y * 0.60, center.z),
        Vec3::new(center.x, min.y + size.y * 0.35, center.z - size.z * 0.25),
    ];
    // A head turns at its connection to the torso, not at a percentage of
    // the species height (antennae and horns can dominate that height).
    if let Some(part) = anatomy.iter().find(|p| {
        p.name
            .split_once(" / ")
            .map_or(p.name.as_str(), |p| p.0)
            .split(|c: char| !c.is_ascii_alphabetic())
            .any(|w| w.eq_ignore_ascii_case("head"))
    }) {
        let (lo, hi) = point_bounds(
            neutral.positions[part.vertices.clone()]
                .iter()
                .copied()
                .map(Vec3::from_array),
        );
        let c = (lo + hi) * 0.5;
        pivots[2] = Vec3::new(
            c.x,
            lo.y + (hi.y - lo.y) * 0.15,
            if matches!(name, "caterpie" | "weedle") {
                lo.z
            } else {
                c.z
            },
        );
    }
    let joints: Vec<_> = NAMES
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let parent = match i {
                0 => None,
                1 | 5..=8 => Some(0),
                _ => Some(1),
            };
            Joint {
                name: format!("{name}/{n}"),
                parent,
                bind: Transform::from_translation(
                    pivots[i] - parent.map_or(Vec3::ZERO, |p| pivots[p]),
                ),
                inverse_bind: Mat4::from_translation(-pivots[i]),
            }
        })
        .collect();
    let named_head = anatomy.iter().any(|part| {
        let category = part
            .name
            .split_once(" / ")
            .map_or(part.name.as_str(), |(category, _)| category);
        category
            .split(|c: char| !c.is_ascii_alphabetic())
            .any(|word| word.eq_ignore_ascii_case("head") || word.eq_ignore_ascii_case("cranium"))
    });
    let mut joint_indices = vec![[1, 0, 0, 0]; neutral.positions.len()];
    for part in &anatomy {
        let (lo, hi) = point_bounds(
            neutral.positions[part.vertices.clone()]
                .iter()
                .copied()
                .map(Vec3::from_array),
        );
        let c = (lo + hi) * 0.5;
        let right = usize::from(c.x >= center.x);
        let source_role = role(&part.name);
        let category = part
            .name
            .split_once(" / ")
            .map_or(part.name.as_str(), |p| p.0);
        let has = |token: &str| {
            category
                .split(|c: char| !c.is_ascii_alphabetic())
                .any(|w| w.eq_ignore_ascii_case(token))
        };
        let part_role = if has("claw")
            && !has("hand")
            && !has("pincer")
            && !has("arm")
            && source_role != Role::Leg
        {
            if c.y < min.y + size.y * 0.3 {
                Role::Leg
            } else {
                Role::Arm
            }
        } else {
            source_role
        };
        let owner = match part_role {
            Role::Head if !named_head => 1,
            Role::Head
                if matches!(
                    name,
                    "magnemite"
                        | "magneton"
                        | "koffing"
                        | "weezing"
                        | "electrode"
                        | "starmie"
                        | "ditto"
                        | "grimer"
                        | "muk"
                ) =>
            {
                1
            }
            Role::Head => 2,
            Role::Arm if quadruped(name) && has("shoulder") => 1,
            Role::Arm => 3 + right,
            Role::Leg => {
                5 + right
                    + if quadruped(name) && c.z < center.z {
                        2
                    } else {
                        0
                    }
            }
            Role::Wing => 9 + right,
            Role::Tail if matches!(name, "caterpie" | "weedle") => 1,
            Role::Tail => 11,
            _ => 1,
        };
        for v in part.vertices.clone() {
            joint_indices[v] = [owner as u16, 0, 0, 0];
        }
    }
    // Explicit attachment labels are metadata, not prose used as anatomy.
    // Ankle flames travel with their leg; forelocks and tail flames keep their
    // actual head/tail owners instead of hanging in space during a pose.
    for part in &anatomy {
        let Some((category, attachment)) = part.name.split_once(" / ") else {
            continue;
        };
        if !matches!(category, "Flame" | "Flame glow") {
            continue;
        }
        let owner = if attachment.starts_with("forelock") {
            2
        } else if attachment.starts_with("tail") {
            11
        } else if attachment.starts_with("ankle") {
            let (lo, hi) = point_bounds(
                neutral.positions[part.vertices.clone()]
                    .iter()
                    .copied()
                    .map(Vec3::from_array),
            );
            let c = (lo + hi) * 0.5;
            5 + usize::from(c.x >= center.x)
                + if quadruped(name) && c.z < center.z {
                    2
                } else {
                    0
                }
        } else {
            1
        };
        for v in part.vertices.clone() {
            joint_indices[v] = [owner as u16, 0, 0, 0];
        }
    }
    let clips = Clip::ALL.map(|clip| {
        let duration = match clip {
            Clip::Idle => 1.6,
            Clip::Attack => 0.6,
            Clip::Hit => 0.4,
        };
        let times = (0..=32)
            .map(|i| duration * i as f32 / 32.)
            .collect::<Vec<_>>();
        let rotations = joints[1..]
            .iter()
            .map(|joint| {
                times
                    .iter()
                    .map(|&t| {
                        let p = t / duration;
                        if p == 0. || p == 1. {
                            return Quat::IDENTITY;
                        }
                        let pulse = (std::f32::consts::PI * p).sin().powi(2);
                        let side = if joint.name.contains("left") { -1. } else { 1. };
                        match (clip, role(&joint.name)) {
                            (Clip::Idle, Role::Wing) => {
                                Quat::from_rotation_z(side * 0.13 * (TAU * p).sin())
                            }
                            (Clip::Idle, Role::Body | Role::Tail) => {
                                Quat::from_rotation_y(0.025 * (TAU * p).sin())
                            }
                            (Clip::Attack, Role::Body | Role::Head) => {
                                Quat::from_rotation_x(-0.20 * pulse)
                            }
                            (Clip::Attack, Role::Arm) => Quat::from_rotation_x(-0.85 * pulse),
                            (Clip::Attack, Role::Wing) => {
                                Quat::from_rotation_z(side * 0.55 * pulse)
                            }
                            (Clip::Attack, Role::Tail) => Quat::from_rotation_y(0.5 * pulse),
                            (Clip::Hit, Role::Body | Role::Head) => {
                                Quat::from_rotation_x(0.18 * pulse)
                            }
                            _ => Quat::IDENTITY,
                        }
                    })
                    .collect()
            })
            .collect();
        SpeciesClip {
            name: format!("{name}.{}", clip.suffix()),
            duration,
            loop_suggested: clip == Clip::Idle,
            cue_relative: clip != Clip::Idle,
            extras: String::new(),
            times,
            rotations,
        }
    });
    let mut rig = SpeciesRig {
        species: Species::Procedural(name),
        source_normals: neutral.normals.clone(),
        joint_weights: vec![[1., 0., 0., 0.]; neutral.positions.len()],
        neutral,
        joint_indices,
        anatomy,
        materials: Vec::new(),
        joints,
        clips,
        neutral_bounds,
        animated_bounds: neutral_bounds,
        hover_clearance: 0.,
        motion_limbs: Vec::new(),
        crawl_segments: Vec::new(),
        walk_legs: Vec::new(),
        walk_cycle: 0.72,
        ground_vertices: Vec::new(),
    };
    weld_weights(&mut rig);
    rig.animated_bounds = rig.compute_envelope();
    rig
}

/// Material and normal splits at the same point share one skin binding.
/// Keeping separate render vertices must not open holes during articulation.
fn weld_weights(rig: &mut SpeciesRig) {
    use std::collections::HashMap;
    let epsilon = (rig.neutral_bounds.1.y - rig.neutral_bounds.0.y) * 0.0001;
    let mut groups: HashMap<[i32; 3], Vec<usize>> = HashMap::new();
    for (i, p) in rig.neutral.positions.iter().enumerate() {
        groups
            .entry(p.map(|v| (v / epsilon).round() as i32))
            .or_default()
            .push(i);
    }
    for indices in groups.values().filter(|v| v.len() > 1) {
        let first = indices[0];
        if indices.iter().all(|&i| {
            rig.joint_indices[i] == rig.joint_indices[first]
                && rig.joint_weights[i] == rig.joint_weights[first]
        }) {
            continue;
        }
        let mut influences: HashMap<u16, f32> = HashMap::new();
        for &i in indices {
            for (&j, w) in rig.joint_indices[i].iter().zip(rig.joint_weights[i]) {
                if w > 0. {
                    *influences.entry(j).or_default() += w;
                }
            }
        }
        let mut weights: Vec<_> = influences.into_iter().collect();
        weights.sort_by(|(a, wa), (b, wb)| wb.total_cmp(wa).then_with(|| a.cmp(b)));
        weights.truncate(4);
        let total = weights.iter().map(|(_, w)| w).sum::<f32>();
        let mut js = [0; 4];
        let mut ws = [0.; 4];
        for (i, (j, w)) in weights.into_iter().enumerate() {
            js[i] = j;
            ws[i] = w / total;
        }
        for &i in indices {
            rig.joint_indices[i] = js;
            rig.joint_weights[i] = ws;
        }
    }
}

pub(crate) fn pidgeotto() -> SpeciesRig {
    let source = crate::pidgeotto_rig::rig();
    let mut indices = 0;
    let anatomy = source
        .anatomy
        .iter()
        .map(|part| {
            let count = source
                .neutral
                .indices
                .iter()
                .filter(|&&v| part.vertices.contains(&(v as usize)))
                .count();
            let result = Anatomy {
                name: part.name.clone(),
                vertices: part.vertices.clone(),
                indices: indices..indices + count,
                material: 0,
            };
            indices += count;
            result
        })
        .collect();
    let mut rig = from_mesh("pidgeotto", source.neutral.clone(), anatomy);
    // Preserve the exact authored wing hinges and idle rotations.
    for wing in 0..2 {
        let index = 9 + wing;
        rig.joints[index].parent = Some(0);
        rig.joints[index].bind.translation = source.pivots[wing];
        rig.joints[index].inverse_bind = Mat4::from_translation(-source.pivots[wing]);
        for part in source.anatomy.iter().filter(|p| p.group == wing + 1) {
            for v in part.vertices.clone() {
                rig.joint_indices[v] = [index as u16, 0, 0, 0];
            }
        }
    }
    rig.clips[0].duration = source.duration;
    rig.clips[0].times = (0..=64).map(|i| source.duration * i as f32 / 64.).collect();
    rig.clips[0].rotations = rig.joints[1..]
        .iter()
        .enumerate()
        .map(|(i, _)| {
            rig.clips[0]
                .times
                .iter()
                .map(|&t| {
                    if i == 8 || i == 9 {
                        source.sample(t).unwrap()[i - 7].rotation
                    } else {
                        Quat::IDENTITY
                    }
                })
                .collect()
        })
        .collect();
    rig.animated_bounds = rig.compute_envelope();
    rig
}

#[derive(Clone, Debug)]
pub(crate) struct MotionLimb {
    joint: usize,
    role: Role,
    authored_flight: bool,
}

fn flight_beat(name: &str) -> (f32, f32) {
    match name {
        "lugia" | "ho_oh" | "articuno" | "zapdos" | "moltres" => (0.85, 0.36),
        "charizard" | "dragonite" | "aerodactyl" => (1.25, 0.40),
        "butterfree" | "beedrill" | "venomoth" | "yanma" | "celebi" => (6., 0.45),
        "zubat" | "golbat" | "crobat" => (3.5, 0.45),
        _ => (2.5, 0.38),
    }
}

fn motion_limbs(rig: &SpeciesRig) -> Vec<MotionLimb> {
    let mut roles: Vec<_> = rig.joints.iter().map(|j| role(&j.name)).collect();
    // Some authored butterfly joints are named upper_left/lower_left. Their
    // named wing geometry supplies the role without interpreting prose.
    for part in rig.anatomy.iter().filter(|p| role(&p.name) == Role::Wing) {
        for v in [
            part.vertices.start,
            (part.vertices.start + part.vertices.end) / 2,
            part.vertices.end - 1,
        ] {
            for (&joint, weight) in rig.joint_indices[v].iter().zip(rig.joint_weights[v]) {
                if joint > 0 && weight > 0.2 && roles[joint as usize] == Role::Other {
                    roles[joint as usize] = Role::Wing;
                }
            }
        }
    }
    (1..rig.joints.len())
        .filter_map(|joint| {
            let r = roles[joint];
            if !matches!(r, Role::Wing | Role::Arm | Role::Tail) {
                return None;
            }
            let mut parent = rig.joints[joint].parent;
            while let Some(p) = parent {
                if roles[p] == r {
                    return None;
                }
                parent = rig.joints[p].parent;
            }
            let authored_flight = (!matches!(rig.species, Species::Procedural(_))
                || rig.species.name() == "pidgeotto")
                && rig
                    .clip(Clip::Idle)
                    .rotations
                    .get(joint - 1)
                    .is_some_and(|keys| {
                        keys.iter()
                            .any(|q| q.angle_between(rig.joints[joint].bind.rotation) > 0.16)
                    });
            Some(MotionLimb {
                joint,
                role: r,
                authored_flight,
            })
        })
        .collect()
}

impl SpeciesRig {
    /// Tiny prolegs must not turn map-speed travel into a strobing 30 Hz gait.
    /// Slow travel remains distance driven; faster travel uses a readable trot.
    pub(crate) fn locomotion_phase_delta(&self, distance: f32, delta: f32) -> f32 {
        let frequency = if self.crawl_segments.is_empty() {
            4.
        } else {
            2.2
        };
        (distance / self.walk_cycle).min(delta.clamp(0., 0.25) * frequency) * TAU
    }

    /// Distance phase drives feet. Airborne species keep a clock-driven wing
    /// beat at rest, then bank into their travelling flight/glide pose.
    pub(crate) fn sample_locomotion(
        &self,
        seconds: f32,
        phase: f32,
        weight: f32,
        pose: &mut [Transform],
    ) {
        self.sample_into(Clip::Idle, seconds, Playback::Loop, pose)
            .expect("finite locomotion clock");
        let mode = locomotion(self.species.name());
        let height = self.neutral_bounds.1.y - self.neutral_bounds.0.y;
        let stride = phase.sin() * weight;
        for limb in &self.motion_limbs {
            let joint = &self.joints[limb.joint];
            let side = if joint.name.contains("left") { -1. } else { 1. };
            let rotation = match (mode, limb.role) {
                (Locomotion::Walk, Role::Arm) => Quat::from_rotation_x(-stride * side * 0.32),
                (Locomotion::Fly, Role::Wing) if !limb.authored_flight => {
                    let (frequency, amplitude) = flight_beat(self.species.name());
                    Quat::from_rotation_z(side * amplitude * (seconds * TAU * frequency).sin())
                }
                (Locomotion::Fly, Role::Wing) => Quat::from_rotation_z(side * 0.06 * weight),
                (Locomotion::Glide, Role::Wing | Role::Arm) => {
                    Quat::from_rotation_z(side * (0.10 * weight + 0.025 * (seconds * TAU).sin()))
                }
                (_, Role::Tail) => Quat::from_rotation_y(stride * 0.13),
                _ => Quat::IDENTITY,
            };
            pose[limb.joint].rotation = (pose[limb.joint].rotation * rotation).normalize();
        }
        if self.crawl_segments.is_empty() && role(&self.joints[1].name) == Role::Body {
            let rotation = match mode {
                Locomotion::Walk => Quat::from_rotation_z(stride * 0.012),
                Locomotion::Fly | Locomotion::Glide => Quat::from_rotation_x(-0.06 * weight),
                Locomotion::Float => Quat::IDENTITY,
            };
            pose[1].rotation = (pose[1].rotation * rotation).normalize();
        }
        pose[0].translation.y += self.hover_clearance
            + match mode {
                Locomotion::Walk => 0.,
                Locomotion::Fly => {
                    height
                        * (0.18
                            + 0.015 * (seconds * TAU * flight_beat(self.species.name()).0).sin())
                }
                Locomotion::Glide => height * (0.14 + 0.018 * (seconds * TAU).sin()),
                Locomotion::Float => height * (0.16 + 0.012 * (seconds * TAU * 0.7).sin()),
            };
        if mode == Locomotion::Walk && weight > 0. {
            crawl::sample(self, phase, weight, pose);
            gait::sample(self, phase, weight, pose);
        }
    }

    /// Cached world culling bound valid for all composed locomotion poses.
    /// A joint's descendant stays inside its bind-path sphere under rotation.
    pub(crate) fn locomotion_bounds(&self) -> (Vec3, Vec3) {
        let mut radius = 0.0_f32;
        for (i, p) in self.neutral.positions.iter().enumerate() {
            for (&id, w) in self.joint_indices[i].iter().zip(self.joint_weights[i]) {
                if w == 0. {
                    continue;
                }
                let mut j = id as usize;
                let mut r = self.joints[j]
                    .inverse_bind
                    .transform_point3(Vec3::from_array(*p))
                    .length();
                loop {
                    r += self.joints[j].bind.translation.length();
                    if let Some(parent) = self.joints[j].parent {
                        j = parent;
                    } else {
                        break;
                    }
                }
                radius = radius.max(r);
            }
        }
        radius += (self.neutral_bounds.1.y - self.neutral_bounds.0.y) * 0.30 + self.hover_clearance;
        (Vec3::splat(-radius), Vec3::splat(radius))
    }
}

/// A few early skins deliberately left feet on their fixed root. Give those
/// named supports independent hinges for travelling, without changing battle
/// geometry, ownership, clips, or the canonical GLB reader.
pub(crate) fn world_rig(canonical: &'static SpeciesRig) -> &'static SpeciesRig {
    use std::sync::Mutex;
    static RIGS: OnceLock<Mutex<std::collections::HashMap<&'static str, &'static SpeciesRig>>> =
        OnceLock::new();
    let mut rigs = RIGS.get_or_init(Default::default).lock().unwrap();
    rigs.entry(canonical.species.name()).or_insert_with(|| {
        let mut rig = canonical.clone();
        if locomotion(rig.species.name()) == Locomotion::Walk {
            crawl::build(&mut rig);
            gait::build(&mut rig);
        }
        rig.motion_limbs = motion_limbs(&rig);
        weld_weights(&mut rig);
        if locomotion(rig.species.name()) != Locomotion::Walk {
            // Broad bat wings may extend below the feet on the downstroke.
            // Resolve their clearance once from actual poses, not each frame.
            let mut pose = rig.joints.iter().map(|j| j.bind).collect::<Vec<_>>();
            let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
            let mut floor = f32::INFINITY;
            for tick in 0..128 {
                rig.sample_locomotion(tick as f32 / 32., 0., 1., &mut pose);
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                for v in 0..rig.neutral.positions.len() {
                    floor = floor.min(rig.skin_point(v, &matrices).y);
                }
            }
            let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
            rig.hover_clearance = (height * 0.04 - floor).max(0.);
        }
        rig.animated_bounds = rig.locomotion_bounds();
        Box::leak(Box::new(rig))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pokemon_motion_tiny_supports_keep_a_readable_cadence_at_map_speed() {
        for name in ["CATERPIE", "WEEDLE", "PIKACHU", "FURRET"] {
            let rig = world_rig(crate::species_rig::for_species(name).unwrap());
            let mut previous: Option<f32> = None;
            for hz in [9, 30, 60] {
                let dt = 1. / hz as f32;
                let cycles = (0..hz)
                    .map(|_| rig.locomotion_phase_delta(4. * dt, dt) / TAU)
                    .sum::<f32>();
                assert!(cycles <= 4.001, "{name} gait aliases at {hz} Hz");
                if let Some(previous) = previous {
                    assert!((cycles - previous).abs() < 0.00001);
                }
                previous = Some(cycles);
            }
            let slow_distance = rig.walk_cycle * 0.01;
            assert!((rig.locomotion_phase_delta(slow_distance, 0.1) - TAU * 0.01).abs() < 0.00001);
        }
    }

    #[test]
    fn pokemon_motion_cloyster_floats_as_one_complete_shell() {
        let canonical = crate::species_rig::for_species("CLOYSTER").unwrap();
        let rig = world_rig(canonical);
        assert_eq!(locomotion("CLOYSTER"), Locomotion::Float);
        assert!(rig.walk_legs.is_empty());
        assert!(
            rig.joint_indices.iter().all(|j| j[0] == 1),
            "shell spikes, eyes and teeth must follow the mollusk body"
        );
        let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
        let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
        for tick in 0..64 {
            rig.sample_locomotion(tick as f32 / 32., tick as f32, 1., &mut pose);
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            assert!(pose[0].translation.y > 0.1);
            for i in 0..rig.neutral.positions.len() {
                assert!(
                    rig.skin_point(i, &matrices).y > 0.1,
                    "Cloyster should hover clear of the floor"
                );
            }
        }
    }

    #[test]
    fn pokemon_motion_larvae_have_segment_waves_and_independent_prolegs() {
        for name in ["CATERPIE", "WEEDLE"] {
            let canonical = crate::species_rig::for_species(name).unwrap();
            let rig = world_rig(canonical);
            assert_eq!(rig.crawl_segments.len(), 6, "{name}");
            assert_eq!(rig.walk_legs.len(), 8, "{name}");
            let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
            let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
            for tick in 0..64 {
                rig.sample_locomotion(0.2, tick as f32 * TAU / 64., 1., &mut pose);
                gait::assert_supports(rig, &pose);
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                assert!(
                    (0..rig.neutral.positions.len())
                        .all(|v| rig.skin_point(v, &matrices).y >= -0.00001),
                    "{name} crawled through the floor"
                );
                let front = &rig.crawl_segments[0];
                let back = &rig.crawl_segments[5];
                assert!(
                    pose[front.joint].translation.distance(front.center) > 0.0001
                        || pose[back.joint].translation.distance(back.center) > 0.0001
                );
            }
            let mut rest = vec![Transform::IDENTITY; rig.joints.len()];
            rig.sample_locomotion(0., 0., 0., &mut rest);
            rig.skin_matrices_into(&rest, &mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                assert!(
                    rig.skin_point(v, &matrices)
                        .distance(Vec3::from_array(rig.neutral.positions[v]))
                        < 0.00001
                );
            }
        }
    }

    #[test]
    fn pokemon_motion_roles_use_anatomy_instead_of_incidental_substrings() {
        for (name, expected) in [
            ("Body / warm pear-shaped torso with ears nearby", Role::Body),
            ("Dragon pear torso.001", Role::Body),
            ("charmeleon/body", Role::Body),
            ("Head / armoured cranium", Role::Head),
            ("Hind claw / small ivory point", Role::Leg),
            ("Forearm / tapered cuff", Role::Arm),
            ("Rear primary / layered flight vane", Role::Wing),
            ("Rear crown / armor spike", Role::Head),
            ("cloyster/rear_left", Role::Leg),
            ("Tail / winglike fan", Role::Tail),
        ] {
            assert_eq!(role(name), expected, "{name}");
        }
    }

    #[test]
    fn pokemon_motion_covers_entire_roster_and_every_unown_form() {
        let names: Vec<_> = crate::battle_species_models::supported_species().collect();
        assert_eq!(names.len(), 251);
        for name in names.iter().copied().chain(
            (b'A'..=b'Z')
                .map(|c| Box::leak(format!("UNOWN_{}", c as char).into_boxed_str()) as &str),
        ) {
            let rig =
                crate::species_rig::for_species(name).unwrap_or_else(|| panic!("missing {name}"));
            let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
            let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
            rig.sample_into(Clip::Idle, 0., Playback::Clamp, &mut pose)
                .unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for i in 0..rig.neutral.positions.len() {
                assert!(
                    rig.skin_point(i, &matrices)
                        .distance(Vec3::from_array(rig.neutral.positions[i]))
                        < 0.00001,
                    "{name} bind vertex {i}"
                );
            }
            for clip in [Clip::Attack, Clip::Hit] {
                rig.sample_into(
                    clip,
                    rig.clip(clip).duration * 0.4,
                    Playback::Clamp,
                    &mut pose,
                )
                .unwrap();
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                assert!(
                    (0..rig.neutral.positions.len()).any(|i| rig
                        .skin_point(i, &matrices)
                        .distance(Vec3::from_array(rig.neutral.positions[i]))
                        > 0.0001),
                    "{name} {clip:?} must move geometry"
                );
            }
            let rig = world_rig(rig);
            pose.resize(rig.joints.len(), Transform::IDENTITY);
            let allocation = pose.as_ptr();
            for phase in [0., 1., 2., 4., 6.] {
                rig.sample_locomotion(0.37, phase, 1., &mut pose);
                assert_eq!(allocation, pose.as_ptr());
                assert!(
                    pose.iter().all(|p| p.translation.is_finite()
                        && p.rotation.is_finite()
                        && p.scale == Vec3::ONE),
                    "{name}"
                );
            }
        }
        assert!(crate::species_rig::for_species("MISSINGNO").is_none());
    }

    #[test]
    fn pokemon_motion_selects_walk_flight_and_glide_with_live_support_joints() {
        for (name, mode) in [
            ("PIKACHU", Locomotion::Walk),
            ("LUGIA", Locomotion::Fly),
            ("GLIGAR", Locomotion::Glide),
        ] {
            let rig = world_rig(crate::species_rig::for_species(name).unwrap());
            assert_eq!(locomotion(name), mode);
            let mut a = vec![Transform::IDENTITY; rig.joints.len()];
            let mut b = a.clone();
            rig.sample_locomotion(0.2, 0., 0., &mut a);
            rig.sample_locomotion(0.2, 1.3, 1., &mut b);
            assert!(
                a.iter()
                    .zip(&b)
                    .skip(1)
                    .any(|(a, b)| a.rotation.angle_between(b.rotation) > 0.01)
            );
            if mode == Locomotion::Walk {
                assert!(
                    !rig.walk_legs.is_empty(),
                    "{name} needs independent supports"
                );
                gait::assert_supports(rig, &b);
            } else {
                assert!(b[0].translation.y > 0.);
            }
            let (lo, hi) = rig.locomotion_bounds();
            let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
            for tick in 0..48 {
                rig.sample_locomotion(tick as f32 / 48., tick as f32 * TAU / 48., 1., &mut b);
                rig.skin_matrices_into(&b, &mut matrices).unwrap();
                for i in 0..rig.neutral.positions.len() {
                    let p = rig.skin_point(i, &matrices);
                    assert!(
                        p.cmpge(lo).all() && p.cmple(hi).all(),
                        "{name} culling bound"
                    );
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "pokemon_motion_quality.rs"]
mod quality;

#[path = "pokemon_gait.rs"]
mod gait;
pub(crate) use gait::WalkLeg;

#[path = "pokemon_crawl.rs"]
mod crawl;
pub(crate) use crawl::CrawlSegment;
