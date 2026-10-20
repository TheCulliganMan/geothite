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

/// Intrinsic aquatic presentation for living map actors. Surf mounts select
/// water motion explicitly, including species that normally travel on land.
pub(crate) fn swims_in_water(name: &str) -> bool {
    matches!(travel::style(name), travel::Style::Swim | travel::Style::Flop)
        || matches!(name, "mantine" | "tentacool" | "tentacruel" | "shellder" | "cloyster"
            | "staryu" | "starmie" | "squirtle" | "wartortle" | "blastoise" | "psyduck"
            | "golduck" | "poliwag" | "poliwhirl" | "poliwrath" | "politoed" | "slowpoke"
            | "slowbro" | "slowking" | "lapras" | "seel" | "dewgong" | "vaporeon" | "omanyte" | "omastar"
            | "kabuto" | "kabutops" | "totodile" | "croconaw" | "feraligatr" | "marill"
            | "azumarill" | "wooper" | "quagsire" | "corsola" | "octillery")
}

/// These authored tendrils support the bell instead of hovering above land.
fn rests_on_tentacles(name: &str) -> bool {
    name.eq_ignore_ascii_case("tentacool") || name.eq_ignore_ascii_case("tentacruel")
}

/// Swimming locomotion does not imply hovering during a battle. Lapras's
/// authored belly is seated; its flippers and neck supply the swimming motion.
pub(crate) fn battle_hovers(name: &str) -> bool {
    locomotion(name) == Locomotion::Float && !name.eq_ignore_ascii_case("lapras") && !rests_on_tentacles(name)
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
        "shank", "haunch", "hip", "hips", "greave", "greaves", "hock", "toe", "toes", "talon", "ankle", "knee", "calf", "paw", "shoe",
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
            | "furret"
            | "ivysaur"
            | "venusaur"
            | "rattata"
            | "vulpix"
            | "ninetales"
            | "nidoran_f"
            | "nidoran_m"
            | "nidorino"
            | "tauros"
            | "mareep"
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
        flop_folds: Vec::new(),
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
    let mut groups: HashMap<([i32; 3], u16), Vec<usize>> = HashMap::new();
    for (i, p) in rig.neutral.positions.iter().enumerate() {
        groups
            .entry((p.map(|v| (v / epsilon).round() as i32), ladybird_flight::weld_domain(rig, i)))
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



#[derive(Clone, Debug)]
pub(crate) struct MotionLimb {
    joint: usize,
    role: Role,
    authored_flight: bool,
    levelled_flight: Option<(Quat, Vec3, usize)>,
}

fn flight_beat(name: &str) -> (f32, f32) {
    match name {
        "lugia" | "ho_oh" | "articuno" | "zapdos" | "moltres" => (0.85, 0.36),
        "charizard" | "dragonite" | "aerodactyl" => (1.25, 0.40),
        "butterfree" | "beedrill" | "venomoth" | "yanma" | "celebi" | "togetic" | "ledyba" | "ledian" => (6., 0.45),
        "zubat" | "golbat" | "crobat" => (3.5, 0.45),
        _ => (2.5, 0.38),
    }
}

fn levelled_flight_wing(rig: &SpeciesRig, joint: usize) -> Option<(Quat, Vec3, usize)> {
    if !matches!(rig.species.name(), "pidgey" | "pidgeotto" | "pidgeot" | "spearow"
        | "fearow" | "noctowl" | "murkrow" | "skarmory" | "articuno" | "zapdos"
        | "moltres" | "ho_oh" | "lugia" | "togetic"
        | "aerodactyl" | "charizard" | "dragonite" | "ledyba" | "ledian") { return None; }
    let pivot = rig.joints[joint].inverse_bind.inverse().transform_point3(Vec3::ZERO);
    let belongs = |mut owner: usize| {
        loop {
            if owner == joint { return true; }
            let Some(parent) = rig.joints[owner].parent else { return false; };
            owner = parent;
        }
    };
    let tip = (0..rig.neutral.positions.len()).filter(|&v| {
        rig.joint_indices[v].iter().zip(rig.joint_weights[v])
            .any(|(&owner, weight)| weight > 0.2 && belongs(owner as usize))
    }).max_by(|&a, &b| (Vec3::from_array(rig.neutral.positions[a]) - pivot).length_squared()
        .total_cmp(&(Vec3::from_array(rig.neutral.positions[b]) - pivot).length_squared()))?;
    let span = Vec3::from_array(rig.neutral.positions[tip]) - pivot;
    let side = if rig.joints[joint].name.contains("left") { -1. } else { 1. };
    // Folded wings can point down and backward. Level their full span in model
    // space, then express that correction in the shoulder parent's frame.
    let parent_rotation = rig.joints[joint].parent.map(|parent|
        rig.joints[parent].inverse_bind.inverse().to_scale_rotation_translation().1
    ).unwrap_or(Quat::IDENTITY);
    let level = Quat::from_rotation_arc(span.normalize(), Vec3::X * side);
    Some((parent_rotation.inverse() * level * parent_rotation,
        parent_rotation.inverse() * Vec3::Z, tip))
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
            let authored_flight = (!matches!(rig.species, Species::Procedural(_)))
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
                levelled_flight: (r == Role::Wing).then(|| levelled_flight_wing(rig, joint)).flatten(),
            })
        })
        .collect()
}

impl SpeciesRig {
    /// Tiny prolegs must not turn map-speed travel into a strobing 30 Hz gait.
    /// Slow travel remains distance driven; faster travel uses a readable trot.
    pub(crate) fn locomotion_phase_delta(&self, distance: f32, delta: f32) -> f32 {
        let frequency = travel::style(self.species.name()).frequency();
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
                (Locomotion::Walk, Role::Arm) => match travel::style(self.species.name()) {
                    travel::Style::Scuttle => Quat::IDENTITY,
                    travel::Style::Hop => Quat::from_rotation_x(-stride * 0.18),
                    _ => Quat::from_rotation_x(-stride * side * 0.32),
                },
                (Locomotion::Fly, Role::Wing) if limb.levelled_flight.is_some() => {
                    // A raised or folded battle wing is not a complete flight
                    // stroke. Centre the world wing on its measured span, then
                    // carry it above and below the shoulder each wingbeat.
                    let (level, axis, _) = limb.levelled_flight.unwrap();
                    let flap = Quat::from_axis_angle(axis, side * 0.65
                        * (seconds * TAU * flight_beat(self.species.name()).0).sin());
                    pose[limb.joint].rotation = flap * level * joint.bind.rotation;
                    Quat::IDENTITY
                }
                (Locomotion::Fly, Role::Wing) if !limb.authored_flight => {
                    let (frequency, amplitude) = flight_beat(self.species.name());
                    Quat::from_rotation_z(side * amplitude * (seconds * TAU * frequency).sin())
                }
                (Locomotion::Fly, Role::Wing) => Quat::from_rotation_z(side * 0.06 * weight),
                (Locomotion::Glide, Role::Wing | Role::Arm) => {
                    Quat::from_rotation_z(side * (0.10 * weight + 0.025 * (seconds * TAU).sin()))
                }
                (_, Role::Tail) if travel::style(self.species.name()) == travel::Style::Flop => Quat::IDENTITY,
                (_, Role::Tail) => Quat::from_rotation_y(stride * 0.13),
                _ => Quat::IDENTITY,
            };
            pose[limb.joint].rotation = (pose[limb.joint].rotation * rotation).normalize();
        }
        ladybird_flight::open_cases(self, pose);
        if self.crawl_segments.is_empty() && role(&self.joints[1].name) == Role::Body {
            let rotation = match mode {
                Locomotion::Walk => Quat::from_rotation_z(stride * 0.012),
                Locomotion::Fly | Locomotion::Glide => Quat::from_rotation_x(-0.06 * weight),
                Locomotion::Float => Quat::IDENTITY,
            };
            pose[1].rotation = (pose[1].rotation * rotation).normalize();
        }
        if travel::style(self.species.name()) == travel::Style::Flop {
            travel::flop(self, phase, weight, pose);
            return;
        }
        if self.species.name() == "lapras" {
            travel::prepare(self,phase,weight,pose);
            travel::finish(self,phase,weight,pose);
            return;
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
                Locomotion::Float if rests_on_tentacles(self.species.name()) => 0.,
                Locomotion::Float => height * (0.16 + 0.012 * (seconds * TAU * 0.7).sin()),
            };
        // Rolling/spinning orientation belongs to accumulated travel, so it
        // freezes at rest rather than snapping the face back upright.
        let turns = matches!(travel::style(self.species.name()), travel::Style::Roll | travel::Style::Spin);
        if mode == Locomotion::Walk && (weight > 0. || turns) {
            travel::prepare(self, phase, weight, pose);
            crawl::sample(self, phase, weight, pose);
            gait::sample(self, phase, weight, pose);
            travel::finish(self, phase, weight, pose);
            gait::settle(self, pose);
        } else if mode == Locomotion::Float && weight > 0. {
            travel::swim(self, phase, weight, pose);
        }
    }

    /// Water travel deliberately bypasses ground planting and land flops.
    /// The actor anchor supplies the waterline; no hover clearance is added.
    pub(crate) fn sample_swimming(
        &self, seconds: f32, phase: f32, weight: f32, pose: &mut [Transform],
    ) {
        self.sample_into(Clip::Idle, seconds, Playback::Loop, pose)
            .expect("finite swimming clock");
        travel::swim_strokes(self, phase, weight.clamp(0., 1.), pose);
        travel::water_tentacles(self, phase, weight.clamp(0., 1.), pose);
        if matches!(self.species.name(), "staryu" | "starmie") {
            // A surface-swimming star presents its broad face upward. Centre
            // the horizontal body at its actor anchor instead of leaving its
            // former upright floor origin at one point of the star.
            let rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
            let middle = (self.neutral_bounds.0 + self.neutral_bounds.1) * 0.5;
            pose[0].rotation = (rotation * pose[0].rotation).normalize();
            pose[0].translation += Vec3::new(middle.x, 0., middle.z) - rotation * middle;
        }
        let height = self.neutral_bounds.1.y - self.neutral_bounds.0.y;
        pose[0].translation.y += height * 0.012 * (seconds * TAU * 0.7).sin();
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
        furret_walk::build(&mut rig);
        if locomotion(rig.species.name()) == Locomotion::Walk {
            crawl::build(&mut rig);
            gait::build(&mut rig);
            travel::build(&mut rig);
        } else if travel::style(rig.species.name()) == travel::Style::Flop
            || matches!(rig.species.name(), "gyarados" | "lapras") {
            travel::build(&mut rig);
        }
        ladybird_flight::build(&mut rig);
        rig.motion_limbs = motion_limbs(&rig);
        weld_weights(&mut rig);
        if locomotion(rig.species.name()) != Locomotion::Walk
            && travel::style(rig.species.name()) != travel::Style::Flop
            && !rests_on_tentacles(rig.species.name()) && rig.species.name() != "lapras" {
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
            // Keep a visible air gap even at the lowest wingtip, rather than
            // merely avoiding intersection with the ground.
            let gap = if locomotion(rig.species.name()) == Locomotion::Fly { 0.16 } else { 0.04 };
            rig.hover_clearance = (height * gap - floor).max(0.);
        }
        rig.animated_bounds = rig.locomotion_bounds();
        Box::leak(Box::new(rig))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lapras_land_belly_drag_stays_grounded_and_keeps_water_motion_separate() {
        let canonical=crate::species_rig::for_species("LAPRAS").unwrap();
        let rig=world_rig(canonical);
        assert_eq!(rig.neutral.positions,canonical.neutral.positions);
        assert_eq!(rig.neutral.colors,canonical.neutral.colors);
        assert_eq!(rig.neutral.indices,canonical.neutral.indices);
        assert_eq!(travel::style("lapras"),travel::Style::Slide);
        assert!(swims_in_water("lapras"));
        assert!(!battle_hovers("lapras"));
        let belly=rig.anatomy.iter().find(|part|part.name.starts_with("Belly /")).unwrap();
        let flipper=rig.anatomy.iter().find(|part|part.name.starts_with("Fore flipper /")).unwrap();
        let height=rig.neutral_bounds.1.y-rig.neutral_bounds.0.y;
        let mut pose=vec![Transform::IDENTITY;rig.joints.len()];
        let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];
        rig.sample_locomotion(0.2,0.,1.,&mut pose);
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        let before:Vec<_>=flipper.vertices.clone().map(|v|rig.skin_point(v,&matrices)).collect();
        let mut pull=0.0_f32;
        for tick in 0..64 {
            rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            let contact=belly.vertices.clone().map(|v|rig.skin_point(v,&matrices).y)
                .fold(f32::INFINITY,f32::min);
            assert!(contact.abs()<0.00001,"Lapras belly left the path: {contact}");
            for v in 0..rig.neutral.positions.len() {
                assert!(rig.skin_point(v,&matrices).y>=-0.00001,"Lapras penetrated the path");
            }
            for (v,old) in flipper.vertices.clone().zip(&before) {
                pull=pull.max(rig.skin_point(v,&matrices).distance(*old));
            }
        }
        assert!(pull>height*0.025,"Lapras must pull with its complete foreflippers");
        rig.sample_locomotion(0.2,1.,0.,&mut pose);
        let stopped=pose.clone();
        rig.sample_locomotion(0.2,4.,0.,&mut pose);
        assert_eq!(pose,stopped,"stopped land pull retained phase motion");
        rig.sample_swimming(0.2,1.,1.,&mut pose);
        assert!(pose[0].translation.y.abs()<=height*0.012+0.000001,
            "water retained the land correction or hover lift");
        assert!(pose.iter().zip(&stopped).any(|(a,b)|!a.rotation.abs_diff_eq(b.rotation,0.01)),
            "water must retain its separate strokes");
    }

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
    fn pokemon_motion_crabs_use_four_complete_walking_limbs() {
        for name in ["KRABBY","KINGLER"] {
            let rig=world_rig(crate::species_rig::for_species(name).unwrap());
            assert_eq!(rig.walk_legs.len(),4,"{name} split a knee or toe into an extra leg");
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];
            let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];
            for tick in 0..64 {
                rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);
                gait::assert_supports(rig,&pose);
                rig.skin_matrices_into(&pose,&mut matrices).unwrap();
                let floor=(0..rig.neutral.positions.len()).map(|v|rig.skin_point(v,&matrices).y).fold(f32::INFINITY,f32::min);
                assert!(floor>=-0.00001 && floor<0.005,"{name} lost ground contact: {floor}");
            }
        }
    }

    #[test]
    fn pokemon_motion_fighting_family_keeps_two_complete_weight_bearing_legs() {
        for (name,legs) in [("MACHOP",2),("MACHOKE",2),("MACHAMP",2)] {
            let rig=world_rig(crate::species_rig::for_species(name).unwrap());assert_eq!(locomotion(name),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),legs,"{name} split or combined a complete limb");
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {
                rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut low=f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {
                    let p=rig.skin_point(v,&matrices);low=low.min(p.y);assert!(p.y>=-0.00001,"{name} walks through the floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all(),"{name} left locomotion culling bounds");
                }
                assert!(low<0.005,"{name} lost ground contact: {low}");
            }
        }
    }

    #[test]
    fn pokemon_motion_fire_horses_keep_four_complete_legs_and_bound_flame_banks() {
        for (name,legs) in [("PONYTA",4),("RAPIDASH",4)] {
            let rig=world_rig(crate::species_rig::for_species(name).unwrap());assert_eq!(locomotion(name),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),legs,"{name} split or combined a complete limb");
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {
                rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut low=f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {
                    let p=rig.skin_point(v,&matrices);low=low.min(p.y);assert!(p.y>=-0.00001,"{name} walks through the floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all(),"{name} left locomotion culling bounds");
                }
                assert!(low<0.005,"{name} lost ground contact: {low}");
            }
        }
    }

    #[test]
    fn kangaskhan_skin_walks_on_two_legs_without_swinging_baby_rim_grips() {
        for (name,legs) in [("KANGASKHAN",2)] {
            let rig=world_rig(crate::species_rig::for_species(name).unwrap());assert_eq!(locomotion(name),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),legs,"{name} split or combined a complete limb");
            for joint in [16,17] {assert_eq!(role(&rig.joints[joint].name),Role::Other);assert!(!rig.motion_limbs.iter().any(|limb|limb.joint==joint));}
            let canonical=crate::species_rig::for_species(name).unwrap();
            for part in canonical.anatomy.iter().filter(|p|p.name.starts_with("Baby ")) {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v]);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}}
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {
                rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut low=f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {
                    let p=rig.skin_point(v,&matrices);low=low.min(p.y);assert!(p.y>=-0.00001,"{name} walks through the floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all(),"{name} left locomotion culling bounds");
                }
                travel::assert_contact(rig, tick as f32*TAU/64., low);
            }
        }
    }

    #[test]
    fn kadabra_skin_walks_on_two_complete_feet_and_preserves_psychic_props() {
        for (name,legs) in [("KADABRA",2)] {
            let rig=world_rig(crate::species_rig::for_species(name).unwrap());assert_eq!(locomotion(name),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),legs,"{name} split or combined a complete limb");
            let canonical=crate::species_rig::for_species(name).unwrap();
            for part in canonical.anatomy.iter().filter(|p|p.name.starts_with("Held silver spoon") || p.name.starts_with("Hollow silver spoon") || p.name.starts_with("Long swept moustache") || p.name.starts_with("Heavy curved psychic tail") || p.name.starts_with("Broad tail saddle band")) {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v]);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}}
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {
                rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut low=f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {
                    let p=rig.skin_point(v,&matrices);low=low.min(p.y);assert!(p.y>=-0.00001,"{name} walks through the floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all(),"{name} left locomotion culling bounds");
                }
                assert!(low<0.005,"{name} lost ground contact: {low}");
            }
        }
    }

    #[test]
    fn psychic_fox_skin_walks_on_two_feet_and_preserves_props_and_tail_weights() {
        for (name,legs) in [("ABRA",2),("ALAKAZAM",2)] {
            let rig=world_rig(crate::species_rig::for_species(name).unwrap());assert_eq!(locomotion(name),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),legs,"{name} split or combined a complete limb");
            let canonical=crate::species_rig::for_species(name).unwrap();
            for part in canonical.anatomy.iter().filter(|p|p.name.starts_with("Spoon /") || p.name.starts_with("Moustache /") || p.name.starts_with("Tail /")) {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v]);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}}
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {
                rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut low=f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {
                    let p=rig.skin_point(v,&matrices);low=low.min(p.y);assert!(p.y>=-0.00001,"{name} walks through the floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all(),"{name} left locomotion culling bounds");
                }
                assert!(low<0.005,"{name} lost ground contact: {low}");
            }
        }
    }

    #[test]
    fn bilateral_support_details_follow_their_own_walking_leg_and_keep_rigid_feet() {
        for (name,prefixes,foot_prefix) in [
            ("LICKITUNG",&["Thigh /","Thigh ring /","Thigh center /","Foot /","Toe /"][..],"Foot /"),
            ("MR__MIME",&["Hip /","Thigh /","Knee /","Shin /","Shoe /","Shoe curl /"][..],"Shoe /"),
            ("SCIZOR",&["Hip armor /","Thigh /","Greave /","Foot /","Toe cap /"][..],"Foot /"),
            ("SCYTHER",&["Haunch /","Haunch thorn /","Shank /","Greave /","Foot /","Foot claw /"][..],"Foot /"),
        ] {
            let canonical=crate::species_rig::for_species(name).unwrap();let rig=world_rig(canonical);
            assert_eq!(locomotion(name),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),2);
            for side in [-1,1] {
                let suffix=format!(" {side}");let foot=rig.anatomy.iter().find(|p|p.name.starts_with(foot_prefix) && p.name.ends_with(&suffix)).unwrap();let owner=rig.joint_indices[foot.vertices.start][0];
                for prefix in prefixes {
                    let parts=rig.anatomy.iter().filter(|p|p.name.starts_with(prefix) && p.name.ends_with(&suffix)).collect::<Vec<_>>();
                    assert_eq!(parts.len(),1,"{name} {prefix}{suffix} missing or combined");
                    for part in parts {for v in part.vertices.clone() {
                        for (&joint,weight) in rig.joint_indices[v].iter().zip(rig.joint_weights[v]) {if weight>0. {assert!((owner-2..=owner).contains(&joint),"{name} {} follows another leg",part.name);}}
                        if ["Foot /","Toe /","Toe cap /","Foot claw /","Shoe /","Shoe curl /"].iter().any(|p|part.name.starts_with(p)) {assert_eq!(rig.joint_indices[v][0],owner);assert_eq!(rig.joint_weights[v][0],1.);}
                    }}
                }
            }
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut low=f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);low=low.min(p.y);assert!(p.y>=-0.00001,"{name} walks through floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all());}assert!(low<0.005,"{name} lost ground contact: {low}");}
        }
    }

    #[test]
    fn lickitung_skin_walks_on_two_complete_feet_without_rebinding_tongue_tail_or_face() {
        let canonical=crate::species_rig::for_species("LICKITUNG").unwrap();let rig=world_rig(canonical);
        assert_eq!(locomotion("LICKITUNG"),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),2);
        for part in canonical.anatomy.iter().filter(|p|["Head /","Mouth lining /","Eye /","Eye glint /","Nostril /","Tongue /","Tongue groove /","Tail /","Arm /","Hand tip /"].iter().any(|prefix|p.name.starts_with(prefix))) {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v]);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}}
        let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
        for tick in 0..64 {rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut low=f32::INFINITY;
            for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);low=low.min(p.y);assert!(p.y>=-0.00001,"Lickitung walks through floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all());}assert!(low<0.005,"Lickitung lost ground contact: {low}");}
    }

    #[test]
    fn mr_mime_skin_walks_on_two_complete_slippers_without_rebinding_gloves_or_face() {
        let canonical=crate::species_rig::for_species("MR__MIME").unwrap();let rig=world_rig(canonical);
        assert_eq!(locomotion("MR__MIME"),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),2);
        for part in canonical.anatomy.iter().filter(|p|["Head /","Mouth lining /","Eye /","Pupil /","Eye glint /","Cheek /","Upper lip /","Lower lip /","Mouth back /","Tongue /","Ear fan /","Arm /","Palm /","Finger /","Fingertip /","Finger pad /"].iter().any(|prefix|p.name.starts_with(prefix))) {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v]);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}}
        let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
        for tick in 0..64 {rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut low=f32::INFINITY;
            for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);low=low.min(p.y);assert!(p.y>=-0.00001,"Mr Mime walks through floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all());}assert!(low<0.005,"Mr Mime lost ground contact: {low}");}
    }

    #[test]
    fn slow_pair_skin_walks_on_two_feet_without_rebinding_shells_crowns_and_collars() {
        for name in ["SLOWBRO","SLOWKING"] {
            let canonical=crate::species_rig::for_species(name).unwrap();let rig=world_rig(canonical);
            assert_eq!(locomotion(name),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),2);
            for part in canonical.anatomy.iter().filter(|p|["Head /","Muzzle /","Jaw /","Tongue /","Fang /","Tooth /","Arm /","Hand /","Hand claw /","Shell ","Crown ","Jewel","Collar panel /","Ear /","Tail /"].iter().any(|prefix|p.name.starts_with(prefix))) {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v]);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}}
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut low=f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);low=low.min(p.y);assert!(p.y>=-0.00001,"{name} walks through floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all());}assert!(low<0.005,"{name} lost ground contact: {low}");}
        }
    }

    #[test]
    fn fire_canine_skin_walks_on_four_complete_paws_and_preserves_face_fur_and_tail() {
        for name in ["GROWLITHE","ARCANINE"] {
            let canonical=crate::species_rig::for_species(name).unwrap();let rig=world_rig(canonical);
            assert_eq!(locomotion(name),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),4);
            for part in canonical.anatomy.iter().filter(|p|["Head /","Muzzle /","Jaw /","Nose /","Lip /","Eye socket /","Eye /","Pupil /","Eye glint /","Forehead stripe /","Ear /","Ear inset /","Body /","Flank stripe /","Back stripe /","Mane /","Mane tuft /","Forelock /","Tail /","Tail tuft /"].iter().any(|prefix|p.name.starts_with(prefix))) {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v]);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}}
            for side in [-1,1] {for row in [0,1] {
                let suffix=format!(" {side} {row}");
                let paw=rig.anatomy.iter().find(|p|p.name.starts_with("Paw /") && p.name.ends_with(&suffix)).unwrap();
                let owner=rig.joint_indices[paw.vertices.start][0];
                for prefix in ["Leg /","Paw /","Paw crease /","Leg stripe /","Ankle fur /","Ankle tuft /"] {
                    let parts=rig.anatomy.iter().filter(|p|p.name.starts_with(prefix) && p.name.ends_with(&suffix)).collect::<Vec<_>>();
                    assert_eq!(parts.len(),if name=="GROWLITHE" && prefix.starts_with("Ankle") {0} else {1},"{name} {prefix}{suffix} missing or combined");
                    for part in parts {for v in part.vertices.clone() {
                        for (&joint,weight) in rig.joint_indices[v].iter().zip(rig.joint_weights[v]) {if weight>0. {assert!((owner-2..=owner).contains(&joint),"{name} {} follows another leg",part.name);}}
                        if ["Paw /","Paw crease /"].iter().any(|p|part.name.starts_with(p)) {assert_eq!(rig.joint_indices[v][0],owner);assert_eq!(rig.joint_weights[v][0],1.);}
                    }}
                }
            }}
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut low=f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);low=low.min(p.y);assert!(p.y>=-0.00001,"{name} walks through floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all());}travel::assert_contact(rig, tick as f32*TAU/64., low);}
        }
    }

    #[test]
    fn nidoran_skin_walks_on_four_complete_paws_and_preserves_face_ears_spots_and_tail() {
        for name in ["NIDORAN_F","NIDORAN_M"] {
            let canonical=crate::species_rig::for_species(name).unwrap();let rig=world_rig(canonical);
            assert_eq!(locomotion(name),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),4);
            for part in canonical.anatomy.iter().filter(|p|["Head /","Muzzle /","Horn /","Mouth /","Eye socket /","Eye /","Iris /","Pupil /","Glint /","Nostril /","Incisor /","Cheek barb /","Ear /","Ear inset /","Ear fold /","Body /","Flank spot /","Back spine /","Tail /"].iter().any(|prefix|p.name.starts_with(prefix))) {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v]);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}}
            // Keep every complete toe set and fitted haunch spot in its own leg's IK tree.
            for side in [-1,1] {for placement in ["front","rear"] {
                let suffix=format!(" {side} {placement}");
                let foot=rig.anatomy.iter().find(|p|p.name.starts_with("Foot /") && p.name.ends_with(&suffix)).unwrap();
                let owner=rig.joint_indices[foot.vertices.start][0];
                for part in rig.anatomy.iter().filter(|p|p.name.ends_with(&suffix) && ["Foot /","Claw /","Leg /","Haunch /","Haunch spot /"].iter().any(|prefix|p.name.starts_with(prefix))) {
                    for v in part.vertices.clone() {
                        for (&joint,weight) in rig.joint_indices[v].iter().zip(rig.joint_weights[v]) {if weight>0. {assert!((owner-2..=owner).contains(&joint),"{name} {} follows another leg",part.name);}}
                        if part.name.starts_with("Claw /") {assert_eq!(rig.joint_indices[v][0],owner);assert_eq!(rig.joint_weights[v][0],1.);}
                    }
                }
                assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Claw /") && p.name.ends_with(&suffix)).count(),1);
            }}
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut low=f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);low=low.min(p.y);assert!(p.y>=-0.00001,"{name} walks through floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all());}travel::assert_contact(rig, tick as f32*TAU/64., low);}
        }
    }

    #[test]
    fn nido_middle_skin_walks_on_anatomical_supports_with_local_marks_and_claws() {
        for (name,legs) in [("NIDORINA",2),("NIDORINO",4)] {
            let canonical=crate::species_rig::for_species(name).unwrap();let rig=world_rig(canonical);assert_eq!(locomotion(name),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),legs);assert_eq!(quadruped(canonical.species.name()),legs==4);
            for part in canonical.anatomy.iter().filter(|p|["Head /","Muzzle /","Jaw /","Horn /","Eye socket /","Eye /","Iris /","Pupil /","Glint /","Nostril /","Fang /","Head spot /","Ear /","Ear inset /","Ear fold /","Body /","Breast /","Flank spot /","Back spine /","Arm /","Hand /","Hand claw /","Tail /"].iter().any(|prefix|p.name.starts_with(prefix))) {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v]);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}}
            for side in [-1,1] {for placement in if legs==4 {vec!["front","rear"]} else {vec![""]} {
                let suffix=if legs==4 {format!(" {side} {placement}")} else {format!(" {side}")};let foot=rig.anatomy.iter().find(|p|p.name.starts_with("Foot /") && p.name.ends_with(&suffix)).unwrap();let owner=rig.joint_indices[foot.vertices.start][0];
                let prefixes=if legs==4 {vec!["Haunch /","Leg /","Leg spot /","Foot /","Claw /"]} else {vec!["Haunch /","Haunch spot /","Leg /","Foot /","Foot claw /"]};
                for prefix in prefixes {let parts=rig.anatomy.iter().filter(|p|p.name.starts_with(prefix) && p.name.ends_with(&suffix)).collect::<Vec<_>>();assert_eq!(parts.len(),1,"{name} {prefix}{suffix} missing or combined");for part in parts {for v in part.vertices.clone() {for (&joint,weight) in rig.joint_indices[v].iter().zip(rig.joint_weights[v]) {if weight>0. {assert!((owner-2..=owner).contains(&joint),"{name} {} follows another leg",part.name);}}if ["Foot /","Foot claw /","Claw /"].iter().any(|p|part.name.starts_with(p)) {assert_eq!(rig.joint_indices[v][0],owner);assert_eq!(rig.joint_weights[v][0],1.);}}}}
            }}
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut low=f32::INFINITY;for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);low=low.min(p.y);assert!(p.y>=-0.00001,"{name} walks through floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all());}travel::assert_contact(rig, tick as f32*TAU/64., low);}
        }
    }

    #[test]
    fn nido_royal_skin_walks_on_anatomical_supports_with_local_marks_and_claws() {
        for (name,legs) in [("NIDOQUEEN",2),("NIDOKING",2)] {
            let canonical=crate::species_rig::for_species(name).unwrap();let rig=world_rig(canonical);assert_eq!(locomotion(name),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),legs);assert_eq!(quadruped(canonical.species.name()),legs==4);
            for part in canonical.anatomy.iter().filter(|p|["Head /","Muzzle /","Jaw /","Horn /","Eye socket /","Eye /","Iris /","Pupil /","Glint /","Nostril /","Fang /","Head spot /","Ear /","Ear inset /","Ear fold /","Body /","Chest plate /","Belly plate /","Back spine /","Arm /","Hand /","Hand claw /","Tail /","Tail collar /","Tail spine /","Shoulder /","Tongue /","Cheek barb /","Chin /","Mouth /"].iter().any(|prefix|p.name.starts_with(prefix))) {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v]);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}}
            for side in [-1,1] {for placement in if legs==4 {vec!["front","rear"]} else {vec![""]} {
                let suffix=if legs==4 {format!(" {side} {placement}")} else {format!(" {side}")};let foot=rig.anatomy.iter().find(|p|p.name.starts_with("Foot /") && p.name.ends_with(&suffix)).unwrap();let owner=rig.joint_indices[foot.vertices.start][0];
                let prefixes=if legs==4 {vec!["Haunch /","Leg /","Leg spot /","Foot /","Claw /"]} else {vec!["Haunch /","Knee /","Leg /","Foot /","Foot claw /"]};
                for prefix in prefixes {let parts=rig.anatomy.iter().filter(|p|p.name.starts_with(prefix) && p.name.ends_with(&suffix)).collect::<Vec<_>>();assert_eq!(parts.len(),1,"{name} {prefix}{suffix} missing or combined");for part in parts {for v in part.vertices.clone() {for (&joint,weight) in rig.joint_indices[v].iter().zip(rig.joint_weights[v]) {if weight>0. {assert!((owner-2..=owner).contains(&joint),"{name} {} follows another leg",part.name);}}if ["Foot /","Foot claw /","Claw /"].iter().any(|p|part.name.starts_with(p)) {assert_eq!(rig.joint_indices[v][0],owner);assert_eq!(rig.joint_weights[v][0],1.);}}}}
            }}
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut low=f32::INFINITY;for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);low=low.min(p.y);assert!(p.y>=-0.00001,"{name} walks through floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all());}assert!(low<0.005,"{name} lost ground contact: {low}");}
        }
    }

    #[test]
    fn fairy_pair_skin_walks_on_complete_feet_without_rebinding_fitted_features() {
        for name in ["CLEFABLE", "WIGGLYTUFF"] {
            let canonical = crate::species_rig::for_species(name).unwrap();
            let rig = world_rig(canonical);
            assert_eq!(locomotion(name), Locomotion::Walk);
            assert_eq!(rig.walk_legs.len(), 2);
            assert!(!quadruped(canonical.species.name()));
            for part in canonical.anatomy.iter().filter(|p| !["Haunch /", "Leg /", "Foot /", "Toe /"].iter().any(|prefix| p.name.starts_with(prefix))) {
                for v in part.vertices.clone() {
                    assert_eq!(rig.joint_indices[v], canonical.joint_indices[v], "{name} {}", part.name);
                    assert_eq!(rig.joint_weights[v], canonical.joint_weights[v], "{name} {}", part.name);
                }
            }
            for side in [-1, 1] {
                let suffix = format!(" {side}");
                let foot = rig.anatomy.iter().find(|p| p.name.starts_with("Foot /") && p.name.ends_with(&suffix)).unwrap();
                let owner = rig.joint_indices[foot.vertices.start][0];
                for prefix in if name == "CLEFABLE" {vec!["Foot /", "Toe /"]} else {vec!["Foot /"]} {
                    let parts = rig.anatomy.iter().filter(|p| p.name.starts_with(prefix) && p.name.ends_with(&suffix)).collect::<Vec<_>>();
                    assert_eq!(parts.len(), 1, "{name} {prefix}{suffix} missing or combined");
                    for part in parts { for v in part.vertices.clone() {
                        for (&joint, weight) in rig.joint_indices[v].iter().zip(rig.joint_weights[v]) {
                            if weight > 0. { assert!((owner-2..=owner).contains(&joint), "{name} {} follows another leg", part.name); }
                        }
                        if ["Foot /", "Toe /"].iter().any(|p| part.name.starts_with(p)) {
                            assert_eq!(rig.joint_indices[v][0], owner);
                            assert_eq!(rig.joint_weights[v][0], 1.);
                        }
                    }}
                }
            }
            let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
            let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
            let (lo, hi) = rig.locomotion_bounds();
            for tick in 0..64 {
                rig.sample_locomotion(0.2, tick as f32*TAU/64., 1., &mut pose);
                gait::assert_supports(rig, &pose);
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                let mut low = f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {
                    let p = rig.skin_point(v, &matrices); low = low.min(p.y);
                    assert!(p.y >= -0.00001, "{name} walks through floor");
                    assert!(p.cmpge(lo).all() && p.cmple(hi).all());
                }
                assert!(low < 0.005, "{name} lost ground contact: {low}");
            }
        }
    }

    #[test]
    fn sand_mammal_skin_walks_on_anatomical_supports_with_local_marks_and_claws() {
        for name in ["SANDSHREW", "SANDSLASH"] {
            let canonical = crate::species_rig::for_species(name).unwrap();
            let rig = world_rig(canonical);
            assert_eq!(locomotion(name), Locomotion::Walk);
            assert_eq!(rig.walk_legs.len(), 2);
            assert!(!quadruped(canonical.species.name()));
            for part in canonical.anatomy.iter().filter(|p| !["Haunch /", "Leg /", "Foot /", "Toe /"].iter().any(|prefix| p.name.starts_with(prefix))) {
                for v in part.vertices.clone() {
                    assert_eq!(rig.joint_indices[v], canonical.joint_indices[v], "{name} {}", part.name);
                    assert_eq!(rig.joint_weights[v], canonical.joint_weights[v], "{name} {}", part.name);
                }
            }
            for side in [-1, 1] {
                let suffix = format!(" {side}");
                let foot = rig.anatomy.iter().find(|p| p.name.starts_with("Foot /") && p.name.ends_with(&suffix)).unwrap();
                let owner = rig.joint_indices[foot.vertices.start][0];
                for prefix in ["Haunch /", "Leg /", "Foot /", "Toe /"] {
                    let parts = rig.anatomy.iter().filter(|p| p.name.starts_with(prefix) && p.name.ends_with(&suffix)).collect::<Vec<_>>();
                    assert_eq!(parts.len(), 1, "{name} {prefix}{suffix} missing or combined");
                    for part in parts { for v in part.vertices.clone() {
                        for (&joint, weight) in rig.joint_indices[v].iter().zip(rig.joint_weights[v]) {
                            if weight > 0. { assert!((owner-2..=owner).contains(&joint), "{name} {} follows another leg", part.name); }
                        }
                        if ["Foot /", "Toe /"].iter().any(|p| part.name.starts_with(p)) {
                            assert_eq!(rig.joint_indices[v][0], owner);
                            assert_eq!(rig.joint_weights[v][0], 1.);
                        }
                    }}
                }
            }
            let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
            let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
            let (lo, hi) = rig.locomotion_bounds();
            for tick in 0..64 {
                rig.sample_locomotion(0.2, tick as f32*TAU/64., 1., &mut pose);
                gait::assert_supports(rig, &pose);
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                let mut low = f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {
                    let p = rig.skin_point(v, &matrices); low = low.min(p.y);
                    assert!(p.y >= -0.00001, "{name} walks through floor");
                    assert!(p.cmpge(lo).all() && p.cmple(hi).all());
                }
                assert!(low < 0.005, "{name} lost ground contact: {low}");
            }
        }
    }

    #[test]
    fn poli_frog_skin_walks_on_anatomical_supports_with_local_marks_and_claws() {
        for (name,legs) in [("POLIWHIRL",2),("POLIWRATH",2),("POLITOED",2)] {
            let canonical=crate::species_rig::for_species(name).unwrap();let rig=world_rig(canonical);assert_eq!(locomotion(name),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),legs);assert_eq!(quadruped(canonical.species.name()),legs==4);
            for part in canonical.anatomy.iter().filter(|p|["Head /","Eye socket /","Eye /","Pupil /","Brow /","Cheek /","Mouth /","Crest /","Belly /","Spiral /","Arm /","Hand /","Thumb /","Finger /","Glove crease /","Body /"].iter().any(|prefix|p.name.starts_with(prefix))) {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v]);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}}
            for side in [-1,1] {for placement in if legs==4 {vec!["front","rear"]} else {vec![""]} {
                let suffix=if legs==4 {format!(" {side} {placement}")} else {format!(" {side}")};let foot=rig.anatomy.iter().find(|p|p.name.starts_with("Foot /") && p.name.ends_with(&suffix)).unwrap();let owner=rig.joint_indices[foot.vertices.start][0];
                let prefixes=if legs==4 {vec!["Haunch /","Leg /","Leg spot /","Foot /","Claw /"]} else {if name=="POLITOED" {vec!["Haunch /","Leg /","Foot /","Toe /"]} else {vec!["Haunch /","Leg /","Foot /"]}};
                for prefix in prefixes {let parts=rig.anatomy.iter().filter(|p|p.name.starts_with(prefix) && p.name.ends_with(&suffix)).collect::<Vec<_>>();assert_eq!(parts.len(),1,"{name} {prefix}{suffix} missing or combined");for part in parts {for v in part.vertices.clone() {for (&joint,weight) in rig.joint_indices[v].iter().zip(rig.joint_weights[v]) {if weight>0. {assert!((owner-2..=owner).contains(&joint),"{name} {} follows another leg",part.name);}}if ["Foot /","Toe /"].iter().any(|p|part.name.starts_with(p)) {assert_eq!(rig.joint_indices[v][0],owner);assert_eq!(rig.joint_weights[v][0],1.);}}}}
            }}
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut low=f32::INFINITY;for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);low=low.min(p.y);assert!(p.y>=-0.00001,"{name} walks through floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all());}travel::assert_contact(rig, tick as f32*TAU/64., low);}
        }
    }

    #[test]
    fn egg_nurse_skin_walks_on_two_complete_feet_without_rebinding_pouches_or_ornaments() {
        for name in ["CHANSEY","BLISSEY"] {
            let canonical=crate::species_rig::for_species(name).unwrap();let rig=world_rig(canonical);
            assert_eq!(locomotion(name),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),2);
            for part in canonical.anatomy.iter().filter(|p|["Pouch /","Egg /","Eye /","Eye glint /","Smile /","Chin /","Mouth lining /","Tongue /","Arm /","Hand /","Ribbon /","Tail /","Crown /","Curl /","Arm frill /","Skirt /"].iter().any(|prefix|p.name.starts_with(prefix))) {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v]);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}}
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut low=f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);low=low.min(p.y);assert!(p.y>=-0.00001,"{name} walks through floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all());}assert!(low<0.005,"{name} lost ground contact: {low}");}
        }
    }

    #[test]
    fn electabuzz_skin_walks_on_two_feet_without_rebinding_stripes_fists_and_antennae() {
        let canonical=crate::species_rig::for_species("ELECTABUZZ").unwrap();let rig=world_rig(canonical);
        assert_eq!(locomotion("ELECTABUZZ"),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),2);
        for part in canonical.anatomy.iter().filter(|p|["Jaw /","Tongue /","Fang /","Arm /","Arm stripe /","Hand /","Knuckle /","Thumb /","Wrist fur /","Antenna /","Antenna ball /","Crest /","Tail /","Tail stripe /","Tail tip /"].iter().any(|prefix|p.name.starts_with(prefix))) {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v]);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}}
        let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
        for tick in 0..64 {rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut low=f32::INFINITY;
            for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);low=low.min(p.y);assert!(p.y>=-0.00001,"Electabuzz walks through floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all());}assert!(low<0.005,"Electabuzz lost ground contact: {low}");}
    }

    #[test]
    fn magmar_skin_walks_on_two_feet_without_rebinding_bill_cuffs_and_flames() {
        let canonical=crate::species_rig::for_species("MAGMAR").unwrap();let rig=world_rig(canonical);
        assert_eq!(locomotion("MAGMAR"),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),2);
        for part in canonical.anatomy.iter().filter(|p|["Bill /","Tongue /","Crown flame /","Crown glow /","Tail flame /","Tail glow /","Arm /","Cuff /","Hand /","Hand claw /"].iter().any(|prefix|p.name.starts_with(prefix))) {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v]);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}}
        let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
        for tick in 0..64 {rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut low=f32::INFINITY;
            for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);low=low.min(p.y);assert!(p.y>=-0.00001,"Magmar walks through floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all());}assert!(low<0.005,"Magmar lost ground contact: {low}");}
    }

    #[test]
    fn turtle_skin_walks_on_two_complete_feet_without_rebinding_shells_and_features() {
        for name in ["WARTORTLE","BLASTOISE"] {
            let canonical=crate::species_rig::for_species(name).unwrap();let rig=world_rig(canonical);assert_eq!(locomotion(name),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),2,"{name} split a complete support limb");
            for part in canonical.anatomy.iter().filter(|p|["Shell /","Shell rim /","Shell seam /","Belly /","Belly seam /","Cannon /","Cannon lining /","Cannon floor /","Cannon mount /","Ear /","Ear lobe /","Ear inset /","Tail /","Tail fold /","Arm /","Hand claw /"].iter().any(|prefix|p.name.starts_with(prefix))) {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v]);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}}
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut low=f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);low=low.min(p.y);assert!(p.y>=-0.00001,"{name} walks through floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all());}assert!(low<0.005,"{name} lost ground contact: {low}");}
        }
    }

    #[test]
    fn crocodile_skin_walks_on_two_complete_feet_with_non_leg_owners_preserved() {
        for name in ["CROCONAW","FERALIGATR"] {
            let canonical=crate::species_rig::for_species(name).unwrap();let rig=world_rig(canonical);assert_eq!(locomotion(name),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),2,"{name} split a complete limb");
            for part in canonical.anatomy.iter().filter(|p|p.name.starts_with("Hand /") || p.name.starts_with("Tooth /") || p.name.starts_with("Tail /") || p.name.starts_with("Crest /") || p.name.starts_with("Armor / four fitted arm")) {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v]);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}}
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut low=f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);low=low.min(p.y);assert!(p.y>=-0.00001,"{name} walks through floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all());}assert!(low<0.005,"{name} lost ground contact: {low}");}
        }
    }

    #[test]
    fn bone_keeper_skin_walks_on_two_feet_with_club_weights_intact() {
        for name in ["CUBONE","MAROWAK"] {
            let canonical=crate::species_rig::for_species(name).unwrap();let rig=world_rig(canonical);
            assert_eq!(locomotion(name),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),2,"{name} split a complete support limb");
            for part in canonical.anatomy.iter().filter(|p|p.name.starts_with("Bone shaft /") || p.name.starts_with("Bone lobe /") || p.name.starts_with("Thumb /")) {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v]);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}}
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {
                rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut low=f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);low=low.min(p.y);assert!(p.y>=-0.00001,"{name} walks through floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all());}
                assert!(low<0.005,"{name} lost ground contact: {low}");
            }
        }
    }

    #[test]
    fn tapir_skin_walks_on_two_feet_and_preserves_held_pendulum_weights() {
        for (name,legs) in [("DROWZEE",2),("HYPNO",2)] {
            let rig=world_rig(crate::species_rig::for_species(name).unwrap());assert_eq!(locomotion(name),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),legs,"{name} split or combined a complete limb");
            let canonical=crate::species_rig::for_species(name).unwrap();
            for part in canonical.anatomy.iter().filter(|p|p.name.starts_with("Pendulum /")) {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v]);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}}
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {
                rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut low=f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {
                    let p=rig.skin_point(v,&matrices);low=low.min(p.y);assert!(p.y>=-0.00001,"{name} walks through the floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all(),"{name} left locomotion culling bounds");
                }
                assert!(low<0.005,"{name} lost ground contact: {low}");
            }
        }
    }

    #[test]
    fn tangela_skin_walks_on_two_shoes_and_preserves_all_authored_vine_weights() {
        for (name,legs) in [("TANGELA",2)] {
            let rig=world_rig(crate::species_rig::for_species(name).unwrap());assert_eq!(locomotion(name),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),legs,"{name} split or combined a complete limb");
            let canonical=crate::species_rig::for_species(name).unwrap();
            for part in canonical.anatomy.iter().filter(|p|!p.name.starts_with("Shoe /")) {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v]);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}}
            assert!(rig.motion_limbs.is_empty());
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {
                rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut low=f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {
                    let p=rig.skin_point(v,&matrices);low=low.min(p.y);assert!(p.y>=-0.00001,"{name} walks through the floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all(),"{name} left locomotion culling bounds");
                }
                assert!(low<0.005,"{name} lost ground contact: {low}");
            }
        }
    }

    #[test]
    fn pokemon_motion_rhinos_keep_four_and_two_complete_weight_bearing_legs() {
        for (name,legs) in [("RHYHORN",4),("RHYDON",2)] {
            let rig=world_rig(crate::species_rig::for_species(name).unwrap());assert_eq!(locomotion(name),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),legs,"{name} split or combined a complete limb");
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {
                rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut low=f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {
                    let p=rig.skin_point(v,&matrices);low=low.min(p.y);assert!(p.y>=-0.00001,"{name} walks through the floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all(),"{name} left locomotion culling bounds");
                }
                assert!(low<0.005,"{name} lost ground contact: {low}");
            }
        }
    }

    #[test]
    fn pokemon_motion_seals_slide_with_continuous_belly_contact() {
        for name in ["SEEL","DEWGONG"] {
            let canonical=crate::species_rig::for_species(name).unwrap();let rig=world_rig(canonical);
            assert_eq!(locomotion(name),Locomotion::Walk);assert!(rig.walk_legs.is_empty());assert_eq!(rig.joints.len(),canonical.joints.len());
            let belly:Vec<_>=rig.anatomy.iter().filter(|part|part.name.starts_with("Body /"))
                .flat_map(|part|part.vertices.clone()).collect();assert!(!belly.is_empty());
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {
                rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();
                for v in 0..rig.neutral.positions.len() {
                    let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.00001,"{name} slides through the floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all(),"{name} left locomotion culling bounds");
                }
                // A pulling seal changes which belly point bears its weight;
                // foreflippers and every former contact point are not pins.
                let contact=belly.iter().map(|&v|rig.skin_point(v,&matrices).y)
                    .fold(f32::INFINITY,f32::min);
                assert!(contact.abs()<0.000001,"{name} lifted its belly: {contact}");
            }
        }
    }

    #[test]
    fn pokemon_motion_slimes_slide_with_broad_fixed_puddles() {
        for name in ["GRIMER","MUK","DITTO"] {
            let canonical=crate::species_rig::for_species(name).unwrap();
            let rig=world_rig(canonical);
            assert_eq!(locomotion(name),Locomotion::Walk);
            assert!(rig.walk_legs.is_empty());
            assert_eq!(rig.joints.len(),canonical.joints.len());
            let floor:Vec<_>=rig.neutral.positions.iter().enumerate().filter_map(|(v,p)|(p[1]<0.0000001).then_some(v)).collect();
            assert!(floor.len()>=48);
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];
            let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];
            let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {
                rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);
                rig.skin_matrices_into(&pose,&mut matrices).unwrap();
                for v in 0..rig.neutral.positions.len() {
                    let p=rig.skin_point(v,&matrices);
                    assert!(p.y>=-0.00001,"{name} slides through the floor");
                    assert!(p.cmpge(lo).all() && p.cmple(hi).all(),"{name} left locomotion culling bounds");
                }
                for &v in &floor {assert!(rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001,"{name} lifted its puddle sole");}
            }
        }
    }

    #[test]
    fn seahorse_skin_preserves_coherent_fins_snouts_and_swimming_clearance() {
        for name in ["HORSEA","SEADRA","KINGDRA"] {
            let canonical=crate::species_rig::for_species(name).unwrap();let rig=world_rig(canonical);
            assert_eq!(locomotion(name),Locomotion::Float);assert!(battle_hovers(name));assert!(rig.walk_legs.is_empty());assert_eq!(rig.motion_limbs.len(),1);assert_eq!(rig.motion_limbs[0].joint,7);assert_eq!(rig.motion_limbs[0].role,Role::Tail);assert_eq!(rig.joints.len(),canonical.joints.len());
            assert_eq!(rig.joint_indices,canonical.joint_indices);assert_eq!(rig.joint_weights,canonical.joint_weights);
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();let height=rig.neutral_bounds.1.y-rig.neutral_bounds.0.y;
            for tick in 0..64 {
                rig.sample_locomotion(tick as f32/32.,tick as f32,1.,&mut pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();
                for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);assert!(p.y>height*0.14,"{name} lost floating clearance");assert!(p.cmpge(lo).all() && p.cmple(hi).all(),"{name} left locomotion culling bounds");}
            }
        }
    }

    #[test]
    fn goldfish_skin_preserves_painted_fins_mouths_and_separates_water_from_land() {
        for name in ["GOLDEEN","SEAKING","MAGIKARP"] {
            let canonical=crate::species_rig::for_species(name).unwrap();let rig=world_rig(canonical);
            assert_eq!(locomotion(name),Locomotion::Float);assert!(battle_hovers(name));assert!(rig.walk_legs.is_empty());assert_eq!(rig.motion_limbs.len(),1);assert!(rig.motion_limbs.iter().all(|limb|limb.role==Role::Tail));
            assert_eq!(rig.joints.len(),canonical.joints.len()+if name=="MAGIKARP" {0}else{2});
            assert_eq!(rig.neutral.positions,canonical.neutral.positions);
            assert_eq!(rig.neutral.colors,canonical.neutral.colors);
            assert_eq!(rig.neutral.indices,canonical.neutral.indices);
            for part in canonical.anatomy.iter().filter(|part|role(&part.name)==Role::Head
                && !part.name.starts_with("Whisker /")) {
                for v in part.vertices.clone() {
                    assert_eq!(rig.joint_indices[v],canonical.joint_indices[v],"{name} {}",part.name);
                    assert_eq!(rig.joint_weights[v],canonical.joint_weights[v],"{name} {}",part.name);
                }
            }
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();let height=rig.neutral_bounds.1.y-rig.neutral_bounds.0.y;
            for tick in 0..64 {
                rig.sample_locomotion(tick as f32/32.,tick as f32,1.,&mut pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();
                for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);assert!(p.y >= -height * 0.001, "{name} flop penetrates floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all(),"{name} left locomotion culling bounds");}
                rig.sample_swimming(tick as f32/32.,tick as f32,1.,&mut pose);
                assert!(pose[0].rotation.abs_diff_eq(Quat::IDENTITY,0.00001),"{name} flops underwater");
                assert!(pose[0].translation.y.abs() <= height*0.012+0.00001,"{name} hovers above its water anchor");
            }
        }
    }

    #[test]
    fn dragon_family_skin_preserves_coils_soles_ribbons_and_authored_flight() {
        for name in ["DRATINI", "DRAGONAIR", "DRAGONITE"] {
            let canonical = crate::species_rig::for_species(name).unwrap();
            let rig = world_rig(canonical);
            let flying = name == "DRAGONITE";
            assert_eq!(locomotion(name), if flying { Locomotion::Fly } else { Locomotion::Walk });
            assert!(rig.walk_legs.is_empty());
            assert!(rig.crawl_segments.is_empty());
            assert_eq!(rig.neutral.positions, canonical.neutral.positions);
            if !flying {
                assert!(rig.joints.len() > canonical.joints.len(), "{name} needs a travelling coil");
            }
            let floor: Vec<_> = (0..rig.neutral.positions.len()).filter(|&v| rig.neutral.positions[v][1] < 0.0000001).collect();
            assert!(!floor.is_empty());
            let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
            let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
            let (lo, hi) = rig.locomotion_bounds();
            let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
            for tick in 0..64 {
                rig.sample_locomotion(tick as f32 / 32., tick as f32 * TAU / 64., 1., &mut pose);
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                let mut low = f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {
                    let p = rig.skin_point(v, &matrices);
                    low = low.min(p.y);
                    assert!(p.y >= -0.000001, "{name} crosses floor");
                    assert!(p.cmpge(lo).all() && p.cmple(hi).all(), "{name} leaves locomotion bounds");
                    if flying { assert!(p.y > height * 0.14, "{name} lost flight clearance"); }
                }
                if !flying { assert!(low < height * 0.005, "{name} lost coil contact"); }
                for &v in &floor {
                    let expected = Vec3::from_array(rig.neutral.positions[v]) + pose[0].translation;
                    assert!((rig.skin_point(v, &matrices).y - expected.y).abs() < 0.000001, "{name} contact {v} left root");
                }
            }
        }
    }

    #[test]
    fn ekans_skin_preserves_grounded_coil_and_canonical_neck_tail_bindings() {
        for name in ["EKANS"] {
            let canonical = crate::species_rig::for_species(name).unwrap();
            let rig = world_rig(canonical);
            let flying = name == "DRAGONITE";
            assert_eq!(locomotion(name), if flying { Locomotion::Fly } else { Locomotion::Walk });
            assert!(rig.walk_legs.is_empty());
            assert!(rig.crawl_segments.is_empty());
            assert_eq!(rig.neutral.positions, canonical.neutral.positions);
            if !flying {
                assert!(rig.joints.len() > canonical.joints.len(), "{name} needs a travelling coil");
            }
            let floor: Vec<_> = (0..rig.neutral.positions.len()).filter(|&v| rig.neutral.positions[v][1] < 0.0000001).collect();
            assert!(!floor.is_empty());
            let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
            let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
            let (lo, hi) = rig.locomotion_bounds();
            let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
            for tick in 0..64 {
                rig.sample_locomotion(tick as f32 / 32., tick as f32 * TAU / 64., 1., &mut pose);
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                let mut low = f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {
                    let p = rig.skin_point(v, &matrices);
                    low = low.min(p.y);
                    assert!(p.y >= -0.000001, "{name} crosses floor");
                    assert!(p.cmpge(lo).all() && p.cmple(hi).all(), "{name} leaves locomotion bounds");
                    if flying { assert!(p.y > height * 0.14, "{name} lost flight clearance"); }
                }
                if !flying { assert!(low < height * 0.005, "{name} lost coil contact"); }
                for &v in &floor {
                    let expected = Vec3::from_array(rig.neutral.positions[v]) + pose[0].translation;
                    assert!((rig.skin_point(v, &matrices).y - expected.y).abs() < 0.000001, "{name} contact {v} left root");
                }
            }
        }
    }

    #[test]
    fn arbok_skin_preserves_grounded_coil_and_canonical_hood_jaw_bindings() {
        for name in ["ARBOK"] {
            let canonical = crate::species_rig::for_species(name).unwrap();
            let rig = world_rig(canonical);
            let flying = name == "DRAGONITE";
            assert_eq!(locomotion(name), if flying { Locomotion::Fly } else { Locomotion::Walk });
            assert!(rig.walk_legs.is_empty());
            assert!(rig.crawl_segments.is_empty());
            assert_eq!(rig.neutral.positions, canonical.neutral.positions);
            if !flying {
                assert!(rig.joints.len() > canonical.joints.len(), "{name} needs a travelling coil");
            }
            let floor: Vec<_> = (0..rig.neutral.positions.len()).filter(|&v| rig.neutral.positions[v][1] < 0.0000001).collect();
            assert!(!floor.is_empty());
            let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
            let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
            let (lo, hi) = rig.locomotion_bounds();
            let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
            for tick in 0..64 {
                rig.sample_locomotion(tick as f32 / 32., tick as f32 * TAU / 64., 1., &mut pose);
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                let mut low = f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {
                    let p = rig.skin_point(v, &matrices);
                    low = low.min(p.y);
                    assert!(p.y >= -0.000001, "{name} crosses floor");
                    assert!(p.cmpge(lo).all() && p.cmple(hi).all(), "{name} leaves locomotion bounds");
                    if flying { assert!(p.y > height * 0.14, "{name} lost flight clearance"); }
                }
                if !flying { assert!(low < height * 0.005, "{name} lost coil contact"); }
                for &v in &floor {
                    let expected = Vec3::from_array(rig.neutral.positions[v]) + pose[0].translation;
                    assert!((rig.skin_point(v, &matrices).y - expected.y).abs() < 0.000001, "{name} contact {v} left root");
                }
            }
        }
    }

    #[test]
    fn steelix_skin_preserves_grounded_supports_and_canonical_armor_jaw_bindings() {
        for name in ["STEELIX"] {
            let canonical = crate::species_rig::for_species(name).unwrap();
            let rig = world_rig(canonical);
            let flying = name == "DRAGONITE";
            assert_eq!(locomotion(name), if flying { Locomotion::Fly } else { Locomotion::Walk });
            assert!(rig.walk_legs.is_empty());
            assert!(rig.crawl_segments.is_empty());
            assert_eq!(rig.joints.len(), canonical.joints.len());
            for v in 0..rig.neutral.positions.len() {
                assert_eq!(rig.joint_indices[v], canonical.joint_indices[v], "{name} vertex {v}");
                assert_eq!(rig.joint_weights[v], canonical.joint_weights[v], "{name} vertex {v}");
            }
            let floor: Vec<_> = (0..rig.neutral.positions.len()).filter(|&v| rig.neutral.positions[v][1] < 0.0000001).collect();
            assert!(!floor.is_empty());
            let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
            let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
            let (lo, hi) = rig.locomotion_bounds();
            let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
            for tick in 0..64 {
                rig.sample_locomotion(tick as f32 / 32., tick as f32 * TAU / 64., 1., &mut pose);
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                let mut low = f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {
                    let p = rig.skin_point(v, &matrices);
                    low = low.min(p.y);
                    assert!(p.y >= -0.000001, "{name} crosses floor");
                    assert!(p.cmpge(lo).all() && p.cmple(hi).all(), "{name} leaves locomotion bounds");
                    if flying { assert!(p.y > height * 0.14, "{name} lost flight clearance"); }
                }
                if !flying { assert!(low < height * 0.005, "{name} lost coil contact"); }
                for &v in &floor {
                    let expected = Vec3::from_array(rig.neutral.positions[v]) + pose[0].translation;
                    assert!((rig.skin_point(v, &matrices).y - expected.y).abs() < 0.000001, "{name} contact {v} left root");
                }
            }
        }
    }

    #[test]
    fn pig_monkey_skin_walks_on_two_complete_supports_with_coherent_fur_faces_fists() {
        for (name,tails) in [("MANKEY",1),("PRIMEAPE",0)] {
            let canonical=crate::species_rig::for_species(name).unwrap();let rig=world_rig(canonical);
            assert_eq!(locomotion(name),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),2,"{name} support chains");
            assert_eq!(rig.motion_limbs.iter().filter(|l|l.role==Role::Arm).count(),2);assert_eq!(rig.motion_limbs.iter().filter(|l|l.role==Role::Tail).count(),tails);
            for part in &canonical.anatomy {
                if ["Leg /","Foot /","Toe /","Ankle cuff /"].iter().any(|p|part.name.starts_with(p)) {continue;}
                for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v],"{name} {}",part.name);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}
            }
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {
                rig.sample_locomotion(tick as f32/20.,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut floor=f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);floor=floor.min(p.y);assert!(p.y>=-0.00001,"{name} crosses floor");assert!(p.cmpge(lo).all()&&p.cmple(hi).all(),"{name} leaves bounds");}
                assert!(floor<0.005,"{name} loses ground contact {floor}");
            }
        }
    }

    #[test]
    fn feline_skin_walks_complete_supports_with_coherent_gaze_whiskers_charms() {
        for (name,supports,arms) in [("MEOWTH",2,2),("PERSIAN",4,0)] {
            let canonical=crate::species_rig::for_species(name).unwrap();let rig=world_rig(canonical);
            assert_eq!(locomotion(name),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),supports,"{name} support chains");
            assert_eq!(rig.motion_limbs.iter().filter(|l|l.role==Role::Arm).count(),arms);assert_eq!(rig.motion_limbs.iter().filter(|l|l.role==Role::Tail).count(),1);
            for part in &canonical.anatomy {
                if ["Leg /","Foot /","Toe /"].iter().any(|p|part.name.starts_with(p)) {continue;}
                for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v],"{name} {}",part.name);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}
            }
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {
                rig.sample_locomotion(tick as f32/20.,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut floor=f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);floor=floor.min(p.y);assert!(p.y>=-0.00001,"{name} crosses floor");assert!(p.cmpge(lo).all()&&p.cmple(hi).all(),"{name} leaves bounds");}
                assert!(floor<0.005,"{name} loses ground contact {floor}");
            }
        }
    }

    #[test]
    fn dodo_skin_walks_complete_supports_with_coherent_separate_heads_and_jaws() {
        for (name,tails) in [("DODUO",0),("DODRIO",3)] {
            let canonical=crate::species_rig::for_species(name).unwrap();let rig=world_rig(canonical);
            assert_eq!(locomotion(name),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),2,"{name} support chains");
            assert_eq!(rig.motion_limbs.iter().filter(|l|l.role==Role::Arm).count(),0);assert_eq!(rig.motion_limbs.iter().filter(|l|l.role==Role::Tail).count(),tails);
            for part in &canonical.anatomy {
                if ["Leg /","Foot /"].iter().any(|p|part.name.starts_with(p)) {continue;}
                for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v],"{name} {}",part.name);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}
            }
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {
                rig.sample_locomotion(tick as f32/20.,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut floor=f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);floor=floor.min(p.y);assert!(p.y>=-0.00001,"{name} crosses floor");assert!(p.cmpge(lo).all()&&p.cmple(hi).all(),"{name} leaves bounds");}
                assert!(floor<0.005,"{name} loses ground contact {floor}");
            }
        }
    }

    #[test]
    fn leek_duck_skin_walks_two_webbed_supports_and_preserves_the_carry_chain() {
        let canonical=crate::species_rig::for_species("FARFETCH_D").unwrap();let rig=world_rig(canonical);
        assert_eq!(locomotion("FARFETCH_D"),Locomotion::Walk);assert!(!battle_hovers("FARFETCH_D"));assert_eq!(rig.walk_legs.len(),2);
        assert_eq!(rig.motion_limbs.iter().filter(|l|l.role==Role::Arm).count(),0,"carried leek must retain authored wing motion");
        for part in &canonical.anatomy {
            if ["Leg /","Foot /"].iter().any(|p|part.name.starts_with(p)) {continue;}
            for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v],"{}",part.name);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}
        }
        let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
        for tick in 0..64 {
            rig.sample_locomotion(tick as f32/20.,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut floor=f32::INFINITY;
            for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);floor=floor.min(p.y);assert!(p.y>=-0.00001,"duck crosses floor");assert!(p.cmpge(lo).all()&&p.cmple(hi).all(),"duck leaves bounds");}
            assert!(floor<0.005,"duck loses ground contact {floor}");
        }
    }

    #[test]
    fn jynx_skin_keeps_world_hem_seated_and_canonical_open_hand_bindings() {
        let canonical=crate::species_rig::for_species("JYNX").unwrap();let rig=world_rig(canonical);
        assert_eq!(locomotion("JYNX"),Locomotion::Walk);assert!(!battle_hovers("JYNX"));assert!(rig.walk_legs.is_empty());assert!(rig.crawl_segments.is_empty());
        assert_eq!(rig.joints.len(),22);assert_eq!(rig.joint_indices,canonical.joint_indices);assert_eq!(rig.joint_weights,canonical.joint_weights);
        let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
        for tick in 0..64 {
            rig.sample_locomotion(tick as f32/32.,tick as f32*TAU/64.,1.,&mut pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut floor=f32::INFINITY;
            for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);floor=floor.min(p.y);assert!(p.y>=-0.000001);assert!(p.cmpge(lo).all()&&p.cmple(hi).all());}
            assert!(floor.abs()<0.000001);
        }
    }

    #[test]
    fn tentacle_skin_keeps_world_tips_seated_without_changing_swimming_class() {
        for name in ["TENTACOOL","TENTACRUEL"] {
            let canonical=crate::species_rig::for_species(name).unwrap();let rig=world_rig(canonical);
            assert_eq!(locomotion(name),Locomotion::Float);assert!(!battle_hovers(name));assert_eq!(rig.hover_clearance,0.);assert!(rig.walk_legs.is_empty());assert!(rig.crawl_segments.is_empty());
            assert_eq!(rig.joints.len(),canonical.joints.len());assert_eq!(rig.joint_indices,canonical.joint_indices);assert_eq!(rig.joint_weights,canonical.joint_weights);
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {
                rig.sample_locomotion(tick as f32/32.,tick as f32*TAU/64.,1.,&mut pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut floor=f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);floor=floor.min(p.y);assert!(p.y>=-0.000001);assert!(p.cmpge(lo).all()&&p.cmple(hi).all());}
                assert!(floor.abs()<0.000001,"{name} left authored floor: {floor}");
            }
        }
        for name in ["HORSEA","PORYGON","KOFFING"] {assert!(battle_hovers(name));}
    }

    #[test]
    fn dugtrio_skin_preserves_grounded_soil_and_three_independent_capsules() {
        for name in ["DUGTRIO"] {
            let canonical = crate::species_rig::for_species(name).unwrap();
            let rig = world_rig(canonical);
            let flying = name == "DRAGONITE";
            assert_eq!(locomotion(name), if flying { Locomotion::Fly } else { Locomotion::Walk });
            assert!(rig.walk_legs.is_empty());
            assert!(rig.crawl_segments.is_empty());
            assert_eq!(rig.joints.len(), canonical.joints.len());
            for v in 0..rig.neutral.positions.len() {
                assert_eq!(rig.joint_indices[v], canonical.joint_indices[v], "{name} vertex {v}");
                assert_eq!(rig.joint_weights[v], canonical.joint_weights[v], "{name} vertex {v}");
            }
            let floor: Vec<_> = (0..rig.neutral.positions.len()).filter(|&v| rig.neutral.positions[v][1] < 0.0000001).collect();
            assert!(!floor.is_empty());
            let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
            let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
            let (lo, hi) = rig.locomotion_bounds();
            let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
            for tick in 0..64 {
                rig.sample_locomotion(tick as f32 / 32., tick as f32 * TAU / 64., 1., &mut pose);
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                let mut low = f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {
                    let p = rig.skin_point(v, &matrices);
                    low = low.min(p.y);
                    assert!(p.y >= -0.000001, "{name} crosses floor");
                    assert!(p.cmpge(lo).all() && p.cmple(hi).all(), "{name} leaves locomotion bounds");
                    if flying { assert!(p.y > height * 0.14, "{name} lost flight clearance"); }
                }
                if !flying { assert!(low < height * 0.005, "{name} lost coil contact"); }
                for &v in &floor {
                    let expected = Vec3::from_array(rig.neutral.positions[v]) + pose[0].translation;
                    assert!(rig.skin_point(v, &matrices).distance(expected) < 0.000001, "{name} contact {v} left root");
                }
            }
        }
    }

    #[test]
    fn aerodactyl_skin_preserves_canonical_jaw_wings_and_authored_flight() {
        let canonical = crate::species_rig::for_species("AERODACTYL").unwrap();
        let rig = world_rig(canonical);
        assert_eq!(locomotion("AERODACTYL"), Locomotion::Fly);
        assert!(rig.walk_legs.is_empty() && rig.crawl_segments.is_empty());
        assert_eq!(rig.joints.len(), canonical.joints.len());
        for v in 0..rig.neutral.positions.len() {
            assert_eq!(rig.joint_indices[v], canonical.joint_indices[v], "vertex {v}");
            assert_eq!(rig.joint_weights[v], canonical.joint_weights[v], "vertex {v}");
        }
        let wings: Vec<_> = rig.motion_limbs.iter().filter(|l| l.role == Role::Wing).collect();
        assert_eq!(wings.len(), 2);
        assert!(wings.iter().all(|l| l.authored_flight));
        let floor: Vec<_> = (0..rig.neutral.positions.len()).filter(|&v| rig.neutral.positions[v][1] < 0.0000001).collect();
        assert!(!floor.is_empty());
        let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
        let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
        let (lo, hi) = rig.locomotion_bounds();
        let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
        for tick in 0..64 {
            rig.sample_locomotion(tick as f32 / 32., tick as f32 * TAU / 64., 1., &mut pose);
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                let p = rig.skin_point(v, &matrices);
                assert!(p.y > height * 0.14);
                assert!(p.cmpge(lo).all() && p.cmple(hi).all(), "vertex {v} leaves flight bounds");
            }
            for &v in &floor { assert!(rig.skin_point(v, &matrices).distance(Vec3::from_array(rig.neutral.positions[v]) + pose[0].translation) < 0.000001); }
        }
    }

    #[test]
    fn pidgeotto_skin_preserves_rooted_shoulders_and_exact_authored_flight() {
        let canonical = crate::species_rig::for_species("PIDGEOTTO").unwrap();
        let rig = world_rig(canonical);
        assert_eq!(locomotion("PIDGEOTTO"), Locomotion::Fly);
        assert!(rig.walk_legs.is_empty() && rig.crawl_segments.is_empty());
        assert_eq!(rig.joints.len(), canonical.joints.len());
        for v in 0..rig.neutral.positions.len() {
            assert_eq!(rig.joint_indices[v], canonical.joint_indices[v], "vertex {v}");
            assert_eq!(rig.joint_weights[v], canonical.joint_weights[v], "vertex {v}");
        }
        let wings: Vec<_> = rig.motion_limbs.iter().filter(|l| l.role == Role::Wing).collect();
        assert_eq!(wings.len(), 2);
        assert!(wings.iter().all(|l| l.authored_flight));
        let floor: Vec<_> = (0..rig.neutral.positions.len()).filter(|&v| rig.neutral.positions[v][1] < 0.0000001).collect();
        assert!(!floor.is_empty());
        let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
        let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
        let (lo, hi) = rig.locomotion_bounds();
        let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
        for tick in 0..64 {
            rig.sample_locomotion(tick as f32 / 32., tick as f32 * TAU / 64., 1., &mut pose);
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                let p = rig.skin_point(v, &matrices);
                assert!(p.y > height * 0.14);
                assert!(p.cmpge(lo).all() && p.cmple(hi).all(), "vertex {v} leaves flight bounds");
            }
            for &v in &floor { assert!(rig.skin_point(v, &matrices).distance(Vec3::from_array(rig.neutral.positions[v]) + pose[0].translation) < 0.000001); }
        }
    }

    #[test]
    fn bird_world_flight_has_upstroke_and_downstroke_instead_of_an_idle_shrug() {
        for name in ["PIDGEY", "PIDGEOTTO", "PIDGEOT", "SPEAROW", "FEAROW", "NOCTOWL", "MURKROW", "SKARMORY",
            "ARTICUNO", "ZAPDOS", "MOLTRES", "HO_OH", "LUGIA", "TOGETIC",
            "AERODACTYL", "CHARIZARD", "DRAGONITE"] {
            let canonical = crate::species_rig::for_species(name).unwrap();
            let rig = world_rig(canonical);
            let wings: Vec<_> = rig.motion_limbs.iter().filter(|l| l.levelled_flight.is_some()).collect();
            assert_eq!(wings.len(), 2, "{name} needs two moving flight wings");
            let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
            let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
            for wing in wings {
                let tip = wing.levelled_flight.unwrap().2;
                let mut low = f32::INFINITY;
                let mut high = f32::NEG_INFINITY;
                let mut length = 0.0_f32;
                for tick in 0..32 {
                    rig.sample_locomotion(tick as f32 / (32. * flight_beat(rig.species.name()).0), 0., 1., &mut pose);
                    rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                    let shoulder = gait::world(rig, &pose, wing.joint).transform_point3(Vec3::ZERO);
                    let relative = rig.skin_point(tip, &matrices) - shoulder;
                    low = low.min(relative.y); high = high.max(relative.y);
                    length = length.max(relative.length());
                }
                assert!(low < -length * 0.25 && high > length * 0.25,
                    "{name} never completes a wing stroke: {low}..{high}, span {length}");
            }
            assert_eq!(rig.neutral.positions, canonical.neutral.positions);
        }
    }

    #[test]
    fn raptor_bird_skin_preserves_rooted_shoulders_and_authored_flight() {
        for name in ["PIDGEOT", "FEAROW"] {
        let canonical = crate::species_rig::for_species(name).unwrap();
        let rig = world_rig(canonical);
        assert_eq!(locomotion(name), Locomotion::Fly);
        assert!(rig.walk_legs.is_empty() && rig.crawl_segments.is_empty());
        assert_eq!(rig.joints.len(), canonical.joints.len());
        for v in 0..rig.neutral.positions.len() {
            assert_eq!(rig.joint_indices[v], canonical.joint_indices[v], "vertex {v}");
            assert_eq!(rig.joint_weights[v], canonical.joint_weights[v], "vertex {v}");
        }
        let wings: Vec<_> = rig.motion_limbs.iter().filter(|l| l.role == Role::Wing).collect();
        assert_eq!(wings.len(), 2);
        assert!(wings.iter().all(|l| l.authored_flight));
        let floor: Vec<_> = (0..rig.neutral.positions.len()).filter(|&v| rig.neutral.positions[v][1] < 0.0000001).collect();
        assert!(!floor.is_empty());
        let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
        let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
        let (lo, hi) = rig.locomotion_bounds();
        let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
        for tick in 0..64 {
            rig.sample_locomotion(tick as f32 / 32., tick as f32 * TAU / 64., 1., &mut pose);
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                let p = rig.skin_point(v, &matrices);
                assert!(p.y > height * 0.14);
                assert!(p.cmpge(lo).all() && p.cmple(hi).all(), "vertex {v} leaves flight bounds");
            }
            for &v in &floor { assert!(rig.skin_point(v, &matrices).distance(Vec3::from_array(rig.neutral.positions[v]) + pose[0].translation) < 0.000001); }
        }
        }
    }

    #[test]
    fn nautilus_skin_reuses_ten_canonical_hinges_and_preserves_grounded_crawl() {
        for name in ["OMANYTE", "OMASTAR"] {
            let canonical = crate::species_rig::for_species(name).unwrap();
            let rig = world_rig(canonical);
            assert_eq!(locomotion(name), Locomotion::Walk);
            assert_eq!(rig.joints.len(), canonical.joints.len());
            assert!(rig.walk_legs.is_empty());
            assert_eq!(rig.crawl_segments.len(), 10);
            assert!(rig.crawl_segments.iter().enumerate().all(|(i,s)| s.joint == i+5));
            for v in 0..rig.neutral.positions.len() {
                assert_eq!(rig.joint_indices[v], canonical.joint_indices[v], "{name} vertex {v}");
                assert_eq!(rig.joint_weights[v], canonical.joint_weights[v], "{name} vertex {v}");
            }
            let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
            let mut authored = pose.clone();
            let mut resting = pose.clone();
            let mut strokes = [false; 10];
            let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
            let (lo, hi) = rig.locomotion_bounds();
            let floor: Vec<_> = (0..rig.neutral.positions.len()).filter(|&v| rig.neutral.positions[v][1] < 0.0000001).collect();
            assert!(!floor.is_empty());
            for tick in 0..64 {
                let seconds = tick as f32 / 20.;
                let phase = tick as f32 * TAU / 64.;
                rig.sample_locomotion(seconds, phase, 1., &mut pose);
                rig.sample_locomotion(seconds, phase, 0., &mut resting);
                canonical.sample_into(Clip::Idle, seconds, Playback::Loop, &mut authored).unwrap();
                for (i, segment) in rig.crawl_segments.iter().enumerate() {
                    assert!(resting[segment.joint].rotation.angle_between(authored[segment.joint].rotation) < 0.001);
                    strokes[i] |= pose[segment.joint].rotation.angle_between(authored[segment.joint].rotation) > 0.01;
                }
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                for v in 0..rig.neutral.positions.len() {
                    let p = rig.skin_point(v, &matrices);
                    assert!(p.y >= -0.000001, "{name} vertex {v} crosses floor");
                    assert!(p.cmpge(lo).all() && p.cmple(hi).all(), "{name} leaves crawl bounds");
                }
                for &v in &floor { assert!(rig.skin_point(v, &matrices).y.abs() < 0.000001, "{name} loses ground contact"); }
            }
            assert!(strokes.into_iter().all(|moved| moved), "{name} needs ten independent crawl strokes");
            rig.sample_locomotion(0.37, 0.8, 1., &mut pose);
            rig.sample_locomotion(0.83, 0.8, 1., &mut resting);
            assert!(rig.crawl_segments.iter().any(|s| pose[s.joint].rotation.angle_between(resting[s.joint].rotation) > 0.01), "{name} lost authored clock motion while crawling");
        }
    }

    #[test]
    fn pitcher_skin_preserves_leaves_mouths_and_grounded_two_root_or_legless_motion() {
        for (name,legs) in [("BELLSPROUT",2),("WEEPINBELL",0),("VICTREEBEL",0)] {
            let canonical=crate::species_rig::for_species(name).unwrap();let rig=world_rig(canonical);assert_eq!(locomotion(name),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),legs,"{name} root supports");assert!(rig.motion_limbs.is_empty());
            if legs==0 {assert_eq!(rig.joints.len(),canonical.joints.len());}
            for part in &canonical.anatomy {if name=="BELLSPROUT" && part.name.starts_with("Root /") {continue;}for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v],"{name} {}",part.name);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}}
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {
                rig.sample_locomotion(tick as f32/20.,tick as f32*TAU/64.,1.,&mut pose);if legs>0 {gait::assert_supports(rig,&pose);}rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut floor=f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);floor=floor.min(p.y);assert!(p.y>=-0.00001,"{name} crosses floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all(),"{name} leaves full bounds");}travel::assert_contact(rig, tick as f32*TAU/64., floor);
            }
        }
    }

    #[test]
    fn armored_bug_skin_keeps_legless_canonical_bindings_and_ground_contact() {
        for name in ["PINECO","FORRETRESS"] {
            let canonical=crate::species_rig::for_species(name).unwrap();let rig=world_rig(canonical);
            assert_eq!(locomotion(name),Locomotion::Walk);assert!(rig.walk_legs.is_empty());assert!(rig.motion_limbs.is_empty());
            assert_eq!(rig.joints.len(),canonical.joints.len());assert_eq!(rig.joint_indices,canonical.joint_indices);assert_eq!(rig.joint_weights,canonical.joint_weights);
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {
                rig.sample_locomotion(tick as f32/20.,tick as f32*TAU/64.,1.,&mut pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut floor=f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);floor=floor.min(p.y);assert!(p.y>=-0.00001,"{name} crosses floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all(),"{name} leaves full bounds");}
                travel::assert_contact(rig, tick as f32*TAU/64., floor);
            }
        }
    }

    #[test]
    fn raichu_skin_keeps_face_curled_ears_and_thin_tail_while_walking() {
        for (name,legs,arms) in [("RAICHU",2,2),("PIKACHU",2,2)] {
            let canonical=crate::species_rig::for_species(name).unwrap();let rig=world_rig(canonical);
            assert_eq!(locomotion(name),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),legs,"{name} support chains");
            assert_eq!(rig.motion_limbs.iter().filter(|l|l.role==Role::Arm).count(),arms);
            assert_eq!(rig.motion_limbs.iter().filter(|l|l.role==Role::Tail).count(),1);
            for part in &canonical.anatomy {
                if ["Leg /","Foot /","Claw /","Thigh /","Shin /","Toe /","Haunch /","Toe crease /"].iter().any(|p|part.name.starts_with(p)) {continue;}
                for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v],"{name} {}",part.name);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}
            }
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {
                rig.sample_locomotion(tick as f32/20.,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut floor=f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);floor=floor.min(p.y);assert!(p.y>=-0.00001,"{name} crosses floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all(),"{name} leaves full bounds");}
                assert!(floor<0.005,"{name} loses ground contact {floor}");
            }
        }
    }

    #[test]
    fn rafflesia_skin_preserves_faces_and_crowns_while_walking() {
        for (name,legs,arms) in [("GLOOM",2,2),("VILEPLUME",2,2)] {
            let canonical=crate::species_rig::for_species(name).unwrap();let rig=world_rig(canonical);
            assert_eq!(locomotion(name),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),legs,"{name} support chains");
            assert_eq!(rig.motion_limbs.iter().filter(|l|l.role==Role::Arm).count(),arms);
            for part in &canonical.anatomy {
                if ["Leg /","Foot /","Claw /","Thigh /","Shin /","Toe /"].iter().any(|p|part.name.starts_with(p)) {continue;}
                for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v],"{name} {}",part.name);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}
            }
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {
                rig.sample_locomotion(tick as f32/20.,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut floor=f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);floor=floor.min(p.y);assert!(p.y>=-0.00001,"{name} crosses floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all(),"{name} leaves full bounds");}
                assert!(floor<0.005,"{name} loses ground contact {floor}");
            }
        }
    }

    #[test]
    fn fossil_scythe_skin_walks_on_four_and_two_complete_supports() {
        for (name,legs,arms) in [("KABUTO",4,0),("KABUTOPS",2,2)] {
            let canonical=crate::species_rig::for_species(name).unwrap();let rig=world_rig(canonical);
            assert_eq!(locomotion(name),Locomotion::Walk);assert_eq!(rig.walk_legs.len(),legs,"{name} support chains");
            assert_eq!(rig.motion_limbs.iter().filter(|l|l.role==Role::Arm).count(),arms);
            for part in &canonical.anatomy {
                if ["Leg /","Foot /","Claw /","Thigh /","Shin /","Toe /"].iter().any(|p|part.name.starts_with(p)) {continue;}
                for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v],"{name} {}",part.name);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}
            }
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {
                rig.sample_locomotion(tick as f32/20.,tick as f32*TAU/64.,1.,&mut pose);gait::assert_supports(rig,&pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut floor=f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);floor=floor.min(p.y);assert!(p.y>=-0.00001,"{name} crosses floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all(),"{name} leaves full bounds");}
                assert!(floor<0.005,"{name} loses ground contact {floor}");
            }
        }
    }

    #[test]
    fn electrode_skin_preserves_ground_contact_and_coherent_shell_without_fake_limbs() {
        let canonical = crate::species_rig::for_species("ELECTRODE").unwrap();
        let rig = world_rig(canonical);
        assert_eq!(locomotion("ELECTRODE"), Locomotion::Walk);
        assert!(rig.walk_legs.is_empty()); assert!(rig.motion_limbs.is_empty());
        assert_eq!(rig.joints.len(), canonical.joints.len());
        assert_eq!(rig.joint_indices, canonical.joint_indices);
        assert_eq!(rig.joint_weights, canonical.joint_weights);
        let mut pose=vec![Transform::IDENTITY;rig.joints.len()];
        let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()]; let (lo,hi)=rig.locomotion_bounds();
        for tick in 0..64 {
            rig.sample_locomotion(tick as f32/20.,tick as f32*TAU/64.,1.,&mut pose);
            rig.skin_matrices_into(&pose,&mut matrices).unwrap(); let mut floor=f32::INFINITY;
            for v in 0..rig.neutral.positions.len() {
                let p=rig.skin_point(v,&matrices);floor=floor.min(p.y);
                assert!(p.y>=-0.00001,"Electrode crosses floor");
                assert!(p.cmpge(lo).all() && p.cmple(hi).all(),"Electrode leaves full bounds");
            }
            assert!(floor<0.005,"Electrode loses ground contact: {floor}");
        }
    }

    #[test]
    fn rock_pupa_skin_preserves_two_larva_supports_and_grounded_legless_pupa() {
        for (name, legs) in [("LARVITAR", 2), ("PUPITAR", 0)] {
            let canonical = crate::species_rig::for_species(name).unwrap(); let rig = world_rig(canonical);
            assert_eq!(locomotion(name), Locomotion::Walk); assert_eq!(rig.walk_legs.len(), legs);
            assert_eq!(rig.motion_limbs.iter().filter(|l| l.role == Role::Arm).count(), if legs==0 {0} else {2});
            if legs==0 { assert_eq!(rig.joints.len(), canonical.joints.len()); }
            for part in &canonical.anatomy {
                if legs>0 && ["Foot /", "Toe /", "Thigh /"].iter().any(|p| part.name.starts_with(p)) {continue;}
                for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],canonical.joint_indices[v],"{name} {}",part.name);assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);}
            }
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {
                rig.sample_locomotion(tick as f32/20.,tick as f32*TAU/64.,1.,&mut pose);if legs>0 {gait::assert_supports(rig,&pose);}
                rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut floor=f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);floor=floor.min(p.y);assert!(p.y>=-0.00001,"{name} vertex {v} crosses floor");assert!(p.cmpge(lo).all() && p.cmple(hi).all(),"{name} leaves full locomotion bounds");}
                travel::assert_contact(rig, tick as f32*TAU/64., floor);
            }
        }
    }

    #[test]
    fn tyranitar_skin_walks_on_two_complete_supports_and_preserves_upper_armor_and_tail() {
        for (name, legs) in [("TYRANITAR", 2)] {
            let canonical = crate::species_rig::for_species(name).unwrap();
            let rig = world_rig(canonical);
            assert_eq!(locomotion(name), Locomotion::Walk);
            assert_eq!(rig.motion_limbs.iter().filter(|l| l.role == Role::Arm).count(), 2);
            assert_eq!(rig.walk_legs.len(), legs, "{name} split a toe or merged a limb");
            for part in &canonical.anatomy {
                if ["Foot /", "Foot claw /", "Thigh /", "Thigh vent /", "Shin /"].iter().any(|p| part.name.starts_with(p)) { continue; }
                for v in part.vertices.clone() {
                    assert_eq!(rig.joint_indices[v], canonical.joint_indices[v], "{name} {}", part.name);
                    assert_eq!(rig.joint_weights[v], canonical.joint_weights[v]);
                }
            }
            let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
            let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
            let (lo, hi) = rig.locomotion_bounds();
            for tick in 0..64 {
                rig.sample_locomotion(tick as f32/20., tick as f32*TAU/64., 1., &mut pose);
                gait::assert_supports(rig, &pose);
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                let mut floor = f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {
                    let p = rig.skin_point(v, &matrices); floor = floor.min(p.y);
                    assert!(p.y >= -0.00001, "{name} vertex {v} crosses floor");
                    assert!(p.cmpge(lo).all() && p.cmple(hi).all(), "{name} leaves locomotion bounds");
                }
                assert!(floor < 0.005, "{name} loses ground contact: {floor}");
            }
        }
    }

    #[test]
    fn fire_badger_skin_walks_on_complete_four_and_two_supports_with_preserved_upper_controls() {
        for (name, legs) in [("QUILAVA", 4), ("TYPHLOSION", 2)] {
            let canonical = crate::species_rig::for_species(name).unwrap();
            let rig = world_rig(canonical);
            assert_eq!(locomotion(name), Locomotion::Walk);
            assert_eq!(rig.walk_legs.len(), legs, "{name} split a toe or merged a limb");
            for part in &canonical.anatomy {
                if ["Foot /", "Toe /", "Leg /", "Haunch /"].iter().any(|p| part.name.starts_with(p)) { continue; }
                for v in part.vertices.clone() {
                    assert_eq!(rig.joint_indices[v], canonical.joint_indices[v], "{name} {}", part.name);
                    assert_eq!(rig.joint_weights[v], canonical.joint_weights[v]);
                }
            }
            let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
            let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
            let (lo, hi) = rig.locomotion_bounds();
            for tick in 0..64 {
                rig.sample_locomotion(tick as f32/20., tick as f32*TAU/64., 1., &mut pose);
                gait::assert_supports(rig, &pose);
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                let mut floor = f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {
                    let p = rig.skin_point(v, &matrices); floor = floor.min(p.y);
                    assert!(p.y >= -0.00001, "{name} vertex {v} crosses floor");
                    assert!(p.cmpge(lo).all() && p.cmple(hi).all(), "{name} leaves locomotion bounds");
                }
                assert!(floor < 0.005, "{name} loses ground contact: {floor}");
            }
        }
    }

    #[test]
    fn fire_dragon_skin_preserves_two_ground_supports_and_authored_flight() {
        for name in ["CHARMELEON", "CHARIZARD"] {
            let canonical = crate::species_rig::for_species(name).unwrap();
            let rig = world_rig(canonical);
            let flying = name == "CHARIZARD";
            assert_eq!(locomotion(name), if flying { Locomotion::Fly } else { Locomotion::Walk });
            assert_eq!(rig.walk_legs.len(), if flying { 0 } else { 2 });
            if flying { assert_eq!(rig.joints.len(), canonical.joints.len()); }
            for part in &canonical.anatomy {
                if !flying && ["Foot /", "Foot claw /", "Haunch /"].iter().any(|p| part.name.starts_with(p)) { continue; }
                for v in part.vertices.clone() {
                    assert_eq!(rig.joint_indices[v], canonical.joint_indices[v], "{name} {}", part.name);
                    assert_eq!(rig.joint_weights[v], canonical.joint_weights[v]);
                }
            }
            let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
            let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
            let (lo, hi) = rig.locomotion_bounds();
            let height = rig.neutral_bounds.1.y-rig.neutral_bounds.0.y;
            for tick in 0..64 {
                rig.sample_locomotion(tick as f32/20., tick as f32*TAU/64., 1., &mut pose);
                if !flying { gait::assert_supports(rig, &pose); }
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                let mut floor = f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {
                    let p = rig.skin_point(v, &matrices); floor = floor.min(p.y);
                    assert!(p.y >= -0.00001, "{name} vertex {v} crosses floor");
                    assert!(p.cmpge(lo).all() && p.cmple(hi).all(), "{name} leaves locomotion bounds");
                    if flying { assert!(p.y > height*0.14, "{name} lost flight clearance"); }
                }
                if !flying { assert!(floor < 0.005, "{name} loses ground contact: {floor}"); }
            }
        }
    }

    #[test]
    fn stone_pair_skin_walks_on_two_supports_and_preserves_complete_upper_anatomy() {
        for (name, arms) in [("GRAVELER", 4), ("GOLEM", 2)] {
            let canonical = crate::species_rig::for_species(name).unwrap();
            let rig = world_rig(canonical);
            assert_eq!(locomotion(name), Locomotion::Walk);
            assert_eq!(rig.walk_legs.len(), 2, "{name} split a toe or heel into another support");
            assert_eq!(rig.motion_limbs.iter().filter(|l| l.role == Role::Arm).count(), arms);
            for part in &canonical.anatomy {
                if ["Foot /", "Foot claw /", "Heel claw /", "Thigh /", "Leg /"].iter().any(|p| part.name.starts_with(p)) { continue; }
                for v in part.vertices.clone() {
                    assert_eq!(rig.joint_indices[v], canonical.joint_indices[v], "{name} {} vertex {v}", part.name);
                    assert_eq!(rig.joint_weights[v], canonical.joint_weights[v]);
                }
            }
            let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
            let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
            let (lo, hi) = rig.locomotion_bounds();
            for tick in 0..64 {
                rig.sample_locomotion(tick as f32/20., tick as f32*TAU/64., 1., &mut pose);
                gait::assert_supports(rig, &pose);
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                let mut floor = f32::INFINITY;
                for v in 0..rig.neutral.positions.len() {
                    let p = rig.skin_point(v, &matrices); floor = floor.min(p.y);
                    assert!(p.y >= -0.00001, "{name} vertex {v} walks through floor");
                    assert!(p.cmpge(lo).all() && p.cmple(hi).all(), "{name} leaves locomotion bounds");
                }
                assert!(floor < 0.005, "{name} loses ground contact: {floor}");
            }
        }
    }

    #[test]
    fn magnet_skin_preserves_mechanical_weights_and_intentional_floating() {
        for name in ["MAGNEMITE","MAGNETON"] {
            let canonical=crate::species_rig::for_species(name).unwrap();let rig=world_rig(canonical);
            assert_eq!(locomotion(name),Locomotion::Float);assert!(battle_hovers(name));assert!(rig.walk_legs.is_empty());assert!(rig.motion_limbs.is_empty());assert_eq!(rig.joints.len(),canonical.joints.len());
            assert_eq!(rig.joint_indices,canonical.joint_indices);assert_eq!(rig.joint_weights,canonical.joint_weights);
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();let height=rig.neutral_bounds.1.y-rig.neutral_bounds.0.y;
            for tick in 0..64 {
                rig.sample_locomotion(tick as f32/32.,tick as f32,1.,&mut pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();
                for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);assert!(p.y>height*0.14,"{name} lost floating clearance");assert!(p.cmpge(lo).all() && p.cmple(hi).all(),"{name} left locomotion culling bounds");}
            }
        }
    }

    #[test]
    fn gas_pair_skin_preserves_authored_weights_and_intentional_floating() {
        for name in ["KOFFING","WEEZING"] {
            let canonical=crate::species_rig::for_species(name).unwrap();let rig=world_rig(canonical);
            assert_eq!(locomotion(name),Locomotion::Float);assert!(battle_hovers(name));assert!(rig.walk_legs.is_empty());assert!(rig.motion_limbs.is_empty());assert_eq!(rig.joints.len(),canonical.joints.len());
            assert_eq!(rig.joint_indices,canonical.joint_indices);assert_eq!(rig.joint_weights,canonical.joint_weights);
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];let (lo,hi)=rig.locomotion_bounds();let height=rig.neutral_bounds.1.y-rig.neutral_bounds.0.y;
            for tick in 0..64 {
                rig.sample_locomotion(tick as f32/32.,tick as f32,1.,&mut pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();
                for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);assert!(p.y>height*0.14,"{name} lost floating clearance");assert!(p.cmpge(lo).all() && p.cmple(hi).all(),"{name} left locomotion culling bounds");}
            }
        }
    }

    #[test]
    fn pokemon_motion_bivalves_keep_authored_valves_while_floating() {
        for name in ["SHELLDER","CLOYSTER"] {
            let canonical=crate::species_rig::for_species(name).unwrap();
            let rig=world_rig(canonical);
            assert_eq!(locomotion(name),Locomotion::Float);
            assert!(rig.walk_legs.is_empty());
            assert_eq!(rig.joints.len(),canonical.joints.len());
            for part in &rig.anatomy {
                if part.name.starts_with("Eye white /") || part.name.starts_with("Pupil /") || part.name.starts_with("Teeth /") {
                    for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[1,0,0,0],"{name} {}",part.name);}
                }
                if part.name.starts_with("Shell /") {
                    let owner=if name=="SHELLDER" || part.name.ends_with("-1") {2} else {3};
                    assert!(part.vertices.clone().any(|v|rig.joint_indices[v][0]==owner && rig.joint_weights[v][0]>0.99));
                }
            }
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];
            let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];
            let (lo,hi)=rig.locomotion_bounds();
            for tick in 0..64 {
                rig.sample_locomotion(tick as f32/32.,tick as f32,1.,&mut pose);
                rig.skin_matrices_into(&pose,&mut matrices).unwrap();
                assert!(pose[0].translation.y>0.1);
                for v in 0..rig.neutral.positions.len() {
                    let p=rig.skin_point(v,&matrices);
                    assert!(p.y>0.1,"{name} lost swimming clearance");
                    assert!(p.cmpge(lo).all() && p.cmple(hi).all(),"{name} left locomotion culling bounds");
                }
            }
        }
    }

    #[test]
    fn pokemon_motion_larvae_have_segment_waves_and_independent_prolegs() {
        for name in ["CATERPIE", "WEEDLE"] {
            let canonical = crate::species_rig::for_species(name).unwrap();
            let rig = world_rig(canonical);
            assert_eq!(rig.crawl_segments.len(), 8, "{name}");
            assert_eq!(rig.walk_legs.len(), if name=="CATERPIE" {6} else {14}, "{name}");
            let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
            let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
            let mut seams=Vec::new();
            if name=="CATERPIE" {
                let a=rig.anatomy.iter().find(|p|p.name.starts_with("Body /")).unwrap();
                let b=rig.anatomy.iter().find(|p|p.name.starts_with("Belly /")).unwrap();
                let left:std::collections::BTreeMap<_,_>=a.vertices.clone().map(|i|(rig.neutral.positions[i].map(f32::to_bits),i)).collect();
                let common:std::collections::BTreeMap<_,_>=b.vertices.clone().filter_map(|i|left.get(&rig.neutral.positions[i].map(f32::to_bits)).map(|&j|(rig.neutral.positions[i].map(f32::to_bits),(j,i)))).collect();
                assert_eq!(common.len(),114);seams.extend(common.into_values());
            }
            for tick in 0..64 {
                rig.sample_locomotion(0.2, tick as f32 * TAU / 64., 1., &mut pose);
                gait::assert_supports(rig, &pose);
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                assert!(
                    (0..rig.neutral.positions.len())
                        .all(|v| rig.skin_point(v, &matrices).y >= -0.00001),
                    "{name} crawled through the floor"
                );
                for &(a,b) in &seams {assert!(rig.skin_point(a,&matrices).distance(rig.skin_point(b,&matrices))<0.000001,"{name} world crawl opened the material seam");}
                let floor=(0..rig.neutral.positions.len()).map(|v|rig.skin_point(v,&matrices).y).fold(f32::INFINITY,f32::min);
                assert!(floor<(rig.neutral_bounds.1.y-rig.neutral_bounds.0.y)*0.005,"{name} world crawl lost ground contact: {floor}");
                let front = &rig.crawl_segments[0];
                let back = rig.crawl_segments.last().unwrap();
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
            ("Hip armor / rounded coxa", Role::Leg),
            ("Greave / flared lower-leg armor", Role::Leg),
            ("scizor/hip_left", Role::Leg),
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

#[path = "pokemon_travel.rs"]
mod travel;

#[path = "pokemon_ladybird_flight.rs"]
mod ladybird_flight;

#[path = "pokemon_furret_walk.rs"]
mod furret_walk;

#[path = "pokemon_fish_fins.rs"]
mod fish_fins;
