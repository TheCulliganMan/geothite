//! One source-bound cottage, with repeatable window bays for wider drawings.
use super::kanto_craft::{block, face, mix, source_ink};
use super::*;

const LEFT: [[u16; 4]; 4] = [
    [5, 6, 7, 7],
    [0x15, 0x16, 0x17, 0x17],
    [0x25, 0x26, 0x0b, 0x0c],
    [0x1d, 0x1a, 0x1b, 0x1c],
];
const BAY: [[u16; 4]; 4] = [[7; 4], [0x17; 4], [0x23, 0x32, 0x0a, 0x23], [0x1a; 4]];
const RIGHT: [[u16; 4]; 4] = [
    [7, 7, 8, 9],
    [0x17, 0x17, 0x18, 0x19],
    [0x0a, 0x0a, 0x28, 0x29],
    [0x1a, 0x1a, 0x1a, 0x3c],
];

pub(super) fn matches(cells: &[&VisualTile], g: &GridGeometry, p: BuildingPlacement) -> bool {
    p.width >= 8
        && p.width % 4 == 0
        && p.height == 4
        && p.roof_rows == 2
        && p.column + p.width <= g.width
        && p.row + 4 <= g.height
        && (0..4).all(|y| {
            (0..p.width).all(|x| {
                let block = if x < 4 {
                    0x02
                } else if x >= p.width - 4 {
                    0x03
                } else {
                    0x09
                };
                let art = if x < 4 {
                    LEFT
                } else if x >= p.width - 4 {
                    RIGHT
                } else {
                    BAY
                };
                cells
                    .get((p.row + y) * g.width + p.column + x)
                    .is_some_and(|c| {
                        let s = &c.source;
                        s.tileset_id.as_ref() == "kanto"
                            && s.metatile_id == block
                            && usize::from(s.subtile_column) == x % 4
                            && usize::from(s.subtile_row) == y
                            && s.tile_index == art[y][x % 4]
                    })
            })
        })
}

#[derive(Clone)]
pub(super) struct Paint {
    roof: [[f32; 3]; 2],
    wall: [f32; 3],
    stone: [f32; 3],
    brick: [f32; 3],
    wood: [f32; 3],
    ink: [f32; 3],
    glass: [f32; 3],
    windows: Vec<(usize, [f32; 3])>,
}
fn linear(c: [u8; 4]) -> [f32; 3] {
    std::array::from_fn(|i| {
        let x = c[i] as f32 / 255.;
        if x <= 0.04045 {
            x / 12.92
        } else {
            ((x + 0.055) / 1.055).powf(2.4)
        }
    })
}
pub(super) fn paint(
    cells: &[&VisualTile],
    g: &GridGeometry,
    p: BuildingPlacement,
    images: &TerrainImageSamples,
) -> Option<Paint> {
    if !matches(cells, g, p) {
        return None;
    }
    let roles = |x: usize, y: usize| -> Option<[Option<[u8; 4]>; 4]> {
        let c = cells[(p.row + y) * g.width + p.column + x];
        let pixels = tile_rgba(images, c).ok()?;
        if pixels.len() != 256 {
            return None;
        }
        let mut colors = [None; 4];
        for (i, pixel) in pixels.chunks_exact(4).enumerate() {
            let color: [u8; 4] = pixel.try_into().ok()?;
            if color[3] != 255 {
                return None;
            }
            let slot = &mut colors[source_ink(c.source.tile_index, i % 8, i / 8)];
            if slot.is_some_and(|old| old != color) {
                return None;
            }
            *slot = Some(color);
        }
        Some(colors)
    };
    let roof = roles(2, 1)?;
    let wall = roles(1, 3)?;
    let door = roles(2, 3)?;
    let top = roles(2, 2)?;
    let first = roles(p.width - 4, 2)?;
    if roof[1] == roof[2] || wall[0] == wall[2] || first[0] == first[3] {
        return None;
    }
    let mut windows = Vec::new();
    for x in 4..p.width {
        let s = &cells[(p.row + 2) * g.width + p.column + x].source;
        if matches!(s.tile_index, 0x0a | 0x32) {
            let colors = roles(x, 2)?;
            if colors[0] == colors[3] {
                return None;
            }
            windows.push((x, linear(colors[0]?)));
        }
    }
    Some(Paint {
        roof: [linear(roof[2]?), linear(roof[1]?)],
        wall: linear(wall[0]?),
        stone: linear(wall[1]?),
        brick: linear(wall[2]?),
        wood: linear(door[2]?),
        ink: linear(first[3]?),
        glass: linear(top[0]?),
        windows,
    })
}

fn prism(out: &mut SurfaceMeshData, top: Vec<Vec3>, thickness: f32, color: [f32; 3], unit: f32) {
    let bottom = top
        .iter()
        .map(|&v| v - Vec3::Y * thickness)
        .collect::<Vec<_>>();
    let center = top.iter().chain(&bottom).copied().sum::<Vec3>() / (top.len() * 2) as f32;
    face(out, top.clone(), center, color, 0., unit);
    face(out, bottom.clone(), center, color, 0., unit);
    for i in 0..top.len() {
        let j = (i + 1) % top.len();
        face(
            out,
            vec![top[i], top[j], bottom[j], bottom[i]],
            center,
            color,
            0.,
            unit,
        );
    }
}

fn roof(out: &mut SurfaceMeshData, b: [f32; 4], unit: f32, p: &Paint) {
    let [w, e, n, s] = b;
    let mid = (n + s) * 0.5;
    let inset = unit * 1.5;
    let low = unit * 3.16;
    let high = unit * 4.43;
    let v = |x, y, z| Vec3::new(x, y, z);
    let ceramic = mix(p.roof[0], p.roof[1], 0.32);
    let lum = Vec3::from_array(ceramic).dot(Vec3::new(0.2126, 0.7152, 0.0722));
    let ceramic = mix(ceramic, [lum; 3], 0.24);
    let planes = [
        vec![
            v(w, low, s),
            v(e, low, s),
            v(e - inset, high, mid),
            v(w + inset, high, mid),
        ],
        vec![
            v(e, low, n),
            v(w, low, n),
            v(w + inset, high, mid),
            v(e - inset, high, mid),
        ],
        vec![v(w, low, n), v(w, low, s), v(w + inset, high, mid)],
        vec![v(e, low, s), v(e, low, n), v(e - inset, high, mid)],
    ];
    for (side, plane) in planes.into_iter().enumerate() {
        prism(
            out,
            plane.clone(),
            unit * 0.13,
            mix(p.roof[0], p.ink, 0.28),
            unit,
        );
        let rows = 5;
        let cols = ((plane[1] - plane[0]).length() / (unit * 0.72)).ceil() as usize;
        for row in 0..rows {
            for col in 0..cols {
                let at = |u: f32, t: f32| {
                    let upper = if plane.len() == 4 {
                        plane[3].lerp(plane[2], u)
                    } else {
                        plane[2]
                    };
                    plane[0].lerp(plane[1], u).lerp(upper, t) + Vec3::Y * unit * 0.025
                };
                let a = (col as f32 + 0.018) / cols as f32;
                let b = (col as f32 + 0.982) / cols as f32;
                let c = (row as f32 + 0.02) / rows as f32;
                let d = (row as f32 + 0.98) / rows as f32;
                let gain = 0.90 + ((row * 3 + col * 7 + side) % 5) as f32 * 0.018;
                prism(
                    out,
                    vec![at(a, c), at(b, c), at(b, d), at(a, d)],
                    unit * 0.045,
                    ceramic.map(|x| x * gain),
                    unit,
                );
            }
        }
    }
    block(
        out,
        [
            w + inset - unit * 0.09,
            e - inset + unit * 0.09,
            high,
            high + unit * 0.10,
            mid - unit * 0.10,
            mid + unit * 0.10,
        ],
        unit * 0.025,
        ceramic,
        0.,
        unit,
    );
}

pub(super) fn append(out: &mut SurfaceMeshData, b: [f32; 4], unit: f32, door: f32, p: Paint) {
    let first = out.positions.len();
    let origin = Vec3::new(b[0], 0., b[2]);
    let b = [0., b[1] - b[0], 0., b[3] - b[2]];
    let door = door - origin.x;
    let lum = Vec3::from_array(p.wall).dot(Vec3::new(0.2126, 0.7152, 0.0722));
    let p = Paint {
        wall: mix(p.wall, [lum * 1.03, lum, lum * 0.94], 0.48),
        ..p
    };
    let [w, e, n, s] = b;
    let back = n + unit * 0.46;
    let front = s - unit * 0.52;
    let stone = mix(p.stone, p.wall, 0.25);
    let mortar = mix(p.stone, p.ink, 0.24);
    let frame = mix(p.stone, p.wood, 0.28);
    let wood = mix(p.wood, stone, 0.16);
    let mut part = |b, bevel, color, kind| block(out, b, unit * bevel, color, kind, unit);
    part(
        [
            w + unit * 0.35,
            e - unit * 0.35,
            0.,
            unit * 0.29,
            back - unit * 0.04,
            front + unit * 0.10,
        ],
        0.045,
        stone,
        3.,
    );
    part(
        [
            w + unit * 0.40,
            e - unit * 0.40,
            unit * 0.26,
            unit * 3.08,
            back,
            front,
        ],
        0.025,
        p.wall,
        3.,
    );
    // Low masonry continues around every wall; upper plaster stays quiet.
    for side in 0..4 {
        let (a, b, z) = if side < 2 {
            (
                w + unit * 0.40,
                e - unit * 0.40,
                if side == 0 { front } else { back },
            )
        } else {
            (
                back,
                front,
                if side == 2 {
                    w + unit * 0.40
                } else {
                    e - unit * 0.40
                },
            )
        };
        let count = ((b - a) / (unit * 0.74)).ceil() as i32;
        let step = (b - a) / count as f32;
        for row in 0..2 {
            for col in -1..count {
                let start = a + step * (col as f32 + if row == 0 { 0.5 } else { 0. });
                let lo = start.max(a) + unit * 0.009;
                let hi = (start + step).min(b) - unit * 0.009;
                if hi <= lo {
                    continue;
                }
                let bottom = unit * (0.32 + row as f32 * 0.28);
                let top = bottom + unit * 0.25;
                let bounds = if side < 2 {
                    [lo, hi, bottom, top, z - unit * 0.025, z + unit * 0.025]
                } else {
                    [z - unit * 0.025, z + unit * 0.025, bottom, top, lo, hi]
                };
                let color = mix(p.brick, stone, 0.4)
                    .map(|v| v * (0.95 + (col + row + 2).rem_euclid(4) as f32 * 0.02));
                part(bounds, 0.01, color, 2.);
            }
        }
    }
    for y in [0.30, 0.90, 2.96] {
        part(
            [
                w + unit * 0.36,
                e - unit * 0.36,
                unit * y,
                unit * (y + 0.07),
                back - unit * 0.03,
                front + unit * 0.07,
            ],
            0.015,
            stone,
            3.,
        );
    }
    // A wooden single door sits at the actual left-hand source seam.
    part(
        [
            door - unit * 0.91,
            door + unit * 0.91,
            unit * 0.25,
            unit * 2.75,
            front + unit * 0.01,
            front + unit * 0.16,
        ],
        0.035,
        frame,
        1.,
    );
    part(
        [
            door - unit * 0.76,
            door + unit * 0.76,
            unit * 0.29,
            unit * 2.59,
            front + unit * 0.16,
            front + unit * 0.20,
        ],
        0.02,
        wood,
        1.,
    );
    part(
        [
            door - unit * 0.53,
            door + unit * 0.53,
            unit * 1.42,
            unit * 2.34,
            front + unit * 0.20,
            front + unit * 0.25,
        ],
        0.015,
        mortar,
        1.,
    );
    part(
        [
            door - unit * 0.40,
            door + unit * 0.40,
            unit * 1.56,
            unit * 2.20,
            front + unit * 0.25,
            front + unit * 0.26,
        ],
        0.,
        p.glass.map(|v| v * 0.82),
        -1.,
    );
    part(
        [
            door - unit * 0.026,
            door + unit * 0.026,
            unit * 1.56,
            unit * 2.20,
            front + unit * 0.26,
            front + unit * 0.29,
        ],
        0.008,
        frame,
        1.,
    );
    for dx in [-0.37, 0.37] {
        part(
            [
                door + unit * (dx - 0.26),
                door + unit * (dx + 0.26),
                unit * 0.48,
                unit * 1.14,
                front + unit * 0.20,
                front + unit * 0.23,
            ],
            0.012,
            mix(wood, p.ink, 0.12),
            1.,
        );
    }
    part(
        [
            door + unit * 0.55,
            door + unit * 0.62,
            unit * 1.13,
            unit * 1.31,
            front + unit * 0.23,
            front + unit * 0.31,
        ],
        0.01,
        mix(p.wood, p.wall, 0.35),
        1.,
    );
    part(
        [
            door - unit * 1.02,
            door + unit * 1.02,
            0.,
            unit * 0.25,
            front + unit * 0.05,
            s - unit * 0.02,
        ],
        0.035,
        stone,
        3.,
    );
    for &(cell, color) in &p.windows {
        let x = (cell as f32 + 0.5) * unit;
        part(
            [
                x - unit * 0.46,
                x + unit * 0.46,
                unit * 1.16,
                unit * 2.56,
                front,
                front + unit * 0.11,
            ],
            0.018,
            frame,
            1.,
        );
        part(
            [
                x - unit * 0.35,
                x + unit * 0.35,
                unit * 1.30,
                unit * 2.43,
                front + unit * 0.11,
                front + unit * 0.12,
            ],
            0.,
            color.map(|v| v * 0.84),
            -1.,
        );
        part(
            [
                x - unit * 0.50,
                x + unit * 0.50,
                unit * 1.10,
                unit * 1.20,
                front - unit * 0.02,
                front + unit * 0.19,
            ],
            0.018,
            stone,
            3.,
        );
    }
    // Fascia, soffit and visible rafter tails close the eaves on every orbit.
    part(
        [
            w + unit * 0.17,
            e - unit * 0.17,
            unit * 3.01,
            unit * 3.16,
            n + unit * 0.13,
            s - unit * 0.12,
        ],
        0.025,
        mix(wood, p.ink, 0.25),
        1.,
    );
    for z in [n + unit * 0.14, s - unit * 0.14] {
        part(
            [
                w + unit * 0.17,
                e - unit * 0.17,
                unit * 2.96,
                unit * 3.08,
                z - unit * 0.07,
                z + unit * 0.07,
            ],
            0.012,
            frame,
            1.,
        );
        for i in 0..((e - w) / unit) as usize {
            let x = w + unit * (i as f32 + 0.52);
            part(
                [
                    x - unit * 0.04,
                    x + unit * 0.04,
                    unit * 2.88,
                    unit * 3.04,
                    z - unit * 0.12,
                    z + unit * 0.12,
                ],
                0.01,
                wood,
                1.,
            );
        }
    }
    drop(part);
    roof(
        out,
        [
            w + unit * 0.10,
            e - unit * 0.10,
            n + unit * 0.07,
            s - unit * 0.04,
        ],
        unit,
        &p,
    );
    // The established cottage chimney retains a recessed flue and four walls.
    let x = w + unit * 1.42;
    let z = n + unit * 0.90;
    for b in [
        [
            x - unit * 0.27,
            x + unit * 0.27,
            unit * 3.75,
            unit * 4.64,
            z - unit * 0.27,
            z - unit * 0.19,
        ],
        [
            x - unit * 0.27,
            x + unit * 0.27,
            unit * 3.75,
            unit * 4.64,
            z + unit * 0.19,
            z + unit * 0.27,
        ],
        [
            x - unit * 0.27,
            x - unit * 0.19,
            unit * 3.75,
            unit * 4.64,
            z - unit * 0.19,
            z + unit * 0.19,
        ],
        [
            x + unit * 0.19,
            x + unit * 0.27,
            unit * 3.75,
            unit * 4.64,
            z - unit * 0.19,
            z + unit * 0.19,
        ],
    ] {
        block(out, b, unit * 0.015, stone, 3., unit);
    }
    block(
        out,
        [
            x - unit * 0.19,
            x + unit * 0.19,
            unit * 4.40,
            unit * 4.42,
            z - unit * 0.19,
            z + unit * 0.19,
        ],
        0.,
        p.ink,
        1.,
        unit,
    );
    for v in &mut out.positions[first..] {
        *v = (Vec3::from_array(*v) + origin).to_array();
    }
}

#[cfg(test)]
pub(super) fn fixture(
    width: usize,
) -> (
    Vec<VisualTile>,
    GridGeometry,
    BuildingPlacement,
    TerrainImageSamples,
) {
    let g = GridGeometry {
        width,
        height: 5,
        tile_width: 8.,
        tile_height: 8.,
        origin_x: 0.,
        origin_z: 0.,
    };
    let p = BuildingPlacement {
        column: 0,
        row: 0,
        width,
        height: 4,
        roof_rows: 2,
        ground_tile_index: 0x2c,
    };
    let mut assets = Assets::<Image>::default();
    let mut images = TerrainImageSamples::default();
    let mut cells = Vec::new();
    for y in 0..5 {
        for x in 0..width {
            let (block, tile) = if y == 4 {
                (0x0a, 0x2c)
            } else if x < 4 {
                (0x02, LEFT[y][x])
            } else if x >= width - 4 {
                (0x03, RIGHT[y][x % 4])
            } else {
                (0x09, BAY[y][x % 4])
            };
            let colors = if tile == 0x17 {
                [
                    [222, 255, 222, 255],
                    [74, 185, 235, 255],
                    [51, 130, 180, 255],
                    [57, 57, 57, 255],
                ]
            } else if tile == 0x1a {
                [
                    [222, 245, 222, 255],
                    [160, 160, 150, 255],
                    [105, 110, 100, 255],
                    [57, 57, 57, 255],
                ]
            } else {
                [
                    [200, 240, 225, 255],
                    [210, 190, 130, 255],
                    [142, 105, 44, 255],
                    [57, 57, 57, 255],
                ]
            };
            let texture = assets.add(Image::default());
            images.pixels.insert(
                texture.id(),
                TileImageSample::Rgba(
                    (0..64)
                        .flat_map(|i| {
                            colors[if matches!(tile, 0x17 | 0x1a | 0x0a | 0x32 | 0x0b | 0x1b) {
                                source_ink(tile, i % 8, i / 8)
                            } else {
                                0
                            }]
                        })
                        .collect(),
                ),
            );
            cells.push(VisualTile {
                column: x as u32,
                row: y as u32,
                source: VisualTileSource {
                    tileset_id: "kanto".into(),
                    metatile_id: block,
                    subtile_column: (x % 4) as u8,
                    subtile_row: (y % 4) as u8,
                    tile_index: tile,
                },
                texture,
                animation_frames: None,
                priority: false,
            });
        }
    }
    (cells, g, p, images)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn kanto_house_preserves_live_independent_window_roles_and_rejects_missing_or_unknown_paint() {
        for width in [8, 12, 16] {
            let (tiles, g, p, mut images) = fixture(width);
            let cells = tiles.iter().collect::<Vec<_>>();
            let original = paint(&cells, &g, p, &images).unwrap();
            assert_eq!(original.windows.len(), width / 2 - 2);
            let x = original.windows[0].0;
            let cell = cells[2 * width + x];
            let texture = cell.texture.id();
            let mut pixels = tile_rgba(&images, cell).unwrap().to_vec();
            for (i, color) in pixels.chunks_exact_mut(4).enumerate() {
                if source_ink(cell.source.tile_index, i % 8, i / 8) == 0 {
                    color.copy_from_slice(&[255, 255, 132, 255]);
                }
            }
            images.pixels.insert(texture, TileImageSample::Rgba(pixels));
            let night = paint(&cells, &g, p, &images).unwrap();
            assert_ne!(night.windows[0], original.windows[0]);
            assert_eq!(night.windows[1..], original.windows[1..]);
            assert_eq!(night.roof, original.roof);
            assert_eq!(night.wall, original.wall);
            assert_eq!(night.wood, original.wood);
            images.pixels.insert(
                texture,
                TileImageSample::Rgba((0..64).flat_map(|i| [i as u8, 0, 255, 255]).collect()),
            );
            assert!(paint(&cells, &g, p, &images).is_none());
            images.pixels.remove(&cells[width + 2].texture.id());
            assert!(paint(&cells, &g, p, &images).is_none());
        }
    }
    #[test]
    fn kanto_house_roof_is_closed_bounded_and_material_stable_under_world_translation() {
        for width in [8, 16] {
            let (tiles, g, p, images) = fixture(width);
            let cells = tiles.iter().collect::<Vec<_>>();
            let paint = paint(&cells, &g, p, &images).unwrap();
            let mut a = SurfaceMeshData::default();
            append(
                &mut a,
                [0., width as f32 * 8., 0., 32.],
                8.,
                24.,
                paint.clone(),
            );
            let mut b = SurfaceMeshData::default();
            append(
                &mut b,
                [-216., width as f32 * 8. - 216., 408., 440.],
                8.,
                -192.,
                paint,
            );
            assert_eq!(a.indices, b.indices);
            assert_eq!(a.colors, b.colors);
            assert_eq!(a.uvs, b.uvs);
            assert!(a.indices.len() / 3 > 1000 && a.indices.len() / 3 < 10000);
            assert!(a.positions.iter().all(|p| p[0] >= 0.
                && p[0] <= width as f32 * 8.
                && p[2] >= 0.
                && p[2] <= 32.
                && p[1] >= 0.
                && p[1] < 8. * 4.7));
            for (a, b) in a.positions.iter().zip(&b.positions) {
                assert!(
                    (a[0] - 216. - b[0]).abs() < 0.0001
                        && (a[1] - b[1]).abs() < 0.0001
                        && (a[2] + 408. - b[2]).abs() < 0.0001
                );
            }
        }
        // The actual roof primitive seals each course's bottom and all returns.
        for top in [
            vec![Vec3::ZERO, Vec3::X, Vec3::new(0.5, 1., 0.5)],
            vec![
                Vec3::ZERO,
                Vec3::X,
                Vec3::new(1., 1., 1.),
                Vec3::new(0., 1., 1.),
            ],
        ] {
            let mut mesh = SurfaceMeshData::default();
            prism(&mut mesh, top, 0.1, [0.2, 0.4, 0.5], 1.);
            let mut edges = std::collections::BTreeMap::new();
            let mut volume = 0.;
            for tri in mesh.indices.chunks_exact(3) {
                let p =
                    [tri[0], tri[1], tri[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
                assert!((p[1] - p[0]).cross(p[2] - p[0]).length_squared() > 1e-9);
                volume += p[0].dot(p[1].cross(p[2])) / 6.;
                for j in 0..3 {
                    let mut edge = [
                        p[j].to_array().map(f32::to_bits),
                        p[(j + 1) % 3].to_array().map(f32::to_bits),
                    ];
                    edge.sort();
                    *edges.entry(edge).or_insert(0) += 1;
                }
            }
            assert!(volume > 0.);
            assert!(edges.values().all(|&n| n == 2));
        }
    }
}
