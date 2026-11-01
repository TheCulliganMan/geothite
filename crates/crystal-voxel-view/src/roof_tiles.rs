//! Curved ceramic courses on the authored town buildings. Structural decking,
//! ridge caps, doors and the model's placement bounds are left intact.
use crate::mesh::SurfaceMeshData;
use bevy::prelude::Vec3;

fn push(mesh: &mut SurfaceMeshData, p: [Vec3; 3], n: [Vec3; 3], color: [f32; 4]) {
    let start = mesh.positions.len() as u32;
    mesh.positions.extend(p.map(|v| v.to_array()));
    mesh.normals.extend(n.map(|n| n.normalize().to_array()));
    mesh.uvs.extend([[0.; 2]; 3]);
    mesh.colors.extend([color; 3]);
    mesh.indices.extend([start, start + 1, start + 2]);
}

/// Returns true only for the untextured ceramic palettes. Exact source
/// perimeter positions are retained; detail rises at most 0.014 model units.
pub(crate) fn refine(mesh: &mut SurfaceMeshData, first: usize, index: usize, name: &str) -> bool {
    let family = if name.starts_with("roof_red") {
        [0.68, 0.25, 0.17]
    } else if name.starts_with("roof_teal") {
        [0.20, 0.45, 0.42]
    } else if name.starts_with("roof_blue") {
        [0.18, 0.34, 0.55]
    } else {
        return false;
    };
    refine_family(mesh, first, index, name, family, false)
}

/// Give the existing traditional-house panels the same closed course detail,
/// retaining their named paint so each source placement can recolor it.
pub(crate) fn refine_traditional(
    mesh: &mut SurfaceMeshData,
    first: usize,
    index: usize,
    name: &str,
    color: [f32; 4],
) -> bool {
    if !name.starts_with("violet") {
        return false;
    }
    refine_family(mesh, first, index, name, [color[0], color[1], color[2]], true)
}

fn refine_family(
    mesh: &mut SurfaceMeshData,
    first: usize,
    index: usize,
    name: &str,
    family: [f32; 3],
    separate_panels: bool,
) -> bool {
    if name.ends_with("_dark") {
        return false;
    }
    let old = SurfaceMeshData {
        positions: mesh.positions.split_off(first),
        normals: mesh.normals.split_off(first),
        uvs: mesh.uvs.split_off(first),
        colors: mesh.colors.split_off(first),
        indices: mesh
            .indices
            .split_off(index)
            .into_iter()
            .map(|i| i - first as u32)
            .collect(),
        ..Default::default()
    };
    for ids in old.indices.chunks(6) {
        if ids.len() != 6 {
            let a = [ids[0] as usize, ids[1] as usize, ids[2] as usize];
            push(
                mesh,
                a.map(|i| Vec3::from_array(old.positions[i])),
                a.map(|i| Vec3::from_array(old.normals[i])),
                old.colors[a[0]],
            );
            continue;
        }
        let p =
            [ids[0], ids[1], ids[2], ids[5]].map(|i| Vec3::from_array(old.positions[i as usize]));
        let normal = Vec3::from_array(old.normals[ids[0] as usize]);
        let area = (p[1] - p[0]).cross(p[3] - p[0]).length();
        let tile = ids[0] == ids[3]
            && ids[2] == ids[4]
            && (0.35..0.98).contains(&normal.y)
            && (0.025..0.8).contains(&area)
            && ids
                .iter()
                .all(|i| Vec3::from_array(old.normals[*i as usize]).dot(normal) > 0.9999)
            && p.iter().all(|v| (*v - p[0]).dot(normal).abs() < 0.0001)
            && p.iter().all(|top| {
                old.positions.iter().any(|b| {
                    (b[0] - top.x).abs() < 0.00001
                        && (b[2] - top.z).abs() < 0.00001
                        && (0.01..0.08).contains(&(top.y - b[1]))
                })
            });
        if !tile {
            for tri in ids.chunks_exact(3) {
                let a = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
                push(
                    mesh,
                    a.map(|i| Vec3::from_array(old.positions[i])),
                    a.map(|i| Vec3::from_array(old.normals[i])),
                    old.colors[a[0]],
                );
            }
            continue;
        }
        let center = p.iter().copied().sum::<Vec3>() * 0.25;
        let variation = (center.x * 8.3 + center.y * 4.7 + center.z * 11.1).sin() * 0.012;
        let color = [
            family[0] + variation,
            family[1] + variation * 0.7,
            family[2] + variation * 0.5,
            1.,
        ];
        let du = p[1] - p[0];
        let dv = p[3] - p[0];
        // Two smaller ceramic panels share each authored closed course box.
        // Shallow crowns and the center valley give tiles a plausible scale
        // without the previous broad cushion shape or additional triangles.
        // All outer edges stay on the original closed box.
        let point = |panel: usize, u: f32, v: f32| {
            let x = 2. * u - 1.;
            let y = 2. * v - 1.;
            let fu = 1. - x.powi(8);
            let fv = 1. - y.powi(4);
            let height = 0.014 * fu * fv;
            let a = du * 0.5 - normal * (0.014 * 16. * x.powi(7) * fv);
            let b = dv - normal * (0.014 * 8. * y.powi(3) * fu);
            let n = a.cross(b).normalize();
            (
                p[0] + du * (panel as f32 * 0.5 + u * 0.5) + dv * v + normal * height,
                n,
            )
        };
        const U: usize = 4;
        const V: usize = 4;
        for panel in 0..2 {
            // Traditional roofs retain their native paint but each small tile
            // needs its own quiet firing variation and a readable side joint.
            // Identical paint across the central valley merged two tiles into
            // one broad sheet at the normal game camera distance.
            let firing = if separate_panels {
                1.0 + ((center.x + panel as f32 * du.x * 0.5) * 17.3
                    + center.z * 9.7).sin() * 0.025
            } else {1.0};
            for y in 0..V {
                for x in 0..U {
                    let q = [
                        point(panel, x as f32 / U as f32, y as f32 / V as f32),
                        point(panel, (x + 1) as f32 / U as f32, y as f32 / V as f32),
                        point(panel, (x + 1) as f32 / U as f32, (y + 1) as f32 / V as f32),
                        point(panel, x as f32 / U as f32, (y + 1) as f32 / V as f32),
                    ];
                    for t in [[0, 1, 2], [0, 2, 3]] {
                        push(mesh, t.map(|i| q[i].0), t.map(|i| q[i].1), color);
                        // The palette shader uses authored light rather than PBR.
                        // Shade the curved surface itself so rolled margins and
                        // ceramic crowns remain visible between cast shadows.
                        let sun = Vec3::new(0.45, 0.80, 0.55).normalize();
                        let first = mesh.colors.len() - 3;
                        for (vertex, i) in t.into_iter().enumerate() {
                            let edge = [x,x+1,x+1,x][i];
                            let joint = if separate_panels && (edge==0 || edge==U) {0.84} else {1.0};
                            let light = (0.60 + 0.40 * q[i].1.dot(sun).max(0.0))*joint*firing;
                            for c in 0..3 {
                                mesh.colors[first + vertex][c] *= light;
                            }
                        }
                    }
                }
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    #[derive(Deserialize)]
    struct Export {
        primitives: Vec<Primitive>,
    }
    #[derive(Deserialize)]
    struct Primitive {
        material: String,
        positions: Vec<f32>,
        normals: Vec<f32>,
        indices: Vec<u32>,
        base_color: [f32; 4],
    }

    #[test]
    fn legacy_town_roofs_gain_curvature_with_fixed_perimeters_and_quiet_colors() {
        // Closed authored Gym/pagoda courses bypass this legacy adapter and
        // have their own raw-geometry preservation checks in new_bark_models.
        for json in [
            include_str!("../models/new_bark/player_house.mesh.json"),
            include_str!("../models/new_bark/lab.mesh.json"),
            include_str!("../models/johto/traditional_house.mesh.json"),
        ] {
            let export: Export = serde_json::from_str(json).unwrap();
            let mut curved = 0;
            for p in export.primitives {
                if !(p.material.starts_with("roof_") || p.material.starts_with("violet"))
                    || p.material.ends_with("_dark")
                {
                    continue;
                }
                let positions: Vec<[f32; 3]> = p
                    .positions
                    .chunks_exact(3)
                    .map(|v| [v[0], v[1], v[2]])
                    .collect();
                let count = positions.len();
                let mut mesh = SurfaceMeshData {
                    positions: positions.clone(),
                    normals: p
                        .normals
                        .chunks_exact(3)
                        .map(|v| [v[0], v[1], v[2]])
                        .collect(),
                    indices: p.indices.clone(),
                    colors: vec![p.base_color; count],
                    uvs: vec![[0.; 2]; count],
                    ..Default::default()
                };
                assert!(
                    refine(&mut mesh, 0, 0, &p.material)
                        || refine_traditional(&mut mesh, 0, 0, &p.material, p.base_color)
                );
                if mesh.indices.len() <= p.indices.len() {
                    continue;
                }
                curved += 1;
                for v in &positions {
                    assert!(
                        mesh.positions
                            .iter()
                            .any(|q| (Vec3::from_array(*q) - Vec3::from_array(*v)).length()
                                < 0.00001)
                    );
                }
                for axis in 0..3 {
                    let lo = positions
                        .iter()
                        .map(|v| v[axis])
                        .fold(f32::INFINITY, f32::min);
                    let hi = positions
                        .iter()
                        .map(|v| v[axis])
                        .fold(f32::NEG_INFINITY, f32::max);
                    assert!(mesh.positions.iter().all(|v| v[axis] >= lo - 0.00001
                        && v[axis] <= hi + if axis == 1 { 0.01401 } else { 0.00001 }));
                }
                assert!(
                    mesh.normals
                        .iter()
                        .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 0.001)
                );
                assert!(
                    mesh.colors
                        .iter()
                        .all(|c| c.iter().all(|v| v.is_finite()) && c[3] == 1.)
                );
            }
            assert_eq!(
                curved, 3,
                "each ceramic shade must contain actual curved tiles"
            );
        }
    }
    #[test]
    fn split_courses_keep_closed_edges_shallow_crowns_and_the_existing_triangle_budget() {
        let du = Vec3::new(0.8, 0., 0.);
        let dv = Vec3::new(0., -0.15, -0.3);
        let normal = du.cross(dv).normalize();
        let top = [Vec3::ZERO, du, du + dv, dv];
        let mut mesh = SurfaceMeshData {
            positions: top
                .into_iter()
                .chain(top.map(|p| p - Vec3::Y * 0.04))
                .map(|p| p.to_array())
                .collect(),
            normals: vec![normal.to_array(); 8],
            colors: vec![[0.18, 0.34, 0.55, 1.]; 8],
            uvs: vec![[0.; 2]; 8],
            indices: vec![0, 1, 2, 0, 2, 3],
            ..Default::default()
        };
        assert!(refine(&mut mesh, 0, 0, "roof_blue"));
        assert_eq!(mesh.indices.len() / 3, 64);
        for p in top.into_iter().chain([du * 0.5, du * 0.5 + dv]) {
            assert!(
                mesh.positions
                    .iter()
                    .any(|q| (Vec3::from_array(*q) - p).length() < 0.00001)
            );
        }
        for u in [0.25, 0.75] {
            let crown = du * u + dv * 0.5 + normal * 0.014;
            assert!(
                mesh.positions
                    .iter()
                    .any(|q| (Vec3::from_array(*q) - crown).length() < 0.00001)
            );
        }
        let valley = du * 0.5 + dv * 0.5;
        assert!(
            mesh.positions
                .iter()
                .any(|q| (Vec3::from_array(*q) - valley).length() < 0.00001)
        );
        assert!(mesh.positions.iter().all(|p| {
            let height = Vec3::from_array(*p).dot(normal);
            height >= -0.00001 && height <= 0.01401
        }));
        for ids in mesh.indices.chunks_exact(3) {
            let corners =
                [ids[0], ids[1], ids[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
            let face = (corners[1] - corners[0]).cross(corners[2] - corners[0]);
            assert!(face.dot(normal) > 0.);
        }
    }
    #[test]
    fn unrelated_materials_are_untouched() {
        let mut m = SurfaceMeshData::default();
        m.positions.push([1., 2., 3.]);
        assert!(!refine(&mut m, 0, 0, "plaster"));
        assert_eq!(m.positions, [[1., 2., 3.]]);
    }
    #[test]
    fn unpaired_ceramic_triangle_is_retained() {
        let mut m = SurfaceMeshData {
            positions: vec![[0., 0., 0.], [1., 0., 0.], [0., 0., 1.]],
            normals: vec![[0., 1., 0.]; 3],
            colors: vec![[0.6, 0.2, 0.1, 1.]; 3],
            uvs: vec![[0.; 2]; 3],
            indices: vec![0, 1, 2],
            ..Default::default()
        };
        let original = m.positions.clone();
        assert!(refine(&mut m, 0, 0, "roof_red"));
        assert_eq!(m.positions, original);
        assert_eq!(m.indices, [0, 1, 2]);
    }
}
