fn visible_pokegear_card_samples_joypad(shell: &BevyRuntimeShell) -> bool {
    let phone_prompt = shell.pokegear_phone_call.as_ref().is_some_and(|call| matches!(
        call.phase, VisiblePokegearPhoneCallPhase::AwaitHangup | VisiblePokegearPhoneCallPhase::NoServicePrompt));
    (shell.pokegear_menu_open || phone_prompt) && !shell.pokegear_standalone_map
        && (shell.pokegear_phone_call.is_none() || phone_prompt)
}

fn visible_menu_physical_down(keys: &ButtonInput<KeyCode>) -> u8 {
    let modified = [KeyCode::ShiftLeft, KeyCode::AltLeft, KeyCode::AltRight,
        KeyCode::ControlLeft, KeyCode::ControlRight].into_iter().any(|key| keys.pressed(key));
    if modified { 0 } else {
        [(KeyCode::KeyZ, GameButton::A), (KeyCode::KeyX, GameButton::B),
            (KeyCode::ShiftRight, GameButton::Select), (KeyCode::Enter, GameButton::Start),
            (KeyCode::ArrowRight, GameButton::Right), (KeyCode::ArrowLeft, GameButton::Left),
            (KeyCode::ArrowUp, GameButton::Up), (KeyCode::ArrowDown, GameButton::Down)]
            .into_iter().filter(|(key, _)| keys.pressed(*key))
            .fold(0, |mask, (_, button)| mask | button.pad_bit())
    }
}

fn sample_visible_pokegear_joypad(keys: &ButtonInput<KeyCode>, shell: &mut BevyRuntimeShell) {
    let down = visible_menu_physical_down(keys);
    shell.pokegear_opening_buttons &= down;
    shell.pokegear_joypad.sample(down & !shell.pokegear_opening_buttons, shell.pokegear_map_radio_delay.is_none());
}

fn apply_visible_pokegear_card_controls(
    keys: &ButtonInput<KeyCode>, shell: &mut BevyRuntimeShell, source_frame: bool,
) -> bool {
    use crate::core::input::{B_PAD_A, B_PAD_B, B_PAD_DOWN, B_PAD_LEFT,
        B_PAD_RIGHT, B_PAD_SELECT, B_PAD_START, B_PAD_UP};
    if !visible_pokegear_card_samples_joypad(shell) { return false; }
    if !source_frame { return true; }
    if !shell.pokegear_joypad_prepared {
        // Direct control callers represent one source frame. The live frame
        // updater already advances VBlank and samples before PlayRadioShow.
        shell.pokegear_joypad.advance_vblanks(1);
        sample_visible_pokegear_joypad(keys, shell);
    }
    let last = shell.pokegear_joypad.last;
    let pressed = shell.pokegear_joypad.pressed;
    if shell.pokegear_phone_call.is_some() {
        // FinishPhoneCall reads hJoyPressed after the outer JoyTextDelay.
        // Preserve its mirror through HangUp so held B is not a new B on
        // the restored contact list. No-service prompt dismissal does likewise.
        if pressed & B_PAD_B != 0 {
            run_bevy_action(shell, press_visible_b_button);
        } else if pressed & B_PAD_A != 0 {
            run_bevy_action(shell, press_visible_a_button);
        }
        return true;
    }
    if shell.pokegear_phone_menu.as_ref().is_some_and(|menu| menu.delete_confirmation.is_some()) {
        // YesNoBox -> VerticalMenu uses fresh buttons plus JoyTextDelay's
        // held directions. _2DMenuInterpretJoypad resolves buttons before
        // moving the cursor; VerticalMenu then treats any B bit as cancel.
        shell.ui_held_direction = None;
        shell.ui_direction_repeat_ticks = 0;
        if pressed & B_PAD_B != 0 {
            run_bevy_action(shell, press_visible_b_button);
        } else if pressed & B_PAD_A != 0 {
            run_bevy_action(shell, press_visible_a_button);
        } else if pressed & (B_PAD_SELECT | B_PAD_START) != 0
            || last & (B_PAD_RIGHT | B_PAD_LEFT) != 0 {
            // Disabled higher-priority inputs do not fall through to Up/Down.
        } else if last & B_PAD_UP != 0 {
            dispatch_visible_ui_direction(shell, GameButton::Up);
        } else if last & B_PAD_DOWN != 0 {
            dispatch_visible_ui_direction(shell, GameButton::Down);
        }
        return true;
    }
    if shell.pokegear_phone_menu.is_some() {
        // PokegearPhoneContactSubmenu tests hJoyPressed Up, Down, then A/B;
        // its dispatch gives B priority over A. Holding directions does not
        // invoke the contact submenu's cursor actions repeatedly.
        if pressed & B_PAD_UP != 0 {
            dispatch_visible_ui_direction(shell, GameButton::Up);
        } else if pressed & B_PAD_DOWN != 0 {
            dispatch_visible_ui_direction(shell, GameButton::Down);
        } else if pressed & B_PAD_B != 0 {
            run_bevy_action(shell, press_visible_b_button);
        } else if pressed & B_PAD_A != 0 {
            run_bevy_action(shell, press_visible_a_button);
        }
        return true;
    }
    let direction = match shell.pokegear_page {
        PokegearPage::Clock => {
            if last & (B_PAD_A | B_PAD_B | B_PAD_SELECT | B_PAD_START) != 0 {
                run_bevy_action(shell, request_visible_pokegear_exit);
                return true;
            }
            (last & B_PAD_RIGHT != 0).then_some(GameButton::Right)
        }
        PokegearPage::Map => {
            if last & B_PAD_B != 0 {
                run_bevy_action(shell, press_visible_b_button);
                return true;
            }
            [(B_PAD_RIGHT, GameButton::Right), (B_PAD_LEFT, GameButton::Left),
                (B_PAD_UP, GameButton::Up), (B_PAD_DOWN, GameButton::Down)]
                .into_iter().find_map(|(bit, direction)| (last & bit != 0).then_some(direction))
        }
        PokegearPage::Phone => {
            if pressed & B_PAD_B != 0 {
                run_bevy_action(shell, press_visible_b_button);
                return true;
            }
            if pressed & B_PAD_A != 0 {
                run_bevy_action(shell, press_visible_a_button);
                return true;
            }
            [(B_PAD_LEFT, GameButton::Left), (B_PAD_RIGHT, GameButton::Right),
                (B_PAD_UP, GameButton::Up), (B_PAD_DOWN, GameButton::Down)]
                .into_iter().find_map(|(bit, direction)| (last & bit != 0).then_some(direction))
        }
        PokegearPage::Radio => {
            if shell.pokegear_map_radio_delay.is_some() {
                if pressed & (B_PAD_A | B_PAD_B) != 0 {
                    run_bevy_action(shell, press_visible_b_button);
                }
                return true;
            }
            if last & B_PAD_B != 0 {
                run_bevy_action(shell, |shell| {
                    press_visible_b_button(shell)?;
                    // The exit bit does not delete the tuning sprite. Its
                    // Down-before-Up update still runs on this final frame.
                    if last & B_PAD_DOWN != 0 { move_visible_pokegear_cursor(shell, 1)?; }
                    else if last & B_PAD_UP != 0 { move_visible_pokegear_cursor(shell, -1)?; }
                    Ok(())
                });
                return true;
            }
            // Jumptable Left precedes the tuning sprite's Down-before-Up.
            [(B_PAD_LEFT, GameButton::Left), (B_PAD_DOWN, GameButton::Down),
                (B_PAD_UP, GameButton::Up)]
                .into_iter().find_map(|(bit, direction)| (last & bit != 0).then_some(direction))
        }
    };
    shell.ui_held_direction = None;
    shell.ui_direction_repeat_ticks = 0;
    if let Some(direction) = direction { dispatch_visible_ui_direction(shell, direction); }
    true
}
