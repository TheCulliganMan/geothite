//! Continuous native rear wall for the low PokéCom clinic wing.
use super::*;

fn mask(block: u16) -> u16 {
    match block {
        2 => 0x00cc,
        4 => 0x00ef,
        6 => 0x00c0,
        12 => 0x0077,
        7 => 0x003f,
        _ => 0,
    }
}
fn blue(s: &VisualTileSource) -> bool {
    s.tileset_id.as_ref() == "pokecom_center"
        && s.subtile_column < 4
        && s.subtile_row < 2
        && s.tile_index == 2
        && mask(s.metatile_id) & (1 << (s.subtile_row * 4 + s.subtile_column)) != 0
}
pub(super) fn wall(map: &str, p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    if map != crate::pokecom::ADMIN_OFFICE
        || p.kind != ModelKind::WallClinical
        || p.width != 1
        || p.column >= g.width
        || p.row + p.height > g.height
        || cells.len() != g.width * g.height
        || p.depth_pixels != 3.
        || p.height_pixels != 32.
        || p.base_pixels != 0.
    {
        return false;
    }
    let s = &cells[p.row * g.width + p.column].source;
    if !blue(s) {
        return false;
    }
    let bits = (mask(s.metatile_id) >> s.subtile_column) & 0x11;
    let (start, height) = match bits {
        0x11 => (0, 2),
        1 => (0, 1),
        0x10 => (1, 1),
        _ => return false,
    };
    s.subtile_row == start
        && p.height == height
        && p.indices(g.width).enumerate().all(|(y, i)| {
            let next = &cells[i].source;
            blue(next)
                && next.metatile_id == s.metatile_id
                && next.subtile_column == s.subtile_column
                && next.subtile_row == start + y as u8
        })
}
pub(super) fn mounted(map: &str, p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    if map != crate::pokecom::ADMIN_OFFICE
        || p.kind != ModelKind::PictureFrame
        || p.depth_pixels != 0.5
        || p.column + p.width > g.width
        || p.row + p.height > g.height
        || cells.len() != g.width * g.height
    {
        return false;
    }
    let s = &cells[p.row * g.width + p.column].source;
    if p.width == 2
        && p.height == 2
        && p.height_pixels == 16.
        && p.base_pixels == 16.
        && p.column + 2 < g.width
    {
        let donor = &cells[p.row * g.width + p.column + 2].source;
        return blue(donor)
            && donor.metatile_id == 2
            && donor.subtile_column == 2
            && donor.subtile_row == 0
            && p.indices(g.width).enumerate().all(|(j, i)| {
                let s = &cells[i].source;
                s.tileset_id.as_ref() == "pokecom_center"
                    && s.metatile_id == 2
                    && s.subtile_column == (j % 2) as u8
                    && s.subtile_row == (j / 2) as u8
                    && s.tile_index == [34, 35, 50, 51][j]
            });
    }
    p.width == 1
        && p.height == 1
        && p.height_pixels == 8.
        && p.base_pixels == 24.
        && s.tileset_id.as_ref() == "pokecom_center"
        && s.subtile_row == 0
        && s.tile_index == 131
        && p.row + 1 < g.height
        && if s.metatile_id == 6 && matches!(s.subtile_column, 2 | 3) {
            let below = &cells[(p.row + 1) * g.width + p.column].source;
            blue(below)
                && below.metatile_id == 6
                && below.subtile_column == s.subtile_column
                && below.subtile_row == 1
        } else if s.metatile_id == 12 && s.subtile_column == 3 && p.column > 0 {
            let next = &cells[p.row * g.width + p.column - 1].source;
            let below = &cells[(p.row + 1) * g.width + p.column].source;
            blue(next)
                && next.metatile_id == 12
                && next.subtile_column == 2
                && next.subtile_row == 0
                && below.tileset_id.as_ref() == "pokecom_center"
                && below.metatile_id == 12
                && below.subtile_column == 3
                && below.subtile_row == 1
                && below.tile_index == 177
        } else {
            false
        }
}
pub(super) fn candidates(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    claimed: &mut [bool],
    out: &mut Vec<Placement>,
) {
    if map != crate::pokecom::ADMIN_OFFICE {
        return;
    }
    let Some(ground) = ground_sample(cells, map, "pokecom_center", 0x11) else {
        return;
    };
    for row in 0..g.height {
        for column in 0..g.width {
            for (kind, width, height, depth_pixels, height_pixels, base_pixels) in [
                (ModelKind::WallClinical, 1, 2, 3., 32., 0.),
                (ModelKind::WallClinical, 1, 1, 3., 32., 0.),
                (ModelKind::PictureFrame, 2, 2, 0.5, 16., 16.),
                (ModelKind::PictureFrame, 1, 1, 0.5, 8., 24.),
            ] {
                let p = Placement {
                    kind,
                    column,
                    row,
                    width,
                    height,
                    ground,
                    depth_pixels,
                    height_pixels,
                    base_pixels,
                    footing_pixels: None,
                };
                if !(wall(map, &p, cells, g) || mounted(map, &p, cells, g))
                    || p.indices(g.width).any(|i| claimed[i])
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
}
pub(super) fn append_backing(
    out: &mut SurfaceMeshData,
    g: &GridGeometry,
    p: &Placement,
    row: usize,
    donor: (usize, usize),
) {
    let scale = g.tile_height / 8.;
    let (w, e) = (
        g.origin_x + p.column as f32 * g.tile_width,
        g.origin_x + (p.column + p.width) as f32 * g.tile_width,
    );
    let south = g.origin_z + (row + 2) as f32 * g.tile_height;
    let first = out.positions.len();
    let indices = out.indices.len();
    architecture_box(
        out,
        [w, e, 0., 32. * scale, south - 3. * scale, south],
        [1.; 3],
    );
    let uv = g.uv(donor.0, donor.1);
    out.uvs[first..].fill([
        uv.0 + (uv.1 - uv.0) * 2.5 / 8.,
        uv.2 + (uv.3 - uv.2) * 2.5 / 8.,
    ]);
    out.indices.drain(indices + 6..indices + 12);
    for course in 0..4 {
        for x in 0..p.width {
            let low = course as f32 * 8. * scale;
            let high = low + 8. * scale;
            let left = w + x as f32 * g.tile_width;
            let right = left + g.tile_width;
            append_quad(
                out,
                [
                    [right, low, south],
                    [right, high, south],
                    [left, high, south],
                    [left, low, south],
                ],
                [0., 0., 1.],
                [[uv.1, uv.3], [uv.1, uv.2], [uv.0, uv.2], [uv.0, uv.3]],
                TEXTURED_SHADE,
            );
        }
    }
}
pub(super) fn append(
    out: &mut SurfaceMeshData,
    g: &GridGeometry,
    p: &Placement,
    cells: &[&VisualTile],
) {
    let s = &cells[p.row * g.width + p.column].source;
    let row = p.row - s.subtile_row as usize;
    if p.kind == ModelKind::WallClinical {
        append_backing(out, g, p, row, (p.column, p.row));
        return;
    }
    if p.width == 2 {
        append_backing(out, g, p, row, (p.column + 2, p.row));
    } else if s.metatile_id == 12 {
        append_backing(out, g, p, row, (p.column - 1, p.row));
    }
    let scale = g.tile_height / 8.;
    let (w, e) = (
        g.origin_x + p.column as f32 * g.tile_width,
        g.origin_x + (p.column + p.width) as f32 * g.tile_width,
    );
    let south = g.origin_z + (row + 2) as f32 * g.tile_height + 0.5 * scale;
    let bottom = p.base_pixels * scale;
    let top = bottom + p.height_pixels * scale;
    let first = out.positions.len();
    let indices = out.indices.len();
    architecture_box(
        out,
        [w, e, bottom, top, south - 0.5 * scale, south],
        [1.; 3],
    );
    let uv = g.uv(p.column, p.row);
    out.uvs[first..].fill([
        uv.0 + (uv.1 - uv.0) * 0.5 / 8.,
        uv.2 + (uv.3 - uv.2) * 0.5 / 8.,
    ]);
    out.indices.drain(indices + 6..indices + 12);
    for y in 0..p.height {
        for x in 0..p.width {
            let uv = g.uv(p.column + x, p.row + y);
            let left = w + x as f32 * g.tile_width;
            let right = left + g.tile_width;
            let high = top - y as f32 * g.tile_height;
            let low = high - g.tile_height;
            append_quad(
                out,
                [
                    [right, low, south],
                    [right, high, south],
                    [left, high, south],
                    [left, low, south],
                ],
                [0., 0., 1.],
                [[uv.1, uv.3], [uv.1, uv.2], [uv.0, uv.2], [uv.0, uv.3]],
                TEXTURED_SHADE,
            );
        }
    }
}
