// engine/battle/sliding_intro.asm: 73 DelayFrame calls, with 72 two-pixel
// OAM steps. BG SCX starts at $90 and reaches zero on the last frame.
const BATTLE_SLIDING_INTRO_FRAMES: u8 = 73;

fn advance_visible_battle_sliding_intro(shell: &mut BevyRuntimeShell) {
    let Some(frame) = shell.visible_battle_sliding_intro.as_mut() else {
        return;
    };
    *frame += 1;
    if *frame >= BATTLE_SLIDING_INTRO_FRAMES {
        shell.visible_battle_sliding_intro = None;
        // BattleStartMessage animates wild frontpics before printing the
        // encounter text. Trainer frontpics still start after their send-out.
        if let Err(error) = begin_visible_wild_entrance_animation(shell) {
            record_visible_runtime_system_error(shell, error);
        }
    }
    mark_runtime_snapshot_dirty(shell);
}

fn begin_visible_wild_entrance_animation(shell: &mut BevyRuntimeShell) -> Result<()> {
    let snapshot = shell.shell.snapshot()?;
    let Some(battle) = snapshot.battle.as_ref() else { return Ok(()); };
    if matches!(battle.kind, RuntimeBattleKind::Wild { .. } | RuntimeBattleKind::StaticWild { .. }) {
        // BattleStartMessage requests ANIM_SEND_OUT_MON with parameter 1:
        // only the shiny sequence, before the frontpic and encounter text.
        if visible_pokemon_is_shiny(&battle.enemy_pokemon) {
            shell.visible_send_out_animation = Some(VisibleSendOutAnimation {
                side: crate::core::battle::turn::BattleSide::Enemy,
                frame: VisibleSendOutAnimation::NORMAL_FRAMES,
                shiny: true,
            });
            queue_visible_shell_sound_effect(shell, "SFX_SHINE")?;
        } else if snapshot.trainer.options.battle_scene == BattleScene::On {
            start_visible_enemy_frontpic_animation(shell, 0)?;
        }
    }
    Ok(())
}

fn visible_wild_entrance_animation_active(shell: &BevyRuntimeShell) -> bool {
    (shell.visible_frontpic_animation.is_some()
        || shell.visible_send_out_animation.as_ref().is_some_and(|animation|
            animation.side == crate::core::battle::turn::BattleSide::Enemy))
        && shell.battle_message_scene.as_ref().and_then(|scene| scene.battle.as_ref())
            .is_some_and(|battle| {
                matches!(battle.kind, RuntimeBattleKind::Wild { .. } | RuntimeBattleKind::StaticWild { .. })
                    && shell.battle_entry_messages_remaining
                        == if battle.battle_type == "BATTLETYPE_TUTORIAL" { 1 } else { 2 }
            })
}

fn battle_sliding_intro_offsets(frame: u8) -> (f32, f32) {
    let frame = frame.min(BATTLE_SLIDING_INTRO_FRAMES - 1);
    // OAM moves before each DelayFrame except the final iteration, whereas
    // the opponent BG uses D before decrementing it for the next iteration.
    let scale = TILE_SIZE / SOURCE_TILE_SIZE as f32;
    let enemy = -(144 - i16::from(frame) * 2) as f32 * scale;
    let player = (144 - i16::from((frame + 1).min(72)) * 2) as f32 * scale;
    (player, enemy)
}

fn battle_sliding_intro_art(
    art: &mut RenderedTilesetArt,
    root: &AssetRoot,
    images: &mut Assets<Image>,
    asset_id: &str,
) -> Result<SpriteFrame> {
    let key = IntroArtKey {
        asset_id: format!("battle-sliding:{asset_id}"),
    };
    if let Some(frame) = art.intro_cache.get(&key) {
        return Ok(frame.clone());
    }
    let (source, palette) = if let Some(species) = asset_id.strip_prefix("pokemon:") {
        let palette = load_pokemon_palette(root, species, PokemonSpriteSide::Front, false)?;
        let frame = pokemon_animation_frame_for_art(
            art,
            root,
            species,
            PokemonSpriteSide::Front,
            false,
            0,
            images,
        )
        .context("battle sliding intro requires opponent frontpic")?;
        (battle_padded_frontpic(art, images, &frame)?, palette)
    } else {
        let source_key = IntroArtKey {
            asset_id: asset_id.to_string(),
        };
        if !art.intro_cache.contains_key(&source_key) {
            let frame = load_oak_intro_frame(root, asset_id, images)?;
            art.intro_cache.insert(source_key.clone(), frame);
        }
        let frame = art.intro_cache[&source_key].clone();
        let (_, palette_path, player_palette) = oak_intro_asset_paths(root, asset_id)?;
        let palette = if player_palette {
            load_oak_intro_player_palette(&palette_path)?
        } else {
            load_gbcpal_palette(&palette_path)?
        };
        (frame, palette)
    };
    let blackout = load_named_predef_palette(root, "PREDEFPAL_BLACKOUT")?;
    let mut image = images
        .get(&source.handle)
        .context("battle sliding intro source image is unavailable")?
        .clone();
    for pixel in image.data.chunks_exact_mut(4) {
        if pixel[3] == 0 {
            continue;
        }
        let index = palette
            .iter()
            .position(|colour| pixel[..3] == colour[..])
            .context("battle sliding intro pixel is absent from its source palette")?;
        pixel[..3].copy_from_slice(&blackout[index]);
    }
    let frame = SpriteFrame {
        handle: images.add(image),
        size: source.size,
    };
    art.intro_cache.insert(key, frame.clone());
    Ok(frame)
}

fn spawn_battle_sliding_intro_pic(commands: &mut Commands, frame: &SpriteFrame, x: f32, y: f32) {
    // Clip against the native LCD: an offscreen picture must not leak into
    // the desktop margins while the two halves scroll in opposite directions.
    let scale = TILE_SIZE / SOURCE_TILE_SIZE as f32;
    let left = x.max(0.0);
    let right = (x + frame.size.x).min(160.0);
    if right <= left {
        return;
    }
    commands.spawn((
        SpriteBundle {
            texture: frame.handle.clone(),
            sprite: Sprite {
                rect: Some(Rect::new(left - x, 0.0, right - x, frame.size.y)),
                custom_size: Some(Vec2::new((right - left) * scale, frame.size.y * scale)),
                ..default()
            },
            transform: Transform::from_xyz(
                PLAYFIELD_LEFT + (left + right) * 0.5 * scale,
                PLAYFIELD_TOP - (y + frame.size.y * 0.5) * scale,
                3.0,
            ),
            ..default()
        },
        BattleBattlerMarker,
    ));
}

fn spawn_visible_battle_sliding_intro(
    commands: &mut Commands,
    snapshot: &RuntimeShellSnapshot,
    battle: &crate::RuntimeBattleSnapshot,
    frame: u8,
    art: &mut RenderedTilesetArt,
    root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<()> {
    let player_id = if battle.battle_type == "BATTLETYPE_TUTORIAL" {
        tutorial_backpic_id(snapshot)
    } else if snapshot.trainer.player_gender == PLAYER_GENDER_FEMALE {
        "battle-player:kris_back"
    } else {
        "battle-player:chris_back"
    };
    let enemy_id = match &battle.kind {
        RuntimeBattleKind::Trainer { trainer_class, .. } => format!(
            "battle-trainer:{}",
            normalize_battle_trainer_sprite_id(trainer_class)
        ),
        RuntimeBattleKind::Wild { .. } | RuntimeBattleKind::StaticWild { .. } => format!(
            "pokemon:{}",
            pokemon_asset_id_for_dvs(&battle.enemy_pokemon.species.id, battle.enemy_pokemon.dvs)
        ),
    };
    let player = battle_sliding_intro_art(art, root, images, player_id)?;
    let enemy = battle_sliding_intro_art(art, root, images, &enemy_id)?;
    let (player_offset, enemy_offset) = battle_sliding_intro_offsets(frame);
    let scale = TILE_SIZE / SOURCE_TILE_SIZE as f32;
    spawn_battle_sliding_intro_pic(commands, &enemy, 96.0 + enemy_offset / scale, 0.0);
    // CopyBackpic's OAM starts at Y=64 (visible Y=48 after the OAM bias).
    spawn_battle_sliding_intro_pic(commands, &player, 16.0 + player_offset / scale, 48.0);
    require_bitmap_font_art(art, root, images)?;
    anyhow::ensure!(
        battle_window_frame_art(art, root, images).is_some(),
        "battle sliding intro requires textbox frame art"
    );
    spawn_battle_window(
        commands,
        art,
        root,
        images,
        BATTLE_TEXT_BOX_LEFT_TILE,
        BATTLE_TEXT_BOX_TOP_TILE,
        BATTLE_TEXT_BOX_WIDTH_TILES,
        BATTLE_TEXT_BOX_HEIGHT_TILES,
        3.5,
    );
    Ok(())
}
