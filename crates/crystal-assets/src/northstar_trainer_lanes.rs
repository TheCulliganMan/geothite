//! Give both ordinary ICE warehouse guards the same field encounter as the other patrols.
use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::grand_marais_modpack::authoring::command;
use crate::{CompiledGamePack, authored_town_publication};

/// Rebinds the unconditionally hostile guards in the cargo basement and relay floor.
///
/// Returns an error if either guard has gained a quest prerequisite, lost its existing
/// battle or victory flag, or moved from the inspected corridor.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    for (name, id, position, facing) in [
        ("MnIceCargoBasement", "MN_ICE_CARGO_GUARD", [12, 12], "LEFT"),
        ("MnIceRelayFloor", "MN_ICE_RELAY_GUARD", [16, 11], "DOWN"),
    ] {
        let mut room = pack.data.maps.get(name).context("ICE guard room")?.clone();
        let flag = format!("EVENT_{id}_DEFEATED");
        let seen = format!("{id}SeenText");
        let won = format!("{id}WonText");
        let lost = format!("{id}LostText");
        let after = format!("{id}After");
        let old = json!([
            command("faceplayer", &[]),
            command("checkevent", &[&flag]),
            command("iftrue", &[&after]),
            command("opentext", &[]),
            command("writetext", &[&seen]),
            command("promptbutton", &[]),
            command("closetext", &[]),
            command("winlosstext", &[&won, &lost]),
            command("loadtrainer", &["GRUNTM", id]),
            command("startbattle", &[]),
            command("reloadmapafterbattle", &[]),
            command("setevent", &[&flag]),
            command("sjump", &[&after])
        ]);
        ensure!(
            room.scripts.get(id) == Some(&old)
                && pack.data.map_scripts.get(id) == Some(&old)
                && pack
                    .data
                    .story_event_script_constants
                    .global
                    .contains_key(&flag)
                && room.scripts.contains_key(&after)
                && !room.trainer_scripts.contains_key(id),
            "unexpected guard program {id}"
        );
        let actor = room
            .objects
            .iter_mut()
            .find(|o| o.script == id)
            .context("warehouse guard")?;
        ensure!(
            [actor.x, actor.y] == position
                && actor.object_type == "OBJECTTYPE_SCRIPT"
                && actor.radius == 0
                && actor.spritemovedata == "SPRITEMOVEDATA_STANDING_DOWN"
                && actor.sightline_direction_override.is_none(),
            "unexpected guard footing {id}"
        );
        let wire = room
            .map_event_section_commands
            .iter_mut()
            .find(|r| r.command == "object_event" && r.args.get(11).is_some_and(|s| s == id))
            .context("warehouse guard object row")?;
        ensure!(
            wire.args[3] == actor.spritemovedata
                && wire.args[9] == actor.object_type
                && wire.args[10] == "0",
            "unexpected guard object wire {id}"
        );
        actor.spritemovedata = format!("SPRITEMOVEDATA_STANDING_{facing}");
        actor.sightline_direction_override = Some(facing.into());
        actor.object_type = "OBJECTTYPE_TRAINER".into();
        actor.radius = 3;
        wire.args[3].clone_from(&actor.spritemovedata);
        wire.args[9].clone_from(&actor.object_type);
        wire.args[10] = actor.radius.to_string();
        room.scripts.insert(
            id.into(),
            json!([command(
                "trainer",
                &["GRUNTM", id, &flag, &seen, &won, &lost, &after]
            )]),
        );
        let scripts = &room.scripts;
        room.trainer_scripts = crate::parse_trainer_scripts(name, scripts)?;
        room.scripted_trainer_battles = crate::parse_scripted_trainer_battles(name, scripts)?;
        room.script_flag_commands = crate::parse_script_flag_commands(name, scripts)?;
        room.script_control_commands = crate::parse_script_control_commands(name, scripts)?;
        room.script_text_commands = crate::parse_script_text_commands(name, scripts)?;
        room.script_object_commands = crate::parse_script_object_commands(name, scripts)?;
        room.script_runtime_commands = crate::parse_script_runtime_commands(name, scripts)?;
        // Native trainer resolution returns through its callback. Remove this old
        // program's scripted reload while preserving the other map transitions.
        room.script_map_commands
            .retain(|row| row.source_script != id);
        authored_town_publication::replace(pack, room)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, Result, ensure};
    use crystal_core::world::map::TilePosition;

    /// Exercise the post-step eligibility and native object roster on both real aisles.
    #[test]
    #[ignore = "requires the compiled edition in MN_TRAIL_BATTLES_PACK"]
    fn trail_battles_both_ice_guards_notice_the_player_once() -> Result<()> {
        let path = std::env::var_os("MN_TRAIL_BATTLES_PACK").context("MN_TRAIL_BATTLES_PACK")?;
        let pack = crate::read_verified_compiled_game_pack(path)?;
        let data = pack.data();
        let music = pack.audio_manifest.music.keys().cloned().collect();
        for (room, id, start, step) in [
            (
                "MnIceCargoBasement",
                "MN_ICE_CARGO_GUARD",
                (12, 12),
                (-1, 0),
            ),
            ("MnIceRelayFloor", "MN_ICE_RELAY_GUARD", (16, 11), (0, 1)),
        ] {
            let flag = format!("EVENT_{id}_DEFEATED");
            for distance in 1..=3 {
                let square =
                    TilePosition::new(start.0 + distance * step.0, start.1 + distance * step.1);
                let (mut state, session) =
                    data.start_overworld_session_at_runtime_tile(room, square, &music)?;
                ensure!(
                    data.check_trainer_sight_after_step(&state, &session)?
                        .is_some_and(|sight| sight.script == id),
                    "{room}: unseen at {square:?}"
                );
                state.flags.set_event_flag(&flag, true)?;
                ensure!(
                    data.check_trainer_sight_after_step(&state, &session)?
                        .is_none_or(|sight| sight.script != id),
                    "{room}: saved victory challenged again"
                );
            }
        }
        Ok(())
    }
}
