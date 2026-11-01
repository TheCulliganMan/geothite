//! Clear the upper Haulage stair approach without changing its pressure network.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;

const MAP: &str = "MnIronHaulage";
const TILESET: &str = "mn_iron_haulage_rock";
const MARKER: &str = "minnesota-haulage-rock-clearance-v1";
const SOURCE_SHA256: &str = "1257acaf832e1986da0849e9cea2721e98bcf90eb0e2006de063557c5a46eaf2";

#[derive(Deserialize)]
struct Proposal {
    schema_version: u8,
    parent_content_hash: String,
    map: String,
    tileset: String,
    dimensions_metatiles: [u16; 2],
    native_module_sha256: String,
    tileset_sha256: String,
    before_rows: Vec<String>,
    rows: Vec<String>,
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn rows(source: &[String]) -> Result<Vec<u16>> {
    ensure!(
        source.len() == 14,
        "fourteen complete Haulage rows required"
    );
    let mut blocks = Vec::with_capacity(252);
    for row in source {
        let values = row
            .split_whitespace()
            .map(|word| u16::from_str_radix(word, 16))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(
            values.len() == 18,
            "eighteen complete Haulage columns required"
        );
        blocks.extend(values);
    }
    Ok(blocks)
}

/// Sets the upper west boulder one metatile back from the native cliff face.
///
/// Only two cells swap. The private alias's `0d` is floor; original cave `0d`
/// must not be substituted. The entire block inventory, art, stairs, five shaft
/// pairs, actors, pressure callback, rewards and unrelated content remain exact.
/// Returns an unselected candidate. Saved terrain overrides are not migrated;
/// native presentation and production input need separate verification.
///
/// # Errors
/// Rejects source/parent/module/art drift, unexpected floor or rock collision,
/// undeclared terrain changes, inventory changes and unrelated residual changes.
pub fn revise_haulage_rock_clearance(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        hash(source.as_bytes()) == SOURCE_SHA256,
        "reviewed Haulage source differs"
    );
    let plan: Proposal = serde_json::from_str(source)?;
    ensure!(
        plan.schema_version == 1
            && plan.parent_content_hash == base.identity.content_hash
            && plan.map == MAP
            && plan.tileset == TILESET
            && plan.dimensions_metatiles == [18, 14]
            && !base.report.manifests.iter().any(|m| m == MARKER),
        "complete reviewed Haulage parent required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let original = base.data.maps.get(MAP).context("original Haulage module")?;
    let tileset = base
        .data
        .tilesets
        .get(TILESET)
        .context("private Haulage art")?;
    ensure!(
        [original.attributes.width, original.attributes.height] == [18, 14]
            && original.attributes.tileset_name == TILESET
            && original.blocks == rows(&plan.before_rows)?
            && hash(&serde_json::to_vec(&serde_json::to_value(original)?)?)
                == plan.native_module_sha256
            && hash(&serde_json::to_vec(&serde_json::to_value(tileset)?)?) == plan.tileset_sha256,
        "complete native Haulage map or art preimage differs"
    );
    for (block, expected) in [("0d", "FLOOR"), ("1d", "WALL")] {
        ensure!(
            tileset
                .collision
                .get(block)
                .is_some_and(|q| q.iter().all(|c| c == expected)),
            "private Haulage floor/rock collision differs"
        );
    }
    let mut blocks = original.blocks.clone();
    for (index, before, after) in [(3 * 18 + 4, 0x1d, 0x0d), (2 * 18 + 4, 0x0d, 0x1d)] {
        ensure!(
            blocks[index] == before,
            "reviewed Haulage boulder preimage differs"
        );
        blocks[index] = after;
    }
    ensure!(
        blocks == rows(&plan.rows)?,
        "unlisted Haulage terrain change"
    );
    let inventory = |blocks: &[u16]| {
        let mut counts = BTreeMap::new();
        for &block in blocks {
            *counts.entry(block).or_insert(0_usize) += 1;
        }
        counts
    };
    ensure!(
        inventory(&blocks) == inventory(&original.blocks),
        "native block inventory differs"
    );
    let bytes = blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let label = original
        .attributes
        .blocks_label
        .as_ref()
        .context("native block mirror")?;
    let mut child = base.clone();
    child
        .data
        .maps
        .get_mut(MAP)
        .context("child Haulage")?
        .blocks = blocks;
    child
        .data
        .map_blocks
        .insert(label.clone(), crate::encode_base64_bytes(&bytes));
    child.report.manifests.push(MARKER.into());
    child.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        child.format_version,
        &child.data,
        &child.audio_manifest,
        &child.runtime_files,
        &child.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&child)?;
    crate::validate_compact_superior_geography(&child)?;
    let mut residual = child.clone();
    residual.data.maps.insert(MAP.into(), original.clone());
    residual.data.map_blocks.insert(
        label.clone(),
        base.data
            .map_blocks
            .get(label)
            .context("original block mirror")?
            .clone(),
    );
    residual.report = base.report.clone();
    residual.identity = base.identity.clone();
    ensure!(
        residual == *base,
        "unrelated content changed during Haulage rock clearance"
    );
    Ok(child)
}
