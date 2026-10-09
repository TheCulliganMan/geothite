// Presentation-only projection of the existing authored timeline/object VM.
// No controller ticks, audio device, generic guessed projectile paths or exports.
fn render_text_battle_replay(
    asset_root: &AssetRoot,
    snapshot: &RuntimeShellSnapshot,
    mut animation: VisibleMoveAnimation,
) -> Result<VisibleBattleReplay> {
    let bundle: serde_json::Value = serde_json::from_str(&snapshot.presentation.battle_anim_bundle)?;
    let mut playback = new_visible_battle_objects(&bundle, &animation)?;
    let mut art = RenderedTilesetArt::default();
    let mut images = Assets::<Image>::default();
    let mut frames = Vec::new();
    // 20 visual frames/sec, <=4 seconds. Long source scripts are sampled, not
    // run by a UI timer. Completion already happened in the game controller.
    let stride = animation.total_frames.div_ceil(80).max(3);
    for tick in (0..animation.total_frames.min(600)).step_by(usize::from(stride)) {
        animation.frame = tick;
        advance_visible_battle_objects(&mut playback, &bundle, &animation)?;
        let (player, enemy) = visible_move_battler_offsets(Some(&animation));
        let screen = visible_move_screen_offset(Some(&animation));
        let scale = TILE_SIZE / SOURCE_TILE_SIZE as f32;
        let offset = |v: Vec3| [((v.x + screen.x) / scale) as i16, (-(v.y + screen.y) / scale) as i16];
        let mut frame = VisibleBattleReplayFrame {
            player_offset: offset(player), enemy_offset: offset(enemy), pieces: Vec::new(),
        };
        let mut palettes = visible_battle_dmg_palette_registers(Some(&animation));
        if let Some((tick, value)) = playback.obp0_write
            && palettes.obp0_write_frame.is_none_or(|frame| tick >= u32::from(frame))
        { palettes.obp0 = value; }
        for (slot, live) in playback.slots.iter().enumerate() {
            let Some(live) = live else { continue; };
            if live.oam.entries.is_empty() { continue; }
            let VisibleMoveObjectCommand::Spawn { object_id, .. } = &animation.object_events[live.event_index].command else { continue; };
            let object = &bundle["objects"][object_id];
            let source_frame = bundle["framesets"][live.frameset].as_array()
                .and_then(|frames| frames.get(live.frame)).context("replay source frameset")?;
            let palette = match live.bytes[5] & 7 {
                0 => "PAL_BATTLE_OB_GRAY", 1 => "PAL_BATTLE_OB_YELLOW", 2 => "PAL_BATTLE_OB_RED",
                3 => "PAL_BATTLE_OB_GREEN", 4 => "PAL_BATTLE_OB_BLUE", 5 => "PAL_BATTLE_OB_BROWN",
                _ => continue,
            };
            let rendered = battle_anim_rendered_frame(&mut art, &bundle, asset_root,
                object_id, object, live.frameset, live.frame, source_frame,
                !animation.player_move, false, false, Some(palette),
                palettes.obp0, palettes.obp1, Some(&live.oam), &mut images)?;
            let image = images.get(&rendered.sprite.handle).context("replay source image")?;
            frame.pieces.push(VisibleBattleReplayPiece {
                x: (live.oam.origin.0 - 8 + i32::from(rendered.offset_x)) as i16,
                y: (live.oam.origin.1 - 16 + i32::from(rendered.offset_y)
                    + visible_rollout_object_y_offset(&animation, slot)) as i16,
                width: image.texture_descriptor.size.width as u16,
                height: image.texture_descriptor.size.height as u16,
                rgba: image.data.clone(),
            });
        }
        frames.push(frame);
    }
    Ok(VisibleBattleReplay { animation_label: animation.animation_label, frames })
}
