//! Reproduce the reviewed indoor fossil service on the complete campaign.
use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;

const PARENT: &str = "a8bde441eaa01db10a8be3dc746353c14d573d08630eda7b69dde24f37091e24";
const CHILD: &str = "0fb868423bc402679f5623cf6fdda2b2af87e7ab0228a5f8d38dfdd3d0667386";
const SOURCE_PARENT: &str = "6b28ef1aab2c5e843d6321d4f2790f4f1b578a442ce9406d1a9a85dd657462fc";

/// Integrates the accepted fossil laboratory into the reviewed full campaign.
///
/// Reuses the original lab builder and all its facade, native-slot, script and
/// traversal checks. Only the plan's parent binding changes. The complete
/// resulting content identity must equal the independently audited candidate.
/// The borrowed parent and all existing saves remain unchanged.
///
/// # Errors
/// Rejects stale or modified input content, an altered source-parent binding,
/// invalid lab geometry or identities, or a result differing from the reviewed
/// complete child. Propagates the original builder's validation errors.
pub fn integrate_comprehensive_fossil_lab(current: &CompiledGamePack) -> Result<CompiledGamePack> {
    let identity = crate::derive_compiled_game_pack_identity_from_manifest(
        current.format_version,
        &current.data,
        &current.audio_manifest,
        &current.runtime_files,
        &current.report,
    )?;
    ensure!(
        identity.content_hash == PARENT,
        "reviewed V17 campaign required"
    );
    let mut plan: Value = serde_json::from_str(include_str!(
        "../../../planning/minnesota/landmarks/fossil-lab-v01/lab.json"
    ))
    .context("authored fossil laboratory plan")?;
    ensure!(
        plan["parent_content_hash"].as_str() == Some(SOURCE_PARENT),
        "reviewed fossil laboratory source binding required"
    );
    plan["parent_content_hash"] = PARENT.into();
    let revised = crate::build_minnesota_fossil_lab(current, &serde_json::to_string(&plan)?)?;
    ensure!(
        revised.identity()?.content_hash == CHILD,
        "fossil integration differs from the independently reviewed V18 child"
    );
    Ok(revised)
}
