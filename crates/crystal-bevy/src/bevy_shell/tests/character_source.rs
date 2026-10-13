// Verify the actual classic player entity after map objects have populated
// the same image store. A source sheet alone cannot prove sprite association.
#[test]
#[ignore = "requires an external production content pack"]
fn classic_player_keeps_human_art_when_item_balls_share_the_map() {
    let pack = PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
        .expect("set CRYSTAL_RENDER_TEST_PACK to the external production pack"));
    for (map_name, tile_x, tile_y) in [
        ("SilverCaveRoom2", 15, 18),
        ("SilverCaveRoom2", 14, 18),
        ("NewBarkTown", 13, 6),
    ] {
        let root = AssetRoot::new(pack.parent().unwrap());
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&pack).unwrap();
        let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded).unwrap();
        let spawn_identifier = runtime.title_new_game_spawn_identifier().unwrap();
        let runtime_shell = initialize_bevy_runtime_shell(root, runtime,
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier, map_name: map_name.into(), tile_x, tile_y,
            }, BevyShellConfig {
                smoke_player_name: Some("CHRIS".into()), ..Default::default()
            }).unwrap();
        assert_eq!(runtime_shell.shell.snapshot().unwrap().overworld.mode, MovementMode::Normal);
        let mut app = App::new();
        app.insert_resource(runtime_shell)
            .insert_resource(RenderedViewport::default())
            .insert_resource(RenderedTilesetArt::default())
            .init_resource::<Assets<Image>>()
            .add_systems(Update, (render_playfield, sync_visible_player_sprite).chain());
        for _ in 0..3 { app.update(); }
        let mut players = app.world_mut().query_filtered::<(&Sprite, &PlayerSpriteFrames), With<PlayerMarker>>();
        let (sprite, frames) = players.single(app.world()).unwrap();
        assert_eq!(frames.source_id.as_ref(), "chris", "{map_name}");
        assert_eq!(sprite.image, frames.standing, "{map_name}");
        let actual = app.world().resource::<Assets<Image>>().get(&sprite.image).unwrap().data.clone();
        let root = app.world().resource::<BevyRuntimeShell>().asset_root.clone();
        let art_key = app.world().resource::<RenderedTilesetArt>().sprite_cache.keys()
            .find(|key| key.sprite_id == "chris").unwrap().clone();
        let mut reference_images = Assets::<Image>::default();
        let reference = load_sprite_art(&root, "chris", art_key.palette_id,
            &art_key.time_of_day, &mut reference_images).unwrap();
        assert_eq!(actual, reference_images.get(&reference.down.standing.handle).unwrap().data,
            "{map_name}: player entity received another actor's image");
        assert!(actual.unwrap().chunks_exact(4).filter(|p| p[3] != 0).count() > 60);
    }
}
