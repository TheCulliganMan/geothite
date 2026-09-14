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
    }
    mark_runtime_snapshot_dirty(shell);
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
        (frame, palette)
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
        "battle-player:dude"
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
            normalize_pokemon_asset_id(&battle.enemy_pokemon.species.id)
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
