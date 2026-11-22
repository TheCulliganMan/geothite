//! Publish the full campaign's concise dialogue without replaying older overlays.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::CompiledGamePack;
use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring::command;

const MANIFEST: &str = "northstar-dialogue-clarity-v1";
const SAVE_POLICY: &str = "data/northstar-save-compatibility.json";
const SOUTH_TEXT: &str = "MnWTrailWardenSouthText";
const SOUTH_CLEAR_TEXT: &str = "MnWTrailWardenSouthClearedText";
const SOUTH_TALK: &str = "MnWTrailWardenSouthTalk";
const SOUTH_CLEAR: &str = "MnWTrailWardenSouthCleared";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Revision {
    id: String,
    parent_content_ids: Vec<String>,
    dialogue_labels: BTreeMap<String, Vec<String>>,
    welcome_labels: Vec<String>,
}

/// Replaces reviewed Minnesota passages and separates Winona's two wardens.
///
/// Pages come from `dialogue_overhaul.texts`; the revision selects the labels.
/// Plain text endings, scene behavior, map geometry, parties and audio remain
/// intact. The south warden retains the same starter check and road trigger.
/// This produces a review pack with a new identity, not an in-place save upgrade.
///
/// # Errors
/// Rejects an unreviewed parent, missing or conflicting text owners, non-text
/// controls in a replacement, changed warden wiring, or invalid native pages.
pub fn revise_northstar_dialogue_clarity(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let source: Value = serde_json::from_str(source).context("canonical story JSON")?;
    let revision: Revision = serde_json::from_value(source["dialogue_clarity_revision"].clone())
        .context("dialogue clarity selection")?;
    ensure!(revision.id == MANIFEST, "unknown dialogue revision");
    ensure!(
        revision
            .parent_content_ids
            .contains(&base.identity.content_hash)
            && !base.report.manifests.iter().any(|id| id == MANIFEST),
        "expected an exact reviewed Fair release parent"
    );
    ensure!(
        !revision.dialogue_labels.is_empty(),
        "empty dialogue revision"
    );
    let mut pack = base.clone();
    separate_wardens(&mut pack)?;
    let texts = &source["dialogue_overhaul"]["texts"];
    let mut selected = BTreeSet::new();
    for (map, labels) in &revision.dialogue_labels {
        ensure!(
            map.starts_with("Mn") && !labels.is_empty(),
            "invalid map selection: {map}"
        );
        for label in labels {
            ensure!(
                selected.insert((map, label)),
                "duplicate dialogue selection: {map}/{label}"
            );
        }
    }
    // Shared text is permitted only when every owner requests identical pages.
    // Validate before mutating any owner, including the global split mirror.
    for (map, label) in &selected {
        let original = pack
            .data
            .maps
            .get(*map)
            .context("selected map")?
            .scripts
            .get(*label)
            .with_context(|| format!("missing text {map}/{label}"))?;
        for (other_name, other) in &pack.data.maps {
            if let Some(body) = other.scripts.get(*label) {
                ensure!(
                    selected.contains(&(other_name, *label))
                        && texts[other_name][*label] == texts[*map][*label]
                        && body == original,
                    "shared dialogue owners disagree: {map}/{label} and {other_name}"
                );
            }
        }
    }
    for (map, label) in selected {
        let pages: Vec<AuthoredDialoguePage> = serde_json::from_value(texts[map][label].clone())
            .with_context(|| format!("authored pages {map}/{label}"))?;
        replace_text(&mut pack, map, label, &pages)?;
    }
    for label in &revision.welcome_labels {
        ensure!(
            matches!(
                label.as_str(),
                "OakText1"
                    | "OakText2"
                    | "OakText3"
                    | "OakText4"
                    | "OakText5"
                    | "OakText6"
                    | "OakText7"
            ),
            "unknown welcome label: {label}"
        );
        let pages: Vec<AuthoredDialoguePage> =
            serde_json::from_value(source["intro_revision"]["welcome"][label].clone())?;
        let text = pack
            .data
            .runtime_title_screen
            .program
            .text
            .iter_mut()
            .find(|text| &text.id == label)
            .context("installed welcome text")?;
        crate::northstar_intro_polish::replace_welcome(text, &pages)?;
    }
    // A review identity must not inherit a migration policy addressed to the
    // public Fair release. Existing saves remain attached to their original pack.
    pack.runtime_files.remove(SAVE_POLICY);
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

fn replace_text(
    pack: &mut CompiledGamePack,
    map_name: &str,
    label: &str,
    pages: &[AuthoredDialoguePage],
) -> Result<()> {
    let map = pack.data.maps.get_mut(map_name).context("dialogue map")?;
    let old = map.scripts.get(label).context("installed dialogue")?;
    let (ending, lines) = old
        .as_array()
        .context("text program")?
        .split_last()
        .context("nonempty text program")?;
    ensure!(
        matches!(ending["command"].as_str(), Some("done" | "prompt"))
            && lines.iter().all(|row| matches!(
                row["command"].as_str(),
                Some("text" | "line" | "para" | "cont")
            )),
        "replacement contains behavior: {map_name}/{label}"
    );
    let mut replacement = authored_dialogue::compile(pages)
        .with_context(|| format!("native pages {map_name}/{label}"))?;
    *replacement
        .as_array_mut()
        .and_then(|rows| rows.last_mut())
        .expect("authored compiler emits a terminator") = ending.clone();
    if let Some(mirror) = pack.data.map_scripts.get_mut(label) {
        // A previous identical shared owner may already have updated this mirror.
        ensure!(
            mirror == old || mirror == &replacement,
            "conflicting split text: {label}"
        );
        *mirror = replacement.clone();
    }
    let scripts = BTreeMap::from([(label.to_owned(), replacement.clone())]);
    let mut bodies = crate::parse_script_text_bodies(map_name, &scripts)?;
    map.script_text_bodies.insert(
        label.into(),
        bodies.remove(label).context("compiled text body")?,
    );
    map.scripts.insert(label.into(), replacement);
    Ok(())
}

fn separate_wardens(pack: &mut CompiledGamePack) -> Result<()> {
    let original = pack.data.maps.get("MnWinona").context("Winona map")?;
    let mut map = original.clone();
    let south_index = map
        .objects
        .iter()
        .position(|actor| actor.object_identifier.as_deref() == Some("MN_W_TRAIL_WARDEN_SOUTH"))
        .context("south trail warden")?;
    ensure!(
        map.objects[south_index].script == "MnWTrailWardenTalk",
        "changed south warden script"
    );
    ensure!(
        map.scripts["MnWTrailWardenTalk"]
            == json!([
                command("faceplayer", &[]),
                command("checkevent", &["EVENT_MN_STARTER_RECEIVED"]),
                command("iftrue", &["MnWTrailWardenCleared"]),
                command("jumptext", &["MnWTrailWardenText"]),
            ]),
        "changed warden starter check"
    );
    for label in [SOUTH_TEXT, SOUTH_CLEAR_TEXT, SOUTH_TALK, SOUTH_CLEAR] {
        ensure!(
            !map.scripts.contains_key(label),
            "south warden label already exists: {label}"
        );
    }
    map.scripts
        .insert(SOUTH_TEXT.into(), map.scripts["MnWTrailWardenText"].clone());
    map.scripts.insert(
        SOUTH_CLEAR_TEXT.into(),
        map.scripts["MnWTrailWardenClearedText"].clone(),
    );
    map.scripts.insert(
        SOUTH_TALK.into(),
        json!([
            command("faceplayer", &[]),
            command("checkevent", &["EVENT_MN_STARTER_RECEIVED"]),
            command("iftrue", &[SOUTH_CLEAR]),
            command("jumptext", &[SOUTH_TEXT]),
        ]),
    );
    map.scripts.insert(
        SOUTH_CLEAR.into(),
        json!([command("jumptext", &[SOUTH_CLEAR_TEXT])]),
    );
    let south_trigger = map
        .scripts
        .get_mut("MnWTrailWardenSouth")
        .and_then(Value::as_array_mut)
        .context("south road trigger")?;
    ensure!(
        south_trigger.get(5) == Some(&command("writetext", &["MnWTrailWardenText"])),
        "changed south trigger"
    );
    south_trigger[5] = command("writetext", &[SOUTH_TEXT]);
    map.objects[south_index].script = SOUTH_TALK.into();
    let event = map
        .map_event_section_commands
        .iter_mut()
        .filter(|row| row.command == "object_event")
        .nth(south_index)
        .context("south object event")?;
    ensure!(
        event.args.get(11).map(String::as_str) == Some("MnWTrailWardenTalk"),
        "changed south object event"
    );
    event.args[11] = SOUTH_TALK.into();
    // Reparse only the affected command families. Preserve all other tables and
    // source aliases rather than rebuilding this already-published map wholesale.
    map.script_flag_commands = crate::parse_script_flag_commands(&map.id, &map.scripts)?;
    map.script_object_commands = crate::parse_script_object_commands(&map.id, &map.scripts)?;
    map.script_control_commands = crate::parse_script_control_commands(&map.id, &map.scripts)?;
    map.script_text_commands = crate::parse_script_text_commands(&map.id, &map.scripts)?;
    map.script_text_bodies = crate::parse_script_text_bodies(&map.id, &map.scripts)?;
    crate::insert_compiled_map_split_payloads(
        &mut pack.data.map_scripts,
        &mut pack.data.npcs,
        &map.id,
        map.attributes
            .map_scripts_label
            .as_deref()
            .context("Winona scripts label")?,
        map.attributes
            .map_events_label
            .as_deref()
            .context("Winona events label")?,
        &map,
    )?;
    pack.data.maps.insert(map.id.clone(), map);
    Ok(())
}

#[cfg(test)]
#[path = "northstar_dialogue_clarity_tests.rs"]
mod tests;
