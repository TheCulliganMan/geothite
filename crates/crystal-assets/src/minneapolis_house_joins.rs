//! Correct misplaced exterior endcaps and upper-floor foundations in Minneapolis.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{CompiledGamePack, TilesetDefinition};

const MAP: &str = "MnMinneapolis";
const TILESET: &str = "mn_minneapolis_city";
const BASE_FILE: &str = "57eefb675b9cb32ac66a01819f56961392e51678a6f9dd959888fbb9427e4e62";
const BASE_IDENTITY: &str = "6ba03931721e00e50adbf8f058ef81f0ab9a94b75c2b8899ded9a198c75ffb2b";
const CONTRACT_SHA: &str = "6d58489a641a58173a68dc598cd774b9fcf3ddaaaeac974f13c8fdb7d7a49a85";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Contract {
    schema_version: u16,
    base_pack_sha256: String,
    base_identity: String,
    base_map_count: usize,
    map: String,
    tileset: String,
    expected_blocks: usize,
    base_module_sha256: String,
    base_definition_sha256: String,
    source_files_sha256: BTreeMap<String, String>,
    blocks: Vec<Block>,
    cells: Vec<Cell>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Block {
    target: u16,
    glyphs: [u8; 16],
    collision: [String; 4],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Cell {
    metatile: [u16; 2],
    before: u16,
    after: u16,
}

/// Repairs native facade joins across the reviewed Minneapolis house fronts.
///
/// Requires the frozen first river-bend parent and v04 correction contract.
/// Twelve appended metatiles use only existing logical glyphs. No bitmap,
/// palette, old metatile or old collision is changed. Eighty-three map block
/// identifiers and their encoded mirror change; every collision quadrant,
/// door, event, script, room, story flag, unrelated map, audio descriptor,
/// report and recovery record stays exact. Upper floors use native wall rows
/// without ground foundations; roof/wall endcaps occur only at outside sides.
/// This is native content construction, not controller or GPU acceptance.
///
/// # Errors
/// Rejects an unreviewed contract, stale native parent/source, occupied new
/// metatile address, new bitmap glyph reference, changed collision or unrelated
/// pack mutation. Cached and uncached native geometry must agree. The borrowed
/// parent remains unchanged on success or failure.
pub fn build_minneapolis_house_joins_pack(
    base: &CompiledGamePack,
    contract_json: &str,
) -> Result<CompiledGamePack> {
    let plan = parse_contract(base, contract_json)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let (definition, metatiles) = append_recipes(base, &plan)?;
    let original = base
        .data
        .maps
        .get(MAP)
        .context("Minneapolis source module")?;
    let label = original
        .attributes
        .blocks_label
        .as_deref()
        .context("city block label")?;
    ensure!(
        base.data.maps.iter().all(|(name, module)| {
            name == MAP || module.attributes.blocks_label.as_deref() != Some(label)
        }),
        "house block payload is shared"
    );
    let before = original
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    ensure!(
        base.data.map_blocks.get(label) == Some(&crate::encode_base64_bytes(&before)),
        "cached city and encoded block payload differ"
    );
    let mut module = original.clone();
    for cell in &plan.cells {
        ensure!(
            cell.metatile[0] < 40 && cell.metatile[1] < 32,
            "house cell outside city"
        );
        let index = usize::from(cell.metatile[1]) * 40 + usize::from(cell.metatile[0]);
        ensure!(
            module.blocks[index] == cell.before
                && base.data.tilesets[TILESET]
                    .collision
                    .get(&format!("{:02x}", cell.before))
                    == definition.collision.get(&format!("{:02x}", cell.after)),
            "house cell preimage or collision differs"
        );
        module.blocks[index] = cell.after;
    }
    let bytes = module
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut output = base.clone();
    output.data.maps.insert(MAP.into(), module);
    output
        .data
        .map_blocks
        .insert(label.into(), crate::encode_base64_bytes(&bytes));
    output
        .runtime_files
        .insert(format!("data/tilesets/{TILESET}_metatiles.bin"), metatiles);
    output.runtime_files.insert(
        format!("data/tilesets/{TILESET}.json"),
        serde_json::to_vec(&definition.collision)?,
    );
    output.data.tilesets.insert(TILESET.into(), definition);
    output.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        output.format_version,
        &output.data,
        &output.audio_manifest,
        &output.runtime_files,
        &output.report,
    )?;
    preserve(base, &output, &plan)?;
    crate::verify_compiled_game_pack_for_runtime(&output)?;
    Ok(output)
}

fn parse_contract(base: &CompiledGamePack, text: &str) -> Result<Contract> {
    ensure!(
        hash(text.as_bytes()) == CONTRACT_SHA,
        "exact reviewed v04 tile-join contract required"
    );
    let plan: Contract = serde_json::from_str(text).context("Minneapolis join contract")?;
    ensure!(
        plan.schema_version == 1
            && plan.base_pack_sha256 == BASE_FILE
            && plan.base_identity == BASE_IDENTITY
            && base.identity.content_hash == BASE_IDENTITY
            && plan.base_map_count == 683
            && base.data.maps.len() == 683
            && plan.map == MAP
            && plan.tileset == TILESET
            && plan.expected_blocks == 157
            && plan.blocks.len() == 12
            && plan.cells.len() == 83
            && plan.source_files_sha256.len() == 6,
        "exact first-bend parent and city join profile required"
    );
    let module = base.data.maps.get(MAP).context("native city module")?;
    let definition = base.data.tilesets.get(TILESET).context("native city art")?;
    ensure!(
        hash(&serde_json::to_vec(&serde_json::to_value(module)?)?) == plan.base_module_sha256
            && hash(&serde_json::to_vec(&serde_json::to_value(definition)?)?)
                == plan.base_definition_sha256
            && module.attributes.tileset_name == TILESET
            && [module.attributes.width, module.attributes.height] == [40, 32],
        "whole city module/art preimage changed"
    );
    for (path, expected) in &plan.source_files_sha256 {
        ensure!(
            hash(base.runtime_files.get(path).context("house source file")?) == *expected,
            "house source changed: {path}"
        );
    }
    ensure!(
        plan.cells
            .iter()
            .map(|cell| cell.metatile)
            .collect::<BTreeSet<_>>()
            .len()
            == 83,
        "house patch cells must be distinct"
    );
    Ok(plan)
}

fn append_recipes(
    base: &CompiledGamePack,
    plan: &Contract,
) -> Result<(TilesetDefinition, Vec<u8>)> {
    let original = base
        .data
        .tilesets
        .get(TILESET)
        .context("old house tileset")?;
    let mut definition = original.clone();
    let mut metatiles = base
        .runtime_files
        .get(&format!("data/tilesets/{TILESET}_metatiles.bin"))
        .context("old house metatiles")?
        .clone();
    ensure!(
        metatiles.len() == 157 * 16 && definition.palette_map.len() == 224,
        "old house art extent changed"
    );
    let used = metatiles.iter().copied().collect::<BTreeSet<_>>();
    for (offset, recipe) in plan.blocks.iter().enumerate() {
        ensure!(
            usize::from(recipe.target) == 157 + offset
                && recipe.glyphs.iter().all(|glyph| used.contains(glyph)),
            "join correction must append recipes from existing logical glyphs only"
        );
        metatiles.extend_from_slice(&recipe.glyphs);
        ensure!(
            definition
                .collision
                .insert(format!("{:02x}", recipe.target), recipe.collision.to_vec())
                .is_none(),
            "join collision address occupied"
        );
    }
    ensure!(
        definition.palette_map == original.palette_map,
        "join correction changed native palette entries"
    );
    Ok((definition, metatiles))
}

fn preserve(base: &CompiledGamePack, output: &CompiledGamePack, plan: &Contract) -> Result<()> {
    let (definition, metatiles) = append_recipes(base, plan)?;
    ensure!(
        output.data.tilesets.get(TILESET) == Some(&definition),
        "old house collision/palette or appended recipe changed"
    );
    let owned = [
        (format!("data/tilesets/{TILESET}_metatiles.bin"), metatiles),
        (
            format!("data/tilesets/{TILESET}.json"),
            serde_json::to_vec(&definition.collision)?,
        ),
    ];
    let mut normalized = output.clone();
    for (path, expected) in owned {
        ensure!(
            output.runtime_files.get(&path) == Some(&expected),
            "house join runtime footprint differs: {path}"
        );
        normalized.runtime_files.insert(
            path.clone(),
            base.runtime_files
                .get(&path)
                .context("old join runtime file")?
                .clone(),
        );
    }
    normalized
        .data
        .tilesets
        .insert(TILESET.into(), base.data.tilesets[TILESET].clone());
    let original = base.data.maps.get(MAP).context("original city module")?;
    let mut expected = original.clone();
    for cell in &plan.cells {
        let index = usize::from(cell.metatile[1]) * 40 + usize::from(cell.metatile[0]);
        ensure!(
            expected.blocks[index] == cell.before,
            "original join cell differs"
        );
        expected.blocks[index] = cell.after;
    }
    ensure!(
        output.data.maps.get(MAP) == Some(&expected),
        "city changed outside reviewed house block identifiers"
    );
    let label = original
        .attributes
        .blocks_label
        .as_deref()
        .context("city native block mirror")?;
    let bytes = expected
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    ensure!(
        output.data.map_blocks.get(label) == Some(&crate::encode_base64_bytes(&bytes)),
        "house join cached/encoded geometry differs"
    );
    normalized.data.maps.insert(MAP.into(), original.clone());
    normalized.data.map_blocks.insert(
        label.into(),
        base.data
            .map_blocks
            .get(label)
            .context("old encoded city blocks")?
            .clone(),
    );
    normalized.identity = base.identity.clone();
    ensure!(
        normalized == *base,
        "join correction changed bitmaps/palettes/events/programs/rooms/unrelated maps/audio/report/recovery"
    );
    let mut before = base.data.clone();
    let mut after = output.data.clone();
    before
        .maps
        .remove(MAP)
        .context("remove original city cache")?;
    after
        .maps
        .remove(MAP)
        .context("remove corrected city cache")?;
    let original_wire = before.assemble_map_module_from_compiled_payloads(MAP)?;
    let mut candidate_wire = after.assemble_map_module_from_compiled_payloads(MAP)?;
    ensure!(
        original_wire.blocks == original.blocks && candidate_wire.blocks == expected.blocks,
        "uncached join geometry differs"
    );
    candidate_wire.blocks = original_wire.blocks.clone();
    ensure!(
        candidate_wire == original_wire,
        "uncached native house programs changed"
    );
    Ok(())
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
#[path = "minneapolis_house_joins_tests.rs"]
mod tests;
