use std::sync::Arc;

fn fixture() -> (Vec<VisualTile>, GridGeometry) {
    let tiles = (0..32)
        .map(|i| {
            let x = i % 8;
            let y = i / 8;
            let plant = (2..6).contains(&x) && y < 3;
            VisualTile {
                column: x as u32,
                row: y as u32,
                texture: Default::default(),
                priority: false,
                source: VisualTileSource {
                    tileset_id: Arc::from("underground"),
                    metatile_id: if x < 4 { 0x29 } else { 0x2a },
                    subtile_column: (x % 4) as u8,
                    subtile_row: y as u8,
                    tile_index: if plant {
                        [[0x1e, 0x1f], [0x2e, 0x2f], [0x3e, 0x3f]][y][x % 2]
                    } else {
                        0x10
                    },
                },
            }
        })
        .collect();
    (
        tiles,
        GridGeometry {
            width: 8,
            height: 4,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: -19.,
            origin_z: 11.,
        },
    )
}

#[test]
fn rocket_plants_are_two_volumetric_palms_with_exact_native_floor_and_unchanged_support() {
    let (tiles, g) = fixture();
    let cells: Vec<_> = tiles.iter().collect();
    let mut claimed = vec![false; cells.len()];
    let placements = resolve(source::MAP, &cells, &g, [24, 24], None, &claimed);
    assert_eq!(placements.len(), 2);
    let mut mesh = TerrainMeshData::default();
    mesh.authored_cells = vec![None; cells.len()];
    // Arbitrary pre-existing support is never overwritten by this visual pass.
    mesh.footing_heights = (0..cells.len()).map(|i| i as f32 / 7.).collect();
    let support = mesh.footing_heights.clone();
    for p in &placements {
        let before_solid = mesh.solid.positions.len();
        let before_uv = mesh.textured.uvs.len();
        assert!(append(&mut mesh, &cells, &g, p, &mut claimed));
        assert!(mesh.solid.positions.len() > before_solid + 100);
        let ground = p.0.ground(g.width);
        let (u0, u1, v0, v1) = g.uv(ground % g.width, ground / g.width);
        let expected_uv = [[u0, v0], [u0, v1], [u1, v1], [u1, v0]];
        assert_eq!(mesh.textured.uvs.len() - before_uv, 24);
        for quad in mesh.textured.uvs[before_uv..].chunks_exact(4) {
            assert_eq!(quad, expected_uv);
        }
        let (west, _, north, _) = g.bounds(p.0.plant_column(), p.0.row);
        for v in &mesh.solid.positions[before_solid..] {
            assert!(v.iter().all(|x| x.is_finite()));
            assert!((west..=west + 16.).contains(&v[0]));
            assert!((-0.0001..=23.0001).contains(&v[1]));
            assert!((north + 11.9999..=north + 24.0001).contains(&v[2]));
        }
    }
    assert_eq!(mesh.footing_heights, support);
    assert_eq!(claimed.iter().filter(|&&v| v).count(), 12);
    assert_eq!(
        mesh.authored_cells.iter().filter(|v| v.is_some()).count(),
        12
    );
    assert!(mesh.textured.positions.iter().all(|v| v[1] == 0.));
    for i in 0..cells.len() {
        assert_eq!(claimed[i], (2..6).contains(&(i % 8)) && i / 8 < 3);
    }
    let before = (mesh.solid.positions.len(), mesh.textured.positions.len());
    assert!(!append(&mut mesh, &cells, &g, &placements[0], &mut claimed));
    assert_eq!(
        before,
        (mesh.solid.positions.len(), mesh.textured.positions.len())
    );
}

#[test]
fn rocket_plants_reject_every_stale_guard_before_append_and_keep_custom_source_ownership() {
    let (tiles, g) = fixture();
    let cells: Vec<_> = tiles.iter().collect();
    let free = vec![false; cells.len()];
    let placements = resolve(source::MAP, &cells, &g, [24, 24], None, &free);
    for p in &placements {
        for i in p.0.guard_indices(g.width) {
            for mode in 0..5 {
                let mut changed = tiles.clone();
                match mode {
                    0 => changed[i].source.tile_index ^= 1,
                    1 => changed[i].source.metatile_id ^= 1,
                    2 => changed[i].source.subtile_column ^= 1,
                    3 => changed[i].source.subtile_row ^= 1,
                    _ => changed[i].source.tileset_id = Arc::from("custom"),
                }
                let changed_cells: Vec<_> = changed.iter().collect();
                let mut mesh = TerrainMeshData::default();
                let mut claim = free.clone();
                assert!(!append(&mut mesh, &changed_cells, &g, p, &mut claim));
                assert!(mesh.solid.positions.is_empty() && mesh.textured.positions.is_empty());
                assert_eq!(claim, free);
            }
            // Raw source matching protects an incomplete user profile even
            // when its ground sample cannot resolve in the current viewport.
            let s = &tiles[i].source;
            let document: Document = serde_json::from_value(serde_json::json!({"objects":[{
                "name":"Custom plant or backing", "map":source::MAP, "tileset":"underground",
                "metatile":s.metatile_id, "origin":[s.subtile_column,s.subtile_row],
                "tiles":[[s.tile_index]], "ground":65535, "top_pixels":8, "depth_pixels":9
            }]}))
            .unwrap();
            let after = resolve(source::MAP, &cells, &g, [24, 24], Some(&document), &free);
            assert!(!after.iter().any(|q| q.0.column == p.0.column));
            let mut claim = free.clone();
            claim[i] = true;
            let before = claim.clone();
            let mut mesh = TerrainMeshData::default();
            assert!(!append(&mut mesh, &cells, &g, p, &mut claim));
            assert_eq!(claim, before);
            assert!(mesh.solid.positions.is_empty() && mesh.textured.positions.is_empty());
        }
    }
}
