use crate::mesh::modeled_interiors::built_in_profiles;

fn wall_sofa_fixture(
    map: &str,
    pack: &crystal_assets::LoadedCompiledGamePack,
) -> (Vec<VisualTile>, GridGeometry) {
    let data = &pack.pack().data().maps[map];
    let art = &pack.pack().runtime_files()["data/tilesets/facility_metatiles.bin"];
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
fn external_facility_wall_sofas_fit_actual_three_missing_plots_and_preserve_silph_and_support() {
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap(),
    )
    .unwrap();
    let mut total = 0;
    for (map, expected) in [
        ("MrPokemonsHouse", vec![(4, 2)]),
        ("PowerPlant", vec![(12, 2), (16, 2)]),
        ("SilphCo1F", vec![(16, 4), (20, 4), (16, 8)]),
    ] {
        let (tiles, g) = wall_sofa_fixture(map, &loaded);
        let before = tiles.clone();
        let refs = tiles.iter().collect::<Vec<_>>();
        let empty = vec![false; tiles.len()];
        let p = resolve(map, &refs, &g, [0, 0], None, &empty)
            .into_iter()
            .filter(|p| p.resolved.kind() == Kind::Sofa)
            .collect::<Vec<_>>();
        assert_eq!(
            p.iter()
                .map(|p| (p.resolved.column, p.resolved.row))
                .collect::<Vec<_>>(),
            expected
        );
        let canonical = resolve(map, &refs, &g, [0, 0], Some(built_in_profiles()), &empty);
        assert_eq!(
            canonical
                .iter()
                .filter(|p| p.resolved.kind() == Kind::Sofa)
                .count(),
            p.len(),
            "canonical fixtures must not hide the missing sofa"
        );
        for p in &p {
            let (x, y) = (p.resolved.column, p.resolved.row);
            let mut claimed = empty.clone();
            let mut mesh = TerrainMeshData {
                footing_heights: vec![2.75; tiles.len()],
                authored_cells: vec![None; tiles.len()],
                ..Default::default()
            };
            assert!(append(&mut mesh, &refs, &g, p, &mut claimed));
            assert_eq!(claimed.iter().filter(|&&c| c).count(), 8);
            assert!(mesh.solid.indices.is_empty());
            assert_eq!(mesh.footing_heights, vec![2.75; tiles.len()]);
            assert_eq!(tiles, before);
            for dy in 0..4 {
                for dx in 0..4 {
                    assert_eq!(
                        claimed[(y + dy) * g.width + x + dx],
                        dy < 2,
                        "guard floor is never owned or raised"
                    );
                }
            }
            assert_eq!(
                mesh.textured.cutaway_ranges,
                vec![32..mesh.textured.positions.len()]
            );
            let (opaque, groups) = crate::occluder_fade::prepare(mesh.textured.clone());
            assert_eq!(opaque.positions.len(), 32);
            assert_eq!(groups.len(), 1);
            let body = &mesh.textured.positions[32..];
            assert!(body.iter().all(|p| p.iter().all(|v| v.is_finite())
                && p[0] >= x as f32 * 8. + 1. - 0.001
                && p[0] <= x as f32 * 8. + 31. + 0.001
                && p[2] >= y as f32 * 8. + 1. - 0.001
                && p[2] <= y as f32 * 8. + 15. + 0.001));
            assert_eq!(body.iter().map(|p| p[1]).fold(0., f32::max), 11.);
            for t in mesh.textured.indices[48..].chunks_exact(3) {
                let [a, b, c] =
                    [0, 1, 2].map(|j| Vec3::from_array(mesh.textured.positions[t[j] as usize]));
                assert!(
                    (b - a)
                        .cross(c - a)
                        .dot(Vec3::from_array(mesh.textured.normals[t[0] as usize]))
                        > 0.
                );
            }
            total += 8;
        }
    }
    assert_eq!(total, 48, "three new sofas and three unchanged Silph sofas");
}

#[test]
fn external_facility_wall_sofas_reject_changed_clipped_reserved_and_custom_source_or_guard_floor() {
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap(),
    )
    .unwrap();
    for map in ["MrPokemonsHouse", "PowerPlant"] {
        let (tiles, g) = wall_sofa_fixture(map, &loaded);
        let refs = tiles.iter().collect::<Vec<_>>();
        let empty = vec![false; tiles.len()];
        let poses = resolve(map, &refs, &g, [0, 0], None, &empty)
            .into_iter()
            .filter(|p| p.resolved.kind() == Kind::Sofa)
            .map(|p| (p.resolved.column, p.resolved.row))
            .collect::<Vec<_>>();
        assert!(!poses.is_empty());
        for (x, y) in poses {
            for dy in 0..4 {
                for dx in 0..4 {
                    let i = (y + dy) * g.width + x + dx;
                    let mut changed = tiles.clone();
                    changed[i].source.tile_index ^= 1;
                    assert!(!resolve(
                        map,
                        &changed.iter().collect::<Vec<_>>(),
                        &g,
                        [0, 0],
                        None,
                        &empty
                    )
                    .iter()
                    .any(|p| p.resolved.kind() == Kind::Sofa
                        && p.resolved.column == x
                        && p.resolved.row == y));
                    let s = &tiles[i].source;
                    let mut custom = built_in_profiles().objects[0].clone();
                    custom.name = "Custom sofa or native guard floor".into();
                    custom.map = Some(map.into());
                    custom.maps = None;
                    custom.tileset = "facility".into();
                    custom.metatile = s.metatile_id;
                    custom.metatiles = None;
                    custom.origin = [s.subtile_column, s.subtile_row];
                    custom.tiles = vec![vec![s.tile_index]];
                    custom.ground = 1;
                    custom.parts.clear();
                    custom.footing_pixels = None;
                    assert!(!resolve(
                        map,
                        &refs,
                        &g,
                        [0, 0],
                        Some(&Document {
                            objects: vec![custom],
                            atmosphere: None
                        }),
                        &empty
                    )
                    .iter()
                    .any(|p| p.resolved.kind() == Kind::Sofa
                        && p.resolved.column == x
                        && p.resolved.row == y));
                }
            }
            let mut reserved = empty.clone();
            reserved[y * g.width + x] = true;
            assert!(!resolve(map, &refs, &g, [0, 0], None, &reserved)
                .iter()
                .any(|p| p.resolved.kind() == Kind::Sofa
                    && p.resolved.column == x
                    && p.resolved.row == y));
            let short = GridGeometry { height: y + 3, ..g };
            assert!(!resolve(
                map,
                &refs[..short.width * short.height],
                &short,
                [0, 0],
                None,
                &empty[..short.width * short.height]
            )
            .iter()
            .any(|p| p.resolved.kind() == Kind::Sofa
                && p.resolved.column == x
                && p.resolved.row == y));
        }
        assert!(
            !resolve("UnreviewedFacilityRoom", &refs, &g, [0, 0], None, &empty)
                .iter()
                .any(|p| p.resolved.kind() == Kind::Sofa)
        );
    }
}
