//! Put local trainer stories on a voluntary second conversation and let the
//! Grand Rapids river watcher remember when the drill went quiet.

use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring::command;

/// Runs after the reviewed text is installed and before the rooms are indexed.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut revised = 0;
    for room in pack
        .data
        .maps
        .values_mut()
        .filter(|room| room.id.starts_with("Mn"))
    {
        for actor in &room.objects {
            let Some(battle) = room.trainer_scripts.get(&actor.script) else {
                continue;
            };
            let return_text = format!("{}ReturnText", actor.script);
            if !room.scripts.contains_key(&return_text) {
                continue;
            }
            let callback = room.scripts.get_mut(&battle.callback).with_context(|| {
                format!("missing return script for {}/{}", room.id, actor.script)
            })?;
            ensure!(
                *callback
                    == json!([
                        command("endifjustbattled", &[]),
                        command("jumptextfaceplayer", &[&battle.win_text]),
                    ]),
                "expected the original victory speech on return to {}/{}",
                room.id,
                actor.script
            );
            *callback = json!([
                command("endifjustbattled", &[]),
                command("jumptextfaceplayer", &[&return_text]),
            ]);
            revised += 1;
        }
    }
    ensure!(
        revised == 276,
        "expected 276 authored return conversations, got {revised}"
    );

    let river = pack
        .data
        .maps
        .get_mut("MnGrandRapids")
        .context("Grand Rapids")?;
    let watcher = "MN_GR_RIVER_WATCHER";
    let quiet = "MnGRWatcherQuiet";
    let quiet_text = "MnGRWatcherQuietText";
    let first = command("jumptextfaceplayer", &["MN_GR_RIVER_WATCHERText"]);
    ensure!(
        river.scripts.get(watcher) == Some(&json!([&first]))
            && !river.scripts.contains_key(quiet)
            && river.scripts.contains_key(quiet_text),
        "expected the Grand Rapids bank witness and both conversations"
    );
    river.scripts.insert(
        watcher.to_owned(),
        json!([
            command("checkevent", &["EVENT_MN_DRILL_STOPPED"]),
            command("iftrue", &[quiet]),
            first,
        ]),
    );
    river.scripts.insert(
        quiet.to_owned(),
        json!([command("jumptextfaceplayer", &[quiet_text])]),
    );
    Ok(())
}
