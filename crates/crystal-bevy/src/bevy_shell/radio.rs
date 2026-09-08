fn load_visible_radio_broadcast(shell: &mut BevyRuntimeShell) -> Result<()> {
    shell.pokegear_radio_input_blocked = false;
    let Some(station) = shell.pokegear_radio_station.as_deref() else {
        shell.pokegear_radio_broadcast = None;
        mark_runtime_snapshot_dirty(shell);
        return Ok(());
    };
    let line = match station {
        "OAKS_POKEMON_TALK" => 0,
        "POKEDEX_SHOW" => 1,
        "POKEMON_MUSIC" => 2,
        "LUCKY_CHANNEL" => 3,
        "BUENAS_PASSWORD" => 4,
        "PLACES_AND_PEOPLE" => 5,
        "LETS_ALL_SING" => 6,
        "ROCKET_RADIO" => 7,
        "POKE_FLUTE_RADIO" => 8,
        "UNOWN_RADIO" => 9,
        "EVOLUTION_RADIO" => 10,
        _ => anyhow::bail!("unknown source radio station {station}"),
    };
    // LoadStation changes the channel program, not wPokegearRadioMusicPlaying.
    // In particular, leaving immediately after retuning an Oak bumper must
    // still execute RestartMapMusic even before the new program gets a turn.
    let music_mode = shell.pokegear_radio_broadcast.as_ref()
        .and_then(|broadcast| broadcast.host.music_mode.clone());
    shell.pokegear_radio_broadcast = Some(crate::assets::radio_broadcast::RadioBroadcast::new(
        shell.shell.runtime().data(),
        line,
        shell.shell.session().state().time.current_day % 7,
    )?);
    shell.pokegear_radio_broadcast.as_mut().expect("loaded station").host.music_mode = music_mode;
    if shell.pokegear_map_radio_delay.is_some() {
        shell
            .pokegear_radio_broadcast
            .as_mut()
            .expect("loaded station")
            .host
            .music_mode = Some(crate::assets::radio_host::RadioMusicEffect::Stop);
    }
    mark_runtime_snapshot_dirty(shell);
    Ok(())
}

fn advance_visible_radio_broadcast(
    shell: &mut BevyRuntimeShell,
    frames: u32,
    held_ab: bool,
) -> Result<bool> {
    if frames != 0 {
        shell.pokegear_radio_input_blocked = false;
    }
    if frames == 0 || !shell.pokegear_menu_open || shell.pokegear_page != PokegearPage::Radio {
        return Ok(false);
    }
    let Some(mut broadcast) = shell.pokegear_radio_broadcast.take() else {
        return Ok(false);
    };
    let result = (|| -> Result<bool> {
        let snapshot = cached_runtime_snapshot(shell)?;
        let in_johto = visible_radio_program_in_johto(&snapshot)?;
        let mut suspended = false;
        let mut game_state_changed = false;
        let before_tiles = broadcast.playback.window.clone();
        let before_name = broadcast.host.name_tiles;
        for _ in 0..frames {
            suspended |= broadcast.playback.call_suspended();
            shell.shell.advance_radio_broadcast(&mut broadcast, in_johto, held_ab)?;
            game_state_changed |= broadcast.host.game_state_changed;
            suspended |= broadcast.playback.call_suspended();
        }
        for effect in std::mem::take(&mut broadcast.host.music) {
            use crate::assets::radio_host::RadioMusicEffect;
            let song = match effect {
                RadioMusicEffect::Stop => None,
                RadioMusicEffect::Restart(song) => Some(song),
                RadioMusicEffect::PokemonChannel => Some("MUSIC_POKEMON_CHANNEL"),
            };
            shell.pending_audio.clear();
            set_visible_stopped_music_state(shell, Some("MUSIC_NONE"));
            if let Some(song) = song {
                let playback = shell
                    .shell
                    .runtime()
                    .audio()
                    .require_playback_entry(AudioKind::Music, song)?;
                let command = BevyAudioCommand {
                    audio_id: song.into(),
                    kind: ModpackAudioKind::Music,
                    mode: playback.mode,
                    looped: matches!(
                        playback.loop_policy,
                        crate::assets::ModpackAudioLoopPolicy::Loop
                    ),
                };
                enqueue_bevy_audio_command(&mut shell.pending_audio, command);
                shell.active_music = Some(song.into());
                shell.faded_music = None;
                if matches!(effect, RadioMusicEffect::Restart(_)) {
                    shell.active_pokegear_radio =
                        Some((snapshot.overworld.map_name.clone(), song.into()));
                }
            } else {
                shell.active_pokegear_radio = None;
            }
        }
        if game_state_changed {
            mark_runtime_snapshot_dirty(shell);
        } else if before_tiles != broadcast.playback.window
            || before_name != broadcast.host.name_tiles
        {
            mark_runtime_presentation_dirty(shell);
        }
        if suspended {
            shell.pending_ui_button_presses.clear();
        }
        // The later hotkey system must not sample input on the frame that
        // finishes a suspended FarCall. PokeGear reaches DelayFrame first.
        shell.pokegear_radio_input_blocked = suspended;
        Ok(suspended)
    })();
    shell.pokegear_radio_broadcast = Some(broadcast);
    result
}

fn visible_radio_tile_row(tiles: &[u8]) -> Result<String> {
    let mut reverse = BTreeMap::new();
    // Stable choice where the renderer accepts multiple names for one tile.
    let ordered: BTreeMap<_, _> = bitmap_font_char_map().into_iter().collect();
    for (character, tile) in ordered {
        reverse.entry(tile).or_insert(character);
    }
    for (tile, character) in [
        (0x70, '\u{e108}'),
        (0x71, '\u{e109}'),
        (0x72, '“'),
        (0x73, '”'),
        (0xea, 'é'),
    ] {
        reverse.insert(tile, character);
    }
    tiles
        .iter()
        .map(|tile| {
            reverse
                .get(&u16::from(*tile))
                .copied()
                .with_context(|| format!("source radio tile {tile:#04x} has no font glyph"))
        })
        .collect()
}

fn visible_radio_text_rows(shell: &BevyRuntimeShell) -> Result<Vec<String>> {
    let broadcast = shell
        .pokegear_radio_broadcast
        .as_ref()
        .context("radio station has no live broadcast")?;
    broadcast.playback.window.tiles[1..5]
        .iter()
        .map(|row| visible_radio_tile_row(&row[1..19]))
        .collect()
}

fn visible_radio_observation_rows(shell: &BevyRuntimeShell) -> Result<Vec<String>> {
    Ok(visible_radio_text_rows(shell)?
        .into_iter()
        .map(|row| {
            row.replace('\u{e108}', "PO")
                .replace('\u{e109}', "Ké")
                .replace('\u{e105}', "PK")
                .replace('\u{e106}', "MN")
                .replace('\u{e107}', ".")
                .replace('\u{e120}', "'d")
                .replace('\u{e121}', "'l")
                .replace('\u{e122}', "'m")
                .replace('\u{e123}', "'r")
                .replace('\u{e124}', "'s")
                .replace('\u{e125}', "'t")
                .replace('\u{e126}', "'v")
                .trim_end()
                .to_string()
        })
        .collect())
}
