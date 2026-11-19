//! Exercise Ely's installed callbacks and rematch choice without a game session.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_core::models::Dv;
use crystal_core::state::GameState;
use crystal_core::systems::script_control::ScriptControlAction;
use crystal_core::world::map::TilePosition;

const ACTIVE: &str = "EVENT_MN_LEAGUE_RUN_ACTIVE";
const TITLE: &str = "EVENT_MN_LEAGUE_CHAMPION";
const RESTORED: &str = "EVENT_MN_SURVEY_RESTORED";
const RUN: &str = "wMnLeagueRun";

#[test]
#[ignore = "requires the selected Minnesota edition in MN_LAKE_OBSERVATIONS_PACK"]
fn lake_observations_five_league_doors_preserve_resources_and_allow_a_rematch_refusal() -> Result<()>
{
    let path = PathBuf::from(std::env::var_os("MN_LAKE_OBSERVATIONS_PACK").context("built pack")?);
    let pack = crate::read_verified_compiled_game_pack(path)?;
    let data = pack.data();
    let mut state = GameState::reset_wram_for_new_game();
    state.flags.set_event_flag(RESTORED, true)?;
    state.flags.set_event_flag(TITLE, true)?;
    state.money = 12_345;
    let mut partner = data.create_pokemon("PIKELING", 49, Dv::default())?;
    partner.hp = 17;
    partner.moves[0].current_pp = 1;
    state.storage.party.pokemon[0] = Some(partner);
    let medicine = data.items.get("HYPER_POTION").context("Hyper Potion")?;
    ensure!(
        state
            .bag
            .add_item(medicine, 3)
            .map_err(anyhow::Error::msg)?,
        "put three potions in the bag"
    );
    let supplies = (state.storage.clone(), state.bag.clone(), state.money);
    let rooms = [
        "MnLeagueRoom1",
        "MnLeagueRoom2",
        "MnLeagueRoom3",
        "MnLeagueRoom4",
        "MnLeagueChampion",
    ];
    for (i, room) in rooms.into_iter().enumerate() {
        let champion = i == 4;
        let (y, closed, open) = if champion {
            (0, 0x0a, 0x0b)
        } else {
            (1, 0x22, 0x16)
        };
        for (admitted, wins, expected) in [
            (false, 5, closed),
            (true, i, closed),
            (true, i + 1, open),
            (true, 5, open),
        ] {
            state.flags.set_event_flag(ACTIVE, admitted)?;
            state
                .script_runtime
                .memory
                .insert(RUN.into(), wins.to_string());
            for setup in ["MAPSETUP_WARP", "MAPSETUP_CONTINUE"] {
                let mut session = data.overworld_session(room, TilePosition::new(4, 12), 0)?;
                data.apply_map_setup_callbacks(&mut state, &mut session, room, setup)?;
                ensure!(
                    session.map.metatile_at(2, y) == Some(expected),
                    "{room} door in {setup}, active={admitted} wins={wins}"
                );
                ensure!(
                    state.storage == supplies.0
                        && state.bag == supplies.1
                        && state.money == supplies.2,
                    "{room} changed the party or resources"
                );
            }
        }
    }
    for yes in [false, true] {
        state.script_runtime.script_value = Some(u8::from(yes).to_string());
        let response = data.apply_script_control_command(
            &mut state,
            rooms[4],
            rooms[4],
            "MnLeagueChampionRematchIntro",
            3,
        )?;
        if yes {
            ensure!(
                matches!(response, ScriptControlAction::Continue { .. }),
                "confirmed rematch"
            );
        } else {
            ensure!(
                matches!(response, ScriptControlAction::Jump { target_script, .. } if target_script == "MnCaseyWaits"),
                "declined rematch"
            );
        }
    }
    // These are the actual clinic and lobby entry callbacks used following a
    // blackout or a voluntary exit. The battle/credit sequences run elsewhere.
    for destination in ["MnElyPokecenter1F", "MnLeagueLobby"] {
        state.flags.set_event_flag(ACTIVE, true)?;
        state.script_runtime.memory.insert(RUN.into(), "4".into());
        let mut session = data.overworld_session(destination, TilePosition::new(3, 4), 0)?;
        data.apply_map_setup_callbacks(&mut state, &mut session, destination, "MAPSETUP_WARP")?;
        ensure!(
            !state.flags.is_event_flag_set(ACTIVE)? && state.script_runtime.memory[RUN] == "0",
            "late run stays open in {destination}"
        );
        ensure!(
            state.flags.is_event_flag_set(TITLE)? && state.flags.is_event_flag_set(RESTORED)?,
            "{destination} erases prior records"
        );
    }
    Ok(())
}
