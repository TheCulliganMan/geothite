//! Availability of the installed machine catalog through Minnesota's native mart.

use anyhow::{Context, Result, ensure};
use crystal_core::models::ITEM_POCKET_TM_HM;
use serde::Deserialize;

use crate::CompiledGamePack;

const MANIFEST: &str = "minnesota-all-tmhm-v1";
const STORE: &str = "MnMinneapolisDepartmentStore5F";
const MART: &str = "MART_MN_MP_DEPT_5_CLERK";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Distribution {
    schema_version: u16,
    base_content_hash: String,
    hm_price: u16,
    hidden_caches: Vec<crate::minnesota_tmhm_caches::HiddenCache>,
}

/// Stocks all 50 TMs and seven reusable HMs at Minneapolis's fifth-floor counter.
///
/// Reads machine identities, move mappings, compatibility and TM prices from the
/// supplied pack. HMs receive the configured positive price so the ordinary shop
/// can sell them. Teaching, consumption and field-move badge rules are retained.
/// Also installs the source's 20 one-time hidden TM caches on existing routes
/// and in caves, with saved collection flags and no terrain changes.
///
/// # Errors
/// Rejects a different parent, a missing production counter, incomplete machine
/// data, duplicate installation, zero HM price, unsafe or overlapping cache
/// positions, or an invalid resulting pack.
pub fn build_minnesota_tmhm(base: &CompiledGamePack, source: &str) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let source: Distribution = serde_json::from_str(source)?;
    ensure!(
        source.schema_version == 1,
        "unsupported machine distribution schema"
    );
    ensure!(source.hm_price > 0, "HMs need a positive shop price");
    ensure!(
        base.identity()?.content_hash == source.base_content_hash,
        "machine distribution parent differs"
    );
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "machines already installed"
    );
    let store = base
        .data
        .maps
        .get(STORE)
        .context("installed Minneapolis fifth floor")?;
    ensure!(
        store
            .script_shop_commands
            .iter()
            .any(|command| command.mart_id == MART),
        "fifth-floor clerk must open the production machine mart"
    );
    ensure!(
        base.data.marts.0.contains_key(MART),
        "installed machine mart required"
    );
    let mut machines = base
        .data
        .items
        .values()
        .filter(|item| item.pocket == ITEM_POCKET_TM_HM)
        .collect::<Vec<_>>();
    machines.sort_by_key(|item| item.tmhm_index);
    ensure!(machines.len() == 57, "expected all 50 TMs and seven HMs");
    for (offset, item) in machines.iter().enumerate() {
        let tm = offset < 50;
        ensure!(
            item.tmhm_index == Some(offset + 1)
                && item.script_name.starts_with(if tm { "TM_" } else { "HM_" })
                && item.consumable == tm
                && item.field_usable
                && item
                    .tmhm_move
                    .as_ref()
                    .is_some_and(|id| base.data.moves.contains_key(id))
                && (!tm || item.price > 0),
            "invalid machine at index {}",
            offset + 1
        );
    }
    let stock = machines
        .iter()
        .map(|item| item.script_name.clone())
        .collect();
    let hm_ids = machines[50..]
        .iter()
        .map(|item| item.script_name.clone())
        .collect::<Vec<_>>();
    let mut pack = base.clone();
    pack.data.marts.0.insert(MART.into(), stock);
    for id in hm_ids {
        pack.data.items.get_mut(&id).context("retained HM")?.price = source.hm_price;
    }
    crate::minnesota_tmhm_caches::install(&mut pack, &source.hidden_caches)?;
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
