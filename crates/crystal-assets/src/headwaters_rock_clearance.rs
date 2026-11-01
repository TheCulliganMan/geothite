//! Set native Headwaters boulders back from the terrace face and stair.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;

const SOURCE_SHA256: &str = "e80f053d3035ad03e764cf96179a245e327bd3dfc2b87546b3840f5812531c18";
const MAP: &str = "MnR08C00R00";
const TILESET: &str = "mn_winter_headwaters_stairs";
const MARKER: &str = "minnesota-headwaters-rock-clearance-v1";

#[derive(Deserialize)]
struct Proposal {
    schema_version: u8,
    parent_identity: Value,
    maps: Vec<Terrain>,
    edits: Vec<Edit>,
}

#[derive(Deserialize)]
struct Terrain {
    id: String,
    tileset: String,
    dimensions_metatiles: [u16; 2],
    native_module_sha256: String,
    before_rows: Vec<String>,
    after_rows: Vec<String>,
}

#[derive(Deserialize)]
struct Edit {
    metatile: [u16; 2],
    before: String,
    after: String,
}

fn rows(rows: &[String]) -> Result<Vec<u16>> {
    ensure!(
        rows.len() == 24,
        "complete twenty-four-row Headwaters required"
    );
    let mut blocks = Vec::with_capacity(768);
    for row in rows {
        let parsed = row
            .split_whitespace()
            .map(|word| u16::from_str_radix(word, 16))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(
            parsed.len() == 32,
            "complete thirty-two-column Headwaters required"
        );
        blocks.extend(parsed);
    }
    Ok(blocks)
}

/// Moves three boulders away from the native terrace face and stair landing.
///
/// Six explicit metatiles change in one existing chunk. All ten boulders and
/// the complete native block inventory remain intact. Art, cliff pieces, stairs,
/// water, tree gates, actors, scripts, rewards, other maps and audio are exact.
/// Returns an unselected candidate; saved terrain overrides are not migrated.
/// Runtime movement needs separate checks.
///
/// # Errors
/// Rejects source/parent/module drift, incomplete rows, undeclared changes,
/// unexpected collision, changed rock counts or unrelated residual changes.
pub fn revise_headwaters_rock_clearance(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        format!("{:x}", Sha256::digest(source.as_bytes())) == SOURCE_SHA256,
        "reviewed Headwaters rock source differs"
    );
    let plan: Proposal = serde_json::from_str(source)?;
    ensure!(
        plan.schema_version == 1
            && plan.maps.len() == 1
            && plan.edits.len() == 6
            && serde_json::to_value(&base.identity)? == plan.parent_identity
            && !base.report.manifests.iter().any(|m| m == MARKER),
        "complete source-bound Headwaters parent required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let terrain = &plan.maps[0];
    let original = base.data.maps.get(MAP).context("original Headwaters map")?;
    ensure!(
        terrain.id == MAP
            && terrain.tileset == TILESET
            && terrain.dimensions_metatiles == [32, 24]
            && original.attributes.tileset_name == TILESET
            && [original.attributes.width, original.attributes.height] == [32, 24]
            && original.blocks == rows(&terrain.before_rows)?
            && format!(
                "{:x}",
                Sha256::digest(serde_json::to_vec(&serde_json::to_value(original)?)?)
            ) == terrain.native_module_sha256,
        "complete native Headwaters module preimage differs"
    );
    let collision = &base
        .data
        .tilesets
        .get(TILESET)
        .context("native thaw art")?
        .collision;
    let mut blocks = original.blocks.clone();
    let mut seen = BTreeSet::new();
    for edit in &plan.edits {
        let [x, y] = edit.metatile;
        ensure!(
            x < 32 && y < 24 && seen.insert(edit.metatile),
            "unique in-map edit required"
        );
        let before = u16::from_str_radix(&edit.before, 16)?;
        let after = u16::from_str_radix(&edit.after, 16)?;
        let expected = match edit.metatile {
            [13, 12] | [16, 14] => (0x0a, 0x03, ["TALL_GRASS"; 4]),
            [20, 12] => (0x0a, 0x71, ["FLOOR"; 4]),
            [13, 11] | [16, 15] => (0x03, 0x0a, ["WALL"; 4]),
            [20, 11] => (0x71, 0x0a, ["WALL"; 4]),
            _ => anyhow::bail!("only the six reviewed boulder/ground cells may change"),
        };
        let index = usize::from(y) * 32 + usize::from(x);
        ensure!(
            blocks[index] == before
                && (before, after) == (expected.0, expected.1)
                && collision
                    .get(&edit.after)
                    .is_some_and(|kinds| kinds.iter().map(String::as_str).eq(expected.2)),
            "reviewed native grass/floor or boulder collision differs"
        );
        blocks[index] = after;
    }
    ensure!(
        blocks == rows(&terrain.after_rows)?,
        "unlisted Headwaters terrain edit"
    );
    let inventory = |blocks: &[u16]| {
        let mut counts = BTreeMap::new();
        for &block in blocks {
            *counts.entry(block).or_insert(0_usize) += 1;
        }
        counts
    };
    ensure!(
        inventory(&original.blocks) == inventory(&blocks)
            && blocks.iter().filter(|&&block| block == 0x0a).count() == 10,
        "all ten boulders and the complete native block inventory required"
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
        .context("original block mirror")?;
    let mut child = base.clone();
    child
        .data
        .maps
        .get_mut(MAP)
        .context("child Headwaters map")?
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
            .context("original source block mirror")?
            .clone(),
    );
    residual.report = base.report.clone();
    residual.identity = base.identity.clone();
    ensure!(
        residual == *base,
        "unrelated content changed during Headwaters rock repair"
    );
    Ok(child)
}
