//! Exercise Dig and Escape Rope with the installed Minnesota cave warp indices.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::input::GameButton;
use crystal_core::models::pokemon::Dv;
use crystal_core::random::ReplayDivider;
use crystal_core::state::GameState;
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::session::OverworldSession;

use crate::{
    GameDataSet, RuntimeMutationCommand, RuntimeMutationResult, RuntimeScriptEventDrainCommand,
    RuntimeScriptEventDrainResult, RuntimeScriptEventQueue, read_verified_compiled_game_pack,
};

fn step_through(
    data: &GameDataSet,
    state: &mut GameState,
    room: &mut OverworldSession,
    music: &BTreeSet<String>,
    trigger: Direction,
    expected: &str,
) -> Result<()> {
    let button = match trigger {
        Direction::Up => GameButton::Up,
        Direction::Right => GameButton::Right,
        _ => anyhow::bail!("unreviewed cave fixture direction"),
    };
    room.player.facing = trigger;
    let outcome = data.apply_overworld_input(
        state,
        room,
        [button],
        music,
        &mut ReplayDivider::new([0; 64]),
    )?;
    ensure!(
        outcome.warp.is_some() && room.map.name == expected,
        "expected warp into {expected}"
    );
    Ok(())
}

#[test]
#[ignore = "requires the Minnesota archive named by MN_CAVE_RETURNS_PACK"]
fn dig_and_escape_rope_return_from_second_floor_to_actual_entrance() -> Result<()> {
    let path = std::env::var_os("MN_CAVE_RETURNS_PACK").context("MN_CAVE_RETURNS_PACK")?;
    let pack = read_verified_compiled_game_pack(path)?;
    let data = pack.data();
    let music = pack.audio_manifest()?.music.into_keys().collect();
    // Place the fixture immediately before each real warp. Traversal of the intervening
    // cave floor is checked independently. Each crossing and field action runs in production.
    for (road, x, y, id, first, ladder, dir, second) in [
        (
            "MnR20C01R00",
            6,
            33,
            3,
            "MnVoyageursIce1F",
            (23, 6),
            Direction::Up,
            "MnVoyageursIceB1F",
        ),
        (
            "MnR01C00R00",
            18,
            17,
            1,
            "MnR01Cave1F",
            (10, 9),
            Direction::Right,
            "MnDriftlessLampGallery",
        ),
        (
            "MnR01C00R00",
            36,
            19,
            7,
            "MnR01Cave1F",
            (10, 9),
            Direction::Right,
            "MnDriftlessLampGallery",
        ),
    ] {
        let (mut state, mut room) = data.start_overworld_session_at_runtime_tile(
            road,
            TilePosition::new(x, y + 1),
            &music,
        )?;
        let digger = data.create_pokemon("SODCALF", 24, Dv::default())?;
        ensure!(
            state.storage.party.add_pokemon(digger),
            "party already full"
        );
        state.sync_party_from_storage();
        let rope = &data.items["ESCAPE_ROPE"];
        ensure!(
            state.bag.add_item(rope, 1).map_err(anyhow::Error::msg)?,
            "could not add fixture Escape Rope"
        );
        step_through(data, &mut state, &mut room, &music, Direction::Up, first)?;
        ensure!(
            state.dig_warp_map_name.as_deref() == Some(road) && state.dig_warp_index == Some(id),
            "the entry did not remember {road} #{id}"
        );
        room.player.tile = TilePosition::new(ladder.0, ladder.1);
        step_through(data, &mut state, &mut room, &music, dir, second)?;
        if second == "MnDriftlessLampGallery" {
            // The gallery's object callback hides its unread cache. The shell
            // presents the completed branch before the player can open Save.
            let presented = data.apply_runtime_mutation_command(
                &mut state,
                &mut room,
                RuntimeMutationCommand::DrainScriptEventQueue(RuntimeScriptEventDrainCommand {
                    queue: RuntimeScriptEventQueue::Control,
                }),
                &music,
                &BTreeSet::new(),
                &BTreeSet::new(),
            )?;
            let RuntimeMutationResult::ScriptEventQueueDrained(
                RuntimeScriptEventDrainResult::Control(events),
            ) = presented.result
            else {
                anyhow::bail!("missing Lamp Gallery object callback");
            };
            ensure!(
                events
                    .iter()
                    .map(|e| (e.source_script.as_str(), e.command_index))
                    .eq([
                        ("MnLampObjects", 1),
                        ("MnLampObjects", 3),
                        ("MnLampHide", 1),
                    ]),
                "unexpected Lamp Gallery callback events: {events:?}"
            );
        }
        let save = serde_json::to_vec(&state)?;
        for use_rope in [false, true] {
            let (mut continued, mut session) =
                data.resume_overworld_session_from_state(serde_json::from_slice(&save)?, &music)?;
            if use_rope {
                data.use_bag_escape_rope_in_session(
                    &mut continued,
                    &mut session,
                    "ESCAPE_ROPE",
                    &music,
                )?;
            } else {
                data.use_dig_field_move_in_session(&mut continued, &mut session, 0, &music)?;
            }
            let pending = data.commit_pending_field_travel(&mut continued, &mut session, &music)?;
            ensure!(
                pending.destination_map == road
                    && pending.destination_warp_index == Some(id)
                    && session.map.name == road
                    && session.player.tile == TilePosition::new(x, y),
                "wrong escape destination for {road} #{id}"
            );
            ensure!(
                continued.bag.quantity(rope) == if use_rope { 0 } else { 1 },
                "unexpected Escape Rope consumption"
            );
            if use_rope {
                // Using a bag item owns input until the presentation consumes the
                // item-use notice. Run the same drain command as the native shell.
                let presentation = data.apply_runtime_mutation_command(
                    &mut continued,
                    &mut session,
                    RuntimeMutationCommand::DrainScriptEventQueue(RuntimeScriptEventDrainCommand {
                        queue: RuntimeScriptEventQueue::ItemUse,
                    }),
                    &music,
                    &BTreeSet::new(),
                    &BTreeSet::new(),
                )?;
                ensure!(
                    matches!(
                        presentation.result,
                        RuntimeMutationResult::ScriptEventQueueDrained(
                            RuntimeScriptEventDrainResult::ItemUse(ref events)
                        ) if matches!(events.as_slice(), [event] if event.item_id == "ESCAPE_ROPE" && event.consumed)
                    ),
                    "missing presentation of the consumed Escape Rope"
                );
            }
            let step = data.apply_overworld_input(
                &mut continued,
                &mut session,
                [],
                &music,
                &mut ReplayDivider::new([0; 64]),
            )?;
            ensure!(
                step.warp.is_none()
                    && session.map.name == road
                    && session.player.facing == Direction::Down
                    && session.player.tile == TilePosition::new(x, y + 1),
                "failed to step clear of {road} #{id} after {}: {:?} at {:?}, movement {:?}",
                pending.move_id,
                session.forced_movement_direction(),
                session.player.tile,
                step.movement
            );
        }
    }
    Ok(())
}
