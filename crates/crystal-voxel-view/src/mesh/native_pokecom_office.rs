//! Source-owned office counter courses and native plant finishes.
use super::*;
fn red(cells: &[&VisualTile]) -> Option<usize> {
    cells
        .iter()
        .position(|t| t.source.tileset_id.as_ref() == "pokecom_center" && t.source.tile_index == 36)
}

fn placed(
    column: usize,
    row: usize,
    width: usize,
    height: usize,
    ground: usize,
    depth: f32,
) -> Placement {
    Placement {
        kind: ModelKind::ReceptionCounter,
        column,
        row,
        width,
        height,
        ground,
        depth_pixels: depth,
        height_pixels: 8.,
        base_pixels: 0.,
        footing_pixels: None,
    }
}
fn scoped(map: &str, p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    map == crate::pokecom::ADMIN_OFFICE
        && p.kind == ModelKind::ReceptionCounter
        && p.column + p.width <= g.width
        && p.row + p.height <= g.height
        && cells.len() == g.width * g.height
        && p.height_pixels == 8.
        && p.base_pixels == 0.
}
fn header(map: &str, p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    if !scoped(map, p, cells, g)
        || p.width != 4
        || p.height != 1
        || p.depth_pixels != 8.
        || p.row + 1 >= g.height
    {
        return false;
    }
    let block = cells[p.row * g.width + p.column].source.metatile_id;
    let front = cells[(p.row + 1) * g.width + p.column].source.metatile_id;
    let (top, bottom) = match (block, front) {
        (49, 53) => ([16, 12, 52, 52], [132, 133, 40, 41]),
        (50, 54) => ([52; 4], [36; 4]),
        (50, 55) => ([52; 4], [40, 41, 36, 36]),
        _ => return false,
    };
    (0..8).all(|j| {
        let s = &cells[(p.row + j / 4) * g.width + p.column + j % 4].source;
        s.tileset_id.as_ref() == "pokecom_center"
            && s.metatile_id == if j < 4 { block } else { front }
            && s.subtile_column == (j % 4) as u8
            && s.subtile_row == if j < 4 { 3 } else { 0 }
            && s.tile_index == if j < 4 { top[j] } else { bottom[j - 4] }
    }) && red(cells).is_some()
}
fn face_header(
    map: &str,
    p: &Placement,
    cells: &[&VisualTile],
    g: &GridGeometry,
) -> Option<Placement> {
    if !scoped(map, p, cells, g)
        || p.width != 1
        || p.height != 1
        || p.depth_pixels != 1.
        || p.row == 0
    {
        return None;
    }
    let s = &cells[p.row * g.width + p.column].source;
    if s.tile_index != 36 || s.subtile_row != 0 || !matches!(s.metatile_id, 54 | 55) {
        return None;
    }
    let h = placed(
        p.column.checked_sub(s.subtile_column as usize)?,
        p.row - 1,
        4,
        1,
        p.ground,
        8.,
    );
    header(map, &h, cells, g).then_some(h)
}
fn return_source(map: &str, p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    if !scoped(map, p, cells, g)
        || p.width != 2
        || p.height != 3
        || p.depth_pixels != 16.
        || p.row == 0
    {
        return false;
    }
    header(
        map,
        &placed(p.column, p.row - 1, 4, 1, p.ground, 8.),
        cells,
        g,
    ) && p.indices(g.width).enumerate().all(|(j, i)| {
        let s = &cells[i].source;
        s.tileset_id.as_ref() == "pokecom_center"
            && s.metatile_id == 53
            && s.subtile_column == (j % 2) as u8
            && s.subtile_row == (j / 2) as u8
            && s.tile_index == [132, 133, 132, 133, 148, 149][j]
    })
}
fn south(map: &str, p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    scoped(map, p, cells, g)
        && p.width == 4
        && p.height == 2
        && p.depth_pixels == 8.
        && p.indices(g.width).enumerate().all(|(j, i)| {
            let s = &cells[i].source;
            s.tileset_id.as_ref() == "pokecom_center"
                && s.metatile_id == 61
                && s.subtile_column == (j % 4) as u8
                && s.subtile_row == (2 + j / 4) as u8
                && s.tile_index == if j < 4 { 52 } else { 36 }
        })
}
pub(super) fn counter(map: &str, p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    header(map, p, cells, g)
        || face_header(map, p, cells, g).is_some()
        || return_source(map, p, cells, g)
        || south(map, p, cells, g)
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
    let Some(ground) = ground_sample(cells, map, "pokecom_center", 129) else {
        return;
    };
    for row in 0..g.height {
        for column in 0..=g.width - 4 {
            let h = placed(column, row, 4, 1, ground, 8.);
            if header(map, &h, cells, g) {
                let mut pieces = vec![h.clone()];
                for x in 0..4 {
                    let face = placed(column + x, row + 1, 1, 1, ground, 1.);
                    if face_header(map, &face, cells, g).is_some() {
                        pieces.push(face);
                    }
                }
                // The complete station now owns the red face cells beside its
                // monitor. They never emitted a separate counter body; retain
                // the complete header above them and avoid duplicate ownership.
                pieces.retain(|piece| {
                    !out.iter().any(|station| {
                        native_pokecom_equipment::computer(map, station, cells, g)
                            && piece
                                .indices(g.width)
                                .all(|i| station.indices(g.width).any(|j| i == j))
                    })
                });
                if pieces
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
            let s = placed(column, row, 4, 2, ground, 8.);
            if south(map, &s, cells, g) && s.indices(g.width).all(|i| !claimed[i]) {
                for i in s.indices(g.width) {
                    claimed[i] = true;
                }
                out.push(s);
            }
        }
    }
    for row in 1..g.height - 2 {
        for column in 0..=g.width - 2 {
            let p = placed(column, row, 2, 3, ground, 16.);
            if !return_source(map, &p, cells, g)
                || p.indices(g.width).any(|i| claimed[i])
                || !out
                    .iter()
                    .any(|h| h.column == column && h.row + 1 == row && header(map, h, cells, g))
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
// The lower workstation has no separate header row: its complete source
// drawing projects the red backing into the keyboard's first two cells.
// Guard its original floor apron above, then use a complete straight header
// from this same room for the matching countertop/front material.
pub(super) fn station_backing_donor(
    map: &str,
    p: &Placement,
    cells: &[&VisualTile],
    g: &GridGeometry,
) -> Option<Placement> {
    if !native_pokecom_equipment::computer(map, p, cells, g) || p.row == 0 {
        return None;
    }
    if !(0..4).all(|x| {
        let s = &cells[(p.row - 1) * g.width + p.column + x].source;
        s.tileset_id.as_ref() == "pokecom_center"
            && s.metatile_id == 0x37
            && s.subtile_column == x as u8
            && s.subtile_row == 3
            && s.tile_index == 0x82
    }) {
        return None;
    }
    for row in 0..g.height.saturating_sub(1) {
        for column in 0..=g.width - 4 {
            let h = placed(column, row, 4, 1, p.ground, 8.);
            if header(map, &h, cells, g)
                && (0..4).all(|x| cells[row * g.width + column + x].source.tile_index == 0x34)
            {
                return Some(h);
            }
        }
    }
    None
}
pub(super) fn append_station_backing(
    out: &mut SurfaceMeshData,
    map: &str,
    p: &Placement,
    cells: &[&VisualTile],
    g: &GridGeometry,
) {
    let Some(donor) = station_backing_donor(map, p, cells, g) else {
        return;
    };
    let (w, _, n, s) = g.bounds(p.column, p.row);
    append(out, map, &donor, cells, g, [w, w + 4. * g.tile_width, n, s]);
}

pub(super) fn south_row(
    map: &str,
    p: &Placement,
    cells: &[&VisualTile],
    g: &GridGeometry,
) -> Option<usize> {
    if header(map, p, cells, g) {
        Some(p.row + 2)
    } else if face_header(map, p, cells, g).is_some() {
        Some(p.row + 1)
    } else {
        None
    }
}
pub(super) fn append(
    out: &mut SurfaceMeshData,
    map: &str,
    p: &Placement,
    cells: &[&VisualTile],
    g: &GridGeometry,
    [w, e, n, s]: [f32; 4],
) {
    if face_header(map, p, cells, g).is_some() {
        return;
    }
    if south(map, p, cells, g) {
        native_pokecom_counters::append(out, g, p, [w, e, n, s], None);
        return;
    }
    let scale = g.tile_height / 8.;
    let is_return = return_source(map, p, cells, g);
    let (w, e) = if is_return {
        (w + g.tile_width / 8., e - g.tile_width / 8.)
    } else {
        (w, e)
    };
    let red = if is_return {
        (p.row + 2) * g.width + p.column + 1
    } else {
        red(cells).unwrap()
    };
    let uv = g.uv(red % g.width, red / g.width);
    let pigment = [
        uv.0 + (uv.1 - uv.0) * 4.5 / 8.,
        uv.2 + (uv.3 - uv.2) * 4.5 / 8.,
    ];
    let first = out.positions.len();
    let indices = out.indices.len();
    architecture_box(out, [w, e, 0., 8. * scale, n, s], [1.; 3]);
    out.uvs[first..].fill(pigment);
    out.indices.drain(indices..indices + 12);
    if is_return {
        let first = out.positions.len();
        append_top(out, [w, e, n, s], 8. * scale, g.uv(p.column, p.row));
        let uv = g.uv(p.column, p.row);
        out.uvs[first..].fill([
            uv.0 + (uv.1 - uv.0) * 4.5 / 8.,
            uv.2 + (uv.3 - uv.2) * 4.5 / 8.,
        ]);
    }
    let count = p.width;
    for x in 0..count {
        let (left, right) = (
            w + (e - w) * x as f32 / count as f32,
            w + (e - w) * (x + 1) as f32 / count as f32,
        );
        if !is_return {
            append_top(
                out,
                [left, right, n, s],
                8. * scale,
                g.uv(p.column + x, p.row),
            );
        }
        let row = p.row + if is_return { 2 } else { 1 };
        let uv = g.uv(p.column + x, row);
        let texture = if is_return || cells[row * g.width + p.column + x].source.tile_index == 36 {
            [[uv.1, uv.3], [uv.1, uv.2], [uv.0, uv.2], [uv.0, uv.3]]
        } else {
            [pigment; 4]
        };
        append_quad(
            out,
            [
                [right, 0., s],
                [right, 8. * scale, s],
                [left, 8. * scale, s],
                [left, 0., s],
            ],
            [0., 0., 1.],
            texture,
            TEXTURED_SHADE,
        );
    }
}
pub(super) fn plant(map: &str, p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    p.kind == ModelKind::PottedPlant
        && p.width == 2
        && p.height == 3
        && p.base_pixels == 0.
        && p.depth_pixels == 12.
        && p.height_pixels == 23.
        && p.column + 2 <= g.width
        && p.row + 3 <= g.height
        && cells.len() == g.width * g.height
        && p.indices(g.width).enumerate().all(|(j, i)| {
            crate::pokecom::plant_local(map, &cells[i].source)
                == Some(((j % 2) as u8, (j / 2) as u8))
        })
}
pub(super) const PLANT_PAINT: [[usize; 2]; 6] = [[7, 1], [7, 2], [8, 0], [8, 20], [7, 17], [7, 16]];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_pokecom_office_counters_and_plants_keep_complete_source_and_support() {
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
        let counters = all
            .iter()
            .filter(|p| counter(map, p, &refs, &g))
            .collect::<Vec<_>>();
        assert_eq!(counters.len(), 9);
        assert_eq!(
            counters
                .iter()
                .filter(|p| header(map, p, &refs, &g))
                .count(),
            3
        );
        assert_eq!(
            counters
                .iter()
                .filter(|p| face_header(map, p, &refs, &g).is_some())
                .count(),
            2
        );
        assert_eq!(
            counters
                .iter()
                .filter(|p| return_source(map, p, &refs, &g))
                .count(),
            1
        );
        assert_eq!(
            counters.iter().filter(|p| south(map, p, &refs, &g)).count(),
            3
        );
        let mut mesh = TerrainMeshData {
            footing_heights: vec![3.25; w * h],
            authored_cells: vec![None; w * h],
            ..Default::default()
        };
        let mut claims = vec![false; w * h];
        for p in counters {
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
            if return_source(map, p, &refs, &g) {
                let bar = all
                    .iter()
                    .find(|q| {
                        q.column == p.column && q.row + 1 == p.row && header(map, q, &refs, &g)
                    })
                    .unwrap();
                assert_eq!(
                    (bar.row + 2) as f32 * 8.,
                    (p.row + p.height) as f32 * 8. - p.depth_pixels,
                    "the left return joins the north bar without an overlapping front face"
                );
            }
            for i in p.indices(w) {
                let mut changed = tiles.clone();
                changed[i].source.tile_index ^= 0x80;
                assert!(!counter(map, p, &changed.iter().collect::<Vec<_>>(), &g));
                let mut reserved = vec![false; w * h];
                reserved[i] = true;
                let mut out = Vec::new();
                candidates(map, &refs, &g, &mut reserved, &mut out);
                assert!(!out.iter().any(|q| q.column == p.column
                    && q.row == p.row
                    && q.width == p.width
                    && q.height == p.height));
                if let Some(bar) = face_header(map, p, &refs, &g) {
                    assert!(
                        !out.iter().any(|q| q.column == bar.column
                            && q.row == bar.row
                            && header(map, q, &refs, &g)),
                        "reserved front art rejects the whole north course"
                    );
                }
            }
            assert!(!counter("PewterPokecenter1F", p, &refs, &g));
        }
        assert_eq!(claims.iter().filter(|&&v| v).count(), 44);
        let stations = all
            .iter()
            .filter(|p| native_pokecom_equipment::computer(map, p, &refs, &g))
            .collect::<Vec<_>>();
        assert_eq!(stations.len(), 3);
        assert_eq!(
            stations
                .iter()
                .flat_map(|p| p.indices(w))
                .filter(|&i| tiles[i].source.tile_index == 0x24)
                .count(),
            6,
            "four former no-body faces and two previously flat red cells belong to complete stations"
        );
        assert_eq!(mesh.footing_heights, vec![3.25; w * h]);
        assert!(mesh.solid.positions.is_empty());
        assert!(
            mesh.textured
                .positions
                .iter()
                .all(|v| v.iter().all(|c| c.is_finite()) && v[1] >= 0. && v[1] <= 8.)
        );
        let plants = all
            .iter()
            .filter(|p| plant(map, p, &refs, &g))
            .collect::<Vec<_>>();
        assert_eq!(plants.len(), 2);
        let png = image::load_from_memory(&pack.runtime_files()["gfx/tilesets/pokecom_center.png"])
            .unwrap()
            .to_luma8();
        let bank_tiles = png.width() * png.height() / 64 / 2;
        for p in plants {
            let pigments = PLANT_PAINT.map(|[x, y]| {
                let s = &tiles[(p.row + y / 8) * w + p.column + x / 8].source;
                let palette =
                    pack.data().tilesets["pokecom_center"].palette_map[s.tile_index as usize];
                let index =
                    (s.tile_index as u32 & 127) + if palette & 8 != 0 { bank_tiles } else { 0 };
                (
                    png.get_pixel(
                        index % 16 * 8 + (x % 8) as u32,
                        index / 16 * 8 + (y % 8) as u32,
                    )[0],
                    palette & 7,
                )
            });
            assert_eq!(
                pigments,
                [(170, 2), (85, 2), (255, 2), (170, 4), (255, 4), (0, 4)]
            );
            let mut mesh = TerrainMeshData {
                footing_heights: vec![3.25; w * h],
                ..Default::default()
            };
            let mut claims = vec![false; w * h];
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
            assert_eq!(claims.iter().filter(|&&v| v).count(), 6);
            assert_eq!(mesh.footing_heights, vec![3.25; w * h]);
            assert!(mesh.solid.positions.is_empty());
            let paint = native_source_swatches(p, &g, PLANT_PAINT);
            assert!(
                mesh.textured
                    .uvs
                    .iter()
                    .skip(24)
                    .all(|uv| paint.contains(uv))
            );
            assert!(
                mesh.textured
                    .positions
                    .iter()
                    .all(|v| v[1] >= 0. && v[1] <= 23.001)
            );
            for i in p.indices(w) {
                let mut changed = tiles.clone();
                changed[i].source.tile_index ^= 0x80;
                assert!(!plant(map, p, &changed.iter().collect::<Vec<_>>(), &g));
            }
            assert!(!plant("GoldenrodPokecenter1F", p, &refs, &g));
        }
    }
}
