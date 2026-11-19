//! Exercise the installed crew-book read and held Ether across a real save boundary.
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, ensure};
use crystal_core::models::bag::{ITEM_POCKET_CAPACITY, MAX_ITEM_STACK};
use crystal_core::save;
use crystal_core::state::GameState;
use crystal_core::systems::map_context::SpawnMemoryUpdate;
use crystal_core::systems::script_control::ScriptControlAction;
use crystal_core::systems::script_items::ScriptItemGrantOutcome;
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::session::{OverworldInteractionTarget, OverworldSession};

use super::{BOOK, ENTERED, NAMES, RECEIPT, UPSTAIRS};
use crate::{GameDataSet, LoadedCompiledGamePack};

const RECORDER: &str = "MN_DULUTH_X_UNIONHALL_F2_HOST";

fn save_and_continue(loaded: &LoadedCompiledGamePack, state: GameState) -> Result<GameState> {
    let unique = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "geothite-crew-book-{}-{unique}",
        std::process::id()
    ));
    let pack_id = loaded.save_modpack_identity()?;
    let content_hash = loaded.pack_content_hash()?;
    fs::create_dir(&directory)?;
    let path = directory.join("pending-ether.crystalsave");
    let result = save::write_save_game_for_modpack(&path, state, &pack_id, &content_hash)
        .and_then(|()| save::read_save_game_for_modpack(&path, &pack_id, &content_hash));
    fs::remove_dir_all(directory)?;
    Ok(result?.into_state())
}

fn read_ledger(
    data: &GameDataSet,
    state: &mut GameState,
    session: &OverworldSession,
    legacy: bool,
) -> Result<()> {
    let hit = session
        .check_interaction_checked(1)
        .map_err(|err| anyhow::anyhow!("{err}"))?
        .context("expected the public book beside the player")?;
    ensure!(
        hit.player_tile == TilePosition::new(8, 7)
            && hit.target_tile == TilePosition::new(9, 7)
            && hit.facing == Direction::Right
            && hit.script == BOOK
            && matches!(hit.target, OverworldInteractionTarget::Background { event_type } if event_type == "BGEVENT_READ"),
        "expected the native east-facing book read"
    );
    let prior = data.check_script_flag_in_session(state, session, UPSTAIRS, BOOK, 0)?;
    ensure!(prior.set == legacy, "incorrect older receipt");
    let mut action =
        data.apply_script_control_command_in_session(state, session, UPSTAIRS, BOOK, 1)?;
    if !legacy {
        ensure!(
            matches!(action, ScriptControlAction::Continue { .. }),
            "the unclaimed Ether bypassed the inscription"
        );
        ensure!(
            data.check_script_flag_in_session(state, session, UPSTAIRS, BOOK, 2)?
                .set,
            "missing names on Continue"
        );
        action = data.apply_script_control_command_in_session(state, session, UPSTAIRS, BOOK, 3)?;
    }
    let ScriptControlAction::Jump { target_script, .. } = action else {
        anyhow::bail!("public ledger still shows the blank heading");
    };
    let page = data
        .script_text_command(UPSTAIRS, &target_script, 0)?
        .text_label
        .as_deref()
        .context("inscribed names")?;
    let body = data.script_text_body_for_map(UPSTAIRS, page)?;
    for name in ["Nora Berg", "Alma Lind", "Gus Ward"] {
        ensure!(
            body.commands
                .iter()
                .flat_map(|r| &r.args)
                .any(|line| line.trim_matches('"') == name),
            "ledger is missing {name}"
        );
    }
    Ok(())
}

#[test]
#[ignore = "requires the external archive in MN_TOWN_RETURNS_PACK"]
fn crew_book_and_held_ether_survive_same_pack_continue() -> Result<()> {
    let pack_path = PathBuf::from(
        std::env::var_os("MN_TOWN_RETURNS_PACK").context("Minnesota Town Returns pack")?,
    );
    let loaded = crate::read_loaded_verified_compiled_game_pack(pack_path)?;
    let pack = loaded.pack();
    let data = pack.data();
    let music = pack.audio_manifest.music.keys().cloned().collect();
    let (mut state, mut session) =
        data.start_overworld_session_at_runtime_tile(UPSTAIRS, TilePosition::new(8, 7), &music)?;
    session.player.facing = Direction::Right;
    for flag in ["EVENT_MN_INTERIOR_UNION_WELCOME", NAMES] {
        state.flags.set_event_flag(flag, true)?;
    }
    let potion = &data.items["POTION"];
    let ether = &data.items["ETHER"];
    for _ in 0..ITEM_POCKET_CAPACITY {
        ensure!(
            state
                .bag
                .add_item(potion, MAX_ITEM_STACK)
                .map_err(anyhow::Error::msg)?,
            "fill a native item pocket slot"
        );
    }
    data.apply_script_flag_mutation_in_session(
        &mut state,
        &mut session,
        UPSTAIRS,
        "MnUnionWriteNames",
        4,
    )?;
    let grant = data.grant_script_item_in_session(&mut state, &session, UPSTAIRS, RECORDER, 10)?;
    ensure!(
        matches!(grant, ScriptItemGrantOutcome::BagFull { item_id, quantity: 1, .. } if item_id == "ETHER"),
        "the full pocket accepted an Ether"
    );
    ensure!(
        state.flags.is_event_flag_set(ENTERED)?
            && !state.flags.is_event_flag_set(RECEIPT)?
            && state.bag.quantity(ether) == 0,
        "recording the crew must be independent from receiving Ether"
    );

    data.commit_overworld_snapshot(&mut state, &session, SpawnMemoryUpdate::Preserve);
    let saved = save_and_continue(&loaded, state)?;
    let (mut state, mut session) = data.resume_overworld_session_from_state(saved, &music)?;
    read_ledger(data, &mut state, &session, false)?;
    ensure!(
        state.bag.quantity(ether) == 0
            && state.bag.quantity(potion) == ITEM_POCKET_CAPACITY as u16 * MAX_ITEM_STACK,
        "Continue changed the pocket"
    );
    ensure!(
        data.check_script_flag_in_session(&mut state, &session, UPSTAIRS, "MnUnionWriteNames", 0)?
            .set
    );
    let held = data.apply_script_control_command_in_session(
        &mut state,
        &session,
        UPSTAIRS,
        "MnUnionWriteNames",
        1,
    )?;
    ensure!(
        matches!(held, ScriptControlAction::Jump { target_script, .. } if target_script == "MnUnionHoldEther"),
        "the recorder asks for the crew list a second time"
    );
    ensure!(
        state
            .bag
            .remove_item(potion, MAX_ITEM_STACK)
            .map_err(anyhow::Error::msg)?
    );
    let grant = data.grant_script_item_in_session(&mut state, &session, UPSTAIRS, RECORDER, 10)?;
    ensure!(
        matches!(grant, ScriptItemGrantOutcome::Granted { item_id, quantity: 1, .. } if item_id == "ETHER")
    );
    data.apply_script_flag_mutation_in_session(&mut state, &mut session, UPSTAIRS, RECORDER, 12)?;
    ensure!(state.bag.quantity(ether) == 1 && state.flags.is_event_flag_set(RECEIPT)?);
    ensure!(
        data.check_script_flag_in_session(&mut state, &session, UPSTAIRS, RECORDER, 2)?
            .set
    );
    let claimed =
        data.apply_script_control_command_in_session(&mut state, &session, UPSTAIRS, RECORDER, 3)?;
    ensure!(
        matches!(claimed, ScriptControlAction::Jump { target_script, .. } if target_script == format!("{RECORDER}InteriorRepeat")),
        "the recorder offers Ether again after the receipt"
    );

    // Older completed saves have the original Ether receipt and no separate names event.
    state.flags.set_event_flag(ENTERED, false)?;
    read_ledger(data, &mut state, &session, true)?;
    Ok(())
}
