//! Retain the native field move from the user's external pinned source file.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{CompiledGamePack, RuntimeScriptCommandRef};

const SOURCE_SHA256: &str = "0031b07bd338e17d630ca5ada459df53f0c9452ec7bea82e8649f492c128b0c6";
pub(super) const LABELS: [&str; 9] = [
    ".SweetScent@SweetScentFromMenu",
    ".BugCatchingContest@SweetScentFromMenu",
    "SweetScentNothing",
    "SweetScentEncounter",
    ".in_bug_contest@SweetScentEncounter",
    ".start_battle@SweetScentEncounter",
    ".no_battle@SweetScentEncounter",
    "UseSweetScentText",
    "SweetScentNothingText",
];

pub(super) fn install(pack: &mut CompiledGamePack, source: &str) -> Result<()> {
    ensure!(
        format!("{:x}", Sha256::digest(source.as_bytes())) == SOURCE_SHA256,
        "native engine/events/sweet_scent.asm differs from the reviewed pinned source"
    );
    let programs = parse(source)?;
    let target = pack
        .data
        .story_events
        .iter_mut()
        .find_map(|catalog| catalog.get_mut("StandardScripts"))
        .and_then(Value::as_object_mut)
        .context("native standard scripts")?;
    for label in LABELS {
        let body = programs
            .get(label)
            .with_context(|| format!("native Sweet Scent label {label}"))?;
        if let Some(previous) = target.get(label) {
            ensure!(
                previous == body,
                "native Sweet Scent overlap changed: {label}"
            );
        } else {
            target.insert(label.into(), body.clone());
        }
    }
    let root = LABELS[0];
    let roots = target
        .get_mut("GlobalScriptRoots")
        .and_then(Value::as_array_mut)
        .context("native executable roots")?;
    if !roots.iter().any(|label| label == root) {
        roots.push(Value::String(root.into()));
    }
    pack.data.materialize_global_scripts()?;
    ensure!(
        pack.data
            .is_exact_sweet_scent_encounter_command(&RuntimeScriptCommandRef::new(
                "MnR13Cave1F",
                root,
                5
            ),)?,
        "native Sweet Scent source certificate failed"
    );
    Ok(())
}

fn parse(source: &str) -> Result<BTreeMap<String, Value>> {
    let mut programs = BTreeMap::<String, Vec<Value>>::new();
    let mut parent = String::new();
    let mut active = None;
    let mut order = Vec::new();
    for raw in source.lines() {
        let line = raw.split(';').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        if line.ends_with(':') || line.starts_with('.') {
            let label = line.trim_end_matches(':');
            let name = if label.starts_with('.') {
                ensure!(!parent.is_empty(), "native local label before its parent");
                format!("{label}@{parent}")
            } else {
                parent = label.into();
                parent.clone()
            };
            ensure!(
                programs.insert(name.clone(), Vec::new()).is_none(),
                "duplicate native label"
            );
            order.push(name.clone());
            active = Some(name);
        } else {
            let (command, operands) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
            let args: Vec<_> = operands
                .split(',')
                .map(str::trim)
                .filter(|arg| !arg.is_empty())
                .collect();
            programs
                .get_mut(active.as_ref().context("native command before its label")?)
                .context("native source block")?
                .push(json!({"command":command,"args":args}));
        }
    }
    ensure!(
        programs.len() == 10,
        "complete pinned Sweet Scent source required"
    );
    // The native contest branch physically falls through to the next block.
    // Make that edge explicit while preserving each source block's own indices.
    let branch = ".in_bug_contest@SweetScentEncounter";
    let next = ".start_battle@SweetScentEncounter";
    ensure!(
        order
            .windows(2)
            .any(|pair| pair[0] == branch && pair[1] == next),
        "native Sweet Scent fallthrough changed"
    );
    programs
        .get_mut(branch)
        .context("native contest branch")?
        .push(json!({"command":"jp","args":[next]}));
    Ok(programs
        .into_iter()
        .map(|(label, body)| (label, Value::Array(body)))
        .collect())
}
