#[test]
fn fly_destinations_use_asm_landmark_order_and_new_bark_default() {
    let destinations = [
        ("ENGINE_FLYPOINT_SILVER_CAVE", 26, "LANDMARK_SILVER_CAVE"),
        ("ENGINE_FLYPOINT_VIOLET", 16, "LANDMARK_VIOLET_CITY"),
        ("ENGINE_FLYPOINT_NEW_BARK", 14, "LANDMARK_NEW_BARK_TOWN"),
        (
            "ENGINE_FLYPOINT_INDIGO_PLATEAU",
            13,
            "LANDMARK_INDIGO_PLATEAU",
        ),
    ]
    .into_iter()
    .map(|(flypoint_flag, destination_spawn_identifier, label)| {
        RuntimeFlyDestinationKey {
            flypoint_flag: flypoint_flag.to_string(),
            destination_spawn_identifier,
            label: label.to_string(),
        }
    })
    .collect::<BTreeSet<_>>();
    let landmarks = crystal_core::models::PokegearLandmarksPayload {
        landmarks: [
            (46, "LANDMARK_SILVER_CAVE", "JOHTO"),
            (6, "LANDMARK_VIOLET_CITY", "JOHTO"),
            (1, "LANDMARK_NEW_BARK_TOWN", "JOHTO"),
            (90, "LANDMARK_INDIGO_PLATEAU", "KANTO"),
        ]
        .into_iter()
        .map(|(id, constant, region)| crystal_core::models::PokegearLandmark {
            id,
            constant: constant.to_string(),
            label: constant.to_string(),
            name: constant.to_string(),
            region: region.to_string(),
            x: 0,
            y: 0,
        })
        .collect(),
        map_to_landmark: BTreeMap::new(),
    };
    let active_flags = ["ENGINE_FLYPOINT_VIOLET".to_string()]
        .into_iter()
        .collect::<BTreeSet<_>>();

    let active =
        ordered_active_fly_destinations(destinations, &landmarks, &active_flags, false)
            .expect("exact Fly table should resolve");

    assert_eq!(
        active
            .iter()
            .map(|destination| destination.flypoint_flag.as_str())
            .collect::<Vec<_>>(),
        ["ENGINE_FLYPOINT_NEW_BARK", "ENGINE_FLYPOINT_VIOLET"]
    );
}

#[test]
fn town_map_markers_project_oam_coordinates_at_full_lcd_scale() {
    let player = crate::core::models::PokegearLandmark {
        id: 1,
        constant: "PLAYER".to_string(),
        label: "PLAYER".to_string(),
        name: "PLAYER".to_string(),
        region: "JOHTO".to_string(),
        x: 40,
        y: 56,
    };
    let cursor = crate::core::models::PokegearLandmark {
        id: 2,
        constant: "CURSOR".to_string(),
        label: "CURSOR".to_string(),
        name: "CURSOR".to_string(),
        region: "JOHTO".to_string(),
        x: 88,
        y: 96,
    };

    let rects = town_map_marker_rects(&player, &cursor);

    assert_eq!(rects.iter().filter(|rect| rect.kind == TownMapMarkerKind::Player).count(), 0);
    assert_eq!(
        rects
            .iter()
            .filter(|rect| rect.kind == TownMapMarkerKind::Cursor)
            .count(),
        8
    );
    assert_eq!(rects[0].center, Vec2::new(75.5, 73.0));
    assert_eq!(rects[0].size, Vec2::new(7.0, 2.0));
    assert_eq!(TILE_SIZE / SOURCE_TILE_SIZE as f32, 4.0);
}

#[test]
fn town_map_labels_wrap_at_the_asm_panel_width() {
    assert_eq!(town_map_label_lines("NATIONAL PARK"), ["NATIONAL", "PARK"]);
    assert_eq!(town_map_label_lines("NEW BARK TOWN"), ["NEW BARK", "TOWN"]);
    assert_eq!(town_map_label_lines("ROUTE 29"), ["ROUTE 29", ""]);
}

#[test]
fn standalone_town_map_frame_matches_the_asm_header_tiles() {
    let mut tilemap = vec![0; 20 * 18];
    apply_standalone_town_map_frame(&mut tilemap);

    assert_eq!(&tilemap[..8], &[0x06, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x17]);
    assert_eq!(tilemap[20 + 7], 0x16);
    assert_eq!(tilemap[40 + 7], 0x26);
    assert_eq!(&tilemap[40 + 8..40 + 20], &[0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x17]);
}

#[test]
fn town_map_frame_reserves_the_two_row_landmark_label_panel() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repository root");
    let asset_root = AssetRoot::new(repo_root);
    let palette_map: serde_json::Value = serde_json::from_slice(
        &crate::read_runtime_asset(
            asset_root
                .runtime_assets()
                .join("data/pokegear_town_map_palette_map.json"),
        )
        .expect("read Pokégear palette map"),
    )
    .expect("decode Pokégear palette map");
    let tokens = |key: &str| {
        palette_map[key]
            .as_array()
            .expect("palette token array")
            .iter()
            .map(|token| token.as_str().expect("palette token").to_string())
            .collect::<Vec<_>>()
    };
    let town = tokens("town_map");
    let pokegear = tokens("pokegear");
    let mut images = Assets::<Image>::default();

    let frame = load_town_map_frame(&asset_root, "johto", 0, &town, &pokegear, false, 0xf, &mut images)
        .expect("render Johto Town Map");
    let image = images.get(&frame.handle).expect("Town Map image");
    let pixel = |x: usize, y: usize| {
        let offset = (y * 160 + x) * 4;
        &image.data[offset..offset + 4]
    };
    let panel = pixel(9 * 8, 8).to_vec();

    assert!((0..16).all(|y| (9 * 8..160).all(|x| pixel(x, y) == panel)));
    assert!(
        (0..8).any(|y| (8 * 8..9 * 8).any(|x| pixel(x, y) != panel)),
        "the map-pin icon must remain at tile (8, 0)"
    );
}

#[test]
fn pokegear_rle_decoder_expands_the_authored_clock_screen() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repository root");
    let bytes =
        crate::read_runtime_asset(repo_root.join("apps/web/assets/gfx/pokegear/clock.tilemap.rle"))
            .expect("read clock tilemap");

    let tilemap = decode_pokegear_card_tilemap(&bytes).expect("decode clock tilemap");

    assert_eq!(tilemap.len(), 20 * 18);
    assert_eq!(
        &tilemap[12..20],
        &[0x30, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x31]
    );
    assert_eq!(tilemap[4 * 20 + 2], 0x06);
    assert_eq!(tilemap[10 * 20 + 2], 0x26);
}

#[test]
fn pokegear_non_map_cards_render_their_distinct_authored_layouts() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repository root");
    let asset_root = AssetRoot::new(repo_root);
    let palette_map: serde_json::Value = serde_json::from_slice(
        &crate::read_runtime_asset(
            asset_root
                .runtime_assets()
                .join("data/pokegear_town_map_palette_map.json"),
        )
        .expect("read Pokégear palette map"),
    )
    .expect("decode Pokégear palette map");
    let tokens = |key: &str| {
        palette_map[key]
            .as_array()
            .expect("palette token array")
            .iter()
            .map(|token| token.as_str().expect("palette token").to_string())
            .collect::<Vec<_>>()
    };
    let town = tokens("town_map");
    let pokegear = tokens("pokegear");
    let mut images = Assets::<Image>::default();
    let mut signatures = Vec::new();

    for page in [
        PokegearPage::Clock,
        PokegearPage::Phone,
        PokegearPage::Radio,
    ] {
        let frame = load_pokegear_card_frame(
            &asset_root,
            page,
            0,
            0b1111,
            true,
            &town,
            &pokegear,
            &mut images,
        )
        .expect("render Pokégear card");
        let image = images.get(&frame.handle).expect("Pokégear card image");
        assert_eq!((image.width(), image.height()), (160, 144));
        assert!(
            image
                .data
                .chunks_exact(4)
                .collect::<std::collections::HashSet<_>>()
                .len()
                >= 5,
            "{page:?} rendered as a flat placeholder"
        );
        if page == PokegearPage::Radio {
            let offset = (8 * 8 * 160 + 8) * 4;
            let rgb5 = image.data[offset..offset + 3].iter().map(|value| value >> 3).collect::<Vec<_>>();
            assert_eq!(rgb5, [28, 31, 20],
                "NoRadioName must clear the RLE station window to the textbox background");
        }
        signatures.push(image.data.clone());
    }

    assert_ne!(signatures[0], signatures[1]);
    assert_ne!(signatures[1], signatures[2]);
    assert_ne!(signatures[0], signatures[2]);
}

#[test]
fn town_map_player_marker_uses_one_source_sprite_scale() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.pokegear_menu_open = true;
    shell.pokegear_standalone_map = true;
    shell.pokegear_page = PokegearPage::Map;
    let snapshot = shell.shell.snapshot().unwrap();
    shell.pokegear_cursor = visible_pokegear_landmark_indices(&snapshot, true).unwrap()[0];
    let mut app = integrated_shell_test_app(shell);
    app.update();
    let world = app.world_mut();
    let marker = world.query_filtered::<(&Sprite, &Transform), With<FieldCommandMarker>>()
        .iter(world)
        .find(|(_, transform)| (transform.translation.z - 3.65).abs() < 0.001)
        .expect("Town Map player marker").0;
    assert_eq!(marker.custom_size, Some(Vec2::splat(64.0)),
        "a 16x16 source sprite must be 64x64 world units, not scaled twice");
}

#[cfg(feature = "fullscreen-scaling")]
#[test]
fn fullscreen_town_map_owns_one_centered_layer_and_restores_world() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.pokegear_menu_open = true;
    shell.pokegear_standalone_map = true;
    shell.pokegear_page = PokegearPage::Map;
    shell.pokegear_cursor = visible_pokegear_landmark_indices(&shell.shell.snapshot().unwrap(), true).unwrap()[0];
    let mut app = integrated_shell_test_app(shell);
    app.world_mut().spawn((Window {
        resolution: WindowResolution::new(1920.0, 1080.0).with_scale_factor_override(1.0),
        ..default()
    }, bevy::window::PrimaryWindow));
    app.add_systems(Startup, setup_fullscreen_scene).add_systems(PostUpdate,
        (sync_fullscreen_scaling, sync_fullscreen_scene_layout, sync_fullscreen_world_layout).chain());
    app.update();
    app.update();
    let world = app.world_mut();
    let world_visibility = world.query_filtered::<&Visibility, With<FullscreenWorldRoot>>().single(world);
    assert_eq!(*world_visibility, Visibility::Hidden, "the room must not surround the Town Map");
    let presenter_parent = world.query_filtered::<&Parent, With<VisibleIntroSurface>>().single(world).get();
    for parent in world.query_filtered::<&Parent, With<FieldCommandMarker>>().iter(world) {
        assert_eq!(parent.get(), presenter_parent, "map, label and markers must share the screen transform");
    }
    let transform = world.get::<Transform>(presenter_parent).unwrap();
    assert_eq!(transform.translation, Vec3::ZERO);
    assert!(transform.scale.x > 1.0, "the map should use the available screen height");
    world.resource_mut::<BevyRuntimeShell>().pokegear_menu_open = false;
    app.update();
    let world = app.world_mut();
    assert_eq!(*world.query_filtered::<&Visibility, With<FullscreenWorldRoot>>().single(world), Visibility::Inherited);
}
