//! Fitted uniform crowns and attached peaks in the supplied head's bind space.
use crate::mesh::SurfaceMeshData;
use bevy::prelude::*;

fn color(rgb: [u8; 3]) -> [f32; 4] {
    let c = rgb.map(|v| Color::srgb_u8(v, v, v).to_linear().red);
    [c[0], c[1], c[2], 1.]
}
fn triangle(mesh: &mut SurfaceMeshData, p: [Vec3; 3], rgb: [u8; 3]) {
    let normal = (p[1] - p[0]).cross(p[2] - p[0]);
    if normal.length_squared() < 1e-12 {
        return;
    }
    let first = mesh.positions.len() as u32;
    for p in p {
        mesh.positions.push(p.to_array());
        mesh.normals.push(normal.normalize().to_array());
        mesh.colors.push(color(rgb));
        mesh.uvs.push([0.; 2]);
    }
    mesh.indices.extend([first, first + 1, first + 2]);
}

fn crown(
    mesh: &mut SurfaceMeshData,
    center: Vec3,
    half: Vec3,
    profiles: &[(f32, f32, f32)],
    rgb: [u8; 3],
) {
    const SIDES: usize = 48;
    let first = mesh.positions.len() as u32;
    for (row, &(y, x, z)) in profiles.iter().enumerate() {
        let lo = profiles[row.saturating_sub(1)];
        let hi = profiles[(row + 1).min(profiles.len() - 1)];
        for side in 0..=SIDES {
            let a = std::f32::consts::TAU * side as f32 / SIDES as f32;
            let p = center + half * Vec3::new(x * a.cos(), y, z * a.sin());
            let along = half * Vec3::new(-x * a.sin(), 0., z * a.cos());
            let up = half
                * Vec3::new(
                    (hi.1 - lo.1) * a.cos(),
                    hi.0 - lo.0,
                    (hi.2 - lo.2) * a.sin(),
                );
            mesh.positions.push(p.to_array());
            mesh.normals.push(up.cross(along).normalize().to_array());
            mesh.colors.push(color(rgb));
            mesh.uvs.push([0.; 2]);
        }
    }
    for row in 0..profiles.len() - 1 {
        for side in 0..SIDES {
            let a = first + (row * (SIDES + 1) + side) as u32;
            let b = a + (SIDES + 1) as u32;
            mesh.indices.extend([a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    for (row, upper) in [(0, false), (profiles.len() - 1, true)] {
        let y = profiles[row].0;
        let middle = center + Vec3::Y * (half.y * y);
        for side in 0..SIDES {
            let a = Vec3::from_array(mesh.positions[first as usize + row * (SIDES + 1) + side]);
            let b = Vec3::from_array(mesh.positions[first as usize + row * (SIDES + 1) + side + 1]);
            triangle(
                mesh,
                if upper {
                    [a, middle, b]
                } else {
                    [a, b, middle]
                },
                rgb,
            );
        }
    }
}

fn peak(mesh: &mut SurfaceMeshData, center: Vec3, half: Vec3) {
    const COLS: usize = 32;
    const ROWS: usize = 5;
    let p = |col: usize, row: usize, lower: bool| {
        let s = col as f32 / COLS as f32 * 2. - 1.;
        let t = row as f32 / ROWS as f32;
        let arch = (1. - s * s).max(0.).sqrt();
        center
            + half
                * Vec3::new(
                    s * 0.92,
                    -0.25 - 0.045 * t + 0.055 * s * s - if lower { 0.028 } else { 0. },
                    0.80 * arch + 0.64 * t * arch,
                )
    };
    let rgb = [25, 31, 40];
    for row in 0..ROWS {
        for col in 0..COLS {
            for lower in [false, true] {
                let q = [
                    p(col, row, lower),
                    p(col, row + 1, lower),
                    p(col + 1, row + 1, lower),
                    p(col + 1, row, lower),
                ];
                for ids in if lower {
                    [[0, 2, 1], [0, 3, 2]]
                } else {
                    [[0, 1, 2], [0, 2, 3]]
                } {
                    triangle(mesh, ids.map(|i| q[i]), rgb);
                }
            }
        }
    }
    for row in [0, ROWS] {
        for col in 0..COLS {
            let q = [
                p(col, row, false),
                p(col + 1, row, false),
                p(col + 1, row, true),
                p(col, row, true),
            ];
            for ids in if row == ROWS {
                [[0, 2, 1], [0, 3, 2]]
            } else {
                [[0, 1, 2], [0, 2, 3]]
            } {
                triangle(mesh, ids.map(|i| q[i]), rgb);
            }
        }
    }
}

/// Returns false for hats handled by the other head accessory styles.
pub(crate) fn append(mesh: &mut SurfaceMeshData, center: Vec3, half: Vec3, hat: &str) -> bool {
    if !matches!(hat, "police" | "captain" | "sailor") {
        return false;
    }
    let origin = Vec3::new(center.x, center.y + half.y, center.z);
    if hat == "sailor" {
        crown(
            mesh,
            origin,
            half,
            &[
                (-0.28, 1.02, 0.94),
                (-0.24, 1.08, 1.00),
                (-0.12, 1.10, 1.02),
                (0.02, 1.07, 0.99),
                (0.08, 1.00, 0.93),
                (0.14, 0.78, 0.73),
                (0.18, 0.46, 0.43),
                (0.19, 0.09, 0.08),
            ],
            [239, 238, 223],
        );
        crown(
            mesh,
            origin,
            half,
            &[(-0.27, 1.025, 0.945), (-0.21, 1.095, 1.015)],
            [38, 57, 86],
        );
    } else {
        crown(
            mesh,
            origin,
            half,
            &[(-0.30, 1.01, 0.93), (-0.18, 1.02, 0.94)],
            [28, 39, 59],
        );
        crown(
            mesh,
            origin,
            half,
            &[
                (-0.20, 1.02, 0.94),
                (-0.13, 1.12, 1.02),
                (0.01, 1.17, 1.06),
                (0.14, 1.12, 1.03),
                (0.22, 0.93, 0.85),
                (0.27, 0.63, 0.57),
                (0.29, 0.29, 0.26),
                (0.30, 0.05, 0.05),
            ],
            if hat == "captain" {
                [239, 238, 223]
            } else {
                [36, 64, 117]
            },
        );
        peak(mesh, origin, half);
        let badge = origin + half * Vec3::new(0., -0.05, 1.065);
        let contour = [
            (-0.12, 0.09),
            (0., 0.13),
            (0.12, 0.09),
            (0.10, -0.07),
            (0., -0.15),
            (-0.10, -0.07),
        ];
        for i in 0..contour.len() {
            let point =
                |j: usize| badge + Vec3::new(contour[j].0 * half.x, contour[j].1 * half.x, 0.008);
            triangle(
                mesh,
                [
                    badge + Vec3::Z * 0.008,
                    point((i + 1) % contour.len()),
                    point(i),
                ],
                [219, 180, 76],
            );
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires locally supplied Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_uniform_caps_fit_above_eyes_in_the_head_attachment_space() {
        use crate::new_bark_actors::character_meshes::CharacterKind;
        use crate::{npc_appearance, open_people::PersonRig};
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let rig =
            PersonRig::parse(&std::fs::read(root.join("people/kenney/human-male.glb")).unwrap())
                .unwrap();
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for ((&p, ids), weights) in rig.mesh.positions.iter().zip(&rig.ids).zip(&rig.weights) {
            let head: f32 = (0..4)
                .filter(|&i| ids[i] as usize == rig.head_joint)
                .map(|i| weights[i])
                .sum();
            if head > 0.5 {
                min = min.min(Vec3::from_array(p));
                max = max.max(Vec3::from_array(p));
            }
        }
        let center = (min + max) * 0.5;
        let half = (max - min) * 0.5;
        let undo = rig.inverse_binds[rig.head_joint].inverse();
        for kind in [
            CharacterKind::Officer,
            CharacterKind::Captain,
            CharacterKind::Sailor,
        ] {
            let mut look = npc_appearance::look(kind);
            // Inspect the cap alone, excluding the captain's facial accessory.
            look.accessories = "";
            let mesh = npc_appearance::head_mesh(&rig, look).unwrap();
            for &p in &mesh.positions {
                let p = undo.transform_point3(Vec3::from_array(p));
                assert!(p.is_finite());
                assert!(
                    p.y > center.y + half.y * 0.60,
                    "cap obscures the forehead/eyes"
                );
                assert!((p.x - center.x).abs() <= half.x * 1.18);
                assert!((p.z - center.z).abs() <= half.z * 1.45);
                assert!(p.y < max.y + half.y * 0.31);
            }
            assert!(
                mesh.normals
                    .iter()
                    .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 1e-5)
            );
        }
    }

    #[test]
    fn distinct_uniform_caps_have_finite_smooth_crowns_and_valid_faces() {
        let mut shapes = Vec::new();
        for hat in ["police", "captain", "sailor"] {
            let mut mesh = SurfaceMeshData::default();
            assert!(append(
                &mut mesh,
                Vec3::ZERO,
                Vec3::new(0.45, 0.52, 0.40),
                hat
            ));
            assert!(mesh.positions.len() > 600);
            assert!(
                mesh.normals
                    .iter()
                    .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 1e-5)
            );
            for face in mesh.indices.chunks_exact(3) {
                let p = [face[0], face[1], face[2]]
                    .map(|i| Vec3::from_array(mesh.positions[i as usize]));
                let n = (p[1] - p[0]).cross(p[2] - p[0]);
                assert!(n.is_finite() && n.length_squared() > 1e-12);
                let shading: Vec3 = face
                    .iter()
                    .map(|&i| Vec3::from_array(mesh.normals[i as usize]))
                    .sum();
                assert!(n.dot(shading) > 0.);
            }
            shapes.push(mesh);
        }
        assert_ne!(shapes[0].positions, shapes[2].positions);
        assert_ne!(shapes[0].colors, shapes[1].colors);
    }
}
