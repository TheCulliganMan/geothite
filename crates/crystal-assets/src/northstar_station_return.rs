//! Bring either approach to Pouchid's capsule back to Tamarack's desk.

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::CompiledGamePack;
use crate::authored_town_publication;
use crate::grand_marais_modpack::authoring::command;

const STATION: &str = "MnWinonaFieldStation";
const CAPSULE: &str = "MN_W_CHIKORITA";
const FRONT_STEPS: &str = "MnOverhaulStarterReturn0";
const RETURN: &str = "MnStationPouchidReturn";
const AISLE: &str = "MnStationPouchidFromAisle";
const AISLE_STEPS: &str = "MnStationPouchidAisleSteps";

/// Builds the Winona station return revision from a Lake Observations edition.
///
/// # Errors
/// Returns an error for an incompatible parent, unexpected starter scene or
/// invalid resulting pack.
pub fn revise_station_return(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        base.report.manifests.last().map(String::as_str) == Some("northstar-lake-observations-v1"),
        "expected Minnesota Lake Observations"
    );
    let mut pack = base.clone();
    apply(&mut pack)?;
    pack.report
        .manifests
        .push("northstar-station-return-v1".into());
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

/// Stages the starter handoff from the aisle as well as the front of the table.
///
/// Returns an error if the capsule, desk or original handoff has moved.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack.data.maps.get(STATION).context(STATION)?.clone();
    for (id, x, y) in [(CAPSULE, 6, 3), ("MN_W_TAMARACK", 5, 2)] {
        ensure!(
            room.objects.iter().any(|actor| {
                actor.object_identifier.as_deref() == Some(id) && actor.x == x && actor.y == y
            }),
            "unexpected station position for {id}"
        );
    }
    let front = json!([
        command("step", &["LEFT"]),
        command("step", &["LEFT"]),
        command("step", &["UP"]),
        command("step", &["UP"]),
        command("step_end", &[]),
    ]);
    ensure!(
        room.scripts.get(FRONT_STEPS) == Some(&front),
        "unexpected Pouchid front approach"
    );
    let walk = command("applymovement", &["PLAYER", FRONT_STEPS]);
    let gift = room
        .scripts
        .get_mut(CAPSULE)
        .and_then(Value::as_array_mut)
        .context("Pouchid gift script")?;
    let site = gift
        .windows(3)
        .position(|rows| {
            rows[0] == command("disappear", &[CAPSULE])
                && rows[1] == walk
                && rows[2] == command("turnobject", &["PLAYER", "RIGHT"])
        })
        .context("Pouchid's original walk back to Tamarack")?;
    gift[site + 1] = command("scall", &[RETURN]);

    // The table is accessible from (6,4) and from the aisle at (5,3).
    // Both walks finish at (4,2), west of Tamarack and north of Reed.
    for (label, body) in [
        (
            RETURN,
            json!([
                command("readvar", &["VAR_XCOORD"]),
                command("ifequal", &["5", AISLE]),
                walk,
                command("end", &[]),
            ]),
        ),
        (
            AISLE,
            json!([
                command("applymovement", &["PLAYER", AISLE_STEPS]),
                command("end", &[]),
            ]),
        ),
        (
            AISLE_STEPS,
            json!([
                command("step", &["LEFT"]),
                command("step", &["UP"]),
                command("step_end", &[]),
            ]),
        ),
    ] {
        ensure!(
            room.scripts.insert(label.into(), body).is_none(),
            "occupied station movement label {label}"
        );
    }
    room.script_control_commands = crate::parse_script_control_commands(STATION, &room.scripts)?;
    room.script_variable_commands = crate::parse_script_variable_commands(STATION, &room.scripts)?;
    room.script_object_commands = crate::parse_script_object_commands(STATION, &room.scripts)?;
    room.script_movements =
        crate::parse_script_movements(STATION, &room.scripts, &room.script_object_commands)?;
    authored_town_publication::replace(pack, room)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use anyhow::{Context, Result, ensure};
    use crystal_core::state::GameState;
    use crystal_core::systems::script_control::ScriptControlAction;
    use crystal_core::world::map::TilePosition;

    use super::{AISLE, CAPSULE, RETURN, STATION};

    #[test]
    #[ignore = "requires the Lake parent in MN_STATION_RETURN_PARENT"]
    fn station_return_original_aisle_walk_finishes_at_the_computer() -> Result<()> {
        let path =
            PathBuf::from(std::env::var_os("MN_STATION_RETURN_PARENT").context("Lake parent")?);
        let parent = crate::read_verified_compiled_game_pack(path)?;
        let data = parent.data();
        let mut state = GameState::reset_wram_for_new_game();
        let mut session = data.overworld_session(STATION, TilePosition::new(5, 3), 0)?;
        let walk =
            data.apply_script_movement_in_session(&mut state, &mut session, STATION, CAPSULE, 24)?;
        ensure!(
            walk.tile == TilePosition::new(3, 1) && session.player.tile == walk.tile,
            "original Pouchid aisle walk no longer reaches the computer"
        );
        Ok(())
    }

    #[test]
    #[ignore = "requires a built edition in MN_STATION_RETURN_PACK"]
    fn station_return_four_capsule_approaches_finish_beside_tamarack() -> Result<()> {
        let path = PathBuf::from(std::env::var_os("MN_STATION_RETURN_PACK").context("built pack")?);
        let pack = crate::read_verified_compiled_game_pack(path)?;
        let data = pack.data();
        for (capsule, x, y, count) in [
            (CAPSULE, 5, 3, 2),
            (CAPSULE, 6, 4, 4),
            ("MN_W_CYNDAQUIL", 7, 4, 5),
            ("MN_W_TOTODILE", 8, 4, 6),
        ] {
            let mut state = GameState::reset_wram_for_new_game();
            let at = TilePosition::new(x, y);
            let mut session = data.overworld_session(STATION, at, 0)?;
            let (caller, index) = if capsule == CAPSULE {
                let action = data.apply_script_control_command_in_session(
                    &mut state, &session, STATION, capsule, 24,
                )?;
                ensure!(
                    matches!(action, ScriptControlAction::Jump { target_script, call: true, .. } if target_script == RETURN),
                    "gift did not call player return"
                );
                data.apply_script_variable_command_now_in_session(
                    &mut state, &session, STATION, RETURN, 0,
                )?;
                let side = data.apply_script_control_command_in_session(
                    &mut state, &session, STATION, RETURN, 1,
                )?;
                match side {
                    ScriptControlAction::Jump {
                        target_script,
                        call: false,
                        ..
                    } if target_script == AISLE => (AISLE, 0),
                    ScriptControlAction::Continue { .. } => (RETURN, 2),
                    _ => anyhow::bail!("unexpected native branch from ({x},{y})"),
                }
            } else {
                (capsule, 24)
            };
            let walk = data.apply_script_movement_in_session(
                &mut state,
                &mut session,
                STATION,
                caller,
                index,
            )?;
            ensure!(
                walk.previous_tile == at
                    && walk.steps_applied == count
                    && walk.tile == TilePosition::new(4, 2),
                "{capsule} at ({x},{y}) missed Tamarack: {walk:?}"
            );
            if capsule == CAPSULE {
                data.apply_script_control_command_in_session(
                    &mut state,
                    &session,
                    STATION,
                    caller,
                    index + 1,
                )?;
                let frame = state
                    .script_runtime
                    .call_stack
                    .last()
                    .context("return to gift")?;
                ensure!(
                    frame.source_script == capsule && frame.next_command_index == 25,
                    "resume the gift at the command that faces Tamarack"
                );
            }
        }
        Ok(())
    }
}
