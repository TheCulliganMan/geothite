//! Battle Tower control walls and real doors, including projected canopy tops.
use super::*;

fn drawing(map: &str, p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> Option<u16> {
    if !matches!(map, "BattleTower1F" | "BattleTowerHallway")
        || cells.len() != g.width * g.height
        || p.column + p.width > g.width
        || p.row + p.height > g.height
        || p.base_pixels != 0.
        || p.footing_pixels.is_some()
    {
        return None;
    }
    let s = &cells[p.row * g.width + p.column].source;
    let (block, origin, size, kind, tiles): (_, _, _, _, &[u16]) = match (s.metatile_id, p.kind) {
        (0x18, ModelKind::WallPanel) if map == "BattleTower1F" => (
            0x18, [0, 0], [2, 4], ModelKind::WallPanel,
            &[0x2e, 0x2f, 0x3e, 0x3f, 0x2e, 0x2f, 0x3e, 0x3f],
        ),
        (0x16, ModelKind::WallPanel) if map == "BattleTower1F" => (
            0x16, [2, 0], [2, 4], ModelKind::WallPanel,
            &[0x2c, 0x2d, 0x3c, 0x3d, 0x2c, 0x2d, 0x3c, 0x3d],
        ),
        (0x1a, ModelKind::WallPanel) if map == "BattleTower1F" => (
            0x1a, [0, 2], [2, 2], ModelKind::WallPanel, &[0x40, 0x11, 0x2e, 0x41],
        ),
        (0x17, ModelKind::WallPanel) if map == "BattleTower1F" => (
            0x17, [2, 2], [2, 2], ModelKind::WallPanel, &[0x11, 0x3b, 0x3a, 0x2d],
        ),
        (4, ModelKind::WallPanel) => (
            4,
            [0, 0],
            [2, 2],
            ModelKind::WallPanel,
            &[0x54, 0x55, 0x50, 0x51],
        ),
        (3, ModelKind::WallPanel) => (
            3,
            [0, 0],
            [4, 2],
            ModelKind::WallPanel,
            &[0x54, 0x55, 0x54, 0x55, 0x50, 0x51, 0x50, 0x51],
        ),
        (0x1b, ModelKind::WallPanel) => (
            0x1b,
            [0, 0],
            [4, 3],
            ModelKind::WallPanel,
            &[
                0x43, 0x45, 0x44, 0x45, 0x53, 0x55, 0x54, 0x55, 0x50, 0x51, 0x50, 0x51,
            ],
        ),
        (0x1d, ModelKind::WallPanel) => (
            0x1d,
            [0, 0],
            [4, 3],
            ModelKind::WallPanel,
            &[
                0x44, 0x45, 0x44, 0x45, 0x54, 0x55, 0x54, 0x55, 0x50, 0x51, 0x50, 0x51,
            ],
        ),
        (0x1e, ModelKind::WallPanel) => (
            0x1e,
            [0, 0],
            [4, 3],
            ModelKind::WallPanel,
            &[
                0x44, 0x45, 0x44, 0x45, 0x48, 0x49, 0x4a, 0x55, 0x50, 0x51, 0x50, 0x51,
            ],
        ),
        (0x2a, ModelKind::WallPanel) => (
            0x2a,
            [0, 0],
            [4, 2],
            ModelKind::WallPanel,
            &[0x54, 0x55, 0x54, 0x55, 0x4f, 0x5f, 0x50, 0x51],
        ),
        (0x2b, ModelKind::WallPanel) => (
            0x2b,
            [0, 0],
            [4, 2],
            ModelKind::WallPanel,
            &[0x54, 0x55, 0x54, 0x55, 0x50, 0x51, 0x4f, 0x5f],
        ),
        (4, ModelKind::GateDoorFrame) => (
            4,
            [2, 0],
            [2, 2],
            ModelKind::GateDoorFrame,
            &[2, 3, 0x12, 0x13],
        ),
        (0x25, ModelKind::GateDoorFrame) if map == "BattleTowerHallway" => (
            0x25,
            [0, 0],
            [4, 3],
            ModelKind::GateDoorFrame,
            &[
                0x43, 0x45, 0x45, 0x45, 0x54, 0x55, 2, 3, 0x50, 0x51, 0x12, 0x13,
            ],
        ),
        _ => return None,
    };
    (p.kind == kind
        && [p.width, p.height] == size
        && p.indices(g.width).enumerate().all(|(j, i)| {
            let s = &cells[i].source;
            s.tileset_id.as_ref() == "battle_tower_inside"
                && s.metatile_id == block
                && s.subtile_column as usize == origin[0] + j % size[0]
                && s.subtile_row as usize == origin[1] + j / size[0]
                && s.tile_index == tiles[j]
        }))
    .then_some(block)
}

fn paint(g: &GridGeometry, column: usize, row: usize) -> [f32; 2] {
    let uv = g.uv(column, row);
    [
        uv.0 + (uv.1 - uv.0) * 0.5 / 8.,
        uv.2 + (uv.3 - uv.2) * 3.5 / 8.,
    ]
}
fn block(out: &mut SurfaceMeshData, b: [f32; 6], uv: [f32; 2]) {
    let first = out.positions.len();
    architecture_box(out, b, [1.; 3]);
    out.uvs[first..].fill(uv);
}
fn art(
    out: &mut SurfaceMeshData,
    g: &GridGeometry,
    column: usize,
    row: usize,
    size: [usize; 2],
    b: [f32; 5],
) {
    let [w, e, low, high, z] = b;
    for y in 0..size[1] {
        for x in 0..size[0] {
            let uv = g.uv(column + x, row + y);
            let left = w + (e - w) * x as f32 / size[0] as f32;
            let right = w + (e - w) * (x + 1) as f32 / size[0] as f32;
            let top = high - (high - low) * y as f32 / size[1] as f32;
            let bottom = high - (high - low) * (y + 1) as f32 / size[1] as f32;
            append_quad(
                out,
                [
                    [right, bottom, z],
                    [right, top, z],
                    [left, top, z],
                    [left, bottom, z],
                ],
                [0., 0., 1.],
                [[uv.1, uv.3], [uv.1, uv.2], [uv.0, uv.2], [uv.0, uv.3]],
                crate::interior_models::authored_face_color([1.; 4], Vec3::Z),
            );
        }
    }
}
fn wall(
    out: &mut SurfaceMeshData,
    g: &GridGeometry,
    column: usize,
    row: usize,
    size: [usize; 2],
    b: [f32; 4],
    uv: [f32; 2],
) {
    let [w, e, n, s] = b;
    let k = g.tile_height / 8.;
    // Keep the original control detail at native scale on a full-height wall.
    block(out, [w, e, 0., 32. * k, n, s - 0.02 * k], uv);
    art(
        out,
        g,
        column,
        row,
        size,
        [w, e, 0., size[1] as f32 * 8. * k, s],
    );
}
fn door(
    out: &mut SurfaceMeshData,
    g: &GridGeometry,
    column: usize,
    row: usize,
    b: [f32; 4],
    uv: [f32; 2],
) {
    let [w, e, n, s] = b;
    let k = g.tile_height / 8.;
    let post = k.min((e - w) * 0.1);
    let back = (s - 9. * k).max(n);
    // Keep the whole approach clear below the lintel. The leaves sit behind
    // the warp's walking center, rather than sealing the front of its tile.
    for [left, right] in [[w, w + post], [e - post, e]] {
        block(out, [left, right, 0., 32. * k, back, s], uv);
    }
    block(out, [w + post, e - post, 26. * k, 32. * k, back, s], uv);
    block(
        out,
        [w + post, e - post, 0., 26. * k, back, back + 0.3 * k],
        uv,
    );
    art(
        out,
        g,
        column,
        row,
        [2, 2],
        [w + post, e - post, 0., 24. * k, back + 0.32 * k],
    );
}

fn side_wall(out: &mut SurfaceMeshData, g: &GridGeometry, p: &Placement, source: u16, b: [f32; 4]) {
    let [w, e, n, s] = b;
    let k = g.tile_height / 8.;
    let west = matches!(source, 0x18 | 0x1a);
    let corner = matches!(source, 0x17 | 0x1a);
    // The entire blocked quadrant connects to the outer backing. Floor on the
    // other side of the course, including the corner's upper half, is untouched.
    // Gold is source shade2. The outer source tile mostly contains projected
    // FLOOR pixels, so it is neither wall paint nor a vertical wall texture.
    let donor_column = p.column + usize::from(west && !corner || !west && corner);
    // Corner cap tiles40/3b use RED, despite their gold-looking grayscale
    // drawing. The lower2e/2d edge carries the actual BROWN wall palette.
    let uv = g.uv(donor_column, p.row + usize::from(corner));
    let pixel = if corner { if west { 6.5 } else { 1.5 } }
        else if west { 4.5 } else { 2.5 };
    let gold = [uv.0 + (uv.1 - uv.0) * pixel / 8., uv.2 + (uv.3 - uv.2) * 0.5 / 8.];
    block(out, [w, e, 0., 32. * k, n, s], gold);
    if corner { return; }
    // Fold only the inward ornamental strip onto the lower wall. Exclude the
    // six-pixel-wide source projection's white floor edge; keep each native
    // repeat along the wall and a plain upper field, as on the rear course.
    let x = if west { e + 0.02 * k } else { w - 0.02 * k };
    for row in 0..p.height {
        let uv = g.uv(donor_column, p.row + row);
        let z0 = n + row as f32 * g.tile_height;
        let z1 = z0 + g.tile_height;
        let u0 = uv.0 + (uv.1 - uv.0) * if west { 2. / 8. } else { 0. };
        let u1 = uv.0 + (uv.1 - uv.0) * if west { 1. } else { 6. / 8. };
        let (positions, uvs) = if west {
            ([[x, 0., z0], [x, 16. * k, z0], [x, 16. * k, z1], [x, 0., z1]],
             [[u0, uv.2], [u1, uv.2], [u1, uv.3], [u0, uv.3]])
        } else {
            ([[x, 0., z1], [x, 16. * k, z1], [x, 16. * k, z0], [x, 0., z0]],
             [[u1, uv.3], [u0, uv.3], [u0, uv.2], [u1, uv.2]])
        };
        let normal = if west { Vec3::X } else { Vec3::NEG_X };
        append_quad(out, positions, normal.to_array(), uvs,
            crate::interior_models::authored_face_color([1.; 4], normal));
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
    let Some(source) = drawing(map, p, cells, g) else {
        return false;
    };
    if p.indices(g.width).any(|i| claimed[i]) {
        return false;
    }
    for i in p.indices(g.width) {
        append_top(
            &mut mesh.textured,
            g.bounds(i % g.width, i / g.width).into(),
            0.,
            g.uv(p.ground % g.width, p.ground / g.width),
        );
    }
    let object_first = mesh.textured.positions.len();
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let k = g.tile_height / 8.;
    let e = w + p.width as f32 * g.tile_width;
    let front = n + p.height as f32 * g.tile_height;
    // Sample the brown field of tile54, not its black crosshatching.
    // The projected elevator has a separate cap row above this same field.
    if matches!(source, 0x16 | 0x17 | 0x18 | 0x1a) {
        side_wall(&mut mesh.textured, g, p, source, [w, e, n, front]);
    } else {
    let projected = p.height == 3;
    // Tile55's left field is also present beside the source wall-mounted sign.
    let donor_x = if source == 0x1e {
        3
    } else if source == 0x1b {
        2
    } else {
        0
    };
    let uv = paint(g, p.column + donor_x, p.row + usize::from(projected));
    if source == 0x25 {
        let mid = w + 2. * g.tile_width;
        let face = n + 2. * g.tile_height;
        // The extra source row projects the cap across a FLOOR quadrant.
        // Fold the control/door onto the wall and keep that floor strip free.
        wall(
            &mut mesh.textured,
            g,
            p.column,
            p.row + 1,
            [2, 2],
            [w, mid, face - 9. * k, face],
            uv,
        );
        door(
            &mut mesh.textured,
            g,
            p.column + 2,
            p.row + 1,
            [mid, e, n, face],
            uv,
        );
        // The first source row is the canopy's TOP, not a tall front face.
        let first_index = mesh.textured.indices.len();
        block(
            &mut mesh.textured,
            [w, e, 31. * k, 32. * k, face, front],
            uv,
        );
        mesh.textured.indices.drain(first_index..first_index + 6);
        for x in 0..4 {
            append_top(
                &mut mesh.textured,
                [
                    w + x as f32 * g.tile_width,
                    w + (x + 1) as f32 * g.tile_width,
                    face,
                    front,
                ],
                32. * k,
                g.uv(p.column + x, p.row),
            );
        }
    } else if p.kind == ModelKind::WallPanel {
        wall(
            &mut mesh.textured,
            g,
            p.column,
            p.row + usize::from(projected),
            [p.width, 2],
            [w, e, n + 7. * k, n + 16. * k],
            uv,
        );
        if projected {
            // The extra row is a horizontal cap projected over a FLOOR row.
            // Keep the face on the WALL boundary rather than consuming that floor.
            let first_index = mesh.textured.indices.len();
            block(
                &mut mesh.textured,
                [w, e, 31. * k, 32. * k, n + 16. * k, front],
                uv,
            );
            mesh.textured.indices.drain(first_index..first_index + 6);
            for x in 0..p.width {
                append_top(
                    &mut mesh.textured,
                    [
                        w + x as f32 * g.tile_width,
                        w + (x + 1) as f32 * g.tile_width,
                        n + 16. * k,
                        front,
                    ],
                    32. * k,
                    g.uv(p.column + x, p.row),
                );
            }
        }
    } else {
        // The wall donor is the adjacent authenticated pair. Door pixels
        // supply their own frame only when that donor is outside this view.
        let wall_uv = p
            .column
            .checked_sub(2)
            .filter(|&x| {
                let mut left = p.clone();
                left.column = x;
                left.kind = ModelKind::WallPanel;
                drawing(map, &left, cells, g) == Some(4)
            })
            .map_or(uv, |x| paint(g, x, p.row));
        door(
            &mut mesh.textured,
            g,
            p.column,
            p.row,
            [w, e, n, front],
            wall_uv,
        );
    }
    }
    mesh.textured
        .cutaway_ranges
        .push(object_first..mesh.textured.positions.len());
    if matches!(source, 0x16 | 0x17 | 0x18 | 0x1a) {
        let plane = if matches!(source, 0x18 | 0x1a) { [1., 0., e] } else { [-1., 0., -w] };
        mesh.camera_wall_textured_ranges.push((object_first..mesh.textured.positions.len(), plane));
    }
    for i in p.indices(g.width) {
        claimed[i] = true;
        if mesh.authored_cells.len() == cells.len() {
            mesh.authored_cells[i] = Some(p.label());
        }
    }
    true
}

pub(super) fn perimeter(
    mesh: &mut TerrainMeshData,
    frame: &VisualWorldFrame,
    cells: &[&VisualTile],
    g: &GridGeometry,
    placements: &[Placement],
) {
    if !matches!(
        frame.map_id.as_ref(),
        "BattleTowerHallway" | "BattleTower1F"
    ) {
        return;
    }
    let Some(size) = frame.source_map_size_core_tiles else {
        return;
    };
    let Some(p) = placements
        .iter()
        .find(|p| p.kind == ModelKind::WallPanel && drawing(&frame.map_id, p, cells, g) == Some(4))
    else {
        return;
    };
    let k = g.tile_height / 8.;
    let uv = paint(g, p.column, p.row);
    let w = g.origin_x - frame.grid_origin.x as f32 * g.tile_width;
    let e = w + size.x as f32 * 2. * g.tile_width;
    let n = g.origin_z + (2 - frame.grid_origin.y) as f32 * g.tile_height;
    let s = g.origin_z + (size.y as i32 * 2 - frame.grid_origin.y) as f32 * g.tile_height;
    let lobby = frame.map_id.as_ref() == "BattleTower1F";
    if lobby {
        if !frame
            .boundary_warps
            .iter()
            .any(|warp| warp.y == size.y as i32 - 1)
        {
            return;
        }
        // The wall continues behind both counter returns. Its only opening is
        // the authenticated source doorway; changed/custom door drawings must
        // never acquire an inferred backing wall.
        let Some(door) = placements.iter().find(|p| {
            p.kind == ModelKind::GateDoorFrame && drawing(&frame.map_id, p, cells, g) == Some(4)
        }) else {
            return;
        };
        let door_w = g.origin_x + door.column as f32 * g.tile_width;
        let door_e = door_w + door.width as f32 * g.tile_width;
        for [left, right] in [[w, door_w], [door_e, e]] {
            let first = mesh.textured.positions.len();
            block(
                &mut mesh.textured,
                [left, right, 0., 32. * k, n - 9. * k, n - 0.04 * k],
                uv,
            );
            mesh.textured
                .cutaway_ranges
                .push(first..mesh.textured.positions.len());
        }
    }
    let mut ranges = vec![
        ([w - 3. * k, w, 0., 32. * k, n, s + 3. * k], [1., 0., w]),
        ([e, e + 3. * k, 0., 32. * k, n, s + 3. * k], [-1., 0., -e]),
    ];
    let mut south_exits = frame
        .boundary_warps
        .iter()
        .filter(|warp| {
            lobby && warp.y == size.y as i32 - 1 && warp.x >= 0 && warp.x < size.x as i32
        })
        .map(|warp| warp.x)
        .collect::<Vec<_>>();
    south_exits.sort_unstable();
    let mut cursor = w;
    for x in south_exits {
        let left = w + x as f32 * 2. * g.tile_width;
        if left > cursor {
            ranges.push(([cursor, left, 0., 32. * k, s, s + 3. * k], [0., -1., -s]));
        }
        cursor = cursor.max(left + 2. * g.tile_width);
    }
    if cursor < e {
        ranges.push(([cursor, e, 0., 32. * k, s, s + 3. * k], [0., -1., -s]));
    }
    for (bounds, plane) in ranges {
        let first = mesh.textured.positions.len();
        block(&mut mesh.textured, bounds, uv);
        mesh.textured
            .cutaway_ranges
            .push(first..mesh.textured.positions.len());
        mesh.camera_wall_textured_ranges
            .push((first..mesh.textured.positions.len(), plane));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::{IVec2, UVec2};

    fn on_triangle(point: Vec3, a: Vec3, b: Vec3, c: Vec3) -> bool {
        let n = (b - a).cross(c - a);
        let len = n.length();
        if len < 0.0001 || ((point - a).dot(n) / len).abs() > 0.001 {
            return false;
        }
        [(a, b), (b, c), (c, a)]
            .iter()
            .all(|&(u, v)| (v - u).cross(point - u).dot(n) >= -0.001)
    }
    fn clear(mesh: &SurfaceMeshData, point: Vec3) -> bool {
        mesh.indices.chunks_exact(3).all(|t| {
            !on_triangle(
                point,
                Vec3::from_array(mesh.positions[t[0] as usize]),
                Vec3::from_array(mesh.positions[t[1] as usize]),
                Vec3::from_array(mesh.positions[t[2] as usize]),
            )
        })
    }
    #[test]
    fn external_battle_tower_architecture_keeps_real_door_apertures_and_native_wall_pairs() {
        let Some(path) = std::env::var_os("CRYSTAL_RENDER_TEST_PACK") else {
            return;
        };
        let pack = crystal_assets::read_loaded_verified_compiled_game_pack(path).unwrap();
        let layout =
            &pack.pack().runtime_files()["data/tilesets/battle_tower_inside_metatiles.bin"];
        for name in ["BattleTowerHallway", "BattleTower1F"] {
            let room = &pack.pack().data().maps[name];
            let (w, h) = (
                room.attributes.width as usize * 4,
                room.attributes.height as usize * 4,
            );
            let g = GridGeometry {
                width: w,
                height: h,
                tile_width: 8.,
                tile_height: 8.,
                origin_x: -24.,
                origin_z: -16.,
            };
            let tiles = (0..w * h)
                .map(|i| {
                    let (x, y) = (i % w, i / w);
                    let block = room.blocks[y / 4 * (w / 4) + x / 4];
                    VisualTile {
                        column: x as u32,
                        row: y as u32,
                        texture: Default::default(),
                        animation_frames: None,
                        priority: false,
                        source: VisualTileSource {
                            tileset_id: "battle_tower_inside".into(),
                            metatile_id: block,
                            subtile_column: (x % 4) as u8,
                            subtile_row: (y % 4) as u8,
                            tile_index: layout[block as usize * 16 + y % 4 * 4 + x % 4] as u16,
                        },
                    }
                })
                .collect::<Vec<_>>();
            let refs = tiles.iter().collect::<Vec<_>>();
            let placements = resolve(name, &refs, &g, None);
            let native = placements
                .iter()
                .filter(|p| drawing(name, p, &refs, &g).is_some())
                .collect::<Vec<_>>();
            let mut counts = [0; 4];
            for p in &native {
                let source = drawing(name, p, &refs, &g).unwrap();
                counts[if matches!(source, 0x16 | 0x17 | 0x18 | 0x1a) {
                    3
                } else if source == 0x25 {
                    2
                } else if p.kind == ModelKind::WallPanel {
                    0
                } else {
                    1
                }] += 1;
                assert!(!placements.iter().any(|p| matches!(
                    p.kind,
                    ModelKind::GateTerminal | ModelKind::PictureFrame
                )
                    && refs[p.row * w + p.column].source.metatile_id == 4));
                let mut mesh = TerrainMeshData {
                    authored_cells: vec![None; w * h],
                    footing_heights: vec![2.75; w * h],
                    ..Default::default()
                };
                let mut claimed = vec![false; w * h];
                assert!(append_with_tables(
                    &mut mesh,
                    name,
                    &refs,
                    &g,
                    p,
                    &mut claimed,
                    [0, 0],
                    &placements
                ));
                assert_eq!(mesh.footing_heights, vec![2.75; w * h]);
                assert_eq!(claimed.iter().filter(|&&v| v).count(), p.width * p.height);
                assert!(mesh.solid.indices.is_empty());
                assert!(
                    mesh.textured
                        .positions
                        .iter()
                        .all(|v| v[1] >= 0. && v[1] <= 32.)
                );
                if matches!(source, 0x16 | 0x17 | 0x18 | 0x1a) {
                    let west = matches!(source, 0x18 | 0x1a);
                    let gold_uv = mesh.textured.uvs[p.width * p.height * 4];
                    let donor_x = (gold_uv[0] * w as f32).floor() as usize;
                    let donor_y = (gold_uv[1] * h as f32).floor() as usize;
                    assert!(matches!(refs[donor_y * w + donor_x].source.tile_index, 0x2c | 0x2d | 0x2e | 0x2f),
                        "wall bodies must sample a BROWN tile, never the RED corner cap or floor");
                    let left = g.origin_x + p.column as f32 * 8.;
                    let right = left + 16.;
                    let start = g.origin_z + p.row as f32 * 8.;
                    let end = start + p.height as f32 * 8.;
                    let face = if west { right } else { left };
                    for z in [start + 0.5, (start + end) * 0.5, end - 0.5] {
                        assert!(!clear(&mesh.textured, Vec3::new(face, 24., z)),
                            "both side courses must reach full wall height");
                        let floor_x = if west { right + 8. } else { left - 8. };
                        assert!(clear(&mesh.textured, Vec3::new(floor_x, 12., z)),
                            "source FLOOR beside the side course stays clear");
                    }
                    if matches!(source, 0x17 | 0x1a) {
                        assert!(clear(&mesh.textured, Vec3::new((left + right) * 0.5, 12., start - 8.)),
                            "the corner block's upper FLOOR quadrant stays open");
                    }
                    let art = mesh.textured.indices.chunks_exact(3).filter(|t| {
                        t.iter().all(|&i| (mesh.textured.positions[i as usize][0]
                            - (face + if west { 0.02 } else { -0.02 })).abs() < 0.001)
                    }).collect::<Vec<_>>();
                    assert_eq!(art.len(), if matches!(source, 0x17 | 0x1a) { 0 } else { p.height * 2 },
                        "only the ornamental source strip folds onto the inward face");
                    assert_eq!(mesh.camera_wall_textured_ranges.len(), 1,
                        "the course must cut away with its side of the outer shell");
                    for t in art {
                        let [a,b,c] = [t[0],t[1],t[2]].map(|i| Vec3::from_array(mesh.textured.positions[i as usize]));
                        let outward = (b-a).cross(c-a).normalize();
                        assert!((outward.x - if west { 1. } else { -1. }).abs() < 0.001);
                    }
                }
                if p.kind == ModelKind::GateDoorFrame {
                    let left =
                        g.origin_x + (p.column + if source == 0x25 { 2 } else { 0 }) as f32 * 8.;
                    let front = g.origin_z + (p.row + p.height) as f32 * 8.;
                    for x in [left + 4., left + 8., left + 12.] {
                        for y in [1., 8., 16., 23.] {
                            assert!(
                                clear(&mesh.textured, Vec3::new(x, y, front)),
                                "{name}: clear door approach"
                            );
                        }
                    }
                    let warp_y = if source == 0x25 { 1 } else { 0 };
                    // The lobby and five scripted hallway room entries are
                    // authoritative source warps, not guessed model positions.
                    if name == "BattleTowerHallway"
                        && (source == 0x25 || p.column / 2 >= 5 && p.column / 2 <= 15)
                    {
                        let core_x = if source == 0x25 {
                            p.column / 2 + 1
                        } else {
                            p.column / 2
                        };
                        if [5, 7, 9, 11, 13, 15].contains(&core_x) {
                            assert!(
                                room.events
                                    .warps
                                    .iter()
                                    .any(|v| v.x as usize == core_x && v.y == warp_y)
                            );
                        }
                    }
                }
                if source == 0x25 || p.kind == ModelKind::WallPanel && p.height == 3 {
                    let cap_triangles = mesh
                        .textured
                        .indices
                        .chunks_exact(3)
                        .filter(|t| {
                            t.iter().all(|&i| {
                                mesh.textured.positions[i as usize][1] == 32.
                                    && mesh.textured.normals[i as usize][1] > 0.9
                            }) && t.iter().any(|&i| {
                                mesh.textured.uvs[i as usize] != mesh.textured.uvs[t[0] as usize]
                            })
                        })
                        .count();
                    assert_eq!(cap_triangles, 8, "all four source cap cells belong on top");
                    // The lower-left source quadrant is FLOOR. The doorstep
                    // and both neighboring floor tiles must stay body-clear.
                    for dx in [-8., 8., 24., 40.] {
                        for y in [1., 8., 16., 23.] {
                            let point = Vec3::new(
                                g.origin_x + p.column as f32 * 8. + dx,
                                y,
                                g.origin_z + (p.row + 3) as f32 * 8.,
                            );
                            assert!(
                                clear(&mesh.textured, point),
                                "projected cap cannot become a wall across the floor strip"
                            );
                        }
                    }
                }
                for i in p.indices(w) {
                    let mut changed = tiles.clone();
                    changed[i].source.tile_index ^= 0x80;
                    assert_eq!(
                        drawing(name, p, &changed.iter().collect::<Vec<_>>(), &g),
                        None
                    );
                }
                assert_eq!(drawing("UnknownTowerRoom", p, &refs, &g), None);
                let mut owned = vec![true; w * h];
                let mut untouched = TerrainMeshData::default();
                assert!(!append(&mut untouched, name, &refs, &g, p, &mut owned));
                assert!(untouched.textured.indices.is_empty());
            }
            assert_eq!(
                counts,
                if name == "BattleTowerHallway" {
                    [10, 10, 1, 0]
                } else {
                    [8, 1, 0, 6]
                }
            );
            {
                let mut frame = crate::mesh::tests::frame(
                    w as u32,
                    h as u32,
                    tiles.iter().map(|t| t.source.clone()).collect(),
                );
                frame.map_id = name.into();
                frame.grid_origin = IVec2::ZERO;
                frame.source_map_size_core_tiles = Some(UVec2::new(w as u32 / 2, h as u32 / 2));
                frame.boundary_warps = room
                    .events
                    .warps
                    .iter()
                    .map(|warp| IVec2::new(i32::from(warp.x), i32::from(warp.y)))
                    .collect();
                let mut mesh = TerrainMeshData {
                    footing_heights: vec![2.75; w * h],
                    ..Default::default()
                };
                perimeter(&mut mesh, &frame, &refs, &g, &placements);
                assert_eq!(
                    mesh.camera_wall_textured_ranges.len(),
                    if name == "BattleTower1F" { 4 } else { 3 }
                );
                assert_eq!(
                    mesh.textured
                        .cutaway_ranges
                        .iter()
                        .map(|r| r.len())
                        .sum::<usize>(),
                    mesh.textured.positions.len()
                );
                assert!(
                    mesh.textured
                        .positions
                        .iter()
                        .any(|v| v[0] == g.origin_x && v[2] == g.origin_z + 16.)
                );
                assert!(
                    mesh.textured
                        .positions
                        .iter()
                        .any(|v| v[0] == g.origin_x + w as f32 * 8. && v[2] == g.origin_z + 16.)
                );
                assert_eq!(mesh.footing_heights, vec![2.75; w * h]);
                if name == "BattleTower1F" {
                    let door = native
                        .iter()
                        .find(|p| p.kind == ModelKind::GateDoorFrame)
                        .unwrap();
                    for x in 0..w {
                        if (door.column..door.column + door.width).contains(&x) {
                            continue;
                        }
                        assert!(
                            !clear(
                                &mesh.textured,
                                Vec3::new(
                                    g.origin_x + (x as f32 + 0.5) * 8.,
                                    24.,
                                    g.origin_z + 16. - 0.04
                                )
                            ),
                            "the rear wall must connect above and behind both counter returns"
                        );
                    }
                    let mut changed = tiles.clone();
                    changed[door.row * w + door.column].source.tile_index ^= 0x80;
                    let mut untouched = TerrainMeshData::default();
                    perimeter(
                        &mut untouched,
                        &frame,
                        &changed.iter().collect::<Vec<_>>(),
                        &g,
                        &placements,
                    );
                    assert!(
                        untouched.textured.indices.is_empty(),
                        "custom doorway cannot be sealed"
                    );
                }
                if name == "BattleTower1F" {
                    for warp in &room.events.warps {
                        if warp.y as usize != h / 2 - 1 {
                            continue;
                        }
                        assert!(
                            clear(
                                &mesh.textured,
                                Vec3::new(
                                    g.origin_x + (warp.x as f32 + 0.5) * 16.,
                                    16.,
                                    g.origin_z + h as f32 * 8. + 1.5
                                )
                            ),
                            "both actual entrance tiles stay open through the south wall thickness"
                        );
                    }
                }
                for warp in &room.events.warps {
                    assert!(clear(
                        &mesh.textured,
                        Vec3::new(
                            g.origin_x + (warp.x as f32 + 0.5) * 16.,
                            12.,
                            g.origin_z + (warp.y as f32 + 0.5) * 16.
                        )
                    ));
                }
            }
        }
    }
}
