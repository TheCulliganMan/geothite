//! Complete native Slowpoke Well, with an open south entrance and recessed steps.
//! Source coordinates, collision, warps and standing support remain unchanged.
use super::*;

const ART: [[u16; 8]; 8] = [
    [0x29, 0x2a, 0x29, 0x2a, 0x29, 0x2a, 5, 5],
    [0x4b, 0x48, 0x4b, 0x48, 0x47, 0x48, 5, 5],
    [0x29, 0x2a, 0x55, 0x57, 0x29, 0x2a, 5, 5],
    [0x4b, 0x48, 0x35, 0x35, 0x47, 0x48, 5, 5],
    [0x3b, 0x4c, 0x5b, 0x5b, 0x4c, 0x3d, 5, 5],
    [0x4b, 0x4c, 0x5b, 0x5b, 0x4c, 0x4d, 5, 5],
    [5; 8],
    [5; 8],
];

fn complete(
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    claimed: &[bool],
    x: usize,
    y: usize,
) -> bool {
    x + 8 <= g.width
        && y + 8 <= g.height
        && (0..8).all(|dy| {
            (0..8).all(|dx| {
                let i = (y + dy) * g.width + x + dx;
                let s = &cells[i].source;
                s.tileset_id.as_ref() == "johto_modern"
                    && s.metatile_id == [[0x5b, 0x5c], [0x5d, 0x5e]][dy / 4][dx / 4]
                    && usize::from(s.subtile_column) == dx % 4
                    && usize::from(s.subtile_row) == dy % 4
                    && s.tile_index == ART[dy][dx]
                    && matches!(
                        shapes[i],
                        CellShape::Flat | CellShape::PlaneAt { height: 0. }
                    )
                    && (dx >= 6 || dy >= 6 || !claimed[i])
            })
        })
}

fn pigment(images: &TerrainImageSamples, cell: &VisualTile) -> Option<[[f32; 4]; 2]> {
    let rgba = tile_rgba(images, cell).ok()?;
    let mut colors = Vec::<[u8; 4]>::new();
    for pixel in rgba.chunks_exact(4) {
        let c: [u8; 4] = pixel.try_into().ok()?;
        if c[3] != 255 {
            return None;
        }
        if !colors.contains(&c) {
            colors.push(c);
        }
    }
    if rgba.len() != 256 || colors.len() < 3 {
        return None;
    }
    colors.sort_by_key(|c| 3 * u32::from(c[0]) + 6 * u32::from(c[1]) + u32::from(c[2]));
    let linear = |c: [u8; 4]| {
        let channel = |v: u8| {
            let s = f32::from(v) / 255.;
            if s <= 0.04045 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        [channel(c[0]), channel(c[1]), channel(c[2]), 1.]
    };
    Some([
        linear(colors[colors.len() / 2]),
        linear(colors[colors.len() - 1]),
    ])
}

fn stair_color(images: &TerrainImageSamples, cell: &VisualTile) -> Option<[f32; 4]> {
    let rgba = tile_rgba(images, cell).ok()?;
    if rgba.len() != 256 || rgba.chunks_exact(4).any(|c| c[3] != 255) {
        return None;
    }
    let c = rgba
        .chunks_exact(4)
        .max_by_key(|c| 3 * u32::from(c[0]) + 6 * u32::from(c[1]) + u32::from(c[2]))?;
    let channel = |v: u8| {
        let s = f32::from(v) / 255.;
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    };
    Some([channel(c[0]), channel(c[1]), channel(c[2]), 1.])
}

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    claimed: &mut [bool],
    images: &TerrainImageSamples,
) {
    if cells.len() != g.width * g.height
        || shapes.len() != cells.len()
        || claimed.len() != cells.len()
    {
        return;
    }
    for y in 0..g.height.saturating_sub(7) {
        for x in 0..g.width.saturating_sub(7) {
            let s = &cells[y * g.width + x].source;
            if s.metatile_id != 0x5b
                || s.subtile_column != 0
                || s.subtile_row != 0
                || !complete(cells, shapes, g, claimed, x, y)
            {
                continue;
            }
            let Some(paint) = pigment(images, cells[y * g.width + x]) else {
                continue;
            };
            let Some(stairs) = stair_color(images, cells[(y + 4) * g.width + x + 2]) else {
                continue;
            };
            let ground = y * g.width + x + 6;
            for dy in 0..6 {
                for dx in 0..6 {
                    let i = (y + dy) * g.width + x + dx;
                    claimed[i] = true;
                    mesh.authored_cells[i] = Some("world-exterior/slowpoke_well");
                    // The entrance shaft and steps remain open below ground.
                    if (2..4).contains(&dx) && dy >= 2 {
                        continue;
                    }
                    let (w, e, n, s) = g.bounds(x + dx, y + dy);
                    let (u0, u1, v0, v1) = g.uv(ground % g.width, ground / g.width);
                    append_quad_colors(
                        &mut mesh.textured,
                        [[w, 0., n], [w, 0., s], [e, 0., s], [e, 0., n]],
                        [0., 1., 0.],
                        [[u0, v0], [u0, v1], [u1, v1], [u1, v0]],
                        [[1.; 4]; 4],
                    );
                }
            }
            let (west, _, north, _) = g.bounds(x, y);
            geometry(
                &mut mesh.solid,
                west,
                north,
                g.tile_width,
                g.tile_height,
                paint,
                stairs,
            );
        }
    }
}

fn geometry(
    out: &mut SurfaceMeshData,
    west: f32,
    north: f32,
    tw: f32,
    th: f32,
    paint: [[f32; 4]; 2],
    stairs: [f32; 4],
) {
    let first = out.positions.len();
    let stone = paint[0];
    let height = th * 1.35;
    // The source is a square masonry enclosure with a two-cell south entry.
    // Recessed joints separate the individual blocks from the continuous shaft.
    let mut box_at = |lo: [f32; 3], hi: [f32; 3], color: [f32; 4]| {
        let [x0, y0, z0] = lo;
        let [x1, y1, z1] = hi;
        for (p, n) in [
            (
                [[x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1]],
                [0., 0., 1.],
            ),
            (
                [[x1, y0, z0], [x0, y0, z0], [x0, y1, z0], [x1, y1, z0]],
                [0., 0., -1.],
            ),
            (
                [[x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0]],
                [-1., 0., 0.],
            ),
            (
                [[x1, y0, z1], [x1, y0, z0], [x1, y1, z0], [x1, y1, z1]],
                [1., 0., 0.],
            ),
            (
                [[x0, y1, z0], [x0, y1, z1], [x1, y1, z1], [x1, y1, z0]],
                [0., 1., 0.],
            ),
            (
                [[x0, y0, z1], [x0, y0, z0], [x1, y0, z0], [x1, y0, z1]],
                [0., -1., 0.],
            ),
        ] {
            let shade = 0.62
                + 0.38
                    * Vec3::from_array(n)
                        .dot(Vec3::new(-0.35, 0.85, 0.4).normalize())
                        .max(0.);
            append_quad_colors(
                out,
                p,
                n,
                [crate::scenery_finish::SOURCE_PAINT_UV; 4],
                [std::array::from_fn(|i| if i == 3 { 1. } else { color[i] * shade }); 4],
            );
        }
    };
    let world = |x: f32, y: f32, z: f32| [west + x * tw, y, north + z * th];
    let mortar = std::array::from_fn(|i| if i == 3 { 1. } else { stone[i] * 0.48 });
    // The backing runs below ground; the entry and shaft have no grass lid.
    for (x0, x1, z0, z1) in [
        (0.35, 5.65, 0.35, 1.95),
        (0.35, 1.95, 1.95, 5.65),
        (4.05, 5.65, 1.95, 5.65),
    ] {
        box_at(world(x0, -1.6 * th, z0), world(x1, 0., z1), mortar);
        box_at(
            world(x0 + 0.05, 0., z0 + if z0 > 1. { -0.05 } else { 0.05 }),
            world(x1 - 0.05, height - 0.025 * th, z1 - 0.05),
            mortar,
        );
    }
    for course in 0..3 {
        let low = height * course as f32 / 3. + 0.015 * th;
        let high = height * (course + 1) as f32 / 3.;
        for (wall, count) in [(0, 6), (1, 4), (2, 4)] {
            for block in 0..count {
                let gain = 0.88 + 0.06 * ((block + course + wall) % 3) as f32;
                let color = std::array::from_fn(|i| if i == 3 { 1. } else { stone[i] * gain });
                let t0 = block as f32 / count as f32;
                let t1 = (block + 1) as f32 / count as f32;
                let (x0, x1, z0, z1) = match wall {
                    0 => (0.35 + 5.3 * t0, 0.35 + 5.3 * t1, 0.35, 1.95),
                    1 => (0.35, 1.95, 1.95 + 3.7 * t0, 1.95 + 3.7 * t1),
                    _ => (4.05, 5.65, 1.95 + 3.7 * t0, 1.95 + 3.7 * t1),
                };
                box_at(
                    world(x0 + 0.018, low, z0 + 0.018),
                    world(x1 - 0.018, high, z1 - 0.018),
                    color,
                );
                // Pale chipped top edges keep the source's cream stone accents.
                if course == 2 {
                    let cap = std::array::from_fn(|i| {
                        if i == 3 {
                            1.
                        } else {
                            color[i] * 0.68 + paint[1][i] * 0.32
                        }
                    });
                    box_at(
                        world(x0 + 0.045, high, z0 + 0.045),
                        world(x1 - 0.045, high + 0.05 * th, z1 - 0.045),
                        cap,
                    );
                }
            }
        }
    }
    for step in 0..4 {
        let south = 6. - step as f32 * 0.5;
        let front = -(step as f32) * th * 0.16;
        box_at(
            world(2., front - 0.16 * th, south - 0.5),
            world(4., front, south),
            stairs,
        );
    }
    box_at(
        world(1.9, -1.66 * th, 1.9),
        world(4.1, -1.6 * th, 6.),
        [0.015, 0.012, 0.008, 1.],
    );
    out.cutaway_ranges.push(first..out.positions.len());
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::Vec2;
    #[test]
    fn complete_well_claims_only_its_art_and_preserves_warp_support() {
        let g = GridGeometry {
            width: 8,
            height: 8,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut assets = Assets::<Image>::default();
        let texture = assets.add(Image::default());
        let mut images = TerrainImageSamples::default();
        images.pixels.insert(
            texture.id(),
            TileImageSample::Rgba(
                (0..64)
                    .flat_map(|i| {
                        [
                            [0, 0, 0, 255],
                            [155, 100, 42, 255],
                            [224, 196, 125, 255],
                            [255, 255, 240, 255],
                        ][i % 4]
                    })
                    .collect(),
            ),
        );
        let tiles = (0..64)
            .map(|i| {
                let (x, y) = (i % 8, i / 8);
                VisualTile {
                    column: x as u32,
                    row: y as u32,
                    source: VisualTileSource {
                        tileset_id: "johto_modern".into(),
                        metatile_id: [[0x5b, 0x5c], [0x5d, 0x5e]][y / 4][x / 4],
                        subtile_column: (x % 4) as u8,
                        subtile_row: (y % 4) as u8,
                        tile_index: ART[y][x],
                    },
                    texture: texture.clone(),
                    animation_frames: None,
                    priority: false,
                }
            })
            .collect::<Vec<_>>();
        let shapes = vec![CellShape::Flat; 64];
        let mut mesh = TerrainMeshData {
            footing_heights: vec![0.; 64],
            authored_cells: vec![None; 64],
            ..Default::default()
        };
        let mut claimed = vec![false; 64];
        append(
            &mut mesh,
            &tiles.iter().collect::<Vec<_>>(),
            &shapes,
            &g,
            &mut claimed,
            &images,
        );
        assert_eq!(claimed.iter().filter(|&&v| v).count(), 36);
        assert!(mesh.footing_heights.iter().all(|&v| v == 0.));
        assert!(!mesh.solid.indices.is_empty());
        assert!(
            mesh.solid
                .uvs
                .iter()
                .all(|uv| *uv == crate::scenery_finish::SOURCE_PAINT_UV)
        );
        for fault in 0..4 {
            let mut changed = tiles.clone();
            match fault {
                0 => changed[17].source.tile_index = 0xffff,
                1 => changed[63].source.metatile_id = 0x5b,
                2 => changed[0].source.tileset_id = "johto".into(),
                _ => changed[8].source.subtile_column = 3,
            }
            assert!(!complete(
                &changed.iter().collect::<Vec<_>>(),
                &shapes,
                &g,
                &[false; 64],
                0,
                0
            ));
        }
        let mut claimed = [false; 64];
        claimed[35] = true;
        assert!(!complete(
            &tiles.iter().collect::<Vec<_>>(),
            &shapes,
            &g,
            &claimed,
            0,
            0
        ));
        assert!(!complete(
            &tiles.iter().collect::<Vec<_>>(),
            &shapes,
            &g,
            &[false; 64],
            1,
            0
        ));
    }
    #[test]
    fn visible_stones_stand_outside_the_recessed_backing() {
        let mut mesh = SurfaceMeshData::default();
        geometry(
            &mut mesh,
            0.,
            0.,
            8.,
            8.,
            [[0.3, 0.2, 0.08, 1.], [0.7, 0.6, 0.3, 1.]],
            [0.9, 0.9, 0.8, 1.],
        );
        // Cast toward the south end of the left wall, below its first joint.
        // A full-width backing previously hid all of the stone courses.
        let point = Vec2::new(8., 1.);
        let front = mesh
            .indices
            .chunks_exact(3)
            .filter_map(|tri| {
                let ids = tri.iter().map(|&i| i as usize).collect::<Vec<_>>();
                if mesh.normals[ids[0]][2] < 0.9 {
                    return None;
                }
                let p = ids
                    .iter()
                    .map(|&i| Vec2::new(mesh.positions[i][0], mesh.positions[i][1]))
                    .collect::<Vec<_>>();
                let side = (0..3)
                    .map(|i| (p[(i + 1) % 3] - p[i]).perp_dot(point - p[i]))
                    .collect::<Vec<_>>();
                if !(side.iter().all(|&v| v >= -0.0001) || side.iter().all(|&v| v <= 0.0001)) {
                    return None;
                }
                Some((mesh.positions[ids[0]][2], mesh.colors[ids[0]][0]))
            })
            .max_by(|a, b| a.0.total_cmp(&b.0))
            .expect("visible south wall");
        assert!(front.1 > 0.15, "backing occludes stone: {front:?}");
    }
    #[test]
    fn native_well_has_open_south_steps_and_outward_surfaces() {
        let mut mesh = SurfaceMeshData::default();
        geometry(
            &mut mesh,
            0.,
            0.,
            8.,
            8.,
            [[0.3, 0.2, 0.08, 1.], [0.7, 0.6, 0.3, 1.]],
            [0.9, 0.9, 0.8, 1.],
        );
        for tri in mesh.indices.chunks_exact(3) {
            let [a, b, c] =
                std::array::from_fn(|i| Vec3::from_array(mesh.positions[tri[i] as usize]));
            let n: Vec3 = tri
                .iter()
                .map(|&i| Vec3::from_array(mesh.normals[i as usize]))
                .sum();
            assert!((b - a).cross(c - a).dot(n) > 0.);
        }
        assert!(mesh.positions.iter().any(|p| p[1] < -8.));
        assert!(mesh.positions.iter().any(|p| p[1] > 8.));
        // Source rows2/3 are the open shaft; stairs occupy rows4/5 only.
        assert!(
            mesh.positions
                .iter()
                .zip(&mesh.colors)
                .filter(|(_, color)| color[2] > 0.6)
                .all(|(p, _)| p[2] >= 4. * 8. - 0.001)
        );
        assert_eq!(mesh.cutaway_ranges, vec![0..mesh.positions.len()]);
    }
}
