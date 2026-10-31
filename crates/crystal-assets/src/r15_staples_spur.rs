//! Restore Staples' walking spur without restoring the removed route fences.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;

/// Replaces four blocking trees with existing native canopy-over-path blocks.
///
/// # Errors
/// Rejects changed map, connection, collision or raw-block preimages, and invalid
/// compiled runtime content or regional geography.
pub fn revise_r15_staples_spur(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let map = base.data.maps.get("MnR15C00R00").context("Route 15")?;
    ensure!(
        map.attributes.width == 32
            && map.attributes.height == 18
            && map.attributes.tileset_name == "kanto"
            && map
                .attributes
                .connections
                .iter()
                .any(|c| c.direction == "north"
                    && c.target_map == "MnStaplesRailJunction"
                    && c.offset == 10),
        "reviewed route and Staples connection required"
    );
    let collision = &base.data.tilesets["kanto"].collision;
    ensure!(
        collision["0f"] == ["WALL", "WALL", "WALL", "WALL"]
            && collision["6c"] == ["WALL", "WALL", "FLOOR", "FLOOR"],
        "native canopy-over-path collision required"
    );
    let label = map
        .attributes
        .blocks_label
        .as_ref()
        .context("raw block label")?;
    let mut bytes = map
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    ensure!(
        base.data.map_blocks.get(label) == Some(&crate::encode_base64_bytes(&bytes)),
        "raw block mirror drift"
    );
    for block in &mut bytes[9..13] {
        ensure!(*block == 0x0f, "blocked spur preimage drift");
        *block = 0x6c;
    }
    let mut pack = base.clone();
    let revised = pack
        .data
        .maps
        .get_mut("MnR15C00R00")
        .context("mutable route")?;
    revised.blocks[9..13].fill(0x6c);
    pack.data
        .map_blocks
        .insert(label.clone(), crate::encode_base64_bytes(&bytes));
    const MANIFEST: &str = "minnesota-r15-fence-free-staples-spur-v1";
    ensure!(
        !base.report.manifests.iter().any(|v| v == MANIFEST),
        "already revised"
    );
    pack.report.manifests.push(MANIFEST.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    crate::validate_compact_superior_geography(&pack)?;
    Ok(pack)
}
