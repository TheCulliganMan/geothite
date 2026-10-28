//! Publish original Minnesota voices while preserving supplied game content.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use crystal_audio::original_cry::CryDesignBook;
use crystal_audio::synth::{self, SynthContext};

use crate::minnesota_pokemon_profiles::{self, Roster};
use crate::{CompiledGamePack, ModpackAudioAsset, ModpackAudioKind, ModpackAudioManifest};
use crate::{
    ModpackAudioSource, ModpackMidiAudioProgram, ModpackPcmAudioFormat, PokemonCryMetadata,
};

/// Install the exact active roster's original scores from `cry-designs.json`.
///
/// Returns errors for missing/extra family or species designs, missing base
/// species, invalid synthesis, duplicate PCM, or a non-MIDI base pack. Existing
/// source audio and all unrelated game data remain unchanged. Reapplying the
/// same book is idempotent; revised authored voices replace only their own IDs.
pub fn build_minnesota_cries(base: &CompiledGamePack, root: &Path) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.audio_compression.as_deref() == Some(crate::PACK_AUDIO_COMPRESSION_MIDI),
        "cry publication requires a MIDI pack"
    );
    let mut roster: Roster = serde_json::from_slice(&fs::read(root.join("roster.json"))?)?;
    let mut profiles =
        minnesota_pokemon_profiles::load_profiles(&root.join("species-profiles.json"))?;
    crate::minnesota_pokemon_curation::apply(root, &mut roster, &mut profiles)?;
    let book: CryDesignBook = serde_json::from_slice(&fs::read(root.join("cry-designs.json"))?)?;
    let programs = book.programs()?;
    ensure!(
        roster
            .families
            .iter()
            .map(|family| &family.id)
            .collect::<BTreeSet<_>>()
            == book.families.keys().collect(),
        "cry family coverage differs from active roster"
    );
    let mut pack = base.clone();
    let mut hashes = BTreeSet::new();
    let context = SynthContext::cartridge()?;
    for family in &roster.families {
        let design = &book.families[&family.id];
        ensure!(
            family
                .members
                .iter()
                .map(|member| &member.id)
                .collect::<BTreeSet<_>>()
                == design.species.keys().collect(),
            "cry species coverage differs for {}",
            family.id
        );
        for member in &family.members {
            let token = member.name.to_ascii_uppercase();
            ensure!(
                pack.data.pokemon.contains_key(&token),
                "base pack lacks {token}"
            );
            let id = format!("CRY_{}", member.id);
            let variant = &design.species[&member.id];
            let program = &programs[&member.id];
            let rendered = synth::render(program, context.clone())
                .with_context(|| format!("render {token}"))?;
            ensure!(rendered.loop_samples.is_empty(), "cry {token} loops");
            let pcm = rendered.downsample();
            let frames = pcm.len() / 2;
            ensure!(
                (3307..=99225).contains(&frames),
                "cry {token} duration outside 0.15..=4.5s"
            );
            ensure!(
                pcm.iter().any(|sample| *sample != 0),
                "cry {token} is silent"
            );
            ensure!(
                pcm.iter().all(|sample| i32::from(*sample).abs() <= 24576),
                "cry {token} lacks output headroom"
            );
            ensure!(
                pcm[pcm.len() - 2..]
                    .iter()
                    .all(|sample| i32::from(*sample).abs() <= 16),
                "cry {token} has an abrupt output endpoint"
            );
            let hash = pcm
                .iter()
                .flat_map(|sample| sample.to_le_bytes())
                .fold(2166136261u32, |hash, byte| {
                    (hash ^ u32::from(byte)).wrapping_mul(16777619)
                });
            ensure!(hashes.insert(hash), "duplicate original cry {token}");
            let asset = ModpackAudioAsset {
                id: id.clone(),
                path: format!("audio/cries/{id}.mid"),
                kind: ModpackAudioKind::Cry,
                source: ModpackAudioSource::Midi,
                sfx_priority: None,
                pcm_format: Some(ModpackPcmAudioFormat {
                    sample_rate_hz: 22050,
                    channels: 2,
                    bits_per_sample: 16,
                }),
                pcm_frame_count: Some(frames),
                payload_hash: Some(format!("{hash:08x}")),
                loop_start_sample: None,
                loop_end_sample: None,
                midi_program: Some(ModpackMidiAudioProgram {
                    profile: program.profile.clone(),
                    midi_base64: synth::encode_midi(program)?,
                }),
            };
            asset.validate()?;
            if let Some(index) = pack
                .data
                .audio
                .iter()
                .position(|existing| existing.id == id)
            {
                pack.data.audio[index] = asset;
            } else {
                pack.data.audio.push(asset);
            }
            pack.data.pokemon_cries.insert(
                token,
                PokemonCryMetadata {
                    cry: id,
                    pitch: variant.pitch,
                    length: i16::try_from(variant.length)?,
                },
            );
        }
    }
    pack.audio_manifest = ModpackAudioManifest::from_assets(&pack.data.audio, &BTreeMap::new())?;
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}
