fn advance_visible_pc_input_vblanks(shell: &mut BevyRuntimeShell, frames: u32) {
    shell.pc_joypad.advance_vblanks(frames);
    shell.pc_menu_input_wait_frames = shell.pc_menu_input_wait_frames.saturating_sub(frames);
}

fn apply_visible_pc_scrolling_list_controls(shell: &mut BevyRuntimeShell, down: u8, a_consumed: bool) {
    use crate::core::input::{B_PAD_A, B_PAD_B, B_PAD_SELECT, B_PAD_START,
        B_PAD_RIGHT, B_PAD_LEFT, B_PAD_UP, B_PAD_DOWN};
    let mailbox = shell.mailbox_cursor.is_some();
    shell.ui_held_direction = None;
    shell.ui_direction_repeat_ticks = 0;
    // Neither DelayFrames nor WaitBGMap polls GetJoypad. In particular, a
    // short press entirely inside an upload must not become a pending action.
    if shell.pc_menu_input_wait_frames != 0 { return; }
    if a_consumed || shell.field_text_consumed_b {
        shell.pc_joypad.get_joypad(down);
        return;
    }
    shell.pc_joypad.sample(down, true);
    let pressed = shell.pc_joypad.pressed;
    let last = shell.pc_joypad.last;
    // MenuJoypadLoop.BGMap_OAM calls WaitBGMap before every subsequent poll,
    // including after suppressed JoyTextDelay input or a disabled direction.
    shell.pc_menu_input_wait_frames = 4;
    if pressed & B_PAD_A != 0 {
        run_bevy_action(shell, press_visible_a_button);
    } else if pressed & B_PAD_B != 0 {
        run_bevy_action(shell, press_visible_b_button);
    } else if pressed & B_PAD_SELECT != 0 {
        if !mailbox { run_bevy_action(shell, press_visible_select_button); }
    } else if pressed & B_PAD_START != 0 || last & (B_PAD_RIGHT | B_PAD_LEFT) != 0 {
        // ScrollingMenuJoyAction does not fall through disabled inputs.
    } else if let Some(direction) = [(B_PAD_UP, GameButton::Up), (B_PAD_DOWN, GameButton::Down)]
        .into_iter().find_map(|(bit, direction)| (last & bit != 0).then_some(direction)) {
        let before = if mailbox { &shell.mailbox_cursor } else { &shell.pc_item_cursor }
            .as_ref().map(|cursor| cursor.option_index);
        dispatch_visible_ui_direction(shell, direction);
        let after = if mailbox { &shell.mailbox_cursor } else { &shell.pc_item_cursor }
            .as_ref().map(|cursor| cursor.option_index);
        if before != after {
            // _ScrollingMenu.zero adds DelayFrames(3) after InitDisplay.
            // CPU time spent in InitDisplay is not represented by these
            // explicit VBlank waits; do not replace them with a measured
            // inventory-specific nine-frame repeat constant.
            shell.pc_menu_input_wait_frames += 3;
        }
    }
}

fn apply_visible_pc_quantity_controls(shell: &mut BevyRuntimeShell, down: u8, a_consumed: bool) {
    use crate::core::input::{B_PAD_A, B_PAD_B, B_PAD_DOWN, B_PAD_UP, B_PAD_LEFT, B_PAD_RIGHT};
    // PrintLetterDelay updates GetJoypad mirrors while printing the question.
    // It does not reset JoyTextDelay's counter. Retain that physical history
    // when PrintText returns, so a held text button is not a new confirmation.
    if !visible_pc_item_quantity_input_ready(shell)
        || a_consumed || shell.field_text_consumed_b
    {
        shell.pc_joypad.get_joypad(down);
        return;
    }
    // JoyTextDelay_ForcehJoyDown temporarily sets hInMenu=1, then combines
    // hJoyPressed buttons with hJoyLast directions. The counter belongs to
    // the joypad, not to a selected direction or a rendered menu row.
    shell.pc_joypad.sample(down, true);
    let pressed = shell.pc_joypad.pressed;
    let last = shell.pc_joypad.last;
    shell.ui_held_direction = None;
    shell.ui_direction_repeat_ticks = 0;
    if pressed & B_PAD_B != 0 {
        run_bevy_action(shell, press_visible_b_button);
    } else if pressed & B_PAD_A != 0 {
        run_bevy_action(shell, press_visible_a_button);
    } else if let Some(direction) = [
        (B_PAD_DOWN, GameButton::Down), (B_PAD_UP, GameButton::Up),
        (B_PAD_LEFT, GameButton::Left), (B_PAD_RIGHT, GameButton::Right),
    ].into_iter().find_map(|(bit, direction)| (last & bit != 0).then_some(direction)) {
        dispatch_visible_ui_direction(shell, direction);
    }
}

fn sample_visible_pc_text_joypad_history(keys: &ButtonInput<KeyCode>, shell: &mut BevyRuntimeShell) {
    let mailbox_reader = shell.mailbox_cursor.is_some() && shell.pending_mail_read.is_some();
    let pc_text = shell.pc_notice.is_some() && shell.pc_item_quantity.is_none()
        && shell.pc_confirmation.is_none();
    if mailbox_reader || pc_text {
        // ReadAnyMail.loop, PrintLetterDelay and PromptButton call GetJoypad.
        // Returning to a scrolling menu must retain their button mirrors;
        // JoyTextDelay must not turn a held dismissal B into a second B.
        shell.pc_joypad.get_joypad(visible_menu_physical_down(keys));
    }
}
