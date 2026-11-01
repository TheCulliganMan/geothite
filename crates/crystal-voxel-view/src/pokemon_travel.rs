//! Reviewed travel styles for all 251 species. Battle clips stay authored.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Style {
    Walk,
    Heavy,
    Scurry,
    Trot,
    Bound,
    Toad,
    Hop,
    ClusterHop,
    Waddle,
    Crawl,
    Scuttle,
    Slither,
    Slide,
    Burrow,
    Roll,
    Spin,
    BirdRun,
    Swim,
    Flop,
    Float,
    Fly,
    Glide,
}

pub(super) fn style(name: &str) -> Style {
    use Style::*;
    match name {
        "pidgey" | "pidgeotto" | "pidgeot" | "spearow" | "fearow" | "zubat" | "golbat"
        | "crobat" | "butterfree" | "beedrill" | "venomoth" | "aerodactyl" | "charizard"
        | "articuno" | "zapdos" | "moltres" | "dragonite" | "noctowl" | "ledyba" | "ledian"
        | "yanma" | "togetic" | "murkrow" | "skarmory" | "lugia" | "ho_oh" | "celebi" => Fly,
        "gligar" | "mantine" | "jumpluff" | "hoppip" | "skiploom" => Glide,
        "horsea" | "seadra" | "kingdra" | "gyarados"
        | "chinchou" | "lanturn" | "qwilfish" => Swim,
        "cloyster" | "shellder" | "mew" | "gastly" | "haunter" | "misdreavus" | "magnemite"
        | "magneton" | "geodude" | "koffing" | "weezing" | "porygon" | "porygon2" | "tentacool"
        | "tentacruel" | "staryu" | "starmie" => Float,
        "ekans" | "arbok" | "onix" | "steelix" | "dratini" | "dragonair" | "dunsparce" => Slither,
        "bulbasaur" | "ivysaur" | "venusaur" => Toad,
        "exeggcute" => ClusterHop,
        "magikarp" | "goldeen" | "seaking" | "remoraid" => Flop,
        "poliwag" | "politoed" | "natu" | "sentret" | "kangaskhan" | "sunkern" | "cleffa"
        | "igglybuff" | "pichu" | "togepi" | "azumarill" | "marill" | "oddish" | "bellsprout"
        | "weepinbell" | "victreebel" | "pineco" | "forretress" | "kakuna" | "metapod"
        | "pupitar" | "aipom" | "venonat" | "raticate" | "nidorina" | "hoothoot" => Hop,
        "nidoran_f" | "nidoran_m" | "nidorino" | "rattata" | "eevee" | "jolteon" | "flareon"
        | "espeon" | "umbreon" | "vulpix" | "ninetales" | "suicune" | "raikou" | "entei"
        | "arcanine" | "growlithe" | "houndour" | "houndoom" => Bound,
        "tauros" | "ponyta" | "rapidash" | "stantler" | "girafarig" | "mareep" | "vaporeon"
        | "chikorita" | "bayleef" | "meganium" => Trot,
        "meowth" | "persian" | "furret" | "swinub" | "sandshrew" | "sandslash" => Scurry,
        "snorlax" | "tyranitar" | "blastoise" | "feraligatr" | "golem" | "graveler" | "rhydon"
        | "rhyhorn" | "donphan" | "piloswine" | "ursaring" | "slowbro" | "slowking"
        | "exeggutor" | "nidoqueen" | "nidoking" | "machamp" | "ampharos" => Heavy,
        "psyduck" | "squirtle" | "wartortle" | "totodile" | "croconaw" | "wooper" | "quagsire"
        | "slowpoke" | "miltank" | "chansey" | "blissey" | "delibird" | "xatu"
        | "clefairy" | "clefable" | "jigglypuff" | "wigglytuff" | "gloom" | "vileplume"
        | "bellossom" | "corsola" | "wobbuffet" | "lickitung" | "teddiursa" | "phanpy"
        | "snubbull" | "flaaffy" | "tangela" => Waddle,
        "caterpie" | "weedle" | "shuckle" | "omanyte" | "omastar" => Crawl,
        "paras" | "parasect" | "spinarak" | "ariados" | "kabuto" | "krabby" | "kingler" => Scuttle,
        "lapras" | "seel" | "dewgong" | "grimer" | "muk" | "ditto" | "slugma" | "magcargo" | "jynx"
        | "octillery" => Slide,
        "diglett" | "dugtrio" => Burrow,
        "voltorb" | "electrode" => Roll,
        "hitmontop" => Spin,
        "doduo" | "dodrio" => BirdRun,
        "abra" | "alakazam" | "charmander" | "charmeleon" | "cubone" | "drowzee" | "electabuzz"
        | "elekid" | "farfetch_d" | "gengar" | "golduck" | "granbull" | "heracross"
        | "hitmonchan" | "hitmonlee" | "hypno" | "kabutops" | "kadabra" | "larvitar"
        | "machoke" | "machop" | "magby" | "magmar" | "mankey" | "marowak" | "mewtwo"
        | "mr__mime" | "pikachu" | "pinsir" | "poliwhirl" | "poliwrath" | "primeape" | "raichu"
        | "scizor" | "scyther" | "smeargle" | "smoochum" | "sneasel" | "sudowoodo" | "sunflora"
        | "cyndaquil" | "quilava" | "typhlosion" | "tyrogue" => Walk,
        "unown" => Float,
        name if name.starts_with("unown_") => Float,
        _ => panic!("species lacks a reviewed travel style: {name}"),
    }
}

impl Style {
    pub(super) fn stance(self) -> f32 {
        match self {
            Self::Heavy => 0.72,
            Self::Toad => 0.68,
            Self::Scurry | Self::Bound | Self::BirdRun => 0.52,
            Self::Scuttle => 0.66,
            _ => 0.60,
        }
    }
    pub(super) fn stride(self) -> f32 {
        match self {
            Self::Heavy => 0.72,
            Self::Scuttle => 0.58,
            Self::Toad => 0.75,
            Self::Bound | Self::BirdRun => 1.10,
            Self::Scurry => 0.82,
            _ => 1.,
        }
    }
    pub(super) fn frequency(self) -> f32 {
        match self {
            Self::Heavy => 2.0,
            Self::Slither => 1.6,
            Self::Hop | Self::ClusterHop | Self::Toad => 2.2,
            Self::Crawl | Self::Slide => 1.8,
            Self::Scurry | Self::Scuttle => 3.5,
            _ => 3.0,
        }
    }
    pub(super) fn hop_height(self, name: &str) -> f32 {
        match self {
            Self::Hop => {
                if name == "kangaskhan" {
                    0.10
                } else if matches!(name, "raticate" | "nidorina") {
                    0.08
                } else {
                    0.13
                }
            }
            Self::Bound => 0.055,
            Self::Toad => {
                if name == "venusaur" {
                    0.018
                } else {
                    0.065
                }
            }
            Self::ClusterHop => 0.15,
            _ => 0.,
        }
    }
}

/// Smooth takeoff and landing, with a real grounded interval each cycle.
pub(super) fn airborne(phase: f32) -> f32 {
    let p = (phase / TAU).rem_euclid(1.);
    if p < 0.60 {
        0.
    } else {
        ((p - 0.60) / 0.40 * std::f32::consts::PI).sin().powi(2)
    }
}

/// Give the fixed lower coils a continuous, blended travelling wave. The
/// canonical asset and its battle bindings remain untouched in the registry.
pub(super) fn build(rig: &mut SpeciesRig) {
    let kind = style(rig.species.name());
    let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
    let swimming_wave = rig.species.name() == "gyarados";
    if kind == Style::Flop {
        fish_fins::build(rig);
    }
    if matches!(rig.species.name(), "seel" | "dewgong") {
        // The canonical resting paddle tips were fixed on the presentation
        // root. A world pull needs the complete fan and its folds on one hinge.
        for part in &rig.anatomy {
            if !part.name.starts_with("Flipper") { continue; }
            let side = part.name.split_ascii_whitespace().find_map(|word|word.parse::<i32>().ok())
                .expect("authored seal flipper side");
            let suffix = if side < 0 { "/flipper_left" } else { "/flipper_right" };
            let owner = rig.joints.iter().position(|joint|joint.name.ends_with(suffix)).unwrap();
            for v in part.vertices.clone() {
                rig.joint_indices[v] = [owner as u16,0,0,0];
                rig.joint_weights[v] = [1.,0.,0.,0.];
            }
        }
    }
    if rig.species.name() == "lapras" {
        for part in &rig.anatomy {
            if !part.name.to_ascii_lowercase().contains("flipper") { continue; }
            let rear=part.name.to_ascii_lowercase().contains("rear");
            let side: i32=part.name.split_ascii_whitespace().last().unwrap().parse().unwrap();
            let suffix=match (rear,side<0) {
                (false,true)=>"/flipper_left",(false,false)=>"/flipper_right",
                (true,true)=>"/rear_left",(true,false)=>"/rear_right",
            };
            let owner=rig.joints.iter().position(|joint|joint.name.ends_with(suffix)).unwrap();
            for v in part.vertices.clone() {
                rig.joint_indices[v]=[owner as u16,0,0,0];
                rig.joint_weights[v]=[1.,0.,0.,0.];
            }
        }
    }
    if swimming_wave || kind == Style::Slither && !matches!(rig.species.name(), "onix" | "steelix") {
        const STRIPS: usize = 12;
        let start = rig.joints.len();
        let axis = if swimming_wave { 1 } else { 2 };
        let lo = rig.neutral_bounds.0[axis];
        let span = (rig.neutral_bounds.1[axis] - lo).max(height * 0.01);
        for i in 0..STRIPS {
            let mut pivot = Vec3::ZERO;
            pivot[axis] = lo + span * i as f32 / (STRIPS - 1) as f32;
            rig.joints.push(Joint {
                name: format!("{}/{}_{i}", rig.species.name(), if swimming_wave { "swim_wave" } else { "travel_wave" }),
                parent: Some(0),
                bind: Transform::from_translation(pivot),
                inverse_bind: Mat4::from_translation(-pivot),
            });
        }
        for v in 0..rig.neutral.positions.len() {
            let p = ((rig.neutral.positions[v][axis] - lo) / span * (STRIPS - 1) as f32)
                .clamp(0., (STRIPS - 1) as f32);
            let a = (p.floor() as usize).min(STRIPS - 2);
            let t = p - a as f32;
            let mut contributions = Vec::with_capacity(5);
            for (&joint, weight) in rig.joint_indices[v].iter().zip(rig.joint_weights[v]) {
                if weight == 0. {
                    continue;
                }
                if joint == 0 {
                    contributions.push(((start + a) as u16, weight * (1. - t)));
                    contributions.push(((start + a + 1) as u16, weight * t));
                } else {
                    contributions.push((joint, weight));
                }
            }
            contributions.sort_by(|a, b| b.1.total_cmp(&a.1));
            let total: f32 = contributions.iter().take(4).map(|p| p.1).sum();
            rig.joint_indices[v] = [0; 4];
            rig.joint_weights[v] = [0.; 4];
            for (slot, &(joint, weight)) in contributions.iter().take(4).enumerate() {
                rig.joint_indices[v][slot] = joint;
                rig.joint_weights[v][slot] = weight / total;
            }
        }
    }
    if matches!(rig.species.name(), "slugma" | "magcargo") {
        // Only the soft foot receives peristaltic skin. The authored neck,
        // eyes and Magcargo shell keep their existing rigid owners.
        let start = rig.joints.len();
        let lo = rig.neutral_bounds.0.z;
        let span = (rig.neutral_bounds.1.z - lo).max(height * 0.01);
        for strip in 0..10 {
            let pivot = Vec3::Z * (lo + span * strip as f32 / 9.);
            rig.joints.push(Joint { name: format!("{}/slide_foot_{strip}", rig.species.name()),
                parent: Some(0), bind: Transform::from_translation(pivot),
                inverse_bind: Mat4::from_translation(-pivot) });
        }
        for v in 0..rig.neutral.positions.len() {
            let point = Vec3::from_array(rig.neutral.positions[v]);
            let soft = rig.joint_indices[v].iter().zip(rig.joint_weights[v])
                .all(|(&joint, weight)| weight == 0. || joint == 0
                    || rig.joints[joint as usize].name.ends_with("/mantle"));
            let blend = (1. - (point.y - rig.neutral_bounds.0.y) / (height * 0.32)).clamp(0., 1.);
            if !soft || blend == 0. { continue; }
            let position = ((point.z - lo) / span * 9.).clamp(0., 9.);
            let a = (position.floor() as usize).min(8);
            let t = position - a as f32;
            let mut weights: Vec<_> = rig.joint_indices[v].iter().copied()
                .zip(rig.joint_weights[v]).map(|(joint,w)|(joint,w*(1.-blend))).collect();
            weights.extend([((start+a) as u16,blend*(1.-t)),((start+a+1) as u16,blend*t)]);
            weights.sort_by(|a,b|b.1.total_cmp(&a.1));
            let total: f32 = weights.iter().take(4).map(|p|p.1).sum();
            for (slot,(joint,w)) in weights.into_iter().take(4).enumerate() {
                rig.joint_indices[v][slot]=joint;rig.joint_weights[v][slot]=w/total;
            }
        }
    }
    if kind == Style::Spin {
        rig.ground_vertices = (0..rig.neutral.positions.len())
            .filter(|&v| rig.neutral.positions[v][1] <= rig.neutral_bounds.0.y + height * 0.003)
            .collect();
    } else if matches!(kind, Style::Roll | Style::Flop) || rig.ground_vertices.is_empty() {
        rig.ground_vertices = (0..rig.neutral.positions.len())
            .filter(|&v| {
                matches!(kind, Style::Roll | Style::Flop)
                    || rig.neutral.positions[v][1] <= rig.neutral_bounds.0.y + height * 0.4
            })
            .collect();
    }
    if rig.walk_legs.is_empty() {
        rig.walk_cycle = height
            * match kind {
                Style::Slither => 0.9,
                Style::Roll => std::f32::consts::PI,
                Style::Hop | Style::ClusterHop => 0.65,
                _ => 0.5,
            };
    }
}

pub(super) fn prepare(rig: &SpeciesRig, phase: f32, weight: f32, pose: &mut [Transform]) {
    let name = rig.species.name();
    let kind = style(name);
    let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
    if kind == Style::Slither {
        // Suppress combat/idle rotations during travel; lateral waves keep the
        // lower coil on the floor instead of tipping the whole snake upright.
        for (p, j) in pose.iter_mut().zip(&rig.joints) {
            p.rotation = p.rotation.slerp(j.bind.rotation, weight).normalize();
        }
        let stone = matches!(name, "onix" | "steelix");
        let wave = |i: usize| {
            let position = rig.joints[i]
                .inverse_bind
                .inverse()
                .transform_point3(Vec3::ZERO);
            let delay = if stone {
                i as f32 * 0.62
            } else {
                position.z / height * 4.0 + position.y / height * 2.5
            };
            Vec3::X * height * (if stone { 0.04 } else { 0.075 }) * (phase - delay).sin() * weight
        };
        for i in (0..rig.joints.len()).rev() {
            let delta = wave(i);
            let local = if let Some(parent) = rig.joints[i].parent {
                let rotation = gait::world(rig, pose, parent)
                    .to_scale_rotation_translation()
                    .1;
                rotation.inverse() * (delta - wave(parent))
            } else {
                delta
            };
            pose[i].translation += local;
        }
    } else if kind == Style::Roll {
        let pivot = (rig.neutral_bounds.0 + rig.neutral_bounds.1) * 0.5;
        let rotation = Quat::from_rotation_x(phase);
        pose[0].rotation = rotation * pose[0].rotation;
        pose[0].translation += pivot - rotation * pivot;
    } else if rig.joints.len() > 1 && role(&rig.joints[1].name) == Role::Body {
        if kind == Style::Scuttle {
            // Wide, low shells cannot replay a combat torso rock while their
            // independently planted legs hold the animal above the ground.
            pose[1].rotation = pose[1]
                .rotation
                .slerp(rig.joints[1].bind.rotation, weight)
                .normalize();
        }
        let roll = match kind {
            Style::Waddle => 0.045,
            Style::Heavy => 0.012,
            Style::Scurry => 0.018,
            _ => 0.,
        } * phase.sin()
            * weight;
        let pitch = match kind {
            Style::Toad => 0.045,
            Style::Bound | Style::BirdRun => 0.025,
            _ => 0.,
        } * phase.sin()
            * weight;
        let yaw = if kind == Style::Slide {
            0.035 * phase.sin() * weight
        } else {
            0.
        };
        pose[1].rotation = (pose[1].rotation
            * Quat::from_euler(bevy::prelude::EulerRot::XYZ, pitch, yaw, roll))
        .normalize();
    }
    if matches!(name, "slugma" | "magcargo") {
        for (i,joint) in rig.joints.iter().enumerate() {
            if let Some(strip) = joint.name.rsplit('/').next().unwrap().strip_prefix("slide_foot_") {
                let strip: usize = strip.parse().expect("soft-foot strip");
                let clock = phase - strip as f32 * TAU / 9.;
                pose[i].rotation = Quat::from_rotation_z(clock.sin() * 0.12 * weight);
                pose[i].translation.z += height * clock.sin() * 0.025 * weight;
                pose[i].translation.y += height * (1. - clock.cos()) * 0.012 * weight;
            }
        }
    }
    if name == "lapras" {
        for (i,joint) in rig.joints.iter().enumerate() {
            let label=joint.name.rsplit('/').next().unwrap();
            let rear=matches!(label,"rear_left"|"rear_right");
            if !rear && !matches!(label,"flipper_left"|"flipper_right") { continue; }
            let side=if label.ends_with("left") {-1.} else {1.};
            let clock=phase-if rear {0.8} else {0.};
            let rotation=Quat::from_rotation_y(side*clock.sin()*if rear {0.12} else {0.35}*weight)
                *Quat::from_rotation_z(side*(1.-clock.cos())*if rear {0.03} else {0.06}*weight);
            pose[i].rotation=(pose[i].rotation*rotation).normalize();
        }
    }
    if matches!(name, "seel" | "dewgong") {
        // Seals pull with paired foreflippers, then flex the trunk and tail.
        // Their authored flippers are not humanoid arms or walking feet.
        for (i, joint) in rig.joints.iter().enumerate() {
            let suffix = joint.name.rsplit('/').next().unwrap();
            let rotation = match suffix {
                "body" => Quat::from_rotation_x(phase.sin() * 0.10 * weight),
                "head" => Quat::from_rotation_x(-phase.sin() * 0.06 * weight),
                "tail_base" => Quat::from_rotation_x((phase - 0.7).sin() * 0.18 * weight),
                "tail_tip" => Quat::from_rotation_x((phase - 1.2).sin() * 0.14 * weight),
                "flipper_left" | "flipper_right" => {
                    let side = if suffix.ends_with("left") { -1. } else { 1. };
                    Quat::from_rotation_y(side * phase.sin() * 0.48 * weight)
                        * Quat::from_rotation_z(side * (1. + phase.cos()) * 0.12 * weight)
                }
                _ => continue,
            };
            pose[i].rotation = (pose[i].rotation * rotation).normalize();
        }
    }
    if name == "octillery" {
        for (i, joint) in rig.joints.iter().enumerate() {
            if let Some(tentacle) = joint
                .name
                .rsplit('/')
                .next()
                .unwrap()
                .strip_prefix("tentacle_")
            {
                let order: usize = tentacle.parse().expect("authored tentacle index");
                pose[i].rotation = (pose[i].rotation
                    * Quat::from_rotation_y(
                        (phase - order as f32 * TAU / 8.).sin() * 0.07 * weight,
                    ))
                .normalize();
            }
        }
    }
    if kind == Style::Scuttle {
        for limb in rig
            .motion_limbs
            .iter()
            .filter(|limb| limb.role == Role::Arm)
        {
            let joint = &rig.joints[limb.joint];
            let side = if joint.name.contains("left") { -1. } else { 1. };
            // Crab pincers sweep above the ground; a human shoulder swing
            // pitches their tips below the feet and falsely lifts the animal.
            pose[limb.joint].rotation =
                (pose[limb.joint].rotation.slerp(joint.bind.rotation, weight)
                    * Quat::from_rotation_y(side * phase.sin() * 0.035 * weight))
                .normalize();
        }
    }
}

pub(super) fn finish(rig: &SpeciesRig, phase: f32, weight: f32, pose: &mut [Transform]) {
    let name = rig.species.name();
    let kind = style(name);
    let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
    if matches!(name, "seel" | "dewgong" | "lapras") {
        let mut matrices = [Mat4::IDENTITY;256];
        rig.skin_matrices_into(pose,&mut matrices[..rig.joints.len()]).unwrap();
        let belly = rig.anatomy.iter().filter(|part|part.name.starts_with(
            if name=="lapras" {"Belly /"} else {"Body /"}))
            .flat_map(|part|part.vertices.clone()).map(|v|rig.skin_point(v,&matrices).y)
            .fold(f32::INFINITY,f32::min);
        pose[0].translation.y -= belly;
        // Clear the head and paddles independently instead of lifting the
        // weight-bearing belly when a pulling tip or low muzzle reaches land.
        for (i,joint) in rig.joints.iter().enumerate() {
            if !joint.name.ends_with("/head") && !joint.name.ends_with("/flipper_left")
                && !joint.name.ends_with("/flipper_right")
                && !(name=="lapras" && (joint.name.ends_with("/rear_left")
                    || joint.name.ends_with("/rear_right"))) { continue; }
            rig.skin_matrices_into(pose,&mut matrices[..rig.joints.len()]).unwrap();
            let lift = (0..rig.neutral.positions.len()).filter_map(|v| {
                let weight: f32 = rig.joint_indices[v].iter().zip(rig.joint_weights[v])
                    .filter(|(owner,_)| {
                        let mut owner=**owner as usize;
                        loop {
                            if owner==i { return true; }
                            let Some(parent)=rig.joints[owner].parent else { return false; };
                            owner=parent;
                        }
                    }).map(|(_,w)|w).sum();
                (weight>0.01).then(||(-rig.skin_point(v,&matrices).y/weight).max(0.))
            }).fold(0.0_f32,f32::max);
            let parent = joint.parent.expect("aquatic appendage parent");
            let rotation = gait::world(rig,pose,parent).to_scale_rotation_translation().1;
            pose[i].translation += rotation.inverse() * (Vec3::Y*lift);
        }
    }
    if kind == Style::Spin {
        // Raised legs stay on the authored spin skin. Turn around the actual
        // crown contact rather than the center displaced by its curved tail.
        let (lo, hi) = point_bounds(rig.ground_vertices.iter()
            .map(|&vertex| Vec3::from_array(rig.neutral.positions[vertex])));
        let pivot = (lo + hi) * 0.5;
        let rotation = Quat::from_rotation_y(phase);
        pose[0].rotation = rotation * pose[0].rotation;
        pose[0].translation += pivot - rotation * pivot;
    }
    if kind == Style::ClusterHop {
        for (i, joint) in rig.joints.iter().enumerate() {
            if let Some(egg) = joint.name.rsplit('/').next().unwrap().strip_prefix("egg_") {
                let egg: usize = egg.parse().expect("authored egg index");
                pose[i].translation.y += height
                    * kind.hop_height(name)
                    * airborne(phase - egg as f32 * TAU / 6.)
                    * weight;
            }
        }
    } else {
        pose[0].translation.y += height * kind.hop_height(name) * airborne(phase) * weight;
    }
    if kind == Style::Burrow {
        // Keep the earth mound rooted; only the exposed body peeks and dips.
        for (i, joint) in rig.joints.iter().enumerate().skip(1) {
            let mole = joint
                .name
                .rsplit('/')
                .next()
                .unwrap()
                .strip_prefix("mole_")
                .map(|n| n.parse::<usize>().expect("authored mole index"));
            if role(&joint.name) == Role::Body || mole.is_some() {
                let clock = phase - mole.unwrap_or(0) as f32 * TAU / 3.;
                pose[i].translation.y += height * 0.006 * (1. - clock.cos()) * weight;
            }
        }
    }
}

/// A stranded fish rests on its side, flexes, and briefly clears the floor.
/// Settle the entire rolled surface before adding the airborne arc so fins
/// cannot penetrate the floor and grounding cannot erase the actual hop.
pub(super) fn flop(rig: &SpeciesRig, phase: f32, weight: f32, pose: &mut [Transform]) {
    let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
    let pulse = airborne(phase);
    let rock = phase.sin() * 0.30 * weight;
    let roll = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2 + rock);
    let pivot = (rig.neutral_bounds.0 + rig.neutral_bounds.1) * 0.5;
    pose[0].rotation = roll * pose[0].rotation;
    pose[0].translation += pivot - roll * pivot;
    for (i, joint) in rig.joints.iter().enumerate().skip(1) {
        let label = joint.name.rsplit('/').next().unwrap();
        let flex = if label == "tail" { -0.65*(1.-phase.cos())*0.5*weight } else { 0. };
        pose[i].rotation = (pose[i].rotation
            * Quat::from_rotation_y(flex)).normalize();
    }
    for &(joint, fold) in &rig.flop_folds {
        let flutter=Quat::from_rotation_z(-rock+(phase+joint as f32*0.9).sin()*0.08*weight);
        pose[joint].rotation=(flutter*fold*pose[joint].rotation).normalize();
    }
    let mut matrices = [Mat4::IDENTITY; 256];
    rig.skin_matrices_into(pose, &mut matrices[..rig.joints.len()]).unwrap();
    let floor = rig.ground_vertices.iter()
        .map(|&v| rig.skin_point(v, &matrices).y)
        .fold(f32::INFINITY, f32::min);
    pose[0].translation.y += height * 0.16 * pulse * weight - floor;
}

pub(super) fn swim(rig: &SpeciesRig, phase: f32, weight: f32, pose: &mut [Transform]) {
    if style(rig.species.name()) != Style::Swim {
        return;
    }
    swim_strokes(rig, phase, weight, pose);
}

/// Free-water articulation, separate from the grounded tentacle support path.
pub(super) fn water_tentacles(rig: &SpeciesRig, phase: f32, weight: f32, pose: &mut [Transform]) {
    let jelly = matches!(rig.species.name(), "tentacool" | "tentacruel");
    let octopus = rig.species.name() == "octillery";
    let nautilus = matches!(rig.species.name(), "omanyte" | "omastar");
    if !(jelly || octopus || nautilus) { return; }
    for (i,joint) in rig.joints.iter().enumerate().skip(1) {
        let label = joint.name.rsplit('/').next().unwrap();
        if jelly && label == "bell" {
            let pulse = phase.cos() * weight;
            pose[i].scale *= Vec3::new(1. + pulse*0.035, 1. - pulse*0.025, 1. + pulse*0.035);
        }
        let Some((_, suffix)) = label.split_once("tentacle_") else { continue; };
        let order: usize = suffix.split('_').next().unwrap().parse().expect("authored tentacle number");
        let bend = label.ends_with("_bend");
        let clock = phase - order as f32 * 0.55 - if bend { 0.65 } else { 0. };
        let rotation = if jelly {
            Quat::from_euler(bevy::prelude::EulerRot::XYZ,
                clock.sin() * if bend {0.22} else {0.13} * weight,
                clock.cos()*0.08*weight, 0.)
        } else {
            Quat::from_euler(bevy::prelude::EulerRot::XYZ,
                clock.cos()*0.14*weight, clock.sin()*0.18*weight, 0.)
        };
        pose[i].rotation = (pose[i].rotation * rotation).normalize();
    }
}

pub(super) fn swim_strokes(rig: &SpeciesRig, phase: f32, weight: f32, pose: &mut [Transform]) {
    let lean = if rig.species.name() == "gyarados" { 0.35 }
        else if rig.walk_legs.len() == 2 { 0.55 } else { 0. };
    // Lean into propulsion; at rest the authored tread-water pose returns.
    // Root rotation also carries the actual skinned rider contact point.
    pose[0].rotation = (Quat::from_rotation_x(lean * weight) * pose[0].rotation).normalize();
    if let Some(head) = rig.joints.iter().position(|j| j.name.rsplit('/').next() == Some("head")) {
        let parent = rig.joints[head].parent.unwrap_or(0);
        let rotation = gait::world(rig, pose, parent).to_scale_rotation_translation().1;
        let counter = rotation.inverse() * Quat::from_rotation_x(-lean * weight * 0.85) * rotation;
        pose[head].rotation = (counter * pose[head].rotation).normalize();
    }
    if rig.species.name() == "gyarados" {
        let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
        // Apply a delayed lateral wave to the fixed coil and existing neck
        // and tail branches. Parent-relative offsets keep attached assemblies
        // together, and blended strips bend the previously rigid body skin.
        let displacement = |i: usize| {
            let p = rig.joints[i].inverse_bind.inverse().transform_point3(Vec3::ZERO);
            Vec3::X * height * 0.045 * (phase - p.y / height * 4.5).sin() * weight
        };
        for i in (0..rig.joints.len()).rev() {
            let delta = displacement(i);
            let local = if let Some(parent) = rig.joints[i].parent {
                let rotation = gait::world(rig, pose, parent).to_scale_rotation_translation().1;
                rotation.inverse() * (delta - displacement(parent))
            } else { delta };
            pose[i].translation += local;
        }
    }
    for limb in &rig.motion_limbs {
        let joint = &rig.joints[limb.joint];
        let side = if joint.name.contains("left") { -1. } else { 1. };
        let rotation = match limb.role {
            Role::Tail => Quat::from_rotation_y(phase.sin() * 0.18 * weight),
            Role::Arm => Quat::from_euler(bevy::prelude::EulerRot::XYZ,
                phase.cos() * 0.32 * weight, 0., side * phase.sin() * 0.20 * weight),
            Role::Wing => Quat::from_rotation_z(side * (phase + side * 0.35).sin() * 0.15 * weight),
            _ => Quat::IDENTITY,
        };
        pose[limb.joint].rotation = (pose[limb.joint].rotation * rotation).normalize();
    }
    // Land motion keeps legs out of `motion_limbs` because the IK solver owns
    // them. Water has no planting solver, so stroke each top-level leg here.
    for (i, joint) in rig.joints.iter().enumerate().skip(1) {
        if role(&joint.name) != Role::Leg { continue; }
        let mut ancestor = joint.parent;
        let mut inherited = false;
        while let Some(parent) = ancestor {
            if role(&rig.joints[parent].name) == Role::Leg { inherited = true; break; }
            ancestor = rig.joints[parent].parent;
        }
        if inherited { continue; }
        let pivot = joint.inverse_bind.inverse().transform_point3(Vec3::ZERO);
        let side = if pivot.x < 0. { -1. } else { 1. };
        let rear = if rig.walk_legs.len() > 2 && pivot.z < 0. { std::f32::consts::PI } else { 0. };
        let clock = phase + side * std::f32::consts::FRAC_PI_2 + rear;
        pose[i].rotation = (pose[i].rotation * Quat::from_rotation_x(clock.sin() * 0.22 * weight)).normalize();
    }
    for (i, joint) in rig.joints.iter().enumerate().skip(1) {
        let label = joint.name.rsplit('/').next().unwrap();
        if !(label.contains("fin") || label.contains("flipper") || label.contains("fluke")
            || label.starts_with("pectoral") || label.starts_with("pelvic"))
            || rig.motion_limbs.iter().any(|limb| limb.joint == i) {
            continue;
        }
        if joint.parent.is_some_and(|p| {
            let parent = &rig.joints[p].name;
            parent.contains("fin") || parent.contains("flipper") || parent.contains("fluke")
        }) {
            continue;
        }
        let side = if label.contains("left") { -1. } else { 1. };
        pose[i].rotation = (pose[i].rotation
            * Quat::from_rotation_y(side * (phase + side * 0.35).sin() * 0.12 * weight))
        .normalize();
    }
}

#[cfg(test)]
pub(super) fn assert_contact(rig: &SpeciesRig, phase: f32, floor: f32) {
    let kind = style(rig.species.name());
    let p = (phase / TAU).rem_euclid(1.);
    let jumping = kind.hop_height(rig.species.name()) > 0.;
    // Hoppers and bounders are deliberately airborne during foot recovery.
    // The independent hop regression requires both takeoff and landing.
    if !jumping || p < kind.stance().min(0.60) {
        assert!(
            floor < 0.005,
            "{} lost stance contact: {floor}",
            rig.species.name()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swimming_paddlers_move_their_feet_not_only_their_arms() {
        for name in ["SQUIRTLE", "TOTODILE", "QUAGSIRE"] {
            let rig=world_rig(crate::species_rig::for_species(name).unwrap());
            let feet:Vec<_>=rig.anatomy.iter().filter(|p| role(&p.name)==Role::Leg)
                .flat_map(|p|p.vertices.clone()).collect();
            assert!(!feet.is_empty(), "{name} needs real leg surfaces for this regression");
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];
            let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];
            rig.sample_swimming(0.,0.,1.,&mut pose);
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            let first:Vec<_>=feet.iter().map(|&v|rig.skin_point(v,&matrices)).collect();
            rig.sample_swimming(0.,std::f32::consts::PI,1.,&mut pose);
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            let movement=feet.iter().zip(first).map(|(&v,p)|rig.skin_point(v,&matrices).distance(p))
                .fold(0.0_f32,f32::max);
            let height=rig.neutral_bounds.1.y-rig.neutral_bounds.0.y;
            assert!(movement>height*0.005,"{name} has no actual swimming foot movement");
        }
    }

    #[test]
    fn swimming_lean_keeps_the_head_looking_forward() {
        for name in ["SQUIRTLE", "TOTODILE", "BLASTOISE", "FERALIGATR", "GYARADOS"] {
            let rig = world_rig(crate::species_rig::for_species(name).unwrap());
            let head = rig.joints.iter().position(|j| j.name.rsplit('/').next() == Some("head")).unwrap();
            let mut idle = vec![Transform::IDENTITY; rig.joints.len()];
            let mut swimming = idle.clone();
            rig.sample_swimming(0., 0., 0., &mut idle);
            rig.sample_swimming(0., 0., 1., &mut swimming);
            assert!(idle[0].rotation.angle_between(swimming[0].rotation) > 0.30,
                "{name} lacks a swimming body lean");
            let a = gait::world(rig, &idle, head).transform_vector3(Vec3::Z).normalize();
            let b = gait::world(rig, &swimming, head).transform_vector3(Vec3::Z).normalize();
            assert!(a.angle_between(b) < 0.16, "{name} looks down while swimming");
        }
    }

    #[test]
    fn gyarados_swimming_wave_bends_the_body_without_changing_battle_skin() {
        let canonical = crate::species_rig::for_species("GYARADOS").unwrap();
        let rig = world_rig(canonical);
        assert_eq!(rig.joints.len(), canonical.joints.len() + 12);
        assert_eq!(rig.neutral.positions, canonical.neutral.positions);
        let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
        let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
        let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
        let body = rig.anatomy.iter().find(|p| p.name.starts_with("Body / ")).unwrap();
        let (lo, hi) = rig.locomotion_bounds();
        for frame in 0..64 {
            rig.sample_swimming(0., frame as f32 * TAU / 64., 1., &mut pose);
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            let mut low = f32::INFINITY;
            let mut high = f32::NEG_INFINITY;
            for v in body.vertices.clone() {
                let p = rig.skin_point(v, &matrices);
                let displacement = p.x - rig.neutral.positions[v][0];
                low = low.min(displacement);
                high = high.max(displacement);
                assert!(p.is_finite() && p.cmpge(lo).all() && p.cmple(hi).all());
            }
            assert!(high-low > height * 0.015, "body is still rigid at frame {frame}");
        }
        assert!(!canonical.joints.iter().any(|j| j.name.contains("swim_wave")));
    }

    #[test]
    fn swimming_strokes_articulate_fish_flippers_and_paddlers_without_land_motion() {
        for name in ["MAGIKARP", "GOLDEEN", "GYARADOS", "LAPRAS", "SEEL", "DEWGONG", "SQUIRTLE", "TOTODILE"] {
            let rig = world_rig(crate::species_rig::for_species(name).unwrap());
            let mut a = vec![Transform::IDENTITY; rig.joints.len()];
            let mut b = a.clone();
            let mut idle = a.clone();
            rig.sample_swimming(0., 0., 1., &mut a);
            rig.sample_swimming(0., TAU * 0.25, 1., &mut b);
            assert!(a.iter().skip(1).zip(&b[1..]).any(|(a,b)|
                a.rotation.angle_between(b.rotation) > 0.01), "{name} has no propulsion stroke");
            assert!(a[0].rotation.abs_diff_eq(b[0].rotation, 1e-6), "{name} replays a land roll");
            assert!((a[0].translation.y-b[0].translation.y).abs() < 1e-6,
                "{name} replays a grounded hop");
            rig.sample_swimming(0., 0.3, 0., &mut a);
            rig.sample_swimming(0., 4.1, 0., &mut b);
            rig.sample_into(Clip::Idle, 0., Playback::Loop, &mut idle).unwrap();
            for ((a,b),idle) in a.iter().zip(&b).zip(&idle) {
                assert!(a.translation.abs_diff_eq(b.translation,1e-6));
                assert!(a.rotation.abs_diff_eq(b.rotation,1e-6));
                assert!(a.rotation.abs_diff_eq(idle.rotation,1e-6), "{name} paddles at rest");
            }
            rig.sample_swimming(0., 0., 1., &mut a);
            rig.sample_swimming(0., TAU, 1., &mut b);
            for (a,b) in a.iter().zip(&b) {
                assert!(a.rotation.abs_diff_eq(b.rotation,1e-6), "{name} swim loop pops");
            }
        }
    }

    #[test]
    fn travel_magikarp_flops_on_its_side_then_lands_and_stops() {
        for name in ["MAGIKARP", "GOLDEEN", "SEAKING", "REMORAID"] {
        let canonical = crate::species_rig::for_species(name).unwrap();
        let rig = world_rig(canonical);
        assert_eq!(style(canonical.species.name()), Style::Flop);
        let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
        let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
        let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
        let mut rise = 0.0_f32;
        let mut landings = 0;
        let body_vertices=rig.anatomy.iter().filter(|part|part.name.starts_with("Body /"))
            .flat_map(|part|part.vertices.clone()).collect::<Vec<_>>();
        for tick in 0..64 {
            rig.sample_locomotion(0., tick as f32 * TAU / 64., 1., &mut pose);
            assert!(pose[0].rotation * Vec3::Y != Vec3::Y);
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            let floor = (0..rig.neutral.positions.len())
                .map(|v| rig.skin_point(v, &matrices).y)
                .fold(f32::INFINITY, f32::min);
            assert!(floor >= -height * 0.001, "fish penetrates floor");
            if (tick as f32/64.) < 0.60 {
                let body_floor=body_vertices.iter().map(|&v|rig.skin_point(v,&matrices).y)
                    .fold(f32::INFINITY,f32::min);
                assert!(body_floor < height*0.025,
                    "{name} stands on a fin during grounded phase {tick}: {body_floor}");
            }
            rise = rise.max(floor);
            if floor.abs() < height * 0.001 { landings += 1; }
        }
        assert!(rise > height * 0.14, "fish never leaves floor");
        assert!(landings > 32, "fish never rests between flops");
        rig.sample_locomotion(0., 0.3, 0., &mut pose);
        let stopped = pose.clone();
        rig.sample_locomotion(0., 4.5, 0., &mut pose);
        for (a, b) in pose.iter().zip(stopped) {
            assert!(a.translation.distance(b.translation) < 1e-6);
            assert!(a.rotation.abs_diff_eq(b.rotation, 1e-6));
        }
        assert!(battle_hovers(canonical.species.name()), "battle presentation changed");
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        let body_floor=rig.anatomy.iter().filter(|part|part.name.starts_with("Body /"))
            .flat_map(|part|part.vertices.clone()).map(|v|rig.skin_point(v,&matrices).y)
            .fold(f32::INFINITY,f32::min);
        assert!(body_floor < height*0.025,"{name} rests on a projecting appendage: {body_floor}");
        assert!(!rig.flop_folds.is_empty());
        assert!(canonical.flop_folds.is_empty());
        rig.sample_swimming(0., 1.7, 1., &mut pose);
        assert!(pose[0].rotation.abs_diff_eq(Quat::IDENTITY,0.00001),
            "{name} remained stranded while swimming");
        }
    }

    #[test]
    fn travel_roster_has_deliberate_styles_and_no_fallback_walk() {
        let names = crate::battle_species_models::supported_species();
        for name in names {
            let rig = crate::species_rig::for_species(name).unwrap();
            let selected = style(rig.species.name());
            assert!(selected.frequency() > 0. && selected.frequency() <= 4.);
        }
        assert_eq!(style("exeggcute"), Style::ClusterHop);
        assert_eq!(style("bulbasaur"), Style::Toad);
        assert_eq!(style("ekans"), Style::Slither);
        assert_eq!(style("onix"), Style::Slither);
    }

    #[test]
    fn travel_hops_leave_the_floor_land_and_stop_without_residual_motion() {
        for name in ["KANGASKHAN", "POLITOED", "NATU", "BULBASAUR", "EXEGGCUTE", "HOOTHOOT"] {
            let canonical = crate::species_rig::for_species(name).unwrap();
            let rig = world_rig(canonical);
            let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
            let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
            let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
            let mut rise = 0.0_f32;
            let mut landing = f32::INFINITY;
            for tick in 0..64 {
                rig.sample_locomotion(0., tick as f32 * TAU / 64., 1., &mut pose);
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                let floor = (0..rig.neutral.positions.len())
                    .map(|v| rig.skin_point(v, &matrices).y)
                    .fold(f32::INFINITY, f32::min);
                assert!(floor >= -height * 0.001, "{name} penetrated floor");
                landing = landing.min(floor);
                if name != "EXEGGCUTE" {
                    rise = rise.max(floor);
                } else {
                    rise = rise.max(pose[1].translation.y - rig.joints[1].bind.translation.y);
                    assert!(
                        (pose[1].translation.y
                            - rig.joints[1].bind.translation.y
                            - pose[2].translation.y
                            + rig.joints[2].bind.translation.y)
                            .abs()
                            > 0.00001
                            || travel::airborne(tick as f32 * TAU / 64.) == 0.
                    );
                }
            }
            assert!(rise > height * 0.025, "{name} never actually hopped");
            assert!(landing < height * 0.005, "{name} never landed");
            let mut idle = pose.clone();
            rig.sample_locomotion(0., 0.7, 0., &mut idle);
            rig.sample_locomotion(0., 4.1, 0., &mut pose);
            for (a, b) in idle.iter().zip(&pose) {
                assert!(
                    a.translation.distance(b.translation) < 1e-6
                        && a.rotation.abs_diff_eq(b.rotation, 1e-6),
                    "{name} moves after stopping"
                );
            }
        }
    }

    #[test]
    fn travel_serpents_undulate_relative_to_each_other_and_keep_ground_contact() {
        for name in [
            "EKANS",
            "ARBOK",
            "ONIX",
            "STEELIX",
            "DRATINI",
            "DRAGONAIR",
            "DUNSPARCE",
        ] {
            let canonical = crate::species_rig::for_species(name).unwrap();
            let rig = world_rig(canonical);
            let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
            let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
            let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
            for tick in 0..32 {
                rig.sample_locomotion(0., tick as f32 * TAU / 32., 1., &mut pose);
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                let displacements: Vec<_> = (0..rig.neutral.positions.len())
                    .map(|v| {
                        rig.skin_point(v, &matrices) - Vec3::from_array(rig.neutral.positions[v])
                    })
                    .collect();
                let lo = displacements
                    .iter()
                    .map(|p| p.x)
                    .fold(f32::INFINITY, f32::min);
                let hi = displacements
                    .iter()
                    .map(|p| p.x)
                    .fold(f32::NEG_INFINITY, f32::max);
                assert!(
                    hi - lo > height * 0.005,
                    "{name} slides rigidly instead of slithering"
                );
                let floor = (0..rig.neutral.positions.len())
                    .map(|v| rig.skin_point(v, &matrices).y)
                    .fold(f32::INFINITY, f32::min);
                assert!(
                    floor >= -height * 0.001 && floor < height * 0.005,
                    "{name} lost belly contact: {floor}"
                );
            }
        }
    }

    #[test]
    fn travel_roster_loops_and_blends_without_floor_penetration() {
        for name in crate::battle_species_models::supported_species() {
            let rig = world_rig(crate::species_rig::for_species(name).unwrap());
            let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
            let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
            let mut start = pose.clone();
            let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
            let (lo, hi) = rig.locomotion_bounds();
            for weight in [0., 0.25, 0.65, 1.] {
                rig.sample_locomotion(0., 0., weight, &mut start);
                rig.sample_locomotion(0., TAU, weight, &mut pose);
                for (a, b) in start.iter().zip(&pose) {
                    assert!(
                        a.translation.distance(b.translation) < height * 0.00001
                            && a.rotation.dot(b.rotation).abs() > 0.99999,
                        "{name} loop pops at movement weight {weight}"
                    );
                }
                for phase in [0.2, 1.5, std::f32::consts::PI, 5.0] {
                    rig.sample_locomotion(0., phase, weight, &mut pose);
                    rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                    for v in 0..rig.neutral.positions.len() {
                        let p = rig.skin_point(v, &matrices);
                        assert!(
                            p.is_finite() && p.y >= -height * 0.001,
                            "{name} weight {weight} phase {phase}: vertex {v} crossed floor ({p:?})"
                        );
                        assert!(
                            p.cmpge(lo).all() && p.cmple(hi).all(),
                            "{name} transition left its culling bounds"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn travel_roll_and_spin_keep_orientation_when_stopping() {
        for name in ["VOLTORB", "ELECTRODE", "HITMONTOP"] {
            let rig = world_rig(crate::species_rig::for_species(name).unwrap());
            let mut moving = vec![Transform::IDENTITY; rig.joints.len()];
            let mut stopped = moving.clone();
            for phase in [0.4, std::f32::consts::PI, 5.7] {
                rig.sample_locomotion(0., phase, 1., &mut moving);
                rig.sample_locomotion(0., phase, 0., &mut stopped);
                assert!(
                    moving[0].rotation.dot(stopped[0].rotation).abs() > 0.99999,
                    "{name} snaps upright when stopped"
                );
            }
        }
    }

    #[test]
    fn travel_dugtrio_peeks_independently_without_lifting_the_earth() {
        let rig = world_rig(crate::species_rig::for_species("DUGTRIO").unwrap());
        let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
        let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
        let mut differences = 0.0_f32;
        for frame in 0..32 {
            rig.sample_locomotion(0., frame as f32 * TAU / 32., 1., &mut pose);
            assert!(
                pose[0].translation.y.abs() < height * 0.005,
                "Dugtrio lifts its soil mound"
            );
            let a = pose[1].translation.y - rig.joints[1].bind.translation.y;
            let b = pose[2].translation.y - rig.joints[2].bind.translation.y;
            differences = differences.max((a - b).abs());
        }
        assert!(
            differences > height * 0.005,
            "three moles move as a rigid group"
        );
    }
    #[test]
    fn slug_foot_wave_deforms_soft_tissue_keeps_shell_rigid_and_stops() {
        for name in ["SLUGMA", "MAGCARGO"] {
            let canonical = crate::species_rig::for_species(name).unwrap();
            let rig = world_rig(canonical);
            let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
            assert_eq!(rig.neutral.positions, canonical.neutral.positions);
            let soft: Vec<_> = (0..rig.neutral.positions.len()).filter(|&v|
                rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|
                    w > 0.2 && rig.joints[j as usize].name.contains("/slide_foot_"))).collect();
            assert!(!soft.is_empty());
            let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
            let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
            let mut first = Vec::new();
            let mut motion = 0.0_f32;
            for frame in 0..16 {
                rig.sample_locomotion(0., frame as f32 * TAU / 16., 1., &mut pose);
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                let floor = rig.ground_vertices.iter().map(|&v|rig.skin_point(v,&matrices).y)
                    .fold(f32::INFINITY,f32::min);
                assert!(floor >= -height*0.00001 && floor < height*0.005, "{name} foot lost contact: {floor}");
                let points: Vec<_> = soft.iter().map(|&v|rig.skin_point(v,&matrices)).collect();
                if frame == 0 { first = points; }
                else { for (a,b) in first.iter().zip(points) { motion=motion.max(a.distance(b)); } }
                for v in 0..canonical.neutral.positions.len() {
                    if canonical.joint_indices[v].iter().zip(canonical.joint_weights[v]).any(|(&j,w)|
                        w > 0. && canonical.joints[j as usize].name.ends_with("/shell")) {
                        assert_eq!(rig.joint_indices[v],canonical.joint_indices[v]);
                        assert_eq!(rig.joint_weights[v],canonical.joint_weights[v]);
                    }
                }
            }
            assert!(motion > height*0.01, "{name} still slides rigidly");
            rig.sample_locomotion(0., 1.7, 0., &mut pose);
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for &v in &soft { assert!(rig.skin_point(v,&matrices)
                .distance(Vec3::from_array(rig.neutral.positions[v])) < height*0.00001); }
        }
    }

    #[test]
    fn seals_pull_with_flippers_keep_belly_contact_and_stop_on_land() {
        for name in ["SEEL", "DEWGONG"] {
            let rig = world_rig(crate::species_rig::for_species(name).unwrap());
            let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
            let vertices: Vec<_> = (0..rig.neutral.positions.len()).filter(|&v|
                rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|
                    w > 0.2 && rig.joints[j as usize].name.contains("/flipper_"))).collect();
            assert!(!vertices.is_empty(), "{name} has no flipper surfaces");
            let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
            let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
            let mut first = Vec::new();
            let mut movement = 0.0_f32;
            for tick in 0..32 {
                rig.sample_locomotion(0., tick as f32 * TAU / 32., 1., &mut pose);
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                let points: Vec<_> = vertices.iter().map(|&v|rig.skin_point(v,&matrices)).collect();
                if tick == 0 { first = points; }
                else { for (a,b) in first.iter().zip(points) { movement=movement.max(a.distance(b)); } }
                let floor = (0..rig.neutral.positions.len()).map(|v|rig.skin_point(v,&matrices).y)
                    .fold(f32::INFINITY,f32::min);
                assert!(floor >= -height*0.00001 && floor < height*0.01,
                    "{name} belly lost the substrate: {floor}");
            }
            assert!(movement > height*0.025, "{name} still slides without pulling");
            rig.sample_locomotion(0., 1.3, 0., &mut pose);
            let stopped = pose.clone();
            rig.sample_locomotion(0., 4.8, 0., &mut pose);
            for (a,b) in pose.iter().zip(stopped) {
                assert!(a.translation.abs_diff_eq(b.translation, 0.00001));
                assert!(a.rotation.abs_diff_eq(b.rotation, 0.00001));
            }
        }
    }

    #[test]
    fn water_tentacles_stroke_actual_surfaces_and_stop_without_land_planting() {
        for name in ["OCTILLERY", "TENTACOOL", "TENTACRUEL", "OMANYTE", "OMASTAR"] {
            let rig = world_rig(crate::species_rig::for_species(name).unwrap());
            let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
            let vertices: Vec<_> = (0..rig.neutral.positions.len()).filter(|&v|
                rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|
                    w > 0.2 && rig.joints[j as usize].name.contains("tentacle_"))).collect();
            assert!(!vertices.is_empty(), "{name} lacks tentacle surfaces");
            let mut pose=vec![Transform::IDENTITY;rig.joints.len()];
            let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];
            rig.sample_swimming(0.,0.,1.,&mut pose);
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            let first:Vec<_>=vertices.iter().map(|&v|rig.skin_point(v,&matrices)).collect();
            rig.sample_swimming(0.,std::f32::consts::PI,1.,&mut pose);
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            let motion=vertices.iter().zip(first).map(|(&v,p)|rig.skin_point(v,&matrices).distance(p))
                .fold(0.0_f32,f32::max);
            assert!(motion > height*0.005, "{name} tentacles remain rigid: {motion}");
            rig.sample_swimming(0.,1.7,0.,&mut pose);
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for &v in &vertices {assert!(rig.skin_point(v,&matrices)
                .distance(Vec3::from_array(rig.neutral.positions[v])) < height*0.00001);}
        }
    }

}
