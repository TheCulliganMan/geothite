//! Low, layered foliage wholly within authored blocked tree plots.
use crate::mesh::SurfaceMeshData;
use bevy::prelude::Vec3;

pub(crate) const MAX_VERTICES: usize = 12_000;

fn seed(x: i32, z: i32, k: u32) -> f32 {
    let mut n = (x as u32).wrapping_mul(0x9e3779b9)
        ^ (z as u32).wrapping_mul(0x85ebca6b)
        ^ k.wrapping_mul(0x7feb352d);
    n ^= n >> 16;
    n = n.wrapping_mul(0x846ca68b);
    n ^= n >> 16;
    (n & 65535) as f32 / 65535.
}
// Broad overlapping color fields vary individual blades through continuous
// patches. Only foliage greens receive this treatment; petals and warm litter
// keep their authored palettes.
fn foliage_tint(color: [f32; 3], position: Vec3) -> [f32; 3] {
    if color[1] <= color[0] * 1.4 || color[1] <= color[2] * 1.5 {
        return color;
    }
    let broad = (position.x * 0.18 + position.z * 0.11).sin() * 0.55
        + (position.x * 0.071 - position.z * 0.21 + 1.8).sin() * 0.45;
    let detail = (position.x * 0.53 + position.z * 0.47).sin() * 0.5
        + (position.x * 1.07 - position.z * 0.71 + 0.9).sin() * 0.5;
    let temperature = (position.x * 0.12 - position.z * 0.16 + 2.1).sin();
    let light = broad * 0.052 + detail * 0.010;
    [
        color[0] * (1. + light + temperature * 0.018),
        color[1] * (1. + light + temperature * 0.004),
        color[2] * (1. + light - temperature * 0.018),
    ]
}

fn leaf(mesh: &mut SurfaceMeshData, root: Vec3, tip: Vec3, width: f32, color: [f32; 3]) {
    let color = foliage_tint(color, root.lerp(tip, 0.35));
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
/// Connect low ground growth across blocked plots, with large divided ferns
/// above it. Every source tree remains owned by the canopy renderer.
pub(crate) fn append(
    mesh: &mut SurfaceMeshData,
    bounds: [f32; 4],
    base: f32,
    tile: f32,
    coordinates: [i32; 2],
) {
    let [x, z] = coordinates;
    let width = bounds[1] - bounds[0];
    let depth = bounds[3] - bounds[2];
    let spacing = width.min(depth);
    if spacing <= 0. || tile <= 0. {
        return;
    }
    let reflect_x = seed(x, z, 3) > 0.5;
    let reflect_z = seed(x, z, 4) > 0.5;
    let origin_at = |mut u: f32, mut v: f32| {
        if reflect_x {
            u = 1. - u;
        }
        if reflect_z {
            v = 1. - v;
        }
        Vec3::new(bounds[0] + u * width, base, bounds[2] + v * depth)
    };
    // Large fronds belong between the trunks, with independent ground roots.
    // Their horizontal reach respects the plot; height remains large enough
    // to read beneath the mature canopy when a full plot is instanced.
    for plant in 0..6 {
        let k = 30 + plant * 9;
        let u = [0.22, 0.61, 0.38, 0.80, 0.22, 0.65][plant as usize] + (seed(x, z, k) - 0.5) * 0.05;
        let v =
            [0.21, 0.22, 0.47, 0.49, 0.80, 0.79][plant as usize] + (seed(x, z, k + 1) - 0.5) * 0.05;
        let origin = origin_at(u, v);
        let edge = (origin.x - bounds[0])
            .min(bounds[1] - origin.x)
            .min(origin.z - bounds[2])
            .min(bounds[3] - origin.z);
        let reach = (tile * 0.80).min(edge * 0.88);
        let height = (tile * (1.70 + 0.40 * seed(x, z, k + 2))).min(spacing * 1.20);
        fern(
            mesh,
            bounds,
            origin,
            reach,
            height,
            seed(x, z, k + 3) * std::f32::consts::TAU,
        );
    }
    // A single divided frond approaches each shared edge at its midpoint.
    // Adjacent plots meet at these few points; the remaining border and the
    // ground between the fern arches remain open, rather than a green sheet.
    for edge in 0..4 {
        let (u, v, du, dv) = match edge {
            0 => (0.30, 0.50, -0.30, 0.),
            1 => (0.70, 0.50, 0.30, 0.),
            2 => (0.50, 0.30, 0., -0.30),
            _ => (0.50, 0.70, 0., 0.30),
        };
        let origin = Vec3::new(bounds[0] + u * width, base, bounds[2] + v * depth);
        frond(
            mesh,
            bounds,
            origin,
            Vec3::new(du * width, 0., dv * depth),
            tile.min(spacing) * 0.85,
        );
    }
    for plant in 0..16 {
        let k = 100 + plant * 13;
        let origin = origin_at(0.12 + seed(x, z, k) * 0.76, 0.12 + seed(x, z, k + 1) * 0.76);
        let edge = (origin.x - bounds[0])
            .min(bounds[1] - origin.x)
            .min(origin.z - bounds[2])
            .min(bounds[3] - origin.z);
        let size = (tile * 0.26).min(spacing * 0.16).min(edge * 0.68);
        let angle = seed(x, z, k + 2) * std::f32::consts::TAU;
        if plant % 3 == 0 {
            flower(mesh, origin, size, angle, seed(x, z, k + 3));
        } else {
            broadleaf(
                mesh,
                origin,
                size,
                (tile * (0.80 + 0.50 * seed(x, z, k + 4))).min(spacing * 0.70),
                angle,
                seed(x, z, k + 3),
            );
        }
    }
    // Individual flat leaves, not a raised moss pile. Warm muted litter sits
    // between the green silhouettes and makes the exposed ground legible.
    let litter = 18 + (seed(x, z, 800) * 10.) as u32;
    for piece in 0..litter {
        let k = 900 + piece * 7;
        let u = 0.08 + seed(x, z, k) * 0.84;
        let v = 0.08 + seed(x, z, k + 1) * 0.84;
        if u > 0.68 && v > 0.68 {
            continue;
        }
        let origin = origin_at(u, v);
        let edge = (origin.x - bounds[0])
            .min(bounds[1] - origin.x)
            .min(origin.z - bounds[2])
            .min(bounds[3] - origin.z);
        let reach = (tile * (0.045 + seed(x, z, k + 2) * 0.045))
            .min(spacing * 0.04)
            .min(edge * 0.6);
        let angle = seed(x, z, k + 3) * std::f32::consts::TAU;
        let tone = seed(x, z, k + 4);
        leaf(
            mesh,
            origin,
            origin + Vec3::new(angle.cos(), 0.025, angle.sin()) * reach,
            reach * 0.26,
            [0.15 + tone * 0.13, 0.10 + tone * 0.07, 0.035 + tone * 0.035],
        );
    }
}

fn broadleaf(
    mesh: &mut SurfaceMeshData,
    origin: Vec3,
    size: f32,
    height: f32,
    phase: f32,
    tone: f32,
) {
    // Three separate, unequal long leaves form an open plant, rather than the
    // seven-layer radial crowns that read as dense mounds under every trunk.
    for blade in 0..3 {
        let angle = phase + blade as f32 * 2.13;
        let forward = Vec3::new(angle.cos(), 0., angle.sin());
        let reach = size * (0.65 + blade as f32 * 0.12);
        let shoot =
            origin + forward * size * 0.12 + Vec3::Y * height * (0.15 + blade as f32 * 0.12);
        leaf(mesh, origin, shoot, size * 0.018, [0.08, 0.19, 0.035]);
        leaf(
            mesh,
            shoot,
            origin + forward * reach + Vec3::Y * height * (0.65 + blade as f32 * 0.175),
            size * 0.36,
            [0.045 + tone * 0.025, 0.24 + tone * 0.07, 0.055],
        );
    }
}

fn flower(mesh: &mut SurfaceMeshData, origin: Vec3, size: f32, phase: f32, tone: f32) {
    let forward = Vec3::new(phase.cos(), 0., phase.sin());
    let crown = origin + forward * size * 0.12 + Vec3::Y * size * 0.80;
    leaf(mesh, origin, crown, size * 0.018, [0.09, 0.23, 0.04]);
    for sign in [-1., 1.] {
        leaf(
            mesh,
            origin + Vec3::Y * size * 0.22,
            origin + forward * sign * size * 0.42 + Vec3::Y * size * 0.41,
            size * 0.10,
            [0.07, 0.28, 0.045],
        );
    }
    let color = if tone < 0.45 {
        [0.81, 0.65, 0.16]
    } else if tone < 0.75 {
        [0.77, 0.78, 0.58]
    } else {
        [0.48, 0.37, 0.65]
    };
    for petal in 0..5 {
        let angle = phase + petal as f32 * std::f32::consts::TAU / 5.;
        leaf(
            mesh,
            crown,
            crown + Vec3::new(angle.cos(), 0.12, angle.sin()) * size * 0.28,
            size * 0.11,
            color,
        );
    }
}

fn bounded_leaf(
    mesh: &mut SurfaceMeshData,
    bounds: [f32; 4],
    root: Vec3,
    tip: Vec3,
    width: f32,
    color: [f32; 3],
) {
    let delta = tip - root;
    let mut fit = 1.0_f32;
    for (d, p, low, high) in [
        (delta.x, root.x, bounds[0], bounds[1]),
        (delta.z, root.z, bounds[2], bounds[3]),
    ] {
        if d > 0. {
            fit = fit.min((high - p) / d);
        }
        if d < 0. {
            fit = fit.min((low - p) / d);
        }
    }
    let tip = root + delta * fit;
    let middle = root.lerp(tip, 0.52);
    let side = Vec3::new(-(tip - root).z, 0., (tip - root).x).normalize_or_zero();
    let mut width = width;
    for (d, p, low, high) in [
        (side.x, middle.x, bounds[0], bounds[1]),
        (side.z, middle.z, bounds[2], bounds[3]),
    ] {
        if d.abs() > 0. {
            width = width.min((p - low).min(high - p) * 0.98 / d.abs());
        }
    }
    if width > f32::EPSILON {
        leaf(mesh, root, tip, width, color);
    }
}

fn frond(mesh: &mut SurfaceMeshData, bounds: [f32; 4], origin: Vec3, reach: Vec3, height: f32) {
    let side = Vec3::new(-reach.z, 0., reach.x).normalize();
    let length = reach.length();
    let stem = |t: f32| {
        origin
            + reach * t
            + Vec3::Y * height * ((t * std::f32::consts::PI).sin().max(0.) + t * 0.12)
    };
    for segment in 0..6 {
        bounded_leaf(
            mesh,
            bounds,
            stem(segment as f32 / 6.),
            stem((segment + 1) as f32 / 6.),
            length * 0.014,
            [0.075, 0.15, 0.035],
        );
    }
    // Six paired leaflets give the arch its recognizable feathered outline.
    // Their lengths taper towards the drooping tip, leaving air between rows.
    for row in 0..6 {
        let t = 0.12 + row as f32 * 0.12;
        let root = stem(t);
        for sign in [-1., 1.] {
            let tip = root
                + reach * 0.055
                + side * sign * length * (0.29 - 0.20 * t)
                + Vec3::Y * height * 0.018;
            bounded_leaf(
                mesh,
                bounds,
                root,
                tip,
                length * (0.050 - 0.026 * t),
                [0.045, 0.17 + row as f32 * 0.007, 0.038],
            );
        }
    }
    bounded_leaf(
        mesh,
        bounds,
        stem(0.82),
        stem(1.),
        length * 0.065,
        [0.055, 0.20, 0.043],
    );
}

fn fern(
    mesh: &mut SurfaceMeshData,
    bounds: [f32; 4],
    origin: Vec3,
    size: f32,
    height: f32,
    phase: f32,
) {
    for shoot in 0..6 {
        let angle = phase + shoot as f32 * 2.399963;
        let reach = Vec3::new(angle.cos(), 0., angle.sin()) * size * (0.83 + shoot as f32 * 0.032);
        frond(
            mesh,
            bounds,
            origin,
            reach,
            height * (0.90 + shoot as f32 * 0.018),
        );
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
            assert!(m.indices.len() > 1000);
            assert!(m.positions.iter().all(|p| p[0] >= 0.
                && p[0] <= 32.
                && p[2] >= 0.
                && p[2] <= 32.
                && p[1] >= 3.
                && p[1] < 39.));
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
    #[test]
    fn varied_plots_keep_foliage_grounded_and_connect_at_plot_edges() {
        let mut signatures = std::collections::HashSet::new();
        for bounds in [
            [0., 32., 0., 32.],
            [-7., -5., 3., 35.],
            [10., 58., -20., -17.],
            [0., 0.2, 0., 0.4],
        ] {
            for x in -12..12 {
                let mut mesh = SurfaceMeshData::default();
                append(&mut mesh, bounds, 3., 16., [x, -3]);
                assert!(!mesh.positions.is_empty());
                assert!(mesh.positions.len() <= MAX_VERTICES);
                assert_eq!(mesh.positions.len(), mesh.normals.len());
                assert_eq!(mesh.positions.len(), mesh.uvs.len());
                assert!(mesh.uvs.iter().all(|uv| *uv == [0., 0.]));
                assert!(mesh
                    .positions
                    .iter()
                    .all(|p| p.iter().all(|v| v.is_finite())
                        && p[0] >= bounds[0]
                        && p[0] <= bounds[1]
                        && p[2] >= bounds[2]
                        && p[2] <= bounds[3]
                        && p[1] >= 3.
                        && p[1] < 39.));
                assert!(mesh.positions.iter().any(|p| p[1] == 3.));
                if bounds == [0., 32., 0., 32.] {
                    let height = mesh.positions.iter().map(|p| p[1] - 3.).fold(0., f32::max);
                    assert!(height > 4., "undergrowth must read above ground cover");
                    signatures.insert((mesh.positions.len(), (height * 100.) as i32));
                    assert!(
                        height > 16. * 1.6,
                        "large fern silhouettes must rise above the low ground plants"
                    );
                    for axis in [0, 2] {
                        for edge in [0., 32.] {
                            assert!(
                                mesh.positions.iter().any(|p| p[axis] == edge),
                                "low growth must meet each plot edge to connect between trees"
                            );
                        }
                    }
                }
                for ids in mesh.indices.chunks_exact(3) {
                    let p = [ids[0], ids[1], ids[2]]
                        .map(|i| Vec3::from_array(mesh.positions[i as usize]));
                    let n = Vec3::from_array(mesh.normals[ids[0] as usize]);
                    assert!(n.is_finite() && (n.length() - 1.).abs() < 0.001);
                    assert!((p[1] - p[0]).cross(p[2] - p[0]).dot(n) > 0.);
                }
            }
        }
        assert!(
            signatures.len() > 12,
            "plots need differing density and silhouettes"
        );
    }
    #[test]
    fn connected_ferns_leave_open_borders_without_ground_planes() {
        let mut mesh = SurfaceMeshData::default();
        append(&mut mesh, [0., 32., 0., 32.], 3., 16., [6, 9]);
        for p in &mesh.positions {
            if p[0] == 0. || p[0] == 32. {
                assert!((p[2] - 16.).abs() < 0.001);
            }
            if p[2] == 0. || p[2] == 32. {
                assert!((p[0] - 16.).abs() < 0.001);
            }
        }
        for ids in mesh.indices.chunks_exact(3) {
            let p = [ids[0], ids[1], ids[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
            let cross = (p[1] - p[0]).cross(p[2] - p[0]);
            assert!(
                cross.y.abs() * 0.5 < 16. * 16. * 0.03,
                "understory should retain individual leaf silhouettes, not broad ground planes"
            );
        }
    }
    #[test]
    fn leaf_color_varies_in_restrained_smooth_patches() {
        let green = [0.055, 0.20, 0.043];
        let mut min = 1.0_f32;
        let mut max = 0.0_f32;
        for x in -32..32 {
            for z in -32..32 {
                let p = Vec3::new(x as f32, 0., z as f32);
                let color = foliage_tint(green, p);
                let nearby = foliage_tint(green, p + Vec3::new(0.1, 0., 0.1));
                for channel in 0..3 {
                    let ratio = color[channel] / green[channel];
                    assert!((0.92..=1.08).contains(&ratio));
                    assert!((color[channel] - nearby[channel]).abs() < green[channel] * 0.005);
                    min = min.min(ratio);
                    max = max.max(ratio);
                }
            }
        }
        assert!(
            max - min > 0.08,
            "local foliage must have visible tonal variety"
        );
        for color in [
            [0.81, 0.65, 0.16],
            [0.77, 0.78, 0.58],
            [0.48, 0.37, 0.65],
            [0.23, 0.14, 0.05],
        ] {
            assert_eq!(foliage_tint(color, Vec3::new(8., 0., 12.)), color);
        }
    }
}
