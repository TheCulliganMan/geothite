//! Reconnect the cleared League lanes to the retained winter route.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;

/// Restores one existing snow-floor apron metatile and its raw block mirror.
///
/// Boundary Waters, guard events and requirements, all other route tiles,
/// creatures, original assets and audio remain unchanged. The output receives
/// a new identity; existing saves are never migrated.
///
/// # Errors
/// Rejects a different parent, stale tile/collision/mirror preimages, absent
/// authoring sources or invalid runtime/geography output.
pub fn repair_comprehensive_league_seam(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        base.identity()?.content_hash
            == "f657896b50632d9308e28185e9f6801a1b08e1afa14b61fb16f5319ed5c814bb",
        "exact reviewed League seam parent required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let plan: serde_json::Value = serde_json::from_str(include_str!(
        "../../../planning/minnesota/revisions/comprehensive-league-seam-v01/plan.json"
    ))?;
    ensure!(
        plan["position_metatiles"] == serde_json::json!([16, 0])
            && plan["open_block"] == "83"
            && plan["closed_block"] == "8b",
        "exact apron source contract required"
    );
    let layout: serde_json::Value = serde_json::from_str(include_str!(
        "../../../planning/minnesota/routes/MN_R13.layout.json"
    ))?;
    let source: Vec<_> = layout["layers"]
        .as_array()
        .context("route layers")?
        .iter()
        .filter(|row| row["kind"] == "point" && row["position"] == plan["position_metatiles"])
        .collect();
    ensure!(
        source.len() == 1 && source[0]["block"] == "83",
        "authoritative route source must retain the apron"
    );
    let map = base.data.maps.get("MnR13C00R00").context("League route")?;
    ensure!(
        map.attributes.width == 32
            && map.attributes.height == 24
            && map.attributes.tileset_name == "mn_outdoor_winter_stairs"
            && map.blocks[16] == 0x8b,
        "expected winter apron preimage"
    );
    let collision = &base
        .data
        .tilesets
        .get(&map.attributes.tileset_name)
        .context("native winter tileset")?
        .collision;
    for (block, kind) in [("8b", "WALL"), ("83", "FLOOR")] {
        let quadrants = collision.get(block).context("native apron collision")?;
        ensure!(
            quadrants.len() == 4 && quadrants.iter().all(|q| q == kind),
            "exact native apron quadrants required"
        );
    }
    let label = map
        .attributes
        .blocks_label
        .clone()
        .context("route block mirror label")?;
    let before = map
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    ensure!(
        base.data.map_blocks.get(&label) == Some(&crate::encode_base64_bytes(&before)),
        "exact raw block mirror required"
    );
    let mut pack = base.clone();
    pack.data
        .maps
        .get_mut("MnR13C00R00")
        .context("mutable League route")?
        .blocks[16] = 0x83;
    let mut after = before;
    after[16] = 0x83;
    pack.data
        .map_blocks
        .insert(label, crate::encode_base64_bytes(&after));
    pack.report
        .manifests
        .push("comprehensive-league-seam-repair-v1".into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    crate::verify_northstar_publication_geometry(&pack)?;
    crate::validate_compact_superior_geography(&pack)?;
    Ok(pack)
}
