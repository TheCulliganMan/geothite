//! Minnesota's prepared foods, from the fair stalls to the North Shore.
#[path = "northstar_fair_drinks.rs"]
mod drinks;
#[path = "northstar_food_neighbors.rs"]
mod food_neighbors;
#[path = "northstar_fair_foods.rs"]
mod foods;

use anyhow::{Context, Result, ensure};

use crate::{CompiledGamePack, authored_town_publication};

/// Registers local food, updates the counters and teaches partners to hold a serving.
///
/// # Errors
/// Returns an error for an unexpected input edition, missing counter or invalid pack.
pub fn revise_fair_table(mut pack: CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        matches!(
            pack.identity.content_hash.as_str(),
            "14c2b902b7bfe0173bbdbea1f0baa6eff97419f30368730498869b56225adedb"
                | "22d0622fc6313f1ca6c18677f1a973c541b97efcda0e2abbbd634bf9cf59591e"
        ) && pack.report.manifests.last().map(String::as_str)
            == Some("northstar-lakeside-stops-v1"),
        "fair table requires a verified Lakeside Stops edition"
    );
    foods::install(&mut pack)?;
    drinks::install(&mut pack)?;
    food_neighbors::install(&mut pack)?;
    let room = "MnNorthShorePieShop";
    let label = "MN_SHORE_PIE_GUESTText";
    let mut map = pack.data.maps.get(room).context("pie counter")?.clone();
    ensure!(map.scripts.contains_key(label), "missing {room}/{label}");
    food_neighbors::install_text(&mut map, label)?;
    map.script_text_bodies = crate::parse_script_text_bodies(room, &map.scripts)?;
    authored_town_publication::replace(&mut pack, map)?;
    pack.report.items = pack.data.items.len();
    pack.report.manifests.push("northstar-fair-table-v1".into());
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
