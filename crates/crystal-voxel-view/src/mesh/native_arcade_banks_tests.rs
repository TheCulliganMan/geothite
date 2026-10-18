use super::*;
fn fixture(
    map: &str,
    pack: &crystal_assets::LoadedCompiledGamePack,
) -> (Vec<VisualTile>, GridGeometry) {
    let p = pack.pack();
    let d = &p.data().maps[map];
    let art = &p.runtime_files()["data/tilesets/game_corner_metatiles.bin"];
    let (w, h) = (
        d.attributes.width as usize * 4,
        d.attributes.height as usize * 4,
    );
    let cells = (0..w * h)
        .map(|i| {
            let (x, y) = (i % w, i / w);
            let b = d.blocks[y / 4 * (w / 4) + x / 4];
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
        .collect();
    (
        cells,
        GridGeometry {
            width: w,
            height: h,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        },
    )
}
#[test]
fn external_native_arcade_banks_keep_four_celadon_three_goldenrod_six_rows_and_outward_fronts() {
    let pack = crystal_assets::read_loaded_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap(),
    )
    .unwrap();
    let selected = crate::interior_models::selected_model(ModelKind::ArcadePair).unwrap();
    assert!(model_compatible(selected));
    let mut banks = 0;
    for (map, columns) in [
        ("CeladonGameCorner", vec![0, 12, 24, 36]),
        ("GoldenrodGameCorner", vec![12, 24, 36]),
    ] {
        let (cells, g) = fixture(map, &pack);
        let original = cells.clone();
        let refs = cells.iter().collect::<Vec<_>>();
        let all = resolve(map, &refs, &g, Some(built_in_profiles()));
        let ps = all
            .iter()
            .filter(|p| p.kind == ModelKind::ArcadePair && p.height == 14)
            .collect::<Vec<_>>();
        assert_eq!(
            ps.iter().map(|p| (p.column, p.row)).collect::<Vec<_>>(),
            columns.iter().map(|&x| (x, 10)).collect::<Vec<_>>()
        );
        let mut matched_stools = 0;
        for p in ps {
            assert!(source(map, p, &refs, &g));
            assert!(
                all.iter()
                    .filter(|q| matches!(q.kind, ModelKind::ArcadeMachine | ModelKind::ArcadePair))
                    .filter(|q| q
                        .indices(g.width)
                        .any(|i| p.indices(g.width).any(|j| i == j)))
                    .count()
                    == 1,
                "the header/end never add separate cabinets to a complete bank"
            );
            let mut scene = TerrainMeshData {
                footing_heights: vec![2.75; cells.len()],
                authored_cells: vec![None; cells.len()],
                ..Default::default()
            };
            let mut claims = vec![false; cells.len()];
            assert!(super::super::append(
                &mut scene,
                map,
                &refs,
                &g,
                p,
                &mut claims,
                [0, 0]
            ));
            assert_eq!(scene.footing_heights, vec![2.75; cells.len()]);
            assert_eq!(cells, original);
            assert_eq!(claims.iter().filter(|&&v| v).count(), 56);
            assert!(scene.solid.indices.is_empty());
            assert!(scene.textured.positions[..224].iter().all(|p| p[1] == 0.));
            let labels = std::array::from_fn(|i| [i as f32 / 10., 0.75]);
            let mut body = SurfaceMeshData::default();
            assert!(append(&mut body, selected, &g, p, labels));
            assert_eq!(body.positions, scene.textured.positions[224..]);
            let mut pair = SurfaceMeshData::default();
            assert!(selected.append_native_arcade(&mut pair, [0., 32., 0., 16.], 32., labels));
            assert_eq!(
                body.positions.len(),
                pair.positions.len() * 6,
                "exactly six pairs, never seven"
            );
            let (w, _, n, _) = g.bounds(p.column, p.row);
            let mut front = [[0usize; 2]; 6];
            for i in 0..body.positions.len() {
                let v = body.positions[i];
                let normal = body.normals[i];
                assert!(
                    v[0] >= w - 0.001
                        && v[0] <= w + 32.001
                        && v[1] >= -0.001
                        && v[1] <= 32.001
                        && v[2] >= n + 16. - 0.001
                        && v[2] <= n + 112. + 0.001,
                    "the source header is projected height, not another physical row"
                );
                let row = (((v[2] - n - 16.) / 16.).floor() as usize).min(5);
                let side = usize::from(v[0] > w + 16.);
                let original_i = i % pair.positions.len();
                if pair.uvs[original_i] == labels[3] && pair.normals[original_i][2] > 0.99 {
                    assert!(
                        if side == 0 {
                            normal[0] < 0.
                        } else {
                            normal[0] > 0.
                        },
                        "all reel faces point outward"
                    );
                    front[row][side] += 1;
                }
            }
            assert!(
                front.into_iter().all(|r| r.into_iter().all(|n| n > 0)),
                "one reel frontage in each of six rows on both outer aisles"
            );
            assert!(
                (body.positions.iter().map(|p| p[1]).fold(0., f32::max) - 32.).abs() < 0.001,
                "each complete cabinet is two source gameplay tiles tall"
            );
            for row in 0..6 {
                let y = p.row + 2 + row * 2;
                for x in [p.column.checked_sub(2), Some(p.column + 4)]
                    .into_iter()
                    .flatten()
                {
                    let stools = all
                        .iter()
                        .filter(|q| q.kind == ModelKind::ArcadeStool && q.column == x && q.row == y)
                        .count();
                    assert!(stools <= 1, "one stool at each source station");
                    matched_stools += stools;
                }
            }
            banks += 1;
        }
        assert_eq!(
            matched_stools,
            if map == "CeladonGameCorner" { 36 } else { 30 },
            "every source bank stool faces exactly one cabinet; table stools are separate"
        );
    }
    assert_eq!(banks, 7);
}

#[test]
fn external_native_arcade_banks_reject_changed_header_middle_end_custom_and_stale_sources() {
    let pack = crystal_assets::read_loaded_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap(),
    )
    .unwrap();
    let map = "GoldenrodGameCorner";
    let (cells, g) = fixture(map, &pack);
    let refs = cells.iter().collect::<Vec<_>>();
    let p = resolve(map, &refs, &g, None)
        .into_iter()
        .find(|p| p.kind == ModelKind::ArcadePair && p.height == 14)
        .unwrap();
    for i in p.indices(g.width) {
        let mut changed = cells.clone();
        changed[i].source.tile_index ^= 0x40;
        let refs = changed.iter().collect::<Vec<_>>();
        assert!(!source(map, &p, &refs, &g));
        let mut mesh = TerrainMeshData::default();
        let mut claims = vec![false; cells.len()];
        assert!(!super::super::append(
            &mut mesh,
            map,
            &refs,
            &g,
            &p,
            &mut claims,
            [0, 0]
        ));
        assert_eq!(mesh, TerrainMeshData::default());
        assert!(!claims.into_iter().any(|c| c));
    }
    for dy in [0, 6, 13] {
        let mut changed = cells.clone();
        changed[(p.row + dy) * g.width + p.column].source.tile_index ^= 0x40;
        let refs = changed.iter().collect::<Vec<_>>();
        let out = resolve(map, &refs, &g, None);
        assert!(
            out.iter()
                .filter(|q| matches!(q.kind, ModelKind::ArcadePair | ModelKind::ArcadeMachine))
                .all(|q| !q
                    .indices(g.width)
                    .any(|i| p.indices(g.width).any(|j| i == j))),
            "edited bank cannot fall back to separate header machines"
        );
    }
    let mut reserved = vec![false; cells.len()];
    reserved[p.row * g.width + p.column] = true;
    let mut out = Vec::new();
    candidates(map, &refs, &g, &mut reserved, &mut out);
    assert!(
        out.iter().all(|q| q.column != p.column || q.row != p.row),
        "custom-owned fragment yields the entire bank"
    );
    let selected = crate::interior_models::selected_model(ModelKind::ArcadePair).unwrap();
    let mut rejected = SurfaceMeshData::default();
    assert!(!append(
        &mut rejected,
        selected,
        &GridGeometry {
            tile_width: 0.,
            ..g
        },
        &p,
        [[0.1; 2]; 9]
    ));
    assert_eq!(
        rejected,
        SurfaceMeshData::default(),
        "invalid fit never partially appends a bank"
    );
    assert!(!source("UnreviewedArcadeRoom", &p, &refs, &g));
}
