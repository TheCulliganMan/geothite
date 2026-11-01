//! A complete side-on bank has a header and six stool-aligned cabinet rows.
//! The header is the projected top of the first machine, not a seventh pair.
use super::*;

pub(super) const ROWS: usize = 6;
fn tile(dy: usize, dx: usize) -> (u16, u8, u16) {
    let (block, row) = match dy {
        0..=1 => (8, dy + 2),
        2..=5 => (7, dy - 2),
        6..=9 => (7, dy - 6),
        _ => (11, dy - 10),
    };
    let art = match (block, row) {
        (8, 2) => [0x80, 0x81, 0x82, 0x83],
        (8, 3) | (7, 1) | (7, 3) | (11, 1) => [0x90, 0x91, 0x92, 0x93],
        (7, 0) | (7, 2) | (11, 0) => [0xa0, 0xa1, 0xa2, 0xa3],
        (11, 2) => [0xb0, 0xb1, 0xb2, 0xb3],
        (11, 3) => [0xc0, 0xc1, 0xc2, 0xc3],
        _ => unreachable!("complete bank source course"),
    };
    (block, row as u8, art[dx])
}
fn layout(map: &str, p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    reviewed_arcade_room(map)
        && p.kind == ModelKind::ArcadePair
        && p.width == 4
        && p.height == 14
        && p.depth_pixels == 112.
        && p.height_pixels == 32.
        && p.base_pixels == 0.
        && g.width.checked_mul(g.height) == Some(cells.len())
        && p.column.checked_add(4).is_some_and(|v| v <= g.width)
        && p.row.checked_add(14).is_some_and(|v| v <= g.height)
        && p.ground < cells.len()
        && p.indices(g.width).enumerate().all(|(j, i)| {
            let s = &cells[i].source;
            let (block, row, _) = tile(j / 4, j % 4);
            s.tileset_id.as_ref() == "game_corner"
                && s.metatile_id == block
                && s.subtile_column == (j % 4) as u8
                && s.subtile_row == row
        })
}
pub(super) fn source(map: &str, p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    layout(map, p, cells, g)
        && p.indices(g.width)
            .enumerate()
            .all(|(j, i)| cells[i].source.tile_index == tile(j / 4, j % 4).2)
        && cells[p.ground].source.tileset_id.as_ref() == "game_corner"
        && cells[p.ground].source.tile_index == 1
}
pub(super) fn candidates(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    claimed: &mut [bool],
    out: &mut Vec<Placement>,
) {
    if !reviewed_arcade_room(map) || g.width < 4 || g.height < 14 || claimed.len() != cells.len() {
        return;
    }
    let Some(ground) = ground_sample(cells, map, "game_corner", 1) else {
        return;
    };
    for row in 0..=g.height - 14 {
        for column in 0..=g.width - 4 {
            let p = Placement {
                kind: ModelKind::ArcadePair,
                column,
                row,
                width: 4,
                height: 14,
                ground,
                depth_pixels: 112.,
                height_pixels: 32.,
                base_pixels: 0.,
                footing_pixels: None,
            };
            if !layout(map, &p, cells, g) {
                continue;
            }
            let valid = source(map, &p, cells, g) && !p.indices(g.width).any(|i| claimed[i]);
            // A changed source drawing or a custom-owned fragment yields the whole
            // bank; old isolated header/end classifiers cannot create extra slots.
            for i in p.indices(g.width) {
                claimed[i] = true;
            }
            if valid {
                out.push(p);
            }
        }
    }
}
pub(super) fn model_compatible(model: &crate::interior_models::Model) -> bool {
    native_arcade_pairs::append(
        &mut SurfaceMeshData::default(),
        model,
        [0.48, 31.52, 0., 16.],
        [0., 32., 0., 16.],
        32.,
        [[0.5; 2]; 9],
    )
}
pub(super) fn append(
    out: &mut SurfaceMeshData,
    model: &crate::interior_models::Model,
    g: &GridGeometry,
    p: &Placement,
    swatches: [[f32; 2]; 9],
) -> bool {
    let mut bank = SurfaceMeshData::default();
    let (w, e, _, _) = g.bounds(p.column, p.row);
    let e = e + 3. * g.tile_width;
    let inset = g.tile_width * 0.06;
    for row in 0..ROWS {
        let (_, _, n, s) = g.bounds(p.column, p.row + 2 + row * 2);
        let s = s + g.tile_height;
        if !native_arcade_pairs::append(
            &mut bank,
            model,
            [w + inset, e - inset, n, s],
            [w, e, n, s],
            p.height_pixels * g.tile_height / 8.,
            swatches,
        ) {
            return false;
        }
    }
    let first = out.positions.len();
    out.positions.extend(bank.positions);
    out.normals.extend(bank.normals);
    out.uvs.extend(bank.uvs);
    out.colors.extend(bank.colors);
    out.indices
        .extend(bank.indices.into_iter().map(|i| i + first as u32));
    out.cutaway_ranges.extend(
        bank.cutaway_ranges
            .into_iter()
            .map(|r| first + r.start..first + r.end),
    );
    true
}

#[cfg(test)]
#[path = "native_arcade_banks_tests.rs"]
mod tests;
