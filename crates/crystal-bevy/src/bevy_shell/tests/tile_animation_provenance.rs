// These fixtures execute the production compositor and frame publisher with
// external cave art. Only the read-only presentation snapshot is arranged to
// place the requested authored metatile inside even the small classic grid.
fn tile_animation_provenance_app(tile_index: u8) -> App {
    let asset_root = AssetRoot::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap(),
    );
    let runtime = workspace_desktop_runtime(&asset_root);
    let spawn_identifier = runtime.title_new_game_spawn_identifier().unwrap();
    let mut shell = initialize_bevy_runtime_shell(
        asset_root,
        runtime,
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier,
            map_name: "UnionCave1F".into(),
            tile_x: 9,
            tile_y: 26,
        },
        BevyShellConfig {
            smoke_player_name: Some("TEST".into()),
            ..Default::default()
        },
    )
    .unwrap();
    complete_visible_smoke_player_name_if_needed(&mut shell, Some("TEST")).unwrap();
    settle_visible_shell_smoke_until_idle(&mut shell).unwrap();
    let mut app = battle_render_regression_app(shell);
    app.init_resource::<crystal_render_api::VisualWorldFrame>()
        .add_systems(Update, publish_visual_world_frame.after(render_playfield));
    app.update();
    assert_eq!(app.world().resource::<BevyRuntimeShell>().last_error, None);
    let (metatile, static_metatile) = {
        let art = &app
            .world()
            .resource::<RenderedTilesetArt>()
            .cache
            .iter()
            .find(|(key, _)| key.tileset_id == "cave")
            .unwrap()
            .1;
        let animated =
            art.metatile_layout
                .chunks_exact(METATILE_TILE_COUNT)
                .position(|tiles| tiles.contains(&tile_index))
                .expect("cave art contains the requested authored animation") as u16;
        let static_tile = art
            .metatile_layout
            .chunks_exact(METATILE_TILE_COUNT)
            .position(|tiles| {
                tiles
                    .iter()
                    .all(|tile| !art.animated_tiles.contains_key(&usize::from(*tile)))
            })
            .expect("cave art contains a static metatile") as u16;
        (animated, static_tile)
    };
    tile_animation_edit_presented_snapshot(&mut app, |snapshot| {
        let map = Arc::make_mut(
            snapshot
                .maps
                .iter_mut()
                .find(|map| map.map_name == "UnionCave1F")
                .unwrap(),
        );
        for (index, block) in map.blocks.iter_mut().enumerate() {
            *block = if index % 2 == 0 {
                metatile
            } else {
                static_metatile
            };
        }
        // Exercise the same producer cache for explicit palette/time variants.
        map.attributes.time_of_day = Some("day".into());
    });
    tile_animation_render_phase(&mut app, 0);
    app
}

fn tile_animation_edit_presented_snapshot(
    app: &mut App,
    edit: impl FnOnce(&mut RuntimeShellSnapshot),
) {
    let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
    let mut snapshot = cached_runtime_snapshot(&mut shell).unwrap();
    edit(Arc::make_mut(&mut snapshot));
    shell.snapshot_revision = shell.snapshot_revision.wrapping_add(1);
    shell.cached_snapshot = Some((shell.snapshot_revision, snapshot));
}

fn tile_animation_render_phase(app: &mut App, frame: u64) {
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        shell.lcd_animation_frame = frame;
        mark_runtime_presentation_dirty(&mut shell);
    }
    app.world_mut()
        .resource_mut::<RenderedViewport>()
        .shell_render_key = None;
    app.update();
    assert_eq!(app.world().resource::<BevyRuntimeShell>().last_error, None);
    assert_eq!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .lcd_animation_frame,
        frame,
        "rendering must not advance the source animation clock"
    );
    let published = app
        .world()
        .resource::<crystal_render_api::VisualWorldFrame>();
    assert!(published.active);
    assert_eq!(published.validate(), Ok(()));
}

fn tile_animation_emitted_family(app: &App, tile_index: u16) -> Arc<[Handle<Image>]> {
    let tiles = &app.world().resource::<RenderedViewport>().visual_tiles;
    let family = tiles
        .iter()
        .find(|tile| {
            tile.source.tileset_id.as_ref() == "cave" && tile.source.tile_index == tile_index
        })
        .unwrap()
        .animation_frames
        .as_ref()
        .expect("animated cell has host provenance")
        .clone();
    for tile in tiles.iter().filter(|tile| {
        tile.source.tileset_id.as_ref() == "cave" && tile.source.tile_index == tile_index
    }) {
        assert!(Arc::ptr_eq(
            tile.animation_frames.as_ref().unwrap(),
            &family
        ));
        assert!(family.contains(&tile.texture));
    }
    family
}

#[test]
fn visual_tile_animation_cave_composite_emits_complete_shared_family() {
    let mut app = tile_animation_provenance_app(0x14);
    let family = tile_animation_emitted_family(&app, 0x14);
    assert_eq!(
        family.len(),
        32,
        "all four water frames and eight scroll phases"
    );
    let art = app.world().resource::<RenderedTilesetArt>();
    let cache_family = &art
        .cache
        .iter()
        .find(|(key, _)| key.tileset_id == "cave" && key.time_of_day == "day")
        .unwrap()
        .1
        .animated_tiles[&0x14]
        .frames;
    assert!(Arc::ptr_eq(&family, cache_family));
    assert_eq!(
        family
            .iter()
            .map(Handle::id)
            .collect::<std::collections::HashSet<_>>()
            .len(),
        32,
        "the complete family comes from authored assets, not observed phase unions"
    );
    let session = app
        .world()
        .resource::<BevyRuntimeShell>()
        .shell
        .session()
        .clone();
    let moves = app
        .world()
        .resource::<BevyRuntimeShell>()
        .visible_move_animations
        .clone();
    let published = app
        .world()
        .resource::<crystal_render_api::VisualWorldFrame>();
    let revision = published.terrain_revision;
    let published_grid = published.tiles.as_ptr();
    let initial_tiles = published.tiles.clone();
    for (tick, expected_phase) in [
        (0, 0),
        (3, 0),
        (4, 1),
        (21, 1),
        (22, 9),
        (23, 10),
        (41, 10),
        (42, 11),
        (43, 11),
        (44, 19),
        (65, 20),
        (66, 28),
        (87, 29),
        (88, 5),
        (136, 23),
        (137, 16),
    ] {
        tile_animation_render_phase(&mut app, tick);
        assert!(Arc::ptr_eq(
            &family,
            &tile_animation_emitted_family(&app, 0x14)
        ));
        let rendered = app.world().resource::<RenderedViewport>();
        assert_eq!(rendered.visual_tiles_revision, Some(revision));
        let mut animated_count = 0;
        let mut static_count = 0;
        for tile in &rendered.visual_tiles {
            if tile.source.tile_index == 0x14 {
                assert_eq!(tile.texture, family[expected_phase], "LCD tick {tick}");
                animated_count += 1;
            } else if tile.source.tile_index != 0x40 {
                assert!(tile.animation_frames.is_none(), "static cells stay strict");
                static_count += 1;
            }
        }
        assert!(
            animated_count > 1,
            "multiple cells share the same allocation"
        );
        assert!(static_count > 0);
        let published = app
            .world()
            .resource::<crystal_render_api::VisualWorldFrame>();
        assert_eq!(published.terrain_revision, revision);
        assert_eq!(
            published.tiles.as_ptr(),
            published_grid,
            "phase-only publication retains the grid allocation"
        );
        assert_eq!(
            published.tiles, initial_tiles,
            "the immutable published grid can retain its earlier valid phase"
        );
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(
            shell.shell.session(),
            &session,
            "render-only work preserves gameplay and RNG clocks"
        );
        assert_eq!(
            shell.visible_move_animations, moves,
            "render-only work preserves attack clocks"
        );
    }
}

#[test]
fn visual_tile_animation_cave_vertical_scroll_emits_complete_shared_family() {
    let mut app = tile_animation_provenance_app(0x40);
    let family = tile_animation_emitted_family(&app, 0x40);
    assert_eq!(family.len(), 8);
    let revision = app
        .world()
        .resource::<RenderedViewport>()
        .visual_tiles_revision;
    for (tick, expected_phase) in [
        (0, 0),
        (15, 0),
        (16, 1),
        (34, 1),
        (35, 2),
        (53, 2),
        (54, 3),
        (148, 7),
        (149, 0),
        (167, 0),
        (168, 1),
    ] {
        tile_animation_render_phase(&mut app, tick);
        assert!(Arc::ptr_eq(
            &family,
            &tile_animation_emitted_family(&app, 0x40)
        ));
        let rendered = app.world().resource::<RenderedViewport>();
        assert_eq!(rendered.visual_tiles_revision, revision);
        for tile in rendered
            .visual_tiles
            .iter()
            .filter(|tile| tile.source.tile_index == 0x40)
        {
            assert_eq!(tile.texture, family[expected_phase], "LCD tick {tick}");
        }
    }
}

#[test]
fn visual_tile_animation_palette_time_and_cache_entries_have_distinct_families() {
    // All variants intentionally use one Bevy asset store and the production
    // cache, so numeric handle reuse from independent stores cannot mask bugs.
    let mut app = tile_animation_provenance_app(0x14);
    let original = tile_animation_emitted_family(&app, 0x14);
    let original_key = app
        .world()
        .resource::<RenderedTilesetArt>()
        .cache
        .keys()
        .find(|key| key.tileset_id == "cave" && key.time_of_day == "day")
        .unwrap()
        .clone();
    let original_cache_len = app.world().resource::<RenderedTilesetArt>().cache.len();
    tile_animation_render_phase(&mut app, 22);
    assert!(Arc::ptr_eq(
        &original,
        &tile_animation_emitted_family(&app, 0x14)
    ));
    assert_eq!(
        app.world().resource::<RenderedTilesetArt>().cache.len(),
        original_cache_len
    );

    tile_animation_edit_presented_snapshot(&mut app, |snapshot| {
        let cave = Arc::make_mut(&mut snapshot.tilesets)
            .iter_mut()
            .find(|tileset| tileset.tileset_id == "cave")
            .unwrap();
        cave.palette_map[0x14] = (cave.palette_map[0x14] & !7) | ((cave.palette_map[0x14] + 1) & 7);
    });
    tile_animation_render_phase(&mut app, 22);
    let remapped = tile_animation_emitted_family(&app, 0x14);
    assert!(!Arc::ptr_eq(&original, &remapped));
    assert!(original.iter().all(|handle| !remapped.contains(handle)));
    assert_eq!(
        app.world().resource::<RenderedTilesetArt>().cache.len(),
        original_cache_len + 1
    );

    tile_animation_edit_presented_snapshot(&mut app, |snapshot| {
        let cave = Arc::make_mut(&mut snapshot.tilesets)
            .iter_mut()
            .find(|tileset| tileset.tileset_id == "cave")
            .unwrap();
        cave.palette_map.clone_from(&original_key.palette_map);
        let map = Arc::make_mut(
            snapshot
                .maps
                .iter_mut()
                .find(|map| map.map_name == "UnionCave1F")
                .unwrap(),
        );
        map.attributes.time_of_day = Some("nite".into());
    });
    tile_animation_render_phase(&mut app, 22);
    let night = tile_animation_emitted_family(&app, 0x14);
    assert!(!Arc::ptr_eq(&original, &night));
    assert!(original.iter().all(|handle| !night.contains(handle)));
    assert!(remapped.iter().all(|handle| !night.contains(handle)));

    tile_animation_edit_presented_snapshot(&mut app, |snapshot| {
        let map = Arc::make_mut(
            snapshot
                .maps
                .iter_mut()
                .find(|map| map.map_name == "UnionCave1F")
                .unwrap(),
        );
        map.attributes.time_of_day = Some("day".into());
    });
    tile_animation_render_phase(&mut app, 23);
    assert!(Arc::ptr_eq(
        &original,
        &tile_animation_emitted_family(&app, 0x14)
    ));
    app.world_mut()
        .resource_mut::<RenderedTilesetArt>()
        .cache
        .remove(&original_key)
        .unwrap();
    tile_animation_render_phase(&mut app, 44);
    let reloaded = tile_animation_emitted_family(&app, 0x14);
    assert!(!Arc::ptr_eq(&original, &reloaded));
    assert!(original.iter().all(|handle| !reloaded.contains(handle)));
}
