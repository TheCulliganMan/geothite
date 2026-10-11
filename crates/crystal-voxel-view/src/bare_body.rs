//! Exposed skin must use the skin island, never the source shirt folds.
use crate::{mesh::SurfaceMeshData, npc_appearance::Look};

pub(crate) fn sculpt(
    mesh: &mut SurfaceMeshData,
    joints: &mut Vec<[u16; 4]>,
    weights: &mut Vec<[f32; 4]>,
    look: Look,
) {
    if !matches!(look.texture_key, "bruno" | "chuck" | "swimmer_guy" | "swimmer_girl") {
        return;
    }
    // The hands and neck already have their own skin mapping. Select the
    // source sleeve island only; wrist and shoulder boundaries remain pinned.
    let arms: Vec<_> = mesh.positions.iter().zip(&mesh.uvs).map(|(&p, &uv)| {
        p[0].abs() > 0.4 && (2.0..2.55).contains(&p[1])
            && uv[0] < 0.625 && uv[1] > 0.60
    }).collect();
    crate::hand_surface::smooth_selection(mesh, joints, weights, &arms);
    let bare_torso = look.texture_key != "swimmer_girl";
    for face in mesh.indices.chunks_exact_mut(3) {
        let ids = [face[0] as usize, face[1] as usize, face[2] as usize];
        let arm = ids.iter().map(|&i| mesh.positions[i][0].abs()).sum::<f32>() / 3.0 > 0.25
            && ids.iter().all(|&i| {
            let p = mesh.positions[i]; let uv = mesh.uvs[i];
            (2.0..2.57).contains(&p[1])
                && uv[0] < 0.625 && uv[1] > 0.60
        });
        let torso = ids.iter().all(|&i| {
            let p = mesh.positions[i]; let uv = mesh.uvs[i];
            (1.25..2.57).contains(&p[1])
                && uv[0] < 0.625 && uv[1] > 0.46
        });
        if !arm && !torso { continue; }
        for (index, &i) in face.iter_mut().zip(&ids) {
            let p = mesh.positions[i];
            *index = mesh.positions.len() as u32;
            mesh.positions.push(p);
            mesh.normals.push(mesh.normals[i]);
            mesh.colors.push(mesh.colors[i]);
            // Stay inside the inspected plain skin patch at the right of
            // the atlas, clear of shoes, fingers, and the lower garment.
            mesh.uvs.push(if torso && !arm && !bare_torso {
                crate::npc_appearance::swimsuit_uv(p)
            } else {
                [0.89 + p[2] * 0.025,
                (0.30 + (2.50 - p[1]) * 0.06).clamp(0.27, 0.40)]
            });
            joints.push(joints[i]); weights.push(weights[i]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{new_bark_actors::character_meshes::CharacterKind, open_people::PersonRig};

    #[test]
    #[ignore = "requires supplied female rig and swimmer painting"]
    fn supplied_swimsuit_exposes_legs_and_keeps_blue_fabric_continuous() {
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let rig = PersonRig::parse(&std::fs::read(root.join("people/kenney/human-female.glb")).unwrap()).unwrap();
        let look = crate::npc_appearance::look(CharacterKind::SwimmerGirl);
        let painting = crate::npc_textures::decode(&std::fs::read(root.join("people/npc-textures/swimmer_girl.png")).unwrap()).unwrap();
        let mut mesh = rig.mesh.clone(); let mut joints = rig.ids.clone(); let mut weights = rig.weights.clone();
        crate::npc_appearance::shorts_body(&rig, &mut mesh, &mut joints, &mut weights, look);
        sculpt(&mut mesh, &mut joints, &mut weights, look);
        let mut legs = [0;2]; let mut fabric = [0;2];
        for &i in &mesh.indices {
            let i=i as usize;let p=mesh.positions[i];let [u,v]=mesh.uvs[i];
            assert!(p.iter().all(|x|x.is_finite()));
            assert!((weights[i].iter().sum::<f32>()-1.0).abs()<0.001);
            assert!(joints[i].iter().all(|&j|(j as usize)<rig.inverse_binds.len()));
            if p[1]<1.2 && p[1]>0.3 && u>0.87 && v<0.45 { legs[usize::from(p[0]>0.0)]+=1; }
            if (0.79..0.85).contains(&u) && (0.83..0.88).contains(&v) {
                fabric[usize::from(p[1]>1.45)]+=1;
                let x=(u*painting.size.0 as f32) as u32;let y=(v*painting.size.1 as f32) as u32;
                let rgb=&painting.rgba[((y*painting.size.0+x)*4) as usize..][..3];
                assert!(rgb[2]>rgb[0] && rgb[2]>rgb[1],"fabric samples non-blue {rgb:?}");
                assert!(p[1]>1.0,"trouser fabric left below swimsuit: {p:?}");
            }
        }
        assert!(legs.iter().all(|&n|n>100),"missing exposed legs {legs:?}");
        assert!(fabric.iter().all(|&n|n>20),"missing connected upper/lower fabric {fabric:?}");
        // The fitted edge must close to the original skin surface with the
        // same influences; otherwise running separates the two boundaries.
        let mut joined=0;
        for face in mesh.indices.chunks_exact(3) {
            if !face.iter().all(|&i|mesh.normals[i as usize][1] < -0.8) {continue;}
            for &i in face {
                let i=i as usize;
                if mesh.uvs[i][0]<0.79 || mesh.uvs[i][0]>0.85 {continue;}
                if (0..mesh.positions.len()).any(|j|mesh.positions[j]==mesh.positions[i]
                    && joints[j]==joints[i] && weights[j]==weights[i]
                    && mesh.uvs[j][0]>0.87 && mesh.uvs[j][1]<0.45) {joined+=1;}
            }
        }
        assert!(joined>20,"swimsuit lip does not share skin anchors: {joined}");
    }

    #[test]
    #[ignore = "requires supplied rigs and paintings in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_bare_arms_and_torsos_use_skin_without_touching_faces_or_trousers() {
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        for kind in [CharacterKind::Bruno, CharacterKind::Chuck,
            CharacterKind::SwimmerGuy, CharacterKind::SwimmerGirl] {
            let look = crate::npc_appearance::look(kind);
            let rig = PersonRig::parse(&std::fs::read(root.join(format!("people/kenney/{}.glb", look.base))).unwrap()).unwrap();
            let painting = crate::npc_textures::decode(&std::fs::read(root.join(format!("people/npc-textures/{}.png", look.texture_key))).unwrap()).unwrap();
            let mut mesh = rig.mesh.clone(); let mut joints = rig.ids.clone(); let mut weights = rig.weights.clone();
            sculpt(&mut mesh, &mut joints, &mut weights, look);
            let mut arms = [0; 2]; let mut torsos = 0;
            for &i in &mesh.indices {
                let i = i as usize; let p = mesh.positions[i]; let [u,v] = mesh.uvs[i];
                if (0.87..0.92).contains(&u) && (0.27..=0.40).contains(&v)
                    && p[1] > 1.25 && p[1] < 2.56 {
                    if p[0].abs() > 0.4 { arms[usize::from(p[0] > 0.0)] += 1; }
                    else { torsos += 1; }
                    let x = (u * painting.size.0 as f32) as u32;
                    let y = (v * painting.size.1 as f32) as u32;
                    let rgb = &painting.rgba[((y * painting.size.0 + x) * 4) as usize..][..3];
                    assert!(rgb[0] > 160 && rgb[0] > rgb[1] && rgb[1] > rgb[2], "{} non-skin {rgb:?}", look.texture_key);
                }
                assert!(weights[i].iter().all(|w| w.is_finite() && *w >= 0.0));
                assert!((weights[i].iter().sum::<f32>() - 1.0).abs() < 0.001);
            }
            assert!(arms.iter().all(|&n| n > 100), "{} arms {arms:?}", look.texture_key);
            if kind != CharacterKind::SwimmerGirl { assert!(torsos > 20, "{} torso", look.texture_key); }
            for i in 0..rig.mesh.positions.len() {
                if rig.mesh.positions[i][1] > 2.56 || rig.mesh.positions[i][1] < 1.25 {
                    assert_eq!(mesh.positions[i], rig.mesh.positions[i]);
                    assert_eq!(mesh.uvs[i], rig.mesh.uvs[i]);
                    assert_eq!(joints[i], rig.ids[i]); assert_eq!(weights[i], rig.weights[i]);
                }
            }
        }
    }
}
