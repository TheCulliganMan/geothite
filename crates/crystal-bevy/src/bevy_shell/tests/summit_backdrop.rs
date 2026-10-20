#[test]
fn summit_panorama_reads_the_complete_connected_outdoors_without_changing_the_session() {
    let root = AssetRoot::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap(),
    );
    let runtime = workspace_desktop_runtime(&root);
    let spawn_identifier = runtime.title_new_game_spawn_identifier().unwrap();
    let mut shell = initialize_bevy_runtime_shell(
        root,
        runtime,
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier,
            map_name: "TinTowerRoof".into(),
            tile_x: 9,
            tile_y: 9,
        },
        BevyShellConfig {
            render_test_hour: Some(12),
            smoke_player_name: Some("TEST".into()),
            ..Default::default()
        },
    )
    .unwrap();
    let snapshot = shell.shell.snapshot().unwrap();
    let before = snapshot.state_checksum.clone();
    let mut cache = RenderedTilesetArt::default();
    let mut images = Assets::<Image>::default();
    let backdrop = summit_backdrop::source_maps(
        &snapshot,
        &shell,
        &mut cache,
        &mut images,
        &crystal_voxel_view::SummitBackdrop::default(),
    )
    .unwrap()
    .unwrap();
    let names = backdrop
        .maps
        .iter()
        .map(|m| m.frame.map_id.to_string())
        .collect::<std::collections::BTreeSet<_>>();
    let expected = (29..=46).chain([26, 27]).map(|route| format!("Route{route}"))
        .chain(["EcruteakCity", "MahoganyTown", "NewBarkTown", "AzaleaTown",
            "CherrygroveCity", "CianwoodCity", "GoldenrodCity", "LakeOfRage",
            "BlackthornCity", "VioletCity", "OlivineCity", "BattleTowerOutside"]
            .into_iter().map(str::to_owned)).collect();
    assert_eq!(names, expected, "every surrounding Johto route and its actual connected settlements");
    assert_eq!(names.len(), backdrop.maps.len(), "one placement per map");
    for source in &backdrop.maps {
        let map = snapshot.maps.iter().find(|m| m.map_name == source.frame.map_id.as_ref()).unwrap();
        for edge in &map.attributes.connections {
            let target = backdrop.maps.iter().find(|m| m.frame.map_id.as_ref() == edge.target_map)
                .expect("complete panorama contains every connection target");
            let offset = edge.offset * 4;
            let delta = match edge.direction.to_ascii_lowercase().as_str() {
                "north" => IVec2::new(offset, -(target.frame.grid_size.y as i32)),
                "south" => IVec2::new(offset, source.frame.grid_size.y as i32),
                "west" => IVec2::new(-(target.frame.grid_size.x as i32), offset),
                "east" => IVec2::new(source.frame.grid_size.x as i32, offset),
                _ => continue,
            };
            assert_eq!(target.origin_source, source.origin_source + delta,
                "authored connection {} -> {} must meet at the correct seam", map.map_name, edge.target_map);
        }
    }
    assert_eq!(
        backdrop.tower_core,
        IVec2::new(37, 7),
        "align with the actual tower, not the west entrance hall"
    );
    for map in &backdrop.maps {
        let actual = snapshot
            .maps
            .iter()
            .find(|m| m.map_name == map.frame.map_id.as_ref())
            .unwrap();
        let w = actual.attributes.width as usize * 4;
        let h = actual.attributes.height as usize * 4;
        assert_eq!(map.frame.tiles.len(), w * h);
        assert_eq!(
            map.frame.tile_size,
            Vec2::splat(TILE_SIZE),
            "native play and distant maps share a scale"
        );
        assert_eq!(map.frame.grid_size, UVec2::new(w as u32, h as u32));
        for tile in &map.frame.tiles {
            let (x, y) = (tile.column as usize, tile.row as usize);
            assert_eq!(
                tile.source.metatile_id,
                actual.blocks[y / 4 * (w / 4) + x / 4],
                "{} source block at {x},{y}",
                actual.map_name
            );
            assert_eq!(
                tile.source.tileset_id.as_ref(),
                actual.attributes.tileset_name
            );
            assert_eq!(tile.source.subtile_column, x as u8 % 4);
            assert_eq!(tile.source.subtile_row, y as u8 % 4);
            assert!(
                images.get(&tile.texture).is_some(),
                "every native model has real palette-colored source samples"
            );
        }
        assert!(
            map.frame.actors.is_empty(),
            "the panorama must not invent or drive another player/NPC session"
        );
        let atlas = images.get(&map.frame.map_texture).unwrap();
        assert_eq!(atlas.width(), w as u32 * 8);
        assert_eq!(atlas.height(), h as u32 * 8);
    }
    let border = backdrop
        .border
        .as_ref()
        .expect("real source border continuation");
    let city = snapshot
        .maps
        .iter()
        .find(|m| m.map_name == "EcruteakCity")
        .unwrap();
    assert_eq!(border.grid_size, UVec2::new(5, 4));
    assert_eq!(border.tiles.len(), 20);
    for tile in border.tiles.iter().filter(|t| t.column < 4) {
        assert_eq!(
            tile.source.metatile_id,
            u16::from(city.attributes.border_block)
        );
        assert_eq!(
            tile.source.tileset_id.as_ref(),
            city.attributes.tileset_name
        );
        assert_eq!(tile.source.subtile_column, tile.column as u8);
        assert_eq!(tile.source.subtile_row, tile.row as u8);
        assert!(images.get(&tile.texture).is_some());
    }
    let built = crystal_voxel_view::audit_terrain_mesh_with_images(border, &images).unwrap();
    assert!(
        built.authored_cells.iter().filter(|c| c.is_some()).count() >= 16,
        "the actual border must become native trees rather than flattened sprites"
    );
    let image_count = images.len();
    assert!(
        summit_backdrop::source_maps(&snapshot, &shell, &mut cache, &mut images, &backdrop)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        images.len(),
        image_count,
        "idle extraction must not rebuild the panorama atlases"
    );
    assert_eq!(shell.shell.snapshot().unwrap().state_checksum, before);
}
