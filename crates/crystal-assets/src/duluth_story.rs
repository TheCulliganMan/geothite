//! Canonical MS10 reveal on the installed city's production harbor receiver.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use serde_json::Value;

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

const WORKSHOP: &str = "MnDuluthHarborWorkshop";
const RECEIVER: &str = "MN_HARBOR_RECEIVER";
const DEPOT: &str = "EVENT_MN_DEPOT_CLEARED";
const SIGNAL: &str = "EVENT_MN_LIGHTHOUSE_SIGNAL";
const MANIFEST: &str = "minnesota-duluth-story-v1";
const LABELS: [&str; 3] = [
    "MN_HARBOR_RECEIVERText",
    "MN_HARBOR_RECEIVERDeniedText",
    "MN_HARBOR_RECEIVERDoneText",
];

/// Binds MS10's Kettlurk reveal to the existing native harbor interaction.
///
/// All eight badges and acknowledged Grand Rapids clearance remain required.
/// The existing receiver script sets only the harbor signal, after every page
/// has been acknowledged. This overlay changes exactly three text bodies;
/// geometry, objects, commands, unrelated catalogs, art and audio are retained.
/// It neither awards prerequisites nor creates a physical meter in the bag:
/// the depot retained the meter and supplied the player with its copied log.
///
/// # Errors
/// Returns an error for duplicate installation, missing or changed native
/// receiver logic/anchor, incompatible milestone prerequisites or results,
/// malformed page text, or native parser/pack verification failures. Borrowed
/// inputs remain unchanged on success and failure.
pub fn build_duluth_story_modpack(
    base: &CompiledGamePack,
    story_json: &str,
) -> Result<CompiledGamePack> {
    rewrite_dialogue(base, story_json, true)
}

/// Refreshes an installed MS10 reveal from the canonical story source.
///
/// Requires the original badge, depot and page acknowledgment checks. Only the
/// three existing receiver text bodies and their parsed views change. Repeating
/// a refresh with the same source is idempotent; no progression is granted.
///
/// # Errors
/// Returns an error for an absent installation, changed receiver logic/anchor,
/// incompatible milestone, malformed text, or pack verification failure. The
/// borrowed input remains unchanged.
pub fn refresh_duluth_story_dialogue(
    base: &CompiledGamePack,
    story_json: &str,
) -> Result<CompiledGamePack> {
    rewrite_dialogue(base, story_json, false)
}

fn rewrite_dialogue(
    base: &CompiledGamePack,
    story_json: &str,
    installing: bool,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.report.manifests.iter().any(|s| s == MANIFEST) != installing,
        "Duluth story installation state does not match the requested operation"
    );
    let texts = source_texts(story_json)?;
    let mut room = base
        .data
        .maps
        .get(WORKSHOP)
        .context("installed harbor workshop")?
        .clone();
    validate_receiver(&room)?;
    for (label, lines) in texts {
        ensure!(
            room.scripts.contains_key(&label) && room.script_text_bodies.contains_key(&label),
            "missing existing receiver text {label}"
        );
        authoring::text(
            &mut room,
            &label,
            &lines.iter().map(String::as_str).collect::<Vec<_>>(),
        );
    }
    authoring::finish(base, &mut room)?;
    let mut pack = base.clone();
    crate::insert_compiled_map_split_payloads(
        &mut pack.data.map_scripts,
        &mut pack.data.npcs,
        WORKSHOP,
        room.attributes
            .map_scripts_label
            .as_deref()
            .context("harbor script section")?,
        room.attributes
            .map_events_label
            .as_deref()
            .context("harbor event section")?,
        &room,
    )?;
    pack.data.maps.insert(WORKSHOP.into(), room);
    if installing {
        pack.report.manifests.push(MANIFEST.into());
    }
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

fn source_texts(source: &str) -> Result<BTreeMap<String, Vec<String>>> {
    let story: Value = serde_json::from_str(source).context("canonical Duluth story")?;
    let found = story["milestones"]
        .as_array()
        .context("story milestones")?
        .iter()
        .filter(|m| m["id"] == "MS10")
        .collect::<Vec<_>>();
    ensure!(found.len() == 1, "Duluth requires one MS10 milestone");
    let m = found[0];
    let requires: Vec<String> = serde_json::from_value(m["requires"].clone())?;
    let mut expected = (1..=8)
        .map(|n| format!("BADGE_{n}"))
        .collect::<BTreeSet<_>>();
    expected.insert(DEPOT.into());
    ensure!(
        m["location"] == WORKSHOP
            && requires.len() == expected.len()
            && requires.into_iter().collect::<BTreeSet<_>>() == expected
            && m["sets"] == serde_json::json!([SIGNAL]),
        "MS10 must require all eight badges and depot clearance and set only the harbor signal"
    );
    let flags: Vec<String> = serde_json::from_value(story["flags"].clone())?;
    ensure!(
        [DEPOT, SIGNAL].iter().all(|f| flags.iter().any(|s| s == f)),
        "undeclared harbor event"
    );
    let texts: BTreeMap<String, Vec<String>> =
        serde_json::from_value(m["native_texts"].clone()).context("MS10 native text bodies")?;
    ensure!(
        texts.keys().map(String::as_str).collect::<BTreeSet<_>>() == BTreeSet::from(LABELS),
        "MS10 requires the exact three existing receiver text labels"
    );
    for (label, lines) in &texts {
        ensure!(
            !lines.is_empty()
                && lines.len() % 2 == 0
                && lines.iter().all(|s| !s.is_empty()
                    && s.chars().count() <= 18
                    && s.chars()
                        .all(|c| c.is_ascii() && !c.is_control() && c != '"')),
            "MS10 text must use complete two-line native pages: {label}"
        );
    }
    Ok(texts)
}

fn validate_receiver(room: &MapModule) -> Result<()> {
    let actors = room
        .objects
        .iter()
        .filter(|a| a.object_identifier.as_deref() == Some(RECEIVER))
        .collect::<Vec<_>>();
    ensure!(
        actors.len() == 1
            && [actors[0].x, actors[0].y] == [11, 5]
            && actors[0].script == RECEIVER
            && actors[0].sprite == "SPRITE_SCIENTIST",
        "changed native harbor receiver anchor"
    );
    let c = authoring::command;
    let mut expected = vec![
        c("faceplayer", &[]),
        c("checkevent", &[SIGNAL]),
        c("iftrue", &["MN_HARBOR_RECEIVERDone"]),
    ];
    for badge in &BADGE_ENGINE_FLAGS[..8] {
        expected.extend([
            c("checkflag", &[badge]),
            c("iffalse", &["MN_HARBOR_RECEIVERDenied"]),
        ]);
    }
    expected.extend([
        c("checkevent", &[DEPOT]),
        c("iffalse", &["MN_HARBOR_RECEIVERDenied"]),
        c("opentext", &[]),
        c("writetext", &[LABELS[0]]),
        c("waitbutton", &[]),
        c("setevent", &[SIGNAL]),
        c("closetext", &[]),
        c("end", &[]),
    ]);
    ensure!(
        room.scripts.get(RECEIVER) == Some(&Value::Array(expected)),
        "changed native MS10 prerequisites, acknowledgment or result"
    );
    for (script, text) in [
        ("MN_HARBOR_RECEIVERDone", LABELS[2]),
        ("MN_HARBOR_RECEIVERDenied", LABELS[1]),
    ] {
        ensure!(
            room.scripts.get(script) == Some(&Value::Array(vec![c("jumptextfaceplayer", &[text])])),
            "changed native MS10 repeat/refusal behavior"
        );
    }
    Ok(())
}
