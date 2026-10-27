//! Local residents react to the persistent lighthouse repair through native scripts.
use anyhow::{Result, ensure};
use serde_json::json;

use super::SIGNAL;
use super::authoring::{command, speech, text};
use super::design::Actor;
use crate::{CompiledGamePack, MapModule};

pub(super) fn add(base: &CompiledGamePack, module: &mut MapModule, actor: &Actor) -> Result<()> {
    let lines = actor.lines.iter().map(String::as_str).collect::<Vec<_>>();
    speech(
        base,
        module,
        &actor.id,
        &actor.sprite,
        (actor.tile[0], actor.tile[1]),
        &lines,
    )?;
    if let Some(lines) = &actor.signal_lines {
        ensure!(
            !lines.is_empty(),
            "{} needs dialogue after the signal repair",
            actor.id
        );
        let script = format!("{}{}", module.id, actor.id);
        let before = format!("{script}Text");
        let after = format!("{script}Signal");
        let after_text = format!("{after}Text");
        module.scripts.insert(
            script,
            json!([
                command("checkevent", &[SIGNAL]),
                command("iftrue", &[&after]),
                command("jumptextfaceplayer", &[&before])
            ]),
        );
        module.scripts.insert(
            after,
            json!([command("jumptextfaceplayer", &[&after_text])]),
        );
        text(
            module,
            &after_text,
            &lines.iter().map(String::as_str).collect::<Vec<_>>(),
        );
    }
    Ok(())
}
