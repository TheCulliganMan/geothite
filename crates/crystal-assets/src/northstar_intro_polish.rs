//! Revise only the authored Northstar welcome and first-journey dialogue.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::{CompiledGamePack, RuntimePresentationText, RuntimePresentationTextCommand};

const MANIFEST: &str = "northstar-intro-polish-v1";
const OPENING_MARKER: &str = "data/title/northstar-opening-v1.json";
const OPENING_CONFIG: &[u8] = b"{\"schema_version\":1}";
const INTRO_MAPS: &[&str] = &[
    "MnWinonaHome1F",
    "MnWinonaHome2F",
    "MnWinona",
    "MnWinonaFieldStation",
    "MnRochesterPokecenter1F",
];
const WELCOME_LABELS: [&str; 7] = [
    "OakText1", "OakText2", "OakText3", "OakText4", "OakText5", "OakText6", "OakText7",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IntroRevision {
    id: String,
    dialogue_labels: BTreeMap<String, Vec<String>>,
    welcome: BTreeMap<String, Vec<AuthoredDialoguePage>>,
}

/// Revises the selected welcome and opening conversations from canonical JSON.
///
/// Keeps scene programs, progression, map geometry, portraits, cries and other content
/// unchanged. Adds the versioned opt-in for the original opening movie and
/// animated Northstar title.
/// The source selects existing dialogue labels in `intro_revision`; their
/// reviewed pages remain in `dialogue_overhaul.texts`.
///
/// # Errors
/// Rejects an incompatible or already revised pack, missing/shared text labels,
/// non-text programs, invalid native pages, or an invalid resulting pack.
pub fn polish_northstar_intro(base: &CompiledGamePack, source: &str) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|id| id == MANIFEST),
        "Northstar intro polish is already installed"
    );
    ensure!(
        !base.runtime_files.contains_key(OPENING_MARKER),
        "Northstar animated opening is already configured"
    );
    for (map, label) in [
        ("MnWinonaHome1F", "MnOverhaulMomScene"),
        ("MnWinonaFieldStation", "MnOverhaulLabEntry"),
        ("MnWinonaFieldStation", "MnNSEReedEncounter"),
    ] {
        ensure!(
            base.data
                .maps
                .get(map)
                .is_some_and(|room| room.scripts.contains_key(label)),
            "installed Northstar opening required: {map}/{label}"
        );
    }
    ensure!(
        base.data
            .runtime_title_screen
            .program
            .subprograms
            .iter()
            .any(|program| {
                program.id == "oak_speech"
                    && program.phases.iter().any(|phase| {
                        phase.operations.iter().any(|operation| {
                            operation.op == "present_intro_portrait"
                                && operation
                                    .fields
                                    .get("trainer_class")
                                    .and_then(Value::as_str)
                                    == Some("MN_PROF_TAMARACK")
                        })
                    })
            }),
        "installed regional Tamarack welcome required"
    );
    let source: Value = serde_json::from_str(source).context("canonical story JSON")?;
    let revision: IntroRevision = serde_json::from_value(source["intro_revision"].clone())
        .context("Northstar intro revision")?;
    ensure!(revision.id == MANIFEST, "unsupported intro revision");
    ensure!(
        !revision.dialogue_labels.is_empty(),
        "intro dialogue selection is empty"
    );
    ensure!(
        revision
            .welcome
            .keys()
            .map(String::as_str)
            .eq(WELCOME_LABELS),
        "intro welcome must contain exactly OakText1 through OakText7"
    );

    let mut pack = base.clone();
    let mut selected = BTreeSet::new();
    for (map_name, labels) in &revision.dialogue_labels {
        ensure!(
            INTRO_MAPS.contains(&map_name.as_str()),
            "outside intro scope: {map_name}"
        );
        ensure!(
            !labels.is_empty(),
            "empty dialogue selection for {map_name}"
        );
        for label in labels {
            ensure!(
                selected.insert(label),
                "duplicate selected dialogue label: {label}"
            );
            replace_dialogue(&mut pack, &source, map_name, label)?;
        }
    }
    for text in &mut pack.data.runtime_title_screen.program.text {
        if let Some(pages) = revision.welcome.get(&text.id) {
            replace_welcome(text, pages)?;
        }
    }
    for label in WELCOME_LABELS {
        ensure!(
            pack.data
                .runtime_title_screen
                .program
                .text
                .iter()
                .filter(|text| text.id == label)
                .count()
                == 1,
            "welcome label must have one installed owner: {label}"
        );
    }
    pack.runtime_files
        .insert(OPENING_MARKER.into(), OPENING_CONFIG.to_vec());
    pack.report.manifests.push(MANIFEST.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

fn replace_dialogue(
    pack: &mut CompiledGamePack,
    source: &Value,
    map_name: &str,
    label: &str,
) -> Result<()> {
    ensure!(
        !pack
            .data
            .maps
            .values()
            .any(|map| map.id != map_name && map.scripts.contains_key(label)),
        "intro dialogue has another map owner: {map_name}/{label}"
    );
    let map = pack
        .data
        .maps
        .get_mut(map_name)
        .with_context(|| format!("intro map {map_name}"))?;
    let old = map
        .scripts
        .get(label)
        .with_context(|| format!("installed intro text {map_name}/{label}"))?;
    let commands = old.as_array().context("intro text commands")?;
    let (ending, body) = commands
        .split_last()
        .context("empty installed intro text")?;
    ensure!(
        matches!(ending["command"].as_str(), Some("done" | "prompt"))
            && body.iter().all(|row| matches!(
                row["command"].as_str(),
                Some("text" | "line" | "para" | "cont")
            )),
        "intro replacement requires a plain text body: {map_name}/{label}"
    );
    let pages: Vec<AuthoredDialoguePage> =
        serde_json::from_value(source["dialogue_overhaul"]["texts"][map_name][label].clone())
            .with_context(|| format!("canonical intro pages {map_name}/{label}"))?;
    let mut replacement = authored_dialogue::compile(&pages)
        .with_context(|| format!("intro text {map_name}/{label}"))?;
    *replacement
        .as_array_mut()
        .and_then(|rows| rows.last_mut())
        .expect("authored compiler emits a terminator") = ending.clone();
    if let Some(mirror) = pack.data.map_scripts.get_mut(label) {
        ensure!(
            mirror == old,
            "conflicting split text mirror: {map_name}/{label}"
        );
        *mirror = replacement.clone();
    }
    // Parse only the selected body: rebuilding all split room payloads could
    // erase unrelated aliases or regenerate other authored command tables.
    let scripts = BTreeMap::from([(label.to_owned(), replacement.clone())]);
    let mut bodies = crate::parse_script_text_bodies(map_name, &scripts)?;
    let body = bodies.remove(label).context("compiled intro text body")?;
    ensure!(
        crate::parse_script_text_commands(map_name, &scripts)?.is_empty(),
        "intro text must not introduce scene commands"
    );
    map.script_text_bodies.insert(label.to_owned(), body);
    map.scripts.insert(label.to_owned(), replacement);
    Ok(())
}

pub(crate) fn replace_welcome(
    text: &mut RuntimePresentationText,
    pages: &[AuthoredDialoguePage],
) -> Result<()> {
    // The welcome alone accepts the existing player-name and Pokemon tokens.
    // Validate their fully expanded widths before preserving the native tokens.
    let expanded: Vec<_> = pages
        .iter()
        .map(|page| AuthoredDialoguePage {
            lines: page
                .lines
                .iter()
                .map(|line| {
                    line.replace("<PLAYER>", "1234567")
                        .replace("#MON", "POKEMON")
                })
                .collect(),
        })
        .collect();
    authored_dialogue::compile(&expanded).with_context(|| format!("welcome pages {}", text.id))?;
    let (ending, body) = text
        .commands
        .split_last()
        .context("installed welcome text")?;
    ensure!(
        matches!(ending.command.as_str(), "prompt" | "done"),
        "unsupported welcome ending: {}",
        text.id
    );
    let leading_prompt = text.id == "OakText3";
    ensure!(
        body.iter().enumerate().all(|(index, row)| {
            matches!(row.command.as_str(), "text" | "line" | "para" | "cont")
                || (leading_prompt && index == 0 && row.command == "text_promptbutton")
        }) && (!leading_prompt
            || body
                .first()
                .is_some_and(|row| row.command == "text_promptbutton")),
        "unexpected welcome controls: {}",
        text.id
    );
    let mut commands = Vec::new();
    if leading_prompt {
        commands.push(body[0].clone());
    }
    for (page_index, page) in pages.iter().enumerate() {
        for (line_index, line) in page.lines.iter().enumerate() {
            let command = if line_index == 1 {
                "line"
            } else if page_index == 0 {
                "text"
            } else {
                "para"
            };
            commands.push(RuntimePresentationTextCommand {
                command: command.into(),
                args: vec![format!("\"{line}\"")],
                source_span: text.source_span.clone(),
            });
        }
    }
    commands.push(ending.clone());
    text.commands = commands;
    Ok(())
}

#[cfg(test)]
#[path = "northstar_intro_polish_tests.rs"]
mod tests;
