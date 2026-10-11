//! Low, layered foliage wholly within authored blocked tree plots.
use crate::mesh::SurfaceMeshData;
use bevy::prelude::Vec3;

fn seed(x: i32, z: i32, k: u32) -> f32 {
    let mut n = (x as u32).wrapping_mul(0x9e3779b9)
        ^ (z as u32).wrapping_mul(0x85ebca6b)
        ^ k.wrapping_mul(0x7feb352d);
    n ^= n >> 16;
    n = n.wrapping_mul(0x846ca68b);
    n ^= n >> 16;
    (n & 65535) as f32 / 65535.
}
fn leaf(mesh: &mut SurfaceMeshData, root: Vec3, tip: Vec3, width: f32, color: [f32; 3]) {
    let axis = tip - root;
    let side = Vec3::new(-axis.z, 0., axis.x).normalize_or_zero() * width;
    let middle = root.lerp(tip, 0.52);
    let ridge = middle + Vec3::Y * width * 0.3;
    for mut p in [
        [root, middle - side, ridge],
        [middle - side, tip, ridge],
        [tip, middle + side, ridge],
        [middle + side, root, ridge],
    ] {
        let mut normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize();
        if normal.y < 0. {
            p.swap(1, 2);
            normal = -normal;
        }
        let start = mesh.positions.len() as u32;
        mesh.positions.extend(p.map(|p| p.to_array()));
        mesh.normals.extend([normal.to_array(); 3]);
        mesh.uvs.extend([[0.; 2]; 3]);
        let shade = 0.65 + 0.35 * normal.dot(Vec3::new(0.45, 0.80, 0.55).normalize()).max(0.);
        mesh.colors
            .extend([[color[0] * shade, color[1] * shade, color[2] * shade, 1.]; 3]);
        mesh.indices.extend([start, start + 1, start + 2]);
    }
}
pub(crate) fn append(
    mesh: &mut SurfaceMeshData,
    bounds: [f32; 4],
    base: f32,
    tile: f32,
    coordinates: [i32; 2],
) {
    let [x, z] = coordinates;
    let spacing = (bounds[1] - bounds[0]).min(bounds[3] - bounds[2]);
    let center = Vec3::new(
        (bounds[0] + bounds[1]) * 0.5,
        base,
        (bounds[2] + bounds[3]) * 0.5,
    );
    let phase = seed(x, z, 1) * std::f32::consts::TAU;
    // Low broadleaf rosettes spread below and between the upright ferns.
    // Their centers and reach are bounded by plot spacing, so even a small
    // authored plot cannot spill leaves into the adjacent walking corridor.
    for patch in 0..16 {
        let origin = center + Vec3::new(
            seed(x,z,200+patch*3)-0.5,
            0.,
            seed(x,z,201+patch*3)-0.5,
        ) * spacing * 0.60;
        let length = (tile * 0.34).min(spacing * 0.12)
            * (0.75 + 0.25 * seed(x,z,202+patch*3));
        let turn = phase + seed(x,z,300+patch) * std::f32::consts::TAU;
        for blade in 0..7 {
            let a = turn + blade as f32 * 2.399963;
            let reach = length * (0.75 + 0.25 * seed(x,z,400+patch*7+blade));
            let tip = origin + Vec3::new(a.cos()*reach,
                reach*(0.35+0.25*seed(x,z,600+patch*7+blade)),a.sin()*reach);
            let tone=seed(x,z,800+patch*7+blade);
            leaf(mesh,origin,tip,reach*0.38,
                [0.035+tone*0.025,0.12+tone*0.08,0.035+tone*0.018]);
        }
    }
    for clump in 0..8 {
        let angle = phase + clump as f32 * 2.399963;
        let radius = spacing * (0.17 + 0.12 * seed(x, z, 10 + clump));
        let origin = center + Vec3::new(angle.cos(), 0., angle.sin()) * radius;
        let size = tile * (0.24 + 0.12 * seed(x, z, 30 + clump));
        for shoot in 0..5 {
            let a = angle + shoot as f32 * 2.399963;
            let forward = Vec3::new(a.cos(), 0., a.sin());
            let side = Vec3::new(-a.sin(), 0., a.cos());
            let length = size * (0.75 + 0.35 * seed(x, z, 50 + clump * 7 + shoot));
            let stem = |t: f32| {
                origin
                    + forward * length * t * 0.65
                    + Vec3::Y * length * (1.15 * t - 0.43 * t * t) * 2.4
            };
            for segment in 0..4 {
                leaf(
                    mesh,
                    stem(segment as f32 / 4.),
                    stem((segment + 1) as f32 / 4.),
                    length * 0.018,
                    [0.13, 0.23, 0.035],
                );
            }
            for row in 0..3 {
                let t = (row as f32 + 1.) / 4.;
                let root = stem(t);
                for sign in [-1., 1.] {
                    let tip = root
                        + forward * length * 0.17
                        + side * sign * length * (0.35 - 0.18 * t)
                        + Vec3::Y * length * 0.05;
                    let color = if row == 2 {
                        [0.13, 0.31, 0.055]
                    } else {
                        [0.075, 0.23, 0.035]
                    };
                    leaf(mesh, root, tip, length * (0.08 - 0.025 * t), color);
                }
            }
            leaf(
                mesh,
                stem(0.75),
                origin + forward * length + Vec3::Y * length * 0.72 * 2.4,
                length * 0.11,
                [0.11, 0.29, 0.045],
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn low_foliage_is_rooted_repeatable_and_confined_to_the_blocked_plot() {
        for x in -5..5 {
            let mut m = SurfaceMeshData::default();
            append(&mut m, [0., 32., 0., 32.], 3., 16., [x, 8]);
            assert!(m.indices.len() > 3000);
            assert!(m.positions.iter().all(|p| p[0] > 0.
                && p[0] < 32.
                && p[2] > 0.
                && p[2] < 32.
                && p[1] >= 3.
                && p[1] < 19.));
            for ids in m.indices.chunks_exact(3) {
                let p = [ids[0], ids[1], ids[2]].map(|i| Vec3::from_array(m.positions[i as usize]));
                let n = Vec3::from_array(m.normals[ids[0] as usize]);
                assert!(n.is_finite() && (n.length() - 1.).abs() < 0.001);
                assert!((p[1] - p[0]).cross(p[2] - p[0]).dot(n) > 0.);
            }
            let mut repeat = SurfaceMeshData::default();
            append(&mut repeat, [0., 32., 0., 32.], 3., 16., [x, 8]);
            assert_eq!(m.positions, repeat.positions);
        }
    }
}
