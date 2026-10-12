//! External weighted CC0 people. Decoded once; only local joint poses change.
//! These resources contain presentation data, never game inputs or movement.
use crate::mesh::SurfaceMeshData;
use base64::Engine as _;
use bevy::prelude::*;
use gltf::animation::{Interpolation, Property};
use std::{
    collections::BTreeMap,
    sync::{Arc, OnceLock},
};

pub const PEOPLE: [&str; 6] = [
    "skater-male",
    "skater-female",
    "criminal-male",
    "cyborg-female",
    "human-male",
    "human-female",
];
const REQUIRED_PEOPLE: [&str; 2] = ["human-male", "human-female"];
const MAX_BYTES: usize = 4 * 1024 * 1024;
static RIGS: OnceLock<BTreeMap<String, Arc<PersonRig>>> = OnceLock::new();

/// Install before the first actor lookup. GLBs and textures remain external.
/// JSON: {"version":1,"people":{"skater-male":"BASE64_GLB",…}}.
/// Unsupported/malformed rigs reject the whole bundle before caches freeze.
pub fn npc_texture_names() -> Vec<&'static str> { crate::new_bark_actors::npc_texture_names() }
pub fn install_open_people_bundle(json: &str) -> Result<(), String> {
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Bundle {
        version: u32,
        people: BTreeMap<String, String>,
        #[serde(default)]
        textures: BTreeMap<String, String>,
    }
    if RIGS.get().is_some() || crate::npc_textures::frozen() {
        return Err("people sources frozen; restart to refresh".into());
    }
    if json.len() > 64 * 1024 * 1024 {
        return Err("people bundle exceeds 64 MiB".into());
    }
    let b: Bundle = serde_json::from_str(json).map_err(|e| e.to_string())?;
    if b.version != 1 || b.people.len() > PEOPLE.len() || b.textures.len() > 75 {
        return Err("unsupported people bundle".into());
    }
    let mut rigs = BTreeMap::new();
    for (name, encoded) in b.people {
        if !PEOPLE.contains(&name.as_str()) || encoded.len() > MAX_BYTES * 4 / 3 + 4 {
            return Err("unsupported person or size".into());
        }
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(|e| e.to_string())?;
        rigs.insert(name, Arc::new(PersonRig::parse(&bytes)?));
    }
    if REQUIRED_PEOPLE.iter().any(|name| !rigs.contains_key(*name)) {
        return Err("required character rigs are missing from people bundle".into());
    }
    let names = npc_texture_names();
    let mut textures = BTreeMap::new();
    for (name, encoded) in b.textures {
        if !names.contains(&name.as_str()) || encoded.len() > MAX_BYTES * 4 / 3 + 4 {
            return Err("unsupported NPC texture name or size".into());
        }
        let bytes = base64::engine::general_purpose::STANDARD.decode(encoded).map_err(|e| e.to_string())?;
        textures.insert(name, crate::npc_textures::decode(&bytes)?);
    }
    crate::npc_textures::require_complete(&textures)?;
    // Both payloads have been validated before either cache is published.
    RIGS.set(rigs).map_err(|_| "people sources frozen; restart to refresh".to_string())?;
    crate::npc_textures::install(textures)
}

/// Native applications and review tools share the same default artwork root.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn model_root() -> std::path::PathBuf {
    std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/open-models"))
}

pub(crate) fn get(name: &str) -> Option<&'static Arc<PersonRig>> {
    RIGS.get_or_init(|| {
        let mut rigs = BTreeMap::new();
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::io::Read;
            let root = model_root();
            for name in PEOPLE {
                let path = root.join(format!("people/kenney/{name}.glb"));
                let file = match std::fs::File::open(&path) {
                    Ok(file) => file,
                    Err(e) if !REQUIRED_PEOPLE.contains(&name) && e.kind() == std::io::ErrorKind::NotFound => continue,
                    Err(e) => panic!("required character asset {}: {e}; set CRYSTAL_OPEN_MODEL_ROOT to a complete artwork directory", path.display()),
                };
                let mut bytes = Vec::new();
                file.take(MAX_BYTES as u64 + 1).read_to_end(&mut bytes)
                    .unwrap_or_else(|e| panic!("read required character asset {}: {e}", path.display()));
                let rig = PersonRig::parse(&bytes)
                    .unwrap_or_else(|e| panic!("invalid required character asset {}: {e}", path.display()));
                rigs.insert(name.to_owned(), Arc::new(rig));
            }
        }
        rigs
    })
    .get(name)
}

pub(crate) struct PersonRig {
    pub nodes: Vec<Transform>,
    pub parents: Vec<Option<usize>>,
    pub skin_nodes: Vec<usize>,
    pub head_joint: usize,
    pub hips_joint: usize,
    pub chest_joint: usize,
    /// Optional hand subtrees in the source skin palette, including fingers.
    pub hand_joints: [Vec<usize>; 2],
    /// Optional articulated finger nodes; excludes wrist and end markers.
    pub finger_nodes: Vec<usize>,
    /// Named thumb bases, when present, for fitted prop grips.
    pub thumb_nodes: [Option<usize>; 2],
    /// Left/right thigh and shin indices in the same skin palette as the body.
    pub leg_joints: [[usize; 2]; 2],
    pub inverse_binds: Vec<Mat4>,
    pub mesh_node: usize,
    pub mesh: SurfaceMeshData,
    pub ids: Vec<[u16; 4]>,
    pub weights: Vec<[f32; 4]>,
    pub rgba: Vec<u8>,
    pub texture_size: (u32, u32),
    pub height: f32,
    pub floor: f32,
    pub bounds: (Vec3, Vec3),
    clips: BTreeMap<String, Clip>,
    order: Vec<usize>,
}
struct Clip {
    duration: f32,
    tracks: Vec<Track>,
}
struct Track {
    step: bool,
    node: usize,
    property: Property,
    times: Vec<f32>,
    values: Vec<Vec4>,
}

impl PersonRig {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_BYTES || !bytes.starts_with(b"glTF") {
            return Err("person needs bounded GLB".into());
        }
        let g = gltf::Gltf::from_slice(bytes).map_err(|e| e.to_string())?;
        let bin = g.blob.as_deref().ok_or("missing embedded person data")?;
        if g.buffers().len() != 1
            || g.skins().len() != 1
            || g.meshes().len() != 1
            || g.nodes().len() > 128
            || g.nodes().len() < 18
            || g.animations().len() != 3
            || g.images().len() != 1
            || g.materials().len() != 1
            || g.accessors().len() > 600
            || g.views().len() > 700
            || g.scenes().len() != 1
            || g.extensions_used().next().is_some()
            || g.cameras().len() != 0
        {
            return Err("unsupported person schema".into());
        }
        if !matches!(
            g.buffers().next().unwrap().source(),
            gltf::buffer::Source::Bin
        ) {
            return Err("external buffers unsupported".into());
        }
        if g.buffers().next().unwrap().length() > bin.len() {
            return Err("embedded person buffer is truncated".into());
        }
        // Validate slices before glTF utility readers allocate or index them.
        for v in g.views() {
            if v.buffer().index() != 0
                || v.offset()
                    .checked_add(v.length())
                    .is_none_or(|n| n > bin.len())
            {
                return Err("person buffer out of bounds".into());
            }
        }
        for a in g.accessors() {
            let v = a.view().ok_or("missing accessor view")?;
            let stride = v.stride().unwrap_or(a.size());
            if a.sparse().is_some()
                || a.count() == 0
                || a.count() > 120_000
                || stride < a.size()
                || (a.count() - 1)
                    .checked_mul(stride)
                    .and_then(|n| n.checked_add(a.size()))
                    .and_then(|n| n.checked_add(a.offset()))
                    .is_none_or(|n| n > v.length())
            {
                return Err("person accessor out of bounds".into());
            }
        }
        let nodes: Vec<_> = g
            .nodes()
            .map(|n| {
                let (t, r, s) = n.transform().decomposed();
                Transform {
                    translation: Vec3::from_array(t),
                    rotation: Quat::from_array(r),
                    scale: Vec3::from_array(s),
                }
            })
            .collect();
        if nodes.iter().any(|n| {
            !n.translation.is_finite()
                || !n.rotation.is_finite()
                || !n.scale.is_finite()
                || n.scale.min_element() <= 0.0
                || (n.rotation.length_squared() - 1.0).abs() > 0.01
        }) {
            return Err("invalid person transforms".into());
        }
        let mut parents = vec![None; nodes.len()];
        for n in g.nodes() {
            for c in n.children() {
                if parents[c.index()].replace(n.index()).is_some() {
                    return Err("person node has multiple parents".into());
                }
            }
        }
        let mut order = Vec::new();
        while order.len() < nodes.len() {
            let before = order.len();
            for i in 0..nodes.len() {
                if !order.contains(&i) && parents[i].is_none_or(|p| order.contains(&p)) {
                    order.push(i);
                }
            }
            if order.len() == before {
                return Err("person node cycle".into());
            }
        }
        let skin = g.skins().next().unwrap();
        let skin_nodes: Vec<_> = skin.joints().map(|j| j.index()).collect();
        if skin_nodes.len() < 16
            || skin_nodes.len() > 100
            || skin_nodes
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len()
                != skin_nodes.len()
        {
            return Err("invalid person skin joints".into());
        }
        let head_node = g.nodes().find(|n| n.name() == Some("Head"))
            .ok_or("missing named Head attachment joint")?.index();
        let head_joint = skin_nodes.iter().position(|&i| i == head_node)
            .ok_or("Head attachment is not in the skin")?;
        let hips_node = g.nodes().find(|n| n.name() == Some("Hips"))
            .ok_or("missing named Hips attachment joint")?.index();
        let hips_joint = skin_nodes.iter().position(|&i| i == hips_node)
            .ok_or("Hips attachment is not in the skin")?;
        let skin_joint = |name| {
            let node = g.nodes().find(|n| n.name() == Some(name))
                .ok_or_else(|| format!("missing named {name} garment joint"))?.index();
            skin_nodes.iter().position(|&i| i == node)
                .ok_or_else(|| format!("{name} garment joint is not in the skin"))
        };
        let leg_joints = [
            [skin_joint("LeftUpLeg")?, skin_joint("LeftLeg")?],
            [skin_joint("RightUpLeg")?, skin_joint("RightLeg")?],
        ];
        let chest_joint = skin_joint("Chest")?;
        let hand_joints = ["LeftHand", "RightHand"].map(|name| {
            let Some(root) = g.nodes().find(|n|n.name()==Some(name)).map(|n|n.index()) else {return Vec::new();};
            skin_nodes.iter().enumerate().filter_map(|(joint,&node)| {
                let mut current=Some(node);
                while let Some(i)=current {
                    if i==root {return Some(joint);}
                    current=parents[i];
                }
                None
            }).collect()
        });
        let finger_nodes = g.nodes().filter(|n| {
            let name = n.name().unwrap_or("");
            (name.contains("HandIndex") || name.contains("HandThumb")) && !name.ends_with("_end")
        }).map(|n| n.index()).collect();
        let thumb_nodes = ["LeftHandThumb1", "RightHandThumb1"].map(|name|
            g.nodes().find(|node| node.name() == Some(name)).map(|node| node.index()));
        let inverse_binds: Vec<_> = skin
            .reader(|_| Some(bin))
            .read_inverse_bind_matrices()
            .ok_or("missing inverse binds")?
            .map(|m| Mat4::from_cols_array_2d(&m))
            .collect();
        if inverse_binds.len() != skin_nodes.len()
            || inverse_binds
                .iter()
                .any(|m| !m.is_finite() || m.determinant().abs() < 1e-8)
        {
            return Err("invalid inverse binds".into());
        }
        let body: Vec<_> = g.nodes().filter(|n| n.mesh().is_some()).collect();
        if body.len() != 1 || body[0].skin().map(|s| s.index()) != Some(0) {
            return Err("person mesh must retain its skin".into());
        }
        let mesh_node = body[0].index();
        let mesh_src = g.meshes().next().unwrap();
        if mesh_src.primitives().len() != 1 || mesh_src.weights().is_some() {
            return Err("unsupported body primitives".into());
        }
        let p = mesh_src.primitives().next().unwrap();
        if p.mode() != gltf::mesh::Mode::Triangles
            || p.morph_targets().len() != 0
            || p.get(&gltf::Semantic::Weights(1)).is_some()
        {
            return Err("unsupported person topology or more than four influences".into());
        }
        let reader = p.reader(|_| Some(bin));
        let positions: Vec<_> = reader
            .read_positions()
            .ok_or("missing positions")?
            .collect();
        let normals: Vec<_> = reader.read_normals().ok_or("missing normals")?.collect();
        let uvs: Vec<_> = reader
            .read_tex_coords(0)
            .ok_or("missing UVs")?
            .into_f32()
            .collect();
        let indices: Vec<_> = reader
            .read_indices()
            .ok_or("missing indices")?
            .into_u32()
            .collect();
        let ids: Vec<_> = reader
            .read_joints(0)
            .ok_or("missing skin indices")?
            .into_u16()
            .collect();
        let weights: Vec<_> = reader
            .read_weights(0)
            .ok_or("missing skin weights")?
            .into_f32()
            .collect();
        let n = positions.len();
        if n == 0
            || n > 20_000
            || indices.len() > 120_000
            || indices.len() % 3 != 0
            || [normals.len(), uvs.len(), ids.len(), weights.len()]
                .iter()
                .any(|&c| c != n)
            || positions
                .iter()
                .flatten()
                .chain(normals.iter().flatten())
                .chain(uvs.iter().flatten())
                .any(|v| !v.is_finite() || v.abs() > 100.0)
            || indices.iter().any(|&i| i as usize >= n)
            || normals
                .iter()
                .any(|n| (Vec3::from_array(*n).length_squared() - 1.0).abs() > 0.01)
            || ids.iter().zip(&weights).any(|(js, ws)| {
                ws.iter()
                    .any(|w| !w.is_finite() || !(0.0..=1.0).contains(w))
                    || (ws.iter().sum::<f32>() - 1.0).abs() > 0.001
                    || js.iter().any(|&j| j as usize >= skin_nodes.len())
            })
        {
            return Err("invalid weighted person geometry".into());
        }
        let mat = p.material().pbr_metallic_roughness();
        let texture = mat.base_color_texture().ok_or("missing body texture")?;
        if texture.tex_coord() != 0 || texture.texture().source().index() != 0 {
            return Err("unsupported texture mapping".into());
        }
        let gltf::image::Source::View { view, mime_type } = g.images().next().unwrap().source()
        else {
            return Err("person texture must be embedded".into());
        };
        if mime_type != "image/png" {
            return Err("person texture must be PNG".into());
        }
        let png = &bin[view.offset()..view.offset() + view.length()];
        let mut decoder =
            image::ImageReader::with_format(std::io::Cursor::new(png), image::ImageFormat::Png);
        let mut limits = image::Limits::default();
        limits.max_image_width = Some(1024);
        limits.max_image_height = Some(1024);
        limits.max_alloc = Some(8 * 1024 * 1024);
        decoder.limits(limits);
        let im = decoder.decode().map_err(|e| e.to_string())?.to_rgba8();
        let texture_size = im.dimensions();
        let mut clips = BTreeMap::new();
        for a in g.animations() {
            let name = a.name().ok_or("missing clip name")?;
            if !["Idle", "Run", "Jump"].contains(&name) || clips.contains_key(name) {
                return Err("unsupported or duplicate clip".into());
            }
            let mut tracks = Vec::new();
            let mut duration = 0.0_f32;
            let mut changing = false;
            let mut targets = std::collections::HashSet::new();
            for c in a.channels() {
                let prop = c.target().property();
                let node = c.target().node().index();
                if !matches!(
                    c.sampler().interpolation(),
                    Interpolation::Linear | Interpolation::Step
                ) || !targets.insert((node, prop as u8))
                    || !skin_nodes.contains(&node)
                {
                    return Err("unsupported person animation channel".into());
                }
                let reader = c.reader(|_| Some(bin));
                let times: Vec<_> = reader.read_inputs().ok_or("missing clip times")?.collect();
                use gltf::animation::util::ReadOutputs;
                let values: Vec<Vec4> = match reader.read_outputs().ok_or("missing clip values")? {
                    ReadOutputs::Translations(v) => {
                        v.map(|v| Vec3::from_array(v).extend(0.0)).collect()
                    }
                    ReadOutputs::Rotations(v) => v.into_f32().map(Vec4::from_array).collect(),
                    ReadOutputs::Scales(v) => v.map(|v| Vec3::from_array(v).extend(0.0)).collect(),
                    _ => return Err("unsupported morph clip".into()),
                };
                if times.len() != values.len()
                    || times.is_empty()
                    || times.len() > 601
                    || times
                        .iter()
                        .any(|t| !t.is_finite() || *t < 0.0 || *t > 10.0)
                    || times.windows(2).any(|w| w[0] >= w[1])
                    || values
                        .iter()
                        .any(|v| !v.is_finite() || v.abs().max_element() > 100.0)
                    || (prop == Property::Rotation
                        && values
                            .iter()
                            .any(|v| (v.length_squared() - 1.0).abs() > 0.01))
                {
                    return Err("invalid person animation values".into());
                }
                changing |= values
                    .iter()
                    .any(|v| (*v - values[0]).abs().max_element() > 1e-5);
                duration = duration.max(*times.last().unwrap());
                tracks.push(Track {
                    step: c.sampler().interpolation() == Interpolation::Step,
                    node,
                    property: prop,
                    times,
                    values,
                });
            }
            if !changing || duration <= 0.0 {
                return Err("static person animation".into());
            }
            clips.insert(name.to_owned(), Clip { duration, tracks });
        }
        let floor = positions.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min);
        let top = positions
            .iter()
            .map(|p| p[1])
            .fold(f32::NEG_INFINITY, f32::max);
        if top - floor < 0.1 {
            return Err("invalid person height".into());
        }
        let mesh = SurfaceMeshData {
            colors: vec![[1.0; 4]; n],
            positions,
            normals,
            uvs,
            indices,
            ..default()
        };
        let mut rig = Self {
            nodes,
            parents,
            skin_nodes,
            head_joint,
            hips_joint,
            chest_joint,
            hand_joints,
            finger_nodes,
            thumb_nodes,
            leg_joints,
            inverse_binds,
            mesh_node,
            mesh,
            ids,
            weights,
            rgba: im.into_raw(),
            texture_size,
            height: top - floor,
            floor,
            bounds: (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
            clips,
            order,
        };
        // Preserve the source envelope and include the visible relaxed fingers
        // so the longer reach cannot be culled at camera edges.
        let mut pose = rig.nodes.clone();
        let mut world = vec![Mat4::IDENTITY; pose.len()];
        for (clip, relaxed) in [("Idle",false),("Run",false),("Jump",false),("Idle",true),("Run",true)] {
            let duration = rig.duration(clip);
            for i in 0..=60 {
                rig.sample(clip, duration * i as f32 / 60.0, &mut pose);
                if relaxed {crate::hand_pose::relax(&rig, &mut pose);}
                for &n in &rig.order {
                    world[n] = rig.parents[n].map(|p| world[p]).unwrap_or(Mat4::IDENTITY)
                        * pose[n].to_matrix();
                }
                let mesh_inv = world[rig.mesh_node].inverse();
                for ((p, ids), weights) in rig.mesh.positions.iter().zip(&rig.ids).zip(&rig.weights)
                {
                    let mut v = Vec3::ZERO;
                    for j in 0..4 {
                        v += (mesh_inv
                            * world[rig.skin_nodes[ids[j] as usize]]
                            * rig.inverse_binds[ids[j] as usize])
                            .transform_point3(Vec3::from_array(*p))
                            * weights[j];
                    }
                    if !v.is_finite() {
                        return Err("invalid animated body envelope".into());
                    }
                    rig.bounds.0 = rig.bounds.0.min(v);
                    rig.bounds.1 = rig.bounds.1.max(v);
                }
            }
        }
        Ok(rig)
    }
    pub fn duration(&self, name: &str) -> f32 {
        self.clips[name].duration
    }
    pub fn sample(&self, name: &str, seconds: f32, pose: &mut [Transform]) {
        pose.copy_from_slice(&self.nodes);
        let clip = &self.clips[name];
        let t = seconds.clamp(0.0, clip.duration);
        for track in &clip.tracks {
            let upper = track.times.partition_point(|v| *v <= t);
            let r = upper.min(track.times.len() - 1);
            let l = upper.saturating_sub(1).min(track.times.len() - 1);
            let f = if l == r || track.step {
                0.0
            } else {
                ((t - track.times[l]) / (track.times[r] - track.times[l])).clamp(0.0, 1.0)
            };
            let a = track.values[l];
            let b = track.values[r];
            let local = &mut pose[track.node];
            match track.property {
                Property::Translation => local.translation = a.lerp(b, f).truncate(),
                Property::Scale => local.scale = a.lerp(b, f).truncate(),
                Property::Rotation => {
                    local.rotation = Quat::from_array(a.to_array())
                        .slerp(Quat::from_array(b.to_array()), f)
                        .normalize()
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_production_rigs_reject_bundle_before_freezing_sources() {
        let frozen = crate::npc_textures::frozen();
        assert!(install_open_people_bundle(r#"{"version":1,"people":{}}"#).is_err());
        assert_eq!(crate::npc_textures::frozen(), frozen);
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn missing_native_rigs_stop_rendering_instead_of_substituting_models() {
        const CHILD: &str = "CRYSTAL_TEST_MISSING_PEOPLE_CHILD";
        if std::env::var_os(CHILD).is_some() {
            let _ = get("human-male");
            panic!("missing character rig was accepted");
        }
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "open_people::tests::missing_native_rigs_stop_rendering_instead_of_substituting_models", "--nocapture"])
            .env(CHILD, "1")
            .env("CRYSTAL_OPEN_MODEL_ROOT", std::env::temp_dir().join("geothite-required-people-missing"))
            .output().unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("required character asset"));
    }

    #[test]
    fn malformed_optional_people_do_not_install_or_escape_the_catalog() {
        assert!(PersonRig::parse(b"not a GLB").is_err());
        assert!(PersonRig::parse(&vec![0; MAX_BYTES + 1]).is_err());
        assert!(
            install_open_people_bundle(r#"{"version":1,"people":{"../outside":"Z2xURg=="}}"#)
                .is_err()
        );
        assert!(install_open_people_bundle(r#"{"version":2,"people":{}}"#).is_err());
    }
    #[test]
    #[ignore = "requires the locally supplied ignored converted Kenney rigs"]
    fn supplied_people_keep_bind_geometry_and_animated_skin_envelopes() {
        let root = std::path::PathBuf::from(
            std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").expect("supply converted CC0 root"),
        );
        for name in ["skater-male", "skater-female", "human-male", "human-female"] {
            let rig = PersonRig::parse(
                &std::fs::read(root.join(format!("people/kenney/{name}.glb"))).unwrap(),
            )
            .unwrap();
            assert!(rig.height > 3.0 && rig.height < 4.0);
            assert!(rig.skin_nodes.len() > 40);
            let mut world = vec![Mat4::IDENTITY; rig.nodes.len()];
            for &n in &rig.order {
                world[n] = rig.parents[n].map(|p| world[p]).unwrap_or(Mat4::IDENTITY)
                    * rig.nodes[n].to_matrix();
            }
            let mesh_inv = world[rig.mesh_node].inverse();
            for (&node, inv) in rig.skin_nodes.iter().zip(&rig.inverse_binds) {
                let delta = mesh_inv * world[node] * *inv - Mat4::IDENTITY;
                assert!(
                    delta.to_cols_array().iter().all(|v| v.abs() < 0.002),
                    "{name} bind pose changes neutral geometry"
                );
            }
            assert!(rig.bounds.0.is_finite() && rig.bounds.1.is_finite());
            assert!(
                rig.bounds.0.y > -0.25 && rig.bounds.1.y < rig.height * 1.6,
                "{name} feet or skeleton distorted: {:?}",
                rig.bounds
            );
            let mut first = rig.nodes.clone();
            let mut later = first.clone();
            for clip in ["Idle", "Run", "Jump"] {
                rig.sample(clip, 0.0, &mut first);
                rig.sample(clip, rig.duration(clip) * 0.37, &mut later);
                assert!(
                    first
                        .iter()
                        .zip(&later)
                        .any(|(a, b)| a.rotation.angle_between(b.rotation) > 0.01
                            || a.translation.distance(b.translation) > 0.01),
                    "{name} {clip} is static"
                );
            }
        }
    }
}
