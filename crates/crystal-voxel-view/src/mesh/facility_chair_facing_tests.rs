fn chair_forward(turn: u8) -> [f32; 2] {
    match turn {
        0 => [0., 1.],
        1 => [1., 0.],
        2 => [0., -1.],
        3 => [-1., 0.],
        _ => panic!("quarter turn"),
    }
}

fn furniture_body(mesh: &TerrainMeshData, owned: usize) -> SurfaceMeshData {
    let first = owned * 4;
    SurfaceMeshData {
        positions: mesh.textured.positions[first..].to_vec(),
        normals: mesh.textured.normals[first..].to_vec(),
        colors: mesh.textured.colors[first..].to_vec(),
        uvs: mesh.textured.uvs[first..].to_vec(),
        indices: mesh.textured.indices[owned * 6..]
            .iter()
            .map(|i| i - first as u32)
            .collect(),
        ..Default::default()
    }
}

fn check_chair_geometry(mesh: &SurfaceMeshData, g: &GridGeometry, p: &source::Match, turn: u8) {
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let sx = g.tile_width / 8.;
    let sz = g.tile_height / 8.;
    assert!(!mesh.positions.is_empty());
    for v in &mesh.positions {
        assert!(
            v.iter().all(|v| v.is_finite())
                && v[0] >= w - 0.001
                && v[0] <= w + 16. * sx + 0.001
                && v[2] >= n - 0.001
                && v[2] <= n + 12. * sz + 0.001
                && v[1] >= -0.001
                && v[1] <= 11.4 * sz + 0.001,
            "chair stays inside its owned plot and clear of the actor foot"
        );
    }
    let back = mesh
        .positions
        .iter()
        .filter(|v| v[1] > 8.5 * sz)
        .collect::<Vec<_>>();
    assert!(!back.is_empty());
    let center = [8., 4. + if turn % 2 == 1 { 2. } else { 0. }];
    let forward = chair_forward(turn);
    let mean = back.iter().fold([0.; 2], |a, v| {
        [
            a[0] + ((v[0] - w) / sx - center[0]) / back.len() as f32,
            a[1] + ((v[2] - n) / sz - center[1]) / back.len() as f32,
        ]
    });
    assert!(
        mean[0] * forward[0] + mean[1] * forward[1] < -1.,
        "the actual high backrest is behind the seat's serving direction"
    );
    for t in mesh.indices.chunks_exact(3) {
        let [a, b, c] = [0, 1, 2].map(|j| Vec3::from_array(mesh.positions[t[j] as usize]));
        let normal = Vec3::from_array(mesh.normals[t[0] as usize]);
        assert!((normal.length() - 1.).abs() < 0.001);
        assert!(
            (b - a).cross(c - a).dot(normal) > 0.,
            "turned faces stay outward after nonuniform fitting"
        );
    }
}

#[test]
fn external_facility_chair_facing_matches_all_twenty_serving_seats_and_two_waiting_seats() {
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap(),
    )
    .unwrap();
    let cases: [(&str, &[(usize, usize, u8, bool)]); 4] = [
        ("MrPokemonsHouse", &[(6, 8, 1, true), (6, 10, 1, true)]),
        (
            "PowerPlant",
            &[
                (6, 4, 0, true),
                (10, 4, 0, true),
                (6, 10, 2, true),
                (10, 10, 2, true),
                (18, 28, 0, false),
                (18, 30, 0, false),
            ],
        ),
        (
            "SilphCo1F",
            &[
                (22, 8, 0, true),
                (26, 8, 0, true),
                (22, 14, 2, true),
                (26, 14, 2, true),
                (0, 2, 0, true),
                (4, 2, 0, true),
            ],
        ),
        (
            "TeamRocketBaseB3F",
            &[
                (22, 4, 0, true),
                (26, 4, 0, true),
                (22, 10, 2, true),
                (26, 10, 2, true),
                (18, 24, 0, true),
                (22, 24, 0, true),
                (18, 30, 2, true),
                (22, 30, 2, true),
            ],
        ),
    ];
    let mut chairs = 0;
    let mut paired = 0;
    let mut changed = 0;
    for (map, expected) in cases {
        let (tiles, g) = wall_sofa_fixture(map, &loaded);
        let original = tiles.clone();
        let refs = tiles.iter().collect::<Vec<_>>();
        let empty = vec![false; tiles.len()];
        let p = resolve(map, &refs, &g, [0, 0], Some(built_in_profiles()), &empty);
        let actual = p
            .iter()
            .filter(|p| p.resolved.kind() == Kind::Chair)
            .map(|p| {
                (
                    (p.resolved.column, p.resolved.row),
                    (p.chair_turn, p.chair_target.is_some()),
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        let wanted = expected
            .iter()
            .map(|&(x, y, q, t)| ((x, y), (q, t)))
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(actual, wanted, "{map}: source table/counter relationships");
        for p in p.iter().filter(|p| p.resolved.kind() == Kind::Chair) {
            let mut mesh = TerrainMeshData {
                footing_heights: vec![2.75; tiles.len()],
                authored_cells: vec![None; tiles.len()],
                ..Default::default()
            };
            let mut claimed = empty.clone();
            assert!(append(&mut mesh, &refs, &g, p, &mut claimed));
            assert_eq!(claimed.iter().filter(|&&v| v).count(), 4);
            assert_eq!(mesh.footing_heights, vec![2.75; tiles.len()]);
            assert_eq!(tiles, original);
            assert!(mesh.textured.positions[..16].iter().all(|p| p[1] == 0.));
            assert!(mesh.solid.positions.is_empty());
            let body = furniture_body(&mesh, 4);
            check_chair_geometry(&body, &g, &p.resolved, p.chair_turn);
            if p.chair_turn == 0 {
                let (w, _, n, _) = g.bounds(p.resolved.column, p.resolved.row);
                let mut baseline = SurfaceMeshData::default();
                assert!(model(Kind::Chair).append_native_facility_furniture(
                    &mut baseline,
                    [w + 2., w + 14., n, n + 8.],
                    11.4,
                    facility_swatches(&g, &p.resolved)
                ));
                assert_eq!(
                    baseline, body,
                    "already-correct headings retain exact native body"
                );
            }
            chairs += 1;
            paired += usize::from(p.chair_target.is_some());
            changed += usize::from(p.chair_turn != 0);
        }
    }
    assert_eq!((chairs, paired, changed), (22, 20, 10));
}

#[test]
fn facility_chair_facing_all_quarter_turns_keep_footprint_normals_material_alpha_and_cached_model()
{
    let (tiles, mut g, map) = fixture(Kind::Chair);
    let refs = tiles.iter().collect::<Vec<_>>();
    let p = resolve(map, &refs, &g, [0, 0], None, &vec![false; tiles.len()]);
    let p = p.iter().find(|p| p.resolved.kind() == Kind::Chair).unwrap();
    for (sx, sz) in [(0.5, 0.5), (1., 1.), (2., 2.), (0.5, 2.), (2., 0.5)] {
        g.tile_width = 8. * sx;
        g.tile_height = 8. * sz;
        let (w, _, n, _) = g.bounds(p.resolved.column, p.resolved.row);
        let mut original = SurfaceMeshData::default();
        model(Kind::Chair).append_fitted(
            &mut original,
            [w + 2. * sx, w + 14. * sx, n, n + 8. * sz],
            0.,
            11.4 * sz,
        );
        for q in 0..4 {
            let mut turned = original.clone();
            chair_facing::rotate(&mut turned, 0, &g, &p.resolved, q);
            check_chair_geometry(&turned, &g, &p.resolved, q);
            assert_eq!(turned.indices, original.indices);
            assert_eq!(turned.uvs, original.uvs);
            assert!(
                turned
                    .colors
                    .iter()
                    .zip(&original.colors)
                    .all(|(a, b)| a[3] == b[3] && a.iter().all(|v| v.is_finite()))
            );
            let mut after = SurfaceMeshData::default();
            model(Kind::Chair).append_fitted(
                &mut after,
                [w + 2. * sx, w + 14. * sx, n, n + 8. * sz],
                0.,
                11.4 * sz,
            );
            assert_eq!(after, original, "no turn mutates the cached model");
        }
    }
}

#[test]
fn external_facility_chair_facing_yields_to_custom_or_edited_tables_and_rechecks_target_before_append()
 {
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap(),
    )
    .unwrap();
    let (tiles, g) = wall_sofa_fixture("MrPokemonsHouse", &loaded);
    let refs = tiles.iter().collect::<Vec<_>>();
    let empty = vec![false; tiles.len()];
    let p = resolve("MrPokemonsHouse", &refs, &g, [0, 0], None, &empty);
    let chair = p.iter().find(|p| p.resolved.kind() == Kind::Chair).unwrap();
    assert_eq!(chair.chair_turn, 1);
    let i = 8 * g.width + 8;
    let mut changed = tiles.clone();
    changed[i].source.tile_index ^= 1;
    let changed = changed.iter().collect::<Vec<_>>();
    let fresh = resolve("MrPokemonsHouse", &changed, &g, [0, 0], None, &empty);
    assert!(
        fresh
            .iter()
            .filter(|p| p.resolved.kind() == Kind::Chair)
            .all(|p| p.chair_turn == 0 && p.chair_target.is_none()),
        "an edited table cannot supply a facing"
    );
    let mut mesh = TerrainMeshData::default();
    assert!(append(&mut mesh, &changed, &g, chair, &mut empty.clone()));
    check_chair_geometry(&furniture_body(&mesh, 4), &g, &chair.resolved, 0);
    let s = &tiles[i].source;
    let mut custom = built_in_profiles().objects[0].clone();
    custom.name = "Custom serving surface".into();
    custom.map = Some("MrPokemonsHouse".into());
    custom.maps = None;
    custom.tileset = "facility".into();
    custom.metatile = s.metatile_id;
    custom.metatiles = None;
    custom.origin = [s.subtile_column, s.subtile_row];
    custom.tiles = vec![vec![s.tile_index]];
    custom.ground = 1;
    custom.parts.clear();
    custom.footing_pixels = None;
    let custom = resolve(
        "MrPokemonsHouse",
        &refs,
        &g,
        [0, 0],
        Some(&Document {
            objects: vec![custom],
            atmosphere: None,
        }),
        &empty,
    );
    let chairs = custom
        .iter()
        .filter(|p| p.resolved.kind() == Kind::Chair)
        .collect::<Vec<_>>();
    assert_eq!(chairs.len(), 2);
    assert!(
        chairs
            .iter()
            .all(|p| p.chair_turn == 0 && p.chair_target.is_none()),
        "custom target geometry keeps first refusal"
    );
}
