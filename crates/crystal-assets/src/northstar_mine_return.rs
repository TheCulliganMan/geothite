//! A lower return drift beneath the existing eastern mine bench.
use anyhow::{Context, Result, ensure};
use base64::Engine;
use serde::Deserialize;

use crate::CompiledGamePack;

const PLAN: &str =
    include_str!("../../../planning/minnesota/caves/r09-return-drift-v01/layout.json");

#[derive(Deserialize)]
struct Plan {
    map: String,
    dimensions: [u16; 2],
    changes: Vec<[usize; 4]>,
    rows: Vec<String>,
}

/// Installs the terrain-only return drift, preserving every event and program.
///
/// # Errors
/// Rejects a changed terrain preimage, dimensions, or block payload. The caller
/// owns the combined release identity and saved-terrain migration.
pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let plan: Plan = serde_json::from_str(PLAN)?;
    let map = pack.data.maps.get(&plan.map).context("R09 mine")?;
    ensure!(
        [map.attributes.width, map.attributes.height] == plan.dimensions,
        "mine dimensions changed"
    );
    let replacement = plan
        .rows
        .iter()
        .flat_map(|row| row.split_whitespace())
        .map(|word| u16::from_str_radix(word, 16))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut preimage = replacement.clone();
    for &[x, y, old, new] in &plan.changes {
        let index = y * usize::from(plan.dimensions[0]) + x;
        ensure!(
            replacement.get(index) == Some(&u16::try_from(new)?),
            "mine plan mismatch"
        );
        *preimage.get_mut(index).context("mine edit extent")? = u16::try_from(old)?;
    }
    ensure!(map.blocks == preimage, "mine terrain preimage changed");
    let label = map
        .attributes
        .blocks_label
        .clone()
        .context("mine blocks label")?;
    let bytes = base64::engine::general_purpose::STANDARD.decode(
        pack.data
            .map_blocks
            .get(&label)
            .context("mine block payload")?,
    )?;
    ensure!(
        bytes.iter().copied().map(u16::from).eq(preimage),
        "mine split blocks differ"
    );
    let bytes = replacement
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    pack.data.map_blocks.insert(
        label,
        base64::engine::general_purpose::STANDARD.encode(bytes),
    );
    pack.data.maps.get_mut(&plan.map).context("mine")?.blocks = replacement;
    Ok(())
}

/// Builds an unselected mine-drift review pack with a fresh content identity.
///
/// # Errors
/// Rejects incompatible terrain or an invalid resulting compiled pack.
pub fn build_mine_return(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    let mut pack = base.clone();
    install(&mut pack)?;
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
