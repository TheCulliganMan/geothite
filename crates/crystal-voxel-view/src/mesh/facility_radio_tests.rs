fn workbench_fixture() -> (Vec<VisualTile>, GridGeometry) {
    let art = [
        [0x02, 0x03, 0x04, 0x05],
        [0x12, 0x13, 0x14, 0x15],
        [0x06, 0x07, 0x08, 0x09],
        [0x26, 0x01, 0x26, 0x01],
    ];
    let tiles = (0..16)
        .map(|i| VisualTile {
            column: (i % 4) as u32,
            row: (i / 4) as u32,
            source: VisualTileSource {
                tileset_id: Arc::from("facility"),
                metatile_id: if i < 12 { 0x28 } else { 0x07 },
                subtile_column: (i % 4) as u8,
                subtile_row: (i / 4) as u8,
                tile_index: art[i / 4][i % 4],
            },
            texture: Default::default(),
            priority: false,
        })
        .collect();
    let g = GridGeometry {
        width: 4,
        height: 4,
        tile_width: 8.,
        tile_height: 8.,
        origin_x: 0.,
        origin_z: 0.,
    };
    (tiles, g)
}
#[test]
fn complete_workbench_keeps_native_checker_floor_and_footing_and_live_readout() {
    let (tiles, g) = workbench_fixture();
    let cells: Vec<_> = tiles.iter().collect();
    let mut claimed = vec![false; 16];
    let out = resolve("MrPokemonsHouse", &cells, &g, [0, 0], None, &claimed);
    assert_eq!(out.len(), 1);
    let mut mesh = TerrainMeshData::default();
    mesh.authored_cells = vec![None; 16];
    mesh.footing_heights = (0..16).map(|i| i as f32 * 0.01).collect();
    let footing = mesh.footing_heights.clone();
    assert!(append(&mut mesh, &cells, &g, &out[0], &mut claimed));
    assert_eq!(
        mesh.footing_heights, footing,
        "furniture does not edit source support heights"
    );
    assert_eq!(claimed.iter().filter(|&&v| v).count(), 12);
    assert!(
        claimed[12..].iter().all(|&v| !v),
        "genuine lower source floor remains unclaimed"
    );
    assert_eq!(
        mesh.authored_cells.iter().filter(|v| v.is_some()).count(),
        12
    );
    assert!(mesh.textured.positions[..48].iter().all(|p| p[1] == 0.));
    for y in 0..3 {
        for x in 0..4 {
            let i = y * 4 + x;
            let sample = out[0].resolved.ground_for(x, y);
            assert_eq!(
                cells[sample].source.tile_index,
                if (x + y) % 2 == 0 { 1 } else { 0x26 }
            );
            let uv = g.uv(sample % 4, sample / 4);
            assert!(
                mesh.textured.uvs[i * 4..i * 4 + 4]
                    .iter()
                    .all(|p| p[0] >= uv.0 && p[0] <= uv.1 && p[1] >= uv.2 && p[1] <= uv.3)
            );
        }
    }
    assert!(
        mesh.textured.positions[48..]
            .iter()
            .all(|p| p[1] > 11. && p[1] < 17. && p[2] > 14.19 && p[2] < 14.21)
    );
    assert!(mesh.solid.positions.iter().all(|p| p[0] >= 0.
        && p[0] <= 32.
        && p[1] >= 0.
        && p[1] <= 18.
        && p[2] >= 6.
        && p[2] <= 24.));
    assert!(mesh.solid.cutaway_ranges.is_empty() && mesh.textured.cutaway_ranges.is_empty());
    let before = mesh.clone();
    assert!(!append(&mut mesh, &cells, &g, &out[0], &mut claimed));
    assert_eq!(mesh, before);
}
#[test]
fn custom_workbench_profile_wins_and_stale_source_or_floor_appends_nothing() {
    let (mut tiles, g) = workbench_fixture();
    let empty = vec![false; 16];
    let profile:Document=serde_json::from_str(r#"{"objects":[{"name":"My offset instrument bench","map":"MrPokemonsHouse","tileset":"facility","metatile":40,"origin":[0,0],"tiles":[[2,3,4,5],[18,19,20,21],[6,7,8,9]],"ground":1,"top_pixels":8,"depth_pixels":18}]}"#).unwrap();
    assert!(
        resolve(
            "MrPokemonsHouse",
            &tiles.iter().collect::<Vec<_>>(),
            &g,
            [0, 0],
            Some(&profile),
            &empty
        )
        .is_empty()
    );
    let out = resolve(
        "MrPokemonsHouse",
        &tiles.iter().collect::<Vec<_>>(),
        &g,
        [0, 0],
        None,
        &empty,
    );
    assert_eq!(out.len(), 1);
    for i in [
        0,
        5,
        10,
        out[0].resolved.ground[0],
        out[0].resolved.ground[1],
    ] {
        let old = tiles[i].source.tile_index;
        tiles[i].source.tile_index ^= 1;
        let mut mesh = TerrainMeshData::default();
        let mut claimed = empty.clone();
        assert!(!append(
            &mut mesh,
            &tiles.iter().collect::<Vec<_>>(),
            &g,
            &out[0],
            &mut claimed
        ));
        assert_eq!(mesh, TerrainMeshData::default());
        assert_eq!(claimed, empty);
        tiles[i].source.tile_index = old;
    }
}
#[test]
fn six_runtime_models_fit_finite_authored_geometry_and_preserve_the_staff_openings() {
    for (kind, index) in [
        (Kind::Workbench, 0),
        (Kind::PhoneDesk, 1),
        (Kind::MemoDesk, 2),
        (Kind::ReceptionU, 3),
        (Kind::ReceptionL, 4),
        (Kind::CounterExtension, 5),
    ] {
        let (bounds, rise) = kind.fitting();
        let mut mesh = SurfaceMeshData::default();
        model(index).append_fitted(&mut mesh, bounds, 0., rise);
        assert!(mesh.indices.len() > 100 && mesh.indices.len() < 7000);
        assert!(
            mesh.positions
                .iter()
                .all(|p| p.iter().all(|v| v.is_finite())
                    && p[0] >= bounds[0] - 0.001
                    && p[0] <= bounds[1] + 0.001
                    && p[1] >= -0.001
                    && p[1] <= rise + 0.001
                    && p[2] >= bounds[2] - 0.001
                    && p[2] <= bounds[3] + 0.001)
        );
        assert!(
            mesh.normals
                .iter()
                .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 0.001)
        );
        if kind == Kind::ReceptionL {
            assert!(
                mesh.positions
                    .iter()
                    .all(|p| p[0] <= 16.001 || p[2] <= 16.001)
            );
        }
        if kind == Kind::ReceptionU {
            assert!(mesh.positions.iter().all(|p| p[0] <= 16.001
                || p[0] >= 143.999
                || p[2] <= 16.001
                || (p[0] >= 79.999 && p[0] <= 96.001 && p[2] <= 24.001)));
        }
        assert!(
            mesh.colors
                .iter()
                .all(|c| c.iter().all(|v| v.is_finite() && (0.0..=1.0).contains(v)))
        );
    }
}
#[test]
fn live_display_triangles_face_outward_and_remain_inside_the_live_source_drawing() {
    let (tiles, g) = workbench_fixture();
    let cells: Vec<_> = tiles.iter().collect();
    let mut claimed = vec![false; 16];
    let out = resolve("MrPokemonsHouse", &cells, &g, [0, 0], None, &claimed);
    let mut mesh = TerrainMeshData::default();
    assert!(append(&mut mesh, &cells, &g, &out[0], &mut claimed));
    for ids in mesh.textured.indices.chunks_exact(3) {
        let a = Vec3::from_array(mesh.textured.positions[ids[0] as usize]);
        let b = Vec3::from_array(mesh.textured.positions[ids[1] as usize]);
        let c = Vec3::from_array(mesh.textured.positions[ids[2] as usize]);
        let normal = Vec3::from_array(mesh.textured.normals[ids[0] as usize]);
        assert!((b - a).cross(c - a).dot(normal) > 0.);
    }
    for uv in &mesh.textured.uvs[48..] {
        assert!(uv[0] >= 0. && uv[0] <= 0.5 && uv[1] >= 0. && uv[1] <= 0.25);
    }
}

fn round_stool_fixture() -> (Vec<VisualTile>, GridGeometry) {
    let g = GridGeometry { width: 6, height: 6, tile_width: 8., tile_height: 8., origin_x: -8., origin_z: 13. };
    let mut cells: Vec<_> = (0..36).map(|i| VisualTile {
        column: (i % 6) as u32, row: (i / 6) as u32, texture: Default::default(), priority: false,
        source: VisualTileSource { tileset_id: Arc::from("radio_tower"), metatile_id: 1,
            subtile_column: (i % 6 % 4) as u8, subtile_row: (i / 6 % 4) as u8, tile_index: 1 },
    }).collect();
    for y in 0..2 { for x in 0..2 {
        cells[(2 + y) * 6 + 2 + x].source = VisualTileSource {
            tileset_id: Arc::from("radio_tower"), metatile_id: 0x27,
            subtile_column: (2 + x) as u8, subtile_row: y as u8,
            tile_index: [[0x2c, 0x2d], [0x3c, 0x3d]][y][x],
        };
    }}
    (cells, g)
}
#[test]
fn radio_round_stool_is_complete_source_contained_and_keeps_native_footing() {
    let (mut cells, g) = round_stool_fixture();
    let empty = vec![false; cells.len()];
    for map in ["RadioTower3F", "RadioTower4F", "RadioTower5F"] {
        let out = resolve(map, &cells.iter().collect::<Vec<_>>(), &g, [0,0], None, &empty);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].resolved.kind(), Kind::RoundStool);
        let mut mesh = TerrainMeshData::default();
        mesh.footing_heights = (0..cells.len()).map(|i| i as f32 * 0.02).collect();
        let footing = mesh.footing_heights.clone();
        let mut claimed = empty.clone();
        assert!(append(&mut mesh, &cells.iter().collect::<Vec<_>>(), &g, &out[0], &mut claimed));
        assert_eq!(mesh.footing_heights, footing);
        assert_eq!(claimed.iter().filter(|&&v| v).count(), 4);
        let (w, _, n, _) = g.bounds(2, 2);
        let mut cached = SurfaceMeshData::default();
        crate::interior_models::model(ModelKind::ArcadeStool)
            .append_fitted(&mut cached, [w + 2., w + 14., n, n + 12.], 0., 7.);
        assert_eq!(mesh.solid, cached);
        for y in 0..2 { for x in 0..2 {
            let i = (2 + y) * g.width + 2 + x;
            cells[i].source.tile_index ^= 1;
            assert!(resolve(map, &cells.iter().collect::<Vec<_>>(), &g, [0,0], None, &empty).is_empty());
            let mut stale = TerrainMeshData::default();
            let mut fresh = empty.clone();
            assert!(!append(&mut stale, &cells.iter().collect::<Vec<_>>(), &g, &out[0], &mut fresh));
            assert_eq!(stale, TerrainMeshData::default());
            assert_eq!(fresh, empty);
            cells[i].source.tile_index ^= 1;
        }}
    }
    assert!(resolve("RadioTower3FBeta", &cells.iter().collect::<Vec<_>>(), &g, [0,0], None, &empty).is_empty());
    let custom: Document = serde_json::from_str(r#"{"objects":[{"name":"My round stool","map":"RadioTower3F","tileset":"radio_tower","metatile":39,"origin":[2,0],"tiles":[[44,45],[60,61]],"ground":1,"top_pixels":8,"depth_pixels":12}]}"#).unwrap();
    assert!(resolve("RadioTower3F", &cells.iter().collect::<Vec<_>>(), &g, [0,0], Some(&custom), &empty).is_empty());
}
