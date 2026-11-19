//! Keep the Grand Rapids meter receipt independent of Finn's drill-room sprite.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::authored_town_publication;
use crate::grand_marais_modpack::authoring::command;
use crate::{CompiledGamePack, MapModule};

const CORE: &str = "MnIronDrillCore";
const FINN: &str = "MN_NDRILL_FINN";
const DELIVERED: &str = "EVENT_MN_DEPOT_CLEARED";

fn approach_crew(room: &mut MapModule) -> Result<()> {
    let route = "MnMineWalkToCrew";
    let front = "MnMineFromMotorFront";
    let detour = "MnMineRoundMotor";
    let walk = command("applymovement", &["PLAYER", "MnNDrillPlayerToCrew"]);
    let entry = room
        .scripts
        .get_mut("MnNDrillStop")
        .and_then(Value::as_array_mut)
        .and_then(|steps| steps.iter_mut().find(|step| **step == walk))
        .context("shutdown walk to the crew")?;
    *entry = command("scall", &[route]);
    // The switch can also be operated from its west at (15,2). Reach the
    // normal front approach at (16,3) before taking the aisle past the drill.
    for (label, body) in [
        (
            route,
            json!([
                command("readvar", &["VAR_XCOORD"]),
                command("ifnotequal", &["15", front]),
                command("applymovement", &["PLAYER", detour]),
                command("sjump", &[front]),
            ]),
        ),
        (front, json!([walk, command("end", &[])])),
        (
            detour,
            json!([
                command("step", &["DOWN"]),
                command("step", &["RIGHT"]),
                command("step_end", &[]),
            ]),
        ),
    ] {
        ensure!(
            room.scripts.insert(label.into(), body).is_none(),
            "occupied mine movement {label}"
        );
    }
    Ok(())
}

/// Restores Finn after a shutdown until the technician accepts his meter.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack
        .data
        .maps
        .get(CORE)
        .context("Iron drill chamber")?
        .clone();
    let finn = room
        .objects
        .iter_mut()
        .find(|actor| actor.object_identifier.as_deref() == Some(FINN))
        .context("drill-room crew member")?;
    ensure!(
        finn.event_flag == DELIVERED,
        "unexpected Finn visibility flag"
    );
    // Native appear/disappear writes to a bound event flag. The callback only
    // needs to hide the local actor, including when reentering a finished mine.
    finn.event_flag = "-1".into();
    let row = room
        .map_event_section_commands
        .iter_mut()
        .find(|row| row.command == "object_event" && row.args.get(11).is_some_and(|id| id == FINN))
        .context("Finn's native event")?;
    let flag = row
        .args
        .get_mut(12)
        .context("Finn's native visibility flag")?;
    ensure!(flag == DELIVERED, "unexpected native Finn visibility flag");
    *flag = "-1".into();

    let callback = room
        .scripts
        .get_mut("MnNDrillCoreActors")
        .and_then(Value::as_array_mut)
        .context("drill chamber entry callback")?;
    ensure!(
        callback.first() == Some(&command("disappear", &[FINN]))
            && callback.get(1) == Some(&command("checkevent", &["EVENT_MN_DRILL_STOPPED"])),
        "unexpected drill crew restoration"
    );
    callback.splice(
        1..1,
        [
            command("checkevent", &[DELIVERED]),
            command("iftrue", &["MnNDrillCoreActorsEnd"]),
        ],
    );
    approach_crew(&mut room)?;
    room.script_flag_commands = crate::parse_script_flag_commands(CORE, &room.scripts)?;
    room.script_control_commands = crate::parse_script_control_commands(CORE, &room.scripts)?;
    room.script_object_commands = crate::parse_script_object_commands(CORE, &room.scripts)?;
    room.script_variable_commands = crate::parse_script_variable_commands(CORE, &room.scripts)?;
    room.script_movements =
        crate::parse_script_movements(CORE, &room.scripts, &room.script_object_commands)?;
    authored_town_publication::replace(pack, room)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crystal_core::state::GameState;
    use crystal_core::world::map::TilePosition;

    use super::{CORE, DELIVERED, FINN};

    #[test]
    #[ignore = "requires the built Minnesota pack in MN_FLOODWATCH_PACK"]
    fn floodwatch_drill_room_entry_preserves_the_delivery_receipt() {
        let path = PathBuf::from(std::env::var_os("MN_FLOODWATCH_PACK").expect("built pack"));
        let pack = crate::read_verified_compiled_game_pack(&path).expect("verified Floodwatch");
        for stopped in [false, true] {
            for delivered in [false, true] {
                let mut state = GameState::reset_wram_for_new_game();
                state
                    .flags
                    .set_event_flag("EVENT_MN_DRILL_STOPPED", stopped)
                    .expect("motor flag");
                state
                    .flags
                    .set_event_flag(DELIVERED, delivered)
                    .expect("delivery flag");
                for _ in 0..2 {
                    let mut session = pack
                        .data
                        .overworld_session(CORE, TilePosition::new(8, 9), 0)
                        .expect("native drill room");
                    pack.data
                        .apply_map_setup_callbacks(&mut state, &mut session, CORE, "MAPSETUP_WARP")
                        .expect("run actual mine entry callbacks");
                    let crew = session
                        .objects
                        .iter()
                        .find(|o| o.object_identifier.as_deref() == Some(FINN))
                        .expect("Finn's native actor");
                    assert_eq!(session.is_object_visible(crew), stopped && !delivered);
                    assert_eq!(
                        state
                            .flags
                            .is_event_flag_set(DELIVERED)
                            .expect("delivery receipt"),
                        delivered
                    );
                }
            }
        }
    }
}
