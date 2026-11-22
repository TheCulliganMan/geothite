//! Native battle-only source encoding, without invented overworld frames.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Result, ensure};
use serde::Deserialize;

use crate::minnesota_pokemon_pixel_art;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NativeBattlePortrait {
    pub schema_version: u8,
    pub id: String,
    pub palette_rgb555: [u16; 4],
    pub battle_rows: Vec<String>,
}

/// Encode version-one battle-only native JSON into PNG, 2bpp and RGB555 files.
///
/// # Errors
/// Rejects extra fields, invalid identities/palettes, empty or non-56x56 grids,
/// and symbols other than `.123`. No field-sprite resources are generated.
pub fn encode_trainer_battle_native(source: &str) -> Result<BTreeMap<String, Vec<u8>>> {
    let source: NativeBattlePortrait = serde_json::from_str(source)?;
    encode(&source)
}

pub(crate) fn encode(source: &NativeBattlePortrait) -> Result<BTreeMap<String, Vec<u8>>> {
    ensure!(
        source.schema_version == 1
            && crate::trainer_portrait_overrides::valid_portrait_id(&source.id),
        "version-one mn_portrait_ identity required"
    );
    let palette = source.palette_rgb555;
    ensure!(
        palette[0] == 32767
            && palette[3] == 0
            && palette.iter().all(|ink| *ink <= 32767)
            && palette.iter().collect::<BTreeSet<_>>().len() == 4,
        "four distinct RGB555 inks required"
    );
    ensure!(
        source.battle_rows.len() == 56 && source.battle_rows.iter().all(|row| row.len() == 56),
        "battle portrait must contain 56 rows of 56 pixels"
    );
    let pixels = source
        .battle_rows
        .iter()
        .flat_map(|row| row.bytes())
        .map(|symbol| match symbol {
            b'.' => Ok(0),
            b'1'..=b'3' => Ok(symbol - b'0'),
            _ => anyhow::bail!("invalid native palette symbol"),
        })
        .collect::<Result<Vec<_>>>()?;
    ensure!(
        pixels.iter().any(|pixel| *pixel != 0),
        "empty battle portrait"
    );
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
            palette.iter().flat_map(|ink| ink.to_le_bytes()).collect(),
        ),
    ]))
}
