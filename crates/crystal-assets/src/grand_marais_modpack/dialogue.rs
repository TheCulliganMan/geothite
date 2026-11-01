//! Resolve authored text without rebuilding the town's native script logic.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use super::authoring::text;
use crate::MapModule;

pub(super) fn apply(
    authored: &BTreeMap<String, BTreeMap<String, Vec<String>>>,
    modules: &mut [MapModule],
) -> Result<()> {
    for name in authored.keys() {
        ensure!(
            modules.iter().any(|module| module.id == *name),
            "dialogue names unknown map {name}"
        );
    }
    for module in modules {
        let mut references = BTreeSet::new();
        for script in module.scripts.values().filter_map(Value::as_array) {
            for command in script {
                if matches!(
                    command.get("command").and_then(Value::as_str),
                    Some("writetext" | "jumptext" | "jumptextfaceplayer")
                ) {
                    let label = command
                        .get("args")
                        .and_then(|args| args.get(0))
                        .and_then(Value::as_str)
                        .context("literal town dialogue label")?;
                    references.insert(label.to_owned());
                }
            }
        }
        if let Some(entries) = authored.get(&module.id) {
            for (label, lines) in entries {
                ensure!(
                    references.contains(label) && !has_text(module, label),
                    "{} dialogue {label} is unused or already authored elsewhere",
                    module.id
                );
                ensure!(
                    !lines.is_empty()
                        && lines
                            .iter()
                            .all(|line| { !line.trim().is_empty() && line.chars().count() <= 18 }),
                    "{} dialogue {label} needs nonempty lines of at most 18 characters",
                    module.id
                );
                text(
                    module,
                    label,
                    &lines.iter().map(String::as_str).collect::<Vec<_>>(),
                );
            }
        }
        for label in references {
            ensure!(
                has_text(module, &label),
                "{} is missing authored dialogue {label}",
                module.id
            );
        }
    }
    Ok(())
}

fn has_text(module: &MapModule, label: &str) -> bool {
    module
        .scripts
        .get(label)
        .and_then(Value::as_array)
        .is_some_and(|commands| {
            commands
                .iter()
                .any(|command| command.get("command").and_then(Value::as_str) == Some("text"))
        })
}
