//! Audio interrupt costs measured by executing the pinned ROM's sound engine.
use super::source_intro_oracle::{measure_routine_state, symbols};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

pub(super) fn export(vendor: &Path) -> Result<Value> {
    let symbols = symbols(&std::fs::read_to_string(vendor.join("pokecrystal.sym"))?)?;
    let address = |name: &str| -> Result<u16> {
        Ok(symbols
            .get(name)
            .with_context(|| format!("missing audio symbol {name}"))?
            .1)
    };
    let channel = address("wChannel1")?;
    let measure = |label: &str, writes: &[(&str, u8)], regs: &[(&str, u16)]| -> Result<Value> {
        let mut registers = vec![("BC", channel)];
        registers.extend_from_slice(regs);
        measure_routine_state(
            vendor,
            label,
            &writes
                .iter()
                .map(|(s, v)| (s.to_string(), *v))
                .collect::<Vec<_>>(),
            &registers,
        )
    };
    let cycles = |label: &str, writes: &[(&str, u8)], regs: &[(&str, u16)]| -> Result<u64> {
        measure(label, writes, regs)?["machine_cycles"]
            .as_u64()
            .context("missing measured machine cycles")
    };
    let prefix = |trace: &Value, end: &str| -> Result<u64> {
        let (bank, addr) = symbols.get(end).context("missing audio trace boundary")?;
        let instructions = trace["instructions"]
            .as_array()
            .context("missing instruction trace")?;
        let stop = instructions
            .iter()
            .position(|i| {
                i["bank"].as_u64() == Some(*bank as u64)
                    && i["address"].as_u64() == Some(*addr as u64)
            })
            .context("audio trace did not reach boundary")?;
        Ok(instructions[..stop]
            .iter()
            .map(|i| i["machine_cycles"].as_u64().unwrap())
            .sum())
    };
    let mut inactive = BTreeMap::new();
    for (key, label, guard) in [
        (
            "apply_pitch_slide",
            "ApplyPitchSlide",
            "SOUND_PITCH_SLIDE_clear",
        ),
        (
            "handle_track_vibrato",
            "HandleTrackVibrato",
            "SOUND_DUTY_LOOP_SOUND_PITCH_OFFSET_SOUND_VIBRATO_clear",
        ),
        ("handle_noise", "HandleNoise", "SOUND_NOISE_clear"),
        ("play_danger", "PlayDanger", "DANGER_ON_clear"),
        ("fade_music", "FadeMusic", "wMusicFade_zero"),
    ] {
        inactive.insert(
            key,
            json!({"guard":guard,"machine_cycles":cycles(label,&[],&[])?}),
        );
    }
    let base = cycles("HandleTrackVibrato", &[], &[])?;
    let duty = cycles("HandleTrackVibrato", &[("wChannel1Flags2", 4)], &[])? - base;
    let pitch = cycles("HandleTrackVibrato", &[("wChannel1Flags2", 16)], &[])? - base;
    let mut vibrato = BTreeMap::new();
    for (key, delay, extent, rate, dir, freq) in [
        ("delay_count_nonzero", 1, 0, 0, 0, 0),
        ("zero_extent", 0, 0, 0, 0, 0),
        ("rate_count_nonzero", 0, 0x11, 1, 0, 0),
        ("toggle_up_no_borrow", 0, 0x11, 0, 1, 32),
        ("toggle_up_borrow", 0, 0x11, 0, 1, 0),
        ("toggle_down_no_carry", 0, 0x11, 0, 0, 32),
        ("toggle_down_carry", 0, 0x11, 0, 0, 255),
    ] {
        vibrato.insert(
            key,
            cycles(
                "HandleTrackVibrato",
                &[
                    ("wChannel1Flags2", 1),
                    ("wChannel1VibratoDelayCount", delay),
                    ("wChannel1VibratoExtent", extent),
                    ("wChannel1VibratoRate", rate),
                    ("wChannel1Flags3", dir),
                    ("wCurTrackFrequency", freq),
                ],
                &[],
            )? - base,
        );
    }
    let mut updates = BTreeMap::new();
    for (key, index, extra, flag) in [
        ("pulse1", 0, "noise_sampling", 16),
        ("pulse2", 1, "vibrato_override", 64),
        ("wave", 2, "noise_sampling", 16),
        ("noise", 3, "noise_sampling", 16),
    ] {
        let mut paths = serde_json::Map::new();
        paths.insert(
            "unchanged".into(),
            json!(cycles("UpdateChannels", &[("wCurChannel", index)], &[])?),
        );
        paths.insert(
            extra.into(),
            json!(cycles(
                "UpdateChannels",
                &[("wCurChannel", index), ("wChannel1NoteFlags", flag)],
                &[]
            )?),
        );
        paths.insert(
            "noise_sampling".into(),
            json!(cycles(
                "UpdateChannels",
                &[("wCurChannel", index), ("wChannel1NoteFlags", 16)],
                &[]
            )?),
        );
        paths.insert(
            "rest".into(),
            json!(cycles(
                "UpdateChannels",
                &[("wCurChannel", index), ("wChannel1NoteFlags", 32)],
                &[]
            )?),
        );
        if index == 0 {
            paths.insert(
                "sfx_prefix_reduction_machine_cycles".into(),
                json!(
                    cycles("UpdateChannels", &[("wCurChannel", 0)], &[])?
                        - cycles("UpdateChannels", &[("wCurChannel", 4)], &[])?
                ),
            );
        }
        updates.insert(key, Value::Object(paths));
    }
    let freq_fixed = cycles("GetFrequency", &[], &[("DE", 0x0701)])?;
    let freq_shift = cycles("GetFrequency", &[], &[("DE", 0x0601)])? - freq_fixed;
    let mul_zero = cycles("SetNoteDuration.Multiply", &[], &[("A", 0)])?;
    let mul_one = cycles("SetNoteDuration.Multiply", &[], &[("A", 1)])?;
    let mul_two = cycles("SetNoteDuration.Multiply", &[], &[("A", 2)])?;
    let per_bit = mul_two - mul_one;
    let set_bit = mul_one - mul_zero;
    let mul_fixed = mul_zero - per_bit;
    let duration = cycles(
        "SetNoteDuration",
        &[("wChannel1NoteLength", 1)],
        &[("A", 0)],
    )?;
    let duration_fixed = duration - 2 * mul_one;
    let stream = address("wCurMusicByte")?;
    let stream_fixture = [
        ("wChannel1MusicAddress", stream as u8),
        ("wChannel1MusicAddress+1", (stream >> 8) as u8),
        ("wCurMusicByte", 0x10),
        ("wChannel1NoteLength", 1),
        ("wChannel1Octave", 7),
    ];
    let normal = cycles("ParseMusic", &stream_fixture, &[])? - duration - freq_fixed;
    let mut noise_fixture = stream_fixture.to_vec();
    noise_fixture.extend([("wChannel1Flags1", 16), ("wCurChannel", 3)]);
    let noise_note = cycles("ParseMusic", &noise_fixture, &[])? - duration;
    // An octave command followed by the same note measures the parser's command loop.
    let mut octave_fixture = stream_fixture.to_vec();
    octave_fixture.extend([("wCurMusicByte", 0xd7), ("wCurMusicByte+1", 0x10)]);
    let octave =
        cycles("ParseMusic", &octave_fixture, &[])? - cycles("ParseMusic", &stream_fixture, &[])?;
    let reader_empty = cycles("ReadNoiseSample", &[], &[])?;
    let reader_fixture = [
        ("wNoiseSampleAddress", stream as u8),
        ("wNoiseSampleAddress+1", (stream >> 8) as u8),
    ];
    let mut ret_fixture = reader_fixture.to_vec();
    ret_fixture.push(("wCurMusicByte", 255));
    let mut sample_fixture = reader_fixture.to_vec();
    sample_fixture.push(("wCurMusicByte", 0x10));
    let mut prefixes = BTreeMap::new();
    for (key, cur, ch8) in [
        ("sfx_channel", 7, 0),
        ("music_channel_with_ch8_off", 3, 0),
        ("music_channel_with_non_noise_ch8", 3, 1),
    ] {
        prefixes.insert(
            key,
            prefix(
                &measure(
                    "HandleNoise",
                    &[
                        ("wChannel1Flags1", 16),
                        ("wCurChannel", cur),
                        ("wChannel8Flags1", ch8),
                    ],
                    &[],
                )?,
                "HandleNoise.next",
            )?,
        );
    }
    let delay_empty = cycles("HandleNoise.next", &[], &[])? - reader_empty;
    let delay_nonzero = cycles("HandleNoise.next", &[("wNoiseSampleDelay", 1)], &[])?;
    let all = cycles("_UpdateSound", &[("wMusicPlaying", 1)], &[])?;
    let mut active_paths = BTreeMap::new();
    let helper_sum = inactive["apply_pitch_slide"]["machine_cycles"]
        .as_u64()
        .unwrap()
        + base
        + inactive["handle_noise"]["machine_cycles"].as_u64().unwrap();
    let inactive_trace = measure("_UpdateSound.loop", &[], &[])?;
    let inactive_iteration = prefix(&inactive_trace, "_UpdateSound.nextchannel")?;
    for (key, cur, shadow) in [
        ("music_without_active_sfx", 0, false),
        ("sfx", 4, false),
        ("music_shadowed_by_active_sfx", 0, true),
    ] {
        let writes = vec![
            ("wMusicPlaying", 1),
            ("wChannel1Flags1", 1),
            ("wChannel1NoteDuration", 2),
            ("wCurChannel", cur),
            ("wChannel5Flags1", if shadow { 1 } else { 0 }),
        ];
        let trace = measure("_UpdateSound.loop", &writes, &[])?;
        let extra = prefix(&trace, "_UpdateSound.nextchannel")? - inactive_iteration;
        let update = if shadow {
            0
        } else {
            cycles("UpdateChannels", &[("wCurChannel", cur)], &[])?
        };
        active_paths.insert(
            key,
            extra
                .checked_sub(helper_sum + update)
                .context("active channel cost underflow")?,
        );
    }
    let sustained = measure(
        "_UpdateSound",
        &[
            ("wMusicPlaying", 1),
            ("wChannel1Flags1", 1),
            ("wChannel1NoteDuration", 2),
        ],
        &[],
    )?;
    let mut note_fixture = stream_fixture.to_vec();
    note_fixture.extend([
        ("wMusicPlaying", 1),
        ("wChannel1Flags1", 1),
        ("wChannel1NoteDuration", 1),
    ]);
    let note_trace = measure("_UpdateSound", &note_fixture, &[])?;
    let note_over = prefix(&note_trace, "ParseMusic")?
        - prefix(&sustained, "_UpdateSound.continue_sound_update")?;
    // Certify the compact arithmetic model against independent routine executions.
    // Costs are fitted above, then checked on inputs not used to fit them.
    let multiply_cost = |value: u8| {
        mul_fixed
            + per_bit * u64::from((8 - value.leading_zeros()).max(1))
            + set_bit * u64::from(value.count_ones())
    };
    for length in [0u8, 3, 6, 12, 255] {
        for nibble in [0u8, 1, 7, 15] {
            let actual = cycles(
                "SetNoteDuration",
                &[("wChannel1NoteLength", length)],
                &[("A", u16::from(nibble))],
            )?;
            let expected = duration_fixed
                + multiply_cost(length)
                + multiply_cost(length.wrapping_mul(nibble + 1));
            ensure!(
                actual == expected,
                "SetNoteDuration timing disagrees at length {length}, duration {nibble}: {actual} != {expected}"
            );
        }
    }
    for octave in 0..=7u16 {
        ensure!(
            cycles("GetFrequency", &[], &[("DE", (octave << 8) | 1)])?
                == freq_fixed + freq_shift * u64::from(7 - octave),
            "GetFrequency timing disagrees at octave {octave}"
        );
    }
    for length in [3u8, 6, 12] {
        for nibble in [0u8, 1, 7, 15] {
            for octave in [0u8, 3, 7] {
                let fixture = [
                    ("wMusicPlaying", 1),
                    ("wChannel1Flags1", 1),
                    ("wChannel1NoteDuration", 1),
                    ("wChannel1MusicAddress", stream as u8),
                    ("wChannel1MusicAddress+1", (stream >> 8) as u8),
                    ("wCurMusicByte", 0x10 | nibble),
                    ("wChannel1NoteLength", length),
                    ("wChannel1Octave", octave),
                ];
                let expected = all
                    + active_paths["music_without_active_sfx"]
                    + helper_sum
                    + updates["pulse1"]["noise_sampling"]
                        .as_u64()
                        .context("missing measured pulse write cost")?
                    + note_over
                    + normal
                    + duration_fixed
                    + multiply_cost(length)
                    + multiply_cost(length.wrapping_mul(nibble + 1))
                    + freq_fixed
                    + freq_shift * u64::from(7 - octave);
                let actual = cycles("_UpdateSound", &fixture, &[])?;
                ensure!(
                    actual == expected,
                    "complete _UpdateSound timing disagrees at length {length}, duration {nibble}, octave {octave}: {actual} != {expected}"
                );
            }
        }
    }
    ensure!(
        freq_shift > 0 && per_bit > 0,
        "sound engine timing model has no measured loop cost"
    );
    let mut model = json!({"routine":"_UpdateSound","cadence":"every_vblank","playing_guard":{"source":"wMusicPlaying","predicate":"nonzero"},"timing":"state_dependent","all_channels_inactive":{"predicate":"all_SOUND_CHANNEL_ON_flags_clear","machine_cycles":all},"helper_inactive_paths":inactive,"active_channel_paths":{"guard":{"sfx_priority":"zero","sustained_note_duration":"at_least_2"},"extra_over_inactive_channel_machine_cycles":active_paths,"note_over_extra_before_parse_machine_cycles":note_over},"helper_cycle_models":{
        "handle_track_vibrato":{"base_machine_cycles":base,"duty_loop_extra_machine_cycles":duty,"pitch_offset_extra_machine_cycles":pitch,"vibrato_extra_machine_cycles":vibrato},
        "update_channels_intro_paths":updates,
        "parse_music_intro_paths":{"normal_note_base_machine_cycles":normal,"music_noise_note_base_machine_cycles":noise_note,"octave_command_machine_cycles":octave,"set_note_duration":{"fixed_machine_cycles":duration_fixed,"multiply_per_bit_machine_cycles":per_bit,"multiply_fixed_machine_cycles":mul_fixed,"multiply_set_bit_extra_machine_cycles":set_bit,"minimum_multiply_iterations":1},"get_frequency":{"fixed_machine_cycles":freq_fixed,"per_right_shift_machine_cycles":freq_shift,"target_octave":7}},
        "handle_noise":{"inactive_machine_cycles":inactive["handle_noise"]["machine_cycles"],"prefix_to_delay_check_machine_cycles":prefixes,"music_blocked_by_noise_ch8_machine_cycles":cycles("HandleNoise",&[("wChannel1Flags1",16),("wChannel8Flags1",17)],&[])?,"delay_machine_cycles":{"nonzero_return":delay_nonzero,"zero_to_sample_reader":delay_empty},"sample_reader_machine_cycles":{"empty_address":reader_empty,"sound_ret":cycles("ReadNoiseSample",&ret_fixture,&[])?,"sample":cycles("ReadNoiseSample",&sample_fixture,&[])?}}
    }});
    model["runtime_source_clock"] = super::source_intro_audio_commands::export(vendor, &model)?;
    model["runtime_source_clock"]["disabled_machine_cycles"] =
        json!(cycles("_UpdateSound", &[], &[])?);
    Ok(model)
}

#[cfg(test)]
mod runtime_source_tests {
    use super::*;
    #[test]
    fn source_audio_runtime_costs_match_independent_engine_fixtures() {
        let Some(vendor) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
            return;
        };
        let vendor = Path::new(&vendor);
        let model = export(vendor).unwrap();
        let symbols =
            symbols(&std::fs::read_to_string(vendor.join("pokecrystal.sym")).unwrap()).unwrap();
        let mut clock = crystal_audio::synth::SourceAudioClock::new().unwrap();
        let root = vendor.parent().unwrap().parent().unwrap();
        let catalog: Vec<crystal_assets::ModpackAudioAsset> = serde_json::from_slice(
            &std::fs::read(root.join("apps/web/assets/data/audio.json")).unwrap(),
        )
        .unwrap();
        let asset = catalog
            .iter()
            .find(|a| a.id == "MUSIC_CRYSTAL_OPENING")
            .unwrap();
        let program =
            crystal_audio::synth::decode_midi(&asset.midi_program.as_ref().unwrap().midi_base64)
                .unwrap();
        clock.play(&program).unwrap();
        for tick in 0..120 {
            let before = clock.channels();
            let steps = clock.update_vblank().unwrap();
            let predicted = crystal_assets::runtime_intro_source_audio_update_machine_cycles(
                &model,
                &steps,
                &before,
                clock.stereo,
            )
            .unwrap();
            let actual = independent_update(
                vendor,
                &symbols,
                &before,
                &steps,
                clock.music_drumkit,
                clock.sfx_drumkit,
            )
            .unwrap();
            assert_eq!(
                predicted, actual,
                "source opening sound-engine cost at update {tick}"
            );
        }
        assert!(symbols.contains_key("ParseMusic"));
    }
    #[test]
    fn source_audio_clock_matches_every_actual_intro_irq_without_reseeding() {
        let Some(vendor) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
            return;
        };
        let vendor = Path::new(&vendor);
        let root = vendor.parent().unwrap().parent().unwrap();
        let trace: Value =
            serde_json::from_slice(&std::fs::read(root.join("intro-trace.json")).unwrap()).unwrap();
        assert!(
            trace["scenes"]
                .as_array()
                .is_some_and(|scenes| scenes.len() == 28),
            "requires complete source intro trace"
        );
        let model = export(vendor).unwrap();
        let certificate =
            super::super::source_intro_audio_clock_certification::certify(vendor, &model, &trace)
                .unwrap();
        assert_eq!(certificate["sound_irqs_compared"], 2442);
        assert_eq!(certificate["playback_callers_compared"], 18);
        assert_eq!(certificate["authored_sfx_shutdowns"], 8);
        eprintln!("Certified evolving source audio clock: {certificate}");
    }

    fn encode_command(command: &crystal_audio::AudioCommand) -> Result<Vec<u8>> {
        let n = |index: usize| -> Result<i32> {
            let text = command
                .args
                .get(index)
                .context("source audio fixture argument")?
                .trim();
            if let Some(hex) = text.strip_prefix('$').or_else(|| text.strip_prefix("0x")) {
                Ok(i32::from_str_radix(hex, 16)?)
            } else {
                Ok(text.parse()?)
            }
        };
        let envelope = |volume: i32, fade: i32| -> u8 {
            (volume as u8) << 4
                | if fade < 0 {
                    8 | (-fade as u8)
                } else {
                    fade as u8
                }
        };
        Ok(match command.command.as_str() {
            "octave" => vec![0xd0 + (8 - n(0)?) as u8],
            "note_type" | "speed" => vec![0xd8, n(0)? as u8, envelope(n(1)?, n(2)?)],
            "drum_speed" => vec![0xd8, n(0)? as u8],
            "tempo" => {
                let value = n(0)? as u16;
                vec![0xda, (value >> 8) as u8, value as u8]
            }
            "duty_cycle" => vec![0xdb, n(0)? as u8],
            "vibrato" => vec![
                0xe1,
                n(0)? as u8,
                if command.args.len() == 2 {
                    n(1)? as u8
                } else {
                    (n(1)? as u8) << 4 | n(2)? as u8
                },
            ],
            "pitch_offset" => {
                let value = n(0)? as u16;
                vec![0xe6, (value >> 8) as u8, value as u8]
            }
            "volume" => vec![0xe5, (n(0)? as u8) << 4 | n(1)? as u8],
            "stereo_panning" | "force_stereo_panning" => vec![
                if command.command == "stereo_panning" {
                    0xef
                } else {
                    0xe4
                },
                (if matches!(command.args[0].as_str(), "TRUE" | "true" | "1") {
                    0xf0
                } else {
                    0
                }) | (if matches!(command.args[1].as_str(), "TRUE" | "true" | "1") {
                    15
                } else {
                    0
                }),
            ],
            "toggle_noise" | "sfx_toggle_noise" => {
                let mut bytes = vec![if command.command == "toggle_noise" {
                    0xe3
                } else {
                    0xf0
                }];
                if !command.args.is_empty() {
                    bytes.push(n(0)? as u8);
                }
                bytes
            }
            "pitch_sweep" | "sweep" => vec![0xdd, envelope(n(0)?, n(1)?)],
            "square_note" | "noise_note" => {
                let frequency = n(3)? as u16;
                let mut bytes = vec![n(0)? as u8, envelope(n(1)?, n(2)?), frequency as u8];
                if command.command == "square_note" {
                    bytes.push((frequency >> 8) as u8);
                }
                bytes
            }
            "note" => {
                let pitch = [
                    "C_", "C#", "D_", "D#", "E_", "F_", "F#", "G_", "G#", "A_", "A#", "B_",
                ]
                .iter()
                .position(|pitch| *pitch == command.args[0])
                .context("source fixture note pitch")? as u8
                    + 1;
                vec![(pitch << 4) | (n(1)? as u8 - 1)]
            }
            "drum_note" => vec![(n(0)? as u8) << 4 | (n(1)? as u8 - 1)],
            "rest" => vec![n(0)? as u8 - 1],
            "sound_ret" => vec![0xff],
            other => anyhow::bail!("source engine fixture encoder needs {other}"),
        })
    }
    fn independent_update(
        vendor: &Path,
        symbols: &BTreeMap<String, (usize, u16)>,
        channels: &[crystal_audio::synth::SourceChannelState],
        steps: &[crystal_audio::synth::SourceChannelStep],
        music_drumkit: i32,
        sfx_drumkit: i32,
    ) -> Result<u64> {
        let base = symbols["wChannel1"].1;
        let stream = symbols["wDecompressScratch"].1;
        let mut writes = vec![
            ("$ff70".to_owned(), 6),
            ("wMusicPlaying".to_owned(), 1),
            ("hCGB".to_owned(), 1),
        ];
        writes.push(("wMusicNoiseSampleSet".into(), music_drumkit as u8));
        writes.push(("wSFXNoiseSampleSet".into(), sfx_drumkit as u8));
        if let Some(step) = steps.first() {
            writes.push(("wNoiseSampleDelay".into(), step.noise_delay_before));
            if let Some(byte) = step.noise_sample_first_byte {
                let pointer = stream + 3000;
                writes.push(("wNoiseSampleAddress".into(), pointer as u8));
                writes.push(("wNoiseSampleAddress+1".into(), (pointer >> 8) as u8));
                for (i, byte) in [byte, 0x77, 0x12, 0xff].iter().enumerate() {
                    writes.push((format!("${:04x}", pointer + i as u16), *byte));
                }
            }
        }
        for state in channels {
            let offset = u16::from(state.channel - 1) * 50;
            for (field, value) in [
                ("Flags1", state.flags1),
                ("Flags2", state.flags2),
                ("Flags3", state.flags3),
                ("NoteFlags", state.note_flags),
                ("DutyCycle", state.duty),
                ("VolumeEnvelope", state.envelope),
                ("Pitch", state.pitch),
                ("Octave", state.octave),
                ("Transposition", state.transpose),
                ("NoteDuration", state.duration),
                ("NoteDurationModifier", state.duration_modifier),
                ("LoopCount", state.loop_count),
                ("NoteLength", state.note_length),
                ("Tracks", state.tracks),
                ("DutyCyclePattern", state.duty_pattern),
                ("VibratoDelayCount", state.vibrato_delay_count),
                ("VibratoDelay", state.vibrato_delay),
                ("VibratoExtent", state.vibrato_extent),
                ("VibratoRate", state.vibrato_rate),
            ] {
                let address = symbols[&format!("wChannel1{field}")].1 + offset;
                writes.push((format!("${address:04x}"), value));
            }
            for (field, value) in [
                ("Frequency", state.frequency),
                ("Tempo", state.tempo),
                ("PitchOffset", state.pitch_offset),
            ] {
                let address = symbols[&format!("wChannel1{field}")].1 + offset;
                writes.push((format!("${address:04x}"), value as u8));
                writes.push((format!("${:04x}", address + 1), (value >> 8) as u8));
            }
            let pointer = stream + u16::from(state.channel - 1) * 256;
            let address = symbols["wChannel1MusicAddress"].1 + offset;
            writes.push((format!("${address:04x}"), pointer as u8));
            writes.push((format!("${:04x}", address + 1), (pointer >> 8) as u8));
            if let Some(step) = steps
                .iter()
                .find(|step| step.before.channel == state.channel)
            {
                let mut bytes = Vec::new();
                for command in &step.commands {
                    bytes.extend(encode_command(command)?);
                }
                bytes.push(0xff);
                ensure!(
                    bytes.len() < 256,
                    "independent source fixture stream overflow"
                );
                for (i, byte) in bytes.iter().enumerate() {
                    writes.push((format!("${:04x}", pointer + i as u16), *byte));
                }
            }
        }
        let measured = measure_routine_state(vendor, "_UpdateSound", &writes, &[])?;
        let _ = base;
        measured["machine_cycles"]
            .as_u64()
            .context("independent sound-engine fixture cost")
    }
}
