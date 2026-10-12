//! Thin rock rims on authored water-corner art. The full water plane and its
//! live atlas sample remain at zero; these are decorative rims, not shelves.
use super::*;
const WEST: u8 = 1;
const NORTH: u8 = 2;
const EAST: u8 = 4;
const NORTH_EAST_CORNER: u8 = 8;
const NORTH_WEST_CORNER: u8 = 16;
fn edges(s: &VisualTileSource) -> Option<u8> {
    if !matches!(s.tileset_id.as_ref(), "cave" | "dark_cave")
        || s.subtile_column >= 4
        || s.subtile_row >= 4
    {
        return None;
    }
    let (x, y, t) = (s.subtile_column, s.subtile_row, s.tile_index);
    match s.metatile_id {
        0x3d if y == 0 && x < 3 && t == 0x26 => Some(NORTH),
        0x3d if y == 0 && x == 3 && t == 0x0b => Some(NORTH | EAST),
        0x3d if y > 0 && x == 3 && t == 0x15 => Some(EAST),
        0x3e if y == 0 && x > 0 && t == 0x26 => Some(NORTH),
        0x3e if y == 0 && x == 0 && t == 0x0a => Some(NORTH | WEST),
        0x3e if y > 0 && x == 0 && t == 0x17 => Some(WEST),
        0x32 if s.tileset_id.as_ref() == "dark_cave" && (x, y, t) == (3, 0, 0x25) => {
            Some(NORTH_EAST_CORNER)
        }
        0x33 if s.tileset_id.as_ref() == "dark_cave" && (x, y, t) == (0, 0, 0x27) => {
            Some(NORTH_WEST_CORNER)
        }
        _ => None,
    }
}
fn mineral_triangle(
    out: &mut SurfaceMeshData,
    p: [[f32; 3]; 3],
    color: [f32; 4],
    g: &GridGeometry,
    origin: [i32; 2],
) {
    let v = p.map(Vec3::from_array);
    let Some(normal) = (v[1] - v[0]).cross(v[2] - v[0]).try_normalize() else {
        return;
    };
    let base = out.positions.len() as u32;
    out.positions.extend(p);
    out.normals.extend([normal.to_array(); 3]);
    out.uvs
        .extend(v.map(|p| cave_cliff::stone_uv(p, g, origin, false)));
    out.colors
        .extend(v.map(|p| cave_cliff::pigment(color, p, g, origin, false)));
    out.indices.extend([base, base + 1, base + 2]);
}

fn rim_width(p: Vec3, g: &GridGeometry, origin: [i32; 2]) -> f32 {
    let x = (p.x - g.origin_x) / g.tile_width + origin[0] as f32;
    let z = (p.z - g.origin_z) / g.tile_height + origin[1] as f32;
    (1.4 + cave_cliff::slab(x * 0.9 + 17., z * 0.9 - 29.) * 0.8) * g.tile_height
        / SOURCE_TILE_HEIGHT
}

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    points: [[f32; 3]; 4],
    normal: [f32; 3],
    cells: &[&VisualTile],
    g: &GridGeometry,
    index: usize,
    origin: [i32; 2],
) -> bool {
    let source = &cells[index].source;
    let Some(flags) = edges(source) else {
        return false;
    };
    let (w, e, n, s) = g.bounds(index % g.width, index / g.width);
    let expected = [[w, 0., n], [w, 0., s], [e, 0., s], [e, 0., n]];
    if normal != [0., 1., 0.]
        || points
            .iter()
            .zip(expected)
            .any(|(a, b)| Vec3::from_array(*a).distance(Vec3::from_array(b)) > 0.001)
    {
        return false;
    }
    let Some(water) = cells.iter().position(|t| {
        t.source.tileset_id == source.tileset_id
            && t.source.tile_index == 0x14
            && t.animation_frames
                .as_ref()
                .is_some_and(|f| f.len() > 1 && f.contains(&t.texture))
    }) else {
        return false;
    };
    // Sampling the owned animated water slot preserves the existing palette,
    // phase updates and shader provenance. No synthesized animation clock.
    append_top(
        &mut mesh.textured,
        [w, e, n, s],
        0.,
        cave_pool_uv(cells[water], g.uv(water % g.width, water / g.width)),
    );
    let u = g.tile_height / SOURCE_TILE_HEIGHT;
    let width = 1.4 * u;
    let h = 0.65 * u;
    let dark = source.tileset_id.as_ref() == "dark_cave";
    let color = if dark {
        [0.31, 0.37, 0.39, 1.]
    } else {
        [0.34, 0.345, 0.31, 1.]
    };
    // These isolated pool cells depict a corner nib, not two complete
    // shore runs. Extending both sides through the whole cell makes a cross
    // projecting into open water at every step of the rock wall.
    if flags == NORTH_EAST_CORNER || flags == NORTH_WEST_CORNER {
        let p = if flags == NORTH_EAST_CORNER {
            [[e, h, n], [e - width, 0., n], [e, 0., n + width]]
        } else {
            [[w, h, n], [w, 0., n + width], [w + width, 0., n]]
        };
        let bottom = [p[0][0], 0., p[0][2]];
        for triangle in [p, [p[0], bottom, p[1]], [p[0], p[2], bottom]] {
            mineral_triangle(&mut mesh.solid, triangle, color, g, origin);
        }
        return true;
    }
    for flag in [WEST, NORTH, EAST] {
        if flags & flag == 0 {
            continue;
        }
        for panel in 0..8 {
            let edge = |t: f32| match flag {
                WEST => Vec3::new(w, h, n + (s - n) * t),
                NORTH => Vec3::new(w + (e - w) * t, h, n),
                _ => Vec3::new(e, h, n + (s - n) * t),
            };
            let inner = |p: Vec3| {
                let width = rim_width(p, g, origin);
                let d = match flag {
                    WEST => Vec3::X,
                    NORTH => Vec3::Z,
                    _ => Vec3::NEG_X,
                };
                p - Vec3::Y * h + d * width
            };
            let a = edge(panel as f32 / 8.);
            let b = edge((panel + 1) as f32 / 8.);
            let (c, d) = (inner(b), inner(a));
            let p = if flag == EAST {
                [d, c, b, a]
            } else if flag == NORTH {
                [a, d, c, b]
            } else {
                [a, b, c, d]
            };
            for tri in [[p[0], p[1], p[2]], [p[0], p[2], p[3]]] {
                mineral_triangle(&mut mesh.solid, tri.map(|p| p.to_array()), color, g, origin);
            }
        }
    }
    true
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cave_pool_atlas_endpoints_stay_in_the_live_slot() {
        let mut tile = VisualTile {
            column: 12, row: 57, priority: false, texture: Handle::default(),
            animation_frames: Some(vec![Handle::default(); 2].into()),
            source: VisualTileSource {
                tileset_id: "dark_cave".into(), metatile_id: 0x27,
                subtile_column: 0, subtile_row: 1, tile_index: 0x14,
            },
        };
        let g = GridGeometry {
            width: 40, height: 72, tile_width: 8., tile_height: 8.,
            origin_x: -160., origin_z: -288.,
        };
        let original = g.uv(12, 57);
        let (u0, u1, v0, v1) = cave_pool_uv(&tile, original);
        for u in [u0, u1] { for v in [v0, v1] {
            assert_eq!((u * 40.).floor() as u32, 12);
            assert_eq!((v * 72.).floor() as u32, 57);
        }}
        tile.animation_frames = None;
        assert_eq!(cave_pool_uv(&tile, original), original);
        tile.animation_frames = Some(vec![Handle::default(); 2].into());
        tile.source.tile_index = 0x26;
        assert_eq!(cave_pool_uv(&tile, original), original);
        tile.source.tile_index = 0x14;
        tile.source.tileset_id = "johto".into();
        assert_eq!(cave_pool_uv(&tile, original), original);
    }
    #[test]
    fn shoreline_facets_join_at_source_seams_and_keep_water_level() {
        let g = GridGeometry {
            width: 3,
            height: 1,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let t: Vec<_> = (0..3)
            .map(|x| VisualTile {
                column: x,
                row: 0,
                priority: false,
                texture: Handle::default(),
                animation_frames: Some(vec![Handle::default(); 2].into()),
                source: VisualTileSource {
                    tileset_id: "cave".into(),
                    metatile_id: 0x3d,
                    subtile_column: x as u8,
                    subtile_row: 0,
                    tile_index: if x == 2 { 0x14 } else { 0x26 },
                },
            })
            .collect();
        let cells: Vec<_> = t.iter().collect();
        let mut parts = Vec::new();
        for x in 0..2 {
            let (w, e, n, s) = g.bounds(x, 0);
            let mut mesh = TerrainMeshData::default();
            assert!(append(
                &mut mesh,
                [[w, 0., n], [w, 0., s], [e, 0., s], [e, 0., n]],
                [0., 1., 0.],
                &cells,
                &g,
                x,
                [11, 19]
            ));
            assert!(mesh.textured.positions.iter().all(|p| p[1] == 0.));
            assert!(
                mesh.solid
                    .positions
                    .iter()
                    .all(|p| p[1] >= 0. && p[1] <= 0.65)
            );
            assert!(
                mesh.solid
                    .normals
                    .iter()
                    .all(|n| n[1] > 0. && (Vec3::from_array(*n).length() - 1.).abs() < 1e-5)
            );
            assert!(
                mesh.solid
                    .uvs
                    .iter()
                    .all(|uv| (uv[1] / 100000.).round() == 6.)
            );
            let mut seam: Vec<_> = mesh
                .solid
                .positions
                .iter()
                .filter(|p| p[0] == 8.)
                .copied()
                .collect();
            seam.sort_by(|a, b| a[2].total_cmp(&b[2]));
            seam.dedup();
            parts.push(seam);
        }
        assert_eq!(parts[0], parts[1]);
        let p = Vec3::new(8., 0.65, 0.);
        let shifted = GridGeometry { origin_x: -8., ..g };
        assert_eq!(
            rim_width(p, &g, [11, 19]),
            rim_width(p - Vec3::X * 8., &shifted, [11, 19])
        );
    }
    #[test]
    fn authentic_rim_keeps_full_live_water_plane_and_zero_support() {
        let mut t = (0..2)
            .map(|x| VisualTile {
                column: x,
                row: 0,
                priority: false,
                texture: Handle::default(),
                animation_frames: Some(vec![Handle::default(); 2].into()),
                source: VisualTileSource {
                    tileset_id: "dark_cave".into(),
                    metatile_id: 0x32,
                    subtile_column: if x == 0 { 3 } else { 0 },
                    subtile_row: 0,
                    tile_index: if x == 0 { 0x25 } else { 0x14 },
                },
            })
            .collect::<Vec<_>>();
        let g = GridGeometry {
            width: 2,
            height: 1,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let p = [[0., 0., 0.], [0., 0., 8.], [8., 0., 8.], [8., 0., 0.]];
        let mut mesh = TerrainMeshData {
            footing_heights: vec![0.; 2],
            ..Default::default()
        };
        assert!(append(
            &mut mesh,
            p,
            [0., 1., 0.],
            &t.iter().collect::<Vec<_>>(),
            &g,
            0,
            [0, 0]
        ));
        assert_eq!(mesh.textured.positions, p);
        assert_eq!(
            mesh.textured.uvs,
            [[0.53125, 0.0625], [0.53125, 0.9375],
                [0.96875, 0.9375], [0.96875, 0.0625]]
        );
        assert_eq!(mesh.footing_heights, vec![0.; 2]);
        assert!(mesh.solid.positions.iter().all(|p| p[1] <= 0.65));
        assert!(
            mesh.solid
                .positions
                .iter()
                .all(|p| p[0] >= 6.6 && p[2] <= 1.4)
        );
        assert_eq!(mesh.solid.indices.len(), 9);
        assert!(
            mesh.solid
                .uvs
                .iter()
                .all(|uv| (uv[1] / 100000.).round() == 6.)
        );
        let mut other_tileset = t[0].source.clone();
        other_tileset.tileset_id = "cave".into();
        assert!(edges(&other_tileset).is_none());
        let mut unknown = t[0].source.clone();
        unknown.subtile_column = 2;
        assert!(edges(&unknown).is_none());
        t[1].animation_frames = None;
        assert!(!append(
            &mut TerrainMeshData::default(),
            p,
            [0., 1., 0.],
            &t.iter().collect::<Vec<_>>(),
            &g,
            0,
            [0, 0]
        ));
    }
}
