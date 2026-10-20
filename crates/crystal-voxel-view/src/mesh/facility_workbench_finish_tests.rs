fn finish_workbench_fixture(
    map: &str,
    loaded: &crystal_assets::LoadedCompiledGamePack,
) -> (Vec<VisualTile>, GridGeometry) {
    let pack = loaded.pack();
    let data = &pack.data().maps[map];
    let art = &pack.runtime_files()["data/tilesets/facility_metatiles.bin"];
    let (w, h) = (
        data.attributes.width as usize * 4,
        data.attributes.height as usize * 4,
    );
    let tiles = (0..w * h)
        .map(|i| {
            let (x, y) = (i % w, i / w);
            let b = data.blocks[y / 4 * (w / 4) + x / 4];
            VisualTile {
                column: x as u32,
                row: y as u32,
                texture: Default::default(),
                priority: false,
                animation_frames: None,
                source: VisualTileSource {
                    tileset_id: "facility".into(),
                    metatile_id: b,
                    subtile_column: (x % 4) as u8,
                    subtile_row: (y % 4) as u8,
                    tile_index: art[b as usize * 16 + y % 4 * 4 + x % 4] as u16,
                },
            }
        })
        .collect();
    (
        tiles,
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
fn external_facility_workbench_finish_keeps_all_ten_native_bodies_readouts_and_floor() {
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap(),
    )
    .unwrap();
    let png = image::load_from_memory(&loaded.pack().runtime_files()["gfx/tilesets/facility.png"])
        .unwrap()
        .to_luma8();
    let mut count = 0;
    for (map, expected) in [
        ("MrPokemonsHouse", 1),
        ("PowerPlant", 3),
        ("RuinsOfAlphResearchCenter", 1),
        ("TeamRocketBaseB3F", 5),
    ] {
        let (tiles, g) = finish_workbench_fixture(map, &loaded);
        let original = tiles.clone();
        let refs = tiles.iter().collect::<Vec<_>>();
        let empty = vec![false; tiles.len()];
        let ps = resolve(
            map,
            &refs,
            &g,
            [0, 0],
            Some(super::super::modeled_interiors::built_in_profiles()),
            &empty,
        );
        let ps = ps
            .iter()
            .filter(|p| p.resolved.kind() == Kind::Workbench)
            .collect::<Vec<_>>();
        assert_eq!(
            ps.len(),
            expected,
            "{map}: every source bench resolves under canonical profiles"
        );
        for p in ps {
            let mut mesh = TerrainMeshData {
                footing_heights: vec![2.75; tiles.len()],
                authored_cells: vec![None; tiles.len()],
                ..Default::default()
            };
            let mut claimed = empty.clone();
            assert!(append(&mut mesh, &refs, &g, p, &mut claimed));
            assert_eq!(claimed.iter().filter(|&&v| v).count(), 12);
            assert_eq!(mesh.footing_heights, vec![2.75; tiles.len()]);
            assert_eq!(tiles, original);
            assert!(mesh.solid.indices.is_empty());
            let (w, _, n, _) = g.bounds(p.resolved.column, p.resolved.row);
            let first=48;let end=mesh.textured.positions.len()-8;
            assert!(end>first);
            assert!(mesh.textured.positions[..first].iter().all(|v|v[1]==0.));
            assert!(mesh.textured.positions[first..end].iter().any(|v|v[0]<w+16.&&v[1]>16.),
                "{map}: retain the original upright monitor");
            assert!(mesh.textured.positions[first..end].iter().zip(&mesh.textured.normals[first..end])
                .any(|(v,n)|v[0]>w+18.&&v[1]>11.&&n[0]< -0.4&&n[2]>0.1),
                "{map}: right enclosure faces its keyboard/operator");
            assert!(mesh.textured.positions[end..].iter().all(|v|
                v[1]>11.&&v[1]<17.&&(v[2]-(n+14.2)).abs()<0.001),
                "{map}: live source display keeps its original monitor face");
            assert!(mesh.textured.normals[end..].iter().all(|n|*n==[0.,0.,1.]));
            let swatches = workbench_swatches(&g, &p.resolved);
            for uv in swatches {
                assert!(mesh.textured.uvs[first..end].contains(&uv));
            }
            for (pixel, gray) in [([4, 14], 170), ([4, 16], 85), ([4, 1], 255), ([4, 17], 0)] {
                let s = &tiles
                    [(p.resolved.row + pixel[1] / 8) * g.width + p.resolved.column + pixel[0] / 8]
                    .source;
                assert_eq!(
                    png.get_pixel(
                        u32::from(s.tile_index % 16) * 8 + pixel[0] as u32 % 8,
                        u32::from(s.tile_index / 16) * 8 + pixel[1] as u32 % 8
                    )[0],
                    gray,
                    "{map}: live swatch must actually represent wood/brown/light/ink in the pack"
                );
            }
            for (uv, (color, normal)) in mesh.textured.uvs[first..end].iter().zip(
                mesh.textured.colors[first..end]
                    .iter()
                    .zip(&mesh.textured.normals[first..end]),
            ) {
                assert!(swatches.contains(uv));
                assert_eq!(
                    *color,
                    crate::interior_models::authored_face_color([1.; 4], Vec3::from_array(*normal))
                );
            }
            count += 1;
        }
    }
    assert_eq!(count, 10);
}

#[test]
fn facility_workbench_finish_rejects_unknown_or_incomplete_parts_and_preserves_cached_mesh() {
    let raw =
        crate::model_storage::include_model!("models/facility_radio/facility_workbench.mesh.json");
    let parsed: serde_json::Value = crate::model_storage::parse(raw).unwrap();
    for mode in 0..4 {
        let mut bad = parsed.clone();
        match mode {
            0 => bad["name"] = "Unreviewed bench".into(),
            1 => bad["primitives"][0]["name"] = "Unreviewed part".into(),
            2 => bad["primitives"][0]["material"] = "Unreviewed material".into(),
            _ => bad["primitives"].as_array_mut().unwrap().retain(|p| {
                let n = p["name"].as_str().unwrap();
                !matches!(
                    n,
                    "Laboratory bench folded front edge"
                        | "Keyboard raised cap"
                        | "Left terminal narrow pale lip"
                        | "Right instrument narrow status marker"
                )
            }),
        }
        let unknown = Model::parse(&bad.to_string()).unwrap();
        let mut out = SurfaceMeshData::default();
        assert!(!unknown.has_native_facility_furniture_materials());
        assert!(!unknown.append_native_facility_furniture(
            &mut out,
            [0., 32., 6., 24.],
            18.,
            [[0.1; 2]; 4]
        ));
        assert_eq!(out, SurfaceMeshData::default());
    }
    for (sx, sy) in [(1., 1.), (0.5, 2.), (2., 0.5)] {
        let b = [0., 32. * sx, 6. * sy, 24. * sy];
        let mut old = SurfaceMeshData::default();
        model(0).append_fitted(&mut old, b, 0., 18. * sy);
        let mut native = SurfaceMeshData::default();
        assert!(model(0).append_native_facility_furniture(&mut native, b, 18. * sy, [[0.1; 2]; 4]));
        assert_eq!(old.positions, native.positions);
        assert_eq!(old.normals, native.normals);
        assert_eq!(old.indices, native.indices);
        let mut after = SurfaceMeshData::default();
        model(0).append_fitted(&mut after, b, 0., 18. * sy);
        assert_eq!(old, after);
    }
}
