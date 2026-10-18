use super::*;

fn synchronize_surf_cards(
    frame: Res<VisualWorldFrame>,
    mut commands: Commands,
    mut cache: ResMut<ActorIdCache>,
    images: Res<Assets<Image>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
    mut cards: Query<
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
    sync_actor_cards(
        &frame,
        &[],
        VoxelCameraControls::default().pose(frame.viewport_size),
        false,
        &Handle::default(),
        &mut commands,
        &mut cache,
        &images,
        &mut materials,
        &mut cards,
    )
    .expect("Surf never uses a substitute sprite card, even without artwork");
}

#[test]
fn surf_rider_has_no_generic_avatar_or_mount_card_fallback() {
    for source in ["surf", "surfing_pikachu", "gfx/sprites/surf.png"] {
        let mut app = App::new();
        let mut frame = VisualWorldFrame::default();
        frame.map_id = "NewBarkTown".into();
        frame.viewport_size = Vec2::new(160., 144.);
        frame.tile_size = Vec2::splat(8.);
        let player = VisualActor {
            id: VisualActorId::Player,
            source_id: source.into(),
            model_palette: None,
            texture: Handle::default(),
            center: Vec2::ZERO,
            size: Vec2::splat(16.),
            flip_x: false,
            above_priority: false,
            facing: Some(Vec2::NEG_Y),
        };
        assert!(new_bark_actors::is_surf_avatar(&player));
        let mut mount = player.clone();
        mount.id = VisualActorId::SurfMount;
        mount.source_id = "species:GOLDEEN:icon_monster".into();
        frame.actors = vec![player, mount];
        app.insert_resource(frame)
            .init_resource::<ActorIdCache>()
            .init_resource::<Assets<Image>>()
            .init_resource::<Assets<VoxelMaterial>>()
            .add_systems(Update, synchronize_surf_cards);
        // A source card retained from an earlier frame must be retired, too.
        let stale = app.world_mut().spawn(VoxelActorCard).id();
        app.world_mut()
            .resource_mut::<ActorIdCache>()
            .entities
            .insert(VisualActorId::Player, stale);
        app.update();
        assert!(app.world().resource::<ActorIdCache>().entities.is_empty());
        assert!(app.world().get_entity(stale).is_err());
    }
}
