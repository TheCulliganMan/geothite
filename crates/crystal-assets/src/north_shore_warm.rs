//! Reverse only the reviewed North Shore winter terrain and map aliases.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{CompiledGamePack, MapModule};

const CONTRACT_SHA: &str = "ad3d97771d1bd0c2e4bcba253a3dba7e060ee7996f8e723527796644fdff0976";
const WARM: &str = "mn_north_shore";
const COLD: &str = "mn_winter_north_shore";
const WARM_DEFINITION: &str = "979d36ed12ff70200b7796722471921775a544bb13b97bd29bd0622c18e7787e";
const ART: [(&str, &str); 6] = [
    (
        "data/tilesets/mn_north_shore.json",
        "c169898d6a17f0dbcd58bbe5ba774bed5ad093f65efb73ecf35431f900defce4",
    ),
    (
        "data/tilesets/mn_north_shore_metatiles.bin",
        "1c735fded81b29e8920ead944b89bce687179dea04d4e40080a3fea6f397e049",
    ),
    (
        "data/tilesets/mn_north_shore_palette_map.json",
        "a80187b7161fd5e10ce29de2cef249df493eb2734ed57a1fe3d808bd0f8127b6",
    ),
    (
        "gfx/tilesets/mn_north_shore.2bpp",
        "72661575e29288cb9ecc73e14ce0b63aef763b2a085b900f2f910c2a2c89f49d",
    ),
    (
        "gfx/tilesets/mn_north_shore.png",
        "a746f27dc358773e2406ae137caf58fed5a3d9a3df19061757de1cbcee516587",
    ),
    (
        "gfx/tilesets/bg_tiles.pal",
        "83ff1f3f9ab57e9d9cb021acb7689c169a7b1d6ebe78da4a294b8f4f7dc11bed",
    ),
];

#[derive(Deserialize)]
struct Contract {
    schema_version: u16,
    id: String,
    expected_map_count: usize,
    changed_metatiles: usize,
    modules: Vec<Module>,
}

#[derive(Deserialize)]
struct Module {
    map: String,
    dimensions_metatiles: [u16; 2],
    module_sha256: String,
    warm_reference_module_sha256: String,
    old_tileset: String,
    new_tileset: String,
    blocks_label: String,
    cells: Vec<Cell>,
}

#[derive(Deserialize)]
struct Cell {
    metatile: [u16; 2],
    before: u16,
    after: u16,
    before_quadrants: [String; 4],
    after_quadrants: [String; 4],
}

/// Restores the exact 828 warm native cells and two North Shore map aliases.
///
/// Requires all eight frozen R11 module preimages, the original warm art, and a
/// valid derived actual-parent identity. Unrelated approved parent revisions
/// are retained. Changes only two block arrays/encoded mirrors and their two
/// tileset names/attribute mirrors, plus derived identity. Removes 52 forced
/// ICE quadrants; every other native collision token, actor, script, flag,
/// connection, room, art, audio and report stays exact. Shared winter art and
/// inland winter maps remain installed. This does not implement the new direct
/// Grand Marais connection or establish controller/GPU/campaign acceptance.
///
/// # Errors
/// Rejects stale contracts, parent identities, owned maps/art, shared payloads,
/// invalid cells, mirror mismatches or changes beyond the frozen restoration.
/// The borrowed parent is unchanged on success and failure.
pub fn build_north_shore_warm_pack(
    base: &CompiledGamePack,
    contract_json: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        hash(contract_json.as_bytes()) == CONTRACT_SHA,
        "frozen warm-coast contract required"
    );
    let contract: Contract = serde_json::from_str(contract_json).context("warm-coast contract")?;
    ensure!(
        contract.schema_version == 1
            && contract.id == "MN_R11_WARM_COAST_RESTORATION"
            && contract.expected_map_count == 683
            && base.data.maps.len() == 683
            && contract.modules.len() == 8
            && contract.changed_metatiles == 828
            && contract
                .modules
                .iter()
                .map(|m| m.cells.len())
                .sum::<usize>()
                == 828
            && contract
                .modules
                .iter()
                .filter(|m| !m.cells.is_empty())
                .count()
                == 2,
        "bounded eight-module warm restoration required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity == identity(base)?,
        "warm-coast parent identity is stale"
    );
    art_preimages(base)?;
    let mut output = base.clone();
    let mut semantic_changes = 0;
    for row in &contract.modules {
        let old = base
            .data
            .maps
            .get(&row.map)
            .context("old North Shore module")?;
        ensure!(
            hash(&serde_json::to_vec(&serde_json::to_value(old)?)?) == row.module_sha256
                && [old.attributes.width, old.attributes.height] == row.dimensions_metatiles
                && old.attributes.tileset_name == row.old_tileset
                && row.new_tileset == WARM
                && row.old_tileset == if row.cells.is_empty() { WARM } else { COLD }
                && old.attributes.blocks_label.as_deref() == Some(row.blocks_label.as_str()),
            "North Shore owned preimage differs: {}",
            row.map
        );
        ensure!(
            base.data.map_attributes.get(&row.map) == Some(&old.attributes)
                && base.data.maps.iter().all(|(name, m)| name == &row.map
                    || m.attributes.blocks_label.as_ref() != Some(&row.blocks_label)),
            "North Shore attribute mirror or block isolation differs"
        );
        ensure!(
            crate::decode_base64_bytes(
                base.data
                    .map_blocks
                    .get(&row.blocks_label)
                    .context("old encoded shore blocks")?
            )?
            .into_iter()
            .map(u16::from)
            .eq(old.blocks.iter().copied()),
            "old shore block mirror differs"
        );
        let new = apply(base, old, row)?;
        semantic_changes += row
            .cells
            .iter()
            .flat_map(|c| c.before_quadrants.iter().zip(&c.after_quadrants))
            .filter(|(a, b)| a != b)
            .count();
        if !row.cells.is_empty() {
            output
                .data
                .map_blocks
                .insert(row.blocks_label.clone(), encode_blocks(&new)?);
            output
                .data
                .map_attributes
                .insert(row.map.clone(), new.attributes.clone());
            output.data.maps.insert(row.map.clone(), new);
        }
    }
    ensure!(
        semantic_changes == 52,
        "only52 ICE-to-Floor quadrants may change semantics"
    );
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

fn art_preimages(base: &CompiledGamePack) -> Result<()> {
    let warm = base
        .data
        .tilesets
        .get(WARM)
        .context("original warm North Shore art")?;
    let cold = base
        .data
        .tilesets
        .get(COLD)
        .context("installed cold North Shore art")?;
    ensure!(
        hash(&serde_json::to_vec(&serde_json::to_value(warm)?)?) == WARM_DEFINITION,
        "warm North Shore collision/palette preimage differs"
    );
    for (path, expected) in ART {
        ensure!(
            hash(
                base.runtime_files
                    .get(path)
                    .context("original shore art file")?
            ) == expected,
            "warm shore art preimage differs: {path}"
        );
    }
    ensure!(
        warm.collision
            .iter()
            .all(|(id, q)| cold.collision.get(id) == Some(q)),
        "cold alias changed original native collision prefix"
    );
    Ok(())
}

fn apply(base: &CompiledGamePack, old: &MapModule, row: &Module) -> Result<MapModule> {
    let mut new = old.clone();
    let mut seen = BTreeSet::new();
    let before = &base.data.tilesets[&row.old_tileset].collision;
    let after = &base.data.tilesets[WARM].collision;
    for cell in &row.cells {
        let [x, y] = cell.metatile;
        ensure!(
            x < old.attributes.width
                && y < old.attributes.height
                && seen.insert([x, y])
                && cell.before >= 0x83
                && cell.after < 0x83,
            "invalid/duplicate restoration cell"
        );
        let index = usize::from(y) * usize::from(old.attributes.width) + usize::from(x);
        ensure!(
            new.blocks.get(index) == Some(&cell.before)
                && before
                    .get(&format!("{:02x}", cell.before))
                    .map(Vec::as_slice)
                    == Some(cell.before_quadrants.as_slice())
                && after.get(&format!("{:02x}", cell.after)).map(Vec::as_slice)
                    == Some(cell.after_quadrants.as_slice())
                && cell
                    .before_quadrants
                    .iter()
                    .zip(&cell.after_quadrants)
                    .all(|(a, b)| a == b || (a == "ICE" && b == "FLOOR")),
            "restoration changes an unreviewed quadrant or stale cell"
        );
        new.blocks[index] = cell.after;
    }
    new.attributes.tileset_name = WARM.into();
    ensure!(
        hash(&serde_json::to_vec(&serde_json::to_value(&new)?)?)
            == row.warm_reference_module_sha256
            && new.blocks.iter().all(|b| *b < 0x83),
        "restored module differs from actual warm native reference"
    );
    Ok(new)
}

fn preserve(base: &CompiledGamePack, output: &CompiledGamePack, contract: &Contract) -> Result<()> {
    let mut normalized = output.clone();
    for row in &contract.modules {
        let old = &base.data.maps[&row.map];
        let expected = apply(base, old, row)?;
        let payload = if row.cells.is_empty() {
            base.data.map_blocks[&row.blocks_label].clone()
        } else {
            encode_blocks(&expected)?
        };
        ensure!(
            output.data.maps.get(&row.map) == Some(&expected)
                && output.data.map_attributes.get(&row.map) == Some(&expected.attributes)
                && output.data.map_blocks.get(&row.blocks_label) == Some(&payload),
            "shore module/attribute/block mirrors changed beyond restoration"
        );
        normalized.data.maps.insert(row.map.clone(), old.clone());
        normalized
            .data
            .map_attributes
            .insert(row.map.clone(), base.data.map_attributes[&row.map].clone());
        normalized.data.map_blocks.insert(
            row.blocks_label.clone(),
            base.data.map_blocks[&row.blocks_label].clone(),
        );
    }
    normalized.identity = base.identity.clone();
    ensure!(
        normalized == *base,
        "warm shore changed unrelated maps/programs/art/flags/audio/report"
    );
    ensure!(
        output.identity == identity(output)?,
        "warm-coast output identity is stale"
    );
    let mut before = base.data.clone();
    let mut after = output.data.clone();
    for row in &contract.modules {
        before
            .maps
            .remove(&row.map)
            .context("remove old shore cache")?;
        after
            .maps
            .remove(&row.map)
            .context("remove new shore cache")?;
    }
    for row in &contract.modules {
        // The inherited wire and typed program caches are not assumed equal.
        // Compare each representation to its own parent with only owned edits.
        let mut expected = before.assemble_map_module_from_compiled_payloads(&row.map)?;
        ensure!(
            expected.blocks == base.data.maps[&row.map].blocks,
            "old uncached shore blocks differ"
        );
        expected
            .blocks
            .clone_from(&output.data.maps[&row.map].blocks);
        expected.attributes.tileset_name = WARM.into();
        ensure!(
            after.assemble_map_module_from_compiled_payloads(&row.map)? == expected,
            "uncached shore restoration changed inherited native programs/metadata"
        );
    }
    Ok(())
}

#[cfg(test)]
#[path = "north_shore_warm_tests.rs"]
mod tests;
