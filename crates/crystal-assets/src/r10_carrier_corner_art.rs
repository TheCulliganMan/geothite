//! Native inward north-rim corner for R10's continuous western carrier ridge.

use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;

/// Appends private85 without changing terrain, existing art or unrelated content.
///
/// Three quadrants reuse native71's brown floor; the upper-right reuses
/// native69's vertical face. The face ends at the adjacent north rim's height,
/// without adding another rim underneath it. Original glyphs, palettes and
/// foregrounds remain exact. Collision follows those native quadrants:
/// FLOOR, WALL, FLOOR, FLOOR.
/// Returns a fresh pack identity and writes no files.
///
/// # Errors
/// Rejects stale geography, shared art, changed native source collision or
/// extent, an already installed join, or failed runtime verification.
pub fn add_r10_carrier_corner_art(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    const TILESET: &str = "mn_r10_rockstairs";
    const MANIFEST: &str = "minnesota-r10-carrier-corner-art-v2";
    crate::validate_compact_superior_geography(base)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|id| id == MANIFEST),
        "R10 carrier corner is already installed"
    );
    let owners = base
        .data
        .maps
        .iter()
        .filter(|(_, map)| map.attributes.tileset_name == TILESET)
        .map(|(name, _)| name.as_str())
        .collect::<BTreeSet<_>>();
    ensure!(
        owners == BTreeSet::from(["MnR10C00R00", "MnR10C01R00"]),
        "R10 carrier art must remain private to its two chunks"
    );
    let definition = base.data.tilesets.get(TILESET).context("R10 native art")?;
    for (block, kinds) in [
        ("71", ["FLOOR"; 4]),
        ("69", ["FLOOR", "WALL", "FLOOR", "WALL"]),
        ("70", ["UP_WALL", "UP_WALL", "FLOOR", "FLOOR"]),
    ] {
        ensure!(
            definition
                .collision
                .get(block)
                .is_some_and(|actual| actual.iter().map(String::as_str).eq(kinds)),
            "R10 carrier source collision differs: {block}"
        );
    }
    ensure!(
        !definition.collision.contains_key("85"),
        "R10 carrier destination already exists"
    );
    let path = format!("data/tilesets/{TILESET}_metatiles.bin");
    let raw = base.runtime_files.get(&path).context("R10 metatiles")?;
    ensure!(
        raw.len() == 0x85 * 16,
        "R10 west terrace art preimage differs"
    );
    let mut glyphs = [0_u8; 16];
    for (quadrant, (block, source)) in [(0x71, 0), (0x69, 1), (0x71, 2), (0x71, 3)]
        .into_iter()
        .enumerate()
    {
        for row in 0..2 {
            for column in 0..2 {
                let destination = (quadrant / 2 * 2 + row) * 4 + quadrant % 2 * 2 + column;
                let index = block * 16 + (source / 2 * 2 + row) * 4 + source % 2 * 2 + column;
                glyphs[destination] = raw[index];
            }
        }
    }
    let mut result = base.clone();
    result
        .runtime_files
        .get_mut(&path)
        .context("R10 metatiles")?
        .extend(glyphs);
    let definition = result
        .data
        .tilesets
        .get_mut(TILESET)
        .context("R10 collision")?;
    definition.collision.insert(
        "85".into(),
        ["FLOOR", "WALL", "FLOOR", "FLOOR"]
            .map(String::from)
            .to_vec(),
    );
    result.runtime_files.insert(
        format!("data/tilesets/{TILESET}.json"),
        serde_json::to_vec(&definition.collision)?,
    );
    result.report.manifests.push(MANIFEST.into());
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    Ok(result)
}
