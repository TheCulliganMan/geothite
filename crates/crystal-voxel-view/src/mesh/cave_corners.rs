//! Shared corner breaks proven from surrounding source art, never viewport edges.
use super::*;
use bevy::prelude::Vec2;

#[derive(Clone, Copy)]
struct RockCorner {
    sign: Vec2,
    wide: bool,
}

pub(in crate::mesh) struct RockCorners {
    width: usize,
    height: usize,
    signs: Vec<Option<RockCorner>>,
}

impl RockCorners {
    pub(in crate::mesh) fn new(cells: &[&VisualTile], g: &GridGeometry) -> Self {
        let mut result = Self {
            width: g.width + 1,
            height: g.height + 1,
            signs: vec![None; (g.width + 1) * (g.height + 1)],
        };
        let sample = |x: i32, z: i32| {
            if x < 0 || z < 0 || x >= g.width as i32 || z >= g.height as i32 {
                None
            } else {
                cells
                    .get(z as usize * g.width + x as usize)
                    .map(|t| &t.source)
            }
        };
        let tall = |s: &VisualTileSource| {
            matches!(s.tileset_id.as_ref(), "cave" | "dark_cave")
                && crate::cave::cave_shape(s)
                    .is_some_and(|s| s.surface_height(SOURCE_TILE_HEIGHT) >= 16.)
        };
        for z in 1..g.height as i32 {
            for x in 1..g.width as i32 {
                let mut occupied = None;
                let mut valid = true;
                for (dx, dz) in [(-1, -1), (0, -1), (-1, 0), (0, 0)] {
                    let Some(source) = sample(x + dx, z + dz) else {
                        valid = false;
                        break;
                    };
                    if tall(source) {
                        if occupied.is_some() {
                            valid = false;
                            break;
                        }
                        occupied = Some((dx, dz, source.tileset_id.clone()));
                    } else if source.metatile_id == 0x09
                        || !matches!(source.tileset_id.as_ref(), "cave" | "dark_cave")
                        || !matches!(source.tile_index, 0x01 | 0x14 | 0x16)
                        || !crate::cave::cave_shape(source)
                            .is_some_and(|s| s.surface_height(SOURCE_TILE_HEIGHT) <= 6.)
                    {
                        valid = false;
                        break;
                    }
                }
                let Some((dx, dz, tileset)) = occupied else {
                    continue;
                };
                if !valid {
                    continue;
                }
                // A two-cell-deep mass prevents opposing breaks from collapsing
                // thin pillars or a one-cell bank. Unknown art is not rock proof.
                let bx = x + if dx < 0 { -2 } else { 0 };
                let bz = z + if dz < 0 { -2 } else { 0 };
                if !(0..2).all(|oz| {
                    (0..2).all(|ox| {
                        sample(bx + ox, bz + oz).is_some_and(|s| s.tileset_id == tileset && tall(s))
                    })
                }) {
                    continue;
                }
                let bx = x + if dx < 0 { -3 } else { 0 };
                let bz = z + if dz < 0 { -3 } else { 0 };
                let wide = (0..3).all(|oz| {
                    (0..3).all(|ox| {
                        sample(bx + ox, bz + oz).is_some_and(|s| s.tileset_id == tileset && tall(s))
                    })
                });
                result.signs[z as usize * result.width + x as usize] = Some(RockCorner {
                    sign: Vec2::new(if dx < 0 { -1. } else { 1. }, if dz < 0 { -1. } else { 1. }),
                    wide,
                });
            }
        }
        result
    }

    pub(in crate::mesh) fn point(&self, p: Vec3, g: &GridGeometry, origin: [i32; 2]) -> Vec3 {
        let y = p.y / g.tile_height * SOURCE_TILE_HEIGHT;
        if y <= crate::cave::CAVE_SHELF_HEIGHT {
            return p;
        }
        let world = Vec2::new(
            (p.x - g.origin_x) / g.tile_width,
            (p.z - g.origin_z) / g.tile_height,
        );
        let bx = world.x.floor() as i32;
        let bz = world.y.floor() as i32;
        let mut beveled = p;
        // Broad corner breaks reach beyond one cell. Include the full support
        // on both sides of a cell boundary so the field cannot jump there.
        for z in bz - 2..=bz + 2 {
            for x in bx - 2..=bx + 2 {
                if x < 0 || z < 0 || x >= self.width as i32 || z >= self.height as i32 {
                    continue;
                }
                let Some(proven) = self.signs[z as usize * self.width + x as usize] else {
                    continue;
                };
                let sign = proven.sign;
                let corner = Vec2::new(x as f32, z as f32);
                let local = (world - corner) * sign;
                let variation = new_bark::meadow_noise(
                    (corner.x + origin[0] as f32) * 0.5 + 61.,
                    (corner.y + origin[1] as f32) * 0.5 - 31.,
                ) + 0.5;
                let radius = if proven.wide {
                    1.10 + variation * 0.35
                } else {
                    0.70 + variation * 0.25
                };
                if local.min_element() >= -0.02 && local.x + local.y < radius {
                    let rise = ((y - 6.) / 16.).clamp(0., 1.);
                    let rise = rise * rise * (3. - 2. * rise);
                    let shift = sign * (radius - local.x - local.y) * 0.28 * rise;
                    beveled.x += shift.x * g.tile_width;
                    beveled.z += shift.y * g.tile_height;
                }
            }
        }
        rock_point(beveled, g, origin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    #[test]
    fn cave_corner_breaks_join_caps_and_sides_without_moving_low_surfaces() {
        let g = GridGeometry {
            width: 6,
            height: 6,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut tiles = (0..36)
            .map(|i| VisualTile {
                column: (i % 6) as u32,
                row: (i / 6) as u32,
                animation_frames: None,
                priority: false,
                texture: Default::default(),
                source: VisualTileSource {
                    tileset_id: Arc::from("cave"),
                    metatile_id: 0x02,
                    subtile_column: (i % 6 % 4) as u8,
                    subtile_row: (i / 6 % 4) as u8,
                    tile_index: 0x16,
                },
            })
            .collect::<Vec<_>>();
        for z in 2..4 {
            for x in 2..4 {
                tiles[z * 6 + x].source.metatile_id = 0x0d;
                tiles[z * 6 + x].source.tile_index = 0x26;
            }
        }
        let refs = tiles.iter().collect::<Vec<_>>();
        let corners = RockCorners::new(&refs, &g);
        for y in [0., 3., 6.] {
            let p = Vec3::new(16., y, 16.);
            assert_eq!(corners.point(p, &g, [0, 0]), p);
        }
        let p = Vec3::new(16., 40., 16.);
        assert!((corners.point(p, &g, [0, 0]) - rock_point(p, &g, [0, 0])).length() > 1.);
        let mut cap = SurfaceMeshData::default();
        let mut side = SurfaceMeshData::default();
        quad_with_point(
            &mut cap,
            [
                [16., 40., 16.],
                [16., 40., 24.],
                [24., 40., 24.],
                [24., 40., 16.],
            ],
            [0.4; 4],
            &g,
            [0, 0],
            |p, g, o| corners.point(p, g, o),
            false,
        );
        quad_with_point(
            &mut side,
            [
                [16., 0., 16.],
                [16., 40., 16.],
                [24., 40., 16.],
                [24., 0., 16.],
            ],
            [0.4; 4],
            &g,
            [0, 0],
            |p, g, o| corners.point(p, g, o),
            false,
        );
        for x in [16., 18., 20., 22., 24.] {
            let p = corners.point(Vec3::new(x, 40., 16.), &g, [0, 0]).to_array();
            assert!(cap.positions.contains(&p));
            assert!(side.positions.contains(&p));
        }
        for tri in cap.indices.chunks_exact(3) {
            let p = [tri[0], tri[1], tri[2]].map(|i| Vec3::from_array(cap.positions[i as usize]));
            assert!((p[1] - p[0]).cross(p[2] - p[0]).y > 0.);
        }
        for height in [32., 40., 48.] {
            for z in 0..32 {
                for x in 0..32 {
                    let p = Vec3::new(16. + x as f32 * 0.5, height, 16. + z as f32 * 0.5);
                    let a = corners.point(p, &g, [0, 0]);
                    let b = corners.point(p + Vec3::Z * 0.5, &g, [0, 0]);
                    let c = corners.point(p + Vec3::X * 0.5, &g, [0, 0]);
                    assert!((b - a).cross(c - a).y > 0., "folded beveled crown at {p:?}");
                }
            }
        }
        assert!(
            side.normals
                .iter()
                .all(|n| Vec3::from_array(*n).is_finite())
        );
        let mut moved = g;
        moved.origin_x = -8.;
        assert!(
            (corners.point(p, &g, [0, 0])
                - corners.point(p - Vec3::X * 8., &moved, [0, 0])
                - Vec3::X * 8.)
                .length()
                < 1e-5
        );
        let mut broad = tiles.clone();
        for tile in &mut broad {
            tile.source.metatile_id = 0x02;
            tile.source.tile_index = 0x16;
        }
        for z in 1..5 {
            for x in 1..5 {
                broad[z * 6 + x].source.metatile_id = 0x0d;
                broad[z * 6 + x].source.tile_index = 0x26;
            }
        }
        let broad_refs = broad.iter().collect::<Vec<_>>();
        let broad_corners = RockCorners::new(&broad_refs, &g);
        for z in 0..64 {
            for x in 0..64 {
                let p = Vec3::new(8. + x as f32 * 0.5, 40., 8. + z as f32 * 0.5);
                let a = broad_corners.point(p, &g, [0, 0]);
                let b = broad_corners.point(p + Vec3::Z * 0.5, &g, [0, 0]);
                let c = broad_corners.point(p + Vec3::X * 0.5, &g, [0, 0]);
                assert!(
                    (b - a).cross(c - a).y > 0.,
                    "folded broad rock corner at {p:?}"
                );
            }
        }
        let mut thin = tiles.clone();
        for z in 2..4 {
            thin[z * 6 + 3].source.metatile_id = 0x02;
            thin[z * 6 + 3].source.tile_index = 0x16;
        }
        let thin_refs = thin.iter().collect::<Vec<_>>();
        assert_eq!(
            RockCorners::new(&thin_refs, &g).point(p, &g, [0, 0]),
            rock_point(p, &g, [0, 0])
        );
        let mut dark = tiles.clone();
        for tile in &mut dark {
            tile.source.tileset_id = Arc::from("dark_cave");
        }
        let dark_refs = dark.iter().collect::<Vec<_>>();
        assert_eq!(
            RockCorners::new(&dark_refs, &g).point(p, &g, [0, 0]),
            corners.point(p, &g, [0, 0])
        );
        drop(refs);
        tiles[1 * 6 + 1].source.metatile_id = 0x09;
        let refs = tiles.iter().collect::<Vec<_>>();
        assert_eq!(
            RockCorners::new(&refs, &g).point(p, &g, [0, 0]),
            rock_point(p, &g, [0, 0])
        );
    }
}
