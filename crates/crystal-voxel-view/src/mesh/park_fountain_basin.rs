//! Carved limestone fountains with hollow bowls and separate moving water.

use std::f32::consts::TAU;

use bevy::prelude::Vec3;

use super::{GridGeometry, SurfaceMeshData, VisualTile, append_quad_colors, append_solid_quad};

#[path = "park_fountain_spray.rs"]
mod spray;

const SEGMENTS: usize = 64;
const STONE: [f32; 3] = [0.55, 0.51, 0.40];

/// The pond's original shallow bowl stays inside its native two-cell center.
pub(super) fn append_pond(
    mesh: &mut SurfaceMeshData,
    center: [f32; 2],
    radius: [f32; 2],
    base: f32,
    tile: f32,
) {
    let top = base + tile * 0.75;
    // The foot, curved belly, thin rolled lip, inner wall and floor form one
    // closed stone section. The water never doubles as a missing stone cap.
    let profile = [
        [0., 0.],
        [0.58, 0.],
        [0.68, 0.06],
        [0.68, 0.15],
        [0.63, 0.20],
        [0.72, 0.28],
        [0.86, 0.40],
        [0.96, 0.57],
        [0.97, 0.63],
        [1., 0.66],
        [1., 0.70],
        [0.97, 0.75],
        [0.83, 0.75],
        [0.80, 0.71],
        [0.80, 0.59],
        [0.73, 0.40],
        [0.53, 0.26],
        [0., 0.26],
    ];
    stone(mesh, center, radius, base, tile, &profile);
    let water = top - tile * 0.125;
    water_disc(
        mesh,
        center,
        [radius[0] * 0.81, radius[1] * 0.81],
        water,
        tile,
    );
    nozzle(mesh, center, water, tile);
    spray::append(mesh, center, radius, water, base, tile, 1.);
}

/// A low tiered pedestal remains distinct from the bowl in the northern pond.
pub(super) fn append_pedestal(mesh: &mut SurfaceMeshData, bounds: [f32; 4], tile: f32) {
    let center = [(bounds[0] + bounds[1]) * 0.5, (bounds[2] + bounds[3]) * 0.5];
    let radius = [(bounds[1] - bounds[0]) * 0.5, (bounds[3] - bounds[2]) * 0.5];
    let profile = [
        [0., 0.],
        [0.92, 0.],
        [1., 0.06],
        [1., 0.16],
        [0.96, 0.22],
        [0.77, 0.22],
        [0.74, 0.27],
        [0.74, 0.38],
        [0.69, 0.43],
        [0.48, 0.43],
        [0.42, 0.51],
        [0.36, 0.72],
        [0.37, 0.84],
        [0.46, 0.89],
        [0.61, 0.95],
        [0.78, 1.06],
        [0.90, 1.22],
        [0.92, 1.32],
        [0.97, 1.35],
        [0.97, 1.41],
        [0.94, 1.46],
        [0.80, 1.46],
        [0.77, 1.40],
        [0.73, 1.24],
        [0.57, 1.12],
        [0., 1.12],
    ];
    stone(mesh, center, radius, 0., tile, &profile);
    let water = tile * 1.34;
    water_disc(
        mesh,
        center,
        [radius[0] * 0.76, radius[1] * 0.76],
        water,
        tile,
    );
    nozzle(mesh, center, water, tile);
    // The small promenade fixture recirculates into its own raised bowl.
    spray::append(
        mesh,
        center,
        [radius[0] * 0.43, radius[1] * 0.43],
        water,
        water,
        tile,
        0.48,
    );
}

fn point(center: [f32; 2], radius: [f32; 2], angle: f32, scale: f32, y: f32) -> Vec3 {
    Vec3::new(
        center[0] + radius[0] * angle.cos() * scale,
        y,
        center[1] + radius[1] * angle.sin() * scale,
    )
}

fn stone(
    mesh: &mut SurfaceMeshData,
    center: [f32; 2],
    radius: [f32; 2],
    base: f32,
    tile: f32,
    profile: &[[f32; 2]],
) {
    for pair in profile.windows(2) {
        let [[r0, y0], [r1, y1]] = [pair[0], pair[1]];
        let dr = r1 - r0;
        let dy = (y1 - y0) * tile;
        for segment in 0..SEGMENTS {
            let a = segment as f32 / SEGMENTS as f32 * TAU;
            let b = (segment + 1) as f32 / SEGMENTS as f32 * TAU;
            let p = [
                point(center, radius, a, r0, base + y0 * tile),
                point(center, radius, a, r1, base + y1 * tile),
                point(center, radius, b, r1, base + y1 * tile),
                point(center, radius, b, r0, base + y0 * tile),
            ];
            let normal = |angle: f32| {
                Vec3::new(
                    dy * angle.cos() / radius[0],
                    -dr,
                    dy * angle.sin() / radius[1],
                )
                .normalize()
            };
            let na = normal(a);
            let nb = normal(b);
            let wedge = (segment / 4) as f32;
            let tone = 0.97 + (wedge * 2.37).sin() * 0.035;
            let colors = [na, na, nb, nb].map(|n| {
                let light = 0.88 + n.dot(Vec3::new(0.45, 0.8, 0.55).normalize()).max(0.) * 0.12;
                let wet = if y1 < 0.2 { 0.83 } else { 1. };
                [
                    STONE[0] * tone * light * wet,
                    STONE[1] * tone * light * wet,
                    STONE[2] * tone * light * wet,
                    1.,
                ]
            });
            // A center fan uses a repeated fourth vertex, matching terrain's
            // four-vertex face contract without an open bottom or inverted face.
            let p = if r0 == 0. {
                [p[0], p[1], p[2], p[0]]
            } else if r1 == 0. {
                [p[0], p[1], p[3], p[0]]
            } else {
                p
            };
            append_quad_colors(
                mesh,
                p.map(|p| p.to_array()),
                na.to_array(),
                [[0.; 2]; 4],
                colors,
            );
            let start = mesh.positions.len() - 4;
            let normals = if r1 == 0. {
                [na, na, nb, na]
            } else {
                [na, na, nb, nb]
            };
            mesh.normals[start..].copy_from_slice(&normals.map(|n| n.to_array()));
            for i in start..mesh.positions.len() {
                let p = mesh.positions[i];
                // Local stone grain shares the same scale on both fixtures.
                mesh.uvs[i] = [
                    63000. + (p[0] - center[0]) / tile,
                    -63000. + (p[2] - center[1]) / tile,
                ];
            }
        }
    }
}

fn water_disc(mesh: &mut SurfaceMeshData, center: [f32; 2], radius: [f32; 2], y: f32, tile: f32) {
    for segment in 0..SEGMENTS {
        let a = segment as f32 / SEGMENTS as f32 * TAU;
        let b = (segment + 1) as f32 / SEGMENTS as f32 * TAU;
        let middle = Vec3::new(center[0], y, center[1]);
        let p = [
            middle,
            point(center, radius, b, 1., y),
            point(center, radius, a, 1., y),
            middle,
        ];
        append_solid_quad(
            mesh,
            p.map(|p| p.to_array()),
            Vec3::Y.to_array(),
            [0.06, 0.27, 0.28, 1.],
        );
        let start = mesh.uvs.len() - 4;
        for i in start..mesh.uvs.len() {
            let p = mesh.positions[i];
            mesh.uvs[i] = [
                61000. + (p[0] - center[0]) / tile,
                -61000. + (p[2] - center[1]) / tile,
            ];
        }
    }
}

fn nozzle(mesh: &mut SurfaceMeshData, center: [f32; 2], water: f32, tile: f32) {
    let start = mesh.positions.len();
    stone(
        mesh,
        center,
        [tile * 0.085; 2],
        water - tile * 0.04,
        tile,
        &[
            [0., 0.],
            [1., 0.],
            [1., 0.06],
            [0.58, 0.09],
            [0.58, 0.13],
            [0., 0.13],
        ],
    );
    for (uv, color) in mesh.uvs[start..].iter_mut().zip(&mut mesh.colors[start..]) {
        *uv = [0.; 2];
        *color = [0.16, 0.22, 0.18, 1.];
    }
}

/// Only an already-owned complete source spray authorizes the pool finish.
/// The center is derived from that placement, including scrolling offsets.
pub(super) fn pool_center(
    cells: &[&VisualTile],
    geometry: &GridGeometry,
    authored: &[Option<&str>],
) -> Option<[f32; 2]> {
    cells.iter().enumerate().find_map(|(i, c)| {
        let s = &c.source;
        (authored.get(i).copied().flatten() == Some("park/pond-basin")
            && s.tileset_id.as_ref() == "park"
            && s.metatile_id == 0x3f
            && s.subtile_column == 3
            && s.subtile_row == 0
            && s.tile_index == 0x5f)
            .then(|| {
                let (w, _, n, _) = geometry.bounds(i % geometry.width, i / geometry.width);
                [w + geometry.tile_width, n + geometry.tile_height * 2.15]
            })
    })
}

pub(super) fn append_pool_surface(
    mesh: &mut SurfaceMeshData,
    positions: [[f32; 3]; 4],
    center: [f32; 2],
    tile: f32,
) {
    let uv = positions.map(|p| {
        [
            62000. + (p[0] - center[0]) / tile,
            -62000. + (p[2] - center[1]) / tile,
        ]
    });
    super::append_quad(
        mesh,
        positions,
        Vec3::Y.to_array(),
        uv,
        [0.06, 0.27, 0.28, 1.],
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fountains_have_closed_stone_bowls_and_finite_outward_faces() {
        for pedestal in [false, true] {
            let mut mesh = SurfaceMeshData::default();
            if pedestal {
                append_pedestal(&mut mesh, [-11.2, 11.2, -6.8, 6.8], 8.);
            } else {
                append_pond(&mut mesh, [0.; 2], [7.04, 5.2], -2., 8.);
            }
            assert!(
                mesh.positions
                    .iter()
                    .all(|p| Vec3::from_array(*p).is_finite())
            );
            assert_eq!(mesh.positions.len(), mesh.normals.len());
            assert_eq!(mesh.positions.len(), mesh.uvs.len());
            assert!(
                mesh.indices
                    .iter()
                    .all(|&i| (i as usize) < mesh.positions.len())
            );
            for (p, normals) in mesh
                .positions
                .as_chunks::<4>()
                .0
                .iter()
                .zip(mesh.normals.as_chunks::<4>().0)
            {
                let [a, b, c] = [p[0], p[1], p[2]].map(Vec3::from_array);
                let face = (b - a).cross(c - a).normalize();
                assert!(face.is_finite());
                for &n in normals {
                    let n = Vec3::from_array(n);
                    assert!((n.length() - 1.).abs() < 0.001);
                    assert!(
                        n.dot(face) > 0.8,
                        "winding and outward normal disagree: {p:?}"
                    );
                }
            }
            let water: Vec<_> = mesh
                .positions
                .iter()
                .zip(&mesh.uvs)
                .filter(|(_, uv)| (60990. ..61010.).contains(&uv[0]))
                .map(|(p, _)| p[1])
                .collect();
            let level = if pedestal { 8. * 1.34 } else { 3. };
            assert!(!water.is_empty() && water.iter().all(|&y| (y - level).abs() < 0.001));
            let top = if pedestal { 8. * 1.46 } else { 4. };
            assert!(mesh.positions.iter().zip(&mesh.uvs).any(|(p, uv)|
                (62990. ..63010.).contains(&uv[0]) && (p[1]-top).abs()<0.001));
        }
    }
}
