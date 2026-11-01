//! Two tapered upper-lip patches fitted to the source face instead of an oval.
use crate::{mesh::SurfaceMeshData, open_people::PersonRig};
use bevy::prelude::*;

pub(crate) fn append(
    mesh: &mut SurfaceMeshData,
    rig: &PersonRig,
    center: Vec3,
    half: Vec3,
    rgb: [u8; 3],
) {
    const COLS: usize = 32;
    const ROWS: usize = 8;
    let rgb = rgb.map(|v| Color::srgb_u8(v, v, v).to_linear().red);
    for side in [-1., 1.] {
        let point = |u: f32, v: f32| {
            let fullness = (1. - u).powf(0.65);
            let x = center.x + side * half.x * (0.004 + 0.45 * u);
            let y = center.y - half.y * (0.36 + 0.055 * u)
                + half.y * (0.004 + 0.052 * fullness) * (1. - 2. * v);
            let z = crate::npc_appearance::head_surface_depth(rig, center, half, x, y)
                + 0.009
                + half.z * 0.045 * fullness * (std::f32::consts::PI * v).sin();
            Vec3::new(x, y, z)
        };
        let first = mesh.positions.len() as u32;
        for row in 0..=ROWS {
            for col in 0..=COLS {
                let u = col as f32 / COLS as f32;
                let v = row as f32 / ROWS as f32;
                let du = point((u + 0.002).min(1.), v) - point((u - 0.002).max(0.), v);
                let dv = point(u, (v + 0.002).min(1.)) - point(u, (v - 0.002).max(0.));
                let shade = 0.91 + 0.09 * (std::f32::consts::PI * v).sin();
                mesh.positions.push(point(u, v).to_array());
                mesh.normals
                    .push((dv.cross(du) * side).normalize().to_array());
                mesh.colors
                    .push([rgb[0] * shade, rgb[1] * shade, rgb[2] * shade, 1.]);
                mesh.uvs.push([0.; 2]);
            }
        }
        for row in 0..ROWS {
            for col in 0..COLS {
                let a = first + (row * (COLS + 1) + col) as u32;
                let b = a + (COLS + 1) as u32;
                for mut face in [[a, b, a + 1], [a + 1, b, b + 1]] {
                    if side < 0. {
                        face.swap(1, 2);
                    }
                    mesh.indices.extend(face);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires locally supplied Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_moustaches_follow_the_upper_lip_without_floating_or_covering_eyes() {
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let rig =
            PersonRig::parse(&std::fs::read(root.join("people/kenney/human-male.glb")).unwrap())
                .unwrap();
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for ((&p, ids), w) in rig.mesh.positions.iter().zip(&rig.ids).zip(&rig.weights) {
            if (0..4)
                .filter(|&i| ids[i] as usize == rig.head_joint)
                .map(|i| w[i])
                .sum::<f32>()
                > 0.5
            {
                min = min.min(Vec3::from_array(p));
                max = max.max(Vec3::from_array(p));
            }
        }
        let center = (min + max) * 0.5;
        let half = (max - min) * 0.5;
        let mut mesh = SurfaceMeshData::default();
        append(&mut mesh, &rig, center, half, [173, 184, 181]);
        for (&p, &n) in mesh.positions.iter().zip(&mesh.normals) {
            let p = Vec3::from_array(p);
            let n = Vec3::from_array(n);
            assert!(p.is_finite() && n.is_finite() && (n.length() - 1.).abs() < 1e-5);
            assert!(p.y < center.y - half.y * 0.29 && p.y > center.y - half.y * 0.49);
            assert!((p.x - center.x).abs() > half.x * 0.003);
            let gap = p.z - crate::npc_appearance::head_surface_depth(&rig, center, half, p.x, p.y);
            assert!(
                (0.008..0.035).contains(&gap),
                "floating upper-lip patch: {gap}"
            );
        }
        for face in mesh.indices.chunks_exact(3) {
            let p =
                [face[0], face[1], face[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
            let n = (p[1] - p[0]).cross(p[2] - p[0]);
            let shading: Vec3 = face
                .iter()
                .map(|&i| Vec3::from_array(mesh.normals[i as usize]))
                .sum();
            assert!(n.length_squared() > 1e-12 && n.dot(shading) > 0.);
        }
    }
}
