//! Restore vegetation and winding trails on the two River Bluffs terraces.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{CompiledGamePack, MapModule};

const SOURCE: &str = "926107619b1366f4535bc4f237e4d53a0c2790161096e4371776941c93241bc8";
const PARENT: &str = "229973e1cc11df7932bde0344ce4776fe4fb67d04f5fafee8f83142447f3a5ec";
const MARKER: &str = "minnesota-r05-wooded-terraces-v1";

#[derive(Deserialize)]
struct Proposal {
    schema_version: u8,
    route: String,
    dimensions_metatiles: [u16; 2],
    parent_content_hash: String,
    source_pixels_modified: bool,
    maps: Vec<Chunk>,
    cliff_repairs: Vec<CliffRepair>,
}

#[derive(Deserialize)]
struct Chunk {
    id: String,
    tileset: String,
    dimensions_metatiles: [u16; 2],
    native_module_sha256: String,
    before_rows: Vec<String>,
    rows: Vec<String>,
    changed_metatiles: usize,
}

#[derive(Deserialize)]
struct CliffRepair {
    map: String,
    metatile: [usize; 2],
    before: String,
    after: String,
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn blocks(rows: &[String], [width, height]: [u16; 2]) -> Result<Vec<u16>> {
    ensure!(
        rows.len() == usize::from(height),
        "complete terrace rows required"
    );
    let mut blocks = Vec::with_capacity(usize::from(width) * usize::from(height));
    for row in rows {
        let row = row
            .split_whitespace()
            .map(|word| u8::from_str_radix(word, 16).map(u16::from))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(
            row.len() == usize::from(width),
            "complete terrace columns required"
        );
        blocks.extend(row);
    }
    Ok(blocks)
}

fn replace_blocks(pack: &mut CompiledGamePack, module: MapModule) -> Result<()> {
    let label = module
        .attributes
        .blocks_label
        .as_ref()
        .context("terrace blocks label")?;
    let bytes = module
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    *pack
        .data
        .map_blocks
        .get_mut(label)
        .context("existing terrace block payload")? = crate::encode_base64_bytes(&bytes);
    pack.data.maps.insert(module.id.clone(), module);
    Ok(())
}

/// Publishes wooded trail surfaces on the two River Bluffs terraces.
///
/// Changes only 135 native metatiles and their encoded mirrors. Complete route
/// chunks are bound to the reviewed parent. Continuous cliff rims replace old boulder rims. Stairs, cave mouths, roads
/// at map boundaries, river connections, fishing, actors, programs, fauna, source
/// art and audio remain exact. Returns an unselected candidate without migrating
/// saved progress.
///
/// # Errors
/// Rejects source or parent drift, incomplete native preimages, shared block
/// labels, mirror drift, edits outside terrace surfaces and unrelated mutation.
/// Borrowed inputs remain unchanged on failure.
pub fn revise_r05_wooded_terraces(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        hash(source.as_bytes()) == SOURCE,
        "reviewed terrace source differs"
    );
    let plan: Proposal = serde_json::from_str(source)?;
    ensure!(
        plan.schema_version == 1
            && plan.route == "MN_R05"
            && plan.dimensions_metatiles == [32, 56]
            && !plan.source_pixels_modified
            && plan.parent_content_hash == PARENT
            && base.identity.content_hash == PARENT
            && plan.maps.len() == 3
            && !base.report.manifests.iter().any(|m| m == MARKER),
        "current complete River Bluffs parent required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let mut child = base.clone();
    for (row, record) in plan.maps.iter().enumerate() {
        let tileset = if row == 2 {
            "mn_headwaters"
        } else {
            "mn_outdoor_headwaters_stairs"
        };
        ensure!(
            record.id == format!("MnR05C00R{row:02}")
                && record.tileset == tileset
                && record.dimensions_metatiles == [32, if row == 2 { 8 } else { 24 }],
            "complete native River Bluffs chunk required"
        );
        let original = base
            .data
            .maps
            .get(&record.id)
            .context("installed terrace chunk")?;
        ensure!(
            hash(&serde_json::to_vec(&serde_json::to_value(original)?)?)
                == record.native_module_sha256
                && original.blocks == blocks(&record.before_rows, record.dimensions_metatiles)?
                && original.attributes.tileset_name == record.tileset
                && [original.attributes.width, original.attributes.height]
                    == record.dimensions_metatiles,
            "complete terrace preimage differs"
        );
        let label = original
            .attributes
            .blocks_label
            .as_ref()
            .context("terrace blocks")?;
        ensure!(
            base.data
                .maps
                .values()
                .filter(|m| m.attributes.blocks_label.as_ref() == Some(label))
                .count()
                == 1,
            "shared terrace blocks label"
        );
        let bytes = original
            .blocks
            .iter()
            .copied()
            .map(u8::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(
            base.data.map_blocks.get(label) == Some(&crate::encode_base64_bytes(&bytes)),
            "terrace encoded preimage differs"
        );
        let mut revised = original.clone();
        revised.blocks = blocks(&record.rows, record.dimensions_metatiles)?;
        let changed = original
            .blocks
            .iter()
            .zip(&revised.blocks)
            .enumerate()
            .filter_map(|(i, (a, b))| (a != b).then_some(i))
            .collect::<Vec<_>>();
        ensure!(
            record.changed_metatiles == [90, 45, 0][row]
                && changed.len() == record.changed_metatiles,
            "terrace terrain delta differs"
        );
        for repair in plan
            .cliff_repairs
            .iter()
            .filter(|repair| repair.map == record.id)
        {
            let [x, y] = repair.metatile;
            ensure!(
                x < 32 && y < usize::from(record.dimensions_metatiles[1]),
                "cliff repair outside chunk"
            );
            let i = y * 32 + x;
            ensure!(
                original.blocks[i] == u16::from_str_radix(&repair.before, 16)?
                    && revised.blocks[i] == u16::from_str_radix(&repair.after, 16)?,
                "cliff repair preimage differs"
            );
        }
        ensure!(
            changed.iter().all(|&i| {
                let owned_cliff = plan
                    .cliff_repairs
                    .iter()
                    .any(|repair| repair.map == record.id && repair.metatile == [i % 32, i / 32]);
                owned_cliff
                    || ((6..=20).contains(&(i % 32))
                        && (if row == 0 { 8..=19 } else { 3..=13 }).contains(&(i / 32))
                        && matches!(original.blocks[i], 1..=4 | 0x71)
                        && matches!(revised.blocks[i], 1..=5))
            }),
            "protected terrace geometry changed"
        );
        if !changed.is_empty() {
            replace_blocks(&mut child, revised)?;
        }
    }
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
    for row in 0..2 {
        replace_blocks(
            &mut residual,
            base.data.maps[&format!("MnR05C00R{row:02}")].clone(),
        )?;
    }
    residual.identity = base.identity.clone();
    residual.report = base.report.clone();
    ensure!(residual == *base, "unrelated River Bluffs content changed");
    Ok(child)
}
