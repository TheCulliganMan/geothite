//! One shared gym for the complete Kanto 0c/0d/0e + 10/11/12 drawing.
use super::kanto_craft::{block, face, mix, source_ink as ink};
use super::*;

const TILES: [[u16; 12]; 8] = [
    [5, 6, 0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 8, 9],
    [
        0x15, 0x38, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x38, 0x19,
    ],
    [
        0x15, 0x38, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x38, 0x19,
    ],
    [
        0x15, 0x16, 0x17, 0x17, 0x17, 0x17, 0x17, 0x17, 0x17, 0x17, 0x18, 0x19,
    ],
    [
        0x25, 0x26, 0x0a, 0x0a, 0x22, 0x2f, 0x3f, 0x22, 0x0a, 0x0a, 0x28, 0x29,
    ],
    [
        0x0f, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x1f,
    ],
    [
        0x0f, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0b, 0x0c, 0x0a, 0x1f,
    ],
    [
        0x1d, 0x1a, 0x1a, 0x1a, 0x1a, 0x1a, 0x1a, 0x1a, 0x1b, 0x1c, 0x1a, 0x3c,
    ],
];
const WINDOWS: [(usize, usize); 12] = [
    (2, 4),
    (3, 4),
    (8, 4),
    (9, 4),
    (1, 6),
    (2, 6),
    (3, 6),
    (4, 6),
    (5, 6),
    (6, 6),
    (7, 6),
    (10, 6),
];

#[derive(Clone, Copy)]
pub(super) struct Paint {
    roof: [[f32; 3]; 2],
    wall: [f32; 3],
    trim: [f32; 3],
    ink: [f32; 3],
    windows: [[f32; 3]; 12],
    door: [f32; 3],
    glass: [f32; 3],
}
fn linear(c: [u8; 4]) -> [f32; 3] {
    std::array::from_fn(|i| {
        let v = c[i] as f32 / 255.;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    })
}
pub(super) fn paint(
    cells: &[&VisualTile],
    g: &GridGeometry,
    p: BuildingPlacement,
    images: &TerrainImageSamples,
) -> Option<Paint> {
    if p.width != 12 || p.height != 8 || p.column + 12 > g.width || p.row + 8 > g.height {
        return None;
    }
    for (y, row) in TILES.iter().enumerate() {
        for (x, &index) in row.iter().enumerate() {
            let s = &cells.get((p.row + y) * g.width + p.column + x)?.source;
            if s.tileset_id.as_ref() != "kanto" || s.tile_index != index {
                return None;
            }
        }
    }
    let sample = |x: usize, y: usize| -> Option<[Option<[u8; 4]>; 4]> {
        let cell = cells[(p.row + y) * g.width + p.column + x];
        let rgba = tile_rgba(images, cell).ok()?;
        if rgba.len() != 256 {
            return None;
        }
        let mut colors = [None; 4];
        for (i, pixel) in rgba.chunks_exact(4).enumerate() {
            let color: [u8; 4] = pixel.try_into().ok()?;
            if color[3] != 255 {
                return None;
            }
            let slot = &mut colors[ink(cell.source.tile_index, i % 8, i / 8)];
            if slot.is_some_and(|old| old != color) {
                return None;
            }
            *slot = Some(color);
        }
        Some(colors)
    };
    let roof = sample(4, 1)?;
    let wall = sample(4, 5)?;
    if roof[1] == roof[2] || wall[0] == wall[1] {
        return None;
    }
    let mut windows = [[0.; 3]; 12];
    for (i, (x, y)) in WINDOWS.into_iter().enumerate() {
        let c = sample(x, y)?;
        if c[0] == c[3] {
            return None;
        }
        windows[i] = linear(c[0]?);
    }
    let window = sample(2, 4)?;
    let upper = sample(8, 6)?;
    let door = sample(8, 7)?;
    Some(Paint {
        roof: [linear(roof[2]?), linear(roof[1]?)],
        wall: linear(wall[0]?),
        trim: linear(wall[1]?),
        ink: linear(window[3]?),
        windows,
        door: linear(door[2]?),
        glass: linear(upper[0]?),
    })
}

fn prism(out: &mut SurfaceMeshData, top: Vec<Vec3>, thickness: f32, color: [f32; 3], unit: f32) {
    let bottom = top
        .iter()
        .map(|&p| p - Vec3::Y * thickness)
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
fn roof(out: &mut SurfaceMeshData, b: [f32; 4], unit: f32, p: Paint) {
    let ceramic = mix(p.roof[0], p.roof[1], 0.20);
    let mineral = Vec3::from_array(ceramic).dot(Vec3::new(0.2126, 0.7152, 0.0722));
    let ceramic = mix(ceramic, [mineral; 3], 0.28);
    let [w, e, n, s] = b;
    let mid = (n + s) * 0.5;
    let inset = unit * 2.0;
    let y = unit * 4.88;
    let high = unit * 6.27;
    let v = |x, y, z| Vec3::new(x, y, z);
    let planes = [
        vec![
            v(w, y, s),
            v(e, y, s),
            v(e - inset, high, mid),
            v(w + inset, high, mid),
        ],
        vec![
            v(e, y, n),
            v(w, y, n),
            v(w + inset, high, mid),
            v(e - inset, high, mid),
        ],
        vec![v(w, y, n), v(w, y, s), v(w + inset, high, mid)],
        vec![v(e, y, s), v(e, y, n), v(e - inset, high, mid)],
    ];
    for (side, plane) in planes.into_iter().enumerate() {
        prism(
            out,
            plane.clone(),
            unit * 0.16,
            mix(p.roof[0], p.ink, 0.30),
            unit,
        );
        // Small closed ceramic courses follow all four slopes. The hip's
        // triangular tips use three vertices, avoiding degenerate quads.
        let columns = if side < 2 { 20 } else { 12 };
        let rows = 6;
        for row in 0..rows {
            for col in 0..columns {
                let a = (col as f32 + 0.025) / columns as f32;
                let b = (col as f32 + 0.975) / columns as f32;
                let c = (row as f32 + 0.025) / rows as f32;
                let d = (row as f32 + 0.975) / rows as f32;
                let at = |u: f32, t: f32| {
                    let upper = if plane.len() == 4 {
                        plane[3].lerp(plane[2], u)
                    } else {
                        plane[2]
                    };
                    plane[0].lerp(plane[1], u).lerp(upper, t) + Vec3::Y * unit * 0.035
                };
                let tile = vec![at(a, c), at(b, c), at(b, d), at(a, d)];
                let gain = 0.82 + ((row * 7 + col * 3 + side) % 5) as f32 * 0.018;
                prism(out, tile, unit * 0.065, ceramic.map(|v| v * gain), unit);
            }
        }
    }
    // Closed ridge cap carries a restrained source-blue highlight.
    block(
        out,
        [
            w + inset - unit * 0.12,
            e - inset + unit * 0.12,
            high - unit * 0.02,
            high + unit * 0.12,
            mid - unit * 0.12,
            mid + unit * 0.12,
        ],
        unit * 0.035,
        mix(p.roof[0], p.roof[1], 0.25),
        0.,
        unit,
    );
}

pub(super) fn append(out: &mut SurfaceMeshData, b: [f32; 4], unit: f32, door: f32, p: Paint) {
    // Author in one bounded local material domain. World-space UVs can leave
    // the source-paint tag's range as the retained viewport moves, applying
    // moon tint to only part of a roof. Translate geometry after authoring;
    // pigment, grain, ceramic response and source illumination stay stable.
    let first = out.positions.len();
    let origin = Vec3::new(b[0], 0., b[2]);
    let b = [0., b[1] - b[0], 0., b[3] - b[2]];
    let door = door - origin.x;
    let luminance = Vec3::from_array(p.wall).dot(Vec3::new(0.2126, 0.7152, 0.0722));
    let p = Paint {
        wall: mix(
            p.wall,
            [luminance * 1.03, luminance, luminance * 0.94],
            0.60,
        ),
        ..p
    };
    let [w, e, n, s] = b;
    let front = s - unit * 0.65;
    let back = n + unit * 0.55;
    let stone = mix(p.wall, p.trim, 0.32);
    let frame = mix(p.trim, p.ink, 0.35);
    let mut part = |b: [f32; 6], bevel: f32, color: [f32; 3], kind: f32| {
        block(out, b, bevel * unit, color, kind, unit)
    };
    part(
        [
            w + unit * 0.38,
            e - unit * 0.38,
            0.,
            unit * 0.38,
            back,
            front + unit * 0.1,
        ],
        0.055,
        stone,
        3.,
    );
    part(
        [
            w + unit * 0.45,
            e - unit * 0.45,
            unit * 0.36,
            unit * 4.65,
            back,
            front,
        ],
        0.025,
        p.wall.map(|v| v * 0.90),
        3.,
    );
    // Horizontal plaster bands and substantial returns, rather than blank
    // monumental fins and a centered door that the source does not contain.
    for y in [0.50, 2.95, 3.18, 4.52] {
        part(
            [
                w + unit * 0.40,
                e - unit * 0.40,
                unit * y,
                unit * (y + 0.09),
                back - unit * 0.025,
                front + unit * 0.095,
            ],
            0.018,
            stone,
            3.,
        );
    }
    for x in [w + unit * 0.48, e - unit * 0.48] {
        part(
            [
                x - unit * 0.11,
                x + unit * 0.11,
                unit * 0.40,
                unit * 4.58,
                back,
                front + unit * 0.12,
            ],
            0.025,
            stone,
            3.,
        );
    }
    part(
        [
            w + unit * 0.18,
            e - unit * 0.18,
            unit * 4.63,
            unit * 4.88,
            n + unit * 0.2,
            s - unit * 0.3,
        ],
        0.035,
        mix(p.roof[0], p.ink, 0.18),
        0.,
    );
    drop(part);
    roof(
        out,
        [
            w + unit * 0.08,
            e - unit * 0.08,
            n + unit * 0.10,
            s - unit * 0.20,
        ],
        unit,
        p,
    );
    let mut pane = |x: f32, y: f32, width: f32, height: f32, glass: [f32; 3]| {
        block(
            out,
            [
                x - width * 0.5 - unit * 0.065,
                x + width * 0.5 + unit * 0.065,
                y - unit * 0.06,
                y + height + unit * 0.06,
                front + unit * 0.015,
                front + unit * 0.14,
            ],
            unit * 0.025,
            frame,
            3.,
            unit,
        );
        let light_first = out.positions.len();
        block(
            out,
            [
                x - width * 0.5,
                x + width * 0.5,
                y,
                y + height,
                front + unit * 0.14,
                front + unit * 0.16,
            ],
            0.,
            mix(glass, p.ink, 0.22),
            -1.,
            unit,
        );
        crate::city_lights::mark_window(out, light_first..out.positions.len(), light_first, true);
        for xx in [x - width * 0.5, x + width * 0.5] {
            block(
                out,
                [
                    xx - unit * 0.025,
                    xx + unit * 0.025,
                    y,
                    y + height,
                    front + unit * 0.16,
                    front + unit * 0.20,
                ],
                unit * 0.008,
                stone,
                3.,
                unit,
            );
        }
        block(
            out,
            [
                x - width * 0.5 - unit * 0.09,
                x + width * 0.5 + unit * 0.09,
                y - unit * 0.10,
                y - unit * 0.035,
                front + unit * 0.10,
                front + unit * 0.28,
            ],
            unit * 0.015,
            stone,
            3.,
            unit,
        );
    };
    for (i, (x, y)) in WINDOWS.into_iter().enumerate() {
        pane(
            w + unit * (x as f32 + 0.5),
            unit * if y == 4 { 3.60 } else { 1.08 },
            unit * 0.78,
            unit * if y == 4 { 0.75 } else { 1.35 },
            p.windows[i],
        );
    }
    drop(pane);
    // Closed pale ashlar gives the end and rear walls material scale. The
    // source's long front window bands remain clear of these shallow returns.
    for side in 0..3 {
        let (start, end) = if side < 2 {
            (back, front)
        } else {
            (w + unit * 0.48, e - unit * 0.48)
        };
        let count: i32 = if side < 2 { 8 } else { 14 };
        let width = (end - start) / count as f32;
        for row in 0..6 {
            for col in -1..count {
                let a = start + width * (col as f32 + if row % 2 == 0 { 0.5 } else { 0. });
                let lo = a.max(start) + unit * 0.012;
                let hi = (a + width).min(end) - unit * 0.012;
                if hi <= lo {
                    continue;
                }
                let bottom = unit * (0.58 + row as f32 * 0.64);
                let top = bottom + unit * 0.62;
                let bounds = match side {
                    0 => [w + unit * 0.38, w + unit * 0.46, bottom, top, lo, hi],
                    1 => [e - unit * 0.46, e - unit * 0.38, bottom, top, lo, hi],
                    _ => [lo, hi, bottom, top, back - unit * 0.07, back + unit * 0.01],
                };
                let gain = 0.95 + (row * 5 + col + 1).rem_euclid(4) as f32 * 0.012;
                block(
                    out,
                    bounds,
                    unit * 0.015,
                    p.wall.map(|v| v * gain),
                    2.,
                    unit,
                );
            }
        }
    }
    block(
        out,
        [
            door - unit * 0.95,
            door + unit * 0.95,
            unit * 0.34,
            unit * 2.88,
            front + unit * 0.08,
            front + unit * 0.29,
        ],
        unit * 0.05,
        stone,
        3.,
        unit,
    );
    block(
        out,
        [
            door - unit * 0.78,
            door + unit * 0.78,
            unit * 0.37,
            unit * 2.72,
            front + unit * 0.29,
            front + unit * 0.34,
        ],
        unit * 0.025,
        p.door,
        1.,
        unit,
    );
    let light_first = out.positions.len();
    block(
        out,
        [
            door - unit * 0.58,
            door + unit * 0.58,
            unit * 1.73,
            unit * 2.48,
            front + unit * 0.34,
            front + unit * 0.365,
        ],
        0.,
        p.glass,
        -1.,
        unit,
    );
    crate::city_lights::mark_window(out, light_first..out.positions.len(), light_first, true);
    block(
        out,
        [
            door + unit * 0.49,
            door + unit * 0.55,
            unit * 0.92,
            unit * 1.31,
            front + unit * 0.36,
            front + unit * 0.43,
        ],
        unit * 0.012,
        stone,
        3.,
        unit,
    );
    block(
        out,
        [
            door - unit * 1.05,
            door + unit * 1.05,
            0.,
            unit * 0.22,
            front + unit * 0.18,
            s - unit * 0.03,
        ],
        unit * 0.035,
        stone,
        3.,
        unit,
    );
    // Source sign content in real raised strokes; no substitute medal icons.
    let sign_x = w + unit * 6.;
    let sign_y = unit * 3.62;
    block(
        out,
        [
            sign_x - unit * 1.35,
            sign_x + unit * 1.35,
            sign_y - unit * 0.08,
            sign_y + unit * 0.78,
            front + unit * 0.08,
            front + unit * 0.20,
        ],
        unit * 0.045,
        stone,
        3.,
        unit,
    );
    for (letter, rows) in [
        [14u8, 17, 16, 23, 17, 17, 14],
        [17, 17, 10, 4, 4, 4, 4],
        [17, 27, 21, 21, 17, 17, 17],
    ]
    .into_iter()
    .enumerate()
    {
        for (row, bits) in rows.into_iter().enumerate() {
            for col in 0..5 {
                if bits & (1 << (4 - col)) == 0 {
                    continue;
                }
                let x = sign_x + unit * (-1.09 + letter as f32 * 0.78 + col as f32 * 0.115);
                let y = sign_y + unit * (0.62 - row as f32 * 0.085);
                block(
                    out,
                    [
                        x,
                        x + unit * 0.125,
                        y,
                        y + unit * 0.095,
                        front + unit * 0.20,
                        front + unit * 0.24,
                    ],
                    0.,
                    p.ink,
                    -1.,
                    unit,
                );
            }
        }
    }
    for position in &mut out.positions[first..] {
        *position = (Vec3::from_array(*position) + origin).to_array();
    }
}

#[cfg(test)]
pub(super) fn fixture() -> (
    Vec<VisualTile>,
    GridGeometry,
    BuildingPlacement,
    TerrainImageSamples,
) {
    let g = GridGeometry {
        width: 12,
        height: 9,
        tile_width: 8.,
        tile_height: 8.,
        origin_x: 0.,
        origin_z: 0.,
    };
    let p = BuildingPlacement {
        column: 0,
        row: 0,
        width: 12,
        height: 8,
        roof_rows: 4,
        ground_tile_index: KANTO_GROUND_TILE_INDEX,
    };
    let mut assets = Assets::<Image>::default();
    let mut images = TerrainImageSamples::default();
    let mut cells = Vec::new();
    for y in 0..9 {
        for x in 0..12 {
            let tile = if y < 8 {
                TILES[y][x]
            } else {
                KANTO_GROUND_TILE_INDEX
            };
            let block = if y < 4 {
                [0x0c, 0x0d, 0x0e][x / 4]
            } else if y < 8 {
                [0x10, 0x11, 0x12][x / 4]
            } else {
                1
            };
            let colors = if tile == 0x12 {
                [
                    [222, 255, 222, 255],
                    [41, 140, 255, 255],
                    [123, 255, 255, 255],
                    [57, 57, 57, 255],
                ]
            } else {
                [
                    [222, 255, 222, 255],
                    [160, 160, 160, 255],
                    [165, 123, 24, 255],
                    [57, 57, 57, 255],
                ]
            };
            let texture = assets.add(Image::default());
            let rgba = (0..64)
                .flat_map(|i| {
                    colors[if matches!(tile, 0x12 | 0x22 | 0x0a | 0x0b | 0x1b) {
                        ink(tile, i % 8, i / 8)
                    } else {
                        0
                    }]
                })
                .collect();
            images
                .pixels
                .insert(texture.id(), TileImageSample::Rgba(rgba));
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
    fn closed(out: &SurfaceMeshData) {
        let mut edges = std::collections::HashMap::new();
        let mut volume = 0.;
        let key = |p: [f32; 3]| p.map(|v| if v == 0. { 0 } else { v.to_bits() });
        for t in out.indices.chunks_exact(3) {
            let [a, b, c] = std::array::from_fn(|j| Vec3::from_array(out.positions[t[j] as usize]));
            assert!(
                (b - a)
                    .cross(c - a)
                    .dot(Vec3::from_array(out.normals[t[0] as usize]))
                    > 1e-8
            );
            volume += a.dot(b.cross(c)) / 6.;
            for j in 0..3 {
                let a = key(out.positions[t[j] as usize]);
                let b = key(out.positions[t[(j + 1) % 3] as usize]);
                let edge = if a < b { (a, b) } else { (b, a) };
                let entry = edges.entry(edge).or_insert((0, 0));
                entry.0 += 1;
                entry.1 += if a < b { 1 } else { -1 };
            }
        }
        assert!(volume > 0.);
        assert!(edges.values().all(|&(n, d)| n == 2 && d == 0));
    }
    #[test]
    fn ceramic_courses_are_closed_on_triangular_and_trapezoidal_slopes() {
        for top in [
            vec![
                Vec3::new(0., 1., 0.),
                Vec3::new(8., 1., 0.),
                Vec3::new(4., 3., 4.),
            ],
            vec![
                Vec3::new(0., 1., 0.),
                Vec3::new(8., 1., 0.),
                Vec3::new(6., 3., 4.),
                Vec3::new(2., 3., 4.),
            ],
        ] {
            let mut out = SurfaceMeshData::default();
            prism(&mut out, top, 0.1, [0.2, 0.4, 0.6], 8.);
            closed(&out);
        }
    }
    #[test]
    fn gym_keeps_plot_budget_offset_entrance_and_live_independent_windows() {
        let (cells, g, p, mut images) = fixture();
        let refs = cells.iter().collect::<Vec<_>>();
        let day = paint(&refs, &g, p, &images).unwrap();
        let mut a = SurfaceMeshData::default();
        append(&mut a, [0., 96., 0., 64.], 8., 72., day);
        let mut translated = SurfaceMeshData::default();
        append(&mut translated, [144., 240., -288., -224.], 8., 216., day);
        assert_eq!(
            translated.uvs, a.uvs,
            "viewport translation cannot change material domains"
        );
        assert_eq!(translated.colors, a.colors);
        assert_eq!(translated.indices, a.indices);
        for (world, local) in translated.positions.iter().zip(&a.positions) {
            assert_eq!(
                *world,
                (Vec3::from_array(*local) + Vec3::new(144., 0., -288.)).to_array()
            );
        }
        assert!(a.indices.len() / 3 < 10000);
        assert!(a.indices.len() / 3 > 4500);
        assert!(a.positions.iter().all(|p| p[0] >= 0.
            && p[0] <= 96.
            && p[2] >= 0.
            && p[2] <= 64.
            && p[1] >= 0.
            && p[1] < 52.));
        for t in a.indices.chunks_exact(3) {
            let [x, y, z] = std::array::from_fn(|i| Vec3::from_array(a.positions[t[i] as usize]));
            assert!(
                (y - x)
                    .cross(z - x)
                    .dot(Vec3::from_array(a.normals[t[0] as usize]))
                    > 1e-8
            );
        }
        let (x, y) = WINDOWS[0];
        let tile = &cells[y * g.width + x];
        if let TileImageSample::Rgba(data) = images.pixels.get_mut(&tile.texture.id()).unwrap() {
            for i in 0..64 {
                if ink(0x0a, i % 8, i / 8) == 0 {
                    data[i * 4..i * 4 + 4].copy_from_slice(&[255, 255, 132, 255]);
                }
            }
        }
        let night = paint(&refs, &g, p, &images).unwrap();
        assert_ne!(night.windows[0], day.windows[0]);
        assert_eq!(night.windows[1..], day.windows[1..]);
        let mut b = SurfaceMeshData::default();
        append(&mut b, [0., 96., 0., 64.], 8., 72., night);
        assert_eq!(a.positions, b.positions);
        assert_eq!(a.indices, b.indices);
        assert_ne!(a.colors, b.colors);
        images.pixels.remove(&tile.texture.id());
        assert!(paint(&refs, &g, p, &images).is_none());
    }
}
