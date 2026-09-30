// External-pack regressions for the render-only bridge, alongside the existing
// production controller tests. These use the exact same Route36 battle fixture.
fn immersive_battle_fixture() -> BevyRuntimeShell {
    let mut shell = route36_battle_shell_for_render_regression();
    shell.visible_battle_transition = None;
    shell.visible_battle_sliding_intro = None;
    shell.visible_send_out_animation = None;
    shell.battle_entry_messages_remaining = 0;
    shell.battle_player_send_out_pending = false;
    shell.battle_enemy_send_out_pending = false;
    shell.battle_messages.clear();
    shell.battle_message_scenes.clear();
    shell.battle_text_reveal = None;
    shell.visible_move_animations.clear();
    shell
}

fn extract_immersive_battle_fixture(
    shell: &BevyRuntimeShell,
    snapshot: &RuntimeShellSnapshot,
    active: bool,
) -> VisualBattleFrame {
    let mut world = World::new();
    let mut queue = bevy::ecs::world::CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &world);
    capture_presented_battle(
        &mut commands,
        snapshot,
        shell,
        active,
        &mut RenderedTilesetArt::default(),
        &mut Assets::<Image>::default(),
    )
    .unwrap();
    queue.apply(&mut world);
    world.remove_resource::<VisualBattleFrame>().unwrap()
}

#[test]
fn immersive_battle_extracts_presented_party_slot_without_mutating_authority() {
    let shell = immersive_battle_fixture();
    let before = shell.shell.snapshot().unwrap();
    let mut presented = before.clone();
    // A renderer must use the currently presented slot identity, not assume
    // lead index zero or request a newer runtime snapshot behind the dialogue.
    presented.party.slots[0].index = 3;
    presented.battle.as_mut().unwrap().active_player_party_index = Some(3);
    let frame = extract_immersive_battle_fixture(&shell, &presented, true);
    assert!(frame.active);
    assert_eq!(frame.validate(), Ok(()));
    assert_eq!(frame.battlers[0].as_ref().unwrap().party_index, Some(3));
    assert_eq!(
        frame.battlers[0].as_ref().unwrap().species_id.as_ref(),
        "CYNDAQUIL"
    );
    assert_eq!(
        shell.shell.snapshot().unwrap(),
        before,
        "render extraction must leave battle, PP, HP and all controller state unchanged"
    );
}

#[test]
fn immersive_battle_uses_retained_transform_without_sampling_future_authority() {
    let shell = immersive_battle_fixture();
    let mut presented = shell.shell.snapshot().unwrap();
    presented
        .battle
        .as_mut()
        .unwrap()
        .player_transformed_species = Some("TOTODILE".into());
    let frame = extract_immersive_battle_fixture(&shell, &presented, true);
    assert_eq!(
        frame.battlers[0].as_ref().unwrap().species_id.as_ref(),
        "TOTODILE"
    );
    assert!(
        shell
            .shell
            .snapshot()
            .unwrap()
            .battle
            .unwrap()
            .player_transformed_species
            .is_none()
    );
}

#[test]
fn immersive_battle_closes_for_world_and_waits_for_existing_intro() {
    let mut shell = immersive_battle_fixture();
    let snapshot = shell.shell.snapshot().unwrap();
    assert!(!extract_immersive_battle_fixture(&shell, &snapshot, false).active);
    shell.battle_entry_messages_remaining = 1;
    assert!(!extract_immersive_battle_fixture(&shell, &snapshot, true).active);
    shell.battle_entry_messages_remaining = 0;
    let mut world = snapshot;
    world.battle = None;
    assert!(!extract_immersive_battle_fixture(&shell, &world, true).active);
}

#[test]
fn immersive_battle_substitute_keeps_honest_source_art() {
    let shell = immersive_battle_fixture();
    let mut snapshot = shell.shell.snapshot().unwrap();
    snapshot.battle.as_mut().unwrap().player_substitute_hp = 12;
    let frame = extract_immersive_battle_fixture(&shell, &snapshot, true);
    assert!(!frame.battlers[0].as_ref().unwrap().allow_species_model);
    assert_ne!(
        frame.battlers[0].as_ref().unwrap().texture,
        Handle::default()
    );
}

#[test]
fn immersive_battle_move_without_visible_hp_loss_never_implies_impact() {
    let shell = immersive_battle_fixture();
    let snapshot = shell.shell.snapshot().unwrap();
    let frame = extract_immersive_battle_fixture(&shell, &snapshot, true);
    assert!(
        !frame
            .cues
            .iter()
            .any(|cue| cue.kind == VisualBattleCueKind::Impact)
    );
}

#[test]
fn immersive_battle_source_hud_eraser_does_not_restore_fainted_hp() {
    let (player_x, player_y) = field_window_center(9.0, 7.0, 11.0, 5.0);
    let (enemy_x, enemy_y) = field_window_center(1.0, 0.0, 10.0, 4.0);
    assert!(immersive_battle_hud_is_erased(
        Vec2::new(player_x, player_y),
        [true, false]
    ));
    assert!(!immersive_battle_hud_is_erased(
        Vec2::new(enemy_x, enemy_y),
        [true, false]
    ));
    assert!(immersive_battle_hud_is_erased(
        Vec2::new(enemy_x, enemy_y),
        [false, true]
    ));
    assert!(!immersive_battle_hud_is_erased(Vec2::ZERO, [false, false]));
}

#[test]
fn immersive_battle_pending_hp_target_waits_for_visible_animation_to_release_input() {
    let mut shell = immersive_battle_fixture();
    let snapshot = shell.shell.snapshot().unwrap();
    let player = &snapshot.party.slots[0].pokemon;
    shell.battle_hp_tween = Some(VisibleBattleHpTween {
        player_hp: player.hp,
        player_target_hp: player.hp.saturating_sub(4),
        player_max_hp: player.max_hp,
        player_pixels: 48,
        player_target_pixels: 40,
        player_frames_until_step: 0,
        enemy_pixels: 48,
        enemy_target_pixels: 48,
        enemy_frames_until_step: 0,
    });
    shell.visible_send_out_animation = Some(VisibleSendOutAnimation {
        side: crate::core::battle::turn::BattleSide::Player,
        frame: 12,
        shiny: false,
    });
    let frame = extract_immersive_battle_fixture(&shell, &snapshot, true);
    assert!(
        !frame
            .cues
            .iter()
            .any(|cue| cue.kind == VisualBattleCueKind::Impact),
        "a staged future HP target must not show impact while another animation owns the frame"
    );
    shell.visible_send_out_animation = None;
    let frame = extract_immersive_battle_fixture(&shell, &snapshot, true);
    assert!(
        frame
            .cues
            .iter()
            .any(|cue| cue.kind == VisualBattleCueKind::Impact)
    );
}
