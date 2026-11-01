//! Pure ReturnMon/EnterMon sample-coordinate remapping for a modeled enemy.
//!
//! All rectangles use top-left origin, X right, Y down, and half-open edges.
//! The model target retains the full viewport resolution and original pose.
use bevy::prelude::{Rect, Vec2};

pub const FRONT_SLOT_MIN: Vec2 = Vec2::new(96.0, 0.0);
pub const FRONT_SLOT_MAX: Vec2 = Vec2::new(152.0, 56.0);
const TILE_SIZE: f32 = 8.0;

/// Affine registration of the original projected model footprint to the
/// original front picture's opaque bounds, in the unchanged full 56x56 slot.
/// Neither rectangle is recomputed from a compacted 5- or 3-tile picture.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaptureActorRegistration {
    /// Original model footprint in normalized full-target viewport coordinates.
    /// May extend beyond [0, 1] when the original model is partly offscreen.
    pub actor_view_rect: Rect,
    /// Original padded front picture's opaque bounds in LCD pixel coordinates.
    /// Must be nonempty and contained in (96, 0)..(152, 56).
    pub source_opaque_rect: Rect,
}

impl CaptureActorRegistration {
    pub fn is_valid(self) -> bool {
        valid_rect(self.actor_view_rect)
            && valid_rect(self.source_opaque_rect)
            && self.source_opaque_rect.min.cmpge(FRONT_SLOT_MIN).all()
            && self.source_opaque_rect.max.cmple(FRONT_SLOT_MAX).all()
    }
}

fn valid_rect(rect: Rect) -> bool {
    rect.min.is_finite()
        && rect.max.is_finite()
        && rect.size().is_finite()
        && rect.size().cmpgt(Vec2::ZERO).all()
}

fn inside(point: Vec2, min: Vec2, max: Vec2) -> bool {
    point.is_finite() && point.cmpge(min).all() && point.cmplt(max).all()
}

fn selected_axes(shown_tiles: u32) -> Option<&'static [u32]> {
    match shown_tiles {
        7 => Some(&[0, 1, 2, 3, 4, 5, 6]),
        5 => Some(&[0, 1, 3, 5, 6]),
        3 => Some(&[0, 3, 6]),
        _ => None,
    }
}

/// Map a destination LCD pixel position to its original LCD pixel position.
///
/// Unlike the source sprite's raster lookup, this function never rounds to
/// whole LCD pixels. Only the tile index is floored; the complete fractional
/// displacement within its selected 8x8 tile survives. `None` means transparent.
pub fn capture_front_source_pixel(destination: Vec2, shown_tiles: u32) -> Option<Vec2> {
    let axes = selected_axes(shown_tiles)?;
    if !inside(destination, FRONT_SLOT_MIN, FRONT_SLOT_MAX) {
        return None;
    }
    let removed = 7 - shown_tiles;
    let active_min = FRONT_SLOT_MIN
        + Vec2::new((removed / 2) as f32, removed as f32) * TILE_SIZE;
    let active_max = active_min + Vec2::splat(shown_tiles as f32 * TILE_SIZE);
    if !inside(destination, active_min, active_max) {
        return None;
    }
    // Exact full-picture identity, including subpixel positions.
    if shown_tiles == 7 {
        return Some(destination);
    }
    let compact = destination - active_min;
    let destination_tile = (compact / TILE_SIZE).floor();
    let within_tile = compact - destination_tile * TILE_SIZE;
    let source_tile = Vec2::new(
        *axes.get(destination_tile.x as usize)? as f32,
        *axes.get(destination_tile.y as usize)? as f32,
    );
    Some(FRONT_SLOT_MIN + source_tile * TILE_SIZE + within_tile)
}

/// For one current viewport fragment, select a sample from the unchanged,
/// full-resolution, transparent-background isolated actor target.
///
/// Chain: current viewport UV -> destination LCD pixel -> inverse tile remap
/// -> original LCD pixel -> original viewport UV. This affine registration is
/// for modeled actors only, not the perspective projection used by source OAM.
///
/// Do not preclip the destination to `actor_view_rect`: compaction can move
/// valid samples outside that original footprint. A full-viewport composite
/// is sufficient. During compacted phases its coverage can be restricted to
/// the registered full source slot; the full-picture phase must preserve the
/// complete target, including antialias fringe outside that slot's coverage.
/// Let this function own destination clipping. The opaque rectangle is an
/// affine basis, never an alpha mask: samples beyond it can contain the model's
/// antialiased edge coverage. Source holes and translucent pixels are preserved
/// by sampling target RGBA. The full-picture phase bypasses remapping entirely.
pub fn capture_actor_sample_uv(
    viewport_uv: Vec2,
    registration: CaptureActorRegistration,
    shown_tiles: u32,
) -> Option<Vec2> {
    if !inside(viewport_uv, Vec2::ZERO, Vec2::ONE) || !registration.is_valid() {
        return None;
    }
    // Full-picture identity includes antialias coverage beyond vertex bounds,
    // even when those bounds register to the outer edge of the original slot.
    if shown_tiles == 7 {
        return Some(viewport_uv);
    }
    let view = registration.actor_view_rect;
    let opaque = registration.source_opaque_rect;
    let destination = opaque.min + (viewport_uv - view.min) / view.size() * opaque.size();
    let source = capture_front_source_pixel(destination, shown_tiles)?;
    let source_uv = view.min + (source - opaque.min) / opaque.size() * view.size();
    // A full-viewport capture has no original samples beyond its boundaries.
    inside(source_uv, Vec2::ZERO, Vec2::ONE).then_some(source_uv)
}

#[cfg(test)]
#[path = "capture_tile_remap_tests.rs"]
mod tests;
