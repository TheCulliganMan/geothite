//! Standalone native audio export. No JavaScript, TypeScript, or sibling repository.
use anyhow::{Context, Result, ensure};
use crystal_assets::{ModpackAudioKind, read_verified_compiled_game_pack};
use crystal_audio::synth::{SynthContext, decode_midi, encode_midi, link, render};
use std::{collections::BTreeMap, path::PathBuf};
fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        args.iter().all(|s| s == "--check"),
        "usage: export_audio [--check]"
    );
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let path = root.join("content-packs/core-modular.browser.crystalpack");
    let pack = read_verified_compiled_game_pack(&path)?;
    // Programs are already embedded in the existing bundled content pack.
    // Do not create or require a second disassembly-derived source catalog.
    let programs = pack
        .data()
        .audio
        .iter()
        .map(|asset| {
            let midi = asset
                .midi_program
                .as_ref()
                .context("audio program missing")?;
            Ok((asset.id.clone(), decode_midi(&midi.midi_base64)?))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    let mut catalog = BTreeMap::new();
    for p in programs.values() {
        catalog.extend(p.music_data.subroutines.clone());
        catalog.extend(p.music_data.shared_sources.clone());
    }
    for p in programs.values() {
        catalog.extend(p.music_data.channels.clone());
    }
    let context = SynthContext::cartridge()?;
    let mut assets = pack.data().audio.clone();
    let mut changed = 0;
    for asset in &mut assets {
        let input = programs
            .get(&asset.id)
            .with_context(|| format!("missing source {}", asset.id))?;
        let program = link(input, &catalog).with_context(|| format!("link {}", asset.id))?;
        let rendered =
            render(&program, context.clone()).with_context(|| format!("render {}", asset.id))?;
        let pcm = rendered.downsample();
        ensure!(!pcm.is_empty(), "{} rendered no PCM", asset.id);
        let mut hash = 2166136261_u32;
        for byte in pcm.iter().flat_map(|s| s.to_le_bytes()) {
            hash = (hash ^ u32::from(byte)).wrapping_mul(16777619);
        }
        let hash = format!("{hash:08x}");
        let frames = pcm.len() / 2;
        let primary = program
            .music_data
            .channels
            .values()
            .filter_map(|c| c.number)
            .min()
            .context("no audio channels")?;
        let loop_start = if matches!(asset.kind, ModpackAudioKind::Music) {
            rendered.loop_samples.get(&primary).map(|s| s.div_ceil(2))
        } else {
            None
        };
        if matches!(asset.kind, ModpackAudioKind::Music) && !rendered.loop_samples.is_empty() {
            ensure!(
                loop_start.is_some(),
                "{} primary channel never loops",
                asset.id
            );
            ensure!(
                rendered.channel_nonzero.values().all(|n| *n > 0),
                "{} has a silent music channel",
                asset.id
            );
        }
        let old_program = decode_midi(
            &asset
                .midi_program
                .as_ref()
                .context("missing MIDI")?
                .midi_base64,
        )?;
        let modified = asset.payload_hash.as_ref() != Some(&hash)
            || asset.pcm_frame_count != Some(frames)
            || asset.loop_start_sample != loop_start
            || asset.loop_end_sample != loop_start.map(|_| frames)
            || old_program != program;
        if modified {
            changed += 1;
            println!(
                "rebuild {}: {} -> {hash}, loop {:?} -> {:?}",
                asset.id,
                asset.payload_hash.as_deref().unwrap_or("missing"),
                asset.loop_start_sample,
                loop_start
            );
        }
        asset.payload_hash = Some(hash.clone());
        asset.pcm_frame_count = Some(frames);
        asset.loop_start_sample = loop_start;
        asset.loop_end_sample = loop_start.map(|_| frames);
        if old_program != program {
            asset.midi_program.as_mut().unwrap().midi_base64 = encode_midi(&program)?;
        }
    }
    let rebuilt = pack.with_rebuilt_browser_audio(assets)?;
    if args.iter().any(|s| s == "--check") {
        ensure!(changed == 0, "{changed} audio assets need export");
    } else {
        rebuilt.write_preserving_storage(&path)?;
    }
    println!(
        "verified {} audio assets; {changed} changed",
        programs.len()
    );
    Ok(())
}
