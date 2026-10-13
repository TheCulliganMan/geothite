//! Keep hand material on complete faces, across the supplied UV islands.
use crate::{mesh::SurfaceMeshData, npc_appearance::Look, open_people::PersonRig};
use std::collections::{BTreeSet, HashMap};

pub(crate) fn adapt(
    rig: &PersonRig,
    mesh: &mut SurfaceMeshData,
    joints: &mut Vec<[u16; 4]>,
    weights: &mut Vec<[f32; 4]>,
    look: Look,
) {
    let gloves = look
        .accessories
        .split(',')
        .any(|item| item == "whitegloves");
    // Dark and fingerless gloves carry their own authored hand painting.
    // A plain bare-skin sample must not erase those outfit identities.
    if !gloves && look.accessories.split(',').any(|item| item == "gloves") {
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
        // Bare wrists share the plain arm material across the adjoining face.
        // Restricting this to wholly hand-weighted faces leaves a painted
        // crease strip between the remapped palm and the forearm. Gloves keep
        // their authored cuff boundary instead of extending onto the arm.
        let plain_face = if gloves {
            face.iter().all(|&i| selected[i as usize])
        } else {
            face.iter().any(|&i| selected[i as usize])
        };
        if !plain_face {
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
                // Match the identity's authored glove or plain arm island.
                // The supplied palm UVs stretch painted crease bands across
                // the reshaped fingers; geometry supplies the pad shading.
                mesh.uvs.push(if gloves { [0.87, 0.69] } else { [0.90, 0.31] });
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
        material_preserves_surface(crate::new_bark_actors::character_meshes::CharacterKind::Will);
    }

    #[test]
    #[ignore = "requires the supplied character rig"]
    fn bare_hand_faces_keep_geometry_skin_and_nonhand_mapping() {
        use crate::new_bark_actors::character_meshes::CharacterKind;
        for kind in [CharacterKind::Trainer, CharacterKind::TrainerFemale, CharacterKind::Nurse] {
            material_preserves_surface(kind);
        }
    }

    #[test]
    #[ignore = "requires supplied character rigs"]
    fn authored_dark_gloves_keep_their_complete_hand_painting() {
        use crate::new_bark_actors::character_meshes::CharacterKind;
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        for kind in [CharacterKind::Biker, CharacterKind::Clair, CharacterKind::Janine,
            CharacterKind::Rocket, CharacterKind::RocketGirl] {
            let look = crate::npc_appearance::look(kind);
            let rig = PersonRig::parse(&std::fs::read(root.join(
                format!("people/kenney/{}.glb", look.base))).unwrap()).unwrap();
            let mut mesh = rig.mesh.clone();
            let mut joints = rig.ids.clone();
            let mut weights = rig.weights.clone();
            adapt(&rig, &mut mesh, &mut joints, &mut weights, look);
            assert_eq!(mesh.indices, rig.mesh.indices);
            assert_eq!(mesh.positions, rig.mesh.positions);
            assert_eq!(mesh.normals, rig.mesh.normals);
            assert_eq!(mesh.uvs, rig.mesh.uvs);
            assert_eq!(mesh.colors, rig.mesh.colors);
            assert_eq!(joints, rig.ids);
            assert_eq!(weights, rig.weights);
        }
    }

    fn material_preserves_surface(kind: crate::new_bark_actors::character_meshes::CharacterKind) {
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let look = crate::npc_appearance::look(kind);
        let rig =
            PersonRig::parse(&std::fs::read(root.join(format!("people/kenney/{}.glb", look.base))).unwrap())
                .unwrap();
        let mut mesh = rig.mesh.clone();
        let mut joints = rig.ids.clone();
        let mut weights = rig.weights.clone();
        let before = mesh.clone();
        let uv = if look.accessories.split(',').any(|a| a == "whitegloves") {
            [0.87, 0.69]
        } else { [0.90, 0.31] };
        adapt(&rig, &mut mesh, &mut joints, &mut weights, look);
        assert_eq!(mesh.indices.len(), before.indices.len());
        let hands: BTreeSet<_> = rig.hand_joints.iter().flatten().copied().collect();
        let mut glove_faces = 0;
        let mut wrist_faces = 0;
        for (old, new) in before
            .indices
            .chunks_exact(3)
            .zip(mesh.indices.chunks_exact(3))
        {
            let selected = |&i: &u32| {
                (0..4)
                    .filter(|&k| hands.contains(&(rig.ids[i as usize][k] as usize)))
                    .map(|k| rig.weights[i as usize][k])
                    .sum::<f32>()
                    > 0.85
            };
            let glove = if look.accessories.split(',').any(|a| a == "whitegloves") {
                old.iter().all(selected)
            } else {
                old.iter().any(selected)
            };
            if glove && !old.iter().all(selected) {
                wrist_faces += 1;
            }
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
                    if glove { uv } else { before.uvs[a] }
                );
            }
        }
        assert!(
            glove_faces > 30,
            "supplied hands did not receive glove material"
        );
        if look.accessories.split(',').any(|a| a == "whitegloves") {
            assert_eq!(wrist_faces, 0, "glove cuff extended onto forearm");
        } else {
            assert!(wrist_faces > 0, "bare wrist retained painted crease faces");
        }
    }
}
