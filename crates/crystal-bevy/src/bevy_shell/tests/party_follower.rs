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
        .init_resource::<crystal_render_api::VisualPokemonDimensions>()
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
            publish_pokemon_dimensions.after(publish_visual_world_frame),
        )
        .add_systems(
            Update,
            publish_party_follower.after(publish_pokemon_dimensions),
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
fn party_follower_large_body_uses_pack_dimensions_without_moving_gameplay_or_resizing_its_card() {
    use bevy::ecs::system::RunSystemOnce;
    use crystal_render_api::{VisualActorId,VisualWorldFrame,VisualPokemonDimensions};
    let root=AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap());
    let runtime=workspace_desktop_runtime(&root);
    let mut shell=initialize_bevy_runtime_shell(root,runtime,
        BevyShellStart::NewGameAtRuntimeTile {spawn_identifier:14,map_name:"Route36".into(),tile_x:20,tile_y:12},
        BevyShellConfig {render_test_party:true,..default()}).unwrap();
    let trainer=shell.shell.snapshot().unwrap().trainer;
    shell.shell.add_party_pokemon("ARCANINE",20,None,None,"TEST",trainer.player_id,Dv::from_non_hp(10,10,10,10)).unwrap();
    shell.shell.swap_party_pokemon(0,1).unwrap();
    shell.shell.session_mut().overworld_mut().set_player_facing(Direction::Down);
    mark_runtime_snapshot_dirty(&mut shell);
    let mut app=integrated_shell_test_app(shell);
    app.init_resource::<VisualWorldFrame>().init_resource::<VisualPokemonDimensions>()
        .insert_resource(crystal_voxel_view::VoxelViewSettings {enabled:true,..default()})
        .add_systems(Update,publish_visual_world_frame.after(render_playfield).after(sync_visible_player_sprite).after(sync_visible_object_sprites))
        .add_systems(Update,publish_pokemon_dimensions.after(publish_visual_world_frame))
        .add_systems(Update,publish_party_follower.after(publish_pokemon_dimensions));
    app.update();app.update();
    // Initial app updates legitimately run gameplay systems. Isolate the
    // production presentation publisher after that initial settling.
    let before=serde_json::to_value(app.world().resource::<BevyRuntimeShell>().shell.session().state()).unwrap();
    app.world_mut().run_system_once(publish_party_follower).unwrap();
    let frame=app.world().resource::<VisualWorldFrame>();
    let player=frame.actors.iter().find(|a|a.id==VisualActorId::Player).unwrap();
    let follower=frame.actors.iter().find(|a|a.id==VisualActorId::PartyFollower).expect("clear road supports the complete body gap");
    assert_eq!(&*follower.source_id,"species:ARCANINE:icon_monster");
    assert_eq!(follower.size,Vec2::splat(frame.tile_size.y*2.));
    let distance=crystal_voxel_view::actor_foot(follower).distance(crystal_voxel_view::actor_foot(player));
    assert!(distance>frame.tile_size.y*3.,"the large animal must have more than one tile of space");
    let shell=app.world().resource::<BevyRuntimeShell>();
    let dex=&shell.shell.runtime().data().pokedex_entries["ARCANINE"];
    assert_eq!(app.world().resource::<VisualPokemonDimensions>().meters["ARCANINE"],pokedex_dimension_meters(dex.height_digits).unwrap());
    assert_eq!(serde_json::to_value(shell.shell.session().state()).unwrap(),before);
}

#[test]
fn surf_rider_mount_uses_the_real_field_move_with_a_non_lead_user() {
    assert_surf_rider_with_non_lead_user("LAPRAS", "CRYSTAL_SURF_RIDER_QA_SAVE");
}

#[test]
fn surf_rider_mount_supports_goldeen_as_the_actual_surf_user() {
    assert_surf_rider_with_non_lead_user("GOLDEEN", "CRYSTAL_GOLDEEN_QA_SAVE");
}

fn assert_surf_rider_with_non_lead_user(surf_species: &str, save_env: &str) {
    use crystal_render_api::{VisualActorId, VisualSurfRider, VisualWorldFrame};
    let root = AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..").canonicalize().unwrap());
    let runtime = workspace_desktop_runtime(&root);
    let spawn_identifier = runtime.title_new_game_spawn_identifier().unwrap();
    let mut shell = initialize_bevy_runtime_shell(root, runtime,
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier, map_name: "Route44".into(), tile_x: 38, tile_y: 3,
        }, BevyShellConfig { quick_save_path: None, ..default() }).unwrap();
    let trainer = shell.shell.snapshot().unwrap().trainer;
    for species in ["EEVEE", surf_species] {
        shell.shell.add_party_pokemon(species, 20, None, None, "TEST",
            trainer.player_id, Dv::from_non_hp(9, 9, 9, 9)).unwrap();
    }
    shell.shell.add_bag_item("HM_SURF", 1).unwrap();
    shell.shell.use_bag_tmhm_on_party_pokemon("HM_SURF", 1, Some(0)).unwrap();
    shell.shell.award_badge(RuntimeBadgeRegion::Johto, 3).unwrap();
    shell.shell.session_mut().overworld_mut().set_player_facing(Direction::Down);
    mark_runtime_snapshot_dirty(&mut shell);
    settle_visible_shell_smoke_until_idle(&mut shell).unwrap();
    let mut app = menu_render_test_app(shell);
    app.init_resource::<VisualWorldFrame>()
        .init_resource::<VisualSurfRider>()
        .init_resource::<crystal_render_api::VisualPokemonDimensions>()
        .init_resource::<crystal_render_api::BattleLocationFrame>()
        .insert_resource(crystal_voxel_view::VoxelViewSettings { enabled: true, ..default() })
        .add_systems(Update, publish_visual_world_frame.after(render_playfield)
            .after(sync_visible_player_sprite).after(sync_visible_object_sprites))
        .add_systems(Update, publish_pokemon_dimensions.after(publish_visual_world_frame))
        .add_systems(Update, publish_party_follower.after(publish_pokemon_dimensions))
        .add_systems(Update, publish_surf_mount.after(publish_party_follower));
    app.update();
    app.update();
    surf_anchor_mount_through_keyboard(&mut app);
    app.update();
    assert_eq!(app.world().resource::<BevyRuntimeShell>().shell.session().snapshot().mode,
        if surf_species == "PIKACHU" { MovementMode::SurfPika } else { MovementMode::Surf });
    let frame = app.world().resource::<VisualWorldFrame>();
    let mount = frame.actors.iter().find(|a| a.id == VisualActorId::SurfMount).unwrap();
    let mount_source = format!("species:{surf_species}:icon_monster");
    assert_eq!(mount.source_id.as_ref(), mount_source);
    let snapshot = app.world().resource::<BevyRuntimeShell>().shell.snapshot().unwrap();
    let dex = &snapshot.presentation.pokedex_entries[surf_species];
    assert_eq!(app.world().resource::<crystal_render_api::VisualPokemonDimensions>()
        .meters[surf_species], pokedex_dimension_meters(dex.height_digits).unwrap());
    assert_eq!(frame.actors.iter().find(|a| a.id == VisualActorId::PartyFollower)
        .unwrap().source_id.as_ref(), "species:EEVEE:icon_monster");
    let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
    if let Some(path) = std::env::var_os(save_env) {
        shell.shell.save(PathBuf::from(path)).unwrap();
    }
    shell.shell.swap_party_pokemon(0, 1).unwrap();
    mark_runtime_snapshot_dirty(&mut shell);
    drop(shell);
    app.update();
    let frame = app.world().resource::<VisualWorldFrame>();
    assert_eq!(frame.actors.iter().find(|a| a.id == VisualActorId::SurfMount)
        .unwrap().source_id.as_ref(), mount_source);
    assert!(!frame.actors.iter().any(|a| a.id == VisualActorId::PartyFollower));
    // A later party selection or a missing identity cannot select a different
    // mount, retain yesterday's body, or guess the first Pokémon with Surf.
    {
        use crate::core::systems::field_moves::{SURF_ACTOR_DVS_MEMORY,
            SURF_ACTOR_PARTY_MEMORY, SURF_ACTOR_SPECIES_MEMORY};
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        let memory = &mut shell.shell.session_mut().state_mut().script_runtime.memory;
        for key in [SURF_ACTOR_DVS_MEMORY, SURF_ACTOR_PARTY_MEMORY, SURF_ACTOR_SPECIES_MEMORY] {
            memory.remove(key);
        }
        memory.insert("wCurPartyMon".into(), "0".into());
        mark_runtime_snapshot_dirty(&mut shell);
        let snapshot = shell.shell.snapshot().unwrap();
        assert_eq!(visible_surf_actor(&snapshot), None);
        assert!(!visible_lead_is_surf_actor(&snapshot));
    }
    app.update();
    assert!(!app.world().resource::<VisualWorldFrame>().actors.iter()
        .any(|actor| actor.id == VisualActorId::SurfMount), "stale mounts must be removed");
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


#[test]
fn party_follower_blocked_seed_waits_for_real_history_and_resets_on_warp() {
    let mut trail = PartyFollowerTrail::default();
    assert_eq!(trail.follow("room", Vec2::ZERO, Vec2::Y, 16., 16., false), None);
    assert_eq!(trail.follow("room", Vec2::X * 8., Vec2::X, 16., 16., true), None);
    assert_eq!(trail.follow("room", Vec2::X * 16., Vec2::X, 16., 16., true), Some(Vec2::ZERO));
    assert_eq!(trail.follow("room", Vec2::new(16., 16.), Vec2::Y, 16., 16., true), Some(Vec2::new(16., 0.)));
    assert_eq!(trail.follow("next", Vec2::new(16., 16.), Vec2::Y, 16., 16., false), None);
    assert_eq!(trail.follow("next", Vec2::new(16., 32.), Vec2::Y, 16., 16., true), Some(Vec2::new(16., 16.)));
    assert_eq!(trail.follow("clear", Vec2::ZERO, Vec2::Y, 16., 16., true), Some(Vec2::new(0., -16.)));
}

#[test]
fn party_follower_seed_rejects_real_tower_stool_and_keeps_the_session_unchanged() {
    let root = AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap());
    let runtime = workspace_desktop_runtime(&root);
    let spawn_identifier = runtime.title_new_game_spawn_identifier().unwrap();
    for (x,y,clear) in [(3,9,false),(8,5,true)] {
        let mut shell = initialize_bevy_runtime_shell(root.clone(), runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {spawn_identifier, map_name:"BattleTower1F".into(),tile_x:x,tile_y:y},
            BevyShellConfig {quick_save_path:None,..default()}).unwrap();
        shell.shell.session_mut().overworld_mut().set_player_facing(Direction::Down);
        let before = serde_json::to_value(shell.shell.session().state()).unwrap();
        assert_eq!(party_follower_seed_is_clear(shell.shell.session().overworld(), 1), clear, "Tower({x},{y})");
        assert_eq!(serde_json::to_value(shell.shell.session().state()).unwrap(), before);
        if clear {
            let world = shell.shell.session_mut().overworld_mut();
            world.player.tile = crystal_core::world::movement::checked_move_by_stride(world.player.tile, Direction::Down, 1).unwrap();
            assert!(party_follower_seed_is_clear(world, 2), "both source floor tiles must be clear before placing the fixture object");
            let index = (0..world.objects.len()).find(|i| world.object_has_loaded_struct(*i)).unwrap();
            let id = world.objects[index].object_identifier.clone().unwrap();
            let second = crystal_core::world::movement::checked_move_by_stride(world.player.tile, Direction::Up, 2).unwrap();
            world.object_runtime_tiles.insert(id.clone(), second);
            assert!(party_follower_seed_is_clear(world, 1), "the first tile alone remains clear");
            let before = serde_json::to_value(shell.shell.session().state()).unwrap();
            assert!(!party_follower_seed_is_clear(shell.shell.session().overworld(), 2), "the longer seed must reject an occupied second tile");
            assert_eq!(serde_json::to_value(shell.shell.session().state()).unwrap(), before);
            let world = shell.shell.session_mut().overworld_mut();
            let behind = crystal_core::world::movement::checked_move_by_stride(world.player.tile, Direction::Up, 1).unwrap();
            world.object_runtime_tiles.insert(id, behind);
            assert!(world.occupied_tiles_checked().unwrap().iter().any(|tile|tile.tile==behind));
            let before = serde_json::to_value(shell.shell.session().state()).unwrap();
            assert!(!party_follower_seed_is_clear(shell.shell.session().overworld(), 1), "occupied floor must not seed a follower");
            assert_eq!(serde_json::to_value(shell.shell.session().state()).unwrap(), before);
        }
    }
}

#[test]
fn steelix_skin_review_fixture_requires_actual_clear_trail_at_original_size() {
    use bevy::ecs::system::RunSystemOnce;
    use crystal_render_api::{VisualActorId,VisualPokemonDimensions,VisualWorldFrame};
    let root=AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap());
    let runtime=workspace_desktop_runtime(&root);
    let make_app=|map:&str,x,y| {
        let mut shell=initialize_bevy_runtime_shell(root.clone(),runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {spawn_identifier:runtime.title_new_game_spawn_identifier().unwrap(),map_name:map.into(),tile_x:x,tile_y:y},
            BevyShellConfig {render_test_party:true,quick_save_path:None,..default()}).unwrap();
        let trainer=shell.shell.snapshot().unwrap().trainer;
        shell.shell.add_party_pokemon("STEELIX",20,None,None,"TEST",trainer.player_id,Dv::from_non_hp(10,10,10,10)).unwrap();
        shell.shell.swap_party_pokemon(0,1).unwrap();
        shell.shell.session_mut().overworld_mut().set_player_facing(Direction::Down);
        mark_runtime_snapshot_dirty(&mut shell);
        let mut app=integrated_shell_test_app(shell);
        app.init_resource::<VisualWorldFrame>().init_resource::<VisualPokemonDimensions>()
            .insert_resource(crystal_voxel_view::VoxelViewSettings {enabled:true,..default()})
            .add_systems(Update,publish_visual_world_frame.after(render_playfield).after(sync_visible_player_sprite).after(sync_visible_object_sprites))
            .add_systems(Update,publish_pokemon_dimensions.after(publish_visual_world_frame))
            .add_systems(Update,publish_party_follower.after(publish_pokemon_dimensions));
        app.update();app.update();app
    };
    let original=make_app("Route36",20,8);
    let frame=original.world().resource::<VisualWorldFrame>();
    let dimensions=original.world().resource::<VisualPokemonDimensions>();
    assert!((dimensions.meters["STEELIX"]-9.1948).abs()<0.00001);
    let gap=crystal_voxel_view::pokemon_following_distance(frame,"STEELIX",dimensions).unwrap();
    let step=frame.tile_size.y*2.;let seed_steps=(gap/step).ceil() as u32;
    assert!(gap>step*4.,"the ordinary four-step review cannot establish this body gap");
    let world=original.world().resource::<BevyRuntimeShell>().shell.session().overworld();
    assert!(!party_follower_seed_is_clear(world,seed_steps));
    assert!(!frame.actors.iter().any(|a|a.id==VisualActorId::PartyFollower));
    let mut selected=None;
    for map in ["Route36","EcruteakCity","GoldenrodCity","Route35"] {
        let (width,height)=runtime.data().saved_map_tile_bounds(map).unwrap();
        let mut candidates:Vec<_>=(0..height).flat_map(|y|(0..width).map(move|x|(x as i16,y as i16))).collect();
        candidates.sort_by_key(|&(x,y)|((i32::from(x)-20).abs()+(i32::from(y)-12).abs(),y,x));
        for (x,y) in candidates {
            let Ok(mut probe)=runtime.data().overworld_session(map,crystal_core::world::map::TilePosition::new(x,y),0) else {continue};
            probe.set_player_facing(Direction::Down);
            if !party_follower_seed_is_clear(&probe,seed_steps) {continue;}
            probe.set_player_facing(Direction::Up);
            if !party_follower_seed_is_clear(&probe,4) {continue;}
            selected=Some((map,x,y));break;
        }
        if selected.is_some() {break;}
    }
    let (map,x,y)=selected.expect("a real source corridor must support the full body gap and four following steps");
    let mut app=make_app(map,x,y);
    let before=serde_json::to_value(app.world().resource::<BevyRuntimeShell>().shell.session().state()).unwrap();
    app.world_mut().run_system_once(publish_party_follower).unwrap();
    let frame=app.world().resource::<VisualWorldFrame>();
    let player=frame.actors.iter().find(|a|a.id==VisualActorId::Player).unwrap();
    let follower=frame.actors.iter().find(|a|a.id==VisualActorId::PartyFollower).expect("the checked full trail must publish the actual original-size companion");
    assert!(follower.source_id.starts_with("species:STEELIX:"));
    assert_eq!(follower.size,Vec2::splat(step));
    assert!((crystal_voxel_view::actor_foot(follower).distance(crystal_voxel_view::actor_foot(player))-gap).abs()<0.001);
    assert_eq!(serde_json::to_value(app.world().resource::<BevyRuntimeShell>().shell.session().state()).unwrap(),before);
    eprintln!("Steelix review verified map={map} x={x} y={y} gap_pixels={gap} source_steps={seed_steps}");
}
