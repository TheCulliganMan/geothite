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
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let runtime = workspace_desktop_runtime(&AssetRoot::new(repo_root));
    let asset_root = runtime.materialize_runtime_files().expect("mount external pack artwork");
    let palette_map = &runtime.data().pokegear_town_map_palette_map;
    let town = palette_map.get("town_map").expect("Town Map palette tokens");
    let pokegear = palette_map.get("pokegear").expect("Pokégear palette tokens");
    let mut images = Assets::<Image>::default();

    let frame = load_town_map_frame(&asset_root, "johto", 0, town, pokegear, false, 0xf, &mut images)
        .expect("render Johto Town Map");
    let image = images.get(&frame.handle).expect("Town Map image");
    let pixel = |x: usize, y: usize| {
        let offset = (y * 160 + x) * 4;
        &image.data.as_ref().expect("CPU image pixels")[offset..offset + 4]
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
                .data.as_ref().expect("CPU image pixels")
                .chunks_exact(4)
                .collect::<std::collections::HashSet<_>>()
                .len()
                >= 5,
            "{page:?} rendered as a flat placeholder"
        );
        if page == PokegearPage::Radio {
            let offset = (8 * 8 * 160 + 8) * 4;
            let rgb5 = image.data.as_ref().expect("CPU image pixels")[offset..offset + 3].iter().map(|value| value >> 3).collect::<Vec<_>>();
            assert_eq!(rgb5, [28, 31, 20],
                "NoRadioName must clear the RLE station window to the textbox background");
        }
        signatures.push(image.data.as_ref().expect("CPU image pixels").clone());
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
        resolution: WindowResolution::new(1920, 1080).with_scale_factor_override(1.0),
        ..default()
    }, bevy::window::PrimaryWindow));
    app.add_systems(Startup, setup_fullscreen_scene).add_systems(PostUpdate,
        (sync_fullscreen_scaling, sync_fullscreen_scene_layout, sync_fullscreen_world_layout).chain());
    app.update();
    app.update();
    let world = app.world_mut();
    let world_visibility = world.query_filtered::<&Visibility, With<FullscreenWorldRoot>>().single(world).unwrap();
    assert_eq!(*world_visibility, Visibility::Hidden, "the room must not surround the Town Map");
    let presenter_parent = world.query_filtered::<&ChildOf, With<VisibleIntroSurface>>().single(world).unwrap().parent();
    for parent in world.query_filtered::<&ChildOf, With<FieldCommandMarker>>().iter(world) {
        assert_eq!(parent.parent(), presenter_parent, "map, label and markers must share the screen transform");
    }
    let transform = world.get::<Transform>(presenter_parent).unwrap();
    assert_eq!(transform.translation, Vec3::ZERO);
    assert!(transform.scale.x > 1.0, "the map should use the available screen height");
    world.resource_mut::<BevyRuntimeShell>().pokegear_menu_open = false;
    app.update();
    let world = app.world_mut();
    assert_eq!(*world.query_filtered::<&Visibility, With<FullscreenWorldRoot>>().single(world).unwrap(), Visibility::Inherited);
}

#[test]
#[ignore = "requires an external Minnesota pack and render directory"]
fn minnesota_town_map_and_fly_share_one_screen_art_and_positions() {
    let path = PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        crystal_assets::read_loaded_verified_compiled_game_pack(&path).expect("verified pack"),
    )
    .expect("runtime");
    let destination = runtime
        .data()
        .fly_destinations
        .get("ENGINE_FLYPOINT_MN_WINONA")
        .expect("registered Winona landing");
    let landing =
        &runtime.data().runtime_spawn_points[&destination.destination_spawn_identifier.to_string()];
    let mut shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("home"),
            map_name: "MnWinona".into(),
            tile_x: landing.tile_x,
            tile_y: landing.tile_y,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..default()
        },
    )
    .expect("explicit atlas fixture");
    complete_visible_smoke_player_name_if_needed(&mut shell, Some("CHRIS")).expect("name");
    let snapshot = shell.shell.snapshot().expect("snapshot");
    assert!(minnesota_map_active(&snapshot));
    let indices = visible_pokegear_landmark_indices(&snapshot, true).expect("regional cursor");
    assert!(indices.iter().all(|index| {
        let landmark = &snapshot.presentation.pokegear_landmarks.landmarks[*index];
        let projected = projected_minnesota_landmark(&snapshot, landmark);
        (16..=151).contains(&(projected.x - 8)) && (32..=119).contains(&(projected.y - 16))
    }));
    shell.pokegear_menu_open = true;
    shell.pokegear_standalone_map = true;
    shell.pokegear_page = PokegearPage::Map;
    shell.pokegear_cursor = indices[0];
    let mut reached = BTreeSet::new();
    for _ in 0..indices.len() {
        reached.insert(shell.pokegear_cursor);
        move_visible_pokegear_cursor(&mut shell, -1).expect("native regional cursor input");
        assert!(
            indices.contains(&shell.pokegear_cursor),
            "cursor escaped into Johto"
        );
    }
    assert_eq!(reached.len(), indices.len());
    assert_eq!(shell.pokegear_cursor, indices[0]);
    shell.pokegear_cursor = indices
        .into_iter()
        .find(|index| {
            projected_minnesota_landmark(
                &snapshot,
                &snapshot.presentation.pokegear_landmarks.landmarks[*index],
            )
            .name
                == "WINONA"
        })
        .expect("Winona cursor");
    let mut app = integrated_shell_test_app(shell);
    app.update();
    save_minnesota_screen_sprites(app.world_mut(), "town-map.png");
    for (town, filename) in [
        ("GRAND MARAIS", "town-map-north-shore.png"),
        ("INTERNATIONAL FALLS", "town-map-border.png"),
    ] {
        let index = visible_pokegear_landmark_indices(&snapshot, true)
            .expect("regional cursor")
            .into_iter()
            .find(|index| {
                projected_minnesota_landmark(
                    &snapshot,
                    &snapshot.presentation.pokegear_landmarks.landmarks[*index],
                )
                .name
                    == town
            })
            .or_else(|| {
                (town == "INTERNATIONAL FALLS")
                    .then(|| {
                        visible_pokegear_landmark_indices(&snapshot, true)
                            .expect("regional cursor")
                            .into_iter()
                            .find(|index| {
                                snapshot.presentation.pokegear_landmarks.landmarks[*index].constant
                                    == "LANDMARK_MN_BORDER_PINES"
                            })
                    })
                    .flatten()
            })
            .expect("installed northern location");
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            shell.pokegear_cursor = index;
            mark_runtime_presentation_dirty(&mut shell);
        }
        app.update();
        save_minnesota_screen_sprites(app.world_mut(), filename);
    }
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        shell.pokegear_menu_open = false;
        shell.party_menu_open = true;
        let flags = runtime
            .data()
            .fly_destinations
            .keys()
            .filter(|flag| flag.starts_with("ENGINE_FLYPOINT_MN_"))
            .cloned()
            .collect::<Vec<_>>();
        assert!(flags.len() >= 15, "use a completed regional HM pack");
        for flag in flags {
            shell
                .shell
                .session_mut()
                .state_mut()
                .flags
                .set_engine_flag(&flag, true)
                .expect("explicit visited-town fixture");
        }
        let bird = runtime
            .data()
            .create_pokemon("LOOBIT", 40, crate::core::models::Dv::default())
            .expect("regional Fly bird fixture");
        let state = shell.shell.session_mut().state_mut();
        assert!(state.storage.party.add_pokemon(bird));
        state.sync_party_from_storage();
        state.badges.johto[4] = true;
        state
            .flags
            .set_engine_flag("ENGINE_STORMBADGE", true)
            .expect("Wind badge fixture");
        shell
            .shell
            .add_bag_item("HM_FLY", 1)
            .expect("owned Fly HM fixture");
        shell
            .shell
            .use_bag_tmhm_on_party_pokemon("HM_FLY", 0, Some(0))
            .expect("teach compatible regional bird");
        mark_runtime_snapshot_dirty(&mut shell);
        let snapshot = shell.shell.snapshot().expect("visited towns");
        let destinations = active_fly_destinations(&snapshot, &shell.shell).expect("regional Fly");
        assert!(
            destinations
                .iter()
                .all(|d| d.flypoint_flag.starts_with("ENGINE_FLYPOINT_MN_"))
        );
        for destination in &destinations {
            let landmark = snapshot
                .presentation
                .pokegear_landmarks
                .landmarks
                .iter()
                .find(|landmark| landmark.constant == destination.label)
                .expect("Fly landmark");
            let projected = projected_minnesota_landmark(&snapshot, landmark);
            assert_eq!(
                Some((projected.x - 8, projected.y - 16)),
                minnesota_town_map::town_position(&fly_destination_label(destination)),
                "Fly target must align with its native town square: {}",
                destination.flypoint_flag
            );
        }
        open_visible_fly_destination_menu(&mut shell).expect("production Fly chooser");
    }
    app.update();
    save_minnesota_screen_sprites(app.world_mut(), "fly-map.png");
    let shell = app
        .world_mut()
        .remove_resource::<BevyRuntimeShell>()
        .expect("rendered session");
    let mut controller = VisibleShellController { shell };
    let target = "ENGINE_FLYPOINT_MN_GRAND_MARAIS";
    for _ in 0..32 {
        let snapshot = controller.snapshot().expect("Fly selection");
        let destinations =
            active_fly_destinations(&snapshot, &controller.shell.shell).expect("Fly entries");
        let selected = controller
            .shell
            .fly_cursor
            .as_ref()
            .expect("Fly cursor")
            .option_index;
        if destinations[selected].flypoint_flag == target {
            break;
        }
        controller
            .press(GameButton::Down)
            .expect("native Fly cursor input");
    }
    let snapshot = controller.snapshot().expect("selected landing");
    let destinations =
        active_fly_destinations(&snapshot, &controller.shell.shell).expect("Fly entries");
    assert_eq!(
        destinations[controller
            .shell
            .fly_cursor
            .as_ref()
            .expect("cursor")
            .option_index]
            .flypoint_flag,
        target
    );
    app.world_mut().insert_resource(controller.shell);
    app.update();
    save_minnesota_screen_sprites(app.world_mut(), "fly-map-north-shore.png");
    controller = VisibleShellController {
        shell: app
            .world_mut()
            .remove_resource::<BevyRuntimeShell>()
            .expect("selected flight session"),
    };
    controller
        .press(GameButton::A)
        .expect("confirm actual flight");
    assert!(
        controller.shell.visible_fly_animation.is_some(),
        "native departure"
    );
    let departure = controller.snapshot().expect("departure").overworld.tile;
    controller.press(GameButton::Right).expect("Fly owns input");
    assert_eq!(
        controller.snapshot().expect("owned input").overworld.tile,
        departure
    );
    for _ in 0..64 {
        let snapshot = controller.snapshot().expect("flight");
        if snapshot.overworld.map_name == "MnGrandMarais"
            && controller.shell.visible_fly_animation.is_none()
        {
            break;
        }
        if snapshot.ui.text_window_open {
            controller
                .press(GameButton::A)
                .expect("native flight dialogue");
        } else {
            controller
                .wait_frames(16)
                .expect("production flight frames");
        }
    }
    let landed = controller.snapshot().expect("actual arrival");
    let destination = &runtime.data().fly_destinations[target];
    let landing =
        &runtime.data().runtime_spawn_points[&destination.destination_spawn_identifier.to_string()];
    assert_eq!(landed.overworld.map_name, "MnGrandMarais");
    assert_eq!(
        (landed.overworld.tile.x, landed.overworld.tile.y),
        (landing.tile_x, landing.tile_y)
    );
}

/// Compose the actual production sprites offscreen at native LCD resolution.
/// This checks the real UI artwork, labels and markers without opening a window.
fn save_minnesota_screen_sprites(world: &mut World, filename: &str) {
    assert!(
        world.resource::<BevyRuntimeShell>().last_error.is_none(),
        "production render error: {:?}",
        world.resource::<BevyRuntimeShell>().last_error
    );
    assert!(
        world
            .resource::<RenderedTilesetArt>()
            .town_map_cache
            .keys()
            .any(|(key, _)| key.contains(":minnesota:")),
        "native atlas was not presented"
    );
    let mut sprites = world
        .query::<(&Sprite, &Transform)>()
        .iter(world)
        .filter(|(_, transform)| transform.translation.z >= 3.4)
        .map(|(sprite, transform)| (sprite.clone(), *transform))
        .collect::<Vec<_>>();
    sprites.sort_by(|a, b| a.1.translation.z.total_cmp(&b.1.translation.z));
    let images = world.resource::<Assets<Image>>();
    let scale = TILE_SIZE / SOURCE_TILE_SIZE as f32;
    let mut output = image::RgbaImage::from_pixel(160, 144, image::Rgba([232, 240, 208, 255]));
    for (sprite, transform) in sprites {
        let image = images.get(&sprite.image);
        let size = sprite.custom_size.unwrap_or_else(|| {
            image.map_or(Vec2::ONE, |image| {
                Vec2::new(image.width() as f32, image.height() as f32)
            })
        });
        let width = (size.x / scale).round() as i32;
        let height = (size.y / scale).round() as i32;
        let left = ((transform.translation.x - PLAYFIELD_LEFT) / scale - width as f32 / 2.0).round()
            as i32;
        let top = ((PLAYFIELD_TOP - transform.translation.y) / scale - height as f32 / 2.0).round()
            as i32;
        for y in 0..height {
            for x in 0..width {
                if !(0..160).contains(&(left + x)) || !(0..144).contains(&(top + y)) {
                    continue;
                }
                let pixel = if let Some(image) = image {
                    let Some(data) = image.data.as_ref() else {
                        continue;
                    };
                    if data.len() != image.width() as usize * image.height() as usize * 4 {
                        continue;
                    }
                    let sx = x as u32 * image.width() / width as u32;
                    let sy = y as u32 * image.height() / height as u32;
                    let offset = ((sy * image.width() + sx) * 4) as usize;
                    <[u8; 4]>::try_from(&data[offset..offset + 4]).expect("RGBA")
                } else {
                    sprite.color.to_srgba().to_u8_array()
                };
                if pixel[3] > 0 {
                    output.put_pixel((left + x) as u32, (top + y) as u32, image::Rgba(pixel));
                }
            }
        }
    }
    let directory =
        PathBuf::from(std::env::var_os("CRYSTAL_TOWN_MAP_RENDER_DIR").expect("render directory"));
    std::fs::create_dir_all(&directory).expect("render directory");
    output
        .save(directory.join(filename))
        .expect("save production sprites");
    image::imageops::resize(&output, 640, 576, image::imageops::FilterType::Nearest)
        .save(directory.join(filename.replace(".png", "-4x.png")))
        .expect("save enlarged production sprites");
}
