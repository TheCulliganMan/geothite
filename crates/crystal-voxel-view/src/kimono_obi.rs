//! Fitted closed obi and a soft rear bow in the production rig's bind space.
use crate::{
    mesh::SurfaceMeshData,
    npc_appearance::{CostumeMesh, Look},
    open_people::PersonRig,
};
use bevy::prelude::*;

fn skin(rig: &PersonRig, influences: &[(usize, f32)]) -> ([u16; 4], [f32; 4]) {
    let mut merged = std::collections::BTreeMap::new();
    for &(joint, weight) in influences {
        // A sash belongs to the torso, even where the body's lower waist
        // triangles blend into the legs. Transfer that amount to the pelvis.
        let joint = if rig.leg_joints.iter().flatten().any(|&leg| leg == joint) {
            rig.hips_joint
        } else {
            joint
        };
        *merged.entry(joint).or_insert(0.0) += weight;
    }
    let mut values: Vec<_> = merged.into_iter().collect();
    values.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    values.truncate(4);
    let sum: f32 = values.iter().map(|p| p.1).sum();
    (
        std::array::from_fn(|i| values.get(i).map_or(0, |p| p.0 as u16)),
        std::array::from_fn(|i| values.get(i).map_or(0., |p| p.1 / sum)),
    )
}
fn vertex(out: &mut CostumeMesh, p: Vec3, n: Vec3, rgb: [u8; 3], bind: ([u16; 4], [f32; 4])) {
    out.surface.positions.push(p.to_array());
    out.surface.normals.push(n.to_array());
    out.surface.colors.push(
        Color::srgb_u8(rgb[0], rgb[1], rgb[2])
            .to_linear()
            .to_f32_array(),
    );
    out.surface.uvs.push([0.; 2]);
    out.joints.push(bind.0);
    out.weights.push(bind.1);
}
fn face(out: &mut CostumeMesh, a: usize, b: usize, c: usize) {
    let p = [a, b, c].map(|i| Vec3::from_array(out.surface.positions[i]));
    let n = [a, b, c]
        .map(|i| Vec3::from_array(out.surface.normals[i]))
        .into_iter()
        .sum::<Vec3>();
    let ids = if (p[1] - p[0]).cross(p[2] - p[0]).dot(n) > 0. {
        [a, b, c]
    } else {
        [a, c, b]
    };
    out.surface.indices.extend(ids.map(|i| i as u32));
}
fn oval(
    out: &mut CostumeMesh,
    center: Vec3,
    half: Vec3,
    tilt: f32,
    rgb: [u8; 3],
    bind: ([u16; 4], [f32; 4]),
) {
    const ROWS: usize = 12;
    const SIDES: usize = 32;
    let first = out.surface.positions.len();
    let rotation = Quat::from_rotation_z(tilt);
    vertex(
        out,
        center + rotation * (Vec3::Y * half.y),
        rotation * Vec3::Y,
        rgb,
        bind,
    );
    for row in 1..ROWS {
        for col in 0..SIDES {
            let phi = std::f32::consts::PI * row as f32 / ROWS as f32;
            let theta = std::f32::consts::TAU * col as f32 / SIDES as f32;
            let d = Vec3::new(phi.sin() * theta.cos(), phi.cos(), phi.sin() * theta.sin());
            vertex(
                out,
                center + rotation * (d * half),
                rotation * (d / half).normalize(),
                rgb,
                bind,
            );
        }
    }
    let south = out.surface.positions.len();
    vertex(
        out,
        center - rotation * (Vec3::Y * half.y),
        rotation * (-Vec3::Y),
        rgb,
        bind,
    );
    for col in 0..SIDES {
        let next = (col + 1) % SIDES;
        face(out, first, first + 1 + col, first + 1 + next);
        for row in 0..ROWS - 2 {
            let a = first + 1 + row * SIDES + col;
            let b = first + 1 + row * SIDES + next;
            face(out, a, b, b + SIDES);
            face(out, a, b + SIDES, a + SIDES);
        }
        let a = first + 1 + (ROWS - 2) * SIDES + col;
        let b = first + 1 + (ROWS - 2) * SIDES + next;
        face(out, a, b, south);
    }
}
pub(crate) fn build(rig: &PersonRig, look: Look) -> Option<CostumeMesh> {
    if !matches!(look.texture_key, "erika" | "falkner" | "kimono_girl") {
        return None;
    }
    let falkner = look.texture_key == "falkner";
    let edge_color = if falkner {
        [184, 179, 162]
    } else {
        [186, 166, 114]
    };
    let fabric_color = if falkner {
        [224, 219, 203]
    } else {
        [218, 203, 154]
    };
    let bow_color = if falkner {
        [211, 206, 188]
    } else {
        [205, 187, 136]
    };
    const ROWS: usize = 8;
    const SIDES: usize = 64;
    let mut out = CostumeMesh {
        surface: SurfaceMeshData::default(),
        joints: vec![],
        weights: vec![],
    };
    let point = |y: f32, angle: f32, inner: bool| -> Option<Vec3> {
        let direction = Vec2::new(angle.sin(), angle.cos());
        let sample = crate::garment_fit::section(rig, y, direction)?;
        let mut average = 0.0;
        for offset in [-0.12, -0.06, 0.0, 0.06, 0.12] {
            let a = angle + offset;
            average +=
                crate::garment_fit::section(rig, y, Vec2::new(a.sin(), a.cos()))?.radius / 5.0;
        }
        // Round the source polygon corners without intersecting the garment.
        let radius = (average + if inner { 0.025 } else { 0.044 })
            .max(sample.radius + if inner { 0.020 } else { 0.033 });
        Some(Vec3::new(
            direction.x * radius,
            y,
            0.05 + direction.y * radius,
        ))
    };
    let layer = (ROWS + 1) * SIDES;
    for inner in [false, true] {
        for row in 0..=ROWS {
            for col in 0..SIDES {
                let y = if falkner {
                    1.87 - 0.24 * row as f32 / ROWS as f32
                } else {
                    1.89 - 0.29 * row as f32 / ROWS as f32
                };
                let angle = std::f32::consts::TAU * col as f32 / SIDES as f32;
                let p = point(y, angle, inner)?;
                let across = point(y, angle + 0.001, inner)? - point(y, angle - 0.001, inner)?;
                let down = point(y - 0.001, angle, inner)? - point(y + 0.001, angle, inner)?;
                let mut normal = down.cross(across).normalize() * if inner { -1. } else { 1. };
                if row == 0 {
                    normal = (normal + Vec3::Y * 0.35).normalize();
                }
                if row == ROWS {
                    normal = (normal - Vec3::Y * 0.35).normalize();
                }
                let sample =
                    crate::garment_fit::section(rig, y, Vec2::new(angle.sin(), angle.cos()))?;
                let edge = row == 0 || row == ROWS;
                vertex(
                    &mut out,
                    p,
                    normal,
                    if edge { edge_color } else { fabric_color },
                    skin(rig, &sample.influences),
                );
            }
        }
    }
    for row in 0..ROWS {
        for col in 0..SIDES {
            let next = (col + 1) % SIDES;
            let a = row * SIDES + col;
            let b = row * SIDES + next;
            for offset in [0, layer] {
                face(&mut out, a + offset, b + offset, b + SIDES + offset);
                face(&mut out, a + offset, b + SIDES + offset, a + SIDES + offset);
            }
        }
    }
    for row in [0, ROWS] {
        for col in 0..SIDES {
            let next = (col + 1) % SIDES;
            let a = row * SIDES + col;
            let b = row * SIDES + next;
            face(&mut out, a, a + layer, b + layer);
            face(&mut out, a, b + layer, b);
        }
    }
    let sample = crate::garment_fit::section(rig, 1.745, Vec2::new(0., -1.))?;
    let bind = skin(rig, &sample.influences);
    let z = 0.05 - sample.radius - if falkner { 0.048 } else { 0.072 };
    if falkner {
        // A compact flat knot and short cloth ends distinguish this sash
        // from Erika's broad rear bow. Every part follows the torso palette.
        for side in [-1., 1.] {
            oval(
                &mut out,
                Vec3::new(side * 0.025, 1.69, z),
                Vec3::new(0.027, 0.075, 0.010),
                side * 0.22,
                bow_color,
                bind,
            );
        }
        oval(
            &mut out,
            Vec3::new(0., 1.745, z - 0.012),
            Vec3::new(0.095, 0.050, 0.026),
            0.,
            edge_color,
            bind,
        );
    } else {
        for side in [-1., 1.] {
            oval(
                &mut out,
                Vec3::new(side * 0.115, 1.76, z),
                Vec3::new(0.155, 0.09, 0.028),
                side * 0.15,
                bow_color,
                bind,
            );
        }
        oval(
            &mut out,
            Vec3::new(0., 1.745, z - 0.025),
            Vec3::new(0.060, 0.07, 0.068),
            0.,
            edge_color,
            bind,
        );
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires supplied male and female rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_obi_is_closed_fitted_and_uses_torso_skinning() {
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        use crate::new_bark_actors::character_meshes::CharacterKind::{Erika, Falkner, KimonoGirl};
        for kind in [Erika, Falkner, KimonoGirl] {
            let look = crate::npc_appearance::look(kind);
            let rig = PersonRig::parse(
                &std::fs::read(root.join(format!("people/kenney/{}.glb", look.base))).unwrap(),
            )
            .unwrap();
            let obi = build(&rig, look).unwrap();
            let mesh = &obi.surface;
            assert_eq!(mesh.positions.len(), obi.joints.len());
            assert_eq!(mesh.positions.len(), obi.weights.len());
            let mut edges = std::collections::HashMap::new();
            for ids in mesh.indices.chunks_exact(3) {
                let p =
                    [ids[0], ids[1], ids[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
                let n = ids
                    .iter()
                    .map(|&i| Vec3::from_array(mesh.normals[i as usize]))
                    .sum::<Vec3>();
                assert!((p[1] - p[0]).cross(p[2] - p[0]).dot(n) > 1e-9);
                for (a, b) in [(ids[0], ids[1]), (ids[1], ids[2]), (ids[2], ids[0])] {
                    *edges.entry((a.min(b), a.max(b))).or_insert(0) += 1;
                }
            }
            assert!(edges.values().all(|&n| n == 2));
            for i in 0..mesh.positions.len() {
                let p = Vec3::from_array(mesh.positions[i]);
                let n = Vec3::from_array(mesh.normals[i]);
                assert!(p.is_finite() && n.is_finite() && (n.length() - 1.).abs() < 1e-4);
                assert!((1.59..1.90).contains(&p.y));
                assert!((obi.weights[i].iter().sum::<f32>() - 1.).abs() < 1e-5);
                for (&j, &w) in obi.joints[i].iter().zip(&obi.weights[i]) {
                    assert!((j as usize) < rig.skin_nodes.len() && w >= 0.);
                    if w > 0.001 {
                        assert!(
                            !rig.leg_joints
                                .iter()
                                .flatten()
                                .any(|&leg| leg == j as usize)
                        );
                    }
                }
            }
            for p in &mesh.positions[..9 * 64 * 2] {
                let direction = Vec2::new(p[0], p[2] - 0.05).normalize();
                let sample = crate::garment_fit::section(&rig, p[1], direction).unwrap();
                let clearance = Vec2::new(p[0], p[2] - 0.05).length() - sample.radius;
                assert!((0.019..0.056).contains(&clearance));
            }
        }
    }
}
