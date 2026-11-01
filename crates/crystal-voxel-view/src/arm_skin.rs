//! A continuous shoulder/elbow skin field on the final authored arm surface.
use crate::{mesh::SurfaceMeshData, open_people::PersonRig};
use bevy::prelude::*;

fn smooth(value: f32) -> f32 {
    let t = value.clamp(0., 1.);
    t * t * (3. - 2. * t)
}

pub(crate) fn ancestry(rig: &PersonRig, side: usize) -> [usize; 5] {
    let hand = &rig.hand_joints[side];
    let wrist = hand
        .iter()
        .map(|&j| rig.skin_nodes[j])
        .find(|&node| {
            rig.parents[node].is_none_or(|p| !hand.iter().any(|&j| rig.skin_nodes[j] == p))
        })
        .expect("human arm requires a wrist");
    let mut node = wrist;
    std::array::from_fn(|_| {
        node = rig.parents[node].expect("human arm requires chest ancestry");
        rig.skin_nodes
            .iter()
            .position(|&n| n == node)
            .expect("arm ancestor must be skinned")
    })
}

pub(crate) fn carriers(rig: &PersonRig, side: usize) -> Vec<usize> {
    let arm=ancestry(rig,side);
    let mut result=arm.to_vec();
    let mut node=rig.parents[rig.skin_nodes[arm[4]]];
    while let Some(current)=node {
        if let Some(joint)=rig.skin_nodes.iter().position(|&n|n==current) {result.push(joint);}
        node=rig.parents[current];
    }
    result
}

/// The curved skin field also needs enough triangles across the shoulder.
/// Linear splitting retains the exact bind surface and UV interpolation;
/// joined edges retain closed seams between different painted islands.
pub(crate) fn refine(
    rig: &PersonRig,
    mesh: &mut SurfaceMeshData,
    ids: &mut Vec<[u16; 4]>,
    weights: &mut Vec<[f32; 4]>,
) {
    refine_with_reference(rig,mesh,ids,weights,None);
}

pub(crate) fn refine_reference(
    rig: &PersonRig, mesh: &mut SurfaceMeshData, ids: &mut Vec<[u16;4]>, weights: &mut Vec<[f32;4]>,
    reference: &mut Vec<[f32;3]>,
) {
    refine_with_reference(rig,mesh,ids,weights,Some(reference));
}

fn refine_with_reference(
    rig: &PersonRig, mesh: &mut SurfaceMeshData, ids: &mut Vec<[u16;4]>, weights: &mut Vec<[f32;4]>,
    mut reference: Option<&mut Vec<[f32;3]>>,
) {
    #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
    if std::env::var_os("CRYSTAL_PEOPLE_REVIEW_ARM_SKIN_BASELINE").is_some() {
        return;
    }
    let arms = [ancestry(rig, 0), ancestry(rig, 1)];
    let torso: Vec<_>=[carriers(rig,0),carriers(rig,1)].into_iter().flatten().collect();
    let length = (rig.inverse_binds[arms[0][0]].inverse().w_axis.truncate()
        - rig.inverse_binds[arms[0][1]].inverse().w_axis.truncate())
    .length();
    let limit = (0.02 * length / 0.47).powi(2);
    let minimum_x=0_f32;
    #[cfg(all(feature="people-review",not(target_arch="wasm32")))]
    let minimum_x=if std::env::var_os("CRYSTAL_PEOPLE_REVIEW_SPINE_REFINEMENT_BASELINE").is_some() {0.08} else {minimum_x};
    for _ in 0..4 {
        let mut selected = vec![false; mesh.positions.len()];
        let positions=reference.as_ref().map(|p|p.as_slice()).unwrap_or(mesh.positions.as_slice());
        for face in mesh.indices.chunks_exact(3) {
            let indices = [face[0] as usize, face[1] as usize, face[2] as usize];
            if !indices.iter().all(|&i| {
                let p = positions[i];
                // Include the full transition triangles below the arm root.
                // A long chest edge can cross the curved weight field even
                // when its lower endpoint has almost no arm influence.
                // A transition fan can reach the spine. Rejecting its inner
                // corner leaves a long, coarse face beside the refined arm.
                (minimum_x..1.04).contains(&p[0].abs())
                    && (1.78..2.68).contains(&p[1])
                    && (0..4)
                        .filter(|&s| torso.contains(&(ids[i][s] as usize)))
                        .map(|s| weights[i][s])
                        .sum::<f32>()
                        > 0.999
            }) {
                continue;
            }
            if !indices.iter().any(|&i| (0..4).any(|s|weights[i][s]>0.001
                && arms.iter().any(|arm|arm[..3].contains(&(ids[i][s] as usize))))) {continue;}
            let p = indices.map(|i| Vec3::from_array(positions[i]));
            if (p[0] - p[1])
                .length_squared()
                .max((p[1] - p[2]).length_squared())
                .max((p[2] - p[0]).length_squared())
                > limit
            {
                for i in indices {
                    selected[i] = true;
                }
            }
        }
        if !selected.iter().any(|&s| s) {
            break;
        }
        if let Some(reference)=reference.as_deref_mut() {
            crate::hand_surface::linear_selection_joined_reference(mesh,ids,weights,&selected,reference);
        } else {
            crate::hand_surface::linear_selection_joined(mesh, ids, weights, &selected);
        }
    }
}

/// Fit after garment shaping/refinement, before attribute compaction. Adjacent
/// source rings jump from 20% to 100% upper-arm influence; lowering that arm
/// turns their shared underarm faces inside out. Both bends need a continuous
/// field, including new sleeve vertices and copies on different UV islands.
/// The bind shape, normals, artwork, wrists and the rig remain authored.
pub(crate) fn fit(
    rig: &PersonRig,
    mesh: &SurfaceMeshData,
    ids: &mut [[u16; 4]],
    weights: &mut [[f32; 4]],
) {
    #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
    if std::env::var_os("CRYSTAL_PEOPLE_REVIEW_ARM_SKIN_BASELINE").is_some() {
        return;
    }
    for side in 0..2 {
        let [forearm, arm, shoulder, upper_chest, chest] = ancestry(rig, side);
        let torso=carriers(rig,side);
        let origin = |joint: usize| {
            rig.inverse_binds[joint]
                .inverse()
                .transform_point3(Vec3::ZERO)
        };
        let upper = origin(arm);
        let elbow = origin(forearm);
        let units = upper.distance(elbow) / 0.47;
        let sign = (upper.x - origin(chest).x).signum();
        let root_x = upper.x * sign;
        let start = root_x - 0.235 * units;
        let end = root_x + 0.335 * units;
        let elbow_start = elbow.x * sign - 0.205 * units;
        let elbow_end = elbow.x * sign + 0.215 * units;
        for ((&p, joints), values) in mesh
            .positions
            .iter()
            .zip(ids.iter_mut())
            .zip(weights.iter_mut())
        {
            let weight = |set: &[usize]| {
                (0..4)
                    .filter(|&s| set.contains(&(joints[s] as usize)))
                    .map(|s| values[s])
                    .sum::<f32>()
            };
            let carried = weight(&[shoulder, arm, forearm]);
            let attached = weight(&torso);
            let x = p[0] * sign;
            if carried <= 0.001 || attached < 0.999 || x >= elbow_end {
                continue;
            }
            let t = smooth((x - start) / (end - start));
            let e = smooth((x - elbow_start) / (elbow_end - elbow_start));
            let c = 0.35 + 0.65 * smooth((p[1] - (upper.y - 0.19 * units)) / (0.25 * units));
            // Fade into the untouched torso where subdivision interpolated a
            // small shoulder influence. Keep the original chest field there.
            let mix = smooth(carried / 0.30)
                * smooth((p[1] - (upper.y - 0.50 * units)) / (0.20 * units));
            if mix==0. {continue;}
            let mut blend = std::collections::BTreeMap::<u16, f32>::new();
            for (&joint, &value) in joints.iter().zip(values.iter()) {
                *blend.entry(joint).or_default() += value * (1. - mix);
            }
            for (joint, value) in [
                (upper_chest, (1. - t) * (1. - c)),
                (shoulder, (1. - t) * c),
                (arm, t * (1. - e)),
                (forearm, t * e),
            ] {
                *blend.entry(joint as u16).or_default() += value * mix;
            }
            let mut blend: Vec<_> = blend.into_iter().collect();
            blend.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            blend.truncate(4);
            let total: f32 = blend.iter().map(|p| p.1).sum();
            *joints = std::array::from_fn(|s| blend.get(s).map_or(0, |p| p.0));
            *values = std::array::from_fn(|s| blend.get(s).map_or(0., |p| p.1 / total));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires supplied human rigs"]
    fn supplied_underarm_spine_blends_are_refined_and_enter_the_arm_field() {
        for name in ["human-male","human-female"] {
            let rig=crate::open_people::get(name).unwrap();
            for side in 0..2 {
                let [_,arm,shoulder,upper,chest]=ancestry(rig,side);
                let spine=rig.skin_nodes.iter().position(|&n|
                    Some(n)==rig.parents[rig.skin_nodes[chest]]).unwrap();
                let sign=if side==0 {1.}else{-1.};
                // The native bare-torso recess contains this mixed Spine /
                // Chest / UpperChest / Shoulder face. A five-joint-only
                // selection left its lower corner coarse and unfitted.
                let points: Vec<[f32;3]>=vec![[-0.32797,2.16904,0.04463],[-0.32968,2.16991,0.03388],[-0.31928,2.12663,0.08123]]
                    .into_iter().map(|mut p: [f32;3]|{p[0]=p[0].abs()*sign;p}).collect::<Vec<_>>();
                let mut mesh=SurfaceMeshData{positions:points.clone(),normals:vec![[0.,0.,1.];3],
                    uvs:vec![[0.3,0.7];3],colors:vec![[1.;4];3],indices:vec![0,1,2],..Default::default()};
                let mut ids=vec![[chest as u16,upper as u16,shoulder as u16,spine as u16];3];
                let mut weights=vec![[0.3125,0.3125,0.3125,0.0625];3];
                refine(rig,&mut mesh,&mut ids,&mut weights);
                assert!(mesh.indices.len()>3,"Spine-weighted underarm face was not refined");
                assert_eq!(&mesh.positions[..3],points.as_slice());
                fit(rig,&mesh,&mut ids,&mut weights);
                for i in 0..3 {
                    let carried: f32=(0..4).filter(|&s|ids[i][s]as usize==arm).map(|s|weights[i][s]).sum();
                    assert!(carried>0.15,"mixed torso corner stayed outside the shoulder field");
                }
                for w in weights {assert!((w.iter().sum::<f32>()-1.).abs()<1e-5);}
            }
        }
    }
    #[test]
    #[ignore = "requires supplied human rigs"]
    fn supplied_arm_field_preserves_surface_and_unrelated_skin() {
        for name in ["human-male", "human-female"] {
            let rig = crate::open_people::get(name).unwrap();
            let body = rig.refined_body();
            let mut mesh = body.mesh.clone();
            let mut ids = body.ids.clone();
            let mut weights = body.weights.clone();
            refine(rig, &mut mesh, &mut ids, &mut weights);
            let old = body.mesh.positions.len();
            assert_eq!(&mesh.positions[..old], &body.mesh.positions);
            assert_eq!(&mesh.normals[..old], &body.mesh.normals);
            assert_eq!(&mesh.uvs[..old], &body.mesh.uvs);
            assert_eq!(&mesh.colors[..old], &body.mesh.colors);
            assert!(mesh.indices.len() > body.mesh.indices.len());
            fit(rig, &mesh, &mut ids, &mut weights);
            let mut changed = 0;
            for i in 0..ids.len() {
                let p = mesh.positions[i];
                if i < old && (p[1] < 1.8 || p[1] > 2.65 || p[0].abs() > 1.1) {
                    assert_eq!(ids[i], body.ids[i], "{name} protected joints {i}");
                    assert_eq!(weights[i], body.weights[i], "{name} protected weights {i}");
                }
                assert!(weights[i].iter().all(|&w| w.is_finite() && w >= 0.));
                assert!((weights[i].iter().sum::<f32>() - 1.).abs() < 1e-5);
                assert!(ids[i].iter().all(|&j| (j as usize) < rig.skin_nodes.len()));
                changed += usize::from(i < old && weights[i] != body.weights[i]);
            }
            assert!(changed > 100, "{name} arm field was not applied");
        }
    }
}

#[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
pub(crate) mod validation {
    use super::*;
    use crate::{
        new_bark_actors::{
            ModeledActor,
            character_meshes::{CharacterKind, SOURCE_KINDS},
        },
        open_people_skin::{PeopleGpu, prepare_appearance},
    };
    use bevy::{
        ecs::{system::SystemState, world::CommandQueue},
        mesh::{VertexAttributeValues, skinning::SkinnedMeshInverseBindposes},
    };

    /// Use final production geometry and Person::update, including proportion
    /// transforms and blended gait planting, rather than the imported clips.
    #[cfg_attr(test, test)]
    #[cfg_attr(test, ignore = "requires supplied human rigs and paintings")]
    pub(crate) fn supplied_cast_arms_remain_outward_through_production_motion() {
        let mut lowest_dot = 1_f32;
        let mut smallest_area = f32::INFINITY;
        let mut checked = 0;
        let mut face_samples=0_u64;
        let mut largest_palette=0;
        let mut lowest_resolution=12;
        let mut rejected = Vec::new();
        for (kind, bicycle) in SOURCE_KINDS
            .iter()
            .map(|&(_, kind)| (kind, kind == CharacterKind::Biker))
            .chain([
                (CharacterKind::Outdoorsman, false),
                (CharacterKind::Trainer, true),
                (CharacterKind::TrainerFemale, true),
            ])
        {
            let look = crate::npc_appearance::look(kind);
            let rig = crate::open_people::get(look.base).unwrap().clone();
            let prepared = prepare_appearance(&rig, kind);
            let mesh = prepared.review_body();
            if let Some(recipe)=prepared.review_arm_deformation() {
                recipe.validate_mesh(mesh).expect("production arm blend palette");
                largest_palette=largest_palette.max(recipe.original_count()+recipe.helper_count());
                lowest_resolution=lowest_resolution.min(recipe.resolution());
            }
            let Some(VertexAttributeValues::Float32x3(p)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                panic!("positions");
            };
            let Some(VertexAttributeValues::Float32x3(n)) = mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
            else {
                panic!("normals");
            };
            let Some(VertexAttributeValues::Uint16x4(j)) =
                mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX)
            else {
                panic!("joints");
            };
            let Some(VertexAttributeValues::Float32x4(w)) =
                mesh.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT)
            else {
                panic!("weights");
            };
            let faces: Vec<[usize; 3]> = mesh
                .indices()
                .unwrap()
                .iter()
                .collect::<Vec<_>>()
                .chunks_exact(3)
                .filter(|f| {
                    f.iter()
                        .all(|&i| (1.9..2.7).contains(&p[i][1]) && p[i][0].abs() < 1.14)
                        && f.iter().any(|&i| {
                            (0.08..1.03).contains(&p[i][0].abs()) && (2.08..2.7).contains(&p[i][1])
                        })
                })
                .map(|f| [f[0], f[1], f[2]])
                .collect();
            assert!(faces.len() > 100, "{} missing shoulders", look.texture_key);
            let vertices: Vec<_> = faces
                .iter()
                .flatten()
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect();
            let lookup: std::collections::HashMap<_, _> =
                vertices.iter().enumerate().map(|(a, &b)| (b, a)).collect();
            let surface: Vec<_> = vertices
                .iter()
                .map(|&i| (Vec3::from_array(p[i]), Vec3::from_array(n[i]), j[i], w[i]))
                .collect();
            let faces: Vec<_> = faces
                .iter()
                .map(|f| {
                    let q = f.map(|i| Vec3::from_array(p[i]));
                    (
                        f.map(|i| lookup[&i]),
                        (q[1] - q[0]).cross(q[2] - q[0]).length(),
                    )
                })
                .collect();
            let mut world = World::new();
            let mut queue = CommandQueue::default();
            let mut commands = Commands::new(&mut queue, &world);
            let actor = commands
                .spawn((Transform::default(), Visibility::default()))
                .id();
            let mut cache = PeopleGpu::default();
            cache.2.lock().unwrap().insert((look.base, kind), prepared);
            let mut person = cache.spawn(
                look.base,
                kind,
                rig.clone(),
                actor,
                false,
                bicycle,
                None,
                &mut commands,
                &mut Assets::<Mesh>::default(),
                &mut Assets::<StandardMaterial>::default(),
                &mut Assets::<Image>::default(),
                &mut Assets::<SkinnedMeshInverseBindposes>::default(),
            );
            queue.apply(&mut world);
            let mut state =
                SystemState::<Query<&mut Transform, With<ModeledActor>>>::new(&mut world);
            let mut failures = 0;
            let mut worst = (1_f32, 1_f32, String::new());
            for (run, weight) in [(0., 0.), (0., 0.55), (0., 1.), (1., 0.55), (1., 1.)] {
                person.run_blend = run;
                for step in 0..32 {
                    let mut query = state.get_mut(&mut world).unwrap();
                    person.update(
                        step as f32 / 32. * std::f32::consts::TAU,
                        weight,
                        step as f32 * 0.23,
                        &mut query,
                    );
                    let palette: Vec<_> = person
                        .review_skin_palette(&query)
                        .iter()
                        .map(Mat4::from_cols_array)
                        .collect();
                    let posed: Vec<_> = surface
                        .iter()
                        .map(|&(p, n, j, w)| {
                            let matrix =
                                (0..4).fold(Mat4::ZERO, |m, s| m + palette[j[s] as usize] * w[s]);
                            (
                                matrix.transform_point3(p),
                                matrix
                                    .inverse()
                                    .transpose()
                                    .transform_vector3(n)
                                    .normalize(),
                            )
                        })
                        .collect();
                    for &(face, bind_area) in &faces {
                        let [a, b, c] = face.map(|i| posed[i]);
                        let geometric = (b.0 - a.0).cross(c.0 - a.0);
                        let agreement = geometric.normalize().dot((a.1 + b.1 + c.1).normalize());
                        let area = geometric.length() / bind_area;
                        if !agreement.is_finite() || agreement <= 0.05 || area <= 0.015 {
                            failures += 1;
                            if agreement < worst.0 || !agreement.is_finite() {
                                worst = (
                                    agreement,
                                    area,
                                    format!(
                                        "run={run}, weight={weight}, frame={step}, bind={:?}",
                                        face.map(|i| surface[i].0)
                                    ),
                                );
                            }
                        }
                        lowest_dot = lowest_dot.min(agreement);
                        smallest_area = smallest_area.min(area);
                    }
                    checked += 1;
                    face_samples+=faces.len() as u64;
                }
            }
            if failures > 0 {
                eprintln!(
                    "{} bicycle={bicycle}: {failures} rejected face samples; worst {worst:?}",
                    look.texture_key
                );
                rejected.push(look.texture_key);
            }
        }
        assert!(
            rejected.is_empty(),
            "production arm surface rejected: {rejected:?}"
        );
        eprintln!(
            "Arm surface checks: 75 final appearances, ordinary movement and three supported riders, {checked} production pose checks, {face_samples} triangle samples; minimum outward agreement={lowest_dot}, minimum area ratio={smallest_area}; sparse blend resolution>={lowest_resolution}, maximum palette={largest_palette}/256"
        );
    }
}
