//! Publish the Minnesota headwaters journey.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;

// The four existing Kanto timber glyphs imported into the Headwaters bank.
const TIMBER: [u8; 16] = [
    0x9f, 0x9f, 0x9f, 0x9f, 0x9d, 0x9e, 0x9d, 0x9e, 0xc3, 0xc3, 0xc3, 0xc3, 0x9f, 0x9f, 0x9f, 0x9f,
];
fn timber_block(pack: &CompiledGamePack) -> Result<u16> {
    let layout = pack
        .runtime_files
        .get("data/tilesets/mn_headwaters_metatiles.bin")
        .context("Headwaters metatiles")?;
    let collision = &pack
        .data
        .tilesets
        .get("mn_headwaters")
        .context("Headwaters permissions")?
        .collision;
    let mut donors = layout
        .as_chunks::<16>()
        .0
        .iter()
        .enumerate()
        .filter(|(i, tiles)| {
            **tiles == TIMBER
                && collision
                    .get(&format!("{i:02x}"))
                    .is_some_and(|kinds| kinds == &["FLOOR"; 4])
        });
    let donor = u16::try_from(
        donors
            .next()
            .context("missing open Headwaters timber deck")?
            .0,
    )?;
    ensure!(donors.next().is_none(), "ambiguous Headwaters timber deck");
    Ok(donor)
}

/// Builds Minnesota's Headwaters Journey revision from a reviewed Station Return edition.
///
/// # Errors
/// Returns an error when the source edition or a revised landmark disagrees
/// with its native records, or when the completed content fails validation.
pub fn revise_headwaters_journey(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        base.report.manifests.last().map(String::as_str) == Some("northstar-station-return-v1"),
        "expected Minnesota Station Return"
    );
    let mut pack = base.clone();
    let timber = timber_block(base)?;
    crate::northstar_itasca_crossing::apply(&mut pack, timber)?;
    crate::northstar_central_handoff::apply(&mut pack)?;
    crate::northstar_leech_return::apply(&mut pack, timber)?;
    crate::northstar_northern_passage::apply(&mut pack)?;
    pack.report
        .manifests
        .push("northstar-headwaters-journey-v1".into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}
