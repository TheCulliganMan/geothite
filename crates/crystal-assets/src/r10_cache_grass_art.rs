//! Native grass/cliff and outdoor stair joins for R10's cache terrace.

use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;

/// Appends a grass-sided right cliff and an outdoor stair to the private R10 art.
///
/// Private81 takes its left two glyph columns from native03 and its right two
/// columns from native69. It reuses existing glyphs, per-glyph palettes and
/// foreground masks without modifying any source image or original metatile.
/// Its left walking quadrants are TALL_GRASS; the right wall stays solid.
/// Private82 retains stair80's gray treads and collision, replacing only its
/// pale upper cave floor with native72's brown outdoor landing rows.
/// All maps, actors, events, scripts, encounters and audio remain unchanged.
/// Returns a fresh pack identity; borrows the parent and writes no files.
///
/// # Errors
/// Rejects invalid or stale geography, a non-private/shared tileset, changed
/// source extent/collisions, an installed destination or failed verification.
pub fn add_r10_cache_art(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    const TILESET: &str = "mn_r10_rockstairs";
    const MANIFEST: &str = "minnesota-r10-cache-terrain-art-v1";
    crate::validate_compact_superior_geography(base)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|id| id == MANIFEST),
        "R10 cache grass edge is already installed"
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
        "R10 grass edge must remain private to the two route chunks"
    );
    let definition = base.data.tilesets.get(TILESET).context("R10 native art")?;
    ensure!(
        definition
            .collision
            .get("03")
            .is_some_and(|k| k.iter().map(String::as_str).eq(["TALL_GRASS"; 4]))
            && definition.collision.get("69").is_some_and(|k| k
                .iter()
                .map(String::as_str)
                .eq(["FLOOR", "WALL", "FLOOR", "WALL"]))
            && definition.collision.get("80").is_some_and(|k| k
                .iter()
                .map(String::as_str)
                .eq(["FLOOR", "FLOOR", "WALL", "FLOOR"]))
            && !definition.collision.contains_key("81")
            && !definition.collision.contains_key("82"),
        "R10 source grass or cliff collision differs"
    );
    let path = format!("data/tilesets/{TILESET}_metatiles.bin");
    let raw = base
        .runtime_files
        .get(&path)
        .context("R10 native metatiles")?;
    ensure!(
        raw.len() == 0x81 * 16,
        "R10 must retain its original128 blocks and native stair80"
    );
    let mut glyphs = Vec::with_capacity(32);
    for row in 0..4 {
        glyphs.extend_from_slice(&raw[0x03 * 16 + row * 4..0x03 * 16 + row * 4 + 2]);
        glyphs.extend_from_slice(&raw[0x69 * 16 + row * 4 + 2..0x69 * 16 + row * 4 + 4]);
    }
    glyphs.extend_from_slice(&raw[0x72 * 16..0x72 * 16 + 8]);
    glyphs.extend_from_slice(&raw[0x80 * 16 + 8..0x81 * 16]);
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
        "81".into(),
        ["TALL_GRASS", "WALL", "TALL_GRASS", "WALL"]
            .map(String::from)
            .to_vec(),
    );
    definition.collision.insert(
        "82".into(),
        ["FLOOR", "FLOOR", "WALL", "FLOOR"]
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

/// Appends grass-lined west and front cliffs for the closed R10 cache terrace.
///
/// Private83 combines native68's left two glyph columns with native03's right
/// two columns. The wall remains solid and its inside lip becomes encounter
/// grass. Private84 combines native03's upper glyph rows with native72's lower
/// retaining face, closing the dry strip across its front. Existing glyphs,
/// palettes, foreground masks and all131 old metatiles remain unchanged.
/// No terrain, actors, events or audio are changed here.
/// Returns a fresh pack identity without writing files.
///
/// # Errors
/// Rejects stale geography, shared art, changed native source collisions or
/// extent, an already installed join, or a pack that fails runtime verification.
pub fn add_r10_west_terrace_art(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    const TILESET: &str = "mn_r10_rockstairs";
    const MANIFEST: &str = "minnesota-r10-west-terrace-art-v1";
    crate::validate_compact_superior_geography(base)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|id| id == MANIFEST),
        "R10 west terrace art is already installed"
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
        "R10 west terrace art must remain private to its two chunks"
    );
    let definition = base.data.tilesets.get(TILESET).context("R10 native art")?;
    ensure!(
        definition
            .collision
            .get("03")
            .is_some_and(|k| k.iter().map(String::as_str).eq(["TALL_GRASS"; 4]))
            && definition.collision.get("68").is_some_and(|k| k
                .iter()
                .map(String::as_str)
                .eq(["WALL", "FLOOR", "WALL", "FLOOR"]))
            && definition.collision.get("72").is_some_and(|k| k
                .iter()
                .map(String::as_str)
                .eq(["FLOOR", "FLOOR", "WALL", "WALL"]))
            && !definition.collision.contains_key("83")
            && !definition.collision.contains_key("84"),
        "R10 west cliff or grass collision differs"
    );
    let path = format!("data/tilesets/{TILESET}_metatiles.bin");
    let raw = base.runtime_files.get(&path).context("R10 metatiles")?;
    ensure!(raw.len() == 0x83 * 16, "R10 cache art preimage differs");
    let mut glyphs = Vec::with_capacity(32);
    for row in 0..4 {
        glyphs.extend_from_slice(&raw[0x68 * 16 + row * 4..0x68 * 16 + row * 4 + 2]);
        glyphs.extend_from_slice(&raw[0x03 * 16 + row * 4 + 2..0x03 * 16 + row * 4 + 4]);
    }
    glyphs.extend_from_slice(&raw[0x03 * 16..0x03 * 16 + 8]);
    glyphs.extend_from_slice(&raw[0x72 * 16 + 8..0x73 * 16]);
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
        "83".into(),
        ["WALL", "TALL_GRASS", "WALL", "TALL_GRASS"]
            .map(String::from)
            .to_vec(),
    );
    definition.collision.insert(
        "84".into(),
        ["TALL_GRASS", "TALL_GRASS", "WALL", "WALL"]
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
