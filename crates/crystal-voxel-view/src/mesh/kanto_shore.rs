//! Source-authenticated Kanto pond/coast caps and the existing shallow drop.
//! Collision, cap contacts, water beds and footing remain unchanged.
use super::*;
const LABEL: &str = "outdoor/kanto-shore";

fn expected(block: u16, x: usize, y: usize) -> Option<u16> {
    Some(match block {
        0x1d => {
            if y == 0 || x == 0 {
                0x33
            } else {
                0x14
            }
        }
        0x1e => {
            if y == 0 {
                0x33
            } else if x == 3 {
                0x54
            } else {
                0x14
            }
        }
        0x1f => {
            if y == 0 {
                0x33
            } else {
                0x14
            }
        }
        0x2d => {
            if y == 0 && x == 3 {
                0x33
            } else {
                0x14
            }
        }
        0x2e => {
            if y == 0 && x == 0 {
                0x33
            } else {
                0x14
            }
        }
        0x64 => {
            if x == 3 {
                0x54
            } else {
                0x14
            }
        }
        0x65 => {
            if x == 0 {
                0x33
            } else {
                0x14
            }
        }
        0x67 => {
            if x < 2 {
                [[0x2a, 0x2b], [0x3a, 0x3b]][y % 2][x]
            } else if y == 0 {
                0x33
            } else {
                0x14
            }
        }
        0x6a => {
            if x >= 2 {
                [[0x2a, 0x2b], [0x3a, 0x3b]][y % 2][x - 2]
            } else if y == 0 {
                0x33
            } else {
                0x14
            }
        }
        0x78 => {
            // Bridge junction bank: authenticate its own ink instead of
            // borrowing the brick deck or ordinary checker-bank drawing.
            if y == 0 {
                0x31
            } else {
                0x14
            }
        }
        _ => return None,
    })
}
fn bank(tile: u16) -> bool {
    matches!(tile, 0x31 | 0x33 | 0x54)
}
fn ink(tile: u16, p: usize) -> usize {
    let rows = if tile == 0x33 {
        [
            "32222223", "13222231", "21322312", "22133122", "32333132", "23311323", "33132332",
            "31323223",
        ]
    } else if tile == 0x31 {
        [
            "22222222", "00000000", "33333333", "22311322", "22311312", "21300312", "21300302",
            "20022000",
        ]
    } else {
        [
            "33232103", "03321031", "30330312", "23033121", "32033212", "23300321", "13232032",
            "32321203",
        ]
    };
    usize::from(rows[p / 8].as_bytes()[p % 8] - b'0')
}
fn palette(
    cells: &[&VisualTile],
    banks: &[usize],
    images: &TerrainImageSamples,
) -> Option<[[f32; 3]; 2]> {
    let mut colors = [None; 4];
    for &i in banks {
        let rgba = tile_rgba(images, cells[i]).ok()?;
        if rgba.len() != 256 {
            return None;
        }
        for (p, b) in rgba.chunks_exact(4).enumerate() {
            let color: [u8; 4] = b.try_into().ok()?;
            if color[3] != 255 {
                return None;
            }
            let slot = &mut colors[ink(cells[i].source.tile_index, p)];
            if slot.is_some_and(|old| old != color) {
                return None;
            }
            *slot = Some(color);
        }
    }
    let [a, b, c] = [colors[1]?, colors[2]?, colors[3]?];
    if a == b || a == c || b == c {
        return None;
    }
    let linear = |v: u8| {
        let s = f32::from(v) / 255.;
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    };
    Some([a, b].map(|c| [linear(c[0]), linear(c[1]), linear(c[2])]))
}

// Accept only the original cell cap or an exact shoreline face to an actual
// adjacent water cell. A borrowed atlas swatch cannot create a bank elsewhere.
pub(in crate::mesh) fn native_face(
    p: &[[f32; 3]],
    n: [f32; 3],
    uv: &[[f32; 2]],
    i: usize,
    shapes: &[CellShape],
    g: &GridGeometry,
) -> bool {
    let x = i % g.width;
    let y = i / g.width;
    let (w, e, north, south) = g.bounds(x, y);
    let (u0, u1, v0, v1) = g.uv(x, y);
    if n == [0., 1., 0.] {
        return p
            == [
                [w, 0., north],
                [w, 0., south],
                [e, 0., south],
                [e, 0., north],
            ]
            && uv == [[u0, v0], [u0, v1], [u1, v1], [u1, v0]];
    }
    let (dx, dz) = match n {
        [-1., 0., 0.] => (-1, 0),
        [1., 0., 0.] => (1, 0),
        [0., 0., -1.] => (0, -1),
        [0., 0., 1.] => (0, 1),
        _ => return false,
    };
    let nx = x as isize + dx;
    let ny = y as isize + dz;
    if nx < 0
        || ny < 0
        || nx >= g.width as isize
        || ny >= g.height as isize
        || shapes[ny as usize * g.width + nx as usize] != CellShape::Water
    {
        return false;
    }
    let bottom = CellShape::Water.surface_height(g.tile_height);
    let crop = v1 - (v1 - v0) * (-bottom / g.tile_height);
    let (positions, uvs) = match (dx, dz) {
        (0, -1) => (
            [
                [w, bottom, north],
                [w, 0., north],
                [e, 0., north],
                [e, bottom, north],
            ],
            [[u0, v1], [u0, crop], [u1, crop], [u1, v1]],
        ),
        (0, 1) => (
            [
                [e, bottom, south],
                [e, 0., south],
                [w, 0., south],
                [w, bottom, south],
            ],
            [[u1, v1], [u1, crop], [u0, crop], [u0, v1]],
        ),
        (-1, 0) => (
            [
                [w, bottom, south],
                [w, 0., south],
                [w, 0., north],
                [w, bottom, north],
            ],
            [[u1, v1], [u1, crop], [u0, crop], [u0, v1]],
        ),
        _ => (
            [
                [e, bottom, north],
                [e, 0., north],
                [e, 0., south],
                [e, bottom, south],
            ],
            [[u0, v1], [u0, crop], [u1, crop], [u1, v1]],
        ),
    };
    p == positions && uv == uvs
}

pub(super) fn finish(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    origin: [i32; 2],
    images: &TerrainImageSamples,
) {
    let old = &mesh.textured;
    if g.width == 0
        || cells.len() != g.width * g.height
        || shapes.len() != cells.len()
        || old.positions.len() % 4 != 0
        || old.indices.len() != old.positions.len() / 4 * 6
        || old.indices.chunks_exact(6).enumerate().any(|(q, idx)| {
            let b = (q * 4) as u32;
            idx != [b, b + 1, b + 2, b, b + 2, b + 3]
        })
    {
        return;
    }
    let mut faces = vec![Vec::new(); cells.len()];
    for (q, (p, uv)) in old
        .positions
        .chunks_exact(4)
        .zip(old.uvs.chunks_exact(4))
        .enumerate()
    {
        let Some(i) = new_bark::sampled_cell(uv, g) else {
            continue;
        };
        if cells[i].source.tileset_id.as_ref() == "kanto"
            && bank(cells[i].source.tile_index)
            && native_face(p, old.normals[q * 4], uv, i, shapes, g)
            && old.normals[q * 4..q * 4 + 4]
                .iter()
                .all(|n| *n == old.normals[q * 4])
        {
            faces[i].push(q);
        }
    }
    let mut removed = std::collections::HashSet::new();
    for start in 0..cells.len() {
        let first = &cells[start].source;
        if first.tileset_id.as_ref() != "kanto"
            || first.subtile_column != 0
            || first.subtile_row != 0
            || expected(first.metatile_id, 0, 0).is_none()
        {
            continue;
        }
        let x = start % g.width;
        let y = start / g.width;
        if x + 4 > g.width || y + 4 > g.height {
            continue;
        }
        let indices: Vec<_> = (0..16).map(|j| (y + j / 4) * g.width + x + j % 4).collect();
        if indices.iter().enumerate().any(|(j, &i)| {
            let s = &cells[i].source;
            s.tileset_id != first.tileset_id
                || s.metatile_id != first.metatile_id
                || s.subtile_column as usize != j % 4
                || s.subtile_row as usize != j / 4
                || Some(s.tile_index) != expected(first.metatile_id, j % 4, j / 4)
                || (s.tile_index == 0x14 && shapes[i] != CellShape::Water)
        }) {
            continue;
        }
        let banks: Vec<_> = indices
            .into_iter()
            .filter(|&i| bank(cells[i].source.tile_index))
            .collect();
        if banks.iter().any(|&i| {
            let mut normals = std::collections::HashSet::new();
            !matches!(
                shapes[i],
                CellShape::Flat | CellShape::PlaneAt { height: 0. }
            ) || mesh.authored_cells.get(i).is_none_or(|c| c.is_some())
                || faces[i]
                    .iter()
                    .filter(|&&q| old.normals[q * 4] == [0., 1., 0.])
                    .count()
                    != 1
                || faces[i].iter().any(|&q| {
                    !normals.insert(old.normals[q * 4].map(f32::to_bits))
                        || mesh
                            .cutaway_links
                            .iter()
                            .any(|&(_, anchor)| anchor / 4 == q)
                })
        }) {
            continue;
        }
        let Some(palette) = palette(cells, &banks, images) else {
            continue;
        };
        for i in banks {
            for &q in &faces[i] {
                let p = old.positions[q * 4..q * 4 + 4].try_into().unwrap();
                super::outdoor_stone::append_coast(
                    &mut mesh.solid,
                    p,
                    old.normals[q * 4],
                    g,
                    origin,
                    palette,
                );
                removed.insert(q);
            }
            mesh.authored_cells[i] = Some(LABEL);
        }
    }
    if removed.is_empty() {
        return;
    }
    let old = std::mem::take(&mut mesh.textured);
    for (_, anchor) in &mut mesh.cutaway_links {
        *anchor -= removed.iter().filter(|&&q| q < *anchor / 4).count() * 4;
    }
    let mut marked = vec![false; old.positions.len()];
    for r in &old.cutaway_ranges {
        marked[r.clone()].fill(true);
    }
    let mut remap = vec![u32::MAX; old.positions.len()];
    mesh.textured = select_surface_indices(
        &old,
        &marked,
        &mut remap,
        old.indices
            .chunks_exact(6)
            .enumerate()
            .filter(|(q, _)| !removed.contains(q))
            .flat_map(|(_, idx)| idx.iter().copied()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    const COLORS: [[u8; 4]; 4] = [
        [245, 232, 206, 255],
        [224, 181, 99, 255],
        [139, 91, 39, 255],
        [27, 24, 19, 255],
    ];
    fn fixture(
        block: u16,
    ) -> (
        Vec<VisualTile>,
        Vec<CellShape>,
        GridGeometry,
        TerrainImageSamples,
        TerrainMeshData,
    ) {
        let g = GridGeometry {
            width: 6,
            height: 6,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: -24.,
            origin_z: 32.,
        };
        let mut assets = Assets::<Image>::default();
        let mut images = TerrainImageSamples::default();
        let mut cells = Vec::new();
        let mut shapes = Vec::new();
        let mut mesh = TerrainMeshData::default();
        for i in 0..36 {
            let x = i % 6;
            let y = i / 6;
            let inside = (1..5).contains(&x) && (1..5).contains(&y);
            let tile = if inside {
                expected(block, x - 1, y - 1).unwrap()
            } else {
                0x14
            };
            let texture = assets.add(Image::default());
            let rgba = (0..64)
                .flat_map(|p| {
                    if bank(tile) {
                        COLORS[ink(tile, p)]
                    } else {
                        [40, 85, 125, 255]
                    }
                })
                .collect();
            images
                .pixels
                .insert(texture.id(), TileImageSample::Rgba(rgba));
            cells.push(VisualTile {
                column: x as u32,
                row: y as u32,
                source: VisualTileSource {
                    tileset_id: Arc::from("kanto"),
                    metatile_id: if inside { block } else { 0x43 },
                    subtile_column: if inside { (x - 1) as u8 } else { 0 },
                    subtile_row: if inside { (y - 1) as u8 } else { 0 },
                    tile_index: tile,
                },
                texture,
                animation_frames: None,
                priority: false,
            });
            shapes.push(if tile == 0x14 {
                CellShape::Water
            } else {
                CellShape::Flat
            });
            mesh.authored_cells
                .push((inside && !bank(tile) && tile != 0x14).then_some("round-rock-fixture"));
            mesh.footing_heights.push(shapes[i].surface_height(8.));
            let (w, e, n, s) = g.bounds(x, y);
            append_top_shaded(
                &mut mesh.textured,
                [w, e, n, s],
                shapes[i].surface_height(8.),
                g.uv(x, y),
                [1.; 4],
            );
        }
        for i in 0..36 {
            if !bank(cells[i].source.tile_index) {
                continue;
            }
            let x = i % 6;
            let y = i / 6;
            let (w, e, n, s) = g.bounds(x, y);
            for d in Direction::ALL {
                let (dx, dy) = d.offset();
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx < 0
                    || ny < 0
                    || nx >= 6
                    || ny >= 6
                    || shapes[ny as usize * 6 + nx as usize] != CellShape::Water
                {
                    continue;
                }
                append_textured_shoreline(&mut mesh.textured, &g, x, y, d, [w, e, n, s], -2., 0.);
            }
        }
        (cells, shapes, g, images, mesh)
    }
    #[test]
    fn native_caps_and_all_water_drop_directions_preserve_contacts_and_other_owners() {
        let mut directions = std::collections::HashSet::new();
        for block in [0x1d, 0x1e, 0x1f, 0x2d, 0x2e, 0x64, 0x65, 0x67, 0x6a, 0x78] {
            let (cells, shapes, g, images, mut mesh) = fixture(block);
            let feet = mesh.footing_heights.clone();
            let old_claims = mesh.authored_cells.clone();
            for (q, uv) in mesh.textured.uvs.chunks_exact(4).enumerate() {
                if new_bark::sampled_cell(uv, &g).is_some_and(|i| bank(cells[i].source.tile_index))
                {
                    let n = mesh.textured.normals[q * 4];
                    if n[1] == 0. {
                        directions.insert(n.map(f32::to_bits));
                    }
                }
            }
            finish(
                &mut mesh,
                &cells.iter().collect::<Vec<_>>(),
                &shapes,
                &g,
                [13, 17],
                &images,
            );
            assert_eq!(mesh.footing_heights, feet);
            for (i, c) in cells.iter().enumerate() {
                if bank(c.source.tile_index) {
                    assert_eq!(mesh.authored_cells[i], Some(LABEL));
                } else {
                    assert_eq!(mesh.authored_cells[i], old_claims[i]);
                }
            }
            assert!(mesh
                .textured
                .uvs
                .chunks_exact(4)
                .all(|uv| new_bark::sampled_cell(uv, &g)
                    .is_none_or(|i| !bank(cells[i].source.tile_index))));
            assert!(mesh
                .solid
                .positions
                .iter()
                .all(|p| p.iter().all(|v| v.is_finite()) && p[1] >= -2. && p[1] <= 0.));
            assert!(mesh.solid.positions.iter().any(|p| p[1] == 0.));
            assert!(mesh.solid.positions.iter().any(|p| p[1] == -2.));
            for (tri, n) in mesh.solid.indices.chunks_exact(3).zip(
                mesh.solid
                    .indices
                    .chunks_exact(3)
                    .map(|t| Vec3::from_array(mesh.solid.normals[t[0] as usize])),
            ) {
                let [a, b, c] = std::array::from_fn(|j| {
                    Vec3::from_array(mesh.solid.positions[tri[j] as usize])
                });
                assert!((b - a).cross(c - a).dot(n) > 0., "outward coast winding");
            }
        }
        assert_eq!(
            directions.len(),
            4,
            "north, south, east and west drops exercised"
        );
    }
    #[test]
    fn changed_cropped_claimed_borrowed_or_elevated_bank_stays_on_source() {
        for block in [0x1d, 0x78] {
            for defect in 0..8 {
                let (mut cells, mut shapes, g, mut images, mut mesh) = fixture(block);
                let i = 7;
                match defect {
                    0 => cells[8].source.subtile_column = 0,
                    1 => cells[8].source.tile_index = 0x11,
                    2 => mesh.authored_cells[i] = Some("other"),
                    3 => shapes[i] = CellShape::PlaneAt { height: 1. },
                    4 => mesh.textured.positions[i * 4][1] = 1.,
                    5 => mesh.textured.uvs[i * 4..i * 4 + 4].fill([0., 0.]),
                    6 => {
                        if let TileImageSample::Rgba(rgba) =
                            images.pixels.get_mut(&cells[i].texture.id()).unwrap()
                        {
                            rgba[3] = 0;
                        }
                    }
                    _ => shapes[14] = CellShape::PlaneAt { height: -2. },
                }
                let old = mesh.textured.clone();
                finish(
                    &mut mesh,
                    &cells.iter().collect::<Vec<_>>(),
                    &shapes,
                    &g,
                    [0, 0],
                    &images,
                );
                assert_eq!(mesh.textured, old, "block {block:#x}, defect {defect}");
                assert!(mesh.solid.positions.is_empty());
            }
            let (mut cells, shapes, g, images, mut mesh) = fixture(block);
            cells.truncate(20);
            let old = mesh.textured.clone();
            finish(
                &mut mesh,
                &cells.iter().collect::<Vec<_>>(),
                &shapes,
                &g,
                [0, 0],
                &images,
            );
            assert_eq!(mesh.textured, old);
        }
    }
    #[test]
    fn source_palette_changes_paint_without_changing_shore_geometry_or_footing() {
        for block in [0x1e, 0x78] {
            let (cells, shapes, g, images, mut day) = fixture(block);
            let mut night_images = images.clone();
            for sample in night_images.pixels.values_mut() {
                if let TileImageSample::Rgba(rgba) = sample {
                    for p in rgba.chunks_exact_mut(4) {
                        p[0] /= 2;
                        p[1] /= 2;
                        p[2] = (u16::from(p[2]) * 3 / 4) as u8;
                    }
                }
            }
            let mut night = day.clone();
            let cells = cells.iter().collect::<Vec<_>>();
            finish(&mut day, &cells, &shapes, &g, [13, 17], &images);
            finish(&mut night, &cells, &shapes, &g, [13, 17], &night_images);
            assert!(!day.solid.positions.is_empty());
            assert_eq!(day.solid.positions, night.solid.positions);
            assert_eq!(day.footing_heights, night.footing_heights);
            assert_ne!(day.solid.colors, night.solid.colors);
        }
    }
}
