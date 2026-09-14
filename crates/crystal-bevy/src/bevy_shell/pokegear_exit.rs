#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum VisiblePokegearExitPhase {
    Requested,
    WaitSound,
    ClearPalettes { frames_remaining: u8 },
    RestoreMusic { song: String },
}

fn request_visible_pokegear_exit(shell: &mut BevyRuntimeShell) -> Result<()> {
    anyhow::ensure!(shell.pokegear_menu_open && !shell.pokegear_standalone_map
        && shell.pokegear_map_radio_delay.is_none(), "portable Pokégear exit requires its card");
    if shell.pokegear_exit.is_none() {
        // The jumptable only sets EXIT. PlaySpriteAnimations still owns the
        // remainder of this frame, before the next loop enters .done.
        shell.pokegear_exit = Some(VisiblePokegearExitPhase::Requested);
        mark_runtime_presentation_dirty(shell);
    }
    Ok(())
}

fn visible_pokegear_exit_palettes_are_clear(shell: &BevyRuntimeShell) -> bool {
    match shell.pokegear_exit {
        // ClearPalettes writes wBGPals2 and requests hCGBPalUpdate. The
        // currently scanned LCD still uses the old palettes; VBlank uploads
        // white before the next full frame (source exit-frame-19 -> 20).
        Some(VisiblePokegearExitPhase::ClearPalettes { frames_remaining }) => frames_remaining < 4,
        Some(VisiblePokegearExitPhase::RestoreMusic { .. }) => true,
        _ => false,
    }
}

fn advance_visible_pokegear_exit(shell: &mut BevyRuntimeShell, frames: u32) -> Result<bool> {
    let Some(phase) = shell.pokegear_exit.clone() else { return Ok(false); };
    shell.pokegear_exit_input_blocked = true;
    if frames == 0 { return Ok(true); }
    match phase {
        VisiblePokegearExitPhase::Requested => {
            queue_visible_shell_sound_effect(shell, "SFX_READ_TEXT_2")?;
            shell.pokegear_exit = Some(VisiblePokegearExitPhase::WaitSound);
        }
        VisiblePokegearExitPhase::WaitSound => {
            if shell.transient_audio_playing || shell.pending_audio.iter()
                .any(|command| !matches!(command.kind, ModpackAudioKind::Music))
            {
                return Ok(true);
            }
            // ClearBGPalettes calls ClearPalettes, then WaitBGMap's four
            // DelayFrames. WaitSFX above follows backend completion, not a
            // guessed duration for the quit sound.
            shell.pokegear_exit = Some(VisiblePokegearExitPhase::ClearPalettes { frames_remaining: 4 });
        }
        VisiblePokegearExitPhase::ClearPalettes { frames_remaining } => {
            let remaining = frames_remaining.saturating_sub(u8::try_from(frames).unwrap_or(u8::MAX));
            if remaining != 0 {
                shell.pokegear_exit = Some(VisiblePokegearExitPhase::ClearPalettes { frames_remaining: remaining });
            } else if let Some(song) = visible_pokegear_radio_exit_song(shell)? {
                // Both RestartMapMusic and PlayMapMusicBike play MUSIC_NONE
                // and DelayFrame before starting the restored map song.
                set_visible_stopped_music_state(shell, Some("MUSIC_NONE"));
                shell.pokegear_exit = Some(VisiblePokegearExitPhase::RestoreMusic { song });
            } else {
                complete_visible_pokegear_exit(shell)?;
            }
        }
        VisiblePokegearExitPhase::RestoreMusic { song } => {
            queue_visible_pokegear_restored_music(shell, song)?;
            complete_visible_pokegear_exit(shell)?;
        }
    }
    mark_runtime_presentation_dirty(shell);
    Ok(true)
}

fn complete_visible_pokegear_exit(shell: &mut BevyRuntimeShell) -> Result<()> {
    finish_visible_pokegear_menu(shell);
    if shell.start_menu_cursor.is_none() {
        continue_visible_script_after_prompt(shell)?;
    }
    Ok(())
}
