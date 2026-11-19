//! Exercise the published pocket from its sole walking approach, including full Bag and Continue.
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, ensure};
use crystal_core::models::{BALL_POCKET_CAPACITY, MAX_ITEM_STACK};
use crystal_core::save;
use crystal_core::state::GameState;
use crystal_core::systems::field_items::FieldItemPickupOutcome;
use crystal_core::systems::map_context::SpawnMemoryUpdate;
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::session::{OverworldInteractionTarget, OverworldSession};

use super::{FEN, PICKUP};
use crate::{GameDataSet, LoadedCompiledGamePack};

const CLAIM: &str = "EVENT_MN_HIDDEN_FEN_SOUTH_BALL";

fn on_facing_a(
    data: &GameDataSet,
    state: &GameState,
    session: &OverworldSession,
    claimed: bool,
) -> Result<()> {
    let hit = session
        .check_interaction_checked(1)
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .context("southern fir event on the tile in front")?;
    ensure!(
        hit.script == PICKUP
            && hit.player_tile == TilePosition::new(8, 22)
            && hit.target_tile == TilePosition::new(8, 23)
            && hit.facing == Direction::Down
            && matches!(&hit.target, OverworldInteractionTarget::Background { event_type } if event_type == "BGEVENT_ITEM"),
        "the A press must read the floor pocket beside the fir, not the fir itself"
    );
    ensure!(
        data.resolve_overworld_interaction(state, &hit)?.is_some() != claimed,
        "collected hidden item is still eligible for an A interaction"
    );
    let found = data.find_itemfinder_hidden_item(state, FEN, hit.player_tile)?;
    if claimed {
        ensure!(found.is_none(), "Itemfinder still reports the empty pocket");
    } else {
        let found = found.context("the Itemfinder missed the spare ball")?;
        ensure!(
            found.tile == hit.target_tile
                && found.item_id == "ULTRA_BALL"
                && found.event_flag == CLAIM
        );
    }
    Ok(())
}

fn collect(
    data: &GameDataSet,
    state: &mut GameState,
    session: &mut OverworldSession,
) -> Result<FieldItemPickupOutcome> {
    data.pickup_script_field_item_in_session(state, session, FEN, PICKUP, 0)
}

fn round_trip(loaded: &LoadedCompiledGamePack, state: GameState, file: &Path) -> Result<GameState> {
    let identity = loaded.save_modpack_identity()?;
    let hash = loaded.pack_content_hash()?;
    save::write_save_game_for_modpack(file, state, &identity, &hash)?;
    Ok(save::read_save_game_for_modpack(file, &identity, &hash)?.into_state())
}

fn verify(loaded: &LoadedCompiledGamePack, file: &Path) -> Result<()> {
    let pack = loaded.pack();
    let data = pack.data();
    let music = pack.audio_manifest.music.keys().cloned().collect();
    let (mut state, mut session) =
        data.start_overworld_session_at_runtime_tile(FEN, TilePosition::new(8, 22), &music)?;
    session.player.facing = Direction::Down;
    let ordinary = &data.items["POKE_BALL"];
    let reward = &data.items["ULTRA_BALL"];
    for _ in 0..BALL_POCKET_CAPACITY {
        ensure!(
            state
                .bag
                .add_item(ordinary, MAX_ITEM_STACK)
                .map_err(anyhow::Error::msg)?,
            "fill a Ball pocket stack"
        );
    }
    on_facing_a(data, &state, &session, false)?;
    ensure!(
        matches!(collect(data,&mut state,&mut session)?, FieldItemPickupOutcome::BagFull{event_flag,..} if event_flag == CLAIM)
    );
    ensure!(!state.flags.is_event_flag_set(CLAIM)? && state.bag.quantity(reward) == 0);
    data.commit_overworld_snapshot(&mut state, &session, SpawnMemoryUpdate::Preserve);
    let (mut state, mut session) =
        data.resume_overworld_session_from_state(round_trip(loaded, state, file)?, &music)?;
    on_facing_a(data, &state, &session, false)?;
    ensure!(
        state
            .bag
            .remove_item(ordinary, MAX_ITEM_STACK)
            .map_err(anyhow::Error::msg)?
    );
    ensure!(
        matches!(collect(data,&mut state,&mut session)?, FieldItemPickupOutcome::Collected{item_id,quantity:1,event_flag,..} if item_id == "ULTRA_BALL" && event_flag == CLAIM)
    );
    data.commit_overworld_snapshot(&mut state, &session, SpawnMemoryUpdate::Preserve);
    let (mut state, mut session) =
        data.resume_overworld_session_from_state(round_trip(loaded, state, file)?, &music)?;
    on_facing_a(data, &state, &session, true)?;
    ensure!(
        matches!(collect(data,&mut state,&mut session)?, FieldItemPickupOutcome::AlreadyCollected{event_flag,..} if event_flag == CLAIM)
    );
    ensure!(state.bag.quantity(reward) == 1 && state.flags.is_event_flag_set(CLAIM)?);
    Ok(())
}

#[test]
#[ignore = "requires the verified Fen Clearings archive in MN_FEN_CACHE_PACK"]
fn southern_fen_spare_ball_full_pocket_itemfinder_and_continue() -> Result<()> {
    let archive =
        PathBuf::from(std::env::var_os("MN_FEN_CACHE_PACK").context("set MN_FEN_CACHE_PACK")?);
    let loaded = crate::read_loaded_verified_compiled_game_pack(archive)?;
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let temp =
        std::env::temp_dir().join(format!("geothite-fen-ball-{}-{stamp}", std::process::id()));
    fs::create_dir(&temp)?;
    let result = verify(&loaded, &temp.join("fen.crystalsave"));
    fs::remove_dir_all(temp)?;
    result
}
