use super::*;

#[test]
fn human_cards_never_spawn_or_reappear_while_a_model_is_warming() {
    let mut app = App::new();
    app.init_resource::<VisualWorldFrame>()
        .init_resource::<ActorIdCache>()
        .init_resource::<Assets<Image>>()
        .init_resource::<Assets<VoxelMaterial>>()
        .add_systems(Update, test_cards);
    // Deliberately missing textures and footing: model preparation may still
    // be running. A human must not silently acquire a temporary billboard.
    for source in [
        "chris",
        "kris",
        "falkner",
        "youngster",
        "rocket",
        "swimmer_guy",
    ] {
        app.world_mut().resource_mut::<VisualWorldFrame>().actors = vec![VisualActor {
            id: VisualActorId::Player,
            source_id: source.into(),
            texture: Handle::default(),
            center: Vec2::ZERO,
            size: Vec2::splat(16.),
            flip_x: false,
            above_priority: false,
            facing: None,
            model_palette: None,
        }];
        app.update();
        assert!(
            app.world().resource::<ActorIdCache>().entities.is_empty(),
            "{source}"
        );
    }
}

#[allow(clippy::type_complexity)]
fn test_cards(
    frame: Res<VisualWorldFrame>,
    mut commands: Commands,
    mut cache: ResMut<ActorIdCache>,
    images: Res<Assets<Image>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
    mut entities: Query<
        (
            &mut Transform,
            &mut Visibility,
            &mut Mesh3d,
            &mut MeshMaterial3d<VoxelMaterial>,
        ),
        (
            With<VoxelActorCard>,
            Without<VoxelActorSilhouette>,
            Without<VoxelWorldCamera>,
            Without<VoxelTerrain>,
        ),
    >,
) {
    assert!(
        sync_actor_cards(
            &frame,
            &[],
            camera_pose(Vec2::splat(160.)),
            false,
            &Handle::default(),
            &mut commands,
            &mut cache,
            &images,
            &mut materials,
            &mut entities
        )
        .is_ok()
    );
}
