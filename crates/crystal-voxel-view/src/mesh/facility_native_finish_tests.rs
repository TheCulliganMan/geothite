#[test]
fn external_facility_native_finish_preserves_all_thirty_two_objects_geometry_and_support() {
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap(),
    )
    .unwrap();
    let mut count = 0;
    let mut cells_owned = 0;
    for map in [
        "MrPokemonsHouse",
        "PowerPlant",
        "RuinsOfAlphResearchCenter",
        "SilphCo1F",
        "TeamRocketBaseB3F",
    ] {
        let (tiles, g) = wall_sofa_fixture(map, &loaded);
        let original = tiles.clone();
        let refs = tiles.iter().collect::<Vec<_>>();
        let empty = vec![false; tiles.len()];
        let ps = resolve(map, &refs, &g, [0, 0], Some(built_in_profiles()), &empty);
        for p in ps.iter().filter(|p| {
            matches!(
                p.resolved.kind(),
                Kind::Chair | Kind::SquareTable | Kind::MeetingTable | Kind::SideDesk
            )
        }) {
            let kind = p.resolved.kind();
            let owned = p.indices(g.width).count();
            let mut mesh = TerrainMeshData {
                footing_heights: vec![2.75; tiles.len()],
                authored_cells: vec![None; tiles.len()],
                ..Default::default()
            };
            let mut claimed = empty.clone();
            assert!(append(&mut mesh, &refs, &g, p, &mut claimed));
            assert_eq!(mesh.footing_heights, vec![2.75; tiles.len()]);
            assert_eq!(claimed.iter().filter(|&&v| v).count(), owned);
            assert_eq!(tiles, original);
            assert!(mesh.solid.indices.is_empty());
            let mut body = furniture_body(&mesh, owned);
            if p.resolved.native_silph_meeting_table() {
                // Its already-source-gold selected model is an unchanged control.
                let (w, _, n, _) = g.bounds(p.resolved.column, p.resolved.row);
                let uv = g.uv(p.resolved.column + 3, p.resolved.row + 3);
                let gold = [
                    uv.0 + (uv.1 - uv.0) * 4.5 / 8.,
                    uv.2 + (uv.3 - uv.2) * 4.5 / 8.,
                ];
                let mut old = SurfaceMeshData::default();
                assert!(native_meeting_model().unwrap().append_native_dining_table(
                    &mut old,
                    [w + 16., w + 64., n + 16., n + 48.],
                    9.135,
                    [gold; 2]
                ));
                assert_eq!(body, old);
            } else {
                // A document's final four vertices are the unchanged live ink face.
                if matches!(kind, Kind::SquareTable | Kind::SideDesk) {
                    let first = body.positions.len() - 4;
                    body.positions.truncate(first);
                    body.normals.truncate(first);
                    body.uvs.truncate(first);
                    body.colors.truncate(first);
                    body.indices.truncate(body.indices.len() - 6);
                }
                let (w, _, n, _) = g.bounds(p.resolved.column, p.resolved.row);
                let ([l, r, t, b], h) = kind.fitting();
                let mut old = SurfaceMeshData::default();
                model(kind).append_fitted(&mut old, [w + l, w + r, n + t, n + b], 0., h);
                if kind == Kind::Chair {
                    chair_facing::rotate(&mut old, 0, &g, &p.resolved, p.chair_turn);
                }
                assert_eq!(
                    body.positions, old.positions,
                    "{map}: material change never moves geometry"
                );
                assert_eq!(body.normals, old.normals);
                assert_eq!(body.indices, old.indices);
                let swatches = facility_swatches(&g, &p.resolved);
                for uv in swatches {
                    assert!(
                        body.uvs.contains(&uv),
                        "all native material roles represented"
                    );
                }
                for (uv, (color, normal)) in
                    body.uvs.iter().zip(body.colors.iter().zip(&body.normals))
                {
                    assert!(swatches.contains(uv));
                    let expected = crate::interior_models::authored_face_color(
                        [1.; 4],
                        Vec3::from_array(*normal),
                    );
                    assert!(
                        color
                            .iter()
                            .zip(expected)
                            .all(|(a, b)| (a - b).abs() < 0.000001)
                    );
                }
            }
            count += 1;
            cells_owned += owned;
        }
    }
    assert_eq!((count, cells_owned), (32, 260));
}

#[test]
fn facility_native_finish_rejects_unknown_part_or_material_and_keeps_cached_geometry() {
    let documents = [
        crate::model_storage::include_model!(
            "models/facility_tables/facility_square_document_table.mesh.json"
        ),
        crate::model_storage::include_model!(
            "models/facility_tables/facility_meeting_table.mesh.json"
        ),
        crate::model_storage::include_model!(
            "models/facility_tables/facility_document_side_desk.mesh.json"
        ),
        crate::model_storage::include_model!(
            "models/facility_tables/facility_square_back_chair.mesh.json"
        ),
    ];
    for raw in documents {
        let source = Model::parse(raw).unwrap();
        assert!(source.has_native_facility_furniture_materials());
        for (sx, sz) in [(1., 1.), (0.5, 2.), (2., 0.5)] {
            let b = [-2., 32. * sx, 3., 32. * sz];
            let mut old = SurfaceMeshData::default();
            source.append_fitted(&mut old, b, 0., 12. * sz);
            let mut native = SurfaceMeshData::default();
            assert!(source.append_native_facility_furniture(
                &mut native,
                b,
                12. * sz,
                [[0.1, 0.2], [0.3, 0.4], [0.5, 0.6], [0.7, 0.8]]
            ));
            assert_eq!(native.positions, old.positions);
            assert_eq!(native.normals, old.normals);
            assert_eq!(native.indices, old.indices);
            let mut after = SurfaceMeshData::default();
            source.append_fitted(&mut after, b, 0., 12. * sz);
            assert_eq!(old, after);
        }
        for field in ["name", "material"] {
            let mut invalid: serde_json::Value =
                serde_json::from_slice(&crate::model_storage::decode_bytes(raw).unwrap()).unwrap();
            invalid["primitives"][0][field] = "Unreviewed finish".into();
            let unknown = Model::parse(&invalid.to_string()).unwrap();
            assert!(!unknown.has_native_facility_furniture_materials());
            let mut out = SurfaceMeshData::default();
            assert!(!unknown.append_native_facility_furniture(
                &mut out,
                [0., 32., 0., 32.],
                12.,
                [[0.1; 2]; 4]
            ));
            assert_eq!(
                out,
                SurfaceMeshData::default(),
                "unknown roles fail before any append"
            );
        }
    }
}
