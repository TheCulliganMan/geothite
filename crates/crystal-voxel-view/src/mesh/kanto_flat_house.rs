//! One shared, source-bound doorless hall. Native bays repeat with plot width.
use super::kanto_craft::{block, mix};
use super::*;

pub(super) fn matches(cells: &[&VisualTile], g: &GridGeometry, p: BuildingPlacement) -> bool {
    p.width >= 8
        && p.width <= 32
        && p.width % 4 == 0
        && p.height == 4
        && p.roof_rows == 2
        && p.column + p.width <= g.width
        && p.row + 4 <= g.height
        && (0..4).all(|y| {
            (0..p.width).all(|x| {
                let s = &cells[(p.row + y) * g.width + p.column + x].source;
                let (b, t) = super::kanto_house::compact_source(p.width, x, y, true);
                s.tileset_id.as_ref() == "kanto"
                    && s.metatile_id == b
                    && s.tile_index == t
                    && s.subtile_column as usize == x % 4
                    && s.subtile_row as usize == y
            })
        })
}
pub(super) struct Paint {
    roof: [[f32; 3]; 3],
    wall: [f32; 3],
    stone: [f32; 3],
    ink: [f32; 3],
    panes: Vec<(usize, [f32; 3], bool)>,
}
fn linear(p: &[u8]) -> [f32; 3] {
    std::array::from_fn(|c| {
        let s = p[c] as f32 / 255.;
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
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
    let sample = |x: usize, y: usize, px: usize, py: usize| {
        let rgba = tile_rgba(images, cells[(p.row + y) * g.width + p.column + x]).ok()?;
        (rgba.len() == 256 && rgba.chunks_exact(4).all(|p| p[3] == 255))
            .then(|| linear(&rgba[(py * 8 + px) * 4..]))
    };
    let roof = super::cabin_roof_palette(cells, g, p, images)?.map(|c| [c[0], c[1], c[2]]);
    let mut panes = Vec::new();
    for x in 0..p.width {
        let tile = cells[(p.row + 2) * g.width + p.column + x]
            .source
            .tile_index;
        if matches!(tile, 0x0a | 0x32) {
            panes.push((x, sample(x, 2, 3, 3)?, tile == 0x0a));
        }
    }
    Some(Paint {
        roof,
        wall: sample(1, 3, 3, 1)?,
        stone: sample(1, 3, 0, 0)?,
        ink: sample(panes.first()?.0, 2, 0, 0)?,
        panes,
    })
}
pub(super) fn append(out: &mut SurfaceMeshData, b: [f32; 4], unit: f32, p: Paint) {
    let p=Paint {roof:p.roof.map(super::kanto_craft::house_roof_paint),..p};
    let first = out.positions.len();
    let (width, depth) = (b[1] - b[0], b[3] - b[2]);
    let material_unit = unit * (width / unit / 14.).max(1.);
    let (w, e, n, s) = (
        unit * 0.35,
        width - unit * 0.35,
        unit * 0.46,
        depth - unit * 0.52,
    );
    let wall_luma = p.wall[0] * 0.2126 + p.wall[1] * 0.7152 + p.wall[2] * 0.0722;
    let wall = mix(
        p.wall,
        [wall_luma * 1.03, wall_luma, wall_luma * 0.94],
        0.48,
    );
    let frame = mix(p.stone, wall, 0.35);
    let stone = mix(p.stone, wall, 0.2);
    let roof = mix(p.roof[1], p.roof[2], 0.30);
    let mut part = |bounds, bevel, color, kind| {
        let a = out.positions.len();
        block(out, bounds, unit * bevel, color, kind, material_unit);
        a..out.positions.len()
    };
    part(
        [
            w - unit * 0.12,
            e + unit * 0.12,
            0.,
            unit * 0.46,
            n - unit * 0.12,
            s + unit * 0.12,
        ],
        0.035,
        stone,
        3.,
    );
    part([w, e, unit * 0.40, unit * 3.08, n, s], 0.025, wall, 1.);
    for y in [0.40, 0.72, 2.91] {
        part(
            [
                w - unit * 0.04,
                e + unit * 0.04,
                unit * y,
                unit * (y + 0.09),
                n - unit * 0.04,
                s + unit * 0.07,
            ],
            0.015,
            stone,
            3.,
        );
    }
    let mut glass = Vec::new();
    for (cell, color, lit) in p.panes {
        let x = (cell as f32 + 0.5) * unit;
        for side in [-1., 1.] {
            part(
                [
                    x + unit * (side * 0.445 - 0.045),
                    x + unit * (side * 0.445 + 0.045),
                    unit * 1.12,
                    unit * 2.32,
                    s,
                    s + unit * 0.22,
                ],
                0.015,
                frame,
                1.,
            );
            let y = if side < 0. { 1.12 } else { 2.23 };
            part(
                [
                    x - unit * 0.40,
                    x + unit * 0.40,
                    unit * y,
                    unit * (y + 0.09),
                    s,
                    s + unit * 0.22,
                ],
                0.012,
                frame,
                1.,
            );
        }
        let range = part(
            [
                x - unit * 0.40,
                x + unit * 0.40,
                unit * 1.21,
                unit * 2.23,
                s + unit * 0.10,
                s + unit * 0.125,
            ],
            0.,
            color,
            -1.,
        );
        glass.push((
            range,
            [x - unit * 0.40, x + unit * 0.40, unit * 1.21, unit * 2.23],
            lit,
        ));
        // A slender meeting rail gives the source pane a seated sash, rather
        // than stretching the native square into a floor-height glass door.
        part(
            [x-unit*0.40,x+unit*0.40,unit*1.70,unit*1.745,s+unit*0.125,s+unit*0.22],
            0.006,frame,1.,
        );
        part(
            [
                x - unit * 0.53,
                x + unit * 0.53,
                unit * 1.02,
                unit * 1.12,
                s - unit * 0.02,
                s + unit * 0.30,
            ],
            0.02,
            stone,
            3.,
        );
    }
    // A single-story wall shares the cottage eave height. Closed roof pans,
    // rolled shoulders, coping and vent remain distinct above that wall.
    part(
        [
            unit * 0.12,
            width - unit * 0.12,
            unit * 3.08,
            unit * 3.28,
            unit * 0.12,
            depth - unit * 0.12,
        ],
        0.025,
        mix(p.ink, roof, 0.25),
        0.,
    );
    let (north, south) = (unit * 0.28, depth - unit * 0.28);
    let step = (south - north) / 6.;
    for row in 0..6 {
        let z = north + row as f32 * step;
        part(
            [
                unit * 0.28,
                width - unit * 0.28,
                unit * 3.27,
                unit * 3.31,
                z,
                (z + step + unit * 0.015).min(south),
            ],
            0.009,
            roof,
            0.,
        );
        if row > 0 {
            part(
                [
                    unit * 0.32,
                    width - unit * 0.32,
                    unit * 3.29,
                    unit * 3.38,
                    z - unit * 0.04,
                    z + unit * 0.04,
                ],
                0.025,
                mix(roof, p.roof[2], 0.18),
                0.,
            );
        }
    }
    for z in [unit * 0.12, depth - unit * 0.24] {
        part(
            [
                unit * 0.12,
                width - unit * 0.12,
                unit * 3.25,
                unit * 3.66,
                z,
                z + unit * 0.12,
            ],
            0.025,
            p.roof[2],
            0.,
        );
    }
    for x in [unit * 0.12, width - unit * 0.24] {
        part(
            [
                x,
                x + unit * 0.12,
                unit * 3.25,
                unit * 3.66,
                unit * 0.12,
                depth - unit * 0.12,
            ],
            0.025,
            p.roof[2],
            0.,
        );
    }
    let (x, z) = (width * 0.75, depth * 0.36);
    part(
        [
            x - unit * 0.45,
            x + unit * 0.45,
            unit * 3.34,
            unit * 3.79,
            z - unit * 0.48,
            z + unit * 0.48,
        ],
        0.035,
        stone,
        3.,
    );
    for dz in [-0.28, 0., 0.28] {
        part(
            [
                x - unit * 0.34,
                x + unit * 0.34,
                unit * 3.79,
                unit * 3.812,
                z + unit * (dz - 0.03),
                z + unit * (dz + 0.03),
            ],
            0.004,
            mix(p.ink, stone, 0.35),
            3.,
        );
    }
    drop(part);
    for (range, bounds, lit) in glass {
        crate::scenery_finish::mark_source_glass(out, range, bounds, lit);
    }
    for v in &mut out.positions[first..] {
        v[0] += b[0];
        v[2] += b[2];
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
    let (mut tiles, g, p, mut images) = super::kanto_house::fixture(width);
    let palette = [
        [210, 240, 220, 255],
        [130, 170, 165, 255],
        [50, 110, 150, 255],
        [30, 35, 40, 255],
    ];
    for y in 0..4 {
        for x in 0..width {
            let tile = &mut tiles[y * g.width + x];
            let (block, art) = super::kanto_house::compact_source(width, x, y, true);
            tile.source.metatile_id = block;
            tile.source.tile_index = art;
            images.pixels.insert(
                tile.texture.id(),
                TileImageSample::Rgba(
                    (0..64)
                        .flat_map(|i| {
                            palette[if matches!(art, 0x17 | 0x1a | 0x0a | 0x32 | 0x22) {
                                super::kanto_craft::source_ink(art, i % 8, i / 8)
                            } else {
                                0
                            }]
                        })
                        .collect(),
                ),
            );
        }
    }
    (tiles, g, p, images)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn flat_halls_follow_native_pane_positions_pigment_and_lamps_at_every_width() {
        for width in [8, 12, 16, 32] {
            let (tiles, g, p, images) = fixture(width);
            let refs = tiles.iter().collect::<Vec<_>>();
            let paint = super::paint(&refs, &g, p, &images).unwrap();
            assert_eq!(paint.panes.len(), width / 2 - 1);
            assert_eq!(paint.panes.iter().filter(|p| p.2).count(), width / 4 + 1);
            let render = |origin: [f32; 2], images: &TerrainImageSamples| {
                let mut out = SurfaceMeshData::default();
                append(
                    &mut out,
                    [
                        origin[0],
                        origin[0] + width as f32 * 8.,
                        origin[1],
                        origin[1] + 32.,
                    ],
                    8.,
                    super::paint(&refs, &g, p, images).unwrap(),
                );
                out
            };
            let day = render([0., 0.], &images);
            let moved = render([64., 96.], &images);
            assert_eq!(day.uvs, moved.uvs);
            assert!(day.positions.iter().all(|p| p[0] >= 0.
                && p[0] <= width as f32 * 8.
                && p[2] >= 0.
                && p[2] <= 32.
                && p[1] >= 0.
                && p[1] <= 3.812 * 8. + 0.001));
            assert!(
                day.positions
                    .iter()
                    .any(|p| (p[1] - 3.812 * 8.).abs() < 0.001)
            );
            let mut front_panes = 0;
            for tri in day.indices.chunks_exact(3) {
                let ids=[tri[0] as usize,tri[1] as usize,tri[2] as usize];
                if day.normals[ids[0]][2]<0.99 || !ids.iter().all(|&i|
                    crate::scenery_finish::source_glass_coordinates(day.uvs[i]).is_some()) {continue;}
                let (lo,hi)=ids.iter().map(|&i|Vec3::from_array(day.positions[i])).fold(
                    (Vec3::splat(f32::INFINITY),Vec3::splat(f32::NEG_INFINITY)),
                    |(lo,hi),p|(lo.min(p),hi.max(p)));
                let aspect=(hi.y-lo.y)/(hi.x-lo.x);
                assert!((1.0..1.5).contains(&aspect),"source windows must not become tall doors: {aspect}");
                assert!(paint.panes.iter().any(|(cell,_,_)|
                    ((lo.x+hi.x)*0.5-(*cell as f32+0.5)*8.).abs()<0.001));
                front_panes+=1;
            }
            assert_eq!(front_panes,paint.panes.len()*2);
            assert!(day.indices.len() / 3 < 10000);
            assert!(
                day.normals
                    .iter()
                    .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 0.001)
            );
            assert!(
                day.uvs
                    .iter()
                    .all(|&uv| crate::scenery_finish::town_paint_kind(uv).is_some()
                        || crate::scenery_finish::source_glass_coordinates(uv).is_some())
            );
            let lights = crate::city_lights::collect(&day);
            assert_eq!(lights.len(), width / 4 + 1);
            assert!(lights.iter().all(|l| l.direction.z > 0.9));
            let mut dark = images.clone();
            for sample in dark.pixels.values_mut() {
                if let TileImageSample::Rgba(p) = sample {
                    for px in p.chunks_exact_mut(4) {
                        for c in &mut px[..3] {
                            *c /= 2;
                        }
                    }
                }
            }
            let night = render([0., 0.], &dark);
            assert_eq!(night.positions, day.positions);
            assert_eq!(night.normals, day.normals);
            assert_eq!(night.indices, day.indices);
            assert_eq!(night.uvs, day.uvs);
            assert_ne!(night.colors, day.colors);
        }
    }
    #[test]
    fn flat_halls_require_the_complete_native_drawing_and_opaque_samples() {
        let (tiles, g, p, images) = fixture(12);
        for y in 0..4 {
            for x in 0..12 {
                let mut bad = tiles.clone();
                bad[y * g.width + x].source.tile_index ^= 1;
                assert!(super::paint(&bad.iter().collect::<Vec<_>>(), &g, p, &images).is_none());
            }
        }
        let mut bad = tiles.clone();
        bad[0].source.tileset_id = "johto".into();
        assert!(super::paint(&bad.iter().collect::<Vec<_>>(), &g, p, &images).is_none());
        let cropped = GridGeometry { height: 3, ..g };
        assert!(!matches(&tiles.iter().collect::<Vec<_>>(), &cropped, p));
        let mut bad_images = images.clone();
        let TileImageSample::Rgba(pixels) = bad_images
            .pixels
            .get_mut(&tiles[2 * g.width + 2].texture.id())
            .unwrap()
        else {
            panic!()
        };
        pixels[3] = 128;
        assert!(super::paint(&tiles.iter().collect::<Vec<_>>(), &g, p, &bad_images).is_none());
    }
}
