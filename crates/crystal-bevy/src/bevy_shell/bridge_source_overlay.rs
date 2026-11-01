//! Opt-in source-pixel bridge overlay and OAM depth bands.
//!
//! Existing glyph handles/palettes/alpha are reused without pixel modification.

use anyhow::{Context, Result, ensure};
use bevy::prelude::{Assets, Handle, Image, Vec2};
use crystal_core::world::collision::PlayerTraversalState;
use crystal_runtime::RuntimeTilesetCatalogSnapshot;

use super::{ResolvedRenderSource, tileset_collision_tokens};

/// Above Surf OAM (ordinary depth minus 0.5), below untouched Foot OAM.
/// Ordinary field actor depth is approximately 1.0..1.18 in the source LCD.
pub(super) const OVERLAY_DEPTH: f32 = 0.8;

fn is_bridge_token(token: &str) -> bool {
    matches!(
        token,
        "BRIDGE_NS_DECK" | "BRIDGE_NS_RAIL" | "BRIDGE_EW_DECK" | "BRIDGE_EW_RAIL"
    )
}

/// Resolves one 8x8 source subtile from its containing 16x16 collision quadrant.
/// Missing/short rows and invalid subtile coordinates retain source errors.
pub(super) fn covers_subtile(
    tileset: &RuntimeTilesetCatalogSnapshot,
    block: u16,
    sub_x: usize,
    sub_y: usize,
) -> Result<bool> {
    ensure!(sub_x < 4 && sub_y < 4, "bridge source subtile out of range");
    let row = tileset_collision_tokens(tileset, block).context("bridge source collision row")?;
    ensure!(
        row.len() == 4,
        "bridge collision row must have four quadrants"
    );
    let quadrant = usize::from(sub_y >= 2) * 2 + usize::from(sub_x >= 2);
    Ok(is_bridge_token(&row[quadrant]))
}

/// Tests installed blocks, rather than merely detecting unused alias tokens.
/// Include every resolved connected source so approach OAM is lowered before
/// its footing reaches the bridge, including the north/south mouth head overlap.
pub(super) fn scene_has_bridge(sources: &[ResolvedRenderSource<'_>]) -> Result<bool> {
    for source in sources {
        if blocks_have_bridge(source.tileset, &source.map.blocks)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn blocks_have_bridge(tileset: &RuntimeTilesetCatalogSnapshot, blocks: &[u16]) -> Result<bool> {
    for &block in blocks {
        let row = tileset_collision_tokens(tileset, block).context("bridge scene collision row")?;
        ensure!(
            row.len() == 4,
            "bridge collision row must have four quadrants"
        );
        if row.iter().any(|token| is_bridge_token(token)) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Full native handle: never use priority_tile_handle, which erases index zero.
/// This spec is accepted by the existing compose_priority_tile_grid function.
pub(super) fn overlay_spec(
    tileset: &RuntimeTilesetCatalogSnapshot,
    block: u16,
    sub_x: usize,
    sub_y: usize,
    source_handle: &Handle<Image>,
) -> Result<Option<(Handle<Image>, usize)>> {
    Ok(covers_subtile(tileset, block, sub_x, sub_y)?.then(|| (source_handle.clone(), 0)))
}

/// Keeps every ordinary map and every Foot actor at exactly its previous depth.
/// Apply identically in full actor spawn and every retained-motion update.
pub(super) fn actor_depth(
    ordinary: f32,
    scene_has_bridge: bool,
    traversal: PlayerTraversalState,
) -> f32 {
    if scene_has_bridge && matches!(traversal, PlayerTraversalState::Surf) {
        ordinary - 0.5
    } else {
        ordinary
    }
}

#[cfg(test)]
#[path = "bridge_source_overlay_tests.rs"]
mod tests;

/// Existing native swimmer movement identifies lower-water residents.
pub(super) fn object_traversal(movement: &str) -> PlayerTraversalState {
    if movement == "SPRITEMOVEDATA_SWIM_WANDER" {
        PlayerTraversalState::Surf
    } else {
        PlayerTraversalState::Walk
    }
}

/// A speculative two-layer frame cannot represent the bridge's separate OAM layer.
/// Callers must clear stale speculation and let the real source renderer build it.
#[derive(Debug)]
pub(super) struct BridgePrewarmUnavailable;

impl std::fmt::Display for BridgePrewarmUnavailable {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("explicit bridge requires the live three-layer source renderer")
    }
}

impl std::error::Error for BridgePrewarmUnavailable {}

/// Masks the existing native palette-transformed background for bridge OAM.
/// Palette-index zero stays opaque inside the bridge, including flash frames.
/// Unrelated terrain remains transparent and the input image is never mutated.
pub(super) fn transition_overlay(
    background: &Handle<Image>,
    bridge_mask: &[bool],
    camera_offset: Vec2,
    existing: Option<Handle<Image>>,
    images: &mut Assets<Image>,
) -> Result<Handle<Image>> {
    // A separate owned image is required: changing alpha in the existing base
    // would punch transparent holes in the entire native battle transition.
    let mut overlay = images
        .get(background)
        .context("bridge transition background")?
        .clone();
    ensure!(
        overlay.width() == 160 && overlay.height() == 144,
        "bridge transition LCD extent"
    );
    let columns = super::CLASSIC_SCROLL_TILES_X as usize;
    let rows = super::CLASSIC_SCROLL_TILES_Y as usize;
    ensure!(
        bridge_mask.len() == columns * rows,
        "bridge transition mask extent"
    );
    let pixels = overlay.data.as_mut().context("bridge transition RGBA")?;
    ensure!(
        pixels.len() == 160 * 144 * 4,
        "bridge transition RGBA extent"
    );
    let (origin_x, origin_y) = super::battle_transition_source_origin(camera_offset);
    for y in 0..144 {
        for x in 0..160 {
            let source_x = (origin_x + x as i32).clamp(0, (columns * 8 - 1) as i32) as usize;
            let source_y = (origin_y + y as i32).clamp(0, (rows * 8 - 1) as i32) as usize;
            pixels[(y * 160 + x) * 4 + 3] = if bridge_mask[source_y / 8 * columns + source_x / 8] {
                255
            } else {
                0
            };
        }
    }
    if let Some(handle) = existing
        && let Some(mut target) = images.get_mut(&handle)
    {
        *target = overlay;
        return Ok(handle);
    }
    Ok(images.add(overlay))
}
