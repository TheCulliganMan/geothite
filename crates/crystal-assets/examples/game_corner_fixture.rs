//! Explicit local test saves and read-only inspection for the MCP regression.
//! This does not earn travel, money, items or regional progression.
use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_core::save::{read_save_game_bytes_for_modpack, write_save_game_for_modpack};

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 3,
        "game_corner_fixture PACK fresh|rich|full|poor|inspect SAVE"
    );
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&args[0])?;
    let identity = loaded.save_modpack_identity()?;
    let hash = loaded.pack_identity()?.content_hash;
    let path = PathBuf::from(&args[2]);
    let data = loaded.pack().data();
    if args[1] == "inspect" {
        let save =
            read_save_game_bytes_for_modpack(&std::fs::read(&path)?, &args[2], &identity, &hash)?;
        let state = save.state();
        let items = [
            "COIN_CASE",
            "MOON_STONE",
            "RARE_CANDY",
            "PP_UP",
            "TM_THUNDER",
            "TM_BLIZZARD",
            "TM_FIRE_BLAST",
        ]
        .map(|id| (id, state.bag.quantity(&data.items[id])))
        .into_iter()
        .collect::<std::collections::BTreeMap<_, _>>();
        println!(
            "{}",
            serde_json::json!({"coins":state.coins, "money":state.money,
            "items":items, "welcome":state.flags.is_event_flag_set("EVENT_MN_GAME_CORNER_WELCOME")?,
            "slots":state.script_runtime.slot_machine, "cards":state.script_runtime.card_flip})
        );
        return Ok(());
    }
    ensure!(
        ["fresh", "rich", "full", "poor"].contains(&args[1].as_str()) && !path.exists(),
        "fresh explicit fixture output required"
    );
    let mut spawn = data
        .runtime_spawn_points
        .values()
        .next()
        .context("native spawn")?
        .clone();
    spawn.map_name = "MnMinneapolis".into();
    spawn.map_constant = "MN_MINNEAPOLIS".into();
    spawn.tile_x = 56;
    spawn.tile_y = 52;
    spawn.metatile_x = 28;
    spawn.metatile_y = 26;
    spawn.subtile_x = 0;
    spawn.subtile_y = 0;
    let music = data
        .audio
        .iter()
        .map(|a| a.id.clone())
        .collect::<BTreeSet<_>>();
    let (mut state, _) = data.start_overworld_session_from_spawn(&spawn, &music)?;
    state.player_name = "CHRIS".into();
    state.money = if args[1] == "poor" { 0 } else { 100000 };
    state.coins = match args[1].as_str() {
        "fresh" => 0,
        "poor" => 20,
        _ => 9999,
    };
    state.set_game_timer_counting(true);
    if args[1] != "fresh" {
        ensure!(
            state
                .bag
                .add_item(&data.items["COIN_CASE"], 1)
                .map_err(anyhow::Error::msg)?,
            "fixture Coin Case"
        );
    }
    if args[1] == "full" {
        for id in [
            "MOON_STONE",
            "RARE_CANDY",
            "PP_UP",
            "TM_THUNDER",
            "TM_BLIZZARD",
            "TM_FIRE_BLAST",
        ] {
            ensure!(
                state
                    .bag
                    .add_item(&data.items[id], 99)
                    .map_err(anyhow::Error::msg)?,
                "fixture full prize stack"
            );
        }
        while state.bag.items.len() < crystal_core::models::bag::ITEM_POCKET_CAPACITY {
            ensure!(
                state
                    .bag
                    .add_item(&data.items["MOON_STONE"], 99)
                    .map_err(anyhow::Error::msg)?,
                "fill every physical Item pocket slot"
            );
        }
    }
    write_save_game_for_modpack(path, state, &identity, &hash)?;
    Ok(())
}
