//! Restore the party-menu approach to Castorock Gallery's movable boulder.
use anyhow::{Context, Result, ensure};
use crystal_core::models::pokemon::LearnedMove;
use crystal_core::state::GameState;
use crystal_core::systems::script_control::ScriptControlAction;
use crystal_core::systems::script_runtime::ScriptRuntimeInputs;
use crystal_core::world::map::TilePosition;
use serde_json::{Value, json};

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring::command;

const MENU: &str = "Script_StrengthFromMenu";
const USED: &str = "Script_UsedStrength";
const ROOM: &str = "MnR03Cave1F";
const FLAG: &str = "ENGINE_STRENGTH_ACTIVE";

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let events = pack
        .data
        .story_events
        .iter_mut()
        .find_map(|catalog| catalog.get_mut("OverworldEvents"))
        .context("field move event catalog")?;
    let body = events
        .get_mut(MENU)
        .and_then(Value::as_array_mut)
        .context("native Strength menu entry")?;
    ensure!(
        *body
            == [
                command("refreshmap", &[]),
                command("special", &["UpdateTimePals"])
            ],
        "unexpected native Strength menu prologue"
    );
    // The two labels are adjacent in Crystal's engine/events/overworld.asm.
    // Each exported label has its own program, so retain the fallthrough explicitly.
    body.push(command("sjump", &[USED]));
    let roots = pack
        .data
        .story_events
        .iter_mut()
        .find_map(|catalog| catalog.get_mut("StandardScripts"))
        .and_then(|catalog| catalog.get_mut("GlobalScriptRoots"))
        .and_then(Value::as_array_mut)
        .context("native script roots")?;
    ensure!(
        !roots.contains(&json!(MENU)),
        "Strength menu already registered"
    );
    roots.push(json!(MENU));
    pack.data.materialize_global_scripts()?;
    verify_strength(pack)
}

fn verify_strength(pack: &CompiledGamePack) -> Result<()> {
    let data = &pack.data;
    let mut state = GameState::reset_wram_for_new_game();
    let mut session = data.overworld_session(ROOM, TilePosition::new(7, 8), 0)?;
    let mut partner = data.create_pokemon("BRACLAW", 16, Default::default())?;
    partner.moves = vec![LearnedMove {
        name: "STRENGTH".into(),
        current_pp: data.moves["STRENGTH"].pp,
        pp_ups: 0,
    }];
    state.storage.party.pokemon[0] = Some(partner);
    ensure!(
        data.queue_strength_from_menu(&mut state, &mut session, 0)
            .is_err(),
        "unearned Strength badge accepted"
    );
    state.badges.johto[2] = true;
    data.queue_strength_from_menu(&mut state, &mut session, 0)?;
    let queued = state
        .script_runtime
        .next_script
        .as_ref()
        .context("Strength menu did not dispatch")?;
    ensure!(
        queued.script == MENU && !state.flags.is_engine_flag_set(FLAG)?,
        "Strength activated before the script"
    );
    let action =
        data.apply_script_control_command_in_session(&mut state, &session, ROOM, MENU, 2)?;
    let ScriptControlAction::Jump { target_script, .. } = action else {
        anyhow::bail!("Strength prologue lost its fallthrough");
    };
    ensure!(
        target_script == USED,
        "Strength menu dispatched to the wrong program"
    );
    data.apply_script_runtime_command_in_session(
        &mut state,
        &mut session,
        ROOM,
        &target_script,
        0,
        ScriptRuntimeInputs::default(),
    )?;
    ensure!(
        state.flags.is_engine_flag_set(FLAG)?,
        "queued native Strength did not enable boulder movement"
    );
    Ok(())
}
