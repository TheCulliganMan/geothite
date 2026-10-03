// This file deliberately loads the external pack through the existing fixture.
// No source art, disassembly, capture, or generated reference image is embedded.
#[test]
fn battle_anim_gust_foreground_survives_hud_clear_at_source_frames_58_and_67() {
    for player_move in [true, false] {
        for frame in [58, 67] {
            let mut shell = route36_battle_shell_for_render_regression();
            let snapshot = shell.shell.snapshot().unwrap();
            let mut animation = battler_row_regression_animation(&snapshot, "GUST", player_move);
            animation.frame = frame;
            let mut art = RenderedTilesetArt::default();
            let bundle = battle_anim_render_bundle(&mut art, &snapshot).unwrap();
            let playback = visible_battle_objects(&bundle, &animation).unwrap();
            let live_entries = playback
                .slots
                .iter()
                .flatten()
                .flat_map(|live| &live.oam.entries);
            assert!(
                live_entries.clone().count() > 0,
                "Gust must emit actual source OAM"
            );
            assert!(live_entries.clone().all(|entry| entry[3] & 0x80 == 0));
            if frame == 67 {
                assert!(
                    playback.slots.iter().flatten().any(|live| {
                        matches!(&animation.object_events[live.event_index].command,
                        VisibleMoveObjectCommand::Spawn { object_id, .. }
                            if object_id == "BATTLE_ANIM_OBJ_HIT_YFIX")
                    }),
                    "frame 67 must exercise the source HIT object as well as wind"
                );
            }
            shell.visible_move_animations.clear();
            shell.visible_move_animations.push_back(animation);
            shell.visible_move_audio_wait = None;
            shell.battle_fainted_hud = [false; 2];
            let mut world = World::new();
            let mut images = Assets::<Image>::default();
            let mut queue = bevy::ecs::world::CommandQueue::default();
            {
                let mut commands = Commands::new(&mut queue, &world);
                spawn_visible_move_animation_objects(
                    &mut commands,
                    &snapshot,
                    &shell,
                    &mut art,
                    &shell.asset_root,
                    &mut images,
                )
                .unwrap();
            }
            queue.apply(&mut world);
            let label = format!("Gust player={player_move} source frame={frame}");
            // Reference the actual emitted source textures before any HUD is
            // present, so this regression isolates the downstream layer bug.
            let source = render_pc_audit_canvas(&mut world, &images, &label);
            {
                let mut commands = Commands::new(&mut queue, &world);
                let battle = snapshot.battle.as_ref().unwrap();
                spawn_battle_hud(
                    &mut commands,
                    &snapshot,
                    battle,
                    0,
                    false,
                    false,
                    false,
                    None,
                    None,
                    &shell.runtime.data().growth_rates,
                    &mut art,
                    &shell.asset_root,
                    &mut images,
                )
                .unwrap();
                // Cover the highest existing HUD layer too: simply lowering
                // the clear would leave these stale party icons visible.
                spawn_battle_party_balls(
                    &mut commands,
                    &snapshot,
                    battle,
                    &mut art,
                    &shell.asset_root,
                    &mut images,
                )
                .unwrap();
            }
            queue.apply(&mut world);
            let uncleared = render_pc_audit_canvas(&mut world, &images, &label);
            {
                let mut commands = Commands::new(&mut queue, &world);
                spawn_visible_battle_hud_clear(&mut commands, &shell);
            }
            queue.apply(&mut world);
            world.insert_resource(images);
            let actual = render_live_battle_canvas_for_test(&mut world, &label);
            let scale = (TILE_SIZE / SOURCE_TILE_SIZE as f32) as u32;
            let (left, top, right, bottom) = if player_move {
                (72, 56, 160, 96)
            } else {
                (8, 0, 88, 32)
            };
            let white = image::Rgba([255, 255, 255, 255]);
            let mut source_pixels = 0;
            let mut source_pixels_in_clear = 0;
            let mut stale_hud_pixels = 0;
            for y in 0..144 {
                for x in 0..160 {
                    let source_pixel = source.get_pixel(x * scale, y * scale);
                    let actual_pixel = actual.get_pixel(x * scale, y * scale);
                    let in_clear = x >= left && x < right && y >= top && y < bottom;
                    if source_pixel.0[3] != 0 {
                        source_pixels += 1;
                        source_pixels_in_clear += usize::from(in_clear);
                        assert_eq!(
                            actual_pixel, source_pixel,
                            "{label}: foreground source pixel ({x}, {y}) was erased by HUD"
                        );
                    } else if in_clear {
                        stale_hud_pixels += usize::from(
                            uncleared.get_pixel(x * scale, y * scale).0[3] != 0
                                && *uncleared.get_pixel(x * scale, y * scale) != white,
                        );
                        assert_eq!(
                            *actual_pixel, white,
                            "{label}: stale HUD pixel ({x}, {y}) survived its clear"
                        );
                    }
                }
            }
            assert!(
                source_pixels > 0,
                "{label}: source coverage must not be vacuous"
            );
            assert!(stale_hud_pixels > 0, "{label}: exercise actual HUD erasure");
            if player_move {
                // Independently audited original-source counts below LCD y=56.
                assert_eq!(
                    source_pixels_in_clear,
                    if frame == 58 { 13 } else { 23 },
                    "{label}"
                );
            }
        }
    }
}

#[test]
fn battle_anim_foreground_oam_covers_retained_bottom_text_without_a_y_cutoff() {
    let mut shell = route36_battle_shell_for_render_regression();
    let snapshot = shell.shell.snapshot().unwrap();
    // A presentation-only placement of the actual HIT source object. This
    // deliberately crosses the bottom textbox; it is not a new move script.
    let mut animation = battle_anim_regression_timeline(
        vec![VisibleMoveObjectEvent {
            frame: 1,
            command: VisibleMoveObjectCommand::Spawn {
                object_id: "BATTLE_ANIM_OBJ_HIT_YFIX".into(),
                x: 64,
                y: 136,
                param: 0,
            },
        }],
        2,
    );
    animation.move_id = "GUST".into();
    let mut art = RenderedTilesetArt::default();
    let bundle = battle_anim_render_bundle(&mut art, &snapshot).unwrap();
    let playback = visible_battle_objects(&bundle, &animation).unwrap();
    assert!(
        playback
            .slots
            .iter()
            .flatten()
            .flat_map(|live| &live.oam.entries)
            .all(|entry| entry[3] & 0x80 == 0)
    );
    shell.visible_move_animations.clear();
    shell.visible_move_animations.push_back(animation);
    shell.visible_move_audio_wait = None;
    shell.visible_battle_sliding_intro = None;
    shell.battle_retained_text = vec!["XXXXXXXXXXXXXXXXXX".into(); 2];
    let mut world = World::new();
    let mut images = Assets::<Image>::default();
    let mut queue = bevy::ecs::world::CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        spawn_visible_move_animation_objects(
            &mut commands,
            &snapshot,
            &shell,
            &mut art,
            &shell.asset_root,
            &mut images,
        )
        .unwrap();
    }
    queue.apply(&mut world);
    let source = render_pc_audit_canvas(&mut world, &images, "source HIT at bottom text");
    {
        let mut commands = Commands::new(&mut queue, &world);
        spawn_battle_command_menu(
            &mut commands,
            &snapshot,
            &shell,
            snapshot.battle.as_ref().unwrap(),
            &mut art,
            &shell.asset_root,
            &mut images,
        )
        .unwrap();
    }
    queue.apply(&mut world);
    let mut text_world = World::new();
    for (sprite, transform, handle) in world
        .query_filtered::<(&Sprite, &Transform, &Handle<Image>), Without<BattleSourceObjectMarker>>(
        )
        .iter(&world)
    {
        text_world.spawn((sprite.clone(), *transform, handle.clone()));
    }
    let text = render_pc_audit_canvas(&mut text_world, &images, "retained bottom text alone");
    world.insert_resource(images);
    let actual = render_live_battle_canvas_for_test(&mut world, "foreground HIT over bottom text");
    let scale = (TILE_SIZE / SOURCE_TILE_SIZE as f32) as u32;
    let mut below_text_start = 0;
    let mut covered_text_ink = 0;
    for y in 96..144 {
        for x in 0..160 {
            let source_pixel = source.get_pixel(x * scale, y * scale);
            if source_pixel.0[3] == 0 {
                continue;
            }
            assert_eq!(
                actual.get_pixel(x * scale, y * scale),
                source_pixel,
                "foreground source pixel ({x}, {y}) must remain above bottom BG/text"
            );
            if y >= 112 {
                below_text_start += 1;
                let text_pixel = text.get_pixel(x * scale, y * scale);
                covered_text_ink += usize::from(
                    text_pixel.0[3] != 0
                        && text_pixel.0[..3] != [255, 255, 255]
                        && text_pixel != source_pixel,
                );
            }
        }
    }
    assert!(
        below_text_start > 0,
        "exercise source pixels beyond the first text row"
    );
    assert!(
        covered_text_ink > 0,
        "exercise actual glyph ink, not only the white textbox"
    );
}

#[test]
fn battle_anim_oam_layer_preserves_source_slot_order_and_mixed_priority_fallback() {
    let shell = route36_battle_shell_for_render_regression();
    let snapshot = shell.shell.snapshot().unwrap();
    let bundle = battle_anim_render_bundle(&mut RenderedTilesetArt::default(), &snapshot).unwrap();
    assert!(BATTLE_HUD_CLEAR_Z > BATTLE_HUD_TOP_Z);
    assert!(VisibleBattleOamLayer::Foreground.depth(9) > BATTLE_HUD_CLEAR_Z);
    assert!(BATTLE_CAPTURE_BALL_Z > VisibleBattleOamLayer::Foreground.depth(0));
    for slot in 1..10 {
        for layer in [
            VisibleBattleOamLayer::Foreground,
            VisibleBattleOamLayer::Legacy,
        ] {
            assert!(layer.depth(slot - 1) > layer.depth(slot));
        }
    }
    for player_move in [true, false] {
        let mut gust = battler_row_regression_animation(&snapshot, "GUST", player_move);
        gust.frame = 67;
        let mut playback = visible_battle_objects(&bundle, &gust).unwrap();
        assert_eq!(
            visible_battle_oam_layer(&playback),
            VisibleBattleOamLayer::Foreground
        );
        // The guard examines live emitted attributes, not just declarations.
        let live = playback
            .slots
            .iter_mut()
            .flatten()
            .find(|live| !live.oam.entries.is_empty())
            .unwrap();
        live.oam.entries[0][3] |= 0x80;
        assert_eq!(
            visible_battle_oam_layer(&playback),
            VisibleBattleOamLayer::Legacy,
            "a single behind-BG piece retains the previous layer for the entire frame"
        );

        let mut tackle = battler_row_regression_animation(&snapshot, "TACKLE", player_move);
        tackle.frame = 2;
        let mut playback = visible_battle_objects(&bundle, &tackle).unwrap();
        assert_eq!(
            visible_battle_oam_layer(&playback),
            VisibleBattleOamLayer::Foreground
        );
        let (player, enemy) = visible_live_battler_row_extractions(&tackle, &playback);
        let row = if player_move { enemy } else { player }.unwrap();
        assert_eq!(row.oam_layer, VisibleBattleOamLayer::Foreground);
        assert_eq!(
            visible_battler_extracted_row_depth(row, 3.0),
            VisibleBattleOamLayer::Foreground.depth(usize::from(row.oam_slot.unwrap()))
        );
        let source_row = playback.battler_rows.iter_mut().flatten().next().unwrap();
        assert!(!source_row.oam.entries.is_empty());
        source_row.oam.entries[0][3] |= 0x80;
        assert_eq!(
            visible_battle_oam_layer(&playback),
            VisibleBattleOamLayer::Legacy
        );
        let (player, enemy) = visible_live_battler_row_extractions(&tackle, &playback);
        let row = if player_move { enemy } else { player }.unwrap();
        assert_eq!(row.oam_layer, VisibleBattleOamLayer::Legacy);
        assert_eq!(
            visible_battler_extracted_row_depth(row, 3.0),
            VisibleBattleOamLayer::Legacy.depth(usize::from(row.oam_slot.unwrap()))
        );
    }
}

#[test]
fn battle_anim_oam_layer_keeps_capture_and_shiny_send_out_presentation_depths() {
    for (capture, blocked) in [(true, false), (true, true), (false, false)] {
        let mut shell = route36_battle_shell_for_render_regression();
        let snapshot = shell.shell.snapshot().unwrap();
        shell.visible_move_animations.clear();
        shell.visible_move_audio_wait = None;
        shell.visible_send_out_animation = None;
        shell.visible_capture_animation = None;
        if capture {
            shell.visible_capture_animation = Some(VisibleCaptureAnimation {
                trigger_message: String::new(),
                ball_id: "POKE_BALL".into(),
                animation_shakes: 3,
                blocked,
                caught: true,
                started: true,
                complete: false,
                sprites_cleared: false,
                frame: if blocked { 12 } else { 36 },
            });
        } else {
            shell.visible_send_out_animation = Some(VisibleSendOutAnimation {
                side: crate::core::battle::turn::BattleSide::Player,
                frame: VisibleSendOutAnimation::NORMAL_FRAMES + 6,
                shiny: true,
            });
        }
        let mut world = World::new();
        let mut art = RenderedTilesetArt::default();
        let mut images = Assets::<Image>::default();
        let mut queue = bevy::ecs::world::CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            spawn_visible_move_animation_objects(
                &mut commands,
                &snapshot,
                &shell,
                &mut art,
                &shell.asset_root,
                &mut images,
            )
            .unwrap();
        }
        queue.apply(&mut world);
        let mut depths = world
            .query_filtered::<&Transform, With<BattleSourceObjectMarker>>()
            .iter(&world)
            .map(|transform| transform.translation.z)
            .collect::<Vec<_>>();
        depths.sort_by(|a, b| b.total_cmp(a));
        assert!(
            !depths.is_empty(),
            "capture={capture} blocked={blocked}: emitted OAM required"
        );
        if capture && !blocked {
            assert_eq!(depths.len(), 2, "retained ball and lid at source frame 36");
        }
        let front = if capture { 4.1 } else { 3.45 };
        let expected = art
            .battle_object_runtime
            .as_ref()
            .unwrap()
            .slots
            .iter()
            .enumerate()
            .filter(|(_, live)| {
                live.as_ref()
                    .is_some_and(|live| !live.oam.entries.is_empty())
            })
            .map(|(slot, _)| front - slot as f32 * 0.001)
            .collect::<Vec<_>>();
        assert_eq!(
            depths, expected,
            "capture={capture} blocked={blocked}: preserve previous presentation layer"
        );
    }
}
