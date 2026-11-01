//! The shared closed Kanto museum hall: source0c/0d*/0e +75/71*/76 +37/7d*/7e.
//! Source windows are continuous; this drawing contains no entrance.
use super::kanto_craft::{block, ceramic_tile, face, mix, source_ink};
use super::*;

const WEST: [[u16; 4]; 8] = [
    [0x15, 0x38, 0x12, 0x12],
    [0x15, 0x38, 0x12, 0x12],
    [0x15, 0x16, 0x17, 0x17],
    [0x25, 0x26, 0x22, 0x22],
    [0x0f, 0x32, 0x0a, 0x0a],
    [0x0f, 0x4b, 0x4b, 0x4b],
    [0x0f, 0x4b, 0x4b, 0x4b],
    [0x1d, 0x1a, 0x1a, 0x1a],
];
const BAY: [[u16; 4]; 8] = [
    [0x12; 4],
    [0x12; 4],
    [0x17; 4],
    [0x22; 4],
    [0x0a, 0x32, 0x0a, 0x0a],
    [0x4b; 4],
    [0x4b; 4],
    [0x1a; 4],
];
const EAST: [[u16; 4]; 8] = [
    [0x12, 0x12, 0x38, 0x19],
    [0x12, 0x12, 0x38, 0x19],
    [0x17, 0x17, 0x18, 0x19],
    [0x22, 0x22, 0x28, 0x29],
    [0x32, 0x0a, 0x32, 0x1f],
    [0x4b, 0x4b, 0x4b, 0x1f],
    [0x4b, 0x4b, 0x4b, 0x1f],
    [0x1a, 0x1a, 0x1a, 0x3c],
];
const ROOF_WEST: [[u16; 4]; 4] = [
    [5, 6, 0x53, 0x53],
    [0x15, 0x38, 0x12, 0x12],
    [0x15, 0x38, 0x12, 0x12],
    [0x15, 0x16, 0x17, 0x17],
];
const ROOF_BAY: [[u16; 4]; 4] = [[0x53; 4], [0x12; 4], [0x12; 4], [0x17; 4]];
const ROOF_EAST: [[u16; 4]; 4] = [
    [0x53, 0x53, 8, 9],
    [0x12, 0x12, 0x38, 0x19],
    [0x12, 0x12, 0x38, 0x19],
    [0x17, 0x17, 0x18, 0x19],
];
fn source(width: usize, x: usize, y: usize) -> (u16, u16) {
    if y < 4 {
        if x < 4 {
            (0x0c, ROOF_WEST[y][x])
        } else if x >= width - 4 {
            (0x0e, ROOF_EAST[y][x % 4])
        } else {
            (0x0d, ROOF_BAY[y][x % 4])
        }
    } else {
        let y = y - 4;
        if x < 4 {
            (if y < 4 { 0x75 } else { 0x37 }, WEST[y][x])
        } else if x >= width - 4 {
            (if y < 4 { 0x76 } else { 0x7e }, EAST[y][x % 4])
        } else {
            (if y < 4 { 0x71 } else { 0x7d }, BAY[y][x % 4])
        }
    }
}
pub(super) fn matches(cells: &[&VisualTile], g: &GridGeometry, p: BuildingPlacement) -> bool {
    p.width >= 8
        && p.width <= 20
        && p.width % 4 == 0
        && p.height == 12
        && p.roof_rows == 8
        && p.column + p.width <= g.width
        && p.row + 12 <= g.height
        && (0..12).all(|y| {
            (0..p.width).all(|x| {
                let (b, t) = source(p.width, x, y);
                cells
                    .get((p.row + y) * g.width + p.column + x)
                    .is_some_and(|c| {
                        let s = &c.source;
                        s.tileset_id.as_ref() == "kanto"
                            && s.metatile_id == b
                            && s.tile_index == t
                            && s.subtile_column as usize == x % 4
                            && s.subtile_row as usize == y % 4
                    })
            })
        })
}
#[derive(Clone)]
pub(super) struct Paint {
    roof: [[f32; 3]; 2],
    brick: [f32; 3],
    mortar: [f32; 3],
    stone: [f32; 3],
    ink: [f32; 3],
    windows: Vec<[f32; 3]>,
    lit_windows: Vec<bool>,
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
    if !matches(cells, g, p) {
        return None;
    }
    let sample = |x: usize, y: usize| -> Option<[Option<[u8; 4]>; 4]> {
        let c = cells[(p.row + y) * g.width + p.column + x];
        let rgba = tile_rgba(images, c).ok()?;
        if rgba.len() != 256 {
            return None;
        }
        let mut roles = [None; 4];
        for (i, pixel) in rgba.chunks_exact(4).enumerate() {
            let color: [u8; 4] = pixel.try_into().ok()?;
            if color[3] != 255 {
                return None;
            }
            let role = &mut roles[source_ink(c.source.tile_index, i % 8, i / 8)];
            if role.is_some_and(|old| old != color) {
                return None;
            }
            *role = Some(color);
        }
        Some(roles)
    };
    let roof = sample(2, 4)?;
    let wall = sample(2, 9)?;
    let stone = sample(2, 11)?;
    if roof[1] == roof[2] || wall[0] == wall[1] {
        return None;
    }
    let mut windows = Vec::new();
    let mut lit_windows = Vec::new();
    for x in 1..p.width - 1 {
        let pane = sample(x, 8)?;
        if pane[0] == pane[3] {
            return None;
        }
        windows.push(linear(pane[0]?));
        lit_windows.push(cells[(p.row+8)*g.width+p.column+x].source.tile_index==0x0a);
    }
    Some(Paint {
        roof: [linear(roof[1]?), linear(roof[2]?)],
        brick: linear(wall[1]?),
        mortar: linear(wall[0]?),
        stone: linear(stone[1]?),
        ink: linear(sample(1, 8)?[3]?),
        windows,
        lit_windows,
    })
}
fn prism(out: &mut SurfaceMeshData, top: Vec<Vec3>, thick: f32, color: [f32; 3], unit: f32) {
    let bottom = top.iter().map(|p| *p - Vec3::Y * thick).collect::<Vec<_>>();
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
fn roof(out: &mut SurfaceMeshData, width: f32, depth: f32, unit: f32, p: &Paint) {
    let mid = depth * 0.5;
    let inset = unit * 1.5;
    let low = unit * 5.02;
    let high = unit * 6.75;
    let v = |x, y, z| Vec3::new(x, y, z);
    let ceramic = mix(p.roof[0], p.roof[1], 0.32);
    let lum = Vec3::from_array(ceramic).dot(Vec3::new(0.2126, 0.7152, 0.0722));
    let ceramic = mix(ceramic, [lum; 3], 0.24);
    let planes = [
        vec![
            v(0., low, depth),
            v(width, low, depth),
            v(width - inset, high, mid),
            v(inset, high, mid),
        ],
        vec![
            v(width, low, 0.),
            v(0., low, 0.),
            v(inset, high, mid),
            v(width - inset, high, mid),
        ],
        vec![v(0., low, 0.), v(0., low, depth), v(inset, high, mid)],
        vec![
            v(width, low, depth),
            v(width, low, 0.),
            v(width - inset, high, mid),
        ],
    ];
    for (side, plane) in planes.into_iter().enumerate() {
        prism(
            out,
            plane.clone(),
            unit * 0.13,
            mix(p.roof[0], p.ink, 0.25),
            unit,
        );
        let rows=9;
        for row in 0..rows {
            let c=row as f32/rows as f32;
            let d=((row as f32+1.08)/rows as f32).min(0.995);
            let at = |u: f32, t: f32| {
                    let upper = if plane.len() == 4 {
                        plane[3]+(plane[2]-plane[3])*u
                    } else {
                        plane[2]
                    };
                    let lower=plane[0]+(plane[1]-plane[0])*u;
                    lower+(upper-lower)*t
                        + Vec3::Y * unit * (0.025+row as f32*0.003)
                };
            let span=(at(1.,(c+d)*0.5)-at(0.,(c+d)*0.5)).length();
            let cols=(span/(unit*0.62)).ceil().max(1.) as usize;
            for col in 0..=cols {
                let stagger=if row%2==0 {0.} else {0.5};
                let a=((col as f32-stagger)/cols as f32).max(0.)+0.009/cols as f32;
                let b=((col as f32+1.-stagger)/cols as f32).min(1.)-0.009/cols as f32;
                if b<=a {continue;}
                let gain = 0.91 + ((row * 3 + col * 7 + side) % 5) as f32 * 0.016;
                ceramic_tile(out,[[at(a,c),at(b,c)],[at(a,d),at(b,d)]],unit,
                    unit*0.045,ceramic.map(|v|v*gain));
            }
        }
    }
    block(
        out,
        [
            inset - unit * 0.08,
            width - inset + unit * 0.08,
            high,
            high + unit * 0.06,
            mid - unit * 0.10,
            mid + unit * 0.10,
        ],
        unit * 0.025,
        ceramic,
        0.,
        unit,
    );
    let count=((width-2.*inset)/(unit*0.70)).ceil().max(1.) as usize;
    for i in 0..count {
        let a=inset+(width-2.*inset)*i as f32/count as f32;
        let b=inset+(width-2.*inset)*(i+1) as f32/count as f32-unit*0.012;
        ceramic_tile(out,[[v(a,high+unit*0.06,mid-unit*0.12),v(a,high+unit*0.06,mid+unit*0.12)],
            [v(b,high+unit*0.06,mid-unit*0.12),v(b,high+unit*0.06,mid+unit*0.12)]],
            unit,unit*0.07,ceramic);
    }
}
fn masonry(out: &mut SurfaceMeshData, width: f32, depth: f32, unit: f32, p: &Paint) {
    // Eight quiet running-bond courses below the continuous clerestory.
    for wall in 0..4 {
        let long = if wall < 2 {
            width - unit * 0.9
        } else {
            depth - unit * 0.9
        };
        for row in 0..8 {
            let y = unit * (0.62 + row as f32 * 0.30);
            let count = (long / (unit * 0.94)).ceil() as usize;
            for col in 0..=count {
                let a = ((col as f32 - 0.5 * (row % 2) as f32) * unit * 0.94).clamp(0., long);
                let b = ((col as f32 + 1. - 0.5 * (row % 2) as f32) * unit * 0.94).clamp(0., long);
                if b - a < unit * 0.08 {
                    continue;
                }
                let a = a + unit * 0.45 + 0.012 * unit;
                let b = b + unit * 0.45 - 0.012 * unit;
                let bounds = match wall {
                    0 => [
                        a,
                        b,
                        y,
                        y + unit * 0.272,
                        depth - unit * 0.46,
                        depth - unit * 0.32,
                    ],
                    1 => [a, b, y, y + unit * 0.272, unit * 0.32, unit * 0.46],
                    2 => [unit * 0.32, unit * 0.46, y, y + unit * 0.272, a, b],
                    _ => [
                        width - unit * 0.46,
                        width - unit * 0.32,
                        y,
                        y + unit * 0.272,
                        a,
                        b,
                    ],
                };
                let gain = 0.94 + ((row * 3 + col * 7 + wall) % 5) as f32 * 0.022;
                block(
                    out,
                    bounds,
                    unit * 0.012,
                    p.brick.map(|v| v * gain),
                    2.,
                    unit,
                );
            }
        }
    }
}
pub(super) fn append(out: &mut SurfaceMeshData, b: [f32; 4], unit: f32, p: Paint) {
    let start = out.positions.len();
    let width = b[1] - b[0];
    let depth = b[3] - b[2];
    let u = unit;
    let w = u * 0.45;
    let e = width - w;
    let n = u * 0.45;
    let s = depth - n;
    let brick = mix(p.mortar, p.brick, 0.46);
    let stone = mix(p.stone, [0.36, 0.36, 0.32], 0.30);
    block(out, [w, e, 0., u * 4.88, n, s], u * 0.025, brick, 2., u);
    masonry(out, width, depth, u, &p);
    // Closed plinth and belt courses, including the unseen rear and returns.
    for (lo, hi, protrude) in [
        (0., 0.23, 0.09),
        (0.25, 0.53, 0.07),
        (2.99, 3.10, 0.10),
        (4.31, 4.46, 0.08),
        (4.73, 4.95, 0.13),
    ] {
        block(
            out,
            [
                w - u * protrude,
                e + u * protrude,
                u * lo,
                u * hi,
                n - u * protrude,
                s + u * protrude,
            ],
            u * 0.025,
            stone,
            1.,
            u,
        );
    }
    let frame = mix(stone, p.mortar, 0.20);
    for (index, color) in p.windows.iter().enumerate() {
        let x = (index as f32 + 1.5) * u;
        let half = u * 0.43;
        let front = depth - u * 0.30;
        // All glazing lies in a shallow framed recess above solid brickwork.
        block(
            out,
            [
                x - half - u * 0.055,
                x + half + u * 0.055,
                u * 3.12,
                u * 4.29,
                front - u * 0.13,
                front,
            ],
            u * 0.018,
            p.ink,
            1.,
            u,
        );
        let first_glass=out.positions.len();
        block(
            out,
            [
                x - half,
                x + half,
                u * 3.20,
                u * 4.21,
                front,
                front + u * 0.008,
            ],
            0.,
            *color,
            -1.,
            u,
        );
        crate::scenery_finish::mark_source_glass(out,first_glass..out.positions.len(),
            [x-half,x+half,u*3.20,u*4.21],p.lit_windows[index]);
        for xx in [x - half - u * 0.028, x + half + u * 0.028] {
            block(
                out,
                [
                    xx - u * 0.025,
                    xx + u * 0.025,
                    u * 3.13,
                    u * 4.28,
                    front + u * 0.01,
                    front + u * 0.065,
                ],
                u * 0.008,
                frame,
                1.,
                u,
            );
        }
        for y in [3.15, 4.25] {
            block(
                out,
                [
                    x - half - u * 0.05,
                    x + half + u * 0.05,
                    u * (y - 0.025),
                    u * (y + 0.025),
                    front + u * 0.01,
                    front + u * 0.065,
                ],
                u * 0.008,
                frame,
                1.,
                u,
            );
        }
        block(
            out,
            [
                x - half - u * 0.09,
                x + half + u * 0.09,
                u * 3.06,
                u * 3.13,
                front - u * 0.06,
                front + u * 0.14,
            ],
            u * 0.018,
            stone,
            1.,
            u,
        );
    }
    // Recessed frieze, continuous fascia and physical soffit support the roof.
    block(
        out,
        [
            w - u * 0.12,
            e + u * 0.12,
            u * 4.92,
            u * 5.005,
            n - u * 0.12,
            s + u * 0.12,
        ],
        u * 0.02,
        mix(stone, p.ink, 0.20),
        1.,
        u,
    );
    roof(out, width, depth, u, &p);
    // Deep museum hips and wide halls exceed the town material's bounded
    // local UV domain. Fit only those paint coordinates before placement;
    // source-colored panes retain their separate normalized glass domain.
    let paint_coordinates=|uv:[f32;2]| {
        let kind=((uv[1]-crate::scenery_finish::TOWN_PAINT_BASE)/32.).round();
        let v=uv[1]-(crate::scenery_finish::TOWN_PAINT_BASE+kind*32.);
        ((0. ..=3.).contains(&kind) && v.abs()<16.).then_some((kind,v))
    };
    let scale=out.uvs[start..].iter().filter_map(|&uv|
        paint_coordinates(uv).map(|(_,v)|(uv[0].abs()/15.).max(v.abs()/7.5)))
        .fold(1.,f32::max);
    for uv in &mut out.uvs[start..] {
        if let Some((kind,v))=paint_coordinates(*uv) {
            *uv=[uv[0]/scale,crate::scenery_finish::TOWN_PAINT_BASE+kind*32.+v/scale];
        }
    }
    let origin = Vec3::new(b[0], 0., b[2]);
    for v in &mut out.positions[start..] {
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
        height: 13,
        tile_width: 8.,
        tile_height: 8.,
        origin_x: 0.,
        origin_z: 0.,
    };
    let p = BuildingPlacement {
        column: 0,
        row: 0,
        width,
        height: 12,
        roof_rows: 8,
        ground_tile_index: 0x2c,
    };
    let mut assets = Assets::<Image>::default();
    let mut images = TerrainImageSamples::default();
    let mut cells = Vec::new();
    for y in 0..13 {
        for x in 0..width {
            let (b, t) = if y == 12 {
                (0x0a, 0x2c)
            } else {
                source(width, x, y)
            };
            let colors = match t {
                0x12 => [
                    [222, 255, 222, 255],
                    [74, 185, 235, 255],
                    [51, 130, 180, 255],
                    [57, 57, 57, 255],
                ],
                0x4b => [
                    [222, 245, 222, 255],
                    [181, 140, 68, 255],
                    [105, 110, 100, 255],
                    [57, 57, 57, 255],
                ],
                0x1a => [
                    [222, 245, 222, 255],
                    [160, 160, 150, 255],
                    [105, 110, 100, 255],
                    [57, 57, 57, 255],
                ],
                _ => [
                    [200, 240, 225, 255],
                    [210, 190, 130, 255],
                    [142, 105, 44, 255],
                    [57, 57, 57, 255],
                ],
            };
            let texture = assets.add(Image::default());
            images.pixels.insert(
                texture.id(),
                TileImageSample::Rgba(
                    (0..64)
                        .flat_map(|i| {
                            colors[if matches!(t, 0x12 | 0x4b | 0x1a | 0x0a | 0x32) {
                                source_ink(t, i % 8, i / 8)
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
                    metatile_id: b,
                    subtile_column: (x % 4) as u8,
                    subtile_row: (y % 4) as u8,
                    tile_index: t,
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
    fn museum_glass_retains_each_native_lamp_without_lighting_dark_panes() {
        for width in [8,16,20] {
            let (tiles,g,p,images)=fixture(width);
            let cells=tiles.iter().collect::<Vec<_>>();
            let paint=paint(&cells,&g,p,&images).unwrap();
            let expected=(1..width-1).filter(|&x|cells[8*width+x].source.tile_index==0x0a).count();
            let mut day=SurfaceMeshData::default();
            append(&mut day,[0.,width as f32*8.,0.,96.],8.,paint.clone());
            assert_eq!(crate::city_lights::collect(&day).len(),expected,
                "exact native lamp count, not every pane");
            let tagged=day.uvs.iter().filter_map(|&uv|crate::scenery_finish::source_glass_coordinates(uv))
                .collect::<Vec<_>>();
            assert!(tagged.iter().any(|(_,lit)|*lit));
            assert!(tagged.iter().any(|(_,lit)|!*lit));
            let mut changed=paint.clone();changed.windows.fill([0.01,0.02,0.03]);
            let mut dim=SurfaceMeshData::default();
            append(&mut dim,[0.,width as f32*8.,0.,96.],8.,changed.clone());
            assert_eq!(crate::city_lights::collect(&dim).len(),expected,
                "day/dusk pigment cannot remove an authored lamp");
            assert_eq!(dim.positions,day.positions);assert_eq!(dim.indices,day.indices);
            assert_eq!(dim.uvs,day.uvs);assert_ne!(dim.colors,day.colors);
            changed.lit_windows.fill(false);
            let mut dark=SurfaceMeshData::default();
            append(&mut dark,[0.,width as f32*8.,0.,96.],8.,changed);
            assert!(crate::city_lights::collect(&dark).is_empty());
            assert_eq!(dark.positions,day.positions);assert_eq!(dark.indices,day.indices);
        }
    }

    #[test]
    fn museum_preserves_each_source_pane_at_night_and_rejects_changed_paint() {
        let (tiles, g, p, mut images) = fixture(16);
        let cells = tiles.iter().collect::<Vec<_>>();
        let original = paint(&cells, &g, p, &images).unwrap();
        assert_eq!(original.windows.len(), 14);
        let c = cells[8 * 16 + 3];
        let mut pixels = tile_rgba(&images, c).unwrap().to_vec();
        for (i, color) in pixels.chunks_exact_mut(4).enumerate() {
            if source_ink(c.source.tile_index, i % 8, i / 8) == 0 {
                color.copy_from_slice(&[255, 255, 132, 255]);
            }
        }
        images
            .pixels
            .insert(c.texture.id(), TileImageSample::Rgba(pixels));
        let night = paint(&cells, &g, p, &images).unwrap();
        assert_ne!(night.windows[2], original.windows[2]);
        for i in (0..14).filter(|&i| i != 2) {
            assert_eq!(night.windows[i], original.windows[i]);
        }
        assert_eq!(night.roof, original.roof);
        assert_eq!(night.brick, original.brick);
        let valid_window = tile_rgba(&images, c).unwrap().to_vec();
        // Arbitrary art cannot inherit a guessed
        // window material merely because the tile number still matches.
        images.pixels.insert(
            c.texture.id(),
            TileImageSample::Rgba((0..64).flat_map(|i| [i as u8, 0, 255, 255]).collect()),
        );
        assert!(paint(&cells, &g, p, &images).is_none());
        images
            .pixels
            .insert(c.texture.id(), TileImageSample::Rgba(valid_window));
        images.pixels.remove(&cells[4 * 16 + 2].texture.id());
        assert!(paint(&cells, &g, p, &images).is_none());
    }
    #[test]
    fn ceramic_courses_are_closed_curved_volumes_even_at_narrow_hips() {
        for narrow in [1.,0.35,0.04] {
            let mut mesh=SurfaceMeshData::default();
            ceramic_tile(&mut mesh,[[Vec3::ZERO,Vec3::X],
                [Vec3::new(0.5-narrow*0.5,0.4,0.8),Vec3::new(0.5+narrow*0.5,0.4,0.8)]],
                1.,0.045,[0.3,0.6,0.8]);
            assert_eq!(mesh.indices.len()/3,20);
            let mut edges=std::collections::BTreeMap::new();let mut volume=0.;
            for tri in mesh.indices.chunks_exact(3) {
                let p=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
                assert!((p[1]-p[0]).cross(p[2]-p[0]).length()>0.000001);
                volume+=p[0].dot(p[1].cross(p[2]))/6.;
                for i in 0..3 {let mut e=[p[i].to_array().map(f32::to_bits),p[(i+1)%3].to_array().map(f32::to_bits)];
                    e.sort();*edges.entry(e).or_insert(0)+=1;}
            }
            assert!(volume>0.);assert!(edges.values().all(|&n|n==2));
            let shoulders=mesh.normals.iter().filter(|n|n[1]>0.5)
                .map(|n|n.map(|v|(v*1000.).round() as i32)).collect::<std::collections::BTreeSet<_>>();
            assert!(shoulders.len()>=3,"curved shoulders must be real geometry");
        }
    }

    #[test]
    fn museum_paint_keeps_native_material_roles_across_wide_and_deep_roofs() {
        for width in [8,12,16,20] {
            let (tiles,g,p,images)=fixture(width);
            let paint=paint(&tiles.iter().collect::<Vec<_>>(),&g,p,&images).unwrap();
            let mut mesh=SurfaceMeshData::default();
            append(&mut mesh,[0.,width as f32*8.,0.,96.],8.,paint);
            for &uv in &mesh.uvs {
                assert!(crate::scenery_finish::town_paint_kind(uv).is_some()
                    ||crate::scenery_finish::source_glass_coordinates(uv).is_some()
                    ||uv==crate::scenery_finish::SOURCE_PAINT_UV,
                    "every museum face must retain its source material, including the front hip: {uv:?}");
            }
        }
    }

    #[test]
    fn museum_geometry_is_closed_bounded_and_translation_stable() {
        let (tiles, g, p, images) = fixture(16);
        let cells = tiles.iter().collect::<Vec<_>>();
        let paint = paint(&cells, &g, p, &images).unwrap();
        let mut a = SurfaceMeshData::default();
        let mut b = SurfaceMeshData::default();
        append(&mut a, [0., 128., 0., 96.], 8., paint.clone());
        append(&mut b, [-216., -88., 408., 504.], 8., paint);
        assert_eq!(a.indices, b.indices);
        assert_eq!(a.colors, b.colors);
        assert_eq!(a.uvs, b.uvs);
        assert!(
            a.indices.len() / 3 > 4000 && a.indices.len() / 3 < 24000,
            "{} triangles",
            a.indices.len() / 3
        );
        for (a, b) in a.positions.iter().zip(&b.positions) {
            assert!(a.iter().all(|v| v.is_finite()));
            assert!(
                a[0] >= 0.
                    && a[0] <= 128.
                    && a[2] >= 0.
                    && a[2] <= 96.
                    && a[1] >= 0.
                    && a[1] < 8. * 7.,
                "museum vertex outside native plot: {a:?}"
            );
            assert!(
                (a[0] - 216. - b[0]).abs() < 0.0001
                    && (a[1] - b[1]).abs() < 0.0001
                    && (a[2] + 408. - b[2]).abs() < 0.0001
            );
        }
        // A closed course has top, underside and every edge, all wound outward.
        let mut tile = SurfaceMeshData::default();
        prism(
            &mut tile,
            vec![
                Vec3::ZERO,
                Vec3::X,
                Vec3::new(1., 1., 1.),
                Vec3::new(0., 1., 1.),
            ],
            0.1,
            [0.3; 3],
            1.,
        );
        assert_eq!(tile.indices.len() / 3, 12);
        assert!(tile.normals.iter().any(|n| n[1] < -0.5));
    }
}
