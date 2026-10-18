//! Cerulean's gray rim is walkable floor; walls stand outside that rim.
use super::*;
use bevy::prelude::UVec2;

fn boundary(x: i32, y: i32) -> Option<(u16, u16)> {
    if !(0..20).contains(&x) || !(0..32).contains(&y) {
        return None;
    }
    if y < 4 {
        return Some((0x2b, if y < 2 { 0x0f } else { 0x0a }));
    }
    if y >= 28 {
        return Some(if (8..12).contains(&x) {
            (
                0x33,
                if y < 30 {
                    5
                } else {
                    [[12, 13, 13, 21], [28, 29, 29, 14]][y as usize - 30][x as usize - 8]
                },
            )
        } else {
            (0x31, 0x0a)
        });
    }
    if x < 2 {
        Some((if y < 8 { 0x37 } else { 0x2e }, 0x0a))
    } else if x >= 18 {
        Some((if y < 8 { 0x38 } else { 0x2f }, 0x0a))
    } else {
        None
    }
}

pub(super) fn source(
    mesh: &TerrainMeshData,
    frame: &VisualWorldFrame,
    cells: &[&VisualTile],
    g: &GridGeometry,
) -> Option<usize> {
    if frame.map_id.as_ref() != "CeruleanGym"
        || frame.source_map_size_core_tiles != Some(UVec2::new(10, 16))
        || cells.len() != g.width * g.height
        || mesh.authored_cells.len() != cells.len()
        || frame.boundary_warps.len() != 2
        || ![IVec2::new(4, 15), IVec2::new(5, 15)]
            .iter()
            .all(|p| frame.boundary_warps.contains(p))
    {
        return None;
    }
    let mut count = 0;
    let mut donor = None;
    for (i, tile) in cells.iter().enumerate() {
        let x = frame.grid_origin.x + (i % g.width) as i32;
        let y = frame.grid_origin.y + (i / g.width) as i32;
        let Some((block, tile_index)) = boundary(x, y) else {
            continue;
        };
        let s = &tile.source;
        if s.tileset_id.as_ref() != "port"
            || s.metatile_id != block
            || s.tile_index != tile_index
            || i32::from(s.subtile_column) != x % 4
            || i32::from(s.subtile_row) != y % 4
            || (matches!(tile_index, 0x0a | 0x0f) && mesh.authored_cells[i].is_some())
        {
            return None;
        }
        count += 1;
        if tile_index == 0x0a {
            donor = Some(i);
        }
    }
    if count >= 16 { donor } else { None }
}

fn panel(g: &GridGeometry, donor: usize) -> SurfaceMeshData {
    let mut out = SurfaceMeshData::default();
    let pixel = g.tile_height / 8.;
    let depth = 3. * pixel;
    let uv = g.uv(donor % g.width, donor / g.width);
    let field = [(uv.0 + uv.1) * 0.5, (uv.2 + uv.3) * 0.5];
    architecture_box(
        &mut out,
        [0., g.tile_width, 0., 32. * pixel, -depth, 0.],
        [1.; 3],
    );
    out.uvs.fill(field);
    for band in 0..4 {
        let bottom = band as f32 * g.tile_height;
        let top = bottom + g.tile_height;
        for (points, normal) in [
            (
                [
                    [0., bottom, 0.001 * pixel],
                    [g.tile_width, bottom, 0.001 * pixel],
                    [g.tile_width, top, 0.001 * pixel],
                    [0., top, 0.001 * pixel],
                ],
                [0., 0., 1.],
            ),
            (
                [
                    [g.tile_width, bottom, -depth - 0.001 * pixel],
                    [0., bottom, -depth - 0.001 * pixel],
                    [0., top, -depth - 0.001 * pixel],
                    [g.tile_width, top, -depth - 0.001 * pixel],
                ],
                [0., 0., -1.],
            ),
        ] {
            append_quad(
                &mut out,
                points,
                normal,
                [[uv.0, uv.3], [uv.1, uv.3], [uv.1, uv.2], [uv.0, uv.2]],
                TEXTURED_SHADE,
            );
        }
    }
    out
}

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    frame: &VisualWorldFrame,
    cells: &[&VisualTile],
    g: &GridGeometry,
) {
    let Some(donor) = source(mesh, frame, cells, g) else {
        return;
    };
    let template = panel(g, donor);
    let depth = 3. * g.tile_height / 8.;
    let w = g.origin_x - frame.grid_origin.x as f32 * g.tile_width;
    let e = w + 20. * g.tile_width;
    let n = g.origin_z + (2 - frame.grid_origin.y) as f32 * g.tile_height;
    let s = g.origin_z + (32 - frame.grid_origin.y) as f32 * g.tile_height;
    // The north two source rows are blank wall padding. The gray rows below
    // them, both side rims and the southern apron retain their original floor.
    for x in 0..20 {
        let first = mesh.textured.positions.len();
        mesh.textured.positions.extend(
            template
                .positions
                .iter()
                .map(|p| [p[0] + w + x as f32 * g.tile_width, p[1], p[2] + n]),
        );
        mesh.textured.normals.extend_from_slice(&template.normals);
        mesh.textured.uvs.extend_from_slice(&template.uvs);
        mesh.textured.colors.extend_from_slice(&template.colors);
        mesh.textured
            .indices
            .extend(template.indices.iter().map(|i| *i + first as u32));
        mesh.textured
            .cutaway_ranges
            .push(first..mesh.textured.positions.len());
        mesh.camera_wall_textured_ranges
            .push((first..mesh.textured.positions.len(), [0., 1., n]));
    }
    let b = [0., g.tile_width, -depth, 0.];
    for (edge, at, from, to, step, plane) in [
        (
            Edge::West,
            w,
            n - depth,
            s + depth,
            g.tile_height,
            [1., 0., w],
        ),
        (
            Edge::East,
            e,
            n - depth,
            s + depth,
            g.tile_height,
            [-1., 0., -e],
        ),
        (
            Edge::South,
            s,
            w - depth,
            w + 8. * g.tile_width,
            g.tile_width,
            [0., -1., -s],
        ),
        (
            Edge::South,
            s,
            w + 12. * g.tile_width,
            e + depth,
            g.tile_width,
            [0., -1., -s],
        ),
    ] {
        let mut start = from;
        while start < to - 0.001 {
            let end = (start + step).min(to);
            let first = mesh.textured.positions.len();
            copy_section(&mut mesh.textured, &template, b, edge, at, start, end);
            mesh.camera_wall_textured_ranges
                .push((first..mesh.textured.positions.len(), plane));
            start = end;
        }
    }
}
