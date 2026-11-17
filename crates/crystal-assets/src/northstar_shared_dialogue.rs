//! Publish a shared text program only when every owner requests the same pages.

use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;
use crate::authored_dialogue::{self, AuthoredDialoguePage};

pub(crate) fn install(pack: &mut CompiledGamePack, story: &Value) -> Result<()> {
    let texts = &story["dialogue_overhaul"]["texts"];
    let mut owners: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (name, room) in &pack.data.maps {
        for label in room.scripts.keys() {
            owners.entry(label.clone()).or_default().push(name.clone());
        }
    }
    for (label, names) in owners {
        if names.len() < 2 || !names.iter().any(|name| texts[name].get(&label).is_some()) {
            continue;
        }
        let first = &names[0];
        let old = pack.data.maps[first].scripts[&label].clone();
        let pages = &texts[first][&label];
        ensure!(
            names.iter().all(|name| name.starts_with("Mn")
                && texts[name].get(&label) == Some(pages)
                && !pages.is_null()
                && pack.data.maps[name].scripts[&label] == old),
            "shared dialogue owners disagree: {label}"
        );
        let (terminal, lines) = old
            .as_array()
            .context("shared text program")?
            .split_last()
            .context("nonempty shared text")?;
        ensure!(
            matches!(terminal["command"].as_str(), Some("done" | "prompt"))
                && lines.iter().all(|line| matches!(
                    line["command"].as_str(),
                    Some("text" | "line" | "para" | "cont")
                )),
            "shared dialogue contains behavior: {label}"
        );
        let pages: Vec<AuthoredDialoguePage> = serde_json::from_value(pages.clone())?;
        let mut replacement = authored_dialogue::compile(&pages)?;
        *replacement
            .as_array_mut()
            .and_then(|rows| rows.last_mut())
            .expect("dialogue compiler emits a terminator") = terminal.clone();
        if let Some(mirror) = pack.data.map_scripts.get(&label) {
            ensure!(mirror == &old, "shared dialogue mirror differs: {label}");
        }
        // Change all definitions together before ordinary per-map publication.
        // Its shared-program invariant can then remain strict and unchanged.
        for name in &names {
            let room = pack.data.maps.get_mut(name).context("shared text owner")?;
            room.scripts.insert(label.clone(), replacement.clone());
            room.script_text_commands = crate::parse_script_text_commands(name, &room.scripts)?;
            room.script_text_bodies = crate::parse_script_text_bodies(name, &room.scripts)?;
        }
        if let Some(mirror) = pack.data.map_scripts.get_mut(&label) {
            *mirror = replacement;
        }
    }
    Ok(())
}
