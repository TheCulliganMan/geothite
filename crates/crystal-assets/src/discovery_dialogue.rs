//! Shared native page lowering for the optional discovery publishers.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::grand_marais_modpack::authoring::command as c;

pub(crate) fn lower(quest: &Value, prefix: &str) -> Result<BTreeMap<String, Value>> {
    let mut scripts = BTreeMap::new();
    for (key, pages) in quest["dialogue_pages"]
        .as_object()
        .context("discovery native dialogue")?
    {
        let pages = pages.as_array().context("native pages")?;
        ensure!(!pages.is_empty(), "empty discovery dialogue {key}");
        let mut commands = Vec::new();
        for (page_index, page) in pages.iter().enumerate() {
            let lines = page.as_array().context("native lines")?;
            ensure!(
                !lines.is_empty() && lines.len() <= 2,
                "two-line discovery page {key}"
            );
            for (line_index, line) in lines.iter().enumerate() {
                let line = line.as_str().context("native line")?;
                ensure!(
                    !line.is_empty()
                        && line.is_ascii()
                        && line.len() <= 18
                        && !line.contains(['"', '\n', '\r']),
                    "invalid discovery line in {key}"
                );
                let op = if line_index > 0 {
                    "line"
                } else if page_index > 0 {
                    "para"
                } else {
                    "text"
                };
                commands.push(c(op, &[&format!("\"{line}\"")]));
            }
        }
        commands.push(c("done", &[]));
        scripts.insert(format!("{prefix}{key}"), Value::Array(commands));
    }
    Ok(scripts)
}

pub(crate) fn script_closure(
    scripts: &BTreeMap<String, Value>,
    roots: &[String],
) -> Result<BTreeSet<String>> {
    let mut visited = BTreeSet::new();
    let mut pending = roots.to_vec();
    while let Some(label) = pending.pop() {
        if !visited.insert(label.clone()) {
            continue;
        }
        let commands = scripts
            .get(&label)
            .with_context(|| format!("missing discovery graph target {label}"))?
            .as_array()
            .context("native script commands")?;
        for command in commands {
            let op = command["command"].as_str().context("native opcode")?;
            let target_index = match op {
                "ifequal" => Some(1),
                "iftrue" | "iffalse" | "sjump" | "scall" | "jumptext" | "writetext" => Some(0),
                _ => None,
            };
            if let Some(index) = target_index {
                pending.push(
                    command["args"][index]
                        .as_str()
                        .context("native label target")?
                        .into(),
                );
            }
        }
    }
    Ok(visited)
}
