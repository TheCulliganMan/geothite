//! Local sleep recovery before Wind, without changing opponents or progression.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::CompiledGamePack;

const MANIFEST: &str = "minnesota-northstar-supplies-v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Supplies {
    schema_version: u16,
    base_content_hash: String,
    mart: String,
    expected_stock: Vec<String>,
    append: String,
}

/// Makes the existing Awakening available at St Cloud before its sleep-using leader.
///
/// Appends one ordinary purchasable item to the pinned Northstar relay parent's
/// installed mart. Prices, maps, scripts, opposing parties, access requirements,
/// recovery, all other datasets and every runtime/audio file remain unchanged.
/// Installation grants no item, money, badge, party member or story progress.
///
/// # Errors
/// Returns an error for a different parent, changed stock/contract, duplicate
/// installation, missing native sleep cure or invalid resulting pack.
pub fn build_northstar_supplies(base: &CompiledGamePack, source: &str) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let source: Supplies = serde_json::from_str(source)?;
    ensure!(
        source.schema_version == 1
            && source.mart == "MART_MN_ST_CLOUD"
            && source.append == "AWAKENING",
        "supported St Cloud sleep-recovery contract required"
    );
    ensure!(
        base.identity()?.content_hash == source.base_content_hash
            || crate::northstar_current_story::is_composing(base),
        "supplies parent differs"
    );
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "supplies already installed"
    );
    let old = base
        .data
        .marts
        .0
        .get(&source.mart)
        .context("installed St Cloud mart")?;
    ensure!(
        *old == source.expected_stock && !old.contains(&source.append),
        "native stock differs"
    );
    let item = base
        .data
        .items
        .get(&source.append)
        .context("existing native Awakening")?;
    ensure!(
        item.price == 250 && item.status_heals.iter().any(|s| s == "SLEEP"),
        "ordinary priced native sleep cure required"
    );
    let mut pack = base.clone();
    pack.data
        .marts
        .0
        .get_mut(&source.mart)
        .context("retained mart")?
        .push(source.append);
    pack.report.manifests.push(MANIFEST.into());
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
