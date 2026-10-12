//! Keep glove material on complete hand faces, across the supplied UV islands.
use crate::{mesh::SurfaceMeshData, npc_appearance::Look, open_people::PersonRig};
use std::collections::{BTreeSet, HashMap};

pub(crate) fn adapt(
    rig: &PersonRig,
    mesh: &mut SurfaceMeshData,
    joints: &mut Vec<[u16; 4]>,
    weights: &mut Vec<[f32; 4]>,
    look: Look,
) {
    if !look
        .accessories
        .split(',')
        .any(|item| item == "whitegloves")
    {
        return;
    }
    let hand_joints: BTreeSet<_> = rig.hand_joints.iter().flatten().copied().collect();
    let selected: Vec<_> = joints
        .iter()
        .zip(weights.iter())
        .map(|(j, w)| {
            (0..4)
                .filter(|&i| hand_joints.contains(&(j[i] as usize)))
                .map(|i| w[i])
                .sum::<f32>()
                > 0.85
        })
        .collect();
    let original = std::mem::take(&mut mesh.indices);
    let mut copies = HashMap::new();
    for face in original.chunks_exact(3) {
        if !face.iter().all(|&i| selected[i as usize]) {
            mesh.indices.extend_from_slice(face);
            continue;
        }
        for &index in face {
            let copy = *copies.entry(index).or_insert_with(|| {
                let i = index as usize;
                let copy = mesh.positions.len() as u32;
                mesh.positions.push(mesh.positions[i]);
                mesh.normals.push(mesh.normals[i]);
                mesh.colors.push(mesh.colors[i]);
                // The authored white-glove island. Palm and finger UVs in
                // the supplied rig otherwise reach the bare-arm strip.
                mesh.uvs.push([0.87, 0.69]);
                joints.push(joints[i]);
                weights.push(weights[i]);
                copy
            });
            mesh.indices.push(copy);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires the supplied character rig"]
    fn white_glove_faces_keep_geometry_skin_and_nonhand_mapping() {
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let rig =
            PersonRig::parse(&std::fs::read(root.join("people/kenney/human-male.glb")).unwrap())
                .unwrap();
        let mut mesh = rig.mesh.clone();
        let mut joints = rig.ids.clone();
        let mut weights = rig.weights.clone();
        let before = mesh.clone();
        let look = crate::npc_appearance::look(
            crate::new_bark_actors::character_meshes::CharacterKind::Will,
        );
        adapt(&rig, &mut mesh, &mut joints, &mut weights, look);
        assert_eq!(mesh.indices.len(), before.indices.len());
        let hands: BTreeSet<_> = rig.hand_joints.iter().flatten().copied().collect();
        let mut glove_faces = 0;
        for (old, new) in before
            .indices
            .chunks_exact(3)
            .zip(mesh.indices.chunks_exact(3))
        {
            let glove = old.iter().all(|&i| {
                (0..4)
                    .filter(|&k| hands.contains(&(rig.ids[i as usize][k] as usize)))
                    .map(|k| rig.weights[i as usize][k])
                    .sum::<f32>()
                    > 0.85
            });
            glove_faces += usize::from(glove);
            for (&a, &b) in old.iter().zip(new) {
                let (a, b) = (a as usize, b as usize);
                assert_eq!(before.positions[a], mesh.positions[b]);
                assert_eq!(before.normals[a], mesh.normals[b]);
                assert_eq!(before.colors[a], mesh.colors[b]);
                assert_eq!(rig.ids[a], joints[b]);
                assert_eq!(rig.weights[a], weights[b]);
                assert_eq!(
                    mesh.uvs[b],
                    if glove { [0.87, 0.69] } else { before.uvs[a] }
                );
            }
        }
        assert!(
            glove_faces > 30,
            "supplied hands did not receive glove material"
        );
    }
}
