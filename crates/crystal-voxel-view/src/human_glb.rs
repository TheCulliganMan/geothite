//! Bounded reader for the authored rigid-node human catalog, not a general glTF
//! renderer. Each named scene must contain the established 16-joint hierarchy.
//! Unsupported skinning, morphs, materials, transforms and animation modes are
//! errors, never static fallbacks. Only a requested scene's vertices and clips
//! are decoded; the existing renderer still owns mesh handles and entity life.
use std::collections::{HashMap, HashSet};

use bevy::prelude::{Quat, Transform, Vec3};
use gltf::{animation::Interpolation, mesh::Semantic};

use super::{CharacterRig, JOINT_COUNT, Joint, NAMES, PARENTS};
use crate::mesh::SurfaceMeshData;

const MAX_GLB_BYTES: usize = 16 * 1024 * 1024;
const MAX_SCENES: usize = 256;
const MAX_VERTICES: usize = 1_000_000;
const MAX_KEYS: usize = 10_000;
const MAX_SCENE_VERTICES: usize = 100_000;
const MAX_SCENE_INDICES: usize = 600_000;
const MAX_SCENE_ANIMATION_SAMPLES: usize = 100_000;

pub(super) struct Catalog {
    gltf: gltf::Gltf,
    scenes: HashMap<String, Scene>,
}

struct Scene {
    /// Canonical runtime joint order, independent of file node ordering.
    nodes: [usize; JOINT_COUNT],
    animations: Vec<usize>,
    animation_samples: usize,
}

/// Imported clips are opt-in data. Loading a rig does not change its bind pose,
/// select a clip, advance time, or replace the distance-driven gameplay gait.
#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct NodeClip {
    pub name: Option<String>,
    pub extras: Option<String>,
    pub start: f32,
    pub duration: f32,
    pub channels: Vec<NodeChannel>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct NodeChannel {
    pub joint: usize,
    pub interpolation: Interpolation,
    pub times: Vec<f32>,
    pub values: NodeValues,
}

#[derive(Debug)]
pub(crate) enum NodeValues {
    Translations(Vec<[f32; 3]>),
    Rotations(Vec<[f32; 4]>),
    Scales(Vec<[f32; 3]>),
}

impl NodeClip {
    /// Evaluate an absolute glTF time, clamped independently to each channel's
    /// first/last key. Untargeted components keep their bind values. Looping and
    /// clip selection belong to the caller, and are never enabled on import.
    #[allow(dead_code)]
    pub fn sample(
        &self,
        bind: &[Transform; JOINT_COUNT],
        time: f32,
    ) -> Result<[Transform; JOINT_COUNT], String> {
        if !time.is_finite() {
            return Err("animation time must be finite".into());
        }
        let mut pose = *bind;
        for channel in &self.channels {
            let upper = channel.times.partition_point(|&t| t <= time);
            let left = upper.saturating_sub(1);
            let right = upper.min(channel.times.len() - 1);
            let weight = if left == right || channel.interpolation == Interpolation::Step {
                0.0
            } else {
                ((time - channel.times[left]) / (channel.times[right] - channel.times[left]))
                    .clamp(0.0, 1.0)
            };
            let transform = &mut pose[channel.joint];
            match &channel.values {
                NodeValues::Translations(values) => {
                    transform.translation = Vec3::from_array(values[left])
                        .lerp(Vec3::from_array(values[right]), weight);
                }
                NodeValues::Scales(values) => {
                    transform.scale = Vec3::from_array(values[left])
                        .lerp(Vec3::from_array(values[right]), weight);
                }
                NodeValues::Rotations(values) => {
                    transform.rotation = Quat::from_array(values[left])
                        .normalize()
                        .slerp(Quat::from_array(values[right]).normalize(), weight)
                        .normalize();
                }
            }
        }
        Ok(pose)
    }
}

impl Catalog {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_GLB_BYTES || !bytes.starts_with(b"glTF") {
            return Err("human catalog must be a bounded binary GLB".into());
        }
        let gltf = gltf::Gltf::from_slice(bytes).map_err(|e| format!("invalid human GLB: {e}"))?;
        let blob = gltf
            .blob
            .as_deref()
            .ok_or("human GLB requires a binary buffer")?;
        if gltf.buffers().len() != 1
            || gltf.buffers().any(|b| {
                !matches!(b.source(), gltf::buffer::Source::Bin) || b.length() > blob.len()
            })
        {
            return Err(
                "human GLB requires one embedded buffer; external data is unsupported".into(),
            );
        }
        if gltf.skins().len() != 0 {
            return Err("human GLB skinning is unsupported; use rigid joint nodes".into());
        }
        if gltf.images().len() != 0 || gltf.textures().len() != 0 || gltf.cameras().len() != 0 {
            return Err("human GLB images, textures and cameras are unsupported".into());
        }
        if gltf.extensions_used().next().is_some() || gltf.extensions_required().next().is_some() {
            return Err("human GLB extensions are unsupported".into());
        }
        // gltf validates JSON references, but its utility readers assume binary
        // slices are in bounds. Check counts, offsets and strides before reading.
        validate_accessors(&gltf, gltf.buffers().next().unwrap().length())?;
        if gltf.scenes().len() == 0
            || gltf.scenes().len() > MAX_SCENES
            || gltf.nodes().len() > MAX_SCENES * JOINT_COUNT
        {
            return Err("human GLB scene or node count exceeds supported bounds".into());
        }
        let mut scenes = HashMap::new();
        let mut owners = vec![None; gltf.nodes().len()];
        let mut scene_names = Vec::new();
        for scene in gltf.scenes() {
            let name = scene
                .name()
                .filter(|n| !n.is_empty())
                .ok_or("human GLB scenes must be named")?;
            let roots: Vec<_> = scene.nodes().collect();
            if roots.len() != 1 {
                return Err(format!(
                    "human scene {name} must have exactly one pelvis root"
                ));
            }
            let mut nodes = [usize::MAX; JOINT_COUNT];
            let mut vertices = 0_usize;
            let mut indices = 0_usize;
            let mut stack = vec![(roots[0].clone(), None)];
            while let Some((node, parent)) = stack.pop() {
                let joint = NAMES
                    .iter()
                    .position(|&name| Some(name) == node.name())
                    .ok_or("unknown or unnamed human joint")?;
                if nodes[joint] != usize::MAX || owners[node.index()].is_some() {
                    return Err("human GLB has repeated, shared or cyclic joint nodes".into());
                }
                if PARENTS[joint] != parent {
                    return Err(format!("invalid human parent for {}", NAMES[joint]));
                }
                bind_transform(&node)?;
                if node.skin().is_some() || node.weights().is_some() {
                    return Err("human GLB skinning and morph weights are unsupported".into());
                }
                let mesh = node
                    .mesh()
                    .ok_or("every human joint must own real geometry")?;
                if mesh.weights().is_some() || mesh.primitives().len() == 0 {
                    return Err(
                        "human GLB morph weights or empty joint mesh are unsupported".into(),
                    );
                }
                for primitive in mesh.primitives() {
                    validate_primitive(&primitive)?;
                    vertices += primitive.get(&Semantic::Positions).unwrap().count();
                    indices += primitive.indices().unwrap().count();
                    if vertices > MAX_SCENE_VERTICES || indices > MAX_SCENE_INDICES {
                        return Err(
                            "human GLB scene geometry exceeds supported allocation bounds".into(),
                        );
                    }
                }
                nodes[joint] = node.index();
                owners[node.index()] = Some(scene.index());
                for child in node.children() {
                    stack.push((child, Some(joint)));
                }
            }
            if nodes.contains(&usize::MAX) {
                return Err(format!("human scene {name} is missing a canonical joint"));
            }
            if scenes
                .insert(
                    name.to_owned(),
                    Scene {
                        nodes,
                        animations: Vec::new(),
                        animation_samples: 0,
                    },
                )
                .is_some()
            {
                return Err("human GLB scene names must be unique".into());
            }
            scene_names.push(name.to_owned());
        }
        if owners.iter().any(Option::is_none) {
            return Err("human GLB contains nodes outside its named scenes".into());
        }
        // gltf 1.4 validates animation samplers but does not traverse channel
        // target validation. Guard these raw references before its convenience
        // methods can unwrap an invalid node index or property.
        for animation in &gltf.as_json().animations {
            for channel in &animation.channels {
                if channel.target.node.value() >= owners.len()
                    || !matches!(
                        channel.target.path,
                        gltf::json::validation::Checked::Valid(_)
                    )
                {
                    return Err("invalid human GLB animation target node or property".into());
                }
            }
        }
        // Determine clip ownership from targets, never a name prefix. Reject
        // clips spanning scenes so no channel can disappear during lazy loading.
        let mut clip_names = HashSet::new();
        for animation in gltf.animations() {
            let mut owner = None;
            let mut targets = HashSet::new();
            let mut samples = 0;
            for channel in animation.channels() {
                let node = channel.target().node().index();
                let scene = owners[node].ok_or("animation targets an unowned node")?;
                if owner
                    .replace(scene)
                    .is_some_and(|previous| previous != scene)
                {
                    return Err("human GLB animation cannot span multiple scenes".into());
                }
                let property = channel.target().property();
                if property == gltf::animation::Property::MorphTargetWeights {
                    return Err("human GLB morph animation is unsupported".into());
                }
                let property_id = match property {
                    gltf::animation::Property::Translation => 0,
                    gltf::animation::Property::Rotation => 1,
                    gltf::animation::Property::Scale => 2,
                    gltf::animation::Property::MorphTargetWeights => unreachable!(),
                };
                if !targets.insert((node, property_id)) {
                    return Err("human GLB clip has duplicate node/property channels".into());
                }
                if !matches!(
                    channel.sampler().interpolation(),
                    Interpolation::Step | Interpolation::Linear
                ) {
                    return Err(
                        "human GLB animation supports STEP/LINEAR only; CUBICSPLINE is unsupported"
                            .into(),
                    );
                }
                let input = channel.sampler().input();
                samples += input.count();
                let output = channel.sampler().output();
                let dimensions = if property_id == 1 {
                    gltf::accessor::Dimensions::Vec4
                } else {
                    gltf::accessor::Dimensions::Vec3
                };
                if input.data_type() != gltf::accessor::DataType::F32
                    || input.dimensions() != gltf::accessor::Dimensions::Scalar
                    || output.data_type() != gltf::accessor::DataType::F32
                    || output.dimensions() != dimensions
                    || input.count() == 0
                    || input.count() > MAX_KEYS
                    || input.count() != output.count()
                {
                    return Err(
                        "invalid human GLB animation accessor schema or sample count".into(),
                    );
                }
            }
            let owner = owner.ok_or("empty human GLB animation")?;
            if let Some(name) = animation.name() {
                if !clip_names.insert((owner, name.to_owned())) {
                    return Err("human GLB clip names must be unique within each scene".into());
                }
            }
            let scene = scenes.get_mut(&scene_names[owner]).unwrap();
            scene.animation_samples += samples;
            if scene.animations.len() >= 64 || scene.animation_samples > MAX_SCENE_ANIMATION_SAMPLES
            {
                return Err("human GLB scene animation exceeds supported allocation bounds".into());
            }
            scene.animations.push(animation.index());
        }
        Ok(Self { gltf, scenes })
    }

    #[allow(dead_code)]
    pub fn scene_names(&self) -> impl Iterator<Item = &str> {
        self.gltf.scenes().map(|scene| scene.name().unwrap())
    }

    pub fn rig(&self, scene_name: &str) -> Result<CharacterRig, String> {
        let scene = self
            .scenes
            .get(scene_name)
            .ok_or_else(|| format!("missing human GLB scene {scene_name}"))?;
        let blob = self.gltf.blob.as_deref().unwrap();
        let mut joints = Vec::with_capacity(JOINT_COUNT);
        for (joint, &index) in scene.nodes.iter().enumerate() {
            let node = self.gltf.nodes().nth(index).unwrap();
            let mut mesh = SurfaceMeshData::default();
            for primitive in node.mesh().unwrap().primitives() {
                let reader = primitive.reader(|_| Some(blob));
                let positions: Vec<_> = reader
                    .read_positions()
                    .ok_or("missing human GLB positions")?
                    .collect();
                let normals: Vec<_> = reader
                    .read_normals()
                    .ok_or("missing human GLB normals")?
                    .collect();
                let indices: Vec<_> = reader
                    .read_indices()
                    .ok_or("missing human GLB indices")?
                    .into_u32()
                    .collect();
                if positions.is_empty()
                    || positions.len() != normals.len()
                    || positions.len() + mesh.positions.len() > MAX_VERTICES
                    || indices.is_empty()
                    || indices.len() % 3 != 0
                    || indices.iter().any(|&i| i as usize >= positions.len())
                    || positions
                        .iter()
                        .flatten()
                        .chain(normals.iter().flatten())
                        .any(|v| !v.is_finite())
                    || normals.iter().any(|n| {
                        !Vec3::from_array(*n).length_squared().is_finite()
                            || Vec3::from_array(*n).length_squared() < 0.5
                    })
                {
                    return Err("invalid human GLB triangle geometry or normals".into());
                }
                let base = mesh.positions.len() as u32;
                let color = primitive
                    .material()
                    .pbr_metallic_roughness()
                    .base_color_factor();
                mesh.uvs
                    .extend(std::iter::repeat_n([0.0, 0.0], positions.len()));
                // glTF baseColorFactor and the authored source are already
                // linear RGBA. No sRGB conversion belongs on this path.
                mesh.colors
                    .extend(std::iter::repeat_n(color, positions.len()));
                mesh.positions.extend(positions);
                // Match the established loader's normalization exactly.
                mesh.normals.extend(
                    normals
                        .into_iter()
                        .map(|n| Vec3::from_array(n).normalize().to_array()),
                );
                mesh.indices.extend(indices.into_iter().map(|i| base + i));
            }
            joints.push(Joint {
                parent: PARENTS[joint],
                bind: bind_transform(&node)?,
                mesh,
            });
        }
        let mut animations = Vec::with_capacity(scene.animations.len());
        for &index in &scene.animations {
            animations.push(self.clip(index, &scene.nodes)?);
        }
        let locomotion = ["walk", "run"].map(|movement| {
            let name = format!("{scene_name}.{movement}");
            animations
                .iter()
                .position(|clip| clip.name.as_deref() == Some(name.as_str()))
        });
        let locomotion = match locomotion {
            [Some(walk), Some(run)] => Some([walk, run]),
            _ => None,
        };
        Ok(CharacterRig {
            joints: joints.try_into().map_err(|_| "invalid human joint count")?,
            scene_name: scene_name.to_owned(),
            animations,
            locomotion,
        })
    }

    fn clip(&self, index: usize, nodes: &[usize; JOINT_COUNT]) -> Result<NodeClip, String> {
        let animation = self.gltf.animations().nth(index).unwrap();
        let blob = self.gltf.blob.as_deref().unwrap();
        let mut channels = Vec::new();
        let mut start = f32::INFINITY;
        let mut end = 0.0_f32;
        for channel in animation.channels() {
            let reader = channel.reader(|_| Some(blob));
            let times: Vec<_> = reader
                .read_inputs()
                .ok_or("missing human animation input")?
                .collect();
            if times.is_empty()
                || times.iter().any(|t| !t.is_finite() || *t < 0.0)
                || times.windows(2).any(|pair| pair[0] >= pair[1])
            {
                return Err(
                    "human animation times must be finite, nonnegative and strictly increasing"
                        .into(),
                );
            }
            start = start.min(times[0]);
            end = end.max(*times.last().unwrap());
            use gltf::animation::util::ReadOutputs;
            let values = match reader
                .read_outputs()
                .ok_or("missing human animation output")?
            {
                ReadOutputs::Translations(values) => NodeValues::Translations(values.collect()),
                ReadOutputs::Rotations(values) => {
                    NodeValues::Rotations(values.into_f32().collect())
                }
                ReadOutputs::Scales(values) => NodeValues::Scales(values.collect()),
                ReadOutputs::MorphTargetWeights(_) => {
                    return Err("human GLB morph animation is unsupported".into());
                }
            };
            match &values {
                NodeValues::Translations(values) | NodeValues::Scales(values) => {
                    if values.len() != times.len()
                        || values.iter().flatten().any(|v| !v.is_finite())
                    {
                        return Err("invalid human animation vector samples".into());
                    }
                }
                NodeValues::Rotations(values) => {
                    if values.len() != times.len() || values.iter().any(|q| !valid_rotation(*q)) {
                        return Err("invalid human animation quaternion samples".into());
                    }
                }
            }
            channels.push(NodeChannel {
                joint: nodes
                    .iter()
                    .position(|&node| node == channel.target().node().index())
                    .ok_or("clip target outside human scene")?,
                interpolation: channel.sampler().interpolation(),
                times,
                values,
            });
        }
        Ok(NodeClip {
            name: animation.name().map(str::to_owned),
            extras: animation
                .extras()
                .as_ref()
                .map(|extras| extras.get().to_owned()),
            start,
            duration: end - start,
            channels,
        })
    }
}

fn valid_rotation(rotation: [f32; 4]) -> bool {
    rotation.iter().all(|v| v.is_finite())
        && (Quat::from_array(rotation).length_squared() - 1.0).abs() <= 0.001
}

fn bind_transform(node: &gltf::Node<'_>) -> Result<Transform, String> {
    let gltf::scene::Transform::Decomposed {
        translation,
        rotation,
        scale,
    } = node.transform()
    else {
        return Err(
            "human GLB matrix transforms are unsupported; author explicit TRS binds".into(),
        );
    };
    if translation.iter().chain(&scale).any(|v| !v.is_finite())
        || scale.iter().any(|&v| v == 0.0)
        || !valid_rotation(rotation)
    {
        return Err("invalid human GLB bind transform".into());
    }
    // The current gameplay foot solver and authored locomotion use the
    // established translation-only bind frame. Reject a changed basis instead
    // of importing it successfully and silently composing the wrong gait.
    if rotation[..3] != [0.0; 3] || rotation[3].abs() != 1.0 || scale != [1.0; 3] {
        return Err("human GLB requires identity bind rotation and unit bind scale".into());
    }
    Ok(Transform {
        translation: Vec3::from_array(translation),
        rotation: Quat::from_array(rotation),
        scale: Vec3::from_array(scale),
    })
}

fn validate_accessors(gltf: &gltf::Gltf, binary_len: usize) -> Result<(), String> {
    for view in gltf.views() {
        if view
            .offset()
            .checked_add(view.length())
            .is_none_or(|end| end > binary_len)
        {
            return Err("human GLB buffer view exceeds embedded bytes".into());
        }
    }
    for accessor in gltf.accessors() {
        if accessor.sparse().is_some() || accessor.normalized() {
            return Err("human GLB sparse and normalized accessors are unsupported".into());
        }
        let view = accessor
            .view()
            .ok_or("human GLB accessor requires a buffer view")?;
        let stride = view.stride().unwrap_or(accessor.size());
        if accessor.count() == 0 || accessor.count() > MAX_VERTICES * 6 || stride < accessor.size()
        {
            return Err("human GLB accessor exceeds supported count or stride".into());
        }
        let end = (accessor.count() - 1)
            .checked_mul(stride)
            .and_then(|size| size.checked_add(accessor.size()))
            .and_then(|size| size.checked_add(accessor.offset()));
        if end.is_none_or(|end| end > view.length()) {
            return Err("human GLB accessor exceeds its buffer view".into());
        }
    }
    Ok(())
}

fn validate_primitive(primitive: &gltf::Primitive<'_>) -> Result<(), String> {
    if primitive.mode() != gltf::mesh::Mode::Triangles || primitive.morph_targets().next().is_some()
    {
        return Err(
            "human GLB supports indexed rigid triangles only, without morph targets".into(),
        );
    }
    for (semantic, accessor) in primitive.attributes() {
        if !matches!(semantic, Semantic::Positions | Semantic::Normals) {
            return Err(format!(
                "unsupported human GLB vertex attribute {semantic:?}"
            ));
        }
        if accessor.data_type() != gltf::accessor::DataType::F32
            || accessor.dimensions() != gltf::accessor::Dimensions::Vec3
        {
            return Err("human GLB positions and normals must be f32 VEC3".into());
        }
    }
    let positions = primitive
        .get(&Semantic::Positions)
        .ok_or("missing human GLB positions")?;
    let normals = primitive
        .get(&Semantic::Normals)
        .ok_or("missing human GLB normals")?;
    let indices = primitive
        .indices()
        .ok_or("human GLB requires triangle indices")?;
    if positions.count() != normals.count()
        || positions.count() > MAX_VERTICES
        || indices.dimensions() != gltf::accessor::Dimensions::Scalar
        || !matches!(
            indices.data_type(),
            gltf::accessor::DataType::U8
                | gltf::accessor::DataType::U16
                | gltf::accessor::DataType::U32
        )
    {
        return Err("invalid human GLB geometry accessor schema".into());
    }
    let material = primitive.material();
    let pbr = material.pbr_metallic_roughness();
    if pbr
        .base_color_factor()
        .iter()
        .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        || pbr.base_color_texture().is_some()
        || pbr.metallic_roughness_texture().is_some()
        || material.normal_texture().is_some()
        || material.occlusion_texture().is_some()
        || material.emissive_texture().is_some()
        || material.emissive_factor() != [0.0; 3]
        || pbr.metallic_factor() != 0.0
        || pbr.roughness_factor() != 1.0
        || material.double_sided()
        || material.alpha_mode() == gltf::material::AlphaMode::Mask
        || (material.alpha_mode() == gltf::material::AlphaMode::Opaque
            && pbr.base_color_factor()[3] != 1.0)
    {
        return Err("human GLB supports untextured linear RGBA materials with metallic=0 and roughness=1 only".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    struct Fixture {
        json: Value,
        binary: Vec<u8>,
    }

    impl Fixture {
        fn new() -> Self {
            let mut fixture = Self {
                json: json!({
                    "asset": {"version": "2.0"}, "scene": 0,
                    "scenes": [{"name": "fixture", "nodes": [0]}],
                    "nodes": [], "meshes": [],
                    "materials": [{"pbrMetallicRoughness": {"baseColorFactor": [0.2, 0.4, 0.6, 0.8],
                        "metallicFactor": 0, "roughnessFactor": 1}, "alphaMode": "BLEND"}],
                    "accessors": [], "bufferViews": [], "animations": []
                }),
                binary: Vec::new(),
            };
            let positions =
                fixture.floats(&[-0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0], "VEC3", 3);
            fixture.json["accessors"][positions]["min"] = json!([0, 0, 0]);
            fixture.json["accessors"][positions]["max"] = json!([1, 1, 0]);
            let normals = fixture.floats(&[0.0, 0.0, 2.0, 0.0, 0.0, 2.0, 0.0, 0.0, 2.0], "VEC3", 3);
            let indices = fixture.accessor(&[0, 0, 1, 0, 2, 0], "SCALAR", 5123, 3);
            fixture.json["meshes"] = json!([{"primitives": [{
                "attributes": {"POSITION": positions, "NORMAL": normals},
                "indices": indices, "material": 0, "mode": 4
            }]}]);
            for (joint, name) in NAMES.iter().enumerate() {
                let children: Vec<_> = PARENTS
                    .iter()
                    .enumerate()
                    .filter_map(|(i, parent)| (*parent == Some(joint)).then_some(i))
                    .collect();
                let mut node = json!({"name": name, "mesh": 0, "translation": [0, 0.25, 0]});
                if !children.is_empty() {
                    node["children"] = json!(children);
                }
                fixture.json["nodes"].as_array_mut().unwrap().push(node);
            }
            fixture
        }

        fn accessor(
            &mut self,
            data: &[u8],
            dimensions: &str,
            component: u32,
            count: usize,
        ) -> usize {
            while self.binary.len() % 4 != 0 {
                self.binary.push(0);
            }
            let views = self.json["bufferViews"].as_array_mut().unwrap();
            let view = views.len();
            views.push(
                json!({"buffer": 0, "byteOffset": self.binary.len(), "byteLength": data.len()}),
            );
            self.binary.extend_from_slice(data);
            let accessors = self.json["accessors"].as_array_mut().unwrap();
            let index = accessors.len();
            accessors.push(json!({"bufferView": view, "componentType": component, "count": count, "type": dimensions}));
            index
        }

        fn floats(&mut self, values: &[f32], dimensions: &str, count: usize) -> usize {
            let bytes: Vec<_> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
            self.accessor(&bytes, dimensions, 5126, count)
        }

        fn animation(&mut self, interpolation: &str) {
            let input = self.floats(&[1.0, 3.0], "SCALAR", 2);
            self.json["accessors"][input]["min"] = json!([1]);
            self.json["accessors"][input]["max"] = json!([3]);
            let translation = self.floats(&[0.0, 1.0, 2.0, 4.0, 3.0, 0.0], "VEC3", 2);
            let scale = self.floats(&[1.0, 1.0, 1.0, 3.0, 2.0, 0.5], "VEC3", 2);
            let q = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2).to_array();
            let rotation =
                self.floats(&[0.0, 0.0, 0.0, 1.0, -q[0], -q[1], -q[2], -q[3]], "VEC4", 2);
            self.json["animations"] = json!([{
                "name": "fixture.motion", "extras": {"purpose": "test"},
                "samplers": [
                    {"input": input, "output": translation, "interpolation": interpolation},
                    {"input": input, "output": scale, "interpolation": interpolation},
                    {"input": input, "output": rotation, "interpolation": interpolation}
                ],
                "channels": [
                    {"sampler": 0, "target": {"node": 0, "path": "translation"}},
                    {"sampler": 1, "target": {"node": 0, "path": "scale"}},
                    {"sampler": 2, "target": {"node": 1, "path": "rotation"}}
                ]
            }]);
        }

        fn bytes(&self) -> Vec<u8> {
            let mut json = self.json.clone();
            json["buffers"] = json!([{"byteLength": self.binary.len()}]);
            let mut json = serde_json::to_vec(&json).unwrap();
            while json.len() % 4 != 0 {
                json.push(b' ');
            }
            let mut binary = self.binary.clone();
            while binary.len() % 4 != 0 {
                binary.push(0);
            }
            let mut output = Vec::new();
            for value in [
                0x46546c67_u32,
                2,
                (28 + json.len() + binary.len()) as u32,
                json.len() as u32,
                0x4e4f534a,
            ] {
                output.extend(value.to_le_bytes());
            }
            output.extend(json);
            output.extend((binary.len() as u32).to_le_bytes());
            output.extend(0x004e4942_u32.to_le_bytes());
            output.extend(binary);
            output
        }

        fn error(&self) -> String {
            match Catalog::parse(&self.bytes()).and_then(|catalog| catalog.rig("fixture")) {
                Err(error) => error,
                Ok(_) => panic!("invalid fixture was accepted"),
            }
        }
    }

    #[test]
    fn binary_geometry_preserves_bits_materials_binds_and_primitive_offsets() {
        let mut fixture = Fixture::new();
        fixture.json["nodes"][0]["translation"] = json!([-0.0, 0.25, 0.5]);
        let mut material = fixture.json["materials"][0].clone();
        material["pbrMetallicRoughness"]["baseColorFactor"] = json!([0.7, 0.1, 0.3, 1.0]);
        fixture.json["materials"]
            .as_array_mut()
            .unwrap()
            .push(material);
        let mut primitive = fixture.json["meshes"][0]["primitives"][0].clone();
        primitive["material"] = json!(1);
        fixture.json["meshes"][0]["primitives"]
            .as_array_mut()
            .unwrap()
            .push(primitive);
        let catalog = Catalog::parse(&fixture.bytes()).unwrap();
        assert_eq!(catalog.scene_names().collect::<Vec<_>>(), ["fixture"]);
        let rig = catalog.rig("fixture").unwrap();
        let joint = &rig.joints[0];
        assert_eq!(
            joint.bind.translation.to_array().map(f32::to_bits),
            [-0.0_f32, 0.25, 0.5].map(f32::to_bits)
        );
        assert_eq!(joint.bind.rotation, Quat::IDENTITY);
        assert_eq!(joint.bind.scale, Vec3::ONE);
        assert_eq!(joint.mesh.positions[0][0].to_bits(), (-0.0_f32).to_bits());
        assert_eq!(joint.mesh.normals, vec![[0.0, 0.0, 1.0]; 6]);
        assert_eq!(
            joint.mesh.colors[0].map(f32::to_bits),
            [0.2_f32, 0.4, 0.6, 0.8].map(f32::to_bits)
        );
        assert_eq!(
            joint.mesh.colors[3].map(f32::to_bits),
            [0.7_f32, 0.1, 0.3, 1.0].map(f32::to_bits)
        );
        assert_eq!(joint.mesh.indices, [0, 1, 2, 3, 4, 5]);
        for (i, joint) in rig.joints.iter().enumerate() {
            assert_eq!(joint.parent, PARENTS[i]);
        }
        assert!(rig.animations.is_empty());
    }

    #[test]
    fn unsigned_index_component_widths_preserve_triangle_values() {
        for (component, data) in [
            (5121, vec![0_u8, 1, 2]),
            (
                5123,
                [0_u16, 1, 2]
                    .into_iter()
                    .flat_map(u16::to_le_bytes)
                    .collect(),
            ),
            (
                5125,
                [0_u32, 1, 2]
                    .into_iter()
                    .flat_map(u32::to_le_bytes)
                    .collect(),
            ),
        ] {
            let mut fixture = Fixture::new();
            let indices = fixture.accessor(&data, "SCALAR", component, 3);
            fixture.json["meshes"][0]["primitives"][0]["indices"] = json!(indices);
            let rig = Catalog::parse(&fixture.bytes())
                .unwrap()
                .rig("fixture")
                .unwrap();
            assert_eq!(rig.joints[0].mesh.indices, [0, 1, 2]);
        }
    }

    #[test]
    fn linear_trs_samples_clamp_and_slerp_without_changing_bind_pose() {
        let mut fixture = Fixture::new();
        fixture.animation("LINEAR");
        let rig = Catalog::parse(&fixture.bytes())
            .unwrap()
            .rig("fixture")
            .unwrap();
        let bind = rig.joints.each_ref().map(|joint| joint.bind);
        let clip = &rig.animations[0];
        assert_eq!(clip.name.as_deref(), Some("fixture.motion"));
        assert_eq!(clip.extras.as_deref(), Some("{\"purpose\":\"test\"}"));
        assert_eq!((clip.start, clip.duration), (1.0, 2.0));
        assert_eq!(clip.channels[0].times, [1.0, 3.0]);
        assert_eq!(clip.channels[0].interpolation, Interpolation::Linear);
        let NodeValues::Rotations(rotations) = &clip.channels[2].values else {
            panic!("rotation channel lost");
        };
        let original = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)
            .to_array()
            .map(|v| -v);
        assert_eq!(rotations[1].map(f32::to_bits), original.map(f32::to_bits));
        let pose = clip.sample(&bind, 2.0).unwrap();
        assert_eq!(pose[0].translation, Vec3::new(2.0, 2.0, 1.0));
        assert_eq!(pose[0].scale, Vec3::new(2.0, 1.5, 0.75));
        assert!(
            pose[1]
                .rotation
                .angle_between(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4))
                < 0.001
        );
        assert_eq!(pose[1].translation, bind[1].translation);
        assert_eq!(pose[2], bind[2]);
        assert_eq!(
            clip.sample(&bind, -5.0).unwrap()[0].translation,
            Vec3::new(0.0, 1.0, 2.0)
        );
        assert_eq!(
            clip.sample(&bind, 7.0).unwrap()[0].translation,
            Vec3::new(4.0, 3.0, 0.0)
        );
        assert_eq!(bind[0], rig.joints[0].bind);
        assert!(clip.sample(&bind, f32::NAN).is_err());
    }

    #[test]
    fn step_animation_changes_at_exact_key_and_keeps_prior_value_between_keys() {
        let mut fixture = Fixture::new();
        fixture.animation("STEP");
        let rig = Catalog::parse(&fixture.bytes())
            .unwrap()
            .rig("fixture")
            .unwrap();
        let bind = rig.joints.each_ref().map(|joint| joint.bind);
        let clip = &rig.animations[0];
        assert_eq!(
            clip.sample(&bind, 2.999).unwrap()[0].translation,
            Vec3::new(0.0, 1.0, 2.0)
        );
        assert_eq!(
            clip.sample(&bind, 3.0).unwrap()[0].translation,
            Vec3::new(4.0, 3.0, 0.0)
        );
    }

    #[test]
    fn geometry_stays_lazy_and_animation_targets_cannot_cross_scenes() {
        let mut fixture = Fixture::new();
        fixture.animation("LINEAR");
        let invalid_positions = fixture.floats(
            &[f32::NAN, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            "VEC3",
            3,
        );
        fixture.json["accessors"][invalid_positions]["min"] = json!([0, 0, 0]);
        fixture.json["accessors"][invalid_positions]["max"] = json!([1, 1, 0]);
        let mut mesh = fixture.json["meshes"][0].clone();
        mesh["primitives"][0]["attributes"]["POSITION"] = json!(invalid_positions);
        fixture.json["meshes"].as_array_mut().unwrap().push(mesh);
        let mut second_nodes = fixture.json["nodes"].as_array().unwrap().clone();
        for node in &mut second_nodes {
            node["mesh"] = json!(1);
            if let Some(children) = node["children"].as_array_mut() {
                for child in children {
                    *child = json!(child.as_u64().unwrap() + JOINT_COUNT as u64);
                }
            }
        }
        fixture.json["nodes"]
            .as_array_mut()
            .unwrap()
            .extend(second_nodes);
        fixture.json["scenes"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name": "second", "nodes": [JOINT_COUNT]}));
        let catalog = Catalog::parse(&fixture.bytes()).unwrap();
        assert!(catalog.rig("fixture").is_ok());
        assert!(catalog.rig("second").is_err());
        fixture.json["animations"][0]["channels"][2]["target"]["node"] = json!(JOINT_COUNT + 1);
        assert!(fixture.error().contains("span multiple scenes"));
    }

    #[test]
    fn production_locomotion_requires_duration_and_complete_articulation() {
        let mut fixture = Fixture::new();
        fixture.animation("LINEAR");
        fixture.json["animations"][0]["name"] = json!("fixture.walk");
        let mut run = fixture.json["animations"][0].clone();
        run["name"] = json!("fixture.run");
        fixture.json["animations"].as_array_mut().unwrap().push(run);
        let rig = Catalog::parse(&fixture.bytes())
            .unwrap()
            .rig("fixture")
            .unwrap();
        assert!(rig.require_locomotion().err().unwrap().contains("missing"));
        let samplers = fixture.json["animations"][0]["samplers"]
            .as_array()
            .unwrap()
            .clone();
        for sampler in samplers {
            for field in ["input", "output"] {
                let index = sampler[field].as_u64().unwrap() as usize;
                fixture.json["accessors"][index]["count"] = json!(1);
            }
        }
        let rig = Catalog::parse(&fixture.bytes())
            .unwrap()
            .rig("fixture")
            .unwrap();
        assert!(
            rig.require_locomotion()
                .err()
                .unwrap()
                .contains("zero-duration")
        );
    }

    #[test]
    fn unsupported_deformation_and_materials_are_errors() {
        let mut fixture = Fixture::new();
        fixture.json["skins"] = json!([{"joints": [0]}]);
        assert!(fixture.error().contains("skinning"));
        let mut fixture = Fixture::new();
        fixture.json["meshes"][0]["primitives"][0]["targets"] = json!([{"POSITION": 0}]);
        assert!(fixture.error().contains("morph"));
        let mut fixture = Fixture::new();
        fixture.json["materials"][0]["pbrMetallicRoughness"]["metallicFactor"] = json!(0.5);
        assert!(fixture.error().contains("materials"));
        let mut fixture = Fixture::new();
        fixture.json["nodes"][0]
            .as_object_mut()
            .unwrap()
            .remove("translation");
        fixture.json["nodes"][0]["matrix"] =
            json!([1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]);
        assert!(fixture.error().contains("matrix transforms"));
        let mut fixture = Fixture::new();
        fixture.animation("CUBICSPLINE");
        assert!(fixture.error().contains("CUBICSPLINE"));
        let mut fixture = Fixture::new();
        fixture.json["nodes"][0]["rotation"] = json!(Quat::from_rotation_x(0.25).to_array());
        assert!(fixture.error().contains("identity bind rotation"));
        let mut fixture = Fixture::new();
        fixture.json["nodes"][0]["scale"] = json!([1.0, 2.0, 1.0]);
        assert!(fixture.error().contains("unit bind scale"));
    }

    #[test]
    fn malformed_hierarchy_buffers_geometry_and_animation_fail_clearly() {
        let mut fixture = Fixture::new();
        fixture.json["nodes"][1]["name"] = json!("mystery");
        assert!(fixture.error().contains("unknown"));
        let mut fixture = Fixture::new();
        fixture.json["nodes"][0]["children"] = json!([1, 2, 10, 13]);
        assert!(fixture.error().contains("parent"));
        let mut fixture = Fixture::new();
        fixture.json["accessors"][0]["count"] = json!(99);
        assert!(fixture.error().contains("exceeds"));
        let mut fixture = Fixture::new();
        let index_view = fixture.json["accessors"][2]["bufferView"].as_u64().unwrap() as usize;
        let offset = fixture.json["bufferViews"][index_view]["byteOffset"]
            .as_u64()
            .unwrap() as usize;
        fixture.binary[offset] = 100;
        assert!(fixture.error().contains("triangle geometry"));
        let mut fixture = Fixture::new();
        fixture.animation("LINEAR");
        let input = fixture.json["animations"][0]["samplers"][0]["input"]
            .as_u64()
            .unwrap() as usize;
        let view = fixture.json["accessors"][input]["bufferView"]
            .as_u64()
            .unwrap() as usize;
        let offset = fixture.json["bufferViews"][view]["byteOffset"]
            .as_u64()
            .unwrap() as usize;
        fixture.binary[offset..offset + 4].copy_from_slice(&3.0_f32.to_le_bytes());
        assert!(fixture.error().contains("strictly increasing"));
        let mut fixture = Fixture::new();
        fixture.animation("LINEAR");
        let channel = fixture.json["animations"][0]["channels"][0].clone();
        fixture.json["animations"][0]["channels"]
            .as_array_mut()
            .unwrap()
            .push(channel);
        assert!(fixture.error().contains("duplicate"));
        for (field, value) in [("node", json!(99999)), ("path", json!("not_a_property"))] {
            let mut fixture = Fixture::new();
            fixture.animation("LINEAR");
            fixture.json["animations"][0]["channels"][0]["target"][field] = value;
            assert!(fixture.error().contains("target node or property"));
        }
    }
}
