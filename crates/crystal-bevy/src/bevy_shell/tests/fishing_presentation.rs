// The encounter commits immediately, but its original rod animation and bite
// notice still own presentation. Location metadata may freeze during that wait.
fn assert_fishing_presentation_retains_the_field(pack: bool) {
    use bevy::ecs::system::RunSystemOnce;
    let (mut app, _) = fishing_anchor_app();
    let original_world = app
        .world()
        .resource::<crystal_render_api::VisualWorldFrame>()
        .clone();
    if pack {
        fishing_anchor_open_pack_rod(&mut app);
        assert!(
            !app.world()
                .resource::<crystal_render_api::VisualWorldFrame>()
                .active,
            "the fullscreen Pack really suspended optional world publication"
        );
    }
    let samples = fishing_anchor_set_trace(&mut app, true, true);
    press_key_for_runtime_hotkey_app(
        &mut app,
        if pack {
            KeyCode::KeyZ
        } else {
            KeyCode::ShiftRight
        },
    );
    let (session, origin, expected_frames) = {
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(shell.last_error, None);
        assert!(
            shell.shell.snapshot().unwrap().battle.is_some(),
            "real cast already committed"
        );
        assert!(shell.field_notice_scene.as_ref().unwrap().battle.is_none());
        let animation = shell.visible_fishing_animation.unwrap();
        assert_eq!(animation.phase, VisibleFishingPhase::Cast);
        assert_eq!(animation.frame, 0);
        let expected_frames = 40 + if animation.facing_up { 33 } else { 32 } + 40;
        (
            shell.shell.session().clone(),
            shell.battle_origin.pending.clone().unwrap(),
            expected_frames,
        )
    };
    assert!(
        app.world()
            .resource::<crystal_render_api::BattleLocationFrame>()
            .location
            .is_some(),
        "checked scenery may be retained before battle presentation starts"
    );
    let mut phases = std::collections::HashSet::new();
    let mut frames = 0;
    loop {
        let phase = {
            let shell = app.world().resource::<BevyRuntimeShell>();
            assert_eq!(shell.last_error, None);
            assert!(
                !shell.battle_lcd_animation_active,
                "fishing owns the field, including Cast frame zero"
            );
            assert!(shell.battle_message_scene.is_none());
            assert!(Arc::ptr_eq(
                shell.battle_origin.pending.as_ref().unwrap(),
                &origin
            ));
            assert_eq!(
                shell.shell.session(),
                &session,
                "presentation must not add gameplay or DIV ticks"
            );
            shell.visible_fishing_animation.unwrap().phase
        };
        phases.insert(phase);
        let world = app.world_mut();
        let visible_world = world.resource::<crystal_render_api::VisualWorldFrame>();
        assert!(
            visible_world.active,
            "a complete field must be published during {phase:?}, pack={pack}"
        );
        assert!(visible_world.validate().is_ok());
        assert_eq!(visible_world.map_id, original_world.map_id);
        assert_eq!(
            visible_world.source_map_size_core_tiles,
            original_world.source_map_size_core_tiles
        );
        assert_eq!(visible_world.grid_origin, original_world.grid_origin);
        assert_eq!(visible_world.grid_size, original_world.grid_size);
        assert!(
            visible_world
                .actors
                .iter()
                .any(|actor| actor.id == crystal_render_api::VisualActorId::Player)
        );
        // The classic raster remains complete as well: 3D recovery must not
        // depend on changing the user's chosen render layers or hiding 2D errors.
        let _surfaces = retained_map_surface_pair(world);
        assert_eq!(
            world
                .query_filtered::<Entity, With<PlayerMarker>>()
                .iter(world)
                .count(),
            1
        );
        assert_eq!(
            world
                .query_filtered::<Entity, With<VisibleIntroSurface>>()
                .iter(world)
                .count(),
            0,
            "the closed Pack must not leave an opaque fullscreen presenter"
        );
        if frames == 0 || phase == VisibleFishingPhase::AwaitText {
            let texture = world
                .resource::<RenderedViewport>()
                .map_texture
                .as_ref()
                .unwrap();
            let raster = world.resource::<Assets<Image>>().get(texture).unwrap();
            let colors: std::collections::HashSet<_> = raster
                .data
                .chunks_exact(4)
                .filter(|pixel| pixel[3] > 0)
                .map(|pixel| [pixel[0], pixel[1], pixel[2]])
                .collect();
            assert!(
                colors.len() > 4,
                "the retained classic map raster must contain actual field art"
            );
        }
        let battle_entities = world
            .query_filtered::<Entity, Or<(
                With<BattleBattlerMarker>,
                With<BattleHudMarker>,
                With<BattleCommandMarker>,
                With<FixedBattleCanvasMarker>,
            )>>()
            .iter(world)
            .count();
        assert_eq!(
            battle_entities, 0,
            "no premature battle HUD, command menu, or battlers in {phase:?}"
        );
        #[cfg(feature = "voxel-view")]
        assert!(
            !world
                .resource::<crystal_render_api::VisualBattleFrame>()
                .active,
            "the optional renderer must also observe no active battle during fishing"
        );
        if phase == VisibleFishingPhase::AwaitText {
            break;
        }
        assert!(frames < expected_frames);
        advance_visible_fishing_animation(&mut world.resource_mut::<BevyRuntimeShell>());
        world.run_system_once(render_playfield);
        world.run_system_once(publish_visual_world_frame);
        frames += 1;
    }
    assert_eq!(
        frames, expected_frames,
        "retain the exact authored phase durations"
    );
    assert_eq!(
        phases,
        [
            VisibleFishingPhase::Cast,
            VisibleFishingPhase::Hook,
            VisibleFishingPhase::Pause,
            VisibleFishingPhase::AwaitText
        ]
        .into_iter()
        .collect()
    );
    let mut controller = VisibleShellController {
        shell: app
            .world_mut()
            .remove_resource::<BevyRuntimeShell>()
            .unwrap(),
    };
    fishing_origin_assert_consumed_divider(&controller, samples.len() + 2);
    assert_eq!(
        controller.shell.field_notice.as_deref(),
        Some("Oh!\nA bite!")
    );
    assert!(controller.shell.pending_field_battle_entry);
    controller.wait_frames(1).unwrap();
    controller.press(GameButton::A).unwrap();
    assert!(controller.shell.visible_fishing_animation.is_none());
    assert!(!controller.shell.pending_field_battle_entry);
    assert!(
        controller
            .shell
            .battle_message_scene
            .as_ref()
            .unwrap()
            .battle
            .is_some()
    );
    assert!(Arc::ptr_eq(
        controller.shell.battle_origin.active.as_ref().unwrap(),
        &origin
    ));
    app.insert_resource(controller.shell);
    app.world_mut().run_system_once(render_playfield);
    assert!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .battle_lcd_animation_active,
        "acknowledgment releases the ordinary source battle-entry sequence"
    );
}

#[test]
fn fishing_presentation_never_leaks_battle_before_the_authored_bite_notice() {
    assert_fishing_presentation_retains_the_field(false);
}

#[test]
fn fishing_presentation_pack_use_restores_positive_world_publication_and_classic_field() {
    assert_fishing_presentation_retains_the_field(true);
}
