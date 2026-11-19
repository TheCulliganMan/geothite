//! The final road to Ely, League chamber exits, and the Champion's trip home.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;

const RESTORE: &str = "FULL_RESTORE";
const MART: &str = "MART_MN_ELY";

/// Builds the League Journey edition from a Fen Clearings archive.
///
/// # Errors
/// Returns an error for an unexpected parent, incompatible scenery or scripts,
/// or an invalid resulting archive.
pub fn revise_league_journey(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        base.report.manifests.last().map(String::as_str) == Some("northstar-fen-clearings-v1"),
        "League Journey requires a Fen Clearings edition"
    );
    let mut pack = base.clone();
    crate::northstar_league_approach_pass::apply(&mut pack)?;
    crate::northstar_league_rooms_pass::apply(&mut pack)?;
    crate::northstar_after_champion_pass::apply(&mut pack)?;
    crate::northstar_clinic_terminals::apply(&mut pack)?;

    // Keep the medicine available on return visits. The pine camp's one-time
    // field supply still rewards exploration before a first challenge.
    let stock = pack.data.marts.0.get_mut(MART).context(MART)?;
    ensure!(
        stock
            == &[
                "ULTRA_BALL",
                "HYPER_POTION",
                "MAX_POTION",
                "FULL_HEAL",
                "REVIVE",
                "MAX_REPEL"
            ],
        "unexpected stock at Ely's clerk"
    );
    stock.insert(3, RESTORE.into());
    pack.report
        .manifests
        .push("northstar-league-journey-v1".into());
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

#[cfg(test)]
mod tests {
    use anyhow::{Context, Result, ensure};
    use crystal_core::state::GameState;

    use super::{MART, RESTORE};

    #[test]
    #[ignore = "requires a League Journey archive in MN_LEAGUE_JOURNEY_PACK"]
    fn league_journey_ely_clerk_sells_full_restores_at_the_native_price() -> Result<()> {
        let path = std::env::var_os("MN_LEAGUE_JOURNEY_PACK").context("MN_LEAGUE_JOURNEY_PACK")?;
        let pack = crate::read_verified_compiled_game_pack(path)?;
        let data = pack.data();
        let item = data.items.get(RESTORE).context(RESTORE)?;
        let mut state = GameState::reset_wram_for_new_game();
        let shop = data.open_script_shop(&mut state, "MnElyMart", "MnElyMart", "MnElyClerk", 2)?;
        ensure!(shop.mart_id == MART && shop.inventory.iter().any(|entry| entry == RESTORE));
        state.money = 2999;
        ensure!(!data.buy_shop_item(&mut state, RESTORE, 1)?.success);
        ensure!(state.money == 2999 && state.bag.quantity(item) == 0);
        state.money = 6000;
        ensure!(data.buy_shop_item(&mut state, RESTORE, 2)?.success);
        ensure!(state.money == 0 && state.bag.quantity(item) == 2);
        Ok(())
    }
}
