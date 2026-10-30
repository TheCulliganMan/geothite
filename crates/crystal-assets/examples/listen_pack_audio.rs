//! Export verified playback references from an existing pack, outside the repo.
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::read_verified_compiled_game_pack;
use crystal_audio::AudioPcmFormat;
use crystal_audio::pcm::decode_midi_pcm;
use crystal_audio::wav::write_canonical_wav;

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let pack = PathBuf::from(
        args.next()
            .context("usage: listen_pack_audio PACK EXTERNAL_OUTPUT_DIRECTORY AUDIO_ID...")?,
    );
    let directory = PathBuf::from(args.next().context("external output directory")?);
    let ids = args.collect::<Vec<_>>();
    ensure!(!ids.is_empty(), "specify at least one existing audio ID");
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    fs::create_dir_all(&directory)?;
    ensure!(
        !directory.canonicalize()?.starts_with(repository),
        "WAV exports must stay outside the repository"
    );
    let pack = read_verified_compiled_game_pack(pack)?;
    for id in ids {
        let id = id.to_str().context("audio ID must be Unicode")?;
        let asset = pack
            .data()
            .audio
            .iter()
            .find(|asset| asset.id == id)
            .with_context(|| format!("pack has no audio program {id}"))?;
        let output = directory.join(format!("{id}.wav"));
        ensure!(!output.exists(), "preserve existing listening exports");
        let frames = asset.pcm_frame_count.context("canonical frame count")?;
        let pcm = decode_midi_pcm(
            &asset
                .midi_program
                .as_ref()
                .context("pack MIDI program")?
                .midi_base64,
            AudioPcmFormat {
                sample_rate_hz: 22_050,
                channels: 2,
                bits_per_sample: 16,
            },
            frames.checked_mul(4).context("PCM byte count overflow")?,
            asset
                .payload_hash
                .as_deref()
                .context("canonical PCM hash")?,
            asset.loop_start_sample,
            asset.loop_end_sample,
        )?;
        write_canonical_wav(&output, &pcm.samples)?;
        println!("Verified {id}: {frames} frames; {}", output.display());
    }
    Ok(())
}
