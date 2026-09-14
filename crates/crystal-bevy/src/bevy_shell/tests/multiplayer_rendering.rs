fn disconnected_render_multiplayer_fixture() -> MultiplayerRuntime {
    let config = BevyMultiplayerConfig {
        server_url: "ws://unused.invalid".into(),
        server_token: None,
        world_id: "render-test".into(),
        player_id: 1,
        display_name: "TEST".into(),
        rating: 1000,
        rating_range: 100,
    };
    MultiplayerRuntime {
        connection: None,
        session: None,
        config: config.clone(),
        queued_mode: None,
        match_mode: None,
        result_reported: false,
        session_settled: false,
        direct_mode: None,
        direct_session: false,
        pending_interaction: None,
        last_profile: None,
        last_presence: None,
        presence_frames_since_send: 0,
        remote_presences: HashMap::new(),
        player_id: config.player_id,
        peer_player_id: None,
        peer_player_name: None,
        trade_id_prefix: "pending".into(),
        trade_sequence: 1,
        owns_internal_clock: false,
        last_sent_link_room: None,
        remote_link_room: None,
        game_link_ready: false,
        party_sent: false,
        remote_party: None,
        active_trade: None,
        peer_trade_offers: VecDeque::new(),
        peer_trade_confirmations: VecDeque::new(),
        link_battle_random: None,
        link_battle_random_sent: false,
        link_battle_started: false,
        failed: false,
        reconnect_frames: 0,
        reconnect_attempt: 0,
        social_notice: None,
        sent_input_count: 0,
        sent_battle_action_count: 0,
        sent_menu_result_count: 0,
        peer_inputs: VecDeque::new(),
        peer_battle_actions: VecDeque::new(),
        peer_menu_results: VecDeque::new(),
    }
}

fn multiplayer_render_test_app() -> App {
    let asset_root = AssetRoot::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .unwrap(),
    );
    let runtime = workspace_desktop_runtime(&asset_root);
    let spawn_identifier = runtime.title_new_game_spawn_identifier().unwrap();
    let shell = initialize_bevy_runtime_shell(
        asset_root,
        runtime,
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier,
            map_name: "NewBarkTown".into(),
            tile_x: 13,
            tile_y: 6,
        },
        BevyShellConfig::default(),
    )
    .unwrap();
    let mut app = App::new();
    app.insert_resource(shell)
        .insert_resource(Time::<()>::default())
        .insert_resource(RenderedViewport {
            viewport_origin: Some((0, 0)),
            ..default()
        })
        .init_resource::<RenderedTilesetArt>()
        .init_resource::<Assets<Image>>()
        .insert_non_send_resource(disconnected_render_multiplayer_fixture())
        .add_systems(Update, sync_multiplayer_ghosts);
    app.world_mut().spawn((
        PlayerMarker,
        Sprite {
            custom_size: Some(Vec2::splat(32.0)),
            ..default()
        },
    ));
    app
}

#[test]
fn multiplayer_render_reuses_presentation_snapshot_and_refreshes_after_runtime_tick() {
    let mut app = multiplayer_render_test_app();
    app.world_mut()
        .non_send_resource_mut::<MultiplayerRuntime>()
        .remote_presences
        .insert(
            "player-2".into(),
            RemotePresence {
                player_gender: 0,
                display_name: "REMOTE".into(),
                map: "NewBarkTown".into(),
                tile_x: 1,
                tile_y: 1,
                direction: "down".into(),
            },
        );
    let initial_state = app
        .world()
        .resource::<BevyRuntimeShell>()
        .shell
        .snapshot()
        .unwrap()
        .state_checksum;
    app.update();
    let ghost_count = app
        .world_mut()
        .query::<&MultiplayerGhost>()
        .iter(app.world())
        .count();
    assert_eq!(ghost_count, 1, "the remote trainer must be rendered");
    let first = app.world().resource::<BevyRuntimeShell>().cached_snapshot.as_ref()
        .expect("remote rendering must share the presentation cache instead of validating and hashing the entire save each frame").1.clone();
    app.update();
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert!(Arc::ptr_eq(
        &first,
        &shell.cached_snapshot.as_ref().unwrap().1
    ));
    assert_eq!(
        shell.shell.snapshot().unwrap().state_checksum,
        initial_state,
        "rendering must not mutate deterministic game state"
    );
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        shell.shell.tick([]).unwrap();
        mark_runtime_snapshot_dirty(&mut shell);
    }
    app.update();
    let shell = app.world().resource::<BevyRuntimeShell>();
    let refreshed = &shell.cached_snapshot.as_ref().unwrap().1;
    assert!(!Arc::ptr_eq(&first, refreshed));
    assert_eq!(
        refreshed.overworld,
        shell.shell.snapshot().unwrap().overworld
    );
}

#[test]
#[ignore = "performance probe; run explicitly with --ignored --nocapture"]
fn multiplayer_render_performance_benchmark() {
    let mut app = multiplayer_render_test_app();
    app.update();
    let mut samples = Vec::new();
    for _ in 0..120 {
        let started = std::time::Instant::now();
        app.update();
        samples.push(started.elapsed().as_secs_f64() * 1_000_000.0);
    }
    samples.sort_by(f64::total_cmp);
    eprintln!(
        "multiplayer_render_perf samples={} median_us={:.2} p95_us={:.2} max_us={:.2}",
        samples.len(),
        (samples[59] + samples[60]) * 0.5,
        samples[113],
        samples[119]
    );
}

#[test]
fn native_audio_device_open_does_not_block_a_presented_frame() {
    // Device enumeration / CoreAudio stream creation can block for hundreds
    // of milliseconds. It must happen before app.run(), never in play().
    let source = include_str!("../../bevy_shell.rs");
    let backend = source.split("impl NativeAudioBackend {").nth(1).unwrap();
    let play = backend
        .split("    fn play(\n")
        .nth(1)
        .unwrap()
        .split("\n#[cfg")
        .next()
        .unwrap();
    assert!(
        !play.contains("OutputStream::try_default()"),
        "native device opening must happen during startup, outside presented frames"
    );
}
