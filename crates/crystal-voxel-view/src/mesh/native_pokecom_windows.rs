//! Complete office windows and the upper lobby course, backed at ceiling height.
use super::*;
const OFFICE: [u16; 12] = [138, 139, 138, 139, 6, 7, 6, 7, 176, 178, 176, 178];
const UPPER: [u16; 16] = [131, 2, 2, 131, 2, 138, 139, 2, 2, 176, 178, 2, 2, 2, 2, 2];
fn complete(
    cells: &[&VisualTile],
    g: &GridGeometry,
    column: usize,
    row: usize,
    block: u16,
    height: usize,
    art: &[u16],
) -> bool {
    column + 4 <= g.width
        && row + height <= g.height
        && (0..height * 4).all(|j| {
            let s = &cells[(row + j / 4) * g.width + column + j % 4].source;
            s.tileset_id.as_ref() == "pokecom_center"
                && s.metatile_id == block
                && s.subtile_column == (j % 4) as u8
                && s.subtile_row == (j / 4) as u8
                && s.tile_index == art[j]
        })
}
fn upper_donor(cells: &[&VisualTile], g: &GridGeometry) -> Option<usize> {
    cells.iter().enumerate().find_map(|(i, t)| {
        let s = &t.source;
        (s.metatile_id == 23
            && s.subtile_column == 0
            && s.subtile_row == 0
            && complete(cells, g, i % g.width, i / g.width, 23, 4, &UPPER))
        .then_some(i)
    })
}
fn paint(cells: &[&VisualTile]) -> Option<usize> {
    cells.iter().position(|t| {
        let s = &t.source;
        s.tileset_id.as_ref() == "pokecom_center"
            && s.tile_index == 2
            && ((s.metatile_id == 48 && s.subtile_column == 0 && s.subtile_row == 3)
                || (s.metatile_id == 23 && s.subtile_column == 1 && s.subtile_row == 0))
    })
}
pub(super) fn source(map: &str, p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    if map != crate::pokecom::ADMIN_OFFICE
        || p.kind != ModelKind::WindowWall
        || p.width != 4
        || p.column + 4 > g.width
        || p.row + p.height > g.height
        || cells.len() != g.width * g.height
        || p.depth_pixels != 3.
    {
        return false;
    }
    let block = cells[p.row * g.width + p.column].source.metatile_id;
    match block {
        48..=50 => {
            p.height == 3
                && p.height_pixels == 24.
                && p.base_pixels == 8.
                && paint(cells).is_some()
                && complete(cells, g, p.column, p.row, block, 3, &OFFICE)
        }
        23 | 38 | 42 | 39 => {
            p.height == if block == 23 { 4 } else { 2 }
                && p.height_pixels == 32.
                && p.base_pixels == 0.
                && paint(cells).is_some()
                && upper_donor(cells, g).is_some()
                && complete(cells, g, p.column, p.row, block, p.height, &UPPER)
        }
        _ => false,
    }
}
pub(super) fn candidates(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    claimed: &mut [bool],
    out: &mut Vec<Placement>,
) {
    if map != crate::pokecom::ADMIN_OFFICE || g.width < 4 || g.height < 3 {
        return;
    }
    for row in 0..g.height {
        for column in 0..=g.width - 4 {
            let s = &cells[row * g.width + column].source;
            let (height, height_pixels, base_pixels, tile) = match s.metatile_id {
                48..=50 => (3, 24., 8., 0x82),
                23 => (4, 32., 0., 0x11),
                38 | 42 | 39 => (2, 32., 0., 0x11),
                _ => continue,
            };
            if s.subtile_column != 0 || s.subtile_row != 0 {
                continue;
            }
            let Some(ground) = ground_sample(cells, map, "pokecom_center", tile) else {
                continue;
            };
            let p = Placement {
                kind: ModelKind::WindowWall,
                column,
                row,
                width: 4,
                height,
                ground,
                depth_pixels: 3.,
                height_pixels,
                base_pixels,
                footing_pixels: None,
            };
            if !source(map, &p, cells, g) || p.indices(g.width).any(|i| claimed[i]) {
                continue;
            }
            for i in p.indices(g.width) {
                claimed[i] = true;
            }
            out.push(p);
        }
    }
}
pub(super) fn append(
    out: &mut SurfaceMeshData,
    g: &GridGeometry,
    p: &Placement,
    cells: &[&VisualTile],
) {
    let office = p.height == 3;
    let rows = if office { 3 } else { 4 };
    let d = paint(cells).unwrap();
    let upper = if office {
        0
    } else {
        upper_donor(cells, g).unwrap()
    };
    native_pokecom_clinic_wall::append_backing(
        out,
        g,
        p,
        p.row + rows - 2,
        (d % g.width, d / g.width),
    );
    let scale = g.tile_height / 8.;
    let south = g.origin_z + (p.row + rows) as f32 * g.tile_height + 0.125 * scale;
    for y in 0..rows {
        for x in 0..4 {
            let left = g.origin_x + (p.column + x) as f32 * g.tile_width;
            let right = left + g.tile_width;
            let top = (32. - y as f32 * 8.) * scale;
            let bottom = top - 8. * scale;
            // The matching complete course continues behind the machinery. Only
            // the visible upper rows are claimed; machinery pixels remain separate.
            let uv = if y < p.height {
                g.uv(p.column + x, p.row + y)
            } else {
                g.uv(upper % g.width + x, upper / g.width + y)
            };
            append_quad(
                out,
                [
                    [right, bottom, south],
                    [right, top, south],
                    [left, top, south],
                    [left, bottom, south],
                ],
                [0., 0., 1.],
                [[uv.1, uv.3], [uv.1, uv.2], [uv.0, uv.2], [uv.0, uv.3]],
                TEXTURED_SHADE,
            );
        }
    }
}
