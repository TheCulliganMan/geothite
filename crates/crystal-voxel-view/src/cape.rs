//! A closed, softly folded cape supported by Lance's torso skin palette.
use crate::{
    mesh::SurfaceMeshData,
    npc_appearance::{CostumeMesh, Look},
    open_people::PersonRig,
};
use bevy::prelude::*;

pub(crate) fn build(rig: &PersonRig, look: Look) -> Option<CostumeMesh> {
    if look.texture_key != "lance" || !look.accessories.contains("redcape") {
        return None;
    }
    const ROWS: usize = 18;
    const COLS: usize = 32;
    let mut out = CostumeMesh {
        surface: SurfaceMeshData::default(),
        joints: vec![],
        weights: vec![],
    };
    let count = (ROWS + 1) * (COLS + 1);
    let mut roots = Vec::with_capacity(COLS + 1);
    for col in 0..=COLS {
        let u = col as f32 / COLS as f32 * 2. - 1.;
        let angle = u * 1.15;
        let dir = Vec2::new(angle.sin(), -angle.cos());
        let radius = crate::garment_fit::section(rig, 2.48, dir)?.radius + 0.028;
        roots.push(Vec3::new(dir.x * radius, 2.48, 0.05 + dir.y * radius));
    }
    for layer in 0..2 {
        for row in 0..=ROWS {
            for col in 0..=COLS {
                let t = row as f32 / ROWS as f32;
                let u = col as f32 / COLS as f32 * 2. - 1.;
                let ease = t * t * (3. - 2. * t);
                let wave = (u * std::f32::consts::PI * 3.).cos();
                let p = Vec3::new(
                    roots[col].x * (1. - ease) + u * 0.66 * ease,
                    2.48 - 1.66 * t + 0.025 * t.powi(6) * wave,
                    roots[col].z * (1. - ease) + (-0.58 + 0.035 * wave) * ease
                        - 0.055 * (std::f32::consts::PI * t).sin()
                        + if layer == 0 { -0.006 } else { 0.006 },
                );
                out.surface.positions.push(p.to_array());
                out.surface.normals.push([0.; 3]);
                let base = if layer == 0 {
                    [157, 33, 43]
                } else {
                    [86, 25, 36]
                };
                let shade = 0.94 + 0.06 * wave;
                let rgb = base.map(|v| (v as f32 * shade).round() as u8);
                out.surface.colors.push(
                    Color::srgb_u8(rgb[0], rgb[1], rgb[2])
                        .to_linear()
                        .to_f32_array(),
                );
                out.surface.uvs.push([col as f32 / COLS as f32, t]);
                let hips = 0.30 * ease;
                out.joints
                    .push([rig.chest_joint as u16, rig.hips_joint as u16, 0, 0]);
                out.weights.push([1. - hips, hips, 0., 0.]);
            }
        }
    }
    let at = |row: usize, col: usize| row * (COLS + 1) + col;
    let mut boundary = std::collections::BTreeMap::new();
    for row in 0..ROWS {
        for col in 0..COLS {
            let a = at(row, col);
            let b = at(row, col + 1);
            let c = at(row + 1, col + 1);
            let d = at(row + 1, col);
            for triangle in [[a, b, c], [a, c, d]] {
                out.surface.indices.extend(triangle.map(|i| i as u32));
                out.surface.indices.extend(
                    [
                        triangle[0] + count,
                        triangle[2] + count,
                        triangle[1] + count,
                    ]
                    .map(|i| i as u32),
                );
                for side in 0..3 {
                    let a = triangle[side];
                    let b = triangle[(side + 1) % 3];
                    let entry = boundary.entry((a.min(b), a.max(b))).or_insert((0, a, b));
                    entry.0 += 1;
                }
            }
        }
    }
    for (_, (uses, a, b)) in boundary {
        if uses == 1 {
            out.surface
                .indices
                .extend([b, a, a + count, b, a + count, b + count].map(|i| i as u32));
        }
    }
    for face in out.surface.indices.chunks_exact(3) {
        let p = [face[0], face[1], face[2]]
            .map(|i| Vec3::from_array(out.surface.positions[i as usize]));
        let n = (p[1] - p[0]).cross(p[2] - p[0]);
        for &i in face {
            let old = Vec3::from_array(out.surface.normals[i as usize]);
            out.surface.normals[i as usize] = (old + n).to_array();
        }
    }
    for n in &mut out.surface.normals {
        *n = Vec3::from_array(*n).normalize().to_array();
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires supplied Lance rig in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_cape_is_closed_and_stays_in_production_bounds_through_animation() {
        use crate::new_bark_actors::character_meshes::CharacterKind::{Koga, Lance};
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let rig =
            PersonRig::parse(&std::fs::read(root.join("people/kenney/human-male.glb")).unwrap())
                .unwrap();
        let cape = build(&rig, crate::npc_appearance::look(Lance)).unwrap();
        let mesh = &cape.surface;
        assert!(build(&rig, crate::npc_appearance::look(Koga)).is_none());
        assert_eq!(mesh.positions.len(), 1254);
        let mut edges = std::collections::BTreeMap::new();
        for f in mesh.indices.chunks_exact(3) {
            let p = [f[0], f[1], f[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
            let n = (p[1] - p[0]).cross(p[2] - p[0]);
            assert!(n.is_finite() && n.length() > 1e-8);
            let normal = f
                .iter()
                .map(|&i| Vec3::from_array(mesh.normals[i as usize]))
                .sum::<Vec3>();
            assert!(n.dot(normal) > 0.);
            for i in 0..3 {
                let a = f[i];
                let b = f[(i + 1) % 3];
                let entry = edges.entry((a.min(b), a.max(b))).or_insert((0, 0));
                entry.0 += 1;
                entry.1 += if a < b { 1 } else { -1 };
            }
        }
        assert!(edges.values().all(|&(n, d)| n == 2 && d == 0));
        for (n, w) in mesh.normals.iter().zip(&cape.weights) {
            assert!((Vec3::from_array(*n).length() - 1.).abs() < 1e-5);
            assert!((w.iter().sum::<f32>() - 1.).abs() < 1e-5);
        }
        fn world(i: usize, pose: &[Transform], parents: &[Option<usize>]) -> Mat4 {
            parents[i]
                .map(|p| world(p, pose, parents))
                .unwrap_or(Mat4::IDENTITY)
                * pose[i].to_matrix()
        }
        for clip in ["Idle", "Run"] {
            for step in 0..=60 {
                let mut pose = rig.nodes.clone();
                rig.sample(clip, rig.duration(clip) * step as f32 / 60., &mut pose);
                let inverse = world(rig.mesh_node, &pose, &rig.parents).inverse();
                let skin = rig
                    .skin_nodes
                    .iter()
                    .zip(&rig.inverse_binds)
                    .map(|(&i, bind)| inverse * world(i, &pose, &rig.parents) * *bind)
                    .collect::<Vec<_>>();
                for ((p, j), w) in mesh.positions.iter().zip(&cape.joints).zip(&cape.weights) {
                    let p = Vec3::from_array(*p);
                    let moved = (0..4)
                        .map(|i| skin[j[i] as usize].transform_point3(p) * w[i])
                        .sum::<Vec3>();
                    assert!(moved.is_finite());
                    assert!(
                        moved.cmpge(rig.bounds.0 - Vec3::splat(0.6)).all()
                            && moved.cmple(rig.bounds.1 + Vec3::splat(0.6)).all()
                    );
                }
            }
        }
    }
}
