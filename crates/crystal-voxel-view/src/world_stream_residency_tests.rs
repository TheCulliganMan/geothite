// Exercise actual render-root ownership across sliding coverage and map edges.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn moving_lookahead_finishes_pending_uploads_before_retargeting() {
    let (mut live, _) = crate::world_stream::test_frames::connected_frames(IVec2::new(40, 0));
    let mut images = Assets::<Image>::default();
    let texture = images.add(Image::default());
    live.map_texture = texture.clone();
    for tile in &mut live.tiles {
        tile.texture = texture.clone();
    }
    let mut publishing = live.clone();
    publishing.grid_origin.x += 4;
    let mut requested = live.clone();
    requested.grid_origin.x += 40;
    assert!(!compatible(&publishing, &requested));
    assert!(compatible(&publishing, &live));
    assert!(!world_stream::advances_coverage(&live, &publishing, &live));
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let root = app
        .world_mut()
        .spawn((VoxelTerrain, Visibility::Hidden, Transform::default()))
        .id();
    let meshes = Assets::<Mesh>::default();
    let mut next = PreparedDestination {
        source_frame: Some(Arc::new(publishing.clone())),
        cache: TerrainRevisionCache {
            key: Some(TerrainCacheKey::from_frame(&publishing)),
            built_frame: Some(publishing.clone()),
            built_footing_heights: vec![0.; 28 * 28],
            solid_entity: Some(root),
            painted_stone: Some(texture.clone()),
            painted_bedrock: Some(texture),
            ..default()
        },
        ..default()
    };
    let _pending_mesh = next
        .cache
        .uploads
        .enqueue(&meshes, Mesh::from(Cuboid::default()));
    next.cache.uploads.seal();
    app.insert_resource(images)
        .insert_resource(meshes)
        .insert_resource(live.clone())
        .insert_resource(TerrainRevisionCache {
            built_frame: Some(live),
            ..default()
        })
        .insert_resource(PreparedWorld { next, ..default() })
        .insert_resource(DestinationPreload {
            stream_frame: Some(requested.clone()),
            ..default()
        })
        .insert_resource(VoxelViewSettings {
            enabled: true,
            ..default()
        })
        .init_resource::<PreparedDestination>()
        .init_resource::<TerrainBuildQueue>()
        .init_resource::<VoxelViewStatus>()
        .init_resource::<live_profiles::LiveProfiles>()
        .init_resource::<Assets<VoxelMaterial>>()
        .add_systems(Update, prepare);
    app.update();
    let next = &app.world().resource::<PreparedWorld>().next;
    assert_eq!(
        next.cache
            .built_frame
            .as_ref()
            .map(|frame| frame.grid_origin),
        Some(publishing.grid_origin)
    );
    assert_eq!(
        next.builds.started, 0,
        "do not discard CPU-complete work during its GPU publication"
    );
    assert_eq!(next.ready_frames, 1);
    assert_eq!(
        app.world().get::<Visibility>(root),
        Some(&Visibility::Hidden)
    );
    assert_eq!(
        app.world()
            .resource::<DestinationPreload>()
            .stream_frame
            .as_ref()
            .map(|frame| frame.grid_origin),
        Some(requested.grid_origin)
    );
    app.update();
    app.update();
    assert_eq!(app.world().resource::<PreparedWorld>().next.ready_frames, 3);
    // The settled bridge is now closer to the moving camera than its current
    // resident. Keep it available for promotion while preparing farther ahead.
    app.world_mut()
        .resource_mut::<TerrainRevisionCache>()
        .built_frame
        .as_mut()
        .expect("resident source")
        .grid_origin
        .x -= 8;
    let resident = app
        .world()
        .resource::<TerrainRevisionCache>()
        .built_frame
        .as_ref()
        .expect("resident source")
        .clone();
    let current = app.world().resource::<VisualWorldFrame>().clone();
    assert!(world_stream::advances_coverage(
        &resident,
        &publishing,
        &current
    ));
    app.update();
    assert_eq!(
        app.world().resource::<PreparedWorld>().next.builds.started,
        1,
        "a finished useful bridge must not block forward preloading"
    );
    let profiles = live_profiles::LiveProfiles::default();
    let bridge = app
        .world_mut()
        .resource_mut::<PreparedWorld>()
        .retained
        .take_ready(&resident, &current, &profiles)
        .expect("the settled bridge stays available during the next build");
    assert_eq!(
        bridge
            .built_frame
            .as_ref()
            .expect("bridge source")
            .grid_origin,
        publishing.grid_origin
    );
    assert_eq!(
        app.world().get::<Visibility>(root),
        Some(&Visibility::Hidden)
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn a_rebased_worker_request_updates_its_source_receipt_after_submission() {
    let (mut old, mut requested) =
        crate::world_stream::test_frames::connected_frames(IVec2::new(40, 0));
    let mut images = Assets::<Image>::default();
    let texture = images.add(Image::default());
    old.map_texture = texture.clone();
    requested.map_texture = texture.clone();
    for tile in old.tiles.iter_mut().chain(&mut requested.tiles) {
        tile.texture = texture.clone();
    }
    requested.grid_origin.x += 80;
    assert!(!compatible(&old, &requested));
    let queued = TerrainCacheKey::from_frame(&old);
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(images)
        .insert_resource(requested.clone())
        .insert_resource(DestinationPreload {
            stream_frame: Some(requested.clone()),
            ..default()
        })
        .insert_resource(PreparedWorld {
            next: PreparedDestination {
                source_frame: Some(Arc::new(old)),
                builds: TerrainBuildQueue {
                    key: Some(queued),
                    ..default()
                },
                ..default()
            },
            ..default()
        })
        .insert_resource(TerrainRevisionCache {
            painted_stone: Some(texture.clone()),
            painted_bedrock: Some(texture),
            ..default()
        })
        .insert_resource(VoxelViewSettings {
            enabled: true,
            ..default()
        })
        .init_resource::<PreparedDestination>()
        .init_resource::<TerrainBuildQueue>()
        .init_resource::<live_profiles::LiveProfiles>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<VoxelMaterial>>()
        .add_systems(Update, prepare);
    app.update();
    let next = &app.world().resource::<PreparedWorld>().next;
    assert_eq!(next.builds.started, 1);
    let source = next.source_frame.as_ref().expect("newly submitted source");
    assert_eq!(source.map_id, requested.map_id);
    assert_eq!(source.grid_origin, requested.grid_origin);
}

#[test]
fn pending_world_coverage_rejects_changed_sources_and_profiles() {
    let (frame, _) = crate::world_stream::test_frames::connected_frames(IVec2::new(40, 0));
    let mut profiles = live_profiles::LiveProfiles::default();
    let mut world = PreparedWorld {
        next: PreparedDestination {
            builds: TerrainBuildQueue {
                key: Some(TerrainCacheKey::from_frame(&frame)),
                ..default()
            },
            source_frame: Some(Arc::new(frame.clone())),
            ..default()
        },
        ..default()
    };
    assert!(pending_world_covers(&frame, &profiles, &mut world));
    let mut changed = frame.clone();
    changed.tiles[0].source.tile_index ^= 1;
    changed.terrain_revision += 1;
    assert!(!pending_world_covers(&changed, &profiles, &mut world));
    profiles.revision += 1;
    assert!(!pending_world_covers(&frame, &profiles, &mut world));
    profiles.revision -= 1;
    world.next.builds.key = None;
    world.next.cache.key = Some(TerrainCacheKey::from_frame(&frame));
    world.next.cache.built_frame = Some(frame.clone());
    assert!(pending_world_covers(&frame, &profiles, &mut world));
    world.next.ready_frames = 3;
    assert!(!pending_world_covers(&frame, &profiles, &mut world));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn a_covering_preload_prevents_a_duplicate_foreground_rebuild() {
    let (mut built, _) = crate::world_stream::test_frames::connected_frames(IVec2::new(40, 0));
    let mut images = Assets::<Image>::default();
    let texture = images.add(Image::default());
    built.map_texture = texture.clone();
    for tile in &mut built.tiles {
        tile.texture = texture.clone();
    }
    let mut live = built.clone();
    live.grid_origin.x += 8;
    assert!(!terrain_tracking::can_reuse(&built, &live));
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let root = app
        .world_mut()
        .spawn((VoxelTerrain, Visibility::Visible, Transform::default()))
        .id();
    app.insert_resource(images)
        .insert_resource(live)
        .insert_resource(TerrainRevisionCache {
            key: Some(TerrainCacheKey::from_frame(&built)), built_frame: Some(built),
            built_footing_heights: vec![4.; 28 * 28], solid_entity: Some(root),
            painted_stone: Some(texture.clone()), painted_bedrock: Some(texture), ..default()
        })
        .insert_resource(TerrainBuildQueue { defer_to_world_preload: true, ..default() })
        .init_resource::<live_profiles::LiveProfiles>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<VoxelMaterial>>()
        .add_systems(Update, |frame: Res<VisualWorldFrame>, profiles: Res<live_profiles::LiveProfiles>,
            mut commands: Commands, mut cache: ResMut<TerrainRevisionCache>, mut builds: ResMut<TerrainBuildQueue>,
            mut images: ResMut<Assets<Image>>, mut meshes: ResMut<Assets<Mesh>>,
            mut materials: ResMut<Assets<VoxelMaterial>>, mut entities: Query<(&mut Visibility, &mut Transform), VoxelTerrainFilter>| {
            assert!(matches!(sync_terrain(&frame, &profiles, &mut commands, &mut cache, &mut builds,
                &mut images, &mut meshes, &mut materials, &mut entities), Ok(TerrainSyncState::Pending)));
        });
    app.update();
    assert_eq!(app.world().resource::<TerrainBuildQueue>().started, 0);
    let cache = app.world().resource::<TerrainRevisionCache>();
    assert_eq!(cache.solid_entity, Some(root));
    assert_eq!(cache.footing_heights[14 * 28 + 14], 4.);
    assert_eq!(
        app.world().get::<Visibility>(root),
        Some(&Visibility::Visible)
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn a_proven_connection_keeps_resident_geometry_while_a_late_replacement_is_pending() {
    let (mut built, mut accepted) =
        crate::world_stream::test_frames::connected_frames(IVec2::new(40, 0));
    let mut images = Assets::<Image>::default();
    let texture = images.add(Image::default());
    for frame in [&mut built, &mut accepted] {
        frame.map_texture = texture.clone();
        for tile in &mut frame.tiles {
            tile.texture = texture.clone();
        }
    }
    accepted.grid_origin.x += 4;
    let mut current = accepted.clone();
    current.grid_origin.x += 4;
    assert!(terrain_tracking::can_reuse(&built, &accepted));
    assert!(!terrain_tracking::can_reuse(&built, &current));
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let root = app
        .world_mut()
        .spawn((VoxelTerrain, Visibility::Visible, Transform::default()))
        .id();
    let task = AsyncComputeTaskPool::get().spawn(future::pending::<TerrainBuildResult>());
    app.insert_resource(images)
        .insert_resource(current.clone())
        .insert_resource(TerrainRevisionCache {
            key: Some(TerrainCacheKey::from_frame(&accepted)),
            built_frame: Some(built),
            solid_entity: Some(root),
            built_footing_heights: vec![4.; 28 * 28],
            painted_stone: Some(texture.clone()),
            painted_bedrock: Some(texture),
            ..default()
        })
        .insert_resource(TerrainBuildQueue {
            started: 7,
            key: Some(TerrainCacheKey::from_frame(&current)),
            task: Some(task),
            ..default()
        })
        .init_resource::<live_profiles::LiveProfiles>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<VoxelMaterial>>()
        .add_systems(Update, |frame: Res<VisualWorldFrame>, profiles: Res<live_profiles::LiveProfiles>,
            mut commands: Commands, mut cache: ResMut<TerrainRevisionCache>, mut builds: ResMut<TerrainBuildQueue>,
            mut images: ResMut<Assets<Image>>, mut meshes: ResMut<Assets<Mesh>>,
            mut materials: ResMut<Assets<VoxelMaterial>>, mut entities: Query<(&mut Visibility, &mut Transform), VoxelTerrainFilter>| {
            assert!(matches!(sync_terrain(&frame, &profiles, &mut commands, &mut cache, &mut builds,
                &mut images, &mut meshes, &mut materials, &mut entities), Ok(TerrainSyncState::Pending)));
        });
    app.world_mut()
        .resource_mut::<TerrainRevisionCache>()
        .uploads
        .require_gpu(true);
    app.update();
    let cache = app.world().resource::<TerrainRevisionCache>();
    assert_eq!(cache.solid_entity, Some(root));
    assert_eq!(
        cache.key.as_ref().expect("accepted logical key").map_id,
        current.map_id
    );
    assert_eq!(cache.footing_origin, Some(current.grid_origin));
    assert_eq!(cache.footing_heights[14 * 28 + 14], 4.);
    assert_eq!(
        *app.world()
            .get::<Visibility>(root)
            .expect("resident root must survive"),
        Visibility::Visible
    );
    assert_eq!(app.world().resource::<TerrainBuildQueue>().started, 7);
    // The logical receipt alone cannot authenticate a changed placement graph.
    app.world_mut()
        .resource_mut::<VisualWorldFrame>()
        .source_regions = Arc::from([]);
    app.update();
    assert!(app.world().get_entity(root).is_err());
    assert!(app.world().resource::<TerrainRevisionCache>().key.is_none());
    assert!(
        app.world()
            .resource::<TerrainRevisionCache>()
            .uploads
            .gpu_required(),
        "retiring an unproven window must preserve render-residency verification"
    );
}

#[test]
fn prepared_world_advances_coverage_inside_a_map_without_starting_new_work() {
    let (built, _) = crate::world_stream::test_frames::connected_frames(IVec2::new(40, 0));
    let mut next = built.clone();
    next.grid_origin.x += 4;
    let back = built.clone();
    assert!(compatible(&built, &next));
    let mut app = App::new();
    let old_root = app.world_mut().spawn(Visibility::Visible).id();
    let next_root = app.world_mut().spawn(Visibility::Hidden).id();
    let mut live = TerrainRevisionCache {
        key: Some(TerrainCacheKey::from_frame(&built)),
        built_frame: Some(built),
        solid_entity: Some(old_root),
        ..default()
    };
    let mut prepared = PreparedWorld {
        next: PreparedDestination {
            ready_frames: 3,
            cache: TerrainRevisionCache {
                key: Some(TerrainCacheKey::from_frame(&next)),
                built_frame: Some(next.clone()),
                solid_entity: Some(next_root),
                ..default()
            },
            ..default()
        },
        ..default()
    };
    let mut builds = TerrainBuildQueue {
        started: 7,
        ..default()
    };
    let mut status = crate::WorldStreamStatus::default();
    let mut assets = Assets::<Mesh>::default();
    let _pending_mesh = prepared
        .next
        .cache
        .uploads
        .enqueue(&assets, Mesh::from(Cuboid::default()));
    prepared.next.cache.uploads.seal();
    let mut queue = bevy::ecs::world::CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        activate_world(
            &next,
            &live_profiles::LiveProfiles::default(),
            &mut prepared,
            &mut status,
            &mut live,
            &mut builds,
            &mut commands,
        );
    }
    queue.apply(app.world_mut());
    assert_eq!(
        status.promotions, 0,
        "a settled counter cannot bypass unfinished mesh publication"
    );
    assert_eq!(live.solid_entity, Some(old_root));
    assert_eq!(
        app.world().get::<Visibility>(old_root),
        Some(&Visibility::Visible)
    );
    prepared.next.cache.uploads.advance(&mut assets);
    let mut queue = bevy::ecs::world::CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        activate_world(
            &next,
            &live_profiles::LiveProfiles::default(),
            &mut prepared,
            &mut status,
            &mut live,
            &mut builds,
            &mut commands,
        );
    }
    queue.apply(app.world_mut());
    assert_eq!(status.promotions, 1);
    assert_eq!(live.solid_entity, Some(next_root));
    assert!(prepared.next.cache.built_frame.is_none());
    assert_eq!(
        builds.started, 7,
        "resident promotion must not start a foreground build"
    );
    assert_eq!(
        *app.world()
            .get::<Visibility>(old_root)
            .expect("retained old root"),
        Visibility::Hidden
    );
    assert_eq!(
        *app.world()
            .get::<Visibility>(next_root)
            .expect("prepared world root"),
        Visibility::Visible
    );
    // Reversing travel uses the actual old render roots, without meshing or
    // waiting for the independent forward-preparation lane.
    let mut queue = bevy::ecs::world::CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        activate_world(
            &back,
            &live_profiles::LiveProfiles::default(),
            &mut prepared,
            &mut status,
            &mut live,
            &mut builds,
            &mut commands,
        );
    }
    queue.apply(app.world_mut());
    assert_eq!(status.promotions, 2);
    assert_eq!(builds.started, 7);
    assert_eq!(live.solid_entity, Some(old_root));
    assert_eq!(
        *app.world()
            .get::<Visibility>(old_root)
            .expect("restored root"),
        Visibility::Visible
    );
    assert_eq!(
        *app.world()
            .get::<Visibility>(next_root)
            .expect("retained next root"),
        Visibility::Hidden
    );
}

#[test]
fn a_ready_destination_does_not_replace_world_coverage_at_a_logical_map_edge() {
    let (built, next) = crate::world_stream::test_frames::connected_frames(IVec2::new(40, 0));
    let mut app = App::new();
    let old_root = app.world_mut().spawn(Visibility::Visible).id();
    let next_root = app.world_mut().spawn(Visibility::Hidden).id();
    let mut live = TerrainRevisionCache {
        key: Some(TerrainCacheKey::from_frame(&built)),
        built_frame: Some(built),
        solid_entity: Some(old_root),
        ..default()
    };
    let mut prepared = PreparedWorld {
        next: PreparedDestination {
            ready_frames: 3,
            cache: TerrainRevisionCache {
                key: Some(TerrainCacheKey::from_frame(&next)),
                built_frame: Some(next.clone()),
                solid_entity: Some(next_root),
                ..default()
            },
            ..default()
        },
        ..default()
    };
    let mut status = crate::WorldStreamStatus::default();
    let mut request = DestinationPreload::default();
    let mut builds = TerrainBuildQueue::default();
    let mut queue = bevy::ecs::world::CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        let profiles = live_profiles::LiveProfiles::default();
        activate_world(
            &next,
            &profiles,
            &mut prepared,
            &mut status,
            &mut live,
            &mut builds,
            &mut commands,
        );
        activate(
            &next,
            &profiles,
            &mut prepared.next,
            &mut request,
            &mut live,
            &mut builds,
            &mut commands,
        );
    }
    queue.apply(app.world_mut());
    assert_eq!(status.promotions, 0);
    assert_eq!(request.hits, 0);
    assert_eq!(live.solid_entity, Some(old_root));
    assert_eq!(
        *app.world()
            .get::<Visibility>(old_root)
            .expect("resident world root"),
        Visibility::Visible
    );
    assert_eq!(
        *app.world()
            .get::<Visibility>(next_root)
            .expect("hidden lookahead root"),
        Visibility::Hidden
    );
}

#[test]
fn advancing_lookahead_keeps_a_ready_nearby_window_for_the_camera() {
    let (mut frame, _) = crate::world_stream::test_frames::connected_frames(IVec2::new(40, 0));
    let mut images = Assets::<Image>::default();
    let texture = images.add(Image::default());
    frame.map_texture = texture.clone();
    for tile in &mut frame.tiles {
        tile.texture = texture.clone();
    }
    let mut nearby = frame.clone();
    nearby.grid_origin.x += 4;
    let mut farther = frame.clone();
    farther.grid_origin.x += 16;
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::transform::TransformPlugin));
    let old_root = app
        .world_mut()
        .spawn((VoxelTerrain, Visibility::Visible, Transform::default()))
        .id();
    let ready_root = app
        .world_mut()
        .spawn((VoxelTerrain, Visibility::Hidden, Transform::default()))
        .id();
    app.insert_resource(images)
        .insert_resource(frame.clone())
        .insert_resource(VoxelViewSettings {
            enabled: true,
            ..default()
        })
        .insert_resource(DestinationPreload {
            stream_frame: Some(farther),
            ..default()
        })
        .insert_resource(TerrainRevisionCache {
            key: Some(TerrainCacheKey::from_frame(&frame)),
            built_frame: Some(frame),
            solid_entity: Some(old_root),
            painted_stone: Some(texture.clone()),
            painted_bedrock: Some(texture.clone()),
            ..default()
        })
        .insert_resource(PreparedWorld {
            next: PreparedDestination {
                ready_frames: 3,
                cache: TerrainRevisionCache {
                    key: Some(TerrainCacheKey::from_frame(&nearby)),
                    built_frame: Some(nearby.clone()),
                    solid_entity: Some(ready_root),
                    painted_stone: Some(texture.clone()),
                    painted_bedrock: Some(texture),
                    ..default()
                },
                ..default()
            },
            ..default()
        })
        .init_resource::<PreparedDestination>()
        .init_resource::<TerrainBuildQueue>()
        .init_resource::<live_profiles::LiveProfiles>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<VoxelMaterial>>()
        .add_systems(Update, prepare);
    app.update();
    let mut prepared = app
        .world_mut()
        .remove_resource::<PreparedWorld>()
        .expect("prepared world");
    let mut live = app
        .world_mut()
        .remove_resource::<TerrainRevisionCache>()
        .expect("resident terrain");
    let mut builds = app
        .world_mut()
        .remove_resource::<TerrainBuildQueue>()
        .expect("foreground queue");
    let mut status = crate::WorldStreamStatus::default();
    let mut queue = bevy::ecs::world::CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        activate_world(
            &nearby,
            &live_profiles::LiveProfiles::default(),
            &mut prepared,
            &mut status,
            &mut live,
            &mut builds,
            &mut commands,
        );
    }
    queue.apply(app.world_mut());
    assert_eq!(
        status.promotions, 1,
        "the ready overlap must survive advancing lookahead"
    );
    assert_eq!(live.solid_entity, Some(ready_root));
    assert_eq!(
        builds.started, 0,
        "the foreground must adopt existing geometry"
    );
    assert_eq!(
        *app.world()
            .get::<Visibility>(ready_root)
            .expect("ready overlap root"),
        Visibility::Visible
    );
    assert_eq!(
        *app.world()
            .get::<Visibility>(old_root)
            .expect("old resident root"),
        Visibility::Hidden
    );
}

#[test]
fn a_window_promoted_without_new_input_still_aligns_its_support() {
    let (mut built, _) = crate::world_stream::test_frames::connected_frames(IVec2::new(40, 0));
    built.map_id = "SupportTest".into();
    Arc::make_mut(&mut built.source_regions)[0].map_id = built.map_id.clone();
    let mut app = App::new();
    app.init_resource::<Assets<Image>>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<VoxelMaterial>>()
        .init_resource::<live_profiles::LiveProfiles>()
        .init_resource::<VoxelViewSettings>()
        .init_resource::<VoxelViewStatus>()
        .init_resource::<NativeHealingEffectsStatus>()
        .init_resource::<TerrainBuildQueue>()
        .init_resource::<ActorIdCache>()
        .init_resource::<PreparedWorld>()
        .init_resource::<crate::WorldStreamStatus>();
    app.world_mut().resource_mut::<VoxelViewSettings>().enabled = true;
    let atlas = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::default());
    built.map_texture = atlas.clone();
    for tile in &mut built.tiles {
        tile.texture = atlas.clone();
    }
    built.actors.push(VisualActor {
        id: VisualActorId::Player,
        source_id: "player".into(),
        model_palette: None,
        texture: atlas.clone(),
        center: Vec2::ZERO,
        size: Vec2::splat(16.),
        flip_x: false,
        above_priority: false,
        facing: None,
    });
    let heights = (0..28 * 28)
        .map(|i| (i % 28) as f32 * 4.)
        .collect::<Vec<_>>();
    let expected = VoxelCameraControls::default()
        .world_pose(&built, &heights)
        .transform();
    let mut live = built.clone();
    live.grid_origin.x += 6;
    live.center.x += live.tile_size.x * 6.;
    let mut future = built.clone();
    future.grid_origin.x += 8;
    future.center.x += future.tile_size.x * 8.;
    let old_root = app
        .world_mut()
        .spawn((VoxelTerrain, Visibility::Visible, Transform::default()))
        .id();
    let next_root = app
        .world_mut()
        .spawn((VoxelTerrain, Visibility::Hidden, Transform::default()))
        .id();
    let camera = app
        .world_mut()
        .spawn((
            Camera::default(),
            Projection::default(),
            Transform::IDENTITY,
            VoxelWorldCamera,
        ))
        .id();
    let quad = app
        .world_mut()
        .resource_mut::<Assets<Mesh>>()
        .add(Mesh::from(Rectangle::new(1., 1.)));
    app.insert_resource(TerrainRevisionCache {
        key: Some(TerrainCacheKey::from_frame(&built)),
        built_frame: Some(built),
        solid_entity: Some(old_root),
        built_footing_heights: heights.clone(),
        footing_heights: heights,
        painted_stone: Some(atlas.clone()),
        painted_bedrock: Some(atlas.clone()),
        ..default()
    })
    .insert_resource(VoxelScene {
        camera: Some(camera),
        actor_quad: Some(quad),
    })
    .insert_resource(live.clone())
    .add_systems(Update, sync_voxel_view);
    app.update();
    assert!(app.world().resource::<VoxelViewStatus>().active);
    assert!(!app.world().resource::<VoxelViewStatus>().terrain_pending);
    app.world_mut().clear_trackers();
    // Only speculative work changes. The player, viewport and settings retain
    // their change ticks, so the old idle fast path would skip support alignment.
    app.world_mut().resource_mut::<PreparedWorld>().next = PreparedDestination {
        ready_frames: 3,
        cache: TerrainRevisionCache {
            key: Some(TerrainCacheKey::from_frame(&future)),
            built_frame: Some(future.clone()),
            solid_entity: Some(next_root),
            built_footing_heights: (0..28 * 28).map(|i| (i % 28 + 8) as f32 * 4.).collect(),
            footing_heights: (0..28 * 28).map(|i| (i % 28 + 8) as f32 * 4.).collect(),
            footing_origin: Some(future.grid_origin),
            footing_map: Some(future.map_id),
            painted_stone: Some(atlas.clone()),
            painted_bedrock: Some(atlas),
            ..default()
        },
        ..default()
    };
    app.update();
    assert_eq!(
        app.world()
            .resource::<crate::WorldStreamStatus>()
            .promotions,
        1
    );
    let cache = app.world().resource::<TerrainRevisionCache>();
    assert_eq!(cache.solid_entity, Some(next_root));
    assert_eq!(cache.footing_origin, Some(live.grid_origin));
    assert_eq!(
        *app.world()
            .get::<Transform>(camera)
            .expect("stationary camera"),
        expected
    );
    assert_eq!(app.world().resource::<TerrainBuildQueue>().started, 0);
}
