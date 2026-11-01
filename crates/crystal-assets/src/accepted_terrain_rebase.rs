//! Apply an accepted terrain delta without replacing newer room programs.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::{CompiledGamePack, MapModule};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Contract {
    schema_version: u8,
    current_content_hash: String,
    accepted_base_content_hash: String,
    accepted_child_content_hash: String,
    manifest: String,
    maps: Vec<String>,
    #[serde(default)]
    actor_relocations: Vec<crate::interior_conversation_relocations::Relocation>,
}

/// Applies a source-bound, previously accepted terrain revision to another branch.
///
/// The accepted pair may differ only in the named Minnesota maps' metatiles and
/// border blocks, plus their native mirrors and publication bookkeeping. Every
/// changed cell must still have its exact preimage in `current`. Unchanged cells,
/// room stories, warps, callbacks, numeric identities, catalogs and audio remain
/// those of `current`. An explicit relocation may move a newer ordinary speech
/// actor to reachable floor; accepted actors and service/progression programs
/// cannot move. All three complete content identities are pinned
/// by the contract. This returns an in-memory pack, without migrating saves or
/// establishing visual, gameplay or whole-region acceptance.
///
/// # Errors
/// Rejects stale inputs, incomplete map inventories, nonterrain accepted changes,
/// conflicting cells/borders, changed dimensions/art or dynamic block programs,
/// unsafe new conversation actors, duplicate publication and invalid native data.
/// The borrowed inputs remain unchanged on failure.
pub fn rebase_accepted_terrain(
    current: &CompiledGamePack,
    accepted_base: &CompiledGamePack,
    accepted_child: &CompiledGamePack,
    contract_json: &str,
) -> Result<CompiledGamePack> {
    let contract: Contract = serde_json::from_str(contract_json)?;
    ensure!(
        contract.schema_version == 1
            && contract
                .manifest
                .starts_with("minnesota-interior-terrain-sync-")
            && !current.report.manifests.contains(&contract.manifest),
        "a fresh, explicit regional terrain synchronization is required"
    );
    for (pack, hash) in [
        (current, &contract.current_content_hash),
        (accepted_base, &contract.accepted_base_content_hash),
        (accepted_child, &contract.accepted_child_content_hash),
    ] {
        crate::verify_compiled_game_pack_for_runtime(pack)?;
        crate::validate_compact_superior_geography(pack)?;
        let identity = crate::derive_compiled_game_pack_identity_from_manifest(
            pack.format_version,
            &pack.data,
            &pack.audio_manifest,
            &pack.runtime_files,
            &pack.report,
        )?;
        ensure!(
            &identity.content_hash == hash,
            "terrain input content differs"
        );
    }
    ensure!(
        accepted_base.runtime_files == accepted_child.runtime_files
            && accepted_base.audio_manifest == accepted_child.audio_manifest,
        "an accepted terrain delta cannot replace native art or audio"
    );
    let mut before = serde_json::to_value(&accepted_base.data)?;
    let mut after = serde_json::to_value(&accepted_child.data)?;
    for field in ["maps", "map_attributes", "map_blocks"] {
        before
            .as_object_mut()
            .context("original native data")?
            .remove(field);
        after
            .as_object_mut()
            .context("accepted native data")?
            .remove(field);
    }
    ensure!(
        before == after,
        "accepted revision changes nonterrain native data"
    );
    let expected = contract
        .maps
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    ensure!(
        !expected.is_empty()
            && expected.len() == contract.maps.len()
            && expected
                .iter()
                .all(|name| name.starts_with("Mn") && !name.contains("Gym"))
            && accepted_base
                .data
                .maps
                .keys()
                .eq(accepted_child.data.maps.keys()),
        "explicit existing non-gym Minnesota terrain maps are required"
    );
    ensure!(
        contract
            .actor_relocations
            .iter()
            .all(|r| expected.contains(r.map.as_str())),
        "resident relocation must belong to a contracted terrain map"
    );
    let mut changed = BTreeSet::new();
    let mut result = current.clone();
    for (name, original) in &accepted_base.data.maps {
        let accepted = &accepted_child.data.maps[name];
        if original == accepted {
            continue;
        }
        ensure!(
            expected.contains(name.as_str()),
            "uncontracted terrain map: {name}"
        );
        let mut normalized = accepted.clone();
        normalized.blocks.clone_from(&original.blocks);
        normalized.attributes.border_block = original.attributes.border_block;
        ensure!(
            normalized == *original,
            "accepted map changes more than terrain: {name}"
        );
        let installed = result.data.maps.get(name).context("current terrain map")?;
        let mut map = apply_delta(original, accepted, installed)?;
        crate::interior_conversation_relocations::apply(
            &result,
            original,
            &mut map,
            &contract.actor_relocations,
        )?;
        for object in map.objects.iter().filter(|object| {
            object.object_type == "OBJECTTYPE_SCRIPT" && !original.objects.contains(object)
        }) {
            ensure!(
                matches!(
                    crate::authored_town_geometry::kind(&result, &map, [object.x, object.y])?,
                    "FLOOR" | "TALL_GRASS"
                ),
                "accepted terrain blocks a newer conversation actor: {}/{}",
                map.id,
                object.script
            );
        }
        crate::authored_town_publication::replace(&mut result, map)?;
        changed.insert(name.as_str());
    }
    ensure!(
        changed == expected,
        "terrain contract includes an unchanged or missing map"
    );
    result.report.manifests.push(contract.manifest);
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    crate::validate_compact_superior_geography(&result)?;
    Ok(result)
}

fn apply_delta(before: &MapModule, after: &MapModule, installed: &MapModule) -> Result<MapModule> {
    ensure!(
        installed.attributes.width == before.attributes.width
            && installed.attributes.height == before.attributes.height
            && installed.attributes.tileset_name == before.attributes.tileset_name
            && installed.attributes.border_block == before.attributes.border_block
            && installed.script_block_changes == before.script_block_changes
            && installed.blocks.len() == before.blocks.len(),
        "current map geometry or dynamic terrain differs: {}",
        before.id
    );
    let mut map = installed.clone();
    for (index, (old, new)) in before.blocks.iter().zip(&after.blocks).enumerate() {
        if old != new {
            ensure!(
                map.blocks[index] == *old,
                "terrain cell conflicts: {} index {index}",
                before.id
            );
            map.blocks[index] = *new;
        }
    }
    map.attributes.border_block = after.attributes.border_block;
    Ok(map)
}

#[cfg(test)]
#[path = "accepted_terrain_rebase_tests.rs"]
mod tests;
