fn fixture(kind: Kind) -> (Vec<VisualTile>, GridGeometry, &'static str) {
    let (map, rows, blocks): (&str, Vec<Vec<u16>>, Vec<Vec<u16>>) = match kind {
        Kind::SquareTable => (
            "MrPokemonsHouse",
            vec![
                vec![64, 65, 65, 66],
                vec![80, 81, 81, 82],
                vec![80, 72, 73, 82],
                vec![83, 58, 58, 84],
            ],
            vec![vec![0x0f; 4]; 4],
        ),
        Kind::SideDesk => (
            "RuinsOfAlphResearchCenter",
            vec![
                vec![64, 65, 65, 66],
                vec![80, 72, 73, 82],
                vec![6, 7, 8, 9],
                vec![38, 1, 38, 1],
            ],
            vec![vec![0x29; 4]; 4],
        ),
        Kind::Chair => (
            "MrPokemonsHouse",
            vec![vec![14, 15], vec![30, 31]],
            vec![vec![0x27; 2]; 2],
        ),
        Kind::MeetingTable => (
            "PowerPlant",
            vec![
                vec![1, 38, 14, 15, 1, 38, 14, 15],
                vec![38, 1, 30, 31, 38, 1, 30, 31],
                vec![1, 38, 64, 65, 65, 65, 65, 66],
                vec![38, 1, 80, 81, 81, 81, 81, 82],
                vec![1, 38, 80, 81, 81, 81, 81, 82],
                vec![38, 1, 83, 58, 58, 58, 58, 84],
                vec![1, 38, 14, 15, 1, 38, 14, 15],
                vec![38, 1, 30, 31, 38, 1, 30, 31],
            ],
            (0..8)
                .map(|y| {
                    (0..8)
                        .map(|x| [[0x10, 0x11], [0x14, 0x15]][y / 4][x / 4])
                        .collect()
                })
                .collect(),
        ),
    };
    let w = rows[0].len() + 2;
    let h = rows.len() + 2;
    let g = GridGeometry {
        width: w,
        height: h,
        tile_width: 8.,
        tile_height: 8.,
        origin_x: -13.,
        origin_z: 7.,
    };
    let mut cells: Vec<_> = (0..w * h)
        .map(|i| VisualTile {
            column: (i % w) as u32,
            row: (i / w) as u32,
            texture: Default::default(),
            priority: false,
            source: VisualTileSource {
                tileset_id: Arc::from("facility"),
                metatile_id: 7,
                subtile_column: (i % w % 4) as u8,
                subtile_row: (i / w % 4) as u8,
                tile_index: if (i % w + i / w) % 2 == 0 { 1 } else { 0x26 },
            },
        })
        .collect();
    for y in 0..rows.len() {
        for x in 0..rows[0].len() {
            cells[y * w + x].source = VisualTileSource {
                tileset_id: Arc::from("facility"),
                metatile_id: blocks[y][x],
                subtile_column: if kind == Kind::Chair {
                    (x + 2) as u8
                } else {
                    (x % 4) as u8
                },
                subtile_row: (y % 4) as u8,
                tile_index: rows[y][x],
            };
        }
    }
    (cells, g, map)
}
#[test]
fn facility_tables_are_complete_finite_and_leave_native_actor_support_alone() {
    for (kind, owned) in [
        (Kind::SquareTable, 16),
        (Kind::MeetingTable, 24),
        (Kind::SideDesk, 12),
        (Kind::Chair, 4),
    ] {
        let (cells, g, map) = fixture(kind);
        let refs: Vec<_> = cells.iter().collect();
        let mut claimed = vec![false; cells.len()];
        let placements = resolve(map, &refs, &g, [0, 0], None, &claimed);
        let p = placements
            .iter()
            .find(|p| p.resolved.kind() == kind)
            .unwrap();
        let mut mesh = TerrainMeshData::default();
        mesh.authored_cells = vec![None; cells.len()];
        mesh.footing_heights = (0..cells.len()).map(|i| i as f32 * 0.02).collect();
        let footing = mesh.footing_heights.clone();
        assert!(append(&mut mesh, &refs, &g, p, &mut claimed));
        assert_eq!(mesh.footing_heights, footing);
        assert_eq!(claimed.iter().filter(|&&v| v).count(), owned);
        assert_eq!(
            mesh.authored_cells.iter().filter(|v| v.is_some()).count(),
            owned
        );
        assert!(
            mesh.textured.positions[..owned * 4]
                .iter()
                .all(|p| p[1] == 0.)
        );
        assert!(mesh.solid.cutaway_ranges.is_empty() && mesh.textured.cutaway_ranges.is_empty());
        let ([west, east, north, south], rise) = kind.fitting();
        assert!(
            mesh.solid
                .positions
                .iter()
                .all(|p| p.iter().all(|v| v.is_finite())
                    && p[0] >= g.origin_x + west - 0.001
                    && p[0] <= g.origin_x + east + 0.001
                    && p[1] >= -0.001
                    && p[1] <= rise + 0.001
                    && p[2] >= g.origin_z + north - 0.001
                    && p[2] <= g.origin_z + south + 0.001)
        );
        assert!(
            mesh.solid
                .normals
                .iter()
                .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 0.001)
        );
        let before = mesh.clone();
        assert!(!append(&mut mesh, &refs, &g, p, &mut claimed));
        assert_eq!(mesh, before);
        if kind == Kind::SideDesk {
            assert!(
                (0..4).all(|x| !claimed[3 * g.width + x]),
                "native lower floor retained"
            );
        }
        if kind == Kind::MeetingTable {
            for y in 0..8 {
                for x in 0..8 {
                    assert_eq!(
                        claimed[y * g.width + x],
                        (2..6).contains(&y) && x >= 2,
                        "chairs and surrounding floor are separate from the table"
                    );
                }
            }
        }
    }
}
#[test]
fn facility_custom_floor_guard_and_stale_source_fail_without_partial_append() {
    let (mut cells, g, map) = fixture(Kind::SideDesk);
    let empty = vec![false; cells.len()];
    let custom:Document=serde_json::from_str(r#"{"objects":[{"name":"Custom native floor beside desk","map":"RuinsOfAlphResearchCenter","tileset":"facility","metatile":41,"origin":[0,3],"tiles":[[38,1,38,1]],"ground":1,"top_pixels":0,"depth_pixels":8}]}"#).unwrap();
    assert!(
        resolve(
            map,
            &cells.iter().collect::<Vec<_>>(),
            &g,
            [0, 0],
            Some(&custom),
            &empty
        )
        .is_empty()
    );
    let placements = resolve(
        map,
        &cells.iter().collect::<Vec<_>>(),
        &g,
        [0, 0],
        None,
        &empty,
    );
    assert_eq!(placements.len(), 1);
    for i in [
        0,
        3 * g.width,
        placements[0].resolved.ground[0],
        placements[0].resolved.ground[1],
    ] {
        cells[i].source.tile_index ^= 1;
        let mut mesh = TerrainMeshData::default();
        let mut claimed = empty.clone();
        assert!(!append(
            &mut mesh,
            &cells.iter().collect::<Vec<_>>(),
            &g,
            &placements[0],
            &mut claimed
        ));
        assert_eq!(mesh, TerrainMeshData::default());
        assert_eq!(claimed, empty);
        cells[i].source.tile_index ^= 1;
    }
}
#[test]
fn facility_document_face_is_live_outward_and_clears_its_physical_paper() {
    for kind in [Kind::SquareTable, Kind::SideDesk] {
        let (cells, g, map) = fixture(kind);
        let mut claimed = vec![false; cells.len()];
        let refs: Vec<_> = cells.iter().collect();
        let placements = resolve(map, &refs, &g, [0, 0], None, &claimed);
        let p = placements
            .iter()
            .find(|p| p.resolved.kind() == kind)
            .unwrap();
        let mut mesh = TerrainMeshData::default();
        assert!(append(&mut mesh, &refs, &g, p, &mut claimed));
        let count = p.indices(g.width).count() * 4;
        assert!(
            mesh.textured.positions[count..]
                .iter()
                .all(|p| (p[1] - 9.835).abs() < 0.001)
        );
        for tri in mesh.textured.indices.chunks_exact(3) {
            let a = Vec3::from_array(mesh.textured.positions[tri[0] as usize]);
            let b = Vec3::from_array(mesh.textured.positions[tri[1] as usize]);
            let c = Vec3::from_array(mesh.textured.positions[tri[2] as usize]);
            let n = Vec3::from_array(mesh.textured.normals[tri[0] as usize]);
            assert!((b - a).cross(c - a).dot(n) > 0.);
        }
        let source_row = if kind == Kind::SquareTable { 2 } else { 1 };
        let uv = g.uv(p.resolved.column + 1, p.resolved.row + source_row);
        let u = |pixel: f32| uv.0 + (uv.1 - uv.0) * pixel / 8.;
        let v = |pixel: f32| uv.2 + (uv.3 - uv.2) * pixel / 8.;
        assert_eq!(mesh.textured.positions.len() - count, 4);
        assert_eq!(
            &mesh.textured.uvs[count..],
            &[
                [u(1.), v(7.)],
                [u(7.), v(7.)],
                [u(7.), v(1.)],
                [u(1.), v(1.)],
            ],
            "the exact 6x6 native ink face keeps its orientation on the page"
        );
    }
}
