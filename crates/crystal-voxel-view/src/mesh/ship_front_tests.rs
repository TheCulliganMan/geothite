// Synthetic narrow source motifs exercise the real guard and adapter without
// embedding an exported map, atlas, collision catalog or content pack.
fn motif(block: u16, x: usize, y: usize) -> u16 {
    let plain = [0x33, 0x11, 0x10, 0x12][y];
    match block {
        0x0b => {
            if (x + y) % 2 == 0 {
                0x0d
            } else {
                0x1d
            }
        }
        0x17 => {
            if x == 3 {
                [0x06, 0x22, 0x21, 0x31][y]
            } else {
                plain
            }
        }
        0x1c => {
            if x == 0 {
                [0x05, 0x32, 0x20, 0x30][y]
            } else {
                plain
            }
        }
        0x1e => match (x, y) {
            (0, 0) => 0x05,
            (0, _) => 0x15,
            (1, 1) => 0x01,
            (1, 2..=3) => 0x16,
            _ => plain,
        },
        0x1f => match (x, y) {
            (3, 0) => 0x06,
            (3, _) => 0x16,
            (2, 1) => 0x01,
            (2, 2..=3) => 0x15,
            _ => plain,
        },
        0x26 => {
            if y == 2 && x < 2 {
                0x02 + x as u16
            } else {
                plain
            }
        }
        _ => panic!("unexpected test motif"),
    }
}
fn fixture(kind: Kind) -> (Vec<VisualTile>, GridGeometry, [i32; 2]) {
    let anchor = kind.anchor();
    let origin = [anchor[0] - 2, anchor[1] - 2];
    let g = GridGeometry {
        width: 28,
        height: 8,
        tile_width: 8.,
        tile_height: 8.,
        origin_x: -113.,
        origin_z: -19.,
    };
    let mut cells: Vec<_> = (0..g.width * g.height)
        .map(|i| {
            let x = (i % g.width) as i32 + origin[0];
            let y = (i / g.width) as i32 + origin[1];
            VisualTile {
                column: (i % g.width) as u32,
                row: (i / g.width) as u32,
                texture: Default::default(),
                priority: false,
                source: VisualTileSource {
                    tileset_id: Arc::from("lighthouse"),
                    metatile_id: 0x0b,
                    subtile_column: x.rem_euclid(4) as u8,
                    subtile_row: y.rem_euclid(4) as u8,
                    tile_index: if (x + y).rem_euclid(2) == 0 {
                        0x0d
                    } else {
                        0x1d
                    },
                },
            }
        })
        .collect();
    let blocks = if kind == Kind::West {
        [0x1e, 0x17, 0x0b, 0x1c, 0x26, 0x1f]
    } else {
        [0x1e, 0x26, 0x17, 0x0b, 0x1c, 0x1f]
    };
    for y in 0..4 {
        for x in 0..24 {
            let block = blocks[x / 4];
            cells[(y + 2) * g.width + x + 2].source = VisualTileSource {
                tileset_id: Arc::from("lighthouse"),
                metatile_id: block,
                subtile_column: (x % 4) as u8,
                subtile_row: y as u8,
                tile_index: motif(block, x % 4, y),
            };
        }
    }
    (cells, g, origin)
}
fn placements(cells: &[VisualTile], g: &GridGeometry, origin: [i32; 2]) -> Vec<Placement> {
    resolve(
        "FastShipB1F",
        &cells.iter().collect::<Vec<_>>(),
        g,
        origin,
        None,
        &vec![false; cells.len()],
    )
}
#[test]
fn ship_fronts_guard_every_field_including_mouth_void_and_crop_origin() {
    for kind in Kind::ALL {
        let (cells, g, origin) = fixture(kind);
        let out = placements(&cells, &g, origin);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].resolved.kind, kind);
        for i in out[0].resolved.guard_indices(g.width) {
            for field in 0..5 {
                let mut changed = cells.clone();
                let s = &mut changed[i].source;
                match field {
                    0 => s.tileset_id = Arc::from("custom-lighthouse"),
                    1 => s.metatile_id ^= 1,
                    2 => s.subtile_column ^= 1,
                    3 => s.subtile_row ^= 1,
                    _ => s.tile_index ^= 1,
                }
                assert!(
                    placements(&changed, &g, origin).is_empty(),
                    "{kind:?} cell {i} field {field}"
                );
            }
        }
        let refs: Vec<_> = cells.iter().collect();
        let empty = vec![false; cells.len()];
        for map in [
            "FastShip1F",
            "FastShipB1FBeta",
            "FastShipCabins_NNW_NNE_NE",
            "OlivineLighthouse1F",
        ] {
            assert!(resolve(map, &refs, &g, origin, None, &empty).is_empty());
        }
        for delta in [[1, 0], [0, 1], [-1, 0], [0, -1]] {
            assert!(
                placements(&cells, &g, [origin[0] + delta[0], origin[1] + delta[1]]).is_empty()
            );
        }
        // Copy just the left 23 columns of the otherwise exact drawing. The
        // changed viewport anchor must not authorize the surviving fragment.
        let clipped: Vec<_> = (0..4)
            .flat_map(|y| (0..23).map(move |x| (x, y)))
            .map(|(x, y)| cells[(y + 2) * g.width + x + 2].clone())
            .collect();
        let crop = GridGeometry {
            width: 23,
            height: 4,
            ..g
        };
        assert!(placements(&clipped, &crop, kind.anchor()).is_empty());
    }
}
#[test]
fn ship_fronts_yield_whole_assembly_to_custom_mouth_void_or_wall() {
    for kind in Kind::ALL {
        let (cells, g, origin) = fixture(kind);
        let refs: Vec<_> = cells.iter().collect();
        let empty = vec![false; cells.len()];
        // Every reserved guard cell must block, including the 18 unclaimed
        // opening/void cells. Reserved does not mean "raise this cell".
        for i in placements(&cells, &g, origin)[0]
            .resolved
            .guard_indices(g.width)
        {
            let mut reserved = empty.clone();
            reserved[i] = true;
            assert!(resolve("FastShipB1F", &refs, &g, origin, None, &reserved).is_empty());
        }
        for (x, y) in [(0, 0), (1, 1), (22, 1), (kind.entrance(), 2)] {
            let s = &cells[(y + 2) * g.width + x + 2].source;
            let custom: Document = serde_json::from_value(serde_json::json!({"objects":[{
                "name":"Custom front detail", "map":"FastShipB1F", "tileset":"lighthouse",
                "metatile":s.metatile_id, "origin":[s.subtile_column,s.subtile_row],
                "tiles":[[s.tile_index]], "ground":13,"top_pixels":0,"depth_pixels":8
            }]}))
            .unwrap();
            assert!(
                resolve("FastShipB1F", &refs, &g, origin, Some(&custom), &empty).is_empty(),
                "custom {x},{y}"
            );
        }
    }
}
#[test]
fn ship_fronts_append_is_atomic_and_rechecks_every_guard_cell() {
    for kind in Kind::ALL {
        let (mut cells, g, origin) = fixture(kind);
        let out = placements(&cells, &g, origin);
        let p = &out[0];
        for i in p.resolved.guard_indices(g.width) {
            cells[i].source.tile_index ^= 1;
            let mut mesh = TerrainMeshData::default();
            mesh.footing_heights = vec![3.25; cells.len()];
            mesh.authored_cells = vec![None; cells.len()];
            let before = mesh.clone();
            let mut claimed = vec![false; cells.len()];
            assert!(!append(
                &mut mesh,
                &cells.iter().collect::<Vec<_>>(),
                &g,
                p,
                &mut claimed
            ));
            assert_eq!(mesh, before);
            assert!(claimed.iter().all(|v| !*v));
            cells[i].source.tile_index ^= 1;
        }
        for i in p.resolved.guard_indices(g.width) {
            let mut mesh = TerrainMeshData::default();
            let before = mesh.clone();
            let mut claimed = vec![false; cells.len()];
            claimed[i] = true;
            let old = claimed.clone();
            assert!(!append(
                &mut mesh,
                &cells.iter().collect::<Vec<_>>(),
                &g,
                p,
                &mut claimed
            ));
            assert_eq!(mesh, before);
            assert_eq!(claimed, old);
        }
    }
}
fn inside(kind: Kind, x: f32, z: f32) -> bool {
    let mouth = kind.entrance() as f32 * 8.;
    let eps = 0.002;
    x >= -eps
        && x <= 192. + eps
        && z >= -eps
        && z <= 32. + eps
        && !(x > mouth + eps && x < mouth + 32. - eps)
        && !(z > 8. + eps
            && z < 16. - eps
            && ((x > 8. + eps && x < 16. - eps) || (x > 176. + eps && x < 184. - eps)))
}
#[test]
fn ship_fronts_preserve_sparse_openings_footing_opaque_floor_and_closed_reverse_faces() {
    for kind in Kind::ALL {
        let (cells, g, origin) = fixture(kind);
        let p = placements(&cells, &g, origin).remove(0);
        let mut mesh = TerrainMeshData::default();
        mesh.footing_heights = vec![3.25; cells.len()];
        mesh.authored_cells = vec![None; cells.len()];
        let support = mesh.footing_heights.clone();
        let mut claimed = vec![false; cells.len()];
        let refs: Vec<_> = cells.iter().collect();
        assert!(append(&mut mesh, &refs, &g, &p, &mut claimed));
        assert_eq!(mesh.footing_heights, support);
        let (w, _, n, _) = g.bounds(p.resolved.column, p.resolved.row);
        assert_eq!(claimed.iter().filter(|&&v| v).count(), 78);
        for y in 0..4 {
            for x in 0..24 {
                assert_eq!(claimed[(y + 2) * g.width + x + 2], kind.owns(x, y));
                assert_eq!(
                    mesh.authored_cells[(y + 2) * g.width + x + 2],
                    kind.owns(x, y).then_some(kind.label())
                );
            }
        }
        assert_eq!(mesh.solid.cutaway_ranges.len(), 1);
        let walls = mesh.solid.cutaway_ranges[0].clone();
        let highest = mesh
            .solid
            .positions
            .iter()
            .map(|p| p[1])
            .fold(f32::NEG_INFINITY, f32::max);
        assert!((highest - crate::ship::B1F_VISUAL_WALL_HEIGHT).abs() < 0.001);

        assert!(walls.start > 0);
        assert!(
            mesh.solid.positions[..walls.start]
                .iter()
                .all(|v| v[1] == 0.)
        );
        assert_eq!(walls.end, mesh.solid.positions.len());
        assert!(mesh.textured.positions.is_empty());
        assert!(
            mesh.solid
                .positions
                .iter()
                .all(|v| inside(kind, v[0] - w, v[2] - n))
        );
        assert!(
            mesh.solid.normals[walls.clone()]
                .iter()
                .any(|v| v[2] < -0.99)
        );
        assert!(
            mesh.solid.normals[walls.clone()]
                .iter()
                .any(|v| v[2] > 0.99)
        );
        assert!(
            mesh.solid.normals[walls.clone()]
                .iter()
                .any(|v| v[0] < -0.99)
        );
        assert!(mesh.solid.normals[walls].iter().any(|v| v[0] > 0.99));
        let before = mesh.clone();
        let old = claimed.clone();
        assert!(!append(&mut mesh, &refs, &g, &p, &mut claimed));
        assert_eq!(mesh, before);
        assert_eq!(claimed, old);
    }
}
#[test]
fn ship_fronts_reuse_cached_faces_with_round_portholes_at_shared_wall_height() {
    for kind in Kind::ALL {
        let (cells, g, origin) = fixture(kind);
        let p = placements(&cells, &g, origin).remove(0);
        let refs: Vec<_> = cells.iter().collect();
        let mut mesh = TerrainMeshData::default();
        assert!(append(
            &mut mesh,
            &refs,
            &g,
            &p,
            &mut vec![false; cells.len()]
        ));
        let (w, _, n, _) = g.bounds(p.resolved.column, p.resolved.row);
        let mut expected = SurfaceMeshData::default();
        let port = if kind == Kind::West { 16 } else { 4 };
        let mouth = kind.entrance();
        for (a, b) in [(2, mouth), (mouth + 4, 22)] {
            let mut x = a;
            while x < b {
                let size = if x == port { 2 } else { 1 };
                let bounds = [
                    w + x as f32 * 8.,
                    w + (x + size) as f32 * 8.,
                    n + 29.,
                    n + 32.,
                ];
                if x == port {
                    crate::dungeon_models::model(crate::dungeon_models::Kind::PortholeBulkhead)
                        .append_porthole(
                            &mut expected,
                            bounds,
                            0.,
                            16.,
                            crate::ship::B1F_VISUAL_WALL_HEIGHT,
                        );
                } else {
                    crate::dungeon_models::extension_model(
                        crate::dungeon_models::ExtensionKind::ShipBulkhead,
                    )
                    .append(
                        &mut expected,
                        bounds,
                        0.,
                        crate::ship::B1F_VISUAL_WALL_HEIGHT,
                    );
                }
                x += size;
            }
        }
        let first = mesh.solid.positions.len() - expected.positions.len();
        assert_eq!(
            &mesh.solid.positions[first..],
            expected.positions.as_slice()
        );
        assert_eq!(&mesh.solid.normals[first..], expected.normals.as_slice());
        assert_eq!(&mesh.solid.colors[first..], expected.colors.as_slice());
        let indices = &mesh.solid.indices[mesh.solid.indices.len() - expected.indices.len()..];
        assert_eq!(
            indices
                .iter()
                .map(|&i| i - first as u32)
                .collect::<Vec<_>>(),
            expected.indices
        );
    }
}

#[test]
fn ship_fronts_replace_only_unchanged_bundled_profiles_and_custom_profiles_win_in_any_order() {
    let canonical: Vec<_> = bundled_profiles()
        .objects
        .iter()
        .filter(|object| canonical_replaced_profile(object))
        .cloned()
        .collect();
    assert_eq!(canonical.len(), 5);
    for kind in Kind::ALL {
        let (cells, g, origin) = fixture(kind);
        let refs: Vec<_> = cells.iter().collect();
        let empty = vec![false; cells.len()];
        assert_eq!(
            resolve(
                "FastShipB1F",
                &refs,
                &g,
                origin,
                Some(bundled_profiles()),
                &empty
            )
            .len(),
            1
        );
        for object in &canonical {
            for edit in 0..4 {
                let mut changed = object.clone();
                match edit {
                    0 => changed.name.push_str(" custom"),
                    1 => changed.depth_pixels = 1.,
                    2 => changed.footing_pixels = Some(2.),
                    _ => changed.map = Some("FastShipB1F".into()),
                }
                assert!(!canonical_replaced_profile(&changed));
                for custom_first in [true, false] {
                    let mut doc = bundled_profiles().clone();
                    if custom_first {
                        doc.objects.insert(0, changed.clone());
                    } else {
                        doc.objects.push(changed.clone());
                    }
                    assert!(
                        resolve("FastShipB1F", &refs, &g, origin, Some(&doc), &empty).is_empty()
                    );
                }
            }
        }
    }
}
