//! Source-authenticated outlines for PokéCom's separated, stepped rooms.
use super::*;
use bevy::prelude::UVec2;

fn cell<'a>(
    cells: &[&'a VisualTile],
    g: &GridGeometry,
    origin: IVec2,
    x: i32,
    y: i32,
) -> Option<&'a VisualTile> {
    let (x, y) = (x - origin.x, y - origin.y);
    (x >= 0 && y >= 0 && (x as usize) < g.width && (y as usize) < g.height)
        .then(|| cells[y as usize * g.width + x as usize])
}
fn topology(frame: &VisualWorldFrame, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    if frame.map_id.as_ref() != crate::pokecom::ADMIN_OFFICE
        || frame.source_map_size_core_tiles != Some(UVec2::new(32, 32))
        || cells.len() != g.width * g.height
    {
        return false;
    }
    let mut observed = 0;
    for (i, t) in cells.iter().enumerate() {
        let (x, y) = (
            frame.grid_origin.x + (i % g.width) as i32,
            frame.grid_origin.y + (i / g.width) as i32,
        );
        if !(0..64).contains(&x) || !(0..64).contains(&y) {
            continue;
        }
        let room = (y < 32 && (x >= 24 || y >= 12)) || (x < 16 && y >= 48);
        let s = &t.source;
        let blank =
            s.tileset_id.as_ref() == "pokecom_center" && s.metatile_id == 27 && s.tile_index == 61;
        if room == blank {
            return false;
        }
        observed += usize::from(!room);
    }
    observed >= 16
}
pub(super) fn append(
    mesh: &mut TerrainMeshData,
    frame: &VisualWorldFrame,
    cells: &[&VisualTile],
    g: &GridGeometry,
) {
    if !topology(frame, cells, g) {
        return;
    }
    let west = g.origin_x - frame.grid_origin.x as f32 * g.tile_width;
    let north = g.origin_z - frame.grid_origin.y as f32 * g.tile_height;
    // Native wall fields differ between the clinic and office atlas courses.
    for (donor_x, donor_y, segments) in [
        (
            6,
            12,
            vec![
                (Edge::West, 0, 14, 32),
                (Edge::West, 24, 4, 14),
                (Edge::East, 64, 4, 32),
                (Edge::South, 32, 0, 12),
                (Edge::South, 32, 16, 64),
            ],
        ),
        (
            0,
            51,
            vec![
                (Edge::West, 0, 51, 64),
                (Edge::East, 16, 51, 64),
                (Edge::South, 64, 4, 16),
            ],
        ),
    ] {
        let Some(donor) = cell(cells, g, frame.grid_origin, donor_x, donor_y) else {
            continue;
        };
        let src = &donor.source;
        if src.tileset_id.as_ref() != "pokecom_center"
            || src.tile_index != 2
            || src.metatile_id != if donor_y == 12 { 4 } else { 48 }
            || src.subtile_column != if donor_y == 12 { 2 } else { 0 }
            || src.subtile_row != if donor_y == 12 { 0 } else { 3 }
        {
            continue;
        }
        let (column, row) = (
            (donor_x - frame.grid_origin.x) as usize,
            (donor_y - frame.grid_origin.y) as usize,
        );
        let p = Placement {
            kind: ModelKind::WallClinical,
            column,
            row,
            width: 1,
            height: 1,
            ground: row * g.width + column,
            depth_pixels: 3.,
            height_pixels: 32.,
            base_pixels: 0.,
            footing_pixels: None,
        };
        let mut template = SurfaceMeshData::default();
        super::super::native_pokecom_clinic_wall::append_backing(
            &mut template,
            g,
            &p,
            row,
            (column, row),
        );
        let b = [
            g.origin_x + column as f32 * g.tile_width,
            g.origin_x + (column + 1) as f32 * g.tile_width,
            g.origin_z + (row + 2) as f32 * g.tile_height - 3. * g.tile_height / 8.,
            g.origin_z + (row + 2) as f32 * g.tile_height,
        ];
        for (edge, at, from, to) in segments {
            let upper_west = matches!(edge, Edge::West) && at == 24;
            let (at, from, to, step, plane) = match edge {
                Edge::North => unreachable!("clinic returns enumerate west, east and south only"),
                Edge::West => (
                    west + at as f32 * g.tile_width,
                    north + from as f32 * g.tile_height,
                    north + to as f32 * g.tile_height,
                    g.tile_height,
                    [1., 0., west + at as f32 * g.tile_width],
                ),
                Edge::East => (
                    west + at as f32 * g.tile_width,
                    north + from as f32 * g.tile_height,
                    north + to as f32 * g.tile_height,
                    g.tile_height,
                    [-1., 0., -west - at as f32 * g.tile_width],
                ),
                Edge::South => (
                    north + at as f32 * g.tile_height,
                    west + from as f32 * g.tile_width,
                    west + to as f32 * g.tile_width,
                    g.tile_width,
                    [0., -1., -north - at as f32 * g.tile_height],
                ),
            };
            // Close outer corner thicknesses. The stepped west return already
            // overlaps the clinic rear course at its south end.
            let from = from
                - if matches!(edge, Edge::West | Edge::East) {
                    3. * g.tile_height / 8.
                } else {
                    0.
                };
            let to = to
                + if matches!(edge, Edge::West | Edge::East) && !upper_west {
                    3. * g.tile_height / 8.
                } else {
                    0.
                };
            let mut start = from;
            while start < to - 0.001 {
                let end = (start + step).min(to);
                let visible = match edge {
                    Edge::North => unreachable!("clinic returns enumerate west, east and south only"),
                    Edge::West | Edge::East => {
                        at >= g.origin_x - 3. * g.tile_width / 8.
                            && at
                                <= g.origin_x
                                    + g.width as f32 * g.tile_width
                                    + 3. * g.tile_width / 8.
                            && end >= g.origin_z
                            && start <= g.origin_z + g.height as f32 * g.tile_height
                    }
                    Edge::South => {
                        at >= g.origin_z - 3. * g.tile_height / 8.
                            && at
                                <= g.origin_z
                                    + g.height as f32 * g.tile_height
                                    + 3. * g.tile_height / 8.
                            && end >= g.origin_x
                            && start <= g.origin_x + g.width as f32 * g.tile_width
                    }
                };
                if visible {
                    let first = mesh.textured.positions.len();
                    copy_section(&mut mesh.textured, &template, b, edge, at, start, end);
                    mesh.camera_wall_textured_ranges
                        .push((first..mesh.textured.positions.len(), plane));
                }
                start = end;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_pokecom_outlines_keep_separated_rooms_and_exit_mats() {
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
        let mut frame = super::super::super::super::tests::frame(
            w as u32,
            h as u32,
            tiles.iter().map(|t| t.source.clone()).collect(),
        );
        frame.map_id = map.into();
        frame.source_map_size_core_tiles = Some(UVec2::new(32, 32));
        let mut mesh = TerrainMeshData {
            footing_heights: vec![2.75; w * h],
            authored_cells: vec![None; w * h],
            ..Default::default()
        };
        append(&mut mesh, &frame, &refs, &g);
        assert_eq!(mesh.camera_wall_textured_ranges.len(), 159);
        assert_eq!(mesh.textured.cutaway_ranges.len(), 159);
        assert_eq!(mesh.footing_heights, vec![2.75; w * h]);
        assert_eq!(mesh.authored_cells, vec![None; w * h]);
        assert!(
            mesh.textured
                .positions
                .iter()
                .flatten()
                .all(|v| v.is_finite())
        );
        assert!(
            mesh.textured
                .normals
                .iter()
                .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 0.001)
        );
        for (range, plane) in &mesh.camera_wall_textured_ranges {
            let points = &mesh.textured.positions[range.clone()];
            assert!(
                points
                    .iter()
                    .all(|v| plane[0] * v[0] + plane[1] * v[2] <= plane[2] + 0.001)
            );
            if plane[0] == 1. {
                let z = points.iter().map(|v| v[2]).fold(f32::INFINITY, f32::min);
                let end = points
                    .iter()
                    .map(|v| v[2])
                    .fold(f32::NEG_INFINITY, f32::max);
                if plane[2] == 0. {
                    assert!((z >= 109. && end <= 259.) || (z >= 405. && end <= 515.));
                } else {
                    assert_eq!(plane[2], 192.);
                    assert!(z >= 29. && end <= 112.);
                }
            }
            if plane[1] == -1. {
                let left = points.iter().map(|v| v[0]).fold(f32::INFINITY, f32::min);
                let right = points
                    .iter()
                    .map(|v| v[0])
                    .fold(f32::NEG_INFINITY, f32::max);
                if plane[2] == -256. {
                    assert!(right <= 96. || left >= 128., "lobby mat remains open");
                } else {
                    assert_eq!(plane[2], -512.);
                    assert!(left >= 32., "office exit remains open");
                }
            }
        }
        let mut changed = tiles.clone();
        changed[0].source.tile_index = 17;
        assert!(!topology(&frame, &changed.iter().collect::<Vec<_>>(), &g));
        let mut wrong = frame.clone();
        wrong.map_id = "PewterPokecenter1F".into();
        assert!(!topology(&wrong, &refs, &g));
        let all = super::super::resolve(map, &refs, &g, None);
        let windows = all
            .iter()
            .filter(|p| super::super::native_pokecom_windows::source(map, p, &refs, &g))
            .collect::<Vec<_>>();
        assert_eq!(windows.len(), 14);
        assert_eq!(
            windows
                .iter()
                .filter(|p| p.height != 3)
                .map(|p| p.width * p.height)
                .sum::<usize>(),
            136
        );
        for p in windows {
            let mut mesh = TerrainMeshData {
                footing_heights: vec![2.75; w * h],
                ..Default::default()
            };
            let mut claimed = vec![false; w * h];
            assert!(super::super::append_with_tables(
                &mut mesh,
                map,
                &refs,
                &g,
                p,
                &mut claimed,
                [0, 0],
                &all
            ));
            assert_eq!(claimed.iter().filter(|&&v| v).count(), p.width * p.height);
            assert_eq!(mesh.footing_heights, vec![2.75; w * h]);
            if p.height == 2 {
                assert!(
                    (2..4).all(|y| (0..4).all(|x| !claimed[(p.row + y) * w + p.column + x])),
                    "machinery overlapping the rear window course keeps its own source cells"
                );
                let mut changed = tiles.clone();
                changed[(p.row + 2) * w + p.column].source.tile_index ^= 1;
                assert!(super::super::native_pokecom_windows::source(
                    map,
                    p,
                    &changed.iter().collect::<Vec<_>>(),
                    &g
                ));
            }
            assert!(mesh.textured.positions.iter().all(|v| v[1] <= 32.));
            assert!(mesh.textured.positions.iter().any(|v| v[1] == 32.));
            let mut expected = SurfaceMeshData::default();
            super::super::native_pokecom_windows::append(&mut expected, &g, p, &refs);
            assert_eq!(
                mesh.textured.positions.len(),
                expected.positions.len(),
                "one full window course, without repeated overlapping copies"
            );
            for i in p.indices(w) {
                let mut changed = tiles.clone();
                changed[i].source.tile_index ^= 0x80;
                let r = changed.iter().collect::<Vec<_>>();
                assert!(!super::super::native_pokecom_windows::source(
                    map, p, &r, &g
                ));
                let mut claimed = vec![false; w * h];
                claimed[i] = true;
                let mut out = Vec::new();
                super::super::native_pokecom_windows::candidates(
                    map,
                    &refs,
                    &g,
                    &mut claimed,
                    &mut out,
                );
                assert!(!out.iter().any(|q| q.column == p.column && q.row == p.row));
            }
        }
    }
}
