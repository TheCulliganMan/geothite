fn visible_deterministic_session_checkpoint(
    shell: &RuntimeGameShell,
    checksum: StateChecksum,
) -> Result<SessionSaveCheckpointFrame> {
    let snapshot = shell.snapshot()?;
    let session_id = format!("bevy-local-start-{}", checksum.frame());
    let descriptor = shell.link_session_descriptor(
        session_id,
        LOCAL_PLAYER_ID,
        snapshot.trainer.player_name.clone(),
    )?;
    if descriptor.checksum.frame() != checksum.frame()
        || descriptor.checksum.hash() != checksum.hash()
    {
        anyhow::bail!(
            "deterministic session checkpoint frame/hash {} {:#010x} does not match session start {} {:#010x}",
            descriptor.checksum.frame(),
            descriptor.checksum.hash(),
            checksum.frame(),
            checksum.hash()
        );
    }
    Ok(descriptor.save_checkpoint)
}

fn required_visible_deterministic_session_checkpoint(
    runtime_shell: &BevyRuntimeShell,
) -> Result<&SessionSaveCheckpointFrame> {
    runtime_shell
        .deterministic_session_checkpoint
        .as_ref()
        .context("deterministic session checkpoint requires confirmed trainer identity")
}

fn setup_shell_view(mut commands: Commands) {
    // Begin with Camera2dBundle's specialized projection. Constructing an
    // OrthographicProjection from its generic Default loses Bevy's 2D depth
    // range and culls every positive-z LCD sprite.
    let mut camera = Camera2dBundle::default();
    // Keep one complete 640x576 logical LCD visible at every host aspect and
    // scale its tiles/text with the browser viewport. Landscape windows gain
    // real horizontal world space for the optional 2.5D renderer, while
    // portrait windows gain vertical space instead of cropping the LCD.
    camera.projection.scaling_mode = bevy::render::camera::ScalingMode::AutoMin {
        min_width: 640.0,
        min_height: 576.0,
    };
    commands.spawn((camera, MainCameraMarker));
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(640.0, 576.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 100.0),
            ..default()
        },
        ScreenFadeOverlay,
    ));
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgba(230.0 / 255.0, 173.0 / 255.0, 1.0, 0.0),
                custom_size: Some(Vec2::new(640.0, 576.0)),
                ..default()
            },
            // LoadPoisonBGPals replaces every CGB background color while
            // leaving OBJ palettes untouched. Keep this plane above the map
            // base but below every sorted overworld object; the priority BG
            // surface is hidden separately while the poison palette is live.
            transform: Transform::from_xyz(0.0, 0.0, 0.9),
            ..default()
        },
        PoisonFlashOverlay,
    ));
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgba(0.06, 0.08, 0.10, 0.0),
                custom_size: Some(Vec2::new(612.0, 116.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -222.0, 5.0),
            ..default()
        },
        DialogPanel,
    ));
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgba(0.07, 0.09, 0.12, 0.0),
                custom_size: Some(Vec2::new(302.0, 128.0)),
                ..default()
            },
            transform: Transform::from_xyz(165.0, 216.0, 4.0),
            ..default()
        },
        BattlePanel,
    ));
    commands.spawn((
        TextBundle::from_section(
            "Loading runtime...",
            TextStyle {
                font_size: 18.0,
                color: Color::srgb(0.88, 0.94, 0.86),
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            left: Val::Px(18.0),
            top: Val::Px(18.0),
            max_width: Val::Px(604.0),
            ..default()
        }),
        StatusText,
    ));
    commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 20.0,
                color: Color::srgb(0.97, 0.97, 0.90),
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            left: Val::Px(22.0),
            bottom: Val::Px(20.0),
            max_width: Val::Px(596.0),
            ..default()
        }),
        DialogText,
    ));
    commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 18.0,
                color: Color::srgb(0.94, 0.97, 0.99),
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            top: Val::Px(18.0),
            max_width: Val::Px(290.0),
            ..default()
        }),
        BattleText,
    ));
}

const SERVER_CLOCK_UNAVAILABLE: &str = "Server clock unavailable. Restore the connection to resume.";

fn apply_keyboard_input(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    rtc_source: Res<NativeRtcSource>,
    mut runtime_shell: ResMut<BevyRuntimeShell>,
    mut timer: ResMut<RuntimeTickTimer>,
) {
    let Some(rtc_sample) = (*rtc_source).try_sample() else {
        if runtime_shell.last_error.as_deref() != Some(SERVER_CLOCK_UNAVAILABLE) {
            runtime_shell.last_error = Some(SERVER_CLOCK_UNAVAILABLE.to_string());
            mark_runtime_presentation_dirty(&mut runtime_shell);
        }
        return;
    };
    if runtime_shell.last_error.as_deref() == Some(SERVER_CLOCK_UNAVAILABLE) {
        runtime_shell.last_error = None;
        mark_runtime_presentation_dirty(&mut runtime_shell);
    }
    if runtime_shell.visible_catch_tutorial.is_none()
        && runtime_shell.title_menu.is_none()
        && runtime_shell.intro_screen.is_none()
        && catch_tutorial_battle_active(&runtime_shell)
        && runtime_shell.pending_standard_capture.is_none()
        && runtime_shell.visible_capture_animation.is_none()
    {
        // Also recover an active tutorial restored from an older saved game.
        runtime_shell.visible_catch_tutorial = Some(VisibleCatchTutorial::default());
    }
    // CatchTutorial owns joypad input until its capture and result text end.
    let tutorial_keys = ButtonInput::<KeyCode>::default();
    let keys = if runtime_shell.visible_catch_tutorial.is_some() {
        runtime_shell.pending_ui_button_presses.clear();
        &tutorial_keys
    } else {
        &*keys
    };
    runtime_shell.field_text_consumed_a = false;
    runtime_shell.field_text_consumed_b = false;
    log_visible_key_presses(&mut runtime_shell, &keys);
    let title_input_active = runtime_shell.title_menu.is_some();
    if let Some(title) = runtime_shell.title_menu.as_mut()
        && matches!(
            title.source_phase(),
            VisibleTitlePhase::Entrance
                | VisibleTitlePhase::Timer
                | VisibleTitlePhase::PressStart
                | VisibleTitlePhase::FadeOut
        )
    {
        let pressed_mask = [
            (KeyCode::KeyZ, 0x01_u8),
            (KeyCode::KeyX, 0x02),
            (KeyCode::ShiftRight, 0x04),
            (KeyCode::Enter, 0x08),
            (KeyCode::ArrowRight, 0x10),
            (KeyCode::ArrowLeft, 0x20),
            (KeyCode::ArrowUp, 0x40),
            (KeyCode::ArrowDown, 0x80),
        ]
        .into_iter()
        .filter_map(|(key, mask)| keys.just_pressed(key).then_some(mask))
        .fold(0_u8, |mask, pressed| mask | pressed);
        // The host and title interpreter advance on separate clocks. Latch
        // each physical edge until one source frame samples hJoyDown.
        title.joypad_mask |= pressed_mask;
    }
    for key in [
        KeyCode::ArrowUp,
        KeyCode::ArrowDown,
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
        KeyCode::KeyZ,
        KeyCode::KeyX,
        KeyCode::Enter,
        KeyCode::ShiftRight,
    ] {
        if !title_input_active
            && keys.just_pressed(key)
            && !runtime_shell.pending_ui_button_presses.contains(&key)
        {
            runtime_shell.pending_ui_button_presses.push_back(key);
        }
    }
    timer.tick(time.delta_seconds_f64());
    let elapsed_vblanks = timer.take_vblanks();
    let elapsed_input_ticks = timer.take_ticks();
    timer.stage_presentation_ticks(elapsed_input_ticks);
    if elapsed_vblanks == 0 && elapsed_input_ticks == 0 {
        let shift_pressed = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
        let alt_pressed = keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight);
        let ctrl_pressed =
            keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
        if !shift_pressed
            && !alt_pressed
            && !ctrl_pressed
            && let Some(direction) = just_pressed_overworld_direction(&keys)
        {
            // Host rendering can run faster than Crystal's 60 Hz input loop.
            // Retain a press edge seen between authoritative ticks so a quick
            // tap still reaches the next joypad sample after the key is up.
            runtime_shell.pending_overworld_direction_press = Some(direction);
        }
        return;
    }
    runtime_shell.pokegear_exit_input_blocked = false;
    runtime_shell.pokegear_radio_input_blocked = false;
    runtime_shell.pokegear_joypad_prepared = false;
    runtime_shell.pokegear_joypad.advance_vblanks(elapsed_vblanks);
    advance_visible_pc_input_vblanks(&mut runtime_shell, elapsed_vblanks);
    runtime_shell.pc_joypad_vblank_prepared = true;
    if let Err(error) = advance_visible_music_fade(&mut runtime_shell, elapsed_vblanks) {
        record_visible_runtime_error(&mut runtime_shell, &error);
        runtime_shell.last_error = Some(error.to_string());
        return;
    }
    // The host can present frames faster than Crystal's input cadence. Keep
    // every physical button edge until the next authoritative input tick;
    // dropping these made dialogue and menus respond only intermittently.
    let mut keys = (*keys).clone();
    for key in runtime_shell.pending_ui_button_presses.drain(..) {
        if !keys.just_pressed(key) {
            keys.press(key);
        }
    }
    if (keys.just_pressed(KeyCode::KeyZ) || keys.just_pressed(KeyCode::KeyX))
        // Input ownership must use authoritative state. A retained render
        // scene may still contain the just-closed textbox while an actor
        // finishes moving; letting that stale frame claim A strands the
        // following script command indefinitely.
        && let Ok(snapshot) = runtime_shell.shell.snapshot()
        && snapshot.ui.text_window_open
        && snapshot.ui.text.is_some()
        && runtime_shell.field_text_reveal.is_some()
        && !visible_field_dialogue_is_fully_revealed(&runtime_shell, &snapshot)
    {
        runtime_shell.field_text_consumed_a = keys.just_pressed(KeyCode::KeyZ);
        runtime_shell.field_text_consumed_b = keys.just_pressed(KeyCode::KeyX);
    }
    let pending_direction_press = runtime_shell.pending_overworld_direction_press.take();
    // GameTimer is a VBlank hook, not an overworld-input side effect. Run it
    // for every authoritative catch-up VBlank before any presentation/modal
    // early return; dialogue, menus, battles, and interpolation all consume
    // real play time unless the source gates explicitly pause it.
    let previous_text_cursor_phase = visible_vblank_counter_bit4(&runtime_shell);
    if elapsed_vblanks > 0 {
        let normal_vblanks = if visible_special_vblank_handler_active(&runtime_shell) {
            0
        } else {
            elapsed_vblanks
        };
        if let Err(error) = runtime_shell
            .shell
            .advance_vblanks(elapsed_vblanks, normal_vblanks)
        {
            record_visible_runtime_error(&mut runtime_shell, &error);
            runtime_shell.last_error = Some(error.to_string());
            return;
        }
    }
    if runtime_shell.field_text_reveal.is_some()
        && previous_text_cursor_phase != visible_vblank_counter_bit4(&runtime_shell)
    {
        // PromptButton blinks on VBlank bit 4 even while the completed text
        // and authoritative dialogue state remain unchanged.
        mark_runtime_presentation_dirty(&mut runtime_shell);
    }
    let mut rtc_changed = runtime_shell.latest_rtc_sample != Some(rtc_sample);
    runtime_shell.latest_rtc_sample = Some(rtc_sample);
    if *rtc_source == NativeRtcSource::Server {
        let clock = &runtime_shell.shell.session().state().time;
        if rtc_changed || clock.current_date != rtc_sample.date
            || clock.registers.hours != rtc_sample.hour
            || clock.registers.minutes != rtc_sample.minute
            || clock.registers.seconds != rtc_sample.second
            || clock.start_time != ClockTime::default()
        {
            if let Err(error) = runtime_shell.shell.update_clock_from_datetime(
                rtc_sample.date, rtc_sample.hour, rtc_sample.minute, rtc_sample.second,
            ) {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
                return;
            }
            mark_runtime_presentation_dirty(&mut runtime_shell);
        }
        rtc_changed = false;
    }

    runtime_shell.lcd_animation_frame = runtime_shell
        .lcd_animation_frame
        .wrapping_add(u64::from(elapsed_input_ticks));
    // Modal profile editing blocks gameplay input, while the presentation clock
    // keeps input release, browser observation, and real-time clock updates live.
    if customization_is_open() || save_manager_is_open() { return; }
    match advance_visible_poison_flash(&mut runtime_shell, elapsed_input_ticks) {
        Ok(true) => return,
        Ok(false) => {}
        Err(error) => {
            record_visible_runtime_error(&mut runtime_shell, &error);
            runtime_shell.last_error = Some(error.to_string());
            return;
        }
    }
    if runtime_shell.bill_pc_move_save.is_some() {
        if let Err(error) =
            advance_visible_bill_pc_move_save(&mut runtime_shell, elapsed_input_ticks)
        {
            record_visible_runtime_error(&mut runtime_shell, &error);
            runtime_shell.last_error = Some(error.to_string());
        }
        return;
    }
    if runtime_shell.pc_release_sequence.is_some() {
        if let Err(error) =
            advance_visible_pc_release_sequence(&mut runtime_shell, elapsed_input_ticks)
        {
            record_visible_runtime_error(&mut runtime_shell, &error);
            runtime_shell.last_error = Some(error.to_string());
        }
        return;
    }
    if elapsed_input_ticks > 0 && runtime_shell.pokegear_exit == Some(VisiblePokegearExitPhase::Requested) {
        // PokeGear.loop samples once more before it checks the EXIT bit.
        sample_visible_pokegear_joypad(&keys, &mut runtime_shell);
    }
    match advance_visible_pokegear_exit(&mut runtime_shell, elapsed_input_ticks) {
        Ok(true) => return,
        Ok(false) => {},
        Err(error) => {
            record_visible_runtime_error(&mut runtime_shell, &error);
            runtime_shell.last_error = Some(error.to_string());
            return;
        }
    }
    if runtime_shell.pc_transfer_sequence.is_some() {
        if let Err(error) =
            advance_visible_pc_transfer_sequence(&mut runtime_shell, elapsed_input_ticks)
        {
            record_visible_runtime_error(&mut runtime_shell, &error);
            runtime_shell.last_error = Some(error.to_string());
        }
        return;
    }
    if runtime_shell.pc_item_move_sequence.is_some() {
        if let Err(error) = advance_visible_pc_item_move_sequence(&mut runtime_shell) {
            record_visible_runtime_error(&mut runtime_shell, &error);
            runtime_shell.last_error = Some(error.to_string());
        }
        return;
    }
    if elapsed_input_ticks > 0 {
        match advance_visible_incoming_phone_sequence(&mut runtime_shell, elapsed_input_ticks) {
            Ok(true) => return,
            Ok(false) => {}
            Err(error) => {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
                return;
            }
        }
        match advance_visible_pokegear_phone_call(&mut runtime_shell, elapsed_input_ticks) {
            Ok(true) => return,
            Ok(false) => {}
            Err(error) => {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
                return;
            }
        }
    }
    advance_visible_pokedex_search(&mut runtime_shell, elapsed_input_ticks);
    if runtime_shell.pokedex_controls.area_region.is_some() {
        let before = (runtime_shell.pokedex_controls.area_frames & 16, runtime_shell.pokedex_controls.area_show_player);
        runtime_shell.pokedex_controls.area_frames = runtime_shell.pokedex_controls.area_frames.wrapping_add(elapsed_input_ticks as u8);
        runtime_shell.pokedex_controls.area_show_player = keys.pressed(KeyCode::ShiftRight);
        let after = (runtime_shell.pokedex_controls.area_frames & 16, runtime_shell.pokedex_controls.area_show_player);
        if before != after { mark_runtime_presentation_dirty(&mut runtime_shell); }
    }

    advance_visible_pokegear_map_animation(&mut runtime_shell, elapsed_input_ticks);
    let radio_hold_active = runtime_shell.pokegear_map_radio_delay.is_some_and(|remaining| remaining != 0);
    let radio_ticks = advance_visible_map_radio_delay(&mut runtime_shell, elapsed_input_ticks);
    if radio_hold_active {
        runtime_shell.pokegear_radio_input_blocked = true;
        if radio_ticks == 0 { return; }
        keys.clear();
    }
    let text_acceleration_requested = keys.pressed(KeyCode::KeyZ) || keys.pressed(KeyCode::KeyX);
    let radio_call_suspended = runtime_shell.pokegear_radio_broadcast.as_ref()
        .is_some_and(|broadcast| broadcast.playback.call_suspended());
    if visible_pokegear_card_samples_joypad(&runtime_shell)
        && !radio_hold_active && !radio_call_suspended
    {
        sample_visible_pokegear_joypad(&keys, &mut runtime_shell);
        runtime_shell.pokegear_joypad_prepared = true;
    }
    let radio_exit_input = runtime_shell.pokegear_joypad_prepared
        && !radio_call_suspended && if runtime_shell.pokegear_map_radio_delay.is_some() {
        runtime_shell.pokegear_joypad.pressed
            & (crate::core::input::B_PAD_A | crate::core::input::B_PAD_B) != 0
    } else {
        runtime_shell.pokegear_joypad.last
            & (crate::core::input::B_PAD_B | crate::core::input::B_PAD_LEFT) != 0
    };
    if !radio_exit_input {
        match advance_visible_radio_broadcast(&mut runtime_shell, radio_ticks, text_acceleration_requested) {
            Ok(true) => return,
            Ok(false) => {},
            Err(error) => {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
                return;
            }
        }
    }
    runtime_shell.pokegear_radio_input_blocked |= radio_hold_active;
    let ambient_phase_changed = runtime_shell.ambient_tileset_animation_active
        && runtime_shell
            .ambient_tileset_animation_schedule
            .iter()
            .any(|(period, offset)| {
                runtime_shell.lcd_animation_frame >= *offset
                    && (runtime_shell.lcd_animation_frame - *offset) % (*period).max(1) == 0
            });
    if ambient_phase_changed {
        mark_runtime_presentation_dirty(&mut runtime_shell);
    }
    if (runtime_shell.battle_lcd_animation_active || runtime_shell.field_text_reveal.is_some())
        && runtime_shell.lcd_animation_frame % 8 == 0
    {
        mark_runtime_presentation_dirty(&mut runtime_shell);
    }
    if runtime_shell
        .shell
        .session()
        .overworld()
        .following
        .as_ref()
        .and_then(|following| following.follower_slot)
        .is_none()
        && (!runtime_shell.pending_follower_walks.is_empty()
            || !runtime_shell.follower_visible_tile_overrides.is_empty())
    {
        let follower_ids = runtime_shell
            .follower_visible_tile_overrides
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        runtime_shell.pending_follower_walks.clear();
        runtime_shell.follower_visible_tile_overrides.clear();
        for object_id in follower_ids {
            runtime_shell.object_walk_from.remove(&object_id);
            runtime_shell
                .object_walk_frame_ticks_by_id
                .remove(&object_id);
            runtime_shell
                .object_walk_total_ticks_by_id
                .remove(&object_id);
        }
    }
    // Presentation must cross each LCD boundary one tick at a time. A slow
    // renderer update can catch simulation time up in bulk, but consuming all
    // of those ticks here skips visible walk substeps and makes overworld
    // movement alternate between smooth motion and sudden jumps.
    let initial_movement_ticks = visible_walk_ticks_for_host_update(elapsed_input_ticks);
    advance_visible_walk_timers(&mut runtime_shell, initial_movement_ticks);
    if runtime_shell.battle_switch_cursor.is_some() {
        mark_runtime_snapshot_dirty(&mut runtime_shell);
    }
    if let Some(emote) = runtime_shell.visible_overworld_emote.as_mut() {
        emote.frames_remaining =
            visible_effect_frames_after_ticks(emote.frames_remaining, elapsed_input_ticks);
        mark_runtime_snapshot_dirty(&mut runtime_shell);
    }
    if let Some(earthquake) = runtime_shell.visible_earthquake.as_mut() {
        earthquake.advance(elapsed_input_ticks);
        mark_runtime_snapshot_dirty(&mut runtime_shell);
    }
    if runtime_shell
        .visible_overworld_emote
        .as_ref()
        .is_some_and(|emote| emote.frames_remaining == 0)
    {
        if let Err(error) = drain_visible_emotes(&mut runtime_shell) {
            record_visible_runtime_error(&mut runtime_shell, &error);
            runtime_shell.last_error = Some(format!("{error:#}"));
        }
        return;
    }
    if runtime_shell
        .visible_earthquake
        .as_ref()
        .is_some_and(|earthquake| earthquake.frames_remaining == 0)
    {
        if let Err(error) = drain_visible_earthquakes(&mut runtime_shell) {
            record_visible_runtime_error(&mut runtime_shell, &error);
            runtime_shell.last_error = Some(format!("{error:#}"));
        }
        return;
    }
    let mut field_object_effect_advanced = false;
    let field_notice_effect_finished = if runtime_shell.field_notice.is_none() {
        if let Some(frames) = runtime_shell.pending_field_notice_effect_frames {
            let frames = frames.saturating_sub(elapsed_input_ticks.min(u32::from(u8::MAX)) as u8);
            runtime_shell.pending_field_notice_effect_frames = Some(frames);
            if let Some(cut) = runtime_shell.visible_cut_animation.as_mut() {
                cut.frame = 34_u8.saturating_sub(frames);
                field_object_effect_advanced = true;
            }
            if let Some(headbutt) = runtime_shell.visible_headbutt_animation.as_mut() {
                headbutt.frame = 32_u8.saturating_sub(frames);
                field_object_effect_advanced = true;
            }
            if let Some(flash) = runtime_shell.visible_flash_animation.as_mut() {
                let previous_frame = flash.frame;
                flash.frame = 16_u8.saturating_sub(frames);
                field_object_effect_advanced = true;
                if previous_frame < 8 && flash.frame >= 8 {
                    runtime_shell.field_notice_scene = None;
                }
            }
            frames == 0
        } else {
            false
        }
    } else {
        false
    };
    if field_object_effect_advanced {
        mark_runtime_snapshot_dirty(&mut runtime_shell);
    }
    if field_notice_effect_finished {
        runtime_shell.pending_field_notice_effect_frames = None;
        runtime_shell.field_notice_scene = None;
        runtime_shell.visible_earthquake = None;
        runtime_shell.visible_cut_animation = None;
        runtime_shell.pending_whirlpool_sound_wait = false;
        runtime_shell.visible_headbutt_animation = None;
        runtime_shell.visible_flash_animation = None;
        runtime_shell.pending_surf_start_from = None;
        if std::mem::take(&mut runtime_shell.pending_field_battle_entry) {
            if let Err(error) = prepare_visible_battle_entry(&mut runtime_shell)
                .and_then(|_| settle_visible_battle_after_action(&mut runtime_shell))
            {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
            }
        } else if let Some(next) = runtime_shell.field_notice_queue.pop_front() {
            runtime_shell.field_notice = Some(next);
        }
        mark_runtime_snapshot_dirty(&mut runtime_shell);
    }
    if runtime_shell.field_notice.is_none() && runtime_shell.visible_waterfall_animation.is_some() {
        for _ in 0..elapsed_input_ticks {
            let Some(animation) = runtime_shell.visible_waterfall_animation else {
                break;
            };
            let Some(total_frames) = animation.steps.checked_mul(4) else {
                let error = anyhow::anyhow!(
                    "WATERFALL visual duration overflows for {} steps",
                    animation.steps
                );
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
                return;
            };
            if animation.frame >= total_frames {
                runtime_shell.visible_waterfall_animation = None;
                runtime_shell.field_notice_scene = None;
                runtime_shell.player_walk_from = None;
                runtime_shell.player_walk_frame_ticks = 0;
                runtime_shell.player_walk_total_ticks = WALK_FRAME_HOLD_TICKS;
                runtime_shell.player_walk_stride = false;
                runtime_shell.player_walk_mirror_stride = false;
            } else {
                let step_index = animation.frame / 4;
                let phase = (animation.frame % 4) as u8;
                if phase == 0
                    && let Err(error) = execute_visible_pending_waterfall_step(
                        &mut runtime_shell,
                        step_index,
                        animation.steps,
                    )
                {
                    record_visible_runtime_error(&mut runtime_shell, &error);
                    runtime_shell.last_error = Some(error.to_string());
                    return;
                }
                let Ok(step_index_i16) = i16::try_from(step_index) else {
                    let error = anyhow::anyhow!(
                        "WATERFALL visual step {step_index} exceeds runtime tile coordinates"
                    );
                    record_visible_runtime_error(&mut runtime_shell, &error);
                    runtime_shell.last_error = Some(error.to_string());
                    return;
                };
                let step_offset = step_index_i16;
                let Some(segment_from_y) = animation.from_tile.y.checked_sub(step_offset) else {
                    let error =
                        anyhow::anyhow!("WATERFALL visual origin underflows at step {step_index}");
                    record_visible_runtime_error(&mut runtime_shell, &error);
                    runtime_shell.last_error = Some(error.to_string());
                    return;
                };
                let Some(segment_to_y) = segment_from_y.checked_sub(1) else {
                    let error = anyhow::anyhow!(
                        "WATERFALL visual destination underflows at step {step_index}"
                    );
                    record_visible_runtime_error(&mut runtime_shell, &error);
                    runtime_shell.last_error = Some(error.to_string());
                    return;
                };
                let segment_from = TilePosition {
                    x: animation.from_tile.x,
                    y: segment_from_y,
                };
                let segment_to = TilePosition {
                    x: segment_from.x,
                    y: segment_to_y,
                };
                if step_index + 1 == animation.steps && segment_to != animation.to_tile {
                    let error = anyhow::anyhow!(
                        "WATERFALL visual path ended at ({}, {}) instead of authoritative ({}, {})",
                        segment_to.x,
                        segment_to.y,
                        animation.to_tile.x,
                        animation.to_tile.y,
                    );
                    record_visible_runtime_error(&mut runtime_shell, &error);
                    runtime_shell.last_error = Some(error.to_string());
                    return;
                }
                if let Some(scene) = runtime_shell.field_notice_scene.as_mut() {
                    Arc::make_mut(scene).overworld.tile = segment_to;
                }
                runtime_shell.player_walk_from = Some(segment_from);
                runtime_shell.player_walk_total_ticks = WALK_FRAME_HOLD_TICKS;
                runtime_shell.player_walk_frame_ticks =
                    WALK_FRAME_HOLD_TICKS.saturating_sub(phase.saturating_mul(2));
                runtime_shell.player_walk_stride = step_index & 1 == 0;
                runtime_shell.player_walk_mirror_stride = step_index % 4 >= 2;
                if let Some(animation) = runtime_shell.visible_waterfall_animation.as_mut() {
                    animation.frame = animation.frame.saturating_add(1);
                }
            }
            mark_runtime_snapshot_dirty(&mut runtime_shell);
        }
        return;
    }
    if runtime_shell.visible_fly_animation.is_some() {
        for _ in 0..elapsed_input_ticks {
            let Some(animation) = runtime_shell.visible_fly_animation else {
                break;
            };
            let counter = match animation.phase {
                VisibleFlyAnimationPhase::From => 128_u8.saturating_sub(animation.frame),
                VisibleFlyAnimationPhase::To => 64_u8.saturating_sub(animation.frame),
            };
            if animation.frame > 0 && counter >= 0x40 && counter & 7 == 0 {
                let BevyRuntimeShell {
                    shell,
                    pending_audio,
                    last_audio_events,
                    ..
                } = &mut *runtime_shell;
                if let Err(error) = queue_visible_sound_effect(
                    shell.runtime().audio(),
                    pending_audio,
                    last_audio_events,
                    "SFX_FLY",
                ) {
                    record_visible_runtime_error(&mut runtime_shell, &error);
                    runtime_shell.last_error = Some(error.to_string());
                    return;
                }
            }
            match animation.phase {
                VisibleFlyAnimationPhase::From => {
                    if animation.frame >= 128 {
                        runtime_shell.field_notice_scene = None;
                        if let Err(error) = settle_visible_overworld_travel(&mut runtime_shell) {
                            record_visible_runtime_error(&mut runtime_shell, &error);
                            runtime_shell.last_error = Some(error.to_string());
                            return;
                        }
                        runtime_shell.visible_fly_animation = Some(VisibleFlyAnimation {
                            phase: VisibleFlyAnimationPhase::To,
                            frame: 0,
                            actor_party_index: animation.actor_party_index,
                        });
                        let BevyRuntimeShell {
                            shell,
                            pending_audio,
                            last_audio_events,
                            ..
                        } = &mut *runtime_shell;
                        if let Err(error) = queue_visible_sound_effect(
                            shell.runtime().audio(),
                            pending_audio,
                            last_audio_events,
                            "SFX_FLY",
                        ) {
                            record_visible_runtime_error(&mut runtime_shell, &error);
                            runtime_shell.last_error = Some(error.to_string());
                            return;
                        }
                    } else if let Some(animation) = runtime_shell.visible_fly_animation.as_mut() {
                        animation.frame = animation.frame.saturating_add(1);
                    }
                }
                VisibleFlyAnimationPhase::To => {
                    if animation.frame >= 64 {
                        runtime_shell.visible_fly_animation = None;
                    } else if let Some(animation) = runtime_shell.visible_fly_animation.as_mut() {
                        animation.frame = animation.frame.saturating_add(1);
                    }
                }
            }
            mark_runtime_snapshot_dirty(&mut runtime_shell);
        }
        return;
    }
    if runtime_shell
        .visible_fishing_animation
        .is_some_and(|animation| animation.phase != VisibleFishingPhase::AwaitText)
    {
        for _ in 0..elapsed_input_ticks {
            if !runtime_shell
                .visible_fishing_animation
                .is_some_and(|animation| animation.phase != VisibleFishingPhase::AwaitText)
            {
                break;
            }
            advance_visible_fishing_animation(&mut runtime_shell);
        }
        return;
    }
    if runtime_shell
        .visible_egg_hatch
        .as_ref()
        .is_some_and(|hatch| {
            matches!(
                hatch.phase,
                VisibleEggHatchPhase::EggHold
                    | VisibleEggHatchPhase::Wobble
                    | VisibleEggHatchPhase::Shell
            ) || (hatch.phase == VisibleEggHatchPhase::Reveal
                && runtime_shell.visible_frontpic_animation.is_none())
        })
    {
        for _ in 0..elapsed_input_ticks {
            let owns_frame = runtime_shell
                .visible_egg_hatch
                .as_ref()
                .is_some_and(|hatch| {
                    matches!(
                        hatch.phase,
                        VisibleEggHatchPhase::EggHold
                            | VisibleEggHatchPhase::Wobble
                            | VisibleEggHatchPhase::Shell
                    ) || (hatch.phase == VisibleEggHatchPhase::Reveal
                        && runtime_shell.visible_frontpic_animation.is_none())
                });
            if !owns_frame {
                break;
            }
            if let Err(error) = advance_visible_egg_hatch(&mut runtime_shell) {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
                break;
            }
        }
        return;
    }
    if runtime_shell
        .visible_field_item_notice
        .as_ref()
        .is_some_and(|notice| matches!(notice.phase, VisibleFieldItemPhase::FanfarePause { .. }))
    {
        for _ in 0..elapsed_input_ticks {
            if !advance_visible_field_item_fanfare_pause(&mut runtime_shell) {
                break;
            }
        }
        return;
    }
    if let Some(frame) = runtime_shell.visible_diploma.as_mut() {
        *frame = frame.wrapping_add(elapsed_input_ticks as u8);
        mark_runtime_snapshot_dirty(&mut runtime_shell);
        return;
    }
    if runtime_shell
        .visible_slot_machine
        .as_ref()
        .is_some_and(|machine| {
            !matches!(
                machine.animation,
                VisibleSlotMachineAnimation::None | VisibleSlotMachineAnimation::AwaitResult
            )
        })
    {
        for _ in 0..elapsed_input_ticks {
            let owns_frame = runtime_shell
                .visible_slot_machine
                .as_ref()
                .is_some_and(|machine| {
                    !matches!(
                        machine.animation,
                        VisibleSlotMachineAnimation::None
                            | VisibleSlotMachineAnimation::AwaitResult
                    )
                });
            if !owns_frame {
                break;
            }
            if let Err(error) = advance_visible_slot_machine_animation(&mut runtime_shell) {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
                break;
            }
        }
        return;
    }
    if runtime_shell
        .visible_card_flip
        .as_ref()
        .is_some_and(|game| {
            matches!(
                game.animation,
                VisibleCardFlipAnimation::WaitStake
                    | VisibleCardFlipAnimation::Deal { .. }
                    | VisibleCardFlipAnimation::Cycle { .. }
                    | VisibleCardFlipAnimation::SelectFlash { .. }
                    | VisibleCardFlipAnimation::WaitBeforeReveal
                    | VisibleCardFlipAnimation::WaitReveal
                    | VisibleCardFlipAnimation::WaitResult { .. }
                    | VisibleCardFlipAnimation::Payout { .. }
                    | VisibleCardFlipAnimation::QuitWaitBefore
                    | VisibleCardFlipAnimation::QuitWaitAfter
            )
        })
    {
        let timed = runtime_shell
            .visible_card_flip
            .as_ref()
            .is_some_and(|game| {
                matches!(
                    game.animation,
                    VisibleCardFlipAnimation::Deal { .. }
                        | VisibleCardFlipAnimation::Cycle { .. }
                        | VisibleCardFlipAnimation::SelectFlash { .. }
                        | VisibleCardFlipAnimation::Payout { .. }
                )
            });
        let frame_budget = if timed { elapsed_input_ticks } else { 1 };
        for _ in 0..frame_budget {
            if let Err(error) = advance_visible_card_flip_animation(&mut runtime_shell) {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
                break;
            }
            if timed
                && !runtime_shell
                    .visible_card_flip
                    .as_ref()
                    .is_some_and(|game| {
                        matches!(
                            game.animation,
                            VisibleCardFlipAnimation::Deal { .. }
                                | VisibleCardFlipAnimation::Cycle { .. }
                                | VisibleCardFlipAnimation::SelectFlash { .. }
                                | VisibleCardFlipAnimation::Payout { .. }
                        )
                    })
            {
                break;
            }
        }
        return;
    }
    if runtime_shell.visible_heal_machine.is_some() {
        for _ in 0..elapsed_input_ticks {
            if let Err(error) = advance_visible_heal_machine(&mut runtime_shell) {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
                break;
            }
            if runtime_shell
                .visible_heal_machine
                .as_ref()
                .is_none_or(visible_heal_machine_is_terminal)
            {
                break;
            }
        }
        return;
    }
    if runtime_shell.visible_magnet_train.is_some() {
        for _ in 0..elapsed_input_ticks {
            if let Err(error) = advance_visible_magnet_train(&mut runtime_shell) {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
                break;
            }
            if runtime_shell
                .visible_magnet_train
                .as_ref()
                .is_none_or(|animation| animation.phase >= 7 && animation.arrival_sfx_played)
            {
                break;
            }
        }
        return;
    }
    if runtime_shell.visible_battle_sliding_intro.is_some() {
        for _ in 0..elapsed_input_ticks {
            advance_visible_battle_sliding_intro(&mut runtime_shell);
            if runtime_shell
                .visible_battle_sliding_intro
                .is_none_or(|frame| frame + 1 >= BATTLE_SLIDING_INTRO_FRAMES)
            {
                break;
            }
        }
        return;
    }
    if runtime_shell.visible_battle_transition.is_some() {
        let waiting_for_step = matches!(
            runtime_shell.pending_overworld_step_boundary,
            Some(PendingOverworldStepBoundary::WildBattle)
        );
        if waiting_for_step {
            if runtime_shell.player_walk_frame_ticks > 0
                || runtime_shell.visible_ledge_jump.is_some()
            {
                return;
            }
        } else {
            // Transition drawing can be one of the most expensive retained
            // LCD surfaces (the cave sine wave composes every scanline). A
            // slow host update may therefore contain several elapsed Game
            // Boy frames. Consume the complete bounded tick budget so that
            // rendering cost cannot stretch battle entry or make its motion
            // alternate between stalls and single-frame steps.
            for _ in 0..elapsed_input_ticks {
                advance_visible_battle_transition(&mut runtime_shell);
                if runtime_shell
                    .visible_battle_transition
                    .as_ref()
                    .is_none_or(visible_battle_transition_is_terminal)
                {
                    break;
                }
            }
            return;
        }
    }
    if visible_battle_animation_owns_frame(&runtime_shell) {
        for _ in 0..elapsed_input_ticks {
            if !visible_battle_animation_owns_frame(&runtime_shell) {
                break;
            }
            if let Err(error) = advance_visible_battle_animation_frame(&mut runtime_shell) {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
                runtime_shell.visible_frontpic_animation = None;
                break;
            }
        }
        // Battle presentation owns the complete host update even when its
        // final animation frame lands during catch-up. Do not leak a buffered
        // command into the newly revealed battle menu.
        return;
    }
    if runtime_shell
        .visible_blackout_phase
        .is_some_and(|phase| phase != VisibleBlackoutPhase::AwaitText)
    {
        return;
    }
    if runtime_shell.visible_walk_warp_phase.is_some() {
        return;
    }
    for _ in 0..elapsed_input_ticks {
        if let Some(jump) = runtime_shell.visible_ledge_jump.as_mut() {
            if jump.frame < 15 {
                jump.frame += 1;
            } else {
                runtime_shell.visible_ledge_jump = None;
                let overworld = &mut runtime_shell.shell.session_mut().overworld_mut();
                overworld.player_last_runtime_tile = None;
                overworld.player_last_tile_occupied_until_frame = 0;
                mark_runtime_snapshot_dirty(&mut runtime_shell);
            }
        }
        // A ledge jump is two chained eight-frame steps. Core reports the
        // final landing tile atomically, but its grass effect belongs to the
        // second step and must not age during the takeoff half.
        let landing_grass_not_started = runtime_shell
            .visible_ledge_jump
            .is_some_and(|jump| jump.frame <= WALK_FRAME_HOLD_TICKS);
        if let Some(rustle) = runtime_shell.visible_grass_rustle.as_mut() {
            if !landing_grass_not_started {
                rustle.age = rustle.age.saturating_add(1);
                rustle.frames_remaining = rustle.frames_remaining.saturating_sub(1);
                if rustle.frames_remaining == 0 {
                    runtime_shell.visible_grass_rustle = None;
                }
            }
            mark_runtime_snapshot_dirty(&mut runtime_shell);
        }
        if let Some(dust) = runtime_shell.visible_strength_boulder_dust.as_mut() {
            dust.age = dust.age.saturating_add(1);
            dust.frames_remaining = dust.frames_remaining.saturating_sub(1);
            if dust.frames_remaining == 0 {
                runtime_shell.visible_strength_boulder_dust = None;
            }
            mark_runtime_snapshot_dirty(&mut runtime_shell);
        }
    }
    if runtime_shell.visible_script_movement.is_some() {
        let mut movement_still_active = false;
        for catch_up_index in 0..elapsed_input_ticks.max(1) {
            if catch_up_index > 0 {
                advance_visible_walk_timers(&mut runtime_shell, 1);
            }
            match advance_visible_script_movement(&mut runtime_shell) {
                Ok(active) => {
                    movement_still_active = active;
                    if !active {
                        break;
                    }
                }
                Err(error) => {
                    record_visible_runtime_error(&mut runtime_shell, &error);
                    runtime_shell.last_error = Some(error.to_string());
                    return;
                }
            }
        }
        if movement_still_active {
            return;
        }
    }
    // FaintEnemyPokemon/FaintYourPokemon animate before printing faint text.
    // HPBarAnim must finish first; the preceding speech remains on the LCD.
    let start_faint = runtime_shell.visible_move_animations.front().is_some_and(|animation| {
        animation.move_id == "FAINT_MON" && !animation.started
            && runtime_shell.battle_messages.front() == Some(&animation.trigger_message)
    }) && !runtime_shell.battle_hp_tween.as_ref().is_some_and(visible_battle_hp_tween_active);
    if start_faint {
        runtime_shell.visible_move_animations.front_mut().unwrap().started = true;
        mark_runtime_presentation_dirty(&mut runtime_shell);
    }
    if !runtime_shell.battle_messages.is_empty()
        && !runtime_shell
            .visible_move_animations
            .front()
            .is_some_and(|animation| animation.started)
    {
        match cached_runtime_snapshot(&mut runtime_shell) {
            Ok(snapshot) => {
                let changed = advance_visible_battle_text_frames(
                    &mut runtime_shell,
                    &snapshot,
                    text_acceleration_requested,
                    elapsed_input_ticks,
                );
                if changed {
                    mark_runtime_presentation_dirty(&mut runtime_shell);
                }
            }
            Err(error) => {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
                return;
            }
        }
    } else if runtime_shell.battle_text_reveal.take().is_some() {
        mark_runtime_presentation_dirty(&mut runtime_shell);
    }
    let mut hp_pixels_changed = false;
    for _ in 0..elapsed_input_ticks {
        let Some(tween) = runtime_shell.battle_hp_tween.as_mut() else {
            break;
        };
        if !visible_battle_hp_tween_active(tween) {
            break;
        }
        let player_changed = advance_visible_hp_pixels(
            &mut tween.player_pixels,
            tween.player_target_pixels,
            &mut tween.player_frames_until_step,
        );
        if player_changed {
            advance_visible_player_hp_number(tween);
        }
        let enemy_changed = advance_visible_hp_pixels(
            &mut tween.enemy_pixels,
            tween.enemy_target_pixels,
            &mut tween.enemy_frames_until_step,
        );
        hp_pixels_changed |= player_changed || enemy_changed;
    }
    if hp_pixels_changed {
        mark_runtime_presentation_dirty(&mut runtime_shell);
    }
    let hp_tween_active = runtime_shell
        .battle_hp_tween
        .as_ref()
        .is_some_and(visible_battle_hp_tween_active);
    if !hp_tween_active
        && runtime_shell
            .visible_move_animations
            .front()
            .is_some_and(|animation| animation.waiting_for_hp)
    {
        let animation = runtime_shell.visible_move_animations.front_mut().unwrap();
        animation.waiting_for_hp = false;
        animation.started = true;
        mark_runtime_presentation_dirty(&mut runtime_shell);
        return;
    }
    if hp_tween_active {
        // UpdateBattleHuds waits for the bar animation before battle command
        // processing resumes. Keeping this frame presentation-only also
        // prevents a hidden cursor from moving beneath the retained HUD.
        return;
    }
    for _ in 0..elapsed_input_ticks {
        let mut exp_segment_finished = false;
        let mut exp_animation_finished = false;
        let mut exp_pixels_changed = false;
        if let Some(tween) = runtime_shell.battle_exp_tween.as_mut()
            && tween.started
        {
            if tween.frames_until_step > 0 {
                tween.frames_until_step -= 1;
            } else if tween.pixels < tween.target_pixels {
                tween.pixels += 1;
                tween.steps_in_segment += 1;
                tween.frames_until_step = if tween.steps_in_segment <= 2 {
                    2
                } else if tween.steps_in_segment <= 4 {
                    1
                } else {
                    0
                };
                exp_pixels_changed = true;
                if tween.pixels == tween.target_pixels {
                    exp_segment_finished = !tween.remaining_targets.is_empty();
                    exp_animation_finished = tween.remaining_targets.is_empty();
                    if exp_segment_finished {
                        tween.level = tween.level.saturating_add(1).min(100);
                    }
                    tween.started = false;
                }
            } else {
                exp_segment_finished = !tween.remaining_targets.is_empty();
                exp_animation_finished = tween.remaining_targets.is_empty();
                if exp_segment_finished {
                    tween.level = tween.level.saturating_add(1).min(100);
                }
                tween.started = false;
            }
        }
        if exp_pixels_changed {
            mark_runtime_snapshot_dirty(&mut runtime_shell);
        }
        if exp_segment_finished {
            if let Err(error) =
                queue_visible_shell_sound_effect(&mut runtime_shell, "SFX_HIT_END_OF_EXP_BAR")
            {
                record_visible_runtime_error(&mut runtime_shell, &error);
            }
            if runtime_shell
                .battle_fanfare_messages
                .front()
                .is_some_and(|fanfare| runtime_shell.battle_messages.front() == Some(fanfare))
            {
                runtime_shell.battle_fanfare_messages.pop_front();
                if let Err(error) =
                    queue_visible_shell_sound_effect(&mut runtime_shell, "SFX_DEX_FANFARE_50_79")
                {
                    record_visible_runtime_error(&mut runtime_shell, &error);
                }
            }
        }
        if exp_animation_finished {
            runtime_shell.battle_exp_tween = runtime_shell.pending_battle_exp_tweens.pop_front();
            if let Some(stats) = runtime_shell.battle_level_stats.front_mut()
                && stats.triggered
            {
                stats.active = true;
                // This activation occurs before the per-frame countdown below.
                stats.frames_before_input = 31;
                mark_runtime_snapshot_dirty(&mut runtime_shell);
            }
            if let Err(error) = finish_visible_empty_battle_reward_presentation(&mut runtime_shell)
            {
                record_visible_runtime_error(&mut runtime_shell, &error);
            }
            if runtime_shell
                .battle_fanfare_messages
                .front()
                .is_some_and(|fanfare| runtime_shell.battle_messages.front() == Some(fanfare))
            {
                runtime_shell.battle_fanfare_messages.pop_front();
                if let Err(error) =
                    queue_visible_shell_sound_effect(&mut runtime_shell, "SFX_DEX_FANFARE_50_79")
                {
                    record_visible_runtime_error(&mut runtime_shell, &error);
                }
            }
        }
        if let Some(stats) = runtime_shell.battle_level_stats.front_mut()
            && stats.active
            && stats.frames_before_input > 0
        {
            stats.frames_before_input -= 1;
        }
    }
    if runtime_shell.visible_catch_tutorial.is_some() {
        if let Err(error) = advance_visible_catch_tutorial(&mut runtime_shell, elapsed_input_ticks) {
            record_visible_runtime_error(&mut runtime_shell, &error);
            runtime_shell.last_error = Some(format!("{error:#}"));
        }
        return;
    }
    if let Some(frames) = runtime_shell.visible_script_delay_frames.as_mut() {
        *frames = frames.saturating_sub(elapsed_input_ticks.min(u32::from(u16::MAX)) as u16);
    }
    if runtime_shell.visible_script_delay_frames == Some(0) {
        if let Err(error) = drain_visible_delays(&mut runtime_shell) {
            record_visible_runtime_error(&mut runtime_shell, &error);
            runtime_shell.last_error = Some(format!("{error:#}"));
        }
        return;
    }
    if runtime_shell.pending_linked_friend_wait {
        return;
    }
    if let Some(frames) = runtime_shell.visible_internal_special_delay_frames.as_mut() {
        *frames = frames.saturating_sub(elapsed_input_ticks.min(u32::from(u8::MAX)) as u8);
        if *frames == 0 {
            runtime_shell.visible_internal_special_delay_frames = None;
            if let Err(error) = continue_visible_script_after_prompt(&mut runtime_shell) {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(format!("{error:#}"));
            }
        } else {
            mark_runtime_snapshot_dirty(&mut runtime_shell);
        }
        return;
    }
    if runtime_shell.visible_special_text_pause_frames.is_some() {
        for _ in 0..elapsed_input_ticks {
            if runtime_shell.visible_special_text_pause_frames.is_none() {
                break;
            }
            if let Err(error) = advance_visible_special_text_pause(&mut runtime_shell) {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(format!("{error:#}"));
                break;
            }
        }
        return;
    }
    if runtime_shell
        .pending_remember_password
        .as_ref()
        .is_some_and(|prompt| prompt.closing_frames.is_some())
    {
        for _ in 0..elapsed_input_ticks {
            if !runtime_shell
                .pending_remember_password
                .as_ref()
                .is_some_and(|prompt| prompt.closing_frames.is_some())
            {
                break;
            }
            if let Err(error) = advance_visible_remember_password_prompt(&mut runtime_shell) {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(format!("{error:#}"));
                break;
            }
        }
        return;
    }
    let field_travel_text_complete = runtime_shell
        .field_notice
        .as_deref()
        .map_or(true, |notice| {
            visible_field_text_reveal_is_complete_for_text(&runtime_shell, notice)
        });
    let field_travel_delay_finished = if field_travel_text_complete
        && let Some(frames) = runtime_shell.pending_field_travel_delay_frames.as_mut()
    {
        *frames = frames.saturating_sub(elapsed_input_ticks.min(u32::from(u16::MAX)) as u16);
        *frames == 0
    } else {
        false
    };
    if field_travel_delay_finished {
        runtime_shell.pending_field_travel_delay_frames = None;
        runtime_shell.field_notice = None;
        runtime_shell.field_notice_queue.clear();
        runtime_shell.pending_field_travel_arrival = false;
        if runtime_shell.visible_field_travel_animation
            == Some(VisibleFieldTravelAnimation::TeleportFrom)
        {
            if let Err(error) = queue_visible_shell_sound_effect(&mut runtime_shell, "SFX_WARP_TO")
                .and_then(|_| begin_visible_teleport_travel_animation(&mut runtime_shell, false))
            {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
            }
            mark_runtime_snapshot_dirty(&mut runtime_shell);
            return;
        }
        runtime_shell.field_notice_scene = None;
        if let Err(error) = settle_visible_overworld_travel(&mut runtime_shell) {
            record_visible_runtime_error(&mut runtime_shell, &error);
            runtime_shell.last_error = Some(error.to_string());
        }
        mark_runtime_snapshot_dirty(&mut runtime_shell);
        return;
    }
    if advance_visible_map_name_sign(
        &mut runtime_shell.visible_map_name_sign,
        elapsed_input_ticks,
    ) {
        mark_runtime_snapshot_dirty(&mut runtime_shell);
    }
    if runtime_shell.pending_trainer_sight.is_some() {
        for _ in 0..elapsed_input_ticks {
            if runtime_shell.pending_trainer_sight.is_none() {
                break;
            }
            if let Err(error) = advance_visible_trainer_sight_cutscene(&mut runtime_shell) {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
                break;
            }
        }
        return;
    }
    if runtime_shell.pending_field_notice_effect_frames.is_some()
        && runtime_shell.field_notice.is_none()
    {
        return;
    }
    if runtime_shell.visible_fly_animation.is_some()
        || (runtime_shell.visible_waterfall_animation.is_some()
            && runtime_shell.field_notice.is_none())
    {
        return;
    }
    if runtime_shell.pending_name_input.is_some()
        || runtime_shell.pending_mail_input.is_some()
        || runtime_shell.pending_mail_read.is_some()
        || runtime_shell.pending_name_choice.is_some()
    {
        return;
    }
    if runtime_shell.intro_screen.is_some() {
        return;
    }
    if runtime_shell.pending_time_set.is_some() {
        return;
    }
    if runtime_shell.pending_oak_intro.is_some() {
        return;
    }
    if runtime_shell.pending_gender_selection.is_some() {
        return;
    }
    if runtime_shell.credits_screen.is_some() {
        for _ in 0..elapsed_input_ticks {
            if runtime_shell.credits_screen.is_none() {
                break;
            }
            tick_visible_credits_screen(&mut runtime_shell);
        }
        return;
    }
    if runtime_shell.pending_delete_save.is_some() {
        return;
    }
    if runtime_shell.pending_clock_reset.is_some() {
        return;
    }
    if runtime_shell.pending_mystery_gift.is_some() {
        return;
    }
    // The title owns the Options overlay, but the overlay still consumes
    // directional input. Only suppress overworld input while the bare title
    // menu is active.
    if runtime_shell.title_menu.is_some() && !runtime_shell.options_menu_open {
        return;
    }

    // Avoid an additional full snapshot merely to discover that there is no
    // text. The core shell has a cheap pending-work predicate which is true
    // for an open textbox and false for normal standing-still frames. The
    // authoritative input transaction still owns its atomic staging clone.
    if runtime_shell.shell.has_pending_script_work()
        || runtime_shell.field_text_reveal.is_some()
        || runtime_shell.field_notice.is_some()
        || runtime_shell.pc_notice.is_some()
        || runtime_shell.visible_wait_sfx_boundary
    {
        let mut text_changed = false;
        for _ in 0..elapsed_input_ticks {
            match tick_visible_field_text_reveal(&mut runtime_shell, text_acceleration_requested) {
                Ok(true) => text_changed = true,
                // A character delay still consumes an LCD frame. Dropping
                // the remaining ticks makes MID/SLOW text host-FPS dependent.
                Ok(false) => {}
                Err(error) => {
                    record_visible_runtime_error(&mut runtime_shell, &error);
                    runtime_shell.last_error = Some(error.to_string());
                    return;
                }
            }
        }
        if text_changed {
            mark_runtime_presentation_dirty(&mut runtime_shell);
        }
        let completed_text_identity =
            runtime_shell
                .shell
                .presentation_snapshot()
                .ok()
                .and_then(|snapshot| {
                    visible_field_dialog_pages(&snapshot, &runtime_shell)?;
                    let reveal = runtime_shell.field_text_reveal.as_ref()?;
                    visible_field_dialogue_is_entirely_consumed(&runtime_shell, &snapshot)
                        .then(|| (reveal.text.clone(), reveal.page_index))
                });
        let completed_text_was_presented =
            completed_text_identity.as_ref().is_none_or(|identity| {
                runtime_shell.rendered_field_text_identity.as_ref() == Some(identity)
            });
        // `waitsfx` is an autonomous ASM sequencing fence. Poll it from the
        // frame loop so the script resumes as soon as the transient channel
        // finishes; requiring an unrelated A/B press leaves item rewards and
        // other fanfares parked forever on their preceding textbox.
        if runtime_shell.visible_wait_sfx_boundary && completed_text_was_presented {
            match runtime_shell.shell.presentation_snapshot() {
                Ok(snapshot) => {
                    match advance_visible_wait_sfx_boundary(&mut runtime_shell, &snapshot, true) {
                        Ok(true) => return,
                        Ok(false) => {}
                        Err(error) => {
                            record_visible_runtime_error(&mut runtime_shell, &error);
                            runtime_shell.last_error = Some(error.to_string());
                            return;
                        }
                    }
                }
                Err(error) => {
                    record_visible_runtime_error(&mut runtime_shell, &error);
                    runtime_shell.last_error = Some(error.to_string());
                    return;
                }
            }
        }
        // Crystal's PrintText is synchronous: no command after `writetext`
        // may begin until the visual printer has finished. The TextLabel
        // boundary therefore resumes automatically only after every page;
        // a following waitbutton/promptbutton remains the player boundary.
        // JumpTextScript includes waitbutton after repeattext. Its pending
        // label and closing wait share one runtime boundary; consuming both
        // here closed signs and NPC dialogue without any A/B press.
        let auto_continue_writetext = match visible_text_label_can_auto_continue(&runtime_shell) {
            Ok(value) => value,
            Err(error) => {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
                return;
            }
        };
        if auto_continue_writetext {
            // One physical edge belongs to one text state. If this frame
            // finishes PrintText, do not let the hotkey system reuse the edge
            // against the wait/prompt published by the resumed script.
            runtime_shell.field_text_consumed_a |= keys.just_pressed(KeyCode::KeyZ);
            runtime_shell.field_text_consumed_b |= keys.just_pressed(KeyCode::KeyX);
            if let Err(error) = advance_visible_text_label(&mut runtime_shell) {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
            }
            return;
        }
        // UseFlashTextScript is text_asm rather than an ordinary field-move
        // prompt: after printing it plays and waits for SFX_FLASH, then the
        // outer script runs BlindingFlash without a button boundary.
        if runtime_shell.visible_flash_animation.is_some() && runtime_shell.field_notice.is_some() {
            match runtime_shell.shell.presentation_snapshot() {
                Ok(snapshot)
                    if visible_field_dialogue_is_fully_revealed(&runtime_shell, &snapshot) =>
                {
                    runtime_shell.field_notice = None;
                    if let Err(error) = play_pending_field_notice_sound(&mut runtime_shell) {
                        record_visible_runtime_error(&mut runtime_shell, &error);
                        runtime_shell.last_error = Some(error.to_string());
                        return;
                    }
                    runtime_shell.visible_wait_sfx_boundary = true;
                    runtime_shell.wait_play_sfx_completion =
                        Some(VisibleWaitPlaySfxCompletion::FlashFieldMove);
                    mark_runtime_snapshot_dirty(&mut runtime_shell);
                    return;
                }
                Ok(_) => {}
                Err(error) => {
                    record_visible_runtime_error(&mut runtime_shell, &error);
                    runtime_shell.last_error = Some(error.to_string());
                    return;
                }
            }
        }
        let field_item_found_text_ready = runtime_shell
            .visible_field_item_notice
            .as_ref()
            .is_some_and(|notice| {
                notice.phase == VisibleFieldItemPhase::FoundText
                    && runtime_shell.field_notice.as_deref()
                        == Some(notice.sound_trigger_text.as_str())
            });
        if field_item_found_text_ready {
            match runtime_shell.shell.presentation_snapshot() {
                Ok(snapshot)
                    if visible_field_dialogue_is_fully_revealed(&runtime_shell, &snapshot) =>
                {
                    if let Err(error) = begin_visible_field_item_sound(&mut runtime_shell) {
                        record_visible_runtime_error(&mut runtime_shell, &error);
                        runtime_shell.last_error = Some(error.to_string());
                    }
                    return;
                }
                Ok(_) => {}
                Err(error) => {
                    record_visible_runtime_error(&mut runtime_shell, &error);
                    runtime_shell.last_error = Some(error.to_string());
                    return;
                }
            }
        }
        let egg_hatch_text_sound_ready = runtime_shell
            .visible_egg_hatch
            .as_ref()
            .is_some_and(|hatch| hatch.phase == VisibleEggHatchPhase::HatchText)
            && runtime_shell.pending_field_notice_sound.as_deref() == Some("SFX_CAUGHT_MON");
        if egg_hatch_text_sound_ready {
            match runtime_shell.shell.presentation_snapshot() {
                Ok(snapshot)
                    if visible_field_dialogue_is_fully_revealed(&runtime_shell, &snapshot) =>
                {
                    if let Err(error) = play_pending_field_notice_sound(&mut runtime_shell) {
                        record_visible_runtime_error(&mut runtime_shell, &error);
                        runtime_shell.last_error = Some(error.to_string());
                    }
                    return;
                }
                Ok(_) => {}
                Err(error) => {
                    record_visible_runtime_error(&mut runtime_shell, &error);
                    runtime_shell.last_error = Some(error.to_string());
                    return;
                }
            }
        }
        let automatic_field_effect_ready = runtime_shell.field_notice.is_some()
            && runtime_shell.visible_flash_animation.is_none()
            && (runtime_shell.pending_whirlpool_sound_wait
                || (runtime_shell.pending_field_notice_effect_frames.is_some()
                    && (runtime_shell.visible_cut_animation.is_some()
                        || runtime_shell.visible_headbutt_animation.is_some())));
        if automatic_field_effect_ready {
            match runtime_shell.shell.presentation_snapshot() {
                Ok(snapshot)
                    if visible_field_dialogue_is_fully_revealed(&runtime_shell, &snapshot) =>
                {
                    runtime_shell.field_notice = None;
                    if let Err(error) = play_pending_field_notice_sound(&mut runtime_shell) {
                        record_visible_runtime_error(&mut runtime_shell, &error);
                        runtime_shell.last_error = Some(error.to_string());
                        return;
                    }
                    if let Err(error) = begin_pending_field_notice_effect(&mut runtime_shell) {
                        record_visible_runtime_error(&mut runtime_shell, &error);
                        runtime_shell.last_error = Some(error.to_string());
                        return;
                    }
                    mark_runtime_snapshot_dirty(&mut runtime_shell);
                    return;
                }
                Ok(_) => {}
                Err(error) => {
                    record_visible_runtime_error(&mut runtime_shell, &error);
                    runtime_shell.last_error = Some(error.to_string());
                    return;
                }
            }
        }
        // A field textbox is a script boundary. The visual printer advances
        // on the host clock, but overworld frames must not: otherwise
        // autonomous object movement continues behind the text and scripted
        // characters (such as Mom) walk away before the player can respond.
        // `apply_runtime_hotkeys` still runs immediately after this system,
        // so A/Return can reveal and advance the current page normally.
        if runtime_shell.field_text_reveal.is_some() {
            // `writetext` itself is not always a joypad boundary. Radio
            // broadcasts use writetext -> pause -> writetext and must advance
            // automatically once the printer finishes; only waitbutton,
            // promptbutton and yesorno hand ownership to the player.
            let field_snapshot = match runtime_shell.shell.presentation_snapshot() {
                Ok(snapshot) => snapshot,
                Err(error) => {
                    record_visible_runtime_error(&mut runtime_shell, &error);
                    runtime_shell.last_error = Some(format!("{error:#}"));
                    return;
                }
            };
            if visible_field_dialog_pages(&field_snapshot, &runtime_shell).is_none()
                && runtime_shell.visible_script_movement_scene.is_none()
            {
                // The authoritative textbox and any retained movement frame
                // are both gone. Release the presentation-only printer now,
                // even while the script cursor continues into later commands.
                // Otherwise the stale printer captures every subsequent A/B
                // edge and strands commands such as disappear/end forever.
                runtime_shell.field_text_reveal = None;
                runtime_shell.rendered_field_text_identity = None;
                mark_runtime_snapshot_dirty(&mut runtime_shell);
            } else {
                let auto_continue =
                    visible_field_dialogue_is_entirely_consumed(&runtime_shell, &field_snapshot)
                        && field_snapshot.ui.pending_text_wait.is_none()
                        && field_snapshot.ui.pending_yes_no.is_none()
                        && !visible_non_text_player_boundary(&runtime_shell, &field_snapshot)
                        && runtime_shell.active_script_cursor.is_some();
                if auto_continue {
                    runtime_shell.field_text_consumed_a |= keys.just_pressed(KeyCode::KeyZ);
                    runtime_shell.field_text_consumed_b |= keys.just_pressed(KeyCode::KeyX);
                    if let Err(error) =
                        advance_visible_script_until_player_boundary(&mut runtime_shell)
                    {
                        record_visible_runtime_error(&mut runtime_shell, &error);
                        runtime_shell.last_error = Some(format!("{error:#}"));
                    }
                }
                return;
            }
        }
    } else if runtime_shell.active_script_cursor.is_none()
        && runtime_shell.field_text_reveal.take().is_some()
    {
        // The typewriter is presentation-only state. A script can close its
        // final textbox while resuming several commands in the same A press
        // (Mom's introductory script does exactly this), leaving no pending
        // authoritative text for the next frame to tick. Do not let that
        // stale cache retain exclusive joypad ownership forever.
        mark_runtime_snapshot_dirty(&mut runtime_shell);
    }

    if runtime_shell.pending_overworld_step_boundary.is_some() {
        if runtime_shell.player_walk_frame_ticks > 0 || runtime_shell.visible_ledge_jump.is_some() {
            // A map connection is still the same uninterrupted overworld
            // walk. Sample the live D-pad while its retained source-map step
            // finishes so a corner pressed during the seam is not lost before
            // the ordinary input path becomes reachable again. Other arrival
            // boundaries deliberately retain their complete input lock.
            let crossing_connection = runtime_shell
                .shell
                .last_frame()
                .is_some_and(|frame| frame.connection.is_some());
            if crossing_connection && runtime_shell.player_walk_frame_ticks > 0 {
                sync_overworld_held_directions(&keys, &mut runtime_shell, false);
                runtime_shell.overworld_buffered_direction = [
                    (KeyCode::ArrowUp, GameButton::Up),
                    (KeyCode::ArrowDown, GameButton::Down),
                    (KeyCode::ArrowLeft, GameButton::Left),
                    (KeyCode::ArrowRight, GameButton::Right),
                ]
                .into_iter()
                .find_map(|(key, direction)| keys.just_pressed(key).then_some(direction))
                .or(runtime_shell.overworld_buffered_direction);
            }
            return;
        }
        let boundary = runtime_shell
            .pending_overworld_step_boundary
            .take()
            .expect("checked pending overworld step boundary");
        match boundary {
            PendingOverworldStepBoundary::TrainerSight => {
                run_bevy_action(&mut runtime_shell, execute_last_trainer_sight_script);
            }
            PendingOverworldStepBoundary::Arrival => {
                runtime_shell.pending_overworld_warp_scene = None;
                run_bevy_action(&mut runtime_shell, settle_visible_overworld_frame_arrival);
            }
            PendingOverworldStepBoundary::CoordEvent => {
                run_bevy_action(&mut runtime_shell, execute_last_coord_event_script);
            }
            PendingOverworldStepBoundary::PhoneCall => {
                run_bevy_action(
                    &mut runtime_shell,
                    advance_visible_script_until_player_boundary,
                );
            }
            PendingOverworldStepBoundary::WildBattle => {
                run_bevy_action(&mut runtime_shell, settle_visible_battle_after_action);
            }
            PendingOverworldStepBoundary::PoisonBlackout(step_event) => {
                run_bevy_action(&mut runtime_shell, |shell| {
                    present_visible_step_event(shell, &step_event)?;
                    shell.pending_poison_blackout = true;
                    Ok(())
                });
            }
            PendingOverworldStepBoundary::StepEvent(step_event) => {
                run_bevy_action(&mut runtime_shell, |shell| {
                    present_visible_step_event(shell, &step_event)
                });
            }
        }
        return;
    }

    // These surfaces run their own joypad loops in Crystal. They still pass
    // through VBlank/GameTimer above, but they must not manufacture an empty
    // overworld frame behind the held presentation.
    if runtime_shell.start_menu_cursor.is_some()
        || runtime_shell.party_menu_open
        || runtime_shell.pokedex_menu_open
        || runtime_shell.pokegear_menu_open
        || runtime_shell.trainer_card_open
        || runtime_shell.options_menu_open
        || runtime_shell.save_menu_open
        || runtime_shell.storage_cursor.is_some()
        || runtime_shell.pc_item_cursor.is_some()
        || visible_field_pack_is_open(&runtime_shell)
        || !matches!(
            runtime_shell.shell.session().state().battle,
            crystal_core::state::BattleMemory::Inactive
        )
    {
        return;
    }

    let shift_pressed = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let alt_pressed = keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight);
    let ctrl_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let plain_input = !shift_pressed && !alt_pressed && !ctrl_pressed;
    // Input ownership queries inspect the semantic snapshot and several
    // catalogs. Do not run all five on every idle LCD frame: only the
    // physically requested control needs routing.
    let shell_consumes_a = if plain_input && keys.pressed(KeyCode::KeyZ) {
        match has_visible_shell_a_action(&mut runtime_shell) {
            Ok(consumes) => consumes,
            Err(error) => {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
                return;
            }
        }
    } else {
        false
    };
    let shell_consumes_b = plain_input
        && keys.pressed(KeyCode::KeyX)
        && has_visible_shell_b_action(&mut runtime_shell);
    let shell_consumes_start = plain_input
        && keys.pressed(KeyCode::Enter)
        && has_visible_shell_start_action(&mut runtime_shell);
    let shell_consumes_select = !alt_pressed
        && !ctrl_pressed
        && keys.pressed(KeyCode::ShiftRight)
        && has_visible_shell_select_action(&mut runtime_shell);
    let direction_requested = plain_input
        && (pending_direction_press.is_some()
            || [
                KeyCode::ArrowUp,
                KeyCode::ArrowDown,
                KeyCode::ArrowLeft,
                KeyCode::ArrowRight,
            ]
            .into_iter()
            .any(|key| keys.pressed(key)));
    let shell_consumes_direction =
        direction_requested && has_visible_shell_direction_action(&mut runtime_shell);
    let visible_shell_pressed = (plain_input
        && ((keys.just_pressed(KeyCode::KeyZ) && shell_consumes_a)
            || (keys.just_pressed(KeyCode::KeyX) && shell_consumes_b)
            || (keys.just_pressed(KeyCode::Enter) && shell_consumes_start)
            || (shell_consumes_direction
                && (just_pressed_overworld_direction(&keys).is_some()
                    || pending_direction_press.is_some()))))
        || (!alt_pressed
            && !ctrl_pressed
            && keys.just_pressed(KeyCode::ShiftRight)
            && shell_consumes_select);
    sync_overworld_held_directions(&keys, &mut runtime_shell, shell_consumes_direction);
    if visible_shell_pressed {
        return;
    }
    let mut buttons = collect_overworld_keyboard_buttons(
        &keys,
        shell_consumes_direction,
        shell_consumes_a,
        shell_consumes_b,
        shell_consumes_start,
        shell_consumes_select,
    );
    let newly_pressed_direction =
        just_pressed_overworld_direction(&keys).or(pending_direction_press);
    buttons.retain(|button| !is_direction_button(*button));
    if !shell_consumes_direction
        && let Some(direction) = newly_pressed_direction
            .or_else(|| runtime_shell.overworld_held_directions.back().copied())
    {
        buttons.insert(0, direction);
    }
    let ledge_jump_in_flight = runtime_shell.visible_ledge_jump.is_some();
    if ledge_jump_in_flight {
        buttons.clear();
    } else if runtime_shell.player_walk_frame_ticks > 0 {
        // Crystal does not dispatch A/Start/Select interactions from the
        // destination tile until the visible tile step has landed. Direction
        // input remains live so held walking can chain without a dead frame.
        buttons.retain(|button| is_direction_button(*button));
    }
    let direction_held = buttons.iter().any(|button| is_direction_button(*button));
    if ledge_jump_in_flight {
        // STEP_LEDGE owns sixteen visible frames even though the ordinary
        // walk interpolation timer reaches zero halfway through it. Keep a
        // newly pressed corner queued for the landing instead of entering the
        // idle branch below, which would consume and discard it while input is
        // still locked. Empty frames may continue advancing independent NPCs
        // and timers during the jump.
        if let Some(direction) = newly_pressed_direction {
            runtime_shell.overworld_buffered_direction = Some(direction);
        }
        runtime_shell.overworld_direction_repeat_ticks = 0;
        buttons.clear();
    } else if runtime_shell.player_walk_frame_ticks > 0 {
        // Direction changes are buffered while the current tile is visibly
        // in flight. Core commits tiles atomically, so forwarding a newly
        // pressed direction here would begin a second step before the first
        // sprite interpolation landed.
        if let Some(direction) = newly_pressed_direction {
            runtime_shell.overworld_buffered_direction = Some(direction);
        }
        runtime_shell.overworld_direction_repeat_ticks = 0;
        buttons.retain(|button| !is_direction_button(*button));
        // Core movement is tile-atomic and also evaluates ice/current/downhill
        // auto-steps on an empty joypad frame. Hold those surfaces at this
        // boundary until the shell finishes drawing the current tile;
        // otherwise a forced chain can commit multiple tiles inside one
        // visible step. On an ordinary tile, however, empty authoritative
        // frames safely keep NPCs and world timers moving concurrently with
        // the player's interpolation, as they do in TypeScript/Crystal.
        if runtime_shell
            .shell
            .session()
            .overworld()
            .forced_movement_direction()
            .is_some()
        {
            return;
        }
    } else {
        if let Some(direction) = runtime_shell.overworld_buffered_direction.take() {
            // A press edge captured during the preceding visible tile is a
            // complete queued joypad command. Execute it once at landing even
            // if the physical key was released meanwhile; requiring a live
            // hold here silently loses quick turns beside walls/counters.
            buttons.retain(|button| !is_direction_button(*button));
            buttons.insert(0, direction);
        }
        if let Some(direction) = buttons
            .iter()
            .copied()
            .find(|button| is_direction_button(*button))
            && (newly_pressed_direction.is_some()
                || runtime_shell.overworld_held_direction != Some(direction))
        {
            // Each directional animation owns its own four-step cycle in
            // TypeScript. A fresh press or turn starts on the ordinary action
            // frame; only an uninterrupted same-direction hold carries the
            // alternate-foot phase across the landing boundary.
            runtime_shell.player_walk_stride = false;
            runtime_shell.player_walk_mirror_stride = false;
        }
        let blocked_by_walking_object_origin = buttons
            .iter()
            .copied()
            .find(|button| is_direction_button(*button))
            .and_then(game_button_direction)
            .and_then(|direction| {
                crate::core::world::movement::checked_move_by_stride(
                    runtime_shell.shell.session().overworld().player.tile,
                    direction,
                    crate::core::world::movement::DEFAULT_RUNTIME_TILE_STRIDE,
                )
            })
            .is_some_and(|target| {
                runtime_shell
                    .object_walk_from
                    .values()
                    .any(|origin| *origin == target)
            });
        if blocked_by_walking_object_origin {
            // InitStep moves OBJECT_MAP_* to the destination while retaining
            // OBJECT_LAST_MAP_* as occupied until the step ends. Core's
            // tile-atomic NPC authority exposes only the destination, so keep
            // its retained visual origin collision-owned here as well.
            runtime_shell.overworld_direction_repeat_ticks = 0;
            buttons.retain(|button| !is_direction_button(*button));
            if !runtime_shell.transient_audio_playing
                && !runtime_shell
                    .pending_audio
                    .iter()
                    .any(|command| !matches!(command.kind, ModpackAudioKind::Music))
                && let Err(error) = queue_visible_shell_sound_effect(&mut runtime_shell, "SFX_BUMP")
            {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
                return;
            }
        } else {
            throttle_held_overworld_direction(&mut runtime_shell, &mut buttons);
        }
    }
    let input_active = !buttons.is_empty() || direction_held;
    let elapsed_ticks = elapsed_input_ticks;
    // Reaching the authoritative input loop consumes this update's gameplay
    // budget here. Modal routes return above and hand the same-update budget
    // to apply_runtime_hotkeys instead.
    timer.take_presentation_ticks();

    let mut tick_ok = false;
    let mut execute_overworld_arrival = false;
    let mut execute_coord_event_script = false;
    let mut execute_trainer_sight_script = false;
    let mut execute_phone_call_script = false;
    let mut execute_interaction_script = false;
    let mut execute_wild_battle_boundary = false;
    let mut execute_poison_blackout = false;
    let mut visible_step_event = None;
    let mut execute_contextual_field_move = false;
    let mut final_frame = None;
    let mut movement_scene_before_final_tick = None;
    let mut tick_error = None;
    let mut player_facing_changed = false;
    let mut remaining_catch_up_ticks = 0;
    for tick_index in 0..elapsed_ticks {
        let movement_scene_before_tick = if buttons.iter().copied().any(is_direction_button)
            || runtime_shell
                .shell
                .session()
                .overworld()
                .forced_movement_direction()
                .is_some()
        {
            match runtime_shell.shell.presentation_snapshot() {
                Ok(snapshot) => Some(snapshot),
                Err(error) => {
                    tick_error = Some(error);
                    break;
                }
            }
        } else {
            None
        };
        let object_tiles_before_tick = {
            let overworld = &runtime_shell.shell.session().overworld();
            let mut tiles = BTreeMap::new();
            for (index, object) in overworld
                .objects
                .iter()
                .enumerate()
                .filter(|(index, object)| {
                    overworld.object_has_loaded_struct(*index)
                        && overworld.is_object_visible(object)
                })
            {
                let Some(object_id) = object.object_identifier.as_ref() else {
                    continue;
                };
                match overworld.object_runtime_tile_checked(index, object) {
                    Ok(tile) => {
                        tiles.insert(object_id.clone(), tile);
                    }
                    Err(error) => {
                        tick_error = Some(error.into());
                        break;
                    }
                }
            }
            tiles
        };
        if tick_error.is_some() {
            break;
        }
        let object_facings_before_tick = runtime_shell
            .shell
            .session()
            .overworld()
            .object_facings
            .clone();
        let player_facing_before_tick = runtime_shell.shell.session().overworld().player.facing;
        let tick = if rtc_changed {
            rtc_changed = false;
            runtime_shell
                .shell
                .tick_with_rtc_after_vblank(buttons.clone(), rtc_sample)
        } else {
            runtime_shell.shell.tick_after_vblank(buttons.clone())
        };
        match tick.map(Clone::clone) {
            Ok(frame) => {
                player_facing_changed |= runtime_shell.shell.session().overworld().player.facing
                    != player_facing_before_tick;
                let reached_boundary = overworld_frame_reaches_presentation_boundary(&frame);
                let object_tiles_after_tick = runtime_shell
                    .shell
                    .session()
                    .overworld()
                    .object_runtime_tiles
                    .clone();
                let object_step_durations_after_tick = runtime_shell
                    .shell
                    .session()
                    .overworld()
                    .object_step_durations
                    .clone();
                let mut newly_walking = object_tiles_before_tick
                    .into_iter()
                    .filter(|(object_id, from)| {
                        object_tiles_after_tick
                            .get(object_id)
                            .is_some_and(|to| to != from)
                    })
                    .collect::<BTreeMap<_, _>>();
                if matches!(frame.movement, Some(StepOutcome::Moved { .. }))
                    && let Some((leader_id, follower_id)) = runtime_shell
                        .shell
                        .session()
                        .overworld()
                        .normal_follow_object_ids()
                    && leader_id == "PLAYER"
                    && follower_id != "PLAYER"
                    && let Some(from) = newly_walking.remove(&follower_id)
                {
                    // TypeScript queues this command at the leader's landing
                    // edge. Keep the authoritative destination, but retain
                    // the follower at its origin until that same LCD edge.
                    let to = object_tiles_after_tick[&follower_id];
                    let direction = if to.y > from.y {
                        Direction::Down
                    } else if to.y < from.y {
                        Direction::Up
                    } else if to.x < from.x {
                        Direction::Left
                    } else {
                        Direction::Right
                    };
                    runtime_shell
                        .pending_follower_walks
                        .push_back(VisibleFollowerWalk {
                            object_id: follower_id.clone(),
                            from,
                            to,
                            direction,
                        });
                    runtime_shell
                        .follower_visible_tile_overrides
                        .entry(follower_id)
                        .or_insert(from);
                }
                let newly_walking_ids = newly_walking.keys().cloned().collect::<BTreeSet<_>>();
                let pushed_boulder = runtime_shell
                    .shell
                    .session()
                    .overworld()
                    .objects
                    .iter()
                    .find_map(|object| {
                        let object_id = object.object_identifier.as_ref()?;
                        (newly_walking_ids.contains(object_id)
                            && object_movement_spawns_strength_dust(&object.spritemovedata))
                            .then(|| {
                                let from = newly_walking.get(object_id).copied();
                                let to = object_tiles_after_tick.get(object_id).copied();
                                let direction = match (from, to) {
                                    (Some(from), Some(to)) if to.y > from.y => Direction::Down,
                                    (Some(from), Some(to)) if to.y < from.y => Direction::Up,
                                    (Some(from), Some(to)) if to.x < from.x => Direction::Left,
                                    (Some(_), Some(_)) => Direction::Right,
                                    _ => frame.snapshot.facing,
                                };
                                (
                                    object_id.clone(),
                                    direction,
                                    object_step_durations_after_tick.get(object_id).copied(),
                                )
                            })
                    });
                for (object_id, from) in newly_walking {
                    let step_ticks = visible_object_step_duration(
                        &object_step_durations_after_tick,
                        &object_id,
                    );
                    runtime_shell
                        .object_walk_from
                        .insert(object_id.clone(), from);
                    runtime_shell
                        .object_walk_frame_ticks_by_id
                        .insert(object_id.clone(), step_ticks);
                    runtime_shell
                        .object_walk_total_ticks_by_id
                        .insert(object_id, step_ticks);
                }
                if let Some((object_id, direction, Some(step_duration))) = pushed_boulder.as_ref() {
                    runtime_shell.visible_strength_boulder_dust =
                        Some(VisibleStrengthBoulderDust {
                            object_id: object_id.clone(),
                            direction: *direction,
                            frames_remaining: visible_strength_dust_duration(*step_duration),
                            age: 0,
                        });
                } else if let Some((object_id, _, None)) = pushed_boulder {
                    tick_error = Some(anyhow::anyhow!(
                        "Strength movement for {object_id} has no authoritative object step duration"
                    ));
                    break;
                }
                let object_facings_after_tick = runtime_shell
                    .shell
                    .session()
                    .overworld()
                    .object_facings
                    .clone();
                for (object_id, direction) in object_facings_after_tick {
                    if newly_walking_ids.contains(&object_id) {
                        advance_object_walk_phase(&mut runtime_shell, &object_id, direction);
                    } else if object_facings_before_tick.get(&object_id) != Some(&direction) {
                        runtime_shell
                            .object_walk_phases
                            .insert(object_id.clone(), 0);
                        runtime_shell
                            .object_walk_direction_phases
                            .insert((object_id, direction), 0);
                    }
                }
                movement_scene_before_final_tick = movement_scene_before_tick;
                final_frame = Some(frame);
                remaining_catch_up_ticks = elapsed_ticks.saturating_sub(tick_index + 1);
                if reached_boundary
                    || final_frame.as_ref().is_some_and(|frame| {
                        frame.movement.is_some() || frame.autonomous_objects_changed
                    })
                {
                    // A queued corner belongs only to this continuous walk.
                    // Never carry it through a warp, encounter, trainer, or
                    // script boundary into a newly controlled scene.
                    if reached_boundary {
                        runtime_shell.overworld_buffered_direction = None;
                    }
                    // Core movement is tile-atomic, while the LCD owns the
                    // following player/object turn, walk, or bump cadence.
                    // Never consume a second accumulated host tick before
                    // that result has installed its visible presentation
                    // boundary.
                    break;
                }
            }
            Err(error) => {
                tick_error = Some(error);
                break;
            }
        }
    }
    let tick_result: Result<crate::RuntimeOverworldFrame> = match (final_frame, tick_error) {
        (Some(frame), None) => Ok(frame),
        (None, Some(error)) => Err(error),
        (None, None) => Err(anyhow::anyhow!(
            "runtime tick timer reported no elapsed frame"
        )),
        (Some(_), Some(error)) => Err(error),
    };
    match tick_result {
        Ok(frame) => {
            tick_ok = true;
            let player_moved = matches!(frame.movement, Some(StepOutcome::Moved { .. }));
            // TypeScript main and Crystal hold a facing change for four LCD
            // frames before the same held direction begins its tile step.
            // A wall/object bump remains immediately retryable; assigning the
            // turn cadence to every non-move made collision feel sticky.
            if direction_held {
                match frame.movement.as_ref() {
                    Some(StepOutcome::Turned { .. }) => {
                        runtime_shell.overworld_direction_repeat_ticks =
                            OVERWORLD_TURN_HOLD_TICKS.saturating_sub(1);
                    }
                    Some(StepOutcome::Blocked { .. })
                    | Some(StepOutcome::BlockedByObject { .. })
                    | Some(StepOutcome::RuntimeTileOverflow { .. }) => {
                        runtime_shell.overworld_direction_repeat_ticks = 0;
                    }
                    _ => {}
                }
            }
            if matches!(
                frame.movement,
                Some(
                    StepOutcome::Blocked { .. }
                        | StepOutcome::BlockedByObject { .. }
                        | StepOutcome::RuntimeTileOverflow { .. }
                )
            ) && matches!(
                frame.snapshot.mode,
                MovementMode::Normal | MovementMode::Bike | MovementMode::Skate
            ) && !runtime_shell.transient_audio_playing
                && !runtime_shell
                    .pending_audio
                    .iter()
                    .any(|command| !matches!(command.kind, ModpackAudioKind::Music))
                && let Err(error) = queue_visible_shell_sound_effect(&mut runtime_shell, "SFX_BUMP")
            {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
                return;
            }
            // A no-input frame still advances the authoritative Game Boy
            // frame counter, but it does not require rebuilding Bevy's
            // semantic snapshot or viewport. Invalidate only when the frame
            // produced a visible/gameplay mutation; script handlers below
            // invalidate their own mutations through `run_bevy_action`.
            if frame.movement.is_some()
                || player_facing_changed
                || direction_held
                || frame.autonomous_objects_changed
                || frame.step_events.is_some()
                || frame.phone_call.is_some()
                || frame.coord_event.is_some()
                || frame.trainer_sight.is_some()
                || frame.interaction.is_some()
                || frame.warp.is_some()
                || frame.connection.is_some()
                || frame.wild_encounter.is_some()
                || frame.wild_battle.is_some()
            {
                mark_runtime_snapshot_dirty(&mut runtime_shell);
            }
            if frame.autonomous_objects_changed {
                runtime_shell.object_walk_total_ticks = WALK_FRAME_HOLD_TICKS;
                runtime_shell.object_walk_frame_ticks = WALK_FRAME_HOLD_TICKS;
            }
            if player_moved {
                let (mut from, speed_multiplier) = match frame.movement.as_ref() {
                    Some(StepOutcome::Moved {
                        from,
                        speed_multiplier,
                        ..
                    }) => (*from, (*speed_multiplier).max(1)),
                    _ => unreachable!("player_moved requires a moved outcome"),
                };
                if let Some(connection) = frame.connection.as_ref() {
                    let opposite = match frame.snapshot.facing {
                        Direction::Up => Direction::Down,
                        Direction::Down => Direction::Up,
                        Direction::Left => Direction::Right,
                        Direction::Right => Direction::Left,
                    };
                    if let Some(connection_from) =
                        crate::core::world::movement::checked_move_by_stride(
                            connection.destination.tile,
                            opposite,
                            crate::core::world::movement::DEFAULT_RUNTIME_TILE_STRIDE,
                        )
                    {
                        // Connection resolution has already translated the
                        // authority into target-map coordinates. Interpolate
                        // from the adjacent target-map edge, never from the
                        // unrelated numeric coordinate in the source map.
                        from = connection_from;
                    }
                }
                let step_ticks = (WALK_FRAME_HOLD_TICKS / speed_multiplier).max(1);
                runtime_shell.player_walk_from = Some(from);
                advance_player_walk_phase(&mut runtime_shell, frame.snapshot.facing);
                runtime_shell.player_walk_total_ticks = step_ticks;
                runtime_shell.player_walk_frame_ticks =
                    visible_new_step_frames_remaining(step_ticks, remaining_catch_up_ticks);
                runtime_shell.overworld_direction_repeat_ticks = step_ticks.saturating_sub(1);
            }
            if let Some(LedgeJumpOutcome::Jumped { from, to, .. }) = frame.ledge_jump {
                runtime_shell.visible_ledge_jump = Some(VisibleLedgeJump {
                    from,
                    to,
                    frame: u8::try_from(remaining_catch_up_ticks)
                        .unwrap_or(u8::MAX)
                        .min(15),
                });
                let BevyRuntimeShell {
                    shell,
                    pending_audio,
                    last_audio_events,
                    ..
                } = &mut *runtime_shell;
                if let Err(error) = queue_visible_sound_effect(
                    shell.runtime().audio(),
                    pending_audio,
                    last_audio_events,
                    "SFX_JUMP_OVER_LEDGE",
                ) {
                    record_visible_runtime_error(&mut runtime_shell, &error);
                    runtime_shell.last_error = Some(error.to_string());
                    return;
                }
            }
            if let Some(rustle) = frame.grass_rustle {
                runtime_shell.visible_grass_rustle = Some(VisibleGrassRustle {
                    tile: rustle.tile,
                    frames_remaining: rustle.duration_frames,
                    age: 0,
                });
            }
            execute_overworld_arrival = frame.warp.is_some() || frame.connection.is_some();
            execute_coord_event_script = frame.coord_event.is_some();
            execute_trainer_sight_script = frame.trainer_sight.is_some();
            execute_phone_call_script = frame.phone_call.is_some();
            let interaction_targets_walking_object = frame
                .interaction
                .as_ref()
                .and_then(|interaction| match &interaction.target {
                    crate::core::world::session::OverworldInteractionTarget::Object {
                        object_identifier: Some(object_id),
                        ..
                    } => Some(object_id),
                    _ => None,
                })
                .is_some_and(|object_id| runtime_shell.object_walk_from.contains_key(object_id));
            execute_interaction_script = keys.just_pressed(KeyCode::KeyZ)
                && runtime_shell.player_walk_frame_ticks == 0
                && !shell_consumes_a
                && !interaction_targets_walking_object
                && frame.interaction.is_some();
            execute_contextual_field_move = keys.just_pressed(KeyCode::KeyZ)
                && runtime_shell.player_walk_frame_ticks == 0
                && !shell_consumes_a
                && frame.interaction.is_none()
                && frame.wild_battle.is_none();
            execute_wild_battle_boundary = frame.wild_battle.is_some();
            visible_step_event = frame.step_events.clone().filter(|events| {
                events.repel_expired.is_some()
                    || events.egg_hatched
                    || events.poison_result.is_some()
            });
            execute_poison_blackout = frame
                .step_events
                .as_ref()
                .and_then(|events| events.poison_result.as_ref())
                .is_some_and(|poison| !poison.fainted_names.is_empty())
                && !runtime_shell
                    .shell
                    .session()
                    .state()
                    .storage
                    .party
                    .pokemon
                    .iter()
                    .flatten()
                    .any(|pokemon| {
                        !pokemon.is_egg && pokemon.species.id != "EGG" && pokemon.hp > 0
                    });
            let movement_is_visibly_in_flight = overworld_boundary_waits_for_visible_landing(
                &frame,
                runtime_shell.player_walk_frame_ticks,
                runtime_shell.visible_ledge_jump.is_some(),
            );
            if movement_is_visibly_in_flight {
                if frame.warp.is_some()
                    && let (Some(mut source_scene), Some(StepOutcome::Moved { from, to, .. })) =
                        (movement_scene_before_final_tick, frame.movement.as_ref())
                {
                    // Core has already installed the destination session. Keep
                    // the source map and place its presentation target on the
                    // triggering warp tile until the walk visibly lands.
                    source_scene.overworld.tile = *to;
                    source_scene.overworld.facing = if to.x > from.x {
                        Direction::Right
                    } else if to.x < from.x {
                        Direction::Left
                    } else if to.y > from.y {
                        Direction::Down
                    } else {
                        Direction::Up
                    };
                    runtime_shell.pending_overworld_warp_scene = Some(Arc::new(source_scene));
                }
                runtime_shell.pending_overworld_step_boundary =
                    pending_overworld_step_boundary_for_frame(
                        &frame,
                        execute_poison_blackout,
                        visible_step_event.clone(),
                    );
                if runtime_shell.pending_overworld_step_boundary.is_some() {
                    // A buffered direction belongs only to uninterrupted
                    // overworld walking. Coord scripts, warps, trainer sight,
                    // calls, battles, and step events take ownership at this
                    // tile; replaying the edge after their relocation or
                    // cutscene can move the player out from under the authored
                    // boundary.
                    runtime_shell.overworld_buffered_direction = None;
                    if matches!(
                        runtime_shell.pending_overworld_step_boundary,
                        Some(PendingOverworldStepBoundary::WildBattle)
                    ) {
                        // Stage the battle data/messages now so the committed
                        // battle snapshot cannot leak directly onto the map.
                        // Its transition remains frozen at frame zero until
                        // the visible step boundary is dispatched.
                        if let Err(error) =
                            prepare_visible_battle_entry_after_visible_step(&mut runtime_shell)
                        {
                            record_visible_runtime_error(&mut runtime_shell, &error);
                            runtime_shell.last_error = Some(error.to_string());
                            return;
                        }
                    }
                    execute_overworld_arrival = false;
                    execute_coord_event_script = false;
                    execute_trainer_sight_script = false;
                    execute_phone_call_script = false;
                    execute_wild_battle_boundary = false;
                    execute_poison_blackout = false;
                    visible_step_event = None;
                }
            }
            let input_frame =
                match deterministic_input_frame_from_post_tick_checksum(&frame.state_checksum) {
                    Ok(input_frame) => input_frame,
                    Err(error) => {
                        record_visible_runtime_error(&mut runtime_shell, &error);
                        runtime_shell.last_error = Some(error.to_string());
                        return;
                    }
                };
            let input_record = VisibleOverworldInputRecord {
                frame: input_frame,
                input_mask: frame.input_mask,
                pressed_mask: frame.pressed_mask,
                player_moved,
                state_checksum: frame.state_checksum.clone(),
            };
            let frame_activity = if input_active {
                summarize_frame_activity(&frame)
            } else {
                None
            };
            if input_active {
                set_visible_runtime_action_from_checksum(
                    &mut runtime_shell,
                    format!(
                        "input:overworld:frame:{}:mask:{:#010b}:pressed:{:#010b}",
                        input_record.frame, input_record.input_mask, input_record.pressed_mask
                    ),
                    &input_record.state_checksum,
                );
            }
            if let Err(error) = record_visible_overworld_input(&mut runtime_shell, input_record) {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
                return;
            }
            runtime_shell.last_error = None;
            if let Some(activity) = frame_activity {
                runtime_shell.last_audio_events.push(activity);
                trim_event_log(&mut runtime_shell.last_audio_events);
            }
        }
        Err(error) => {
            record_visible_runtime_error(&mut runtime_shell, &error);
            runtime_shell.last_error = Some(error.to_string());
        }
    }
    sync_visible_battle_action_cursor(&mut runtime_shell);
    if !tick_ok {
        return;
    }
    if execute_trainer_sight_script {
        run_bevy_action(&mut runtime_shell, execute_last_trainer_sight_script);
    } else if execute_overworld_arrival {
        run_bevy_action(&mut runtime_shell, settle_visible_overworld_frame_arrival);
    } else if execute_coord_event_script {
        run_bevy_action(&mut runtime_shell, execute_last_coord_event_script);
    } else if execute_phone_call_script {
        run_bevy_action(
            &mut runtime_shell,
            advance_visible_script_until_player_boundary,
        );
    } else if execute_interaction_script {
        runtime_shell.overworld_interaction_consumed_a = true;
        run_bevy_action(&mut runtime_shell, execute_last_interaction_script);
    } else if execute_wild_battle_boundary {
        if let Err(error) = prepare_visible_battle_entry(&mut runtime_shell) {
            record_visible_runtime_error(&mut runtime_shell, &error);
            runtime_shell.last_error = Some(error.to_string());
            return;
        }
        run_bevy_action(&mut runtime_shell, settle_visible_battle_after_action);
    } else if execute_poison_blackout {
        if let Some(step_event) = visible_step_event {
            run_bevy_action(&mut runtime_shell, |shell| {
                present_visible_step_event(shell, &step_event)?;
                shell.pending_poison_blackout = true;
                Ok(())
            });
        }
    } else if let Some(step_event) = visible_step_event {
        run_bevy_action(&mut runtime_shell, |shell| {
            present_visible_step_event(shell, &step_event)
        });
    } else if execute_contextual_field_move {
        match execute_visible_contextual_field_move(&mut runtime_shell) {
            Ok(true) => {}
            Ok(false) => match advance_visible_script_until_player_boundary(&mut runtime_shell) {
                Ok(()) => {}
                Err(error) => {
                    record_visible_runtime_error(&mut runtime_shell, &error);
                    runtime_shell.last_error = Some(error.to_string());
                }
            },
            Err(error) => {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
            }
        }
    } else if runtime_shell.pending_overworld_step_boundary.is_none()
        && (runtime_shell.active_script_cursor.is_some()
            || runtime_shell.shell.has_pending_script_work())
    {
        // Do not take an additional snapshot/scan solely to discover pending
        // script work. The cheap predicate enters this path only when a
        // script can actually make progress; input transaction staging is
        // intentionally accounted for separately.
        match advance_visible_script_until_player_boundary(&mut runtime_shell) {
            Ok(()) => {}
            Err(error) => {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
            }
        }
    }
}

fn advance_visible_map_name_sign(
    sign: &mut Option<VisibleMapNameSign>,
    elapsed_input_ticks: u32,
) -> bool {
    // PlaceMapNameSign hides old timer values 60 and 59. The old-59 pass
    // decrements to 58, initializes the name, and exposes WY=$70; timer zero
    // remains visible until the following pass takes the disappear branch.
    let was_visible = sign
        .as_ref()
        .is_some_and(|sign| sign.frames_remaining <= 58);
    let mut clear = false;
    if let Some(sign) = sign.as_mut() {
        for _ in 0..elapsed_input_ticks {
            if sign.frames_remaining == 0 {
                clear = true;
                break;
            }
            sign.frames_remaining -= 1;
        }
    }
    if clear {
        *sign = None;
    }
    let is_visible = sign
        .as_ref()
        .is_some_and(|sign| sign.frames_remaining <= 58);
    was_visible != is_visible
}

fn advance_visible_poison_flash(
    runtime_shell: &mut BevyRuntimeShell,
    elapsed_input_ticks: u32,
) -> Result<bool> {
    if runtime_shell.poison_flash_frames_remaining == 0 {
        return Ok(false);
    }
    runtime_shell.poison_flash_frames_remaining = runtime_shell
        .poison_flash_frames_remaining
        .saturating_sub(elapsed_input_ticks.min(u32::from(u8::MAX)) as u8);
    if runtime_shell.poison_flash_frames_remaining == 0
        && let Some(notice) = runtime_shell.field_notice_queue.pop_front()
    {
        let scene = Arc::new(runtime_shell.shell.snapshot()?);
        runtime_shell.field_notice = Some(notice);
        runtime_shell.field_notice_scene = Some(scene);
        runtime_shell.field_text_reveal = None;
    }
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(true)
}

/// Presentation states that install an hVBlank handler other than
/// VBlank_Normal in the original engine. Those handlers deliberately skip
/// the two DIV reads which advance hRandomAdd/hRandomSub.
fn visible_special_vblank_handler_active(runtime_shell: &BevyRuntimeShell) -> bool {
    runtime_shell.credits_screen.is_some()
        || runtime_shell.visible_magnet_train.is_some()
        || runtime_shell.visible_battle_transition.is_some()
        || runtime_shell
            .visible_capture_animation
            .as_ref()
            .is_some_and(|animation| animation.started)
        || runtime_shell
            .visible_move_animations
            .front()
            .is_some_and(|animation| animation.started)
        || runtime_shell.visible_send_out_animation.is_some()
}

fn visible_vblank_counter_bit4(runtime_shell: &BevyRuntimeShell) -> bool {
    runtime_shell.shell.session().state().vblank_counter & (1 << 4) != 0
}

fn advance_visible_music_fade(
    runtime_shell: &mut BevyRuntimeShell,
    elapsed_vblanks: u32,
) -> Result<()> {
    for _ in 0..elapsed_vblanks {
        let Some(fade) = runtime_shell.music_fade.as_mut() else {
            break;
        };
        if fade.count > 0 {
            fade.count -= 1;
            continue;
        }
        fade.count = fade.rate;
        if fade.fading_in {
            if runtime_shell.music_volume < 7 {
                runtime_shell.music_volume += 1;
                continue;
            }
            runtime_shell.music_fade = None;
            runtime_shell.faded_music = None;
            runtime_shell
                .last_audio_events
                .push("completed ASM bicycle music fade-in".to_string());
            trim_event_log(&mut runtime_shell.last_audio_events);
            continue;
        }
        if runtime_shell.music_volume > 0 {
            runtime_shell.music_volume -= 1;
            continue;
        }

        let target_music = fade.target_music.clone();
        let bicycle_fade_in = runtime_shell.shell.snapshot()?.overworld.mode == MovementMode::Bike;
        if bicycle_fade_in {
            // MusicFadeRestart calls _InitSound, which clears wMusicFade and
            // wMusicFadeCount. The bicycle branch then sets only bit 7, so
            // its fade-in always advances at rate zero.
            fade.rate = 0;
            fade.count = 0;
            fade.fading_in = true;
        } else {
            runtime_shell.music_fade = None;
            runtime_shell.faded_music = None;
            runtime_shell.music_volume = 7;
        }
        runtime_shell.pending_music_stop = true;
        runtime_shell.pending_full_audio_reset = true;
        clear_pending_music_commands(&mut runtime_shell.pending_audio);
        runtime_shell.active_music = Some(target_music.clone());
        if !is_silent_music_id(&target_music) {
            let playback = runtime_shell
                .shell
                .runtime()
                .audio()
                .require_playback_entry(AudioKind::Music, &target_music)?;
            enqueue_bevy_audio_command(
                &mut runtime_shell.pending_audio,
                BevyAudioCommand {
                    battle_sound: None,
                    cry_parameters: None,
                    audio_id: target_music.clone(),
                    kind: ModpackAudioKind::Music,
                    mode: playback.mode,
                    looped: matches!(
                        playback.loop_policy,
                        crate::assets::ModpackAudioLoopPolicy::Loop
                    ),
                },
            );
        }
        runtime_shell
            .last_audio_events
            .push(format!("loaded ASM music fade target {target_music}"));
        trim_event_log(&mut runtime_shell.last_audio_events);
    }
    Ok(())
}

fn release_input_on_focus_loss(
    mut focus_events: EventReader<WindowFocused>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut runtime_shell: ResMut<BevyRuntimeShell>,
) {
    if !focus_events.read().any(|event| !event.focused) {
        return;
    }
    reset_keyboard_after_focus_loss(&mut keys);
    runtime_shell.pending_overworld_direction_press = None;
    runtime_shell.pending_ui_button_presses.clear();
    runtime_shell.ui_held_direction = None;
    runtime_shell.ui_direction_repeat_ticks = 0;
    runtime_shell.overworld_interaction_consumed_a = false;
    runtime_shell.field_text_consumed_a = false;
    runtime_shell.field_text_consumed_b = false;
}

fn reset_keyboard_after_focus_loss(keys: &mut ButtonInput<KeyCode>) {
    keys.reset_all();
}

fn overworld_frame_reaches_presentation_boundary(frame: &crate::RuntimeOverworldFrame) -> bool {
    frame.step_events.as_ref().is_some_and(|events| {
        events.repel_expired.is_some() || events.egg_hatched || events.poison_result.is_some()
    }) || frame.trainer_sight.is_some()
        || frame.coord_event.is_some()
        || frame.phone_call.is_some()
        || frame.interaction.is_some()
        || frame.warp.is_some()
        || frame.connection.is_some()
        || frame.wild_battle.is_some()
}

fn overworld_boundary_waits_for_visible_landing(
    frame: &crate::RuntimeOverworldFrame,
    retained_walk_frames: u8,
    retained_ledge_jump: bool,
) -> bool {
    matches!(frame.movement, Some(StepOutcome::Moved { .. }))
        || matches!(frame.ledge_jump, Some(LedgeJumpOutcome::Jumped { .. }))
        || (frame.phone_call.is_some() && (retained_walk_frames > 0 || retained_ledge_jump))
}

fn pending_overworld_step_boundary_for_frame(
    frame: &crate::RuntimeOverworldFrame,
    poison_blackout: bool,
    visible_step_event: Option<crate::core::systems::step_events::StepEventResult>,
) -> Option<PendingOverworldStepBoundary> {
    if frame.trainer_sight.is_some() {
        Some(PendingOverworldStepBoundary::TrainerSight)
    } else if frame.warp.is_some() || frame.connection.is_some() {
        Some(PendingOverworldStepBoundary::Arrival)
    } else if frame.coord_event.is_some() {
        Some(PendingOverworldStepBoundary::CoordEvent)
    } else if frame.phone_call.is_some() {
        Some(PendingOverworldStepBoundary::PhoneCall)
    } else if frame.wild_battle.is_some() {
        Some(PendingOverworldStepBoundary::WildBattle)
    } else if poison_blackout {
        visible_step_event.map(PendingOverworldStepBoundary::PoisonBlackout)
    } else {
        visible_step_event.map(PendingOverworldStepBoundary::StepEvent)
    }
}

fn advance_visible_trainer_sight_cutscene(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    if runtime_shell
        .visible_overworld_emote
        .as_ref()
        .is_some_and(|emote| emote.frames_remaining > 0)
    {
        return Ok(());
    }
    runtime_shell.visible_overworld_emote = None;

    let Some(pending) = runtime_shell.pending_trainer_sight.as_mut() else {
        return Ok(());
    };
    if pending.frames_until_step > 0 {
        pending.frames_until_step -= 1;
        return Ok(());
    }
    if pending.steps_remaining == 0 {
        finish_visible_trainer_sight_script(runtime_shell)?;
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(());
    }

    let object_id = pending.object_id.clone();
    let direction = pending.direction;
    let current = runtime_shell
        .shell
        .session()
        .overworld()
        .object_runtime_tile_by_id(&object_id)?;
    let stride = crate::core::world::movement::DEFAULT_RUNTIME_TILE_STRIDE;
    let next = match direction {
        Direction::Up => TilePosition {
            x: current.x,
            y: current
                .y
                .checked_sub(stride)
                .context("trainer approach moved above runtime bounds")?,
        },
        Direction::Down => TilePosition {
            x: current.x,
            y: current
                .y
                .checked_add(stride)
                .context("trainer approach moved below runtime bounds")?,
        },
        Direction::Left => TilePosition {
            x: current
                .x
                .checked_sub(stride)
                .context("trainer approach moved left of runtime bounds")?,
            y: current.y,
        },
        Direction::Right => TilePosition {
            x: current
                .x
                .checked_add(stride)
                .context("trainer approach moved right of runtime bounds")?,
            y: current.y,
        },
    };
    {
        let overworld = &mut runtime_shell.shell.session_mut().overworld_mut();
        overworld.set_object_runtime_facing(&object_id, direction)?;
        overworld.set_object_runtime_tile(&object_id, next)?;
    }
    let pending = runtime_shell
        .pending_trainer_sight
        .as_mut()
        .context("trainer approach state disappeared while applying a step")?;
    pending.steps_remaining -= 1;
    // `slow_step` is twice the ordinary eight-frame walking cadence.
    pending.frames_until_step = WALK_FRAME_HOLD_TICKS.saturating_mul(2);
    advance_object_walk_phase(runtime_shell, &object_id, direction);
    runtime_shell.trainer_walk_from = Some((object_id, current));
    runtime_shell.object_walk_stride = !runtime_shell.object_walk_stride;
    runtime_shell.object_walk_total_ticks = WALK_FRAME_HOLD_TICKS.saturating_mul(2);
    runtime_shell.object_walk_frame_ticks = WALK_FRAME_HOLD_TICKS.saturating_mul(2);
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(())
}

fn visible_battle_animation_owns_frame(runtime_shell: &BevyRuntimeShell) -> bool {
    visible_trainer_result_animation_active(runtime_shell)
        || runtime_shell.visible_frontpic_animation.is_some()
        || runtime_shell
            .visible_capture_animation
            .as_ref()
            .is_some_and(|animation| animation.started)
        || runtime_shell
            .visible_move_animations
            .front()
            .is_some_and(|animation| animation.started)
        || runtime_shell.visible_send_out_animation.is_some()
        || runtime_shell.visible_trainer_exit_animation.is_some()
}

fn advance_visible_battle_animation_frame(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    if visible_trainer_result_animation_active(runtime_shell) {
        runtime_shell.battle_trainer_result.as_mut().unwrap().1 += 1;
        mark_runtime_presentation_dirty(runtime_shell);
        return Ok(());
    }
    if runtime_shell.visible_frontpic_animation.is_some() {
        return advance_visible_frontpic_animation(runtime_shell);
    }
    if runtime_shell
        .visible_capture_animation
        .as_ref()
        .is_some_and(|animation| animation.started)
    {
        return advance_visible_capture_animation(runtime_shell);
    }
    if runtime_shell
        .visible_move_animations
        .front()
        .is_some_and(|animation| animation.started)
    {
        return advance_visible_move_animation(runtime_shell);
    }
    if runtime_shell.visible_send_out_animation.is_some() {
        return advance_visible_send_out_animation(runtime_shell);
    }
    if runtime_shell.visible_trainer_exit_animation.is_some() {
        return advance_visible_trainer_exit_animation(runtime_shell);
    }
    Ok(())
}

fn next_player_walk_stride(_remaining_ticks: u8, current_stride: bool) -> bool {
    // TypeScript primes one entry of [standing, step, standing, mirrored step]
    // for each tile. `true` selects an action frame; the separate mirror bit
    // distinguishes the two action frames.
    !current_stride
}

fn visible_new_step_frames_remaining(step_ticks: u8, _ticks_elapsed_before_step: u32) -> u8 {
    // The host accumulator is drained before this authoritative movement is
    // created. Its remaining catch-up ticks therefore predate the new walk
    // and must not shorten the interpolation that starts in this update.
    step_ticks
}

fn visible_walk_ticks_for_host_update(elapsed_input_ticks: u32) -> u8 {
    u8::from(elapsed_input_ticks > 0)
}

fn advance_player_walk_phase(runtime_shell: &mut BevyRuntimeShell, direction: Direction) {
    let phase = next_directional_walk_phase(
        runtime_shell
            .player_walk_direction_phases
            .get(&direction)
            .copied(),
    );
    runtime_shell
        .player_walk_direction_phases
        .insert(direction, phase);
    runtime_shell.player_walk_stride = player_walk_uses_action_frame(phase & 1 == 1);
    runtime_shell.player_walk_mirror_stride = phase == 3;
}

fn player_walk_uses_action_frame(stride: bool) -> bool {
    stride
}

fn object_walk_uses_action_frame(phase: u8) -> bool {
    phase & 1 == 1
}

fn object_walk_uses_mirrored_action_frame(phase: u8) -> bool {
    phase & 3 == 3
}

fn advance_visible_walk_timers(runtime_shell: &mut BevyRuntimeShell, elapsed_ticks: u8) {
    if elapsed_ticks == 0 {
        return;
    }
    let player_landed = runtime_shell.player_walk_frame_ticks > 0
        && runtime_shell.player_walk_frame_ticks <= elapsed_ticks;
    runtime_shell.player_walk_frame_ticks = runtime_shell
        .player_walk_frame_ticks
        .saturating_sub(elapsed_ticks);
    if runtime_shell.player_walk_frame_ticks == 0 {
        runtime_shell.player_walk_from = None;
    }
    if player_landed {
        let overworld = &mut runtime_shell.shell.session_mut().overworld_mut();
        overworld.player_last_runtime_tile = None;
        overworld.player_last_tile_occupied_until_frame = 0;
    }
    runtime_shell.object_walk_frame_ticks = runtime_shell
        .object_walk_frame_ticks
        .saturating_sub(elapsed_ticks);
    for remaining in runtime_shell.object_walk_frame_ticks_by_id.values_mut() {
        *remaining = remaining.saturating_sub(elapsed_ticks);
    }
    let landed_object_ids = runtime_shell
        .object_walk_frame_ticks_by_id
        .iter()
        .filter_map(|(object_id, remaining)| (*remaining == 0).then_some(object_id.clone()))
        .collect::<Vec<_>>();
    let object_landed = !landed_object_ids.is_empty();
    for object_id in landed_object_ids {
        runtime_shell
            .object_walk_frame_ticks_by_id
            .remove(&object_id);
        runtime_shell
            .object_walk_total_ticks_by_id
            .remove(&object_id);
        runtime_shell.object_walk_from.remove(&object_id);
        runtime_shell
            .follower_visible_tile_overrides
            .remove(&object_id);
        let overworld = &mut runtime_shell.shell.session_mut().overworld_mut();
        overworld.object_last_runtime_tiles.remove(&object_id);
        overworld
            .object_last_tiles_occupied_until_frame
            .remove(&object_id);
    }
    if player_landed || object_landed {
        start_next_queued_follower_walk(runtime_shell);
        mark_runtime_snapshot_dirty(runtime_shell);
    }
    if runtime_shell.object_walk_frame_ticks == 0 {
        runtime_shell.trainer_walk_from = None;
        if runtime_shell.pending_trainer_sight.is_none()
            && runtime_shell.object_walk_frame_ticks_by_id.is_empty()
        {
            runtime_shell.object_walk_stride = false;
        }
    }
}

fn start_next_queued_follower_walk(runtime_shell: &mut BevyRuntimeShell) {
    let Some(next) = runtime_shell.pending_follower_walks.front() else {
        return;
    };
    if runtime_shell
        .object_walk_frame_ticks_by_id
        .contains_key(&next.object_id)
    {
        return;
    }
    let next = runtime_shell
        .pending_follower_walks
        .pop_front()
        .expect("front follower walk exists");
    advance_object_walk_phase(runtime_shell, &next.object_id, next.direction);
    runtime_shell
        .follower_visible_tile_overrides
        .insert(next.object_id.clone(), next.to);
    runtime_shell
        .object_walk_from
        .insert(next.object_id.clone(), next.from);
    runtime_shell
        .object_walk_total_ticks_by_id
        .insert(next.object_id.clone(), WALK_FRAME_HOLD_TICKS);
    runtime_shell
        .object_walk_frame_ticks_by_id
        .insert(next.object_id, WALK_FRAME_HOLD_TICKS);
}

fn advance_object_walk_phase(
    runtime_shell: &mut BevyRuntimeShell,
    object_id: &str,
    direction: Direction,
) {
    // TypeScript owns one SpriteAnimation per direction. Preserve that same
    // independent four-frame gait for followers that turn a corner and later
    // return to a direction they have already used.
    let direction_key = (object_id.to_string(), direction);
    let phase = next_directional_walk_phase(
        runtime_shell
            .object_walk_direction_phases
            .get(&direction_key)
            .copied(),
    );
    runtime_shell
        .object_walk_direction_phases
        .insert(direction_key, phase);
    runtime_shell
        .object_walk_phases
        .insert(object_id.to_string(), phase);
}

fn next_directional_walk_phase(previous_direction_phase: Option<u8>) -> u8 {
    previous_direction_phase.unwrap_or(0).wrapping_add(1) % 4
}

fn start_next_visible_script_movement_phase(runtime_shell: &mut BevyRuntimeShell) -> Result<bool> {
    loop {
        let next = runtime_shell
            .visible_script_movement
            .as_mut()
            .and_then(|movement| {
                movement
                    .phases
                    .pop_front()
                    .map(|phase| (movement.object_id.clone(), phase))
            });
        let Some((object_id, phase)) = next else {
            // GetFollowerNextMovementIndex leaves the newest command queued
            // when its queue length reaches zero. step_end does not enqueue
            // another command, so retain that one-step lag across programs.
            let pending = runtime_shell
                .visible_script_movement
                .as_mut()
                .and_then(|movement| movement.pending_programs.pop_front());
            if let Some(program) = pending {
                if let Some(movement) = runtime_shell.visible_script_movement.as_ref() {
                    if movement.object_id == "PLAYER" {
                        runtime_shell.visible_player_sprite_y_offset = movement.stationary_y_offset;
                    }
                }
                let revealed_object = if !program.previous_hidden && program.object_id != "PLAYER" {
                    runtime_shell
                        .shell
                        .session()
                        .overworld()
                        .objects
                        .iter()
                        .find(|object| {
                            object.object_identifier.as_deref() == Some(program.object_id.as_str())
                        })
                        .cloned()
                } else {
                    None
                };
                let scene = Arc::make_mut(
                    runtime_shell
                        .visible_script_movement_scene
                        .as_mut()
                        .context("queued visible script movement has no retained scene")?,
                );
                if program.object_id == "PLAYER" {
                    scene.overworld.tile = program.previous_tile;
                    scene.overworld.facing = program.previous_facing;
                    scene.overworld_player_hidden = program.previous_hidden;
                } else {
                    scene
                        .visible_object_runtime_tiles
                        .insert(program.object_id.clone(), program.previous_tile);
                    scene
                        .visible_object_facings
                        .insert(program.object_id.clone(), program.previous_facing);
                    if program.previous_hidden {
                        scene.visible_objects.retain(|object| {
                            object.object_identifier.as_deref() != Some(program.object_id.as_str())
                        });
                    } else if !scene.visible_objects.iter().any(|object| {
                        object.object_identifier.as_deref() == Some(program.object_id.as_str())
                    }) {
                        scene.visible_objects.push(revealed_object.with_context(|| {
                            format!(
                                "queued movement cannot restore unknown object {}",
                                program.object_id
                            )
                        })?);
                    }
                }
                let movement = runtime_shell
                    .visible_script_movement
                    .as_mut()
                    .context("visible script movement disappeared while switching programs")?;
                movement.object_id = program.object_id;
                movement.phases = program.phases;
                movement.hold_frames_remaining = 0;
                movement.active_jump_duration = None;
                movement.active_uses_standing_frame = false;
                movement.active_tree_shake_duration = None;
                movement.active_stationary_effect = None;
                movement.active_stationary_duration = 0;
                movement.stationary_y_offset = 0;
                movement.stationary_initial_facing = program.previous_facing;
                movement.follower_object_id = program.follower_object_id;
                movement.follower_queued_step = program.follower_queued_step;
                continue;
            }
            if let Some(movement) = runtime_shell.visible_script_movement.as_ref() {
                if movement.object_id == "PLAYER" {
                    runtime_shell.visible_player_sprite_y_offset = movement.stationary_y_offset;
                }
            }
            let completed_object = runtime_shell
                .visible_script_movement
                .as_ref()
                .map(|movement| movement.object_id.clone())
                .unwrap_or_else(|| "unknown".to_string());
            log_visible_movement_event(
                runtime_shell,
                "end",
                &completed_object,
                "retained movement scene released".to_string(),
            );
            runtime_shell.visible_script_movement = None;
            runtime_shell.visible_script_movement_scene = None;
            runtime_shell.player_walk_from = None;
            runtime_shell.player_walk_frame_ticks = 0;
            runtime_shell.player_walk_total_ticks = WALK_FRAME_HOLD_TICKS;
            runtime_shell.trainer_walk_from = None;
            runtime_shell.object_walk_frame_ticks = 0;
            runtime_shell.object_walk_total_ticks = WALK_FRAME_HOLD_TICKS;
            mark_runtime_snapshot_dirty(runtime_shell);
            return Ok(false);
        };
        match phase {
            VisibleScriptMovementPhase::Sound { audio_id } => {
                queue_visible_shell_sound_effect(runtime_shell, &audio_id)?;
                continue;
            }
            VisibleScriptMovementPhase::Hold { duration } => {
                if duration == 0 {
                    continue;
                }
                let movement = runtime_shell
                    .visible_script_movement
                    .as_mut()
                    .context("visible script movement disappeared while starting hold")?;
                movement.hold_frames_remaining = duration;
                movement.active_jump_duration = None;
                movement.active_uses_standing_frame = true;
                movement.active_tree_shake_duration = None;
                movement.active_stationary_effect = None;
                movement.active_stationary_duration = 0;
                mark_runtime_snapshot_dirty(runtime_shell);
                return Ok(true);
            }
            VisibleScriptMovementPhase::TreeShake { duration } => {
                if duration == 0 {
                    continue;
                }
                let movement = runtime_shell
                    .visible_script_movement
                    .as_mut()
                    .context("visible script movement disappeared while starting tree shake")?;
                movement.hold_frames_remaining = duration;
                movement.active_jump_duration = None;
                movement.active_uses_standing_frame = false;
                movement.active_tree_shake_duration = Some(duration);
                movement.active_stationary_effect = None;
                movement.active_stationary_duration = 0;
                movement.stationary_y_offset = 0;
                mark_runtime_snapshot_dirty(runtime_shell);
                return Ok(true);
            }
            VisibleScriptMovementPhase::Visibility { hidden } => {
                let revealed_object = if !hidden && object_id != "PLAYER" {
                    runtime_shell
                        .shell
                        .session()
                        .overworld()
                        .objects
                        .iter()
                        .find(|object| {
                            object.object_identifier.as_deref() == Some(object_id.as_str())
                        })
                        .cloned()
                } else {
                    None
                };
                let scene = Arc::make_mut(
                    runtime_shell
                        .visible_script_movement_scene
                        .as_mut()
                        .context("visible script visibility change has no retained scene")?,
                );
                if object_id == "PLAYER" {
                    scene.overworld_player_hidden = hidden;
                } else if hidden {
                    scene.visible_objects.retain(|object| {
                        object.object_identifier.as_deref() != Some(object_id.as_str())
                    });
                } else if !scene
                    .visible_objects
                    .iter()
                    .any(|object| object.object_identifier.as_deref() == Some(object_id.as_str()))
                {
                    scene.visible_objects.push(revealed_object.with_context(|| {
                        format!("visible script cannot reveal unknown object {object_id}")
                    })?);
                }
                mark_runtime_snapshot_dirty(runtime_shell);
                continue;
            }
            VisibleScriptMovementPhase::Stationary { duration, effect } => {
                if duration == 0 {
                    continue;
                }
                let initial_facing = runtime_shell
                    .visible_script_movement_scene
                    .as_ref()
                    .and_then(|scene| {
                        if object_id == "PLAYER" {
                            Some(scene.overworld.facing)
                        } else {
                            scene.visible_object_facings.get(&object_id).copied()
                        }
                    })
                    .context("stationary movement actor has no retained facing")?;
                let movement = runtime_shell.visible_script_movement.as_mut().context(
                    "visible script movement disappeared while starting stationary effect",
                )?;
                movement.hold_frames_remaining = duration;
                movement.active_jump_duration = None;
                movement.active_uses_standing_frame = !matches!(
                    effect,
                    VisibleStationaryMovementEffect::SkyfallFall
                        | VisibleStationaryMovementEffect::RockSmash
                );
                movement.active_tree_shake_duration = None;
                movement.active_stationary_effect = Some(effect);
                movement.active_stationary_duration = duration;
                movement.stationary_initial_facing = initial_facing;
                update_visible_stationary_movement_frame(runtime_shell)?;
                mark_runtime_snapshot_dirty(runtime_shell);
                return Ok(true);
            }
            VisibleScriptMovementPhase::ScreenShake { parameter } => {
                runtime_shell.visible_earthquake = Some(VisibleEarthquake::screen_shake(parameter));
                mark_runtime_snapshot_dirty(runtime_shell);
                continue;
            }
            VisibleScriptMovementPhase::Turn {
                direction,
                duration,
            } => {
                let scene = Arc::make_mut(
                    runtime_shell
                        .visible_script_movement_scene
                        .as_mut()
                        .context("visible script movement turn has no retained scene")?,
                );
                if object_id == "PLAYER" {
                    scene.overworld.facing = direction;
                } else {
                    scene.visible_object_facings.insert(object_id, direction);
                }
                let movement = runtime_shell
                    .visible_script_movement
                    .as_mut()
                    .context("visible script movement disappeared while starting turn")?;
                movement.hold_frames_remaining = u16::from(duration.max(1));
                movement.active_jump_duration = None;
                movement.active_uses_standing_frame = true;
                movement.active_tree_shake_duration = None;
                movement.active_stationary_effect = None;
                movement.active_stationary_duration = 0;
                mark_runtime_snapshot_dirty(runtime_shell);
                return Ok(true);
            }
            VisibleScriptMovementPhase::Move {
                from,
                to,
                direction,
                duration,
                jump,
                update_facing,
                standing_frame,
            } => {
                log_visible_movement_event(
                    runtime_shell,
                    "step",
                    &object_id,
                    format!(
                        "from=({}, {}) to=({}, {}) direction={direction:?} duration={duration} jump={jump}",
                        from.x, from.y, to.x, to.y,
                    ),
                );
                let leader_stride = u8::try_from(
                    (i32::from(to.x) - i32::from(from.x))
                        .unsigned_abs()
                        .max((i32::from(to.y) - i32::from(from.y)).unsigned_abs()),
                )
                .context("visible leader stride exceeds follower movement range")?;
                begin_visible_follower_step(
                    runtime_shell,
                    direction,
                    leader_stride,
                    duration,
                    jump,
                    standing_frame,
                )?;
                let scene = Arc::make_mut(
                    runtime_shell
                        .visible_script_movement_scene
                        .as_mut()
                        .context("visible script movement step has no retained scene")?,
                );
                if object_id == "PLAYER" {
                    scene.overworld.tile = to;
                    if update_facing {
                        scene.overworld.facing = direction;
                    }
                    runtime_shell.player_walk_from = Some(from);
                    runtime_shell.player_walk_total_ticks = duration;
                    runtime_shell.player_walk_frame_ticks = duration;
                    advance_player_walk_phase(runtime_shell, direction);
                } else {
                    scene
                        .visible_object_runtime_tiles
                        .insert(object_id.clone(), to);
                    if update_facing {
                        scene
                            .visible_object_facings
                            .insert(object_id.clone(), direction);
                    }
                    advance_object_walk_phase(runtime_shell, &object_id, direction);
                    runtime_shell.trainer_walk_from = Some((object_id, from));
                    runtime_shell.object_walk_total_ticks = duration;
                    runtime_shell.object_walk_frame_ticks = duration;
                    runtime_shell.object_walk_stride = !runtime_shell.object_walk_stride;
                }
                let movement = runtime_shell
                    .visible_script_movement
                    .as_mut()
                    .context("visible script movement disappeared while starting step")?;
                movement.hold_frames_remaining = 0;
                movement.active_jump_duration = jump.then_some(duration);
                movement.active_uses_standing_frame = standing_frame;
                movement.active_tree_shake_duration = None;
                movement.active_stationary_effect = None;
                movement.active_stationary_duration = 0;
                movement.stationary_y_offset = 0;
                mark_runtime_snapshot_dirty(runtime_shell);
                return Ok(true);
            }
        }
    }
}

#[cfg(not(test))]
fn log_visible_movement_event(
    _runtime_shell: &mut BevyRuntimeShell,
    _event_kind: &str,
    _object_id: &str,
    _detail: String,
) {
}

#[cfg(test)]
fn log_visible_movement_event(
    runtime_shell: &mut BevyRuntimeShell,
    event_kind: &str,
    object_id: &str,
    detail: String,
) {
    let dialogue = runtime_shell
        .visible_script_movement_scene
        .as_deref()
        .cloned()
        .or_else(|| runtime_shell.shell.snapshot().ok())
        .and_then(|snapshot| snapshot.ui.text.map(|text| text.label))
        .unwrap_or_else(|| "none".to_string());
    let script = runtime_shell
        .active_script_cursor
        .as_ref()
        .map(|cursor| format!("{}:{}", cursor.source_script, cursor.next_command_index))
        .unwrap_or_else(|| "none".to_string());
    let event = format!(
        "crystal-bevy movement frame={} event={} object={} dialogue={} script={} {}",
        runtime_shell.lcd_animation_frame, event_kind, object_id, dialogue, script, detail,
    );
    eprintln!("{event}");
    runtime_shell.movement_log_events.push_back(event);
    if runtime_shell.movement_log_events.len() > 4096 {
        runtime_shell.movement_log_events.pop_front();
    }
}

#[cfg(not(test))]
fn log_visible_key_presses(_runtime_shell: &mut BevyRuntimeShell, _keys: &ButtonInput<KeyCode>) {}

#[cfg(test)]
fn log_visible_key_presses(runtime_shell: &mut BevyRuntimeShell, keys: &ButtonInput<KeyCode>) {
    let pressed = [
        (KeyCode::ArrowUp, "UP"),
        (KeyCode::ArrowDown, "DOWN"),
        (KeyCode::ArrowLeft, "LEFT"),
        (KeyCode::ArrowRight, "RIGHT"),
        (KeyCode::KeyZ, "A"),
        (KeyCode::KeyX, "B"),
        (KeyCode::Enter, "START"),
        (KeyCode::ShiftRight, "SELECT"),
    ]
    .into_iter()
    .filter_map(|(key, name)| keys.just_pressed(key).then_some(name))
    .collect::<Vec<_>>();
    if pressed.is_empty() {
        return;
    }
    let snapshot = runtime_shell.shell.snapshot().ok();
    let map = snapshot
        .as_ref()
        .map(|snapshot| snapshot.overworld.map_name.as_str())
        .unwrap_or("unknown");
    let tile = snapshot.as_ref().map(|snapshot| snapshot.overworld.tile);
    let dialogue = snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.ui.text.as_ref().map(|text| text.label.as_str()))
        .unwrap_or("none");
    let page = runtime_shell
        .dialogue_log_identity
        .as_ref()
        .filter(|(label, _)| label == dialogue)
        .map(|(_, page_index)| page_index + 1);
    let owner = if runtime_shell.pending_name_input.is_some() {
        "name-entry"
    } else if runtime_shell.pending_name_choice.is_some() {
        "name-choice"
    } else if runtime_shell.pending_day_of_week.is_some() {
        "weekday"
    } else if snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.ui.pending_yes_no.is_some())
    {
        "yes-no"
    } else if dialogue != "none" {
        "dialogue"
    } else if runtime_shell.active_script_cursor.is_some() {
        "script"
    } else {
        "overworld"
    };
    let script = runtime_shell
        .active_script_cursor
        .as_ref()
        .map(|cursor| format!("{}:{}", cursor.source_script, cursor.next_command_index))
        .unwrap_or_else(|| "none".to_string());
    for key in pressed {
        let event = format!(
            "crystal-bevy input frame={} key={} owner={} map={} tile={:?} dialogue={} page={:?} script={}",
            runtime_shell.lcd_animation_frame, key, owner, map, tile, dialogue, page, script,
        );
        eprintln!("{event}");
        runtime_shell.input_log_events.push_back(event);
        if runtime_shell.input_log_events.len() > 4096 {
            runtime_shell.input_log_events.pop_front();
        }
    }
}

fn advance_visible_script_movement(runtime_shell: &mut BevyRuntimeShell) -> Result<bool> {
    let Some(movement) = runtime_shell.visible_script_movement.as_mut() else {
        return Ok(false);
    };
    let hold_frames_remaining = if movement.hold_frames_remaining > 0 {
        movement.hold_frames_remaining -= 1;
        Some(movement.hold_frames_remaining)
    } else {
        None
    };
    if let Some(hold_frames_remaining) = hold_frames_remaining {
        update_visible_stationary_movement_frame(runtime_shell)?;
        if hold_frames_remaining > 0 {
            mark_runtime_snapshot_dirty(runtime_shell);
            return Ok(true);
        }
    } else {
        let movement = runtime_shell
            .visible_script_movement
            .as_ref()
            .context("visible script movement disappeared while advancing step")?;
        let leader_in_flight = visible_actor_walk_in_flight(
            &movement.object_id,
            runtime_shell.player_walk_frame_ticks,
            runtime_shell.object_walk_frame_ticks,
            &runtime_shell.object_walk_frame_ticks_by_id,
        );
        let follower_in_flight = movement
            .follower_object_id
            .as_deref()
            .is_some_and(|object_id| {
                visible_actor_walk_in_flight(
                    object_id,
                    runtime_shell.player_walk_frame_ticks,
                    runtime_shell.object_walk_frame_ticks,
                    &runtime_shell.object_walk_frame_ticks_by_id,
                )
            });
        let movement_in_flight = leader_in_flight || follower_in_flight;
        if movement_in_flight {
            return Ok(true);
        }
    }
    if start_next_visible_script_movement_phase(runtime_shell)? {
        return Ok(true);
    }
    match runtime_shell.visible_field_travel_animation {
        Some(VisibleFieldTravelAnimation::DigOut) => {
            settle_visible_overworld_travel(runtime_shell)?;
            queue_visible_shell_sound_effect(runtime_shell, "SFX_WARP_FROM")?;
            runtime_shell.visible_field_travel_animation =
                Some(VisibleFieldTravelAnimation::DigReturn);
            begin_visible_dig_travel_animation(runtime_shell, true)?;
        }
        Some(VisibleFieldTravelAnimation::DigReturn) => {
            runtime_shell.visible_field_travel_animation = None;
            runtime_shell.field_notice_scene = None;
        }
        Some(VisibleFieldTravelAnimation::TeleportFrom) => {
            settle_visible_overworld_travel(runtime_shell)?;
            queue_visible_shell_sound_effect(runtime_shell, "SFX_WARP_FROM")?;
            runtime_shell.visible_field_travel_animation =
                Some(VisibleFieldTravelAnimation::TeleportTo);
            begin_visible_teleport_travel_animation(runtime_shell, true)?;
        }
        Some(VisibleFieldTravelAnimation::TeleportTo) => {
            runtime_shell.visible_field_travel_animation = None;
            runtime_shell.field_notice_scene = None;
        }
        Some(VisibleFieldTravelAnimation::Pitfall) => {
            runtime_shell.visible_field_travel_animation = None;
            settle_visible_overworld_arrival(runtime_shell, "pitfall")?;
        }
        None => advance_visible_script_until_player_boundary(runtime_shell)?,
    }
    Ok(true)
}

fn visible_actor_walk_in_flight(
    object_id: &str,
    player_walk_frame_ticks: u8,
    scripted_object_walk_frame_ticks: u8,
    object_walk_frame_ticks_by_id: &BTreeMap<String, u8>,
) -> bool {
    if object_id == "PLAYER" {
        player_walk_frame_ticks > 0
    } else {
        object_walk_frame_ticks_by_id
            .get(object_id)
            .copied()
            .unwrap_or(scripted_object_walk_frame_ticks)
            > 0
    }
}

fn visible_object_step_duration(
    object_step_durations: &BTreeMap<String, u8>,
    object_id: &str,
) -> u8 {
    object_step_durations
        .get(object_id)
        .copied()
        .unwrap_or(WALK_FRAME_HOLD_TICKS)
}

fn object_movement_spawns_strength_dust(movement: &str) -> bool {
    crate::core::world::session::object_event_uses_strength_movement(movement)
}

fn visible_strength_dust_duration(object_step_duration: u8) -> u8 {
    object_step_duration.wrapping_add(1).wrapping_mul(2)
}

#[cfg(test)]
#[test]
fn visible_object_steps_use_the_authoritative_runtime_duration() {
    let durations = BTreeMap::from([("BOULDER".to_string(), 16)]);

    assert_eq!(visible_object_step_duration(&durations, "BOULDER"), 16);
    assert_eq!(
        visible_object_step_duration(&durations, "UNTRACKED"),
        WALK_FRAME_HOLD_TICKS
    );
}

#[cfg(test)]
#[test]
fn strength_dust_uses_the_exact_strength_movement_function_members() {
    for movement in [
        "SPRITEMOVEDATA_STRENGTH_BOULDER",
        "SPRITEMOVEDATA_BIGDOLLASYM",
        "SPRITEMOVEDATA_BIGDOLL",
    ] {
        assert!(
            object_movement_spawns_strength_dust(movement),
            "{movement} calls MovementFunction_Strength"
        );
    }
    assert!(!object_movement_spawns_strength_dust(
        "SPRITEMOVEDATA_BIGDOLLSYM"
    ));
}

#[cfg(test)]
#[test]
fn strength_dust_duration_uses_the_source_tracking_object_formula() {
    assert_eq!(visible_strength_dust_duration(16), 34);
}

fn update_visible_stationary_movement_frame(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let Some(movement) = runtime_shell.visible_script_movement.as_ref() else {
        return Ok(());
    };
    let Some(effect) = movement.active_stationary_effect else {
        return Ok(());
    };
    let elapsed = movement.active_stationary_duration.saturating_sub(
        movement
            .hold_frames_remaining
            .min(movement.active_stationary_duration),
    );
    let stationary_y_offset = match effect {
        VisibleStationaryMovementEffect::TeleportRise => {
            // StepFunction_TeleportFrom's InitSpinRise falls through into
            // DoSpinRise. Its first rendered rise sample is therefore
            // Sine($11, $60) - $60 == -1, not another frame at ground level.
            const OFFSETS: [i16; 17] = [
                -1, -2, -5, -8, -12, -17, -22, -29, -36, -43, -51, -60, -69, -78, -87, -96, -96,
            ];
            OFFSETS[usize::from(elapsed.min(16))]
        }
        VisibleStationaryMovementEffect::TeleportWait => -96,
        VisibleStationaryMovementEffect::TeleportDescent => {
            const OFFSETS: [i16; 17] = [
                -96, -87, -78, -69, -60, -51, -43, -36, -29, -22, -17, -12, -8, -5, -2, -1, 0,
            ];
            OFFSETS[usize::from(elapsed.min(16))]
        }
        VisibleStationaryMovementEffect::TeleportSpin => 0,
        // StepFunction_Skyfall's setup phase does not write
        // OBJECT_SPRITE_Y_OFFSET. Scripted collapses inherit +$60 from a
        // preceding skyfall_top, while an ordinary PIT tile inherits zero.
        VisibleStationaryMovementEffect::SkyfallWait => {
            runtime_shell.visible_player_sprite_y_offset
        }
        VisibleStationaryMovementEffect::SkyfallFall => {
            // StepFunction_Skyfall's setup countdown falls through into
            // .Fall when it reaches zero. That same object tick increments
            // OBJECT_JUMP_HEIGHT to one and writes Sine(1, $60) - $60;
            // there is no extra frame at the inherited $60 offset between
            // the setup and fall phases.
            const OFFSETS: [i16; 17] = [
                -87, -78, -69, -60, -51, -43, -36, -29, -22, -17, -12, -8, -5, -2, -1, 0, 0,
            ];
            OFFSETS[usize::from(elapsed.min(16))]
        }
        VisibleStationaryMovementEffect::SkyfallTop if elapsed >= 16 => 96,
        VisibleStationaryMovementEffect::SkyfallTop => 0,
        VisibleStationaryMovementEffect::DigSpin => 0,
        VisibleStationaryMovementEffect::RockSmash => 0,
    };
    let frames_per_facing = if effect == VisibleStationaryMovementEffect::SkyfallTop {
        2
    } else {
        4
    };
    let facing_index = if effect == VisibleStationaryMovementEffect::DigSpin {
        let initial = match movement.stationary_initial_facing {
            Direction::Down => 0,
            Direction::Right => 1,
            Direction::Up => 2,
            Direction::Left => 3,
        };
        (initial + elapsed / frames_per_facing) % 4
    } else {
        (elapsed / frames_per_facing) % 4
    };
    let facing = match facing_index {
        0 => Direction::Down,
        1 => Direction::Right,
        2 => Direction::Up,
        _ => Direction::Left,
    };
    let object_id = movement.object_id.clone();
    runtime_shell
        .visible_script_movement
        .as_mut()
        .context("stationary movement disappeared while storing sprite offset")?
        .stationary_y_offset = stationary_y_offset;
    if matches!(
        effect,
        VisibleStationaryMovementEffect::TeleportWait
            | VisibleStationaryMovementEffect::SkyfallWait
            | VisibleStationaryMovementEffect::SkyfallFall
            | VisibleStationaryMovementEffect::SkyfallTop
            | VisibleStationaryMovementEffect::RockSmash
    ) {
        return Ok(());
    }
    let scene = Arc::make_mut(
        runtime_shell
            .visible_script_movement_scene
            .as_mut()
            .context("stationary movement effect has no retained scene")?,
    );
    if object_id == "PLAYER" {
        scene.overworld.facing = facing;
    } else {
        scene.visible_object_facings.insert(object_id, facing);
    }
    Ok(())
}

fn collect_overworld_keyboard_buttons(
    keys: &ButtonInput<KeyCode>,
    shell_consumes_direction: bool,
    shell_consumes_a: bool,
    shell_consumes_b: bool,
    shell_consumes_start: bool,
    shell_consumes_select: bool,
) -> Vec<GameButton> {
    let mut buttons = Vec::new();
    for (key, button, shell_consumes_button) in [
        (KeyCode::ArrowUp, GameButton::Up, shell_consumes_direction),
        (
            KeyCode::ArrowDown,
            GameButton::Down,
            shell_consumes_direction,
        ),
        (
            KeyCode::ArrowLeft,
            GameButton::Left,
            shell_consumes_direction,
        ),
        (
            KeyCode::ArrowRight,
            GameButton::Right,
            shell_consumes_direction,
        ),
        (KeyCode::KeyZ, GameButton::A, shell_consumes_a),
        (KeyCode::KeyX, GameButton::B, shell_consumes_b),
        (KeyCode::Enter, GameButton::Start, shell_consumes_start),
        (
            KeyCode::ShiftRight,
            GameButton::Select,
            shell_consumes_select,
        ),
    ] {
        if keys.pressed(key) && !shell_consumes_button {
            buttons.push(button);
        }
    }
    buttons
}

fn is_direction_button(button: GameButton) -> bool {
    matches!(
        button,
        GameButton::Up | GameButton::Down | GameButton::Left | GameButton::Right
    )
}

fn game_button_direction(button: GameButton) -> Option<Direction> {
    match button {
        GameButton::Up => Some(Direction::Up),
        GameButton::Down => Some(Direction::Down),
        GameButton::Left => Some(Direction::Left),
        GameButton::Right => Some(Direction::Right),
        _ => None,
    }
}

fn just_pressed_overworld_direction(keys: &ButtonInput<KeyCode>) -> Option<GameButton> {
    [
        (KeyCode::ArrowUp, GameButton::Up),
        (KeyCode::ArrowDown, GameButton::Down),
        (KeyCode::ArrowLeft, GameButton::Left),
        (KeyCode::ArrowRight, GameButton::Right),
    ]
    .into_iter()
    .find_map(|(key, direction)| keys.just_pressed(key).then_some(direction))
}

fn sync_overworld_held_directions(
    keys: &ButtonInput<KeyCode>,
    runtime_shell: &mut BevyRuntimeShell,
    shell_consumes_direction: bool,
) {
    if shell_consumes_direction {
        runtime_shell.overworld_held_directions.clear();
        runtime_shell.overworld_held_direction = None;
        runtime_shell.overworld_buffered_direction = None;
        return;
    }
    for (key, direction) in [
        (KeyCode::ArrowUp, GameButton::Up),
        (KeyCode::ArrowDown, GameButton::Down),
        (KeyCode::ArrowLeft, GameButton::Left),
        (KeyCode::ArrowRight, GameButton::Right),
    ] {
        if !keys.pressed(key) {
            runtime_shell
                .overworld_held_directions
                .retain(|held| *held != direction);
        }
        // Host frames can clear the press edge before a simulation tick
        // samples it. Recover a still-held direction without reordering
        // directions already tracked (the latest deliberate press wins).
        if keys.pressed(key)
            && (keys.just_pressed(key) || !runtime_shell.overworld_held_directions.contains(&direction))
        {
            runtime_shell
                .overworld_held_directions
                .retain(|held| *held != direction);
            runtime_shell.overworld_held_directions.push_back(direction);
        }
    }
}

/// The core advances movement at tile granularity.  Preserve the Game Boy's
/// visible walking pace by forwarding a newly held direction immediately and
/// then at eight-frame intervals. Direction arbitration has already reduced
/// the physical keyboard state to the single most recently pressed held
/// direction before this cadence gate.
fn throttle_held_overworld_direction(
    runtime_shell: &mut BevyRuntimeShell,
    buttons: &mut Vec<GameButton>,
) {
    let directions = buttons
        .iter()
        .copied()
        .filter(|button| is_direction_button(*button))
        .collect::<Vec<_>>();
    if directions.len() != 1 {
        runtime_shell.overworld_direction_repeat_ticks = 0;
        runtime_shell.overworld_held_direction = None;
        return;
    }
    let direction = directions[0];
    if runtime_shell.overworld_held_direction == Some(direction)
        && runtime_shell.overworld_direction_repeat_ticks > 0
    {
        runtime_shell.overworld_direction_repeat_ticks -= 1;
        buttons.retain(|button| !is_direction_button(*button));
        return;
    }
    runtime_shell.overworld_held_direction = Some(direction);
    runtime_shell.overworld_direction_repeat_ticks = OVERWORLD_STEP_REPEAT_TICKS - 1;
}

fn apply_runtime_hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    mut timer: ResMut<RuntimeTickTimer>,
    mut runtime_shell: ResMut<BevyRuntimeShell>,
) {
    if runtime_shell.last_error.as_deref() == Some(SERVER_CLOCK_UNAVAILABLE) {
        return;
    }
    let elapsed_input_ticks = timer.take_presentation_ticks();
    let alt_pressed = keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight);
    let ctrl_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    // The frame system above advances these effects and returns before the
    // authoritative overworld can consume joypad input. Give the visible
    // animation the same exclusive ownership here as well, otherwise START,
    // SELECT, or A/B can open a shell surface over Cut, Whirlpool, Headbutt,
    // fishing, or field travel even though Crystal's animation loop is still
    // holding the joypad. Field-travel and Cut-family state is installed
    // before its use text closes, so keep that preceding textbox interactive
    // until it is gone.
    if visible_noninteractive_field_animation_owns_input(&runtime_shell) {
        runtime_shell.ui_held_direction = None;
        runtime_shell.ui_direction_repeat_ticks = 0;
        return;
    }
    if runtime_shell.pending_field_notice_effect_frames.is_some()
        && runtime_shell.field_notice.is_none()
    {
        return;
    }
    if runtime_shell.visible_fly_animation.is_some()
        || (runtime_shell.visible_waterfall_animation.is_some()
            && runtime_shell.field_notice.is_none())
    {
        return;
    }
    if runtime_shell.pending_time_set.is_some() {
        apply_visible_time_set_input_keys(&keys, &mut runtime_shell, elapsed_input_ticks);
        if keys.just_pressed(KeyCode::KeyX) {
            run_bevy_action(&mut runtime_shell, press_visible_time_set_b_button);
        }
        // A Bevy update can contain multiple Game Boy frames (for example
        // while the renderer is busy compiling a screen texture).  Advancing
        // only once per update stretches every timed transition, most visibly
        // the 8-frame ASM palette fades.  Consume every elapsed tick so the
        // animation remains tied to GB time rather than host FPS.
        for _ in 0..elapsed_input_ticks {
            if let Err(error) = tick_visible_time_set_screen(&mut runtime_shell) {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
                break;
            }
        }
        return;
    }
    if runtime_shell.pending_oak_intro.is_some() {
        if keys.just_pressed(KeyCode::KeyZ) {
            run_bevy_action(&mut runtime_shell, press_visible_oak_intro_a_button);
        }
        if keys.just_pressed(KeyCode::KeyX) {
            run_bevy_action(&mut runtime_shell, press_visible_oak_intro_b_button);
        }
        for _ in 0..elapsed_input_ticks {
            if let Err(error) = tick_visible_oak_intro(&mut runtime_shell) {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
                break;
            }
        }
        return;
    }
    if runtime_shell.pending_gender_selection.is_some() {
        if keys.just_pressed(KeyCode::ArrowUp) {
            run_bevy_action(&mut runtime_shell, |shell| {
                move_visible_gender_selection(shell, -1)
            });
        }
        if keys.just_pressed(KeyCode::ArrowDown) {
            run_bevy_action(&mut runtime_shell, |shell| {
                move_visible_gender_selection(shell, 1)
            });
        }
        if keys.just_pressed(KeyCode::KeyZ) {
            run_bevy_action(&mut runtime_shell, confirm_visible_gender_selection);
        }
        for _ in 0..elapsed_input_ticks {
            if let Err(error) = tick_visible_gender_selection(&mut runtime_shell) {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
                break;
            }
        }
        return;
    }
    if runtime_shell.pending_name_choice.is_some() {
        if runtime_shell.pending_name_choice.as_ref()
            .is_some_and(|choice| choice.nickname_pages.len() > 1)
        {
            if keys.just_pressed(KeyCode::KeyZ) || keys.just_pressed(KeyCode::KeyX) {
                run_bevy_action(&mut runtime_shell, |shell| {
                    advance_visible_nickname_prompt(shell);
                    Ok(())
                });
            }
            return;
        }
        if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::ArrowLeft) {
            run_bevy_action(&mut runtime_shell, |shell| {
                move_visible_name_choice(shell, -1)
            });
        }
        if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::ArrowRight) {
            run_bevy_action(&mut runtime_shell, |shell| {
                move_visible_name_choice(shell, 1)
            });
        }
        if keys.just_pressed(KeyCode::KeyZ) {
            run_bevy_action(&mut runtime_shell, confirm_visible_name_choice);
        }
        if keys.just_pressed(KeyCode::KeyX) && runtime_shell.pending_standard_capture.is_some() {
            run_bevy_action(&mut runtime_shell, |shell| {
                shell.pending_name_choice = None;
                finish_visible_capture_nickname(shell, None)
            });
        }
        if keys.just_pressed(KeyCode::KeyX) && runtime_shell.pending_egg_hatch_nickname.is_some() {
            run_bevy_action(&mut runtime_shell, |shell| {
                shell.pending_name_choice = None;
                finish_visible_egg_hatch_nickname(shell, None)
            });
        }
        if keys.just_pressed(KeyCode::KeyX) && runtime_shell.pending_gift_pokemon_nickname.is_some()
        {
            run_bevy_action(&mut runtime_shell, |shell| {
                shell.pending_name_choice = None;
                finish_visible_gift_pokemon_nickname(shell, None)
            });
        }
        for _ in 0..elapsed_input_ticks {
            if let Err(error) = tick_visible_player_name_choice(&mut runtime_shell) {
                record_visible_runtime_error(&mut runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
                break;
            }
        }
        return;
    }
    if runtime_shell.pending_mail_read.is_some() {
        sample_visible_pc_text_joypad_history(&keys, &mut runtime_shell);
        if keys.just_pressed(KeyCode::KeyZ) || keys.just_pressed(KeyCode::KeyX) {
            run_bevy_action(&mut runtime_shell, close_visible_mail_read);
        }
        return;
    }
    if runtime_shell.pending_mail_input.is_some() {
        apply_visible_mail_input_keys(&keys, &mut runtime_shell);
        return;
    }
    if runtime_shell.pending_name_input.is_some() {
        apply_visible_name_input_keys(&keys, &mut runtime_shell);
        return;
    }
    if runtime_shell.intro_screen.is_some() {
        if !alt_pressed && !ctrl_pressed {
            if keys.just_pressed(KeyCode::KeyZ) {
                run_bevy_action(&mut runtime_shell, |shell| {
                    skip_visible_intro_screen(shell, GameButton::A)
                });
            }
            if keys.just_pressed(KeyCode::Enter) {
                run_bevy_action(&mut runtime_shell, |shell| {
                    skip_visible_intro_screen(shell, GameButton::Start)
                });
            }
            if keys.just_pressed(KeyCode::KeyX) {
                run_bevy_action(&mut runtime_shell, |shell| {
                    skip_visible_intro_screen(shell, GameButton::B)
                });
            }
        }
        return;
    }
    if runtime_shell.credits_screen.is_some() {
        if !alt_pressed && !ctrl_pressed {
            if keys.just_pressed(KeyCode::KeyZ) {
                run_bevy_action(&mut runtime_shell, press_visible_credits_a_button);
            }
            if keys.just_pressed(KeyCode::KeyX) {
                run_bevy_action(&mut runtime_shell, press_visible_credits_b_button);
            }
        }
        return;
    }
    if runtime_shell.pending_delete_save.is_some() {
        if !alt_pressed && !ctrl_pressed {
            if keys.just_pressed(KeyCode::ArrowUp)
                || keys.just_pressed(KeyCode::ArrowDown)
                || keys.just_pressed(KeyCode::ArrowLeft)
                || keys.just_pressed(KeyCode::ArrowRight)
            {
                run_bevy_action(&mut runtime_shell, move_visible_delete_save_cursor);
            }
            if keys.just_pressed(KeyCode::KeyZ) {
                run_bevy_action(&mut runtime_shell, confirm_visible_delete_save_screen);
            }
            if keys.just_pressed(KeyCode::KeyX) {
                run_bevy_action(&mut runtime_shell, |shell| {
                    close_visible_delete_save_screen(shell, "cancel")
                });
            }
        }
        return;
    }
    if runtime_shell.pending_clock_reset.is_some() {
        if !alt_pressed && !ctrl_pressed {
            if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::ArrowRight) {
                run_bevy_action(&mut runtime_shell, |shell| {
                    move_visible_clock_reset_cursor(shell, 1)
                });
            }
            if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::ArrowLeft) {
                run_bevy_action(&mut runtime_shell, |shell| {
                    move_visible_clock_reset_cursor(shell, -1)
                });
            }
            if keys.just_pressed(KeyCode::KeyZ) {
                run_bevy_action(&mut runtime_shell, confirm_visible_clock_reset_screen);
            }
            if keys.just_pressed(KeyCode::KeyX) {
                run_bevy_action(&mut runtime_shell, |shell| {
                    close_visible_clock_reset_screen(shell, "cancel")
                });
            }
        }
        return;
    }
    if runtime_shell.pending_mystery_gift.is_some() {
        if !alt_pressed && !ctrl_pressed {
            if keys.just_pressed(KeyCode::KeyZ) {
                run_bevy_action(&mut runtime_shell, press_visible_mystery_gift_a_button);
            }
            if keys.just_pressed(KeyCode::KeyX) {
                run_bevy_action(&mut runtime_shell, |shell| {
                    close_visible_mystery_gift_screen(shell, "cancel")
                });
            }
        }
        return;
    }
    if runtime_shell.options_menu_open {
        let held = [
            (KeyCode::ArrowUp, GameButton::Up),
            (KeyCode::ArrowDown, GameButton::Down),
            (KeyCode::ArrowLeft, GameButton::Left),
            (KeyCode::ArrowRight, GameButton::Right),
        ]
        .into_iter()
        .filter_map(|(key, direction)| keys.pressed(key).then_some((key, direction)))
        .collect::<Vec<_>>();
        if held.len() == 1 {
            let (key, direction) = held[0];
            let newly_pressed = keys.just_pressed(key);
            let changed_direction = runtime_shell.ui_held_direction != Some(direction);
            let repeated = !newly_pressed
                && runtime_shell.ui_held_direction == Some(direction)
                && elapsed_input_ticks > 0
                && runtime_shell.ui_direction_repeat_ticks == 0;
            if newly_pressed || changed_direction || repeated {
                dispatch_visible_options_direction(&mut runtime_shell, direction);
                runtime_shell.ui_held_direction = Some(direction);
                runtime_shell.ui_direction_repeat_ticks = if newly_pressed { 15 } else { 4 };
            } else if runtime_shell.ui_held_direction == Some(direction) && elapsed_input_ticks > 0
            {
                runtime_shell.ui_direction_repeat_ticks =
                    runtime_shell.ui_direction_repeat_ticks.saturating_sub(1);
            }
        } else {
            runtime_shell.ui_held_direction = None;
            runtime_shell.ui_direction_repeat_ticks = 0;
        }
        if keys.just_pressed(KeyCode::KeyZ) {
            runtime_shell.ui_held_direction = None;
            runtime_shell.ui_direction_repeat_ticks = 0;
            run_bevy_action(&mut runtime_shell, press_visible_a_button);
        }
        if keys.just_pressed(KeyCode::KeyX) || keys.just_pressed(KeyCode::Enter) {
            runtime_shell.ui_held_direction = None;
            runtime_shell.ui_direction_repeat_ticks = 0;
            run_bevy_action(&mut runtime_shell, press_visible_b_button);
        }
        return;
    }
    if runtime_shell.trainer_card_open {
        apply_visible_runtime_controls(&keys, &mut runtime_shell, elapsed_input_ticks > 0);
        if runtime_shell.trainer_card_open {
            for _ in 0..elapsed_input_ticks {
                runtime_shell.trainer_card_colon_ticks += 1;
                if runtime_shell.trainer_card_colon_ticks == 32 {
                    runtime_shell.trainer_card_colon_ticks = 0;
                    runtime_shell.trainer_card_colon_visible =
                        !runtime_shell.trainer_card_colon_visible;
                    if runtime_shell.trainer_card_page == VisibleTrainerCardPage::Info {
                        mark_runtime_snapshot_dirty(&mut runtime_shell);
                    }
                }
                if runtime_shell.trainer_card_page == VisibleTrainerCardPage::JohtoBadges {
                    runtime_shell.trainer_card_badge_ticks =
                        runtime_shell.trainer_card_badge_ticks.wrapping_add(1);
                    if runtime_shell.trainer_card_badge_ticks & 0x07 == 0 {
                        runtime_shell.trainer_card_badge_frame =
                            (runtime_shell.trainer_card_badge_frame + 1) & 0x07;
                        mark_runtime_snapshot_dirty(&mut runtime_shell);
                    }
                }
            }
        }
        return;
    }
    if runtime_shell.title_menu.is_some() {
        let Some(title) = runtime_shell.title_menu.as_mut() else {
            return;
        };
        if !matches!(title.source_phase(), VisibleTitlePhase::MainMenu) {
            return;
        }
        if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::ArrowDown) {
            let delta = if keys.just_pressed(KeyCode::ArrowUp) {
                -1
            } else {
                1
            };
            let input = if delta < 0 {
                GameButton::Up
            } else {
                GameButton::Down
            };
            match press_visible_title_direction_button(&mut runtime_shell, input, delta) {
                Ok(()) => runtime_shell.last_error = None,
                Err(error) => {
                    record_visible_runtime_error(&mut runtime_shell, &error);
                    runtime_shell.last_error = Some(error.to_string());
                }
            }
        }
        if keys.just_pressed(KeyCode::KeyZ) {
            let input = GameButton::A;
            match press_visible_title_confirm_button(&mut runtime_shell, input) {
                Ok(()) => runtime_shell.last_error = None,
                Err(error) => {
                    record_visible_runtime_error(&mut runtime_shell, &error);
                    runtime_shell.last_error = Some(error.to_string());
                }
            }
        }
        if keys.just_pressed(KeyCode::KeyX) {
            match press_visible_title_cancel_button(&mut runtime_shell) {
                Ok(()) => runtime_shell.last_error = None,
                Err(error) => {
                    record_visible_runtime_error(&mut runtime_shell, &error);
                    runtime_shell.last_error = Some(error.to_string());
                }
            }
        }
        return;
    }
    apply_visible_runtime_controls(&keys, &mut runtime_shell, elapsed_input_ticks > 0);
}

fn dispatch_visible_options_direction(runtime_shell: &mut BevyRuntimeShell, direction: GameButton) {
    // Options redraws its cursor and current value after each input in ASM.
    // Use the action boundary to invalidate both the cached settings and the
    // render revision, without resuming the script beneath this modal menu.
    run_bevy_nonadvancing_action(runtime_shell, |shell| match direction {
        GameButton::Up => move_visible_options_cursor(shell, -1),
        GameButton::Down => move_visible_options_cursor(shell, 1),
        GameButton::Left => change_visible_options_selection(shell, -1),
        GameButton::Right => change_visible_options_selection(shell, 1),
        _ => Ok(()),
    });
}

fn drain_unused_runtime_ticks(mut timer: ResMut<RuntimeTickTimer>) {
    // Title/intro/credits and ordinary gameplay advance on their own clocks.
    // Do not carry their keyboard-timer ticks into a later modal screen and
    // accidentally fast-forward its fade or text animation.
    timer.take_ticks();
    timer.take_presentation_ticks();
}

fn sync_visible_earthquake_camera(
    runtime_shell: Res<BevyRuntimeShell>,
    mut cameras: Query<&mut Transform, With<MainCameraMarker>>,
) {
    let Ok(mut transform) = cameras.get_single_mut() else {
        return;
    };
    let earthquake_offset = visible_earthquake_camera_offset(runtime_shell.visible_earthquake);
    let battle_offset =
        visible_move_screen_shake_offset(runtime_shell.visible_move_animations.front());
    let (x, y) = (
        earthquake_offset.0 + battle_offset.0,
        earthquake_offset.1 + battle_offset.1,
    );
    transform.translation.x = x;
    transform.translation.y = y;
}

fn visible_earthquake_camera_offset(
    earthquake: Option<VisibleEarthquake>,
) -> (f32, f32) {
    earthquake
        .filter(|earthquake| earthquake.shake_frames_remaining > 0)
        .map(|earthquake| {
            // MovementFunction_ScreenShake initializes the counter, then its
            // step function decrements before drawing. Each later update
            // first removes the prior vector and derives the next sign from
            // the remaining counter. Counter zero restores the baseline and
            // deletes the temporary object before OAM is built.
            let counter = earthquake.shake_frames_remaining.saturating_sub(1);
            if counter == 0 {
                return (0.0, 0.0);
            }
            let distance = f32::from(earthquake.intensity.max(1)) * 4.0;
            // An even counter adds +intensity to SCY and an odd counter adds
            // -intensity. Bevy's world Y axis points upward, so SCY's screen
            // displacement projects with the opposite sign.
            let bevy_y = if counter & 1 == 0 {
                -distance
            } else {
                distance
            };
            (0.0, bevy_y)
        })
        .unwrap_or((0.0, 0.0))
}

fn visible_move_screen_shake_offset(animation: Option<&VisibleMoveAnimation>) -> (f32, f32) {
    let Some(animation) = animation.filter(|animation| animation.started) else {
        return (0.0, 0.0);
    };
    let mut offset = (0.0, 0.0);
    for effect in animation
        .bg_events
        .iter()
        .filter(|effect| !effect.incremented && effect.frame <= animation.frame)
    {
        if animation.bg_events.iter().any(|candidate| {
            !candidate.incremented
                && candidate.effect_id == effect.effect_id
                && candidate.frame > effect.frame
                && candidate.frame <= animation.frame
        }) {
            continue;
        }
        if effect.effect_id == "BATTLE_BG_EFFECT_WOBBLE_SCREEN" {
            let reset_frame = animation
                .bg_events
                .iter()
                .filter(|candidate| {
                    candidate.incremented
                        && candidate.effect_id == effect.effect_id
                        && candidate.frame >= effect.frame
                        && candidate.frame <= animation.frame
                })
                .map(|candidate| candidate.frame)
                .max()
                .unwrap_or(effect.frame);
            let active_age = animation.frame.saturating_sub(reset_frame);
            let frequency = if effect.duration == 0 {
                4
            } else {
                effect.duration
            };
            let lifetime = if effect.duration == 0 {
                frequency.saturating_mul(2)
            } else {
                effect.duration
            };
            if active_age < lifetime.max(1) {
                let phase_age = animation.frame.saturating_sub(effect.frame);
                let amplitude = if effect.param == 0 { 3 } else { effect.param };
                let value = (f64::from(amplitude)
                    * (f64::from(phase_age) * std::f64::consts::PI / f64::from(frequency.max(1)))
                        .sin())
                .round() as f32
                    * (TILE_SIZE / SOURCE_TILE_SIZE as f32);
                offset.0 += value;
            }
            continue;
        }
        let axis_x = match effect.effect_id.as_str() {
            "BATTLE_BG_EFFECT_SHAKE_SCREEN_X" => true,
            "BATTLE_BG_EFFECT_SHAKE_SCREEN_Y" => false,
            _ => continue,
        };
        let age = animation.frame.saturating_sub(effect.frame);
        if age >= effect.duration.max(1) {
            continue;
        }
        let amplitude = parse_visible_battle_animation_int(&effect.target)
            .filter(|value| *value > 0)
            .unwrap_or_else(|| {
                let encoded = effect.param >> 4;
                i32::from(if encoded == 0 { 4 } else { encoded })
            });
        let encoded_frequency = effect.param & 0x0f;
        let frequency = u16::from(if encoded_frequency == 0 {
            2
        } else {
            encoded_frequency
        });
        let signed = if (age / frequency) % 2 == 0 {
            amplitude
        } else {
            -amplitude
        } as f32
            * (TILE_SIZE / SOURCE_TILE_SIZE as f32);
        if axis_x {
            offset.0 += signed;
        } else {
            offset.1 += signed;
        }
    }
    offset
}

fn apply_visible_runtime_controls(
    keys: &ButtonInput<KeyCode>,
    runtime_shell: &mut BevyRuntimeShell,
    advance_repeat: bool,
) {
    if runtime_shell.visible_script_movement.is_some()
        || runtime_shell.incoming_phone_sequence.is_some()
        || visible_noninteractive_field_animation_owns_input(runtime_shell)
        || visible_noninteractive_battle_animation_owns_input(runtime_shell)
    {
        runtime_shell.ui_held_direction = None;
        runtime_shell.ui_direction_repeat_ticks = 0;
        return;
    }
    if advance_repeat && runtime_shell.pokegear_phone_call.as_ref()
        .is_some_and(|call| call.phase == VisiblePokegearPhoneCallPhase::Calling)
        && runtime_shell.visible_special_text_pause_frames.is_none()
        && !runtime_shell.visible_wait_sfx_boundary
        && runtime_shell.shell.session().state().script_runtime.pending_delays.is_empty()
    {
        // PrintLetterDelay and the conversation prompts call GetJoypad. Keep
        // their mirrors for FinishPhoneCall's later JoyTextDelay; GetJoypad
        // itself does not restart the menu's 15/5-frame repeat counter.
        runtime_shell.pokegear_joypad.get_joypad(visible_menu_physical_down(keys));
    }
    if apply_visible_pokegear_card_controls(keys, runtime_shell, advance_repeat) {
        return;
    }
    // The generic text owner can consume fresh A/B even without advancing
    // menu repeat ticks, so its GetJoypad mirrors must be updated there too.
    sample_visible_pc_text_joypad_history(keys, runtime_shell);
    let pc_item_list_active = runtime_shell.pc_item_cursor.is_some()
        && runtime_shell.pc_item_quantity.is_none() && runtime_shell.pc_notice.is_none();
    let pc_item_quantity_active = runtime_shell.pc_item_quantity.is_some();
    let mailbox_menu_active = runtime_shell.mailbox_cursor.is_some()
        && runtime_shell.pc_notice.is_none() && runtime_shell.pc_confirmation.is_none()
        && runtime_shell.pending_mail_read.is_none() && !runtime_shell.party_menu_open;
    let mailbox_confirmation_active = matches!(runtime_shell.pc_confirmation,
        Some(VisiblePcConfirmation::PutMailInPack(_)));
    let pc_a_consumed = if pc_item_list_active || pc_item_quantity_active || mailbox_menu_active || mailbox_confirmation_active {
        std::mem::take(&mut runtime_shell.overworld_interaction_consumed_a)
            || runtime_shell.field_text_consumed_a
    } else { false };
    if pc_item_list_active || pc_item_quantity_active || mailbox_menu_active || mailbox_confirmation_active {
        if !advance_repeat { return; }
        // Live updates have already applied VBlank; direct control callers
        // represent a single source frame, as for the Pokégear sampler.
        if !runtime_shell.pc_joypad_vblank_prepared { advance_visible_pc_input_vblanks(runtime_shell, 1); }
        let down = visible_menu_physical_down(keys);
        if mailbox_confirmation_active {
            apply_visible_mailbox_confirmation_controls(runtime_shell, down, pc_a_consumed);
            return;
        }
        if pc_item_quantity_active {
            apply_visible_pc_quantity_controls(runtime_shell, down, pc_a_consumed);
            return;
        }
        if mailbox_menu_active && runtime_shell.mailbox_action_cursor.is_some() {
            apply_visible_mailbox_action_controls(runtime_shell, down, pc_a_consumed);
        } else {
            apply_visible_pc_scrolling_list_controls(runtime_shell, down, pc_a_consumed);
        }
        return;
    }
    let shift_pressed = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let alt_pressed = keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight);
    let ctrl_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let plain_input = !alt_pressed && !ctrl_pressed && !shift_pressed;
    if runtime_shell.party_menu_open && runtime_shell.party_summary_open {
        // StatsScreen_GetJoypad reads hJoyPressed, not JoyTextDelay. One
        // fresh sample takes exactly one branch of StatsScreen_JoypadAction;
        // a held arrow must not repeat or combine a page change with A.
        runtime_shell.ui_held_direction = None;
        runtime_shell.ui_direction_repeat_ticks = 0;
        if !plain_input { return; }
        let egg = runtime_shell.shell.session().state().storage.party.pokemon
            .get(runtime_shell.party_cursor)
            .and_then(Option::as_ref)
            .is_some_and(|pokemon| pokemon.is_egg);
        let key = if egg {
            // EggStatsJoypad handles A first, then masks out Left/Right.
            [KeyCode::KeyZ, KeyCode::KeyX, KeyCode::ArrowUp, KeyCode::ArrowDown]
                .into_iter().find(|key| keys.just_pressed(*key))
        } else {
            [KeyCode::KeyX, KeyCode::ArrowLeft, KeyCode::ArrowRight,
             KeyCode::KeyZ, KeyCode::ArrowUp, KeyCode::ArrowDown]
                .into_iter().find(|key| keys.just_pressed(*key))
        };
        match key {
            Some(KeyCode::KeyX) if !runtime_shell.field_text_consumed_b =>
                run_bevy_action(runtime_shell, press_visible_b_button),
            Some(KeyCode::KeyZ) if !runtime_shell.field_text_consumed_a =>
                run_bevy_action(runtime_shell, press_visible_a_button),
            Some(KeyCode::ArrowLeft) => run_bevy_nonadvancing_action(runtime_shell,
                |shell| cycle_visible_party_summary_page(shell, -1)),
            Some(KeyCode::ArrowRight) => run_bevy_nonadvancing_action(runtime_shell,
                |shell| cycle_visible_party_summary_page(shell, 1)),
            Some(KeyCode::ArrowUp) => run_bevy_nonadvancing_action(runtime_shell,
                |shell| move_visible_party_summary_pokemon(shell, -1)),
            Some(KeyCode::ArrowDown) => run_bevy_nonadvancing_action(runtime_shell,
                |shell| move_visible_party_summary_pokemon(shell, 1)),
            _ => {}
        }
        return;
    }
    if keys.just_pressed(KeyCode::KeyZ)
        || keys.just_pressed(KeyCode::KeyX)
        || keys.just_pressed(KeyCode::Enter)
        || keys.just_pressed(KeyCode::ShiftRight)
    {
        // Confirm/cancel commonly replaces the active cursor surface. Each
        // TypeScript menu owns its own repeat state, so a held direction must
        // begin a fresh initial delay after that boundary.
        runtime_shell.ui_held_direction = None;
        runtime_shell.ui_direction_repeat_ticks = 0;
    }
    if plain_input && has_visible_shell_direction_action(runtime_shell) {
        let direction_order = [
            (KeyCode::ArrowUp, GameButton::Up), (KeyCode::ArrowDown, GameButton::Down),
            (KeyCode::ArrowLeft, GameButton::Left), (KeyCode::ArrowRight, GameButton::Right),
        ];
        let held_directions = direction_order.into_iter()
            .filter_map(|(key, direction)| keys.pressed(key).then_some((key, direction)))
            .collect::<Vec<_>>();
        let newly_pressed_direction = held_directions.iter()
            .find_map(|(key, direction)| keys.just_pressed(*key).then_some(*direction));
        let active_held_direction = runtime_shell.ui_held_direction.filter(|active| {
            held_directions
                .iter()
                .any(|(_, direction)| direction == active)
        });
        if let Some(direction) = newly_pressed_direction {
            dispatch_visible_ui_direction(runtime_shell, direction);
            runtime_shell.ui_held_direction = Some(direction);
            runtime_shell.ui_direction_repeat_ticks =
                visible_ui_initial_repeat_ticks(runtime_shell);
        } else if let Some(direction) = active_held_direction {
            let repeated = advance_repeat
                && runtime_shell.ui_direction_repeat_ticks == 0
                && visible_ui_direction_can_repeat(runtime_shell);
            if repeated {
                dispatch_visible_ui_direction(runtime_shell, direction);
                runtime_shell.ui_direction_repeat_ticks = 4;
            } else if advance_repeat {
                runtime_shell.ui_direction_repeat_ticks =
                    runtime_shell.ui_direction_repeat_ticks.saturating_sub(1);
            }
        } else if let Some((_, direction)) = held_directions.first().copied() {
            // A surface replacement may occur while a direction remains held.
            // Adopt it without manufacturing a new press, then give the new
            // menu its complete initial repeat delay.
            runtime_shell.ui_held_direction = Some(direction);
            runtime_shell.ui_direction_repeat_ticks =
                visible_ui_initial_repeat_ticks(runtime_shell);
        } else {
            runtime_shell.ui_held_direction = None;
            runtime_shell.ui_direction_repeat_ticks = 0;
        }
    } else {
        runtime_shell.ui_held_direction = None;
        runtime_shell.ui_direction_repeat_ticks = 0;
    }
    let overworld_interaction_consumed_this_press =
        std::mem::take(&mut runtime_shell.overworld_interaction_consumed_a);
    if keys.just_pressed(KeyCode::KeyZ)
        && plain_input
        && !overworld_interaction_consumed_this_press
        && !runtime_shell.field_text_consumed_a
    {
        match has_visible_shell_a_action(runtime_shell) {
            Ok(true) => {
                let pokegear_was_open = runtime_shell.pokegear_menu_open;
                run_bevy_action(runtime_shell, press_visible_a_button);
                if !pokegear_was_open && runtime_shell.pokegear_menu_open {
                    // A menu confirmation can span many source frames on a
                    // touchscreen. Only buttons held at entry are suppressed,
                    // until release; new card inputs still work immediately.
                    runtime_shell.pokegear_opening_buttons = visible_menu_physical_down(keys)
                        & (crate::core::input::B_PAD_A | crate::core::input::B_PAD_B
                            | crate::core::input::B_PAD_SELECT | crate::core::input::B_PAD_START);
                    return;
                }
            },
            Ok(false) => {}
            Err(error) => {
                record_visible_runtime_error(runtime_shell, &error);
                runtime_shell.last_error = Some(error.to_string());
            }
        }
    }
    if keys.just_pressed(KeyCode::ShiftRight) && !alt_pressed && !ctrl_pressed {
        if has_visible_shell_select_action(runtime_shell) {
            run_bevy_action(runtime_shell, press_visible_select_button);
        }
    }
    if keys.just_pressed(KeyCode::KeyX) && plain_input && !runtime_shell.field_text_consumed_b {
        if has_visible_shell_b_action(runtime_shell) {
            run_bevy_action(runtime_shell, press_visible_b_button);
        }
    }
    if keys.just_pressed(KeyCode::Enter) && plain_input {
        if has_visible_shell_start_action(runtime_shell) {
            run_bevy_nonadvancing_action(runtime_shell, press_visible_start_button);
        }
    }
}

fn visible_ui_direction_can_repeat(runtime_shell: &BevyRuntimeShell) -> bool {
    // MoveSelectionScreen calls ScrollingMenuJoypad without setting hInMenu.
    // Therefore JoyTextDelay is edge-only before Credits and admits held
    // directions after Credits leaks TRUE into hInMenu.
    runtime_shell.battle_move_cursor.is_none() || runtime_shell.h_in_menu != 0
}

fn apply_visible_time_set_input_keys(
    keys: &ButtonInput<KeyCode>,
    runtime_shell: &mut BevyRuntimeShell,
    elapsed_input_ticks: u32,
) {
    // SetHour/SetMinutes test newly pressed A before hJoyLast directions.
    if keys.just_pressed(KeyCode::KeyZ) {
        runtime_shell.ui_held_direction = None;
        runtime_shell.ui_direction_repeat_ticks = 0;
        run_bevy_action(runtime_shell, press_visible_time_set_a_button);
        return;
    }
    if keys.just_pressed(KeyCode::KeyX) {
        runtime_shell.ui_held_direction = None;
        runtime_shell.ui_direction_repeat_ticks = 0;
        run_bevy_action(runtime_shell, press_visible_time_set_b_button);
        return;
    }
    let Some((phase, selector_active, first_repeat_frames, later_repeat_frames)) = runtime_shell
        .pending_time_set
        .as_ref()
        .map(|time_set| {
            (
                time_set.phase,
                matches!(
                    time_set.phase,
                    VisibleTimeSetPhase::SetHour | VisibleTimeSetPhase::SetMinute
                ) && time_set.input_delay_frames == 0,
                time_set.direction_first_repeat_frames,
                time_set.direction_later_repeat_frames,
            )
        })
    else {
        return;
    };
    if matches!(
        phase,
        VisibleTimeSetPhase::HourConfirm | VisibleTimeSetPhase::MinuteConfirm
    ) {
        runtime_shell.ui_held_direction = None;
        runtime_shell.ui_direction_repeat_ticks = 0;
        if keys.just_pressed(KeyCode::ArrowUp) {
            run_bevy_action(runtime_shell, |shell| {
                move_visible_time_set_direction(shell, VisibleTimeSetDirection::Up)
            });
        } else if keys.just_pressed(KeyCode::ArrowDown) {
            run_bevy_action(runtime_shell, |shell| {
                move_visible_time_set_direction(shell, VisibleTimeSetDirection::Down)
            });
        }
        return;
    }
    if !selector_active {
        runtime_shell.ui_held_direction = None;
        runtime_shell.ui_direction_repeat_ticks = 0;
        return;
    }
    let held_direction = if keys.pressed(KeyCode::ArrowUp) {
        Some((KeyCode::ArrowUp, GameButton::Up, VisibleTimeSetDirection::Up))
    } else if keys.pressed(KeyCode::ArrowDown) {
        Some((
            KeyCode::ArrowDown,
            GameButton::Down,
            VisibleTimeSetDirection::Down,
        ))
    } else {
        None
    };
    let Some((key, button, direction)) = held_direction else {
        runtime_shell.ui_held_direction = None;
        runtime_shell.ui_direction_repeat_ticks = 0;
        return;
    };
    if keys.just_pressed(key) {
        run_bevy_action(runtime_shell, |shell| {
            move_visible_time_set_direction(shell, direction)
        });
        runtime_shell.ui_held_direction = Some(button);
        runtime_shell.ui_direction_repeat_ticks = first_repeat_frames.saturating_sub(1);
        return;
    }
    if runtime_shell.ui_held_direction != Some(button) {
        // hInMenu makes hJoyDown visible even if the physical press began
        // during the selector's ten blocking DelayFrames.
        run_bevy_action(runtime_shell, |shell| {
            move_visible_time_set_direction(shell, direction)
        });
        runtime_shell.ui_held_direction = Some(button);
        runtime_shell.ui_direction_repeat_ticks = later_repeat_frames.saturating_sub(1);
        return;
    }
    for _ in 0..elapsed_input_ticks {
        if runtime_shell.ui_direction_repeat_ticks == 0 {
            run_bevy_action(runtime_shell, |shell| {
                move_visible_time_set_direction(shell, direction)
            });
            runtime_shell.ui_direction_repeat_ticks = later_repeat_frames.saturating_sub(1);
        } else {
            runtime_shell.ui_direction_repeat_ticks -= 1;
        }
    }
}

fn visible_noninteractive_battle_animation_owns_input(runtime_shell: &BevyRuntimeShell) -> bool {
    visible_trainer_result_animation_active(runtime_shell)
        || runtime_shell.visible_battle_transition.is_some()
        || runtime_shell.visible_battle_sliding_intro.is_some()
        || runtime_shell.visible_frontpic_animation.is_some()
        || runtime_shell
            .visible_capture_animation
            .as_ref()
            .is_some_and(|animation| animation.started)
        || runtime_shell
            .visible_move_animations
            .front()
            .is_some_and(|animation| animation.started)
        || runtime_shell.visible_send_out_animation.is_some()
        || runtime_shell.visible_trainer_exit_animation.is_some()
        || runtime_shell
            .battle_hp_tween
            .as_ref()
            .is_some_and(visible_battle_hp_tween_active)
}

fn visible_noninteractive_field_animation_owns_input(runtime_shell: &BevyRuntimeShell) -> bool {
    runtime_shell.pokegear_exit.is_some() || runtime_shell.pokegear_exit_input_blocked || (runtime_shell.pokegear_menu_open
        && runtime_shell.pokegear_page == PokegearPage::Radio
        && (runtime_shell.pokegear_radio_input_blocked
            || runtime_shell.pokegear_map_radio_delay.is_some_and(|remaining| remaining != 0)
            || runtime_shell.pokegear_radio_broadcast.as_ref()
                .is_some_and(|broadcast| broadcast.playback.call_suspended())))
        || runtime_shell.bill_pc_move_save.is_some()
        || runtime_shell.pc_release_sequence.is_some()
        || runtime_shell.pc_transfer_sequence.is_some()
        || runtime_shell.pc_item_move_sequence.is_some()
        || runtime_shell.pending_trainer_sight.is_some()
        || runtime_shell.visible_walk_warp_phase.is_some()
        || runtime_shell.visible_heal_machine.is_some()
        || runtime_shell.visible_magnet_train.is_some()
        || runtime_shell
            .visible_blackout_phase
            .is_some_and(|phase| phase != VisibleBlackoutPhase::AwaitText)
        || runtime_shell
            .visible_fishing_animation
            .is_some_and(|animation| animation.phase != VisibleFishingPhase::AwaitText)
        || runtime_shell.visible_fly_animation.is_some()
        || (runtime_shell.visible_waterfall_animation.is_some()
            && runtime_shell.field_notice.is_none())
        || runtime_shell.visible_flash_animation.is_some()
        || (runtime_shell.field_notice.is_none()
            && (runtime_shell.visible_field_travel_animation.is_some()
                || runtime_shell.visible_cut_animation.is_some()
                || runtime_shell.pending_whirlpool_sound_wait
                || runtime_shell.visible_headbutt_animation.is_some()))
}

fn visible_ui_initial_repeat_ticks(runtime_shell: &BevyRuntimeShell) -> u8 {
    let battle_surface = runtime_shell.battle_action_cursor.is_some()
        || runtime_shell.battle_move_cursor.is_some()
        || runtime_shell.battle_switch_cursor.is_some()
        || runtime_shell.battle_faint_prompt_cursor.is_some()
        || runtime_shell.battle_shift_prompt_cursor.is_some()
        || runtime_shell.battle_party_action_cursor.is_some()
        || runtime_shell.battle_pack_target_mode.is_some();
    if battle_surface { 8 } else { 12 }
}

fn dispatch_visible_ui_direction(runtime_shell: &mut BevyRuntimeShell, direction: GameButton) {
    if !runtime_shell.battle_messages.is_empty()
        || runtime_shell
            .battle_exp_tween
            .as_ref()
            .is_some_and(|tween| tween.started)
        || runtime_shell
            .battle_level_stats
            .front()
            .is_some_and(|stats| stats.active)
    {
        // Battle text owns the complete joypad while it is visible. Crystal's
        // text engine ignores directional input here; it must not move the
        // retained battle-menu cursor hidden underneath the textbox. Besides
        // selecting the wrong command after the text closes, moving that
        // hidden cursor changes the render key every repeat tick and can turn
        // a held direction into an expensive redraw loop.
        return;
    }
    let (input_action, action) = match direction {
        GameButton::Up => (
            "input:ui:Up",
            move_visible_primary_cursor_up as fn(&mut BevyRuntimeShell) -> Result<()>,
        ),
        GameButton::Down => (
            "input:ui:Down",
            move_visible_primary_cursor_down as fn(&mut BevyRuntimeShell) -> Result<()>,
        ),
        GameButton::Left => (
            "input:ui:Left",
            move_visible_primary_cursor_left as fn(&mut BevyRuntimeShell) -> Result<()>,
        ),
        GameButton::Right => (
            "input:ui:Right",
            move_visible_primary_cursor_right as fn(&mut BevyRuntimeShell) -> Result<()>,
        ),
        _ => return,
    };
    run_bevy_input_action(runtime_shell, input_action, action);
}

fn run_bevy_input_action(
    runtime_shell: &mut BevyRuntimeShell,
    input_action: &'static str,
    action: fn(&mut BevyRuntimeShell) -> Result<()>,
) {
    if let Err(error) = record_visible_runtime_action(runtime_shell, input_action) {
        record_visible_runtime_error(runtime_shell, &error);
        runtime_shell.last_error = Some(error.to_string());
        return;
    }
    run_bevy_action(runtime_shell, action);
}

fn run_bevy_action<F>(runtime_shell: &mut BevyRuntimeShell, action: F)
where
    F: FnOnce(&mut BevyRuntimeShell) -> Result<()>,
{
    // Options is a live interactive surface.  Advancing the script runner after
    // opening or editing it treats the menu as a non-interactive window, closes
    // it immediately, and resets its cursor on the same frame.  Keep the frame
    // in the menu until the player explicitly presses B.
    let options_was_open = runtime_shell.options_menu_open;
    let action_result = action(runtime_shell);
    let should_advance = action_result.is_ok()
        && !options_was_open
        && !runtime_shell.options_menu_open
        // Title navigation is already a player boundary.  The title action
        // must not feed its own menu selection back into the script pump.
        && runtime_shell.title_menu.is_none()
        && runtime_shell.pending_gender_selection.is_none()
        && runtime_shell.pending_name_choice.is_none()
        && runtime_shell.pending_name_input.is_none()
        && runtime_shell.pending_mail_input.is_none()
        && runtime_shell.pending_mail_read.is_none()
        && runtime_shell.pending_oak_intro.is_none()
        && runtime_shell.pending_time_set.is_none()
        && runtime_shell.pending_trainer_sight.is_none();
    let result = action_result.and_then(|()| {
        if should_advance {
            advance_visible_script_until_player_boundary(runtime_shell)
        } else {
            Ok(())
        }
    });
    match result {
        Ok(()) => {
            runtime_shell.last_error = None;
            mark_runtime_snapshot_dirty(runtime_shell);
        }
        Err(error) => {
            record_visible_runtime_error(runtime_shell, &error);
            runtime_shell.last_error = Some(format!("{error:#}"));
        }
    }
    sync_visible_battle_action_cursor(runtime_shell);
}

fn run_bevy_nonadvancing_action<F>(runtime_shell: &mut BevyRuntimeShell, action: F)
where
    F: FnOnce(&mut BevyRuntimeShell) -> Result<()>,
{
    match action(runtime_shell) {
        Ok(()) => {
            runtime_shell.last_error = None;
            mark_runtime_snapshot_dirty(runtime_shell);
        }
        Err(error) => {
            record_visible_runtime_error(runtime_shell, &error);
            runtime_shell.last_error = Some(format!("{error:#}"));
        }
    }
    sync_visible_battle_action_cursor(runtime_shell);
}

fn mark_runtime_snapshot_dirty(runtime_shell: &mut BevyRuntimeShell) {
    runtime_shell.snapshot_revision = runtime_shell.snapshot_revision.wrapping_add(1);
    runtime_shell.cached_snapshot = None;
}

fn mark_runtime_presentation_dirty(runtime_shell: &mut BevyRuntimeShell) {
    runtime_shell.snapshot_revision = runtime_shell.snapshot_revision.wrapping_add(1);
    if let Some((revision, _)) = runtime_shell.cached_snapshot.as_mut() {
        *revision = runtime_shell.snapshot_revision;
    }
}

fn cached_runtime_snapshot(
    runtime_shell: &mut BevyRuntimeShell,
) -> Result<Arc<RuntimeShellSnapshot>> {
    if let Some((revision, snapshot)) = runtime_shell.cached_snapshot.as_ref() {
        if *revision == runtime_shell.snapshot_revision {
            return Ok(Arc::clone(snapshot));
        }
    }
    let snapshot = Arc::new(runtime_shell.shell.presentation_snapshot()?);
    runtime_shell.cached_snapshot = Some((runtime_shell.snapshot_revision, Arc::clone(&snapshot)));
    Ok(snapshot)
}

fn advance_visible_script_until_player_boundary(
    runtime_shell: &mut BevyRuntimeShell,
) -> Result<()> {
    const MAX_AUTO_SCRIPT_STEPS: usize = 256;
    let mut advanced = 0usize;
    loop {
        if close_visible_noninteractive_runtime_surface(runtime_shell)? {
            continue;
        }
        let snapshot = runtime_shell.shell.presentation_snapshot()?;
        if visible_player_boundary(runtime_shell, &snapshot)
            || !has_visible_auto_script_action(runtime_shell, &snapshot)
        {
            return Ok(());
        }
        if advanced >= MAX_AUTO_SCRIPT_STEPS {
            anyhow::bail!(
                "visible script auto-advance exceeded {MAX_AUTO_SCRIPT_STEPS} steps before reaching a player boundary"
            );
        }
        press_visible_a_button(runtime_shell)?;
        mark_runtime_snapshot_dirty(runtime_shell);
        advanced += 1;
    }
}

fn visible_player_boundary(
    runtime_shell: &BevyRuntimeShell,
    snapshot: &RuntimeShellSnapshot,
) -> bool {
    (runtime_shell.field_text_reveal.is_some()
        && !visible_field_dialogue_is_entirely_consumed(runtime_shell, snapshot))
        || visible_non_text_player_boundary(runtime_shell, snapshot)
}

fn visible_non_text_player_boundary(
    runtime_shell: &BevyRuntimeShell,
    snapshot: &RuntimeShellSnapshot,
) -> bool {
    if runtime_shell.incoming_phone_sequence.is_some() {
        return true;
    }
    // waitsfx resumes on a later host update once playback finishes. Treat
    // it as a boundary here too, so auto-advance cannot spin pressing A.
    runtime_shell.visible_wait_sfx_boundary
        || runtime_shell.field_notice.is_some()
        || runtime_shell.pc_notice.is_some()
        || runtime_shell
            .visible_script_delay_frames
            .is_some_and(|frames| frames > 0)
        || runtime_shell
            .visible_earthquake
            .as_ref()
            .is_some_and(|earthquake| earthquake.frames_remaining > 0)
        || runtime_shell
            .visible_overworld_emote
            .as_ref()
            .is_some_and(|emote| emote.frames_remaining > 0)
        || runtime_shell.visible_mom_bank.is_some()
        || runtime_shell.visible_script_movement.is_some()
        || runtime_shell.visible_headbutt_animation.is_some()
        || runtime_shell.visible_card_flip.is_some()
        || runtime_shell.visible_slot_machine.is_some()
        || runtime_shell.visible_unown_puzzle.is_some()
        || runtime_shell.visible_unown_printer.is_some()
        || runtime_shell.pending_day_of_week.is_some()
        || runtime_shell.kurt_apricorn_cursor.is_some()
        || runtime_shell.visible_buena_password.is_some()
        || runtime_shell.visible_battle_tower_challenge_menu.is_some()
        || runtime_shell.visible_battle_tower_room_menu.is_some()
        || runtime_shell.buena_prize_cursor.is_some()
        || runtime_shell.pending_trainer_sight.is_some()
        || snapshot.script_events.pending_text_label.is_some()
        || snapshot.ui.pending_yes_no.is_some()
        || runtime_shell.pending_phone_prompt.is_some()
        || runtime_shell.pending_remember_password.is_some()
        || snapshot.ui.pending_text_wait.is_some()
        || snapshot.pending_move_learn.is_some()
        || snapshot.pending_shop.is_some()
        || (snapshot.ui.text_window_open && runtime_shell.active_script_cursor.is_none())
        || snapshot.ui.window_open
        || snapshot.ui.active_pokemon_picture.is_some()
        || runtime_shell.elevator_cursor.is_some()
        || visible_menu_has_selectable_options(snapshot)
        || snapshot.battle.is_some()
        || runtime_shell.battle_message_scene.is_some()
        || !runtime_shell.battle_messages.is_empty()
        || runtime_shell.start_menu_cursor.is_some()
        || runtime_shell.party_menu_open
        || runtime_shell.trainer_card_open
        || runtime_shell.pokedex_menu_open
        || runtime_shell.pokegear_menu_open
        || runtime_shell.options_menu_open
        || runtime_shell.save_menu_open
        || runtime_shell.special_boundary.is_some()
        || runtime_shell.intro_screen.is_some()
        || runtime_shell.pending_gender_selection.is_some()
        || runtime_shell.pending_name_choice.is_some()
        || runtime_shell.pending_name_input.is_some()
        || runtime_shell.pending_mail_input.is_some()
        || runtime_shell.pending_mail_read.is_some()
        || runtime_shell.pending_oak_intro.is_some()
        || runtime_shell.pending_time_set.is_some()
        || runtime_shell.credits_screen.is_some()
        || visible_field_pack_is_open(runtime_shell)
        || runtime_shell.bill_pc_action_cursor.is_some()
        || runtime_shell.pc_hub_cursor.is_some()
        || runtime_shell.decoration_menu.is_some()
        || runtime_shell.player_pc_action_cursor.is_some()
        || runtime_shell.mailbox_cursor.is_some()
        || runtime_shell.mailbox_action_cursor.is_some()
        || runtime_shell.storage_cursor.is_some()
        || runtime_shell.pc_item_cursor.is_some()
        || runtime_shell.decoration_menu.is_some()
        || runtime_shell.player_pc_action_cursor.is_some()
        || runtime_shell.mailbox_cursor.is_some()
        || runtime_shell.mailbox_action_cursor.is_some()
}

fn has_visible_auto_script_action(
    runtime_shell: &BevyRuntimeShell,
    snapshot: &RuntimeShellSnapshot,
) -> bool {
    runtime_shell.pending_scene_script.is_some()
        || snapshot.script_events.pending_text_label.is_some()
        || snapshot.script_events.pending_map_load.is_some()
        || snapshot.script_events.pending_map_refresh.is_some()
        || snapshot.script_events.pending_music_fade.is_some()
        || snapshot.script_events.pending_screen_fade.is_some()
        || !snapshot.script_events.pending_delays.is_empty()
        || !snapshot.script_events.pending_earthquakes.is_empty()
        || !snapshot.script_events.pending_emotes.is_empty()
        || snapshot.script_events.pending_script_warp.is_some()
        || !snapshot.script_events.command_queue.is_empty()
        || snapshot.script_events.next_script.is_some()
        || snapshot.script_events.map_reentry_script.is_some()
        || !snapshot.script_events.deferred_scripts.is_empty()
        || snapshot.script_events.script_ended.is_some()
        || !snapshot.script_events.audio_events.is_empty()
        || has_visible_pending_non_audio_script_events(snapshot)
        || visible_auto_runtime_flag(snapshot).is_some()
        || runtime_shell.active_script_cursor.is_some()
}

fn has_visible_pending_non_audio_script_events(snapshot: &RuntimeShellSnapshot) -> bool {
    !snapshot.script_events.graphics_events.is_empty()
        || !snapshot.script_events.money_events.is_empty()
        || !snapshot.script_events.map_events.is_empty()
        || !snapshot.script_events.control_events.is_empty()
        || !snapshot.script_events.shop_events.is_empty()
        || !snapshot.script_events.item_use_events.is_empty()
}

fn visible_menu_has_selectable_options(snapshot: &RuntimeShellSnapshot) -> bool {
    snapshot.ui.menu.as_ref().is_some_and(|menu| {
        menu.layout
            .vertical_menus
            .iter()
            .any(|vertical| !vertical.options.is_empty())
    })
}

fn close_visible_noninteractive_runtime_surface(
    runtime_shell: &mut BevyRuntimeShell,
) -> Result<bool> {
    // Some core menus have no selectable rows because Bevy presents their
    // interactive surface itself. In particular OverworldTownMap owns the
    // Pokegear map screen. Auto-closing that core marker here erased the map
    // in the same continuation pass that opened it.
    if runtime_shell.pokegear_menu_open
        || runtime_shell.pc_hub_session_open
        || runtime_shell.special_boundary.is_some()
        || runtime_shell.visible_heal_machine.is_some()
        || runtime_shell.visible_magnet_train.is_some()
    {
        return Ok(false);
    }
    let snapshot = runtime_shell.shell.presentation_snapshot()?;
    let Some(menu) = &snapshot.ui.menu else {
        return Ok(false);
    };
    if menu
        .layout
        .vertical_menus
        .iter()
        .any(|vertical| !vertical.options.is_empty())
    {
        return Ok(false);
    }
    close_active_runtime_surface(runtime_shell)?;
    Ok(true)
}

include!("deterministic_session_settling.rs");
