//! Curved, folded grass rooted in verified encounter cells. Gameplay support
//! and encounter ownership stay in the source grid.
use crate::mesh::SurfaceMeshData;
use bevy::prelude::Vec3;

fn random(x: i32, z: i32, i: u32) -> f32 {
    let mut n = (x as u32).wrapping_mul(0x9e3779b9)
        ^ (z as u32).wrapping_mul(0x85ebca6b)
        ^ i.wrapping_mul(0x7feb352d);
    n ^= n >> 16;
    n = n.wrapping_mul(0x846ca68b);
    n ^= n >> 16;
    (n & 65535) as f32 / 65535.
}
pub(crate) fn append(
    out: &mut SurfaceMeshData,
    bounds: [f32; 4],
    base: f32,
    height: f32,
    coordinates: [i32; 2],
    detailed: bool,
) {
    append_patch(out, bounds, base, height, coordinates, detailed, [false; 4]);
}

/// Verified neighboring encounter cells allow foliage to cross their shared
/// edge. Outer edges remain contained, including at corners of a patch.
pub(crate) fn append_patch(
    out: &mut SurfaceMeshData,
    bounds: [f32; 4],
    base: f32,
    height: f32,
    coordinates: [i32; 2],
    detailed: bool,
    joined: [bool; 4], // west, east, north, south
) {
    append_styled_patch(out, bounds, base, height, coordinates, detailed, joined, PatchKind::Meadow);
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum PatchKind {
    Meadow,
    ParkShort,
    ParkLong,
}

impl PatchKind {
    /// Source-pixel height before deterministic taper. Shared by the overworld
    /// and retained battle scenery (whose world scale is one sixteenth).
    pub(crate) fn height(self, tile_height: f32) -> f32 {
        tile_height * match self {
            Self::Meadow | Self::ParkShort => 3.0,
            Self::ParkLong => 4.75,
        }
    }
}

pub(crate) fn append_styled_patch(
    out: &mut SurfaceMeshData,
    bounds: [f32; 4],
    base: f32,
    height: f32,
    coordinates: [i32; 2],
    detailed: bool,
    joined: [bool; 4],
    kind: PatchKind,
) {
    let [w, e, n, s] = bounds;
    let unit = (e - w).min(s - n);
    // Construct locally before translating: normals and colors stay exact when
    // the visible source grid scrolls, even at large world coordinates.
    let connected = joined.iter().any(|v| *v);
    let count = if connected {
        if detailed { 64 } else { 32 }
    } else if detailed {
        32
    } else {
        16
    };
    for blade in 0..count {
        let seed = random(coordinates[0], coordinates[1], blade * 7);
        // Distribute shared-edge coverage without planting a straight row.
        // Stagger roots in a band while keeping both seam endpoints covered.
        let edge_count = if detailed { 8 } else { 4 };
        let edge = joined
            .iter()
            .enumerate()
            .filter(|(_, v)| **v)
            .nth((blade / edge_count) as usize)
            .map(|(edge, _)| edge);
        let angle = if let Some(edge) = edge {
            [
                std::f32::consts::PI,
                0.,
                -std::f32::consts::FRAC_PI_2,
                std::f32::consts::FRAC_PI_2,
            ][edge]
                + (random(coordinates[0], coordinates[1], blade * 7 + 1) - 0.5) * 1.5
        } else {
            random(coordinates[0], coordinates[1], blade * 7 + 1) * std::f32::consts::TAU
        };
        let forward = Vec3::new(angle.cos(), 0., angle.sin());
        let side = Vec3::new(-angle.sin(), 0., angle.cos());
        let placement = |a: bool, b: bool, salt: u32| {
            let lo = if a { 0.0 } else { 0.18 };
            let hi = if b { 1.0 } else { 0.82 };
            unit * (lo + random(coordinates[0], coordinates[1], salt) * (hi - lo))
        };
        let mut root = Vec3::new(
            placement(joined[0], joined[1], blade * 7 + 2),
            0.,
            placement(joined[2], joined[3], blade * 7 + 3),
        );
        if let Some(edge) = edge {
            let (a, b) = if edge < 2 {
                (joined[2], joined[3])
            } else {
                (joined[0], joined[1])
            };
            let lo = if a { 0.04 } else { 0.18 };
            let hi = if b { 0.96 } else { 0.82 };
            let slot = blade % edge_count;
            let stagger = if slot == 0 || slot == edge_count - 1 { 0. } else { (seed - 0.5) * 0.7 };
            let along = unit * (lo + (slot as f32 + stagger)
                / (edge_count - 1) as f32 * (hi - lo));
            let inset = unit * (0.015 + random(coordinates[0], coordinates[1], blade * 7 + 4) * 0.035);
            match edge {
                0 => {
                    root.x = inset;
                    root.z = along;
                }
                1 => {
                    root.x = unit - inset;
                    root.z = along;
                }
                2 => {
                    root.z = inset;
                    root.x = along;
                }
                _ => {
                    root.z = unit - inset;
                    root.x = along;
                }
            }
        }
        // Park's two native grass drawings need different silhouettes as well
        // as heights: fine cool tufts and broader warm long-grass leaves.
        let park = kind != PatchKind::Meadow;
        let edge_distance = [
            (!joined[0]).then_some(root.x),
            (!joined[1]).then_some(unit - root.x),
            (!joined[2]).then_some(root.z),
            (!joined[3]).then_some(unit - root.z),
        ].into_iter().flatten().fold(unit, f32::min);
        let edge_height = if park {
            let t = (edge_distance / (unit * 0.30)).clamp(0., 1.);
            0.90 + 0.10 * t * t * (3. - 2. * t)
        } else { 1. };
        let h = height * (0.80 + seed * 0.20) * edge_height;
        // Pigment follows a continuous field in authored source coordinates,
        // with a weaker blade-to-blade change instead of a tint per tile.
        let pigment = if kind == PatchKind::Meadow {
            let mut patch = crate::foliage::leaf_pigment(Vec3::new(
                coordinates[0] as f32 + root.x/unit,0.4,
                coordinates[1] as f32 + root.z/unit),2.7);
            let blade_tone = 1.+(random(coordinates[0],coordinates[1],blade*7+6)-0.5)*0.05;
            for channel in &mut patch { *channel *= blade_tone; }
            patch
        } else { [1.;3] };
        let width = unit * match kind {
            PatchKind::Meadow => 0.035 + seed * 0.025,
            PatchKind::ParkShort => 0.030 + seed * 0.016,
            PatchKind::ParkLong => 0.062 + seed * 0.025,
        };
        let bend = match kind {
            PatchKind::Meadow => 0.10 + random(coordinates[0], coordinates[1], blade * 7 + 5) * 0.08,
            PatchKind::ParkLong => 0.24,
            PatchKind::ParkShort => 0.17,
        };
        // Taller leaves need a proportionate arch. Keep outer-edge tips inside
        // their authored patch instead of clipping a long leaf into a flat wall.
        let mut reach = h * bend;
        for (axis, low, high) in [(0, joined[0], joined[1]), (2, joined[2], joined[3])] {
            let direction = forward[axis];
            if direction > 0.001 && !high {
                reach = reach.min((unit - root[axis]) / direction * 0.75);
            } else if direction < -0.001 && !low {
                reach = reach.min(-root[axis] / direction * 0.75);
            }
        }
        let point = |row: usize, col: usize| {
            let t = row as f32 / 4.;
            let taper = (1. - t).powf(0.7);
            let across = col as f32 - 1.;
            let mut p = root
                + Vec3::Y * (h * (t - 0.10 * t * t))
                + forward * (reach * t * t)
                + side * (across * width * taper)
                + forward * (if col == 1 { width * 0.30 * taper } else { 0. });
            if !joined[0] {
                p.x = p.x.max(unit * 0.001);
            }
            if !joined[1] {
                p.x = p.x.min(unit * 0.999);
            }
            if !joined[2] {
                p.z = p.z.max(unit * 0.001);
            }
            if !joined[3] {
                p.z = p.z.min(unit * 0.999);
            }
            p
        };
        for row in 0..4 {
            for col in 0..2 {
                let p = [
                    point(row, col),
                    point(row + 1, col),
                    point(row + 1, col + 1),
                    point(row, col + 1),
                ];
                for ids in [[0, 1, 2], [0, 2, 3]] {
                    let p = ids.map(|i| p[i]);
                    let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
                    if normal.length_squared() < 0.5 {
                        continue;
                    }
                    for reverse in [false, true] {
                        let first = out.positions.len() as u32;
                        let order = if reverse { [0, 2, 1] } else { [0, 1, 2] };
                        for i in order {
                            let t = p[i].y / h;
                            let shade = 0.68
                                + 0.32 * normal.dot(Vec3::new(0.45, 0.80, 0.55).normalize()).abs();
                            out.positions
                                .push((p[i] + Vec3::new(w, base, n)).to_array());
                            out.normals
                                .push((if reverse { -normal } else { normal }).to_array());
                            // Native foliage plus its height above its actual
                            // root, for battle-only trimming. Shape and pigment
                            // remain the original map geometry.
                            out.uvs.push([84000.0, p[i].y]);
                            let color = match kind {
                                PatchKind::Meadow => [0.055 + seed * 0.045 + t * 0.22,
                                    0.18 + seed * 0.05 + t * 0.27, 0.065 + t * 0.035],
                                PatchKind::ParkShort => [0.035 + seed * 0.025 + t * 0.11,
                                    0.16 + seed * 0.035 + t * 0.24, 0.085 + t * 0.065],
                                PatchKind::ParkLong => [0.075 + seed * 0.04 + t * 0.27,
                                    0.19 + seed * 0.04 + t * 0.29, 0.045 + t * 0.045],
                            };
                            out.colors.push([color[0]*shade*pigment[0],color[1]*shade*pigment[1],
                                color[2]*shade*pigment[2],1.]);
                        }
                        out.indices.extend([first, first + 1, first + 2]);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tall_grass_reaches_trainer_chest_height_in_retained_battles() {
        // Retained scenery uses 16 source pixels per battle world unit. Even
        // the shortest edge blades reach an adult trainer's chest (1.38 * .68).
        let chest = 1.38_f32 * 0.68;
        for kind in [PatchKind::Meadow, PatchKind::ParkShort, PatchKind::ParkLong] {
            for (coordinates, tile_height) in [([0, 0], 8.), ([20, 24], 32.), ([-17, 31], 64.)] {
                let mut mesh = SurfaceMeshData::default();
                append_styled_patch(&mut mesh, [0., tile_height, 0., tile_height], 2.,
                    kind.height(tile_height), coordinates, true, [false; 4], kind);
                // Find actual leaf tips through shared geometric neighbors;
                // two-sided triangles and clipped edges need not have fixed counts.
                let mut peaks = std::collections::HashMap::new();
                for triangle in mesh.positions.chunks_exact(3) {
                    for p in triangle {
                        let entry = peaks.entry(p.map(f32::to_bits)).or_insert((p[1] - 2., false));
                        entry.1 |= triangle.iter().any(|q| q[1] > p[1]);
                    }
                }
                let tips: Vec<_> = peaks.values().filter(|(_, higher)| !higher).collect();
                assert!(tips.len() >= 16);
                for &&(tip, _) in &tips {
                    let world_tip = tip / 16. * crate::grass_rustle::retained_height_scale(tile_height).x;
                    assert!(world_tip >= chest, "blade tip {world_tip} falls below battle chest height");
                }
                assert!(mesh.positions.iter().any(|p| p[1] == 2.), "taller blades retain their roots");
            }
        }
    }

    #[test]
    fn park_grasses_have_distinct_leaf_shapes_and_palette_with_rooted_edges() {
        for coordinates in [[0,0],[20,24],[-17,31]] {
            let mut short = SurfaceMeshData::default();
            let mut long = SurfaceMeshData::default();
            for (mesh, kind, height) in [
                (&mut short,PatchKind::ParkShort,PatchKind::ParkShort.height(8.)),
                (&mut long,PatchKind::ParkLong,PatchKind::ParkLong.height(8.)),
            ] {
                append_styled_patch(mesh,[0.,8.,0.,8.],2.,height,coordinates,true,[false;4],kind);
                assert!(!mesh.positions.is_empty());
                assert!(mesh.positions.iter().all(|p| Vec3::from_array(*p).is_finite()
                    && p[1]>=2. && (0. ..=8.).contains(&p[0]) && (0. ..=8.).contains(&p[2])));
                assert!(mesh.positions.iter().any(|p| p[1]==2.),"grass stays rooted at the source floor");
            }
            let top = |m:&SurfaceMeshData| m.positions.iter().map(|p|p[1]-2.).fold(0.,f32::max);
            assert!(top(&long)>top(&short)*1.45,"long grass must read distinctly taller");
            let warmth = |m:&SurfaceMeshData| {
                let sum=m.colors.iter().fold([0.;2],|mut sum,c| {sum[0]+=c[0];sum[1]+=c[1];sum});
                sum[0]/sum[1]
            };
            assert!(warmth(&long)>warmth(&short)*1.5,"two source regions need visible palette contrast");
            let blade_width = |m:&SurfaceMeshData|
                Vec3::from_array(m.positions[0]).distance(Vec3::from_array(m.positions[8]));
            assert!(blade_width(&long)>blade_width(&short)*1.5,"long-grass leaves have a broader folded silhouette");
        }
    }
    #[test]
    fn every_shared_edge_has_distributed_foliage_for_all_neighbor_patterns() {
        for detailed in [false, true] {
            for seed in 0..8 {
                for mask in 1..16 {
                    let joined = std::array::from_fn(|edge| mask & (1 << edge) != 0);
                    let mut mesh = SurfaceMeshData::default();
                    append_patch(
                        &mut mesh,
                        [0., 8., 0., 8.],
                        2.,
                        5.,
                        [seed * 19 - 71, seed * 13 + 3],
                        detailed,
                        joined,
                    );
                    for edge in 0..4 {
                        let across = |p: &[f32; 3]| match edge {
                            0 => p[0] < 0.,
                            1 => p[0] > 8.,
                            2 => p[2] < 0.,
                            _ => p[2] > 8.,
                        };
                        let crossings: Vec<_> =
                            mesh.positions.iter().filter(|p| across(p)).collect();
                        if !joined[edge] {
                            assert!(
                                crossings.is_empty(),
                                "foliage spills onto a non-grass neighbor"
                            );
                        } else {
                            let axis = if edge < 2 { 2 } else { 0 };
                            let min = crossings
                                .iter()
                                .map(|p| p[axis])
                                .fold(f32::INFINITY, f32::min);
                            let max = crossings
                                .iter()
                                .map(|p| p[axis])
                                .fold(f32::NEG_INFINITY, f32::max);
                            assert!(
                                max - min > 4.4,
                                "shared edge has a sparse join: {edge}, {mask}, {seed}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn shared_edges_overlap_without_spilling_onto_paths() {
        for detailed in [false, true] {
            let mut left = SurfaceMeshData::default();
            let mut right = SurfaceMeshData::default();
            let mut shifted = SurfaceMeshData::default();
            append_patch(
                &mut left,
                [0., 8., 0., 8.],
                2.,
                5.,
                [91, -32],
                detailed,
                [false, true, false, false],
            );
            append_patch(
                &mut right,
                [8., 16., 0., 8.],
                2.,
                5.,
                [92, -32],
                detailed,
                [true, false, false, false],
            );
            append_patch(
                &mut shifted,
                [-8., 0., -16., -8.],
                2.,
                5.,
                [91, -32],
                detailed,
                [false, true, false, false],
            );
            assert!(left.positions.iter().any(|p| p[0] > 8.));
            assert!(right.positions.iter().any(|p| p[0] < 8.));
            assert!(
                left.positions
                    .iter()
                    .all(|p| p[0] > 0. && p[2] > 0. && p[2] < 8.)
            );
            assert!(
                right
                    .positions
                    .iter()
                    .all(|p| p[0] < 16. && p[2] > 0. && p[2] < 8.)
            );
            assert_eq!(left.colors, shifted.colors);
            assert_eq!(left.normals, shifted.normals);
            assert_eq!(left.indices, shifted.indices);
            for (p, q) in left.positions.iter().zip(&shifted.positions) {
                assert!((p[0] - q[0] - 8.).abs() < 0.00001);
                assert!((p[2] - q[2] - 16.).abs() < 0.00001);
                assert_eq!(p[1], q[1]);
            }
        }
    }

    #[test]
    fn scrolling_preserves_curvature_colors_and_roots_for_both_detail_levels() {
        for detailed in [false, true] {
            let mut a = SurfaceMeshData::default();
            let mut b = SurfaceMeshData::default();
            append(&mut a, [0., 8., 0., 8.], 2., 5., [91, -32], detailed);
            append(&mut b, [-8., 0., -16., -8.], 2., 5., [91, -32], detailed);
            assert_eq!(a.indices, b.indices);
            assert_eq!(a.colors, b.colors);
            assert_eq!(a.normals, b.normals);
            assert_eq!(a.uvs, b.uvs, "blade-root metadata stays fixed while the grid scrolls");
            for (point, uv) in a.positions.iter().zip(&a.uvs) {
                assert_eq!(uv[0], 84000.0);
                assert!((uv[1] - (point[1] - 2.0)).abs() < 0.00001,
                    "trimming must measure from the original blade root");
            }
            for (p, q) in a.positions.iter().zip(&b.positions) {
                assert!((p[0] - q[0] - 8.).abs() < 0.00001);
                assert!((p[2] - q[2] - 16.).abs() < 0.00001);
                assert_eq!(p[1], q[1]);
                assert!(
                    p[0] > 0. && p[0] < 8. && p[2] > 0. && p[2] < 8. && p[1] >= 2. && p[1] < 7.
                );
            }
            assert!(
                a.normals
                    .iter()
                    .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 0.00001)
            );
        }
    }
}
