//! Uncover the Harbor Gym's west skating lane once Norek awards the badge.
use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::CompiledGamePack;
use crate::authored_dialogue;
use crate::grand_marais_modpack::authoring::command;

const GYM: &str = "MnDuluthGym";
const CLOSED: &str = "MnDuluthGymV2Gate1Closed";
const OPEN: &str = "MnDuluthGymV2Gate1Open";
const VICTORY: &str = "MnDuluthGymLeaderWonText";
const WEST: &str = "EVENT_MN_GYM_8_WEST_RETURN_OPEN";
const WIN: &str = "EVENT_MN_GYM_8_DEFEATED";

// Metatile coordinates with the installed first-visit block and the exposed rink block.
// Livra stands on the top-right quadrant of the second block, which stays solid footing.
const LANE: [(u16, u16, u16, u16); 4] = [(1, 4, 5, 7), (1, 5, 1, 8), (1, 8, 1, 7), (2, 8, 1, 4)];

/// Publishes the west lane on the existing victory callback and the leader's directions.
///
/// Returns an error if the original gate, rink, teacher footing or win script has changed.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let room = pack.data.maps.get_mut(GYM).context("Harbor Gym")?;
    ensure!(
        room.attributes.tileset_name == "mn_gym_8" && room.attributes.width == 7,
        "unexpected Harbor rink"
    );
    let bank = &pack
        .data
        .tilesets
        .get("mn_gym_8")
        .context("rink art")?
        .collision;
    for (id, expected) in [
        ("01", ["FLOOR"; 4]),
        ("04", ["ICE"; 4]),
        ("05", ["WALL"; 4]),
        ("07", ["FLOOR", "ICE", "ICE", "ICE"]),
        ("08", ["ICE", "FLOOR", "ICE", "ICE"]),
    ] {
        ensure!(
            bank.get(id).is_some_and(|v| v.as_slice() == expected),
            "Harbor rink block {id}"
        );
    }
    for (x, y, original, _) in LANE {
        ensure!(
            room.blocks.get(usize::from(y * 7 + x)) == Some(&original),
            "unexpected west lane {x},{y}"
        );
    }
    ensure!(
        room.scripts.get("MnDuluthGymV2Gate1")
            == Some(&json!([
                command("checkevent", &[WIN]),
                command("iffalse", &[CLOSED]),
                command("sjump", &[OPEN])
            ])),
        "west gate no longer follows Norek's victory"
    );
    for (label, won, set) in [(CLOSED, false, "clearevent"), (OPEN, true, "setevent")] {
        let body = room.scripts.get_mut(label).context("west gate callback")?;
        let previous = if won { "1" } else { "5" };
        ensure!(
            *body
                == json!([
                    command("changeblock", &["2", "8", previous]),
                    command(set, &[WEST]),
                    command("end", &[])
                ])
                && pack.data.map_scripts.get(label) == Some(body),
            "unexpected west gate branch {label}"
        );
        let mut rows = Vec::with_capacity(LANE.len() + 2);
        for (x, y, before, after) in LANE {
            rows.push(command(
                "changeblock",
                &[
                    &(x * 2).to_string(),
                    &(y * 2).to_string(),
                    &(if won { after } else { before }).to_string(),
                ],
            ));
        }
        rows.push(command(set, &[WEST]));
        rows.push(command("end", &[]));
        *body = rows.into();
        pack.data.map_scripts.insert(label.into(), body.clone());
    }
    let original = room.scripts.get(VICTORY).context("Norek's victory text")?;
    ensure!(
        *original
            == json!([
                command("text", &["\"A clean finish.\""]),
                command("line", &["\"You earned it.\""]),
                command("done", &[])
            ])
            && pack.data.map_scripts.get(VICTORY) == Some(original),
        "unexpected victory directions"
    );
    authored_dialogue::install_canonical(room, VICTORY)?;
    pack.data
        .map_scripts
        .insert(VICTORY.into(), room.scripts[VICTORY].clone());
    room.script_text_bodies = crate::parse_script_text_bodies(GYM, &room.scripts)?;
    room.script_block_changes = crate::parse_script_block_changes(GYM, &room.scripts)?;
    room.script_control_commands = crate::parse_script_control_commands(GYM, &room.scripts)?;
    room.script_flag_commands = crate::parse_script_flag_commands(GYM, &room.scripts)?;
    room.script_runtime_commands = crate::parse_script_runtime_commands(GYM, &room.scripts)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, Result, ensure};
    use crystal_core::world::map::TilePosition;
    use std::path::PathBuf;

    use super::{GYM, LANE, WIN};
    use crate::grand_marais_modpack::authoring::command;

    #[test]
    #[ignore = "requires the local MN_RETURN_VISITS_PACK archive"]
    fn harbor_exit_runs_on_reward_reentry_and_continue() -> Result<()> {
        let path = PathBuf::from(
            std::env::var_os("MN_RETURN_VISITS_PACK").context("return visits archive")?,
        );
        let pack = crate::read_verified_compiled_game_pack(path)?;
        let data = pack.data();
        let music = pack.audio_manifest.music.keys().cloned().collect();
        let room = data.maps.get(GYM).context("Harbor Gym map")?;
        let refresh = "MnDuluthGymRefreshGates";
        let leader = room.scripts["MnDuluthGymLeader"]
            .as_array()
            .context("leader battle")?;
        ensure!(
            leader.windows(3).any(|rows| rows
                == [
                    command("setevent", &[WIN]),
                    command("scall", &[refresh]),
                    command("sjump", &["MnDuluthGymLeaderReward"])
                ]),
            "victory must open lane before a full TM pocket can defer the gift"
        );
        let body = room.scripts[refresh].as_array().context("gate refresh")?;
        let (mut state, mut session) =
            data.start_overworld_session_at_runtime_tile(GYM, TilePosition::new(7, 3), &music)?;
        for flag in [
            "EVENT_MN_GYM_8_TRAINER_1_DEFEATED",
            "EVENT_MN_GYM_8_TRAINER_2_DEFEATED",
        ] {
            state
                .flags
                .set_event_flag(flag, true)
                .map_err(|e| anyhow::anyhow!("set sight victory: {e:?}"))?;
        }
        let check = |state: &crystal_core::state::GameState,
                     session: &crystal_core::world::session::OverworldSession,
                     won: bool|
         -> Result<()> {
            let overrides = state
                .map_block_overrides
                .get(GYM)
                .context("saved west lane")?;
            for (x, y, original, uncovered) in LANE {
                let block = if won { uncovered } else { original };
                ensure!(
                    session
                        .map
                        .metatile_at(i16::try_from(x)?, i16::try_from(y)?)
                        == Some(block)
                        && overrides.get(&(x, y)) == Some(&block),
                    "wrong saved or visible tile {x},{y}"
                );
            }
            Ok(())
        };
        for won in [false, true] {
            state
                .flags
                .set_event_flag(WIN, won)
                .map_err(|e| anyhow::anyhow!("set victory: {e:?}"))?;
            data.execute_map_callback_script(&mut state, &mut session, GYM, refresh, body)?;
            check(&state, &session, won)?;
            for setup in ["MAPSETUP_RELOADMAP", "MAPSETUP_WARP", "MAPSETUP_CONTINUE"] {
                let overrides = state.map_block_overrides.entry(GYM.into()).or_default();
                for (x, y, old, new) in LANE {
                    overrides.insert((x, y), if won { old } else { new });
                }
                if setup == "MAPSETUP_CONTINUE" {
                    // Continue first restores the persisted cells, then runs the room's TILES callback.
                    (state, session) = data.resume_overworld_session_from_state(state, &music)?;
                } else {
                    data.apply_map_setup_callbacks(&mut state, &mut session, GYM, setup)?;
                }
                check(&state, &session, won)?;
            }
        }
        Ok(())
    }
}
