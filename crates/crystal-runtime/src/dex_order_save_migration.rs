//! Copy exact Fair presentation saves into the numbering-only release.

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use super::{Compatibility, CompatibleSaveSource, fair_save_migration};

const MARKER: &str = "+northstar-dex-order-v1";
const SUFFIX: &str = "+realtime-clock+player-customization";
const PARENT_HASHES: [&str; 2] = [
    "c28b37a15c0ae59151da94d4d70dde6ac3cc066b47afe993665cb071e7a4b3d4",
    "f123f566ed0b70cd9e551a2c88673faac132555e610c3eb5e3ecc95816d1bc57",
];
const PARENT_CONTENT: [&str; 2] = [
    "d09a77f306578627e661a0811d2bacc12d22552e5e84041e0f002f6635db46c2",
    "d03e25c72b2852c6c87e31924a7c2467f35f51469a7fc7e327370f7a84676a3e",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Release {
    schema_version: u8,
    sources: [CompatibleSaveSource; 2],
    inherited: serde_json::Value,
}

pub(super) fn parse(bytes: &[u8], target: &str) -> Result<Compatibility> {
    let release: Release = serde_json::from_slice(bytes)?;
    let [raw, hosted] = &release.sources;
    let target_base = target.strip_suffix(SUFFIX).unwrap_or(target);
    let parent = target_base
        .strip_suffix(MARKER)
        .context("invalid Pokédex-order target")?;
    ensure!(
        release.schema_version == 11
            && parent == raw.modpack_id
            && parent.ends_with("+northstar-fair-presentation-v1")
            && hosted.modpack_id.strip_prefix(parent) == Some(SUFFIX)
            && release.sources.iter().enumerate().all(|(i, source)| {
                source.modpack_hash == PARENT_HASHES[i] && source.content_hash == PARENT_CONTENT[i]
            }),
        "expected exact Fair presentation sources"
    );
    let source_id = if target_base == target {
        &raw.modpack_id
    } else {
        &hosted.modpack_id
    };
    let mut policy = fair_save_migration::parse_presentation(
        &serde_json::to_vec(&release.inherited)?,
        source_id,
    )?;
    policy.unchanged_checkpoint_parents.extend(
        release
            .sources
            .iter()
            .map(|source| source.modpack_id.clone()),
    );
    policy.sources.splice(0..0, release.sources);
    Ok(policy)
}

/// These exact archives already contain the Fair layout corrections. Replaying
/// the older layout migration would discard saved actor positions and terrain.
pub(super) fn preserves_fair_layout(source: &CompatibleSaveSource) -> bool {
    PARENT_HASHES.contains(&source.modpack_hash.as_str())
}

#[cfg(test)]
#[path = "dex_order_save_migration_tests.rs"]
mod tests;
