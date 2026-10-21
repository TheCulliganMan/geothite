//! Broaden the existing kimono arm fabric, retaining skin islands and seam rings.
use crate::{mesh::SurfaceMeshData, npc_appearance::Look};
use bevy::prelude::*;
use std::collections::HashSet;
fn key(p: [f32; 3]) -> [i32; 3] {
    p.map(|v| (v * 100_000.).round() as i32)
}
pub(crate) fn sculpt(
    mesh: &mut SurfaceMeshData,
    ids: &mut Vec<[u16; 4]>,
    weights: &mut Vec<[f32; 4]>,
    look: Look,
) {
    if !matches!(look.texture_key, "erika" | "falkner" | "kimono_girl") {
        return;
    }
    let selected: Vec<_> = mesh
        .positions
        .iter()
        .zip(&mesh.uvs)
        .map(|(p, uv)| {
            (0.40..1.34).contains(&p[0].abs())
                && (2.0..2.60).contains(&p[1])
                && uv[0] < 0.625
                && uv[1] > 0.60
        })
        .collect();
    // Weld texture duplicates for the purpose of pinning shared skin seams.
    let mut pinned = HashSet::new();
    for face in mesh.indices.chunks_exact(3) {
        if !face.iter().all(|&i| selected[i as usize]) {
            pinned.extend(face.iter().map(|&i| key(mesh.positions[i as usize])));
        }
    }
    let smooth = |v: f32| {
        let t = v.clamp(0., 1.);
        t * t * (3. - 2. * t)
    };
    for (i, p) in mesh.positions.iter_mut().enumerate() {
        if !selected[i] || pinned.contains(&key(*p)) {
            continue;
        }
        let x = p[0].abs();
        let fullness = smooth((x - 0.43) / 0.20) * smooth((1.30 - x) / 0.20);
        let arm_center = 2.36 - 0.10 * (x - 0.5);
        let lower = smooth((arm_center - p[1]) / 0.14);
        let (hang, depth) = if look.texture_key == "falkner" {
            (0.38, 0.18)
        } else {
            (0.29, 0.45)
        };
        p[1] -= hang * fullness * lower;
        p[2] = 0.025 + (p[2] - 0.025) * (1. + depth * fullness);
    }
    // Refine the coarse original arm rings into a rounded hanging surface.
    // The neighboring torso triangles must share the new sleeve edge points.
    // An unsplit edge only meets them in bind pose and opens during skinning.
    crate::hand_surface::smooth_selection_joined(mesh, ids, weights, &selected);
}

/// Hanging sleeves no longer have the source T-pose barrel's normals.
/// Finish their actual refined surface across geometric texture seams.
pub(crate) fn finish_normals(mesh: &mut SurfaceMeshData, look: Look) {
    if !matches!(look.texture_key,"erika"|"falkner"|"kimono_girl") {return;}
    let mut normals=std::collections::HashMap::<[i32;3],Vec3>::new();
    for face in mesh.indices.chunks_exact(3) {
        let p=[face[0],face[1],face[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
        let normal=(p[1]-p[0]).cross(p[2]-p[0]).normalize_or_zero();
        for corner in 0..3 {
            let a=(p[(corner+1)%3]-p[corner]).normalize_or_zero();
            let b=(p[(corner+2)%3]-p[corner]).normalize_or_zero();
            *normals.entry(key(p[corner].to_array())).or_default()+=normal*a.dot(b).clamp(-1.,1.).acos();
        }
    }
    for ((p,n),uv) in mesh.positions.iter().zip(&mut mesh.normals).zip(&mesh.uvs) {
        if (0.25..1.34).contains(&p[0].abs())&&(1.75..2.60).contains(&p[1])
            && uv[0]<0.625 && uv[1]>=0.475 {
            *n=normals[&key(*p)].normalize_or_zero().to_array();
        }
    }
}

#[cfg(any(test, all(feature = "people-review", not(target_arch = "wasm32"))))]
pub(crate) mod tests {
    use super::*;
    #[cfg_attr(test, test)]
    #[cfg_attr(test, ignore = "requires supplied male and female rigs in CRYSTAL_OPEN_MODEL_ROOT")]
    pub(crate) fn supplied_kimono_sleeves_preserve_skin_rig_and_unrelated_characters() {
        use crate::{
            new_bark_actors::character_meshes::CharacterKind::{Erika, Falkner, KimonoGirl, Mom},
            open_people::PersonRig,
        };
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        for kind in [Erika, Falkner, KimonoGirl] {
            let look = crate::npc_appearance::look(kind);
            let rig = PersonRig::parse(
                &std::fs::read(root.join(format!("people/kenney/{}.glb", look.base))).unwrap(),
            )
            .unwrap();
            let mut mesh = rig.mesh.clone();
            let mut ids = rig.ids.clone();
            let mut weights = rig.weights.clone();
            sculpt(
                &mut mesh,
                &mut ids,
                &mut weights,
                crate::npc_appearance::look(Mom),
            );
            assert_eq!(mesh.positions, rig.mesh.positions);
            assert_eq!(mesh.normals, rig.mesh.normals);
            assert_eq!(mesh.uvs, rig.mesh.uvs);
            assert_eq!(mesh.indices, rig.mesh.indices);
            assert_eq!(ids, rig.ids);
            assert_eq!(weights, rig.weights);
            sculpt(&mut mesh, &mut ids, &mut weights, look);
            let old = rig.mesh.positions.len();
            let mut changed = [0; 2];
            let mut drop = 0_f32;
            assert!(mesh.positions.len() > old + 100);
            for i in 0..old {
                assert_eq!(mesh.uvs[i], rig.mesh.uvs[i]);
                assert_eq!(ids[i], rig.ids[i]);
                assert_eq!(weights[i], rig.weights[i]);
                if mesh.positions[i] != rig.mesh.positions[i] {
                    let p = rig.mesh.positions[i];
                    let uv = rig.mesh.uvs[i];
                    assert!(
                        p[0].abs() > 0.4 && uv[0] < 0.625 && uv[1] > 0.60,
                        "skin or torso changed"
                    );
                    changed[usize::from(p[0] > 0.)] += 1;
                    drop = drop.max(p[1] - mesh.positions[i][1]);
                }
            }
            assert!(changed.iter().all(|&n| n > 40));
            assert!(drop > 0.18);
            let mut edges = std::collections::HashMap::new();
            for face in mesh.indices.chunks_exact(3) {
                for edge in 0..3 {
                    let a = key(mesh.positions[face[edge] as usize]);
                    let b = key(mesh.positions[face[(edge+1)%3] as usize]);
                    *edges.entry((a.min(b), a.max(b))).or_insert(0) += 1;
                }
            }
            for ((a,b), count) in edges {
                if [a,b].iter().all(|p| (190_000..260_000).contains(&p[1]) && p[0].abs() < 134_000) {
                    assert_eq!(count, 2, "{} open sleeve/torso edge {a:?}/{b:?}", look.texture_key);
                }
            }
            for i in 0..mesh.positions.len() {
                assert!(Vec3::from_array(mesh.positions[i]).is_finite());
                let n = Vec3::from_array(mesh.normals[i]);
                assert!(n.is_finite() && (n.length() - 1.).abs() < 1e-4);
                assert!(weights[i].iter().all(|&w| w.is_finite() && w >= 0.));
                assert!((weights[i].iter().sum::<f32>() - 1.).abs() < 1e-5);
                assert!(ids[i].iter().all(|&j| (j as usize) < rig.skin_nodes.len()));
            }
            fn world(i: usize, pose: &[Transform], parents: &[Option<usize>]) -> Mat4 {
                parents[i]
                    .map(|p| world(p, pose, parents))
                    .unwrap_or(Mat4::IDENTITY)
                    * pose[i].to_matrix()
            }
            let mut pose = rig.nodes.clone();
            for step in 0..8 {
                rig.sample("Run", rig.duration("Run") * step as f32 / 8., &mut pose);
                let inverse = world(rig.mesh_node, &pose, &rig.parents).inverse();
                let skin: Vec<_> = rig
                    .skin_nodes
                    .iter()
                    .zip(&rig.inverse_binds)
                    .map(|(&node, bind)| inverse * world(node, &pose, &rig.parents) * *bind)
                    .collect();
                for i in 0..mesh.positions.len() {
                    let p = Vec3::from_array(mesh.positions[i]);
                    let moved = (0..4)
                        .map(|slot| {
                            skin[ids[i][slot] as usize].transform_point3(p) * weights[i][slot]
                        })
                        .sum::<Vec3>();
                    assert!(moved.is_finite());
                    assert!(
                        moved.cmpge(rig.bounds.0 - Vec3::splat(0.4)).all()
                            && moved.cmple(rig.bounds.1 + Vec3::splat(0.4)).all()
                    );
                }
            }
        }
    }
}
