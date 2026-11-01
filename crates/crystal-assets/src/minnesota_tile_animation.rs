//! Original flame keyframes lowered into the same native tile clips as flowers.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};

use crate::minnesota_pokemon_pixel_art;
use crate::{CompiledGamePack, MinnesotaTileCatalog, MinnesotaTilePixels};

/// One packed native 8px glyph animation, using the production ambient clock.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredTileAnimation {
    pub frame_ticks: u64,
    pub frames: Vec<[u8; 16]>,
}

impl AuthoredTileAnimation {
    /// Reject empty/unbounded clips or unsupported frame durations.
    pub fn validate(&self) -> Result<()> {
        ensure!(
            (1..=120).contains(&self.frame_ticks),
            "animation frame duration must be 1–120 ticks"
        );
        ensure!(
            (2..=16).contains(&self.frames.len()),
            "animation needs 2–16 frames"
        );
        Ok(())
    }
}

/// Reproducible original-source recipe. File paths are resolved by the CLI.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MinnesotaTileAnimationRecipe {
    pub base_catalog: String,
    pub base_source: String,
    pub sources: Vec<TileAnimationSource>,
    pub region: [usize; 4],
    pub frame_order: Vec<usize>,
    pub frame_ticks: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TileAnimationSource {
    pub path: String,
    pub sha256: String,
}

impl MinnesotaTileAnimationRecipe {
    /// Import original source frames, changing only the reviewed flame rectangle.
    /// The rest of every 32px frame remains byte-for-byte the original campfire.
    ///
    /// Errors on source hash/crop drift, invalid timing/region/order, unchanged
    /// variants or malformed native source. The catalog must contain one item.
    pub fn compile(
        &self,
        catalog_json: &[u8],
        base_source: &[u8],
        sources: &[Vec<u8>],
    ) -> Result<Vec<MinnesotaTilePixels>> {
        let catalog: MinnesotaTileCatalog = serde_json::from_slice(catalog_json)?;
        ensure!(
            catalog.items.len() == 1,
            "animation recipe requires one original item"
        );
        let base = catalog.convert(base_source)?.remove(0);
        ensure!(
            sources.len() == self.sources.len() && !sources.is_empty(),
            "animation sources missing"
        );
        let [x, y, width, height] = self.region;
        ensure!(
            width > 0
                && height > 0
                && x.checked_add(width).is_some_and(|end| end <= 32)
                && y.checked_add(height).is_some_and(|end| end <= 32),
            "animation region outside native frame"
        );
        AuthoredTileAnimation {
            frame_ticks: self.frame_ticks,
            frames: vec![[0; 16]; self.frame_order.len()],
        }
        .validate()?;
        let mut frames = vec![base];
        let mut unique = BTreeSet::from([frames[0].indices.clone()]);
        for (bytes, source) in sources.iter().zip(&self.sources) {
            let mut value: serde_json::Value = serde_json::from_slice(catalog_json)?;
            value["source_sha256"] = source.sha256.clone().into();
            let variant: MinnesotaTileCatalog = serde_json::from_value(value)?;
            let imported = variant.convert(bytes)?.remove(0);
            let mut indices = frames[0].indices.clone();
            for row in y..y + height {
                indices[row * 32 + x..row * 32 + x + width]
                    .copy_from_slice(&imported.indices[row * 32 + x..row * 32 + x + width]);
            }
            ensure!(
                unique.insert(indices.clone()),
                "animation variant has no distinct native flame"
            );
            frames.push(MinnesotaTilePixels { indices });
        }
        ensure!(
            self.frame_order.iter().all(|index| *index < frames.len()),
            "animation order references missing frame"
        );
        Ok(frames)
    }
}

/// Add authored ambient clips to an installed original Minnesota prop block.
///
/// Only uniquely owned, changed glyphs may animate. Original runtime bytes,
/// static ring/log pixels, maps, collision, palettes and audio remain unchanged.
/// Errors on source/frame drift, shared glyphs, occupied output metadata or
/// invalid pack/animation content.
pub fn with_minnesota_tile_animation(
    base: &CompiledGamePack,
    tileset: &str,
    block: u8,
    frames: &[MinnesotaTilePixels],
    order: &[usize],
    frame_ticks: u64,
) -> Result<CompiledGamePack> {
    ensure!(
        block >= 128 && !frames.is_empty(),
        "animation requires an appended prop block"
    );
    let path = format!("data/tilesets/{tileset}_animations.json");
    ensure!(
        !base.runtime_files.contains_key(&path),
        "authored animation already installed"
    );
    let layout = base
        .runtime_files
        .get(&format!("data/tilesets/{tileset}_metatiles.bin"))
        .context("animation metatile layout")?;
    let glyphs = layout
        .get(usize::from(block) * 16..(usize::from(block) + 1) * 16)
        .context("animation prop block")?;
    let definition = base
        .data
        .tilesets
        .get(tileset)
        .context("animation tileset")?;
    let graphics = base
        .runtime_files
        .get(&format!("gfx/tilesets/{tileset}.2bpp"))
        .context("animation static glyphs")?;
    let encoded = frames
        .iter()
        .map(|frame| {
            ensure!(
                frame.indices.len() == 1024 && frame.indices.iter().all(|value| *value < 4),
                "invalid native animation indices"
            );
            Ok(minnesota_pokemon_pixel_art::bitplanes(
                &frame.indices,
                32,
                32,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    let mut clips = BTreeMap::new();
    for (tile, &glyph) in glyphs.iter().enumerate() {
        let start = tile * 16;
        let first = &encoded[0][start..start + 16];
        if encoded
            .iter()
            .all(|frame| &frame[start..start + 16] == first)
        {
            continue;
        }
        ensure!(
            layout.iter().filter(|value| **value == glyph).count() == 1,
            "animated glyph is shared with unrelated art"
        );
        let palette = definition.palette_map[usize::from(glyph)];
        let slot = usize::from(glyph & 127) + usize::from(palette >> 3) * (graphics.len() / 32);
        ensure!(
            &graphics[slot * 16..slot * 16 + 16] == first,
            "animation frame 0 differs from installed static art"
        );
        let clip = AuthoredTileAnimation {
            frame_ticks,
            frames: order
                .iter()
                .map(|&frame| {
                    let bytes = encoded.get(frame).context("missing animation frame")?;
                    Ok(bytes[start..start + 16].try_into()?)
                })
                .collect::<Result<_>>()?,
        };
        clip.validate()?;
        clips.insert(glyph, clip);
    }
    ensure!(
        !clips.is_empty(),
        "no native glyphs change across animation frames"
    );
    let mut pack = base.clone();
    pack.runtime_files.insert(path, serde_json::to_vec(&clips)?);
    pack.report.manifests.push(format!(
        "{}-ambient-animation-v1",
        tileset.replace('_', "-")
    ));
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
