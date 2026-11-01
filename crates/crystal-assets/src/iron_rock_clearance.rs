//! Restore native terrace edges under misplaced decorative Iron Range rocks.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;

const SOURCE_SHA256: &str = "d879a62cfa7e4220b4b05321e1555c58e9c0995842b79440c356fac5e3992dc2";
const MARKER: &str = "minnesota-iron-rock-clearance-v1";
const TILESET: &str = "mn_winter_iron_range";

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

fn block(value: &str) -> Result<u16> {
    u16::from_str_radix(value, 16).context("native Iron Range block")
}

fn rows(rows: &[String], size: [u16; 2]) -> Result<Vec<u16>> {
    ensure!(
        rows.len() == usize::from(size[1]),
        "complete terrain height required"
    );
    let mut result = Vec::new();
    for row in rows {
        let words = row.split_whitespace().collect::<Vec<_>>();
        ensure!(
            words.len() == usize::from(size[0]),
            "complete terrain width required"
        );
        result.extend(words.into_iter().map(block).collect::<Result<Vec<_>>>()?);
    }
    Ok(result)
}

/// Moves twelve rocks inward and removes six surplus decorative rocks, restoring
/// continuous original cliff faces/corners and a pine shoulder in thirty cells.
///
/// The four complete R09 modules and parent identity are bound to the reviewed
/// source. Existing stairs, grass, Cut, water, rails, actors, programs, rooms,
/// encounters, audio and all native art remain exact. This returns a verified
/// unselected candidate and does not migrate existing saved terrain overrides.
///
/// # Errors
/// Rejects source/parent drift, incomplete terrain, undeclared edits, changed
/// protected terrain, mismatched block mirrors or invalid resulting content.
pub fn revise_iron_rock_clearance(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        format!("{:x}", Sha256::digest(source.as_bytes())) == SOURCE_SHA256,
        "reviewed rock-clearance source changed"
    );
    let plan: Proposal = serde_json::from_str(source)?;
    ensure!(
        plan.schema_version == 1 && serde_json::to_value(&base.identity)? == plan.parent_identity,
        "reviewed parent identity required"
    );
    ensure!(
        plan.maps.len() == 4
            && plan.edits.len() == 30
            && !base.report.manifests.iter().any(|m| m == MARKER),
        "four chunks and thirty original edits required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let mut result = base.clone();
    let mut seen_maps = BTreeSet::new();
    let mut applied = BTreeSet::new();
    let mut old_rocks = 0;
    let mut new_rocks = 0;
    for terrain in &plan.maps {
        let (column, row, size) = match terrain.id.as_str() {
            "MnR09C00R00" => (0, 0, [32, 24]),
            "MnR09C01R00" => (1, 0, [32, 24]),
            "MnR09C00R01" => (0, 1, [32, 12]),
            "MnR09C01R01" => (1, 1, [32, 12]),
            _ => anyhow::bail!("only the four complete R09 chunks may change"),
        };
        ensure!(
            seen_maps.insert(&terrain.id)
                && terrain.tileset == TILESET
                && terrain.dimensions_metatiles == size,
            "chunk identity or private winter art differs"
        );
        let original = base
            .data
            .maps
            .get(&terrain.id)
            .context("native R09 module")?;
        ensure!(
            [original.attributes.width, original.attributes.height] == size
                && original.attributes.tileset_name == TILESET,
            "installed R09 dimensions or art differs"
        );
        ensure!(
            format!(
                "{:x}",
                Sha256::digest(serde_json::to_vec(&serde_json::to_value(original)?)?)
            ) == terrain.native_module_sha256,
            "complete R09 module preimage differs"
        );
        ensure!(
            original.blocks == rows(&terrain.before_rows, size)?,
            "complete original terrain differs"
        );
        let mut expected = original.blocks.clone();
        for edit in &plan.edits {
            let [x, y] = edit.metatile;
            if x / 32 != column || y / 24 != row {
                continue;
            }
            ensure!(
                applied.insert(edit.metatile) && y % 24 < size[1],
                "duplicate or off-map edit"
            );
            let index = usize::from(y % 24) * usize::from(size[0]) + usize::from(x % 32);
            let before = block(&edit.before)?;
            let after = block(&edit.after)?;
            ensure!(
                expected[index] == before
                    && ((before == 0x0a
                        && matches!(after, 0x05 | 0x70 | 0x71 | 0x72 | 0x6e | 0x6d | 0x6b))
                        || (before == 0x71 && after == 0x0a)),
                "only declared rock/edge/ordinary-floor cells may change"
            );
            expected[index] = after;
        }
        ensure!(
            expected == rows(&terrain.after_rows, size)?,
            "unlisted terrain change"
        );
        old_rocks += original.blocks.iter().filter(|&&id| id == 0x0a).count();
        new_rocks += expected.iter().filter(|&&id| id == 0x0a).count();
        let bytes = expected
            .iter()
            .copied()
            .map(u8::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let label = original
            .attributes
            .blocks_label
            .as_ref()
            .context("R09 blocks label")?;
        result
            .data
            .maps
            .get_mut(&terrain.id)
            .context("R09 child map")?
            .blocks = expected;
        result
            .data
            .map_blocks
            .insert(label.clone(), crate::encode_base64_bytes(&bytes));
    }
    ensure!(
        applied.len() == 30 && old_rocks == 37 && new_rocks == 31,
        "complete declared rock relocation required"
    );
    result.report.manifests.push(MARKER.into());
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    crate::validate_compact_superior_geography(&result)?;
    let mut residual = result.clone();
    for terrain in &plan.maps {
        let original = &base.data.maps[&terrain.id];
        let label = original
            .attributes
            .blocks_label
            .as_ref()
            .context("original blocks label")?;
        residual
            .data
            .maps
            .insert(terrain.id.clone(), original.clone());
        residual.data.map_blocks.insert(
            label.clone(),
            base.data
                .map_blocks
                .get(label)
                .context("original block mirror")?
                .clone(),
        );
    }
    residual.report = base.report.clone();
    residual.identity = base.identity.clone();
    ensure!(
        residual == *base,
        "unrelated content changed during rock clearance"
    );
    Ok(result)
}
