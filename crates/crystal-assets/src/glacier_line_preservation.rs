//! Reject any service overlay change outside the five conductor programs.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::glacier_line_service::{MANIFEST, PREFIX, Station};

pub(crate) fn validate(
    base: &CompiledGamePack,
    output: &CompiledGamePack,
    stations: &[Station],
    added_flags: &[String],
) -> Result<()> {
    ensure!(
        base.runtime_files == output.runtime_files
            && base.audio_manifest == output.audio_manifest
            && base.compiled_audio == output.compiled_audio
            && base.audio_compression == output.audio_compression,
        "native train art, music, SFX and all unrelated audio must stay exact"
    );
    let mut data = output.data.clone();
    let mut owned_raw = BTreeSet::new();
    for stop in stations {
        let old = &base.data.maps[&stop.map];
        let new = &data.maps[&stop.map];
        let mut attributes = old.attributes.clone();
        attributes.fishing_group = Some("FISHGROUP_NONE".into());
        ensure!(
            attributes == new.attributes
                && old.blocks == new.blocks
                && old.objects == new.objects
                && old.events == new.events,
            "station terrain, people and doors must stay exact; only unused fishing metadata may change"
        );
        ensure!(
            new.scripts[&format!("{}Original", stop.prefix())]
                == old.scripts[&stop.conductor_script],
            "original conductor speech must survive"
        );
        for (label, commands) in &old.scripts {
            if label != &stop.conductor_script {
                ensure!(
                    new.scripts.get(label) == Some(commands),
                    "old station story changed: {label}"
                );
            }
        }
        for (label, text) in &old.script_text_bodies {
            ensure!(
                new.script_text_bodies.get(label) == Some(text),
                "old native text changed: {label}"
            );
        }
        ensure!(
            new.map_script_section_commands
                .starts_with(&old.map_script_section_commands),
            "old story/object callbacks must remain in order"
        );
        owned_raw.insert(stop.conductor_script.clone());
        owned_raw.insert(
            old.attributes
                .map_scripts_label
                .clone()
                .context("native script section")?,
        );
        data.maps.insert(stop.map.clone(), old.clone());
        data.map_attributes
            .insert(stop.map.clone(), old.attributes.clone());
    }
    let extra: Vec<_> = data
        .map_scripts
        .keys()
        .filter(|k| !base.data.map_scripts.contains_key(*k))
        .cloned()
        .collect();
    for key in extra {
        ensure!(key.starts_with(PREFIX), "unowned new raw program: {key}");
        data.map_scripts.remove(&key);
    }
    for key in owned_raw {
        let old = base
            .data
            .map_scripts
            .get(&key)
            .context("old conductor/section payload")?;
        data.map_scripts.insert(key, old.clone());
    }
    ensure!(
        added_flags.len() == 3,
        "only three missing visit events may be allocated"
    );
    for key in added_flags {
        ensure!(
            !base
                .data
                .story_event_script_constants
                .global
                .contains_key(key),
            "old flag replacement"
        );
        data.story_event_script_constants
            .global
            .remove(key)
            .context("new visit event missing")?;
    }
    ensure!(
        data == base.data,
        "service changed unrelated maps, native mirrors, identities, catalogs or original flags"
    );
    let mut report = output.report.clone();
    report.manifests.retain(|m| m != MANIFEST);
    ensure!(
        report == base.report,
        "unrelated publication report changed"
    );
    Ok(())
}
