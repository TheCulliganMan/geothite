//! Compact source-engine parser and hardware-write cost tables for live audio.
#[path = "source_intro_audio_callers.rs"]
mod source_intro_audio_callers;
use super::source_intro_oracle::{measure_routine_state, symbols};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

pub(super) fn export(vendor: &Path, base: &Value) -> Result<Value> {
    let symbols = symbols(&std::fs::read_to_string(vendor.join("pokecrystal.sym"))?)?;
    let address = |name: &str| -> Result<u16> {
        Ok(symbols
            .get(name)
            .with_context(|| format!("missing source sound symbol {name}"))?
            .1)
    };
    let channel = address("wChannel1")?;
    let stream = address("wDecompressScratch")?;
    let target = stream + 32;
    let measure =
        |label: &str, slot: u8, flags: u8, bytes: &[u8], extra: &[(&str, u8)]| -> Result<Value> {
            let mut writes = vec![
                ("$ff70".to_string(), 6),
                ("hCGB".into(), 1),
                ("wCurChannel".into(), slot),
                ("wChannel1Flags1".into(), flags),
                ("wChannel1NoteLength".into(), 1),
                ("wChannel1Octave".into(), 7),
                ("wChannel1MusicAddress".into(), stream as u8),
                ("wChannel1MusicAddress+1".into(), (stream >> 8) as u8),
                ("wChannel1LastMusicAddress".into(), target as u8),
                ("wChannel1LastMusicAddress+1".into(), (target >> 8) as u8),
            ];
            for (i, byte) in bytes.iter().enumerate() {
                writes.push((format!("wDecompressScratch+{i}"), *byte));
            }
            for (i, byte) in [0x10, 0x77, 0x12, 0x03, 0xff].iter().enumerate() {
                writes.push((format!("wDecompressScratch+{}", 32 + i), *byte));
            }
            writes.extend(extra.iter().map(|(key, value)| (key.to_string(), *value)));
            measure_routine_state(vendor, label, &writes, &[("BC", channel)])
        };
    let duration = base
        .pointer("/helper_cycle_models/parse_music_intro_paths/set_note_duration")
        .context("source duration model")?;
    let c = |v: &Value, key: &str| -> Result<u64> {
        v[key]
            .as_u64()
            .with_context(|| format!("source audio cost {key}"))
    };
    let multiply = |v: u8| -> Result<u64> {
        Ok(c(duration, "multiply_fixed_machine_cycles")?
            + c(duration, "multiply_per_bit_machine_cycles")?
                * u64::from((8 - v.leading_zeros()).max(1))
            + c(duration, "multiply_set_bit_extra_machine_cycles")? * u64::from(v.count_ones()))
    };
    let duration_one = c(duration, "fixed_machine_cycles")? + 2 * multiply(1)?;
    let mut hardware = Vec::new();
    for slot in 0..8 {
        let mut costs = Vec::new();
        for flags in 0..128 {
            costs.push(c(
                &measure(
                    "UpdateChannels",
                    slot,
                    1,
                    &[0xff],
                    &[("wChannel1NoteFlags", flags)],
                )?,
                "machine_cycles",
            )?);
        }
        hardware.push(costs);
    }
    let mut commands = Vec::new();
    let mut terminals = Vec::new();
    for slot in 0..8 {
        let mut paths = BTreeMap::new();
        for (name, bytes, flags, extra) in [
            ("octave", vec![0xd4], 1, vec![]),
            ("note_type", vec![0xd8, 12, 0xa3], 1, vec![]),
            ("transpose", vec![0xd9, 0x11], 1, vec![]),
            ("tempo", vec![0xda, 0x01, 0x00], 1, vec![]),
            ("duty_cycle", vec![0xdb, 2], 1, vec![]),
            ("volume_envelope", vec![0xdc, 0xa3], 1, vec![]),
            ("pitch_sweep", vec![0xdd, 0x22], 1, vec![]),
            ("duty_cycle_pattern", vec![0xde, 0xa3], 1, vec![]),
            ("toggle_sfx_on", vec![0xdf], 1, vec![]),
            ("toggle_sfx_off", vec![0xdf], 9, vec![]),
            ("vibrato", vec![0xe1, 10, 0x23], 1, vec![]),
            ("toggle_noise_on", vec![0xe3, 0], 1, vec![]),
            ("toggle_noise_off", vec![0xe3], 17, vec![]),
            ("force_stereo_panning", vec![0xe4, 0xff], 1, vec![]),
            ("volume", vec![0xe5, 0x77], 1, vec![]),
            ("pitch_offset", vec![0xe6, 0x01, 0x23], 1, vec![]),
            ("sfx_priority_on", vec![0xec], 1, vec![]),
            ("sfx_priority_off", vec![0xed], 1, vec![]),
            (
                "stereo_panning_off",
                vec![0xef, 0xff],
                1,
                vec![("wOptions", 0)],
            ),
            (
                "stereo_panning_on",
                vec![0xef, 0xff],
                1,
                vec![("wOptions", 32)],
            ),
            ("sfx_toggle_noise_on", vec![0xf0, 0], 1, vec![]),
            ("sfx_toggle_noise_off", vec![0xf0], 17, vec![]),
            (
                "sound_jump",
                vec![0xfc, target as u8, (target >> 8) as u8],
                1,
                vec![],
            ),
            (
                "sound_call",
                vec![0xfe, target as u8, (target >> 8) as u8],
                1,
                vec![],
            ),
            ("sound_ret_subroutine", vec![0xff], 3, vec![]),
            (
                "sound_loop_unset_zero",
                vec![0xfd, 0, target as u8, (target >> 8) as u8],
                1,
                vec![],
            ),
            (
                "sound_loop_unset_one",
                vec![0xfd, 1, target as u8, (target >> 8) as u8],
                1,
                vec![],
            ),
            (
                "sound_loop_unset_many",
                vec![0xfd, 3, target as u8, (target >> 8) as u8],
                1,
                vec![],
            ),
            (
                "sound_loop_set_empty",
                vec![0xfd, 3, target as u8, (target >> 8) as u8],
                5,
                vec![("wChannel1LoopCount", 0)],
            ),
            (
                "sound_loop_set_remaining",
                vec![0xfd, 3, target as u8, (target >> 8) as u8],
                5,
                vec![("wChannel1LoopCount", 2)],
            ),
        ] {
            let mut stream_bytes = bytes;
            stream_bytes.extend([0x10, 0x77, 0x12, 0x03, 0xff]);
            let measured = measure("ParseMusic", slot, flags, &stream_bytes, &extra)?;
            let tape = measured["instructions"]
                .as_array()
                .context("source parser tape")?;
            let parser = address("ParseMusic")?;
            let stop = tape
                .iter()
                .enumerate()
                .skip(1)
                .find(|(_, i)| i["address"].as_u64() == Some(u64::from(parser)))
                .map(|(index, _)| index)
                .with_context(|| {
                    format!("command {name} slot {slot} did not return to source parser")
                })?;
            let cost = tape[..stop]
                .iter()
                .map(|i| i["machine_cycles"].as_u64().unwrap())
                .sum::<u64>();
            paths.insert(name, cost);
        }
        commands.push(paths);
        let mut paths = BTreeMap::new();
        for (name, flags, bytes, extra, duration_included, frequency_included) in [
            ("normal_note", 1, vec![0x10], vec![], true, true),
            ("normal_rest", 1, vec![0x00], vec![], true, false),
            (
                "sfx_note",
                9,
                vec![0x00, 0xa3, 0x12, 0x03],
                vec![],
                true,
                false,
            ),
            (
                "cry_note",
                33,
                vec![0x00, 0xa3, 0x12, 0x03],
                vec![],
                true,
                false,
            ),
            ("noise_note", 17, vec![0x10], vec![], slot % 4 == 3, false),
            ("noise_rest", 17, vec![0x00], vec![], slot % 4 == 3, false),
            (
                "noise_note_blocked",
                17,
                vec![0x10],
                vec![("wChannel8Flags1", 1)],
                slot % 4 == 3,
                false,
            ),
            (
                "noise_rest_blocked",
                17,
                vec![0x00],
                vec![("wChannel8Flags1", 1)],
                slot % 4 == 3,
                false,
            ),
            ("end", 1, vec![0xff], vec![], false, false),
            (
                "end_shadowed",
                1,
                vec![0xff],
                vec![("wChannel5Flags1", 1)],
                false,
                false,
            ),
            ("cry_end", 33, vec![0xff], vec![], false, false),
        ] {
            let actual = c(
                &measure("ParseMusic", slot, flags, &bytes, &extra)?,
                "machine_cycles",
            )?;
            let frequency = if frequency_included {
                base.pointer("/helper_cycle_models/parse_music_intro_paths/get_frequency/fixed_machine_cycles").and_then(Value::as_u64).context("source frequency cost")?
            } else {
                0
            };
            let cost = actual
                .checked_sub(if duration_included { duration_one } else { 0 } + frequency)
                .context("source terminal cost underflow")?;
            paths.insert(name, cost);
        }
        terminals.push(paths);
    }
    ensure!(
        commands.len() == 8 && hardware.len() == 8,
        "source audio model lacks channel slots"
    );
    Ok(
        json!({"schema_version":1,"update_channels_machine_cycles":hardware,"parser_command_machine_cycles":commands,"parser_terminal_base_machine_cycles":terminals,"playback":source_intro_audio_callers::export(vendor)?}),
    )
}
