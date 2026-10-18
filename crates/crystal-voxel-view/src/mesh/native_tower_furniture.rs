//! The Tower's low service counter, integrated PCs, and gold waiting stools.
use super::*;
use bevy::prelude::Vec2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shape {
    Front,
    Return,
    TerminalReturn,
    Pc,
    Stool,
}

fn spec(block: u16, sx: u8, sy: u8) -> Option<(Shape, usize, usize, &'static [u16])> {
    Some(match (block, sx, sy) {
        (2, 0, 0) => (Shape::Front, 4, 2, &[6, 10, 10, 10, 22, 26, 26, 26]),
        (0x28, 0, 0) => (Shape::Front, 4, 2, &[10, 10, 42, 43, 26, 26, 26, 27]),
        (0x29, 0, 0) => (Shape::Front, 4, 2, &[53, 129, 10, 10, 22, 26, 26, 26]),
        (0x14, 0, 0) | (0x15, 2, 0) => (Shape::Return, 2, 4, &[53, 11, 53, 11, 53, 11, 53, 11]),
        (0x2a, 0, 2) | (0x2b, 2, 2) => (Shape::Return, 2, 2, &[53, 11, 53, 11]),
        (0x19, 2, 0) => (
            Shape::TerminalReturn,
            2,
            4,
            &[53, 11, 53, 11, 40, 41, 56, 57],
        ),
        (5, 0, 0) => (Shape::Pc, 2, 2, &[12, 13, 28, 29]),
        (7, 0 | 2, 0) => (Shape::Stool, 2, 2, &[14, 15, 30, 31]),
        _ => return None,
    })
}
fn dimensions(shape: Shape, height: usize) -> (ModelKind, f32, f32) {
    match shape {
        Shape::Front => (ModelKind::ReceptionCounter, 8., 8.),
        Shape::Return => (ModelKind::ReceptionCounter, height as f32 * 8., 8.),
        Shape::TerminalReturn => (ModelKind::ReceptionCounter, height as f32 * 8. + 8., 20.),
        Shape::Pc => (ModelKind::Computer, 8., 18.),
        Shape::Stool => (ModelKind::Stool, 16., 7.),
    }
}
fn source(map: &str, p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> Option<Shape> {
    if map != "BattleTower1F"
        || cells.len() != g.width * g.height
        || p.column + p.width > g.width
        || p.row + p.height > g.height
        || p.base_pixels != 0.
        || p.footing_pixels.is_some()
    {
        return None;
    }
    let first = &cells[p.row * g.width + p.column].source;
    let (shape, w, h, tiles) = spec(first.metatile_id, first.subtile_column, first.subtile_row)?;
    let (kind, depth, height) = dimensions(shape, h);
    if shape == Shape::TerminalReturn {
        let footer = Placement {
            column: p.column.checked_sub(2)?,
            row: p.row + p.height,
            width: 4,
            height: 2,
            depth_pixels: 8.,
            height_pixels: 8.,
            ..p.clone()
        };
        if source(map, &footer, cells, g) != Some(Shape::Front)
            || cells[footer.row * g.width + footer.column]
                .source
                .metatile_id
                != 0x28
        {
            return None;
        }
    }
    (p.kind == kind
        && p.width == w
        && p.height == h
        && p.depth_pixels == depth
        && p.height_pixels == height
        && p.indices(g.width).enumerate().all(|(j, i)| {
            let s = &cells[i].source;
            s.tileset_id.as_ref() == "battle_tower_inside"
                && s.metatile_id == first.metatile_id
                && s.subtile_column == first.subtile_column + (j % w) as u8
                && s.subtile_row == first.subtile_row + (j / w) as u8
                && s.tile_index == tiles[j]
        }))
    .then_some(shape)
}
pub(super) fn candidates(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    claimed: &mut [bool],
    out: &mut Vec<Placement>,
) {
    if map != "BattleTower1F" || g.width == 0 {
        return;
    }
    let Some(ground) = ground_sample(cells, map, "battle_tower_inside", 0x11) else {
        return;
    };
    for (i, tile) in cells.iter().enumerate() {
        let s = &tile.source;
        let Some((shape, width, height, _)) = spec(s.metatile_id, s.subtile_column, s.subtile_row)
        else {
            continue;
        };
        let (kind, depth_pixels, height_pixels) = dimensions(shape, height);
        let p = Placement {
            kind,
            column: i % g.width,
            row: i / g.width,
            width,
            height,
            ground,
            depth_pixels,
            height_pixels,
            base_pixels: 0.,
            footing_pixels: None,
        };
        if source(map, &p, cells, g).is_none() || p.indices(g.width).any(|j| claimed[j]) {
            continue;
        }
        if shape == Shape::TerminalReturn
            && (0..2).any(|y| {
                (0..4).any(|x| claimed[(p.row + p.height + y) * g.width + p.column - 2 + x])
            })
        {
            continue;
        }
        for j in p.indices(g.width) {
            claimed[j] = true;
        }
        out.push(p);
    }
}
fn texel(g: &GridGeometry, x: usize, y: usize, px: f32, py: f32) -> [f32; 2] {
    let (u0, u1, v0, v1) = g.uv(x, y);
    [u0 + (u1 - u0) * px / 8., v0 + (v1 - v0) * py / 8.]
}
fn painted_box(out: &mut SurfaceMeshData, b: [f32; 6], uv: [f32; 2]) {
    let first = out.positions.len();
    architecture_box(out, b, [1.; 3]);
    out.uvs[first..].fill(uv);
}
fn front(
    out: &mut SurfaceMeshData,
    g: &GridGeometry,
    p: &Placement,
    row: usize,
    rows: usize,
    [w, e, low, high, z]: [f32; 5],
) {
    for y in 0..rows {
        for x in 0..p.width {
            let (u0, u1, v0, v1) = g.uv(p.column + x, row + y);
            let a = w + (e - w) * x as f32 / p.width as f32;
            let b = w + (e - w) * (x + 1) as f32 / p.width as f32;
            let top = high - (high - low) * y as f32 / rows as f32;
            let bottom = high - (high - low) * (y + 1) as f32 / rows as f32;
            append_quad(
                out,
                [[b, bottom, z], [b, top, z], [a, top, z], [a, bottom, z]],
                [0., 0., 1.],
                [[u1, v1], [u1, v0], [u0, v0], [u0, v1]],
                TEXTURED_SHADE,
            );
        }
    }
}
fn body_paint(cells: &[&VisualTile], g: &GridGeometry) -> Option<[f32; 2]> {
    cells.iter().enumerate().find_map(|(i, tile)| {
        let s = &tile.source;
        (s.tileset_id.as_ref() == "battle_tower_inside"
            && s.metatile_id == 2
            && s.subtile_row == 1
            && s.subtile_column == 1
            && s.tile_index == 26)
            .then(|| texel(g, i % g.width, i / g.width, 3.5, 3.5))
    })
}
// Clip a cushion's source charts to its real chamfered outline, preserving the
// atlas seam and the engraved/button detail without stretching its feet on top.
fn clipped(mut polygon: Vec<Vec2>, axis: usize, edge: f32, greater: bool) -> Vec<Vec2> {
    let input = std::mem::take(&mut polygon);
    if input.is_empty() {
        return polygon;
    }
    let inside = |v: Vec2| {
        if greater {
            v[axis] >= edge
        } else {
            v[axis] <= edge
        }
    };
    let mut a = *input.last().unwrap();
    for b in input {
        if inside(a) != inside(b) {
            polygon.push(a + (b - a) * ((edge - a[axis]) / (b[axis] - a[axis])));
        }
        if inside(b) {
            polygon.push(b);
        }
        a = b;
    }
    polygon
}
fn ring(w: f32, e: f32, n: f32, s: f32, b: f32) -> Vec<Vec2> {
    vec![
        Vec2::new(w + b, n),
        Vec2::new(w, n + b),
        Vec2::new(w, s - b),
        Vec2::new(w + b, s),
        Vec2::new(e - b, s),
        Vec2::new(e, s - b),
        Vec2::new(e, n + b),
        Vec2::new(e - b, n),
    ]
}
fn stool(out: &mut SurfaceMeshData, g: &GridGeometry, p: &Placement, [w, e, n, s]: [f32; 4]) {
    let k = g.tile_height / 8.;
    let gold = texel(g, p.column, p.row, 4.5, 3.5);
    let dark = texel(g, p.column, p.row + 1, 3.5, 3.5);
    let (cx, cz) = ((w + e) * 0.5, (n + s) * 0.5);
    for x in [-4.5, 4.5] {
        for z in [-4.5, 4.5] {
            painted_box(
                out,
                [
                    cx + (x - 1.) * k,
                    cx + (x + 1.) * k,
                    0.,
                    3. * k,
                    cz + (z - 1.) * k,
                    cz + (z + 1.) * k,
                ],
                dark,
            );
        }
    }
    let outline = ring(w + k, e - k, n + k, s - k, 2. * k);
    let top = outline
        .iter()
        .map(|v| Vec2::new(cx, cz) + (*v - Vec2::new(cx, cz)) * 0.93)
        .collect::<Vec<_>>();
    for j in 0..8 {
        let a = outline[j];
        let b = outline[(j + 1) % 8];
        for (low, high, upper) in [(3. * k, 6.25 * k, false), (6.25 * k, 7. * k, true)] {
            let (c, d) = if upper {
                (top[(j + 1) % 8], top[j])
            } else {
                (b, a)
            };
            let vertices = [
                [a.x, low, a.y],
                [b.x, low, b.y],
                [c.x, high, c.y],
                [d.x, high, d.y],
            ];
            let normal = (Vec3::from_array(vertices[1]) - Vec3::from_array(vertices[0]))
                .cross(Vec3::from_array(vertices[2]) - Vec3::from_array(vertices[0]))
                .normalize();
            append_quad(
                out,
                vertices,
                normal.to_array(),
                [gold; 4],
                crate::interior_models::authored_face_color([1.; 4], normal),
            );
        }
    }
    // Finish the underside; the four feet meet this closed slab.
    for j in 1..outline.len() - 1 {
        let first = out.positions.len() as u32;
        for v in [outline[0], outline[j + 1], outline[j]] {
            out.positions.push([v.x, 3. * k, v.y]);
            out.normals.push([0., -1., 0.]);
            out.uvs.push(dark);
            out.colors.push(TEXTURED_SHADE);
        }
        out.indices.extend([first, first + 1, first + 2]);
    }
    let (tw, te, tn, ts) = (cx - 6.51 * k, cx + 6.51 * k, cz - 6.51 * k, cz + 6.51 * k);
    for y in 0..2 {
        for x in 0..2 {
            let (sx0, sx1) = ((x * 8).max(1), ((x + 1) * 8).min(15));
            let (sy0, sy1) = ((y * 8).max(1), ((y + 1) * 8).min(10));
            let [a, b, c, d] = [
                tw + (te - tw) * (sx0 - 1) as f32 / 14.,
                tw + (te - tw) * (sx1 - 1) as f32 / 14.,
                tn + (ts - tn) * (sy0 - 1) as f32 / 9.,
                tn + (ts - tn) * (sy1 - 1) as f32 / 9.,
            ];
            let mut polygon = top.clone();
            for (axis, edge, greater) in [(0, a, true), (0, b, false), (1, c, true), (1, d, false)]
            {
                polygon = clipped(polygon, axis, edge, greater);
            }
            if polygon.len() < 3 {
                continue;
            }
            let first = out.positions.len() as u32;
            let (u0, u1, v0, v1) = g.uv(p.column + x, p.row + y);
            for v in &polygon {
                let sx = 1. + (v.x - tw) / (te - tw) * 14. - x as f32 * 8.;
                let sy = 1. + (v.y - tn) / (ts - tn) * 9. - y as f32 * 8.;
                out.positions.push([v.x, 7. * k, v.y]);
                out.normals.push([0., 1., 0.]);
                out.uvs
                    .push([u0 + (u1 - u0) * sx / 8., v0 + (v1 - v0) * sy / 8.]);
                out.colors.push(TEXTURED_SHADE);
            }
            for j in 1..polygon.len() - 1 {
                if (polygon[j] - polygon[0])
                    .perp_dot(polygon[j + 1] - polygon[0])
                    .abs()
                    < 0.0001
                {
                    continue;
                }
                out.indices
                    .extend([first, first + j as u32, first + j as u32 + 1]);
            }
        }
    }
}
pub(super) fn append(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    p: &Placement,
    claimed: &mut [bool],
) -> bool {
    let Some(shape) = source(map, p, cells, g) else {
        return false;
    };
    if p.indices(g.width).any(|i| claimed[i]) {
        return false;
    }
    let paint = if matches!(shape, Shape::Front) {
        texel(g, p.column + 1, p.row + 1, 3.5, 3.5)
    } else if matches!(shape, Shape::Return | Shape::TerminalReturn) {
        let Some(uv) = body_paint(cells, g) else {
            return false;
        };
        uv
    } else {
        texel(g, p.column, p.row, 2.5, 1.5)
    };
    for i in p.indices(g.width) {
        append_top(
            &mut mesh.textured,
            g.bounds(i % g.width, i / g.width).into(),
            0.,
            g.uv(p.ground % g.width, p.ground / g.width),
        );
    }
    let first = mesh.textured.positions.len();
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let k = g.tile_height / 8.;
    let e = w + p.width as f32 * g.tile_width;
    match shape {
        Shape::Stool => stool(
            &mut mesh.textured,
            g,
            p,
            [w, e, n, n + p.height as f32 * g.tile_height],
        ),
        Shape::Pc => {
            painted_box(
                &mut mesh.textured,
                [w, e, 0., 18. * k, n + 8. * k, n + 16. * k - 0.02 * k],
                paint,
            );
            front(
                &mut mesh.textured,
                g,
                p,
                p.row,
                2,
                [w, e, 0., 18. * k, n + 16. * k],
            );
        }
        Shape::Front => {
            // The second source row is vertical fascia: anchor its foot at
            // the source drawing's front edge, leaving its apron on the floor.
            let indices = mesh.textured.indices.len();
            painted_box(
                &mut mesh.textured,
                [w, e, 0., 8. * k, n + 8. * k, n + 16. * k - 0.02 * k],
                paint,
            );
            mesh.textured.indices.drain(indices..indices + 6);
            let corner = cells[p.row * g.width + p.column].source.metatile_id == 0x29;
            if corner {
                let indices = mesh.textured.indices.len();
                painted_box(
                    &mut mesh.textured,
                    [w, w + 2. * g.tile_width, 0., 8. * k, n, n + 8. * k],
                    paint,
                );
                mesh.textured.indices.drain(indices..indices + 6);
            }
            for x in 0..p.width {
                append_top(
                    &mut mesh.textured,
                    [
                        w + x as f32 * g.tile_width,
                        w + (x + 1) as f32 * g.tile_width,
                        if corner && x < 2 { n } else { n + 8. * k },
                        n + 16. * k,
                    ],
                    8. * k,
                    g.uv(p.column + x, p.row),
                );
            }
            front(
                &mut mesh.textured,
                g,
                p,
                p.row + 1,
                1,
                [w, e, 0., 8. * k, n + 16. * k],
            );
        }
        Shape::Return | Shape::TerminalReturn => {
            let source_end = n + p.height as f32 * g.tile_height;
            let s = source_end
                + if shape == Shape::TerminalReturn {
                    8. * k
                } else {
                    0.
                };
            let indices = mesh.textured.indices.len();
            painted_box(&mut mesh.textured, [w, e, 0., 8. * k, n, s], paint);
            mesh.textured.indices.drain(indices..indices + 6);
            for y in 0..p.height {
                for x in 0..2 {
                    let start = mesh.textured.positions.len();
                    append_top(
                        &mut mesh.textured,
                        [
                            w + x as f32 * g.tile_width,
                            w + (x + 1) as f32 * g.tile_width,
                            n + y as f32 * g.tile_height,
                            n + (y + 1) as f32 * g.tile_height,
                        ],
                        8. * k,
                        g.uv(p.column + x, p.row + y),
                    );
                    if shape == Shape::TerminalReturn && y >= 2 {
                        let white = texel(g, p.column + 1, p.row, 3.5, 3.5);
                        mesh.textured.uvs[start..].fill(white);
                    }
                }
            }
            if shape == Shape::TerminalReturn {
                let first = mesh.textured.positions.len();
                append_top(
                    &mut mesh.textured,
                    [w, e, source_end, s],
                    8. * k,
                    g.uv(p.column + 1, p.row),
                );
                mesh.textured.uvs[first..].fill(texel(g, p.column + 1, p.row, 3.5, 3.5));
            }
            if shape == Shape::TerminalReturn {
                let gray = texel(g, p.column, p.row + 2, 4.5, 2.5);
                painted_box(&mut mesh.textured, [w, e, 8. * k, 20. * k, s - k, s], gray);
                front(
                    &mut mesh.textured,
                    g,
                    p,
                    p.row + 2,
                    2,
                    [w, e, 8. * k, 20. * k, s + 0.02 * k],
                );
            }
        }
    }
    mesh.textured
        .cutaway_ranges
        .push(first..mesh.textured.positions.len());
    for i in p.indices(g.width) {
        claimed[i] = true;
        if mesh.authored_cells.len() == cells.len() {
            mesh.authored_cells[i] = Some(p.label());
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    fn covered_top(out: &SurfaceMeshData, point: Vec3) -> bool {
        out.indices.chunks_exact(3).any(|t| {
            if out.normals[t[0] as usize][1] < 0.99 {
                return false;
            }
            let [a, b, c] = [0, 1, 2].map(|j| Vec3::from_array(out.positions[t[j] as usize]));
            let n = (b - a).cross(c - a);
            n.length() > 0.0001
                && ((point - a).dot(n) / n.length()).abs() < 0.001
                && [(a, b), (b, c), (c, a)]
                    .iter()
                    .all(|&(u, v)| (v - u).cross(point - u).dot(n) >= -0.001)
        })
    }
    #[test]
    fn external_tower_service_counter_and_stools_keep_native_roles_and_walkable_support() {
        let Some(path) = std::env::var_os("CRYSTAL_RENDER_TEST_PACK") else {
            return;
        };
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(path).unwrap();
        let pack = loaded.pack();
        let room = &pack.data().maps["BattleTower1F"];
        let layout = &pack.runtime_files()["data/tilesets/battle_tower_inside_metatiles.bin"];
        let (w, h) = (
            room.attributes.width as usize * 4,
            room.attributes.height as usize * 4,
        );
        let tiles = (0..w * h)
            .map(|i| {
                let (x, y) = (i % w, i / w);
                let b = room.blocks[y / 4 * (w / 4) + x / 4];
                VisualTile {
                    column: x as u32,
                    row: y as u32,
                    texture: Default::default(),
                    animation_frames: None,
                    priority: false,
                    source: VisualTileSource {
                        tileset_id: "battle_tower_inside".into(),
                        metatile_id: b,
                        subtile_column: (x % 4) as u8,
                        subtile_row: (y % 4) as u8,
                        tile_index: layout[b as usize * 16 + y % 4 * 4 + x % 4] as u16,
                    },
                }
            })
            .collect::<Vec<_>>();
        let refs = tiles.iter().collect::<Vec<_>>();
        let g = GridGeometry {
            width: w,
            height: h,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: -24.,
            origin_z: -16.,
        };
        let resolved = resolve("BattleTower1F", &refs, &g, None);
        let selected = resolved
            .iter()
            .filter(|p| source("BattleTower1F", p, &refs, &g).is_some())
            .collect::<Vec<_>>();
        let mut combined = TerrainMeshData::default();
        let mut combined_claimed = vec![false; w * h];
        let mut counts = [0; 5];
        let mut reserved = vec![false; w * h];
        for p in &selected {
            let shape = source("BattleTower1F", p, &refs, &g).unwrap();
            counts[match shape {
                Shape::Front => 0,
                Shape::Return => 1,
                Shape::TerminalReturn => 2,
                Shape::Pc => 3,
                Shape::Stool => 4,
            }] += 1;
            for i in p.indices(w) {
                assert!(!reserved[i], "furniture ownership cannot overlap");
                reserved[i] = true;
            }
            let mut mesh = TerrainMeshData {
                footing_heights: vec![2.75; w * h],
                authored_cells: vec![None; w * h],
                ..Default::default()
            };
            let mut claimed = vec![false; w * h];
            assert!(append_with_tables(
                &mut mesh,
                "BattleTower1F",
                &refs,
                &g,
                p,
                &mut claimed,
                [0, 0],
                &resolved
            ));
            assert_eq!(
                mesh.footing_heights,
                vec![2.75; w * h],
                "a counter/stool cannot raise walking support"
            );
            assert_eq!(claimed.iter().filter(|&&v| v).count(), p.width * p.height);
            assert!(mesh.solid.indices.is_empty());
            assert!(
                mesh.textured
                    .positions
                    .iter()
                    .all(|v| v.iter().all(|x| x.is_finite()))
            );
            assert!(
                mesh.textured
                    .positions
                    .iter()
                    .all(|v| v[1] >= 0. && v[1] <= p.height_pixels)
            );
            for t in mesh.textured.indices.chunks_exact(3) {
                let [a, b, c] =
                    [0, 1, 2].map(|j| Vec3::from_array(mesh.textured.positions[t[j] as usize]));
                let normal = Vec3::from_array(mesh.textured.normals[t[0] as usize]);
                assert!(
                    (b - a).cross(c - a).dot(normal) > 0.,
                    "closed visible faces retain correct winding"
                );
            }
            if shape == Shape::Stool {
                assert_eq!(p.kind, ModelKind::Stool);
                assert_eq!(
                    p.seat_height_pixels("BattleTower1F", &refs[p.row * w + p.column].source),
                    Some(7.)
                );
                let object = mesh.textured.cutaway_ranges.last().unwrap();
                assert!(
                    mesh.textured.positions[object.clone()]
                        .iter()
                        .all(|v| v[1] <= 7.),
                    "waiting stools never acquire a tall office-chair back"
                );
                assert!(
                    mesh.textured.normals[object.clone()]
                        .iter()
                        .any(|v| v[0].abs() > 0.2 && v[2].abs() > 0.2),
                    "the cushion has real chamfered corners"
                );
                assert!(
                    mesh.textured.positions[object.clone()]
                        .iter()
                        .filter(|v| v[1] == 0.)
                        .count()
                        >= 16,
                    "four supporting feet meet the floor"
                );
            }
            assert!(append(
                &mut combined,
                "BattleTower1F",
                &refs,
                &g,
                p,
                &mut combined_claimed
            ));
            if shape == Shape::TerminalReturn {
                let mut changed = tiles.clone();
                changed[(p.row + p.height) * w + p.column].source.tile_index ^= 0x80;
                assert!(
                    source("BattleTower1F", p, &changed.iter().collect::<Vec<_>>(), &g).is_none(),
                    "monitor return requires its actual keyboard/front course"
                );
            }
            for i in p.indices(w) {
                let mut changed = tiles.clone();
                changed[i].source.tile_index ^= 0x80;
                assert!(
                    source("BattleTower1F", p, &changed.iter().collect::<Vec<_>>(), &g).is_none()
                );
            }
            assert!(source("BattleTowerHallway", p, &refs, &g).is_none());
            let mut altered = (*p).clone();
            altered.height_pixels += 1.;
            assert!(source("BattleTower1F", &altered, &refs, &g).is_none());
            let mut blocked = vec![false; w * h];
            blocked[p.row * w + p.column] = true;
            let mut untouched = TerrainMeshData::default();
            assert!(!append(
                &mut untouched,
                "BattleTower1F",
                &refs,
                &g,
                p,
                &mut blocked
            ));
            assert!(untouched.textured.indices.is_empty());
        }
        for x in [4.5 * 16., 11.5 * 16.] {
            for z in (17..112).step_by(3) {
                assert!(
                    covered_top(
                        &combined.textured,
                        Vec3::new(g.origin_x + x, 8., g.origin_z + z as f32)
                    ),
                    "the entire U return must connect to its front corner without a gap at {x},{z}"
                );
            }
        }
        for x in (65..96).chain(129..192).step_by(3) {
            assert!(
                covered_top(
                    &combined.textured,
                    Vec3::new(g.origin_x + x as f32, 8., g.origin_z + 108.)
                ),
                "front courses retain a contiguous low countertop"
            );
        }
        assert_eq!(
            counts,
            [3, 5, 1, 1, 6],
            "complete U counter, terminal, blue PC and six gold stools"
        );
        assert_eq!(selected.len(), 16);
    }
}
