//! Packed native animation data shared by the renderer and pack builders.

use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

/// One packed native 8px glyph animation, using the production ambient clock.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredTileAnimation {
    pub frame_ticks: u64,
    pub frames: Vec<[u8; 16]>,
}

impl AuthoredTileAnimation {
    /// Rejects empty or unbounded clips and unsupported frame durations.
    ///
    /// # Errors
    /// Returns an error unless the clip has 2–16 frames of 1–120 ticks each.
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
