//! Restore the two return lanes promised by the Ely League's registration desk.
use anyhow::{Context, Result, ensure};

use crate::{CompiledGamePack, authored_town_publication};

const RUGS: &str = "data/tilesets/champions_room_metatiles.bin";
const SOUTH: [&str; 4] = ["FLOOR", "FLOOR", "WARP_CARPET_DOWN", "WARP_CARPET_DOWN"];

/// Fits south-facing exits to the existing return warps in Veil's and Casey's rooms.
///
/// Returns an error if the room, native rug, or its return lanes differ from the expected layout.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let rugs = pack.runtime_files.get_mut(RUGS).context(RUGS)?;
    let bank = &mut pack
        .data
        .tilesets
        .get_mut("champions_room")
        .context("Champion's room")?
        .collision;
    let exit = bank.get("08").context("stock doorway")?.clone();
    ensure!(
        rugs.len() == 64 * 16
            && exit.iter().map(String::as_str).eq(SOUTH)
            && bank
                .get("31")
                .is_some_and(|v| v.iter().all(|q| q == "FLOOR"))
            && !bank.contains_key("40"),
        "unexpected Champion's room runner"
    );
    // The long red runner keeps its border all the way to the thin native
    // threshold. The upper half remains ordinary walkable arrival ground.
    let mut rug = [0; 16];
    rug[..8].copy_from_slice(&rugs[0x31 * 16..0x31 * 16 + 8]);
    rug[8..].copy_from_slice(&rugs[0x08 * 16 + 8..0x09 * 16]);
    rugs.extend(rug);
    bank.insert("40".into(), exit);
    for (name, tileset, bottom, before, after) in [
        ("MnLeagueRoom3", "elite_four_room", 17, 0x28, 0x1c),
        ("MnLeagueChampion", "champions_room", 23, 0x31, 0x40),
    ] {
        let mut room = pack.data.maps.get(name).context(name)?.clone();
        let roles = pack
            .data
            .tilesets
            .get(tileset)
            .and_then(|bank| bank.collision.get(&format!("{after:02x}")))
            .with_context(|| format!("missing {name} exit carpet"))?;
        ensure!(
            room.attributes.tileset_name == tileset
                && room.attributes.width == 5
                && i32::from(room.attributes.height) * 2 == bottom + 1
                && roles.iter().map(String::as_str).eq(SOUTH)
                && room
                    .events
                    .warps
                    .iter()
                    .take(2)
                    .map(|w| (i32::from(w.x), i32::from(w.y), w.target_warp_id))
                    .eq([(4, bottom, 7), (5, bottom, 8)]),
            "unexpected south warp positions in {name}"
        );
        let at = room
            .blocks
            .len()
            .checked_sub(3)
            .context("empty League room")?;
        ensure!(
            room.blocks[at] == before,
            "unexpected southern rug in {name}"
        );
        room.blocks[at] = after;
        authored_town_publication::replace(pack, room)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use anyhow::{Context, Result, ensure};
    use crystal_core::input::GameButton;
    use crystal_core::models::Dv;
    use crystal_core::random::ReplayDivider;
    use crystal_core::state::GameState;
    use crystal_core::world::map::{Direction, TilePosition};

    #[test]
    #[ignore = "requires both League carpets in MN_LEAGUE_JOURNEY_PACK"]
    fn league_journey_each_return_lane_walks_out_without_replaying_a_battle() -> Result<()> {
        let path = std::env::var_os("MN_LEAGUE_JOURNEY_PACK").context("MN_LEAGUE_JOURNEY_PACK")?;
        let pack = crate::read_verified_compiled_game_pack(path)?;
        let data = pack.data();
        let music = pack
            .audio_manifest()?
            .music
            .into_keys()
            .collect::<BTreeSet<_>>();
        for (room, bottom, destination, wins) in [
            ("MnLeagueRoom3", 17, "MnLeagueRoom2", "2"),
            ("MnLeagueChampion", 23, "MnLeagueRoom4", "4"),
        ] {
            for x in [4, 5] {
                let mut state = GameState::reset_wram_for_new_game();
                state
                    .flags
                    .set_event_flag("EVENT_MN_LEAGUE_RUN_ACTIVE", true)?;
                state
                    .script_runtime
                    .memory
                    .insert("wMnLeagueRun".into(), wins.into());
                let mut session =
                    data.overworld_session(room, TilePosition::new(x, bottom - 1), 0)?;
                session.player.facing = Direction::Down;
                let mut divider = ReplayDivider::new([0; 64]);
                let first = data.apply_overworld_input(
                    &mut state,
                    &mut session,
                    [GameButton::Down],
                    &music,
                    &mut divider,
                )?;
                ensure!(
                    session.player.tile == TilePosition::new(x, bottom) && first.warp.is_none(),
                    "{room} lane {x} left before the player crossed the threshold"
                );
                let leave = data.apply_overworld_input(
                    &mut state,
                    &mut session,
                    [GameButton::Down],
                    &music,
                    &mut divider,
                )?;
                ensure!(
                    leave.warp.is_some()
                        && session.map.name == destination
                        && session.player.tile == TilePosition::new(x, 4)
                        && state
                            .flags
                            .is_event_flag_set("EVENT_MN_LEAGUE_RUN_ACTIVE")?
                        && state.script_runtime.memory["wMnLeagueRun"] == wins,
                    "{room} lane {x} lost the return location or existing wins"
                );
            }
        }
        Ok(())
    }

    #[test]
    #[ignore = "requires the Ely register in MN_LEAGUE_JOURNEY_PACK"]
    fn league_journey_register_records_party_and_preserves_training_for_the_return() -> Result<()> {
        const HALL: &str = "MnLeagueHallOfFame";
        let path = std::env::var_os("MN_LEAGUE_JOURNEY_PACK").context("MN_LEAGUE_JOURNEY_PACK")?;
        let pack = crate::read_verified_compiled_game_pack(path)?;
        let data = pack.data();
        let mut room = data.overworld_session(HALL, TilePosition::new(6, 5), 0)?;
        let mut state = GameState::reset_wram_for_new_game();
        for (species, level) in [("PIKELING", 48), ("SODCALF", 52)] {
            let mut partner = data.create_pokemon(species, level, Dv::default())?;
            partner.hp = 7;
            partner.moves[0].current_pp = 1;
            ensure!(
                state.storage.party.add_pokemon(partner),
                "League team is full"
            );
        }
        state.sync_party_from_storage();
        let party = state.storage.party.clone();
        let hall = data.maps.get(HALL).context(HALL)?;
        let record = hall
            .script_runtime_commands
            .iter()
            .find(|record| {
                record.command == "halloffame" && record.source_script == "MnElyRecordVictory"
            })
            .context("record team in Ely")?;
        data.apply_script_runtime_command_in_session(
            &mut state,
            &mut room,
            HALL,
            &record.source_script,
            record.command_index,
            crate::ScriptRuntimeInputs::default(),
        )?;
        let entry = state
            .hall_of_fame
            .entries
            .first()
            .context("registered Ely team")?;
        let marker = u16::try_from(data.story_event_script_constants.global["SPAWN_LANCE"])?;
        ensure!(
            state.script_runtime.hall_of_fame_requested
                && state.hall_of_fame.spawn_after_champion == Some(marker)
                && state.hall_of_fame.count == 1
                && entry.win_count == 1
                && entry
                    .team
                    .iter()
                    .flatten()
                    .map(|p| (p.species.as_str(), p.level))
                    .eq([("PIKELING", 48), ("SODCALF", 52)])
                && state.storage.party == party,
            "registering the team changed its HP/PP, reordered the roster, or failed to request the Hall of Fame"
        );
        Ok(())
    }
}
