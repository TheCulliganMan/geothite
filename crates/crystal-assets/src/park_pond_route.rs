//! Apply the reviewed park pond closure, bridge rails and elm shoulders.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{CompiledGamePack, MapModule, TilesetDefinition};

const MAP: &str = "MnR04C00R00";
const TILESET: &str = "mn_river_park";
const TEXT: &str = "MN_R04_BRIDGE_SIGNText";
const BASE_FILE: &str = "3ddbad16c6538e52f219c12497e4fbe6e41a7f18af5c42589b7b92357cfad243";
const BASE_IDENTITY: &str = "3ead916d12c705f2c0559c98b983f5af4760c9a8f557d8e31f62b36ea197e29a";
const CONTRACT_SHA: &str = "7c55b8813a53655daaa8dba88507e7a2f4d7a8e4b4d3c8ee2f90f7c1973a7cfb";

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
    sign_text_label: String,
    sign_before: String,
    sign_after: String,
    base_sign_raw_sha256: String,
    base_sign_body_sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Block {
    target: u16,
    glyphs: [u8; 16],
    collision: [String; 4],
    row_sources: [RowSource; 4],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RowSource {
    block: u16,
    row: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Cell {
    metatile: [u16; 2],
    before: u16,
    after: u16,
}

/// Builds the reviewed 57-cell civic park revision over the exact long-river parent.
///
/// Closes the decorative pond's unconnected ends, frames its original Foot-only
/// bridge with visible blocking rails, and tightens six eastern lawn shoulders.
/// Four appended metatiles rearrange existing native glyph rows only. The existing
/// bridge sign's first text line becomes `PARK POND BRIDGE`. Every other program,
/// actor, event, encounter, connection, room, art pixel, palette, old collision,
/// audio descriptor, report and recovery record remains exact. The caller owns
/// fresh publication and strict written readback. This does not certify native
/// visual quality, controller traversal or earned progression.
///
/// # Errors
/// Rejects an unreviewed contract, stale parent or native preimage, shared block
/// payload, inconsistent typed/raw/cached mirrors, unsupported row recipe or any
/// mutation outside the exact terrain/art/sign footprint. The borrowed parent
/// remains unchanged on success and failure.
pub fn build_park_pond_route_pack(
    base: &CompiledGamePack,
    contract_json: &str,
) -> Result<CompiledGamePack> {
    let plan = parse_contract(base, contract_json)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity == identity(base)?,
        "park parent identity is stale"
    );
    let (definition, metatiles) = append_recipes(base, &plan)?;
    let original = base.data.maps.get(MAP).context("native park module")?;
    let label = original
        .attributes
        .blocks_label
        .as_deref()
        .context("park block label")?;
    ensure!(
        base.data.maps.iter().all(|(name, module)| {
            name == MAP || module.attributes.blocks_label.as_deref() != Some(label)
        }),
        "park block payload is shared"
    );
    let before = block_bytes(original)?;
    ensure!(
        base.data.map_blocks.get(label) == Some(&crate::encode_base64_bytes(&before)),
        "park cached and encoded blocks differ"
    );
    let module = changed_module(original, &plan, &definition)?;
    let bytes = block_bytes(&module)?;
    let mut output = base.clone();
    output.data.maps.insert(MAP.into(), module);
    output
        .data
        .map_blocks
        .insert(label.into(), crate::encode_base64_bytes(&bytes));
    change_raw_sign(
        output
            .data
            .map_scripts
            .get_mut(TEXT)
            .context("park raw sign text")?,
        &plan,
    )?;
    output
        .runtime_files
        .insert(format!("data/tilesets/{TILESET}_metatiles.bin"), metatiles);
    output.runtime_files.insert(
        format!("data/tilesets/{TILESET}.json"),
        serde_json::to_vec(&definition.collision)?,
    );
    output.data.tilesets.insert(TILESET.into(), definition);
    output.identity = identity(&output)?;
    preserve(base, &output, &plan)?;
    crate::verify_compiled_game_pack_for_runtime(&output)?;
    Ok(output)
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

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn value_hash(value: &impl serde::Serialize) -> Result<String> {
    Ok(hash(&serde_json::to_vec(&serde_json::to_value(value)?)?))
}

fn block_bytes(module: &MapModule) -> Result<Vec<u8>> {
    Ok(module
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?)
}

fn parse_contract(base: &CompiledGamePack, text: &str) -> Result<Contract> {
    ensure!(
        hash(text.as_bytes()) == CONTRACT_SHA,
        "exact reviewed combined park contract required"
    );
    let plan: Contract = serde_json::from_str(text).context("combined park contract")?;
    ensure!(
        plan.schema_version == 1
            && plan.base_pack_sha256 == BASE_FILE
            && plan.base_identity == BASE_IDENTITY
            && base.identity.content_hash == BASE_IDENTITY
            && plan.base_map_count == 683
            && base.data.maps.len() == 683
            && plan.map == MAP
            && plan.tileset == TILESET
            && plan.expected_blocks == 68
            && plan.blocks.len() == 4
            && plan.cells.len() == 57
            && plan.source_files_sha256.len() == 5
            && plan.sign_text_label == TEXT
            && plan.sign_before == "\"MISSISSIPPI BRIDGE\""
            && plan.sign_after == "\"PARK POND BRIDGE\"",
        "exact 683-map long-river parent and park profile required"
    );
    let module = base.data.maps.get(MAP).context("original park")?;
    let definition = base
        .data
        .tilesets
        .get(TILESET)
        .context("original park art")?;
    ensure!(
        value_hash(module)? == plan.base_module_sha256
            && value_hash(definition)? == plan.base_definition_sha256
            && module.attributes.tileset_name == TILESET
            && [module.attributes.width, module.attributes.height] == [28, 18]
            && base.data.map_attributes.get(MAP) == Some(&module.attributes),
        "whole native park/art preimage differs"
    );
    for (path, expected) in &plan.source_files_sha256 {
        ensure!(
            hash(
                base.runtime_files
                    .get(path)
                    .context("park native source file")?
            ) == *expected,
            "park source file changed: {path}"
        );
    }
    let raw = base
        .data
        .map_scripts
        .get(TEXT)
        .context("original raw park sign text")?;
    ensure!(
        value_hash(raw)? == plan.base_sign_raw_sha256
            && module.scripts.get(TEXT) == Some(raw)
            && value_hash(
                module
                    .script_text_bodies
                    .get(TEXT)
                    .context("typed park sign body")?
            )? == plan.base_sign_body_sha256,
        "park sign raw/typed/cache preimages differ"
    );
    ensure!(
        plan.cells
            .iter()
            .map(|c| c.metatile)
            .collect::<BTreeSet<_>>()
            .len()
            == 57,
        "park patch cells must be distinct"
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
        .context("original park tileset")?;
    let raw = base
        .runtime_files
        .get(&format!("data/tilesets/{TILESET}_metatiles.bin"))
        .context("original park metatiles")?;
    ensure!(
        raw.len() == 68 * 16 && original.collision.len() == 68 && original.palette_map.len() == 224,
        "park native art extent differs"
    );
    let mut definition = original.clone();
    let mut metatiles = raw.clone();
    for (index, block) in plan.blocks.iter().enumerate() {
        ensure!(
            usize::from(block.target) == 68 + index && block.collision.iter().all(|q| q == "WALL"),
            "park appended cap/rail address or collision differs"
        );
        for (row, source) in block.row_sources.iter().enumerate() {
            ensure!(
                source.block < 68 && source.row < 4,
                "park row donor outside original art"
            );
            let start = usize::from(source.block) * 16 + usize::from(source.row) * 4;
            ensure!(
                block.glyphs[row * 4..row * 4 + 4] == raw[start..start + 4],
                "park cap/rail must use exact existing native glyph rows"
            );
        }
        metatiles.extend_from_slice(&block.glyphs);
        ensure!(
            definition
                .collision
                .insert(format!("{:02x}", block.target), block.collision.to_vec())
                .is_none(),
            "park appended collision address occupied"
        );
    }
    Ok((definition, metatiles))
}

fn change_raw_sign(raw: &mut Value, plan: &Contract) -> Result<()> {
    let commands = raw.as_array_mut().context("park sign text commands")?;
    ensure!(
        commands.len() == 3
            && commands[0] == serde_json::json!({"command":"text","args":[plan.sign_before]})
            && commands[1]
                == serde_json::json!({"command":"line","args":["\"Keep to the deck.\""]})
            && commands[2] == serde_json::json!({"command":"done","args":[]}),
        "exact existing park sign text required"
    );
    commands[0]["args"][0] = Value::String(plan.sign_after.clone());
    Ok(())
}

fn changed_module(
    original: &MapModule,
    plan: &Contract,
    definition: &TilesetDefinition,
) -> Result<MapModule> {
    let mut module = original.clone();
    for cell in &plan.cells {
        let [x, y] = cell.metatile;
        ensure!(
            x < 28
                && y < 18
                && cell.before != cell.after
                && definition
                    .collision
                    .contains_key(&format!("{:02x}", cell.after)),
            "invalid park cell"
        );
        let index = usize::from(y) * 28 + usize::from(x);
        ensure!(
            module.blocks[index] == cell.before,
            "park cell preimage differs"
        );
        module.blocks[index] = cell.after;
    }
    change_raw_sign(
        module
            .scripts
            .get_mut(TEXT)
            .context("park cached raw sign")?,
        plan,
    )?;
    let body = module
        .script_text_bodies
        .get_mut(TEXT)
        .context("park typed sign body")?;
    ensure!(
        body.label == TEXT && body.commands.len() == 3,
        "park typed text extent differs"
    );
    let first = body
        .commands
        .first_mut()
        .context("park sign first text command")?;
    ensure!(
        first.command == "text"
            && first.command_index == 0
            && first.args == [plan.sign_before.clone()],
        "park typed first line differs"
    );
    first.args[0] = plan.sign_after.clone();
    Ok(module)
}

fn preserve(base: &CompiledGamePack, output: &CompiledGamePack, plan: &Contract) -> Result<()> {
    let (definition, metatiles) = append_recipes(base, plan)?;
    ensure!(
        output.data.tilesets.get(TILESET) == Some(&definition),
        "park old collision/palette or appended collision differs"
    );
    let original = base.data.maps.get(MAP).context("original park module")?;
    let expected = changed_module(original, plan, &definition)?;
    ensure!(
        output.data.maps.get(MAP) == Some(&expected),
        "park changed beyond 57 cells and first sign text line"
    );
    let label = original
        .attributes
        .blocks_label
        .as_deref()
        .context("park block mirror label")?;
    ensure!(
        output.data.map_blocks.get(label)
            == Some(&crate::encode_base64_bytes(&block_bytes(&expected)?)),
        "park cached/encoded geometry differs"
    );
    let mut raw_sign = base
        .data
        .map_scripts
        .get(TEXT)
        .context("original global sign text")?
        .clone();
    change_raw_sign(&mut raw_sign, plan)?;
    ensure!(
        output.data.map_scripts.get(TEXT) == Some(&raw_sign),
        "park raw sign text differs"
    );
    let mut normalized = output.clone();
    for (path, expected_bytes) in [
        (format!("data/tilesets/{TILESET}_metatiles.bin"), metatiles),
        (
            format!("data/tilesets/{TILESET}.json"),
            serde_json::to_vec(&definition.collision)?,
        ),
    ] {
        ensure!(
            output.runtime_files.get(&path) == Some(&expected_bytes),
            "park runtime footprint differs: {path}"
        );
        normalized.runtime_files.insert(
            path.clone(),
            base.runtime_files
                .get(&path)
                .context("old park runtime file")?
                .clone(),
        );
    }
    normalized
        .data
        .tilesets
        .insert(TILESET.into(), base.data.tilesets[TILESET].clone());
    normalized.data.maps.insert(MAP.into(), original.clone());
    normalized.data.map_blocks.insert(
        label.into(),
        base.data
            .map_blocks
            .get(label)
            .context("old park block payload")?
            .clone(),
    );
    normalized
        .data
        .map_scripts
        .insert(TEXT.into(), base.data.map_scripts[TEXT].clone());
    normalized.identity = base.identity.clone();
    ensure!(
        normalized == *base,
        "park revision changed unrelated geometry/actors/programs/art/audio/report/recovery"
    );
    ensure!(
        output.identity == identity(output)?,
        "park output identity is stale"
    );
    let mut old_data = base.data.clone();
    let mut new_data = output.data.clone();
    old_data.maps.remove(MAP).context("remove old park cache")?;
    new_data.maps.remove(MAP).context("remove new park cache")?;
    let old_wire = old_data.assemble_map_module_from_compiled_payloads(MAP)?;
    ensure!(
        old_wire.blocks == original.blocks,
        "old park cached/uncached geometry differs"
    );
    // Reassembly may omit pre-existing cached lowering metadata. Preserve both
    // representations and require the same bounded change in the wire path.
    let expected_wire = changed_module(&old_wire, plan, &definition)?;
    ensure!(
        new_data.assemble_map_module_from_compiled_payloads(MAP)? == expected_wire,
        "new park cached/uncached terrain/sign/program mirrors differ"
    );
    Ok(())
}

#[cfg(test)]
#[path = "park_pond_route_tests.rs"]
mod tests;
