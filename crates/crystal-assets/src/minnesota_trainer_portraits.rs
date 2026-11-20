//! Revise installed battle portraits without touching field sprites or gameplay.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::CompiledGamePack;
use crate::minnesota_pokemon_pixel_art;
use crate::minnesota_sprite_sheet::MinnesotaSheet;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    schema_version: u8,
    portraits: Vec<Revision>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Revision {
    previous_source: PathBuf,
    source: PathBuf,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct NativePortrait {
    schema_version: u8,
    id: String,
    palette_rgb555: [u16; 4],
    battle_rows: Vec<String>,
    overworld_frames: Vec<NativeFrame>,
}

#[derive(Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct NativeFrame {
    pose: String,
    rows: Vec<String>,
}

/// Replace only the named installed Minnesota trainer battle PNG and 2bpp banks.
///
/// `source` is a version-one JSON plan with `portraits` entries containing
/// `previous_source` and `source` native-pixel JSON paths, resolved from the
/// process's working directory. The palette and all six overworld frames must
/// remain unchanged. The previous source must exactly match the installed tile
/// and palette bytes and decoded RGBA pixels. Every other pack field is retained.
///
/// # Errors
/// Returns an error for invalid or duplicate identities, missing input files,
/// stale installed art, invalid native grids, palette or field changes, unchanged
/// portraits, or an invalid parent/result pack. The parent is never modified.
pub fn revise_minnesota_trainer_portraits(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let plan: Plan = serde_json::from_str(source).context("trainer portrait revision plan")?;
    ensure!(
        plan.schema_version == 1 && !plan.portraits.is_empty(),
        "nonempty version-one portrait plan required"
    );
    let mut seen = BTreeSet::new();
    let mut replacements = BTreeMap::new();
    for revision in plan.portraits {
        let previous: NativePortrait = serde_json::from_slice(
            &fs::read(&revision.previous_source)
                .with_context(|| revision.previous_source.display().to_string())?,
        )?;
        let current: NativePortrait = serde_json::from_slice(
            &fs::read(&revision.source).with_context(|| revision.source.display().to_string())?,
        )?;
        ensure!(
            seen.insert(current.id.clone()),
            "duplicate portrait identity"
        );
        let class = current.id.to_ascii_uppercase();
        ensure!(
            base.data.trainer_class_names.contains_key(&class)
                || base
                    .data
                    .sprite_palette_defaults
                    .contains_key(&format!("SPRITE_{class}")),
            "portrait must belong to an installed regional character: {}",
            current.id
        );
        replacements.extend(replacement_files(&base.runtime_files, &previous, &current)?);
    }
    let mut pack = base.clone();
    pack.runtime_files.extend(replacements);
    pack.identity = if pack.audio_compression.is_some() {
        crate::derive_compiled_game_pack_identity_from_manifest(
            pack.format_version,
            &pack.data,
            &pack.audio_manifest,
            &pack.runtime_files,
            &pack.report,
        )?
    } else {
        crate::derive_compiled_game_pack_identity(
            pack.format_version,
            &pack.data,
            &pack.compiled_audio,
            &pack.runtime_files,
            &pack.report,
        )?
    };
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

fn grid(rows: &[String], size: usize) -> Result<Vec<u8>> {
    ensure!(
        rows.len() == size && rows.iter().all(|row| row.len() == size),
        "native grid must be {size} square"
    );
    let pixels = rows
        .iter()
        .flat_map(|row| row.bytes())
        .map(|symbol| match symbol {
            b'.' => Ok(0),
            b'1'..=b'3' => Ok(symbol - b'0'),
            _ => anyhow::bail!("invalid native palette index"),
        })
        .collect::<Result<Vec<_>>>()?;
    ensure!(pixels.iter().any(|pixel| *pixel != 0), "empty native grid");
    Ok(pixels)
}

fn encode(source: &NativePortrait) -> Result<BTreeMap<String, Vec<u8>>> {
    ensure!(
        source.schema_version == 1
            && source.id.starts_with("mn_")
            && source
                .id
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte == b'_'),
        "native Minnesota portrait identity required"
    );
    let palette = source.palette_rgb555;
    ensure!(
        palette[0] == 32767
            && palette[3] == 0
            && palette.iter().all(|color| *color <= 32767)
            && palette.iter().collect::<BTreeSet<_>>().len() == 4,
        "four distinct native RGB555 inks required"
    );
    let poses = [
        "down_idle",
        "up_idle",
        "left_idle",
        "down_step",
        "up_step",
        "left_step",
    ];
    ensure!(
        source.overworld_frames.len() == poses.len(),
        "six field poses required"
    );
    for (frame, pose) in source.overworld_frames.iter().zip(poses) {
        ensure!(frame.pose == pose, "native field pose order changed");
        grid(&frame.rows, 16)?;
    }
    let pixels = grid(&source.battle_rows, 56)?;
    let stem = format!("gfx/trainers/{}", source.id);
    Ok(BTreeMap::from([
        (
            format!("{stem}.png"),
            minnesota_pokemon_pixel_art::png_rgb(
                56,
                56,
                &minnesota_pokemon_pixel_art::colored(&pixels, palette),
            )?,
        ),
        (
            format!("{stem}.2bpp"),
            minnesota_pokemon_pixel_art::bitplanes(&pixels, 56, 56),
        ),
        (
            format!("{stem}.gbcpal"),
            palette
                .iter()
                .flat_map(|color| color.to_le_bytes())
                .collect(),
        ),
    ]))
}

fn matching_png(actual: &[u8], expected: &[u8]) -> Result<bool> {
    let actual = MinnesotaSheet::from_png(actual)?;
    let expected = MinnesotaSheet::from_png(expected)?;
    Ok(actual.width == expected.width
        && actual.height == expected.height
        && (0..expected.height).all(|y| {
            (0..expected.width).all(|x| {
                actual.pixel(x, y) == expected.pixel(x, y)
                    && actual.alpha(x, y) == expected.alpha(x, y)
            })
        }))
}

fn replacement_files(
    installed: &BTreeMap<String, Vec<u8>>,
    previous: &NativePortrait,
    current: &NativePortrait,
) -> Result<BTreeMap<String, Vec<u8>>> {
    ensure!(previous.id == current.id, "portrait identity changed");
    ensure!(
        previous.palette_rgb555 == current.palette_rgb555
            && previous.overworld_frames == current.overworld_frames,
        "battle revision must preserve palette and overworld frames"
    );
    ensure!(
        previous.battle_rows != current.battle_rows,
        "portrait is unchanged"
    );
    let expected = encode(previous)?;
    let replacements = encode(current)?;
    for (key, bytes) in &expected {
        let actual = installed
            .get(key)
            .with_context(|| format!("missing installed portrait resource {key}"))?;
        let matches = if key.ends_with(".png") {
            matching_png(actual, bytes)?
        } else {
            actual == bytes
        };
        ensure!(matches, "stale portrait preimage: {key}");
    }
    Ok(replacements)
}

#[cfg(test)]
#[path = "minnesota_trainer_portraits_tests.rs"]
mod tests;
