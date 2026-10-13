//! Authored global script catalogs; source bodies retain CPU/data commands.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

fn source_files(root: &Path, output: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            source_files(&path, output)?;
        } else if path.extension().is_some_and(|extension| extension == "asm") {
            output.push(path);
        }
    }
    Ok(())
}

pub(super) fn read(vendor: &Path) -> Result<(Value, Value)> {
    let mut paths = Vec::new();
    source_files(&vendor.join("engine/phone/scripts"), &mut paths)?;
    paths.sort();
    let mut phone = BTreeMap::<String, Value>::new();
    for path in paths {
        for (label, body) in super::source_scripts::parse(&std::fs::read_to_string(&path)?)
            .with_context(|| format!("parse authored phone source {}", path.display()))?
        {
            ensure!(
                phone.insert(label.clone(), body).is_none(),
                "duplicate phone label {label}"
            );
        }
    }
    // Phone callbacks live beside the CPU dispatcher, outside scripts/.
    // Preserve their authored bodies and text aliases without importing the
    // dispatcher as executable script code.
    let dispatcher = super::source_scripts::parse(&std::fs::read_to_string(
        vendor.join("engine/phone/phone.asm"),
    )?)?;
    for root in [
        "LoadPhoneScriptBank",
        "LoadOutOfAreaScript",
        "PhoneOutOfAreaScript",
        "PhoneOutOfAreaText",
        "PhoneScript_JustTalkToThem",
        "PhoneJustTalkToThemText",
    ] {
        let body = dispatcher
            .get(root)
            .with_context(|| format!("missing authored phone callback {root}"))?;
        ensure!(
            phone.insert(root.into(), body.clone()).is_none(),
            "duplicate phone callback {root}"
        );
    }
    let mut events = super::source_scripts::parse(&std::fs::read_to_string(
        vendor.join("engine/overworld/events.asm"),
    )?)?;
    for relative in [
        "engine/events/overworld.asm",
        "engine/events/whiteout.asm",
        "engine/events/bug_contest/contest.asm",
        "engine/events/trainer_scripts.asm",
        "engine/events/misc_scripts.asm",
        "engine/overworld/player_object.asm",
        "engine/pokemon/breeding.asm",
        "engine/events/treemons.asm",
    ] {
        let path = vendor.join(relative);
        for (label, body) in super::source_scripts::parse(&std::fs::read_to_string(&path)?)
            .with_context(|| format!("parse authored events {relative}"))?
        {
            ensure!(
                events.insert(label.clone(), body).is_none(),
                "duplicate event label {label}"
            );
        }
    }
    ensure!(
        events.contains_key("PlayerEventScriptPointers"),
        "missing authored player event pointer table"
    );
    // Pointer consumers certify the entry sequence itself. Local labels remain
    // separate definitions; appended local branches are not entry instructions.
    let roots = events["PlayerEventScriptPointers"]
        .as_array()
        .context("player pointers")?
        .iter()
        .filter(|entry| entry["command"] == "dba")
        .map(|entry| {
            entry["args"][0]
                .as_str()
                .context("player pointer target")
                .map(str::to_owned)
        })
        .collect::<Result<Vec<_>>>()?;
    for root in roots {
        let body = events
            .get_mut(&root)
            .with_context(|| format!("missing player event {root}"))?
            .as_array_mut()
            .context("player event body")?;
        if root == "OverworldWhiteoutScript" {
            let source = std::fs::read_to_string(vendor.join("engine/events/whiteout.asm"))?;
            let tail = source
                .split("OverworldWhiteoutScript::")
                .nth(1)
                .context("whiteout source entry")?;
            let next = tail
                .lines()
                .map(super::source_syntax::code_part)
                .find(|line| line.ends_with(':'))
                .context("whiteout fallthrough label")?;
            ensure!(
                next == "Script_Whiteout:",
                "unexpected whiteout fallthrough {next}"
            );
            body.push(json!({"command":"sjump", "args":["Script_Whiteout"]}));
        }
        let end = body
            .iter()
            .position(|entry| {
                matches!(
                    entry["command"].as_str(),
                    Some("end" | "sjump" | "reloadend")
                )
            })
            .with_context(|| format!("unterminated authored player entry {root}"))?;
        body.truncate(end + 1);
    }
    // The menu entry falls through into the adjacent shared field script.
    let source = std::fs::read_to_string(vendor.join("engine/events/overworld.asm"))?;
    let tail = source
        .split("RockSmashFromMenuScript:")
        .nth(1)
        .context("Rock Smash menu source entry")?;
    let next = tail
        .lines()
        .map(super::source_syntax::code_part)
        .find(|line| line.ends_with(':'))
        .context("Rock Smash menu fallthrough label")?;
    ensure!(
        next == "RockSmashScript:",
        "unexpected Rock Smash menu fallthrough {next}"
    );
    events
        .get_mut("RockSmashFromMenuScript")
        .context("Rock Smash menu entry")?
        .as_array_mut()
        .context("Rock Smash menu body")?
        .push(json!({"command":"sjump", "args":["RockSmashScript"]}));
    // Typed encounter certificates address the entry range, while branch
    // definitions retain their own source ranges (including unused data).
    for (root, terminator) in [
        ("RockMonEncounter", "ret"),
        ("TreeMonEncounter", "ret"),
        ("RockSmashScript", "end"),
        ("RockSmashFromMenuScript", "sjump"),
    ] {
        let body = events
            .get_mut(root)
            .with_context(|| format!("missing authored encounter entry {root}"))?
            .as_array_mut()
            .context("encounter entry body")?;
        let end = body
            .iter()
            .position(|entry| {
                entry["command"] == terminator
                    && (terminator != "ret" || entry["args"].as_array().is_some_and(Vec::is_empty))
            })
            .with_context(|| format!("unterminated authored encounter entry {root}"))?;
        body.truncate(end + 1);
    }
    Ok((json!(phone), json!({"OverworldEvents": events})))
}

pub fn export(root: &Path) -> Result<()> {
    super::source_scripts::ensure_standard_catalog(root)?;
    let data = root.join("apps/web/assets/data");
    let (phone, events) = read(&root.join("vendor/pokecrystal"))?;
    for (name, value) in [
        ("phone_scripts/Global.json", phone),
        ("story_events/OverworldEvents.json", events),
    ] {
        let bytes = serde_json::to_vec_pretty(&value)?;
        for directory in [&data, &data.join("content-packs/core-modular")] {
            let path = directory.join(name);
            std::fs::create_dir_all(path.parent().context("global script output parent")?)?;
            std::fs::write(path, &bytes)?;
        }
    }
    Ok(())
}
