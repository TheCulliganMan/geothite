//! Compare actual intro IRQ executions with independently replayed sound programs.
//! Observed cycle totals are never inputs to the prediction.
use super::source_intro_oracle::{measure_routine_state, symbols};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

pub(super) fn certify(vendor: &Path, model: &Value, trace: &Value) -> Result<Value> {
    ensure!(
        trace["runner"] == "pack_core Rust intro oracle"
            && trace["rom_sha1"] == "f4cd194bdee0d04ca4eac29e09b8e4e9d818c133"
            && trace["scenes"]
                .as_array()
                .is_some_and(|scenes| scenes.len() == 28),
        "audio certificate requires the complete pinned Rust intro trace"
    );
    let symbols = symbols(&std::fs::read_to_string(vendor.join("pokecrystal.sym"))?)?;
    let audio_start = symbols.get("wAudio").context("missing wAudio")?.1;
    let audio_end = symbols.get("wAudioEnd").context("missing wAudioEnd")?.1;
    let value = |pointer: &str| {
        model
            .pointer(pointer)
            .and_then(Value::as_u64)
            .with_context(|| format!("missing audio timing {pointer}"))
    };
    let records = trace["records"]
        .as_array()
        .context("missing actual intro records")?;
    let time = |r: &Value, key: &str| {
        r[key]
            .as_u64()
            .with_context(|| format!("missing audio record {key}"))
    };
    let mut helper_records = records
        .iter()
        .filter(|r| {
            [
                "ApplyPitchSlide",
                "HandleTrackVibrato",
                "HandleNoise",
                "UpdateChannels",
                "ParseMusic",
                "PlayDanger",
                "FadeMusic",
            ]
            .contains(&r["hook"].as_str().unwrap_or(""))
        })
        .map(|r| {
            Ok((
                time(r, "start_cpu_t_cycles")?,
                time(r, "end_cpu_t_cycles")?,
                r,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    helper_records.sort_by_key(|r| r.0);
    let mut cache = BTreeMap::<String, u64>::new();
    let mut helper_count = 0usize;
    let mut sound_updates = Vec::new();
    for update in records.iter().filter(|r| r["hook"] == "_UpdateSound") {
        let begin = time(update, "start_cpu_t_cycles")?;
        let end = time(update, "end_cpu_t_cycles")?;
        let state = update["audio_ram"]
            .as_array()
            .context("missing actual sound RAM")?;
        ensure!(
            update["audio_ram_start_address"].as_u64() == Some(u64::from(audio_start))
                && state.len() == usize::from(audio_end - audio_start),
            "actual audio state has wrong extent"
        );
        let first = helper_records.partition_point(|r| r.0 < begin);
        let last = helper_records.partition_point(|r| r.0 < end);
        let current_helpers = &helper_records[first..last];
        ensure!(
            current_helpers
                .iter()
                .any(|r| r.2["hook"] == "PlayDanger" && r.1 <= end),
            "nonplaying audio guard is not calibrated in this intro model"
        );
        let mut predicted = value("/all_channels_inactive/machine_cycles")?;
        let mut channels = BTreeMap::<u16, (bool, bool)>::new();
        let mut invocation_returns = std::collections::BTreeSet::new();
        let mut helper_predictions = Vec::new();
        for (_, helper_end, record) in current_helpers {
            if *helper_end > end {
                continue;
            }
            let Some(hook) = record["hook"].as_str() else {
                continue;
            };
            if ![
                "ApplyPitchSlide",
                "HandleTrackVibrato",
                "HandleNoise",
                "UpdateChannels",
                "ParseMusic",
                "PlayDanger",
                "FadeMusic",
            ]
            .contains(&hook)
            {
                continue;
            }
            let channel = record["bc"]
                .as_u64()
                .and_then(|v| u16::try_from(v).ok())
                .context("invalid active channel address")?;
            // ParseMusic's authored jr ParseMusic command loop re-enters the same
            // symbol without a new call. Retain its outer invocation only.
            if !invocation_returns.insert((
                hook.to_owned(),
                time(record, "end_cpu_t_cycles")?,
                channel,
            )) {
                continue;
            }
            if [
                "ApplyPitchSlide",
                "HandleTrackVibrato",
                "HandleNoise",
                "UpdateChannels",
                "ParseMusic",
            ]
            .contains(&hook)
            {
                let entry = channels.entry(channel).or_default();
                if hook == "UpdateChannels" {
                    entry.0 = true;
                }
                if hook == "ParseMusic" {
                    entry.1 = true;
                }
            }
            let ram = record["audio_ram"]
                .as_array()
                .context("missing helper live RAM")?;
            let byte = |address: u16| -> Result<u8> {
                ram.get(usize::from(
                    address
                        .checked_sub(audio_start)
                        .context("audio predictor address precedes snapshot")?,
                ))
                .and_then(Value::as_u64)
                .and_then(|v| u8::try_from(v).ok())
                .context("audio predictor byte outside snapshot")
            };
            let named = |name: &str| -> Result<u8> {
                byte(
                    symbols
                        .get(name)
                        .with_context(|| format!("missing predictor symbol {name}"))?
                        .1,
                )
            };
            let helper_model = match hook {
                "ApplyPitchSlide" if byte(channel + 4)? & 2 == 0 => Some(value(
                    "/helper_inactive_paths/apply_pitch_slide/machine_cycles",
                )?),
                "PlayDanger" if named("wLowHealthAlarm")? & 128 == 0 => {
                    Some(value("/helper_inactive_paths/play_danger/machine_cycles")?)
                }
                "FadeMusic" if named("wMusicFade")? == 0 => {
                    Some(value("/helper_inactive_paths/fade_music/machine_cycles")?)
                }
                "HandleTrackVibrato" => {
                    let flags = byte(channel + 4)?;
                    let mut cost =
                        value("/helper_cycle_models/handle_track_vibrato/base_machine_cycles")?;
                    if flags & 4 != 0 {
                        cost += value(
                            "/helper_cycle_models/handle_track_vibrato/duty_loop_extra_machine_cycles",
                        )?;
                    }
                    if flags & 16 != 0 {
                        cost += value(
                            "/helper_cycle_models/handle_track_vibrato/pitch_offset_extra_machine_cycles",
                        )?;
                    }
                    if flags & 1 != 0 {
                        let extent = byte(channel + 31)?;
                        let frequency = named("wCurTrackFrequency")?;
                        let path = if byte(channel + 29)? != 0 {
                            "delay_count_nonzero"
                        } else if extent == 0 {
                            "zero_extent"
                        } else if byte(channel + 32)? & 15 != 0 {
                            "rate_count_nonzero"
                        } else if byte(channel + 5)? & 1 != 0 {
                            if frequency < extent & 15 {
                                "toggle_up_borrow"
                            } else {
                                "toggle_up_no_borrow"
                            }
                        } else if u16::from(frequency) + u16::from(extent >> 4) > 255 {
                            "toggle_down_carry"
                        } else {
                            "toggle_down_no_carry"
                        };
                        cost += value(&format!(
                            "/helper_cycle_models/handle_track_vibrato/vibrato_extra_machine_cycles/{path}"
                        ))?;
                    }
                    Some(cost)
                }
                "HandleNoise" => {
                    if byte(channel + 3)? & 16 == 0 {
                        Some(value(
                            "/helper_cycle_models/handle_noise/inactive_machine_cycles",
                        )?)
                    } else {
                        let cur = named("wCurChannel")?;
                        let ch8 = named("wChannel8Flags1")?;
                        if cur & 4 == 0 && ch8 & 17 == 17 {
                            Some(value(
                                "/helper_cycle_models/handle_noise/music_blocked_by_noise_ch8_machine_cycles",
                            )?)
                        } else {
                            let prefix = if cur & 4 != 0 {
                                "sfx_channel"
                            } else if ch8 & 1 == 0 {
                                "music_channel_with_ch8_off"
                            } else {
                                "music_channel_with_non_noise_ch8"
                            };
                            let mut cost = value(&format!(
                                "/helper_cycle_models/handle_noise/prefix_to_delay_check_machine_cycles/{prefix}"
                            ))?;
                            if named("wNoiseSampleDelay")? != 0 {
                                cost += value(
                                    "/helper_cycle_models/handle_noise/delay_machine_cycles/nonzero_return",
                                )?;
                            } else {
                                cost += value(
                                    "/helper_cycle_models/handle_noise/delay_machine_cycles/zero_to_sample_reader",
                                )?;
                                let address = symbols
                                    .get("wNoiseSampleAddress")
                                    .context("missing noise address")?
                                    .1;
                                let pointer =
                                    u16::from_le_bytes([byte(address)?, byte(address + 1)?]);
                                let path = if pointer == 0 {
                                    "empty_address"
                                } else if record["noise_sample_first_byte"].as_u64() == Some(255) {
                                    "sound_ret"
                                } else {
                                    "sample"
                                };
                                cost += value(&format!(
                                    "/helper_cycle_models/handle_noise/sample_reader_machine_cycles/{path}"
                                ))?;
                            }
                            Some(cost)
                        }
                    }
                }
                "UpdateChannels" => {
                    let cur = named("wCurChannel")?;
                    let flags = byte(channel + 12)?;
                    let slot = ["pulse1", "pulse2", "wave", "noise"][usize::from(cur & 3)];
                    let path = if flags & 32 != 0 {
                        Some("rest")
                    } else if flags & 16 != 0 {
                        Some("noise_sampling")
                    } else if cur & 3 == 1 && flags & 64 != 0 {
                        Some("vibrato_override")
                    } else if flags == 0 {
                        Some("unchanged")
                    } else {
                        None
                    };
                    if cur & 3 == 0
                        && (flags & 8 != 0 || (cur < 4 && named("wLowHealthAlarm")? & 128 != 0))
                    {
                        None
                    } else if let Some(path) = path {
                        let mut cost = value(&format!(
                            "/helper_cycle_models/update_channels_intro_paths/{slot}/{path}"
                        ))?;
                        if cur == 4 {
                            cost -= value(
                                "/helper_cycle_models/update_channels_intro_paths/pulse1/sfx_prefix_reduction_machine_cycles",
                            )?;
                        }
                        Some(cost)
                    } else {
                        None
                    }
                }
                _ => None,
            };
            let mut writes = ram
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    Ok((
                        format!("wAudio+{i}"),
                        u8::try_from(v.as_u64().context("invalid audio state byte")?)?,
                    ))
                })
                .collect::<Result<Vec<_>>>()?;
            if symbols.contains_key("hROMBank") {
                writes.push((
                    "hROMBank".into(),
                    u8::try_from(record["hROMBank"].as_u64().unwrap_or(time(record, "bank")?))?,
                ));
            }
            for name in ["wOptions", "wPlayerState"] {
                if let Some(v) = record.get(name).and_then(Value::as_u64) {
                    writes.push((name.into(), u8::try_from(v)?));
                }
            }
            let registers = [
                ("BC", channel),
                ("DE", u16::try_from(time(record, "de")?)?),
                ("HL", u16::try_from(time(record, "hl")?)?),
                ("A", u16::try_from(time(record, "a")?)?),
            ];
            let key = if hook == "UpdateChannels" {
                // These are the only conditional inputs to UpdateChannels; duty,
                // frequency and envelope bytes are hardware write data, not branches.
                serde_json::to_string(&json!([
                    hook,
                    named("wCurChannel")?,
                    byte(channel + 12)?,
                    named("wLowHealthAlarm")?
                ]))?
            } else {
                serde_json::to_string(&json!([hook, writes, registers]))?
            };
            let cost = if let Some(cost) = helper_model {
                cost
            } else if let Some(cost) = cache.get(&key) {
                *cost
            } else {
                let result = measure_routine_state(vendor, hook, &writes, &registers)?;
                let cost = result["machine_cycles"]
                    .as_u64()
                    .context("missing independently replayed cost")?;
                cache.insert(key, cost);
                cost
            };
            let actual = time(record, "instruction_machine_cycles")?;
            ensure!(
                cost == actual,
                "actual intro {hook} replay cost disagrees in scene {}, tick {}: {cost} != {actual}",
                record["dispatcher_entry"],
                record["dispatch_tick"]
            );
            helper_count += 1;
            // Danger/fade inactive costs are already included in the calibrated base.
            if hook == "PlayDanger" {
                predicted += cost - value("/helper_inactive_paths/play_danger/machine_cycles")?;
            } else if hook == "FadeMusic" {
                predicted += cost - value("/helper_inactive_paths/fade_music/machine_cycles")?;
            } else {
                predicted += cost;
            }
            helper_predictions
                .push(json!({"hook":hook,"machine_cycles":cost,"channel_address":channel}));
        }
        for (address, (writes_hardware, parsed_note)) in &channels {
            let index = usize::from(
                address
                    .checked_sub(audio_start)
                    .context("channel precedes audio RAM")?,
            ) / 50;
            ensure!(index < 8, "unexpected channel address");
            let class = if index >= 4 {
                "sfx"
            } else if *writes_hardware {
                "music_without_active_sfx"
            } else {
                "music_shadowed_by_active_sfx"
            };
            predicted += value(&format!(
                "/active_channel_paths/extra_over_inactive_channel_machine_cycles/{class}"
            ))?;
            if *parsed_note {
                predicted +=
                    value("/active_channel_paths/note_over_extra_before_parse_machine_cycles")?;
            }
        }
        let actual = time(update, "instruction_machine_cycles")?;
        ensure!(
            predicted == actual,
            "complete actual intro sound IRQ disagrees in scene {}, tick {}: {predicted} != {actual}",
            update["dispatcher_entry"],
            update["dispatch_tick"]
        );
        sound_updates.push(json!({"start_cpu_t_cycles":begin,"end_cpu_t_cycles":end,"machine_cycles":predicted,"helpers":helper_predictions}));
    }
    ensure!(
        !sound_updates.is_empty() && helper_count > 0,
        "actual intro audio certificate has no live executions"
    );
    Ok(
        json!({"schema_version":1,"sound_update_count":sound_updates.len(),"helper_count":helper_count,"independent_replay_state_count":cache.len(),"audio_irq_model_matches_rom":true,"sound_updates":sound_updates}),
    )
}
