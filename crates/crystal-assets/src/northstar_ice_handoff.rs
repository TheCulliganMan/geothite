//! Stage both dispatch-office entrances around the actors already in the scene.

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;
use crate::authored_dialogue;
use crate::authored_town_publication;
use crate::grand_marais_modpack::authoring::command;

const OFFICE: &str = "MnMinneapolisDispatchOffice";
const RIGHT_WALK: &str = "MnNICEPlayerWatchesRight";

/// Routes the right entrance around Reed, and gives directions to Ada and back.
///
/// Returns an error when the expected office, movement, or compiled program is missing.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack
        .data
        .maps
        .get(OFFICE)
        .context("Minneapolis dispatch office")?
        .clone();
    let movement = room
        .scripts
        .get_mut(RIGHT_WALK)
        .and_then(Value::as_array_mut)
        .context("right dispatch-office entrance walk")?;
    let up = command("step", &["UP"]);
    ensure!(
        movement.len() == 6
            && movement[..5].iter().all(|row| row == &up)
            && movement[5] == command("step_end", &[]),
        "unexpected right entrance movement"
    );
    // Both triggers share the same final camera. Cross to the open lane by the
    // desk, move north past Reed, then step back in front of him.
    movement.insert(2, command("step", &["LEFT"]));
    movement.insert(6, command("step", &["RIGHT"]));
    room.script_movements =
        crate::parse_script_movements(OFFICE, &room.scripts, &room.script_object_commands)?;
    authored_town_publication::replace(pack, room)?;
    for (name, labels) in [
        ("MnIceOperations", &["MnIceOperationsWorkerText"][..]),
        (
            "MnIceRecords",
            &["MnIceHQCardGivenText", "MnIceHQCardAfterText"][..],
        ),
    ] {
        let mut room = pack
            .data
            .maps
            .get(name)
            .with_context(|| format!("ICE floor {name}"))?
            .clone();
        for &label in labels {
            ensure!(
                room.script_text_bodies.contains_key(label),
                "missing ICE handoff {label}"
            );
            authored_dialogue::install_canonical(&mut room, label)?;
        }
        room.script_text_bodies = crate::parse_script_text_bodies(name, &room.scripts)?;
        authored_town_publication::replace(pack, room)?;
    }
    Ok(())
}
