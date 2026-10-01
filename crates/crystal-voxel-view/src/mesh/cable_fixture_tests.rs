//! Regression coverage for the actual sign/door split and beta-room reuse.
use super::*;
use std::sync::Arc;
fn fixture(name: &str) -> (Object, Vec<VisualTile>, GridGeometry) {
    let object = built_in_profiles()
        .objects
        .iter()
        .find(|o| o.name == name)
        .unwrap()
        .clone();
    let (width, height) = (object.tiles[0].len() + 2, object.tiles.len() + 2);
    let g = GridGeometry {
        width,
        height,
        tile_width: 8.,
        tile_height: 8.,
        origin_x: 0.,
        origin_z: 0.,
    };
    let mut cells = (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .map(|(x, y)| VisualTile {
            column: x as u32,
            row: y as u32,
            texture: Default::default(),
            priority: false,
            source: VisualTileSource {
                tileset_id: Arc::from("pokecenter"),
                metatile_id: 4,
                subtile_column: (x % 4) as u8,
                subtile_row: (y % 4) as u8,
                tile_index: 0x11,
            },
        })
        .collect::<Vec<_>>();
    for y in 0..object.tiles.len() {
        for x in 0..object.tiles[0].len() {
            let s = &mut cells[y * width + x].source;
            s.metatile_id = object.metatiles.as_ref().map_or(object.metatile, |bs| {
                bs[(object.origin[1] as usize + y) / 4][(object.origin[0] as usize + x) / 4]
            });
            s.subtile_column = ((object.origin[0] as usize + x) % 4) as u8;
            s.subtile_row = ((object.origin[1] as usize + y) % 4) as u8;
            s.tile_index = object.tiles[y][x];
        }
    }
    (object, cells, g)
}
#[test]
fn upstairs_reuses_exact_pc_and_sign_models_in_all_inspected_rooms() {
    let maps = [
        "Pokecenter2F",
        "CeladonPokecenter2FBeta",
        "CeruleanPokecenter2FBeta",
        "CinnabarPokecenter2FBeta",
        "FuchsiaPokecenter2FBeta",
        "LavenderPokecenter2FBeta",
        "PewterPokecenter2FBeta",
        "Route10Pokecenter2FBeta",
        "SaffronPokecenter2FBeta",
        "VermilionPokecenter2FBeta",
        "ViridianPokecenter2FBeta",
    ];
    for (name, kind, count) in [
        ("Pokecenter upstairs complete PC", ModelKind::Computer, 6),
        (
            "Pokecenter upstairs wall displays 2b",
            ModelKind::PictureFrame,
            4,
        ),
        (
            "Pokecenter upstairs wall displays 0a",
            ModelKind::PictureFrame,
            4,
        ),
    ] {
        let (_, cells, g) = fixture(name);
        let refs = cells.iter().collect::<Vec<_>>();
        for map in maps {
            let found = resolve(map, &refs, &g, None);
            let found = found.iter().filter(|p| p.kind == kind).collect::<Vec<_>>();
            assert_eq!(found.len(), 1, "{map}: {name}");
            assert_eq!(found[0].indices(g.width).count(), count);
            if kind == ModelKind::PictureFrame {
                assert_eq!(found[0].width, 2);
                assert!(found[0].indices(g.width).all(|i| i % g.width < 2));
            }
            let mut mesh = TerrainMeshData::default();
            mesh.footing_heights = vec![3.25; cells.len()];
            mesh.authored_cells = vec![None; cells.len()];
            let mut claimed = vec![false; cells.len()];
            assert!(append(&mut mesh, &refs, &g, found[0], &mut claimed));
            assert!(mesh.footing_heights.iter().all(|&h| h == 3.25));
            assert_eq!(claimed.iter().filter(|&&c| c).count(), count);
            if kind == ModelKind::PictureFrame {
                assert!(
                    !claimed[2] && !claimed[3] && !claimed[g.width + 2] && !claimed[g.width + 3]
                );
            }
        }
        for map in [
            "OtherPokecenter2FBeta",
            "GoldenrodPokecenter1F",
            "Pokecenter2FCustom",
        ] {
            assert!(!resolve(map, &refs, &g, None).iter().any(|p| p.kind == kind));
        }
    }
}
#[test]
fn upstairs_reuse_rejects_changed_source_phase_guard_door_and_missing_floor() {
    for name in [
        "Pokecenter upstairs complete PC",
        "Pokecenter upstairs wall displays 2b",
        "Pokecenter upstairs wall displays 0a",
    ] {
        let (o, cells, g) = fixture(name);
        let kind = profile_kind(&o).unwrap();
        for y in 0..o.tiles.len() {
            for x in 0..o.tiles[0].len() {
                for field in 0..5 {
                    let mut changed = cells.clone();
                    let s = &mut changed[y * g.width + x].source;
                    match field {
                        0 => s.tileset_id = Arc::from("mart"),
                        1 => s.metatile_id ^= 1,
                        2 => s.subtile_column ^= 1,
                        3 => s.subtile_row ^= 1,
                        _ => s.tile_index ^= 1,
                    }
                    assert!(
                        !resolve(
                            "CeladonPokecenter2FBeta",
                            &changed.iter().collect::<Vec<_>>(),
                            &g,
                            None
                        )
                        .iter()
                        .any(|p| p.kind == kind),
                        "{name}, {x},{y}, {field}"
                    );
                }
            }
        }
        let mut missing = cells.clone();
        for c in &mut missing {
            if c.source.tile_index == 0x11 {
                c.source.tile_index = u16::MAX;
            }
        }
        assert!(
            !resolve(
                "CeladonPokecenter2FBeta",
                &missing.iter().collect::<Vec<_>>(),
                &g,
                None
            )
            .iter()
            .any(|p| p.kind == kind)
        );
        let narrow = GridGeometry {
            width: o.tiles[0].len() - 1,
            ..g
        };
        let clipped = cells
            .iter()
            .take(narrow.width * narrow.height)
            .collect::<Vec<_>>();
        assert!(
            !resolve("CeladonPokecenter2FBeta", &clipped, &narrow, None)
                .iter()
                .any(|p| p.kind == kind)
        );
    }
}
#[test]
fn upstairs_custom_profiles_keep_ownership_over_source_fixture_and_door_guard() {
    for name in [
        "Pokecenter upstairs complete PC",
        "Pokecenter upstairs wall displays 2b",
    ] {
        let (mut o, cells, g) = fixture(name);
        let kind = profile_kind(&o).unwrap();
        o.map = Some("CeladonPokecenter2FBeta".into());
        o.name = "A user's exact upstairs fixture".into();
        let doc = Document {
            objects: vec![o],
            atmosphere: None,
        };
        assert!(
            !resolve(
                "CeladonPokecenter2FBeta",
                &cells.iter().collect::<Vec<_>>(),
                &g,
                Some(&doc)
            )
            .iter()
            .any(|p| p.kind == kind)
        );
    }
}
