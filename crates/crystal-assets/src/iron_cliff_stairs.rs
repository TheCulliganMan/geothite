//! Bounded Iron Range West cliff repair with original cave ledge stairs.
use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;
use crate::authored_route_design::RouteDesign;
use crate::authored_route_layout::Layout;
use crate::authored_route_revision::{clipped_blocks, installed_chunks, validate_anchors};

/// Repairs the selected R09 rock formations and imports cave12/30/36 stairs.
///
/// Frozen authored sources certify the exact preimage, retained actors and rooms,
/// two-way stair courses and dry one-way returns. Only existing exterior blocks,
/// their mirrors, the route's private tileset art, its import manifest and derived
/// identity change. Existing glyphs and all other routes remain intact.
///
/// # Errors
/// Returns an error for source drift, stale installed terrain, unsafe geometry,
/// unavailable native glyph slots or invalid resulting content. Writes no files.
pub fn revise_iron_cliff_stairs(
    base: &CompiledGamePack,
    previous: &str,
    revised: &str,
    dossier: &str,
) -> Result<CompiledGamePack> {
    for (source, expected) in [
        (
            previous,
            "7ba33cabad563eaba6756160e6746e9ba04b5ba57bac1036db24c31e1c1435f9",
        ),
        (
            revised,
            "a9e13689c9290d7beb558723caa3c287bee346c7cfef5ffaac2d34d37dd420f6",
        ),
        (
            dossier,
            "b31819d308cac7735143c43c8abfffc0c32a185a10447021ba00f232a08c0270",
        ),
    ] {
        ensure!(
            format!("{:x}", Sha256::digest(source.as_bytes())) == expected,
            "Iron Range frozen authoring source drift"
        );
    }
    let old = Layout::parse(previous)?;
    let new = Layout::parse(revised)?;
    let design = RouteDesign::parse(dossier)?;
    let before = old.compile()?;
    let after = new.compile()?;
    let chunks = installed_chunks(base, &design, &old, &before)?;
    validate_anchors(base, &design)?;
    let mut result = crate::north_shore_art::append_native_cave_stairs(
        base,
        "mn_iron_range",
        0x85,
        &[0x12, 0x30, 0x36],
        "minnesota-iron-cliff-native-stairs-v1",
    )?;
    crate::authored_route_modpack::validate_geometry(&result, &design, &after, "mn_iron_range")
        .context("Iron Range repaired cliff and stair geometry")?;
    for chunk in chunks {
        let blocks = clipped_blocks(&after, 64, &chunk);
        let bytes = blocks
            .iter()
            .copied()
            .map(u8::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        result
            .data
            .maps
            .get_mut(&chunk.name)
            .context("R09 exterior")?
            .blocks = blocks;
        result
            .data
            .map_blocks
            .insert(chunk.label, crate::encode_base64_bytes(&bytes));
    }
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    Ok(result)
}
