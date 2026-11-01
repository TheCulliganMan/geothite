//! Frozen native pine-country snow fading westward into green Itasca.
use anyhow::Result;

use crate::CompiledGamePack;
use crate::tamarack_snowline::{self, SnowlineBinding};

/// Builds the complete four-chunk Bemidji-to-Itasca snowline from frozen rows.
///
/// Preserves every collision quadrant, native ledge, pine-grove Cut shortcut,
/// gatehouse/room pair, actor, script, habitat, reward and unrelated map/audio
/// record. Appended cold/melt tree and grass pieces receive exact native Cut
/// replacements. Returns an unselected candidate; controller, clock/weather and
/// native visual review remain separate. Existing save overrides are not migrated.
///
/// # Errors
/// Rejects source/parent/module/art/catalog drift, changed collision, unsafe
/// glyph storage, altered original Cut gates and unrelated residual changes.
pub fn build_itasca_snowline(base: &CompiledGamePack, source: &str) -> Result<CompiledGamePack> {
    tamarack_snowline::build_bound_snowline(
        base,
        source,
        &SnowlineBinding {
            source_hash: "ced7b7a8196f401200c19982a63bb6487feafe1e97c2407b4eb920cdd2e5f52d",
            marker: "minnesota-itasca-snowline-v1",
            source_tileset: "mn_headwaters",
            full_alias: "mn_winter_itasca_trail",
            partial_alias: "mn_itasca_thaw",
            art_manifest: "minnesota-itasca-thaw-art-v1",
            maps: &[
                ("MnR14C00R00", [32, 24], "mn_itasca_thaw"),
                ("MnR14C00R01", [32, 8], "mn_itasca_thaw"),
                ("MnR14C01R00", [16, 24], "mn_winter_itasca_trail"),
                ("MnR14C01R01", [16, 8], "mn_winter_itasca_trail"),
            ],
            unchanged_cut_gates: &[("MnR14C01R00", 0x5b)],
            extra_cut_variants: 10,
        },
    )
}
