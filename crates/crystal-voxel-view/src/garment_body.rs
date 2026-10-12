//! Hide body surfaces covered by a closed garment while preserving exposed legs.
use crate::{mesh::SurfaceMeshData, npc_appearance::Look};
use bevy::prelude::*;
use std::collections::{BTreeMap, HashMap};

pub(crate) fn hem(look: Look) -> Option<f32> {
    if matches!(look.costume, "robe" | "kimono") {
        Some(0.36)
    } else if look.costume == "dress" || look.accessories.split(',').any(|a| a == "skirt") {
        Some(0.77)
    } else {
        None
    }
}

/// Nurse's dress leaves bare calves, rather than the supplied trousers.
/// Copy complete lower-body faces so the shoe and garment UV seams stay intact.
pub(crate) fn nurse_legs(
    mesh: &mut SurfaceMeshData,
    joints: &mut Vec<[u16; 4]>,
    weights: &mut Vec<[f32; 4]>,
    look: Look,
) {
    if look.texture_key != "nurse" { return; }
    for face in mesh.indices.chunks_exact_mut(3) {
        let ids = [face[0] as usize, face[1] as usize, face[2] as usize];
        if !ids.iter().all(|&i| mesh.uvs[i][0] >= 0.605 && mesh.uvs[i][1] >= 0.745) {
            continue;
        }
        for (index, &i) in face.iter_mut().zip(&ids) {
            let p = mesh.positions[i];
            *index = mesh.positions.len() as u32;
            mesh.positions.push(p);
            mesh.normals.push(mesh.normals[i]);
            mesh.colors.push(mesh.colors[i]);
            mesh.uvs.push([0.87 + p[2] * 0.02, 0.36 + p[1] * 0.025]);
            joints.push(joints[i]);
            weights.push(weights[i]);
        }
    }
}
pub(crate) fn cover(
    mesh: &mut SurfaceMeshData,
    joints: &mut Vec<[u16; 4]>,
    weights: &mut Vec<[f32; 4]>,
    look: Look,
) {
    let Some(hem) = hem(look) else {
        return;
    };
    let faces = std::mem::take(&mut mesh.indices);
    let mut cuts = HashMap::new();
    for face in faces.chunks_exact(3) {
        let ids = [face[0] as usize, face[1] as usize, face[2] as usize];
        // Verified lower-body atlas islands exclude shirts, sleeves and hands.
        if !ids
            .iter()
            .all(|&i| mesh.uvs[i][0] >= 0.605 && mesh.uvs[i][1] >= 0.745)
        {
            mesh.indices.extend_from_slice(face);
            continue;
        }
        let mut polygon = Vec::new();
        for edge in 0..3 {
            let a = ids[edge];
            let b = ids[(edge + 1) % 3];
            let ay = mesh.positions[a][1];
            let by = mesh.positions[b][1];
            let limit = hem + 0.015;
            if ay <= limit {
                polygon.push(a);
            }
            if (ay <= limit) != (by <= limit) {
                let cut = *cuts.entry((a.min(b), a.max(b))).or_insert_with(|| {
                    let t = (limit - ay) / (by - ay);
                    let index = mesh.positions.len();
                    mesh.positions.push(
                        Vec3::from_array(mesh.positions[a])
                            .lerp(Vec3::from_array(mesh.positions[b]), t)
                            .to_array(),
                    );
                    mesh.normals.push(
                        Vec3::from_array(mesh.normals[a])
                            .lerp(Vec3::from_array(mesh.normals[b]), t)
                            .normalize()
                            .to_array(),
                    );
                    mesh.uvs.push(
                        Vec2::from_array(mesh.uvs[a])
                            .lerp(Vec2::from_array(mesh.uvs[b]), t)
                            .to_array(),
                    );
                    mesh.colors.push(
                        Vec4::from_array(mesh.colors[a])
                            .lerp(Vec4::from_array(mesh.colors[b]), t)
                            .to_array(),
                    );
                    let mut blend = BTreeMap::<u16, f32>::new();
                    for (i, factor) in [(a, 1.0 - t), (b, t)] {
                        for slot in 0..4 {
                            *blend.entry(joints[i][slot]).or_default() += weights[i][slot] * factor;
                        }
                    }
                    let mut blend: Vec<_> = blend.into_iter().collect();
                    blend.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
                    blend.truncate(4);
                    let sum: f32 = blend.iter().map(|v| v.1).sum();
                    joints.push(std::array::from_fn(|i| blend.get(i).map_or(0, |v| v.0)));
                    weights.push(std::array::from_fn(|i| {
                        blend.get(i).map_or(0.0, |v| v.1 / sum)
                    }));
                    index
                });
                polygon.push(cut);
            }
        }
        for i in 1..polygon.len().saturating_sub(1) {
            let tri = [polygon[0] as u32, polygon[i] as u32, polygon[i + 1] as u32];
            let p = tri.map(|i| Vec3::from_array(mesh.positions[i as usize]));
            if (p[1] - p[0]).cross(p[2] - p[0]).length_squared() > 1e-18 {
                mesh.indices.extend(tri);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn world(i: usize, pose: &[Transform], parents: &[Option<usize>]) -> Mat4 {
        parents[i]
            .map(|p| world(p, pose, parents))
            .unwrap_or(Mat4::IDENTITY)
            * pose[i].to_matrix()
    }
    #[test]
    #[ignore = "requires shared Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn kurt_running_shoes_do_not_cross_robe_panels() {
        use crate::{new_bark_actors::character_meshes::CharacterKind, open_people::PersonRig};
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let rig = PersonRig::parse(&std::fs::read(root.join("people/kenney/human-male.glb")).unwrap()).unwrap();
        let cloth = crate::npc_appearance::costume_mesh(&rig, crate::npc_appearance::look(CharacterKind::Kurt)).unwrap();
        let shoes = rig.mesh.indices.chunks_exact(3).filter(|f|
            f.iter().all(|&i| rig.mesh.positions[i as usize][1] < 0.27)).collect::<Vec<_>>();
        assert!(!shoes.is_empty());
        let panels = cloth.surface.indices.chunks_exact(3).filter(|f|
            f.iter().all(|&i| cloth.surface.positions[i as usize][1] < 1.0)).collect::<Vec<_>>();
        let deform = |p: [f32; 3], ids: [u16; 4], weights: [f32; 4], skin: &[Mat4]| {
            (0..4).map(|i| skin[ids[i] as usize].transform_point3(Vec3::from_array(p)) * weights[i]).sum::<Vec3>()
        };
        // Segment/triangle intersection catches a shoe piercing fabric even
        // when both meshes still have valid weights and outward winding.
        let crosses = |a: Vec3, b: Vec3, tri: [Vec3; 3]| {
            let dir = b - a; let e1 = tri[1] - tri[0]; let e2 = tri[2] - tri[0];
            let h = dir.cross(e2); let det = e1.dot(h);
            if det.abs() < 1e-10 { return false; }
            let s = a - tri[0]; let u = s.dot(h) / det;
            if !(0.0001..0.9999).contains(&u) { return false; }
            let q = s.cross(e1); let v = dir.dot(q) / det; let t = e2.dot(q) / det;
            v > 0.0001 && u + v < 0.9999 && t > 0.0001 && t < 0.9999
        };
        for step in 0..32 {
            let mut pose = rig.nodes.clone(); rig.sample("Run", rig.duration("Run") * step as f32 / 32., &mut pose);
            let inverse = world(rig.mesh_node, &pose, &rig.parents).inverse();
            let skin = rig.skin_nodes.iter().zip(&rig.inverse_binds).map(|(&n, b)| inverse * world(n, &pose, &rig.parents) * *b).collect::<Vec<_>>();
            let body = rig.mesh.positions.iter().enumerate().map(|(i, &p)| deform(p, rig.ids[i], rig.weights[i], &skin)).collect::<Vec<_>>();
            let fabric = cloth.surface.positions.iter().enumerate().map(|(i, &p)| deform(p, cloth.joints[i], cloth.weights[i], &skin)).collect::<Vec<_>>();
            for shoe in &shoes { for panel in &panels {
                let tri = [fabric[panel[0] as usize], fabric[panel[1] as usize], fabric[panel[2] as usize]];
                for edge in 0..3 {
                    assert!(!crosses(body[shoe[edge] as usize], body[shoe[(edge + 1) % 3] as usize], tri),
                        "Kurt shoe crosses robe at Run {step}/32: shoe={shoe:?} panel={panel:?}");
                }
            }}
        }
    }

    #[test]
    #[ignore = "requires shared Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_closed_garment_panels_keep_outward_winding_through_run() {
        use crate::{new_bark_actors::character_meshes::SOURCE_KINDS, open_people::PersonRig};
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let load = |name| {
            PersonRig::parse(
                &std::fs::read(root.join(format!("people/kenney/{name}.glb"))).unwrap(),
            )
            .unwrap()
        };
        let male = load("human-male");
        let female = load("human-female");
        let mut checked = 0;
        let mut failures = std::collections::BTreeMap::new();
        let mut examples = Vec::new();
        for (_, kind) in SOURCE_KINDS {
            let look = crate::npc_appearance::look(kind);
            if hem(look).is_none() {
                continue;
            }
            let rig = if look.base == "human-female" {
                &female
            } else {
                &male
            };
            let cloth = crate::npc_appearance::costume_mesh(rig, look).unwrap();
            let mut pose = rig.nodes.clone();
            checked += 1;
            for step in 0..16 {
                rig.sample("Run", rig.duration("Run") * step as f32 / 16.0, &mut pose);
                let inverse = world(rig.mesh_node, &pose, &rig.parents).inverse();
                let skin: Vec<_> = rig
                    .skin_nodes
                    .iter()
                    .zip(&rig.inverse_binds)
                    .map(|(&node, bind)| inverse * world(node, &pose, &rig.parents) * *bind)
                    .collect();
                let positions: Vec<_> = cloth
                    .surface
                    .positions
                    .iter()
                    .enumerate()
                    .map(|(i, &p)| {
                        (0..4)
                            .map(|slot| {
                                skin[cloth.joints[i][slot] as usize]
                                    .transform_point3(Vec3::from_array(p))
                                    * cloth.weights[i][slot]
                            })
                            .sum::<Vec3>()
                    })
                    .collect();
                let normals: Vec<_> = cloth
                    .surface
                    .normals
                    .iter()
                    .enumerate()
                    .map(|(i, &n)| {
                        let matrix=(0..4).fold(Mat4::ZERO,|sum,slot|
                            sum+skin[cloth.joints[i][slot] as usize]*cloth.weights[i][slot]);
                        let basis=Mat3::from_cols(matrix.x_axis.truncate(),
                            matrix.y_axis.truncate(),matrix.z_axis.truncate());
                        (basis.inverse().transpose()*Vec3::from_array(n)).normalize()
                    })
                    .collect();
                for face in cloth.surface.indices.chunks_exact(3) {
                    let ids = [face[0] as usize, face[1] as usize, face[2] as usize];
                    let bind = ids.map(|i| Vec3::from_array(cloth.surface.positions[i]));
                    // Shared rim vertices retain side normals; check the
                    // vertical fabric panels rather than horizontal rim caps.
                    if (bind[0].y - bind[1].y).abs() < 1e-6 && (bind[0].y - bind[2].y).abs() < 1e-6
                    {
                        continue;
                    }
                    let p = ids.map(|i| positions[i]);
                    let normal = ids
                        .map(|i| normals[i])
                        .into_iter()
                        .sum::<Vec3>()
                        .normalize();
                    let geometric = (p[1] - p[0]).cross(p[2] - p[0]);
                    assert!(
                        geometric.length_squared() > 1e-12,
                        "{} collapsed panel at {step}/16",
                        look.texture_key
                    );
                    let facing = geometric.normalize().dot(normal);
                    if !(facing > 0.0) {
                        *failures.entry((look.texture_key, step)).or_insert(0usize) += 1;
                        if examples.len() < 12 {
                            examples.push(format!("{} reversed panel at {step}/16: facing={facing} ids={ids:?} bind={bind:?} posed={p:?} normal={normal:?} geometric={geometric:?}", look.texture_key));
                        }
                    }
                }
            }
        }
        assert_eq!(checked, 25);
        assert!(failures.is_empty(), "reversed panels by role and Run phase: {failures:?}\n{}", examples.join("\n"));
    }

    #[test]
    #[ignore = "requires shared Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_closed_garments_cover_legs_without_changing_exposed_body_or_skinning() {
        use crate::{new_bark_actors::character_meshes::SOURCE_KINDS, open_people::PersonRig};
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let load = |name| {
            PersonRig::parse(
                &std::fs::read(root.join(format!("people/kenney/{name}.glb"))).unwrap(),
            )
            .unwrap()
        };
        let male = load("human-male");
        let female = load("human-female");
        let mut changed = 0;
        for (_, kind) in SOURCE_KINDS {
            let look = crate::npc_appearance::look(kind);
            let rig = if look.base == "human-female" {
                &female
            } else {
                &male
            };
            let mut mesh = rig.mesh.clone();
            let mut joints = rig.ids.clone();
            let mut weights = rig.weights.clone();
            cover(&mut mesh, &mut joints, &mut weights, look);
            let Some(hem) = hem(look) else {
                assert_eq!(mesh.indices, rig.mesh.indices);
                assert_eq!(mesh.positions, rig.mesh.positions);
                assert_eq!(joints, rig.ids);
                assert_eq!(weights, rig.weights);
                continue;
            };
            changed += 1;
            assert_ne!(mesh.indices, rig.mesh.indices);
            let original = rig.mesh.positions.len();
            assert_eq!(&mesh.positions[..original], rig.mesh.positions.as_slice());
            assert_eq!(&mesh.normals[..original], rig.mesh.normals.as_slice());
            assert_eq!(&mesh.uvs[..original], rig.mesh.uvs.as_slice());
            assert_eq!(&joints[..original], rig.ids.as_slice());
            assert_eq!(&weights[..original], rig.weights.as_slice());
            assert_eq!(mesh.positions.len(), joints.len());
            assert_eq!(joints.len(), weights.len());
            for face in rig.mesh.indices.chunks_exact(3) {
                if face
                    .iter()
                    .all(|&i| rig.mesh.positions[i as usize][1] < 0.24)
                    || !face.iter().all(|&i| {
                        rig.mesh.uvs[i as usize][0] >= 0.605 && rig.mesh.uvs[i as usize][1] >= 0.745
                    })
                {
                    assert!(
                        mesh.indices.chunks_exact(3).any(|f| f == face),
                        "{} exposed body changed",
                        look.texture_key
                    );
                }
            }
            for face in mesh.indices.chunks_exact(3) {
                if face
                    .iter()
                    .all(|&i| mesh.uvs[i as usize][0] >= 0.605 && mesh.uvs[i as usize][1] >= 0.745)
                {
                    assert!(
                        face.iter()
                            .all(|&i| mesh.positions[i as usize][1] <= hem + 0.01501)
                    );
                }
                let p = [face[0], face[1], face[2]]
                    .map(|i| Vec3::from_array(mesh.positions[i as usize]));
                let n = face
                    .iter()
                    .map(|&i| Vec3::from_array(mesh.normals[i as usize]))
                    .sum::<Vec3>();
                assert!((p[1] - p[0]).cross(p[2] - p[0]).dot(n) >= -1e-6);
            }
            assert!(
                weights
                    .iter()
                    .all(|w| w.iter().all(|v| v.is_finite() && *v >= 0.0)
                        && (w.iter().sum::<f32>() - 1.0).abs() < 1e-5)
            );
        }
        assert_eq!(
            changed, 25,
            "all closed garments, excluding four open coats"
        );
    }
}
