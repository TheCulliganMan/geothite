//! Restore the authored Prairie Forest Link north entrance to Detroit Lakes.
use crate::CompiledGamePack;
use anyhow::{Context, Result, ensure};

/// Opens the existing north road and installs its reciprocal native connections.
///
/// Existing route guards, objects, services, art and audio remain unchanged.
/// Saves are not migrated; the child receives a new content identity.
///
/// # Errors
/// Rejects stale parent, road/canopy preimages or occupied connection sides,
/// and any invalid runtime or regional geometry output.
pub fn restore_r19_detroit_seam(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    let plan: serde_json::Value = serde_json::from_str(include_str!(
        "../../../planning/minnesota/revisions/route-grass-v01/current-r19-successor-review-v01/southern-seam-plan-v01.json"
    ))?;
    ensure!(
        base.identity()?.content_hash
            == plan["parent_content_identity"]
                .as_str()
                .context("parent identity")?,
        "reviewed parent required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let town = base
        .data
        .maps
        .get("MnDetroitLakes")
        .context("Detroit town")?;
    ensure!(
        town.attributes.width == 24 && town.attributes.height == 18 && town.blocks[12] == 0x05,
        "north canopy preimage"
    );
    let collision = &base.data.tilesets[&town.attributes.tileset_name].collision;
    ensure!(
        collision["05"].iter().all(|q| q == "WALL") && collision["01"].iter().all(|q| q == "FLOOR"),
        "native town road recipes"
    );
    let label = town
        .attributes
        .blocks_label
        .clone()
        .context("town block label")?;
    let mut bytes = town
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    ensure!(
        base.data.map_blocks.get(&label) == Some(&crate::encode_base64_bytes(&bytes)),
        "town block mirror"
    );
    let mut pack = base.clone();
    for (name, side, target) in [
        ("MnR19C00R01", "south", "MnDetroitLakes"),
        ("MnDetroitLakes", "north", "MnR19C00R01"),
    ] {
        let mut map = pack.data.maps.get(name).context("seam map")?.clone();
        ensure!(
            !map.attributes
                .connections
                .iter()
                .any(|c| c.direction == side),
            "connection side already occupied"
        );
        crate::lake_country_geometry::connect(&mut map, side, target, 0)?;
        if name == "MnDetroitLakes" {
            map.blocks[12] = 0x01;
        }
        pack.data
            .map_attributes
            .insert(name.into(), map.attributes.clone());
        pack.data.maps.insert(name.into(), map);
    }
    bytes[12] = 0x01;
    pack.data
        .map_blocks
        .insert(label, crate::encode_base64_bytes(&bytes));
    pack.report
        .manifests
        .push("minnesota-r19-detroit-seam-v1".into());
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
