//! Import the pinned native current pushback program directly from local source.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;

pub(crate) fn install(pack: &mut CompiledGamePack, source: &str) -> Result<()> {
    ensure!(
        format!("{:x}", Sha256::digest(source.as_bytes()))
            == "ebc02a71ca708b145292687150ae1c3e8617b2a32d042c090e6c68d16e0a7abb",
        "native forced-movement source differs from the pinned source"
    );
    let mut programs = BTreeMap::<String, Vec<Value>>::new();
    let mut active = None;
    for raw in source.lines() {
        let line = raw.split(';').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        if line.ends_with(':') || line.starts_with('.') {
            let label = line.trim_end_matches(':');
            let name = if label.starts_with('.') {
                format!("{label}@Script_ForcedMovement")
            } else {
                ensure!(
                    label == "Script_ForcedMovement",
                    "unexpected native movement root"
                );
                label.into()
            };
            ensure!(
                programs.insert(name.clone(), Vec::new()).is_none(),
                "duplicate movement label"
            );
            active = Some(name);
        } else {
            let (command, tail) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
            let args = tail
                .split(',')
                .map(str::trim)
                .filter(|arg| !arg.is_empty())
                .collect::<Vec<_>>();
            programs
                .get_mut(active.as_ref().context("movement command before root")?)
                .context("active native movement label")?
                .push(json!({"command": command, "args": args}));
        }
    }
    ensure!(programs.len() == 9, "complete native movement program");
    let catalog = pack
        .data
        .story_events
        .iter_mut()
        .find_map(|event| event.get_mut("OverworldEvents"))
        .and_then(Value::as_object_mut)
        .context("native overworld source catalog")?;
    for (label, rows) in programs {
        let body = Value::Array(rows);
        if let Some(previous) = catalog.get(&label) {
            ensure!(
                previous == &body,
                "native movement source overlap changed: {label}"
            );
        } else {
            catalog.insert(label, body);
        }
    }
    Ok(())
}
