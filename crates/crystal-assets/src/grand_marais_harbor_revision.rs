//! Close only the accepted Grand Marais south edge using existing native buoys.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::authored_route_layout::Layout;
use crate::authored_town_geometry::kind;
use crate::{CompiledGamePack, MapModule};

const MAP: &str = "MnGrandMarais";
const BASE_IDENTITY: &str = "11a5f3b39ffba6252fb2246af17433ed70b6b4d722d0b3a67394a1526845b03a";
const BASE_FILE: &str = "c4816242295a236fdfe2a6a96fb248cd3eab11de1b526be0decb7df36e5e95eb";
const CONTRACT_SHA: &str = "2ae0af459e1a6c63bc8930646ba5fc12bb3071b9de62445daa631820d09c6cb3";
const LAYOUT_SHA: &str = "a6e3e4aea3c405e6bff28d9425bbdb1d0499e03f2d3d633002fe7786c3c79986";

#[derive(Deserialize)]
struct Contract {
    schema_version: u16,
    map: String,
    base_pack_sha256: String,
    base_module_sha256: String,
    base_blocks_sha256: String,
    candidate_blocks_sha256: String,
    candidate_layout_sha256: String,
    dimensions_metatiles: [u16; 2],
    cell_patches: Vec<Cell>,
}

#[derive(Deserialize)]
struct Cell {
    metatile: [u16; 2],
    before_block: u16,
    after_block: u16,
}

/// Apply the exact reviewed twenty-cell native Grand Marais buoy line.
///
/// Requires the fixed 681-map cohesion parent and byte-exact accepted contract
/// and complete layout. Only this town's blocks, its uniquely owned encoded
/// block payload and the derived identity change. Existing native pixels and
/// palettes are reused. All other maps, attributes, programs, actors, catalogs,
/// report/manifests, recovery data and audio remain exact. Uncached assembly is
/// compared against its own parent representation to retain inherited parser
/// differences. The caller owns writing and strict readback.
///
/// This validates geometry and data preservation, not production input, visual
/// acceptance, earned Surf access or compatibility with saved block overrides.
///
/// # Errors
/// Rejects a stale parent/source/preimage, shared or inconsistent block payload,
/// unreviewed cell, incorrect buoy collision/endcap, uncached disagreement or
/// unrelated data mutation. Borrowed inputs remain unchanged on failure.
pub fn revise_grand_marais_harbor_boundary(
    base: &CompiledGamePack,
    contract_json: &str,
    layout_json: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        hash(contract_json.as_bytes()) == CONTRACT_SHA
            && hash(layout_json.as_bytes()) == LAYOUT_SHA,
        "exact accepted Grand Marais contract/layout bytes required"
    );
    let plan: Contract = serde_json::from_str(contract_json)?;
    ensure!(
        base.identity.content_hash == BASE_IDENTITY
            && base.data.maps.len() == 681
            && plan.schema_version == 1
            && plan.map == MAP
            && plan.base_pack_sha256 == BASE_FILE
            && plan.candidate_layout_sha256 == LAYOUT_SHA
            && plan.dimensions_metatiles == [20, 19],
        "exact 681-map Grand Marais parent/profile required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let original = base.data.maps.get(MAP).context("installed Grand Marais")?;
    ensure!(
        hash(&serde_json::to_vec(&serde_json::to_value(original)?)?) == plan.base_module_sha256
            && block_hash(&original.blocks)? == plan.base_blocks_sha256
            && original.blocks.len() == 380
            && original.attributes.tileset_name == "johto"
            && [original.attributes.width, original.attributes.height] == [20, 19],
        "Grand Marais whole native preimage differs"
    );
    let label = original
        .attributes
        .blocks_label
        .as_deref()
        .context("harbor block label")?;
    ensure!(
        base.data.maps.iter().all(|(name, map)| name == MAP
            || map.attributes.blocks_label.as_deref() != Some(label)),
        "harbor block payload is shared"
    );
    ensure!(
        crate::decode_base64_bytes(base.data.map_blocks.get(label).context("harbor payload")?)?
            .into_iter()
            .map(u16::from)
            .eq(original.blocks.iter().copied()),
        "encoded harbor preimage disagrees with cached geometry"
    );
    let mut replacement = original.clone();
    ensure!(
        plan.cell_patches.len() == 20,
        "exact twenty south cells required"
    );
    for (x, cell) in plan.cell_patches.iter().enumerate() {
        ensure!(
            cell.metatile == [u16::try_from(x)?, 18]
                && cell.before_block == 0x35
                && cell.after_block == expected_block(x)
                && original.blocks[360 + x] == cell.before_block,
            "unreviewed south edge cell {x}"
        );
        replacement.blocks[360 + x] = cell.after_block;
    }
    let compiled = compile_source(layout_json)?;
    ensure!(
        compiled == replacement.blocks && block_hash(&compiled)? == plan.candidate_blocks_sha256,
        "whole native source layout differs from bounded delta"
    );
    validate_geometry(base, original, &replacement)?;
    let bytes = replacement
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut result = base.clone();
    *result
        .data
        .map_blocks
        .get_mut(label)
        .context("existing harbor payload")? = crate::encode_base64_bytes(&bytes);
    result.data.maps.insert(MAP.into(), replacement);
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    validate_preservation(base, &result)?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    Ok(result)
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn block_hash(blocks: &[u16]) -> Result<String> {
    Ok(hash(
        &blocks
            .iter()
            .copied()
            .map(u8::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?,
    ))
}

fn expected_block(x: usize) -> u16 {
    match x {
        0 => 0x38,
        19 => 0x3a,
        _ => 0x39,
    }
}

fn compile_source(text: &str) -> Result<Vec<u16>> {
    let mut value: Value = serde_json::from_str(text)?;
    ensure!(
        value["id"] == "MnGrandMaraisHarborClosureCandidate"
            && value["schema_version"] == 2
            && value["dimensions_metatiles"] == serde_json::json!([20, 19])
            && value["compiled_tileset"] == "johto"
            && value["tileset"] == "johto"
            && value["source_pixels_modified"] == false
            && value["shore_fitting"] == "explicit_source_edges",
        "native town source profile differs"
    );
    // The accepted complete source has unnamed, explicit unit block_rect rows.
    // Names are required editorial diagnostics for the shared Rust compiler.
    // Adapt only this in-memory parse value; exact source SHA guards run first.
    for (index, layer) in value["layers"]
        .as_array_mut()
        .context("source cells")?
        .iter_mut()
        .enumerate()
    {
        layer
            .as_object_mut()
            .context("source block rectangle")?
            .insert(
                "name".into(),
                Value::String(format!("Harbor source cell {index}")),
            );
    }
    let layout: Layout = serde_json::from_value(value).context("native town source compiler")?;
    layout.compile()
}

fn validate_geometry(pack: &CompiledGamePack, old: &MapModule, new: &MapModule) -> Result<()> {
    use crystal_core::world::collision::{
        PlayerTraversalState, is_permission_passable, sample_collision,
    };
    use crystal_core::world::map::{Direction, OverworldMapData, TilePosition};

    ensure!(
        old.blocks.len() == 380 && new.blocks.len() == 380,
        "town extent differs"
    );
    ensure!(
        old.blocks[..360] == new.blocks[..360],
        "terrain outside south row changed"
    );
    let mut changes = 0;
    let tileset = pack.data.tileset_collision("johto")?;
    let map = OverworldMapData::from_attributes(MAP, &new.attributes, new.blocks.clone());
    for y in 0..38 {
        for x in 0..40 {
            let at = [x, y];
            let before = kind(pack, old, at)?;
            let after = kind(pack, new, at)?;
            if before != after {
                ensure!(
                    before == "WATER" && after == "BUOY",
                    "non-buoy terrain change"
                );
                changes += 1;
            }
            if y == 37 || y == 36 && (x == 0 || x == 39) {
                ensure!(after == "BUOY", "visible buoy gap at {at:?}");
                let sample = sample_collision(
                    &map,
                    &tileset,
                    TilePosition::new(i16::try_from(x)?, i16::try_from(y)?),
                )
                .context("native buoy collision sample")?;
                for direction in [
                    Direction::Up,
                    Direction::Down,
                    Direction::Left,
                    Direction::Right,
                ] {
                    ensure!(
                        !is_permission_passable(
                            sample.permission,
                            direction,
                            PlayerTraversalState::Surf
                        ),
                        "native Surf can pass the buoy at {at:?}"
                    );
                }
            } else if y == 36 {
                ensure!(after == "WATER", "inner harbor lane changed");
            }
        }
    }
    for at in old
        .objects
        .iter()
        .map(|a| [a.x, a.y])
        .chain(old.events.warps.iter().map(|a| [a.x, a.y]))
        .chain(old.events.bg_events.iter().map(|a| [a.x, a.y]))
        .chain(old.events.coord_events.iter().map(|a| [a.x, a.y]))
    {
        ensure!(
            kind(pack, old, at)? == kind(pack, new, at)?,
            "retained harbor actor/event standing collision changed at {at:?}"
        );
    }
    ensure!(changes == 42, "exact 42 Water-to-BUOY changes required");
    for x in 0..20 {
        ensure!(
            old.blocks[360 + x] == 0x35 && new.blocks[360 + x] == expected_block(x),
            "visible native buoy recipe differs"
        );
    }
    // Both turn-up caps meet blocking actual neighbor terrain; no hidden gap
    // is masked by the active map's water-filled fallback border.
    for (direction, source_y, target_x, expected_target) in [
        ("west", 37_i32, 63, "MnR12C01R01"),
        ("east", 37_i32, 0, "MnMaraisCedarApproach"),
    ] {
        let edge = old
            .attributes
            .connections
            .iter()
            .find(|e| e.direction == direction)
            .context("actual harbor side shore connection")?;
        let neighbor = pack
            .data
            .maps
            .get(&edge.target_map)
            .context("actual shore neighbor")?;
        ensure!(
            edge.target_map == expected_target
                && kind(
                    pack,
                    neighbor,
                    [target_x, u16::try_from(source_y - edge.offset * 2)?]
                )? == "WALL",
            "buoy endcap does not meet actual blocking shore"
        );
    }
    Ok(())
}

fn validate_preservation(base: &CompiledGamePack, result: &CompiledGamePack) -> Result<()> {
    let old = base.data.maps.get(MAP).context("old harbor")?;
    let new = result.data.maps.get(MAP).context("new harbor")?;
    let label = old
        .attributes
        .blocks_label
        .as_deref()
        .context("harbor label")?;
    let mut normalized = result.clone();
    normalized
        .data
        .maps
        .get_mut(MAP)
        .context("retained town")?
        .blocks = old.blocks.clone();
    *normalized
        .data
        .map_blocks
        .get_mut(label)
        .context("retained payload")? = base
        .data
        .map_blocks
        .get(label)
        .context("old payload")?
        .clone();
    normalized.identity = base.identity.clone();
    ensure!(
        &normalized == base,
        "change outside owned blocks/payload/identity"
    );
    // Preserve the parent's cached/uncached distinction rather than repairing
    // unrelated old text/index tables when changing only encoded geometry.
    let mut before_data = base.data.clone();
    let mut after_data = result.data.clone();
    before_data
        .maps
        .remove(MAP)
        .context("remove old cached town")?;
    after_data
        .maps
        .remove(MAP)
        .context("remove new cached town")?;
    let before = before_data.assemble_map_module_from_compiled_payloads(MAP)?;
    let mut after = after_data.assemble_map_module_from_compiled_payloads(MAP)?;
    ensure!(
        before.blocks == old.blocks && after.blocks == new.blocks,
        "uncached native geometry differs from cached blocks"
    );
    after.blocks = before.blocks.clone();
    ensure!(
        after == before,
        "uncached native programs/attributes changed beyond blocks"
    );
    Ok(())
}

#[cfg(test)]
#[path = "grand_marais_harbor_revision_tests.rs"]
mod tests;
