//! Disposable Mothlight review setup; all subsequent actions use normal inputs.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::models::pokemon::Dv;
use crystal_runtime::{CrystalRuntime, RuntimeGameShell};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let path = PathBuf::from(
        args.next()
            .context("usage: mothlight_fixture PACK SAVE [--full-bag|--inspect]")?,
    );
    let output = PathBuf::from(args.next().context("missing save")?);
    let root = AssetRoot::new(path.parent().context("pack parent")?);
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path)?,
    )?;
    let mode = args.next();
    if mode.as_deref() == Some("--next-day") {
        let mut state = runtime.load_save(&output)?;
        state.apply_daily_reset(&mut crystal_core::random::ReplayDivider::new([0, 0]))?;
        runtime.save_game(&output, state)?;
        return Ok(());
    }
    if mode.as_deref() == Some("--inspect") {
        let state = runtime.load_save(&output)?;
        let lead = state.storage.party.pokemon[0]
            .as_ref()
            .context("fixture party")?;
        println!(
            "{}",
            serde_json::json!({
                "hp":lead.hp,"max_hp":lead.max_hp,"moves":lead.moves,
                "rare_candy":state.bag.quantity(&runtime.data().items["RARE_CANDY"]),
                "found":state.flags.is_event_flag_set("EVENT_MN_MOTHLIGHT_CAKE_FOUND")?,
                "juice":state.bag.quantity(&runtime.data().items["BERRY_JUICE"]),
                "cafe_today":state.flags.is_event_flag_set("EVENT_MN_MOTHLIGHT_CAFE_WON_TODAY")?,
                "cafe_pending":state.flags.is_event_flag_set("EVENT_MN_MOTHLIGHT_CAFE_PRIZE_PENDING")?,
                "rewarded":state.flags.is_event_flag_set("EVENT_MN_MOTHLIGHT_CAKE_REWARD")?
            })
        );
        return Ok(());
    }
    ensure!(!output.try_exists()?, "choose a fresh disposable save");
    let spawn = if mode.as_deref() == Some("--lose") {
        // Fixture-only nearby blackout destination so the retry can be walked.
        let point = runtime.data().runtime_spawn_point(517)?;
        ensure!(
            point.map_name == "MnGrandMaraisPokecenter",
            "Grand Marais clinic spawn"
        );
        517
    } else {
        runtime.title_new_game_spawn_identifier()?
    };
    let mut game = RuntimeGameShell::new_game_at_runtime_tile(
        root,
        runtime,
        spawn,
        "MnMaraisCedarApproach",
        26,
        23,
    )?;
    game.set_trainer_identity("CHRIS", 1)?;
    game.add_party_pokemon(
        "KINDLIT",
        60,
        None,
        None,
        "CHRIS",
        1,
        Dv::from_non_hp(10, 10, 10, 10),
    )?;
    game.add_bag_item("MAX_REPEL", 1)?;
    game.use_bag_repel_in_field("MAX_REPEL")?;
    game.set_party_pokemon_recovery_state(0, 1, None, None)?;
    if mode.as_deref() == Some("--full-bag") {
        for item in [
            "POTION",
            "SUPER_POTION",
            "HYPER_POTION",
            "MAX_POTION",
            "FULL_RESTORE",
            "REVIVE",
            "MAX_REVIVE",
            "ANTIDOTE",
            "PARLYZ_HEAL",
            "AWAKENING",
            "BURN_HEAL",
            "ICE_HEAL",
            "FULL_HEAL",
            "ESCAPE_ROPE",
            "REPEL",
            "SUPER_REPEL",
            "MAX_REPEL",
            "X_ATTACK",
            "X_DEFEND",
            "X_SPEED",
        ] {
            game.add_bag_item(item, 1)?;
        }
    }
    game.save(output)?;
    Ok(())
}
