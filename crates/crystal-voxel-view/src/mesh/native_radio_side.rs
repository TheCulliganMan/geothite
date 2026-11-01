//! The pale studio edge is the cap of a standing glass partition.
use super::*;

fn drawing(map: &str) -> Option<(u16, usize, usize)> {
    match map {
        "RadioTower2F" => Some((0x20, 2, 10)),
        "RadioTower5F" => Some((0x3e, 2, 10)),
        "LavRadioTower1F" => Some((0x20, 0, 4)),
        _ => None,
    }
}

pub(super) fn matches(map: &str, p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    let Some((tail, first_row, height)) = drawing(map) else {
        return false;
    };
    p.kind == ModelKind::WindowWall
        && p.width == 1
        && p.height == height
        && p.base_pixels == 0.
        && p.height_pixels == 32.
        && p.depth_pixels == height as f32 * 8.
        && p.column < g.width
        && p.row + height <= g.height
        && cells.len() == g.width * g.height
        && (0..height).all(|y| {
            let row = (first_row + y) % 4;
            let block = if first_row == 2 && y < 2 { 0x3d } else { tail };
            tile(
                cells,
                g,
                p.column,
                p.row + y,
                block,
                3,
                row as u8,
                if row % 2 == 0 { 0x8a } else { 0x8b },
            )
        })
}

fn donor<'a>(
    map: &str,
    p: &Placement,
    cells: &[&VisualTile],
    g: &GridGeometry,
    furnishings: &'a [Placement],
) -> Option<&'a Placement> {
    furnishings
        .iter()
        .filter(|q| panel(map, q, cells, g))
        .min_by_key(|q| q.column.abs_diff(p.column) + q.row.abs_diff(p.row))
}

pub(super) fn missing_context(
    map: &str,
    p: &Placement,
    cells: &[&VisualTile],
    g: &GridGeometry,
    furnishings: &[Placement],
) -> bool {
    matches(map, p, cells, g) && donor(map, p, cells, g, furnishings).is_none()
}

pub(super) fn candidates(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    claimed: &mut [bool],
    out: &mut Vec<Placement>,
    ground: usize,
) {
    let Some((_, _, height)) = drawing(map) else {
        return;
    };
    if height > g.height {
        return;
    }
    for row in 0..=g.height - height {
        for column in 0..g.width {
            let p = Placement {
                kind: ModelKind::WindowWall,
                column,
                row,
                width: 1,
                height,
                ground,
                depth_pixels: height as f32 * 8.,
                height_pixels: 32.,
                base_pixels: 0.,
                footing_pixels: None,
            };
            if !matches(map, &p, cells, g)
                || p.indices(g.width).any(|i| claimed[i])
                || donor(map, &p, cells, g, out).is_none()
            {
                continue;
            }
            for i in p.indices(g.width) {
                claimed[i] = true;
            }
            out.push(p);
        }
    }
}

pub(super) fn replaces_return(map: &str) -> bool {
    // This studio's authored side wall is beyond its doorway. A cropped,
    // altered or user-owned cap must not revive a wall across that doorway.
    drawing(map).is_some_and(|(tail, _, _)| tail == 0x3e)
}

pub(super) fn append(
    out: &mut SurfaceMeshData,
    map: &str,
    p: &Placement,
    cells: &[&VisualTile],
    g: &GridGeometry,
    furnishings: &[Placement],
) {
    let front = donor(map, p, cells, g, furnishings).expect("complete studio glass context");
    let scale = g.tile_height / 8.;
    let (_, east, north, _) = g.bounds(p.column, p.row);
    let west = east - 3. * scale;
    let end = g.origin_z + (p.row + p.height) as f32 * g.tile_height;
    // Lavender's cap starts below the perspective rows of its front pane.
    // Join that verified overlapping pane without claiming intervening floor.
    let joined = front.column <= p.column
        && p.column < front.column + 2
        && front.row + 4 <= p.row
        && p.row - (front.row + 4) <= 2;
    let start = if joined {
        g.origin_z + (front.row + 4) as f32 * g.tile_height
    } else {
        north
    };
    let paints = native_source_swatches(front, g, [[0, 0], [3, 11], [3, 6]]);
    let cap = native_source_swatches(p, g, [[3, 0]])[0];
    let first = out.positions.len();
    architecture_box(out, [west, east, 0., 30. * scale, start, end], [1.; 3]);
    out.uvs[first..].fill(paints[1]);
    super::super::super::radio_glazing::mark(out, first, 0.05);
    let first = out.positions.len();
    architecture_box(
        out,
        [west, east, 30. * scale, 32. * scale, start, end],
        [1.; 3],
    );
    out.uvs[first..].fill(paints[0]);
    let first = out.positions.len();
    architecture_box(
        out,
        [
            west + 0.2 * scale,
            east - 0.2 * scale,
            32. * scale,
            32.01 * scale,
            start,
            end,
        ],
        [1.; 3],
    );
    out.uvs[first..].fill(cap);
    let count = ((end - start) / g.tile_height).round() as usize;
    for segment in 0..count {
        let near = start + segment as f32 * g.tile_height;
        let far = near + g.tile_height;
        for (x, normal) in [(east + 0.01 * scale, 1.), (west - 0.01 * scale, -1.)] {
            for band in 0..3 {
                let uv = g.uv(front.column + segment % 2, front.row + 1);
                let top = (30. - band as f32 * 10.) * scale;
                let mut points = [
                    [x, top - 10. * scale, far],
                    [x, top, far],
                    [x, top, near],
                    [x, top - 10. * scale, near],
                ];
                let mut uvs = [[uv.1, uv.3], [uv.1, uv.2], [uv.0, uv.2], [uv.0, uv.3]];
                if normal > 0. {
                    points.reverse();
                    uvs.reverse();
                }
                let first=out.positions.len();
                append_quad(out, points, [normal, 0., 0.], uvs, TEXTURED_SHADE);
                super::super::super::radio_glazing::mark(out, first, 0.12);
            }
        }
    }
    // Upright mullions preserve individual panes along the complete run.
    for boundary in 0..=count / 2 {
        let z = (start + boundary as f32 * 2. * g.tile_height).min(end);
        let first = out.positions.len();
        architecture_box(
            out,
            [
                west - 0.02 * scale,
                east + 0.02 * scale,
                0.,
                30. * scale,
                (z - 0.3 * scale).max(start),
                (z + 0.3 * scale).min(end),
            ],
            [1.; 3],
        );
        out.uvs[first..].fill(paints[0]);
    }
}
