//! Bounded reader for authored Pokémon standard glTF skins.
//! Geometry is decoded once, without baking joint transforms or changing its
//! neutral coordinate system. Battle time/placement and GPU entities are owned
//! by the caller. Unsupported content is an error, never a static fallback.
use std::{collections::HashSet, ops::Range, sync::OnceLock};

use bevy::prelude::{Mat4, Quat, Transform, Vec3};
use gltf::{
    accessor::{DataType, Dimensions},
    animation::{Interpolation, Property},
    mesh::{Semantic, util::ReadWeights},
};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::mesh::SurfaceMeshData;

const MAX_BYTES: usize = 1024 * 1024;
const MAX_VERTICES: usize = 20_000;
const MAX_INDICES: usize = 120_000;
const MAX_ACCESSORS: usize = 256;
const MAX_KEYS: usize = 257;
const MAX_DURATION: f32 = 10.0;
const ENVELOPE_HZ: f32 = 30.0;
const FLOAT_MARGIN: f32 = 0.0001;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Species {
    Chikorita,
    Cyndaquil,
    Totodile,
    Gengar,
    Spearow,
}

impl Species {
    pub const ALL: [Self; 5] = [
        Self::Chikorita,
        Self::Cyndaquil,
        Self::Totodile,
        Self::Gengar,
        Self::Spearow,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Chikorita => "chikorita",
            Self::Cyndaquil => "cyndaquil",
            Self::Totodile => "totodile",
            Self::Gengar => "gengar",
            Self::Spearow => "spearow",
        }
    }

    fn joint_names(self) -> &'static [&'static str] {
        match self {
            Self::Chikorita => &[
                "root",
                "torso",
                "head",
                "foreleg_left",
                "foreleg_right",
                "hindleg_left",
                "hindleg_right",
                "tail_nub",
                "leaf_petiole",
                "leaf_mid_fold",
                "leaf_tip_fold",
            ],
            Self::Cyndaquil => &[
                "root",
                "torso",
                "head",
                "foreleg_left",
                "foreleg_right",
                "upper_left_flame_quill",
                "upper_right_flame_quill",
                "lower_left_flame_quill",
                "lower_right_flame_quill",
                "central_rear_flame_quill",
            ],
            Self::Totodile => &[
                "root",
                "torso",
                "head",
                "lower_jaw",
                "forepaw_left",
                "forepaw_right",
                "tail_base",
                "tail_tip",
            ],
            Self::Gengar => &[
                "root",
                "torso",
                "face_core",
                "shoulder_left",
                "hand_left",
                "shoulder_right",
                "hand_right",
                "ear_left",
                "ear_right",
                "dorsal_crown",
                "dorsal_left",
                "dorsal_right",
                "dorsal_lower",
                "tail_spine",
            ],
            Self::Spearow => &[
                "root",
                "torso",
                "head",
                "lower_beak",
                "wing_left",
                "flight_feathers_left",
                "wing_right",
                "flight_feathers_right",
                "tail_fan",
                "crown",
            ],
        }
    }

    fn parents(self) -> &'static [Option<usize>] {
        match self {
            Self::Chikorita => &[
                None,
                Some(0),
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                Some(2),
                Some(8),
                Some(9),
            ],
            Self::Cyndaquil => &[
                None,
                Some(0),
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                Some(1),
            ],
            Self::Totodile => &[
                None,
                Some(0),
                Some(1),
                Some(2),
                Some(1),
                Some(1),
                Some(0),
                Some(6),
            ],
            Self::Gengar => &[
                None,
                Some(0),
                Some(1),
                Some(1),
                Some(3),
                Some(1),
                Some(5),
                Some(2),
                Some(2),
                Some(2),
                Some(2),
                Some(2),
                Some(1),
                Some(1),
            ],
            Self::Spearow => &[
                None,
                Some(0),
                Some(1),
                Some(2),
                Some(1),
                Some(4),
                Some(1),
                Some(6),
                Some(1),
                Some(2),
            ],
        }
    }

    fn part_count(self) -> usize {
        match self {
            Self::Chikorita => 19,
            Self::Cyndaquil => 30,
            Self::Totodile => 27,
            Self::Gengar => 27,
            Self::Spearow => 28,
        }
    }

    fn neutral_digest(self) -> &'static str {
        match self {
            Self::Chikorita => "e6f4c6ed5c602ef3562f3bc4b19bd800944abdd2d5722ecd5df832d3bf49bc96",
            Self::Cyndaquil => "551051e53987f2461c2e5ed22187b44b336be64f300b469950d381b5b174b397",
            Self::Totodile => "3e3f8de35f5d7cabcc889604baff4103f050f6abf2a24f6336f9ea3fea9a47d0",
            Self::Gengar => "d458acb7c2853288efef3b83c24ac453bcb409f9db01d24a96008e9a937485aa",
            Self::Spearow => "42ad1c0255af66f84d1770d9050c2d5b8fa7833a1a38180fa380665915b09fe7",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Clip {
    Idle,
    Attack,
    Hit,
}

impl Clip {
    pub const ALL: [Self; 3] = [Self::Idle, Self::Attack, Self::Hit];
    fn index(self) -> usize {
        match self {
            Self::Idle => 0,
            Self::Attack => 1,
            Self::Hit => 2,
        }
    }
    fn suffix(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Attack => "attack",
            Self::Hit => "hit",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Playback {
    Loop,
    Clamp,
}

#[derive(Debug)]
pub(crate) struct Joint {
    /// Original skin order, also the GPU JOINTS_0 indexing order.
    pub name: String,
    pub parent: Option<usize>,
    pub bind: Transform,
    pub inverse_bind: Mat4,
}

#[derive(Debug)]
pub(crate) struct Anatomy {
    pub name: String,
    pub vertices: Range<usize>,
    pub indices: Range<usize>,
    pub material: usize,
}

#[derive(Debug)]
pub(crate) struct Material {
    pub name: String,
    pub color: [f32; 4],
    pub alpha_mode: gltf::material::AlphaMode,
}

#[derive(Debug)]
pub(crate) struct SpeciesClip {
    pub name: String,
    pub duration: f32,
    pub loop_suggested: bool,
    pub cue_relative: bool,
    /// Preserve author metadata without deriving gameplay semantics from it.
    pub extras: String,
    times: Vec<f32>,
    /// Joint zero stays at identity; all other joints have authored rotations.
    rotations: Vec<Vec<Quat>>,
}

#[derive(Debug)]
pub(crate) struct SpeciesRig {
    pub species: Species,
    /// Original primitive order and f32 position/color bits. Indices are
    /// offset only to address the concatenated vertex buffer; anatomy reverses
    /// that offset exactly. This remains the physical-size/grounding source.
    pub neutral: SurfaceMeshData,
    /// Exact GLB/source normal bits for audit. `neutral.normals` applies the
    /// same one-time normalization as the prior JSON reader for shading parity.
    pub source_normals: Vec<[f32; 3]>,
    pub joint_indices: Vec<[u16; 4]>,
    pub joint_weights: Vec<[f32; 4]>,
    pub anatomy: Vec<Anatomy>,
    pub materials: Vec<Material>,
    pub joints: Vec<Joint>,
    pub clips: [SpeciesClip; 3],
    pub neutral_bounds: (Vec3, Vec3),
    /// Model-space camera/culling envelope. Never use it to renormalize height.
    pub animated_bounds: (Vec3, Vec3),
}

pub(crate) fn rig(species: Species) -> &'static SpeciesRig {
    static CHIKORITA: OnceLock<SpeciesRig> = OnceLock::new();
    static CYNDAQUIL: OnceLock<SpeciesRig> = OnceLock::new();
    static TOTODILE: OnceLock<SpeciesRig> = OnceLock::new();
    static GENGAR: OnceLock<SpeciesRig> = OnceLock::new();
    static SPEAROW: OnceLock<SpeciesRig> = OnceLock::new();
    let cache = match species {
        Species::Chikorita => &CHIKORITA,
        Species::Cyndaquil => &CYNDAQUIL,
        Species::Totodile => &TOTODILE,
        Species::Gengar => &GENGAR,
        Species::Spearow => &SPEAROW,
    };
    cache.get_or_init(|| {
        let source = match species {
            Species::Chikorita => {
                crate::model_storage::include_model!("models/actor_props/battle_chikorita.glb")
            }
            Species::Cyndaquil => {
                crate::model_storage::include_model!("models/actor_props/battle_cyndaquil.glb")
            }
            Species::Totodile => {
                crate::model_storage::include_model!("models/actor_props/battle_totodile.glb")
            }
            Species::Gengar => {
                crate::model_storage::include_model!("models/battle_species/gengar.glb")
            }
            Species::Spearow => {
                crate::model_storage::include_model!("models/battle_species/spearow.glb")
            }
        };
        let bytes =
            crate::model_storage::decode_bytes(source).expect("valid authored species GLB storage");
        SpeciesRig::parse(species, &bytes).expect("valid authored species skin and clips")
    })
}

pub(crate) fn for_species(species: &str) -> Option<&'static SpeciesRig> {
    if species.eq_ignore_ascii_case("CHIKORITA") {
        Some(rig(Species::Chikorita))
    } else if species.eq_ignore_ascii_case("CYNDAQUIL") {
        Some(rig(Species::Cyndaquil))
    } else if species.eq_ignore_ascii_case("TOTODILE") {
        Some(rig(Species::Totodile))
    } else if species.eq_ignore_ascii_case("GENGAR") {
        Some(rig(Species::Gengar))
    } else if species.eq_ignore_ascii_case("SPEAROW") {
        Some(rig(Species::Spearow))
    } else {
        None
    }
}

impl SpeciesRig {
    /// Call through the owner's per-species OnceLock. No load or decode work is
    /// required after the immutable rig and GPU assets have been cached.
    pub fn parse(species: Species, bytes: &[u8]) -> Result<Self, String> {
        validate_container(bytes)?;
        let gltf =
            gltf::Gltf::from_slice(bytes).map_err(|e| format!("invalid species GLB: {e}"))?;
        let binary = gltf.blob.as_deref().ok_or("missing species binary data")?;
        let joint_count = species.joint_names().len();
        if gltf.buffers().len() != 1
            || gltf.scenes().len() != 1
            || gltf.meshes().len() != 1
            || gltf.skins().len() != 1
            || gltf.nodes().len() != joint_count + 1
            || gltf.animations().len() != 3
            || gltf.materials().len() == 0
            || gltf.materials().len() > species.part_count()
            || gltf.accessors().len() > MAX_ACCESSORS
            || gltf.views().len() > MAX_ACCESSORS
            || gltf.images().len() != 0
            || gltf.textures().len() != 0
            || gltf.cameras().len() != 0
            || gltf.extensions_used().next().is_some()
            || gltf.extensions_required().next().is_some()
        {
            return Err("unsupported species scene/schema counts or resources".into());
        }
        let buffer = gltf.buffers().next().unwrap();
        if !matches!(buffer.source(), gltf::buffer::Source::Bin)
            || buffer.length() > binary.len()
            || binary.len() - buffer.length() > 3
        {
            return Err("species requires one embedded buffer".into());
        }
        validate_accessors(&gltf, buffer.length())?;
        let scene = gltf.scenes().next().unwrap();
        if scene.name() != Some(species.name())
            || scene.nodes().map(|n| n.index()).collect::<Vec<_>>() != [0, joint_count]
            || gltf.default_scene().map(|s| s.index()) != Some(0)
        {
            return Err("species requires named skeleton and mesh scene roots".into());
        }

        let materials = gltf
            .materials()
            .map(read_material)
            .collect::<Result<Vec<_>, _>>()?;
        let joints = read_joints(species, &gltf, binary)?;
        let mesh = gltf.meshes().next().unwrap();
        if mesh.name() != Some(species.name())
            || mesh.weights().is_some()
            || mesh.primitives().len() != species.part_count()
        {
            return Err("invalid species anatomy mesh".into());
        }

        let mut neutral = SurfaceMeshData::default();
        let mut source_normals = Vec::new();
        let mut joint_indices = Vec::new();
        let mut joint_weights = Vec::new();
        let mut anatomy = Vec::new();
        let mut names = HashSet::new();
        let mut digest = Sha256::new();
        for primitive in mesh.primitives() {
            validate_primitive(&primitive)?;
            let positions_count = primitive.get(&Semantic::Positions).unwrap().count();
            let indices_count = primitive.indices().unwrap().count();
            if neutral.positions.len() + positions_count > MAX_VERTICES
                || neutral.indices.len() + indices_count > MAX_INDICES
            {
                return Err("species total geometry exceeds allocation bounds".into());
            }
            let meta: PartExtras = read_extras(primitive.extras())?;
            if meta.part.is_empty() || meta.part.len() > 160 || !names.insert(meta.part.clone()) {
                return Err("species anatomy names must be bounded and unique".into());
            }
            let material = primitive
                .material()
                .index()
                .ok_or("missing species material")?;
            let reader = primitive.reader(|_| Some(binary));
            let positions: Vec<_> = reader
                .read_positions()
                .ok_or("missing positions")?
                .collect();
            let normals: Vec<_> = reader.read_normals().ok_or("missing normals")?.collect();
            let indices: Vec<_> = reader
                .read_indices()
                .ok_or("missing indices")?
                .into_u32()
                .collect();
            let ids: Vec<_> = reader
                .read_joints(0)
                .ok_or("missing joints")?
                .into_u16()
                .collect();
            let Some(ReadWeights::U8(weights)) = reader.read_weights(0) else {
                return Err("species requires exact normalized u8 weights".into());
            };
            let weights: Vec<_> = weights.collect();
            if positions.len() != positions_count
                || normals.len() != positions_count
                || ids.len() != positions_count
                || weights.len() != positions_count
                || indices.len() != indices_count
                || positions
                    .iter()
                    .flatten()
                    .chain(normals.iter().flatten())
                    .any(|v| !v.is_finite())
                || positions.iter().flatten().any(|v| v.abs() > 4.0)
                || normals
                    .iter()
                    .any(|n| !(0.99..=1.01).contains(&Vec3::from_array(*n).length_squared()))
                || indices.iter().any(|&i| i as usize >= positions_count)
                || ids.iter().zip(&weights).any(|(js, ws)| {
                    ws.iter().map(|&w| u16::from(w)).sum::<u16>() != 255
                        || js
                            .iter()
                            .zip(ws)
                            .any(|(&j, &w)| j as usize >= joint_count || (w == 0 && j != 0))
                })
            {
                return Err("invalid species geometry or skin influences".into());
            }
            // Same digest order as the surviving source/generator. No rounded
            // source JSON, duplicate neutral asset or lossy conversion needed.
            for value in positions.iter().flatten().chain(normals.iter().flatten()) {
                digest.update(value.to_le_bytes());
            }
            for value in &indices {
                digest.update(value.to_le_bytes());
            }
            for value in materials[material].color {
                digest.update(value.to_le_bytes());
            }
            let vertex_start = neutral.positions.len();
            let index_start = neutral.indices.len();
            neutral.positions.extend(positions);
            // The prior prop reader supplied neutral UVs for every vertex.
            neutral
                .uvs
                .extend(std::iter::repeat_n([0.0, 0.0], positions_count));
            neutral.normals.extend(
                normals
                    .iter()
                    .map(|n| Vec3::from_array(*n).normalize().to_array()),
            );
            source_normals.extend(normals);
            neutral.colors.extend(std::iter::repeat_n(
                materials[material].color,
                positions_count,
            ));
            neutral
                .indices
                .extend(indices.into_iter().map(|i| i + vertex_start as u32));
            joint_indices.extend(ids);
            // glTF normalized unsigned byte conversion, without reweighting.
            joint_weights.extend(weights.into_iter().map(|w| w.map(|v| f32::from(v) / 255.0)));
            anatomy.push(Anatomy {
                name: meta.part,
                vertices: vertex_start..neutral.positions.len(),
                indices: index_start..neutral.indices.len(),
                material,
            });
        }
        if format!("{:x}", digest.finalize()) != species.neutral_digest() {
            return Err("species neutral geometry differs from the reviewed source".into());
        }
        let clips = read_clips(species, &gltf, binary)?;
        let neutral_bounds = point_bounds(neutral.positions.iter().copied().map(Vec3::from_array));
        if neutral_bounds.0.y.abs() > 0.0001
            || (neutral_bounds.1 - neutral_bounds.0).min_element() < 0.01
        {
            return Err("species must be volumetric and floor rooted".into());
        }
        let mut rig = Self {
            species,
            neutral,
            source_normals,
            joint_indices,
            joint_weights,
            anatomy,
            materials,
            joints,
            clips,
            neutral_bounds,
            animated_bounds: neutral_bounds,
        };
        rig.animated_bounds = rig.compute_envelope();
        Ok(rig)
    }

    pub fn clip(&self, clip: Clip) -> &SpeciesClip {
        &self.clips[clip.index()]
    }

    /// Fully overwrites caller-owned storage with local TRS, including root.
    /// Idle normally loops; attack/hit normally clamp to cue-relative seconds.
    /// Caller controls playback explicitly. Untargeted bind translations remain
    /// exact, root stays identity, and closed endpoints return exact bind TRS.
    pub fn sample_into(
        &self,
        clip: Clip,
        seconds: f32,
        playback: Playback,
        pose: &mut [Transform],
    ) -> Result<(), &'static str> {
        if !seconds.is_finite() || pose.len() != self.joints.len() {
            return Err("species sample needs finite time and exact joint storage");
        }
        let clip = self.clip(clip);
        let time = match playback {
            Playback::Loop => seconds.rem_euclid(clip.duration),
            Playback::Clamp => seconds.clamp(0.0, clip.duration),
        };
        for (local, joint) in pose.iter_mut().zip(&self.joints) {
            *local = joint.bind;
        }
        if time == 0.0 || time == clip.duration {
            return Ok(());
        }
        let upper = clip.times.partition_point(|&t| t <= time);
        let left = upper.saturating_sub(1);
        let right = upper.min(clip.times.len() - 1);
        let fraction = if left == right {
            0.0
        } else {
            ((time - clip.times[left]) / (clip.times[right] - clip.times[left])).clamp(0.0, 1.0)
        };
        for (local, keys) in pose[1..].iter_mut().zip(&clip.rotations) {
            local.rotation = keys[left]
                .normalize()
                .slerp(keys[right].normalize(), fraction)
                .normalize();
        }
        Ok(())
    }

    /// Audit/envelope utility; GPU playback should write local joint entities.
    /// Result is species-model-space skin matrices, without instance placement.
    pub fn skin_matrices_into(
        &self,
        pose: &[Transform],
        matrices: &mut [Mat4],
    ) -> Result<(), &'static str> {
        if pose.len() != self.joints.len() || matrices.len() != self.joints.len() {
            return Err("species matrices need exact joint storage");
        }
        for (i, joint) in self.joints.iter().enumerate() {
            let local = pose[i].compute_matrix();
            matrices[i] = joint
                .parent
                .map_or(local, |parent| matrices[parent] * local);
        }
        for (matrix, joint) in matrices.iter_mut().zip(&self.joints) {
            *matrix *= joint.inverse_bind;
        }
        Ok(())
    }

    fn skin_point(&self, index: usize, matrices: &[Mat4]) -> Vec3 {
        let position = Vec3::from_array(self.neutral.positions[index]);
        self.joint_indices[index]
            .iter()
            .zip(self.joint_weights[index])
            .fold(Vec3::ZERO, |p, (&j, w)| {
                p + matrices[j as usize].transform_point3(position) * w
            })
    }

    fn compute_envelope(&self) -> (Vec3, Vec3) {
        let mut bounds = self.neutral_bounds;
        let mut pose = vec![Transform::IDENTITY; self.joints.len()];
        let mut matrices = vec![Mat4::IDENTITY; self.joints.len()];
        for which in Clip::ALL {
            let clip = self.clip(which);
            let mut angular_speed = vec![0.0_f32; self.joints.len()];
            for (joint, keys) in clip.rotations.iter().enumerate() {
                angular_speed[joint + 1] = keys
                    .windows(2)
                    .zip(clip.times.windows(2))
                    .map(|(q, t)| {
                        let a = q[0].as_dquat().normalize();
                        let mut b = q[1].as_dquat().normalize();
                        if a.dot(b) < 0.0 {
                            b = -b;
                        }
                        // 4*tan(angle/4)/dt bounds both true slerp speed and
                        // normalized-lerp's maximum speed (glam's near-zero path).
                        let speed =
                            4.0 * (a - b).length() / (a + b).length() / f64::from(t[1] - t[0]);
                        (speed * 1.01) as f32
                    })
                    .fold(0.0, f32::max);
            }
            // A rotating ancestor moves an influenced vertex at <= omega*r.
            // Triangle inequality bounds r for every possible intervening pose.
            // We sum nonnegative skin weights before taking max vertex speed.
            let mut maximum_speed = 0.0_f32;
            for (i, p) in self.neutral.positions.iter().enumerate() {
                let point = Vec3::from_array(*p);
                let mut speed = 0.0;
                for (&id, weight) in self.joint_indices[i].iter().zip(self.joint_weights[i]) {
                    if weight == 0.0 {
                        continue;
                    }
                    let mut joint = id as usize;
                    let mut radius = self.joints[joint]
                        .inverse_bind
                        .transform_point3(point)
                        .length();
                    let mut influence_speed = 0.0;
                    loop {
                        influence_speed += angular_speed[joint] * radius;
                        let Some(parent) = self.joints[joint].parent else {
                            break;
                        };
                        radius += self.joints[joint].bind.translation.length();
                        joint = parent;
                    }
                    speed += weight * influence_speed;
                }
                maximum_speed = maximum_speed.max(speed);
            }
            // The longer three-joint leaf needs a finer cached envelope to
            // keep the same conservative speed bound from enlarging the
            // whole camera box. This runs once when the GLB is loaded; frame
            // playback still uses the authored keys and original cue clock.
            let envelope_hz = match self.species {
                Species::Chikorita => 90.0,
                _ => ENVELOPE_HZ,
            };
            let steps = (clip.duration * envelope_hz).ceil() as usize;
            let margin =
                Vec3::splat(maximum_speed * (clip.duration / steps as f32) * 0.5 + FLOAT_MARGIN);
            let mut clip_bounds = self.neutral_bounds;
            for frame in 0..=steps {
                let time = clip.duration * (frame as f32 / steps as f32);
                self.sample_into(which, time, Playback::Clamp, &mut pose)
                    .unwrap();
                self.skin_matrices_into(&pose, &mut matrices).unwrap();
                for i in 0..self.neutral.positions.len() {
                    let point = self.skin_point(i, &matrices);
                    clip_bounds.0 = clip_bounds.0.min(point);
                    clip_bounds.1 = clip_bounds.1.max(point);
                }
            }
            bounds.0 = bounds.0.min(clip_bounds.0 - margin);
            bounds.1 = bounds.1.max(clip_bounds.1 + margin);
        }
        bounds
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PartExtras {
    part: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClipExtras {
    loop_suggested: bool,
    cue_relative: bool,
}

fn read_extras<T: serde::de::DeserializeOwned>(extras: &gltf::json::Extras) -> Result<T, String> {
    serde_json::from_str(extras.as_ref().ok_or("missing species metadata")?.get())
        .map_err(|e| e.to_string())
}

fn point_bounds(points: impl Iterator<Item = Vec3>) -> (Vec3, Vec3) {
    points.fold(
        (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
        |(lo, hi), p| (lo.min(p), hi.max(p)),
    )
}

fn validate_container(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() < 28
        || bytes.len() > MAX_BYTES
        || !bytes.starts_with(b"glTF")
        || u32::from_le_bytes(bytes[4..8].try_into().unwrap()) != 2
        || u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize != bytes.len()
    {
        return Err("species requires a bounded binary GLB v2".into());
    }
    let glb = gltf::binary::Glb::from_slice(bytes).map_err(|e| e.to_string())?;
    let binary = glb.bin.as_deref().ok_or("missing binary chunk")?;
    if glb.json.len() % 4 != 0
        || binary.len() % 4 != 0
        || 28 + glb.json.len() + binary.len() != bytes.len()
    {
        return Err("species requires exactly two aligned GLB chunks".into());
    }
    let json: serde_json::Value = serde_json::from_slice(&glb.json).map_err(|e| e.to_string())?;
    // Undeclared extension data may disappear in gltf builds without features.
    fn has_extensions(value: &serde_json::Value) -> bool {
        match value {
            serde_json::Value::Object(o) => o
                .iter()
                .any(|(k, v)| k == "extensions" || (k != "extras" && has_extensions(v))),
            serde_json::Value::Array(a) => a.iter().any(has_extensions),
            _ => false,
        }
    }
    if json["asset"]["version"] != "2.0"
        || json["asset"].get("minVersion").is_some_and(|v| v != "2.0")
        || has_extensions(&json)
    {
        return Err("unsupported species glTF version or extension".into());
    }
    Ok(())
}

fn validate_accessors(gltf: &gltf::Gltf, binary_len: usize) -> Result<(), String> {
    for view in gltf.views() {
        if view.buffer().index() != 0
            || view.stride().is_some()
            || view.offset() % 4 != 0
            || view
                .offset()
                .checked_add(view.length())
                .is_none_or(|end| end > binary_len)
        {
            return Err(
                "species buffer views must be tightly packed, aligned and in bounds".into(),
            );
        }
    }
    for accessor in gltf.accessors() {
        let view = accessor
            .view()
            .ok_or("species accessor needs a buffer view")?;
        let valid_type = matches!(
            (
                accessor.data_type(),
                accessor.dimensions(),
                accessor.normalized()
            ),
            (
                DataType::F32,
                Dimensions::Scalar | Dimensions::Vec3 | Dimensions::Vec4 | Dimensions::Mat4,
                false
            ) | (DataType::U16, Dimensions::Scalar, false)
                | (DataType::U8, Dimensions::Vec4, _)
        );
        if !valid_type
            || accessor.sparse().is_some()
            || accessor.offset() % 4 != 0
            || accessor.count() == 0
            || accessor.count() > MAX_INDICES
            || accessor
                .count()
                .checked_mul(accessor.size())
                .and_then(|n| n.checked_add(accessor.offset()))
                .is_none_or(|end| end > view.length())
        {
            return Err("unsupported species accessor encoding or bounds".into());
        }
    }
    Ok(())
}

fn read_material(material: gltf::Material<'_>) -> Result<Material, String> {
    let pbr = material.pbr_metallic_roughness();
    let color = pbr.base_color_factor();
    let name = material.name().ok_or("species materials must be named")?;
    if name.is_empty()
        || name.len() > 160
        || color
            .iter()
            .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        || pbr.metallic_factor() != 0.0
        || pbr.roughness_factor() != 1.0
        || pbr.base_color_texture().is_some()
        || pbr.metallic_roughness_texture().is_some()
        || material.normal_texture().is_some()
        || material.occlusion_texture().is_some()
        || material.emissive_texture().is_some()
        || material.emissive_factor() != [0.0; 3]
        || material.double_sided()
        || material.alpha_cutoff().is_some()
        || material.alpha_mode() == gltf::material::AlphaMode::Mask
        || (material.alpha_mode() == gltf::material::AlphaMode::Opaque && color[3] != 1.0)
    {
        return Err(
            "species requires named, untextured metallic=0 roughness=1 RGBA materials".into(),
        );
    }
    Ok(Material {
        name: name.to_owned(),
        color,
        alpha_mode: material.alpha_mode(),
    })
}

fn validate_primitive(primitive: &gltf::Primitive<'_>) -> Result<(), String> {
    if primitive.mode() != gltf::mesh::Mode::Triangles
        || primitive.morph_targets().next().is_some()
        || primitive.attributes().len() != 4
    {
        return Err("species requires indexed POSITION/NORMAL/JOINTS_0/WEIGHTS_0 triangles".into());
    }
    let expected = [
        (Semantic::Positions, DataType::F32, Dimensions::Vec3, false),
        (Semantic::Normals, DataType::F32, Dimensions::Vec3, false),
        (Semantic::Joints(0), DataType::U8, Dimensions::Vec4, false),
        (Semantic::Weights(0), DataType::U8, Dimensions::Vec4, true),
    ];
    let count = primitive
        .get(&Semantic::Positions)
        .ok_or("missing species positions")?
        .count();
    if count < 3 || count > MAX_VERTICES {
        return Err("species primitive exceeds vertex bounds".into());
    }
    for (semantic, data_type, dimensions, normalized) in expected {
        let a = primitive
            .get(&semantic)
            .ok_or("missing species skin attribute")?;
        if a.data_type() != data_type
            || a.dimensions() != dimensions
            || a.normalized() != normalized
            || a.count() != count
        {
            return Err("invalid species skin attribute schema".into());
        }
    }
    let indices = primitive.indices().ok_or("missing species indices")?;
    if indices.data_type() != DataType::U16
        || indices.dimensions() != Dimensions::Scalar
        || indices.count() % 3 != 0
        || indices.count() > MAX_INDICES
        || indices.normalized()
    {
        return Err("invalid species triangle indices".into());
    }
    Ok(())
}

fn read_joints(species: Species, gltf: &gltf::Gltf, binary: &[u8]) -> Result<Vec<Joint>, String> {
    let count = species.joint_names().len();
    let nodes: Vec<_> = gltf.nodes().collect();
    let skin = gltf.skins().next().unwrap();
    if skin.name() != Some(species.name())
        || skin.skeleton().map(|n| n.index()) != Some(0)
        || skin.joints().map(|n| n.index()).collect::<Vec<_>>() != (0..count).collect::<Vec<_>>()
    {
        return Err("species skin must preserve named authored joint order".into());
    }
    let accessor = skin
        .inverse_bind_matrices()
        .ok_or("missing inverse bind matrices")?;
    if accessor.data_type() != DataType::F32
        || accessor.dimensions() != Dimensions::Mat4
        || accessor.count() != count
    {
        return Err("invalid inverse bind matrix schema".into());
    }
    let inverse: Vec<_> = skin
        .reader(|_| Some(binary))
        .read_inverse_bind_matrices()
        .ok_or("missing inverse binds")?
        .map(|m| Mat4::from_cols_array_2d(&m))
        .collect();
    if inverse.len() != count {
        return Err("truncated species inverse binds".into());
    }
    let mut parents = vec![None; count + 1];
    for node in &nodes {
        if node.weights().is_some() || node.camera().is_some() {
            return Err("species node morphs and cameras are unsupported".into());
        }
        for child in node.children() {
            if child.index() <= node.index()
                || parents[child.index()].replace(node.index()).is_some()
            {
                return Err("species joint hierarchy is cyclic, shared or out of order".into());
            }
        }
    }
    if parents[..count] != *species.parents()
        || parents[count].is_some()
        || nodes[count].children().next().is_some()
        || nodes[count].mesh().map(|m| m.index()) != Some(0)
        || nodes[count].skin().map(|s| s.index()) != Some(0)
        || nodes[count].name() != Some(species.name())
        || bind_transform(&nodes[count])? != Transform::IDENTITY
    {
        return Err("species requires exact authored hierarchy and identity mesh root".into());
    }
    let mut joints = Vec::with_capacity(count);
    let mut globals = vec![Mat4::IDENTITY; count];
    for i in 0..count {
        let name = format!("{}/{}", species.name(), species.joint_names()[i]);
        if nodes[i].name() != Some(name.as_str())
            || nodes[i].mesh().is_some()
            || nodes[i].skin().is_some()
        {
            return Err(
                "species joint name or geometry ownership differs from the authored rig".into(),
            );
        }
        let bind = bind_transform(&nodes[i])?;
        if i == 0 && bind != Transform::IDENTITY {
            return Err("species root must stay identity".into());
        }
        let local = bind.compute_matrix();
        globals[i] = parents[i].map_or(local, |p| globals[p] * local);
        if !inverse[i].is_finite()
            || (globals[i] * inverse[i])
                .to_cols_array()
                .iter()
                .zip(Mat4::IDENTITY.to_cols_array())
                .any(|(a, b)| (a - b).abs() > 0.000002)
        {
            return Err("species inverse bind does not cancel its neutral joint transform".into());
        }
        joints.push(Joint {
            name,
            parent: parents[i],
            bind,
            inverse_bind: inverse[i],
        });
    }
    Ok(joints)
}

fn bind_transform(node: &gltf::Node<'_>) -> Result<Transform, String> {
    let gltf::scene::Transform::Decomposed {
        translation,
        rotation,
        scale,
    } = node.transform()
    else {
        return Err("species requires translation-only TRS bind nodes".into());
    };
    if translation.iter().any(|v| !v.is_finite() || v.abs() > 4.0)
        || rotation != [0.0, 0.0, 0.0, 1.0]
        || scale != [1.0; 3]
    {
        return Err("species requires finite translation-only bind transforms".into());
    }
    Ok(Transform::from_translation(Vec3::from_array(translation)))
}

fn read_clips(
    species: Species,
    gltf: &gltf::Gltf,
    binary: &[u8],
) -> Result<[SpeciesClip; 3], String> {
    let count = species.joint_names().len();
    let mut clips = Vec::new();
    for (animation, kind) in gltf.animations().zip(Clip::ALL) {
        let name = format!("{}.{}", species.name(), kind.suffix());
        let metadata: ClipExtras = read_extras(animation.extras())?;
        if animation.name() != Some(name.as_str())
            || animation.channels().count() != count - 1
            || animation.samplers().count() != count - 1
            || metadata.loop_suggested != (kind == Clip::Idle)
            || metadata.cue_relative != (kind != Clip::Idle)
        {
            return Err("species requires authored idle/attack/hit clip semantics".into());
        }
        let mut times = Vec::new();
        let mut rotations = vec![Vec::new(); count - 1];
        for channel in animation.channels() {
            let target = channel.target().node().index();
            if target == 0
                || target >= count
                || !rotations[target - 1].is_empty()
                || channel.target().property() != Property::Rotation
                || channel.sampler().interpolation() != Interpolation::Linear
            {
                return Err(
                    "species clips require one LINEAR rotation channel per articulated joint"
                        .into(),
                );
            }
            let input = channel.sampler().input();
            let output = channel.sampler().output();
            if input.data_type() != DataType::F32
                || input.dimensions() != Dimensions::Scalar
                || input.count() < 2
                || input.count() > MAX_KEYS
                || output.count() != input.count()
                || output.data_type() != DataType::F32
                || output.dimensions() != Dimensions::Vec4
            {
                return Err("invalid species animation accessor schema".into());
            }
            let reader = channel.reader(|_| Some(binary));
            let channel_times: Vec<_> = reader
                .read_inputs()
                .ok_or("missing species key times")?
                .collect();
            if channel_times.len() != input.count()
                || channel_times[0] != 0.0
                || channel_times
                    .iter()
                    .any(|t| !t.is_finite() || *t < 0.0 || *t > MAX_DURATION)
                || channel_times.windows(2).any(|t| t[1] - t[0] < 0.0001)
            {
                return Err(
                    "species key times must be finite and strictly increasing from zero".into(),
                );
            }
            if times.is_empty() {
                times = channel_times;
            } else if times != channel_times {
                return Err("species clip channels must share one authored clock".into());
            }
            let Some(gltf::animation::util::ReadOutputs::Rotations(values)) = reader.read_outputs()
            else {
                return Err("missing species rotation keys".into());
            };
            let keys: Vec<_> = values.into_f32().map(Quat::from_array).collect();
            if keys.len() != times.len()
                || keys
                    .iter()
                    .any(|q| !q.is_finite() || (q.length_squared() - 1.0).abs() > 0.000002)
                || keys.first() != Some(&Quat::IDENTITY)
                || keys.last() != Some(&Quat::IDENTITY)
            {
                return Err(
                    "species rotation keys must be finite unit quaternions with neutral endpoints"
                        .into(),
                );
            }
            rotations[target - 1] = keys;
        }
        if rotations.iter().any(Vec::is_empty) {
            return Err("missing species articulated joint channel".into());
        }
        clips.push(SpeciesClip {
            name,
            duration: *times.last().ok_or("empty species clip")?,
            times,
            rotations,
            loop_suggested: metadata.loop_suggested,
            cue_relative: metadata.cue_relative,
            extras: animation.extras().as_ref().unwrap().get().to_owned(),
        });
    }
    clips
        .try_into()
        .map_err(|_| "expected three species clips".into())
}

#[cfg(test)]
#[path = "species_rig_tests.rs"]
mod tests;
