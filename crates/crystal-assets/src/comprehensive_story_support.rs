//! Merge the reviewed supporting conversations without republishing current maps.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::comprehensive_current_world::adopt_entry;
use crate::northstar_story_refresh::preserve_unrevised_rows;

const MARKER: &str = "comprehensive-story-support-v27-integration-v1";
const TRANSFER: &str = "MnIceTransfer";

/// Integrates the reviewed v27 staff, mine, cousin and captive conversations.
///
/// Only 134 changed programs and their text caches are adopted from the exact
/// v23/v27 pair. Tess's existing rescue flag gains the reviewed departure
/// binding in its four mirrors. Current terrain, actors' positions and art,
/// controls, rewards, 225-species research and audio remain intact. Existing
/// saves are never rewritten; the output receives a fresh content identity.
///
/// # Errors
/// Rejects different inputs, stale program/actor preimages, removed labels,
/// unreviewed metadata changes, changed delta counts or invalid resulting data.
pub fn integrate_comprehensive_story_support(
    current: &CompiledGamePack,
    common: &CompiledGamePack,
    latest: &CompiledGamePack,
) -> Result<CompiledGamePack> {
    for (pack, hash) in [
        (
            current,
            "4a56ea4c8e94bfb9c2c18c8350775d22776e1b7bf7d1fdb3df533c12329443a2",
        ),
        (
            common,
            "74bfb7af0c8147c13812005770425d82a4dbf5a38655ac3e563d49708c7c11eb",
        ),
        (
            latest,
            "ccf84221d047e958736a4ca5cbecc917713a4496e03ffe89527e72e2d2593914",
        ),
    ] {
        ensure!(
            pack.identity()?.content_hash == hash,
            "exact supporting-story inputs required"
        );
        crate::verify_compiled_game_pack_for_runtime(pack)?;
    }
    ensure!(
        !current.report.manifests.iter().any(|m| m == MARKER),
        "fresh support integration required"
    );
    let mut pack = current.clone();
    let mut adopted = 0;
    let mut maps = 0;
    for (name, revised) in &latest.data.maps {
        let Some(before) = common.data.maps.get(name) else {
            continue;
        };
        let changed = revised
            .scripts
            .iter()
            .filter_map(|(label, program)| {
                (before.scripts.get(label) != Some(program)).then_some(label.clone())
            })
            .collect::<BTreeSet<_>>();
        if changed.is_empty() {
            continue;
        }
        ensure!(
            before
                .scripts
                .keys()
                .all(|k| revised.scripts.contains_key(k)),
            "removed supporting label in {name}"
        );
        let mut checked = revised.clone();
        checked.scripts.clone_from(&before.scripts);
        checked
            .script_text_bodies
            .clone_from(&before.script_text_bodies);
        checked
            .script_text_commands
            .clone_from(&before.script_text_commands);
        if name == TRANSFER {
            checked.objects.clone_from(&before.objects);
            checked
                .map_event_section_commands
                .clone_from(&before.map_event_section_commands);
        }
        ensure!(
            checked == *before,
            "unreviewed supporting metadata changed in {name}"
        );
        let map = pack
            .data
            .maps
            .get_mut(name)
            .context("current supporting map")?;
        for label in &changed {
            ensure!(
                map.scripts.get(label) == before.scripts.get(label),
                "conflicting supporting program {name}/{label}"
            );
            let program = revised
                .scripts
                .get(label)
                .context("reviewed supporting program")?
                .clone();
            ensure!(
                current.data.map_scripts.get(label) == common.data.map_scripts.get(label)
                    && latest.data.map_scripts.get(label) == Some(&program),
                "unreviewed global supporting program {label}"
            );
            pack.data.map_scripts.insert(label.clone(), program.clone());
            map.scripts.insert(label.clone(), program);
            if let Some(body) = revised.script_text_bodies.get(label) {
                map.script_text_bodies.insert(label.clone(), body.clone());
            } else {
                map.script_text_bodies.remove(label);
            }
            adopted += 1;
        }
        let mut text_commands = revised.script_text_commands.clone();
        preserve_unrevised_rows(
            &mut text_commands,
            &map.script_text_commands,
            &changed,
            |row| &row.source_script,
        );
        map.script_text_commands = text_commands;
        maps += 1;
    }
    ensure!(
        (adopted, maps) == (134, 19),
        "reviewed supporting delta changed: {adopted}/{maps}"
    );
    let before = common
        .data
        .maps
        .get(TRANSFER)
        .context("common transfer room")?;
    let revised = latest
        .data
        .maps
        .get(TRANSFER)
        .context("reviewed transfer room")?;
    let map = pack
        .data
        .maps
        .get_mut(TRANSFER)
        .context("current transfer room")?;
    ensure!(
        map.objects == before.objects,
        "current captive actor preimage changed"
    );
    let mut expected = before.objects.clone();
    let tess = expected
        .iter_mut()
        .find(|actor| actor.object_identifier.as_deref() == Some("MN_ICE_HQ_TESS"))
        .context("existing captive Tess")?;
    ensure!(
        tess.event_flag == "-1" && (tess.x, tess.y) == (25, 5),
        "reviewed captive preimage required"
    );
    tess.event_flag = "EVENT_MN_ICE_CAPTIVE_RESCUED".into();
    ensure!(
        revised.objects == expected,
        "unreviewed captive actor delta"
    );
    map.objects = expected;
    ensure!(
        map.map_event_section_commands == before.map_event_section_commands,
        "current captive event mirror changed"
    );
    map.map_event_section_commands
        .clone_from(&revised.map_event_section_commands);
    adopt_entry(
        &mut pack.data.npcs,
        &common.data.npcs,
        &latest.data.npcs,
        TRANSFER,
    )?;
    adopt_entry(
        &mut pack.data.map_scripts,
        &common.data.map_scripts,
        &latest.data.map_scripts,
        "MnIceTransfer_MapEvents",
    )?;
    pack.report.manifests.push(MARKER.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    crate::verify_northstar_publication_geometry(&pack)?;
    crate::validate_compact_superior_geography(&pack)?;
    Ok(pack)
}
