// MailboxPC.TopMenuData: four rows, normal scrolling items plus CANCEL.
fn visible_mailbox_window(snapshot: &RuntimeShellSnapshot, shell: &BevyRuntimeShell) -> Result<(usize, usize)> {
    let total = snapshot.mailbox.len() + 1;
    let selected = strict_readonly_cursor_index(&shell.mailbox_cursor, "pc:mailbox", total)
        .context("mailbox requires a message or CANCEL cursor")?;
    let scroll = shell.mailbox_scroll;
    anyhow::ensure!(scroll <= total.saturating_sub(4) && selected >= scroll && selected < scroll + 4,
        "mailbox cursor {selected} is outside its four-row window at {scroll}");
    Ok((selected, scroll))
}

fn move_visible_mailbox_cursor(shell: &mut BevyRuntimeShell, delta: isize) -> Result<()> {
    if shell.mailbox_action_cursor.is_some() {
        // SubMenuData has STATICMENU_CURSOR, without STATICMENU_WRAP.
        let selected = strict_readonly_cursor_index(&shell.mailbox_action_cursor,
            "pc:mailbox-actions", VISIBLE_MAILBOX_ACTIONS.len())
            .context("mailbox submenu requires a valid cursor")?;
        shell.mailbox_action_cursor.as_mut().unwrap().option_index =
            selected.saturating_add_signed(delta).min(VISIBLE_MAILBOX_ACTIONS.len() - 1);
    } else {
        let snapshot = shell.shell.snapshot()?;
        let (selected, _) = visible_mailbox_window(&snapshot, shell)?;
        let next = selected.saturating_add_signed(delta).min(snapshot.mailbox.len());
        if next < shell.mailbox_scroll { shell.mailbox_scroll = next; }
        else if next >= shell.mailbox_scroll + 4 { shell.mailbox_scroll = next - 3; }
        shell.mailbox_cursor.as_mut().unwrap().option_index = next;
    }
    mark_runtime_presentation_dirty(shell);
    Ok(())
}

fn restore_visible_mailbox_position(shell: &mut BevyRuntimeShell, selected: usize) -> Result<()> {
    let total = shell.shell.snapshot()?.mailbox.len() + 1;
    let mut row = selected.checked_sub(shell.mailbox_scroll)
        .context("mailbox cursor precedes its retained window")?;
    anyhow::ensure!(row < 4, "mailbox row is outside its source window");
    // InitScrollingMenuCursor preserves the screen row while clamping scroll.
    // Even after deleting the last letter, MailboxPC.loop still shows CANCEL.
    shell.mailbox_scroll = shell.mailbox_scroll.min(total.saturating_sub(4));
    if shell.mailbox_scroll + row >= total { shell.mailbox_scroll = 0; row = 0; }
    shell.mailbox_cursor = Some(MenuCursor {
        surface_id: "pc:mailbox".into(), option_index: shell.mailbox_scroll + row,
    });
    mark_runtime_presentation_dirty(shell);
    Ok(())
}

fn close_visible_mailbox(shell: &mut BevyRuntimeShell) {
    shell.mailbox_cursor = None;
    shell.mailbox_action_cursor = None;
    shell.mailbox_scroll = 0;
    shell.player_pc_action_cursor = Some(MenuCursor { surface_id: "pc:player-actions".into(), option_index: 3 });
    mark_runtime_presentation_dirty(shell);
}

fn visible_mailbox_text(snapshot: &RuntimeShellSnapshot, label: &str) -> Result<String> {
    let text = snapshot.presentation.asm_text.get(label)
        .with_context(|| format!("mailbox source text {label} is missing"))?;
    Ok(normalize_visible_script_text_with_context(text,
        &snapshot.trainer.player_name, visible_rival_name(snapshot),
        snapshot.progression.time.day_of_week))
}

fn spawn_visible_mailbox_screen(
    commands: &mut Commands,
    snapshot: &RuntimeShellSnapshot,
    shell: &BevyRuntimeShell,
    art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<()> {
    let (selected, scroll) = visible_mailbox_window(snapshot, shell)?;
    let palette = source_map_text_palette(snapshot, asset_root)?;
    let frame_id = textbox_frame_id(snapshot.trainer.options.frame);
    let key = (frame_id, palette);
    if !art.pc_window_frame_cache.contains_key(&key) {
        let frame = load_window_frame_art_with_palette(asset_root, frame_id, &palette, images)?;
        art.pc_window_frame_cache.insert(key, frame);
    }
    // ScrollingMenu_InitFlags expands TopMenuHeader by one tile at each edge.
    spawn_pc_item_palette_window(commands, &art.pc_window_frame_cache[&key],
        palette[0], 7.0, 0.0, 13, 12, 4.0);
    for index in scroll..(scroll + 4).min(snapshot.mailbox.len() + 1) {
        let label = snapshot.mailbox.get(index).map(|entry| entry.mail.author.as_str()).unwrap_or("CANCEL");
        let row = 2.0 + (index - scroll) as f32 * 2.0;
        let (x, y) = battle_hud_tile_origin(9.0, row);
        spawn_scene_dialog_bitmap_text(commands, art, asset_root, images, label, x, y, 4.2);
        if index == selected {
            let glyph = if shell.mailbox_action_cursor.is_some() || shell.pc_notice.is_some() { "▷" } else { "▶" };
            let (x, y) = battle_hud_tile_origin(8.0, row);
            spawn_scene_dialog_bitmap_text(commands, art, asset_root, images, glyph, x, y, 4.2);
        }
    }
    for (visible, row, glyph) in [(scroll > 0, 1.0, "▲"),
        (scroll + 4 < snapshot.mailbox.len() + 1, 10.0, "▼")] {
        if visible {
            let (x, y) = battle_hud_tile_origin(18.0, row);
            spawn_scene_dialog_bitmap_text(commands, art, asset_root, images, glyph, x, y, 4.2);
        }
    }
    if shell.mailbox_action_cursor.is_some() {
        anyhow::ensure!(selected < snapshot.mailbox.len(), "mailbox CANCEL has no message submenu");
        let action = strict_readonly_cursor_index(&shell.mailbox_action_cursor,
            "pc:mailbox-actions", VISIBLE_MAILBOX_ACTIONS.len()).context("invalid mailbox action cursor")?;
        spawn_pc_item_palette_window(commands, &art.pc_window_frame_cache[&key],
            palette[0], 0.0, 0.0, 14, 10, 4.5);
        for (index, label) in VISIBLE_MAILBOX_ACTIONS.iter().enumerate() {
            let (x, y) = battle_hud_tile_origin(1.0, 2.0 + index as f32 * 2.0);
            spawn_scene_dialog_bitmap_text(commands, art, asset_root, images,
                &format!("{}{label}", if action == index { "▶" } else { " " }), x, y, 4.8);
        }
    }
    spawn_pc_item_notice(commands, snapshot, shell, art, asset_root, images, palette)?;
    if matches!(shell.pc_confirmation, Some(VisiblePcConfirmation::PutMailInPack(_)))
        && shell.pc_notice.as_deref().is_some_and(|text| visible_field_text_reveal_is_complete_for_text(shell, text))
    {
        let selected = strict_readonly_cursor_index(&shell.yes_no_cursor, "pc:confirmation", 2)
            .context("mailbox confirmation requires a valid cursor")?;
        spawn_pc_item_palette_window(commands, &art.pc_window_frame_cache[&key],
            palette[0], 14.0, 7.0, 6, 5, 4.5);
        for (index, label) in ["YES", "NO"].iter().enumerate() {
            let (x, y) = battle_hud_tile_origin(15.0, 8.0 + index as f32 * 2.0);
            spawn_scene_dialog_bitmap_text(commands, art, asset_root, images,
                &format!("{}{label}", if selected == index { "▶" } else { " " }), x, y, 4.8);
        }
    }
    Ok(())
}

fn apply_visible_mailbox_action_controls(shell: &mut BevyRuntimeShell, down: u8, a_consumed: bool) {
    use crate::core::input::{B_PAD_A, B_PAD_B, B_PAD_SELECT, B_PAD_START,
        B_PAD_RIGHT, B_PAD_LEFT, B_PAD_UP, B_PAD_DOWN};
    shell.ui_held_direction = None;
    shell.ui_direction_repeat_ticks = 0;
    if shell.pc_menu_input_wait_frames != 0 { return; }
    if a_consumed || shell.field_text_consumed_b {
        shell.pc_joypad.get_joypad(down);
        return;
    }
    // _ScrollingMenu.exit clears hInMenu before MailboxPC.Submenu calls
    // VerticalMenu. Its JoyTextDelay therefore exposes only fresh directions.
    shell.pc_joypad.sample(down, false);
    let pressed = shell.pc_joypad.pressed;
    let last = shell.pc_joypad.last;
    let buttons = pressed & (B_PAD_A | B_PAD_B | B_PAD_SELECT | B_PAD_START);
    if buttons == 0 && last & (B_PAD_RIGHT | B_PAD_LEFT | B_PAD_UP | B_PAD_DOWN) == 0 {
        // StaticMenu's Do2DMenuRTCJoypad stays in its polling loop until input.
        return;
    }
    shell.pc_menu_input_wait_frames = 4;
    if pressed & B_PAD_B != 0 {
        run_bevy_action(shell, press_visible_b_button);
    } else if pressed & B_PAD_A != 0 {
        run_bevy_action(shell, press_visible_a_button);
    } else if buttons != 0 || last & (B_PAD_RIGHT | B_PAD_LEFT) != 0 {
        // Disabled buttons and horizontal inputs suppress lower priorities.
    } else if last & B_PAD_UP != 0 {
        dispatch_visible_ui_direction(shell, GameButton::Up);
    } else if last & B_PAD_DOWN != 0 {
        dispatch_visible_ui_direction(shell, GameButton::Down);
    }
}

fn move_visible_mailbox_confirmation_cursor(shell: &mut BevyRuntimeShell, delta: isize) -> Result<()> {
    if shell.mailbox_confirmation_response.is_some() { return Ok(()); }
    let selected = strict_readonly_cursor_index(&shell.yes_no_cursor, "pc:confirmation", 2)
        .context("mailbox confirmation requires a valid cursor")?;
    shell.yes_no_cursor.as_mut().unwrap().option_index = selected.saturating_add_signed(delta).min(1);
    mark_runtime_presentation_dirty(shell);
    Ok(())
}

fn apply_visible_mailbox_confirmation_controls(shell: &mut BevyRuntimeShell, down: u8, a_consumed: bool) {
    use crate::core::input::{B_PAD_A, B_PAD_B, B_PAD_SELECT, B_PAD_START,
        B_PAD_RIGHT, B_PAD_LEFT, B_PAD_UP, B_PAD_DOWN};
    shell.ui_held_direction = None;
    shell.ui_direction_repeat_ticks = 0;
    if let Some(accepted) = shell.mailbox_confirmation_response {
        // InterpretTwoOptionMenu's DelayFrames(15) does not poll GetJoypad.
        if shell.pc_menu_input_wait_frames == 0 {
            run_bevy_action(shell, |shell| resolve_visible_pc_confirmation(shell, accepted));
            shell.pc_menu_input_wait_frames = 4;
        }
        return;
    }
    if a_consumed || shell.field_text_consumed_b || !shell.pc_notice.as_deref()
        .is_some_and(|text| visible_field_text_reveal_is_complete_for_text(shell, text))
    {
        // PrintLetterDelay retains physical button history. Entering the
        // YesNoBox must not reinterpret held printer A/B as a fresh choice.
        shell.pc_joypad.get_joypad(down);
        shell.pc_menu_input_wait_frames = 4;
        return;
    }
    if shell.pc_menu_input_wait_frames != 0 { return; }
    shell.pc_joypad.sample(down, false);
    let pressed = shell.pc_joypad.pressed;
    let last = shell.pc_joypad.last;
    let buttons = pressed & (B_PAD_A | B_PAD_B | B_PAD_SELECT | B_PAD_START);
    if buttons == 0 && last & (B_PAD_RIGHT | B_PAD_LEFT | B_PAD_UP | B_PAD_DOWN) == 0 { return; }
    shell.pc_menu_input_wait_frames = 4;
    if pressed & (B_PAD_A | B_PAD_B) != 0 {
        run_bevy_action(shell, |shell| {
            let selected = strict_readonly_cursor_index(&shell.yes_no_cursor, "pc:confirmation", 2)
                .context("mailbox confirmation requires a valid cursor")?;
            queue_visible_shell_sound_effect(shell, "SFX_READ_TEXT_2")?;
            shell.mailbox_confirmation_response = Some(pressed & B_PAD_B == 0 && selected == 0);
            shell.pc_menu_input_wait_frames = 15;
            mark_runtime_presentation_dirty(shell);
            Ok(())
        });
    } else if buttons != 0 || last & (B_PAD_RIGHT | B_PAD_LEFT) != 0 {
        // Disabled higher-priority inputs do not fall through to directions.
    } else if last & B_PAD_UP != 0 {
        run_bevy_action(shell, |shell| move_visible_mailbox_confirmation_cursor(shell, -1));
    } else if last & B_PAD_DOWN != 0 {
        run_bevy_action(shell, |shell| move_visible_mailbox_confirmation_cursor(shell, 1));
    }
}
