//! Preserve the full campaign while installing the reviewed fitted crosscut.
use anyhow::{Context, Result, ensure};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;

const BEFORE: &str =
    include_str!("../../../planning/minnesota/caves/iron-crosscut-v01/before.json");
const AFTER: &str = include_str!("../../../planning/minnesota/caves/iron-crosscut-v01/after.json");
const CONTRACT: &str =
    include_str!("../../../planning/minnesota/caves/iron-crosscut-v01/contract.json");

/// Installs the reviewed crosscut terrain onto the exact V21 campaign.
///
/// Pins the original source and contract bytes, rebinds the complete-map
/// preimage and provenance, and invokes the original builder's native anchor,
/// ledge, stair, traversal and preservation guards. Does not migrate saves or
/// change the active world selection.
///
/// # Errors
/// Rejects altered campaign/source inputs, incompatible original anchors,
/// invalid native terrain or an invalid resulting runtime/geography pack.
pub fn integrate_comprehensive_iron_crosscut(
    current: &CompiledGamePack,
) -> Result<CompiledGamePack> {
    ensure!(
        current.identity()?.content_hash
            == "9bb329910b5d111550d792b3297d3b05aca381fe6aa098f8a41e576dd3e30f05",
        "reviewed V21 campaign required"
    );
    for (source, expected) in [
        (
            BEFORE,
            "6436db550946f06127cf46d671b5b5a0f01533ef7e1f200d532b98a11b44cf99",
        ),
        (
            AFTER,
            "8416ec3715365d2719408b35bcea723db684ccc7943d1ebdbabd5fb5ffe92ea9",
        ),
        (
            CONTRACT,
            "fcaee36c3fb853728af48c1d982e10c902877550028fafca69ac7be10715e280",
        ),
    ] {
        ensure!(
            format!("{:x}", Sha256::digest(source.as_bytes())) == expected,
            "reviewed crosscut source required"
        );
    }
    let mut binding: Value = serde_json::from_str(CONTRACT)?;
    let room = current
        .data
        .maps
        .get("MnIronDrillApproach")
        .context("campaign mine approach")?;
    binding["native_module_sha256"] = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&serde_json::to_value(room)?)?)
    )
    .into();
    binding["parent_pack"] =
        "content-packs/minnesota-comprehensive-native-review-v21.crystalpack".into();
    binding["parent_pack_sha256"] =
        "f4300b54e4af14db80742744147880fd759821979ce8b62d80aec079dc024617".into();
    binding["parent_revision"] = "comprehensive_research_grotto_v21".into();
    let pack =
        crate::rebuild_iron_crosscut(current, BEFORE, AFTER, &serde_json::to_string(&binding)?)?;
    crate::verify_northstar_publication_geometry(&pack)?;
    crate::validate_compact_superior_geography(&pack)?;
    Ok(pack)
}
