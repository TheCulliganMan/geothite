//! Source-command audio costs driven by the evolving eight-channel sound clock.
use anyhow::{Context, Result, bail, ensure};
use crystal_audio::synth::{SourceChannelState, SourceChannelStep, SourceCommandStep};
use serde_json::Value;

pub fn runtime_intro_source_audio_update_machine_cycles(
    model: &Value,
    steps: &[SourceChannelStep],
    channels_before: &[SourceChannelState],
    stereo: bool,
) -> Result<u64> {
    let number = |pointer: &str| -> Result<u64> {
        model
            .pointer(pointer)
            .and_then(Value::as_u64)
            .with_context(|| format!("missing source-clock audio timing {pointer}"))
    };
    let runtime = &model["runtime_source_clock"];
    ensure!(
        runtime["schema_version"] == 1,
        "audio timing lacks its source-clock model"
    );
    let state = |channel: u8| {
        channels_before
            .iter()
            .find(|state| state.channel == channel)
    };
    let mut total = number("/all_channels_inactive/machine_cycles")?;
    for step in steps {
        let slot = usize::from(
            step.before
                .channel
                .checked_sub(1)
                .context("source audio channel zero")?,
        );
        ensure!(
            slot < 8 && step.before.flags1 & 1 != 0,
            "invalid active source audio channel"
        );
        let class = if slot >= 4 {
            "sfx"
        } else if step.shadowed {
            "music_shadowed_by_active_sfx"
        } else {
            "music_without_active_sfx"
        };
        total += number(&format!(
            "/active_channel_paths/extra_over_inactive_channel_machine_cycles/{class}"
        ))?;
        if step.note_over {
            total += number("/active_channel_paths/note_over_extra_before_parse_machine_cycles")?;
            for (index, command) in step.command_steps.iter().enumerate() {
                let last = index + 1 == step.command_steps.len();
                let op = command.command.command.as_str();
                if matches!(
                    op,
                    "note" | "rest" | "drum_note" | "square_note" | "noise_note" | "wave_note"
                ) || (op == "sound_ret" && command.before.flags1 & 2 == 0)
                {
                    ensure!(last, "terminal source audio command did not end parser");
                    total += terminal_cost(
                        model,
                        runtime,
                        command,
                        state(step.before.channel.saturating_add(4))
                            .is_some_and(|s| s.flags1 & 1 != 0),
                        state(8).is_some_and(|s| s.flags1 & 1 != 0),
                    )?;
                } else {
                    let key = command_key(command, stereo)?;
                    total += runtime["parser_command_machine_cycles"][slot][key]
                        .as_u64()
                        .with_context(|| {
                            format!(
                                "unmodeled source audio command {key} on channel {}",
                                step.before.channel
                            )
                        })?;
                }
            }
            ensure!(
                !step.command_steps.is_empty(),
                "active source audio parser ended without authored terminal command"
            );
        }
        ensure!(
            step.parsed.flags2 & 2 == 0,
            "intro source pitch-slide timing is not modeled"
        );
        total += number("/helper_inactive_paths/apply_pitch_slide/machine_cycles")?;
        let channel = &step.parsed;
        total += number("/helper_cycle_models/handle_track_vibrato/base_machine_cycles")?;
        let mut frequency = channel.frequency;
        if channel.flags2 & 4 != 0 {
            total +=
                number("/helper_cycle_models/handle_track_vibrato/duty_loop_extra_machine_cycles")?;
        }
        if channel.flags2 & 16 != 0 {
            total += number(
                "/helper_cycle_models/handle_track_vibrato/pitch_offset_extra_machine_cycles",
            )?;
            frequency = frequency.wrapping_add(channel.pitch_offset);
        }
        if channel.flags2 & 1 != 0 {
            let path = if channel.vibrato_delay_count != 0 {
                "delay_count_nonzero"
            } else if channel.vibrato_extent == 0 {
                "zero_extent"
            } else if channel.vibrato_rate & 15 != 0 {
                "rate_count_nonzero"
            } else if channel.flags3 & 1 != 0 {
                if (frequency as u8) < channel.vibrato_extent & 15 {
                    "toggle_up_borrow"
                } else {
                    "toggle_up_no_borrow"
                }
            } else if u16::from(frequency as u8) + u16::from(channel.vibrato_extent >> 4) > 255 {
                "toggle_down_carry"
            } else {
                "toggle_down_no_carry"
            };
            total += number(&format!(
                "/helper_cycle_models/handle_track_vibrato/vibrato_extra_machine_cycles/{path}"
            ))?;
        }
        if channel.flags1 & 16 == 0 {
            total += number("/helper_cycle_models/handle_noise/inactive_machine_cycles")?;
        } else {
            let ch8 = state(8).map_or(0, |s| s.flags1);
            if slot < 4 && ch8 & 17 == 17 {
                total += number(
                    "/helper_cycle_models/handle_noise/music_blocked_by_noise_ch8_machine_cycles",
                )?;
            } else {
                let prefix = if slot >= 4 {
                    "sfx_channel"
                } else if ch8 & 1 == 0 {
                    "music_channel_with_ch8_off"
                } else {
                    "music_channel_with_non_noise_ch8"
                };
                total += number(&format!(
                    "/helper_cycle_models/handle_noise/prefix_to_delay_check_machine_cycles/{prefix}"
                ))?;
                if step.noise_delay_before != 0 {
                    total += number(
                        "/helper_cycle_models/handle_noise/delay_machine_cycles/nonzero_return",
                    )?;
                } else {
                    total += number(
                        "/helper_cycle_models/handle_noise/delay_machine_cycles/zero_to_sample_reader",
                    )?;
                    let sample = match step.noise_sample_first_byte {
                        None => "empty_address",
                        Some(255) => "sound_ret",
                        Some(_) => "sample",
                    };
                    total += number(&format!(
                        "/helper_cycle_models/handle_noise/sample_reader_machine_cycles/{sample}"
                    ))?;
                }
            }
        }
        if step.hardware_write {
            let flags = usize::from(step.after.note_flags);
            ensure!(
                flags < 128,
                "intro hardware note flags exceed the calibrated source domain"
            );
            total += runtime["update_channels_machine_cycles"][slot][flags]
                .as_u64()
                .context("missing exact source UpdateChannels flag path")?;
        }
    }
    Ok(total)
}

fn arg(command: &SourceCommandStep, index: usize) -> Result<i32> {
    let value = command
        .command
        .args
        .get(index)
        .context("source timing command argument")?
        .trim();
    if let Some(value) = value.strip_prefix('$').or_else(|| value.strip_prefix("0x")) {
        Ok(i32::from_str_radix(value, 16)?)
    } else if let Some(value) = value.strip_prefix('%') {
        Ok(i32::from_str_radix(value, 2)?)
    } else {
        Ok(value.parse()?)
    }
}
fn command_key(command: &SourceCommandStep, stereo: bool) -> Result<&str> {
    Ok(match command.command.command.as_str() {
        "speed" | "drum_speed" | "note_type" => "note_type",
        "octave" => "octave",
        "transpose"
        | "tempo"
        | "duty_cycle"
        | "duty_cycle_pattern"
        | "vibrato"
        | "volume"
        | "pitch_offset"
        | "force_stereo_panning"
        | "sound_jump"
        | "sound_call" => command.command.command.as_str(),
        "volume_envelope" | "channel_volume" => "volume_envelope",
        "pitch_sweep" | "sweep" => "pitch_sweep",
        "toggle_sfx" => {
            if command.before.flags1 & 8 != 0 {
                "toggle_sfx_off"
            } else {
                "toggle_sfx_on"
            }
        }
        "toggle_noise" => {
            if command.before.flags1 & 16 != 0 {
                "toggle_noise_off"
            } else {
                "toggle_noise_on"
            }
        }
        "sfx_toggle_noise" => {
            if command.before.flags1 & 16 != 0 {
                "sfx_toggle_noise_off"
            } else {
                "sfx_toggle_noise_on"
            }
        }
        "stereo_panning" => {
            if stereo {
                "stereo_panning_on"
            } else {
                "stereo_panning_off"
            }
        }
        "sound_ret" => "sound_ret_subroutine",
        "sound_loop" => {
            if command.before.flags1 & 4 != 0 {
                if command.before.loop_count == 0 {
                    "sound_loop_set_empty"
                } else {
                    "sound_loop_set_remaining"
                }
            } else {
                match arg(command, 0)? {
                    0 => "sound_loop_unset_zero",
                    1 => "sound_loop_unset_one",
                    _ => "sound_loop_unset_many",
                }
            }
        }
        "sfx_priority_on" | "sfx_priority_off" => {
            bail!("intro source SFX-priority branch requires an explicit priority state")
        }
        other => bail!("unmodeled intro source audio command {other}"),
    })
}
fn terminal_cost(
    model: &Value,
    runtime: &Value,
    command: &SourceCommandStep,
    shadowed: bool,
    noise_blocked: bool,
) -> Result<u64> {
    let state = &command.before;
    let slot = usize::from(state.channel - 1);
    let op = command.command.command.as_str();
    let end = op == "sound_ret";
    let sfx = state.flags1 & 8 != 0;
    let cry = state.flags1 & 32 != 0;
    let noise = state.flags1 & 16 != 0;
    let path = if end {
        if cry {
            "cry_end"
        } else if shadowed && slot < 4 {
            "end_shadowed"
        } else {
            "end"
        }
    } else if sfx {
        "sfx_note"
    } else if cry {
        "cry_note"
    } else if noise {
        if op == "rest" {
            if noise_blocked && slot < 4 {
                "noise_rest_blocked"
            } else {
                "noise_rest"
            }
        } else if noise_blocked && slot < 4 {
            "noise_note_blocked"
        } else {
            "noise_note"
        }
    } else if op == "rest" {
        "normal_rest"
    } else {
        "normal_note"
    };
    let mut total = runtime["parser_terminal_base_machine_cycles"][slot][path]
        .as_u64()
        .with_context(|| format!("missing source ParseMusic terminal {path}"))?;
    if end {
        return Ok(total);
    }
    if sfx || cry || !noise || slot % 4 == 3 {
        let encoded = if sfx || cry {
            arg(command, 0)? as u8
        } else if op == "rest" {
            arg(command, 0)?.wrapping_sub(1) as u8 & 15
        } else {
            arg(command, 1)?.wrapping_sub(1) as u8 & 15
        };
        let duration =
            &model["helper_cycle_models"]["parse_music_intro_paths"]["set_note_duration"];
        let c = |key: &str| {
            duration[key]
                .as_u64()
                .with_context(|| format!("missing source duration cost {key}"))
        };
        let multiply = |v: u8| -> Result<u64> {
            Ok(c("multiply_fixed_machine_cycles")?
                + c("multiply_per_bit_machine_cycles")? * u64::from((8 - v.leading_zeros()).max(1))
                + c("multiply_set_bit_extra_machine_cycles")? * u64::from(v.count_ones()))
        };
        total += c("fixed_machine_cycles")?
            + multiply(state.note_length)?
            + multiply(state.note_length.wrapping_mul(encoded.wrapping_add(1)))?;
    }
    if !sfx && !cry && !noise && op != "rest" {
        let freq = &model["helper_cycle_models"]["parse_music_intro_paths"]["get_frequency"];
        total += freq["fixed_machine_cycles"]
            .as_u64()
            .context("frequency base cost")?
            + freq["per_right_shift_machine_cycles"]
                .as_u64()
                .context("frequency shift cost")?
                * u64::from(7u8.saturating_sub(state.octave.wrapping_add(state.transpose >> 4)));
    }
    Ok(total)
}

#[derive(Clone, Debug)]
pub struct RuntimeIntroAudioPlaybackTiming {
    pub instruction_machine_cycles: Vec<u64>,
    pub music_playing_effects: Vec<(usize, u8)>,
    pub playback_instruction_index: Option<usize>,
    pub accepted: bool,
}

/// Choose the authored caller path before changing the SourceAudioClock.
/// Lower source SFX IDs have priority; rejected callers retain the old program.
pub fn runtime_intro_source_audio_playback_timing(
    model: &Value,
    asset: &str,
    channels_before: &[SourceChannelState],
    current_sfx_priority: Option<u8>,
) -> Result<RuntimeIntroAudioPlaybackTiming> {
    let callers = &model["runtime_source_clock"]["playback"];
    ensure!(
        callers["schema_version"] == 1,
        "missing source audio caller model"
    );
    let entry = &callers["callers"][asset];
    let (path, accepted) = if asset.starts_with("MUSIC_") {
        (&entry["music"], true)
    } else {
        let mut mask = 0usize;
        for state in channels_before {
            if (5..=8).contains(&state.channel) && state.flags1 & 1 != 0 {
                mask |= 1usize << (state.channel - 5);
            }
        }
        let next = entry["source_id"]
            .as_u64()
            .and_then(|n| u8::try_from(n).ok())
            .context("missing authored SFX priority")?;
        let accepted = mask == 0
            || current_sfx_priority.context("active SFX lacks authoritative source priority")?
                >= next;
        (
            &entry[if accepted {
                "sfx_accepted"
            } else {
                "sfx_rejected"
            }][mask],
            accepted,
        )
    };
    ensure!(
        path["accepted"].as_bool() == Some(accepted),
        "missing authored audio caller {asset}"
    );
    let tape = path["instruction_machine_cycles"]
        .as_array()
        .context("source caller instruction tape")?
        .iter()
        .map(|n| n.as_u64().context("source caller instruction cost"))
        .collect::<Result<Vec<_>>>()?;
    let effects = path["music_playing_effects"]
        .as_array()
        .context("source caller effects")?
        .iter()
        .map(|effect| {
            Ok((
                usize::try_from(
                    effect["instruction_index"]
                        .as_u64()
                        .context("source caller effect index")?,
                )?,
                u8::try_from(
                    effect["music_playing"]
                        .as_u64()
                        .context("source caller playing effect")?,
                )?,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    let playback = path["playback_instruction_index"]
        .as_u64()
        .map(usize::try_from)
        .transpose()?;
    ensure!(
        !tape.is_empty()
            && effects.iter().all(|(index, _)| *index < tape.len())
            && playback.is_none_or(|index| index < tape.len()),
        "source caller tape has invalid effect boundaries"
    );
    Ok(RuntimeIntroAudioPlaybackTiming {
        instruction_machine_cycles: tape,
        music_playing_effects: effects,
        playback_instruction_index: playback,
        accepted,
    })
}
