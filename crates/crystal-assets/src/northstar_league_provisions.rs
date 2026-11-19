//! Stock the rope and basic medicine offered at the Duluth dock before the northern journey.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;

const ROOM: &str = "MnDuluthDockSupplies";
const CLERK: &str = "MN_DULUTH_X_DOCKSUPPLIES_F1_HOST";
const SHOP: &str = "MART_MN_DULUTH_X_DOCKSUPPLIES_F1_HOST";

/// Adds the two promised goods to this counter at their ordinary regional prices.
/// Returns an error if the shop or the original range has been replaced.
pub(super) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let room = pack.data.maps.get(ROOM).context("Duluth dock supplies")?;
    ensure!(
        room.objects.iter().any(|actor| actor.script == CLERK)
            && room
                .script_shop_commands
                .iter()
                .any(|sale| sale.source_script == CLERK
                    && sale.mart_id == SHOP
                    && sale.command_index == 4),
        "expected the working dock-supply counter"
    );
    for (id, price) in [("POTION", 300), ("ESCAPE_ROPE", 550)] {
        ensure!(
            pack.data
                .items
                .get(id)
                .is_some_and(|item| item.price == price),
            "expected {id} at its native price"
        );
    }
    let stock = pack
        .data
        .marts
        .0
        .get_mut(SHOP)
        .context("dock clerk's stock")?;
    ensure!(
        stock
            .iter()
            .map(String::as_str)
            .eq(["FULL_HEAL", "HYPER_POTION", "REVIVE"]),
        "unexpected parent dock supplies"
    );
    stock.extend(["ESCAPE_ROPE".to_owned(), "POTION".to_owned()]);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CLERK, ROOM, SHOP};
    use anyhow::{Context, Result, ensure};
    use crystal_core::state::GameState;

    #[test]
    #[ignore = "requires the archive supplied through MN_CAVE_RETURNS_PACK"]
    fn cave_returns_dock_clerk_sells_the_rope_and_potion_from_the_room() -> Result<()> {
        let path = std::env::var_os("MN_CAVE_RETURNS_PACK").context("MN_CAVE_RETURNS_PACK")?;
        let pack = crate::read_verified_compiled_game_pack(path)?;
        let data = pack.data();
        let mut state = GameState::reset_wram_for_new_game();
        let sale = data.open_script_shop(&mut state, ROOM, ROOM, CLERK, 4)?;
        ensure!(
            sale.mart_id == SHOP
                && sale.inventory.iter().map(String::as_str).eq([
                    "FULL_HEAL",
                    "HYPER_POTION",
                    "REVIVE",
                    "ESCAPE_ROPE",
                    "POTION"
                ])
        );
        state.money = 549;
        ensure!(!data.buy_shop_item(&mut state, "ESCAPE_ROPE", 1)?.success);
        ensure!(state.money == 549 && state.bag.quantity(&data.items["ESCAPE_ROPE"]) == 0);
        state.money = 850;
        for item in ["ESCAPE_ROPE", "POTION"] {
            ensure!(data.buy_shop_item(&mut state, item, 1)?.success);
            ensure!(state.bag.quantity(&data.items[item]) == 1);
        }
        ensure!(state.money == 0);
        Ok(())
    }
}
