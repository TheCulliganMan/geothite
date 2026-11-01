//! Bootstrap authored script catalogs from external pret source, without dumps.
use super::radio_text_export::{code_part, source_arguments};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

fn files(path: &Path, output: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(path)? {
        let path = entry?.path();
        if path.is_dir() {
            files(&path, output)?;
        } else if path.extension().is_some_and(|ext| ext == "asm") {
            output.push(path);
        }
    }
    Ok(())
}

pub(super) fn parse(source: &str) -> Result<BTreeMap<String, Value>> {
    let expanded = expand_underground_doors(source)?;
    let source = expanded.as_str();
    let mut scripts = BTreeMap::new();
    let mut parent = String::new();
    let mut active: Option<String> = None;
    let mut macro_body = false;
    let mut conditionals: Vec<(bool, bool)> = Vec::new();
    let mut ordered = Vec::new();
    for raw in source.lines() {
        let line = code_part(raw);
        if line.starts_with("MACRO ") {
            macro_body = true;
            continue;
        }
        if line == "ENDM" {
            macro_body = false;
            continue;
        }
        if macro_body || line.is_empty() {
            continue;
        }
        if let Some(condition) = line.strip_prefix("if ") {
            ensure!(
                condition.starts_with("DEF(") && condition.ends_with(')'),
                "unsupported source conditional {line}"
            );
            let parent = conditionals.iter().all(|(parent, value)| *parent && *value);
            conditionals.push((parent, false));
            continue;
        }
        if line == "else" {
            let (_, value) = conditionals.last_mut().context("source else without if")?;
            *value = !*value;
            continue;
        }
        if line == "endc" {
            ensure!(conditionals.pop().is_some(), "source endc without if");
            continue;
        }
        if !conditionals.iter().all(|(parent, value)| *parent && *value) {
            continue;
        }
        let label = if line.ends_with(':') {
            Some(line.trim_end_matches(':'))
        } else if line.starts_with('.') && !line.contains(char::is_whitespace) {
            Some(line)
        } else {
            None
        };
        if let Some(label) = label {
            let scoped = if label.starts_with('.') {
                ensure!(!parent.is_empty(), "local label without parent: {label}");
                format!("{label}@{parent}")
            } else {
                parent = label.to_owned();
                label.to_owned()
            };
            ordered.push(scoped.clone());
            ensure!(
                scripts.insert(scoped.clone(), json!([])).is_none(),
                "duplicate script {scoped}"
            );
            active = Some(scoped);
            continue;
        }
        let Some(label) = active.as_ref() else {
            continue;
        };
        let (command, operands) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
        if matches!(
            command,
            "INCLUDE"
                | "SECTION"
                | "DEF"
                | "REDEF"
                | "IF"
                | "ELSE"
                | "ENDC"
                | "assert"
                | "assert_table_length"
                | "table_width"
        ) {
            continue;
        }
        let mut args = source_arguments(operands)?;
        // The runtime stores map, x, y, facing; pret authors facing first.
        if command == "warpfacing" {
            ensure!(args.len() == 4, "source warpfacing arguments");
            args = vec![
                args[1].clone(),
                args[2].clone(),
                args[3].clone(),
                args[0].clone(),
            ];
        }
        let entry = json!({"command": command, "args": args});
        scripts
            .get_mut(label)
            .unwrap()
            .as_array_mut()
            .unwrap()
            .push(entry.clone());
        if label != &parent {
            scripts
                .get_mut(&parent)
                .unwrap()
                .as_array_mut()
                .unwrap()
                .push(entry);
        }
    }
    ensure!(conditionals.is_empty(), "unterminated source conditional");
    // Movement labels can enter a shared byte sequence midway. Materialize
    // that exact forward sequence through its authored terminating opcode.
    for (index, label) in ordered.iter().enumerate().rev() {
        let body = scripts[label].as_array().unwrap();
        let movement = body
            .first()
            .and_then(|entry| entry["command"].as_str())
            .is_some_and(|command| {
                matches!(
                    command,
                    "step" | "big_step" | "turn_head" | "slow_step" | "slide_step"
                )
            });
        if (body.is_empty() || movement && !body.iter().any(|entry| entry["command"] == "step_end"))
            && let Some(next) = ordered.get(index + 1)
        {
            let continuation = scripts[next].as_array().unwrap().clone();
            scripts
                .get_mut(label)
                .unwrap()
                .as_array_mut()
                .unwrap()
                .extend(continuation);
        }
    }
    Ok(scripts)
}

/// Preserve authored event-bytecode fallthrough across source labels. An
/// explicit jump retains each label's own command indices for battle resumes.
pub(super) fn materialize_event_fallthroughs(
    vendor: &Path,
    source: &str,
    scripts: &mut BTreeMap<String, Value>,
) -> Result<()> {
    let macros = std::fs::read_to_string(vendor.join("macros/scripts/events.asm"))?;
    let opcodes = macros
        .lines()
        .map(code_part)
        .filter_map(|line| line.strip_prefix("MACRO ").map(str::to_owned))
        .collect::<BTreeSet<_>>();
    ensure!(
        opcodes.contains("closetext") && opcodes.contains("sjump"),
        "source event opcode macros incomplete"
    );
    let source = expand_underground_doors(source)?;
    let mut ordered = Vec::<String>::new();
    let mut parent = String::new();
    let mut in_macro = false;
    for line in source.lines().map(code_part) {
        if line.starts_with("MACRO ") {
            in_macro = true;
            continue;
        }
        if line == "ENDM" {
            in_macro = false;
            continue;
        }
        if in_macro {
            continue;
        }
        let label = if line.ends_with(':') {
            Some(line.trim_end_matches(':'))
        } else if line.starts_with('.') && !line.contains(char::is_whitespace) {
            Some(line)
        } else {
            None
        };
        let Some(label) = label else {
            continue;
        };
        let scoped = if label.starts_with('.') {
            format!("{label}@{parent}")
        } else {
            parent = label.to_owned();
            label.to_owned()
        };
        if scripts.contains_key(&scoped) {
            ordered.push(scoped);
        }
    }
    let mut jumps = Vec::new();
    for (index, label) in ordered.iter().enumerate() {
        let body = scripts[label].as_array().context("event source body")?;
        let Some(last) = body.last().and_then(|entry| entry["command"].as_str()) else {
            continue;
        };
        if matches!(
            last,
            "end"
                | "endall"
                | "endcallback"
                | "reloadend"
                | "sjump"
                | "farsjump"
                | "memjump"
                | "stopandsjump"
                | "jumpstd"
                | "jumptext"
                | "jumptextfaceplayer"
                | "farjumptext"
                | "sret"
        ) || !body.iter().all(|entry| {
            entry["command"]
                .as_str()
                .is_some_and(|command| opcodes.contains(command))
        }) {
            continue;
        }
        // Global bodies also contain their local labels; their physical tail
        // follows the final local label rather than restarting the first one.
        let next = ordered[index + 1..]
            .iter()
            .find(|next| label.starts_with('.') || !next.starts_with('.'));
        let Some(next) = next else {
            continue;
        };
        if !scripts[next]
            .as_array()
            .and_then(|body| body.first())
            .and_then(|entry| entry["command"].as_str())
            .is_some_and(|command| opcodes.contains(command))
        {
            continue;
        }
        jumps.push((label.clone(), next.clone()));
    }
    for (label, next) in jumps {
        scripts
            .get_mut(&label)
            .unwrap()
            .as_array_mut()
            .unwrap()
            .push(json!({"command":"sjump", "args":[next]}));
    }
    Ok(())
}

// These source macros define each door's literal block edits. Expand their
// authored arguments before script serialization, preserving local labels.
fn expand_underground_doors(source: &str) -> Result<String> {
    if !source.contains("MACRO ugdoor_def") {
        return Ok(source.to_owned());
    }
    let mut doors = Vec::new();
    for line in source.lines().map(code_part) {
        if let Some(args) = line.strip_prefix("ugdoor_def ") {
            let args = source_arguments(args)?;
            ensure!(
                !args.is_empty() && args.len() % 4 == 0,
                "invalid source underground door"
            );
            doors.push(args);
        }
    }
    let lines: Vec<_> = source.lines().collect();
    let mut output = String::new();
    let mut index = 0;
    let mut inside_macro = false;
    while index < lines.len() {
        let line = code_part(lines[index]);
        if line.starts_with("MACRO ") {
            inside_macro = true;
        }
        if line == "ENDM" {
            inside_macro = false;
        }
        if !inside_macro && line == "for n, 1, ugdoor_n + 1" {
            let start = index + 1;
            let end = (start..lines.len())
                .find(|&i| code_part(lines[i]) == "endr")
                .context("unterminated source door loop")?;
            for (door, args) in doors.iter().enumerate() {
                for raw in &lines[start..end] {
                    let expanded = raw.replace("{d:n}", &(door + 1).to_string());
                    let code = code_part(&expanded);
                    if let Some(call) = code.strip_prefix("changeugdoor ") {
                        let call = source_arguments(call)?;
                        ensure!(
                            call.len() == 2 && call[0] == "n",
                            "unexpected source door invocation"
                        );
                        let state = match call[1].as_str() {
                            "OPEN" => 3,
                            "CLOSED" => 2,
                            _ => anyhow::bail!("source door state"),
                        };
                        for block in args.chunks_exact(4) {
                            output.push_str(&format!(
                                "\tchangeblock {}, {}, {}\n",
                                block[0], block[1], block[state]
                            ));
                        }
                    } else {
                        output.push_str(&expanded);
                        output.push('\n');
                    }
                }
            }
            index = end + 1;
            continue;
        }
        output.push_str(lines[index]);
        output.push('\n');
        index += 1;
    }
    Ok(output)
}

pub(super) fn ensure_standard_catalog(repository: &Path) -> Result<()> {
    let data = repository.join("apps/web/assets/data");
    let aggregate = data.join("story_events/StandardScripts.json");
    let vendor = repository.join("vendor/pokecrystal");
    let standard = parse(&std::fs::read_to_string(
        vendor.join("engine/events/std_scripts.asm"),
    )?)?;
    let pointers = standard
        .get("StdScripts")
        .and_then(Value::as_array)
        .context("source StdScripts pointer table")?;
    let mut roots = BTreeSet::new();
    for entry in pointers {
        ensure!(
            entry["command"] == "add_stdscript",
            "invalid source standard-script pointer"
        );
        let label = entry["args"][0]
            .as_str()
            .context("source standard-script target")?;
        ensure!(
            standard
                .get(label)
                .and_then(Value::as_array)
                .is_some_and(|body| !body.is_empty()),
            "source standard script {label} missing"
        );
        roots.insert(label.to_owned());
    }
    let mut catalog = standard;
    let mut paths = Vec::new();
    files(&vendor.join("data/text"), &mut paths)?;
    paths.sort();
    for path in paths {
        for (label, body) in parse(&std::fs::read_to_string(&path)?)
            .with_context(|| format!("parse {}", path.display()))?
        {
            ensure!(
                catalog.insert(label.clone(), body).is_none(),
                "duplicate global text {label}"
            );
        }
    }
    catalog.insert("GlobalScriptRoots".into(), json!(roots));
    let mut bytes = serde_json::to_vec_pretty(&json!({"StandardScripts": catalog}))?;
    bytes.push(b'\n');
    for path in [
        aggregate,
        data.join("content-packs/core-modular/story_events/StandardScripts.json"),
    ] {
        std::fs::create_dir_all(path.parent().context("script output parent")?)?;
        std::fs::write(path, &bytes)?;
    }
    Ok(())
}

#[cfg(test)]
mod rebuild_tests {
    use super::*;

    #[test]
    fn standard_scripts_rebuild_with_stale_aggregate_and_missing_modular_file() {
        let root = std::env::temp_dir().join(format!(
            "geothite-standard-rebuild-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let result = (|| -> Result<()> {
            let source = root.join("vendor/pokecrystal/engine/events/std_scripts.asm");
            std::fs::create_dir_all(source.parent().unwrap())?;
            std::fs::create_dir_all(root.join("vendor/pokecrystal/data/text"))?;
            std::fs::write(
                &source,
                "StdScripts:\n add_stdscript FreshScript\nFreshScript:\n return\n",
            )?;
            let data = root.join("apps/web/assets/data");
            std::fs::create_dir_all(data.join("story_events"))?;
            std::fs::create_dir_all(data.join("content-packs"))?;
            std::fs::write(data.join("story_events/StandardScripts.json"), "stale")?;
            std::fs::write(
                data.join("content-packs/core-modular.generated.json"),
                "stale",
            )?;
            ensure_standard_catalog(&root)?;
            let aggregate = std::fs::read(data.join("story_events/StandardScripts.json"))?;
            let modular = std::fs::read(
                data.join("content-packs/core-modular/story_events/StandardScripts.json"),
            )?;
            ensure!(aggregate == modular, "standard script copies differ");
            let catalog: Value = serde_json::from_slice(&aggregate)?;
            ensure!(
                catalog["StandardScripts"]["FreshScript"].is_array(),
                "fresh source standard script missing"
            );
            std::fs::write(
                &source,
                "StdScripts:\n add_stdscript ChangedScript\nChangedScript:\n end\n",
            )?;
            ensure_standard_catalog(&root)?;
            let changed: Value = serde_json::from_slice(&std::fs::read(
                data.join("story_events/StandardScripts.json"),
            )?)?;
            ensure!(
                changed["StandardScripts"]["ChangedScript"].is_array()
                    && changed["StandardScripts"].get("FreshScript").is_none(),
                "existing outputs masked changed source"
            );
            Ok(())
        })();
        let _ = std::fs::remove_dir_all(root);
        result.unwrap();
    }
}
