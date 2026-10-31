//! Merge the reviewed later story programs without replacing current content.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;

const BASE: &str = "f416ff3c899c316a63eef38f65e0546d369eafc4922981969031d95b834e3782";
const DONOR: &str = "74bfb7af0c8147c13812005770425d82a4dbf5a38655ac3e563d49708c7c11eb";
const MARKER: &str = "northstar-current-story-refresh-v2";

/// Carries the reviewed later story changes into a current Verdigrim descendant.
///
/// Only changed script labels are merged from the exact donor and common base.
/// Current containment instructions and the225-species completion threshold prevail.
/// Geometry, encounters, creatures, constants and audio come from `current`.
/// Save identities are never migrated and borrowed inputs remain unchanged.
///
/// # Errors
/// Rejects different donor/base identities, missing current scene, repeated
/// publication, removed labels, unexpected merge conflicts, changed delta counts,
/// inconsistent geometry or invalid resulting runtime data.
pub fn refresh_northstar_current_story(
    current: &CompiledGamePack,
    common: &CompiledGamePack,
    donor: &CompiledGamePack,
) -> Result<CompiledGamePack> {
    ensure!(
        common.identity()?.content_hash == BASE,
        "reviewed common story base required"
    );
    ensure!(
        donor.identity()?.content_hash == DONOR,
        "reviewed later story donor required"
    );
    crate::verify_compiled_game_pack_for_runtime(current)?;
    crate::verify_northstar_publication_geometry(current)?;
    crate::validate_compact_superior_geography(current)?;
    ensure!(
        current
            .report
            .manifests
            .iter()
            .any(|m| m == "verdigrim-intake-scene-v1")
            && !current.report.manifests.iter().any(|m| m == MARKER),
        "unrefreshed current Verdigrim descendant required"
    );
    let mut pack = current.clone();
    let mut adopted = 0;
    let mut already_current = 0;
    let mut retained = 0;
    for (name, revised) in &donor.data.maps {
        let Some(before) = common.data.maps.get(name) else {
            ensure!(
                matches!(
                    name.as_str(),
                    "MnNorthShorePieShop" | "MnNorthShoreSmokehouse"
                ),
                "unreviewed donor-only map {name}"
            );
            // Town interiors are retained by the separate current-world composition.
            continue;
        };
        ensure!(
            before
                .scripts
                .keys()
                .all(|label| revised.scripts.contains_key(label)),
            "donor removed story labels in {name}"
        );
        let mut map = pack
            .data
            .maps
            .get(name)
            .context("current story map")?
            .clone();
        let mut changed = BTreeSet::new();
        for (label, wanted) in &revised.scripts {
            let original = before.scripts.get(label);
            if original == Some(wanted) {
                continue;
            }
            let now = map.scripts.get(label);
            if retain_current(name, label, now)? {
                retained += 1;
            } else if now == Some(wanted) {
                already_current += 1;
            } else {
                ensure!(now == original, "conflicting story program {name}/{label}");
                let mut body = wanted.clone();
                if name == "MnWinonaFieldStation" && label == "MnRewardDexClaim4Text" {
                    ensure!(
                        body[0]["command"] == "text"
                            && body[0]["args"] == serde_json::json!(["\"You've caught 224\""]),
                        "reviewed donor completion wording changed"
                    );
                    body[0]["args"] = serde_json::json!(["\"You've caught 225\""]);
                }
                pack.data.map_scripts.insert(label.clone(), body.clone());
                map.scripts.insert(label.clone(), body);
                changed.insert(label.clone());
                adopted += 1;
            }
        }
        if !changed.is_empty() {
            // Refresh executable metadata without republishing terrain, actors
            // or the split event/script sections. Their assembled aliases and
            // facing overrides are not dialogue authoring inputs.
            crate::ely_league_rooms::finish(&pack, &mut map)?;
            map.script_audio_commands = crate::parse_script_audio_commands(name, &map.scripts)?;
            let original = &current.data.maps[name];
            preserve_unrevised_rows(
                &mut map.script_control_commands,
                &original.script_control_commands,
                &changed,
                |row| &row.source_script,
            );
            preserve_unrevised_rows(
                &mut map.script_text_commands,
                &original.script_text_commands,
                &changed,
                |row| &row.source_script,
            );
            map.map_event_section_commands = original.map_event_section_commands.clone();
            map.map_script_section_commands = original.map_script_section_commands.clone();
            pack.data.maps.insert(name.clone(), map);
        }
    }
    ensure!(
        matches!(
            (adopted, already_current, retained),
            (217, 2, 3) | (209, 10, 3)
        ),
        "reviewed story delta changed: {adopted}/{already_current}/{retained}"
    );
    for (name, original) in &current.data.maps {
        let revised = pack.data.maps.get(name).context("preserved current map")?;
        ensure!(
            revised.attributes == original.attributes
                && revised.blocks == original.blocks
                && revised.objects == original.objects
                && revised.events == original.events
                && revised.scenes == original.scenes,
            "story refresh changed physical map {name}"
        );
    }
    let mut preserved = pack.data.clone();
    // These are the only changed program bodies and their derived mirrors.
    preserved.maps = current.data.maps.clone();
    preserved.map_scripts = current.data.map_scripts.clone();
    ensure!(
        preserved == current.data,
        "story refresh changed unrelated game data"
    );
    ensure!(
        pack.runtime_files == current.runtime_files
            && pack.audio_manifest == current.audio_manifest
            && pack.compiled_audio == current.compiled_audio
            && pack.audio_compression == current.audio_compression,
        "story refresh changed original assets or audio"
    );
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

pub(crate) fn preserve_unrevised_rows<T: Clone>(
    refreshed: &mut Vec<T>,
    original: &[T],
    changed: &BTreeSet<String>,
    source: impl Fn(&T) -> &str,
) {
    let revised = refreshed
        .iter()
        .filter(|row| changed.contains(source(row)))
        .cloned()
        .collect::<Vec<_>>();
    *refreshed = original
        .iter()
        .filter(|row| !changed.contains(source(row)))
        .cloned()
        .chain(revised)
        .collect();
}

fn retain_current(map: &str, label: &str, current: Option<&Value>) -> Result<bool> {
    if map == "MnWinonaFieldStation" && label == "MnRewardDexTier4" {
        ensure!(
            current
                .and_then(Value::as_array)
                .is_some_and(|body| body.iter().any(|row| {
                    row["command"] == "ifless"
                        && row["args"] == serde_json::json!(["225", "MnRewardDexNeed"])
                })),
            "current completion threshold must remain225"
        );
        return Ok(true);
    }
    if map == "MnBoundaryIntake" && label == "MnArcIntakeIvyText" {
        ensure!(
            current.and_then(Value::as_array).is_some_and(|body| {
                body.first().is_some_and(|row| {
                    row["command"] == "text"
                        && row["args"] == serde_json::json!(["\"REED is guarding\""])
                }) && body
                    .iter()
                    .any(|row| row["args"] == serde_json::json!(["\"drill, outlet and\""]))
            }),
            "current separate containment controls must remain clear"
        );
        return Ok(true);
    }
    if map == "MnBoundaryIntake" && label == "MnArcIntakeFinnText" {
        ensure!(
            current.and_then(Value::as_array).is_some_and(|body| {
                body.first().is_some_and(|row| {
                    row["command"] == "text"
                        && row["args"] == serde_json::json!(["\"Stop the drill.\""])
                }) && body
                    .iter()
                    .any(|row| row["args"] == serde_json::json!(["\"bypass beside the\""]))
            }),
            "current intake control directions required"
        );
        return Ok(true);
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use std::collections::BTreeSet;

    use super::{preserve_unrevised_rows, retain_current};

    #[test]
    fn metadata_refresh_replaces_only_revised_labels_without_reordering_old_rows() {
        let original = vec![("Z", 9), ("A", 2), ("CHANGED", 1), ("A", 3)];
        let mut refreshed = vec![("A", 2), ("A", 3), ("CHANGED", 4), ("Z", 9)];
        preserve_unrevised_rows(
            &mut refreshed,
            &original,
            &BTreeSet::from(["CHANGED".into()]),
            |row| row.0,
        );
        assert_eq!(
            refreshed,
            vec![("Z", 9), ("A", 2), ("A", 3), ("CHANGED", 4)]
        );
    }

    #[test]
    fn completion_guard_requires_the_current_225_threshold() {
        let current = json!([{"command":"ifless","args":["225","MnRewardDexNeed"]}]);
        assert!(
            retain_current("MnWinonaFieldStation", "MnRewardDexTier4", Some(&current))
                .expect("reviewed225 threshold")
        );
        let stale = json!([{"command":"ifless","args":["224","MnRewardDexNeed"]}]);
        assert!(retain_current("MnWinonaFieldStation", "MnRewardDexTier4", Some(&stale)).is_err());
    }

    #[test]
    fn intake_resolution_rejects_the_obsolete_gauge_conversation() {
        let current = json!([
            {"command":"text","args":["\"Stop the drill.\""]},
            {"command":"line","args":["\"bypass beside the\""]}
        ]);
        assert!(
            retain_current("MnBoundaryIntake", "MnArcIntakeFinnText", Some(&current))
                .expect("current containment directions")
        );
        let stale = json!([{"command":"text","args":["\"FINN: This gauge\""]}]);
        assert!(retain_current("MnBoundaryIntake", "MnArcIntakeFinnText", Some(&stale)).is_err());
        assert!(retain_current("MnBoundaryIntake", "MnArcIntakeFinnText", None).is_err());
    }

    #[test]
    fn intake_ivy_retains_three_separate_controls() {
        let current = json!([
            {"command":"text","args":["\"REED is guarding\""]},
            {"command":"para","args":["\"drill, outlet and\""]}
        ]);
        assert!(
            retain_current("MnBoundaryIntake", "MnArcIntakeIvyText", Some(&current))
                .expect("current separate controls")
        );
        let premature = json!([{"command":"text","args":["\"Now we can get\""]}]);
        assert!(
            retain_current("MnBoundaryIntake", "MnArcIntakeIvyText", Some(&premature)).is_err()
        );
    }
}
