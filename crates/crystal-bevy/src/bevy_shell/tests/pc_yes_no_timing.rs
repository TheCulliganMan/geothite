// Keep the source PC question visible until its post-answer VBlank wait ends.
#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn pc_source_yes_no_retains_choice_and_defers_side_effects_for_fifteen_frames() {
    for oak in [false, true] {
        for (choice, cancel, chord) in [
            (0, false, false),
            (1, false, false),
            (0, true, false),
            (0, true, true),
        ] {
            let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
            let potion = shell.shell.runtime().data().items["POTION"].clone();
            shell.shell.session_mut().state_mut().bag.pc_items.clear();
            shell
                .shell
                .session_mut()
                .state_mut()
                .bag
                .add_pc_item(&potion, 3)
                .unwrap();
            if oak {
                open_visible_oak_rating_question(&mut shell).unwrap();
            } else {
                shell.pc_item_action = Some(VisiblePlayerPcAction::TossItem);
                shell.pc_item_cursor = Some(MenuCursor {
                    surface_id: "pc:items".into(),
                    option_index: 0,
                });
                shell.pc_item_quantity = Some(VisiblePcItemQuantity {
                    action: VisiblePlayerPcAction::TossItem,
                    item_id: "POTION".into(),
                    stack_index: 0,
                    quantity: 1,
                    maximum: 3,
                });
                commit_visible_pc_item_quantity(&mut shell).unwrap();
            }
            let text = shell.pc_notice.clone().unwrap();
            shell.field_text_reveal = Some(VisibleFieldTextReveal {
                text: text.clone(),
                page_index: 0,
                visible_chars: text.chars().count(),
                frames_until_next_char: 0,
            });
            shell.yes_no_cursor.as_mut().unwrap().option_index = choice;
            shell.pc_menu_input_wait_frames = 0;
            let before = shell.shell.session().state().clone();
            let mut keys = ButtonInput::<KeyCode>::default();
            keys.press(if cancel { KeyCode::KeyX } else { KeyCode::KeyZ });
            if chord {
                keys.press(KeyCode::KeyZ);
                keys.press(KeyCode::ArrowDown);
            }
            apply_visible_runtime_controls(&keys, &mut shell, true);
            let accepted = !cancel && choice == 0;
            assert_eq!(shell.mailbox_confirmation_response, Some(accepted));
            assert_eq!(shell.pc_menu_input_wait_frames, 15);
            for frame in 0..14 {
                // Direct handlers must not bypass the wait, change the chosen
                // cursor, toss an item or enter Oak's rating/shutdown branch.
                for press in [
                    press_visible_a_button,
                    press_visible_b_button,
                    move_visible_primary_cursor_up,
                    move_visible_primary_cursor_down,
                    move_visible_primary_cursor_left,
                    move_visible_primary_cursor_right,
                    press_visible_start_button,
                    press_visible_select_button,
                ] {
                    press(&mut shell).unwrap();
                }
                assert_eq!(shell.yes_no_cursor.as_ref().unwrap().option_index, choice);
                assert_eq!(shell.pc_notice.as_deref(), Some(text.as_str()));
                assert!(shell.pc_confirmation.is_some());
                assert_eq!(
                    shell.shell.session().state(),
                    &before,
                    "frame {frame}: mutation before CloseWindow"
                );
                keys.reset_all();
                keys.press(KeyCode::ArrowDown);
                apply_visible_runtime_controls(&keys, &mut shell, true);
                assert!(shell.pc_confirmation.is_some());
            }
            assert_eq!(shell.pc_menu_input_wait_frames, 1);
            apply_visible_runtime_controls(&keys, &mut shell, true);
            assert!(shell.pc_confirmation.is_none());
            assert!(shell.mailbox_confirmation_response.is_none());
            assert!(shell.yes_no_cursor.is_none());
            assert_eq!(
                shell.shell.session().state().bag.pc_item_quantity(&potion),
                if !oak && accepted { 2 } else { 3 }
            );
            if oak {
                assert_eq!(
                    shell.special_boundary.as_ref().unwrap().label,
                    if accepted {
                        "ProfOaksPcBoot"
                    } else {
                        "OakPCShutdown"
                    }
                );
                assert_eq!(shell.pending_special_sound.is_some(), accepted);
            }
            assert_eq!(shell.last_error, None);
        }
    }
}
