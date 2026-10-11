//! Authenticated cave boulders: irregular weathered stone with an ice crust,
//! rather than identical gemstone meshes. No collision or support changes.
use super::*;
fn hash(x: i32, z: i32, k: u32) -> f32 {
    let mut v = (x as u32).wrapping_mul(0x9e3779b9)
        ^ (z as u32).wrapping_mul(0x85ebca6b)
        ^ k.wrapping_mul(0x7feb352d);
    v ^= v >> 16;
    v = v.wrapping_mul(0x846ca68b);
    v ^= v >> 16;
    (v & 65535) as f32 / 65535.
}
fn triangle(
    out: &mut SurfaceMeshData,
    p: [Vec3; 3],
    ice: bool,
    dark: bool,
    base: f32,
    height: f32,
    variation: f32,
) {
    // Material detail needs samples inside the broad stone planes. Subdivide
    // without displacing them so frost and seams cannot change the footprint.
    for q in [
        [p[0], (p[0] + p[1]) * 0.5, (p[0] + p[2]) * 0.5],
        [(p[0] + p[1]) * 0.5, p[1], (p[1] + p[2]) * 0.5],
        [(p[0] + p[2]) * 0.5, (p[1] + p[2]) * 0.5, p[2]],
        [
            (p[0] + p[1]) * 0.5,
            (p[1] + p[2]) * 0.5,
            (p[0] + p[2]) * 0.5,
        ],
    ] {
        stone_face(out, q, ice, dark, base, height, variation);
    }
}
fn stone_face(
    out: &mut SurfaceMeshData,
    p: [Vec3; 3],
    ice: bool,
    dark: bool,
    base: f32,
    height: f32,
    variation: f32,
) {
    let cross = (p[1] - p[0]).cross(p[2] - p[0]);
    // Intersecting shear planes can leave a tiny basal facet. Dropping
    // it at the ordinary weathered-stone threshold opens the closed mass.
    if cross.length_squared() < 1e-14 {
        return;
    }
    let normal = cross.normalize();
    let first = out.positions.len() as u32;
    out.positions.extend(p.map(|v| v.to_array()));
    out.normals.extend([normal.to_array(); 3]);
    out.uvs.extend([[0.; 2]; 3]);
    let shade = 0.65 + 0.35 * normal.dot(Vec3::new(-0.4, 0.8, 0.45).normalize()).max(0.);
    out.colors.extend(p.map(|p| {
        let frost = if ice {
            let crust = ((p.y - base) / height - 0.64) * 3.6;
            let broken_edge = (p.x * 0.71 + p.z * 0.49 + variation * 19.).sin() * 0.22
                + (p.x * 1.43 - p.z * 0.81).sin() * 0.09;
            (crust + normal.y.max(0.) * 0.20 + broken_edge).clamp(0., 1.)
        } else {
            0.
        };
        let stone = if ice {
            [0.18, 0.31, 0.36]
        } else if dark {
            [0.19, 0.25, 0.28]
        } else {
            [0.29, 0.265, 0.215]
        };
        let snow = [0.57, 0.74, 0.78];
        let grain = ((p.x * 1.73 + p.z * 2.11).sin() * (p.y * 2.3).sin()) * 0.016;
        let seam_distance = (p.x * 0.91 + p.y * 0.48 - p.z * 0.37
            + (p.z * 0.74 + variation * 11.).sin() * 0.32)
            .sin()
            .abs();
        let seam = (1. - seam_distance / 0.10).clamp(0., 1.)
            * (1. - frost)
            * (0.20 + normal.y.abs() * 0.12);
        let mineral = if ice {
            [0.39, 0.64, 0.69]
        } else {
            [0.39, 0.34, 0.25]
        };
        [
            ((stone[0] * (1. - frost) + snow[0] * frost + (mineral[0] - stone[0]) * seam)
                * shade
                * variation
                + grain)
                .clamp(0., 1.),
            ((stone[1] * (1. - frost) + snow[1] * frost + (mineral[1] - stone[1]) * seam)
                * shade
                * variation
                + grain)
                .clamp(0., 1.),
            ((stone[2] * (1. - frost) + snow[2] * frost + (mineral[2] - stone[2]) * seam)
                * shade
                * variation
                + grain)
                .clamp(0., 1.),
            1.,
        ]
    }));
    out.indices.extend([first, first + 1, first + 2]);
}
// Build broken stones from intersecting shear planes. Broad faces
// belong to a closed convex mass instead of repeated rings around a box.
fn fractured_mass(
    out: &mut SurfaceMeshData,
    bounds: [f32; 4],
    base: f32,
    height: f32,
    ice: bool,
    dark: bool,
    coords: [i32; 2],
) {
    let [w, e, n, s] = bounds;
    let [x, z] = coords;
    let phase = hash(x, z, 0) * std::f32::consts::TAU;
    let family = hash(x, z, 9);
    let height = height
        * if !ice {
            if family < 0.25 {
                0.40 + hash(x, z, 4) * 0.20
            } else {
                0.62 + hash(x, z, 4) * 0.38
            }
        } else if family < 0.34 {
            0.26 + hash(x, z, 4) * 0.14
        } else {
            0.48 + hash(x, z, 4) * 0.34
        };
    let tilt = if !ice {
        0.26 + hash(x, z, 6) * 0.28
    } else if family > 0.70 {
        0.32
    } else {
        0.13
    };
    let mut planes = vec![
        (Vec3::NEG_Y, 0.),
        (Vec3::Y, 1.),
        (Vec3::new(phase.cos() * tilt, 1., phase.sin() * tilt), 0.85),
        (Vec3::X, 0.98),
        (Vec3::NEG_X, 0.98),
        (Vec3::Z, 0.98),
        (Vec3::NEG_Z, 0.98),
    ];
    let sides = 5 + (hash(x, z, 8) * 3.99) as usize;
    for i in 0..sides {
        let angle = phase
            + std::f32::consts::TAU * (i as f32 + (hash(x, z, 10 + i as u32) - 0.5) * 0.22)
                / sides as f32;
        planes.push((
            Vec3::new(
                angle.cos(),
                if ice {
                    0.08 + hash(x, z, 30 + i as u32) * 0.28
                } else {
                    0.22 + hash(x, z, 30 + i as u32) * 0.26
                },
                angle.sin(),
            ),
            0.78 + hash(x, z, 50 + i as u32) * 0.19,
        ));
    }
    // Unequal basal bevels and one oblique fracture leave an asymmetric
    // crown, leaning slabs and wedges with broad, continuous shear faces.
    for i in 0..3 {
        let angle = phase + i as f32 * std::f32::consts::TAU / 3.;
        planes.push((
            Vec3::new(angle.cos(), if ice { -0.38 } else { -0.65 }, angle.sin()),
            0.64 + hash(x, z, 70 + i) * 0.18,
        ));
    }
    planes.push((
        Vec3::new(-phase.sin(), 0.42, phase.cos()),
        0.74 + hash(x, z, 80) * 0.12,
    ));
    if !ice {
        // Weathered blocks have broken shoulders rather than the elongated
        // glacial shear: an independent upper bed crosses the tilted crown.
        planes.push((
            Vec3::new(phase.sin() * 0.48, 1., -phase.cos() * 0.48),
            0.80 + hash(x, z, 81) * 0.15,
        ));
    }
    let mut vertices: Vec<Vec3> = Vec::new();
    for a in 0..planes.len() {
        for b in a + 1..planes.len() {
            for c in b + 1..planes.len() {
                let (na, da) = planes[a];
                let (nb, db) = planes[b];
                let (nc, dc) = planes[c];
                let determinant = na.dot(nb.cross(nc));
                if determinant.abs() < 0.00001 {
                    continue;
                }
                let mut p =
                    (nb.cross(nc) * da + nc.cross(na) * db + na.cross(nb) * dc) / determinant;
                if !p.is_finite() || planes.iter().any(|&(n, d)| n.dot(p) > d + 0.00001) {
                    continue;
                }
                if p.y.abs() < 0.00001 {
                    p.y = 0.;
                }
                if (p.y - 1.).abs() < 0.00001 {
                    p.y = 1.;
                }
                if !vertices.iter().any(|q| q.distance_squared(p) < 0.00000001) {
                    vertices.push(p);
                }
            }
        }
    }
    let radius = Vec3::new(
        (e - w) * 0.43 * (0.85 + hash(x, z, 3) * 0.15),
        height,
        (s - n)
            * 0.43
            * if ice && family < 0.34 {
                0.50 + hash(x, z, 7) * 0.25
            } else {
                0.64 + hash(x, z, 7) * 0.36
            },
    );
    // Rotate the unequal shear faces as one closed mass, then fit the complete
    // contour inside its authored footprint. Small stable offsets break the
    // tile-center procession without changing collision or walking support.
    let turn = |p: Vec3| {
        Vec3::new(
            p.x * phase.cos() - p.z * phase.sin(),
            p.y,
            p.x * phase.sin() + p.z * phase.cos(),
        )
    };
    let extent = vertices
        .iter()
        .map(|p| turn(*p * radius).abs())
        .fold(Vec3::ZERO, Vec3::max);
    let fit = ((e - w) * 0.44 / extent.x)
        .min((s - n) * 0.44 / extent.z)
        .min(1.);
    let stone_point = |p: Vec3| {
        let p = turn(p * radius);
        Vec3::new(p.x * fit, p.y, p.z * fit)
    };
    let first = out.positions.len();
    let variation = 0.89 + hash(x, z, 2) * 0.18;
    for (normal, distance) in planes {
        let mut face = vertices
            .iter()
            .copied()
            .filter(|p| (normal.dot(*p) - distance).abs() < 0.00005)
            .collect::<Vec<_>>();
        if face.len() < 3 {
            continue;
        }
        let center = face.iter().copied().sum::<Vec3>() / face.len() as f32;
        let u = normal
            .cross(if normal.y.abs() < 0.9 {
                Vec3::Y
            } else {
                Vec3::X
            })
            .normalize();
        let v = normal.cross(u).normalize();
        face.sort_by(|a, b| {
            let a = *a - center;
            let b = *b - center;
            a.dot(v)
                .atan2(a.dot(u))
                .total_cmp(&b.dot(v).atan2(b.dot(u)))
        });
        for i in 0..face.len() {
            triangle(
                out,
                [
                    stone_point(center),
                    stone_point(face[i]),
                    stone_point(face[(i + 1) % face.len()]),
                ],
                ice,
                dark,
                0.,
                height,
                variation,
            );
        }
    }
    for p in &mut out.positions[first..] {
        *p = (Vec3::from_array(*p)
            + Vec3::new(
                (w + e) * 0.5 + (hash(x, z, 91) - 0.5) * (e - w) * 0.08,
                base,
                (n + s) * 0.5 + (hash(x, z, 92) - 0.5) * (s - n) * 0.08,
            ))
        .to_array();
    }
}

pub(super) fn append(
    out: &mut SurfaceMeshData,
    bounds: [f32; 4],
    base: f32,
    height: f32,
    ice: bool,
    dark: bool,
    coords: [i32; 2],
) {
    fractured_mass(out, bounds, base, height, ice, dark, coords);
}
/// Fallen fragments on a proven dry boundary. The caller verifies both source
/// cells; the footprint stays within the adjoining half of the dry neighbor.
pub(super) fn edge_debris(
    out: &mut SurfaceMeshData,
    bounds: [f32; 4],
    low: f32,
    high: f32,
    side: usize,
    dark: bool,
    frozen: bool,
    g: &GridGeometry,
    origin: [i32; 2],
    coords: [i32; 2],
) {
    let unit = g.tile_width.min(g.tile_height);
    if high - low < unit * 0.35 || hash(coords[0], coords[1], 110 + side as u32) < 0.70 {
        return;
    }
    let [w, e, n, s] = bounds;
    let outward = match side {
        0 => Vec3::NEG_Z,
        1 => Vec3::X,
        2 => Vec3::Z,
        _ => Vec3::NEG_X,
    };
    let cliff = high - low > unit * 1.5;
    let cluster = hash(coords[0], coords[1], 130 + side as u32);
    let count = if cliff {
        2 + (cluster * 3.9) as usize
    } else {
        1 + (cluster * 1.9) as usize
    };
    let anchor = 0.30 + cluster * 0.40;
    for i in 0..count {
        let seed = hash(coords[0], coords[1], 151 + side as u32 * 7 + i as u32);
        let along = anchor + (hash(coords[0], coords[1], 170 + i as u32) - 0.5) * 0.20;
        let edge = match side {
            0 => Vec3::new(w + (e - w) * along, 0., n),
            1 => Vec3::new(e, 0., n + (s - n) * along),
            2 => Vec3::new(w + (e - w) * along, 0., s),
            _ => Vec3::new(w, 0., n + (s - n) * along),
        };
        let center = edge + outward * unit * (0.02 + seed * 0.10);
        let width = unit * (0.14 + seed * 0.30);
        let depth = unit * (0.20 + seed * 0.22);
        let height = unit * (0.12 + seed * 0.32) * if cliff { 1.0 } else { 0.5 };
        let base = if cliff { low } else { high - height * 0.60 };
        let first = out.positions.len();
        append(
            out,
            [
                center.x - width * 0.5,
                center.x + width * 0.5,
                center.z - depth * 0.5,
                center.z + depth * 0.5,
            ],
            base,
            height,
            frozen,
            dark,
            [
                coords[0].wrapping_mul(17) + side as i32,
                coords[1].wrapping_mul(19) + i as i32,
            ],
        );
        if cliff {
            // Join the bank's lower relief field without moving the dry datum.
            for p in &mut out.positions[first..] {
                *p = if frozen {
                    frozen_wall::frozen_point(Vec3::from_array(*p), g, origin)
                } else {
                    cave_cliff::rock_point(Vec3::from_array(*p), g, origin)
                }
                .to_array();
            }
            for j in (first..out.positions.len()).step_by(3) {
                let p = [out.positions[j], out.positions[j + 1], out.positions[j + 2]]
                    .map(Vec3::from_array);
                let normal = (p[1] - p[0])
                    .cross(p[2] - p[0])
                    .try_normalize()
                    .unwrap_or(Vec3::from_array(out.normals[j]));
                out.normals[j..j + 3].fill(normal.to_array());
            }
        }
        for j in first..out.positions.len() {
            out.uvs[j] =
                cave_cliff::stone_uv(Vec3::from_array(out.positions[j]), g, origin, frozen);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fractured_masses_are_closed_and_every_face_points_outward() {
        use std::collections::BTreeMap;
        for (coords, ice) in [[15, 20]]
            .into_iter()
            .chain((0..64).map(|x| [x, 20 + (x % 7) * 11]))
            .flat_map(|coords| [false, true].map(|ice| (coords, ice)))
        {
            let mut mesh = SurfaceMeshData::default();
            append(&mut mesh, [0., 16., 0., 12.], 6., 10., ice, false, coords);
            assert!(!mesh.indices.is_empty());
            assert!(
                mesh.positions.iter().all(|p| p[0] > 0.
                    && p[0] < 16.
                    && p[2] > 0.
                    && p[2] < 12.
                    && p[1] >= 6.
                    && p[1] <= 16.),
                "stone escaped its authored footprint at {coords:?}"
            );
            let interior = Vec3::new(8., 6.2, 6.);
            let key = |p: Vec3| p.to_array().map(|v| (v * 10000.).round() as i32);
            let mut edges = BTreeMap::new();
            for ids in mesh.indices.chunks_exact(3) {
                let p =
                    [ids[0], ids[1], ids[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
                assert!(
                    (p[1] - p[0]).cross(p[2] - p[0]).dot(p[0] - interior) > 0.,
                    "inward face at {coords:?}, ice={ice}"
                );
                for i in 0..3 {
                    let mut edge = [key(p[i]), key(p[(i + 1) % 3])];
                    edge.sort();
                    *edges.entry(edge).or_insert(0) += 1;
                }
            }
            assert!(
                edges.values().all(|&n| n == 2),
                "open edge at {coords:?}, ice={ice}: {:?}",
                edges
                    .iter()
                    .filter(|(_, n)| **n != 2)
                    .take(12)
                    .collect::<Vec<_>>()
            );
        }
    }
    #[test]
    fn scrolling_keeps_stone_shape_normals_and_material_fixed() {
        let mut a = SurfaceMeshData::default();
        let mut b = SurfaceMeshData::default();
        append(&mut a, [0., 16., 0., 12.], 6., 10., true, false, [10, 20]);
        append(&mut b, [-8., 8., -16., -4.], 6., 10., true, false, [10, 20]);
        assert_eq!(a.normals, b.normals);
        assert_eq!(a.colors, b.colors);
        assert_eq!(a.indices, b.indices);
        for (p, q) in a.positions.iter().zip(&b.positions) {
            assert!(
                (p[0] - q[0] - 8.).abs() < 0.00001
                    && (p[2] - q[2] - 16.).abs() < 0.00001
                    && p[1] == q[1]
            );
        }
    }
    #[test]
    fn stones_are_varied_contained_grounded_and_have_outward_faces() {
        for ice in [false, true] {
            let mut a = SurfaceMeshData::default();
            let mut b = SurfaceMeshData::default();
            append(&mut a, [0., 16., 0., 12.], 6., 10., ice, false, [10, 20]);
            append(&mut b, [0., 16., 0., 12.], 6., 10., ice, false, [11, 20]);
            assert_ne!(a.positions, b.positions);
            assert!(a.positions.iter().any(|p| p[1] == 6.));
            assert!(a.positions.iter().all(|p| p[0] > 0.
                && p[0] < 16.
                && p[2] > 0.
                && p[2] < 12.
                && p[1] >= 6.
                && p[1] <= 16.));
            assert!(
                a.normals
                    .iter()
                    .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 1e-5)
            );
            assert!(
                a.colors
                    .iter()
                    .all(|c| c.iter().all(|v| (0.0..=1.).contains(v)))
            );
        }
    }
}
