//! Ordinary club conversations after the trainer's immediate battle response.

use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::CompiledGamePack;
use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring::command;

pub(crate) fn install(
    pack: &mut CompiledGamePack,
    texts: &BTreeMap<String, Vec<AuthoredDialoguePage>>,
) -> Result<()> {
    let mut room = pack
        .data
        .maps
        .get("MnMinneapolisBattleClub")
        .context("Minneapolis battle club")?
        .clone();
    let mut changed = false;
    for trainer in [
        "MN_MP_CLUB_RIVET",
        "MN_MP_CLUB_BANKS",
        "MN_MP_CLUB_CEDAR",
        "MN_MP_CLUB_COACH_INEZ",
    ] {
        let label = format!("{trainer}Text");
        let Some(pages) = texts.get(&label) else {
            continue;
        };
        let after = format!("{trainer}After");
        let won = format!("{trainer}Won");
        let original = json!([
            command("endifjustbattled", &[]),
            command("jumptextfaceplayer", &[&won]),
        ]);
        let revised = json!([
            command("endifjustbattled", &[]),
            command("jumptextfaceplayer", &[&label]),
        ]);
        let installed = room
            .scripts
            .get(&after)
            .context("club return interaction")?;
        ensure!(
            installed == &original || installed == &revised,
            "changed club return interaction: {after}"
        );
        // Retain the trainer macro, battle result pages and just-battled fence.
        // The personal conversation belongs to a later voluntary interaction.
        room.scripts.insert(after, revised);
        room.scripts
            .insert(label, authored_dialogue::compile(pages)?);
        changed = true;
    }
    if changed {
        crate::northstar_story_revision::publish(pack, room)?;
    }
    Ok(())
}
