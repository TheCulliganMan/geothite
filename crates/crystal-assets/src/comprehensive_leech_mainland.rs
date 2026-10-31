//! Bind the reviewed mainland builder to the complete campaign's current town.
use anyhow::{Context, Result, ensure};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;

const PARENT: &str = "97976793b82861495b5c6a6bc13732c5ab588a9f5b0ba4fd43741163d2e305df";
const SOURCE: &str =
    include_str!("../../../planning/minnesota/towns/leech-lake-mainland-v01/after.json");
const CONTRACT: &str =
    include_str!("../../../planning/minnesota/towns/leech-lake-mainland-v01/contract.json");

/// Installs the reviewed mainland trail while retaining the complete campaign.
///
/// Pins the complete V19 input, authored source and original contract binding.
/// Rebinds only provenance and the complete town preimage before invoking the
/// original mainland builder, including all its traversal/preservation checks.
/// Existing saves and borrowed inputs remain unchanged.
///
/// # Errors
/// Rejects altered campaign/source/contract inputs, invalid native geometry,
/// occupied identities, nonregional wildlife or invalid resulting runtime data.
pub fn integrate_comprehensive_leech_mainland(
    current: &CompiledGamePack,
) -> Result<CompiledGamePack> {
    let identity = crate::derive_compiled_game_pack_identity_from_manifest(
        current.format_version,
        &current.data,
        &current.audio_manifest,
        &current.runtime_files,
        &current.report,
    )?;
    ensure!(
        identity.content_hash == PARENT,
        "reviewed V19 campaign required"
    );
    ensure!(
        format!("{:x}", Sha256::digest(SOURCE.as_bytes()))
            == "6c314a65e5f0c2feca8171584a701f04ae7a0807851228a4c6c9b3a146a792f4",
        "reviewed complete mainland source required"
    );
    let mut contract: Value = serde_json::from_str(CONTRACT)?;
    ensure!(
        contract["parent_pack_sha256"]
            == "20fcdad6de43bb2e46a41faa49d7a8916cfe5bd2224680709f135166d3d2801b"
            && contract["native_module_sha256"]
                == "e1d9562e57acedb3f86048893c4ba11977c27deecafd777dad98de1621f82419",
        "reviewed original mainland contract required"
    );
    let lake = current
        .data
        .maps
        .get("MnLeechLake")
        .context("current campaign lake")?;
    contract["native_module_sha256"] = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&serde_json::to_value(lake)?)?)
    )
    .into();
    contract["parent_pack"] =
        "content-packs/minnesota-comprehensive-native-review-v19.crystalpack".into();
    contract["parent_pack_sha256"] =
        "02c4278605188d7c833a149930088a5d691bfccc2ba6000bb838a9c65ba4da3a".into();
    contract["parent_revision"] = "comprehensive_water_habitats_v19".into();
    let pack =
        crate::rebuild_leech_lake_mainland(current, SOURCE, &serde_json::to_string(&contract)?)?;
    ensure!(
        pack.identity()?.content_hash
            == "8c847f3ae7628d8117bebe571fb14a437fbe3108186dd88cfa6ae79054c995af",
        "mainland integration differs from the independently preserved V20 child"
    );
    crate::verify_northstar_publication_geometry(&pack)?;
    crate::validate_compact_superior_geography(&pack)?;
    Ok(pack)
}
