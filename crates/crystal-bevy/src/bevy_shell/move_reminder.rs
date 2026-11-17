// Move Reminder uses the ordinary script party selector and tutor confirmation
// flow. Only the forgotten-move list is new; the core commits learning and its
// fee atomically after the player has made every choice.
fn move_reminder_cursor_surface_id(party_index: usize) -> String {
    format!("party:relearn:{party_index}")
}

fn resolve_visible_move_reminder(
    runtime_shell: &mut BevyRuntimeShell,
    selected_party: Option<usize>,
    moves: Vec<String>,
    confirmed_party: Option<usize>,
) -> Result<()> {
    if let Some(party_index) = selected_party {
        let selected = strict_readonly_cursor_index(
            &runtime_shell.party_move_cursor,
            &move_reminder_cursor_surface_id(party_index),
            moves.len() + 1,
        )
        .context("Move Reminder has no valid move-or-CANCEL cursor")?;
        runtime_shell.party_move_cursor = None;
        runtime_shell.pc_notice = None;
        if confirmed_party.is_none() || selected == moves.len() {
            runtime_shell.pending_script_party_selection =
                Some(PendingScriptPartySelection::MoveReminder {
                    party_index: None,
                    moves: Vec::new(),
                });
            set_shell_action_status(runtime_shell, "WHICH POKéMON?");
            mark_runtime_snapshot_dirty(runtime_shell);
            return Ok(());
        }
        let move_id = moves
            .get(selected)
            .context("Move Reminder selection is outside its legal move list")?
            .clone();
        runtime_shell.pending_script_party_selection =
            Some(PendingScriptPartySelection::MoveTutor {
                move_id,
                party_index: None,
                reminder: true,
            });
        return resolve_visible_script_party_selection(runtime_shell, Some(party_index));
    }
    let Some(party_index) = confirmed_party else {
        set_visible_script_numeric_value(runtime_shell, u8::MAX);
        close_visible_party_menu(runtime_shell);
        mark_runtime_snapshot_dirty(runtime_shell);
        return continue_visible_script_after_prompt(runtime_shell);
    };
    let snapshot = runtime_shell.shell.snapshot()?;
    let pokemon = &snapshot
        .party
        .slots
        .iter()
        .find(|slot| slot.index == party_index)
        .context("Move Reminder party slot is empty")?
        .pokemon;
    let moves = if pokemon.is_egg || pokemon.species.id == "EGG" {
        Vec::new()
    } else {
        runtime_shell.shell.relearnable_party_moves(party_index)?
    };
    if moves.is_empty() {
        runtime_shell.pending_script_party_selection =
            Some(PendingScriptPartySelection::MoveReminder {
                party_index: None,
                moves,
            });
        runtime_shell.special_boundary = Some(SpecialBoundaryDisplay {
            label: "MoveReminderNoMoves".to_string(),
            details: vec!["This POKéMON has no\nforgotten moves it\ncan relearn yet.".to_string()],
        });
    } else {
        runtime_shell.party_move_cursor = Some(MenuCursor {
            surface_id: move_reminder_cursor_surface_id(party_index),
            option_index: 0,
        });
        runtime_shell.pending_script_party_selection =
            Some(PendingScriptPartySelection::MoveReminder {
                party_index: Some(party_index),
                moves,
            });
        set_shell_action_status(runtime_shell, "RELEARN WHICH MOVE?");
    }
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(())
}

fn visible_script_move_menu_entries(
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
) -> Result<Option<Vec<String>>> {
    let (title, moves, selected) = match runtime_shell.pending_script_party_selection.as_ref() {
        Some(PendingScriptPartySelection::MoveReminder {
            party_index: Some(party_index),
            moves,
        }) => {
            let selected = strict_readonly_cursor_index(
                &runtime_shell.party_move_cursor,
                &move_reminder_cursor_surface_id(*party_index),
                moves.len() + 1,
            )
            .context("Move Reminder has no valid move-or-CANCEL cursor")?;
            ("RELEARN A MOVE", moves.clone(), selected)
        }
        Some(PendingScriptPartySelection::MoveTutor {
            party_index: Some(party_index),
            ..
        }) => {
            let pokemon = &snapshot
                .party
                .slots
                .iter()
                .find(|slot| slot.index == *party_index)
                .context("Move Tutor party slot is empty")?
                .pokemon;
            let selected = strict_readonly_cursor_index(
                &runtime_shell.party_move_cursor,
                &party_move_cursor_surface_id(*party_index),
                pokemon.moves.len() + 1,
            )
            .context("Move Tutor has no valid move-or-CANCEL cursor")?;
            (
                "FORGET WHICH MOVE?",
                pokemon
                    .moves
                    .iter()
                    .map(|learned| learned.name.clone())
                    .collect(),
                selected,
            )
        }
        _ => return Ok(None),
    };
    let labels = moves
        .iter()
        .map(|move_id| battle_move_display_name(snapshot, move_id))
        .collect::<Vec<_>>();
    script_move_menu_entries(title, &labels, selected).map(Some)
}

fn script_move_menu_entries(
    title: &str,
    labels: &[String],
    selected: usize,
) -> Result<Vec<String>> {
    let count = labels.len() + 1;
    anyhow::ensure!(
        selected < count,
        "script move cursor is outside its move-or-CANCEL rows"
    );
    let visible_rows = SCENE_MENU_VISIBLE_ROWS - 1;
    let start = visible_window_start(selected, count, visible_rows);
    let mut entries = vec![title.to_string()];
    for index in start..(start + visible_rows).min(count) {
        let label = labels.get(index).map_or("CANCEL", String::as_str);
        entries.push(format!(
            "{}{label}",
            if index == selected { ">" } else { " " }
        ));
    }
    Ok(entries)
}

fn move_visible_script_move_cursor(
    runtime_shell: &mut BevyRuntimeShell,
    delta: isize,
) -> Result<bool> {
    let (surface_id, count) = match runtime_shell.pending_script_party_selection.as_ref() {
        Some(PendingScriptPartySelection::MoveReminder {
            party_index: Some(party_index),
            moves,
        }) => (
            move_reminder_cursor_surface_id(*party_index),
            moves.len() + 1,
        ),
        Some(PendingScriptPartySelection::MoveTutor {
            party_index: Some(party_index),
            ..
        }) => {
            let snapshot = runtime_shell.shell.snapshot()?;
            let count = snapshot
                .party
                .slots
                .iter()
                .find(|slot| slot.index == *party_index)
                .context("Move Tutor party slot is empty")?
                .pokemon
                .moves
                .len()
                + 1;
            (party_move_cursor_surface_id(*party_index), count)
        }
        _ => return Ok(false),
    };
    move_visible_cursor_slot(
        &mut runtime_shell.party_move_cursor,
        surface_id,
        count,
        delta,
        &mut runtime_shell.last_audio_events,
    )?;
    Ok(true)
}

fn spawn_visible_script_move_menu(
    commands: &mut Commands,
    entries: &[String],
    runtime_shell: &BevyRuntimeShell,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<()> {
    commands.spawn((
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(PLAYFIELD_WIDTH, PLAYFIELD_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 3.4),
        FieldCommandMarker,
    ));
    for (index, entry) in entries.iter().enumerate() {
        let (x, y) = battle_hud_tile_origin(1.0, 1.0 + index as f32 * 1.5);
        spawn_field_command_bitmap_text(
            commands,
            rendered_art,
            asset_root,
            images,
            entry,
            x,
            y,
            3.8,
        );
    }
    if runtime_shell.pc_notice.is_none() {
        let (x, y) = battle_hud_tile_origin(1.0, 15.0);
        spawn_field_command_bitmap_text(
            commands,
            rendered_art,
            asset_root,
            images,
            "A: CHOOSE  B: BACK",
            x,
            y,
            3.8,
        );
    }
    spawn_pc_notice(commands, runtime_shell, rendered_art, asset_root, images)
}

#[cfg(test)]
mod move_reminder_menu_tests {
    use super::script_move_menu_entries;

    #[test]
    fn long_move_lists_keep_selection_and_cancel_visible() {
        let labels = (0..12)
            .map(|index| format!("MOVE {index}"))
            .collect::<Vec<_>>();
        for selected in 0..=labels.len() {
            let entries = script_move_menu_entries("RELEARN A MOVE", &labels, selected)
                .expect("legal move selection");
            assert!(entries.len() <= 6);
            let selected_label = labels.get(selected).map_or("CANCEL", String::as_str);
            assert!(entries.contains(&format!(">{selected_label}")));
            assert_eq!(
                entries
                    .iter()
                    .filter(|entry| entry.starts_with('>'))
                    .count(),
                1
            );
        }
    }

    #[test]
    fn invalid_move_cursor_is_rejected_without_selecting_another_move() {
        let labels = vec!["TACKLE".to_string()];
        assert!(script_move_menu_entries("RELEARN A MOVE", &labels, 2).is_err());
        assert_eq!(
            script_move_menu_entries("RELEARN A MOVE", &[], 0)
                .expect("empty list has cancellation"),
            vec!["RELEARN A MOVE", ">CANCEL"]
        );
    }
}
