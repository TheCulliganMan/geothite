//! Complete long counter returns, their rear caps, and the right corner control.
use super::*;

fn assembly(
    map: &str,
    column: usize,
    row: usize,
    ground: usize,
    cells: &[&VisualTile],
    g: &GridGeometry,
) -> Option<(Placement, Vec<Placement>)> {
    if map != crate::pokecom::ADMIN_OFFICE
        || g.width == 0
        || column >= g.width
        || row < 1
        || row + 13 > g.height
        || cells.len() != g.width * g.height
    {
        return None;
    }
    let s = &cells.get(row * g.width + column)?.source;
    let (left, height, blocks, header_block, footer_block) = match (s.metatile_id, s.subtile_column)
    {
        (60, 0) => (column, 11, [60, 11, 17], 60, 59),
        (20, 3) => (column.checked_sub(3)?, 10, [20, 18, 19], 20, 16),
        _ => return None,
    };
    if left + 4 > g.width {
        return None;
    }
    let right = height == 10;
    let source = |x: usize, y: usize, block: u16, sx: usize, sy: usize, tile: u16| {
        let s = &cells[y * g.width + x].source;
        s.tileset_id.as_ref() == "pokecom_center"
            && s.metatile_id == block
            && s.subtile_column == sx as u8
            && s.subtile_row == sy as u8
            && s.tile_index == tile
    };
    if !(0..height).all(|y| {
        source(
            column,
            row + y,
            blocks[(y + 1) / 4],
            if right { 3 } else { 0 },
            (y + 1) % 4,
            177,
        )
    }) {
        return None;
    }
    let top = if right {
        [52, 52, 52, 12]
    } else {
        [12, 52, 52, 52]
    };
    if !(0..4).all(|x| source(left + x, row - 1, header_block, x, 0, top[x])) {
        return None;
    }
    if right && !(0..2).all(|x| source(left + 2 + x, row + 10, 19, 2 + x, 3, [3, 37][x])) {
        return None;
    }
    let footer = Placement {
        kind: ModelKind::ReceptionCounter,
        column: left,
        row: row + 11,
        width: 4,
        height: 2,
        ground,
        depth_pixels: 8.,
        height_pixels: 8.,
        base_pixels: 0.,
        footing_pixels: None,
    };
    if cells[footer.row * g.width + left].source.metatile_id != footer_block
        || !native_pokecom_counters::matches(map, &footer, cells, g)
    {
        return None;
    }
    let side = Placement {
        column,
        row,
        width: 1,
        height,
        depth_pixels: 88.,
        ..footer.clone()
    };
    let header = Placement {
        column: left,
        row: row - 1,
        width: 4,
        height: 1,
        ..footer.clone()
    };
    let mut pieces = vec![side, header];
    if right {
        pieces.push(Placement {
            kind: ModelKind::HealingConsole,
            column: left + 2,
            row: row + 10,
            width: 2,
            height: 1,
            depth_pixels: 6.,
            height_pixels: 6.,
            base_pixels: 8.,
            ..footer.clone()
        });
    }
    Some((footer, pieces))
}

fn context(
    map: &str,
    p: &Placement,
    cells: &[&VisualTile],
    g: &GridGeometry,
) -> Option<(Placement, Vec<Placement>)> {
    if p.column >= g.width || p.row >= g.height || cells.len() != g.width * g.height {
        return None;
    }
    let s = &cells[p.row * g.width + p.column].source;
    let (column, row) = match (s.metatile_id, s.subtile_row, p.kind, p.width, p.height) {
        (60 | 20, 1, ModelKind::ReceptionCounter, 1, 10 | 11) => (p.column, p.row),
        (60, 0, ModelKind::ReceptionCounter, 4, 1) => (p.column, p.row + 1),
        (20, 0, ModelKind::ReceptionCounter, 4, 1) => (p.column + 3, p.row + 1),
        (19, 3, ModelKind::HealingConsole, 2, 1) => (p.column + 1, p.row.checked_sub(10)?),
        _ => return None,
    };
    let a = assembly(map, column, row, p.ground, cells, g)?;
    a.1.iter()
        .any(|q| {
            q.column == p.column
                && q.row == p.row
                && q.kind == p.kind
                && q.width == p.width
                && q.height == p.height
                && q.depth_pixels == p.depth_pixels
                && q.height_pixels == p.height_pixels
                && q.base_pixels == p.base_pixels
        })
        .then_some(a)
}
pub(super) fn side(map: &str, p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    p.width == 1 && context(map, p, cells, g).is_some()
}
pub(super) fn header(map: &str, p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    p.width == 4 && context(map, p, cells, g).is_some()
}
pub(super) fn panel(map: &str, p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    p.kind == ModelKind::HealingConsole && context(map, p, cells, g).is_some()
}
pub(super) fn candidates(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    claimed: &mut [bool],
    out: &mut Vec<Placement>,
) {
    if map != crate::pokecom::ADMIN_OFFICE || g.height < 14 {
        return;
    }
    let Some(ground) = ground_sample(cells, map, "pokecom_center", 0x11) else {
        return;
    };
    for row in 1..=g.height - 13 {
        for column in 0..g.width {
            let Some((footer, pieces)) = assembly(map, column, row, ground, cells, g) else {
                continue;
            };
            if !out.iter().any(|p| {
                p.column == footer.column
                    && p.row == footer.row
                    && native_pokecom_counters::matches(map, p, cells, g)
            }) || pieces
                .iter()
                .flat_map(|p| p.indices(g.width))
                .any(|i| claimed[i])
            {
                continue;
            }
            for i in pieces.iter().flat_map(|p| p.indices(g.width)) {
                claimed[i] = true;
            }
            out.extend(pieces);
        }
    }
}
pub(super) fn covered_columns(
    map: &str,
    counter: &Placement,
    cells: &[&VisualTile],
    g: &GridGeometry,
    furnishings: &[Placement],
) -> Option<usize> {
    furnishings
        .iter()
        .filter(|p| p.kind == ModelKind::HealingConsole)
        .find_map(|p| {
            let (footer, _) = context(map, p, cells, g)?;
            (footer.column == counter.column && footer.row == counter.row)
                .then(|| p.column - counter.column)
        })
}
pub(super) fn append_header(
    out: &mut SurfaceMeshData,
    g: &GridGeometry,
    p: &Placement,
    cells: &[&VisualTile],
    [w, e, n, s]: [f32; 4],
) {
    let first = out.positions.len();
    let indices = out.indices.len();
    architecture_box(out, [w, e, 0., g.tile_height, n, s], [1.; 3]);
    let side_column = p.column
        + if cells[p.row * g.width + p.column].source.metatile_id == 20 {
            3
        } else {
            0
        };
    // Red is identical on both long returns; their header's own palette-correct
    // cream source has no red pixel. Use the adjoining strip at the true corner.
    let uv = g.uv(side_column, p.row + 1);
    out.uvs[first..].fill([
        uv.0 + (uv.1 - uv.0) * 1.5 / 8.,
        uv.2 + (uv.3 - uv.2) * 4.5 / 8.,
    ]);
    out.indices.drain(indices..indices + 6);
    for x in 0..4 {
        append_top(
            out,
            [
                w + (e - w) * x as f32 / 4.,
                w + (e - w) * (x + 1) as f32 / 4.,
                n,
                s,
            ],
            g.tile_height,
            g.uv(p.column + x, p.row),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_pokecom_long_returns_close_both_ends_and_keep_support() {
        let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
            .map(std::path::PathBuf::from)
            .unwrap();
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let pack = loaded.pack();
        let map = crate::pokecom::ADMIN_OFFICE;
        let data = &pack.data().maps[map];
        let art = &pack.runtime_files()["data/tilesets/pokecom_center_metatiles.bin"];
        let (w, h) = (
            data.attributes.width as usize * 4,
            data.attributes.height as usize * 4,
        );
        let tiles = (0..w * h)
            .map(|i| {
                let (x, y) = (i % w, i / w);
                let b = data.blocks[y / 4 * (w / 4) + x / 4];
                VisualTile {
                    column: x as u32,
                    row: y as u32,
                    texture: Default::default(),
                    animation_frames: None,
                    priority: false,
                    source: VisualTileSource {
                        tileset_id: "pokecom_center".into(),
                        metatile_id: b,
                        subtile_column: (x % 4) as u8,
                        subtile_row: (y % 4) as u8,
                        tile_index: art[b as usize * 16 + y % 4 * 4 + x % 4] as u16,
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
            origin_x: 0.,
            origin_z: 0.,
        };
        let all = resolve(map, &refs, &g, None);
        let sides = all
            .iter()
            .filter(|p| side(map, p, &refs, &g))
            .collect::<Vec<_>>();
        assert_eq!(sides.len(), 2);
        assert_eq!(all.iter().filter(|p| header(map, p, &refs, &g)).count(), 2);
        assert_eq!(all.iter().filter(|p| panel(map, p, &refs, &g)).count(), 1);
        for side in sides {
            let (footer, pieces) = context(map, side, &refs, &g).unwrap();
            assert_eq!(
                (side.column, side.row),
                if side.height == 11 { (28, 5) } else { (39, 5) }
            );
            let mut mesh = TerrainMeshData {
                footing_heights: vec![2.75; w * h],
                authored_cells: vec![None; w * h],
                ..Default::default()
            };
            let mut claims = vec![false; w * h];
            for p in pieces.iter().chain(std::iter::once(&footer)) {
                assert!(append_with_tables(
                    &mut mesh,
                    map,
                    &refs,
                    &g,
                    p,
                    &mut claims,
                    [0, 0],
                    &all
                ));
            }
            assert_eq!(
                claims.iter().filter(|&&c| c).count(),
                if side.height == 11 { 23 } else { 24 }
            );
            assert_eq!(mesh.footing_heights, vec![2.75; w * h]);
            assert!(
                mesh.textured
                    .positions
                    .iter()
                    .flatten()
                    .all(|v| v.is_finite())
            );
            let top = pieces.iter().find(|p| header(map, p, &refs, &g)).unwrap();
            let side_north = (side.row + 12) as f32 * 8. - side.depth_pixels;
            assert_eq!(
                side_north,
                (top.row + 2) as f32 * 8.,
                "rear cap meets the return without overlap"
            );
            assert_eq!(
                (side.row + 12) as f32 * 8.,
                (footer.row + 1) as f32 * 8.,
                "front course meets the return without overlap"
            );
            let panel_offset = if side.height == 10 { Some(2) } else { None };
            assert_eq!(covered_columns(map, &footer, &refs, &g, &all), panel_offset);
            assert_eq!(covered_columns(map, &footer, &refs, &g, &[]), None);
            let owned = pieces.iter().flat_map(|p| p.indices(w)).collect::<Vec<_>>();
            for &i in &owned {
                let mut reserved = vec![false; w * h];
                reserved[i] = true;
                let mut out = all
                    .iter()
                    .filter(|p| native_pokecom_counters::matches(map, p, &refs, &g))
                    .cloned()
                    .collect::<Vec<_>>();
                candidates(map, &refs, &g, &mut reserved, &mut out);
                assert!(
                    !out.iter()
                        .any(|p| p.column == side.column && p.row == side.row)
                );
                assert!(
                    !out.iter()
                        .any(|p| p.column == top.column && p.row == top.row)
                );
            }
            for i in owned.into_iter().chain(footer.indices(w)) {
                let mut damaged = tiles.clone();
                damaged[i].source.tile_index ^= 0x80;
                let r = damaged.iter().collect::<Vec<_>>();
                for p in &pieces {
                    assert!(
                        context(map, p, &r, &g).is_none(),
                        "changed source rejects the complete assembly"
                    );
                }
            }
            assert!(context("GoldenrodPokecenter1F", side, &refs, &g).is_none());
        }
    }
}
