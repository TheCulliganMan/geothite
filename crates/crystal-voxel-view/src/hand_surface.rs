//! One gentle subdivision pass on the supplied hand surfaces only.
//! Geometric welding crosses texture seams; attribute islands remain separate.
use crate::{mesh::SurfaceMeshData, open_people::PersonRig};
use bevy::prelude::*;
use std::collections::{BTreeMap, BTreeSet, HashMap};
fn key(p: [f32; 3]) -> [i32; 3] {
    p.map(|v| (v * 100_000.0).round() as i32)
}
pub(crate) fn smooth(
    rig: &PersonRig,
    mesh: &mut SurfaceMeshData,
    ids: &mut Vec<[u16; 4]>,
    weights: &mut Vec<[f32; 4]>,
) {
    let groups: BTreeSet<_> = rig.hand_joints.iter().flatten().copied().collect();
    if groups.is_empty() {
        return;
    }
    let selected: Vec<_> = ids
        .iter()
        .zip(weights.iter())
        .map(|(j, w)| {
            (0..4)
                .filter(|&i| groups.contains(&(j[i] as usize)))
                .map(|i| w[i])
                .sum::<f32>()
                > 0.85
        })
        .collect();
    smooth_selection(mesh,ids,weights,&selected);
    // Pinning the wrist geometry must not retain a stale normal on the hand
    // side of that boundary. Shade the refined surface across geometric UV
    // welds, while leaving every non-hand vertex's normal untouched.
    let mut normals = BTreeMap::<[i32; 3], Vec3>::new();
    for face in mesh.indices.chunks_exact(3) {
        let p = [face[0], face[1], face[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
        let n = (p[1]-p[0]).cross(p[2]-p[0]).normalize_or_zero();
        for corner in 0..3 {
            let a = (p[(corner+1)%3]-p[corner]).normalize_or_zero();
            let b = (p[(corner+2)%3]-p[corner]).normalize_or_zero();
            *normals.entry(key(p[corner].to_array())).or_default() += n * a.dot(b).clamp(-1.0,1.0).acos();
        }
    }
    for i in 0..mesh.positions.len() {
        let hand = (0..4).filter(|&slot| groups.contains(&(ids[i][slot] as usize)))
            .map(|slot| weights[i][slot]).sum::<f32>() > 0.85;
        if hand {
            mesh.normals[i] = normals[&key(mesh.positions[i])].normalize_or_zero().to_array();
        }
    }
}

/// Round an isolated source surface while pinning its boundaries. Attribute
/// islands and the original skin palette remain separate from geometric welds.
pub(crate) fn smooth_selection(
    mesh: &mut SurfaceMeshData,
    ids: &mut Vec<[u16;4]>,
    weights: &mut Vec<[f32;4]>,
    selected: &[bool],
) {
    refine_selection(mesh, ids, weights, selected, false);
}

/// Split neighboring triangles along a refined boundary too. Otherwise a
/// separately weighted edge midpoint can pull away from an unsplit triangle
/// during skinning, even when the bind-pose surface appears joined.
pub(crate) fn smooth_selection_joined(
    mesh: &mut SurfaceMeshData,
    ids: &mut Vec<[u16;4]>,
    weights: &mut Vec<[f32;4]>,
    selected: &[bool],
) {
    refine_selection(mesh, ids, weights, selected, true);
}

fn refine_selection(
    mesh: &mut SurfaceMeshData,
    ids: &mut Vec<[u16;4]>,
    weights: &mut Vec<[f32;4]>,
    selected: &[bool],
    join_boundary: bool,
) {
    let old = mesh.indices.clone();
    let original = mesh.positions.clone();
    let mut adjacent = BTreeMap::<[i32; 3], BTreeSet<[i32; 3]>>::new();
    let mut pinned = BTreeSet::new();
    let mut points = BTreeMap::new();
    let mut edge_faces = BTreeMap::<([i32; 3], [i32; 3]), Vec<[i32; 3]>>::new();
    for face in old.chunks_exact(3) {
        let face = [face[0] as usize, face[1] as usize, face[2] as usize];
        let keys = face.map(|i| key(original[i]));
        if !face.iter().all(|&i| selected[i]) {
            pinned.extend(keys);
            continue;
        }
        for i in 0..3 {
            points.insert(keys[i], Vec3::from_array(original[face[i]]));
            adjacent
                .entry(keys[i])
                .or_default()
                .extend([keys[(i + 1) % 3], keys[(i + 2) % 3]]);
            let a = keys[i];
            let b = keys[(i + 1) % 3];
            edge_faces
                .entry((a.min(b), a.max(b)))
                .or_default()
                .push(keys[(i + 2) % 3]);
        }
    }
    for (&(a, b), faces) in &edge_faces {
        if faces.len() != 2 {
            pinned.extend([a, b]);
        }
    }
    let mut rounded = BTreeMap::new();
    for (&k, neighbors) in &adjacent {
        let p = points[&k];
        let n = neighbors.len();
        let target = if pinned.contains(&k) || n < 3 {
            p
        } else {
            let beta = if n == 3 {
                3.0 / 16.0
            } else {
                3.0 / (8.0 * n as f32)
            };
            let loop_point = p * (1.0 - n as f32 * beta)
                + neighbors.iter().map(|q| points[q] * beta).sum::<Vec3>();
            // Retain most of the authored grip and finger length.
            p.lerp(loop_point, 0.55)
        };
        rounded.insert(k, target);
    }
    for (i, p) in mesh.positions.iter_mut().enumerate() {
        if selected[i] {
            if let Some(q) = rounded.get(&key(*p)) {
                *p = q.to_array();
            }
        }
    }
    let mut midpoints = HashMap::new();
    mesh.indices.clear();
    for face in old.chunks_exact(3) {
        let [a, b, c] = [face[0], face[1], face[2]];
        let whole = [a, b, c].iter().all(|&i| selected[i as usize]);
        let split = [(a, b), (b, c), (c, a)].map(|(a, b)| {
            let ka = key(original[a as usize]);
            let kb = key(original[b as usize]);
            whole || (join_boundary && edge_faces.contains_key(&(ka.min(kb), ka.max(kb))))
        });
        if !split.iter().any(|&split| split) {
            mesh.indices.extend_from_slice(face);
            continue;
        }
        let mut midpoint = |a: u32, b: u32| -> u32 {
            *midpoints.entry((a.min(b), a.max(b))).or_insert_with(|| {
                let (a, b) = (a as usize, b as usize);
                let ka = key(original[a]);
                let kb = key(original[b]);
                let edge = &edge_faces[&(ka.min(kb), ka.max(kb))];
                let straight =
                    (Vec3::from_array(original[a]) + Vec3::from_array(original[b])) * 0.5;
                let p = if edge.len() == 2 && !pinned.contains(&ka) && !pinned.contains(&kb) {
                    let curve = (points[&ka] + points[&kb]) * 0.375
                        + (points[&edge[0]] + points[&edge[1]]) * 0.125;
                    straight.lerp(curve, 0.55)
                } else {
                    straight
                };
                let index = mesh.positions.len() as u32;
                mesh.positions.push(p.to_array());
                mesh.normals.push(
                    (Vec3::from_array(mesh.normals[a]) + Vec3::from_array(mesh.normals[b]))
                        .normalize_or_zero()
                        .to_array(),
                );
                mesh.uvs.push(
                    Vec2::from_array(mesh.uvs[a])
                        .lerp(Vec2::from_array(mesh.uvs[b]), 0.5)
                        .to_array(),
                );
                mesh.colors.push(
                    Vec4::from_array(mesh.colors[a])
                        .lerp(Vec4::from_array(mesh.colors[b]), 0.5)
                        .to_array(),
                );
                let mut blend = BTreeMap::<u16, f32>::new();
                for i in [a, b] {
                    for slot in 0..4 {
                        *blend.entry(ids[i][slot]).or_default() += weights[i][slot] * 0.5;
                    }
                }
                let mut blend: Vec<_> = blend.into_iter().collect();
                blend.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
                blend.truncate(4);
                let total: f32 = blend.iter().map(|p| p.1).sum();
                ids.push(std::array::from_fn(|i| blend.get(i).map_or(0, |p| p.0)));
                weights.push(std::array::from_fn(|i| {
                    blend.get(i).map_or(0.0, |p| p.1 / total)
                }));
                index
            })
        };
        let corners = [a, b, c];
        let mids: [Option<u32>; 3] = std::array::from_fn(|edge| {
            split[edge].then(|| midpoint(corners[edge], corners[(edge + 1) % 3]))
        });
        match split.iter().filter(|&&split| split).count() {
            1 => {
                let edge = split.iter().position(|&split| split).unwrap();
                let [a, b, c] = std::array::from_fn(|i| corners[(edge + i) % 3]);
                let ab = mids[edge].unwrap();
                mesh.indices.extend([a, ab, c, ab, b, c]);
            }
            2 => {
                let edge = (split.iter().position(|&split| !split).unwrap() + 1) % 3;
                let [a, b, c] = std::array::from_fn(|i| corners[(edge + i) % 3]);
                let ab = mids[edge].unwrap();
                let bc = mids[(edge + 1) % 3].unwrap();
                mesh.indices.extend([b, bc, ab, ab, bc, c, a, ab, c]);
            }
            3 => {
                let [ab, bc, ca] = mids.map(Option::unwrap);
                mesh.indices.extend([a, ab, ca, ab, b, bc, ca, bc, c, ab, bc, ca]);
            }
            _ => unreachable!(),
        }
    }
    let mut normals = HashMap::<[i32; 3], Vec3>::new();
    for face in mesh.indices.chunks_exact(3) {
        let p = [face[0], face[1], face[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
        let n = (p[1] - p[0]).cross(p[2] - p[0]);
        for &i in face {
            *normals.entry(key(mesh.positions[i as usize])).or_default() += n;
        }
    }
    for i in 0..mesh.positions.len() {
        if (i >= selected.len() || selected[i])
            && !pinned.contains(&key(original.get(i).copied().unwrap_or(mesh.positions[i])))
        {
            if let Some(n) = normals.get(&key(mesh.positions[i])) {
                if n.length_squared() > 1e-12 {
                    mesh.normals[i] = n.normalize().to_array();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn joined_refinement_keeps_uv_boundary_watertight_when_skinning_bends() {
        // Two triangles share a diagonal through separate UV islands and
        // different joint influences at its ends. Only the first is rounded.
        let mut mesh = SurfaceMeshData {
            positions: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.],
                [1.,0.,0.], [1.,1.,0.], [0.,1.,0.]],
            normals: vec![[0.,0.,1.]; 6],
            colors: vec![[1.;4]; 6],
            uvs: vec![[0.,0.], [1.,0.], [0.,1.], [0.2,0.], [1.,1.], [0.,0.2]],
            indices: vec![0,1,2,3,4,5],
            ..Default::default()
        };
        let mut ids = vec![[0,1,0,0];6];
        let mut weights = vec![[1.,0.,0.,0.], [0.,1.,0.,0.], [1.,0.,0.,0.],
            [0.,1.,0.,0.], [0.,1.,0.,0.], [1.,0.,0.,0.]];
        smooth_selection_joined(&mut mesh, &mut ids, &mut weights,
            &[true,true,true,false,false,false]);
        let mut edges = BTreeMap::new();
        for face in mesh.indices.chunks_exact(3) {
            let p = [face[0],face[1],face[2]]
                .map(|i| Vec3::from_array(mesh.positions[i as usize]));
            assert!((p[1]-p[0]).cross(p[2]-p[0]).z > 0., "reversed or degenerate face");
            for (a,b) in [(0,1),(1,2),(2,0)] {
                let a=key(p[a].to_array()); let b=key(p[b].to_array());
                *edges.entry((a.min(b),a.max(b))).or_insert(0) += 1;
            }
        }
        for ((a,b),count) in edges {
            if count == 1 {
                assert!((a[0]==b[0] && [0,100_000].contains(&a[0]))
                    || (a[1]==b[1] && [0,100_000].contains(&a[1])),
                    "open interior edge across refined boundary: {a:?} {b:?}");
            } else { assert_eq!(count,2,"non-manifold refined surface"); }
        }
        for step in 0..16 {
            let moving = Mat4::from_rotation_y(step as f32 * 0.1)
                * Mat4::from_translation(Vec3::new(0.2,0.1,0.));
            let mut welded = BTreeMap::new();
            for (i,&p) in mesh.positions.iter().enumerate() {
                let p3=Vec3::from_array(p);
                let deformed=(0..4).map(|slot| {
                    if ids[i][slot]==0 { p3*weights[i][slot] }
                    else { moving.transform_point3(p3)*weights[i][slot] }
                }).sum::<Vec3>();
                if let Some(previous) = welded.insert(key(p),deformed) {
                    assert!(previous.distance(deformed)<0.000001,
                        "UV copies separate during skinning at step {step}");
                }
            }
        }
    }

    #[test]
    #[ignore = "requires shared Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_hands_are_smoother_without_changing_body_attributes_or_wrist_boundaries() {
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        for name in ["human-male", "human-female", "skater-male", "skater-female"] {
            let rig = PersonRig::parse(
                &std::fs::read(root.join(format!("people/kenney/{name}.glb"))).unwrap(),
            )
            .unwrap();
            assert!(rig.hand_joints.iter().all(|group| group.len() >= 4));
            let groups: BTreeSet<_> = rig.hand_joints.iter().flatten().copied().collect();
            let selected: Vec<_> = rig
                .ids
                .iter()
                .zip(&rig.weights)
                .map(|(ids, w)| {
                    (0..4)
                        .filter(|&i| groups.contains(&(ids[i] as usize)))
                        .map(|i| w[i])
                        .sum::<f32>()
                        > 0.85
                })
                .collect();
            let mut mesh = rig.mesh.clone();
            let mut ids = rig.ids.clone();
            let mut weights = rig.weights.clone();
            smooth(&rig, &mut mesh, &mut ids, &mut weights);
            let original = rig.mesh.positions.len();
            assert!(
                mesh.positions.len() > original + 80,
                "{name} missing hand subdivision"
            );
            assert_eq!(mesh.positions.len(), ids.len());
            assert_eq!(ids.len(), weights.len());
            assert_eq!(&ids[..original], rig.ids.as_slice());
            assert_eq!(&weights[..original], rig.weights.as_slice());
            assert_eq!(&mesh.uvs[..original], rig.mesh.uvs.as_slice());
            assert_eq!(&mesh.colors[..original], rig.mesh.colors.as_slice());
            let mut moved = 0;
            for i in 0..original {
                if !selected[i] {
                    assert_eq!(mesh.positions[i], rig.mesh.positions[i]);
                    assert_eq!(mesh.normals[i], rig.mesh.normals[i]);
                } else {
                    let distance = Vec3::from_array(mesh.positions[i])
                        .distance(Vec3::from_array(rig.mesh.positions[i]));
                    assert!(
                        distance < 0.065,
                        "{name} hand shape changed too far: {distance}"
                    );
                    moved += usize::from(distance > 1e-5);
                }
            }
            assert!(moved > 15, "{name} only subdivided straight facets");
            for face in rig.mesh.indices.chunks_exact(3) {
                if !face.iter().all(|&i| selected[i as usize]) {
                    assert!(
                        mesh.indices.chunks_exact(3).any(|f| f == face),
                        "{name} non-hand triangle changed"
                    );
                    for &i in face {
                        assert_eq!(
                            mesh.positions[i as usize], rig.mesh.positions[i as usize],
                            "{name} wrist boundary moved"
                        );
                    }
                }
            }
            let bind_min = rig
                .mesh
                .positions
                .iter()
                .map(|&p| Vec3::from_array(p))
                .fold(Vec3::splat(f32::INFINITY), Vec3::min);
            let bind_max = rig
                .mesh
                .positions
                .iter()
                .map(|&p| Vec3::from_array(p))
                .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max);
            for (p, n) in mesh.positions.iter().zip(&mesh.normals) {
                let p = Vec3::from_array(*p);
                let n = Vec3::from_array(*n);
                assert!(
                    p.is_finite() && n.is_finite() && (n.length() - 1.0).abs() < 1e-4,
                    "{name} invalid surface"
                );
                assert!(
                    p.cmpge(bind_min - Vec3::splat(0.001)).all()
                        && p.cmple(bind_max + Vec3::splat(0.001)).all()
                );
            }
            for face in mesh.indices.chunks_exact(3) {
                let p = [face[0], face[1], face[2]]
                    .map(|i| Vec3::from_array(mesh.positions[i as usize]));
                let n = face
                    .iter()
                    .map(|&i| Vec3::from_array(mesh.normals[i as usize]))
                    .sum::<Vec3>();
                assert!(
                    (p[1] - p[0]).cross(p[2] - p[0]).dot(n) >= -1e-6,
                    "{name} reversed hand surface"
                );
            }
            for w in &weights {
                assert!(
                    w.iter().all(|v| v.is_finite() && *v >= 0.0)
                        && (w.iter().sum::<f32>() - 1.0).abs() < 1e-5
                );
            }
            fn world(i: usize, pose: &[Transform], parents: &[Option<usize>]) -> Mat4 {
                parents[i]
                    .map(|p| world(p, pose, parents))
                    .unwrap_or(Mat4::IDENTITY)
                    * pose[i].to_matrix()
            }
            let mut pose = rig.nodes.clone();
            for clip in ["Idle", "Run", "Jump"] {
                for step in 0..=60 {
                    rig.sample(clip, rig.duration(clip) * step as f32 / 60.0, &mut pose);
                    let inverse = world(rig.mesh_node, &pose, &rig.parents).inverse();
                    let palette: Vec<_> = rig
                        .skin_nodes
                        .iter()
                        .zip(&rig.inverse_binds)
                        .map(|(&node, bind)| inverse * world(node, &pose, &rig.parents) * *bind)
                        .collect();
                    for ((&p, joints), weights) in mesh.positions.iter().zip(&ids).zip(&weights) {
                        let p = (0..4)
                            .map(|i| {
                                palette[joints[i] as usize].transform_point3(Vec3::from_array(p))
                                    * weights[i]
                            })
                            .sum::<Vec3>();
                        assert!(p.is_finite());
                        assert!(
                            p.cmpge(rig.bounds.0 - Vec3::splat(0.002)).all()
                                && p.cmple(rig.bounds.1 + Vec3::splat(0.002)).all(),
                            "{name} hand escapes {clip} envelope at {step}/60"
                        );
                    }
                }
            }
            let mut repeated = rig.mesh.clone();
            let mut repeated_ids = rig.ids.clone();
            let mut repeated_weights = rig.weights.clone();
            smooth(
                &rig,
                &mut repeated,
                &mut repeated_ids,
                &mut repeated_weights,
            );
            assert_eq!(mesh.positions, repeated.positions);
            assert_eq!(mesh.normals, repeated.normals);
            assert_eq!(mesh.indices, repeated.indices);
        }
    }
}
