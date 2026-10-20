//! The entrance's two halls have separate outlines, not one shared rectangle.
use super::*;

fn topology(
    frame: &VisualWorldFrame,
    cells: &[&VisualTile],
    g: &GridGeometry,
    profiles: Option<&Document>,
) -> bool {
    if frame.map_id.as_ref() != "EcruteakTinTowerEntrance"
        || frame.source_map_size_core_tiles != Some(bevy::math::UVec2::new(20, 18))
        || g.width.checked_mul(g.height) != Some(cells.len())
    {
        return false;
    }
    // Authenticate every cell of the finite layout, including its complete
    // black separator. A cropped region cannot infer an unseen room boundary.
    for y in 0..36 {
        for x in 0..40 {
            let (column, row) = (x - frame.grid_origin.x, y - frame.grid_origin.y);
            if column < 0 || row < 0 || column as usize >= g.width || row as usize >= g.height {
                return false;
            }
            let s = &cells[row as usize * g.width + column as usize].source;
            if s.tileset_id.as_ref() != "tower"
                || i32::from(s.subtile_column) != x % 4
                || i32::from(s.subtile_row) != y % 4
            {
                return false;
            }
            let blank = s.metatile_id == 1 && s.tile_index == 1;
            if blank != (20..28).contains(&x) || (!(20..28).contains(&x) && s.metatile_id == 1) {
                return false;
            }
        }
    }
    if let Some(profiles) = profiles {
        let canonical = built_in_profiles();
        let mut objects = Vec::new();
        for custom in &profiles.objects {
            if canonical.objects.contains(custom) {
                continue;
            }
            objects.push(custom.clone());
            if let Some(old) = canonical.objects.iter().find(|o| o.name == custom.name) {
                objects.push(old.clone());
            }
        }
        let custom = Document {
            objects,
            atmosphere: None,
        };
        if live::resolve(cells, g.width, g.height, &frame.map_id, Some(&custom))
            .iter()
            .any(|p| {
                p.indices(g.width).any(|i| {
                    let x = frame.grid_origin.x + (i % g.width) as i32;
                    let y = frame.grid_origin.y + (i / g.width) as i32;
                    (0..40).contains(&x)
                        && (0..36).contains(&y)
                        && (y < 4 || y >= 32 || x < 2 || (18..30).contains(&x) || x >= 38)
                })
            })
        {
            return false;
        }
    }
    true
}

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    frame: &VisualWorldFrame,
    cells: &[&VisualTile],
    g: &GridGeometry,
    profiles: Option<&Document>,
    solid: &SurfaceMeshData,
    textured: &SurfaceMeshData,
    b: [f32; 4],
    rear: f32,
) {
    if !topology(frame, cells, g, profiles) {
        return;
    }
    let west = g.origin_x - frame.grid_origin.x as f32 * g.tile_width;
    let north = g.origin_z - frame.grid_origin.y as f32 * g.tile_height;
    let south = north + 36. * g.tile_height;
    let thickness = b[3] - b[2];
    for (left, right) in [(0, 20), (28, 40)] {
        let w = west + left as f32 * g.tile_width;
        let e = west + right as f32 * g.tile_width;
        for (edge, at, from, to, step) in [
            (Edge::West, w, rear.max(north), south, g.tile_height * 2.),
            (Edge::East, e, rear.max(north), south, g.tile_height * 2.),
            (
                Edge::South,
                south,
                w - thickness,
                e + thickness,
                g.tile_width * 2.,
            ),
        ] {
            let mut start = from;
            while start < to - 0.001 {
                let end = (start + step).min(to);
                let mut runs = vec![(start, end)];
                if matches!(edge, Edge::South) {
                    for warp in frame
                        .boundary_warps
                        .iter()
                        .filter(|p| p.y == 17 && p.x * 2 >= left && p.x * 2 < right)
                    {
                        let a = west + warp.x as f32 * 2. * g.tile_width;
                        let b = a + 2. * g.tile_width;
                        runs = runs
                            .into_iter()
                            .flat_map(|(s, e)| {
                                if a >= e || b <= s {
                                    vec![(s, e)]
                                } else {
                                    [(s, a.min(e)), (b.max(s), e)]
                                        .into_iter()
                                        .filter(|(a, b)| b - a > 0.001)
                                        .collect()
                                }
                            })
                            .collect();
                    }
                }
                for (a, bound) in runs {
                    append_section(mesh, solid, textured, b, edge, at, a, bound);
                }
                start = end;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (VisualWorldFrame, GridGeometry, Vec<VisualTile>) {
        let g = GridGeometry {
            width: 40,
            height: 36,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let tiles = (0..1440)
            .map(|i| {
                let (x, y) = (i % 40, i / 40);
                let blank = (20..28).contains(&x);
                VisualTile {
                    column: x as u32,
                    row: y as u32,
                    texture: Default::default(),
                    animation_frames: None,
                    priority: false,
                    source: VisualTileSource {
                        tileset_id: "tower".into(),
                        metatile_id: if blank { 1 } else { 9 },
                        tile_index: if blank { 1 } else { 2 },
                        subtile_column: (x % 4) as u8,
                        subtile_row: (y % 4) as u8,
                    },
                }
            })
            .collect();
        let frame = VisualWorldFrame {
            map_id: "EcruteakTinTowerEntrance".into(),
            source_map_size_core_tiles: Some(bevy::math::UVec2::new(20, 18)),
            boundary_warps: vec![
                IVec2::new(4, 17),
                IVec2::new(5, 17),
                IVec2::new(5, 3),
                IVec2::new(17, 15),
                IVec2::new(17, 3),
            ]
            .into(),
            ..Default::default()
        };
        (frame, g, tiles)
    }
    #[test]
    fn separated_halls_close_their_own_edges_without_walling_stairs_or_exits() {
        let (frame, g, tiles) = fixture();
        let refs = tiles.iter().collect::<Vec<_>>();
        assert!(topology(&frame, &refs, &g, None));
        let mut donor = SurfaceMeshData::default();
        super::super::super::append_quad(
            &mut donor,
            [[0., 0., 0.], [0., 32., 0.], [16., 32., 0.], [16., 0., 0.]],
            [0., 0., -1.],
            [[0., 0.]; 4],
            [1.; 4],
        );
        let mut mesh = TerrainMeshData {
            footing_heights: vec![3.; 1440],
            ..Default::default()
        };
        append(
            &mut mesh,
            &frame,
            &refs,
            &g,
            None,
            &donor,
            &SurfaceMeshData::default(),
            [0., 16., 0., 2.],
            0.,
        );
        assert_eq!(mesh.footing_heights, vec![3.; 1440]);
        for at in [0., 160., 224., 320.] {
            let spans = mesh
                .camera_wall_solid_ranges
                .iter()
                .filter(|(_, p)| p[0].abs() == 1. && p[2].abs() == at)
                .map(|(r, _)| {
                    let points = &mesh.solid.positions[r.clone()];
                    (
                        points.iter().map(|p| p[2]).fold(f32::INFINITY, f32::min),
                        points
                            .iter()
                            .map(|p| p[2])
                            .fold(f32::NEG_INFINITY, f32::max),
                    )
                })
                .collect::<Vec<_>>();
            assert!(!spans.is_empty());
            let mut cursor = 0.;
            for (a, b) in spans {
                assert!(a <= cursor + 0.001);
                cursor = cursor.max(b);
            }
            assert_eq!(cursor, 288.);
        }
        for (range, plane) in &mesh.camera_wall_solid_ranges {
            if plane[1] != -1. {
                continue;
            }
            let points = &mesh.solid.positions[range.clone()];
            assert!(
                points.iter().all(|p| p[0] <= 64. || p[0] >= 96.),
                "both external doors remain open"
            );
            assert!(
                points.iter().all(|p| p[0] <= 162. || p[0] >= 222.),
                "black separator stays unbridged"
            );
        }
        assert!(
            mesh.camera_wall_solid_ranges
                .iter()
                .any(|(r, p)| p[1] == -1.
                    && mesh.solid.positions[r.clone()]
                        .iter()
                        .any(|p| p[0] >= 272. && p[0] <= 288.)),
            "the right stair at y15 must not punch a hole in the south wall at y18"
        );
    }
    #[test]
    fn external_entrance_uses_native_timber_and_preserves_every_warp_and_custom_owner() {
        let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
            .map(std::path::PathBuf::from)
            .expect("set CRYSTAL_RENDER_TEST_PACK for this external-pack regression");
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let pack = loaded.pack();
        let map = &pack.data().maps["EcruteakTinTowerEntrance"];
        let art = &pack.runtime_files()["data/tilesets/tower_metatiles.bin"];
        let (mut frame, g, mut tiles) = fixture();
        for (i, tile) in tiles.iter_mut().enumerate() {
            let (x, y) = (i % 40, i / 40);
            let block = map.blocks[y / 4 * 10 + x / 4];
            tile.source.metatile_id = block;
            tile.source.tile_index = art[block as usize * 16 + y % 4 * 4 + x % 4] as u16;
        }
        frame.boundary_warps = map
            .events
            .warps
            .iter()
            .map(|p| IVec2::new(p.x.into(), p.y.into()))
            .collect();
        let refs = tiles.iter().collect::<Vec<_>>();
        assert!(topology(&frame, &refs, &g, None));
        let placements = super::super::super::resolve(&frame.map_id, &refs, &g, None);
        let dungeon = crate::mesh::modeled_dungeons::resolve(&frame.map_id, &refs, &g, None);
        assert!(dungeon.iter().any(|p| p.native_tower_wall()));
        let build = |geometry: &GridGeometry, profiles: Option<&Document>| {
            let mut mesh = TerrainMeshData {
                footing_heights: vec![3.; 1440],
                ..Default::default()
            };
            super::super::append(
                &mut mesh,
                &frame,
                &refs,
                geometry,
                &placements,
                &dungeon,
                None,
                profiles,
            );
            assert_eq!(mesh.footing_heights, vec![3.; 1440]);
            mesh
        };
        let mesh = build(&g, None);
        assert!(
            !mesh.textured.indices.is_empty(),
            "native timber retains its sampled source pigment"
        );
        for at in [0., 160., 224., 320.] {
            let mut spans = Vec::new();
            for (surface, ranges) in [
                (&mesh.solid, &mesh.camera_wall_solid_ranges),
                (&mesh.textured, &mesh.camera_wall_textured_ranges),
            ] {
                for (range, plane) in ranges {
                    if plane[0].abs() != 1. || plane[2].abs() != at {
                        continue;
                    }
                    let points = &surface.positions[range.clone()];
                    spans.push((
                        points.iter().map(|p| p[2]).fold(f32::INFINITY, f32::min),
                        points
                            .iter()
                            .map(|p| p[2])
                            .fold(f32::NEG_INFINITY, f32::max),
                    ));
                    assert!(
                        points.iter().all(|p| p[1] <= 32.001),
                        "native ceiling height stays exact"
                    );
                }
            }
            spans.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mut cursor = spans.first().unwrap().0;
            assert!(cursor <= 16., "each side meets its rear wall");
            for (a, b) in spans {
                assert!(a <= cursor + 0.001, "side at {at} has a gap");
                cursor = cursor.max(b);
            }
            assert_eq!(cursor, 288.);
        }
        let mut shifted = g.clone();
        shifted.origin_x += 112.;
        shifted.origin_z -= 48.;
        let translated = build(&shifted, None);
        for (a, b) in translated
            .textured
            .positions
            .iter()
            .zip(&mesh.textured.positions)
        {
            assert!(
                Vec3::from_array(*a).distance(Vec3::from_array(*b) + Vec3::new(112., 0., -48.))
                    < 0.001
            );
        }
        let custom:Document=serde_json::from_value(serde_json::json!({"objects":[{
            "name":"custom entrance floor at boundary", "tileset":"tower", "maps":["EcruteakTinTowerEntrance"],
            "metatile":9,"origin":[0,0],"tiles":[[2]],"ground":2,"top_pixels":12,"depth_pixels":8}]})).unwrap();
        assert!(!topology(&frame, &refs, &g, Some(&custom)));
        let edited = build(&g, Some(&custom));
        assert!(
            edited.solid.indices.is_empty() && edited.textured.indices.is_empty(),
            "inferred architecture must yield to an edited boundary owner"
        );
    }
    #[test]
    fn changed_separator_foreign_map_or_cropped_source_cannot_infer_halls() {
        let (mut frame, g, mut tiles) = fixture();
        tiles[20].source.tile_index = 2;
        assert!(!topology(
            &frame,
            &tiles.iter().collect::<Vec<_>>(),
            &g,
            None
        ));
        tiles[20].source.tile_index = 1;
        frame.map_id = "TinTower1F".into();
        assert!(!topology(
            &frame,
            &tiles.iter().collect::<Vec<_>>(),
            &g,
            None
        ));
        frame.map_id = "EcruteakTinTowerEntrance".into();
        frame.grid_origin.x = 1;
        assert!(!topology(
            &frame,
            &tiles.iter().collect::<Vec<_>>(),
            &g,
            None
        ));
    }
}
