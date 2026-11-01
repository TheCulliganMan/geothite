//! Test-only menu save; production never imports it.
use anyhow::{Context, Result};
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::models::pokemon::Dv;
use crystal_runtime::{CrystalRuntime, RuntimeGameShell};
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let pack = PathBuf::from(
        args.next()
            .context("usage: browser_menu_fixture PACK SAVE")?,
    );
    let output = PathBuf::from(args.next().context("missing save")?);
    let root = AssetRoot::new(pack.parent().context("pack parent")?);
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&pack)?,
    )?;
    let spawn = runtime.title_new_game_spawn_identifier()?;
    let mut game =
        RuntimeGameShell::new_game_at_runtime_tile(root, runtime, spawn, "PlayersHouse2F", 3, 3)?;
    game.set_trainer_identity("CHRIS", 1)?;
    game.add_party_pokemon(
        "CYNDAQUIL",
        5,
        None,
        None,
        "TUI_TEST",
        1,
        Dv::from_non_hp(10, 10, 10, 10),
    )?;
    game.set_party_pokemon_recovery_state(0, 1, None, None)?;
    for flag in [
        "ENGINE_POKEDEX",
        "ENGINE_POKEGEAR",
        "ENGINE_MAP_CARD",
        "ENGINE_PHONE_CARD",
        "ENGINE_RADIO_CARD",
    ] {
        game.set_script_flag_for_smoke(flag)?;
    }
    game.record_pokedex_caught("CYNDAQUIL")?;
    game.initialize_permanent_phone_numbers()?;
    game.add_bag_item("POTION", 2)?;
    game.add_bag_item("POKE_BALL", 3)?;
    game.add_bag_item("ITEMFINDER", 1)?;
    let tm = game
        .runtime()
        .data()
        .items
        .iter()
        .find(|(_, item)| item.tmhm_index == Some(1))
        .map(|(id, _)| id.clone())
        .context("pack has no TM01")?;
    game.add_bag_item(&tm, 1)?;
    game.save(output)?;
    Ok(())
}
