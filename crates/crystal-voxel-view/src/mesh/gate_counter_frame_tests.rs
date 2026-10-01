// Included in gate_counters::tests. Mirrors native 84x82 padded VisualWorldFrame
// canvases and their signed, scrolling grid origins without a graphics context.
fn frame_fixture(
    source: &[VisualTile],
    anchor: [i32; 2],
    origin: [i32; 2],
    size: [usize; 2],
) -> (Vec<VisualTile>, GridGeometry) {
    let [width, height] = size;
    let mut cells = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            cells.push(VisualTile {
                column: x as u32,
                row: y as u32,
                texture: Default::default(),
                priority: false,
                source: VisualTileSource {
                    tileset_id: Arc::from("gate"),
                    metatile_id: u16::MAX,
                    subtile_column: (x % 4) as u8,
                    subtile_row: (y % 4) as u8,
                    tile_index: 0,
                },
            });
        }
    }
    for tile in source {
        let x = anchor[0] + tile.column as i32 - origin[0];
        let y = anchor[1] + tile.row as i32 - origin[1];
        if x >= 0 && y >= 0 && (x as usize) < width && (y as usize) < height {
            let dst = &mut cells[y as usize * width + x as usize];
            dst.source = tile.source.clone();
        }
    }
    (
        cells,
        GridGeometry {
            width,
            height,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        },
    )
}
fn resolved_frame(
    networks: &'static [Network],
    cells: &[VisualTile],
    g: &GridGeometry,
    origin: [i32; 2],
    profiles: Option<&Document>,
) -> Vec<Placement> {
    resolve_networks(
        networks,
        "TestGate",
        &cells.iter().collect::<Vec<_>>(),
        g,
        origin,
        profiles,
        &vec![false; cells.len()],
    )
}
#[test]
fn original_map_edges_survive_padding_scrolling_and_complete_viewport_crops() {
    for (anchor, boundary) in [
        ([0, 3], [true, false, false, false]),
        ([16, 3], [false, false, true, false]),
        ([8, 0], [false, true, false, false]),
        ([8, 12], [false, false, false, true]),
    ] {
        let (source, _, mut n) = fixture();
        n.map_size = [20, 16];
        n.boundary = boundary;
        let networks = Box::leak(vec![n].into_boxed_slice());
        let mut expected = Vec::new();
        for y in 2..4 {
            for x in 0..4 {
                expected.push((anchor[0] + x, anchor[1] + y));
            }
        }
        expected.sort_unstable();
        for (origin, size) in [
            ([0, 0], [20, 16]),
            ([-32, -32], [84, 82]),
            ([-25, -27], [84, 82]),
            (anchor, [4, 4]),
        ] {
            let (cells, g) = frame_fixture(&source, anchor, origin, size);
            let placements = resolved_frame(networks, &cells, &g, origin, None);
            assert_eq!(placements.len(), 1, "edge {boundary:?}, origin {origin:?}");
            let mut actual: Vec<_> = placements[0]
                .indices(g.width)
                .map(|i| {
                    (
                        origin[0] + (i % g.width) as i32,
                        origin[1] + (i / g.width) as i32,
                    )
                })
                .collect();
            actual.sort_unstable();
            assert_eq!(actual, expected);
            let mut mesh = TerrainMeshData::default();
            mesh.authored_cells = vec![None; cells.len()];
            mesh.footing_heights = vec![2.75; cells.len()];
            let mut claimed = vec![false; cells.len()];
            assert!(append(
                &mut mesh,
                &cells.iter().collect::<Vec<_>>(),
                &g,
                &placements[0],
                &mut claimed
            ));
            assert_eq!(claimed.iter().filter(|&&c| c).count(), 8);
            assert!(mesh.footing_heights.iter().all(|&h| h == 2.75));
        }
        // A canvas crop that loses even an unowned guard row is incomplete.
        for (origin, size) in [
            ([anchor[0] + 1, anchor[1]], [3, 4]),
            ([anchor[0], anchor[1] + 1], [4, 3]),
        ] {
            let (cells, g) = frame_fixture(&source, anchor, origin, size);
            assert!(resolved_frame(networks, &cells, &g, origin, None).is_empty());
        }
        // Identical artwork at an interior or false viewport edge cannot
        // acquire the missing-halo exemption of a genuine original map edge.
        for (fake_anchor, origin, size) in [
            ([8, 6], [-32, -32], [84, 82]),
            ([8, 6], [8, 6], [4, 4]),
            ([-4, 3], [-4, 3], [4, 4]),
        ] {
            let (cells, g) = frame_fixture(&source, fake_anchor, origin, size);
            assert!(resolved_frame(networks, &cells, &g, origin, None).is_empty());
        }
    }
}
#[test]
fn shifted_native_frames_keep_halo_overlap_and_custom_profile_rejection() {
    let (source, _, mut n) = fixture();
    n.map_size = [20, 16];
    n.boundary = [true, false, false, false];
    let networks = Box::leak(vec![n].into_boxed_slice());
    let anchor = [0, 3];
    let origin = [-29, -31];
    let (cells, g) = frame_fixture(&source, anchor, origin, [84, 82]);
    let placements = resolved_frame(networks, &cells, &g, origin, None);
    assert_eq!(placements.len(), 1);
    let p = &placements[0];
    let index = p.row * g.width + p.column;
    let mut changed = cells.clone();
    changed[index].source.tile_index ^= 1; // unowned halo
    assert!(resolved_frame(networks, &changed, &g, origin, None).is_empty());
    let mut claims = vec![false; cells.len()];
    claims[p.indices(g.width).next().unwrap()] = true;
    let mut mesh = TerrainMeshData::default();
    let original = claims.clone();
    assert!(!append(
        &mut mesh,
        &cells.iter().collect::<Vec<_>>(),
        &g,
        p,
        &mut claims
    ));
    assert_eq!(claims, original);
    assert!(mesh.solid.positions.is_empty());
    let document: Document=serde_json::from_value(serde_json::json!({
        "objects":[{"name":"Custom native-frame counter","tileset":"gate","map":"TestGate",
            "metatile":11,"origin":[0,2],"tiles":[[7]],"ground":1,"top_pixels":1,"depth_pixels":3.0}],
        "atmosphere":null
    })).unwrap();
    assert!(resolved_frame(networks, &cells, &g, origin, Some(&document)).is_empty());
    // Recheck map provenance at append, not only during source reservation.
    let bad = Placement {
        network: p.network,
        column: p.column,
        row: p.row,
        ground: p.ground,
        grid_origin: [origin[0] + 1, origin[1]],
    };
    claims.fill(false);
    assert!(!append(
        &mut mesh,
        &cells.iter().collect::<Vec<_>>(),
        &g,
        &bad,
        &mut claims
    ));
    assert!(claims.iter().all(|&c| !c));
    assert!(mesh.textured.positions.is_empty());
}
#[test]
fn interior_networks_do_not_match_repeated_art_outside_original_map() {
    let (source, _, mut n) = fixture();
    n.map_size = [20, 16];
    let networks = Box::leak(vec![n].into_boxed_slice());
    for anchor in [[-4, 4], [20, 4], [4, -4], [4, 16]] {
        let (cells, g) = frame_fixture(&source, anchor, [-32, -32], [84, 82]);
        assert!(resolved_frame(networks, &cells, &g, [-32, -32], None).is_empty());
    }
}
