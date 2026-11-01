use anyhow::{Context, Result, ensure};

#[path = "outdoor_terrain_blocks.rs"]
mod blocks;
use blocks::restore_blocks;

use crate::{
    CompiledGamePack, decode_base64_bytes, derive_compiled_game_pack_identity_from_manifest,
    encode_base64_bytes, verify_compiled_game_pack_for_runtime,
};

/// Restore the four explicitly authored lake-removal overrides in a local pack.
/// Source exports remain untouched; all other maps, events and audio are retained.
pub fn build_outdoor_terrain_modpack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    verify_compiled_game_pack_for_runtime(base)?;
    let mut pack = base.clone();
    for (name, width, height, edits) in [
        (
            "CherrygroveCity",
            20,
            9,
            &[(17, 0, 0x58), (18, 0, 0x59)][..],
        ),
        ("Route30", 10, 27, &[(9, 25, 0x54), (9, 26, 0x58)][..]),
    ] {
        let map = pack
            .data
            .maps
            .get_mut(name)
            .with_context(|| format!("missing {name}"))?;
        ensure!(
            usize::from(map.attributes.width) == width
                && usize::from(map.attributes.height) == height,
            "unexpected {name} block dimensions"
        );
        let label = map
            .attributes
            .blocks_label
            .as_deref()
            .context("map has no blocks label")?;
        let encoded = pack
            .data
            .map_blocks
            .get(label)
            .context("missing encoded map blocks")?;
        let mut bytes = decode_base64_bytes(encoded)?;
        ensure!(
            bytes.len() == width * height && map.blocks.len() == bytes.len(),
            "unexpected {name} block count"
        );
        ensure!(
            map.blocks
                .iter()
                .zip(&bytes)
                .all(|(&a, &b)| a == u16::from(b)),
            "{name} block representations disagree"
        );
        restore_blocks(&mut bytes, width, edits)?;
        for (target, &byte) in map.blocks.iter_mut().zip(&bytes) {
            *target = u16::from(byte);
        }
        pack.data
            .map_blocks
            .insert(label.to_owned(), encode_base64_bytes(&bytes));
    }
    pack.identity = derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}
