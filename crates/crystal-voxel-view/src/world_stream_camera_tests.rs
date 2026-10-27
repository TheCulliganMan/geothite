// Exercise the production camera while a same-map replacement remains pending.
#[test]
#[cfg(not(target_arch = "wasm32"))]
fn pending_world_window_aligns_support_before_resolving_the_camera() {
    check_pending_window(false, "SupportTest");
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn production_route_and_doorway_keep_the_complete_view_while_replacement_loads() {
    for map in ["SupportTest", "RouteB", "GateHouse"] {
        check_pending_window(true, map);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn check_pending_window(gpu_required: bool, map: &str) {
    let (mut built, _) = crate::world_stream::test_frames::connected_frames(IVec2::new(40, 0));
    built.map_id = "SupportTest".into();
    std::sync::Arc::make_mut(&mut built.source_regions)[0].map_id = built.map_id.clone();
    built.actors.push(VisualActor {
        id: VisualActorId::Player,
        source_id: "player".into(),
        model_palette: None,
        texture: Handle::default(),
        center: Vec2::ZERO,
        size: Vec2::splat(16.),
        flip_x: false,
        above_priority: false,
        facing: None,
    });
    let heights = (0..28 * 28)
        .map(|i| (i % 28) as f32 * 4.)
        .collect::<Vec<_>>();
    let mut app = App::new();
    app.init_resource::<Assets<Image>>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<VoxelMaterial>>()
        .init_resource::<live_profiles::LiveProfiles>()
        .init_resource::<VoxelViewSettings>()
        .init_resource::<VoxelViewStatus>()
        .init_resource::<NativeHealingEffectsStatus>()
        .init_resource::<ActorIdCache>();
    app.world_mut().resource_mut::<VoxelViewSettings>().enabled = true;
    let atlas = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::default());
    built.map_texture = atlas.clone();
    for tile in &mut built.tiles {
        tile.texture = atlas.clone();
    }
    built.actors[0].texture = atlas.clone();
    let expected = VoxelCameraControls::default()
        .world_pose(&built, &heights)
        .transform();
    let mut live = built.clone();
    live.map_id = map.into();
    live.grid_origin.x += 8;
    live.center.x += live.tile_size.x * 8.;
    if map == "RouteB" {
        live.grid_origin.x -= 40;
        live.source_regions = vec![
            crystal_render_api::VisualSourceRegion {
                map_id: map.into(),
                origin: IVec2::ZERO,
                size: UVec2::splat(40),
            },
            crystal_render_api::VisualSourceRegion {
                map_id: built.map_id.clone(),
                origin: IVec2::new(-40, 0),
                size: UVec2::splat(40),
            },
        ]
        .into();
    } else {
        std::sync::Arc::make_mut(&mut live.source_regions)[0].map_id = live.map_id.clone();
    }
    live.validate().expect("complete destination frame");
    assert!(
        !terrain_tracking::can_reuse(&built, &live),
        "replacement work stays pending"
    );
    let pool = bevy::tasks::AsyncComputeTaskPool::get_or_init(|| {
        bevy::tasks::TaskPoolBuilder::new().num_threads(2).build()
    });
    let builds = TerrainBuildQueue {
        key: Some(TerrainCacheKey::from_frame(&live)),
        task: Some(pool.spawn(std::future::pending::<TerrainBuildResult>())),
        ..default()
    };
    let camera = app
        .world_mut()
        .spawn((
            Camera::default(),
            Projection::default(),
            expected,
            VoxelWorldCamera,
        ))
        .id();
    let quad = app
        .world_mut()
        .resource_mut::<Assets<Mesh>>()
        .add(Mesh::from(Rectangle::new(1., 1.)));
    let root_pose = terrain_transform(&built);
    let root = app
        .world_mut()
        .spawn((VoxelTerrain, Visibility::Visible, root_pose))
        .id();
    app.insert_resource(builds)
        .insert_resource(TerrainRevisionCache {
            key: Some(TerrainCacheKey::from_frame(&built)),
            built_frame: Some(built),
            solid_entity: Some(root),
            built_footing_heights: heights.clone(),
            footing_heights: heights,
            painted_stone: Some(atlas.clone()),
            painted_bedrock: Some(atlas),
            ..default()
        })
        .insert_resource(VoxelScene {
            camera: Some(camera),
            actor_quad: Some(quad),
        })
        .insert_resource(live)
        .add_systems(Update, sync_voxel_view);
    app.world_mut()
        .resource_mut::<TerrainRevisionCache>()
        .uploads
        .require_gpu(gpu_required);
    if gpu_required {
        app.world_mut().resource_mut::<VoxelViewStatus>().active = true;
    }
    for _ in 0..2 {
        app.update();
        let status = app.world().resource::<VoxelViewStatus>();
        assert!(status.active, "{:?}", status.inactive_reason);
        assert!(status.terrain_pending);
        assert_eq!(status.scene_held, gpu_required);
        assert_eq!(
            app.world().get::<Visibility>(root),
            Some(&Visibility::Visible)
        );
        assert_eq!(app.world().get::<Transform>(root), Some(&root_pose));
        assert_eq!(
            *app.world().get::<Transform>(camera).expect("world camera"),
            expected,
            "scrolling a retained support grid must not move a stationary camera"
        );
    }
}
