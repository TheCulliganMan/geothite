fn prepare_visible_battle_entry(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    prepare_visible_battle_entry_with_music_reset(runtime_shell, true)
}

fn prepare_visible_battle_entry_after_visible_step(
    runtime_shell: &mut BevyRuntimeShell,
) -> Result<()> {
    prepare_visible_battle_entry_with_music_reset(runtime_shell, false)
}

fn prepare_visible_battle_entry_with_music_reset(
    runtime_shell: &mut BevyRuntimeShell,
    reset_music: bool,
) -> Result<()> {
    // Validate first. A failed entry must not erase a retained terminal scene.
    let snapshot = runtime_shell.shell.snapshot()?;
    let battle = snapshot
        .battle
        .as_ref()
        .context("battle entry requires an active battle snapshot")?;
    // A wild step or checked fishing cast may have committed before its visible
    // landing/notice. Adopt that exact source, and preserve it on repeated entry.
    let origin = take_visible_battle_origin_for_entry(runtime_shell, &snapshot, battle);
    let bound_static = runtime_shell.battle_origin.bound_static.take()
        .filter(|bound| bound.generation == origin.generation);
    let bound_fishing = runtime_shell.battle_origin.bound_fishing.take()
        .filter(|bound| bound.origin.generation == origin.generation);
    let bound_walking = runtime_shell.battle_origin.bound_walking.take()
        .filter(|bound| bound.origin.generation == origin.generation);
    reset_visible_battle_presentation(runtime_shell);
    runtime_shell.battle_origin.active = Some(origin);
    runtime_shell.battle_origin.bound_static = bound_static;
    runtime_shell.battle_origin.bound_fishing = bound_fishing;
    runtime_shell.battle_origin.bound_walking = bound_walking;
    if reset_music {
        reset_visible_music_state(runtime_shell);
    }
    runtime_shell.battle_enemy_hp_at_player_send_out = Some(battle.enemy_pokemon.hp);
    let active_player = battle
        .active_player_party_index
        .and_then(|index| snapshot.party.slots.iter().find(|slot| slot.index == index))
        .context("battle entry requires its active party slot in the runtime snapshot")?;
    let player_level = active_player.pokemon.level;
    let environment = snapshot
        .maps
        .iter()
        .find(|map| map.map_name == snapshot.overworld.map_name)
        .with_context(|| {
            format!(
                "battle entry map {} is absent from the runtime map catalog",
                snapshot.overworld.map_name
            )
        })?
        .attributes
        .environment
        .as_deref()
        .context("battle entry map has no source environment")?;
    runtime_shell.visible_battle_transition = Some(VisibleBattleTransition {
        frame: 0,
        stronger_enemy: player_level.saturating_add(3) < battle.enemy_pokemon.level,
        cave_environment: ["CAVE", "ENVIRONMENT_5", "DUNGEON"]
            .iter()
            .any(|candidate| environment.eq_ignore_ascii_case(candidate)),
        trainer_battle: matches!(&battle.kind, crate::RuntimeBattleKind::Trainer { .. }),
    });
    let (player_hp, player_max_hp, player_pixels) = (
        active_player.pokemon.hp,
        active_player.pokemon.max_hp,
        battle_hud_hp_pixels(active_player.pokemon.hp, active_player.pokemon.max_hp),
    );
    let enemy_pixels = battle_hud_hp_pixels(battle.enemy_pokemon.hp, battle.enemy_pokemon.max_hp);
    runtime_shell.battle_hp_tween = Some(VisibleBattleHpTween {
        player_hp,
        player_target_hp: player_hp,
        player_max_hp,
        player_pixels,
        player_target_pixels: player_pixels,
        player_frames_until_step: 0,
        enemy_pixels,
        enemy_target_pixels: enemy_pixels,
        enemy_frames_until_step: 0,
    });
    let player_send_out_message = visible_player_send_out_message(
        &snapshot,
        active_player.index,
        true,
    )?;
    match &battle.kind {
        crate::RuntimeBattleKind::Trainer { trainer_name, .. } => {
            runtime_shell
                .battle_messages
                .push_back(format!("{trainer_name}\nwants to battle!"));
            runtime_shell.battle_messages.push_back(format!(
                "{}\nsent out\n{}!",
                trainer_name, battle.enemy_pokemon.nickname
            ));
            runtime_shell
                .battle_messages
                .push_back(player_send_out_message.clone());
        }
        crate::RuntimeBattleKind::Wild { .. } | crate::RuntimeBattleKind::StaticWild { .. } => {
            runtime_shell
                .battle_messages
                .push_back(format!("Wild {}\nappeared!", battle.enemy_pokemon.nickname));
            if battle.battle_type != "BATTLETYPE_TUTORIAL" {
                runtime_shell
                    .battle_messages
                    .push_back(player_send_out_message.clone());
            }
        }
    }
    // Entry narration is already a retained battle presentation.
    // Keep the exact battle-start snapshot behind those messages so
    // the renderer never falls back to an absent scene (a blank/error
    // frame) before the first text page is acknowledged.
    runtime_shell.battle_message_scene = Some(Arc::new(snapshot.clone()));
    if battle.battle_type == "BATTLETYPE_TUTORIAL" {
        runtime_shell.visible_catch_tutorial = Some(VisibleCatchTutorial::default());
        runtime_shell.pending_ui_button_presses.clear();
    }
    runtime_shell.battle_entry_messages_remaining = runtime_shell.battle_messages.len();
    Ok(())
}

fn advance_visible_battle_transition(runtime_shell: &mut BevyRuntimeShell) {
    let Some(transition) = runtime_shell.visible_battle_transition.as_mut() else {
        return;
    };
    transition.frame = transition.frame.saturating_add(1);
    if transition.frame >= visible_battle_transition_total_frames(transition) {
        runtime_shell.visible_battle_transition = None;
        runtime_shell.visible_battle_sliding_intro = Some(0);
    }
    mark_runtime_presentation_dirty(runtime_shell);
}

fn visible_battle_transition_total_frames(transition: &VisibleBattleTransition) -> u16 {
    let prefix_frames = if transition.trainer_battle { 4 } else { 3 };
    let (between_frames, outro_frames, finish_frames) =
        match (transition.cave_environment, transition.stronger_enemy) {
            // DETERMINE/LOAD/SETUP precede all four paths. After the three
            // flashes, ordinary paths and outdoor scatter own NEXT + SETUP;
            // the stronger cave zoom owns NEXT only.
            (true, false) => (2, 15, 1),
            (true, true) => (1, 9, 2),
            (false, false) => (2, 61, 4),
            (false, true) => (2, 21, 1),
        };
    prefix_frames + 75 + between_frames + outro_frames + finish_frames
}

fn visible_battle_transition_is_terminal(transition: &VisibleBattleTransition) -> bool {
    transition.frame.saturating_add(1) >= visible_battle_transition_total_frames(transition)
}

const BATTLE_TRANSITION_POKEBALL: [&str; 16] = [
    "......XXXX......",
    "....XXXXXXXX....",
    "..XXXX....XXXX..",
    "..XX........XX..",
    ".XX..........XX.",
    ".XX...XXXX...XX.",
    "XX...XX..XX...XX",
    "XXXXXX....XXXXXX",
    "XXXXXX....XXXXXX",
    "XX...XX..XX...XX",
    ".XX...XXXX...XX.",
    ".XX..........XX.",
    "..XX........XX..",
    "..XXXX....XXXX..",
    "....XXXXXXXX....",
    "......XXXX......",
];

fn battle_transition_bgp(transition: VisibleBattleTransition, dark: bool) -> u8 {
    // StartTrainerBattle_Flash: twelve BGP writes, each held for two
    // frames, followed by a sentinel frame. Repeat the sequence three times.
    const BGP: [u8; 12] = [
        0xf9, 0xfe, 0xff, 0xfe, 0xf9, 0xe4, 0x90, 0x40, 0x00, 0x40, 0x90, 0xe4,
    ];
    let prefix = if transition.trainer_battle { 4 } else { 3 };
    if !dark && transition.frame >= prefix && transition.frame < prefix + 75 {
        BGP.get(((transition.frame - prefix) % 25 / 2) as usize)
            .copied()
            .unwrap_or(0xe4)
    } else {
        0xe4
    }
}

fn prepare_battle_transition_texture(
    root: &AssetRoot,
    transition: VisibleBattleTransition,
    tiles: &[BattleTransitionTile],
    camera_offset: Vec2,
    dark: bool,
    priority_only: bool,
    existing: Option<Handle<Image>>,
    images: &mut Assets<Image>,
) -> Result<Handle<Image>> {
    let trainer = transition.trainer_battle && transition.frame >= 2;
    let (ball_tile, trainer_palette) = if trainer {
        let path = root.runtime_assets().join("gfx/overworld");
        let tile = crate::read_runtime_asset(&path.join("battle_transition_tiles.2bpp"))?;
        anyhow::ensure!(tile.len() >= 16, "battle transition square tile is missing");
        let palette_bytes = crate::read_runtime_asset(&path.join(if dark {
            "trainer_battle_dark.pal"
        } else {
            "trainer_battle.pal"
        }))?;
        let palette = parse_palette_file(std::str::from_utf8(&palette_bytes)?, None)?
            .into_iter()
            .next()
            .context("trainer battle palette is missing")?;
        (tile, palette)
    } else {
        (Vec::new(), [[0; 3]; 4])
    };
    let bgp = battle_transition_bgp(transition, dark);
    let mut data = vec![0; 160 * 144 * 4];
    let scale = TILE_SIZE / SOURCE_TILE_SIZE as f32;
    let origin_x =
        i32::from(CLASSIC_SCROLL_HALO_TILES) * 8 - (camera_offset.x / scale).round() as i32;
    let origin_y =
        i32::from(CLASSIC_SCROLL_HALO_TILES) * 8 + (camera_offset.y / scale).round() as i32;
    for y in 0..144 {
        for x in 0..160 {
            let source_x =
                (origin_x + x as i32).clamp(0, i32::from(CLASSIC_SCROLL_TILES_X) * 8 - 1) as usize;
            let source_y =
                (origin_y + y as i32).clamp(0, i32::from(CLASSIC_SCROLL_TILES_Y) * 8 - 1) as usize;
            let tile = tiles
                .get(source_y / 8 * CLASSIC_SCROLL_TILES_X as usize + source_x / 8)
                .context("battle transition source tile is missing")?;
            let mut index = tile.indices[source_y % 8 * 8 + source_x % 8];
            let tile_x = x / 8;
            let tile_y = y / 8;
            if trainer
                && (2..18).contains(&tile_x)
                && (1..17).contains(&tile_y)
                && BATTLE_TRANSITION_POKEBALL[tile_y - 1].as_bytes()[tile_x - 2] == b'X'
            {
                // LoadPokeBallGraphics writes textured tile FE at each X;
                // all other cells keep the underlying overworld tile.
                let bit = 7 - x % 8;
                index = ((ball_tile[y % 8 * 2] >> bit) & 1)
                    | (((ball_tile[y % 8 * 2 + 1] >> bit) & 1) << 1);
            }
            let palette = if trainer {
                &trainer_palette
            } else {
                &tile.palette
            };
            let rgb = palette[((bgp >> (index * 2)) & 3) as usize];
            let offset = (y * 160 + x) * 4;
            data[offset..offset + 3].copy_from_slice(&rgb);
            data[offset + 3] = if !priority_only
                || (index != 0
                    && tile
                        .priority_from_row
                        .is_some_and(|row| source_y % 8 >= row as usize))
            {
                255
            } else {
                0
            };
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: 160,
            height: 144,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    if let Some(handle) = existing {
        if let Some(target) = images.get_mut(&handle) {
            *target = image;
            return Ok(handle);
        }
    }
    Ok(images.add(image))
}

fn spawn_visible_battle_transition(
    commands: &mut Commands,
    transition: VisibleBattleTransition,
    viewport_texture: Option<Handle<Image>>,
    priority_texture: Option<Handle<Image>>,
) {
    let frame = usize::from(transition.frame);
    let prefix_frames = if transition.trainer_battle { 4 } else { 3 };
    if frame < prefix_frames {
        return;
    }
    let flash_frames = 75;
    let effect_frame = frame - prefix_frames;
    if effect_frame < flash_frames {
        return;
    }

    let (between_frames, outro_frames) =
        match (transition.cave_environment, transition.stronger_enemy) {
            (true, false) => (2, 15),
            (true, true) => (1, 9),
            (false, false) => (2, 61),
            (false, true) => (2, 21),
        };
    if effect_frame < flash_frames + between_frames {
        return;
    }
    let outro = effect_frame - flash_frames - between_frames;
    if outro >= outro_frames {
        // DoBattleTransition finishes by blacking every BG palette and holding
        // that complete frame before battle setup takes over. Keep this as an
        // explicit surface instead of retaining the final (sometimes partial)
        // outro geometry beneath the white battle canvas handoff.
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::BLACK,
                    custom_size: Some(Vec2::new(PLAYFIELD_WIDTH, PLAYFIELD_HEIGHT)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, 2.76),
                ..default()
            },
            BattleCommandMarker,
        ));
        return;
    }
    // `outro == 0` is already the first call of the selected effect state:
    // NextScene and the optional setup state are accounted for above.  Drive
    // every effect from the one-based number of source calls that have run so
    // the first mutation is visible and the final source mutation is not
    // replaced prematurely by the terminal-black hold.
    let effect_step = outro.saturating_add(1);
    match (transition.cave_environment, transition.stronger_enemy) {
        (true, true) => {
            // StartTrainerBattle_ZoomToBlack writes nine centered boxes in
            // one BG-map update apiece: 4x2 through the full 20x18 LCD.
            let boxes = effect_step.min(9);
            let width_tiles = 2 + boxes * 2;
            let height_tiles = boxes * 2;
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::BLACK,
                        custom_size: Some(Vec2::new(
                            width_tiles as f32 * TILE_SIZE,
                            height_tiles as f32 * TILE_SIZE,
                        )),
                        ..default()
                    },
                    // Tile (0, 0) now begins at the LCD edge, so the growing
                    // even-sized box is centered on the camera itself.
                    transform: Transform::from_xyz(0.0, 0.0, 2.7),
                    ..default()
                },
                BattleCommandMarker,
            ));
        }
        (false, true) => {
            // SpeckleToBlack chooses twelve previously unfilled LCD tiles on
            // each of sixteen frames, then holds for three frames. Reproduce
            // a deterministic scatter with rejection of cells already filled.
            // The trainer's textured FE tiles are not black FF cells.
            let calls = effect_step.min(16) * 12;
            let mut black = [false; 20 * 18];
            let mut seed = 0_u32;
            for _ in 0..calls {
                for _ in 0..(20 * 18) {
                    seed = (seed * 9_301 + 49_297) % 233_280;
                    let y = (seed * 18 / 233_280) as usize;
                    seed = (seed * 9_301 + 49_297) % 233_280;
                    let x = (seed * 20 / 233_280) as usize;
                    if !black[y * 20 + x] {
                        black[y * 20 + x] = true;
                        break;
                    }
                }
            }
            for (index, filled) in black.into_iter().enumerate() {
                if filled {
                    spawn_visible_battle_transition_black_tile(commands, index % 20, index / 20);
                }
            }
        }
        (false, false) => {
            // SpinToBlack has twenty wedge entries, each held for two LCD
            // delay frames after each write. The twentieth entry therefore
            // remains current for the source terminal hold as well.
            let wedge_count = ((effect_step + 2) / 3).min(20);
            for wedge_index in 0..wedge_count {
                spawn_visible_battle_transition_wedge(commands, wedge_index);
            }
        }
        (true, false) => {
            // The retained Bevy viewport is the complete source surface. Draw
            // every native scanline independently and wrap it like the Game
            // Boy BG map when the transition changes SCX.
            if let Some(texture) = viewport_texture {
                let mut counter = 0_u8;
                let mut offset = 0_u8;
                let mut amplitude = 0_u8;
                for _ in 0..effect_step {
                    amplitude = counter;
                    let previous_offset = offset;
                    offset = offset.wrapping_add(1);
                    // StartTrainerBattle_SineWave increments the offset byte
                    // in memory while A still contains its previous value;
                    // that old value is what the counter adds this frame.
                    counter = counter.wrapping_add(previous_offset);
                }
                let source_scale = TILE_SIZE / SOURCE_TILE_SIZE as f32;
                for source_y in 0..TITLE_SCREEN_HEIGHT {
                    // The LCD override loop feeds angles 0, 2, 4, ... to
                    // calc_sine_wave for successive scanlines. Each Bevy strip
                    // is one native scanline and uses the raw counter amplitude.
                    let angle = (source_y as u8).wrapping_mul(2);
                    let shift = visible_battle_anim_sine(angle, amplitude) as f32;
                    let shift = shift * source_scale;
                    let wrap_shift = if shift > 0.0 {
                        Some(shift - PLAYFIELD_WIDTH)
                    } else if shift < 0.0 {
                        Some(shift + PLAYFIELD_WIDTH)
                    } else {
                        None
                    };
                    for (texture, z) in std::iter::once((&texture, 0.0))
                        .chain(priority_texture.as_ref().map(|priority| (priority, 2.4)))
                    {
                        for x in std::iter::once(shift).chain(wrap_shift) {
                            commands.spawn((
                                SpriteBundle {
                                    texture: texture.clone(),
                                    sprite: Sprite {
                                        rect: Some(Rect::new(
                                            0.0,
                                            source_y as f32,
                                            TITLE_SCREEN_WIDTH as f32,
                                            source_y as f32 + 1.0,
                                        )),
                                        custom_size: Some(Vec2::new(PLAYFIELD_WIDTH, source_scale)),
                                        ..default()
                                    },
                                    transform: Transform::from_xyz(
                                        x,
                                        PLAYFIELD_TOP - (source_y as f32 + 0.5) * source_scale,
                                        z,
                                    ),
                                    ..default()
                                },
                                BattleCommandMarker,
                            ));
                        }
                    }
                }
            }
        }
    }
}

fn spawn_visible_battle_transition_black_tile(commands: &mut Commands, x: usize, y: usize) {
    let (tile_x, tile_y) = render_tile_playfield_position(x as i16, y as i16);
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::splat(TILE_SIZE)),
                ..default()
            },
            transform: Transform::from_xyz(tile_x, tile_y, 2.7),
            ..default()
        },
        BattleCommandMarker,
    ));
}

fn spawn_visible_battle_transition_wedge(commands: &mut Commands, wedge_index: usize) {
    const WEDGE_1: &[i8] = &[2, 3, 5, 4, 9];
    const WEDGE_2: &[i8] = &[1, 1, 2, 2, 4, 2, 4, 2, 3];
    const WEDGE_3: &[i8] = &[2, 1, 3, 1, 4, 1, 4, 1, 4, 1, 3, 1, 2, 1, 1, 1, 1];
    const WEDGE_4: &[i8] = &[4, 1, 4, 0, 3, 1, 3, 0, 2, 1, 2, 0, 1];
    const WEDGE_5: &[i8] = &[4, 0, 3, 0, 3, 0, 2, 0, 2, 0, 1, 0, 1, 0, 1];
    // quadrant bits match battle_transition.asm: bit 0 = right, bit 1 = lower.
    const ENTRIES: [(u8, &[i8], i8, i8); 20] = [
        (0, WEDGE_1, 1, 6),
        (0, WEDGE_2, 0, 3),
        (0, WEDGE_3, 1, 0),
        (0, WEDGE_4, 5, 0),
        (0, WEDGE_5, 9, 0),
        (1, WEDGE_5, 10, 0),
        (1, WEDGE_4, 14, 0),
        (1, WEDGE_3, 18, 0),
        (1, WEDGE_2, 19, 3),
        (1, WEDGE_1, 18, 6),
        (3, WEDGE_1, 18, 11),
        (3, WEDGE_2, 19, 14),
        (3, WEDGE_3, 18, 17),
        (3, WEDGE_4, 14, 17),
        (3, WEDGE_5, 10, 17),
        (2, WEDGE_5, 9, 17),
        (2, WEDGE_4, 5, 17),
        (2, WEDGE_3, 1, 17),
        (2, WEDGE_2, 0, 14),
        (2, WEDGE_1, 1, 11),
    ];
    let Some(&(quadrant, data, mut x, mut y)) = ENTRIES.get(wedge_index) else {
        return;
    };
    let right = quadrant & 1 != 0;
    let lower = quadrant & 2 != 0;
    let mut cursor = 0;
    while cursor < data.len() {
        let width = data[cursor];
        cursor += 1;
        let row_start_x = x;
        for _ in 0..width {
            if (0..20).contains(&x) && (0..18).contains(&y) {
                let (tile_x, tile_y) = render_tile_playfield_position(i16::from(x), i16::from(y));
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::BLACK,
                            custom_size: Some(Vec2::splat(TILE_SIZE)),
                            ..default()
                        },
                        transform: Transform::from_xyz(tile_x, tile_y, 2.7),
                        ..default()
                    },
                    BattleCommandMarker,
                ));
            }
            x += if right { 1 } else { -1 };
        }
        x = row_start_x;
        y += if lower { -1 } else { 1 };
        let Some(&gap) = data.get(cursor) else {
            break;
        };
        cursor += 1;
        x += if right { -gap } else { gap };
    }
}

fn advance_visible_capture_animation(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let Some(animation) = runtime_shell.visible_capture_animation.as_mut() else {
        return Ok(());
    };
    if !animation.started {
        return Ok(());
    }
    animation.frame = animation.frame.saturating_add(1);
    let frame = animation.frame;
    let caught = animation.caught;
    let total_frames = animation.total_frames();
    let sound = animation.sound_at_frame(frame);
    if let Some(sound) = sound {
        queue_visible_shell_sound_effect(runtime_shell, sound)?;
    }
    if frame >= total_frames {
        if caught {
            // Text_BallCaught's sound_caught_mon command runs as the Gotcha
            // page becomes visible, after the ball animation has completed.
            queue_visible_shell_sound_effect(runtime_shell, "SFX_CAUGHT_MON")?;
            if let Some(animation) = runtime_shell.visible_capture_animation.as_mut() {
                animation.started = false;
                animation.complete = true;
            }
        } else {
            runtime_shell.visible_capture_animation = None;
        }
    }
    mark_runtime_presentation_dirty(runtime_shell);
    Ok(())
}

fn advance_visible_move_animation(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let audio_wait_completed = if let Some(remaining) = runtime_shell.visible_move_audio_wait.as_mut() {
        *remaining = remaining.saturating_sub(1);
        if *remaining > 0 { return Ok(()); }
        runtime_shell.visible_move_audio_wait = None;
        true
    } else { false };
    let (due_sounds, due_cries, player_move) = {
        let Some(animation) = runtime_shell.visible_move_animations.front_mut() else {
            return Ok(());
        };
        if !animation.started {
            return Ok(());
        }
        if !audio_wait_completed {
            animation.frame = animation.frame.saturating_add(1);
        }
        let start = animation.next_sound_event;
        while animation
            .sound_events
            .get(animation.next_sound_event)
            .is_some_and(|(frame, _)| *frame <= animation.frame)
        {
            animation.next_sound_event += 1;
        }
        let sounds = animation.sound_events[start..animation.next_sound_event]
            .iter()
            .map(|(_, sound)| sound.clone())
            .collect::<Vec<_>>();
        let cry_start = animation.next_cry_event;
        while animation
            .cry_events
            .get(animation.next_cry_event)
            .is_some_and(|(frame, _)| *frame <= animation.frame)
        {
            animation.next_cry_event += 1;
        }
        let cries = animation.cry_events[cry_start..animation.next_cry_event]
            .iter()
            .map(|(_, selector)| *selector)
            .collect::<Vec<_>>();
        (sounds, cries, animation.player_move)
    };
    for sound in due_sounds {
        queue_visible_battle_sound_effect(runtime_shell, &sound, player_move)?;
    }
    if !due_cries.is_empty() {
        let species = visible_move_animation_user_species(runtime_shell, player_move)
            .context("battle animation cry has no visible user species")?;
        for selector in due_cries {
            queue_visible_pokemon_animation_cry(runtime_shell, &species, selector)?;
        }
    }
    let finished_trigger = {
        let Some(animation) = runtime_shell.visible_move_animations.front_mut() else {
            return Ok(());
        };
        (animation.frame >= animation.total_frames).then(|| animation.trigger_message.clone())
    };
    if let Some(trigger_message) = finished_trigger {
        if !audio_wait_completed {
            let animation = runtime_shell.visible_move_animations.front().unwrap();
            let wait = visible_surf_source_audio_wait(runtime_shell, animation)?;
            if wait > 0 {
                runtime_shell.visible_move_audio_wait = Some(wait);
                mark_runtime_presentation_dirty(runtime_shell);
                return Ok(());
            }
        }
        let completed = runtime_shell.visible_move_animations.pop_front().unwrap();
        if completed.move_id == "FAINT_MON" {
            runtime_shell.battle_fainted_hud[usize::from(!completed.player_move)] = true;
        }
        let completed_before_trigger_message = runtime_shell
            .battle_messages
            .front()
            .is_some_and(|message| message == &trigger_message);
        if completed_before_trigger_message {
            let continues_same_command =
                runtime_shell
                    .visible_move_animations
                    .front()
                    .is_some_and(|animation| {
                        !animation.started && animation.trigger_message == trigger_message
                    });
            let mut applied_scene = false;
            if let Some(index) = runtime_shell
                .pending_battle_scenes_after_message
                .iter()
                .position(|(trigger, _)| trigger == &trigger_message)
            {
                let (_, scene) = runtime_shell
                    .pending_battle_scenes_after_message
                    .remove(index)
                    .unwrap();
                retarget_visible_battle_hp_tween(runtime_shell, &scene);
                runtime_shell.battle_message_scene = Some(scene);
                applied_scene = true;
            }
            if continues_same_command {
                let next = runtime_shell.visible_move_animations.front_mut().unwrap();
                if applied_scene {
                    next.waiting_for_hp = true;
                } else {
                    next.started = true;
                }
            }
            mark_runtime_presentation_dirty(runtime_shell);
            return Ok(());
        }
        let continues_same_command =
            runtime_shell
                .visible_move_animations
                .front()
                .is_some_and(|animation| {
                    !animation.started && animation.trigger_message == trigger_message
                });
        if continues_same_command {
            let intermediate_scene = runtime_shell
                .pending_battle_scenes_after_message
                .iter()
                .position(|(trigger, _)| trigger == &trigger_message)
                .and_then(|index| {
                    runtime_shell
                        .pending_battle_scenes_after_message
                        .remove(index)
                        .map(|(_, scene)| scene)
                });
            if let Some(scene) = intermediate_scene {
                retarget_visible_battle_hp_tween(runtime_shell, &scene);
                runtime_shell.battle_message_scene = Some(scene);
                runtime_shell
                    .visible_move_animations
                    .front_mut()
                    .unwrap()
                    .waiting_for_hp = true;
            } else {
                runtime_shell
                    .visible_move_animations
                    .front_mut()
                    .unwrap()
                    .started = true;
            }
        } else {
            runtime_shell.battle_message_scenes.pop_front();
            if let Some(scene) = runtime_shell.battle_message_scenes.front().cloned() {
                retarget_visible_battle_hp_tween(runtime_shell, &scene);
                runtime_shell.battle_message_scene = Some(scene);
            }
            if let Some(index) = runtime_shell
                .pending_battle_scenes_after_message
                .iter()
                .position(|(trigger, _)| trigger == &trigger_message)
            {
                let (_, scene) = runtime_shell
                    .pending_battle_scenes_after_message
                    .remove(index)
                    .unwrap();
                if runtime_shell.battle_message_scenes.is_empty() {
                    retarget_visible_battle_hp_tween(runtime_shell, &scene);
                    runtime_shell.battle_message_scene = Some(scene);
                }
            }
        }
    }
    mark_runtime_presentation_dirty(runtime_shell);
    Ok(())
}

fn visible_move_animation_user_species(
    runtime_shell: &BevyRuntimeShell,
    player_move: bool,
) -> Option<String> {
    let scene = runtime_shell.battle_message_scene.as_deref()?;
    let battle = scene.battle.as_ref()?;
    if !player_move {
        return Some(
            battle
                .enemy_transformed_species
                .clone()
                .unwrap_or_else(|| battle.enemy_pokemon.species.id.clone()),
        );
    }
    let active_index = battle.active_player_party_index?;
    let slot = scene
        .party
        .slots
        .iter()
        .find(|slot| slot.index == active_index)?;
    Some(
        battle
            .player_transformed_species
            .clone()
            .unwrap_or_else(|| slot.pokemon.species.id.clone()),
    )
}

fn advance_visible_send_out_animation(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let wild_entrance = visible_wild_entrance_animation_active(runtime_shell);
    let Some(animation) = runtime_shell.visible_send_out_animation.as_mut() else {
        return Ok(());
    };
    animation.frame = animation.frame.saturating_add(1);
    let shiny_sound = animation.shiny
        && animation.frame >= VisibleSendOutAnimation::NORMAL_FRAMES
        && animation.frame < VisibleSendOutAnimation::NORMAL_FRAMES + 32
        && (animation.frame - VisibleSendOutAnimation::NORMAL_FRAMES) % 4 == 0;
    let finished = animation.frame >= animation.total_frames();
    let side = animation.side;
    if shiny_sound {
        queue_visible_shell_sound_effect(runtime_shell, "SFX_SHINE")?;
    }
    if finished {
        runtime_shell.visible_send_out_animation = None;
        if wild_entrance {
            if runtime_shell.shell.snapshot()?.trainer.options.battle_scene == BattleScene::On {
                start_visible_enemy_frontpic_animation(runtime_shell, 0)?;
            }
            mark_runtime_presentation_dirty(runtime_shell);
            return Ok(());
        }
        // Use the retained send-out scene: the authoritative turn may already
        // include damage whose animation has not played yet.
        let scene = if let Some(scene) = runtime_shell.battle_message_scene.as_deref() {
            scene.clone()
        } else {
            runtime_shell.shell.presentation_snapshot()?
        };
        initialize_visible_send_out_hp(runtime_shell, &scene, side);
        if side == crate::core::battle::turn::BattleSide::Enemy {
            let snapshot = runtime_shell.shell.snapshot()?;
            let speed = snapshot
                .battle
                .as_ref()
                .map(|battle| match battle.kind {
                    RuntimeBattleKind::Trainer { .. } => 4,
                    RuntimeBattleKind::Wild { .. } | RuntimeBattleKind::StaticWild { .. } => 0,
                })
                .unwrap_or(0);
            start_visible_enemy_frontpic_animation(runtime_shell, speed)?;
        }
        if let Some((species_id, reason, _)) = runtime_shell
            .pending_battle_cries_after_messages
            .pop_front()
        {
            queue_visible_pokemon_cry(runtime_shell, &species_id, &reason)?;
        }
    }
    mark_runtime_presentation_dirty(runtime_shell);
    Ok(())
}

fn visible_send_out_side_is_shiny(
    runtime_shell: &BevyRuntimeShell,
    side: crate::core::battle::turn::BattleSide,
) -> Result<bool> {
    let snapshot = runtime_shell.shell.snapshot()?;
    let Some(battle) = snapshot.battle.as_ref() else {
        return Ok(false);
    };
    let pokemon = match side {
        crate::core::battle::turn::BattleSide::Enemy => &battle.enemy_pokemon,
        crate::core::battle::turn::BattleSide::Player => {
            let Some(active_index) = battle.active_player_party_index else {
                return Ok(false);
            };
            let Some(slot) = snapshot
                .party
                .slots
                .iter()
                .find(|slot| slot.index == active_index)
            else {
                return Ok(false);
            };
            &slot.pokemon
        }
    };
    Ok(visible_pokemon_is_shiny(pokemon))
}

fn start_visible_enemy_frontpic_animation(
    runtime_shell: &mut BevyRuntimeShell,
    speed: u16,
) -> Result<()> {
    let snapshot = runtime_shell.shell.snapshot()?;
    let species_id = snapshot
        .battle
        .as_ref()
        .context("enemy frontpic animation requires an active battle")?
        .enemy_pokemon
        .species
        .id
        .clone();
    snapshot
        .presentation
        .pokemon_frontpic_anim
        .get(&species_id)
        .with_context(|| format!("missing exported frontpic animation for {species_id}"))?;
    runtime_shell.visible_frontpic_animation = Some(VisibleFrontpicAnimation {
        species_id,
        speed,
        pointer: 0,
        repeat: 0,
        wait: 0,
        frame: 0,
    });
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(())
}

fn advance_visible_frontpic_animation(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let Some(mut animation) = runtime_shell.visible_frontpic_animation.take() else {
        return Ok(());
    };
    let snapshot = cached_runtime_snapshot(runtime_shell)?;
    let program = snapshot
        .presentation
        .pokemon_frontpic_anim
        .get(&animation.species_id)
        .with_context(|| {
            format!(
                "missing exported frontpic animation for {}",
                animation.species_id
            )
        })?
        .clone();
    if !step_visible_frontpic_animation(&mut animation, &program)? {
        runtime_shell.visible_frontpic_animation = Some(animation);
    }
    mark_runtime_presentation_dirty(runtime_shell);
    Ok(())
}

fn step_visible_frontpic_animation(
    animation: &mut VisibleFrontpicAnimation,
    program: &crate::core::models::frontpic_anim::FrontpicAnimProgram,
) -> Result<bool> {
    crystal_runtime::frontpic_animation::step_frontpic_animation(animation, program)
}

fn advance_visible_fishing_animation(runtime_shell: &mut BevyRuntimeShell) {
    let Some(animation) = runtime_shell.visible_fishing_animation.as_mut() else {
        return;
    };
    let mut result_text = None;
    match animation.phase {
        VisibleFishingPhase::Cast => {
            animation.frame = animation.frame.saturating_add(1);
            if animation.frame >= 40 {
                animation.frame = 0;
                if animation.bite {
                    animation.phase = VisibleFishingPhase::Hook;
                } else {
                    animation.phase = VisibleFishingPhase::AwaitText;
                    result_text = Some("Not even a nibble!".to_string());
                }
            }
        }
        VisibleFishingPhase::Hook => {
            // Four fish_got_bite movement commands, each held for one
            // ordinary eight-frame overworld movement cadence. Facing Up's
            // source movement adds `step_sleep 1` before `show_emote`.
            animation.frame = animation.frame.saturating_add(1);
            let hook_frames = if animation.facing_up { 33 } else { 32 };
            if animation.frame >= hook_frames {
                animation.frame = 0;
                animation.phase = VisibleFishingPhase::Pause;
            }
        }
        VisibleFishingPhase::Pause => {
            animation.frame = animation.frame.saturating_add(1);
            if animation.frame >= 40 {
                animation.frame = 0;
                animation.phase = VisibleFishingPhase::AwaitText;
                result_text = Some("Oh!\nA bite!".to_string());
            }
        }
        VisibleFishingPhase::AwaitText => return,
    }
    if let Some(text) = result_text {
        runtime_shell.field_notice = Some(text);
        runtime_shell.pending_field_battle_entry = runtime_shell
            .visible_fishing_animation
            .as_ref()
            .is_some_and(|animation| animation.starts_battle);
    }
    mark_runtime_snapshot_dirty(runtime_shell);
}

fn advance_visible_trainer_exit_animation(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let Some(animation) = runtime_shell.visible_trainer_exit_animation.as_mut() else {
        return Ok(());
    };
    animation.frame = animation.frame.saturating_add(1);
    if animation.frame >= animation.total_frames() {
        let start_send_out = animation.send_out_after;
        let side = animation.side;
        runtime_shell.visible_trainer_exit_animation = None;
        if start_send_out {
            let shiny = visible_send_out_side_is_shiny(runtime_shell, side)?;
            runtime_shell.battle_fainted_hud[usize::from(side == crate::core::battle::turn::BattleSide::Enemy)] = false;
            runtime_shell.visible_send_out_animation = Some(VisibleSendOutAnimation {
                side,
                frame: 0,
                shiny,
            });
            queue_visible_shell_sound_effect(runtime_shell, "SFX_BALL_POOF")?;
        }
    }
    mark_runtime_presentation_dirty(runtime_shell);
    Ok(())
}

fn reset_visible_battle_item_cursors(runtime_shell: &mut BevyRuntimeShell) {
    runtime_shell.bag_cursor = None;
    runtime_shell.key_item_cursor = None;
    runtime_shell.ball_cursor = None;
    runtime_shell.tmhm_cursor = None;
    runtime_shell.custom_item_cursor = None;
    runtime_shell.field_pack_pocket = None;
    runtime_shell.field_pack_action_cursor = None;
    runtime_shell.field_pack_target_mode = None;
    runtime_shell.battle_pack_target_mode = None;
    runtime_shell.party_move_cursor = None;
    runtime_shell.battle_party_action_cursor = None;
    runtime_shell.battle_party_summary_open = false;
}

fn prepare_visible_local_link_descriptor(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let snapshot = runtime_shell.shell.snapshot()?;
    let session_id = format!("bevy-local-{}", snapshot.state_checksum.frame());
    let descriptor = visible_local_link_descriptor(runtime_shell, session_id.clone())?;
    let checkpoint = descriptor.save_checkpoint.checkpoint();
    if checkpoint.summary().state_hash() != descriptor.checksum.hash()
        || checkpoint.checksum().hash() != descriptor.checksum.hash()
        || checkpoint.summary().state_frame() != descriptor.checksum.frame()
        || checkpoint.checksum().frame() != descriptor.checksum.frame()
    {
        anyhow::bail!(
            "runtime link descriptor checkpoint does not match current state checksum: summary frame/hash {} {:#010x}, checkpoint frame/hash {} {:#010x}, current frame/hash {} {:#010x}",
            checkpoint.summary().state_frame(),
            checkpoint.summary().state_hash(),
            checkpoint.checksum().frame(),
            checkpoint.checksum().hash(),
            descriptor.checksum.frame(),
            descriptor.checksum.hash()
        );
    }
    let journal = runtime_shell.shell.local_input_journal(
        &descriptor,
        descriptor.checksum.clone(),
        std::iter::empty(),
    )?;
    let journal_bytes = journal.journal.canonical_bytes()?;
    let journal_frame_count = journal.journal.frames().len();
    let journal_message = runtime_shell.shell.input_journal_message(journal.clone())?;
    let journal_message_bytes = encode_link_message_bytes(&journal_message)?;
    let save_resume_message = runtime_shell.shell.save_resume_replay_message(
        &descriptor,
        journal,
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )?;
    let save_resume_message_bytes = encode_link_message_bytes(&save_resume_message)?;
    let retained_replay = visible_retained_save_resume_replay_bundle(runtime_shell)?;
    let retained_direct_replay_message =
        LinkMessage::DeterministicReplay(retained_replay.replay().clone());
    let retained_direct_replay_message_bytes =
        encode_link_message_bytes(&retained_direct_replay_message)?;
    let retained_replay_message = LinkMessage::SaveResumeReplay(retained_replay.clone());
    let retained_replay_message_bytes = encode_link_message_bytes(&retained_replay_message)?;
    let retained_input_message_bytes = encode_retained_input_messages(runtime_shell)?;
    let retained_battle_action_message_bytes =
        encode_retained_battle_action_messages(runtime_shell)?;
    let retained_menu_choice_message_bytes = encode_retained_menu_choice_messages(runtime_shell)?;
    let retained_menu_result_message_bytes = encode_retained_menu_result_messages(runtime_shell)?;
    let retained_runtime_command_message_bytes =
        encode_retained_runtime_command_messages(runtime_shell)?;
    let retained_runtime_result_message_bytes =
        encode_retained_runtime_result_messages(runtime_shell)?;
    let retained_state_hash_message_bytes = encode_retained_state_hash_messages(runtime_shell)?;
    let retained_journal_frames = retained_replay
        .replay()
        .input_journal()
        .journal()
        .frames()
        .len();
    let deterministic_session_checkpoint =
        required_visible_deterministic_session_checkpoint(runtime_shell)?;
    let retained_session_id = deterministic_session_checkpoint
        .session()
        .session_id()
        .to_string();
    let retained_checkpoint_frame = deterministic_session_checkpoint
        .checkpoint()
        .summary()
        .state_frame();
    let retained_checkpoint_hash = deterministic_session_checkpoint
        .checkpoint()
        .summary()
        .state_hash();
    record_visible_runtime_action(
        runtime_shell,
        format!(
            "link:descriptor:{}:{}:{}:{}:{}:{}:{:#010x}",
            descriptor.session.session_id(),
            descriptor.local_player.id(),
            descriptor.session.modpack().id(),
            descriptor.session.modpack().hash(),
            descriptor.session.pack_content_hash(),
            descriptor.checksum.frame(),
            descriptor.checksum.hash()
        ),
    )?;
    runtime_shell.last_audio_events.push(format!(
            "link descriptor session={} player={} checksum_frame={} checksum_hash={:#010x} checkpoint_frame={} checkpoint_hash={:#010x} journal_frames={} journal_bytes={} journal_msg_bytes={} save_resume_msg_bytes={} retained_session={} retained_input_start={} retained_input_start_hash={:#010x} retained_checkpoint_frame={} retained_checkpoint_hash={:#010x} retained_inputs={} retained_journal_frames={} retained_battle_actions={} retained_menu_results={} retained_runtime_commands={} retained_runtime_results={} retained_state_hash_msg_bytes={} retained_direct_replay_msg_bytes={} retained_replay_msg_bytes={} retained_input_msg_bytes={} retained_battle_action_msg_bytes={} retained_menu_choice_msg_bytes={} retained_menu_result_msg_bytes={} retained_runtime_command_msg_bytes={} retained_runtime_result_msg_bytes={}",
        session_id,
        descriptor.local_player.id(),
        descriptor.checksum.frame(),
        descriptor.checksum.hash(),
        descriptor.save_checkpoint.checkpoint().summary().state_frame(),
        descriptor.save_checkpoint.checkpoint().summary().state_hash(),
        journal_frame_count,
        journal_bytes.len(),
        journal_message_bytes.len(),
        save_resume_message_bytes.len(),
        retained_session_id,
        runtime_shell.deterministic_session_start.frame(),
        runtime_shell.deterministic_session_start.hash(),
        retained_checkpoint_frame,
        retained_checkpoint_hash,
        runtime_shell.deterministic_input_frames.len(),
        retained_journal_frames,
        runtime_shell.deterministic_battle_actions.len(),
        runtime_shell.deterministic_menu_results.len(),
        runtime_shell.shell.retained_runtime_commands().len(),
        runtime_shell.shell.retained_runtime_results().len(),
        retained_state_hash_message_bytes,
        retained_direct_replay_message_bytes.len(),
        retained_replay_message_bytes.len(),
        retained_input_message_bytes,
        retained_battle_action_message_bytes,
        retained_menu_choice_message_bytes,
        retained_menu_result_message_bytes,
        retained_runtime_command_message_bytes,
        retained_runtime_result_message_bytes
    ));
    trim_event_log(&mut runtime_shell.last_audio_events);
    Ok(())
}

fn encode_retained_input_messages(runtime_shell: &BevyRuntimeShell) -> Result<usize> {
    let terminal_frame = runtime_shell
        .shell
        .state_checksum_frame(LOCAL_PLAYER_ID)
        .context("checksum current runtime state for retained input stream")?
        .frame();
    let input_masks = retained_input_masks_by_frame(
        runtime_shell,
        runtime_shell.deterministic_session_start.frame(),
        terminal_frame,
    )?;
    let mut byte_count = 0usize;
    for input in retained_input_frames_from_masks(
        runtime_shell.deterministic_session_start.frame(),
        terminal_frame,
        &input_masks,
    )? {
        let message = LinkMessage::Input(input.clone());
        byte_count = byte_count
            .checked_add(encode_link_message_bytes(&message)?.len())
            .context("retained input message byte count overflow")?;
    }
    Ok(byte_count)
}

fn retained_input_masks_by_frame(
    runtime_shell: &BevyRuntimeShell,
    start_frame: u64,
    terminal_frame: u64,
) -> Result<BTreeMap<u64, u8>> {
    let mut masks = BTreeMap::new();
    for input in &runtime_shell.deterministic_input_frames {
        if input.player_id() != LOCAL_PLAYER_ID {
            anyhow::bail!(
                "retained input has player {}, expected {}",
                input.player_id(),
                LOCAL_PLAYER_ID
            );
        }
        if input.frame() < start_frame || input.frame() >= terminal_frame {
            anyhow::bail!(
                "retained input frame {} is outside session frame range {}..{}",
                input.frame(),
                start_frame,
                terminal_frame
            );
        }
        if masks.insert(input.frame(), input.joypad_mask()).is_some() {
            anyhow::bail!(
                "retained input frame {} appears more than once",
                input.frame()
            );
        }
    }
    Ok(masks)
}

fn retained_input_frames_from_masks(
    start_frame: u64,
    terminal_frame: u64,
    input_masks: &BTreeMap<u64, u8>,
) -> Result<Vec<PlayerInputFrame>> {
    if terminal_frame < start_frame {
        anyhow::bail!(
            "retained input terminal frame {terminal_frame} is before start frame {start_frame}"
        );
    }
    let mut frames = Vec::with_capacity((terminal_frame - start_frame) as usize);
    for frame in start_frame..terminal_frame {
        frames.push(
            PlayerInputFrame::new(
                LOCAL_PLAYER_ID,
                Frame(frame),
                input_masks.get(&frame).copied().unwrap_or(0),
            )
            .context("build retained input message frame")?,
        );
    }
    Ok(frames)
}

fn retained_lockstep_frames_from_masks(
    start_frame: u64,
    terminal_frame: u64,
    input_masks: &BTreeMap<u64, u8>,
) -> Result<Vec<LockstepFrame>> {
    retained_input_frames_from_masks(start_frame, terminal_frame, input_masks)?
        .into_iter()
        .map(|input| {
            LockstepFrame::new(
                input.frame(),
                BTreeMap::from([(LOCAL_PLAYER_ID, input.joypad_mask())]),
            )
            .context("build retained deterministic lockstep frame")
        })
        .collect()
}

fn encode_retained_state_hash_messages(runtime_shell: &BevyRuntimeShell) -> Result<usize> {
    let start = required_visible_deterministic_session_checkpoint(runtime_shell)?
        .checkpoint()
        .checksum()
        .clone();
    let current = runtime_shell
        .shell
        .state_checksum_frame(LOCAL_PLAYER_ID)
        .context("checksum current runtime state for retained state hash stream")?;
    validate_retained_state_hash_stream(&start, &current)?;
    let mut byte_count = 0usize;
    for checksum in [start, current] {
        let message = LinkMessage::StateHash(StateChecksumFrame::new(
            checksum.player_id(),
            Frame(checksum.frame()),
            checksum.hash(),
        ));
        byte_count = byte_count
            .checked_add(encode_link_message_bytes(&message)?.len())
            .context("retained state hash message byte count overflow")?;
    }
    Ok(byte_count)
}

fn validate_retained_state_hash_stream(
    start: &StateChecksumFrame,
    current: &StateChecksumFrame,
) -> Result<()> {
    if start.player_id() != LOCAL_PLAYER_ID || current.player_id() != LOCAL_PLAYER_ID {
        anyhow::bail!(
            "retained state hash players {}/{} do not match local player {}",
            start.player_id(),
            current.player_id(),
            LOCAL_PLAYER_ID
        );
    }
    if current.frame() < start.frame() {
        anyhow::bail!(
            "retained state hash current frame {} is before start frame {}",
            current.frame(),
            start.frame()
        );
    }
    Ok(())
}

fn encode_retained_battle_action_messages(runtime_shell: &BevyRuntimeShell) -> Result<usize> {
    let mut byte_count = 0usize;
    let start_frame = runtime_shell.deterministic_session_start.frame();
    let terminal_frame = runtime_shell
        .shell
        .state_checksum_frame(LOCAL_PLAYER_ID)
        .context("checksum current runtime state for retained battle action stream")?
        .frame();
    validate_retained_battle_actions(runtime_shell, start_frame, terminal_frame)?;
    for action in &runtime_shell.deterministic_battle_actions {
        let message = LinkMessage::BattleAction(action.clone());
        byte_count = byte_count
            .checked_add(encode_link_message_bytes(&message)?.len())
            .context("retained battle action message byte count overflow")?;
    }
    Ok(byte_count)
}

fn validate_retained_battle_actions(
    runtime_shell: &BevyRuntimeShell,
    start_frame: u64,
    terminal_frame: u64,
) -> Result<()> {
    let mut previous_turn = None;
    for action in &runtime_shell.deterministic_battle_actions {
        action
            .validate()
            .context("validate retained battle action before replay")?;
        if action.player_id() != LOCAL_PLAYER_ID {
            anyhow::bail!(
                "retained battle action has player {}, expected {}",
                action.player_id(),
                LOCAL_PLAYER_ID
            );
        }
        if action.turn() < start_frame || action.turn() > terminal_frame {
            anyhow::bail!(
                "retained battle action turn {} is outside session frame range {}..={}",
                action.turn(),
                start_frame,
                terminal_frame
            );
        }
        validate_retained_battle_action_order(previous_turn, action.turn())?;
        previous_turn = Some(action.turn());
    }
    Ok(())
}

fn validate_retained_battle_action_order(previous_turn: Option<u64>, turn: u64) -> Result<()> {
    if let Some(previous) = previous_turn {
        if turn <= previous {
            anyhow::bail!(
                "retained battle action turn {} is not strictly after previous turn {}",
                turn,
                previous
            );
        }
    }
    Ok(())
}

fn encode_retained_menu_choice_messages(runtime_shell: &BevyRuntimeShell) -> Result<usize> {
    let start_frame = runtime_shell.deterministic_session_start.frame();
    let terminal_frame = runtime_shell
        .shell
        .state_checksum_frame(LOCAL_PLAYER_ID)
        .context("checksum current runtime state for retained menu choice stream")?
        .frame();
    validate_retained_menu_results(runtime_shell, start_frame, terminal_frame)?;
    let mut byte_count = 0usize;
    for result in &runtime_shell.deterministic_menu_results {
        let choice = result.choice();
        let message = LinkMessage::MenuChoice(choice.clone());
        byte_count = byte_count
            .checked_add(encode_link_message_bytes(&message)?.len())
            .context("retained menu choice message byte count overflow")?;
    }
    Ok(byte_count)
}

fn encode_retained_menu_result_messages(runtime_shell: &BevyRuntimeShell) -> Result<usize> {
    let start_frame = runtime_shell.deterministic_session_start.frame();
    let terminal_frame = runtime_shell
        .shell
        .state_checksum_frame(LOCAL_PLAYER_ID)
        .context("checksum current runtime state for retained menu result stream")?
        .frame();
    validate_retained_menu_results(runtime_shell, start_frame, terminal_frame)?;
    let mut byte_count = 0usize;
    for result in &runtime_shell.deterministic_menu_results {
        let message = LinkMessage::MenuChoiceResult(result.clone());
        byte_count = byte_count
            .checked_add(encode_link_message_bytes(&message)?.len())
            .context("retained menu result message byte count overflow")?;
    }
    Ok(byte_count)
}

fn validate_retained_menu_results(
    runtime_shell: &BevyRuntimeShell,
    start_frame: u64,
    terminal_frame: u64,
) -> Result<()> {
    let mut previous_choice_frame = None;
    for result in &runtime_shell.deterministic_menu_results {
        result
            .validate()
            .context("validate retained menu result before replay")?;
        if result.choice().player_id() != LOCAL_PLAYER_ID
            || result.checksum().player_id() != LOCAL_PLAYER_ID
        {
            anyhow::bail!(
                "retained menu result has choice/checksum players {}/{}, expected {}",
                result.choice().player_id(),
                result.checksum().player_id(),
                LOCAL_PLAYER_ID
            );
        }
        let choice_frame = result.choice().frame();
        let checksum_frame = result.checksum().frame();
        validate_retained_frame_pair(
            "retained menu result",
            choice_frame,
            checksum_frame,
            start_frame,
            terminal_frame,
        )?;
        validate_retained_menu_choice_order(previous_choice_frame, choice_frame)?;
        previous_choice_frame = Some(choice_frame);
    }
    Ok(())
}

fn validate_retained_menu_choice_order(
    previous_choice_frame: Option<u64>,
    choice_frame: u64,
) -> Result<()> {
    if let Some(previous) = previous_choice_frame {
        if choice_frame <= previous {
            anyhow::bail!(
                "retained menu choice frame {} is not strictly after previous choice frame {}",
                choice_frame,
                previous
            );
        }
    }
    Ok(())
}

fn validate_retained_frame_pair(
    label: &str,
    first_frame: u64,
    second_frame: u64,
    start_frame: u64,
    terminal_frame: u64,
) -> Result<()> {
    if first_frame < start_frame
        || first_frame > terminal_frame
        || second_frame < start_frame
        || second_frame > terminal_frame
    {
        anyhow::bail!(
            "{label} frames first={} second={} outside session frame range {}..={}",
            first_frame,
            second_frame,
            start_frame,
            terminal_frame
        );
    }
    Ok(())
}

fn encode_retained_runtime_command_messages(runtime_shell: &BevyRuntimeShell) -> Result<usize> {
    let start_frame = runtime_shell.deterministic_session_start.frame();
    let terminal_frame = runtime_shell
        .shell
        .state_checksum_frame(LOCAL_PLAYER_ID)
        .context("checksum current runtime state for retained runtime command stream")?
        .frame();
    validate_retained_runtime_commands(runtime_shell, start_frame, terminal_frame)?;
    let mut byte_count = 0usize;
    for command in visible_retained_runtime_command_frames(
        runtime_shell,
        required_visible_deterministic_session_checkpoint(runtime_shell)?,
    )? {
        let message = LinkMessage::SessionRuntimeCommand(command);
        byte_count = byte_count
            .checked_add(encode_link_message_bytes(&message)?.len())
            .context("retained runtime command message byte count overflow")?;
    }
    Ok(byte_count)
}

fn validate_retained_runtime_commands(
    runtime_shell: &BevyRuntimeShell,
    start_frame: u64,
    terminal_frame: u64,
) -> Result<()> {
    let mut previous_sequence = None;
    for command in runtime_shell.shell.retained_runtime_commands() {
        command
            .validate()
            .context("validate retained runtime command before replay")?;
        if command.player_id() != LOCAL_PLAYER_ID {
            anyhow::bail!(
                "retained runtime command has player {}, expected {}",
                command.player_id(),
                LOCAL_PLAYER_ID
            );
        }
        if command.expected_state().frame() < start_frame
            || command.expected_state().frame() > terminal_frame
        {
            anyhow::bail!(
                "retained runtime command sequence {} expected frame {} outside session frame range {}..={}",
                command.sequence(),
                command.expected_state().frame(),
                start_frame,
                terminal_frame
            );
        }
        if let Some(previous) = previous_sequence {
            if command.sequence() <= previous {
                anyhow::bail!(
                    "retained runtime command sequence {} is not strictly after previous sequence {}",
                    command.sequence(),
                    previous
                );
            }
        }
        previous_sequence = Some(command.sequence());
    }
    Ok(())
}

fn encode_retained_runtime_result_messages(runtime_shell: &BevyRuntimeShell) -> Result<usize> {
    let start_frame = runtime_shell.deterministic_session_start.frame();
    let terminal_frame = runtime_shell
        .shell
        .state_checksum_frame(LOCAL_PLAYER_ID)
        .context("checksum current runtime state for retained runtime command result stream")?
        .frame();
    validate_retained_runtime_results(runtime_shell, start_frame, terminal_frame)?;
    let mut byte_count = 0usize;
    for result in visible_retained_runtime_result_frames(
        runtime_shell,
        required_visible_deterministic_session_checkpoint(runtime_shell)?,
    )? {
        let message = LinkMessage::SessionRuntimeCommandResult(result);
        byte_count = byte_count
            .checked_add(encode_link_message_bytes(&message)?.len())
            .context("retained runtime command result message byte count overflow")?;
    }
    Ok(byte_count)
}

fn validate_retained_runtime_results(
    runtime_shell: &BevyRuntimeShell,
    start_frame: u64,
    terminal_frame: u64,
) -> Result<()> {
    let commands = runtime_shell.shell.retained_runtime_commands();
    let results = runtime_shell.shell.retained_runtime_results();
    if results.len() != commands.len() {
        anyhow::bail!(
            "retained runtime command/result count mismatch: commands={} results={}",
            commands.len(),
            results.len()
        );
    }
    let mut previous_sequence = None;
    for (index, result) in results.iter().enumerate() {
        result
            .validate()
            .context("validate retained runtime command result before replay")?;
        let command = &commands[index];
        if result.request() != command {
            anyhow::bail!(
                "retained runtime command result at index {} is for sequence {}, expected command sequence {}",
                index,
                result.request().sequence(),
                command.sequence()
            );
        }
        if result.request().player_id() != LOCAL_PLAYER_ID
            || result.checksum().player_id() != LOCAL_PLAYER_ID
        {
            anyhow::bail!(
                "retained runtime command result has request/checksum players {}/{}, expected {}",
                result.request().player_id(),
                result.checksum().player_id(),
                LOCAL_PLAYER_ID
            );
        }
        let request_frame = result.request().expected_state().frame();
        let checksum_frame = result.checksum().frame();
        validate_retained_frame_pair(
            &format!(
                "retained runtime command result sequence {}",
                result.request().sequence()
            ),
            request_frame,
            checksum_frame,
            start_frame,
            terminal_frame,
        )?;
        if let Some(previous) = previous_sequence {
            if result.request().sequence() <= previous {
                anyhow::bail!(
                    "retained runtime command result sequence {} is not strictly after previous sequence {}",
                    result.request().sequence(),
                    previous
                );
            }
        }
        previous_sequence = Some(result.request().sequence());
    }
    Ok(())
}

fn visible_retained_save_resume_replay_bundle(
    runtime_shell: &BevyRuntimeShell,
) -> Result<SaveResumeReplayBundle> {
    let checkpoint = required_visible_deterministic_session_checkpoint(runtime_shell)?.clone();
    let start_checksum = checkpoint.checkpoint().checksum().clone();
    let terminal_checksum = runtime_shell
        .shell
        .state_checksum_frame(LOCAL_PLAYER_ID)
        .context("checksum current runtime state for retained deterministic replay")?;
    let start_frame = start_checksum.frame();
    let terminal_frame = terminal_checksum.frame();
    terminal_frame.checked_sub(start_frame).with_context(|| {
        format!(
            "retained deterministic replay terminal frame {terminal_frame} is before start frame {start_frame}"
        )
    })?;
    let input_masks = retained_input_masks_by_frame(runtime_shell, start_frame, terminal_frame)?;
    validate_retained_runtime_commands(runtime_shell, start_frame, terminal_frame)?;
    validate_retained_runtime_results(runtime_shell, start_frame, terminal_frame)?;
    validate_retained_menu_results(runtime_shell, start_frame, terminal_frame)?;
    let frames = retained_lockstep_frames_from_masks(start_frame, terminal_frame, &input_masks)?;

    let journal = DeterministicInputJournal::new(
        checkpoint.session().clone(),
        [LOCAL_PLAYER_ID],
        start_checksum,
        terminal_checksum.clone(),
        frames,
    )
    .context("build retained deterministic input journal")?;
    let journal_frame = DeterministicInputJournalFrame::new(journal)
        .context("fingerprint retained deterministic input journal")?;
    let runtime_commands = visible_retained_runtime_command_frames(runtime_shell, &checkpoint)?;
    let runtime_results = visible_retained_runtime_result_frames(runtime_shell, &checkpoint)?;
    let replay = DeterministicReplayBundle::new(
        journal_frame,
        runtime_commands,
        runtime_results,
        runtime_shell
            .deterministic_menu_results
            .iter()
            .cloned()
            .collect(),
        terminal_checksum,
    )
    .context("build retained deterministic replay bundle")?;
    crate::validate_deterministic_replay_runtime_authority(&replay, LOCAL_PLAYER_ID)
        .context("validate retained runtime replay command authority")?;
    SaveResumeReplayBundle::new(checkpoint, replay)
        .context("build retained save-resume replay bundle")
}

fn visible_retained_runtime_command_frames(
    runtime_shell: &BevyRuntimeShell,
    checkpoint: &SessionSaveCheckpointFrame,
) -> Result<Vec<SessionRuntimeCommandFrame>> {
    runtime_shell
        .shell
        .retained_runtime_commands()
        .iter()
        .cloned()
        .map(|command| {
            SessionRuntimeCommandFrame::new(checkpoint.session().clone(), command)
                .context("bind retained runtime command to deterministic session")
        })
        .collect()
}

fn visible_retained_runtime_result_frames(
    runtime_shell: &BevyRuntimeShell,
    checkpoint: &SessionSaveCheckpointFrame,
) -> Result<Vec<SessionRuntimeCommandResultFrame>> {
    runtime_shell
        .shell
        .retained_runtime_results()
        .iter()
        .cloned()
        .map(|result| {
            SessionRuntimeCommandResultFrame::new(checkpoint.session().clone(), result)
                .context("bind retained runtime command result to deterministic session")
        })
        .collect()
}

fn switch_visible_pc_move_container(
    runtime_shell: &mut BevyRuntimeShell,
    delta: isize,
) -> Result<()> {
    let container_count = crate::core::models::MAX_PC_BOXES + 1;
    let current = if runtime_shell.bill_pc_move_party_open {
        0
    } else {
        runtime_shell.bill_pc_move_loaded_box + 1
    };
    let next = wrapped_index(current, container_count, delta);
    if next == 0 {
        runtime_shell.bill_pc_move_party_open = true;
        runtime_shell.pc_list_scroll = 0;
        runtime_shell.storage_cursor = Some(MenuCursor {
            surface_id: pc_party_surface_id().to_string(),
            option_index: 0,
        });
        set_shell_action_status(runtime_shell, "PARTY");
        return Ok(());
    }
    record_visible_runtime_action(runtime_shell, format!("pc:move:browse_box:{}", next - 1))?;
    runtime_shell.bill_pc_move_loaded_box = next - 1;
    runtime_shell.bill_pc_move_party_open = false;
    runtime_shell.pc_list_scroll = 0;
    runtime_shell.storage_cursor = Some(MenuCursor {
        surface_id: storage_cursor_surface_id(runtime_shell.bill_pc_move_loaded_box),
        option_index: 0,
    });
    mark_runtime_snapshot_dirty(runtime_shell);
    set_shell_action_status(
        runtime_shell,
        format!("BOX {}", runtime_shell.bill_pc_move_loaded_box + 1),
    );
    Ok(())
}

fn ensure_no_visible_special_boundary(runtime_shell: &BevyRuntimeShell) -> Result<()> {
    if let Some(boundary) = runtime_shell.special_boundary.as_ref() {
        anyhow::bail!(
            "active special boundary {} blocks this action",
            boundary.label
        );
    }
    Ok(())
}

fn deposit_visible_party_pokemon(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    ensure_no_visible_special_boundary(runtime_shell)?;
    let snapshot = runtime_shell.shell.snapshot()?;
    let party_index = selected_party_index(runtime_shell)?;
    let slot = snapshot
        .party
        .slots
        .iter()
        .find(|slot| slot.index == party_index)
        .with_context(|| format!("selected party index {party_index} is not in the party"))?;
    if snapshot.storage.party_count <= 1 {
        record_visible_runtime_action(
            runtime_shell,
            format!("pc:deposit_party:{party_index}:last_pokemon"),
        )?;
        runtime_shell
            .last_audio_events
            .push("cannot deposit the last Pokemon".to_string());
        begin_visible_pc_transfer_refusal(
            runtime_shell,
            VisiblePcTransferKind::Deposit,
            snapshot.storage.current_pc_box,
            "It's your last <PK><MN>!",
            true,
        )?;
        set_shell_action_status(runtime_shell, "CAN'T DEPOSIT LAST POKEMON");
        trim_event_log(&mut runtime_shell.last_audio_events);
        return Ok(());
    }
    let has_other_usable_pokemon = snapshot.party.slots.iter().any(|other| {
        other.index != party_index
            && other.pokemon.hp > 0
            && !other.pokemon.is_egg
            && other.pokemon.species.id != "EGG"
    });
    if !has_other_usable_pokemon {
        record_visible_runtime_action(
            runtime_shell,
            format!("pc:deposit_party:{party_index}:no_more_usable_pokemon"),
        )?;
        begin_visible_pc_transfer_refusal(
            runtime_shell,
            VisiblePcTransferKind::Deposit,
            snapshot.storage.current_pc_box,
            "No more usable <PK><MN>!",
            true,
        )?;
        set_shell_action_status(runtime_shell, "NO MORE USABLE POKEMON");
        return Ok(());
    }
    if slot
        .pokemon
        .item
        .as_deref()
        .is_some_and(crate::core::models::item::is_mail_item_id)
    {
        record_visible_runtime_action(
            runtime_shell,
            format!("pc:deposit_party:{party_index}:remove_mail"),
        )?;
        begin_visible_pc_transfer_refusal(
            runtime_shell,
            VisiblePcTransferKind::Deposit,
            snapshot.storage.current_pc_box,
            "Remove MAIL.",
            true,
        )?;
        set_shell_action_status(runtime_shell, "REMOVE MAIL");
        return Ok(());
    }
    let current_box = visible_storage_box(&snapshot, runtime_shell)?;
    if current_box.count >= crate::core::models::MAX_BOX_MONS {
        record_visible_runtime_action(
            runtime_shell,
            format!(
                "pc:deposit_party:{party_index}:box_full:{}",
                snapshot.storage.current_pc_box
            ),
        )?;
        runtime_shell.last_audio_events.push(format!(
            "PC box {} is full",
            snapshot.storage.current_pc_box
        ));
        begin_visible_pc_transfer_refusal(
            runtime_shell,
            VisiblePcTransferKind::Deposit,
            snapshot.storage.current_pc_box,
            "The BOX is full.",
            false,
        )?;
        set_shell_action_status(runtime_shell, "BOX IS FULL");
        trim_event_log(&mut runtime_shell.last_audio_events);
        return Ok(());
    }
    if slot.is_active_battle_pokemon {
        anyhow::bail!(
            "selected party index {party_index} is active in battle and cannot be deposited"
        );
    }
    record_visible_runtime_action(runtime_shell, format!("pc:deposit_party:{party_index}"))?;
    let deposit = runtime_shell
        .shell
        .deposit_party_pokemon_to_current_box(party_index)?;
    runtime_shell.last_audio_events.push(format!(
        "pc deposit party_index={} pokemon={} box={} slot={} checksum={:?}",
        deposit.party_index,
        deposit.pokemon.species.id,
        deposit.box_index,
        deposit.box_slot,
        deposit.state_checksum
    ));
    trim_event_log(&mut runtime_shell.last_audio_events);
    if !deposit.pokemon.is_egg {
        queue_visible_pokemon_cry(runtime_shell, &deposit.pokemon.species.id, "bill_pc_deposit")?;
    }
    set_shell_action_status(
        runtime_shell,
        format!(
            "DEPOSITED {} BOX {} SLOT {}",
            deposit.pokemon.species.id, deposit.box_index, deposit.box_slot
        ),
    );
    runtime_shell.party_cursor = 0;
    close_visible_party_detail_state(runtime_shell);
    runtime_shell.pc_notice = None;
    runtime_shell.field_text_reveal = None;
    runtime_shell.pc_transfer_sequence = Some(VisiblePcTransferSequence {
        kind: VisiblePcTransferKind::Deposit,
        box_index: deposit.box_index,
        phase: VisiblePcTransferPhase::SuccessWaitCry,
        frames_remaining: 0,
        close_submenu_after_hold: false,
        success: Some(VisiblePcTransferSuccess {
            names: snapshot.party.slots.iter().map(|slot| slot.pokemon.nickname.clone()).collect(),
            pokemon: visible_pc_pokemon_info(&slot.pokemon),
            message: format!("Stored {}!", deposit.pokemon.nickname),
        }),
    });
    mark_runtime_presentation_dirty(runtime_shell);
    Ok(())
}

fn withdraw_visible_pc_pokemon(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    ensure_no_visible_special_boundary(runtime_shell)?;
    let snapshot = runtime_shell.shell.snapshot()?;
    let current_box = visible_storage_box(&snapshot, runtime_shell)?;
    if current_box.slots.is_empty() {
        record_visible_runtime_action(
            runtime_shell,
            format!(
                "pc:withdraw_pokemon:{}:empty",
                snapshot.storage.current_pc_box
            ),
        )?;
        runtime_shell.last_audio_events.push(format!(
            "PC box {} has no Pokemon",
            snapshot.storage.current_pc_box
        ));
        set_shell_action_status(runtime_shell, "BOX IS EMPTY");
        runtime_shell.pc_notice = Some("The BOX is empty.".to_string());
        mark_runtime_snapshot_dirty(runtime_shell);
        trim_event_log(&mut runtime_shell.last_audio_events);
        return Ok(());
    }
    if snapshot.storage.party_count >= crate::core::models::PARTY_SIZE {
        record_visible_runtime_action(
            runtime_shell,
            format!(
                "pc:withdraw_pokemon:{}:party_full",
                snapshot.storage.current_pc_box
            ),
        )?;
        runtime_shell
            .last_audio_events
            .push("party is full".to_string());
        begin_visible_pc_transfer_refusal(
            runtime_shell,
            VisiblePcTransferKind::Withdraw,
            snapshot.storage.current_pc_box,
            "The party's full!",
            false,
        )?;
        set_shell_action_status(runtime_shell, "PARTY IS FULL");
        trim_event_log(&mut runtime_shell.last_audio_events);
        return Ok(());
    }
    let box_slot = selected_current_box_slot_index(runtime_shell)?;
    let current_box = snapshot.storage.current_pc_box;
    let retained_pokemon = visible_pc_pokemon_info(visible_pc_pokemon_at(&snapshot,
        VisiblePcPokemonLocation::Box { box_index: current_box, box_slot })?);
    record_visible_runtime_action(
        runtime_shell,
        format!("pc:withdraw_pokemon:{current_box}:{box_slot}"),
    )?;
    let withdraw = runtime_shell
        .shell
        .withdraw_current_box_pokemon_to_party(box_slot)?;
    runtime_shell.last_audio_events.push(format!(
        "pc withdraw box={} slot={} pokemon={} party_index={} checksum={:?}",
        withdraw.box_index,
        withdraw.box_slot,
        withdraw.pokemon.species.id,
        withdraw.party_index,
        withdraw.state_checksum
    ));
    trim_event_log(&mut runtime_shell.last_audio_events);
    if !withdraw.pokemon.is_egg {
        queue_visible_pokemon_cry(runtime_shell, &withdraw.pokemon.species.id, "bill_pc_withdraw")?;
    }
    set_shell_action_status(
        runtime_shell,
        format!(
            "WITHDREW {} PARTY #{}",
            withdraw.pokemon.species.id, withdraw.party_index
        ),
    );
    runtime_shell.party_cursor = 0;
    close_visible_party_detail_state(runtime_shell);
    runtime_shell.pc_notice = None;
    runtime_shell.field_text_reveal = None;
    runtime_shell.pc_transfer_sequence = Some(VisiblePcTransferSequence {
        kind: VisiblePcTransferKind::Withdraw,
        box_index: withdraw.box_index,
        phase: VisiblePcTransferPhase::SuccessWaitCry,
        frames_remaining: 0,
        close_submenu_after_hold: false,
        success: Some(VisiblePcTransferSuccess {
            names: visible_storage_box(&snapshot, runtime_shell)?.slots.iter().map(|slot| slot.pokemon.nickname.clone()).collect(),
            pokemon: retained_pokemon,
            message: format!("Got {}!", withdraw.pokemon.nickname),
        }),
    });
    mark_runtime_presentation_dirty(runtime_shell);
    Ok(())
}

fn begin_visible_pc_transfer_refusal(
    runtime_shell: &mut BevyRuntimeShell,
    kind: VisiblePcTransferKind,
    box_index: usize,
    message: &str,
    close_submenu_after_hold: bool,
) -> Result<()> {
    runtime_shell.pc_notice = Some(message.to_string());
    runtime_shell.field_text_reveal = None;
    queue_visible_shell_sound_effect(runtime_shell, "SFX_WRONG")?;
    runtime_shell.pc_transfer_sequence = Some(VisiblePcTransferSequence {
        kind,
        box_index,
        phase: VisiblePcTransferPhase::RefusalWaitSfx,
        frames_remaining: 0,
        close_submenu_after_hold,
        success: None,
    });
    mark_runtime_presentation_dirty(runtime_shell);
    Ok(())
}

fn advance_visible_pc_transfer_sequence(
    runtime_shell: &mut BevyRuntimeShell,
    elapsed_frames: u32,
) -> Result<()> {
    if runtime_shell
        .pc_transfer_sequence
        .as_ref()
        .is_some_and(|active| matches!(active.phase,
            VisiblePcTransferPhase::RefusalWaitSfx | VisiblePcTransferPhase::SuccessWaitCry))
    {
        if !visible_wait_sfx_finished(runtime_shell) {
            return Ok(());
        }
        let active = runtime_shell
            .pc_transfer_sequence
            .as_mut()
            .context("PC transfer disappeared after its sound completed")?;
        if active.phase == VisiblePcTransferPhase::SuccessWaitCry {
            runtime_shell.pc_notice = Some(active.success.as_ref()
                .context("PC transfer is missing its retained success presentation")?.message.clone());
            active.phase = VisiblePcTransferPhase::SuccessHold;
        } else {
            active.phase = VisiblePcTransferPhase::RefusalHold;
        }
        active.frames_remaining = 50;
        mark_runtime_presentation_dirty(runtime_shell);
        return Ok(());
    }

    let Some(active) = runtime_shell.pc_transfer_sequence.as_mut() else {
        return Ok(());
    };
    let elapsed = elapsed_frames.min(u32::from(active.frames_remaining));
    active.frames_remaining = active
        .frames_remaining
        .saturating_sub(u8::try_from(elapsed).unwrap_or(u8::MAX));
    if active.frames_remaining > 0 {
        return Ok(());
    }

    let finished = runtime_shell
        .pc_transfer_sequence
        .take()
        .context("PC transfer sequence disappeared at its final frame")?;
    let snapshot = runtime_shell.shell.snapshot()?;
    anyhow::ensure!(
        snapshot.storage.current_pc_box == finished.box_index,
        "PC box changed during the locked transfer sequence"
    );
    runtime_shell.pc_notice = None;
    runtime_shell.field_text_reveal = None;
    if finished.phase == VisiblePcTransferPhase::RefusalHold {
        // Admission checks return carry only after WaitSFX + DelayFrames.
        // The caller then cancels the submenu; capacity failures retain it.
        if finished.close_submenu_after_hold {
            runtime_shell.bill_pc_pokemon_action_cursor = None;
        }
        mark_runtime_presentation_dirty(runtime_shell);
        return Ok(());
    }
    runtime_shell.bill_pc_pokemon_action_cursor = None;
    match finished.kind {
        VisiblePcTransferKind::Deposit => {
            runtime_shell.party_cursor = 0;
            runtime_shell.pc_list_scroll = 0;
            runtime_shell.storage_cursor = Some(MenuCursor {
                surface_id: pc_party_surface_id().to_string(),
                option_index: 0,
            });
            runtime_shell.party_menu_open = false;
        }
        VisiblePcTransferKind::Withdraw => {
            visible_storage_box(&snapshot, runtime_shell)?;
            // _WithdrawPKMN.Init always rebuilds CopyBoxmonSpecies, whose
            // CANCEL sentinel remains selectable after the last withdrawal.
            runtime_shell.pc_list_scroll = 0;
            runtime_shell.storage_cursor = Some(MenuCursor {
                surface_id: storage_cursor_surface_id(finished.box_index),
                option_index: 0,
            });
        }
        VisiblePcTransferKind::BoxPrint => {}
    }
    set_shell_action_status(runtime_shell, "BILL'S PC");
    mark_runtime_presentation_dirty(runtime_shell);
    Ok(())
}

fn deposit_visible_selected_pack_item_to_pc(
    runtime_shell: &mut BevyRuntimeShell,
    stack_index: usize,
    quantity: u16,
) -> Result<()> {
    ensure_no_visible_special_boundary(runtime_shell)?;
    let item_id = selected_field_pack_item_id(runtime_shell)?;
    let item_name = item_display_name(&runtime_shell.shell.snapshot()?, &item_id);
    record_visible_runtime_action(
        runtime_shell,
        format!("pc:deposit_item:{item_id}:{quantity}"),
    )?;
    let transfer = runtime_shell
        .shell
        .deposit_bag_item_to_pc(&item_id, stack_index, quantity)?;
    runtime_shell.last_audio_events.push(format!(
        "pc item deposit item={} quantity={} bag_after={} pc_after={} checksum={:?}",
        transfer.item_id,
        transfer.quantity,
        transfer.bag_quantity_after,
        transfer.pc_quantity_after,
        transfer.state_checksum
    ));
    trim_event_log(&mut runtime_shell.last_audio_events);
    set_shell_action_status(
        runtime_shell,
        format!(
            "DEPOSITED {} x{} PC={}",
            transfer.item_id, transfer.quantity, transfer.pc_quantity_after
        ),
    );
    // PlayerDepositItemMenu returns to DepositSellPack after each transfer,
    // including when removing the last stack leaves only CANCEL.
    runtime_shell.field_pack_action_cursor = None;
    runtime_shell.field_pack_target_mode = None;
    runtime_shell.pc_item_cursor = None;
    runtime_shell.player_pc_action_cursor = None;
    move_visible_active_field_pack_cursor(runtime_shell, 0)?;
    runtime_shell.pc_notice = Some(format!("Deposited {quantity}\n{item_name}(S)."));
    Ok(())
}

fn withdraw_visible_pc_item_to_bag(
    runtime_shell: &mut BevyRuntimeShell,
    stack_index: usize,
    quantity: u16,
) -> Result<()> {
    ensure_no_visible_special_boundary(runtime_shell)?;
    let item_id = selected_pc_item_id(runtime_shell)?;
    let item_name = item_display_name(&runtime_shell.shell.snapshot()?, &item_id);
    record_visible_runtime_action(
        runtime_shell,
        format!("pc:withdraw_item:{item_id}:{quantity}"),
    )?;
    let transfer = runtime_shell
        .shell
        .withdraw_pc_item_to_bag(&item_id, stack_index, quantity)?;
    runtime_shell.last_audio_events.push(format!(
        "pc item withdraw item={} quantity={} bag_after={} pc_after={} checksum={:?}",
        transfer.item_id,
        transfer.quantity,
        transfer.bag_quantity_after,
        transfer.pc_quantity_after,
        transfer.state_checksum
    ));
    trim_event_log(&mut runtime_shell.last_audio_events);
    set_shell_action_status(
        runtime_shell,
        format!(
            "WITHDREW {} x{} BAG={}",
            transfer.item_id, transfer.quantity, transfer.bag_quantity_after
        ),
    );
    runtime_shell.bag_cursor = None;
    runtime_shell.key_item_cursor = None;
    runtime_shell.ball_cursor = None;
    runtime_shell.tmhm_cursor = None;
    runtime_shell.custom_item_cursor = None;
    runtime_shell.field_pack_pocket = None;
    runtime_shell.field_pack_action_cursor = None;
    runtime_shell.field_pack_target_mode = None;
    restore_visible_pc_item_list_position(runtime_shell)?;
    runtime_shell.pc_notice = Some(format!("Withdrew {quantity}\n{item_name}(S)."));
    Ok(())
}

fn begin_visible_pc_item_quantity(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let action = runtime_shell
        .pc_item_action
        .context("Player PC item quantity requires an active action")?;
    let snapshot = runtime_shell.shell.snapshot()?;
    let item_id = if action == VisiblePlayerPcAction::DepositItem {
        selected_field_pack_item_id(runtime_shell)?
    } else {
        selected_pc_item_id(runtime_shell)?
    };
    let item = snapshot
        .items
        .iter()
        .find(|item| item.item_id == item_id)
        .with_context(|| format!("selected PC item {item_id} is missing from the catalog"))?;
    let has_quantity = !item
        .property
        .split('|')
        .any(|flag| flag.trim() == "CANT_TOSS");
    if action == VisiblePlayerPcAction::TossItem {
        if !has_quantity {
            runtime_shell.pc_notice = Some("That's too important to toss out!".to_string());
            mark_runtime_snapshot_dirty(runtime_shell);
            return Ok(());
        }
    }
    let (stack_index, available) = if action == VisiblePlayerPcAction::DepositItem {
        let pocket = active_visible_field_pack_pocket(runtime_shell);
        let cursor = match &pocket {
            FieldPackPocket::Items => &runtime_shell.bag_cursor,
            FieldPackPocket::Balls => &runtime_shell.ball_cursor,
            FieldPackPocket::KeyItems => &runtime_shell.key_item_cursor,
            FieldPackPocket::TmHm => &runtime_shell.tmhm_cursor,
            FieldPackPocket::Custom(id) => {
                anyhow::bail!("source PC deposit has no custom pocket {id}")
            }
        };
        let stack_index = cursor
            .as_ref()
            .context("Player PC deposit has no selected pocket cursor")?
            .option_index;
        let quantity = match pocket {
            FieldPackPocket::Items => snapshot
                .bag
                .items
                .get(stack_index)
                .filter(|item| item.item_id == item_id)
                .map(|item| item.quantity),
            FieldPackPocket::Balls => snapshot
                .bag
                .balls
                .get(stack_index)
                .filter(|item| item.item_id == item_id)
                .map(|item| item.quantity),
            FieldPackPocket::KeyItems => snapshot
                .bag
                .key_items
                .get(stack_index)
                .filter(|item| item.item_id == item_id)
                .map(|item| item.quantity),
            FieldPackPocket::TmHm => snapshot
                .bag
                .tm_hm
                .get(stack_index)
                .filter(|item| item.item_id == item_id)
                .map(|item| item.quantity),
            FieldPackPocket::Custom(_) => unreachable!(),
        };
        (stack_index, quantity)
    } else {
        let stack_index = runtime_shell
            .pc_item_cursor
            .as_ref()
            .context("Player PC item action has no PC-item cursor")?
            .option_index;
        let quantity = snapshot
            .bag
            .pc_items
            .get(stack_index)
            .filter(|item| item.item_id == item_id)
            .map(|item| item.quantity);
        (stack_index, quantity)
    };
    let available = available
        .filter(|quantity| *quantity > 0)
        .with_context(|| format!("Player PC item {item_id} has no selectable quantity"))?;
    let maximum = if has_quantity { available } else { 1 };
    runtime_shell.pc_item_quantity = Some(VisiblePcItemQuantity {
        action,
        item_id: item_id.clone(),
        stack_index,
        quantity: 1,
        maximum,
    });
    // _CheckTossableItem bypasses SelectQuantityToToss for protected items
    // on deposit and withdrawal. It does not prohibit storing them.
    if !has_quantity {
        return commit_visible_pc_item_quantity(runtime_shell);
    }
    runtime_shell.pc_notice = Some(match action {
        VisiblePlayerPcAction::WithdrawItem => "How many do you\nwant to withdraw?".to_string(),
        VisiblePlayerPcAction::DepositItem => "How many do you\nwant to deposit?".to_string(),
        VisiblePlayerPcAction::TossItem => format!(
            "Toss out how many\n{}(S)?",
            item_display_name(&snapshot, &item_id)
        ),
        _ => unreachable!("quantity action was validated before selecting an item"),
    });
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(())
}

fn visible_pc_item_quantity_input_ready(runtime_shell: &BevyRuntimeShell) -> bool {
    // MenuTextbox/PrintText returns before SelectQuantityToToss polls input.
    runtime_shell.pc_notice.as_deref().is_some_and(|question| {
        visible_field_text_reveal_is_complete_for_text(runtime_shell, question)
    })
}

fn adjust_visible_pc_item_quantity(runtime_shell: &mut BevyRuntimeShell, delta: i16) -> Result<()> {
    let pending = runtime_shell
        .pc_item_quantity
        .as_mut()
        .context("no Player PC item quantity is active")?;
    // BuySellToss_InterpretJoypad changes only the number window. The
    // question has already printed and must not restart its text reveal.
    pending.quantity = match delta {
        -1 if pending.quantity == 1 => pending.maximum,
        1 if pending.quantity == pending.maximum => 1,
        _ => (i32::from(pending.quantity) + i32::from(delta)).clamp(1, i32::from(pending.maximum))
            as u16,
    };
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(())
}

fn commit_visible_pc_item_quantity(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let pending = runtime_shell
        .pc_item_quantity
        .take()
        .context("no Player PC item quantity is active")?;
    runtime_shell.pc_notice = None;
    let snapshot = runtime_shell.shell.snapshot()?;
    let destination_capacity = match pending.action {
        VisiblePlayerPcAction::DepositItem => visible_item_pocket_free_capacity(
            &snapshot.bag.pc_items,
            &pending.item_id,
            crate::core::models::PC_ITEM_CAPACITY,
        ),
        VisiblePlayerPcAction::WithdrawItem => {
            let item = snapshot
                .items
                .iter()
                .find(|item| item.item_id == pending.item_id)
                .context("PC withdrawal item has no pocket definition")?;
            match item.pocket.as_str() {
                "ITEM" => visible_item_pocket_free_capacity(
                    &snapshot.bag.items,
                    &pending.item_id,
                    crate::core::models::ITEM_POCKET_CAPACITY,
                ),
                "BALL" => visible_item_pocket_free_capacity(
                    &snapshot.bag.balls,
                    &pending.item_id,
                    crate::core::models::BALL_POCKET_CAPACITY,
                ),
                // ReceiveKeyItem appends one slot, even for a duplicate key.
                "KEY_ITEM" => u16::from(
                    snapshot.bag.key_items.len() < crate::core::models::KEY_ITEM_POCKET_CAPACITY,
                ),
                "TM_HM" => crate::core::models::MAX_ITEM_STACK.saturating_sub(
                    snapshot
                        .bag
                        .tm_hm
                        .iter()
                        .find(|entry| entry.item_id == pending.item_id)
                        .map_or(0, |entry| entry.quantity),
                ),
                pocket => anyhow::bail!("source PC withdrawal has no pocket {pocket}"),
            }
        }
        _ => pending.quantity,
    };
    if pending.quantity > destination_capacity {
        runtime_shell.pc_notice = Some(if pending.action == VisiblePlayerPcAction::DepositItem {
            "The PC is full.".to_string()
        } else {
            "The PACK is full.".to_string()
        });
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(());
    }
    match pending.action {
        VisiblePlayerPcAction::DepositItem => deposit_visible_selected_pack_item_to_pc(
            runtime_shell,
            pending.stack_index,
            pending.quantity,
        ),
        VisiblePlayerPcAction::WithdrawItem => {
            withdraw_visible_pc_item_to_bag(runtime_shell, pending.stack_index, pending.quantity)
        }
        VisiblePlayerPcAction::TossItem => {
            runtime_shell.pc_confirmation = Some(VisiblePcConfirmation::TossItem {
                item_id: pending.item_id.clone(),
                stack_index: pending.stack_index,
                quantity: pending.quantity,
            });
            runtime_shell.yes_no_cursor = Some(MenuCursor {
                surface_id: "pc:confirmation".to_string(),
                option_index: 0,
            });
            runtime_shell.pc_notice = Some(format!(
                "Throw away {} x{}?",
                item_display_name(&snapshot, &pending.item_id),
                pending.quantity
            ));
            Ok(())
        }
        _ => anyhow::bail!("Player PC quantity is invalid for {pending:?}"),
    }
}

fn visible_item_pocket_free_capacity(
    items: &[RuntimeBagItemSnapshot],
    item_id: &str,
    slot_capacity: usize,
) -> u16 {
    let matching_space = items
        .iter()
        .filter(|item| item.item_id == item_id)
        .map(|item| u32::from(crate::core::models::MAX_ITEM_STACK - item.quantity))
        .sum::<u32>();
    let empty_slots = slot_capacity.saturating_sub(items.len());
    let empty_space = u32::try_from(empty_slots)
        .unwrap_or(u32::MAX)
        .saturating_mul(u32::from(crate::core::models::MAX_ITEM_STACK));
    matching_space
        .saturating_add(empty_space)
        .min(u32::from(u16::MAX)) as u16
}

fn request_visible_pc_pokemon_release(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    ensure_no_visible_special_boundary(runtime_shell)?;
    let snapshot = runtime_shell.shell.snapshot()?;
    let location = selected_visible_pc_pokemon_location(runtime_shell, &snapshot)?;
    let pokemon = visible_pc_pokemon_at(&snapshot, location)?;
    if let VisiblePcPokemonLocation::Party(index) = location {
        let refusal = if snapshot.party.slots.len() <= 1 {
            Some("It's your last <PK><MN>!")
        } else if !snapshot.party.slots.iter().any(|slot| slot.index != index && slot.pokemon.hp > 0) {
            Some("No more usable <PK><MN>!")
        } else if pokemon.item.as_deref().is_some_and(crate::core::models::item::is_mail_item_id) {
            Some("Remove MAIL.")
        } else if pokemon.is_egg {
            Some("No releasing EGGS!")
        } else { None };
        if let Some(text) = refusal {
            return begin_visible_pc_transfer_refusal(runtime_shell, VisiblePcTransferKind::Deposit,
                snapshot.storage.current_pc_box, text, true);
        }
    } else if pokemon.is_egg {
        return begin_visible_pc_transfer_refusal(runtime_shell, VisiblePcTransferKind::Withdraw,
            snapshot.storage.current_pc_box, "No releasing EGGS!", false);
    }
    runtime_shell.pending_pc_release = Some(VisiblePcReleasePrompt { location });
    runtime_shell.yes_no_cursor = Some(MenuCursor {
        surface_id: "pc:release-confirm".to_string(), option_index: 0,
    });
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(())
}

fn open_visible_bill_pc_pokemon_actions(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let snapshot = runtime_shell.shell.snapshot()?;
    let current_box = visible_storage_box(&snapshot, runtime_shell)?;
    let (surface, count) = if runtime_shell.bill_pc_deposit_open || (runtime_shell.bill_pc_move_open && runtime_shell.bill_pc_move_party_open) {
        (pc_party_surface_id().to_string(), snapshot.party.slots.len())
    } else {
        (storage_cursor_surface_id(current_box.index), current_box.slots.len())
    };
    let selected = strict_readonly_cursor_index(&runtime_shell.storage_cursor, &surface, count + 1)
        .context("PC list requires a valid Pokémon or CANCEL cursor")?;
    if selected == count {
        return if runtime_shell.bill_pc_move_open { close_visible_bill_pc_move_list(runtime_shell) }
            else { close_visible_pc_surface(runtime_shell) };
    }
    selected_visible_pc_pokemon_location(runtime_shell, &snapshot)?;
    runtime_shell.bill_pc_pokemon_action_cursor = Some(MenuCursor {
        surface_id: "pc:pokemon-actions".to_string(),
        option_index: 0,
    });
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(())
}

fn confirm_visible_bill_pc_pokemon_action(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let selected = strict_readonly_cursor_index(
        &runtime_shell.bill_pc_pokemon_action_cursor,
        "pc:pokemon-actions",
        visible_pc_pokemon_action_labels(runtime_shell).len(),
    )
    .context("boxed Pokemon action menu requires a valid cursor")?;
    // Transfer refusals may return to this submenu. Successful operations
    // and source cancellation paths clear it when their outcome is known.
    if selected + 1 == visible_pc_pokemon_action_labels(runtime_shell).len() {
        runtime_shell.bill_pc_pokemon_action_cursor = None;
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(());
    }
    match selected {
        0 if runtime_shell.bill_pc_move_open => begin_visible_bill_pc_move_source(runtime_shell),
        0 if runtime_shell.bill_pc_deposit_open => deposit_visible_party_pokemon(runtime_shell),
        0 => withdraw_visible_pc_pokemon(runtime_shell),
        1 => {
            let snapshot = runtime_shell.shell.snapshot()?;
            let location = selected_visible_pc_pokemon_location(runtime_shell, &snapshot)?;
            runtime_shell.bill_pc_pokemon_summary = Some(VisiblePcPokemonSummary {
                location,
                page: 1,
            });
            queue_visible_stats_egg_sound(runtime_shell, visible_pc_pokemon_at(&snapshot, location)?)?;
            mark_runtime_snapshot_dirty(runtime_shell);
            Ok(())
        }
        2 => request_visible_pc_pokemon_release(runtime_shell),
        _ => {
            mark_runtime_snapshot_dirty(runtime_shell);
            Ok(())
        }
    }
}

fn confirm_visible_pc_release_prompt(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let prompt = runtime_shell
        .pending_pc_release
        .clone()
        .context("PC release confirmation is not active")?;
    let selected =
        strict_readonly_cursor_index(&runtime_shell.yes_no_cursor, "pc:release-confirm", 2)
            .context("PC release confirmation requires a valid cursor")?;
    runtime_shell.pending_pc_release = None;
    runtime_shell.yes_no_cursor = None;
    if selected != 0 {
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(());
    }
    let snapshot = runtime_shell.shell.snapshot()?;
    anyhow::ensure!(
        selected_visible_pc_pokemon_location(runtime_shell, &snapshot)? == prompt.location,
        "PC release selection changed while confirmation was open"
    );
    release_visible_pc_pokemon(runtime_shell)
}

fn release_visible_pc_pokemon(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    ensure_no_visible_special_boundary(runtime_shell)?;
    let snapshot = runtime_shell.shell.snapshot()?;
    let location = selected_visible_pc_pokemon_location(runtime_shell, &snapshot)?;
    let retained_names = match location {
        VisiblePcPokemonLocation::Party(_) => snapshot.party.slots.iter().map(|slot| slot.pokemon.nickname.clone()).collect(),
        VisiblePcPokemonLocation::Box { .. } => visible_storage_box(&snapshot, runtime_shell)?.slots.iter().map(|slot| slot.pokemon.nickname.clone()).collect(),
    };
    record_visible_runtime_action(runtime_shell, format!("pc:release_pokemon:{location:?}"))?;
    let pokemon = match location {
        VisiblePcPokemonLocation::Party(party_index) => {
            let mutation = runtime_shell.shell.apply_runtime_mutation_command(
                crate::RuntimeMutationCommand::ReleasePartyPokemon(crate::assets::RuntimePartySlotCommand { party_index }),
            )?;
            let crate::RuntimeMutationResult::PartyPokemonReleased(pokemon) = mutation.result else {
                anyhow::bail!("party release returned a different runtime result");
            };
            pokemon
        }
        VisiblePcPokemonLocation::Box { box_slot, .. } => runtime_shell.shell.release_current_box_pokemon(box_slot)?.pokemon,
    };
    queue_visible_pokemon_cry(runtime_shell, &pokemon.species.id, "bill_pc_release")?;
    set_shell_action_status(runtime_shell, format!("RELEASED {}", pokemon.species.id));
    runtime_shell.pc_notice = Some("Released <PK><MN>.".to_string());
    runtime_shell.field_text_reveal = None;
    runtime_shell.pc_release_sequence = Some(VisiblePcReleaseSequence {
        box_index: snapshot.storage.current_pc_box,
        species_name: crate::core::models::pokemon_species_display_name(&pokemon.species.id),
        retained_names,
        phase: VisiblePcReleasePhase::Released,
        frames_remaining: 80,
    });
    close_visible_party_detail_state(runtime_shell);
    mark_runtime_presentation_dirty(runtime_shell);
    Ok(())
}

fn advance_visible_pc_release_sequence(
    runtime_shell: &mut BevyRuntimeShell,
    elapsed_frames: u32,
) -> Result<()> {
    let mut elapsed_frames = elapsed_frames;
    while elapsed_frames > 0 {
        let Some(active) = runtime_shell.pc_release_sequence.as_mut() else {
            break;
        };
        let elapsed = elapsed_frames.min(u32::from(active.frames_remaining));
        active.frames_remaining = active
            .frames_remaining
            .saturating_sub(u8::try_from(elapsed).unwrap_or(u8::MAX));
        elapsed_frames -= elapsed;
        if active.frames_remaining > 0 {
            break;
        }
        match active.phase {
            VisiblePcReleasePhase::Released => {
                active.phase = VisiblePcReleasePhase::Bye;
                active.frames_remaining = 50;
                runtime_shell.pc_notice = Some(format!("Bye, {}!", active.species_name));
                runtime_shell.field_text_reveal = None;
                set_shell_action_status(runtime_shell, "BYE, POKEMON!");
                mark_runtime_presentation_dirty(runtime_shell);
            }
            VisiblePcReleasePhase::Bye => {
                let finished = runtime_shell
                    .pc_release_sequence
                    .take()
                    .context("PC release sequence disappeared at its final frame")?;
                let snapshot = runtime_shell.shell.snapshot()?;
                anyhow::ensure!(
                    snapshot.storage.current_pc_box == finished.box_index,
                    "PC box changed during the locked release sequence"
                );
                runtime_shell.pc_list_scroll = 0;
                runtime_shell.storage_cursor = Some(MenuCursor {
                    surface_id: if runtime_shell.bill_pc_deposit_open { pc_party_surface_id().to_string() }
                        else { storage_cursor_surface_id(finished.box_index) },
                    option_index: 0,
                });
                runtime_shell.bill_pc_pokemon_action_cursor = None;
                runtime_shell.pc_notice = None;
                runtime_shell.field_text_reveal = None;
                set_shell_action_status(runtime_shell, "BILL'S PC");
                mark_runtime_presentation_dirty(runtime_shell);
            }
        }
    }
    Ok(())
}

fn close_visible_party_detail_state(runtime_shell: &mut BevyRuntimeShell) {
    runtime_shell.party_menu_open = false;
    runtime_shell.party_return_start_menu_cursor = None;
    runtime_shell.party_summary_open = false;
    runtime_shell.party_action_cursor = None;
    runtime_shell.party_give_take_cursor = None;
    runtime_shell.party_mail_take_stage = None;
    runtime_shell.party_move_reorder_open = false;
    runtime_shell.party_move_reorder_origin = None;
    runtime_shell.party_switch_cursor = None;
    runtime_shell.party_hp_transfer_source = None;
    runtime_shell.party_hp_transfer_move = None;
    runtime_shell.party_move_cursor = None;
}

fn apply_visible_heal_party(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    record_visible_runtime_action(runtime_shell, "special:heal_party")?;
    let special = runtime_shell.shell.heal_party_special()?;
    runtime_shell.last_audio_events.push(format!(
        "special heal outcome={:?} checksum={:?}",
        special.outcome.effect, special.state_checksum
    ));
    Ok(())
}

fn fade_visible_music_special(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    record_visible_runtime_action(runtime_shell, "special:fade_music")?;
    let special = runtime_shell.shell.fade_out_music_special()?;
    runtime_shell.last_audio_events.push(format!(
        "special music fade outcome={:?} checksum={:?}",
        special.outcome.effect, special.state_checksum
    ));
    Ok(())
}

fn wait_visible_sfx_special(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    record_visible_runtime_action(runtime_shell, "special:wait_sfx")?;
    let special = runtime_shell.shell.wait_sfx_special()?;
    runtime_shell.last_audio_events.push(format!(
        "special wait sfx outcome={:?} checksum={:?}",
        special.outcome.effect, special.state_checksum
    ));
    Ok(())
}

fn play_visible_map_music_special(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    record_visible_runtime_action(runtime_shell, "special:play_map_music")?;
    let special = runtime_shell.shell.play_map_music_special()?;
    runtime_shell.last_audio_events.push(format!(
        "special play map music outcome={:?} checksum={:?}",
        special.outcome.effect, special.state_checksum
    ));
    Ok(())
}

fn restart_visible_map_music_special(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    record_visible_runtime_action(runtime_shell, "special:restart_map_music")?;
    let special = runtime_shell.shell.restart_map_music_special()?;
    runtime_shell.last_audio_events.push(format!(
        "special restart map music outcome={:?} checksum={:?}",
        special.outcome.effect, special.state_checksum
    ));
    Ok(())
}

fn full_heal_visible_party_lead(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let party_index = selected_party_index(runtime_shell)?;
    record_visible_runtime_action(runtime_shell, format!("party:full_heal:{party_index}"))?;
    let recovered = runtime_shell.shell.full_heal_party_pokemon(party_index)?;
    runtime_shell.last_audio_events.push(format!(
        "party recovery slot={} species={} hp {}->{} status {:?}->{:?} pp_moves={} checksum={:?}",
        recovered.party_index,
        recovered.species_id,
        recovered.hp_before,
        recovered.hp_after,
        recovered.status_before,
        recovered.status_after,
        recovered.pp_restored.len(),
        recovered.state_checksum
    ));
    Ok(())
}

fn full_heal_visible_whole_party(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    record_visible_runtime_action(runtime_shell, "party:full_heal:all")?;
    let recovered = runtime_shell.shell.full_heal_whole_party()?;
    let checksum = recovered.last().map(|entry| &entry.state_checksum);
    runtime_shell.last_audio_events.push(format!(
        "whole party recovery slots={} checksum={:?}",
        recovered.len(),
        checksum
    ));
    Ok(())
}

fn resolve_visible_blackout(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    if runtime_shell.visible_blackout_phase.is_some() {
        return Ok(());
    }
    record_visible_runtime_action(runtime_shell, "overworld:blackout:resolve")?;
    let blackout_scene = runtime_shell.shell.snapshot()?;
    let player_name = if blackout_scene.trainer.player_name.is_empty() {
        "PLAYER"
    } else {
        blackout_scene.trainer.player_name.as_str()
    };
    runtime_shell
        .battle_messages
        .push_back(format!("{player_name} is out of\nuseable POKéMON!"));
    runtime_shell
        .battle_messages
        .push_back(format!("{player_name} whited\nout!"));
    // Loss cleanup may already have removed the authoritative battle while
    // its final hit and faint animation are still queued. Keep their retained
    // scene until the whiteout fade has finished.
    if runtime_shell.battle_message_scene.is_none() {
        runtime_shell.battle_message_scene = Some(Arc::new(blackout_scene));
    }
    runtime_shell.visible_blackout_phase = Some(VisibleBlackoutPhase::AwaitText);
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(())
}

fn begin_visible_poison_blackout_after_faint_text(
    runtime_shell: &mut BevyRuntimeShell,
) -> Result<bool> {
    if !runtime_shell.pending_poison_blackout {
        return Ok(false);
    }
    runtime_shell.pending_poison_blackout = false;
    runtime_shell.field_notice_scene = None;
    resolve_visible_blackout(runtime_shell)?;
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(true)
}

fn commit_visible_blackout_recovery(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let recovered = runtime_shell.shell.resolve_blackout_to_last_spawn()?;
    runtime_shell.last_audio_events.push(format!(
        "blackout recovery spawn={:?} map={} tile=({}, {}) healed={} checksum={:?}",
        recovered.spawn_identifier,
        recovered.map_name,
        recovered.tile.x,
        recovered.tile.y,
        recovered.healed.len(),
        recovered.state_checksum
    ));
    // Script_Whiteout commits healing, money loss, and the spawn warp after
    // the 40-frame white hold. MAPSETUP_WARP then reveals the destination;
    // ordinary scene scripts must not run under the still-white palette.
    reset_visible_navigation_state(runtime_shell);
    reset_visible_battle_presentation(runtime_shell);
    queue_visible_current_music(runtime_shell)?;
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(())
}

fn present_visible_step_event(
    runtime_shell: &mut BevyRuntimeShell,
    event: &crate::core::systems::step_events::StepEventResult,
) -> Result<()> {
    if let Some(item_id) = event.repel_expired.as_deref() {
        record_visible_runtime_action(runtime_shell, format!("overworld:repel_expired:{item_id}"))?;
        runtime_shell
            .last_audio_events
            .push(format!("{item_id} wore off"));
        runtime_shell.field_notice = Some("REPEL's effect wore off.".to_string());
        mark_runtime_snapshot_dirty(runtime_shell);
        set_shell_action_status(runtime_shell, "REPEL WORE OFF");
    } else if event.egg_hatched {
        let species = event
            .hatched_species
            .as_deref()
            .context("hatched step event is missing its species")?;
        let party_index = event
            .hatched_party_index
            .context("hatched step event is missing its party index")?;
        record_visible_runtime_action(runtime_shell, format!("overworld:egg_hatched:{species}"))?;
        runtime_shell
            .last_audio_events
            .push(format!("egg hatched into {species}"));
        runtime_shell.visible_egg_hatch = Some(VisibleEggHatch {
            party_index,
            species_id: species.to_string(),
            phase: VisibleEggHatchPhase::HuhText,
            frame: 0,
        });
        runtime_shell.field_notice = Some("Huh?".to_string());
        mark_runtime_snapshot_dirty(runtime_shell);
        set_shell_action_status(
            runtime_shell,
            format!(
                "EGG HATCHED: {}",
                crate::core::models::pokemon_species_display_name(species)
            ),
        );
    } else if let Some(poison) = event.poison_result.as_ref() {
        record_visible_runtime_action(runtime_shell, "overworld:poison_step")?;
        // LoadPoisonBGPals owns four purple BG-palette VBlanks, restores the
        // map palettes, then PlayPoisonSFX owns one trailing normal DelayFrame
        // before DoPoisonStep or its faint script can continue.
        runtime_shell.poison_flash_frames_remaining = 5;
        queue_visible_shell_sound_effect(runtime_shell, "SFX_POISON")?;
        runtime_shell.last_audio_events.push(format!(
            "poison step damaged={:?} fainted={:?}",
            poison.damaged_names, poison.fainted_names
        ));
        if poison.fainted_names.is_empty() {
            set_shell_action_status(runtime_shell, "POISON HURT THE PARTY");
        } else {
            let notices = poison
                .fainted_names
                .iter()
                .map(|name| format!("{name} fainted!"));
            runtime_shell.field_notice_queue.extend(notices);
            mark_runtime_snapshot_dirty(runtime_shell);
            set_shell_action_status(
                runtime_shell,
                format!("FAINTED: {}", poison.fainted_names.join(", ")),
            );
        }
    }
    trim_event_log(&mut runtime_shell.last_audio_events);
    Ok(())
}

fn apply_visible_pokemon_center_pc(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    record_visible_runtime_action(runtime_shell, "special:pc:pokemon_center")?;
    let special = runtime_shell.shell.open_pokemon_center_pc_special()?;
    runtime_shell.last_audio_events.push(format!(
        "pokemon center pc outcome={:?} checksum={:?}",
        special.outcome.effect, special.state_checksum
    ));
    activate_visible_special_boundary_if_needed(runtime_shell, &special.outcome.effect)?;
    Ok(())
}

fn apply_visible_players_house_pc(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    record_visible_runtime_action(runtime_shell, "special:pc:players_house")?;
    let special = runtime_shell.shell.open_players_house_pc_special()?;
    runtime_shell.last_audio_events.push(format!(
        "players house pc outcome={:?} checksum={:?}",
        special.outcome.effect, special.state_checksum
    ));
    activate_visible_special_boundary_if_needed(runtime_shell, &special.outcome.effect)?;
    Ok(())
}

fn apply_visible_overworld_town_map(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    record_visible_runtime_action(runtime_shell, "special:town_map:overworld")?;
    let special = runtime_shell.shell.open_overworld_town_map_special()?;
    runtime_shell.last_audio_events.push(format!(
        "overworld town map outcome={:?} checksum={:?}",
        special.outcome.effect, special.state_checksum
    ));
    activate_visible_special_boundary_if_needed(runtime_shell, &special.outcome.effect)?;
    Ok(())
}

fn apply_visible_move_deletion(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let snapshot = runtime_shell.shell.snapshot()?;
    let party_index = selected_party_index(runtime_shell)?;
    let move_count = snapshot
        .party
        .slots
        .iter()
        .find(|slot| slot.index == party_index)
        .with_context(|| format!("selected party index {party_index} is not in the party"))?
        .pokemon
        .moves
        .len();
    if move_count <= 1 {
        record_visible_runtime_action(
            runtime_shell,
            format!("special:move_deletion:{party_index}:no_deletable_move"),
        )?;
        runtime_shell.last_audio_events.push(format!(
            "selected party index {party_index} has no deletable move"
        ));
        set_shell_action_status(runtime_shell, "NO MOVE TO DELETE");
        trim_event_log(&mut runtime_shell.last_audio_events);
        return Ok(());
    }
    let move_slot = selected_party_move_slot(runtime_shell, party_index)?;
    if move_slot >= move_count {
        record_visible_runtime_action(
            runtime_shell,
            format!("special:move_deletion:{party_index}:{move_slot}:unavailable"),
        )?;
        runtime_shell
            .last_audio_events
            .push(format!("selected move slot {move_slot} is not deletable"));
        set_shell_action_status(runtime_shell, "MOVE UNAVAILABLE");
        trim_event_log(&mut runtime_shell.last_audio_events);
        return Ok(());
    }
    record_visible_runtime_action(
        runtime_shell,
        format!("special:move_deletion:{party_index}:{move_slot}"),
    )?;
    let special = runtime_shell
        .shell
        .delete_party_move_special(party_index, move_slot)?;
    runtime_shell.last_audio_events.push(format!(
        "move deletion party_index={} move_slot={} outcome={:?} checksum={:?}",
        party_index, move_slot, special.outcome.effect, special.state_checksum
    ));
    activate_visible_special_boundary_if_needed(runtime_shell, &special.outcome.effect)?;
    Ok(())
}

fn apply_visible_name_rater(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let snapshot = runtime_shell.shell.snapshot()?;
    let party_index = selected_party_index(runtime_shell)?;
    let slot = snapshot
        .party
        .slots
        .iter()
        .find(|slot| slot.index == party_index)
        .with_context(|| format!("selected party index {party_index} is not in the party"))?;
    let slot_index = slot.index;
    let nickname = slot.pokemon.nickname.clone();
    record_visible_runtime_action(runtime_shell, format!("special:name_rater:{party_index}"))?;
    let special = runtime_shell
        .shell
        .rate_party_nickname_special(slot_index, nickname)?;
    runtime_shell.last_audio_events.push(format!(
        "name rater outcome={:?} checksum={:?}",
        special.outcome.effect, special.state_checksum
    ));
    activate_visible_special_boundary_if_needed(runtime_shell, &special.outcome.effect)?;
    Ok(())
}

fn apply_visible_name_rival_for_script_command(
    runtime_shell: &mut BevyRuntimeShell,
    source_script: &str,
    command_index: usize,
) -> Result<bool> {
    if compiled_special_routine_at(runtime_shell, source_script, command_index)?.as_deref()
        != Some("NameRival")
    {
        return Ok(false);
    }
    apply_visible_name_rival(runtime_shell, source_script, command_index)?;
    Ok(true)
}

fn open_visible_day_care_for_script_command(
    runtime_shell: &mut BevyRuntimeShell,
    source_script: &str,
    command_index: usize,
) -> Result<bool> {
    let Some(routine) = compiled_special_routine_at(runtime_shell, source_script, command_index)?
    else {
        return Ok(false);
    };
    if routine == "DayCareManOutside" {
        let snapshot = runtime_shell.shell.snapshot()?;
        if snapshot.day_care.egg_present {
            let mut boundaries = visible_exported_special_text_boundaries(
                runtime_shell,
                "DayCareFoundEggText",
                "_FoundAnEggText",
            )?;
            let prompt = boundaries
                .pop_back()
                .context("Day-Care Egg offer has no final yes/no page")?;
            runtime_shell.pc_notice = Some(
                prompt
                    .details
                    .into_iter()
                    .next()
                    .context("Day-Care Egg offer final page is empty")?,
            );
            runtime_shell.special_boundary = boundaries.pop_front();
            runtime_shell.special_boundary_queue = boundaries;
            runtime_shell.pc_confirmation = Some(VisiblePcConfirmation::DayCareEggPickup);
            runtime_shell.yes_no_cursor = Some(MenuCursor {
                surface_id: "pc:confirmation".to_string(),
                option_index: 0,
            });
            set_shell_action_status(runtime_shell, "DAY-CARE EGG");
        } else {
            let mut boundaries = visible_exported_special_text_boundaries(
                runtime_shell,
                "DayCareNotYetText",
                "_NotYetText",
            )?;
            runtime_shell.special_boundary = boundaries.pop_front();
            runtime_shell.special_boundary_queue = boundaries;
        }
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(true);
    }
    let caretaker = match routine.as_str() {
        "DayCareMan" => "man",
        "DayCareLady" => "lady",
        _ => return Ok(false),
    };
    let snapshot = runtime_shell.shell.snapshot()?;
    let resident = if caretaker == "man" {
        &snapshot.day_care.man
    } else {
        &snapshot.day_care.lady
    };
    let owner = if caretaker == "man" { "MAN" } else { "LADY" };
    if let Some(pokemon) = resident.pokemon.as_ref() {
        let nickname = if pokemon.nickname.trim().is_empty() {
            canonical_species_display_name(&pokemon.species.id)
        } else {
            pokemon.nickname.clone()
        };
        let current_level = crate::core::systems::special_routines::day_care_level_from_experience(
            pokemon,
            runtime_shell.shell.runtime().growth_rates(),
        )?;
        let gained = current_level.saturating_sub(pokemon.level);
        let (text_target, confirm_withdrawal) = if gained == 0 {
            ("_BackAlreadyText", true)
        } else {
            ("_AreWeGeniusesText", false)
        };
        let mut boundaries = visible_exported_special_text_boundaries_with_buffer(
            runtime_shell,
            "DayCareWithdrawText",
            text_target,
            Some(&nickname),
        )?;
        let prompt = boundaries
            .pop_back()
            .context("Day-Care withdrawal offer has no final yes/no page")?;
        runtime_shell.pc_notice = Some(
            prompt
                .details
                .into_iter()
                .next()
                .context("Day-Care withdrawal final yes/no page is empty")?,
        );
        runtime_shell.special_boundary = boundaries.pop_front();
        runtime_shell.special_boundary_queue = boundaries;
        runtime_shell.pc_confirmation = Some(VisiblePcConfirmation::DayCareWithdraw {
            caretaker: caretaker.to_string(),
            confirm_withdrawal,
        });
        runtime_shell.yes_no_cursor = Some(MenuCursor {
            surface_id: "pc:confirmation".to_string(),
            option_index: 0,
        });
        set_shell_action_status(runtime_shell, format!("DAY-CARE {owner}"));
    } else {
        let pending = PendingScriptPartySelection::DayCareDeposit {
            caretaker: caretaker.to_string(),
        };
        let intro_target = if caretaker == "man" {
            "_DayCareManIntroText"
        } else if snapshot.day_care.lady.active {
            "_DayCareLadyIntroEggText"
        } else {
            "_DayCareLadyIntroText"
        };
        let caretaker_kind = if caretaker == "man" {
            RuntimeDayCareCaretaker::Man
        } else {
            RuntimeDayCareCaretaker::Lady
        };
        let opened =
            runtime_shell
                .shell
                .use_day_care(caretaker_kind, RuntimeDayCareAction::Open, None)?;
        runtime_shell.last_audio_events.push(format!(
            "day-care introduction outcome={:?} checksum={:?}",
            opened.outcome.effect, opened.state_checksum
        ));
        let mut boundaries = visible_exported_special_text_boundaries(
            runtime_shell,
            "DayCareIntroText",
            intro_target,
        )?;
        let prompt = boundaries
            .pop_back()
            .context("Day-Care introduction has no final yes/no page")?;
        runtime_shell.pc_notice = Some(
            prompt
                .details
                .into_iter()
                .next()
                .context("Day-Care final yes/no page is empty")?,
        );
        runtime_shell.special_boundary = boundaries.pop_front();
        runtime_shell.special_boundary_queue = boundaries;
        runtime_shell.pc_confirmation = Some(VisiblePcConfirmation::ScriptPartyIntro(pending));
        runtime_shell.yes_no_cursor = Some(MenuCursor {
            surface_id: "pc:confirmation".to_string(),
            option_index: 0,
        });
        set_shell_action_status(runtime_shell, "DAY-CARE WHICH ONE?");
    }
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(true)
}

fn open_visible_script_party_selection_for_command(
    runtime_shell: &mut BevyRuntimeShell,
    source_script: &str,
    command_index: usize,
) -> Result<bool> {
    let origin_map_name = runtime_shell.shell.current_map_name().to_string();
    let command = runtime_shell.shell.script_runtime_command_at(
        &origin_map_name,
        source_script,
        command_index,
    );
    let pending = if command.as_ref().is_some_and(|command| {
        command.command == "trade"
            && command.args.first().is_some_and(|trade_id| {
                !runtime_shell
                    .shell
                    .session()
                    .state()
                    .script_runtime
                    .completed_trades
                    .contains(trade_id)
            })
    }) {
        let trade_id = command
            .as_ref()
            .and_then(|command| command.args.first().cloned())
            .context("trade command is missing its trade id")?;
        PendingScriptPartySelection::NpcTrade {
            origin_map_name,
            source_script: source_script.to_string(),
            command_index,
            trade_id,
        }
    } else if command
        .as_ref()
        .is_some_and(|command| command.command == "checkpokemail")
    {
        PendingScriptPartySelection::CheckPokeMail {
            origin_map_name,
            source_script: source_script.to_string(),
            command_index,
        }
    } else {
        match compiled_special_routine_at(runtime_shell, source_script, command_index)?.as_deref() {
            Some("BillsGrandfather") => PendingScriptPartySelection::BillsGrandfather,
            Some("ReturnShuckie") => PendingScriptPartySelection::ReturnShuckie,
            Some("CheckMagikarpLength") => PendingScriptPartySelection::CheckMagikarpLength,
            Some("PhotoStudio") => PendingScriptPartySelection::PhotoStudio,
            Some("PokeSeer") => PendingScriptPartySelection::PokeSeer,
            Some("NameRater") => PendingScriptPartySelection::NameRater,
            Some("OlderHaircutBrother") => PendingScriptPartySelection::OlderHaircutBrother,
            Some("YoungerHaircutBrother") => PendingScriptPartySelection::YoungerHaircutBrother,
            Some("DaisysGrooming") => PendingScriptPartySelection::DaisysGrooming,
            Some("MoveDeletion") => PendingScriptPartySelection::MoveDeletion { party_index: None },
            Some("MoveTutor") => {
                let value = runtime_shell
                    .shell
                    .snapshot()?
                    .script_events
                    .script_value
                    .context("MoveTutor has no setval move selector")?;
                let move_id = match value.as_str() {
                    "1" | "MOVETUTOR_FLAMETHROWER" => "FLAMETHROWER",
                    "2" | "MOVETUTOR_THUNDERBOLT" => "THUNDERBOLT",
                    "3" | "MOVETUTOR_ICE_BEAM" => "ICE_BEAM",
                    other => anyhow::bail!("MoveTutor has unknown move selector {other}"),
                };
                PendingScriptPartySelection::MoveTutor {
                    move_id: move_id.to_string(),
                    party_index: None,
                }
            }
            _ => return Ok(false),
        }
    };
    record_visible_runtime_action(
        runtime_shell,
        format!("script:special:party_selection:{pending:?}:open"),
    )?;
    if let PendingScriptPartySelection::NpcTrade { trade_id, .. } = &pending {
        let snapshot = runtime_shell.shell.snapshot()?;
        let rule = snapshot.special.npc_trades.get(trade_id).with_context(|| {
            format!("NPC trade {trade_id} is missing from the runtime snapshot")
        })?;
        runtime_shell.pc_notice = Some(visible_npc_trade_intro_text(rule));
        runtime_shell.pc_confirmation = Some(VisiblePcConfirmation::NpcTrade(pending));
        runtime_shell.yes_no_cursor = Some(MenuCursor {
            surface_id: "pc:confirmation".to_string(),
            option_index: 0,
        });
        set_shell_action_status(runtime_shell, "TRADE?");
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(true);
    }
    if matches!(
        &pending,
        PendingScriptPartySelection::NameRater
            | PendingScriptPartySelection::MoveDeletion { party_index: None }
    ) {
        let (label, text_target) = match &pending {
            PendingScriptPartySelection::NameRater => ("NameRaterHelloText", "_NameRaterHelloText"),
            PendingScriptPartySelection::MoveDeletion { .. } => {
                ("DeleterIntroText", "_DeleterIntroText")
            }
            _ => unreachable!("script party intro matched an unsupported routine"),
        };
        let mut boundaries =
            visible_exported_special_text_boundaries(runtime_shell, label, text_target)?;
        let prompt = boundaries
            .pop_back()
            .context("source special introduction has no final yes/no page")?;
        runtime_shell.pc_notice = Some(
            prompt
                .details
                .into_iter()
                .next()
                .context("source special final yes/no page is empty")?,
        );
        runtime_shell.special_boundary = boundaries.pop_front();
        runtime_shell.special_boundary_queue = boundaries;
        runtime_shell.pc_confirmation = Some(VisiblePcConfirmation::ScriptPartyIntro(pending));
        runtime_shell.yes_no_cursor = Some(MenuCursor {
            surface_id: "pc:confirmation".to_string(),
            option_index: 0,
        });
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(true);
    }
    if matches!(&pending, PendingScriptPartySelection::PhotoStudio) {
        runtime_shell.pending_script_party_selection = Some(pending);
        let mut boundaries = visible_exported_special_text_boundaries(
            runtime_shell,
            "WhichMonPhotoText",
            "_WhichMonPhotoText",
        )?;
        runtime_shell.special_boundary = boundaries.pop_front();
        runtime_shell.special_boundary_queue = boundaries;
        set_shell_action_status(runtime_shell, "WHICH POKEMON?");
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(true);
    }
    if matches!(&pending, PendingScriptPartySelection::PokeSeer) {
        runtime_shell.pending_script_party_selection = Some(pending);
        let mut boundaries = visible_exported_special_text_boundaries(
            runtime_shell,
            "SeerSeeAllText",
            "_SeerSeeAllText",
        )?;
        runtime_shell.special_boundary = boundaries.pop_front();
        runtime_shell.special_boundary_queue = boundaries;
        set_shell_action_status(runtime_shell, "I SEE ALL");
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(true);
    }
    runtime_shell.pending_script_party_selection = Some(pending);
    open_visible_party_menu(runtime_shell)?;
    set_shell_action_status(runtime_shell, "CHOOSE A POKEMON");
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(true)
}

fn visible_exported_special_text_boundaries(
    runtime_shell: &BevyRuntimeShell,
    label: &str,
    text_target: &str,
) -> Result<VecDeque<SpecialBoundaryDisplay>> {
    visible_exported_special_text_boundaries_with_buffer(runtime_shell, label, text_target, None)
}

fn visible_exported_special_text_boundaries_with_buffer(
    runtime_shell: &BevyRuntimeShell,
    label: &str,
    text_target: &str,
    string_buffer_1: Option<&str>,
) -> Result<VecDeque<SpecialBoundaryDisplay>> {
    let snapshot = runtime_shell.shell.presentation_snapshot()?;
    let mut named_buffers = snapshot.script_events.named_buffers.clone();
    if let Some(value) = string_buffer_1 {
        named_buffers.insert("STRING_BUFFER_1".to_string(), value.to_string());
    }
    visible_exported_special_text_boundaries_with_named_buffers(
        runtime_shell,
        label,
        text_target,
        &named_buffers,
    )
}

fn visible_exported_special_text_boundaries_with_named_buffers(
    runtime_shell: &BevyRuntimeShell,
    label: &str,
    text_target: &str,
    named_buffers: &BTreeMap<String, String>,
) -> Result<VecDeque<SpecialBoundaryDisplay>> {
    let snapshot = runtime_shell.shell.presentation_snapshot()?;
    let text = runtime_shell.shell.text_snapshot(text_target)?;
    let asm_text = text
        .asm_text
        .as_deref()
        .with_context(|| format!("special text {text_target} has no exported ASM body"))?;
    let pages = render_visible_asm_text_pages(
        asm_text,
        named_buffers,
        &snapshot.trainer.player_name,
        visible_rival_name(&snapshot),
        snapshot.progression.time.day_of_week,
    );
    anyhow::ensure!(
        !pages.is_empty(),
        "special text {text_target} rendered no source pages"
    );
    Ok(pages
        .into_iter()
        .map(|page| SpecialBoundaryDisplay {
            label: label.to_string(),
            details: vec![page],
        })
        .collect())
}

fn visible_move_tutor_text_boundaries(
    runtime_shell: &BevyRuntimeShell,
    label: &str,
    text_target: &str,
    nickname: &str,
    move_id: &str,
) -> Result<VecDeque<SpecialBoundaryDisplay>> {
    Ok(
        visible_move_learning_text_pages(runtime_shell, text_target, nickname, nickname, move_id)?
            .into_iter()
            .map(|page| SpecialBoundaryDisplay {
                label: label.to_string(),
                details: vec![page],
            })
            .collect(),
    )
}

fn visible_move_tutor_forgot_text_boundaries(
    runtime_shell: &BevyRuntimeShell,
    nickname: &str,
    forgotten_move_id: &str,
) -> Result<VecDeque<SpecialBoundaryDisplay>> {
    Ok(visible_move_learning_text_pages(
        runtime_shell,
        "_MoveForgotText",
        nickname,
        forgotten_move_id,
        "",
    )?
    .into_iter()
    .map(|page| SpecialBoundaryDisplay {
        label: "MoveForgotText".to_string(),
        details: vec![page],
    })
    .collect())
}

fn install_visible_move_learn_result_sequence(
    runtime_shell: &mut BevyRuntimeShell,
    nickname: &str,
    forgotten_move_id: Option<&str>,
    learned_move_id: &str,
) -> Result<()> {
    runtime_shell.special_boundary = None;
    runtime_shell.special_boundary_queue.clear();
    runtime_shell.visible_special_text_pause_frames = None;
    runtime_shell.visible_internal_special_delay_frames = None;
    if let Some(forgotten_move_id) = forgotten_move_id {
        let mut count = visible_move_tutor_text_boundaries(
            runtime_shell,
            "Text_1_2_and_Poof",
            "Text_MoveForgetCount",
            nickname,
            learned_move_id,
        )?;
        runtime_shell.special_boundary = count.pop_front();
        anyhow::ensure!(
            count.is_empty(),
            "move-learning count rendered multiple source pages"
        );
        let mut forgot =
            visible_move_tutor_forgot_text_boundaries(runtime_shell, nickname, forgotten_move_id)?;
        forgot
            .front_mut()
            .context("move-learning forgot text rendered no source pages")?
            .label = "MoveForgotPoofText".to_string();
        runtime_shell.special_boundary_queue.extend(forgot);
        runtime_shell.visible_special_text_pause_frames = Some(30);
    }
    let learned = visible_move_tutor_text_boundaries(
        runtime_shell,
        "LearnedMoveText",
        "_LearnedMoveText",
        nickname,
        learned_move_id,
    )?;
    if runtime_shell.special_boundary.is_some() {
        runtime_shell.special_boundary_queue.extend(learned);
    } else {
        runtime_shell.special_boundary = learned.front().cloned();
        runtime_shell
            .special_boundary_queue
            .extend(learned.into_iter().skip(1));
        queue_visible_shell_sound_effect(runtime_shell, "SFX_DEX_FANFARE_50_79")?;
    }
    Ok(())
}

fn visible_move_learning_text_pages(
    runtime_shell: &BevyRuntimeShell,
    text_target: &str,
    nickname: &str,
    string_buffer_1: &str,
    move_id: &str,
) -> Result<Vec<String>> {
    let snapshot = runtime_shell.shell.presentation_snapshot()?;
    let mut named_buffers = snapshot.script_events.named_buffers.clone();
    named_buffers.insert("wMonOrItemNameBuffer".to_string(), nickname.to_string());
    named_buffers.insert(
        "STRING_BUFFER_1".to_string(),
        string_buffer_1.replace('_', " "),
    );
    let move_name = snapshot
        .moves
        .iter()
        .find(|move_data| move_data.move_id == move_id)
        .map(|move_data| move_data.name.replace('_', " "))
        .unwrap_or_else(|| move_id.replace('_', " "));
    named_buffers.insert("STRING_BUFFER_2".to_string(), move_name);
    let text = runtime_shell.shell.text_snapshot(text_target)?;
    let asm_text = text
        .asm_text
        .as_deref()
        .with_context(|| format!("move-learning text {text_target} has no exported ASM body"))?;
    let pages = render_visible_asm_text_pages(
        asm_text,
        &named_buffers,
        &snapshot.trainer.player_name,
        visible_rival_name(&snapshot),
        snapshot.progression.time.day_of_week,
    );
    anyhow::ensure!(
        !pages.is_empty(),
        "move-learning text {text_target} rendered no source pages"
    );
    Ok(pages)
}

fn visible_pending_move_learn_intro_pages(
    runtime_shell: &BevyRuntimeShell,
    nickname: &str,
    move_id: &str,
) -> Result<Vec<String>> {
    let mut pages = visible_move_learning_text_pages(
        runtime_shell,
        "_AskForgetMoveText",
        nickname,
        nickname,
        move_id,
    )?;
    pages
        .pop()
        .context("pending move-learning text has no final decision page")?;
    Ok(pages)
}

const KURT_APRICORN_ORDER: [&str; 7] = [
    "RED_APRICORN",
    "BLU_APRICORN",
    "YLW_APRICORN",
    "GRN_APRICORN",
    "WHT_APRICORN",
    "BLK_APRICORN",
    "PNK_APRICORN",
];

const BUENA_PRIZE_ORDER: [&str; 9] = [
    "ULTRA_BALL",
    "FULL_RESTORE",
    "NUGGET",
    "RARE_CANDY",
    "PROTEIN",
    "IRON",
    "CARBOS",
    "CALCIUM",
    "HP_UP",
];

fn visible_buena_prize_choices(snapshot: &RuntimeShellSnapshot) -> Result<Vec<(String, u8)>> {
    BUENA_PRIZE_ORDER
        .iter()
        .map(|item_id| {
            snapshot
                .special
                .buena_prizes
                .get(*item_id)
                .map(|cost| ((*item_id).to_string(), *cost))
                .with_context(|| format!("Buena prize {item_id} is missing from the runtime pack"))
        })
        .collect()
}

const VISIBLE_SLOT_REELS: [[&str; 15]; 3] = [
    [
        "SEVEN", "CHERRY", "STARYU", "PIKACHU", "SQUIRTLE", "SEVEN", "CHERRY", "STARYU", "PIKACHU",
        "SQUIRTLE", "POKEBALL", "CHERRY", "STARYU", "PIKACHU", "SQUIRTLE",
    ],
    [
        "SEVEN", "PIKACHU", "CHERRY", "SQUIRTLE", "STARYU", "POKEBALL", "PIKACHU", "CHERRY",
        "SQUIRTLE", "STARYU", "POKEBALL", "PIKACHU", "CHERRY", "SQUIRTLE", "STARYU",
    ],
    [
        "SEVEN", "PIKACHU", "CHERRY", "SQUIRTLE", "STARYU", "PIKACHU", "CHERRY", "SQUIRTLE",
        "STARYU", "PIKACHU", "POKEBALL", "CHERRY", "SQUIRTLE", "STARYU", "PIKACHU",
    ],
];

fn visible_slot_windows(offsets: [usize; 3]) -> [[String; 3]; 3] {
    std::array::from_fn(|reel| {
        std::array::from_fn(|row| VISIBLE_SLOT_REELS[reel][(offsets[reel] + row) % 15].to_string())
    })
}

fn visible_slot_has_match(machine: &VisibleSlotMachine) -> bool {
    let windows = &machine.windows;
    let same = |a: (usize, usize), b: (usize, usize), c: (usize, usize)| {
        windows[a.0][a.1] == windows[b.0][b.1] && windows[b.0][b.1] == windows[c.0][c.1]
    };
    same((0, 1), (1, 1), (2, 1))
        || machine.bet >= 2 && (same((0, 0), (1, 0), (2, 0)) || same((0, 2), (1, 2), (2, 2)))
        || machine.bet >= 3 && (same((0, 2), (1, 1), (2, 0)) || same((0, 0), (1, 1), (2, 2)))
}

fn open_visible_slot_machine_for_script_command(
    runtime_shell: &mut BevyRuntimeShell,
    source_script: &str,
    command_index: usize,
) -> Result<bool> {
    if compiled_special_routine_at(runtime_shell, source_script, command_index)?.as_deref()
        != Some("SlotMachine")
    {
        return Ok(false);
    }
    let snapshot = runtime_shell.shell.snapshot()?;
    let unavailable = if snapshot.trainer.coins == 0 {
        Some(("_NoCoinsText", vec!["You have no coins.".to_string()]))
    } else if carried_item_quantity(&snapshot, "COIN_CASE").unwrap_or(0) == 0 {
        Some((
            "_NoCoinCaseText",
            vec!["You don't have a".to_string(), "COIN CASE.".to_string()],
        ))
    } else {
        None
    };
    if let Some((label, details)) = unavailable {
        runtime_shell.special_boundary = Some(SpecialBoundaryDisplay {
            label: label.to_string(),
            details,
        });
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(true);
    }
    clear_visible_slot_runtime_state(runtime_shell);
    let lucky = runtime_shell
        .shell
        .session()
        .state()
        .script_runtime
        .script_value
        .as_deref()
        == Some("1");
    runtime_shell
        .shell
        .session_mut()
        .state_mut()
        .script_runtime
        .pending_slot_machine_input = Some(SlotMachineInput::Enter { lucky });
    let entered = runtime_shell
        .shell
        .apply_declared_special_routine("SlotMachine")?;
    if !matches!(
        entered.outcome.effect,
        SpecialRoutineEffect::SlotMachineEntered { .. }
    ) {
        anyhow::bail!("SlotMachine entry returned a different special effect");
    }
    let offsets = [14; 3];
    runtime_shell.visible_slot_machine = Some(VisibleSlotMachine {
        phase: VisibleSlotMachinePhase::Betting,
        animation: VisibleSlotMachineAnimation::None,
        yes_no_index: 0,
        bet: 3,
        coins: snapshot.trainer.coins,
        payout: 0,
        offsets,
        spin_ticks: [0; 3],
        spinning: [false; 3],
        next_reel: 1,
        actor: None,
        secondary_actor: None,
        background_y_offset: 0,
        windows: visible_slot_windows(offsets),
        message: "BET 3".to_string(),
    });
    set_shell_action_status(runtime_shell, "SLOT MACHINE");
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(true)
}

fn change_visible_slot_machine_bet(runtime_shell: &mut BevyRuntimeShell, delta: i8) -> Result<()> {
    let machine = runtime_shell
        .visible_slot_machine
        .as_mut()
        .context("no slot machine is open")?;
    match machine.phase {
        VisibleSlotMachinePhase::Betting => {
            machine.bet = (machine.bet as i8 + delta).clamp(1, 3) as u8;
            machine.message = format!("BET {}", machine.bet);
        }
        VisibleSlotMachinePhase::PlayAgain => {
            machine.yes_no_index = (machine.yes_no_index as i16 - i16::from(delta)).clamp(0, 1) as usize;
        }
        VisibleSlotMachinePhase::Spinning
        | VisibleSlotMachinePhase::Result
        | VisibleSlotMachinePhase::RanOut
        | VisibleSlotMachinePhase::Quitting => return Ok(()),
    }
    // VerticalMenu plays its click on A/B, not on cursor movement.
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(())
}

fn spin_visible_slot_machine(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let (phase, animation, yes_no_index, bet, coins) = runtime_shell
        .visible_slot_machine
        .as_ref()
        .map(|machine| {
            (
                machine.phase,
                machine.animation,
                machine.yes_no_index,
                machine.bet,
                machine.coins,
            )
        })
        .context("no slot machine is open")?;
    match phase {
        VisibleSlotMachinePhase::Result => {
            if animation == VisibleSlotMachineAnimation::AwaitResult {
                runtime_shell
                    .shell
                    .session_mut()
                    .state_mut()
                    .script_runtime
                    .pending_slot_machine_input = Some(SlotMachineInput::AcknowledgeResult);
                let result = runtime_shell
                    .shell
                    .apply_declared_special_routine("SlotMachine")?;
                let SpecialRoutineEffect::SlotMachineResultAcknowledged { can_play_again, .. } =
                    result.outcome.effect
                else {
                    anyhow::bail!(
                        "SlotMachine result acknowledgement returned a different special effect"
                    );
                };
                let machine = runtime_shell.visible_slot_machine.as_mut().unwrap();
                machine.animation = VisibleSlotMachineAnimation::None;
                machine.yes_no_index = 0;
                if !can_play_again {
                    machine.phase = VisibleSlotMachinePhase::RanOut;
                    machine.message = "DARN… RAN OUT OF\nCOINS…".to_string();
                } else {
                    machine.phase = VisibleSlotMachinePhase::PlayAgain;
                    machine.message = "PLAY AGAIN?".to_string();
                }
                mark_runtime_snapshot_dirty(runtime_shell);
            }
            return Ok(());
        }
        VisibleSlotMachinePhase::PlayAgain => {
            if yes_no_index == 1 {
                return close_visible_slot_machine(runtime_shell);
            }
            runtime_shell
                .shell
                .session_mut()
                .state_mut()
                .script_runtime
                .pending_slot_machine_input = Some(SlotMachineInput::Continue);
            let result = runtime_shell
                .shell
                .apply_declared_special_routine("SlotMachine")?;
            if !matches!(
                result.outcome.effect,
                SpecialRoutineEffect::SlotMachineReplayAccepted { .. }
            ) {
                anyhow::bail!("SlotMachine replay returned a different special effect");
            }
            let machine = runtime_shell.visible_slot_machine.as_mut().unwrap();
            machine.phase = VisibleSlotMachinePhase::Betting;
            machine.yes_no_index = 0;
            machine.payout = 0;
            machine.message = format!("BET {}", machine.bet);
            mark_runtime_snapshot_dirty(runtime_shell);
            return Ok(());
        }
        VisibleSlotMachinePhase::Spinning => {
            let machine = runtime_shell.visible_slot_machine.as_mut().unwrap();
            if let VisibleSlotMachineAnimation::Spinning {
                start_delay,
                requested_stop,
            } = &mut machine.animation
                && *start_delay == 0
                && !*requested_stop
            {
                *requested_stop = true;
                mark_runtime_snapshot_dirty(runtime_shell);
            }
            return Ok(());
        }
        VisibleSlotMachinePhase::Quitting => return Ok(()),
        VisibleSlotMachinePhase::RanOut => {
            if animation == VisibleSlotMachineAnimation::None {
                runtime_shell
                    .visible_slot_machine
                    .as_mut()
                    .unwrap()
                    .animation = VisibleSlotMachineAnimation::RanOutDelay {
                    frames_remaining: 60,
                };
                mark_runtime_snapshot_dirty(runtime_shell);
            }
            return Ok(());
        }
        VisibleSlotMachinePhase::Betting => {}
    }
    if coins < u16::from(bet) {
        runtime_shell.visible_slot_machine.as_mut().unwrap().message =
            "NEED MORE COINS".to_string();
        queue_visible_shell_sound_effect(runtime_shell, "SFX_WRONG")?;
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(());
    }
    let lucky = runtime_shell
        .shell
        .session()
        .state()
        .script_runtime
        .slot_machine
        .as_ref()
        .map(|machine| machine.lucky)
        .unwrap_or_else(|| {
            runtime_shell
                .shell
                .session()
                .state()
                .script_runtime
                .script_value
                .as_deref()
                == Some("1")
        });
    runtime_shell
        .shell
        .session_mut()
        .state_mut()
        .script_runtime
        .pending_slot_machine_input = Some(SlotMachineInput::Start { bet, lucky });
    queue_visible_shell_sound_effect(runtime_shell, "SFX_SLOT_MACHINE_START")?;
    let result = runtime_shell
        .shell
        .apply_declared_special_routine("SlotMachine")?;
    let SpecialRoutineEffect::SlotMachineStarted {
        coins,
        offsets,
        windows,
        ..
    } = result.outcome.effect
    else {
        anyhow::bail!("SlotMachine returned a different special effect");
    };
    let machine = runtime_shell
        .visible_slot_machine
        .as_mut()
        .context("slot machine closed during spin")?;
    machine.coins = coins;
    machine.payout = 0;
    machine.offsets = offsets;
    machine.spin_ticks = [0; 3];
    machine.spinning = [true; 3];
    machine.next_reel = 1;
    machine.windows = windows;
    machine.phase = VisibleSlotMachinePhase::Spinning;
    machine.animation = VisibleSlotMachineAnimation::Spinning {
        start_delay: 32,
        requested_stop: false,
    };
    machine.message = "PRESS A".to_string();
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(())
}

fn resolve_visible_slot_stop(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let (reel, offsets) = runtime_shell
        .visible_slot_machine
        .as_ref()
        .map(|machine| (machine.next_reel, machine.offsets))
        .context("no slot machine is open")?;
    runtime_shell
        .shell
        .session_mut()
        .state_mut()
        .script_runtime
        .pending_slot_machine_input = Some(SlotMachineInput::StopReel {
        reel,
        offsets: offsets
            .map(|offset| u8::try_from(offset).expect("visible Slot Machine offset fits byte")),
    });
    let result = runtime_shell
        .shell
        .apply_declared_special_routine("SlotMachine")?;
    let SpecialRoutineEffect::SlotMachineReelStopped {
        reel,
        mode,
        animation_start_offset,
        animation_count,
        offsets: target_offsets,
        coins,
        ..
    } = result.outcome.effect
    else {
        anyhow::bail!("SlotMachine stop returned a different special effect");
    };
    let mode = match mode.as_str() {
        "normal" => VisibleSlotStopMode::Normal,
        "skip_to_seven" => VisibleSlotStopMode::SkipToSeven,
        "slow" => VisibleSlotStopMode::Slow,
        "golem" => VisibleSlotStopMode::Golem,
        "chansey" => VisibleSlotStopMode::Chansey,
        value => anyhow::bail!("unknown slot stop mode {value}"),
    };
    let machine = runtime_shell.visible_slot_machine.as_mut().unwrap();
    machine.coins = coins;
    machine.actor = None;
    machine.secondary_actor = None;
    machine.background_y_offset = 0;
    let target = target_offsets[usize::from(reel - 1)];
    machine.animation = match mode {
        VisibleSlotStopMode::Normal | VisibleSlotStopMode::SkipToSeven => {
            VisibleSlotMachineAnimation::Stopping {
                reel,
                mode,
                target,
                pause: u16::from(
                    mode == VisibleSlotStopMode::SkipToSeven
                        && target != offsets[usize::from(reel - 1)],
                ) * 32,
                steps: 0,
                minimum_steps: 0,
                terminal_delay: 0,
            }
        }
        VisibleSlotStopMode::Slow | VisibleSlotStopMode::Golem | VisibleSlotStopMode::Chansey => {
            VisibleSlotMachineAnimation::SpecialPrepare {
                mode,
                target,
                start_offset: animation_start_offset,
                count: animation_count,
            }
        }
    };
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(())
}

fn advance_visible_spinning_reels(machine: &mut VisibleSlotMachine, except: Option<usize>) {
    for reel in 0..3 {
        if !machine.spinning[reel] || except == Some(reel) {
            continue;
        }
        machine.spin_ticks[reel] += 1;
        if machine.spin_ticks[reel] == 4 {
            machine.spin_ticks[reel] = 0;
            machine.offsets[reel] = (machine.offsets[reel] + 1) % 15;
        }
    }
    machine.windows = visible_slot_windows(machine.offsets);
}

fn tick_visible_slot_actor(machine: &mut VisibleSlotMachine) {
    machine.actor = match machine.actor {
        Some(VisibleSlotActor::Golem {
            x,
            y_offset,
            mut frame,
            mut frame_tick,
            ..
        }) => {
            frame_tick += 1;
            if frame_tick == 8 {
                frame_tick = 0;
                frame = (frame + 1) % 4;
            }
            Some(VisibleSlotActor::Golem {
                x,
                y_offset,
                frame,
                frame_tick,
                flip_x: frame == 3,
                flip_y: frame == 2,
            })
        }
        Some(VisibleSlotActor::Chansey {
            x,
            mut frame,
            mut frame_tick,
            finishing,
        }) => {
            frame_tick += 1;
            if frame_tick == 8 {
                frame_tick = 0;
                if finishing {
                    frame = (frame + 1).min(4);
                } else {
                    frame = (frame + 1) % 4;
                }
            }
            Some(VisibleSlotActor::Chansey {
                x,
                frame,
                frame_tick,
                finishing,
            })
        }
        actor => actor,
    };
}

fn visible_slot_terminal_stop(machine: &mut VisibleSlotMachine, target: usize) {
    machine.actor = None;
    machine.secondary_actor = None;
    machine.background_y_offset = 0;
    machine.spinning[2] = false;
    machine.spin_ticks[2] = 0;
    machine.animation = VisibleSlotMachineAnimation::Stopping {
        reel: 3,
        mode: VisibleSlotStopMode::Normal,
        target,
        pause: 0,
        steps: 0,
        minimum_steps: 0,
        terminal_delay: 0,
    };
}

fn resolve_visible_slot_result(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    runtime_shell
        .shell
        .session_mut()
        .state_mut()
        .script_runtime
        .pending_slot_machine_input = Some(SlotMachineInput::ResolveResult);
    let result = runtime_shell
        .shell
        .apply_declared_special_routine("SlotMachine")?;
    let SpecialRoutineEffect::SlotMachineResult {
        payout,
        matched_symbol,
        coins,
        ..
    } = result.outcome.effect
    else {
        anyhow::bail!("SlotMachine result returned a different special effect");
    };
    let result_sound = matched_symbol.as_deref().map(|symbol| match symbol {
        "SEVEN" => "SFX_2ND_PLACE",
        "POKEBALL" => "SFX_3RD_PLACE",
        _ => "SFX_PRESENT",
    });
    let machine = runtime_shell.visible_slot_machine.as_mut().unwrap();
    machine.phase = VisibleSlotMachinePhase::Result;
    machine.coins = coins;
    machine.payout = payout;
    machine.message = if payout > 0 {
        format!("WIN {payout}")
    } else {
        "DARN".to_string()
    };
    if let Some(sound) = result_sound {
        queue_visible_shell_sound_effect(runtime_shell, sound)?;
        runtime_shell
            .visible_slot_machine
            .as_mut()
            .unwrap()
            .animation = VisibleSlotMachineAnimation::WaitResult { payout };
    } else {
        runtime_shell
            .visible_slot_machine
            .as_mut()
            .unwrap()
            .animation = VisibleSlotMachineAnimation::AwaitResult;
    }
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(())
}

fn advance_visible_slot_machine_animation(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let animation = runtime_shell
        .visible_slot_machine
        .as_ref()
        .map(|machine| machine.animation)
        .context("no slot machine is open")?;
    match animation {
        VisibleSlotMachineAnimation::None | VisibleSlotMachineAnimation::AwaitResult => Ok(()),
        VisibleSlotMachineAnimation::Spinning {
            start_delay,
            requested_stop,
        } => {
            let selected = usize::from(
                runtime_shell
                    .visible_slot_machine
                    .as_ref()
                    .unwrap()
                    .next_reel
                    - 1,
            );
            if requested_stop
                && runtime_shell
                    .visible_slot_machine
                    .as_ref()
                    .unwrap()
                    .spin_ticks[selected]
                    == 0
            {
                resolve_visible_slot_stop(runtime_shell)?;
                return advance_visible_slot_machine_animation(runtime_shell);
            }
            let machine = runtime_shell.visible_slot_machine.as_mut().unwrap();
            advance_visible_spinning_reels(machine, None);
            machine.animation = VisibleSlotMachineAnimation::Spinning {
                start_delay: start_delay.saturating_sub(1),
                requested_stop,
            };
            mark_runtime_snapshot_dirty(runtime_shell);
            Ok(())
        }
        VisibleSlotMachineAnimation::SpecialPrepare {
            mode,
            target,
            start_offset,
            count,
        } => {
            let machine = runtime_shell.visible_slot_machine.as_mut().unwrap();
            if machine.offsets[2] != start_offset {
                machine.spin_ticks[2] += 1;
                if machine.spin_ticks[2] == 4 {
                    machine.spin_ticks[2] = 0;
                    machine.offsets[2] = (machine.offsets[2] + 1) % 15;
                    machine.windows = visible_slot_windows(machine.offsets);
                }
                mark_runtime_snapshot_dirty(runtime_shell);
                return Ok(());
            }
            machine.spinning[2] = false;
            machine.spin_ticks[2] = 0;
            machine.animation = VisibleSlotMachineAnimation::SpecialWait {
                mode,
                target,
                count,
                frames_remaining: 16,
            };
            queue_visible_shell_sound_effect(runtime_shell, "SFX_STOP_SLOT")?;
            mark_runtime_snapshot_dirty(runtime_shell);
            Ok(())
        }
        VisibleSlotMachineAnimation::SpecialWait {
            mode,
            target,
            count,
            frames_remaining,
        } => {
            if frames_remaining > 1 {
                runtime_shell
                    .visible_slot_machine
                    .as_mut()
                    .unwrap()
                    .animation = VisibleSlotMachineAnimation::SpecialWait {
                    mode,
                    target,
                    count,
                    frames_remaining: frames_remaining - 1,
                };
                mark_runtime_snapshot_dirty(runtime_shell);
                return Ok(());
            }
            let machine = runtime_shell.visible_slot_machine.as_mut().unwrap();
            machine.animation = match mode {
                VisibleSlotStopMode::Slow => VisibleSlotMachineAnimation::SlowAdvance {
                    target,
                    steps_remaining: count,
                    // Setting rate 1 consumes the first of sixteen subpixel
                    // advances in this same source frame.
                    frames_until_step: 15,
                },
                VisibleSlotStopMode::Golem => VisibleSlotMachineAnimation::Golem {
                    target,
                    remaining: count,
                    phase: VisibleSlotGolemPhase::Init,
                    phase_frame: 0,
                },
                VisibleSlotStopMode::Chansey => {
                    machine.actor = Some(VisibleSlotActor::Chansey {
                        x: 96,
                        frame: 0,
                        frame_tick: 0,
                        finishing: false,
                    });
                    VisibleSlotMachineAnimation::Chansey {
                        target,
                        remaining_eggs: count,
                        phase: VisibleSlotChanseyPhase::Walk,
                        phase_frame: 0,
                    }
                }
                _ => unreachable!("only special reel modes enter the fixed wait"),
            };
            mark_runtime_snapshot_dirty(runtime_shell);
            advance_visible_slot_machine_animation(runtime_shell)
        }
        VisibleSlotMachineAnimation::SlowAdvance {
            target,
            steps_remaining,
            frames_until_step,
        } => {
            let machine = runtime_shell.visible_slot_machine.as_mut().unwrap();
            if steps_remaining == 0 {
                visible_slot_terminal_stop(machine, target);
                mark_runtime_snapshot_dirty(runtime_shell);
                return Ok(());
            }
            if frames_until_step > 0 {
                machine.animation = VisibleSlotMachineAnimation::SlowAdvance {
                    target,
                    steps_remaining,
                    frames_until_step: frames_until_step - 1,
                };
            } else {
                machine.offsets[2] = (machine.offsets[2] + 1) % 15;
                machine.windows = visible_slot_windows(machine.offsets);
                let remaining = steps_remaining - 1;
                machine.animation = VisibleSlotMachineAnimation::SlowAdvance {
                    target,
                    steps_remaining: remaining,
                    frames_until_step: 15,
                };
                if remaining > 0 {
                    queue_visible_shell_sound_effect(runtime_shell, "SFX_GOT_SAFARI_BALLS")?;
                }
            }
            mark_runtime_snapshot_dirty(runtime_shell);
            Ok(())
        }
        VisibleSlotMachineAnimation::Golem {
            target,
            remaining,
            phase,
            phase_frame,
        } => {
            let machine = runtime_shell.visible_slot_machine.as_mut().unwrap();
            let mut sound = None;
            match phase {
                VisibleSlotGolemPhase::Init => {
                    if remaining == 0 {
                        visible_slot_terminal_stop(machine, target);
                    } else {
                        let angle = 0x30;
                        machine.actor = Some(VisibleSlotActor::Golem {
                            x: 96,
                            y_offset: visible_battle_anim_sine(angle, 14 * 8) as i16,
                            frame: 0,
                            frame_tick: 0,
                            flip_x: false,
                            flip_y: false,
                        });
                        machine.animation = VisibleSlotMachineAnimation::Golem {
                            target,
                            remaining: remaining - 1,
                            phase: VisibleSlotGolemPhase::Fall,
                            phase_frame: angle - 1,
                        };
                    }
                }
                VisibleSlotGolemPhase::Fall => {
                    if phase_frame >= 0x20 {
                        if let Some(VisibleSlotActor::Golem { y_offset, .. }) =
                            machine.actor.as_mut()
                        {
                            *y_offset = visible_battle_anim_sine(phase_frame, 14 * 8) as i16;
                        }
                        tick_visible_slot_actor(machine);
                        machine.animation = VisibleSlotMachineAnimation::Golem {
                            target,
                            remaining,
                            phase,
                            phase_frame: phase_frame - 1,
                        };
                    } else {
                        sound = Some("SFX_PLACE_PUZZLE_PIECE_DOWN");
                        machine.animation = VisibleSlotMachineAnimation::Golem {
                            target,
                            remaining,
                            phase: VisibleSlotGolemPhase::Roll,
                            phase_frame: 0,
                        };
                    }
                }
                VisibleSlotGolemPhase::Roll => {
                    let x_offset = phase_frame.saturating_mul(2);
                    if x_offset >= 9 * 8 {
                        machine.background_y_offset = 0;
                        machine.actor = Some(VisibleSlotActor::Golem {
                            x: 96 + i16::from(x_offset + 2),
                            y_offset: 0,
                            frame: 0,
                            frame_tick: 0,
                            flip_x: false,
                            flip_y: false,
                        });
                        machine.animation = VisibleSlotMachineAnimation::Golem {
                            target,
                            remaining,
                            phase: VisibleSlotGolemPhase::Init,
                            phase_frame: 0,
                        };
                    } else {
                        if phase_frame == 1 {
                            machine.offsets[2] = (machine.offsets[2] + 1) % 15;
                            machine.windows = visible_slot_windows(machine.offsets);
                        }
                        if phase_frame % 2 == 0 {
                            machine.background_y_offset = -machine.background_y_offset;
                            if machine.background_y_offset == 0 {
                                machine.background_y_offset = -2;
                            }
                        }
                        if let Some(VisibleSlotActor::Golem { x, y_offset, .. }) =
                            machine.actor.as_mut()
                        {
                            *x = 96 + i16::from(x_offset + 2);
                            *y_offset = 0;
                        }
                        tick_visible_slot_actor(machine);
                        machine.animation = VisibleSlotMachineAnimation::Golem {
                            target,
                            remaining,
                            phase,
                            phase_frame: phase_frame + 1,
                        };
                    }
                }
            }
            if let Some(sound) = sound {
                queue_visible_shell_sound_effect(runtime_shell, sound)?;
            }
            mark_runtime_snapshot_dirty(runtime_shell);
            Ok(())
        }
        VisibleSlotMachineAnimation::Chansey {
            target,
            remaining_eggs,
            phase,
            phase_frame,
        } => {
            let machine = runtime_shell.visible_slot_machine.as_mut().unwrap();
            let mut sound = None;
            match phase {
                VisibleSlotChanseyPhase::Walk => {
                    let old_x = match machine.actor {
                        Some(VisibleSlotActor::Chansey { x, .. }) => x,
                        _ => 96,
                    };
                    if let Some(VisibleSlotActor::Chansey { x, .. }) = machine.actor.as_mut() {
                        *x += 1;
                    }
                    tick_visible_slot_actor(machine);
                    if old_x == 13 * 8 {
                        machine.animation = VisibleSlotMachineAnimation::Chansey {
                            target,
                            remaining_eggs,
                            phase: VisibleSlotChanseyPhase::PrepareEgg,
                            phase_frame: 0,
                        };
                    } else {
                        if old_x & 0xf == 0 {
                            sound = Some("SFX_JUMP_OVER_LEDGE");
                        }
                        machine.animation = VisibleSlotMachineAnimation::Chansey {
                            target,
                            remaining_eggs,
                            phase,
                            phase_frame: phase_frame + 1,
                        };
                    }
                }
                VisibleSlotChanseyPhase::PrepareEgg => {
                    if let Some(VisibleSlotActor::Chansey {
                        frame,
                        frame_tick,
                        finishing,
                        ..
                    }) = machine.actor.as_mut()
                    {
                        if phase_frame == 0 {
                            *frame = 0;
                            *frame_tick = 0;
                            *finishing = true;
                        }
                    }
                    tick_visible_slot_actor(machine);
                    if phase_frame >= 8 {
                        machine.secondary_actor =
                            Some(VisibleSlotActor::Egg { x: 96, y_offset: 0 });
                        machine.animation = VisibleSlotMachineAnimation::Chansey {
                            target,
                            remaining_eggs,
                            phase: VisibleSlotChanseyPhase::Egg,
                            phase_frame: 0,
                        };
                    } else {
                        machine.animation = VisibleSlotMachineAnimation::Chansey {
                            target,
                            remaining_eggs,
                            phase,
                            phase_frame: phase_frame + 1,
                        };
                    }
                }
                VisibleSlotChanseyPhase::Egg => {
                    let index = 0_u8.wrapping_sub(phase_frame);
                    let mut landed = false;
                    if let Some(VisibleSlotActor::Egg { x, y_offset }) =
                        machine.secondary_actor.as_mut()
                    {
                        if index & 1 != 0 {
                            if *x >= 15 * 8 {
                                landed = true;
                            } else {
                                *x += 1;
                            }
                        }
                        if !landed {
                            *y_offset = visible_battle_anim_sine(index, 32) as i16;
                        }
                    }
                    tick_visible_slot_actor(machine);
                    if landed {
                        machine.secondary_actor = None;
                        sound = Some("SFX_PLACE_PUZZLE_PIECE_DOWN");
                        machine.animation = VisibleSlotMachineAnimation::Chansey {
                            target,
                            remaining_eggs,
                            phase: VisibleSlotChanseyPhase::DropReel,
                            phase_frame: 0,
                        };
                    } else {
                        machine.animation = VisibleSlotMachineAnimation::Chansey {
                            target,
                            remaining_eggs,
                            phase,
                            phase_frame: phase_frame + 1,
                        };
                    }
                }
                VisibleSlotChanseyPhase::DropReel => {
                    machine.offsets[2] = (machine.offsets[2] + 1) % 15;
                    machine.windows = visible_slot_windows(machine.offsets);
                    tick_visible_slot_actor(machine);
                    machine.animation = VisibleSlotMachineAnimation::Chansey {
                        target,
                        remaining_eggs,
                        phase: if phase_frame >= 16 {
                            VisibleSlotChanseyPhase::CheckMatch
                        } else {
                            phase
                        },
                        phase_frame: if phase_frame >= 16 {
                            0
                        } else {
                            phase_frame + 1
                        },
                    };
                }
                VisibleSlotChanseyPhase::CheckMatch => {
                    let remaining = remaining_eggs - 1;
                    if remaining == 0 {
                        visible_slot_terminal_stop(machine, target);
                    } else {
                        machine.animation = VisibleSlotMachineAnimation::Chansey {
                            target,
                            remaining_eggs: remaining,
                            phase: VisibleSlotChanseyPhase::PrepareEgg,
                            phase_frame: 0,
                        };
                    }
                }
            }
            if let Some(sound) = sound {
                queue_visible_shell_sound_effect(runtime_shell, sound)?;
            }
            mark_runtime_snapshot_dirty(runtime_shell);
            Ok(())
        }
        VisibleSlotMachineAnimation::Stopping {
            reel,
            mode,
            target,
            pause,
            steps,
            minimum_steps,
            terminal_delay,
        } => {
            let selected = usize::from(reel - 1);
            let machine = runtime_shell.visible_slot_machine.as_mut().unwrap();
            advance_visible_spinning_reels(machine, Some(selected));
            if terminal_delay > 0 {
                if terminal_delay > 1 {
                    machine.animation = VisibleSlotMachineAnimation::Stopping {
                        reel,
                        mode,
                        target,
                        pause: 0,
                        steps,
                        minimum_steps,
                        terminal_delay: terminal_delay - 1,
                    };
                } else {
                    machine.next_reel = reel + 1;
                    let mut resolve_without_flash = false;
                    if reel < 3 {
                        machine.animation = VisibleSlotMachineAnimation::Spinning {
                            start_delay: 0,
                            requested_stop: false,
                        };
                        machine.message = "PRESS A".to_string();
                    } else {
                        resolve_without_flash = !visible_slot_has_match(machine);
                        machine.animation = if resolve_without_flash {
                            VisibleSlotMachineAnimation::AwaitResult
                        } else {
                            VisibleSlotMachineAnimation::FlashResult {
                                frames_remaining: 16,
                            }
                        };
                    }
                    queue_visible_shell_sound_effect(runtime_shell, "SFX_STOP_SLOT")?;
                    if resolve_without_flash {
                        resolve_visible_slot_result(runtime_shell)?;
                    }
                }
                mark_runtime_snapshot_dirty(runtime_shell);
                return Ok(());
            }
            if pause > 0 {
                machine.animation = VisibleSlotMachineAnimation::Stopping {
                    reel,
                    mode,
                    target,
                    pause: pause - 1,
                    steps,
                    minimum_steps,
                    terminal_delay: 0,
                };
                mark_runtime_snapshot_dirty(runtime_shell);
                return Ok(());
            }
            let interval = match mode {
                VisibleSlotStopMode::Normal => 4,
                VisibleSlotStopMode::SkipToSeven | VisibleSlotStopMode::Golem => 2,
                VisibleSlotStopMode::Slow => 16,
                VisibleSlotStopMode::Chansey => 1,
            };
            let at_target = machine.offsets[selected] == target && steps >= minimum_steps;
            if at_target {
                machine.spinning[selected] = false;
                machine.spin_ticks[selected] = 0;
                machine.animation = VisibleSlotMachineAnimation::Stopping {
                    reel,
                    mode,
                    target,
                    pause: 0,
                    steps,
                    minimum_steps,
                    terminal_delay: 4,
                };
            } else {
                machine.spin_ticks[selected] += 1;
                let mut next_steps = steps;
                if machine.spin_ticks[selected] >= interval {
                    machine.spin_ticks[selected] = 0;
                    machine.offsets[selected] = (machine.offsets[selected] + 1) % 15;
                    next_steps += 1;
                    machine.windows = visible_slot_windows(machine.offsets);
                }
                machine.animation = VisibleSlotMachineAnimation::Stopping {
                    reel,
                    mode,
                    target,
                    pause: 0,
                    steps: next_steps,
                    minimum_steps,
                    terminal_delay: 0,
                };
            }
            mark_runtime_snapshot_dirty(runtime_shell);
            Ok(())
        }
        VisibleSlotMachineAnimation::FlashResult { frames_remaining } => {
            if frames_remaining > 1 {
                runtime_shell
                    .visible_slot_machine
                    .as_mut()
                    .unwrap()
                    .animation = VisibleSlotMachineAnimation::FlashResult {
                    frames_remaining: frames_remaining - 1,
                };
                mark_runtime_snapshot_dirty(runtime_shell);
                Ok(())
            } else {
                resolve_visible_slot_result(runtime_shell)
            }
        }
        VisibleSlotMachineAnimation::QuitWaitBefore => {
            if !visible_wait_sfx_finished(runtime_shell) {
                return Ok(());
            }
            queue_visible_shell_sound_effect(runtime_shell, "SFX_QUIT_SLOTS")?;
            runtime_shell
                .visible_slot_machine
                .as_mut()
                .unwrap()
                .animation = VisibleSlotMachineAnimation::QuitWaitAfter;
            mark_runtime_snapshot_dirty(runtime_shell);
            Ok(())
        }
        VisibleSlotMachineAnimation::QuitWaitAfter => {
            if !visible_wait_sfx_finished(runtime_shell) {
                return Ok(());
            }
            runtime_shell.visible_slot_machine = None;
            clear_visible_slot_runtime_state(runtime_shell);
            mark_runtime_snapshot_dirty(runtime_shell);
            continue_visible_script_after_prompt(runtime_shell)
        }
        VisibleSlotMachineAnimation::RanOutDelay { frames_remaining } => {
            let machine = runtime_shell.visible_slot_machine.as_mut().unwrap();
            if frames_remaining > 1 {
                machine.animation = VisibleSlotMachineAnimation::RanOutDelay {
                    frames_remaining: frames_remaining - 1,
                };
            } else {
                machine.phase = VisibleSlotMachinePhase::Quitting;
                machine.animation = VisibleSlotMachineAnimation::QuitWaitBefore;
                machine.message.clear();
            }
            mark_runtime_snapshot_dirty(runtime_shell);
            Ok(())
        }
        VisibleSlotMachineAnimation::WaitStart {
            payout,
            result_sound,
        } => {
            if !visible_wait_sfx_finished(runtime_shell) {
                return Ok(());
            }
            if let Some(sound) = result_sound {
                queue_visible_shell_sound_effect(runtime_shell, sound)?;
                runtime_shell
                    .visible_slot_machine
                    .as_mut()
                    .unwrap()
                    .animation = VisibleSlotMachineAnimation::WaitResult { payout };
            } else {
                runtime_shell
                    .visible_slot_machine
                    .as_mut()
                    .unwrap()
                    .animation = VisibleSlotMachineAnimation::AwaitResult;
            }
            mark_runtime_snapshot_dirty(runtime_shell);
            Ok(())
        }
        VisibleSlotMachineAnimation::WaitResult { payout } => {
            if !visible_wait_sfx_finished(runtime_shell) {
                return Ok(());
            }
            runtime_shell
                .visible_slot_machine
                .as_mut()
                .unwrap()
                .animation = VisibleSlotMachineAnimation::Payout {
                remaining: payout,
                frames_until_coin: 1,
                delay_counter: 0,
            };
            mark_runtime_snapshot_dirty(runtime_shell);
            Ok(())
        }
        VisibleSlotMachineAnimation::Payout {
            remaining,
            frames_until_coin,
            delay_counter,
        } => {
            if frames_until_coin > 0 {
                runtime_shell
                    .visible_slot_machine
                    .as_mut()
                    .unwrap()
                    .animation = VisibleSlotMachineAnimation::Payout {
                    remaining,
                    frames_until_coin: frames_until_coin - 1,
                    delay_counter: delay_counter + 1,
                };
            } else if remaining == 0 {
                runtime_shell
                    .visible_slot_machine
                    .as_mut()
                    .unwrap()
                    .animation = VisibleSlotMachineAnimation::AwaitResult;
            } else {
                runtime_shell
                    .shell
                    .session_mut()
                    .state_mut()
                    .script_runtime
                    .pending_slot_machine_input = Some(SlotMachineInput::PayoutFrame);
                let result = runtime_shell
                    .shell
                    .apply_declared_special_routine("SlotMachine")?;
                let SpecialRoutineEffect::SlotMachinePayout {
                    coins_before,
                    payout_remaining,
                    coins,
                    ..
                } = result.outcome.effect
                else {
                    anyhow::bail!("SlotMachine payout returned a different special effect");
                };
                let next_delay_counter = delay_counter + 1;
                let machine = runtime_shell.visible_slot_machine.as_mut().unwrap();
                machine.coins = coins;
                machine.payout = payout_remaining;
                machine.animation = VisibleSlotMachineAnimation::Payout {
                    remaining: payout_remaining,
                    frames_until_coin: 1,
                    delay_counter: next_delay_counter,
                };
                if coins > coins_before && next_delay_counter & 7 != 0 {
                    queue_visible_shell_sound_effect(runtime_shell, "SFX_GET_COIN_FROM_SLOTS")?;
                }
            }
            mark_runtime_snapshot_dirty(runtime_shell);
            Ok(())
        }
    }
}

fn clear_visible_slot_runtime_state(runtime_shell: &mut BevyRuntimeShell) {
    let script_runtime = &mut runtime_shell.shell.session_mut().state_mut().script_runtime;
    script_runtime.slot_machine = None;
    script_runtime.pending_slot_machine_input = None;
}

fn close_visible_slot_machine(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    if let Some(machine) = runtime_shell.visible_slot_machine.as_ref()
        && matches!(
            machine.phase,
            VisibleSlotMachinePhase::Result | VisibleSlotMachinePhase::RanOut
        )
    {
        return spin_visible_slot_machine(runtime_shell);
    }
    if runtime_shell
        .visible_slot_machine
        .as_ref()
        .is_some_and(|machine| {
            matches!(
                machine.phase,
                VisibleSlotMachinePhase::Spinning | VisibleSlotMachinePhase::Quitting
            )
        })
    {
        return Ok(());
    }
    let core_can_quit = runtime_shell
        .shell
        .session()
        .state()
        .script_runtime
        .slot_machine
        .as_ref()
        .is_some_and(|machine| {
            matches!(
                machine.phase,
                SlotMachinePhase::Betting | SlotMachinePhase::PlayAgain
            )
        });
    if core_can_quit {
        runtime_shell
            .shell
            .session_mut()
            .state_mut()
            .script_runtime
            .pending_slot_machine_input = Some(SlotMachineInput::Quit);
        let result = runtime_shell
            .shell
            .apply_declared_special_routine("SlotMachine")?;
        if !matches!(
            result.outcome.effect,
            SpecialRoutineEffect::SlotMachineExited { .. }
        ) {
            anyhow::bail!("SlotMachine quit returned a different special effect");
        }
    }
    let machine = runtime_shell
        .visible_slot_machine
        .as_mut()
        .context("no slot machine is open")?;
    machine.phase = VisibleSlotMachinePhase::Quitting;
    machine.animation = VisibleSlotMachineAnimation::QuitWaitBefore;
    machine.message.clear();
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(())
}

fn open_visible_card_flip_for_script_command(
    runtime_shell: &mut BevyRuntimeShell,
    source_script: &str,
    command_index: usize,
) -> Result<bool> {
    if compiled_special_routine_at(runtime_shell, source_script, command_index)?.as_deref()
        != Some("CardFlip")
    {
        return Ok(false);
    }
    let snapshot = runtime_shell.shell.snapshot()?;
    let unavailable = if carried_item_quantity(&snapshot, "COIN_CASE").unwrap_or(0) == 0 {
        Some((
            "_NoCoinCaseText",
            vec!["You don't have a".to_string(), "COIN CASE.".to_string()],
        ))
    } else {
        None
    };
    if let Some((label, details)) = unavailable {
        runtime_shell.special_boundary = Some(SpecialBoundaryDisplay {
            label: label.to_string(),
            details,
        });
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(true);
    }
    runtime_shell.visible_card_flip = Some(VisibleCardFlip {
        phase: VisibleCardFlipPhase::AskPlay,
        animation: VisibleCardFlipAnimation::None,
        yes_no_index: 0,
        which_card: 0,
        bet_x: 2,
        bet_y: 2,
        round: 0,
        face_card: None,
        coins: snapshot.trainer.coins,
        payout: 0,
        deck: Vec::new(),
        revealed: vec![false; 24],
        message: "PLAY WITH THREE COINS?".to_string(),
    });
    let script_runtime = &mut runtime_shell.shell.session_mut().state_mut().script_runtime;
    script_runtime.card_flip = None;
    script_runtime.pending_card_flip_input = None;
    set_shell_action_status(runtime_shell, "CARD FLIP");
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(true)
}

fn move_visible_card_flip_cursor(
    runtime_shell: &mut BevyRuntimeShell,
    dx: isize,
    dy: isize,
) -> Result<()> {
    let game = runtime_shell
        .visible_card_flip
        .as_mut()
        .context("no Card Flip game is open")?;
    let sound = match game.phase {
        VisibleCardFlipPhase::AskPlay | VisibleCardFlipPhase::PlayAgain => {
            game.yes_no_index = (game.yes_no_index as isize + dy.signum()).clamp(0, 1) as usize;
            // YesNoBox uses VerticalMenu; only confirmation/cancellation clicks.
            mark_runtime_snapshot_dirty(runtime_shell);
            return Ok(());
        }
        VisibleCardFlipPhase::ChooseCard
        | VisibleCardFlipPhase::Result
        | VisibleCardFlipPhase::Shuffled => None,
        VisibleCardFlipPhase::PlaceBet => {
            let before = (game.bet_x, game.bet_y);
            if dx < 0 {
                if game.bet_y == 0 {
                    game.bet_x &= !1;
                    if game.bet_x < 3 {
                        game.bet_x = 1;
                        game.bet_y = 2;
                    } else {
                        game.bet_x -= 2;
                    }
                } else if game.bet_y == 1 && game.bet_x < 3 {
                    game.bet_x = 1;
                    game.bet_y = 2;
                } else if game.bet_x > 0 {
                    game.bet_x -= 1;
                }
            } else if dx > 0 {
                if game.bet_y == 0 {
                    game.bet_x &= !1;
                    if game.bet_x < 4 {
                        game.bet_x += 2;
                    }
                } else if game.bet_x < 5 {
                    game.bet_x += 1;
                }
            } else if dy < 0 {
                if game.bet_x == 0 {
                    game.bet_y &= !1;
                    if game.bet_y < 3 {
                        game.bet_x = 2;
                        game.bet_y = 1;
                    } else {
                        game.bet_y -= 2;
                    }
                } else if game.bet_x == 1 && game.bet_y < 3 {
                    game.bet_x = 2;
                    game.bet_y = 1;
                } else if game.bet_y > 0 {
                    game.bet_y -= 1;
                }
            } else if dy > 0 {
                if game.bet_x == 0 {
                    game.bet_y &= !1;
                    if game.bet_y < 6 {
                        game.bet_y += 2;
                    }
                } else if game.bet_y < 7 {
                    game.bet_y += 1;
                }
            }
            ((game.bet_x, game.bet_y) != before).then_some("SFX_POKEBALLS_PLACED_ON_TABLE")
        }
        VisibleCardFlipPhase::NotEnoughCoins => return Ok(()),
    };
    if let Some(sound) = sound {
        queue_visible_shell_sound_effect(runtime_shell, sound)?;
        mark_runtime_snapshot_dirty(runtime_shell);
    }
    Ok(())
}

fn flip_visible_card(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let phase = runtime_shell
        .visible_card_flip
        .as_ref()
        .map(|game| (game.phase.clone(), game.yes_no_index, game.coins))
        .context("no Card Flip game is open")?;
    match phase {
        (VisibleCardFlipPhase::AskPlay, 1, _) | (VisibleCardFlipPhase::PlayAgain, 1, _) => {
            return close_visible_card_flip(runtime_shell);
        }
        (VisibleCardFlipPhase::AskPlay, _, _) => {
            return start_visible_card_flip_round(runtime_shell);
        }
        (VisibleCardFlipPhase::PlayAgain, _, _) => {
            return start_visible_card_flip_round(runtime_shell);
        }
        (VisibleCardFlipPhase::NotEnoughCoins, _, _) => {
            return close_visible_card_flip(runtime_shell);
        }
        (VisibleCardFlipPhase::ChooseCard, _, _) => {
            let game = runtime_shell.visible_card_flip.as_mut().unwrap();
            if !matches!(game.animation, VisibleCardFlipAnimation::Cycle { .. }) {
                return Ok(());
            }
            game.animation = VisibleCardFlipAnimation::SelectFlash { frame: 0 };
            queue_visible_shell_sound_effect(runtime_shell, "SFX_SLOT_MACHINE_START")?;
            mark_runtime_snapshot_dirty(runtime_shell);
            return Ok(());
        }
        (VisibleCardFlipPhase::Result, _, _) => {
            return acknowledge_visible_card_flip_result(runtime_shell);
        }
        (VisibleCardFlipPhase::Shuffled, _, _) => {
            return start_visible_card_flip_round(runtime_shell);
        }
        (VisibleCardFlipPhase::PlaceBet, _, _) => {}
    }
    runtime_shell
        .visible_card_flip
        .as_mut()
        .context("no Card Flip game is open")?
        .animation = VisibleCardFlipAnimation::WaitBeforeReveal;
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(())
}

fn reveal_visible_card_flip(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let (which_card, bet_x, bet_y) = runtime_shell
        .visible_card_flip
        .as_ref()
        .map(|game| (game.which_card, game.bet_x, game.bet_y))
        .context("no Card Flip game is open")?;
    runtime_shell
        .shell
        .session_mut()
        .state_mut()
        .script_runtime
        .pending_card_flip_input = Some(CardFlipInput::Reveal {
        which_card: u8::try_from(which_card).context("Card Flip side fits a byte")?,
        cursor_x: u8::try_from(bet_x).context("Card Flip bet x fits a byte")?,
        cursor_y: u8::try_from(bet_y).context("Card Flip bet y fits a byte")?,
    });
    let result = runtime_shell
        .shell
        .apply_declared_special_routine("CardFlip")?;
    let SpecialRoutineEffect::CardFlipRevealed {
        card_name,
        card_level,
        payout,
        deck,
        revealed,
        coins,
        ..
    } = result.outcome.effect
    else {
        anyhow::bail!("CardFlip returned a different special effect");
    };
    let game = runtime_shell
        .visible_card_flip
        .as_mut()
        .context("Card Flip closed during play")?;
    game.coins = coins;
    game.payout = payout;
    game.deck = deck;
    game.revealed = revealed;
    game.face_card = Some((card_name.clone(), card_level));
    game.phase = VisibleCardFlipPhase::Result;
    game.animation = VisibleCardFlipAnimation::WaitResult { payout };
    game.yes_no_index = 0;
    game.message = if payout > 0 {
        "YEAH!".to_string()
    } else {
        "DARN…".to_string()
    };
    queue_visible_shell_sound_effect(
        runtime_shell,
        if payout > 0 {
            "SFX_2ND_PLACE"
        } else {
            "SFX_WRONG"
        },
    )?;
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(())
}

fn acknowledge_visible_card_flip_result(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let animation = runtime_shell
        .visible_card_flip
        .as_ref()
        .map(|game| game.animation)
        .context("no Card Flip game is open")?;
    if animation != VisibleCardFlipAnimation::AwaitResult {
        return Ok(());
    }
    runtime_shell
        .shell
        .session_mut()
        .state_mut()
        .script_runtime
        .pending_card_flip_input = Some(CardFlipInput::AcknowledgeResult);
    let result = runtime_shell
        .shell
        .apply_declared_special_routine("CardFlip")?;
    if !matches!(
        result.outcome.effect,
        SpecialRoutineEffect::CardFlipResultAcknowledged { .. }
    ) {
        anyhow::bail!("CardFlip result acknowledgement returned a different special effect");
    }
    let game = runtime_shell.visible_card_flip.as_mut().unwrap();
    game.phase = VisibleCardFlipPhase::PlayAgain;
    game.animation = VisibleCardFlipAnimation::None;
    game.yes_no_index = 0;
    game.message = "WANT TO PLAY\nAGAIN?".to_string();
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(())
}

fn start_visible_card_flip_round(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let input = match runtime_shell
        .shell
        .session()
        .state()
        .script_runtime
        .card_flip
        .as_ref()
        .map(|game| game.phase)
    {
        None | Some(CardFlipPhase::Quit) => CardFlipInput::Start,
        Some(CardFlipPhase::PlayAgain) => CardFlipInput::Continue,
        Some(CardFlipPhase::Shuffled) => CardFlipInput::ResumeAfterShuffle,
        Some(phase) => anyhow::bail!("CardFlip cannot start a round during phase {phase:?}"),
    };
    runtime_shell
        .shell
        .session_mut()
        .state_mut()
        .script_runtime
        .pending_card_flip_input = Some(input);
    let result = runtime_shell
        .shell
        .apply_declared_special_routine("CardFlip")?;
    let (deck, revealed, coins) = match result.outcome.effect {
        SpecialRoutineEffect::CardFlipStarted {
            deck,
            revealed,
            coins,
            ..
        } => (deck, revealed, coins),
        SpecialRoutineEffect::CardFlipShuffled {
            deck,
            revealed,
            coins,
            ..
        } => {
            let game = runtime_shell
                .visible_card_flip
                .as_mut()
                .context("Card Flip closed during shuffle")?;
            game.phase = VisibleCardFlipPhase::Shuffled;
            game.animation = VisibleCardFlipAnimation::None;
            game.round = 0;
            game.face_card = None;
            game.coins = coins;
            game.payout = 0;
            game.deck = deck;
            game.revealed = revealed;
            game.message = "THE CARDS HAVE\nBEEN SHUFFLED.".to_string();
            mark_runtime_snapshot_dirty(runtime_shell);
            return Ok(());
        }
        SpecialRoutineEffect::GameCornerGameUnavailable { .. } => {
            let game = runtime_shell
                .visible_card_flip
                .as_mut()
                .context("Card Flip closed during coin check")?;
            game.phase = VisibleCardFlipPhase::NotEnoughCoins;
            game.animation = VisibleCardFlipAnimation::None;
            game.message = "NOT ENOUGH COINS…".to_string();
            mark_runtime_snapshot_dirty(runtime_shell);
            return Ok(());
        }
        _ => anyhow::bail!("CardFlip start returned a different special effect"),
    };
    let round = runtime_shell
        .shell
        .session()
        .state()
        .script_runtime
        .card_flip
        .as_ref()
        .map_or(0, |state| usize::from(state.num_cards_played));
    let game = runtime_shell
        .visible_card_flip
        .as_mut()
        .context("Card Flip closed during start")?;
    game.phase = VisibleCardFlipPhase::ChooseCard;
    game.animation = VisibleCardFlipAnimation::WaitStake;
    game.yes_no_index = 0;
    game.payout = 0;
    game.face_card = None;
    game.coins = coins;
    game.round = round;
    game.deck = deck;
    game.revealed = revealed;
    game.message = "CHOOSE A CARD.".to_string();
    queue_visible_shell_sound_effect(runtime_shell, "SFX_TRANSACTION")?;
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(())
}

include!("battle_entry_progress.rs");
