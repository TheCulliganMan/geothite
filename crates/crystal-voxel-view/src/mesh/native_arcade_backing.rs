//! Rear equipment does not erase the Game Corner's continuous room boundary.
//! Only complete, unchanged north-row cabinet/rack drawings admit backing.
use super::*;

pub(super) fn gaps(
    frame: &VisualWorldFrame,
    cells: &[&VisualTile],
    g: &GridGeometry,
    profiles: Option<&Document>,
) -> Vec<(i32, i32)> {
    if !matches!(
        frame.map_id.as_ref(),
        "CeladonGameCorner" | "GoldenrodGameCorner"
    ) || g.width.checked_mul(g.height) != Some(cells.len())
    {
        return Vec::new();
    }
    let Some(size) = frame.source_map_size_core_tiles else {
        return Vec::new();
    };
    let custom = Document {
        objects: profiles.map_or_else(Vec::new, |document| {
            document
                .objects
                .iter()
                .filter(|object| {
                    !built_in_profiles()
                        .objects
                        .iter()
                        .any(|canonical| canonical == *object)
                })
                .cloned()
                .collect()
        }),
        atmosphere: None,
    };
    let mut reserved = vec![false; cells.len()];
    for placement in live::resolve(cells, g.width, g.height, &frame.map_id, Some(&custom)) {
        for i in placement.indices(g.width) {
            reserved[i] = true;
        }
    }
    let mut spans = Vec::<(i32, i32)>::new();
    for row in 0..g.height {
        if frame.grid_origin.y + row as i32 != 0 || row + 4 > g.height {
            continue;
        }
        for column in 0..g.width.saturating_sub(3) {
            let s = &cells[row * g.width + column].source;
            let x = frame.grid_origin.x + column as i32;
            if s.tileset_id.as_ref() != "game_corner"
                || s.subtile_column != 0
                || s.subtile_row != 0
                || !matches!(s.metatile_id, 0x04 | 0x0e)
                || x < 0
                || x + 4 > size.x as i32 * 2
                || (0..4).any(|y| (0..4).any(|dx| reserved[(row + y) * g.width + column + dx]))
            {
                continue;
            }
            let complete = if s.metatile_id == 0x04 {
                let drawing = casino_drawing(0x04).expect("known complete prize rack");
                (0..4).all(|y| {
                    (0..4).all(|dx| {
                        let a = &cells[(row + y) * g.width + column + dx].source;
                        a.tileset_id == s.tileset_id
                            && a.metatile_id == s.metatile_id
                            && a.subtile_column == dx as u8
                            && a.subtile_row == y as u8
                            && a.tile_index == drawing[y * 4 + dx]
                    })
                })
            } else {
                // Reuse both existing complete native machine guards rather
                // than maintain another copy of the rear-cabinet drawing.
                [0, 2].into_iter().all(|dx| {
                    let p = Placement {
                        kind: ModelKind::ArcadeMachine,
                        column: column + dx,
                        row,
                        width: 2,
                        height: 4,
                        ground: 0,
                        depth_pixels: 10.,
                        height_pixels: 29.,
                        base_pixels: 0.,
                        footing_pixels: None,
                    };
                    native_arcade_source(&frame.map_id, &p, cells, g).is_some()
                })
            };
            if complete {
                spans.push((x, x + 4));
            }
        }
    }
    spans.sort_unstable();
    let mut joined = Vec::<(i32, i32)>::new();
    for (a, b) in spans {
        if let Some(last) = joined.last_mut()
            && a <= last.1
        {
            last.1 = last.1.max(b);
        } else {
            joined.push((a, b));
        }
    }
    joined
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ray_hits(surface: &SurfaceMeshData, point: [f32; 2], depth: [f32; 2]) -> bool {
        surface.indices.chunks_exact(3).any(|t| {
            let [a, b, c] = [t[0], t[1], t[2]].map(|i| surface.positions[i as usize]);
            let u = [b[0] - a[0], b[1] - a[1]];
            let v = [c[0] - a[0], c[1] - a[1]];
            let p = [point[0] - a[0], point[1] - a[1]];
            let det = u[0] * v[1] - u[1] * v[0];
            if det.abs() < 0.000001 {
                return false;
            }
            let s = (p[0] * v[1] - p[1] * v[0]) / det;
            let q = (u[0] * p[1] - u[1] * p[0]) / det;
            let z = a[2] + s * (b[2] - a[2]) + q * (c[2] - a[2]);
            s >= -0.00001 && q >= -0.00001 && s + q <= 1.00001 && z >= depth[0] && z <= depth[1]
        })
    }

    #[test]
    fn native_arcade_backing_requires_whole_north_equipment_and_closes_real_wall_rays() {
        let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../content-packs/text-tui.crystalpack")
            });
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(path).unwrap();
        let pack = loaded.pack();
        let art = &pack.runtime_files()["data/tilesets/game_corner_metatiles.bin"];
        for map in ["CeladonGameCorner", "GoldenrodGameCorner"] {
            let data = &pack.data().maps[map];
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
                        priority: false,
                        animation_frames: None,
                        source: VisualTileSource {
                            tileset_id: "game_corner".into(),
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
            let mut frame = crate::mesh::tests::frame(
                w as u32,
                h as u32,
                tiles.iter().map(|t| t.source.clone()).collect(),
            );
            frame.map_id = map.into();
            frame.source_map_size_core_tiles =
                Some(bevy::prelude::UVec2::new(w as u32 / 2, h as u32 / 2));
            let spans = gaps(&frame, &refs, &g, None);
            assert!(
                !spans.is_empty(),
                "actual rear cabinets/racks require backing"
            );
            let placements = resolve(map, &refs, &g, None);
            let mut mesh = TerrainMeshData {
                footing_heights: vec![2.75; w * h],
                authored_cells: vec![None; w * h],
                ..Default::default()
            };
            super::super::append(&mut mesh, &frame, &refs, &g, &placements, &[], None, None);
            assert_eq!(mesh.footing_heights, vec![2.75; w * h]);
            assert_eq!(mesh.authored_cells, vec![None; w * h]);
            for &(a, b) in &spans {
                for x in a..b {
                    for dx in [0.15, 0.5, 0.85] {
                        for y in [2., 14., 26., 31.] {
                            let point = [(x as f32 + dx) * 8., y];
                            assert!(
                                ray_hits(&mesh.solid, point, [0., 8.1])
                                    || ray_hits(&mesh.textured, point, [0., 8.1]),
                                "{map}: closed backing behind equipment at {point:?}"
                            );
                        }
                    }
                }
            }
            let mut wrong = frame.clone();
            wrong.map_id = "GoldenrodDeptStore6F".into();
            assert!(gaps(&wrong, &refs, &g, None).is_empty());
            // Every cell of each admitted top-row drawing is required, even
            // the lower shelf/cabinet face that is hidden from this camera.
            for column in (0..w)
                .step_by(4)
                .filter(|&x| matches!(tiles[x].source.metatile_id, 0x04 | 0x0e))
            {
                for y in 0..4 {
                    for dx in 0..4 {
                        let mut changed = tiles.clone();
                        changed[y * w + column + dx].source.tile_index ^= 1;
                        let remaining = gaps(&frame, &changed.iter().collect::<Vec<_>>(), &g, None);
                        assert!(
                            !remaining
                                .iter()
                                .any(|&(a, b)| a <= column as i32 && b >= column as i32 + 4)
                        );
                    }
                }
            }
        }
    }
}
