#[test]
fn party_follower_uses_first_slot_and_preserves_authoritative_state() {
    use crystal_render_api::{VisualActorId, VisualWorldFrame};
    let root = AssetRoot::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap(),
    );
    let runtime = workspace_desktop_runtime(&root);
    let mut shell = initialize_bevy_runtime_shell(
        root,
        runtime,
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: 14,
            map_name: "CherrygroveCity".into(),
            tile_x: 29,
            tile_y: 4,
        },
        BevyShellConfig {
            render_test_party: true,
            ..Default::default()
        },
    )
    .unwrap();
    let trainer = shell.shell.snapshot().unwrap().trainer;
    shell
        .shell
        .add_party_pokemon(
            "PIKACHU",
            10,
            None,
            None,
            "TEST",
            trainer.player_id,
            Dv::from_non_hp(10, 10, 10, 10),
        )
        .unwrap();
    let original_position = shell.shell.snapshot().unwrap().overworld.tile;
    let original_party = shell.shell.session().state().storage.party.clone();
    let mut app = integrated_shell_test_app(shell);
    app.init_resource::<VisualWorldFrame>()
        .insert_resource(crystal_voxel_view::VoxelViewSettings {
            enabled: true,
            ..default()
        })
        .add_systems(
            Update,
            publish_visual_world_frame
                .after(render_playfield)
                .after(sync_visible_player_sprite)
                .after(sync_visible_object_sprites),
        )
        .add_systems(
            Update,
            publish_party_follower.after(publish_visual_world_frame),
        );
    app.update();
    app.update();
    let frame = app.world().resource::<VisualWorldFrame>();
    assert!(frame.active);
    let follower = frame
        .actors
        .iter()
        .find(|a| a.id == VisualActorId::PartyFollower)
        .unwrap();
    assert_eq!(&*follower.source_id, "species:TOTODILE:icon_monster");
    assert!(follower.model_palette.is_none(), "location preview Totodile must use its normal blue model palette");
    let player = frame
        .actors
        .iter()
        .find(|a| a.id == VisualActorId::Player)
        .unwrap();
    assert!(
        (crystal_voxel_view::actor_foot(player).distance(crystal_voxel_view::actor_foot(follower))
            - TILE_SIZE * 2.0)
            .abs()
            < 0.001
    );
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(
        shell.shell.snapshot().unwrap().overworld.tile,
        original_position
    );
    assert_eq!(shell.shell.session().state().storage.party, original_party);
    app.world_mut()
        .resource_mut::<BevyRuntimeShell>()
        .shell
        .swap_party_pokemon(0, 1)
        .unwrap();
    mark_runtime_snapshot_dirty(&mut app.world_mut().resource_mut::<BevyRuntimeShell>());
    app.update();
    let frame = app.world().resource::<VisualWorldFrame>();
    assert_eq!(
        frame
            .actors
            .iter()
            .filter(|a| a.id == VisualActorId::PartyFollower)
            .count(),
        1
    );
    assert_eq!(
        &*frame
            .actors
            .iter()
            .find(|a| a.id == VisualActorId::PartyFollower)
            .unwrap()
            .source_id,
        "species:PIKACHU:icon_monster"
    );
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ArrowLeft);
    for _ in 0..48 {
        app.update();
    }
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::ArrowLeft);
    let shell = app.world().resource::<BevyRuntimeShell>();
    let moved = shell.shell.snapshot().unwrap().overworld.tile;
    assert!(
        moved.x < original_position.x,
        "production keyboard movement must advance the player"
    );
    assert_eq!(shell.last_error, None);
    let frame = app.world().resource::<VisualWorldFrame>();
    let player = frame
        .actors
        .iter()
        .find(|a| a.id == VisualActorId::Player)
        .unwrap();
    let follower = frame
        .actors
        .iter()
        .find(|a| a.id == VisualActorId::PartyFollower)
        .unwrap();
    let delta = crystal_voxel_view::actor_foot(follower) - crystal_voxel_view::actor_foot(player);
    assert!(
        delta.abs_diff_eq(Vec2::X * TILE_SIZE * 2.0, 0.01),
        "the follower must stay one tile behind actual rendered movement, got {delta:?}"
    );
    app.world_mut()
        .resource_mut::<crystal_voxel_view::VoxelViewSettings>()
        .enabled = false;
    app.update();
    assert!(!app.world().resource::<VisualWorldFrame>().active);
}

#[test]
fn surf_mount_keeps_actual_surf_user_after_party_reorder_and_save() {
    use crystal_render_api::{VisualActorId, VisualSurfRider, VisualWorldFrame};
    let mut app = surf_anchor_app_at_shore();
    app.init_resource::<VisualSurfRider>()
        .insert_resource(crystal_voxel_view::VoxelViewSettings {
            enabled: true,
            ..default()
        })
        .add_systems(
            Update,
            publish_party_follower.after(publish_visual_world_frame),
        )
        .add_systems(Update, publish_surf_mount.after(publish_party_follower));
    surf_anchor_mount_through_keyboard(&mut app);
    app.update();
    let snapshot = app
        .world()
        .resource::<BevyRuntimeShell>()
        .shell
        .snapshot()
        .unwrap();
    let (species, dvs) = visible_surf_actor(&snapshot).expect("retained Surf user");
    let frame = app.world().resource::<VisualWorldFrame>();
    let mount = frame
        .actors
        .iter()
        .find(|a| a.id == VisualActorId::SurfMount)
        .unwrap();
    assert_eq!(&*mount.source_id, format!("species:{species}:icon_monster"));
    assert!(
        !frame
            .actors
            .iter()
            .any(|a| a.id == VisualActorId::PartyFollower),
        "the lead Surf user must not be duplicated behind the rider"
    );
    let player = frame
        .actors
        .iter()
        .find(|a| a.id == VisualActorId::Player)
        .unwrap();
    assert_eq!(
        player.source_id.as_ref(),
        "surf",
        "source battle witness must be preserved"
    );
    assert_eq!(
        crystal_voxel_view::actor_foot(player),
        crystal_voxel_view::actor_foot(mount)
    );
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        let trainer = shell.shell.snapshot().unwrap().trainer;
        shell
            .shell
            .add_party_pokemon(
                "PIKACHU",
                10,
                None,
                None,
                "TEST",
                trainer.player_id,
                Dv::from_non_hp(10, 10, 10, 10),
            )
            .unwrap();
        shell.shell.swap_party_pokemon(0, 1).unwrap();
        mark_runtime_snapshot_dirty(&mut shell);
        assert_eq!(
            visible_surf_actor(&shell.shell.snapshot().unwrap()),
            Some((species.clone(), dvs))
        );
        let mut evolved = shell.shell.snapshot().unwrap();
        evolved.party.slots[1].pokemon.species.id = "FERALIGATR".into();
        assert_eq!(
            visible_surf_actor(&evolved),
            Some(("FERALIGATR".into(), dvs)),
            "the same Surf user must show its evolved appearance"
        );
        let path = std::env::temp_dir().join(format!(
            "geothite-surf-mount-{}.crystalsave",
            std::process::id()
        ));
        shell.shell.save(&path).unwrap();
        if let Some(qa_path) = std::env::var_os("CRYSTAL_COMPANION_QA_SAVE") {
            let qa_path = PathBuf::from(qa_path);
            if let Some(parent) = qa_path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            shell.shell.save(qa_path).unwrap();
        }
        let saved = shell.shell.runtime().load_save(&path).unwrap();
        assert_eq!(
            saved
                .script_runtime
                .memory
                .get(crate::core::systems::field_moves::SURF_ACTOR_SPECIES_MEMORY),
            Some(&species)
        );
        std::fs::remove_file(path).unwrap();
    }
    app.update();
    assert_eq!(
        app.world()
            .resource::<VisualWorldFrame>()
            .actors
            .iter()
            .find(|a| a.id == VisualActorId::PartyFollower)
            .unwrap()
            .source_id
            .as_ref(),
        "species:PIKACHU:icon_monster",
        "a different lead must follow on water"
    );
    assert_eq!(
        app.world()
            .resource::<VisualWorldFrame>()
            .actors
            .iter()
            .find(|a| a.id == VisualActorId::SurfMount)
            .unwrap()
            .source_id
            .as_ref(),
        format!("species:{species}:icon_monster")
    );
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        let trainer = shell.shell.snapshot().unwrap().trainer;
        shell
            .shell
            .add_party_pokemon(&species, 20, None, None, "TEST", trainer.player_id, dvs)
            .unwrap();
        shell.shell.swap_party_pokemon(0, 2).unwrap();
        mark_runtime_snapshot_dirty(&mut shell);
        let snapshot = shell.shell.snapshot().unwrap();
        assert_eq!(snapshot.party.slots[0].pokemon.species.id, species);
        assert_eq!(snapshot.party.slots[0].pokemon.dvs, dvs);
        assert!(
            !visible_lead_is_surf_actor(&snapshot),
            "an identical second Pokémon must still follow"
        );
    }
    app.update();
    assert!(
        app.world()
            .resource::<VisualWorldFrame>()
            .actors
            .iter()
            .any(|actor| actor.id == VisualActorId::PartyFollower)
    );
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        shell.shell.swap_party_pokemon(0, 1).unwrap();
        mark_runtime_snapshot_dirty(&mut shell);
        assert!(visible_lead_is_surf_actor(&shell.shell.snapshot().unwrap()));
    }
    app.update();
    assert!(
        !app.world()
            .resource::<VisualWorldFrame>()
            .actors
            .iter()
            .any(|actor| actor.id == VisualActorId::PartyFollower)
    );
}
