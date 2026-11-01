//! Frozen native snow-to-Superior art for the complete Iron Range descent.
use anyhow::Result;

use crate::CompiledGamePack;
use crate::tamarack_snowline::{self, SnowlineBinding};

/// Builds the two-chunk descent from snowy Grand Rapids to green Duluth.
///
/// Preserves every collision quadrant, native stair, cave mouth, habitat mask,
/// actor, script, reward, original warm pixel and unrelated map/audio record.
/// Returns an unselected candidate. Gameplay and visual review remain separate.
///
/// # Errors
/// Rejects source, parent, complete module or native art drift; changed collision;
/// unsafe glyph storage; and any unrelated residual content changes.
pub fn build_iron_descent_snowline(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    tamarack_snowline::build_bound_snowline(
        base,
        source,
        &SnowlineBinding {
            source_hash: "83009e35f33336350e22b0d0d5ea2ba56dfcd19cbda2e5a4c73d37bb75b4ef29",
            marker: "minnesota-iron-descent-snowline-v1",
            source_tileset: "mn_r10_rockstairs",
            full_alias: "mn_winter_iron_descent",
            partial_alias: "mn_iron_descent_thaw",
            art_manifest: "minnesota-iron-descent-thaw-art-v1",
            maps: &[
                ("MnR10C00R00", [32, 20], "mn_iron_descent_thaw"),
                ("MnR10C01R00", [16, 20], "mn_iron_descent_thaw"),
            ],
            unchanged_cut_gates: &[],
            extra_cut_variants: 0,
        },
    )
}
