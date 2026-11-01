//! Restore existing warm coastal art without changing the Cedar approach's play.
use anyhow::{Context, Result, bail, ensure};

use crate::{CompiledGamePack, TilesetDefinition};

const MAP: &str = "MnMaraisCedarApproach";
const COLD: &str = "mn_winter_north_shore";
const WARM: &str = "mn_north_shore";
const MANIFEST: &str = "minnesota-marais-cedar-warmth-v1";

/// Restores the compact Grand Marais approach's existing warm coastal artwork.
///
/// Replaces its private snow floor, grass, sign and canopy with the original
/// coastal blocks. Every used collision quadrant and the border must agree.
/// Only that map's blocks, tileset/border attributes and their native mirrors,
/// a provenance manifest and the derived identity change. Shared artwork and
/// approved inland winter maps remain untouched. This does not write a pack or
/// establish controller, progression or visual acceptance.
///
/// # Errors
/// Rejects invalid packs, an unexpected source map/alias/extent, shared or stale
/// payloads, scripted block changes, unknown snow blocks, collision changes,
/// missing warm art, invalid current geography or final pack validation failure.
/// The borrowed parent remains unchanged on both success and failure.
pub fn restore_coastal_cedar_warmth(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let old = base.data.maps.get(MAP).context("current Cedar approach")?;
    ensure!(
        old.attributes.tileset_name == COLD
            && [old.attributes.width, old.attributes.height] == [20, 12]
            && old.attributes.border_block == 0x8b,
        "require the current compact snowy Cedar approach"
    );
    ensure!(
        old.script_block_changes.is_empty(),
        "unexpected Cedar block-change programs"
    );
    ensure!(
        base.data.map_attributes.get(MAP) == Some(&old.attributes),
        "Cedar attribute mirror differs"
    );
    let label = old
        .attributes
        .blocks_label
        .as_ref()
        .context("Cedar block label")?;
    ensure!(
        base.data.maps.iter().all(|(name, module)| {
            name == MAP || module.attributes.blocks_label.as_ref() != Some(label)
        }),
        "Cedar block payload is shared"
    );
    let encoded = base
        .data
        .map_blocks
        .get(label)
        .context("Cedar encoded blocks")?;
    ensure!(
        crate::decode_base64_bytes(encoded)?
            .into_iter()
            .map(u16::from)
            .eq(old.blocks.iter().copied()),
        "Cedar cached and encoded blocks differ"
    );
    let cold = base
        .data
        .tilesets
        .get(COLD)
        .context("installed winter definition")?;
    let warm = base
        .data
        .tilesets
        .get(WARM)
        .context("existing warm coast definition")?;
    let mut revised = old.clone();
    for block in &mut revised.blocks {
        *block = checked_warm_block(*block, cold, warm)?;
    }
    revised.attributes.border_block = u8::try_from(checked_warm_block(
        u16::from(old.attributes.border_block),
        cold,
        warm,
    )?)
    .context("warm Cedar border exceeds native byte range")?;
    revised.attributes.tileset_name = WARM.into();
    let bytes = revised
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Cedar block exceeds native byte range")?;
    let mut output = base.clone();
    output
        .data
        .map_attributes
        .insert(MAP.into(), revised.attributes.clone());
    output
        .data
        .map_blocks
        .insert(label.clone(), crate::encode_base64_bytes(&bytes));
    output.data.maps.insert(MAP.into(), revised);
    ensure!(
        !output.report.manifests.iter().any(|name| name == MANIFEST),
        "Cedar warmth already recorded"
    );
    output.report.manifests.push(MANIFEST.into());
    output.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        output.format_version,
        &output.data,
        &output.audio_manifest,
        &output.runtime_files,
        &output.report,
    )?;
    // The parent is intentionally the snowy map being repaired. Validate the
    // complete geography after the collision-preserving warm replacement.
    crate::validate_compact_superior_geography(&output)?;
    crate::verify_compiled_game_pack_for_runtime(&output)?;
    Ok(output)
}

fn checked_warm_block(
    block: u16,
    cold: &TilesetDefinition,
    warm: &TilesetDefinition,
) -> Result<u16> {
    let replacement = warm_block(block)?;
    let before = cold
        .collision
        .get(&format!("{block:02x}"))
        .context("old Cedar collision")?;
    let after = warm
        .collision
        .get(&format!("{replacement:02x}"))
        .context("warm Cedar collision")?;
    ensure!(
        before == after,
        "Cedar collision would change at block {block:02x}"
    );
    ensure!(
        after.iter().all(|kind| kind != "ICE"),
        "Cedar cannot contain ICE"
    );
    Ok(replacement)
}

fn warm_block(block: u16) -> Result<u16> {
    match block {
        0x83 => Ok(0x01),
        0x84 => Ok(0x03),
        0x88 => Ok(0x45),
        0x8b => Ok(0x05),
        0..=0x82 => Ok(block),
        _ => bail!("unreviewed Cedar winter block {block:02x}"),
    }
}

#[cfg(test)]
#[path = "coastal_cedar_warmth_tests.rs"]
mod tests;
