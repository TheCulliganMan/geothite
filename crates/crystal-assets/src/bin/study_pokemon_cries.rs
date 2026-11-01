//! Inspect supplied cry programs in memory without exporting command streams.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use crystal_assets::{self, ModpackAudioKind};
use crystal_audio::synth::{self, SynthContext};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let path = args.next().context(
        "usage: study_pokemon_cries PACK [ALL|CRY_ID_OR_SPECIES [EXTERNAL_WAV_DIRECTORY]]",
    )?;
    let selected = args.next();
    let all = selected.as_deref() == Some("ALL");
    let export = args.next().map(PathBuf::from);
    ensure!(args.next().is_none(), "too many arguments");
    if let Some(directory) = &export {
        ensure!(
            selected.is_some() && !all,
            "select one cry or species for an export"
        );
        fs::create_dir_all(directory)?;
        let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?;
        ensure!(
            !directory.canonicalize()?.starts_with(repository),
            "listening WAVs must stay outside the repository"
        );
    }
    let pack = crystal_assets::read_verified_compiled_game_pack(path)?;
    let species = selected
        .as_deref()
        .and_then(|name| pack.data().pokemon_cries.get(name));
    let matching_id = species
        .map(|cry| cry.cry.as_str())
        .or(selected.as_deref().filter(|_| !all));
    let mut count = 0;
    let mut channel_sets = BTreeMap::new();
    let mut active_channel_sets = BTreeMap::new();
    let mut operations = BTreeMap::new();
    let mut min_frames = usize::MAX;
    let mut max_frames = 0;
    for asset in pack.data().audio.iter().filter(|asset| {
        asset.kind == ModpackAudioKind::Cry && matching_id.is_none_or(|id| id == asset.id)
    }) {
        let program = synth::decode_midi(
            &asset
                .midi_program
                .as_ref()
                .context("cry program missing")?
                .midi_base64,
        )?;
        let rendered = synth::render(&program, SynthContext::cartridge()?)?;
        let pcm = rendered.downsample();
        let frames = pcm.len() / 2;
        let hash = pcm
            .iter()
            .flat_map(|sample| sample.to_le_bytes())
            .fold(2166136261u32, |hash, byte| {
                (hash ^ u32::from(byte)).wrapping_mul(16777619)
            });
        ensure!(
            asset.payload_hash.as_deref() == Some(&format!("{hash:08x}")),
            "{} PCM hash mismatch",
            asset.id
        );
        ensure!(
            asset.pcm_frame_count == Some(frames),
            "{} frame count mismatch",
            asset.id
        );
        ensure!(
            rendered.loop_samples.is_empty()
                && asset.loop_start_sample.is_none()
                && asset.loop_end_sample.is_none(),
            "{} cry loops",
            asset.id
        );
        let mut numbers = program
            .music_data
            .channels
            .values()
            .filter_map(|channel| channel.number)
            .collect::<Vec<_>>();
        numbers.sort();
        *channel_sets.entry(numbers).or_insert(0) += 1;
        let active = rendered
            .channel_nonzero
            .iter()
            .filter(|(_, count)| **count > 0)
            .map(|(number, _)| *number)
            .collect::<Vec<_>>();
        *active_channel_sets.entry(active).or_insert(0) += 1;
        if selected.is_some() {
            println!(
                "{}: {:.3}s; synchronized render frames {:?}",
                asset.id,
                frames as f64 / 22050.0,
                rendered.channel_frames
            );
            summarize_gestures(&program)?;
            if let Some(cry) = species {
                let mut variant = program.clone();
                variant.cry_pitch = Some(i64::from(cry.pitch));
                variant.cry_length = Some(i64::from(cry.length));
                let effective = synth::render(&variant, SynthContext::cartridge()?)?;
                println!(
                    "  effective species registers pitch={}, length={}; duration {:.3}s; channel frames {:?}",
                    cry.pitch,
                    cry.length,
                    effective.downsample().len() as f64 / 44100.0,
                    effective.channel_frames
                );
                summarize_gestures(&variant)?;
                if let Some(directory) = &export {
                    export_listening_wav(
                        directory,
                        selected.as_deref().context("selected species")?,
                        &effective.downsample(),
                    )?;
                }
            } else if let Some(directory) = &export {
                export_listening_wav(directory, &asset.id, &pcm)?;
            }
            let users = pack
                .data()
                .pokemon_cries
                .iter()
                .filter(|(_, cry)| cry.cry == asset.id)
                .map(|(species, cry)| {
                    format!("{species} (pitch {}, length {})", cry.pitch, cry.length)
                })
                .collect::<Vec<_>>();
            println!("species variants: {}", users.join(", "));
            if all {
                for (name, cry) in pack
                    .data()
                    .pokemon_cries
                    .iter()
                    .filter(|(_, cry)| cry.cry == asset.id)
                {
                    let mut variant = program.clone();
                    variant.cry_pitch = Some(i64::from(cry.pitch));
                    variant.cry_length = Some(i64::from(cry.length));
                    let effective = synth::render(&variant, SynthContext::cartridge()?)?;
                    let samples = effective.downsample();
                    let onset = samples
                        .as_chunks::<2>()
                        .0
                        .iter()
                        .position(|frame| frame.iter().any(|v| i32::from(*v).abs() >= 512));
                    println!(
                        "  species {name}: pitch {}, length {}, {:.3}s, onset {:?}ms",
                        cry.pitch,
                        cry.length,
                        samples.len() as f64 / 44100.0,
                        onset.map(|frame| frame as f64 / 22.05)
                    );
                    summarize_gestures(&variant)?;
                }
            }
        }
        for channel in program.music_data.channels.values() {
            for command in &channel.commands {
                *operations.entry(command.command.clone()).or_insert(0) += 1;
            }
        }
        min_frames = min_frames.min(frames);
        max_frames = max_frames.max(frames);
        count += 1;
    }
    ensure!(count > 0, "no matching cries");
    println!("{count} cry programs; every PCM hash, frame count and finite loop range verified");
    println!(
        "base-program duration: {:.3}..={:.3} seconds",
        min_frames as f64 / 22050.0,
        max_frames as f64 / 22050.0
    );
    println!("channel sets (program counts): {channel_sets:?}");
    println!("channels with nonzero kernel output (program counts): {active_channel_sets:?}");
    println!("top-level instruction counts (not expanded calls): {operations:?}");
    println!(
        "{} species register records in the supplied pack",
        pack.data().pokemon_cries.len()
    );
    Ok(())
}

fn export_listening_wav(directory: &Path, name: &str, samples: &[i16]) -> Result<()> {
    let size = u32::try_from(samples.len().checked_mul(2).context("PCM size overflow")?)?;
    let mut bytes = Vec::with_capacity(samples.len() * 2 + 44);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(
        &size
            .checked_add(36)
            .context("WAV size overflow")?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&2_u16.to_le_bytes());
    bytes.extend_from_slice(&22050_u32.to_le_bytes());
    bytes.extend_from_slice(&88200_u32.to_le_bytes());
    bytes.extend_from_slice(&4_u16.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&size.to_le_bytes());
    bytes.extend(samples.iter().flat_map(|sample| sample.to_le_bytes()));
    let path = directory.join(format!("{name}.wav"));
    fs::write(&path, bytes)?;
    println!("listening WAV: {}", path.display());
    Ok(())
}

#[derive(Default)]
struct GestureSummary {
    attacks: BTreeSet<usize>,
    voiced: usize,
    silent: usize,
    pitches: BTreeSet<u16>,
    duties: BTreeSet<u8>,
    fades: BTreeMap<i32, usize>,
    first: Option<usize>,
    last: usize,
    durations: Vec<usize>,
}

fn summarize_gestures(program: &synth::Program) -> Result<()> {
    let mut clock = synth::SourceAudioClock::new()?;
    clock.play(program)?;
    let mut channels: BTreeMap<u8, GestureSummary> = BTreeMap::new();
    for frame in 0..10000 {
        if clock
            .channels()
            .iter()
            .all(|channel| channel.flags1 & 1 == 0)
        {
            break;
        }
        ensure!(frame < 9999, "cry failed to terminate in source clock");
        for step in clock.update_vblank()? {
            let summary = channels.entry(step.after.channel).or_default();
            for command in &step.commands {
                if matches!(command.command.as_str(), "square_note" | "noise_note") {
                    summary.attacks.insert(frame);
                    summary.first.get_or_insert(frame);
                    let duration: usize = command.args[0].parse()?;
                    summary.durations.push(duration + 1);
                    let fade: i32 = command.args[2].parse()?;
                    *summary.fades.entry(fade).or_default() += 1;
                }
            }
            if step.after.flags1 & 1 != 0 {
                summary.last = frame;
                if step.after.envelope >> 4 > 0 {
                    summary.voiced += 1;
                    summary.pitches.insert(step.after.frequency);
                    summary.duties.insert(step.after.duty >> 6);
                } else {
                    summary.silent += 1;
                }
            }
        }
    }
    for (number, summary) in &channels {
        let mut durations = summary.durations.clone();
        durations.sort();
        let median = durations.get(durations.len() / 2).copied().unwrap_or(0);
        let first = summary
            .first
            .map_or_else(|| "never".into(), |frame| format!("{frame}f"));
        let range = summary.pitches.first().zip(summary.pitches.last());
        println!(
            "  ch{number}: {} retriggers; starts {first}, ends {}f; {} nonzero-initial-envelope / {} zero-initial-envelope frames; {} distinct register pitches, range {range:?}; duties {:?}; median encoded duration {median}f; fade counts {:?}",
            summary.attacks.len(),
            summary.last,
            summary.voiced,
            summary.silent,
            summary.pitches.len(),
            summary.duties,
            summary.fades
        );
    }
    if let (Some(a), Some(b)) = (channels.get(&5), channels.get(&6)) {
        println!(
            "  pulse onset alignment: {} shared / {} total distinct onset frames",
            a.attacks.intersection(&b.attacks).count(),
            a.attacks.union(&b.attacks).count()
        );
    }
    Ok(())
}
