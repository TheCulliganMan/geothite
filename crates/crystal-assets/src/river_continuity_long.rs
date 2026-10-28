//! Publish the frozen long wooded rivers without rebuilding native programs.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{CompiledGamePack, MapModule};

const CONTRACT_SHA: &str = "67c451f315baf3939f92ae619aae44c50963c069633dab177076ee18b99cd3cd";
const PARENT_SHA: &str = "57eefb675b9cb32ac66a01819f56961392e51678a6f9dd959888fbb9427e4e62";
const PARENT_IDENTITY: &str = "6ba03931721e00e50adbf8f058ef81f0ab9a94b75c2b8899ded9a198c75ffb2b";

#[derive(Deserialize)]
struct Contract {
    schema_version: u16,
    parent_pack_sha256: String,
    parent_identity: String,
    changes: Vec<Change>,
}

#[derive(Deserialize)]
struct Change {
    map: String,
    native_module_sha256: String,
    dimensions_metatiles: [u16; 2],
    cells: Vec<Cell>,
}

#[derive(Deserialize)]
struct Cell {
    metatile: [u16; 2],
    before: String,
    after: String,
}

/// Applies the reviewed 1,678 cells while preserving an approved actual parent.
///
/// Changes only 14 map block arrays and their encoded block payloads. A fifteenth
/// map is guarded but unchanged. All native programs, events, attributes,
/// connections, actors, art, audio, report, runtime files and manifest stay exact.
/// The contract identifies the source baseline. An approved later parent may
/// contain unrelated changes while all 15 native map preimages remain exact.
/// The caller must bind its actual input file hash, publish to a fresh path and
/// verify the written readback.
/// This does not certify controller traversal, visual acceptance or progression.
///
/// # Errors
/// Rejects a stale parent or source contract, changed module preimages, duplicate
/// or unsupported cells, shared block payloads, inconsistent cached/uncached
/// mirrors, or any change outside the frozen terrain contract. The borrowed
/// parent is unchanged on success and failure.
pub fn build_river_continuity_long_pack(
    base: &CompiledGamePack,
    contract_json: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        hash(contract_json.as_bytes()) == CONTRACT_SHA,
        "frozen long-river contract required"
    );
    let contract: Contract = serde_json::from_str(contract_json).context("long-river contract")?;
    ensure!(
        contract.schema_version == 1
            && contract.parent_pack_sha256 == PARENT_SHA
            && contract.parent_identity == PARENT_IDENTITY
            && base.data.maps.len() == 683
            && contract.changes.len() == 15
            && contract
                .changes
                .iter()
                .filter(|c| !c.cells.is_empty())
                .count()
                == 14
            && contract
                .changes
                .iter()
                .map(|c| c.cells.len())
                .sum::<usize>()
                == 1678,
        "683-map long-river profile and frozen source baseline required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity == identity(base)?,
        "long-river parent identity is stale"
    );
    let mut output = base.clone();
    for change in &contract.changes {
        let original = base
            .data
            .maps
            .get(&change.map)
            .context("original river module")?;
        ensure!(
            hash(&serde_json::to_vec(&serde_json::to_value(original)?)?)
                == change.native_module_sha256,
            "long-river typed module hash differs: {}",
            change.map
        );
        let expected_tileset = if change.map.starts_with("MnR05") {
            "mn_headwaters"
        } else {
            "johto"
        };
        ensure!(
            [original.attributes.width, original.attributes.height] == change.dimensions_metatiles
                && original.attributes.tileset_name == expected_tileset,
            "long-river native dimension/tileset profile differs: {}",
            change.map
        );
        ensure!(
            base.data.map_attributes.get(&change.map) == Some(&original.attributes),
            "long-river attribute mirror differs: {}",
            change.map
        );
        let label = original
            .attributes
            .blocks_label
            .as_ref()
            .context("river block label")?;
        ensure!(
            base.data.maps.iter().all(|(name, m)| name == &change.map
                || m.attributes.blocks_label.as_ref() != Some(label)),
            "shared long-river block payload"
        );
        let old_payload = base
            .data
            .map_blocks
            .get(label)
            .context("original river block payload")?;
        ensure!(
            crate::decode_base64_bytes(old_payload)?
                .into_iter()
                .map(u16::from)
                .eq(original.blocks.iter().copied()),
            "original long-river block mirror differs"
        );
        let module = apply_cells(base, original, change)?;
        if !change.cells.is_empty() {
            output
                .data
                .map_blocks
                .insert(label.clone(), encode_blocks(&module)?);
            output.data.maps.insert(change.map.clone(), module);
        }
    }
    output.identity = identity(&output)?;
    preserve(base, &output, &contract)?;
    crate::verify_compiled_game_pack_for_runtime(&output)?;
    Ok(output)
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn identity(pack: &CompiledGamePack) -> Result<crate::CompiledGamePackIdentity> {
    crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )
}

fn encode_blocks(module: &MapModule) -> Result<String> {
    let bytes = module
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(crate::encode_base64_bytes(&bytes))
}

fn apply_cells(
    base: &CompiledGamePack,
    original: &MapModule,
    change: &Change,
) -> Result<MapModule> {
    let mut module = original.clone();
    let mut visited = BTreeSet::new();
    let tileset = base
        .data
        .tilesets
        .get(&original.attributes.tileset_name)
        .context("original native river collision")?;
    for cell in &change.cells {
        let [x, y] = cell.metatile;
        ensure!(
            x < original.attributes.width
                && y < original.attributes.height
                && visited.insert([x, y]),
            "invalid or duplicate long-river cell"
        );
        let before = u16::from_str_radix(&cell.before, 16)?;
        let after = u16::from_str_radix(&cell.after, 16)?;
        let index = usize::from(y) * usize::from(original.attributes.width) + usize::from(x);
        ensure!(
            module.blocks.get(index) == Some(&before)
                && before != after
                && u8::try_from(after).is_ok(),
            "stale/no-op long-river cell"
        );
        ensure!(
            tileset.collision.contains_key(&format!("{after:02x}")),
            "unsupported river metatile"
        );
        module.blocks[index] = after;
    }
    Ok(module)
}

fn preserve(base: &CompiledGamePack, output: &CompiledGamePack, contract: &Contract) -> Result<()> {
    let mut normalized = output.clone();
    for change in &contract.changes {
        let old = base
            .data
            .maps
            .get(&change.map)
            .context("old river module")?;
        let expected = apply_cells(base, old, change)?;
        ensure!(
            output.data.maps.get(&change.map) == Some(&expected),
            "river module changed beyond listed cells"
        );
        let label = old
            .attributes
            .blocks_label
            .as_ref()
            .context("old river label")?;
        let old_payload = base
            .data
            .map_blocks
            .get(label)
            .context("old river payload")?;
        let expected_payload = if change.cells.is_empty() {
            old_payload.clone()
        } else {
            encode_blocks(&expected)?
        };
        ensure!(
            output.data.map_blocks.get(label) == Some(&expected_payload),
            "river block mirror drift"
        );
        normalized.data.maps.insert(change.map.clone(), old.clone());
        normalized
            .data
            .map_blocks
            .insert(label.clone(), old_payload.clone());
    }
    normalized.identity = base.identity.clone();
    ensure!(
        normalized == *base,
        "long river changed unrelated maps/programs/art/audio/report"
    );
    ensure!(
        output.identity == identity(output)?,
        "long-river output identity is stale"
    );
    let mut before = base.data.clone();
    let mut after = output.data.clone();
    for change in &contract.changes {
        before
            .maps
            .remove(&change.map)
            .context("remove old cached river")?;
        after
            .maps
            .remove(&change.map)
            .context("remove new cached river")?;
    }
    for change in &contract.changes {
        let mut expected = before.assemble_map_module_from_compiled_payloads(&change.map)?;
        expected
            .blocks
            .clone_from(&output.data.maps[&change.map].blocks);
        ensure!(
            after.assemble_map_module_from_compiled_payloads(&change.map)? == expected,
            "uncached long-river terrain lost native program/event behavior"
        );
    }
    Ok(())
}

#[cfg(test)]
#[path = "river_continuity_long_tests.rs"]
mod tests;
