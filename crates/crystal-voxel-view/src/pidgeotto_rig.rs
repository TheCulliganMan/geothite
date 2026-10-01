//! The bounded Pidgeotto rigid-node GLB contract. The canonical asset retains
//! its 34 source anatomy primitives, shoulder hierarchy and standard glTF clip.
//! We batch the immutable geometry into three draw groups once, and sample only
//! their transforms. Neutral vertices keep their original world-coordinate bits.
use std::{ops::Range, sync::OnceLock};

use bevy::prelude::{Quat, Transform, Vec3};
use gltf::{
    accessor::{DataType, Dimensions},
    animation::{Interpolation, Property},
    mesh::Semantic,
};
use serde::Deserialize;

use crate::mesh::SurfaceMeshData;

pub(crate) const GROUP_COUNT: usize = 3;
pub(crate) const CLIP_NAME: &str = "pidgeotto.idle_wings";
const PART_COUNT: usize = 34;
const NODE_COUNT: usize = PART_COUNT + 5;
const DURATION: f32 = 0.8;
const MAX_GLB_BYTES: usize = 1024 * 1024;
const MAX_VERTICES: usize = 20_000;
const MAX_INDICES: usize = 120_000;
const MAX_ACCESSORS: usize = 128;
const MAX_KEYS: usize = 257;
const COORDINATES: &str = "+Y up; front +Z; floor-centered root";

/// Source-order ownership is retained independently of the three draw batches.
#[derive(Debug)]
pub(crate) struct Anatomy {
    pub name: String,
    pub group: usize,
    pub vertices: Range<usize>,
    pub group_vertices: Range<usize>,
}

#[derive(Debug)]
pub(crate) struct RigGroup {
    pub name: &'static str,
    pub mesh: SurfaceMeshData,
    pub parts: Vec<usize>,
}

#[derive(Debug)]
pub(crate) struct PidgeottoRig {
    /// Source-order surface, for existing neutral model/preview consumers.
    pub neutral: SurfaceMeshData,
    /// Body, left wing, right wing. All vertices remain in source coordinates.
    pub groups: [RigGroup; GROUP_COUNT],
    pub anatomy: [Anatomy; PART_COUNT],
    pub pivots: [Vec3; 2],
    pub duration: f32,
    /// Conservative local bounds of every pose, computed once for camera fitting.
    /// Physical height, grounding and placement still use the neutral mesh.
    pub animated_bounds: (Vec3, Vec3),
    times: Vec<f32>,
    rotations: [Vec<Quat>; 2],
}

pub(crate) fn rig() -> &'static PidgeottoRig {
    static RIG: OnceLock<PidgeottoRig> = OnceLock::new();
    RIG.get_or_init(|| {
        let bytes = crate::model_storage::decode_bytes(crate::model_storage::include_model!(
            "models/battle_species/pidgeotto.glb"
        ))
        .expect("valid Pidgeotto GLB storage");
        PidgeottoRig::parse(&bytes).expect("valid authored Pidgeotto rig")
    })
}

impl PidgeottoRig {
    /// Stateless, allocation-free local transforms for the three sibling draw
    /// groups. The caller owns instance time, placement, scale and facing.
    pub fn sample(&self, seconds: f32) -> Result<[Transform; GROUP_COUNT], &'static str> {
        if !seconds.is_finite() {
            return Err("Pidgeotto animation time must be finite");
        }
        let time = seconds.rem_euclid(self.duration);
        let mut result = [Transform::IDENTITY; GROUP_COUNT];
        // Preserve exact identity at the closed loop seam, including translation.
        if time == 0.0 {
            return Ok(result);
        }
        let upper = self.times.partition_point(|&t| t <= time);
        let left = upper.saturating_sub(1);
        let right = upper.min(self.times.len() - 1);
        let weight = if left == right {
            0.0
        } else {
            (time - self.times[left]) / (self.times[right] - self.times[left])
        };
        for wing in 0..2 {
            let rotation = self.rotations[wing][left]
                .slerp(self.rotations[wing][right], weight)
                .normalize();
            let pivot = self.pivots[wing];
            result[wing + 1] = Transform {
                translation: pivot - rotation * pivot,
                rotation,
                scale: Vec3::ONE,
            };
        }
        Ok(result)
    }

    fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 28
            || bytes.len() > MAX_GLB_BYTES
            || !bytes.starts_with(b"glTF")
            || u32::from_le_bytes(bytes[4..8].try_into().unwrap()) != 2
            || u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize != bytes.len()
        {
            return Err("Pidgeotto requires a bounded binary GLB".into());
        }
        reject_extensions(bytes)?;
        let gltf =
            gltf::Gltf::from_slice(bytes).map_err(|e| format!("invalid Pidgeotto GLB: {e}"))?;
        let binary = gltf
            .blob
            .as_deref()
            .ok_or("Pidgeotto GLB needs embedded data")?;
        if gltf.buffers().len() != 1
            || gltf.buffers().any(|buffer| {
                !matches!(buffer.source(), gltf::buffer::Source::Bin)
                    || buffer.length() > binary.len()
                    || binary.len() - buffer.length() > 3
            })
        {
            return Err("Pidgeotto GLB requires exactly one embedded buffer".into());
        }
        if gltf.skins().len() != 0
            || gltf.images().len() != 0
            || gltf.textures().len() != 0
            || gltf.cameras().len() != 0
            || gltf.extensions_used().next().is_some()
            || gltf.extensions_required().next().is_some()
        {
            return Err(
                "Pidgeotto GLB skins, images, textures, cameras and extensions are unsupported"
                    .into(),
            );
        }
        if gltf.nodes().len() != NODE_COUNT
            || gltf.meshes().len() != PART_COUNT
            || gltf.scenes().len() != 1
            || gltf.materials().len() > PART_COUNT
            || gltf.accessors().len() > MAX_ACCESSORS
            || gltf.views().len() > MAX_ACCESSORS
        {
            return Err("Pidgeotto GLB exceeds its fixed anatomy/schema bounds".into());
        }
        validate_accessors(&gltf, gltf.buffers().next().unwrap().length())?;
        for material in gltf.materials() {
            validate_material(&material)?;
        }
        let scene = gltf.scenes().next().unwrap();
        if scene.name() != Some("pidgeotto")
            || scene.nodes().map(|n| n.index()).collect::<Vec<_>>() != [0]
            || gltf.default_scene().map(|s| s.index()) != Some(0)
        {
            return Err("Pidgeotto GLB requires its single named root scene".into());
        }
        let nodes: Vec<_> = gltf.nodes().collect();
        let root = &nodes[0];
        let metadata: RootExtras = extras(root.extras())?;
        if metadata.coordinate_system != COORDINATES
            || metadata.source_schema_version != 1
            || metadata.rig_schema_version != 1
            || metadata.source_mesh_sha256.len() != 64
            || !metadata
                .source_mesh_sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || root.name() != Some("pidgeotto")
        {
            return Err("Pidgeotto GLB source/rig identity is unsupported".into());
        }
        let mut parents = [None; NODE_COUNT];
        for node in &nodes {
            if node.skin().is_some() || node.weights().is_some() || node.camera().is_some() {
                return Err("Pidgeotto nodes cannot carry skins, morphs or cameras".into());
            }
            for child in node.children() {
                if child.index() <= node.index()
                    || parents[child.index()].replace(node.index()).is_some()
                {
                    return Err(
                        "Pidgeotto hierarchy has a cycle, reordered parent or shared child".into(),
                    );
                }
            }
        }
        if parents[0].is_some() || parents[1..].iter().any(Option::is_none) {
            return Err("Pidgeotto hierarchy has an extra or disconnected root".into());
        }
        if bind_translation(root)? != Vec3::ZERO || root.mesh().is_some() {
            return Err("Pidgeotto root must have an identity bind and no geometry".into());
        }
        let mut pivots = [Vec3::ZERO; 2];
        let mut expected_parts = [Vec::new(), Vec::new()];
        for (wing, (hinge, cancel, side)) in [(1, 2, "l"), (3, 4, "r")].into_iter().enumerate() {
            let hinge_node = &nodes[hinge];
            let cancel_node = &nodes[cancel];
            let metadata: HingeExtras = extras(hinge_node.extras())?;
            pivots[wing] = bind_translation(hinge_node)?;
            if hinge_node.name() != Some(format!("wing_{side}_hinge").as_str())
                || cancel_node.name() != Some(format!("wing_{side}_source_coordinates").as_str())
                || parents[hinge] != Some(0)
                || parents[cancel] != Some(hinge)
                || hinge_node.children().map(|n| n.index()).collect::<Vec<_>>() != [cancel]
                || hinge_node.mesh().is_some()
                || cancel_node.mesh().is_some()
                || bind_translation(cancel_node)? != -pivots[wing]
                || !metadata.authored_for_runtime
                || metadata.pivot_basis != "nearest-centerline source layered-wing vertex"
                || metadata.source_primitive_indices.len() != 5
                || metadata
                    .source_primitive_indices
                    .windows(2)
                    .any(|v| v[0] >= v[1])
                || metadata
                    .source_primitive_indices
                    .iter()
                    .any(|&v| v >= PART_COUNT)
            {
                return Err("Pidgeotto requires two authored shoulder/cancellation pairs".into());
            }
            expected_parts[wing] = metadata.source_primitive_indices;
        }
        let mut neutral = SurfaceMeshData::default();
        let mut groups = ["body", "wing_l", "wing_r"].map(|name| RigGroup {
            name,
            mesh: SurfaceMeshData::default(),
            parts: Vec::new(),
        });
        let mut anatomy = Vec::with_capacity(PART_COUNT);
        let mut total_vertices = 0;
        let mut total_indices = 0;
        for index in 0..PART_COUNT {
            let node = &nodes[index + 5];
            let source: PartExtras = extras(node.extras())?;
            let name = node
                .name()
                .filter(|s| !s.is_empty())
                .ok_or("missing Pidgeotto anatomy name")?;
            if source.source_primitive != index
                || anatomy.iter().any(|a: &Anatomy| a.name == name)
                || node.children().next().is_some()
                || bind_translation(node)? != Vec3::ZERO
            {
                return Err("Pidgeotto anatomy order, identity or leaf transform differs".into());
            }
            let group = match parents[index + 5] {
                Some(0) => 0,
                Some(2) => 1,
                Some(4) => 2,
                _ => return Err("unsupported Pidgeotto anatomy parent".into()),
            };
            let mesh = node.mesh().ok_or("Pidgeotto anatomy must own geometry")?;
            if mesh.index() != index
                || mesh.name() != Some(name)
                || mesh.primitives().len() != 1
                || mesh.weights().is_some()
            {
                return Err("Pidgeotto anatomy requires its original single primitive mesh".into());
            }
            let primitive = mesh.primitives().next().unwrap();
            validate_primitive(&primitive)?;
            total_vertices += primitive.get(&Semantic::Positions).unwrap().count();
            total_indices += primitive.indices().unwrap().count();
            if total_vertices > MAX_VERTICES || total_indices > MAX_INDICES {
                return Err("Pidgeotto geometry exceeds allocation bounds".into());
            }
            let part = read_primitive(&primitive, binary)?;
            let vertices = neutral.positions.len()..neutral.positions.len() + part.positions.len();
            let group_vertices = groups[group].mesh.positions.len()
                ..groups[group].mesh.positions.len() + part.positions.len();
            append_mesh(&mut neutral, &part);
            append_mesh(&mut groups[group].mesh, &part);
            groups[group].parts.push(index);
            anatomy.push(Anatomy {
                name: name.to_owned(),
                group,
                vertices,
                group_vertices,
            });
        }
        for wing in 0..2 {
            if groups[wing + 1].parts != expected_parts[wing] {
                return Err("Pidgeotto wing metadata differs from hierarchy ownership".into());
            }
            for &part in &groups[wing + 1].parts {
                let name = &anatomy[part].name;
                if !name.starts_with("Layered wing") && !name.starts_with("Flight feather") {
                    return Err("Pidgeotto wing hierarchy includes non-wing anatomy".into());
                }
            }
            let layered: Vec<_> = groups[wing + 1]
                .parts
                .iter()
                .copied()
                .filter(|&part| anatomy[part].name.starts_with("Layered wing"))
                .collect();
            if layered.len() != 1 {
                return Err("Pidgeotto wing requires one layered shoulder surface".into());
            }
            let positions = &neutral.positions[anatomy[layered[0]].vertices.clone()];
            let expected_pivot = positions
                .iter()
                .min_by(|a, b| {
                    a[0].abs()
                        .total_cmp(&b[0].abs())
                        .then_with(|| b[1].total_cmp(&a[1]))
                        .then_with(|| a[2].total_cmp(&b[2]))
                })
                .unwrap();
            if pivots[wing].to_array() != *expected_pivot
                || groups[wing + 1]
                    .mesh
                    .positions
                    .iter()
                    .any(|p| if wing == 0 { p[0] >= 0.0 } else { p[0] <= 0.0 })
            {
                return Err(
                    "Pidgeotto shoulder pivot or wing side differs from source geometry".into(),
                );
            }
        }
        if groups[0].parts.len() != 24
            || groups[0].parts.iter().any(|&part| {
                anatomy[part].name.starts_with("Layered wing")
                    || anatomy[part].name.starts_with("Flight feather")
            })
        {
            return Err("Pidgeotto body includes wing anatomy or has missing source parts".into());
        }
        let min = neutral
            .positions
            .iter()
            .fold(Vec3::splat(f32::INFINITY), |a, p| {
                a.min(Vec3::from_array(*p))
            });
        let max = neutral
            .positions
            .iter()
            .fold(Vec3::splat(f32::NEG_INFINITY), |a, p| {
                a.max(Vec3::from_array(*p))
            });
        if min.y.abs() > 0.0001
            || (max - min).min_element() < 0.005
            || min.abs().max_element() >= 4.0
            || max.abs().max_element() >= 4.0
        {
            return Err("Pidgeotto neutral geometry must be volumetric and floor rooted".into());
        }
        let (times, rotations) = read_clip(&gltf, binary)?;
        let animated_bounds = animated_bounds(&groups, pivots, &rotations);
        Ok(Self {
            neutral,
            groups,
            anatomy: anatomy.try_into().map_err(|_| "invalid anatomy count")?,
            pivots,
            duration: DURATION,
            animated_bounds,
            times,
            rotations,
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RootExtras {
    coordinate_system: String,
    source_schema_version: u32,
    source_mesh_sha256: String,
    rig_schema_version: u32,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct HingeExtras {
    authored_for_runtime: bool,
    pivot_basis: String,
    source_primitive_indices: Vec<usize>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PartExtras {
    source_primitive: usize,
}

fn extras<T: serde::de::DeserializeOwned>(value: &gltf::json::Extras) -> Result<T, String> {
    serde_json::from_str(
        value
            .as_ref()
            .ok_or("missing Pidgeotto source metadata")?
            .get(),
    )
    .map_err(|e| format!("invalid Pidgeotto source metadata: {e}"))
}

// Unlisted extensions can otherwise disappear in gltf builds without their
// feature flags. Reject them from the source JSON before semantic decoding.
fn reject_extensions(bytes: &[u8]) -> Result<(), String> {
    let glb = gltf::binary::Glb::from_slice(bytes).map_err(|e| e.to_string())?;
    let binary = glb.bin.as_deref().ok_or("missing Pidgeotto binary chunk")?;
    if glb.json.len() % 4 != 0
        || binary.len() % 4 != 0
        || 28 + glb.json.len() + binary.len() != bytes.len()
    {
        return Err("Pidgeotto requires exactly two aligned GLB chunks".into());
    }
    let json: serde_json::Value = serde_json::from_slice(&glb.json).map_err(|e| e.to_string())?;
    if json["asset"]["version"] != "2.0"
        || json["asset"].get("minVersion").is_some_and(|v| v != "2.0")
    {
        return Err("unsupported Pidgeotto glTF version".into());
    }
    fn visit(value: &serde_json::Value) -> bool {
        match value {
            serde_json::Value::Object(object) => object
                .iter()
                .any(|(key, value)| key == "extensions" || (key != "extras" && visit(value))),
            serde_json::Value::Array(values) => values.iter().any(visit),
            _ => false,
        }
    }
    if visit(&json) {
        return Err("Pidgeotto GLB extensions are unsupported".into());
    }
    Ok(())
}

fn bind_translation(node: &gltf::Node<'_>) -> Result<Vec3, String> {
    let gltf::scene::Transform::Decomposed {
        translation,
        rotation,
        scale,
    } = node.transform()
    else {
        return Err("Pidgeotto matrix transforms are unsupported".into());
    };
    if translation.iter().any(|v| !v.is_finite())
        || rotation != [0.0, 0.0, 0.0, 1.0]
        || scale != [1.0; 3]
    {
        return Err("Pidgeotto requires finite translation-only bind transforms".into());
    }
    Ok(Vec3::from_array(translation))
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
            return Err("Pidgeotto requires aligned, tightly packed embedded buffer views".into());
        }
    }
    for accessor in gltf.accessors() {
        let view = accessor
            .view()
            .ok_or("Pidgeotto accessor requires a buffer view")?;
        if accessor.sparse().is_some()
            || accessor.normalized()
            || accessor.offset() % 4 != 0
            || accessor.count() == 0
            || accessor.count() > MAX_INDICES
            || !matches!(accessor.data_type(), DataType::F32 | DataType::U32)
            || !matches!(
                accessor.dimensions(),
                Dimensions::Scalar | Dimensions::Vec3 | Dimensions::Vec4
            )
            || accessor
                .count()
                .checked_mul(accessor.size())
                .and_then(|n| n.checked_add(accessor.offset()))
                .is_none_or(|end| end > view.length())
        {
            return Err("Pidgeotto accessor encoding or bounds are unsupported".into());
        }
    }
    Ok(())
}

fn validate_material(material: &gltf::Material<'_>) -> Result<(), String> {
    let pbr = material.pbr_metallic_roughness();
    if material.index().is_none()
        || pbr
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
        || material.alpha_cutoff().is_some()
        || material.alpha_mode() == gltf::material::AlphaMode::Mask
        || (material.alpha_mode() == gltf::material::AlphaMode::Opaque
            && pbr.base_color_factor()[3] != 1.0)
    {
        return Err(
            "Pidgeotto supports untextured linear RGBA, metallic=0, roughness=1 materials only"
                .into(),
        );
    }
    Ok(())
}

fn validate_primitive(primitive: &gltf::Primitive<'_>) -> Result<(), String> {
    if primitive.mode() != gltf::mesh::Mode::Triangles
        || primitive.morph_targets().next().is_some()
        || primitive.attributes().len() != 2
    {
        return Err("Pidgeotto requires rigid indexed POSITION/NORMAL triangles".into());
    }
    let positions = primitive
        .get(&Semantic::Positions)
        .ok_or("missing Pidgeotto positions")?;
    let normals = primitive
        .get(&Semantic::Normals)
        .ok_or("missing Pidgeotto normals")?;
    let indices = primitive
        .indices()
        .ok_or("missing Pidgeotto triangle indices")?;
    if [&positions, &normals]
        .iter()
        .any(|a| a.data_type() != DataType::F32 || a.dimensions() != Dimensions::Vec3)
        || positions.count() < 3
        || positions.count() != normals.count()
        || positions.count() > MAX_VERTICES
        || indices.data_type() != DataType::U32
        || indices.dimensions() != Dimensions::Scalar
        || indices.count() % 3 != 0
        || indices.count() > MAX_INDICES
    {
        return Err("invalid Pidgeotto geometry accessor schema".into());
    }
    validate_material(&primitive.material())
}

fn read_primitive(
    primitive: &gltf::Primitive<'_>,
    binary: &[u8],
) -> Result<SurfaceMeshData, String> {
    let reader = primitive.reader(|_| Some(binary));
    let positions: Vec<_> = reader
        .read_positions()
        .ok_or("missing Pidgeotto positions")?
        .collect();
    let normals: Vec<_> = reader
        .read_normals()
        .ok_or("missing Pidgeotto normals")?
        .collect();
    let indices: Vec<_> = reader
        .read_indices()
        .ok_or("missing Pidgeotto indices")?
        .into_u32()
        .collect();
    if positions
        .iter()
        .flatten()
        .chain(normals.iter().flatten())
        .any(|v| !v.is_finite())
        || indices.iter().any(|&i| i as usize >= positions.len())
        || normals.iter().any(|&n| {
            let n = Vec3::from_array(n).length_squared();
            !n.is_finite() || n < 0.000_001
        })
    {
        return Err("invalid Pidgeotto vertices, indices or zero normals".into());
    }
    Ok(SurfaceMeshData {
        uvs: vec![[0.0; 2]; positions.len()],
        colors: vec![
            primitive
                .material()
                .pbr_metallic_roughness()
                .base_color_factor();
            positions.len()
        ],
        positions,
        // Same f32 operation as the former Rust JSON loader, exactly once.
        normals: normals
            .into_iter()
            .map(|n| Vec3::from_array(n).normalize().to_array())
            .collect(),
        indices,
        cutaway_ranges: Vec::new(),
    })
}

fn append_mesh(target: &mut SurfaceMeshData, source: &SurfaceMeshData) {
    let base = target.positions.len() as u32;
    target.positions.extend_from_slice(&source.positions);
    target.normals.extend_from_slice(&source.normals);
    target.uvs.extend_from_slice(&source.uvs);
    target.colors.extend_from_slice(&source.colors);
    target
        .indices
        .extend(source.indices.iter().map(|&i| base + i));
}

/// Bound continuous SLERP motion, including extrema between authored samples.
/// Unwrap each shortest-arc Z interval before taking the full angular envelope.
fn animated_bounds(
    groups: &[RigGroup; GROUP_COUNT],
    pivots: [Vec3; 2],
    rotations: &[Vec<Quat>; 2],
) -> (Vec3, Vec3) {
    use std::f64::consts::{FRAC_PI_2, PI, TAU};
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    let mut include = |point: Vec3| {
        min = min.min(point);
        max = max.max(point);
    };
    for &point in &groups[0].mesh.positions {
        include(Vec3::from_array(point));
    }
    for wing in 0..2 {
        let (mut low, mut high, mut current) = (0.0_f64, 0.0_f64, 0.0_f64);
        for q in &rotations[wing] {
            let angle = 2.0 * (q.z as f64).atan2(q.w as f64);
            current += (angle - current + PI).rem_euclid(TAU) - PI;
            low = low.min(current);
            high = high.max(current);
        }
        let pivot = pivots[wing];
        for point in &groups[wing + 1].mesh.positions {
            let x = point[0] as f64 - pivot.x as f64;
            let y = point[1] as f64 - pivot.y as f64;
            let at = |angle: f64| {
                let (sin, cos) = angle.sin_cos();
                Vec3::new(
                    (pivot.x as f64 + x * cos - y * sin) as f32,
                    (pivot.y as f64 + x * sin + y * cos) as f32,
                    point[2],
                )
            };
            if high - low >= TAU {
                let radius = x.hypot(y) as f32;
                include(Vec3::new(pivot.x - radius, pivot.y - radius, point[2]));
                include(Vec3::new(pivot.x + radius, pivot.y + radius, point[2]));
                continue;
            }
            include(at(low));
            include(at(high));
            for base in [-y.atan2(x), FRAC_PI_2 - y.atan2(x)] {
                let first = ((low - base) / PI).ceil() as i32;
                let last = ((high - base) / PI).floor() as i32;
                for turn in first..=last {
                    include(at(base + turn as f64 * PI));
                }
            }
        }
    }
    // The renderer applies f32 quaternion arithmetic. Expand the f64 analytic
    // envelope slightly to include its rounding, including unchanged Z values.
    let guard = Vec3::splat(0.000_01);
    (min - guard, max + guard)
}

fn read_clip(gltf: &gltf::Gltf, binary: &[u8]) -> Result<(Vec<f32>, [Vec<Quat>; 2]), String> {
    if gltf.animations().len() != 1 {
        return Err("Pidgeotto requires one canonical idle wing clip".into());
    }
    let raw = &gltf.as_json().animations[0];
    for channel in &raw.channels {
        if channel.target.node.value() >= NODE_COUNT
            || !matches!(
                channel.target.path,
                gltf::json::validation::Checked::Valid(_)
            )
        {
            return Err("invalid Pidgeotto animation target".into());
        }
    }
    let animation = gltf.animations().next().unwrap();
    if animation.name() != Some(CLIP_NAME)
        || animation.channels().count() != 2
        || animation.samplers().count() != 2
    {
        return Err("Pidgeotto requires its named two-shoulder idle clip".into());
    }
    let mut times = Vec::new();
    let mut rotations = [Vec::new(), Vec::new()];
    for channel in animation.channels() {
        let wing = match channel.target().node().index() {
            1 => 0,
            3 => 1,
            _ => return Err("Pidgeotto animation may target only the shoulder hinges".into()),
        };
        let sampler = channel.sampler();
        let input = sampler.input();
        let output = sampler.output();
        if channel.target().property() != Property::Rotation
            || !rotations[wing].is_empty()
            || sampler.interpolation() != Interpolation::Linear
            || input.data_type() != DataType::F32
            || input.dimensions() != Dimensions::Scalar
            || output.data_type() != DataType::F32
            || output.dimensions() != Dimensions::Vec4
            || input.count() < 5
            || input.count() > MAX_KEYS
            || input.count() != output.count()
        {
            return Err(
                "Pidgeotto supports two unique LINEAR f32 shoulder-rotation tracks only".into(),
            );
        }
        let reader = channel.reader(|_| Some(binary));
        let input: Vec<_> = reader
            .read_inputs()
            .ok_or("missing Pidgeotto clip times")?
            .collect();
        if input.first() != Some(&0.0)
            || input.last() != Some(&DURATION)
            || input.iter().any(|t| !t.is_finite())
            || input.windows(2).any(|t| t[0] >= t[1])
            || (!times.is_empty() && times != input)
        {
            return Err(
                "Pidgeotto idle tracks must share increasing closed 0.8-second times".into(),
            );
        }
        let gltf::animation::util::ReadOutputs::Rotations(values) = reader
            .read_outputs()
            .ok_or("missing Pidgeotto clip rotations")?
        else {
            return Err("Pidgeotto idle clip requires rotations".into());
        };
        let values: Vec<_> = values.into_f32().collect();
        if values.len() != input.len()
            || values.first() != Some(&[0.0, 0.0, 0.0, 1.0])
            || values.last() != Some(&[0.0, 0.0, 0.0, 1.0])
            || values.iter().any(|q| {
                q.iter().any(|v| !v.is_finite())
                    || q[0] != 0.0
                    || q[1] != 0.0
                    || (Quat::from_array(*q).length_squared() - 1.0).abs() > 0.00001
            })
        {
            return Err(
                "Pidgeotto clip requires unit Z-axis rotations with an exact identity seam".into(),
            );
        }
        times = input;
        rotations[wing] = values
            .into_iter()
            .map(|q| Quat::from_array(q).normalize())
            .collect();
    }
    if rotations.iter().any(Vec::is_empty) {
        return Err("missing Pidgeotto shoulder track".into());
    }
    Ok((times, rotations))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use sha2::{Digest, Sha256};

    const ASSET: &[u8] = include_bytes!("../models/battle_species/pidgeotto.glb");

    fn digest(mesh: &SurfaceMeshData) -> String {
        let mut hash = Sha256::new();
        for value in mesh
            .positions
            .iter()
            .flatten()
            .chain(mesh.normals.iter().flatten())
            .chain(mesh.uvs.iter().flatten())
            .chain(mesh.colors.iter().flatten())
        {
            hash.update(value.to_bits().to_le_bytes());
        }
        for &index in &mesh.indices {
            hash.update(index.to_le_bytes());
        }
        format!("{:x}", hash.finalize())
    }

    #[test]
    fn neutral_surface_matches_original_rust_json_loader_digest() {
        let rig = rig();
        assert_eq!(rig.neutral.positions.len(), 1274);
        assert_eq!(rig.neutral.indices.len(), 5388);
        // Frozen from the former JSON loader, after its Vec3::normalize call.
        // No duplicate geometry fixture is needed once the JSON is removed.
        assert_eq!(
            digest(&rig.neutral),
            "00507708cb46762547b119624694eab05102ad27ec2f21409add010e24505b4c"
        );
        assert_eq!(rig.anatomy.len(), PART_COUNT);
        assert_eq!(rig.anatomy[0].name, "Bird body");
        assert_eq!(rig.anatomy[33].name, "Head crest.002");
        assert_eq!(rig.groups[0].parts.len(), 24);
        assert_eq!(rig.groups[1].parts, [14, 15, 16, 17, 18]);
        assert_eq!(rig.groups[2].parts, [23, 24, 25, 26, 27]);
        assert_eq!(
            rig.groups.each_ref().map(|g| g.name),
            ["body", "wing_l", "wing_r"]
        );
        let mut indices_by_group = [0; GROUP_COUNT];
        for anatomy in &rig.anatomy {
            let group = &rig.groups[anatomy.group].mesh;
            for (neutral, grouped) in rig.neutral.positions[anatomy.vertices.clone()]
                .iter()
                .zip(&group.positions[anatomy.group_vertices.clone()])
            {
                assert_eq!(neutral.map(f32::to_bits), grouped.map(f32::to_bits));
            }
            assert_eq!(
                rig.neutral.normals[anatomy.vertices.clone()],
                group.normals[anatomy.group_vertices.clone()]
            );
            assert_eq!(
                rig.neutral.colors[anatomy.vertices.clone()],
                group.colors[anatomy.group_vertices.clone()]
            );
            assert_eq!(
                rig.neutral.uvs[anatomy.vertices.clone()],
                group.uvs[anatomy.group_vertices.clone()]
            );
            let source_indices: Vec<_> = rig
                .neutral
                .indices
                .iter()
                .copied()
                .filter(|&i| anatomy.vertices.contains(&(i as usize)))
                .collect();
            let begin = indices_by_group[anatomy.group];
            for (&source, &grouped) in source_indices
                .iter()
                .zip(&group.indices[begin..begin + source_indices.len()])
            {
                assert_eq!(
                    source as usize - anatomy.vertices.start,
                    grouped as usize - anatomy.group_vertices.start
                );
            }
            indices_by_group[anatomy.group] += source_indices.len();
        }
        for (group, count) in rig.groups.iter().zip(indices_by_group) {
            assert_eq!(group.mesh.indices.len(), count);
        }
    }

    #[test]
    fn articulation_moves_only_wings_and_keeps_shoulders_anchored() {
        let rig = rig();
        let pose = rig.sample(0.103).unwrap(); // Between 65-key samples.
        assert_eq!(pose[0], Transform::IDENTITY);
        assert!(pose[1].rotation.z > 0.0 && pose[2].rotation.z < 0.0);
        let moved: Vec<_> = rig
            .anatomy
            .iter()
            .filter(|part| {
                rig.neutral.positions[part.vertices.clone()]
                    .iter()
                    .any(|&point| {
                        pose[part.group]
                            .transform_point(Vec3::from_array(point))
                            .distance(Vec3::from_array(point))
                            > 0.001
                    })
            })
            .map(|part| part.name.as_str())
            .collect();
        assert_eq!(moved.len(), 10);
        assert!(
            moved
                .iter()
                .all(|name| name.starts_with("Layered wing") || name.starts_with("Flight feather"))
        );
        for time in [0.037, 0.2, 0.337, 0.6, 0.799] {
            let pose = rig.sample(time).unwrap();
            for wing in 0..2 {
                assert!(
                    pose[wing + 1]
                        .transform_point(rig.pivots[wing])
                        .distance(rig.pivots[wing])
                        < 0.000_000_1
                );
                assert_eq!(pose[wing + 1].scale, Vec3::ONE);
                assert!((pose[wing + 1].rotation.length_squared() - 1.0).abs() < 0.000_001);
            }
        }
    }

    #[test]
    fn idle_loop_is_closed_smooth_and_independently_sampled() {
        let rig = rig();
        assert_eq!(rig.duration, 0.8);
        assert_eq!(rig.times.len(), 65);
        for seam in [0.0, rig.duration, 2.0 * rig.duration, -rig.duration] {
            assert_eq!(
                rig.sample(seam).unwrap(),
                [Transform::IDENTITY; GROUP_COUNT]
            );
        }
        let first = rig.sample(0.137).unwrap();
        let other = rig.sample(0.537).unwrap();
        assert_ne!(first[1], other[1]);
        assert_eq!(rig.sample(0.137).unwrap(), first);
        assert!(std::ptr::eq(super::rig(), rig));
        assert!(
            (rig.sample(0.137 + rig.duration).unwrap()[1].rotation - first[1].rotation).length()
                < 0.000_001
        );
        // The sampled sine has matching velocity on each side of the wrap;
        // quarter-cycle keys approach a turning point instead of a demo cusp.
        let epsilon = 0.0001;
        let before = rig.sample(-epsilon).unwrap()[1].rotation.z;
        let after = rig.sample(epsilon).unwrap()[1].rotation.z;
        assert!((before + after).abs() < 0.000_001);
        let peak = rig.sample(0.2).unwrap()[1].rotation.z;
        let before_peak = rig.sample(0.2 - epsilon).unwrap()[1].rotation.z;
        assert!((peak - before_peak).abs() / epsilon < 0.1);
        assert_eq!(rig.rotations[0][0].to_array(), [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(rig.rotations[0][64].to_array(), [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn sampling_rejects_nonfinite_time_without_poisoning_other_instances() {
        let rig = rig();
        let before = rig.sample(0.123).unwrap();
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(rig.sample(invalid).is_err());
        }
        assert_eq!(rig.sample(0.123).unwrap(), before);
        assert!(
            rig.sample(f32::MAX)
                .unwrap()
                .iter()
                .all(|t| t.rotation.is_finite() && t.translation.is_finite())
        );
    }

    #[test]
    fn animated_bounds_include_every_sampled_pose_and_between_key_extrema() {
        let rig = rig();
        let (min, max) = rig.animated_bounds;
        assert!(min.x < -0.84 && max.x > 0.84);
        assert!(max.y > 1.08 && max.y < 1.09);
        assert!(min.x > -0.86 && max.x < 0.86);
        for frame in 0..=1024 {
            let pose = rig.sample(rig.duration * frame as f32 / 1024.0).unwrap();
            for (group, transform) in rig.groups.iter().zip(pose) {
                for &point in &group.mesh.positions {
                    let point = transform.transform_point(Vec3::from_array(point));
                    assert!(
                        point.cmpge(min).all() && point.cmple(max).all(),
                        "pose {frame}: {point:?} outside {min:?}..{max:?}"
                    );
                }
            }
        }
        // The analytic maximum is not only a union of keyframe boxes: rotate a
        // diagnostic wing tip across its exact X extremum between sparse keys.
        let mut groups = ["body", "wing_l", "wing_r"].map(|name| RigGroup {
            name,
            mesh: SurfaceMeshData::default(),
            parts: Vec::new(),
        });
        groups[0].mesh.positions.push([0.0; 3]);
        groups[1].mesh.positions.push([1.0, 0.2, 0.3]);
        groups[2].mesh.positions.push([-1.0, 0.2, 0.3]);
        let rotations = [
            vec![Quat::IDENTITY, Quat::from_rotation_z(-0.4), Quat::IDENTITY],
            vec![Quat::IDENTITY, Quat::from_rotation_z(0.4), Quat::IDENTITY],
        ];
        let (min, max) = animated_bounds(&groups, [Vec3::ZERO; 2], &rotations);
        let radius = 1.04_f32.sqrt();
        assert!(max.x >= radius && min.x <= -radius);
        assert!(max.x < radius + 0.000_02 && min.x > -radius - 0.000_02);
    }

    struct Fixture {
        json: Value,
        binary: Vec<u8>,
    }
    impl Fixture {
        fn new() -> Self {
            let glb = gltf::binary::Glb::from_slice(ASSET).unwrap();
            Self {
                json: serde_json::from_slice(&glb.json).unwrap(),
                binary: glb.bin.unwrap().into_owned(),
            }
        }
        fn bytes(&self) -> Vec<u8> {
            let mut json = serde_json::to_vec(&self.json).unwrap();
            while json.len() % 4 != 0 {
                json.push(b' ');
            }
            let mut binary = self.binary.clone();
            while binary.len() % 4 != 0 {
                binary.push(0);
            }
            let mut bytes = Vec::new();
            for word in [
                0x46546c67_u32,
                2,
                (28 + json.len() + binary.len()) as u32,
                json.len() as u32,
                0x4e4f534a,
            ] {
                bytes.extend_from_slice(&word.to_le_bytes());
            }
            bytes.extend_from_slice(&json);
            bytes.extend_from_slice(&(binary.len() as u32).to_le_bytes());
            bytes.extend_from_slice(&0x004e4942_u32.to_le_bytes());
            bytes.extend_from_slice(&binary);
            bytes
        }
        fn byte_offset(&self, accessor: usize) -> usize {
            let accessor = &self.json["accessors"][accessor];
            let view = &self.json["bufferViews"][accessor["bufferView"].as_u64().unwrap() as usize];
            view["byteOffset"].as_u64().unwrap_or(0) as usize
                + accessor["byteOffset"].as_u64().unwrap_or(0) as usize
        }
        fn float(&mut self, accessor: usize, scalar: usize, value: f32) {
            let offset = self.byte_offset(accessor) + scalar * 4;
            self.binary[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
    }

    #[test]
    fn rejects_unsupported_hierarchy_materials_animation_and_buffer_data() {
        let changes: &[(&str, fn(&mut Fixture))] = &[
            ("scene", |f| f.json["scene"] = json!(99)),
            ("glTF version", |f| {
                f.json["asset"]["version"] = json!("1.0")
            }),
            ("extreme position", |f| f.float(0, 0, f32::MAX)),
            ("root", |f| {
                f.json["nodes"][0]["translation"] = json!([0, 1, 0])
            }),
            ("matrix", |f| {
                f.json["nodes"][0]["matrix"] =
                    json!([1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1])
            }),
            ("scale", |f| f.json["nodes"][5]["scale"] = json!([2, 2, 2])),
            ("missing source identity", |f| {
                f.json["nodes"][5]["extras"] = json!({})
            }),
            ("reordered anatomy", |f| {
                f.json["nodes"][5]["extras"]["sourcePrimitive"] = json!(1)
            }),
            ("reused mesh", |f| f.json["nodes"][6]["mesh"] = json!(0)),
            ("shared child", |f| {
                f.json["nodes"][2]["children"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!(5))
            }),
            ("cycle", |f| f.json["nodes"][5]["children"] = json!([0])),
            ("hinge metadata", |f| {
                f.json["nodes"][1]["extras"]["sourcePrimitiveIndices"] = json!([14, 15, 16, 17, 19])
            }),
            ("wrong pivot", |f| {
                f.json["nodes"][1]["translation"][0] = json!(-0.2)
            }),
            ("metallic", |f| {
                f.json["materials"][0]["pbrMetallicRoughness"]["metallicFactor"] = json!(0.5)
            }),
            ("masked", |f| {
                f.json["materials"][0]["alphaMode"] = json!("MASK")
            }),
            ("double sided", |f| {
                f.json["materials"][0]["doubleSided"] = json!(true)
            }),
            ("unlisted extension", |f| {
                f.json["materials"][0]["extensions"] = json!({"KHR_materials_unlit": {}})
            }),
            ("emissive", |f| {
                f.json["materials"][0]["emissiveFactor"] = json!([1, 0, 0])
            }),
            ("missing material", |f| {
                f.json["meshes"][0]["primitives"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("material");
            }),
            ("morph", |f| {
                f.json["meshes"][0]["primitives"][0]["targets"] = json!([{"POSITION": 0}])
            }),
            ("extra attribute", |f| {
                f.json["meshes"][0]["primitives"][0]["attributes"]["TEXCOORD_0"] = json!(0)
            }),
            ("missing clip", |f| f.json["animations"] = json!([])),
            ("wrong clip", |f| {
                f.json["animations"][0]["name"] = json!("pidgeotto.attack")
            }),
            ("out of range target", |f| {
                f.json["animations"][0]["channels"][0]["target"]["node"] = json!(999)
            }),
            ("body motion", |f| {
                f.json["animations"][0]["channels"][0]["target"]["node"] = json!(0)
            }),
            ("duplicate shoulder", |f| {
                f.json["animations"][0]["channels"][1]["target"]["node"] = json!(1)
            }),
            ("unknown path", |f| {
                f.json["animations"][0]["channels"][0]["target"]["path"] = json!("unknown")
            }),
            ("scale motion", |f| {
                f.json["animations"][0]["channels"][0]["target"]["path"] = json!("scale")
            }),
            ("STEP", |f| {
                f.json["animations"][0]["samplers"][0]["interpolation"] = json!("STEP")
            }),
            ("CUBICSPLINE", |f| {
                f.json["animations"][0]["samplers"][0]["interpolation"] = json!("CUBICSPLINE")
            }),
            ("external buffer", |f| {
                f.json["buffers"][0]["uri"] = json!("untrusted.bin")
            }),
            ("buffer bounds", |f| {
                f.json["bufferViews"][0]["byteOffset"] = json!(1_000_000)
            }),
            ("stride", |f| {
                f.json["bufferViews"][0]["byteStride"] = json!(16)
            }),
            ("normalized", |f| {
                f.json["accessors"][0]["normalized"] = json!(true)
            }),
            ("accessor bounds", |f| {
                f.json["accessors"][0]["count"] = json!(1_000_000)
            }),
            ("NaN position", |f| f.float(0, 0, f32::NAN)),
            ("zero normal", |f| {
                for i in 0..3 {
                    f.float(1, i, 0.0);
                }
            }),
            ("invalid index", |f| {
                let offset = f.byte_offset(2);
                f.binary[offset..offset + 4].copy_from_slice(&u32::MAX.to_le_bytes());
            }),
            ("bad time", |f| {
                let a = f.json["animations"][0]["samplers"][0]["input"]
                    .as_u64()
                    .unwrap() as usize;
                f.float(a, 1, 0.0);
            }),
            ("open seam", |f| {
                let a = f.json["animations"][0]["samplers"][0]["output"]
                    .as_u64()
                    .unwrap() as usize;
                f.float(a, 64 * 4 + 2, 0.1);
            }),
            ("nonunit rotation", |f| {
                let a = f.json["animations"][0]["samplers"][0]["output"]
                    .as_u64()
                    .unwrap() as usize;
                f.float(a, 4 + 3, 2.0);
            }),
        ];
        for &(name, change) in changes {
            let mut fixture = Fixture::new();
            change(&mut fixture);
            let bytes = fixture.bytes();
            let result = std::panic::catch_unwind(|| PidgeottoRig::parse(&bytes));
            assert!(result.is_ok(), "parser panicked for {name}");
            assert!(result.unwrap().is_err(), "accepted unsupported {name}");
        }
        for bytes in [&ASSET[..10], &ASSET[..ASSET.len() - 4], b"{}"] {
            assert!(PidgeottoRig::parse(bytes).is_err());
        }
        for length in [0_u32, 11, (ASSET.len() - 1) as u32] {
            let mut bytes = ASSET.to_vec();
            bytes[8..12].copy_from_slice(&length.to_le_bytes());
            assert!(
                std::panic::catch_unwind(|| PidgeottoRig::parse(&bytes))
                    .unwrap()
                    .is_err()
            );
        }
        let mut trailing = ASSET.to_vec();
        trailing.extend_from_slice(&[0; 8]);
        let length = trailing.len() as u32;
        trailing[8..12].copy_from_slice(&length.to_le_bytes());
        assert!(PidgeottoRig::parse(&trailing).is_err());
    }
}
