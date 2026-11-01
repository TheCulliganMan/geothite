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

fn peak(
    mesh: &mut SurfaceMeshData,
    center: Vec3,
    half: Vec3,
    upper_rgb: [u8; 3],
    lower_rgb: [u8; 3],
    thickness: f32,
    bend: f32,
) {
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
                    -0.25 - bend * t + 0.055 * s * s - if lower { thickness } else { 0. },
                    0.80 * arch + 0.64 * t * arch,
                )
    };
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
                    triangle(
                        mesh,
                        ids.map(|i| q[i]),
                        if lower { lower_rgb } else { upper_rgb },
                    );
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
                triangle(mesh, ids.map(|i| q[i]), upper_rgb);
            }
        }
    }
}

fn nurse_cap(mesh: &mut SurfaceMeshData, origin: Vec3, half: Vec3) {
    // A folded cloth crown with a fitted lower edge, rounded corners and a
    // flared front replaces the unrelated oval resting on top of the hair.
    const SIDES: usize = 64;
    let profiles = [
        (-0.15, 0.82, 0.48),
        (-0.13, 0.84, 0.49),
        (0.16, 1.025, 0.44),
        (0.18, 1.03, 0.43),
    ];
    let point = |row: usize, a: f32| {
        let (y, x, z) = profiles[row];
        let rounded = |v: f32| {
            if v.abs() < 1e-6 {
                0.
            } else {
                v.signum() * v.abs().sqrt()
            }
        };
        let sx = rounded(a.cos());
        let sz = rounded(a.sin());
        origin + half * Vec3::new(x * sx, y + 0.055 * sz, z * sz)
    };
    let first = mesh.positions.len() as u32;
    for row in 0..profiles.len() {
        for side in 0..=SIDES {
            let a = std::f32::consts::TAU * side as f32 / SIDES as f32;
            let p = point(row, a);
            let along = point(row, a + 0.001) - point(row, a - 0.001);
            let up = point((row + 1).min(profiles.len() - 1), a) - point(row.saturating_sub(1), a);
            mesh.positions.push(p.to_array());
            mesh.normals.push(up.cross(along).normalize().to_array());
            mesh.colors.push(color([241, 241, 231]));
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
        let middle = origin + Vec3::Y * (half.y * profiles[row].0);
        for side in 0..SIDES {
            let a = point(row, std::f32::consts::TAU * side as f32 / SIDES as f32);
            let b = point(
                row,
                std::f32::consts::TAU * (side + 1) as f32 / SIDES as f32,
            );
            triangle(
                mesh,
                if upper {
                    [a, middle, b]
                } else {
                    [a, b, middle]
                },
                [241, 241, 231],
            );
        }
    }
    // Three non-overlapping cloth panels form the cross. Fit their depth to
    // the sloping front, rather than using two intersecting red ellipsoids.
    let badge = |x: f32, y: f32| {
        let y = y + 0.055;
        let z = 0.48 - (y + 0.095) / 0.33 * 0.05;
        origin + half * Vec3::new(x, y, z) + Vec3::Z * 0.006
    };
    for (left, right, low, high) in [
        (-0.04, 0.04, -0.11, -0.04),
        (-0.14, 0.14, -0.04, 0.04),
        (-0.04, 0.04, 0.04, 0.11),
    ] {
        let p = [
            badge(left, low),
            badge(right, low),
            badge(right, high),
            badge(left, high),
        ];
        triangle(mesh, [p[0], p[1], p[2]], [183, 36, 48]);
        triangle(mesh, [p[0], p[2], p[3]], [183, 36, 48]);
    }
}

/// Returns false for hats handled by the other head accessory styles.
pub(crate) fn append(mesh: &mut SurfaceMeshData, center: Vec3, half: Vec3, hat: &str) -> bool {
    append_with_color(mesh, center, half, hat, None)
}

/// Only the police cloth crown follows the source pigment; visor, band and badge retain their materials.
pub(crate) fn append_with_color(mesh: &mut SurfaceMeshData, center: Vec3, half: Vec3, hat: &str, color: Option<[u8; 3]>) -> bool {
    if !matches!(
        hat,
        "trainer" | "kris" | "police" | "captain" | "sailor" | "top" | "nurse"
    ) {
        return false;
    }
    let origin = Vec3::new(center.x, center.y + half.y, center.z);
    if hat == "nurse" {
        nurse_cap(mesh, origin, half);
        return true;
    }
    if hat == "kris" {
        // A soft, fitted crown with an attached lower band replaces the
        // separate oval. Keep the established blue cap and open forehead.
        crown(
            mesh,
            origin,
            half,
            &[
                (-0.38, 0.99, 0.93),
                (-0.30, 1.05, 0.98),
                (-0.19, 1.10, 1.02),
                (-0.06, 1.09, 0.99),
                (0.08, 0.84, 0.76),
                (0.14, 0.52, 0.47),
                (0.16, 0.10, 0.09),
            ],
            [32, 62, 128],
        );
        crown(
            mesh,
            origin,
            half,
            &[(-0.40, 0.99, 0.93), (-0.33, 1.04, 0.975)],
            [25, 43, 88],
        );
        return true;
    }
    if hat == "trainer" {
        // The crown follows the scalp; its bill attaches below the dome and
        // curves downward gently rather than hovering as a separate oval.
        crown(
            mesh,
            origin,
            half,
            &[
                (-0.54, 1.015, 0.985),
                (-0.43, 1.035, 1.005),
                (-0.29, 1.015, 0.985),
                (-0.14, 0.950, 0.925),
                (0.00, 0.815, 0.795),
                (0.11, 0.600, 0.585),
                (0.18, 0.335, 0.330),
                (0.20, 0.065, 0.065),
            ],
            [191, 35, 37],
        );
        crown(
            mesh,
            origin,
            half,
            &[(-0.55, 1.018, 0.988), (-0.51, 1.030, 1.000)],
            [153, 30, 35],
        );
        peak(
            mesh,
            origin - Vec3::Y * half.y * 0.25,
            half,
            [181, 32, 36],
            [103, 30, 35],
            0.065,
            0.09,
        );
        // Small cloth-covered top button closes the crown without a broad
        // flat cap at its apex.
        crown(
            mesh,
            origin,
            half,
            &[
                (0.19, 0.072, 0.072),
                (0.225, 0.065, 0.065),
                (0.24, 0.025, 0.025),
            ],
            [174, 31, 35],
        );
        return true;
    }
    if hat == "top" {
        // A flat topped, slightly flared crown, rather than the old dome.
        // All dimensions follow the supplied head and its animation anchor.
        crown(
            mesh,
            origin,
            half,
            &[
                (-0.24, 0.95, 0.88),
                (-0.18, 0.96, 0.89),
                (0.68, 0.99, 0.92),
                (0.83, 1.03, 0.97),
                (0.88, 1.02, 0.96),
            ],
            [29, 31, 37],
        );
        crown(
            mesh,
            origin,
            half,
            &[
                (-0.32, 1.30, 1.17),
                (-0.30, 1.36, 1.23),
                (-0.25, 1.36, 1.23),
                (-0.22, 1.30, 1.17),
            ],
            [29, 31, 37],
        );
        crown(
            mesh,
            origin,
            half,
            &[(-0.19, 0.976, 0.906), (-0.06, 0.980, 0.911)],
            [47, 51, 60],
        );
        return true;
    }
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
                color.unwrap_or([36, 64, 117])
            },
        );
        peak(mesh, origin, half, [25, 31, 40], [25, 31, 40], 0.028, 0.045);
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
            CharacterKind::Gentleman,
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
                assert!(
                    (p.x - center.x).abs()
                        <= half.x
                            * if kind == CharacterKind::Gentleman {
                                1.38
                            } else {
                                1.18
                            }
                );
                assert!((p.z - center.z).abs() <= half.z * 1.45);
                assert!(
                    p.y < max.y
                        + half.y
                            * if kind == CharacterKind::Gentleman {
                                0.90
                            } else {
                                0.31
                            }
                );
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
        for hat in ["police", "captain", "sailor", "top", "nurse"] {
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

#[cfg(test)]
mod source_color_tests {
    use super::*;
    #[test]
    fn police_source_crown_preserves_geometry_visor_band_and_badge() {
        let mut blue = SurfaceMeshData::default();
        let mut green = SurfaceMeshData::default();
        assert!(append(&mut blue, Vec3::ZERO, Vec3::splat(0.3), "police"));
        assert!(append_with_color(&mut green, Vec3::ZERO, Vec3::splat(0.3), "police", Some([57,189,24])));
        assert_eq!(blue.positions, green.positions);
        assert_eq!(blue.normals, green.normals);
        assert_eq!(blue.uvs, green.uvs);
        assert_eq!(blue.indices, green.indices);
        let base = Color::srgb_u8(36,64,117).to_linear().to_f32_array();
        let target = Color::srgb_u8(57,189,24).to_linear().to_f32_array();
        let mut changed = 0;
        for (a,b) in blue.colors.iter().zip(&green.colors) {
            if *a == base { assert_eq!(*b,target); changed += 1; }
            else { assert_eq!(a,b); }
        }
        assert!(changed > 100);
        assert!(changed < blue.colors.len());
    }
}
