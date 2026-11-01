//! Reconcile a reviewed story/assets release with independently reviewed terrain.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, bail, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::CompiledGamePack;

const MANIFEST: &str = "minnesota-current-release-composition-v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Contract {
    schema_version: u8,
    current_content_hash: String,
    common_base_content_hash: String,
    released_content_hash: String,
}

/// Applies the reviewed release delta to the current accepted regional world.
///
/// Requires three exact content identities. Disjoint changes compose; different
/// edits to the same value fail with their source path. Current outdoor terrain,
/// attributes and spatial mirrors remain authoritative. Manifest additions are
/// combined without deleting an existing marker. The resulting runtime and
/// regional geography are validated before returning. No input is modified and
/// no save identity is migrated.
///
/// # Errors
/// Rejects invalid packs/contracts, stale identities, overlapping changes,
/// removed manifest markers, repeated application or invalid resulting content.
/// Compiled command caches merge by source program. Different revisions of the
/// same program still conflict, including when their individual indices differ.
pub fn compose_current_minnesota_release(
    current: &CompiledGamePack,
    before: &CompiledGamePack,
    released: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    let contract: Contract = serde_json::from_str(source)?;
    ensure!(contract.schema_version == 1, "release composition schema");
    for (pack, hash) in [
        (current, &contract.current_content_hash),
        (before, &contract.common_base_content_hash),
        (released, &contract.released_content_hash),
    ] {
        crate::verify_compiled_game_pack_for_runtime(pack)?;
        let identity = crate::derive_compiled_game_pack_identity_from_manifest(
            pack.format_version,
            &pack.data,
            &pack.audio_manifest,
            &pack.runtime_files,
            &pack.report,
        )?;
        ensure!(
            &identity.content_hash == hash,
            "release composition input drift"
        );
    }
    crate::verify_northstar_publication_geometry(current)?;
    crate::validate_compact_superior_geography(current)?;
    ensure!(
        !current.report.manifests.iter().any(|id| id == MANIFEST),
        "current release already composed"
    );
    ensure!(
        current.format_version == before.format_version
            && current.format_version == released.format_version,
        "pack format changes need a separate migration"
    );
    // Binary art/audio stays in byte maps. Expanding it into three JSON byte
    // arrays would multiply the memory needed for this offline composition.
    let mut installed = serde_json::json!({"data": &current.data, "report": &current.report});
    let mut ancestor = serde_json::json!({"data": &before.data, "report": &before.report});
    let mut donor = serde_json::json!({"data": &released.data, "report": &released.report});
    preserve_outdoor_geometry(&installed, &mut ancestor, &mut donor)?;
    let manifests = merge_manifests(
        &current.report.manifests,
        &before.report.manifests,
        &released.report.manifests,
    )?;
    for pack in [&mut installed, &mut ancestor, &mut donor] {
        pack["report"]["manifests"] = serde_json::json!([]);
    }
    let mut merged = merge_value(Some(&installed), Some(&ancestor), Some(&donor), "pack")?
        .context("composition cannot remove the pack")?;
    let mut result = current.clone();
    result.data =
        serde_json::from_value(merged["data"].take()).context("typed composed game data")?;
    result.report =
        serde_json::from_value(merged["report"].take()).context("typed composed report")?;
    merge_byte_entries(
        &mut result.runtime_files,
        &before.runtime_files,
        &released.runtime_files,
        "runtime_files",
    )?;
    merge_byte_entries(
        &mut result.compiled_audio,
        &before.compiled_audio,
        &released.compiled_audio,
        "compiled_audio",
    )?;
    if released.audio_manifest != before.audio_manifest {
        ensure!(
            current.audio_manifest == before.audio_manifest
                || current.audio_manifest == released.audio_manifest,
            "overlapping current/released audio manifest"
        );
        result.audio_manifest.clone_from(&released.audio_manifest);
    }
    if released.audio_compression != before.audio_compression {
        ensure!(
            current.audio_compression == before.audio_compression
                || current.audio_compression == released.audio_compression,
            "overlapping current/released audio encoding"
        );
        result
            .audio_compression
            .clone_from(&released.audio_compression);
    }
    result.report.manifests = manifests;
    if !result.report.manifests.iter().any(|id| id == MANIFEST) {
        result.report.manifests.push(MANIFEST.into());
    }
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    crate::verify_northstar_publication_geometry(&result)?;
    crate::validate_compact_superior_geography(&result)?;
    for (name, map) in &current.data.maps {
        if matches!(
            map.attributes.environment.as_deref(),
            Some("ROUTE" | "TOWN")
        ) {
            let after = result
                .data
                .maps
                .get(name)
                .context("accepted outdoor map removed")?;
            ensure!(
                map.attributes == after.attributes && map.blocks == after.blocks,
                "accepted outdoor geometry changed: {name}"
            );
        }
    }
    ensure!(
        result.data.pokemon.contains_key("VERDIGRIM"),
        "released Verdigrim assets missing"
    );
    Ok(result)
}

// The accepted regional branch owns exterior placement and terrain. Story text,
// actors and event programs still use strict three-way merging. Explicit later
// geography repairs (such as the reviewed League turnout) remain separate stages.
fn preserve_outdoor_geometry(current: &Value, before: &mut Value, after: &mut Value) -> Result<()> {
    let maps = current["data"]["maps"]
        .as_object()
        .context("current map modules")?;
    for (name, map) in maps {
        if !matches!(
            map["attributes"]["environment"].as_str(),
            Some("ROUTE" | "TOWN")
        ) {
            continue;
        }
        for branch in [&mut *before, &mut *after] {
            if branch["data"]["maps"].get(name).is_none() {
                continue;
            }
            for field in ["attributes", "blocks"] {
                branch["data"]["maps"][name][field] = map[field].clone();
            }
            if let Some(value) = current["data"]["map_attributes"].get(name) {
                branch["data"]["map_attributes"][name] = value.clone();
            }
            if let Some(label) = map["attributes"]["blocks_label"].as_str()
                && let Some(value) = current["data"]["map_blocks"].get(label)
            {
                branch["data"]["map_blocks"][label] = value.clone();
            }
            if let Some(constant) = map["attributes"]["map_constant"].as_str() {
                for table in ["runtime_map_metadata", "map_dimensions"] {
                    if let Some(value) = current["data"][table].get(constant) {
                        branch["data"][table][constant] = value.clone();
                    }
                }
            }
        }
    }
    for (name, map) in after["data"]["maps"]
        .as_object()
        .context("released map modules")?
    {
        ensure!(
            !matches!(
                map["attributes"]["environment"].as_str(),
                Some("ROUTE" | "TOWN")
            ) || maps.contains_key(name),
            "release adds unreviewed outdoor geography: {name}"
        );
    }
    Ok(())
}

fn merge_byte_entries(
    target: &mut BTreeMap<String, Vec<u8>>,
    before: &BTreeMap<String, Vec<u8>>,
    after: &BTreeMap<String, Vec<u8>>,
    table: &str,
) -> Result<()> {
    let keys: BTreeSet<_> = before.keys().chain(after.keys()).collect();
    for key in keys {
        let old = before.get(key);
        let new = after.get(key);
        if old == new {
            continue;
        }
        ensure!(
            target.get(key) == old || target.get(key) == new,
            "overlapping current/released content at pack.{table}.{key}"
        );
        if let Some(bytes) = new {
            target.insert(key.clone(), bytes.clone());
        } else {
            target.remove(key);
        }
    }
    Ok(())
}

fn merge_manifests(current: &[String], before: &[String], after: &[String]) -> Result<Vec<String>> {
    let mut result = current.to_vec();
    let mut seen: BTreeSet<_> = current.iter().cloned().collect();
    ensure!(
        before.iter().all(|id| seen.contains(id)),
        "current removed a base manifest"
    );
    let released: BTreeSet<_> = after.iter().map(String::as_str).collect();
    ensure!(
        before.iter().all(|id| released.contains(id.as_str())),
        "release removed a base manifest"
    );
    for id in after {
        if seen.insert(id.clone()) {
            result.push(id.clone());
        }
    }
    Ok(result)
}

fn merge_value(
    current: Option<&Value>,
    before: Option<&Value>,
    after: Option<&Value>,
    path: &str,
) -> Result<Option<Value>> {
    if after == before {
        return Ok(current.cloned());
    }
    if current == before || current == after {
        return Ok(after.cloned());
    }
    // Command rows are derived independently for each source program.
    // Two branches can revise different programs in one room without either
    // branch owning the other program's cached rows.
    if is_command_cache(path)
        && let (Some(Value::Array(current)), Some(Value::Array(before)), Some(Value::Array(after))) =
            (current, before, after)
    {
        let current = command_rows_by_script(current)?;
        let before = command_rows_by_script(before)?;
        let after = command_rows_by_script(after)?;
        let merged = merge_value(
            Some(&current),
            Some(&before),
            Some(&after),
            &format!("{path}.programs"),
        )?
        .context("command cache program table cannot disappear")?;
        let mut rows = Vec::new();
        for commands in merged
            .as_object()
            .context("merged command cache program table")?
            .values()
        {
            rows.extend(
                commands
                    .as_array()
                    .context("merged command cache rows")?
                    .iter()
                    .cloned(),
            );
        }
        return Ok(Some(Value::Array(rows)));
    }
    if let (Some(Value::Object(current)), Some(Value::Object(before)), Some(Value::Object(after))) =
        (current, before, after)
    {
        let keys: BTreeSet<_> = current
            .keys()
            .chain(before.keys())
            .chain(after.keys())
            .collect();
        let mut result = serde_json::Map::new();
        for key in keys {
            if let Some(value) = merge_value(
                current.get(key),
                before.get(key),
                after.get(key),
                &format!("{path}.{key}"),
            )? {
                result.insert(key.clone(), value);
            }
        }
        return Ok(Some(Value::Object(result)));
    }
    bail!("overlapping current/released content at {path}")
}

fn is_command_cache(path: &str) -> bool {
    path.starts_with("pack.data.maps.")
        && matches!(
            path.rsplit('.').next(),
            Some(
                "script_text_commands"
                    | "script_control_commands"
                    | "script_variable_commands"
                    | "script_flag_commands"
                    | "script_item_grants"
                    | "script_item_checks"
                    | "script_item_takes"
                    | "gift_pokemon_scripts"
                    | "script_runtime_commands"
                    | "script_object_commands"
                    | "script_audio_commands"
                    | "script_economy_commands"
                    | "script_map_commands"
                    | "script_scene_commands"
                    | "script_shop_commands"
                    | "script_phone_commands"
                    | "script_swarm_commands"
                    | "script_block_changes"
            )
        )
}

fn command_rows_by_script(rows: &[Value]) -> Result<Value> {
    let mut programs = BTreeMap::<String, Vec<Value>>::new();
    for row in rows {
        let source = row["source_script"]
            .as_str()
            .context("cached command requires its source program")?;
        programs.entry(source.into()).or_default().push(row.clone());
    }
    Ok(serde_json::to_value(programs)?)
}

#[cfg(test)]
mod tests {
    use super::{merge_byte_entries, merge_manifests, merge_value, preserve_outdoor_geometry};
    use serde_json::json;
    use std::collections::BTreeMap;

    #[test]
    fn independent_program_text_metadata_composes_but_same_program_conflicts() {
        let before = json!([
            {"source_script":"A", "command":"writetext", "text_label":"OldA", "command_index":0},
            {"source_script":"B", "command":"writetext", "text_label":"OldB", "command_index":0}
        ]);
        let mut current = before.clone();
        current[0]["text_label"] = json!("CurrentA");
        let mut after = before.clone();
        after[1]["text_label"] = json!("StoryB");
        let merged = merge_value(
            Some(&current),
            Some(&before),
            Some(&after),
            "pack.data.maps.MnDuluth.script_text_commands",
        )
        .expect("different source programs compose")
        .expect("text commands");
        assert_eq!(merged[0], current[0]);
        assert_eq!(merged[1], after[1]);
        after[0]["text_label"] = json!("ConflictingA");
        let error = merge_value(
            Some(&current),
            Some(&before),
            Some(&after),
            "pack.data.maps.MnDuluth.script_text_commands",
        )
        .expect_err("same program must not silently choose a branch");
        assert!(error.to_string().contains("programs.A"));
    }

    #[test]
    fn independent_aide_and_dex_caches_compose_without_merging_one_program() {
        for field in [
            "script_control_commands",
            "script_variable_commands",
            "script_flag_commands",
            "gift_pokemon_scripts",
        ] {
            let before = json!([
                {"source_script":"Aide", "command_index":0, "value":"late"},
                {"source_script":"Dex", "command_index":0, "value":224}
            ]);
            let current = json!([
                {"source_script":"Aide", "command_index":2, "value":"early"},
                {"source_script":"Dex", "command_index":0, "value":224},
                {"source_script":"NewPartner", "command_index":0, "value":"free"}
            ]);
            let released = json!([
                {"source_script":"Aide", "command_index":0, "value":"late"},
                {"source_script":"Dex", "command_index":0, "value":225}
            ]);
            let path = format!("pack.data.maps.MnWinonaFieldStation.{field}");
            let merged = merge_value(Some(&current), Some(&before), Some(&released), &path)
                .expect("different native programs compose")
                .expect("cache");
            assert_eq!(
                merged,
                json!([
                    {"source_script":"Aide", "command_index":2, "value":"early"},
                    {"source_script":"Dex", "command_index":0, "value":225},
                    {"source_script":"NewPartner", "command_index":0, "value":"free"}
                ])
            );
            let mut conflicting = released.clone();
            conflicting[0]["value"] = json!("another change to Aide");
            assert!(merge_value(Some(&current), Some(&before), Some(&conflicting), &path).is_err());
            assert!(
                merge_value(
                    Some(&current),
                    Some(&before),
                    Some(&released),
                    "pack.data.maps.MnWinonaFieldStation.objects"
                )
                .is_err(),
                "ordinary arrays remain atomic"
            );
        }
    }

    #[test]
    fn independent_terrain_and_story_compose_without_restoring_legacy_placement() {
        let before =
            json!({"room":{"floor":[1,1],"placement":-1,"text":"old"},"art":{"original":[7]}});
        let current = json!({"room":{"floor":[1,2],"placement":0,"text":"old"},"art":{"original":[7],"stairs":[8]}});
        let after = json!({"room":{"floor":[1,1],"placement":-1,"text":"new"},"art":{"original":[7],"wolf":[9]}});
        let result = merge_value(Some(&current), Some(&before), Some(&after), "pack")
            .expect("disjoint changes")
            .expect("pack");
        assert_eq!(
            result,
            json!({"room":{"floor":[1,2],"placement":0,"text":"new"},"art":{"original":[7],"stairs":[8],"wolf":[9]}})
        );
    }

    #[test]
    fn released_legacy_connections_do_not_override_accepted_world_geometry() {
        let before = json!({"data":{"maps":{"town":{"attributes":{"environment":"TOWN","offset":0},"blocks":[1],"text":"old"}}}});
        let current = json!({"data":{"maps":{"town":{"attributes":{"environment":"TOWN","offset":0},"blocks":[2],"text":"old"}}}});
        let after = json!({"data":{"maps":{"town":{"attributes":{"environment":"TOWN","offset":-1},"blocks":[3],"text":"new"}}}});
        let mut ancestor = before.clone();
        let mut donor = after.clone();
        preserve_outdoor_geometry(&current, &mut ancestor, &mut donor)
            .expect("accepted spatial projection");
        let merged = merge_value(Some(&current), Some(&ancestor), Some(&donor), "pack")
            .expect("story only")
            .expect("pack");
        assert_eq!(merged["data"]["maps"]["town"]["attributes"]["offset"], 0);
        assert_eq!(merged["data"]["maps"]["town"]["blocks"], json!([2]));
        assert_eq!(merged["data"]["maps"]["town"]["text"], "new");
        assert_eq!(before["data"]["maps"]["town"]["blocks"], json!([1]));
    }

    #[test]
    fn byte_maps_preserve_current_art_and_reject_overlapping_audio() {
        let before = BTreeMap::from([("original".into(), vec![1])]);
        let after = BTreeMap::from([("original".into(), vec![1]), ("wolf".into(), vec![2])]);
        let mut current =
            BTreeMap::from([("original".into(), vec![1]), ("stairs".into(), vec![3])]);
        merge_byte_entries(&mut current, &before, &after, "runtime_files")
            .expect("disjoint original art");
        assert_eq!(
            current,
            BTreeMap::from([
                ("original".into(), vec![1]),
                ("stairs".into(), vec![3]),
                ("wolf".into(), vec![2])
            ])
        );
        let mut conflicting = BTreeMap::from([("original".into(), vec![4])]);
        let changed = BTreeMap::from([("original".into(), vec![5])]);
        let error = merge_byte_entries(&mut conflicting, &before, &changed, "compiled_audio")
            .expect_err("changed current program");
        assert!(error.to_string().contains("pack.compiled_audio.original"));
    }

    #[test]
    fn overlapping_art_and_removed_manifest_are_rejected() {
        let error = merge_value(
            Some(&json!({"art":[2]})),
            Some(&json!({"art":[1]})),
            Some(&json!({"art":[3]})),
            "pack",
        )
        .expect_err("different edits must fail");
        assert!(error.to_string().contains("pack.art"));
        assert!(merge_manifests(&["base".into()], &["base".into()], &[]).is_err());
        assert_eq!(
            merge_manifests(
                &["base".into(), "terrain".into()],
                &["base".into()],
                &["base".into(), "story".into()]
            )
            .expect("additions"),
            vec!["base", "terrain", "story"]
        );
    }
}
