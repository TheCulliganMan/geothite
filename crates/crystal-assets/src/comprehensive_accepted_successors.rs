//! Reconcile the reviewed world successors with the complete regional campaign.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, bail, ensure};
use serde_json::{Value, json};

use crate::CompiledGamePack;

const MARKER: &str = "comprehensive-accepted-successors-v1";

/// Integrates the reviewed cave, station, winter-town and habitat successors.
///
/// Preserves campaign programs, event identities, fossil services and every
/// audio/art byte. The reviewed larger gallery replaces its older layout;
/// existing gallery event and runtime map identities remain authoritative.
/// Inputs and existing saves are never modified or migrated.
///
/// # Errors
/// Rejects unreviewed input identities, asset changes, duplicate installation,
/// incompatible script edits, exhausted event/trainer identities or invalid
/// resulting runtime data and geography.
pub fn integrate_comprehensive_accepted_successors(
    current: &CompiledGamePack,
    common: &CompiledGamePack,
    latest: &CompiledGamePack,
) -> Result<CompiledGamePack> {
    for (pack, expected) in [
        (
            current,
            "c6feb99daa624a3772c5e76cfc8878b0760d4ce9b498b47eacf145b5156f1d22",
        ),
        (
            common,
            "b828c463da5d05885a90a49b4f822fee5e6d7c2d5f997443143d22bf376face9",
        ),
        (
            latest,
            "6b28ef1aab2c5e843d6321d4f2790f4f1b578a442ce9406d1a9a85dd657462fc",
        ),
    ] {
        let actual = crate::derive_compiled_game_pack_identity_from_manifest(
            pack.format_version,
            &pack.data,
            &pack.audio_manifest,
            &pack.runtime_files,
            &pack.report,
        )?;
        ensure!(
            actual.content_hash == expected,
            "exact reviewed successor inputs required"
        );
        crate::verify_compiled_game_pack_for_runtime(pack)?;
    }
    ensure!(
        !current.report.manifests.iter().any(|m| m == MARKER),
        "successors already installed"
    );
    ensure!(
        common.runtime_files == latest.runtime_files
            && common.compiled_audio == latest.compiled_audio
            && common.audio_manifest == latest.audio_manifest,
        "successors must retain native art/audio"
    );
    let mut live = serde_json::to_value(&current.data)?;
    let mut before = serde_json::to_value(&common.data)?;
    let mut after = serde_json::to_value(&latest.data)?;
    normalize_event_identities(&live, &mut after)?;
    normalize_skier_identity(current, latest, &mut after)?;
    let order = catalog_objects(&mut live)?;
    catalog_objects(&mut before)?;
    catalog_objects(&mut after)?;
    let mut merged = merge(Some(&before), Some(&live), Some(&after), "data")?
        .context("regional data cannot disappear")?;
    restore_catalogs(&mut merged, &order)?;
    let mut pack = current.clone();
    pack.data = serde_json::from_value(merged).context("typed successor data")?;
    for marker in &latest.report.manifests {
        if !common.report.manifests.contains(marker) && !pack.report.manifests.contains(marker) {
            pack.report.manifests.push(marker.clone());
        }
    }
    pack.report.manifests.push(MARKER.into());
    pack.report.maps = pack.data.maps.len();
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

fn normalize_event_identities(current: &Value, donor: &mut Value) -> Result<()> {
    let existing = current["story_event_script_constants"]["global"]
        .as_object()
        .context("current event constants")?;
    let mut next = existing
        .iter()
        .filter(|(name, _)| name.starts_with("EVENT_"))
        .map(|(_, value)| value.as_i64().context("event identity"))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .max()
        .context("native events")?;
    let globals = donor["story_event_script_constants"]["global"]
        .as_object_mut()
        .context("donor event constants")?;
    for (name, value) in globals
        .iter_mut()
        .filter(|(name, _)| name.starts_with("EVENT_"))
    {
        if let Some(old) = existing.get(name) {
            *value = old.clone();
        } else {
            next = next.checked_add(1).context("event identity overflow")?;
            u16::try_from(next).context("event identity space exhausted")?;
            *value = json!(next);
        }
    }
    Ok(())
}

fn normalize_skier_identity(
    current: &CompiledGamePack,
    donor: &CompiledGamePack,
    after: &mut Value,
) -> Result<()> {
    let name = "MN_EXTRA_FALLS_SKIER";
    ensure!(
        !current.data.trainers.trainers.contains_key(name),
        "skier identity occupied"
    );
    let trainer = donor
        .data
        .trainers
        .trainers
        .get(name)
        .context("accepted skier")?;
    let globals = &current.data.story_event_script_constants.global;
    let maximum = current
        .data
        .trainers
        .trainers
        .values()
        .filter(|old| old.trainer_class == trainer.trainer_class)
        .filter_map(|old| globals.get(&old.trainer_id))
        .copied()
        .max()
        .context("native skier class")?;
    let next = maximum
        .checked_add(1)
        .context("trainer identity overflow")?;
    ensure!(next <= 255, "trainer class exhausted");
    after["story_event_script_constants"]["global"][name] = json!(next);
    Ok(())
}

fn catalog_objects(data: &mut Value) -> Result<Vec<String>> {
    let rows = data["story_events"].as_array().context("story catalogs")?;
    let mut order = Vec::new();
    let mut catalogs = serde_json::Map::new();
    for row in rows {
        let object = row.as_object().context("catalog object")?;
        ensure!(object.len() == 1, "one named catalog per payload");
        let (key, value) = object.iter().next().context("named catalog")?;
        ensure!(
            catalogs.insert(key.clone(), value.clone()).is_none(),
            "duplicate catalog {key}"
        );
        order.push(key.clone());
    }
    data["story_events"] = Value::Object(catalogs);
    Ok(order)
}

fn restore_catalogs(data: &mut Value, order: &[String]) -> Result<()> {
    let mut catalogs = data["story_events"]
        .as_object()
        .context("merged catalogs")?
        .clone();
    let mut rows = Vec::new();
    for key in order {
        let value = catalogs.remove(key).context("retained catalog")?;
        rows.push(json!({key: value}));
    }
    rows.extend(catalogs.into_iter().map(|(key, value)| json!({key: value})));
    data["story_events"] = Value::Array(rows);
    Ok(())
}

fn grouped(rows: &Value) -> Result<BTreeMap<String, Vec<Value>>> {
    let mut groups = BTreeMap::<String, Vec<Value>>::new();
    for row in rows.as_array().context("script cache rows")? {
        let name = row["source_script"]
            .as_str()
            .context("cache source script")?;
        groups.entry(name.into()).or_default().push(row.clone());
    }
    Ok(groups)
}

fn cache_rows(before: &Value, current: &Value, after: &Value) -> Result<Value> {
    let old = grouped(before)?;
    let live = grouped(current)?;
    let donor = grouped(after)?;
    let keys = old.keys().chain(donor.keys()).collect::<BTreeSet<_>>();
    let mut changed = BTreeSet::new();
    for key in keys {
        if old.get(key) == donor.get(key) || live.get(key) == donor.get(key) {
            continue;
        }
        ensure!(
            live.get(key) == old.get(key),
            "incompatible script cache {key}"
        );
        changed.insert(key.as_str());
    }
    let mut rows = current
        .as_array()
        .context("current cache")?
        .iter()
        .filter(|row| !changed.contains(row["source_script"].as_str().unwrap_or_default()))
        .cloned()
        .collect::<Vec<_>>();
    rows.extend(
        after
            .as_array()
            .context("accepted cache")?
            .iter()
            .filter(|row| changed.contains(row["source_script"].as_str().unwrap_or_default()))
            .cloned(),
    );
    Ok(Value::Array(rows))
}

fn is_cache(path: &str) -> bool {
    matches!(
        path,
        "data.maps.MnStPaulUnionDepot.script_flag_commands"
            | "data.maps.MnStPaulUnionDepot.script_movements"
            | "data.maps.MnStPaulUnionDepot.script_text_commands"
            | "data.maps.MnStPaulUnionDepot.script_runtime_commands"
            | "data.maps.MnStPaulUnionDepot.script_control_commands"
            | "data.maps.MnStPaulUnionDepot.script_object_commands"
            | "data.maps.MnStPaulUnionDepot.script_variable_commands"
            | "data.global_scripts.script_text_commands"
            | "data.global_scripts.script_runtime_commands"
            | "data.global_scripts.script_map_commands"
            | "data.global_scripts.script_control_commands"
    )
}

fn gallery_field(path: &str) -> bool {
    matches!(
        path,
        "data.maps.MnDriftlessLampGallery"
            | "data.map_attributes.MnDriftlessLampGallery"
            | "data.map_blocks.MnDriftlessLampGallery_Blocks"
            | "data.npcs.MnDriftlessLampGallery"
            | "data.map_dimensions.MN_DRIFTLESS_LAMP_GALLERY"
            | "data.map_scripts.MnDriftlessLampGallery_MapEvents"
            | "data.map_scripts.MnLampFossilsText"
    )
}

fn depot_sections(current: &Value, after: &Value) -> Result<Value> {
    let mut rows = current
        .as_array()
        .context("current depot sections")?
        .clone();
    for row in after.as_array().context("accepted depot sections")? {
        if rows
            .iter()
            .any(|old| old["command"] == row["command"] && old["args"] == row["args"])
        {
            continue;
        }
        ensure!(
            row["command"] == "callback"
                && row["args"] == json!(["MAPCALLBACK_NEWMAP", "MnGlacierST_PAULFirstVisit"]),
            "unreviewed depot section addition"
        );
        rows.push(row.clone());
    }
    for (index, row) in rows.iter_mut().enumerate() {
        if row.get("command_index").is_some() {
            row["command_index"] = json!(index);
        }
    }
    Ok(Value::Array(rows))
}

fn merge(
    before: Option<&Value>,
    current: Option<&Value>,
    after: Option<&Value>,
    path: &str,
) -> Result<Option<Value>> {
    if after == before || current == after {
        return Ok(current.cloned());
    }
    if current == before {
        return Ok(after.cloned());
    }
    if gallery_field(path) {
        return Ok(after.cloned());
    }
    if path == "data.runtime_map_metadata.MN_DRIFTLESS_LAMP_GALLERY" {
        let mut revised = after.context("accepted gallery metadata")?.clone();
        let old = current.context("current gallery metadata")?;
        for key in ["groupId", "mapId", "groupName"] {
            revised[key] = old[key].clone();
        }
        return Ok(Some(revised));
    }
    if path.ends_with(".fishing_group")
        && [
            "MnDetroitLakesStation",
            "MnWinonaStation",
            "MnStaplesStation",
        ]
        .iter()
        .any(|map| path.contains(map))
    {
        ensure!(
            after.and_then(Value::as_str) == Some("FISHGROUP_NONE")
                && current.and_then(Value::as_str) == Some("FISHGROUP_MN_SHORE"),
            "unreviewed station fishing overlap"
        );
        return Ok(after.cloned());
    }
    let (before, current, after) = (
        before.context("overlap lacks common value")?,
        current.context("overlap removes current value")?,
        after.context("overlap removes donor value")?,
    );
    if is_cache(path) {
        return Ok(Some(cache_rows(before, current, after)?));
    }
    if matches!(
        path,
        "data.maps.MnStPaulUnionDepot.map_script_section_commands"
            | "data.map_scripts.MnStPaulUnionDepot_MapScripts"
    ) {
        return Ok(Some(depot_sections(current, after)?));
    }
    if path.ends_with(".GlobalScriptRoots") {
        let mut roots = current.as_array().context("current roots")?.clone();
        let donor = after.as_array().context("accepted roots")?;
        ensure!(
            before
                .as_array()
                .context("common roots")?
                .iter()
                .all(|root| roots.contains(root) && donor.contains(root)),
            "native global script root removed"
        );
        for root in donor {
            if !roots.contains(root) {
                roots.push(root.clone());
            }
        }
        return Ok(Some(Value::Array(roots)));
    }
    if let (Some(old), Some(live), Some(new)) =
        (before.as_object(), current.as_object(), after.as_object())
    {
        let keys = old
            .keys()
            .chain(live.keys())
            .chain(new.keys())
            .collect::<BTreeSet<_>>();
        let mut result = serde_json::Map::new();
        for key in keys {
            if let Some(value) = merge(
                old.get(key),
                live.get(key),
                new.get(key),
                &format!("{path}.{key}"),
            )? {
                result.insert(key.clone(), value);
            }
        }
        return Ok(Some(Value::Object(result)));
    }
    bail!("unresolved accepted-world overlap at {path}")
}

#[cfg(test)]
mod tests {
    use super::{cache_rows, depot_sections, normalize_event_identities};
    use serde_json::json;

    #[test]
    fn successor_cache_keeps_campaign_and_adds_service_without_replacing_shared_rows() {
        let old = json!([{"source_script":"Flash","value":0}]);
        let current =
            json!([{"source_script":"Flash","value":1},{"source_script":"Campaign","value":2}]);
        let donor =
            json!([{"source_script":"Flash","value":1},{"source_script":"Train","value":3}]);
        assert_eq!(
            cache_rows(&old, &current, &donor).expect("compatible caches"),
            json!([{"source_script":"Flash","value":1},{"source_script":"Campaign","value":2},{"source_script":"Train","value":3}])
        );
        assert!(
            cache_rows(
                &old,
                &current,
                &json!([{"source_script":"Flash","value":9}])
            )
            .is_err()
        );
    }

    #[test]
    fn successor_sections_retain_scene_and_add_visit_callback() {
        let current = json!([{"command":"def_scene_scripts","args":[],"command_index":0},
            {"command":"scene_script","args":["Campaign","SCENE"],"command_index":1},
            {"command":"def_callbacks","args":[],"command_index":2}]);
        let donor = json!([{"command":"def_scene_scripts","args":[],"command_index":0},
            {"command":"def_callbacks","args":[],"command_index":1},
            {"command":"callback","args":["MAPCALLBACK_NEWMAP","MnGlacierST_PAULFirstVisit"],"command_index":2}]);
        let result = depot_sections(&current, &donor).expect("independent visit callback");
        assert_eq!(result[1], current[1]);
        assert_eq!(result[3]["command_index"], 3);
        assert!(
            depot_sections(
                &current,
                &json!([{"command":"scene_script","args":["Other","SCENE"]}])
            )
            .is_err()
        );
    }

    #[test]
    fn successor_events_keep_existing_addresses_and_allocate_unoccupied_new_flags() {
        let current =
            json!({"story_event_script_constants":{"global":{"EVENT_OLD":90,"EVENT_LAMP":91}}});
        let mut donor = json!({"story_event_script_constants":{"global":{"EVENT_OLD":1,"EVENT_LAMP":2,"EVENT_NEW":90}}});
        normalize_event_identities(&current, &mut donor).expect("rebased event names");
        assert_eq!(
            donor["story_event_script_constants"]["global"]["EVENT_LAMP"],
            91
        );
        assert_eq!(
            donor["story_event_script_constants"]["global"]["EVENT_NEW"],
            92
        );
        let invalid = json!({"story_event_script_constants":{"global":{"EVENT_FULL":65535}}});
        assert!(normalize_event_identities(&invalid, &mut donor).is_err());
    }
}
