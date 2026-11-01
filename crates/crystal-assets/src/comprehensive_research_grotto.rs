//! Compose reviewed research access and spring geometry onto the full campaign.
use anyhow::{Context, Result, ensure};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;

const BEFORE: &str =
    include_str!("../../../planning/minnesota/caves/gunflint-grotto-v01/before.json");
const AFTER: &str =
    include_str!("../../../planning/minnesota/caves/gunflint-grotto-v01/after.json");
const CONTRACT: &str =
    include_str!("../../../planning/minnesota/caves/gunflint-grotto-v01/contract.json");

/// Returns a research-only intermediate and the complete grotto successor.
///
/// Pins the V20 campaign and all reviewed geometry inputs. Only the original
/// grotto contract's complete-map preimage and provenance are rebound; both
/// original builders retain their native script and geometry guards. Saves and
/// active world selection remain untouched.
///
/// # Errors
/// Rejects stale campaign or authoring inputs, incompatible original programs,
/// invalid native geometry, or an invalid resulting runtime pack.
pub fn integrate_comprehensive_research_grotto(
    current: &CompiledGamePack,
) -> Result<(CompiledGamePack, CompiledGamePack)> {
    ensure!(
        current.identity()?.content_hash
            == "8c847f3ae7628d8117bebe571fb14a437fbe3108186dd88cfa6ae79054c995af",
        "reviewed V20 campaign required"
    );
    for (source, expected) in [
        (
            BEFORE,
            "49e7a8545234c6d04fadc6f4599aca7a888b9bb937b1e4b31d212e44912db5e5",
        ),
        (
            AFTER,
            "3a7eae3bec59ef9cb3713119f1cd30f2a5e443891443a2514cb80ecd9150e067",
        ),
        (
            CONTRACT,
            "f9897093e7edec8bcb91db65586642a2d2f08204e21553a164ac64def5ad1434",
        ),
    ] {
        ensure!(
            format!("{:x}", Sha256::digest(source.as_bytes())) == expected,
            "reviewed grotto source required"
        );
    }
    let research = crate::build_minnesota_research_access(current)?;
    let mut contract: Value = serde_json::from_str(CONTRACT)?;
    let room = research
        .data
        .maps
        .get("MnR12Cave1F")
        .context("campaign spring refuge")?;
    contract["native_module_sha256"] = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&serde_json::to_value(room)?)?)
    )
    .into();
    // The intermediate is identified by content, rather than an invented storage hash.
    contract
        .as_object_mut()
        .context("grotto contract object")?
        .remove("parent_pack_sha256");
    contract["parent_revision"] = "comprehensive_v20_early_research".into();
    contract["parent_pack"] =
        "content-packs/minnesota-comprehensive-research-stage-v01.crystalpack".into();
    contract["parent_content_identity"] = research.identity()?.content_hash.into();
    let grotto = crate::rebuild_gunflint_grotto(
        &research,
        BEFORE,
        AFTER,
        &serde_json::to_string(&contract)?,
    )?;
    crate::verify_northstar_publication_geometry(&grotto)?;
    crate::validate_compact_superior_geography(&grotto)?;
    Ok((research, grotto))
}
