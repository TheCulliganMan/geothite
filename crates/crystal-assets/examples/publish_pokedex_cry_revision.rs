//! Publish current pack cries while preserving a frozen Pokédex's other content.
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use crystal_audio::pcm::{CrySynthesisParameters, ModifiedCryRequest, decode_modified_cry};
use crystal_audio::{AudioPcmFormat, wav};
use serde_json::Value;

fn copy_release(source: &Path, output: &Path, replacements: &[(&str, &str)]) -> Result<()> {
    fs::create_dir_all(output)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let target = output.join(entry.file_name());
        ensure!(
            !entry.file_type()?.is_symlink(),
            "release contains a symlink"
        );
        if entry.file_type()?.is_dir() {
            copy_release(&path, &target, replacements)?;
        } else {
            let bytes = fs::read(&path)?;
            if matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("html" | "json" | "js" | "css" | "svg")
            ) {
                let mut text = String::from_utf8(bytes)?;
                for &(old, new) in replacements {
                    text = text.replace(old, new);
                }
                fs::write(target, text)?;
            } else {
                fs::write(target, bytes)?;
            }
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 6,
        "usage: publish_pokedex_cry_revision PACK BASE_RELEASE OUTPUT ASSET_BASE CRY_BASE EXTERNAL_AUDIO_DIRECTORY"
    );
    let pack = crystal_assets::read_verified_compiled_game_pack(&args[0])?;
    let baseline = Path::new(&args[1]);
    let output = Path::new(&args[2]);
    let asset_base = &args[3];
    let cry_base = &args[4];
    let audio_directory = Path::new(&args[5]);
    ensure!(!output.exists(), "release output already exists");
    ensure!(!audio_directory.exists(), "audio output already exists");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(output)
            .status()?
            .success(),
        "release output must remain ignored"
    );
    for (base, prefix) in [
        (asset_base, "/pokedex/releases/"),
        (cry_base, "/pokedex/media/cries/"),
    ] {
        ensure!(
            base.starts_with(prefix)
                && base[prefix.len()..]
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
                && base.len() > prefix.len(),
            "invalid immutable URL base"
        );
    }
    let old_asset_base = format!(
        "/pokedex/releases/{}",
        baseline
            .file_name()
            .and_then(|name| name.to_str())
            .context("baseline release name")?
    );
    ensure!(asset_base != &old_asset_base, "release URL must be new");
    let index = fs::read_to_string(baseline.join("index.html"))?;
    let old_cry_url = index
        .split("data-cry=\"")
        .nth(1)
        .context("baseline cry control")?
        .split('"')
        .next()
        .context("baseline cry URL")?;
    let old_cry_base = old_cry_url
        .rsplit_once('/')
        .context("baseline cry directory")?
        .0;
    ensure!(cry_base != old_cry_base, "audio URL must be new");
    let mut catalog: Value = serde_json::from_slice(&fs::read(baseline.join("pokedex.json"))?)?;
    let expected = catalog["cry_species_count"].as_u64().context("cry count")? as usize;
    let species = catalog["species"].as_array().context("active species")?;
    ensure!(
        species.len() == expected && expected > 0,
        "incomplete frozen roster"
    );
    ensure!(
        index.matches("data-cry=\"").count() == expected,
        "incomplete index controls"
    );
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    fs::create_dir_all(audio_directory)?;
    ensure!(
        !audio_directory.canonicalize()?.starts_with(repository),
        "WAVs must stay outside the repository"
    );
    for species in species {
        let id = species["identity"]["id"].as_str().context("species ID")?;
        ensure!(
            id.bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_'),
            "unsafe species ID"
        );
        let token = species["identity"]["name"]
            .as_str()
            .context("species name")?
            .to_ascii_uppercase();
        let metadata = pack
            .data()
            .pokemon_cries
            .get(&token)
            .context("pack species cry")?;
        ensure!(
            metadata.cry == format!("CRY_{id}"),
            "pack lacks authored cry for {id}"
        );
        let asset = pack
            .data()
            .audio
            .iter()
            .find(|asset| asset.id == metadata.cry)
            .context("pack cry asset")?;
        ensure!(
            asset.loop_start_sample.is_none() && asset.loop_end_sample.is_none(),
            "cry must be finite"
        );
        let request = ModifiedCryRequest {
            format: AudioPcmFormat {
                sample_rate_hz: 22_050,
                channels: 2,
                bits_per_sample: 16,
            },
            byte_len: asset.pcm_frame_count.context("PCM frame count")? * 4,
            payload_hash: asset.payload_hash.clone().context("PCM hash")?,
            parameters: CrySynthesisParameters {
                pitch: metadata.pitch as u16,
                length: metadata.length as u16,
            },
        };
        let decoded = decode_modified_cry(
            &asset
                .midi_program
                .as_ref()
                .context("cry program")?
                .midi_base64,
            &request,
        )?;
        wav::write_canonical_wav(&audio_directory.join(format!("{id}.wav")), &decoded.samples)?;
    }
    catalog["audio_pack_content_hash"] = serde_json::to_value(pack.identity()?.content_hash)?;
    catalog["cry_audio_base"] = Value::String(cry_base.clone());
    copy_release(
        baseline,
        output,
        &[(&old_asset_base, asset_base), (old_cry_base, cry_base)],
    )?;
    fs::write(
        output.join("pokedex.json"),
        serde_json::to_vec_pretty(&catalog)?,
    )?;
    println!(
        "Exported {expected} runtime-verified cries to {} and preserved the frozen release in {}",
        audio_directory.display(),
        output.display()
    );
    Ok(())
}
