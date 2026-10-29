//! Keep regional research membership and its final award aligned with the roster.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;
use crate::northstar_rival_arc::{map, publish};

const PREFIX: &str = "MN_DEX_MEMBER_";
const MANIFEST: &str = "minnesota-dex-roster-sync-v1";

/// Adds newly compiled active species to regional research and updates only the
/// final caught-count threshold and its displayed number. Existing award flags,
/// starter wrappers and all unrelated content remain. The obsolete 50-species
/// EXP Share award becomes ten Ultra Balls using its existing claim flag.
///
/// # Errors
/// Rejects missing species, retired existing members, duplicate installation,
/// malformed roster/curation, or an ambiguous final award program/text.
pub fn build_minnesota_dex_roster_sync(
    base: &CompiledGamePack,
    roster: &str,
    curation: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "Dex sync already installed"
    );
    let roster: Value = serde_json::from_str(roster)?;
    let curation: Value = serde_json::from_str(curation)?;
    let retired: BTreeSet<_> = curation["retired_families"]
        .as_array()
        .context("retired families")?
        .iter()
        .filter_map(|f| f["id"].as_str())
        .collect();
    let mut active = BTreeSet::new();
    for family in roster["families"].as_array().context("roster families")? {
        if retired.contains(family["id"].as_str().context("family ID")?) {
            continue;
        }
        for member in family["members"].as_array().context("family members")? {
            let name = member["name"]
                .as_str()
                .context("species name")?
                .to_ascii_uppercase();
            ensure!(
                base.data.pokemon.contains_key(&name),
                "compile active species first: {name}"
            );
            ensure!(active.insert(name), "duplicate active species");
        }
    }
    let previous: BTreeSet<_> = base
        .data
        .story_event_script_constants
        .global
        .keys()
        .filter_map(|key| key.strip_prefix(PREFIX).map(str::to_owned))
        .collect();
    ensure!(
        !previous.is_empty() && previous.is_subset(&active) && previous.len() < active.len(),
        "requires an expanded, compiled roster without retiring existing members"
    );
    let old_count = previous.len().to_string();
    let new_count = active.len().to_string();
    let mut pack = base.clone();
    for name in active.difference(&previous) {
        pack.data
            .story_event_script_constants
            .global
            .insert(format!("{PREFIX}{name}"), 1);
    }
    let mut room = map(&pack, "MnWinonaFieldStation")?;
    let mut claim = None;
    let mut guards = 0;
    for (label, program) in &mut room.scripts {
        if label != "MnRewardDex" && !label.starts_with("MnRewardDexTier") {
            continue;
        }
        let commands = program.as_array_mut().context("research program")?;
        let mut changed = false;
        for command in commands.iter_mut() {
            if command["command"] == "ifless"
                && command["args"][0] == old_count
                && command["args"][1] == "MnRewardDexNeed"
            {
                command["args"][0] = new_count.clone().into();
                guards += 1;
                changed = true;
            }
        }
        if changed {
            claim = commands
                .iter()
                .find(|c| c["command"] == "sjump")
                .and_then(|c| c["args"][0].as_str())
                .map(str::to_owned);
        }
    }
    ensure!(guards == 1, "exactly one final reward threshold required");
    let claim = claim.context("final award branch")?;
    let text = room.scripts[&claim]
        .as_array()
        .context("final claim program")?
        .iter()
        .find(|c| c["command"] == "writetext")
        .and_then(|c| c["args"][0].as_str())
        .context("final claim text")?
        .to_owned();
    let mut replacements = 0;
    for command in room
        .scripts
        .get_mut(&text)
        .context("claim text body")?
        .as_array_mut()
        .context("claim text instructions")?
    {
        for operand in command["args"].as_array_mut().context("text operands")? {
            if let Some(row) = operand.as_str().filter(|row| row.contains(&old_count)) {
                let updated = row.replace(&old_count, &new_count);
                ensure!(
                    updated.trim_matches('"').chars().count() <= 18,
                    "final reward text exceeds native width"
                );
                *operand = updated.into();
                replacements += 1;
            }
        }
    }
    ensure!(
        replacements == 1,
        "exactly one displayed final count required"
    );
    let mut fifty_text = None;
    for (label, program) in &mut room.scripts {
        if !label.starts_with("MnRewardDexClaim") {
            continue;
        }
        let Some(commands) = program.as_array_mut() else {
            continue;
        };
        let mut replaced = false;
        for command in commands.iter_mut() {
            if command["command"] == "verbosegiveitem" && command["args"][0] == "EXP_SHARE" {
                command["args"][0] = "ULTRA_BALL".into();
                command["args"][1] = "10".into();
                replaced = true;
            }
        }
        if replaced {
            ensure!(fifty_text.is_none(), "ambiguous obsolete research reward");
            fifty_text = commands
                .iter()
                .find(|c| c["command"] == "writetext")
                .and_then(|c| c["args"][0].as_str())
                .map(str::to_owned);
        }
    }
    if let Some(label) = fifty_text {
        authoring::text(
            &mut room,
            &label,
            &[
                "You've caught 50 Minnesota species! Your field notes earned this research award.",
                "ULTRA BALLs help catch elusive Pokemon. Keep exploring!",
                "Each award is given once.",
            ],
        );
        authoring::text(
            &mut room,
            "MnRewardDexNeedText",
            &[
                "I award research supplies for 10, 25, 50, 100 and all Minnesota species caught.",
                "Seen entries don't count. Partners may stay in the PC.",
                "All healthy party Pokemon already share battle experience. No item is needed.",
            ],
        );
    }
    publish(&mut pack, room)?;
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
