//! Finish already-proven Ice Path floor paint at its original walking datum.
use super::*;
use bevy::prelude::Vec2;

// Sliding ice has its own opaque paint domain. It remains outside the wall
// cutaway and water animation paths while carrying map coordinates to the
// fragment shader, where fine frost is sampled without triangle-sized patches.
fn ice_uv(p: Vec3, g: &GridGeometry, origin: [i32; 2]) -> [f32; 2] {
    let mut uv = cave_cliff::rock_uv(p, g, origin, true);
    uv[1] += 500000.;
    uv
}

fn color(p: Vec3, ice: bool, g: &GridGeometry, origin: [i32; 2]) -> [f32; 4] {
    let x = (p.x - g.origin_x) / g.tile_width + origin[0] as f32;
    let z = (p.z - g.origin_z) / g.tile_height + origin[1] as f32;
    let broad = new_bark::meadow_noise(x * 0.23, z * 0.23);
    let fine = new_bark::meadow_noise(x * 2.1 + 13., z * 2.1 - 9.);
    if !ice {
        let cold = (broad * 1.3 + 0.45).clamp(0., 1.);
        let warm = Vec3::new(0.33, 0.305, 0.255);
        let slate = Vec3::new(0.27, 0.325, 0.345);
        let mineral = (x * 1.9 + z * 0.43 + broad * 7.).sin().abs();
        let vein = (1. - mineral / 0.08).clamp(0., 1.) * 0.035;
        let c = warm.lerp(slate, cold) + Vec3::splat(broad * 0.07 + fine * 0.018 + vein);
        return c.extend(1.).to_array();
    }
    let streak = (x * 0.71 - z * 0.48 + broad * 3.3).sin();
    let deep = Vec3::new(0.12, 0.43, 0.48);
    let clear = Vec3::new(0.38, 0.70, 0.71);
    // Grain stays below the broad translucent bands. Strong frost sampled at
    // mesh vertices forms isolated bright polygon patches at gameplay scale.
    let cloud = new_bark::meadow_noise(x * 0.68 + 37., z * 0.68 - 21.);
    let c = deep.lerp(clear, (0.50 + streak * 0.13 + broad * 0.35 + cloud * 0.25).clamp(0., 1.))
        + Vec3::splat(fine * 0.015);
    c.extend(1.).to_array()
}

pub(super) fn append(
    out: &mut SurfaceMeshData,
    points: [[f32; 3]; 4],
    ice: bool,
    g: &GridGeometry,
    origin: [i32; 2],
) {
    append_blended(out, points, ice, g, origin, &[]);
}

// Sample distances to authenticated material cells, rather than fading each
// tile independently. Shared edges receive the same frost color and the field
// remains fixed when the published grid scrolls. No support geometry changes.
fn blended_color(p: Vec3, ice: bool, g: &GridGeometry, origin: [i32; 2], cells: &[&VisualTile]) -> [f32; 4] {
    let local = Vec2::new((p.x - g.origin_x) / g.tile_width, (p.z - g.origin_z) / g.tile_height);
    let mut seam = f32::INFINITY;
    let mut wall = f32::INFINITY;
    for z in local.y.floor() as i32 - 2..=local.y.floor() as i32 + 1 {
        for x in local.x.floor() as i32 - 2..=local.x.floor() as i32 + 1 {
            if x < 0 || z < 0 || x >= g.width as i32 || z >= g.height as i32 { continue; }
            let Some(tile) = cells.get(z as usize * g.width + x as usize) else { continue; };
            if tile.source.tileset_id.as_ref() != "ice_path" { continue; }
            let distance = (local - local.clamp(Vec2::new(x as f32, z as f32), Vec2::new(x as f32 + 1., z as f32 + 1.))).length();
            let material = new_bark::ice_path_ground_material(&tile.source);
            if matches!(material, Some(new_bark::GroundMaterial::Ice | new_bark::GroundMaterial::Stone))
                && (material == Some(new_bark::GroundMaterial::Ice)) != ice {
                seam = seam.min(distance);
            }
            if crate::ice_path::rock_mass_local(&tile.source).is_some() {
                wall = wall.min(distance);
            }
        }
    }
    let x = local.x + origin[0] as f32;
    let z = local.y + origin[1] as f32;
    let noise = new_bark::meadow_noise;
    let width = 0.65 + noise(x * 1.2 + 43., z * 1.2 - 17.) * 0.30;
    let feather = |d: f32| { let t = (1. - d / width).clamp(0., 1.); t * t * (3. - 2. * t) };
    let base = Vec3::from_array(color(p, ice, g, origin)[..3].try_into().unwrap());
    let frost = Vec3::new(0.41, 0.60, 0.61) + Vec3::splat(noise(x * 0.68 + 37., z * 0.68 - 21.) * 0.045);
    let c = base.lerp(frost, feather(seam)).lerp(Vec3::new(0.39, 0.51, 0.54), feather(wall) * 0.75);
    c.extend(1.).to_array()
}

pub(super) fn append_blended(
    out: &mut SurfaceMeshData,
    points: [[f32; 3]; 4],
    ice: bool,
    g: &GridGeometry,
    origin: [i32; 2],
    cells: &[&VisualTile],
) {
    let sample = |u: usize, v: usize| {
        let u = u as f32 / 6.;
        let v = v as f32 / 6.;
        Vec3::from_array(points[0])
            .lerp(Vec3::from_array(points[3]), u)
            .lerp(
                Vec3::from_array(points[1]).lerp(Vec3::from_array(points[2]), u),
                v,
            )
    };
    for v in 0..6 {
        for u in 0..6 {
            let p = [
                sample(u, v),
                sample(u, v + 1),
                sample(u + 1, v + 1),
                sample(u + 1, v),
            ];
            append_quad_colors(
                out,
                p.map(|p| p.to_array()),
                [0., 1., 0.],
                p.map(|p| {
                    if ice {
                        ice_uv(p, g, origin)
                    } else {
                        cave_cliff::floor_uv(p, g, origin, true)
                    }
                }),
                p.map(|p| blended_color(p, ice, g, origin, cells)),
            );
        }
    }
    if ice {
        fractures(out, points, g, origin);
    }
}

fn site(cell: Vec2) -> Vec2 {
    cell + Vec2::new(
        new_bark::meadow_noise(cell.x * 17. + 61., cell.y * 13. - 33.) + 0.5,
        new_bark::meadow_noise(cell.x * 11. - 47., cell.y * 19. + 29.) + 0.5,
    )
}

/// Clip Voronoi bisectors directly instead of interpolating a narrow bright
/// band between coarse vertices. Neighbouring source cells share the lines.
fn fractures(out: &mut SurfaceMeshData, points: [[f32; 3]; 4], g: &GridGeometry, origin: [i32; 2]) {
    let to_map = |p: [f32; 3]| {
        Vec2::new(
            (p[0] - g.origin_x) / g.tile_width + origin[0] as f32,
            (p[2] - g.origin_z) / g.tile_height + origin[1] as f32,
        ) / 3.7
    };
    let min = to_map(points[0]);
    let max = to_map(points[2]);
    let cell = ((min + max) * 0.5).floor();
    let mut sites = Vec::new();
    for z in -2..=2 {
        for x in -2..=2 {
            sites.push(site(cell + Vec2::new(x as f32, z as f32)));
        }
    }
    for (i, &a) in sites.iter().enumerate() {
        for &b in &sites[i + 1..] {
            let mid = (a + b) * 0.5;
            let d = (b - a).perp().normalize_or_zero();
            if d == Vec2::ZERO {
                continue;
            }
            let mut interval = [-4_f32, 4_f32];
            let mut clip = |n: Vec2, limit: f32| {
                let denom = n.dot(d);
                let value = limit - n.dot(mid);
                if denom.abs() < 0.000001 {
                    return value >= -0.000001;
                }
                if denom > 0. {
                    interval[1] = interval[1].min(value / denom);
                } else {
                    interval[0] = interval[0].max(value / denom);
                }
                interval[0] < interval[1]
            };
            let valid = clip(Vec2::NEG_X, -min.x)
                && clip(Vec2::X, max.x)
                && clip(Vec2::NEG_Y, -min.y)
                && clip(Vec2::Y, max.y)
                && sites
                    .iter()
                    .filter(|&&c| c != a && c != b)
                    .all(|&c| clip(c - a, (c.length_squared() - a.length_squared()) * 0.5));
            if !valid || interval[1] - interval[0] < 0.0001 {
                continue;
            }
            let world = |q: Vec2| {
                Vec3::new(
                    (q.x * 3.7 - origin[0] as f32) * g.tile_width + g.origin_x,
                    points[0][1] + g.tile_height * 0.0001,
                    (q.y * 3.7 - origin[1] as f32) * g.tile_height + g.origin_z,
                )
            };
            let p = world(mid + d * interval[0]);
            let q = world(mid + d * interval[1]);
            let half = (q - p).cross(Vec3::Y).normalize() * g.tile_width * 0.005;
            // Clamp strip corners to the exact source face; no line leaks into a
            // neighbouring rock, stair, ladder or unknown texture sample.
            let clamp = |p: Vec3| {
                [
                    p.x.clamp(points[0][0], points[2][0]),
                    p.y,
                    p.z.clamp(points[0][2], points[2][2]),
                ]
            };
            append_quad_colors(
                out,
                [
                    clamp(p + half),
                    clamp(q + half),
                    clamp(q - half),
                    clamp(p - half),
                ],
                [0., 1., 0.],
                [[0.; 2]; 4],
                [[0.41, 0.62, 0.66, 1.]; 4],
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frost_crosses_authenticated_ice_stone_joins_without_touching_support() {
        let g = GridGeometry { width: 2, height: 1, tile_width: 8., tile_height: 8., origin_x: 0., origin_z: 0. };
        let tiles = [
            VisualTile { column: 0, row: 0, animation_frames: None, priority: false, texture: Default::default(),
                source: VisualTileSource { tileset_id: "ice_path".into(), metatile_id: 0x02, subtile_column: 1, subtile_row: 0, tile_index: 0x19 } },
            VisualTile { column: 1, row: 0, animation_frames: None, priority: false, texture: Default::default(),
                source: VisualTileSource { tileset_id: "ice_path".into(), metatile_id: 0x1f, subtile_column: 0, subtile_row: 0, tile_index: 0xc6 } },
        ];
        let cells = tiles.iter().collect::<Vec<_>>();
        let p = Vec3::new(8., 0., 4.);
        assert_eq!(blended_color(p, false, &g, [20, 30], &cells), blended_color(p, true, &g, [20, 30], &cells));
        for ice in [false, true] {
            let side = if ice { 1. } else { -1. };
            let a = blended_color(p + Vec3::X * side * 0.001, ice, &g, [20, 30], &cells);
            let b = blended_color(p, ice, &g, [20, 30], &cells);
            assert!(a.iter().zip(b).all(|(a,b)| (a-b).abs() < 0.001));
            let mut mesh = SurfaceMeshData::default();
            let x = if ice { 8. } else { 0. };
            append_blended(&mut mesh, [[x,0.,0.],[x,0.,8.],[x+8.,0.,8.],[x+8.,0.,0.]], ice, &g, [20,30], &cells);
            assert!(mesh.positions.iter().all(|p| p[1] >= 0. && p[1] <= 0.00081));
        }
        let mut moved = g;
        moved.origin_x -= 8.;
        assert_eq!(blended_color(p, false, &g, [20,30], &cells), blended_color(p - Vec3::X * 8., false, &moved, [20,30], &cells));
        let mut unknown = tiles;
        unknown[1].source.tile_index = 0x00;
        let unknown = unknown.iter().collect::<Vec<_>>();
        assert_eq!(blended_color(p, false, &g, [20,30], &unknown), color(p, false, &g, [20,30]));
    }
    #[test]
    fn finish_is_continuous_and_stays_on_the_source_plane() {
        let g = GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: -16.,
            origin_z: -16.,
        };
        let mut moved = g;
        moved.origin_x -= 8.;
        moved.origin_z += 16.;
        for ice in [false, true] {
            let p = Vec3::new(-5., 0., 7.);
            assert_eq!(ice_uv(p, &g, [20, 30]), ice_uv(p + Vec3::new(-8., 0., 16.), &moved, [20, 30]));
            assert_eq!((ice_uv(p, &g, [20, 30])[1] / 100000.).round(), 8.);
            assert_eq!(
                color(p, ice, &g, [20, 30]),
                color(p + Vec3::new(-8., 0., 16.), ice, &moved, [20, 30])
            );
            let mut mesh = SurfaceMeshData::default();
            append(
                &mut mesh,
                [[0., 0., 0.], [0., 0., 8.], [8., 0., 8.], [8., 0., 0.]],
                ice,
                &g,
                [20, 30],
            );
            assert!(mesh.positions.iter().take(144).all(|p| p[1] == 0.));
            assert!(
                mesh.positions
                    .iter()
                    .all(|p| (0. ..=0.00081).contains(&p[1])
                        && (0. ..=8.).contains(&p[0])
                        && (0. ..=8.).contains(&p[2]))
            );
            assert!(
                mesh.colors
                    .iter()
                    .all(|c| c[3] == 1. && c.iter().all(|v| (0. ..=1.).contains(v)))
            );
        }
        let mut field = SurfaceMeshData::default();
        fractures(
            &mut field,
            [[0., 0., 0.], [0., 0., 64.], [64., 0., 64.], [64., 0., 0.]],
            &g,
            [20, 30],
        );
        assert!(
            !field.indices.is_empty(),
            "a large ice field contains continuous fractures"
        );
        for tri in field.indices.chunks_exact(3) {
            let p = [tri[0], tri[1], tri[2]].map(|i| Vec3::from_array(field.positions[i as usize]));
            assert!(
                (p[1] - p[0]).cross(p[2] - p[0]).y >= 0.,
                "fractures face the camera above the floor"
            );
        }
    }
}
