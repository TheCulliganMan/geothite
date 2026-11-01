//! Apply the frozen inland winter terrain after the exact coastal connection revision.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{CompiledGamePack, MapModule};

const CONTRACT_SHA: &str = "7e345514059a5333fe383db501a0d82785fc294cb9f9b8d398b46f5ffeea2d9d";
const ART_CONTRACT: &str = include_str!(
    "../../../planning/minnesota/revisions/north-shore-ely-geography-v01/inland-winter-r12-r13/source-v02/art-and-geography.preimages.json"
);
const ART_SHA: &str = "a6d09552382045ba5e334fb2fc5a9b2ba42a43783a70a14f6f7590ea426f0511";

#[derive(Deserialize)]
struct Contract {
    schema_version: u16,
    id: String,
    changed_metatiles: usize,
    modules: Vec<Module>,
}

#[derive(Deserialize)]
struct Module {
    map: String,
    dimensions_metatiles: [u16; 2],
    module_sha256: String,
    candidate_module_sha256: String,
    old_tileset: String,
    new_tileset: String,
    blocks_label: String,
    old_blocks_sha256: String,
    new_blocks_sha256: String,
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

/// Applies exactly 4,586 reviewed R12/R13 winter cells and nine map aliases.
///
/// Requires the final geography's sole R12 east-connection replacement. Every
/// original full module remains guarded after reversing that exact replacement
/// for comparison. Preserves native lake water, field-move behavior, scripts,
/// audio and all unrelated parent data. Only forty FLOOR quadrants become ICE.
/// This builder does not establish controller or visual acceptance.
///
/// # Errors
/// Rejects stale source contracts, art, identities, map/block preimages, missing
/// final geography, unsupported collision changes or unrelated mutations.
/// The borrowed parent remains unchanged on success and failure.
pub fn build_inland_winter_pack(
    base: &CompiledGamePack,
    contract_json: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        hash(contract_json.as_bytes()) == CONTRACT_SHA,
        "frozen inland winter contract required"
    );
    let contract: Contract =
        serde_json::from_str(contract_json).context("inland winter contract")?;
    ensure!(
        contract.schema_version == 1
            && contract.id == "MN_INLAND_R12_R13_WINTER"
            && base.data.maps.len() == 694
            && contract.modules.len() == 9
            && contract.changed_metatiles == 4586
            && contract
                .modules
                .iter()
                .map(|m| m.cells.len())
                .sum::<usize>()
                == 4586,
        "bounded nine-module inland winter required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity == identity(base)?,
        "inland winter parent identity is stale"
    );
    art_preimages(base)?;
    let mut output = base.clone();
    let mut semantic_changes = 0;
    for row in &contract.modules {
        let old = base
            .data
            .maps
            .get(&row.map)
            .context("old inland winter module")?;
        ensure!(
            json_hash(&baseline(old)?)? == row.module_sha256
                && [old.attributes.width, old.attributes.height] == row.dimensions_metatiles
                && old.attributes.tileset_name == row.old_tileset
                && ((row.map.starts_with("MnR12")
                    && row.old_tileset == "mn_gunflint"
                    && row.new_tileset == "mn_winter_gunflint")
                    || (row.map.starts_with("MnR13")
                        && row.old_tileset == "johto"
                        && row.new_tileset == "mn_winter_north_shore"))
                && old.attributes.blocks_label.as_deref() == Some(row.blocks_label.as_str()),
            "inland winter owned preimage differs: {}",
            row.map
        );
        ensure!(
            base.data.map_attributes.get(&row.map) == Some(&old.attributes)
                && base.data.maps.iter().all(|(name, m)| name == &row.map
                    || m.attributes.blocks_label.as_ref() != Some(&row.blocks_label)),
            "inland winter attribute mirror or block isolation differs"
        );
        ensure!(
            crate::decode_base64_bytes(
                base.data
                    .map_blocks
                    .get(&row.blocks_label)
                    .context("old encoded winter blocks")?
            )?
            .into_iter()
            .map(u16::from)
            .eq(old.blocks.iter().copied()),
            "old winter block mirror differs"
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
        semantic_changes == 40,
        "only40 FLOOR-to-ICE quadrants may change semantics"
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

fn json_hash(value: &impl serde::Serialize) -> Result<String> {
    Ok(hash(&serde_json::to_vec(&serde_json::to_value(value)?)?))
}

// This is an exact, reversible preimage rebind, never an omitted module guard.
fn baseline(module: &MapModule) -> Result<MapModule> {
    let mut original = module.clone();
    if module.id == "MnR12C01R01" {
        let east = original
            .attributes
            .connections
            .iter_mut()
            .find(|c| c.direction == "east")
            .context("final inland east seam")?;
        ensure!(
            east.target_map == "MnMaraisInlandNeck" && east.offset == 0,
            "final geography R12 east seam required"
        );
        east.target_map = "MnGrandMarais".into();
    }
    Ok(original)
}

fn art_preimages(base: &CompiledGamePack) -> Result<()> {
    ensure!(
        hash(ART_CONTRACT.as_bytes()) == ART_SHA,
        "frozen inland art guards required"
    );
    let guard: serde_json::Value = serde_json::from_str(ART_CONTRACT)?;
    for (name, expected) in guard["tilesets_sha256"]
        .as_object()
        .context("tileset guards")?
    {
        ensure!(
            json_hash(
                base.data
                    .tilesets
                    .get(name)
                    .context("native winter tileset")?
            )? == expected.as_str().context("tileset hash")?,
            "winter tileset preimage differs: {name}"
        );
    }
    for (path, expected) in guard["runtime_files_sha256"]
        .as_object()
        .context("art file guards")?
    {
        ensure!(
            hash(base.runtime_files.get(path).context("winter art file")?)
                == expected.as_str().context("file hash")?,
            "winter art file differs: {path}"
        );
    }
    Ok(())
}

fn apply(base: &CompiledGamePack, old: &MapModule, row: &Module) -> Result<MapModule> {
    let mut new = old.clone();
    let mut seen = BTreeSet::new();
    let before = &base.data.tilesets[&row.old_tileset].collision;
    let after = &base.data.tilesets[&row.new_tileset].collision;
    ensure!(
        hash(
            &old.blocks
                .iter()
                .copied()
                .map(u8::try_from)
                .collect::<std::result::Result<Vec<_>, _>>()?
        ) == row.old_blocks_sha256,
        "winter block preimage differs"
    );
    for cell in &row.cells {
        let [x, y] = cell.metatile;
        ensure!(
            x < old.attributes.width
                && y < old.attributes.height
                && seen.insert([x, y])
                && cell.before != cell.after
                && u8::try_from(cell.after).is_ok(),
            "invalid/duplicate winter terrain cell"
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
                    .all(|(a, b)| a == b || (a == "FLOOR" && b == "ICE")),
            "winter terrain changes an unreviewed quadrant or stale cell"
        );
        new.blocks[index] = cell.after;
    }
    new.attributes.tileset_name = row.new_tileset.clone();
    ensure!(
        json_hash(&{
            let mut candidate = baseline(&new)?;
            candidate.attributes.tileset_name = row.new_tileset.clone();
            candidate
        })? == row.candidate_module_sha256
            && hash(
                &new.blocks
                    .iter()
                    .copied()
                    .map(u8::try_from)
                    .collect::<std::result::Result<Vec<_>, _>>()?
            ) == row.new_blocks_sha256,
        "winter module differs from frozen native candidate"
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
            "winter module/attribute/block mirrors changed beyond winter terrain"
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
        "inland winter changed unrelated maps/programs/art/flags/audio/report"
    );
    ensure!(
        output.identity == identity(output)?,
        "inland winter output identity is stale"
    );
    let mut before = base.data.clone();
    let mut after = output.data.clone();
    for row in &contract.modules {
        before
            .maps
            .remove(&row.map)
            .context("remove old winter cache")?;
        after
            .maps
            .remove(&row.map)
            .context("remove new winter cache")?;
    }
    for row in &contract.modules {
        // The inherited wire and typed program caches are not assumed equal.
        // Compare each representation to its own parent with only owned edits.
        let mut expected = before.assemble_map_module_from_compiled_payloads(&row.map)?;
        ensure!(
            expected.blocks == base.data.maps[&row.map].blocks,
            "old uncached winter blocks differ"
        );
        expected
            .blocks
            .clone_from(&output.data.maps[&row.map].blocks);
        expected.attributes.tileset_name = row.new_tileset.clone();
        ensure!(
            after.assemble_map_module_from_compiled_payloads(&row.map)? == expected,
            "uncached inland winter terrain changed inherited native programs/metadata"
        );
    }
    Ok(())
}

#[cfg(test)]
#[path = "inland_winter_tests.rs"]
mod tests;
