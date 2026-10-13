//! Exposed skin must use the skin island, never the source shirt folds.
use crate::{mesh::SurfaceMeshData, npc_appearance::Look, open_people::PersonRig};

/// Spread the shoulder and elbow bends across the exposed arm. The supplied weights
/// jump from a chest/shoulder mix to a rigid arm over one short edge, folding
/// those faces when the arm lowers. UV copies receive the same position field.
fn exposed_arm_weights(rig: &PersonRig, mesh: &SurfaceMeshData,
    joints: &mut [[u16; 4]], weights: &mut [[f32; 4]], shoulder_span: f32) {
    let ancestry = |side: usize| {
        let hand = &rig.hand_joints[side];
        let root = *hand.iter().find(|&&joint| {
            rig.parents[rig.skin_nodes[joint]].is_none_or(|parent|
                !hand.iter().any(|&j| rig.skin_nodes[j] == parent))
        }).expect("exposed arm requires a hand attachment");
        let mut node = rig.skin_nodes[root];
        let mut ancestor = || {
            node = rig.parents[node].expect("exposed arm requires shoulder ancestry");
            rig.skin_nodes.iter().position(|&n| n == node)
                .expect("shoulder ancestor must be in the skin") as u16
        };
        let forearm = ancestor();
        let arm = ancestor();
        let shoulder = ancestor();
        let upper_chest = ancestor();
        (upper_chest, shoulder, arm, forearm)
    };
    let sides = [ancestry(0), ancestry(1)];
    let smooth = |t: f32| { let t = t.clamp(0., 1.); t*t*(3.-2.*t) };
    for ((&p, ids), values) in mesh.positions.iter().zip(joints.iter_mut()).zip(weights.iter_mut()) {
        let x = p[0].abs();
        if !(2.0..2.56).contains(&p[1]) || !(0.25..0.75).contains(&x) { continue; }
        let (chest, shoulder, arm, _) = sides[usize::from(p[0] < 0.)];
        if ids.iter().zip(values.iter()).any(|(&joint, &weight)| weight > 0.00001
            && ![rig.chest_joint as u16, chest, shoulder, arm].contains(&joint)) { continue; }
        let t = smooth((x-0.25)/shoulder_span);
        let fade = smooth((p[1]-2.0)/0.17) * smooth((x-0.25)/0.05);
        let mut blend = std::collections::BTreeMap::<u16, f32>::new();
        for (&joint, &weight) in ids.iter().zip(values.iter()) {
            *blend.entry(joint).or_default() += weight*(1.-fade);
        }
        for (joint, weight) in [(chest,(1.-t)*0.5), (shoulder,(1.-t)*0.5), (arm,t)] {
            *blend.entry(joint).or_default() += weight*fade;
        }
        let mut blend: Vec<_> = blend.into_iter().collect();
        blend.sort_by(|a,b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        blend.truncate(4);
        let total: f32 = blend.iter().map(|p| p.1).sum();
        *ids = std::array::from_fn(|i| blend.get(i).map_or(0, |p| p.0));
        *values = std::array::from_fn(|i| blend.get(i).map_or(0., |p| p.1/total));
    }
    // The elbow has the same short, stepped influence transition. Spread its
    // bend over the forearm without affecting wrist/finger influences.
    for ((&p, ids), values) in mesh.positions.iter().zip(joints).zip(weights) {
        let x=p[0].abs();
        if !(2.0..2.56).contains(&p[1]) || !(0.65..1.10).contains(&x) { continue; }
        let (_, _, arm, forearm)=sides[usize::from(p[0]<0.)];
        if ids.iter().zip(values.iter()).any(|(&joint,&weight)| weight>0.00001
            && ![arm,forearm].contains(&joint)) { continue; }
        let t=smooth((x-0.55)/0.60);
        *ids=[arm,forearm,0,0];
        *values=[1.-t,t,0.,0.];
    }
}

/// Shape each exposed torso instead of retaining the shirt's straight
/// front. Boundaries at the neck, hips and arms fade to the supplied surface.
fn exposed_torso(
    mesh: &mut SurfaceMeshData,
    joints: &mut Vec<[u16; 4]>,
    weights: &mut Vec<[f32; 4]>,
    role: &str,
) {
    let selection = |mesh: &SurfaceMeshData| -> Vec<bool> {
        mesh.positions.iter().zip(&mesh.uvs).map(|(&p, &uv)| {
            p[0].abs() < 0.43 && (1.34..2.50).contains(&p[1])
                && uv[0] < 0.625 && uv[1] > 0.46
        }).collect()
    };
    let selected = selection(mesh);
    let key = |p: [f32; 3]| p.map(|v| (v * 100_000.).round() as i32);
    crate::hand_surface::smooth_selection_joined(mesh, joints, weights, &selected);
    let selected = selection(mesh);
    // Include newly split UV boundary copies as well as source vertices.
    let protected: std::collections::BTreeSet<_> = mesh.positions.iter()
        .zip(&selected).filter(|(_, selected)| !**selected)
        .map(|(&p, _)| key(p)).collect();
    let bell = |value: f32, center: f32, radius: f32| {
        (-((value - center) / radius).powi(2)).exp()
    };
    for (p, &selected) in mesh.positions.iter_mut().zip(&selected) {
        if !selected || protected.contains(&key(*p)) { continue; }
        let hip = ((p[1] - 1.34) / 0.18).clamp(0., 1.);
        let neck = ((2.50 - p[1]) / 0.18).clamp(0., 1.);
        let arm = ((0.43 - p[0].abs()) / 0.10).clamp(0., 1.);
        let fade = hip * neck * arm;
        let chest = bell(p[1], 2.13, 0.20);
        let waist = bell(p[1], 1.67, 0.20);
        let front = ((p[2] - 0.05) / 0.12).clamp(0., 1.);
        let pectoral = bell(p[0].abs(), 0.15, 0.19) * chest;
        let sternum = bell(p[0], 0., 0.055) * chest;
        let abdomen = bell(p[0].abs(), 0.09, 0.11) * bell(p[1], 1.80, 0.23);
        let (width, depth, back_depth) = match role {
            "chuck" => {
                // A broad, heavy martial artist: the abdomen has volume,
                // rather than stretching the same narrow athlete sideways.
                let belly = bell(p[1], 1.78, 0.29);
                (0.10 * chest + 0.16 * belly,
                    0.035 * pectoral + 0.105 * belly - 0.004 * sternum,
                    0.025 * chest + 0.018 * belly)
            }
            "swimmer_guy" => (
                0.055 * chest - 0.065 * waist,
                0.024 * pectoral - 0.006 * sternum + 0.008 * abdomen,
                0.015 * chest,
            ),
            "bruno" => (
                0.09 * chest - 0.10 * waist,
                0.040 * pectoral - 0.009 * sternum + 0.012 * abdomen,
                0.,
            ),
            _ => unreachable!("unsupported exposed torso profile"),
        };
        let back = ((-p[2] - 0.05) / 0.12).clamp(0., 1.);
        p[0] *= 1. + fade * width;
        p[2] += fade * (front * depth - back * back_depth);
    }
    // Refine the shaped surface and rebuild welded geometric normals while
    // preserving separate UV islands and interpolated skin influences.
    let selected = selection(mesh);
    crate::hand_surface::smooth_selection_joined(mesh, joints, weights, &selected);
}

pub(crate) fn sculpt(
    rig: &PersonRig,
    mesh: &mut SurfaceMeshData,
    joints: &mut Vec<[u16; 4]>,
    weights: &mut Vec<[f32; 4]>,
    look: Look,
) {
    if look.texture_key == "nurse" {
        // Round the shoulder, cuff and forearm as one connected surface.
        // Starting at x=.4 leaves long underarm triangles spanning the
        // shoulder weight field; those triangles fold when the arm lowers.
        for _ in 0..2 {
            let arms: Vec<_> = mesh.positions.iter().zip(joints.iter()).zip(weights.iter())
                .map(|((&p, ids), values)| {
                    p[0].abs() > 0.25 && p[0].abs() < 1.20
                        && (2.0..2.55).contains(&p[1])
                        && !(0..4).any(|slot| values[slot] > 0.01
                            && rig.hand_joints.iter().flatten().any(|&j| j == ids[slot] as usize))
                }).collect();
            crate::hand_surface::smooth_selection_joined(mesh, joints, weights, &arms);
        }
        // Finish the shoulder transition at the start of the elbow field.
        // Overlapping these fields retained small shoulder influences on one
        // elbow ring and prevented the adjacent ring from receiving the blend.
        exposed_arm_weights(rig, mesh, joints, weights, 0.40);
        return;
    }
    if !matches!(look.texture_key, "bruno" | "chuck" | "swimmer_guy" | "swimmer_girl") {
        return;
    }
    if look.texture_key != "swimmer_girl" {
        exposed_torso(mesh, joints, weights, look.texture_key);
    }
    // The hands and neck already have their own skin mapping. Select the
    // source sleeve island only; wrist and shoulder boundaries remain pinned.
    let arms: Vec<_> = mesh.positions.iter().zip(&mesh.uvs).map(|(&p, &uv)| {
        p[0].abs() > 0.4 && (2.0..2.55).contains(&p[1])
            && uv[0] < 0.625 && uv[1] > 0.60
    }).collect();
    crate::hand_surface::smooth_selection_joined(mesh, joints, weights, &arms);
    exposed_arm_weights(rig, mesh, joints, weights, 0.50);
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
    #[ignore = "requires supplied rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_exposed_arms_keep_shading_on_the_outward_surface() {
        use bevy::prelude::*;
        fn world(i: usize, pose: &[Transform], parents: &[Option<usize>]) -> Mat4 {
            parents[i].map(|p|world(p,pose,parents)).unwrap_or(Mat4::IDENTITY)
                * pose[i].to_matrix()
        }
        let root=std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        for kind in [CharacterKind::Bruno,CharacterKind::Chuck,
            CharacterKind::SwimmerGuy,CharacterKind::SwimmerGirl,CharacterKind::Nurse] {
            let look=crate::npc_appearance::look(kind);
            let rig=PersonRig::parse(&std::fs::read(root.join(format!(
                "people/kenney/{}.glb",look.base))).unwrap()).unwrap();
            let mut mesh=rig.mesh.clone();let mut ids=rig.ids.clone();let mut weights=rig.weights.clone();
            if kind == CharacterKind::Nurse {
                crate::painted_torso::short_sleeves(&mut mesh, &mut ids, &mut weights, look);
            }
            sculpt(&rig,&mut mesh,&mut ids,&mut weights,look);
            let faces:Vec<_>=mesh.indices.chunks_exact(3).filter(|face|face.iter().all(|&i| {
                let p=mesh.positions[i as usize];
                (2.0..2.56).contains(&p[1]) && p[0].abs()<1.20
            })).collect();
            assert!(faces.len()>100,"missing exposed arm surface for {}",look.texture_key);
            for clip in ["Idle","Run"] { for step in 0..32 {
                let mut pose=rig.nodes.clone();
                rig.sample(clip,rig.duration(clip)*step as f32/32.,&mut pose);
                let inverse=world(rig.mesh_node,&pose,&rig.parents).inverse();
                let palette:Vec<_>=rig.skin_nodes.iter().zip(&rig.inverse_binds)
                    .map(|(&node,bind)|inverse*world(node,&pose,&rig.parents)* *bind).collect();
                let skin=|i:usize| {
                    let matrix=(0..4).fold(Mat4::ZERO,|sum,j|
                        sum+palette[ids[i][j] as usize]*weights[i][j]);
                    let basis=Mat3::from_cols(matrix.x_axis.truncate(),
                        matrix.y_axis.truncate(),matrix.z_axis.truncate());
                    (matrix.transform_point3(Vec3::from_array(mesh.positions[i])),
                        (basis.inverse().transpose()*Vec3::from_array(mesh.normals[i])).normalize())
                };
                for face in &faces {
                    let samples=[face[0],face[1],face[2]].map(|i|skin(i as usize));
                    let normal=(samples[1].0-samples[0].0).cross(samples[2].0-samples[0].0);
                    if normal.length_squared()<1e-18 { continue; }
                    for (_, shading) in samples {
                        let agreement=normal.normalize().dot(shading);
                        assert!(agreement>-0.01,
                            "{} exposed arm folds against shading in {clip} {step}/32 at {face:?}; agreement={agreement}; bind positions={:?}; influences={:?}",
                            look.texture_key,
                            face.iter().map(|&i|mesh.positions[i as usize]).collect::<Vec<_>>(),
                            face.iter().map(|&i|(ids[i as usize],weights[i as usize])).collect::<Vec<_>>());
                    }
                }
            }}
        }
    }

    #[test]
    #[ignore = "requires supplied rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_bare_body_uv_copies_remain_joined_through_run() {
        use bevy::prelude::*;
        fn world(i: usize, pose: &[Transform], parents: &[Option<usize>]) -> Mat4 {
            parents[i].map(|p|world(p,pose,parents)).unwrap_or(Mat4::IDENTITY)
                * pose[i].to_matrix()
        }
        let root=std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        for kind in [CharacterKind::Bruno,CharacterKind::Chuck,
            CharacterKind::SwimmerGuy,CharacterKind::SwimmerGirl] {
            let look=crate::npc_appearance::look(kind);
            let rig=PersonRig::parse(&std::fs::read(root.join(format!(
                "people/kenney/{}.glb",look.base))).unwrap()).unwrap();
            let mut mesh=rig.mesh.clone();let mut ids=rig.ids.clone();let mut weights=rig.weights.clone();
            sculpt(&rig, &mut mesh,&mut ids,&mut weights,look);
            let mut copies=std::collections::BTreeMap::<_,Vec<usize>>::new();
            for &index in &mesh.indices {
                let i=index as usize;let p=mesh.positions[i];
                if !(1.25..2.57).contains(&p[1]) {continue;}
                let key=p.map(|v|(v*100_000.).round() as i32);
                let group=copies.entry(key).or_default();
                if !group.contains(&i) {group.push(i);}
            }
            assert!(copies.values().filter(|g|g.len()>1).count()>20,
                "{} has no UV boundary copies to check",look.texture_key);
            let mut pose=rig.nodes.clone();
            for step in 0..16 {
                rig.sample("Run",rig.duration("Run")*step as f32/16.,&mut pose);
                let inverse=world(rig.mesh_node,&pose,&rig.parents).inverse();
                let palette:Vec<_>=rig.skin_nodes.iter().zip(&rig.inverse_binds)
                    .map(|(&node,bind)|inverse*world(node,&pose,&rig.parents)* *bind).collect();
                let skin=|i:usize| (0..4).map(|j|palette[ids[i][j] as usize]
                    .transform_point3(Vec3::from_array(mesh.positions[i]))*weights[i][j]).sum::<Vec3>();
                for group in copies.values().filter(|g|g.len()>1) {
                    let anchor=skin(group[0]);
                    for &i in &group[1..] {
                        let separation=anchor.distance(skin(i));
                        assert!(separation<0.00005,
                            "{} UV seam opens in Run {step}/16 at {:?}: {separation}",
                            look.texture_key,mesh.positions[i]);
                    }
                }
            }
        }
    }

    #[test]
    #[ignore = "requires supplied female rig and swimmer painting"]
    fn supplied_swimsuit_exposes_legs_and_keeps_blue_fabric_continuous() {
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let rig = PersonRig::parse(&std::fs::read(root.join("people/kenney/human-female.glb")).unwrap()).unwrap();
        let look = crate::npc_appearance::look(CharacterKind::SwimmerGirl);
        let painting = crate::npc_textures::decode(&std::fs::read(root.join("people/npc-textures/swimmer_girl.png")).unwrap()).unwrap();
        let mut mesh = rig.mesh.clone(); let mut joints = rig.ids.clone(); let mut weights = rig.weights.clone();
        crate::npc_appearance::shorts_body(&rig, &mut mesh, &mut joints, &mut weights, look);
        sculpt(&rig, &mut mesh, &mut joints, &mut weights, look);
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
            sculpt(&rig, &mut mesh, &mut joints, &mut weights, look);
            if matches!(kind, CharacterKind::Chuck | CharacterKind::SwimmerGuy) {
                let mut waist_width_change = 0.;
                let mut front_volume_change = 0.;
                for (i, &source) in rig.mesh.positions.iter().enumerate() {
                    if (1.55..1.95).contains(&source[1]) && source[0].abs() < 0.43 {
                        waist_width_change += mesh.positions[i][0].abs() - source[0].abs();
                        if source[2] > 0.05 {
                            front_volume_change += mesh.positions[i][2] - source[2];
                        }
                    }
                }
                if kind == CharacterKind::Chuck {
                    assert!(waist_width_change > 0.01,
                        "Chuck must have a fuller abdomen: {waist_width_change}");
                    assert!(front_volume_change > 0.01,
                        "Chuck must gain abdominal volume: {front_volume_change}");
                } else {
                    assert!(waist_width_change < -0.005,
                        "Swimmer must have a tapered waist: {waist_width_change}");
                }
            }
            // Separate UV islands can duplicate one geometric seam vertex.
            // Refinement must not move its skin copy away from its fabric copy.
            let mut seams = std::collections::BTreeMap::new();
            for (i, &p) in rig.mesh.positions.iter().enumerate() {
                let key = p.map(|v| (v * 100_000.).round() as i32);
                if let Some(&other) = seams.get(&key) {
                    let separation = bevy::prelude::Vec3::from_array(mesh.positions[i])
                        .distance(bevy::prelude::Vec3::from_array(mesh.positions[other]));
                    assert!(separation < 0.00002,
                        "{} opened a UV seam at {p:?}: {separation}", look.texture_key);
                } else { seams.insert(key, i); }
            }
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
