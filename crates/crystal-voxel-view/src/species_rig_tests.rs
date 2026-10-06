use super::*;
use serde_json::{json, Value};
use std::{path::PathBuf, sync::OnceLock};

fn candidate(species: Species) -> Vec<u8> {
    let key = format!("{}_GLB", species.name().to_uppercase());
    let canonical = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(match species {
        Species::Cyndaquil | Species::Totodile => {
            format!("models/actor_props/battle_{}.glb", species.name())
        }
        Species::Gengar | Species::Spearow => {
            format!("models/battle_species/{}.glb", species.name())
        }
    });
    let path = std::env::var_os(key)
        .map(PathBuf::from)
        .unwrap_or(canonical);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn fixture(species: Species) -> &'static SpeciesRig {
    static CYNDAQUIL: OnceLock<SpeciesRig> = OnceLock::new();
    static TOTODILE: OnceLock<SpeciesRig> = OnceLock::new();
    static GENGAR: OnceLock<SpeciesRig> = OnceLock::new();
    static SPEAROW: OnceLock<SpeciesRig> = OnceLock::new();
    let cache = match species {
        Species::Cyndaquil => &CYNDAQUIL,
        Species::Totodile => &TOTODILE,
        Species::Gengar => &GENGAR,
        Species::Spearow => &SPEAROW,
    };
    cache.get_or_init(|| {
        SpeciesRig::parse(species, &candidate(species)).expect("authored skin parses")
    })
}

fn storage(rig: &SpeciesRig) -> (Vec<Transform>, Vec<Mat4>) {
    (
        vec![Transform::IDENTITY; rig.joints.len()],
        vec![Mat4::IDENTITY; rig.joints.len()],
    )
}

fn assert_vec_close(a: Vec3, b: Vec3, epsilon: f32) {
    assert!(
        (a - b).abs().max_element() <= epsilon,
        "{a:?} differs from {b:?}"
    );
}

fn assert_bits(a: impl Iterator<Item = f32>, b: impl Iterator<Item = f32>) {
    assert_eq!(
        a.map(f32::to_bits).collect::<Vec<_>>(),
        b.map(f32::to_bits).collect::<Vec<_>>()
    );
}

#[test]
fn species_schema_retains_anatomy_skin_order_and_physical_size() {
    for (species, vertices, triangles, height) in [
        (Species::Cyndaquil, 4506, 2086, 0.91_f32),
        (Species::Totodile, 5714, 6374, 1.02_f32),
        (Species::Gengar, 6952, 5560, 1.05_f32),
        (Species::Spearow, 2139, 1929, 0.75_f32),
    ] {
        let rig = fixture(species);
        assert_eq!(rig.species, species);
        assert_eq!(rig.neutral.positions.len(), vertices);
        assert_eq!(rig.neutral.indices.len() / 3, triangles);
        assert_eq!(rig.anatomy.len(), species.part_count());
        assert_eq!(rig.joint_indices.len(), vertices);
        assert_eq!(rig.joint_weights.len(), vertices);
        assert_eq!(rig.source_normals.len(), vertices);
        assert_eq!(rig.neutral_bounds.0.y, 0.0);
        assert!((rig.neutral_bounds.1.y - height).abs() < 0.000001);
        assert_eq!(
            rig.joints.iter().map(|j| j.parent).collect::<Vec<_>>(),
            species.parents()
        );
        for (joint, suffix) in rig.joints.iter().zip(species.joint_names()) {
            assert_eq!(joint.name, format!("{}/{suffix}", species.name()));
        }
        let mut cursor = (0, 0);
        for part in &rig.anatomy {
            assert!(!part.name.is_empty());
            assert_eq!((part.vertices.start, part.indices.start), cursor);
            cursor = (part.vertices.end, part.indices.end);
            let material = &rig.materials[part.material];
            assert!(!material.name.is_empty());
            assert_eq!(material.alpha_mode, gltf::material::AlphaMode::Opaque);
            assert!(rig.neutral.colors[part.vertices.clone()]
                .iter()
                .all(|c| *c == material.color));
        }
        assert_eq!(cursor, (vertices, triangles * 3));
        for (&source, &normal) in rig.source_normals.iter().zip(&rig.neutral.normals) {
            assert_bits(
                Vec3::from_array(source).normalize().to_array().into_iter(),
                normal.into_iter(),
            );
        }
    }
}

#[test]
fn spearow_lookup_keeps_canonical_geometry_and_authored_clip_data() {
    let rig = fixture(Species::Spearow);
    for name in ["SPEAROW", "spearow", "SpEaRoW"] {
        let registered = for_species(name).expect("Spearow uses the generic skin runtime");
        assert!(std::ptr::eq(registered, super::rig(Species::Spearow)));
        assert_eq!(registered.neutral, rig.neutral);
        assert_eq!(
            crate::battle_species_models::mesh(name).unwrap(),
            rig.neutral
        );
    }
    assert_eq!(rig.neutral_bounds.0, Vec3::new(-0.368, 0.0, -0.425));
    assert_eq!(rig.neutral_bounds.1, Vec3::new(0.368, 0.75, 0.425));

    let bytes = candidate(Species::Spearow);
    let gltf = gltf::Gltf::from_slice(&bytes).unwrap();
    let binary = gltf.blob.as_deref().unwrap();
    for ((animation, kind), (duration, keys)) in
        gltf.animations()
            .zip(Clip::ALL)
            .zip([(2.8_f32, 73), (1.0, 65), (1.0, 65)])
    {
        let clip = rig.clip(kind);
        assert_eq!(clip.duration, duration);
        assert_eq!(clip.times.len(), keys);
        assert_eq!(clip.extras, animation.extras().as_ref().unwrap().get());
        for channel in animation.channels() {
            let reader = channel.reader(|_| Some(binary));
            assert_bits(reader.read_inputs().unwrap(), clip.times.iter().copied());
            let Some(gltf::animation::util::ReadOutputs::Rotations(rotations)) =
                reader.read_outputs()
            else {
                panic!("Spearow clip must retain rotation channels");
            };
            assert_bits(
                rotations.into_f32().flatten(),
                clip.rotations[channel.target().node().index() - 1]
                    .iter()
                    .flat_map(|q| q.to_array()),
            );
        }
    }
}

#[test]
fn spearow_each_joint_deforms_its_anatomy_while_feet_stay_planted() {
    let rig = fixture(Species::Spearow);
    let feet: Vec<_> = rig
        .anatomy
        .iter()
        .filter(|part| part.name.starts_with("Feet /"))
        .collect();
    assert_eq!(feet.len(), 4);
    let feet: Vec<_> = feet.iter().flat_map(|part| part.vertices.clone()).collect();
    for &vertex in &feet {
        assert_eq!(rig.joint_indices[vertex], [0; 4]);
        assert_eq!(rig.joint_weights[vertex], [1.0, 0.0, 0.0, 0.0]);
    }
    let bind: Vec<_> = rig.joints.iter().map(|joint| joint.bind).collect();
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        let mut peak_angles = vec![0.0_f32; rig.joints.len()];
        let mut peak_rotations = vec![Quat::IDENTITY; rig.joints.len()];
        let frames = (rig.clip(clip).duration * 60.0).ceil() as usize;
        for frame in 0..=frames {
            rig.sample_into(
                clip,
                rig.clip(clip).duration * frame as f32 / frames as f32,
                Playback::Clamp,
                &mut pose,
            )
            .unwrap();
            assert_eq!(pose[0], Transform::IDENTITY);
            for joint in 1..pose.len() {
                let angle = pose[joint].rotation.angle_between(bind[joint].rotation);
                if angle > peak_angles[joint] {
                    peak_angles[joint] = angle;
                    peak_rotations[joint] = pose[joint].rotation;
                }
            }
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for &vertex in &feet {
                assert_vec_close(
                    rig.skin_point(vertex, &matrices),
                    Vec3::from_array(rig.neutral.positions[vertex]),
                    0.000001,
                );
            }
        }
        for joint in 1..rig.joints.len() {
            assert!(
                peak_angles[joint] > 0.01,
                "{} lacks local articulation of {}",
                rig.clip(clip).name,
                rig.joints[joint].name
            );
            // Isolate this joint's own rotation. An ancestor or whole-body
            // motion cannot satisfy the anatomical displacement requirement.
            pose.copy_from_slice(&bind);
            pose[joint].rotation = peak_rotations[joint];
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            let displacement = (0..rig.neutral.positions.len())
                .filter(|&vertex| {
                    rig.joint_indices[vertex]
                        .iter()
                        .zip(rig.joint_weights[vertex])
                        .any(|(&id, weight)| id as usize == joint && weight > 0.0)
                })
                .map(|vertex| {
                    rig.skin_point(vertex, &matrices)
                        .distance(Vec3::from_array(rig.neutral.positions[vertex]))
                })
                .fold(0.0_f32, f32::max);
            assert!(
                displacement > 0.0005,
                "{} does not move the geometry owned by {}: {displacement}",
                rig.clip(clip).name,
                rig.joints[joint].name
            );
        }
    }
}

#[test]
fn spearow_pack_scale_and_contact_ignore_animation_envelope() {
    use crate::battle_layout::{
        model_scale, reference_span, BattleBody, BattleSceneLayout, WORLD_UNITS_PER_METER,
    };
    use bevy::prelude::Vec2;

    let rig = fixture(Species::Spearow);
    let (min, max) = rig.neutral_bounds;
    assert_eq!(reference_span("SPEAROW", min, max), 0.75);
    assert!(model_scale("SPEAROW", None, min, max).is_none());
    // The current pack supplies 0.3048 m. Also check another supplied value
    // so a hidden per-species constant cannot satisfy the physical-size test.
    for meters in [0.3048_f32, 0.6096] {
        let scale = model_scale("SPEAROW", Some(meters), min, max).unwrap();
        assert!(((max.y - min.y) * scale / WORLD_UNITS_PER_METER - meters).abs() < 0.000001);
        let neutral = BattleBody::modeled("SPEAROW", min, max, scale);
        let mut animated = neutral;
        animated.visual_bounds = Some(rig.animated_bounds);
        for viewport in [Vec2::new(1180.0, 812.0), Vec2::new(600.0, 1000.0)] {
            let before = BattleSceneLayout::for_bodies([Some(neutral); 2], viewport);
            let after = BattleSceneLayout::for_bodies([Some(animated); 2], viewport);
            for side in 0..2 {
                let pose = after.body_poses[side].unwrap();
                assert_eq!(pose.scale, Vec3::splat(scale));
                assert_eq!(pose.scale, before.body_poses[side].unwrap().scale);
                assert_eq!(
                    pose.translation.y,
                    before.body_poses[side].unwrap().translation.y
                );
                assert_eq!(after.hit_anchors[side].y, before.hit_anchors[side].y);
                let contact_y = rig
                    .neutral
                    .positions
                    .iter()
                    .map(|point| pose.transform_point(Vec3::from_array(*point)).y)
                    .fold(f32::INFINITY, f32::min);
                assert!((contact_y - after.origins[side].y).abs() < 0.000001);
            }
        }
    }
}

#[test]
fn weighted_bind_and_clip_endpoints_preserve_neutral_vertices() {
    for species in Species::ALL {
        let rig = fixture(species);
        let (mut pose, mut matrices) = storage(rig);
        for clip in Clip::ALL {
            assert_eq!(
                rig.clip(clip).name,
                format!("{}.{}", species.name(), clip.suffix())
            );
            assert_eq!(rig.clip(clip).loop_suggested, clip == Clip::Idle);
            assert_eq!(rig.clip(clip).cue_relative, clip != Clip::Idle);
            assert!(!rig.clip(clip).extras.is_empty());
            for time in [
                -1.0,
                0.0,
                rig.clip(clip).duration,
                rig.clip(clip).duration + 1.0,
            ] {
                rig.sample_into(clip, time, Playback::Clamp, &mut pose)
                    .unwrap();
                assert_eq!(pose, rig.joints.iter().map(|j| j.bind).collect::<Vec<_>>());
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                for m in &matrices {
                    assert!(m
                        .to_cols_array()
                        .iter()
                        .zip(Mat4::IDENTITY.to_cols_array())
                        .all(|(a, b)| (a - b).abs() < 0.000002));
                }
                for (i, p) in rig.neutral.positions.iter().enumerate() {
                    assert_vec_close(rig.skin_point(i, &matrices), Vec3::from_array(*p), 0.000002);
                }
            }
        }
    }
}

#[test]
fn authored_clips_are_finite_at_30_and_60_hz_and_do_articulate() {
    for species in Species::ALL {
        let rig = fixture(species);
        let (mut pose, mut matrices) = storage(rig);
        let allocation = pose.as_ptr();
        for clip in Clip::ALL {
            let mut displaced = false;
            for hz in [30.0, 60.0] {
                let frames = (rig.clip(clip).duration * hz).ceil() as usize;
                for frame in 0..=frames {
                    let t = rig.clip(clip).duration * frame as f32 / frames as f32;
                    rig.sample_into(clip, t, Playback::Clamp, &mut pose)
                        .unwrap();
                    assert_eq!(pose.as_ptr(), allocation);
                    assert_eq!(pose[0], Transform::IDENTITY);
                    assert!(pose.iter().all(|p| p.translation.is_finite()
                        && p.rotation.is_finite()
                        && p.scale == Vec3::ONE));
                    rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                    for (i, p) in rig.neutral.positions.iter().enumerate() {
                        let posed = rig.skin_point(i, &matrices);
                        assert!(posed.is_finite());
                        displaced |= posed.distance(Vec3::from_array(*p)) > 0.001;
                    }
                }
            }
            assert!(
                displaced,
                "{} was flattened to static geometry",
                rig.clip(clip).name
            );
        }
        for t in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(rig
                .sample_into(Clip::Idle, t, Playback::Loop, &mut pose)
                .is_err());
        }
        assert!(rig
            .sample_into(Clip::Idle, 0.0, Playback::Loop, &mut [])
            .is_err());
    }
}

#[test]
fn idle_loop_is_closed_and_cue_clips_clamp_to_neutral() {
    for species in Species::ALL {
        let rig = fixture(species);
        let (mut pose, mut matrices) = storage(rig);
        let duration = rig.clip(Clip::Idle).duration;
        for time in [-duration, 0.0, duration, duration * 2.0] {
            rig.sample_into(Clip::Idle, time, Playback::Loop, &mut pose)
                .unwrap();
            assert_eq!(pose, rig.joints.iter().map(|j| j.bind).collect::<Vec<_>>());
        }
        for time in [duration - 0.00001, duration + 0.00001] {
            rig.sample_into(Clip::Idle, time, Playback::Loop, &mut pose)
                .unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for (i, p) in rig.neutral.positions.iter().enumerate() {
                assert_vec_close(rig.skin_point(i, &matrices), Vec3::from_array(*p), 0.00001);
            }
        }
    }
}

#[test]
fn animation_envelope_covers_dense_off_grid_samples_without_resizing() {
    for species in Species::ALL {
        let rig = fixture(species);
        let (mut pose, mut matrices) = storage(rig);
        let envelope_extent = rig.animated_bounds.1 - rig.animated_bounds.0;
        let mut sampled_bounds = rig.neutral_bounds;
        for clip in Clip::ALL {
            // Prime grid intentionally misses both the authored keys and the
            // 30 Hz envelope samples. Include every vertex and clip endpoints.
            for frame in 0..=251 {
                let time = rig.clip(clip).duration * frame as f32 / 251.0;
                rig.sample_into(clip, time, Playback::Clamp, &mut pose)
                    .unwrap();
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                for i in 0..rig.neutral.positions.len() {
                    let point = rig.skin_point(i, &matrices);
                    sampled_bounds.0 = sampled_bounds.0.min(point);
                    sampled_bounds.1 = sampled_bounds.1.max(point);
                    assert!(
                        point.cmpge(rig.animated_bounds.0).all()
                            && point.cmple(rig.animated_bounds.1).all(),
                        "{} t={time} vertex={i} outside {:?}: {point:?}",
                        rig.clip(clip).name,
                        rig.animated_bounds
                    );
                }
            }
        }
        // Bound padding around actual animation, not motion relative to rest.
        // Gengar's larger pose legitimately grows depth; neutral bounds still
        // exclusively control physical scale and footing in separate tests.
        // Independent decoded sampling (including 2,100 blended poses) found
        // Gengar's conservative/animated extent ratios to be 1.052/1.067/1.092.
        let sampled_extent = sampled_bounds.1 - sampled_bounds.0;
        assert!(
            (envelope_extent / sampled_extent).max_element() < 1.10,
            "camera padding exceeds its budget: {sampled_extent:?} -> {envelope_extent:?}"
        );
        for (from_clip, to_clip) in [
            (Clip::Idle, Clip::Attack),
            (Clip::Attack, Clip::Hit),
            (Clip::Hit, Clip::Idle),
        ] {
            for phase in [0.2, 0.43, 0.75] {
                let mut from = pose.clone();
                let mut target = pose.clone();
                rig.sample_into(
                    from_clip,
                    phase * rig.clip(from_clip).duration,
                    Playback::Clamp,
                    &mut from,
                )
                .unwrap();
                rig.sample_into(
                    to_clip,
                    (1.0 - phase) * rig.clip(to_clip).duration,
                    Playback::Clamp,
                    &mut target,
                )
                .unwrap();
                for blend in [0.25, 0.5, 0.75] {
                    for ((p, a), b) in pose.iter_mut().zip(&from).zip(&target) {
                        *p = *b;
                        p.rotation = a.rotation.slerp(b.rotation, blend).normalize();
                    }
                    rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                    for i in 0..rig.neutral.positions.len() {
                        let point = rig.skin_point(i, &matrices);
                        assert!(
                            point.cmpge(rig.animated_bounds.0).all()
                                && point.cmple(rig.animated_bounds.1).all(),
                            "{} blended vertex {i} outside {:?}: {point:?}",
                            rig.species.name(),
                            rig.animated_bounds
                        );
                    }
                }
            }
        }
    }
}

fn rewrite(bytes: &[u8], edit: impl FnOnce(&mut Value, &mut Vec<u8>)) -> Vec<u8> {
    let glb = gltf::binary::Glb::from_slice(bytes).unwrap();
    let mut doc = serde_json::from_slice(&glb.json).unwrap();
    let mut binary = glb.bin.unwrap().to_vec();
    edit(&mut doc, &mut binary);
    let mut json = serde_json::to_vec(&doc).unwrap();
    while json.len() % 4 != 0 {
        json.push(b' ');
    }
    while binary.len() % 4 != 0 {
        binary.push(0);
    }
    let mut out = Vec::new();
    out.extend(b"glTF");
    out.extend(2_u32.to_le_bytes());
    out.extend(((28 + json.len() + binary.len()) as u32).to_le_bytes());
    out.extend((json.len() as u32).to_le_bytes());
    out.extend(b"JSON");
    out.extend(json);
    out.extend((binary.len() as u32).to_le_bytes());
    out.extend(b"BIN\0");
    out.extend(binary);
    out
}

fn offset(doc: &Value, accessor: usize) -> usize {
    let a = &doc["accessors"][accessor];
    doc["bufferViews"][a["bufferView"].as_u64().unwrap() as usize]["byteOffset"]
        .as_u64()
        .unwrap_or(0) as usize
        + a["byteOffset"].as_u64().unwrap_or(0) as usize
}

#[test]
fn parser_rejects_malformed_or_unsupported_json_before_reading_binary() {
    let species = Species::Totodile;
    let bytes = candidate(species);
    for length in [0, 4, 12, 20, 27, bytes.len() - 1] {
        assert!(SpeciesRig::parse(species, &bytes[..length]).is_err());
    }
    type Edit = fn(&mut Value, &mut Vec<u8>);
    let edits: &[Edit] = &[
        |d, _| d["buffers"][0]["uri"] = json!("other.bin"),
        |d, _| d["buffers"][0]["byteLength"] = json!(999_999_999),
        |d, _| d["bufferViews"][0]["byteOffset"] = json!(999_999_996),
        |d, _| d["bufferViews"][0]["byteStride"] = json!(64),
        |d, _| d["accessors"][0]["count"] = json!(999_999),
        |d, _| d["accessors"][0]["byteOffset"] = json!(999_999_996),
        |d, _| d["accessors"][0]["normalized"] = json!(true),
        |d, _| d["nodes"][2]["children"] = json!([1]),
        |d, _| d["nodes"][0]["children"] = json!([1, 1, 6]),
        |d, _| d["nodes"][3]["name"] = json!("totodile/wrong_joint"),
        |d, _| d["nodes"][8]["translation"] = json!([1, 0, 0]),
        |d, _| d["nodes"][1]["scale"] = json!([2, 1, 1]),
        |d, _| d["skins"][0]["joints"] = json!([0, 2, 1, 3, 4, 5, 6, 7]),
        |d, _| d["meshes"][0]["primitives"][0]["mode"] = json!(1),
        |d, _| d["meshes"][0]["primitives"][0]["attributes"]["TEXCOORD_0"] = json!(1),
        |d, _| d["materials"][0]["pbrMetallicRoughness"]["metallicFactor"] = json!(1),
        |d, _| d["materials"][0]["extensions"] = json!({"KHR_materials_unlit": {}}),
        |d, _| d["animations"][0]["samplers"][0]["interpolation"] = json!("STEP"),
        |d, _| d["animations"][0]["channels"][0]["target"]["path"] = json!("translation"),
        |d, _| d["animations"][0]["channels"][0]["target"]["node"] = json!(0),
        |d, _| d["animations"][0]["extras"]["loopSuggested"] = json!(false),
        |d, _| d["scenes"][0]["nodes"] = json!([0]),
    ];
    for (i, edit) in edits.iter().enumerate() {
        assert!(
            SpeciesRig::parse(species, &rewrite(&bytes, edit)).is_err(),
            "malformed JSON case {i} accepted"
        );
    }
}

#[test]
fn parser_rejects_nonfinite_keys_bad_weights_invalid_binds_and_changed_geometry() {
    let species = Species::Totodile;
    let bytes = candidate(species);
    type Edit = fn(&mut Value, &mut Vec<u8>);
    let edits: &[Edit] = &[
        |d, b| {
            let o = offset(d, 0);
            b[o..o + 4].copy_from_slice(&f32::NAN.to_le_bytes());
        },
        |d, b| {
            let o = offset(d, 0);
            b[o..o + 4].copy_from_slice(&2_f32.to_le_bytes());
        },
        |d, b| {
            let o = offset(d, 1);
            b[o..o + 4].copy_from_slice(&f32::INFINITY.to_le_bytes());
        },
        |d, b| {
            let o = offset(d, 1);
            b[o..o + 4].copy_from_slice(&0.123_f32.to_le_bytes());
        },
        |d, b| {
            let o = offset(d, 3);
            b[o] = 99;
        },
        |d, b| {
            let o = offset(d, 4);
            b[o..o + 4].fill(0);
        },
        |d, b| {
            let o = offset(d, 5);
            b[o..o + 2].copy_from_slice(&u16::MAX.to_le_bytes());
        },
        |d, b| {
            let a = d["animations"][0]["samplers"][0]["input"].as_u64().unwrap() as usize;
            let o = offset(d, a) + 4;
            b[o..o + 4].copy_from_slice(&0_f32.to_le_bytes());
        },
        |d, b| {
            let a = d["animations"][0]["samplers"][0]["output"]
                .as_u64()
                .unwrap() as usize;
            let o = offset(d, a) + 16;
            b[o..o + 16].fill(0);
        },
        |d, b| {
            let a = d["animations"][0]["samplers"][0]["output"]
                .as_u64()
                .unwrap() as usize;
            let o = offset(d, a) + 16;
            b[o..o + 4].copy_from_slice(&f32::NAN.to_le_bytes());
        },
    ];
    for (i, edit) in edits.iter().enumerate() {
        assert!(
            SpeciesRig::parse(species, &rewrite(&bytes, edit)).is_err(),
            "malformed binary case {i} accepted"
        );
    }
}

#[test]
#[ignore = "one-time migration audit: set CYNDAQUIL_MESH_SOURCE, TOTODILE_MESH_SOURCE and GENGAR_MESH_SOURCE to retired source JSON"]
fn exact_neutral_equivalence_to_surviving_source_json() {
    // These three assets migrated unchanged from JSON. Spearow's canonical
    // sculpture was authored directly as GLB and has no equivalent JSON asset.
    for species in [Species::Cyndaquil, Species::Totodile, Species::Gengar] {
        let rig = fixture(species);
        let path = std::env::var(format!("{}_MESH_SOURCE", species.name().to_uppercase()))
            .expect("source path required");
        let source: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let primitives = source["primitives"].as_array().unwrap();
        assert_eq!(primitives.len(), rig.anatomy.len());
        for (part, original) in rig.anatomy.iter().zip(primitives) {
            let floats = |name: &str| {
                original[name]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_f64().unwrap() as f32)
                    .collect::<Vec<_>>()
            };
            assert_bits(
                rig.neutral.positions[part.vertices.clone()]
                    .iter()
                    .flatten()
                    .copied(),
                floats("positions").into_iter(),
            );
            assert_bits(
                rig.source_normals[part.vertices.clone()]
                    .iter()
                    .flatten()
                    .copied(),
                floats("normals").into_iter(),
            );
            assert_bits(
                rig.materials[part.material].color.into_iter(),
                floats("base_color").into_iter(),
            );
            let expected: Vec<_> = original["indices"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap() as u32)
                .collect();
            let actual: Vec<_> = rig.neutral.indices[part.indices.clone()]
                .iter()
                .map(|i| i - part.vertices.start as u32)
                .collect();
            assert_eq!(actual, expected);
        }
    }
}
