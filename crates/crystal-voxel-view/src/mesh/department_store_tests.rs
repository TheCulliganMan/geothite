// Synthetic identities exercise matching independently of the external pack.
// tools/check-department-store.py separately verifies every actual map binding.
fn fixture(
    which: usize,
    origin: [i32; 2],
    size: [usize; 2],
) -> (Vec<VisualTile>, GridGeometry, &'static [Network]) {
    let n = &NETWORKS[which];
    let mut sources = Vec::new();
    for y in 0..n.height {
        for x in 0..n.width {
            sources.push(VisualTileSource {
                tileset_id: Arc::from("mart"),
                metatile_id: 3,
                subtile_column: ((n.anchor[0] as usize + x) % 4) as u8,
                subtile_row: ((n.anchor[1] as usize + y) % 4) as u8,
                tile_index: (32 + x + y * n.width) as u16,
            });
        }
    }
    let nn = Network {
        floor: n.floor,
        anchor: n.anchor,
        width: n.width,
        height: n.height,
        fingerprint: identity_hash(sources.iter()),
        rows: n.rows,
        parts: n.parts,
        label: n.label,
    };
    let [width, height] = size;
    let cells = (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .map(|(x, y)| {
            let px = origin[0] + x as i32 - n.anchor[0];
            let py = origin[1] + y as i32 - n.anchor[1];
            let source = if px >= 0 && py >= 0 && px < n.width as i32 && py < n.height as i32 {
                sources[py as usize * n.width + px as usize].clone()
            } else {
                VisualTileSource {
                    tileset_id: Arc::from("mart"),
                    metatile_id: 4,
                    subtile_column: (x % 4) as u8,
                    subtile_row: (y % 4) as u8,
                    tile_index: 1,
                }
            };
            VisualTile {
                column: x as u32,
                row: y as u32,
                source,
                texture: Default::default(),
                priority: false,
            }
        })
        .collect();
    (
        cells,
        GridGeometry {
            width,
            height,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        },
        Box::leak(vec![nn].into_boxed_slice()),
    )
}
fn resolve_fixture(
    n: &'static [Network],
    cells: &[VisualTile],
    g: &GridGeometry,
    origin: [i32; 2],
    doc: Option<&Document>,
    reserved: &[bool],
) -> Vec<Placement> {
    resolve_networks(
        n,
        n[0].floor,
        "GoldenrodDeptStore5F",
        &cells.iter().collect::<Vec<_>>(),
        g,
        origin,
        doc,
        reserved,
    )
}
#[test]
fn all_seven_exact_drawings_reject_every_changed_source_and_missing_guard() {
    for which in 0..NETWORKS.len() {
        let (cells, g, n) = fixture(which, [0, 0], [33, 17]);
        let x = n[0].anchor[0] as usize;
        let y = n[0].anchor[1] as usize;
        assert!(complete(
            &cells.iter().collect::<Vec<_>>(),
            &g,
            [0, 0],
            x,
            y,
            &n[0]
        ));
        for dy in 0..n[0].height {
            for dx in 0..n[0].width {
                for field in 0..5 {
                    let mut changed = cells.clone();
                    let s = &mut changed[(y + dy) * g.width + x + dx].source;
                    match field {
                        0 => s.tileset_id = Arc::from("custom_mart"),
                        1 => s.metatile_id ^= 1,
                        2 => s.subtile_column ^= 1,
                        3 => s.subtile_row ^= 1,
                        _ => s.tile_index ^= 1,
                    }
                    assert!(
                        !complete(&changed.iter().collect::<Vec<_>>(), &g, [0, 0], x, y, &n[0]),
                        "network {which}, cell {dx},{dy}, field {field}"
                    );
                }
            }
        }
        assert!(!complete(
            &cells[..cells.len() - 1].iter().collect::<Vec<_>>(),
            &g,
            [0, 0],
            x,
            y,
            &n[0]
        ));
        assert!(!complete(
            &cells.iter().collect::<Vec<_>>(),
            &g,
            [1, 0],
            x,
            y,
            &n[0]
        ));
    }
    for invalid in [
        "GoldenrodDeptStoreB1F",
        "GoldenrodDeptStoreRoof",
        "GoldenrodDeptStoreElevator",
        "CeladonDeptStore7F",
        "GoldenrodDeptStore1FBeta",
        "Route1",
    ] {
        assert_eq!(floor(invalid), None);
    }
}
#[test]
fn native_padding_scrolling_and_crops_preserve_exact_sparse_ownership() {
    for which in 0..NETWORKS.len() {
        let n = &NETWORKS[which];
        let expected: usize = n.rows.iter().map(|r| r.count_ones() as usize).sum();
        for (origin, size) in [
            ([0, 0], [33, 17]),
            ([-32, -32], [84, 82]),
            ([-25, -27], [84, 82]),
        ] {
            let (cells, g, n) = fixture(which, origin, size);
            let reserved = vec![false; cells.len()];
            let p = resolve_fixture(n, &cells, &g, origin, None, &reserved);
            assert_eq!(p.len(), 1);
            let actual = p[0].indices(g.width).collect::<Vec<_>>();
            assert_eq!(actual.len(), expected);
            for i in actual {
                let wx = origin[0] + (i % g.width) as i32;
                let wy = origin[1] + (i / g.width) as i32;
                assert!(
                    n[0].rows[(wy - n[0].anchor[1]) as usize] & (1 << (wx - n[0].anchor[0])) != 0
                );
            }
        }
        // Cropping even an unowned source guard row rejects the entire object.
        for (origin, size) in [
            (n.anchor, [n.width, n.height - 1]),
            ([n.anchor[0] + 1, n.anchor[1]], [n.width, n.height]),
        ] {
            let (cells, g, nn) = fixture(which, origin, size);
            assert!(
                resolve_fixture(nn, &cells, &g, origin, None, &vec![false; cells.len()]).is_empty()
            );
        }
    }
}
#[test]
fn every_wall_avoids_lift_doors_stairs_and_front_walkway() {
    for n in &NETWORKS[..6] {
        for y in 0..4 {
            for x in 4..6 {
                assert_eq!(n.rows[y] & (1 << x), 0);
            }
        }
        assert_eq!(n.rows[2], 0);
        assert_eq!(n.rows[3], 0);
        for x in 30..32 {
            assert_eq!(n.rows[0] & (1 << x), 0);
            assert_eq!(n.rows[1] & (1 << x), 0);
        }
        let stairs = if n.floor == 1 {
            None
        } else if n.floor == 6 {
            Some(26)
        } else {
            Some(24)
        };
        if let Some(x) = stairs {
            for y in 0..2 {
                assert_eq!(n.rows[y] & (3 << x), 0);
            }
        }
    }
    let u = &NETWORKS[6];
    assert_eq!(u.rows.iter().map(|r| r.count_ones()).sum::<u32>(), 56);
    for y in 4..8 {
        assert_eq!(u.rows[y], 255, "the entire lower drawing is closed WALL");
    }
    for y in 2..4 {
        assert_eq!(
            u.rows[y] & 0b00111100,
            0,
            "the upper native floor inset stays open"
        );
    }
}
#[test]
fn reserved_cells_custom_profiles_and_missing_ground_keep_whole_object_fallback() {
    let (cells, g, n) = fixture(4, [0, 0], [33, 17]);
    let mut reserved = vec![false; cells.len()];
    reserved[0] = true;
    assert!(resolve_fixture(n, &cells, &g, [0, 0], None, &reserved).is_empty());
    reserved[0] = false;
    let doc:Document=serde_json::from_value(serde_json::json!({"objects":[{"name":"Custom wall panel","tileset":"mart","metatile":3,"origin":[0,0],"tiles":[[32]],"ground":1,"top_pixels":12,"depth_pixels":8}]})).unwrap();
    assert!(resolve_fixture(n, &cells, &g, [0, 0], Some(&doc), &reserved).is_empty());
    let mut missing = cells.clone();
    for c in &mut missing {
        if c.source.metatile_id == 4 {
            c.source.tile_index = 2;
        }
    }
    assert!(resolve_fixture(n, &missing, &g, [0, 0], None, &reserved).is_empty());
}
#[test]
fn append_is_atomic_preserves_footing_and_keeps_all_geometry_inside_owned_cells() {
    for which in 0..NETWORKS.len() {
        let origin = [-25, -27];
        let (cells, mut g, n) = fixture(which, origin, [84, 82]);
        g.tile_width = 12.;
        g.tile_height = 6.;
        let refs = cells.iter().collect::<Vec<_>>();
        let mut claims = vec![false; cells.len()];
        let p = resolve_fixture(n, &cells, &g, origin, None, &claims);
        let mut mesh = TerrainMeshData::default();
        mesh.footing_heights = vec![2.75; cells.len()];
        mesh.authored_cells = vec![None; cells.len()];
        let first = p[0].indices(g.width).next().unwrap();
        claims[first] = true;
        let before = claims.clone();
        assert!(!append(&mut mesh, &refs, &g, &p[0], &mut claims));
        assert_eq!(claims, before);
        assert!(mesh.solid.positions.is_empty());
        assert!(mesh.textured.positions.is_empty());
        claims[first] = false;
        assert!(append(&mut mesh, &refs, &g, &p[0], &mut claims));
        assert!(mesh.footing_heights.iter().all(|&h| h == 2.75));
        assert_eq!(
            mesh.solid.cutaway_ranges,
            vec![0..mesh.solid.positions.len()]
        );
        let (w, _, north, _) = g.bounds(p[0].column, p[0].row);
        assert!(mesh.solid.positions.iter().all(|&[x, y, z]| x >= w
            && x <= w + n[0].width as f32 * g.tile_width
            && z >= north
            && z <= north + n[0].height as f32 * g.tile_height
            && y >= 0.
            && y <= 16. * g.tile_height / 8.));
        assert!(
            mesh.solid
                .normals
                .iter()
                .all(|n| (Vec3::from_array(*n).length_squared() - 1.).abs() < 0.0001)
        );
        assert_eq!(
            claims.iter().filter(|&&v| v).count(),
            n[0].rows
                .iter()
                .map(|r| r.count_ones() as usize)
                .sum::<usize>()
        );
        if which == 6 {
            for &[x, _, z] in &mesh.solid.positions {
                let x = (x - w) / g.tile_width;
                let z = (z - north) / g.tile_height;
                assert!(
                    !(x > 2.0001 && x < 5.9999 && z > 2.0001 && z < 3.9999),
                    "geometry must not enter the genuine floor inset"
                );
            }
        }
    }
}
#[test]
fn live_directory_and_lift_button_use_original_uvs_and_outward_faces() {
    let (cells, g, n) = fixture(0, [0, 0], [33, 17]);
    let mut claimed = vec![false; cells.len()];
    let p = resolve_fixture(n, &cells, &g, [0, 0], None, &claimed);
    let mut mesh = TerrainMeshData::default();
    assert!(append(
        &mut mesh,
        &cells.iter().collect::<Vec<_>>(),
        &g,
        &p[0],
        &mut claimed
    ));
    let base = p[0].indices(g.width).count() * 4;
    assert_eq!(mesh.textured.positions.len(), base + 20);
    let expected = [
        g.uv(6, 0),
        g.uv(28, 0),
        g.uv(29, 0),
        g.uv(28, 1),
        g.uv(29, 1),
    ];
    for (i, (u0, u1, v0, v1)) in expected.into_iter().enumerate() {
        let k = base + i * 4;
        assert_eq!(
            &mesh.textured.uvs[k..k + 4],
            &[[u0, v1], [u1, v1], [u1, v0], [u0, v0]]
        );
        let a = Vec3::from_array(mesh.textured.positions[k]);
        let b = Vec3::from_array(mesh.textured.positions[k + 1]);
        let c = Vec3::from_array(mesh.textured.positions[k + 2]);
        assert!((b - a).cross(c - a).z > 0.);
    }
}
