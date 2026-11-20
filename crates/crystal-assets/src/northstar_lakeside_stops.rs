//! Minnesota's lakeside services and city shop entrances.
#[path = "northstar_shore_bakery.rs"]
mod bakery;
#[path = "northstar_capitol_chart.rs"]
mod library;
#[path = "northstar_depot_garden.rs"]
mod depot;
#[path = "northstar_store_elevator.rs"]
mod elevator;
#[path = "northstar_shop_landings.rs"]
mod shops;

use crate::CompiledGamePack;
use anyhow::{Result, ensure};

/// Adds the Minnesota store's working elevator and the region's lakeside services.
///
/// # Errors
/// Returns an error if the input is not an Arcade Frontage edition, a service
/// no longer matches its authored location, or the resulting archive is invalid.
pub fn revise_lakeside_stops(mut pack: CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        matches!(
            pack.identity.content_hash.as_str(),
            "a23ccea3c5bec079160017d4600a67e7e1c7d653d5891ef6775f5a5fcdab3ca8"
                | "0886c571cd0dd9b49906b995d7ab9570f754e575a460a0291875a18a17554e66"
        ) && pack.report.manifests.last().map(String::as_str)
            == Some("northstar-arcade-frontage-v1"),
        "lakeside stops require a verified Arcade Frontage edition"
    );
    shops::install(&mut pack)?;
    elevator::install(&mut pack)?;
    bakery::install(&mut pack)?;
    library::install(&mut pack)?;
    depot::install(&mut pack)?;
    pack.report.maps = pack.data.maps.len();
    pack.report.items = pack.data.items.len();
    pack.report
        .manifests
        .push("northstar-lakeside-stops-v1".into());
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
