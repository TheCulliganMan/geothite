//! The Ely registrar explains which retreat ends a League challenge.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};

use crate::grand_marais_modpack::authoring::command;
use crate::{CompiledGamePack, authored_dialogue, authored_town_publication};

const LOBBY: &str = "MnLeagueLobby";
const LABEL: &str = "MnElyRegistrationText";

/// Installs the directions before the registrar's existing yes/no question.
/// Returns an error if the prior rule or the original desk prompt has changed.
pub(super) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack
        .data
        .maps
        .get(LOBBY)
        .context("Ely League lobby")?
        .clone();
    let previous = room
        .scripts
        .get(LABEL)
        .context("Ely League registration text")?;
    ensure!(
        previous
            .as_array()
            .is_some_and(|body| body.contains(&command("para", &["\"A loss or an exit\""])))
            && pack.data.map_scripts.get(LABEL) == Some(previous),
        "unexpected previous League retreat rule"
    );
    ensure!(
        room.scripts
            .get("MnElyRegister")
            .and_then(|s| s.as_array())
            .is_some_and(|body| {
                body.windows(2).any(|p| {
                    p[0] == command("writetext", &[LABEL]) && p[1] == command("yesorno", &[])
                })
            }),
        "registration no longer asks for a decision"
    );
    authored_dialogue::install_canonical(&mut room, LABEL)?;
    let updated = BTreeMap::from([(LABEL.to_owned(), room.scripts[LABEL].clone())]);
    room.script_text_bodies
        .extend(crate::parse_script_text_bodies(LOBBY, &updated)?);
    authored_town_publication::replace(pack, room)
}
