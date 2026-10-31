//! Reconcile current regional habitats with the complete campaign.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, bail, ensure};
use serde_json::{Value, json};

use crate::CompiledGamePack;

const HINT: &str = "MN_LEECH_SHORE_FISHERText";
const NATURALIST: &str = "MN_EXTRA_LEECH_ISLAND_NATURALIST";
const MARKER: &str = "comprehensive-water-habitats-v1";

/// Adds the reviewed island and water/cave habitats to the complete campaign.
///
/// Retains current campaign programs, existing event identities, lake directions,
/// audio and native art. Only the pinned donor's new animation metadata may be
/// added. Inputs and existing saves are never modified or migrated.
///
/// # Errors
/// Rejects modified input identities, incompatible source programs, unexpected
/// overlapping terrain or fishing headers, occupied trainer identities, asset
/// replacement, and invalid resulting runtime data or regional geography.
pub fn integrate_comprehensive_water_habitats(
    current: &CompiledGamePack,
    common: &CompiledGamePack,
    donor: &CompiledGamePack,
) -> Result<CompiledGamePack> {
    for (pack, expected) in [
        (
            current,
            "0fb868423bc402679f5623cf6fdda2b2af87e7ab0228a5f8d38dfdd3d0667386",
        ),
        (
            common,
            "eee053cc55a1f3ddcc98c9cab722cf9b2a2630794993bb0d6b1cdfbe249bcbe0",
        ),
        (
            donor,
            "933857a91d19b2f638e3a1dc592c3eedb786321b73de7c09bb01aa8c75b5f175",
        ),
    ] {
        let identity = crate::derive_compiled_game_pack_identity_from_manifest(
            pack.format_version,
            &pack.data,
            &pack.audio_manifest,
            &pack.runtime_files,
            &pack.report,
        )?;
        ensure!(
            identity.content_hash == expected,
            "exact reviewed habitat inputs required"
        );
        crate::verify_compiled_game_pack_for_runtime(pack)?;
    }
    ensure!(
        common.audio_manifest == donor.audio_manifest
            && common.compiled_audio == donor.compiled_audio,
        "donor audio changed"
    );
    let mut pack = current.clone();
    for (name, bytes) in &common.runtime_files {
        ensure!(
            donor.runtime_files.get(name) == Some(bytes),
            "donor replaced native asset {name}"
        );
    }
    for (name, bytes) in &donor.runtime_files {
        if common.runtime_files.contains_key(name) {
            continue;
        }
        ensure!(
            name == "data/tilesets/mn_headwaters_animations.json",
            "unexpected new asset {name}"
        );
        if let Some(existing) = pack.runtime_files.get(name) {
            ensure!(existing == bytes, "conflicting native animation metadata");
        } else {
            pack.runtime_files.insert(name.clone(), bytes.clone());
        }
    }
    let mut old = serde_json::to_value(&common.data)?;
    let mut live = serde_json::to_value(&current.data)?;
    let mut new = serde_json::to_value(&donor.data)?;
    normalize_identities(current, donor, &mut new)?;
    let order = catalogs(&mut live)?;
    catalogs(&mut old)?;
    catalogs(&mut new)?;
    let mut merged =
        merge(Some(&old), Some(&live), Some(&new), "data")?.context("retained regional data")?;
    let mut rows = merged["story_events"]
        .as_object()
        .context("merged catalogs")?
        .clone();
    let mut ordered = Vec::new();
    for name in order {
        ordered.push(json!({name.clone(): rows.remove(&name).context("existing story catalog")?}));
    }
    ordered.extend(rows.into_iter().map(|(name, value)| json!({name: value})));
    merged["story_events"] = Value::Array(ordered);
    pack.data = serde_json::from_value(merged)?;
    let mut lake = pack.data.maps["MnLeechLake"].clone();
    crate::minnesota_field_use::hint(&mut lake, HINT, &["The pine island's", "landing is west."])?;
    let mut bodies = crate::parse_script_text_bodies(&lake.id, &lake.scripts)?;
    lake.script_text_bodies.insert(
        HINT.into(),
        bodies.remove(HINT).context("combined fisher directions")?,
    );
    crate::authored_town_publication::replace(&mut pack, lake)?;
    for marker in &donor.report.manifests {
        if !common.report.manifests.contains(marker) && !pack.report.manifests.contains(marker) {
            pack.report.manifests.push(marker.clone());
        }
    }
    ensure!(
        !pack.report.manifests.iter().any(|m| m == MARKER),
        "habitats already installed"
    );
    pack.report.manifests.push(MARKER.into());
    ensure!(
        pack.data.maps.len() == current.data.maps.len(),
        "habitats must not replace the campaign map set"
    );
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

fn normalize_identities(
    current: &CompiledGamePack,
    donor: &CompiledGamePack,
    new: &mut Value,
) -> Result<()> {
    let existing = &current.data.story_event_script_constants.global;
    let mut next = existing
        .iter()
        .filter(|(name, _)| name.starts_with("EVENT_"))
        .map(|(_, value)| *value)
        .max()
        .context("current events")?;
    for (name, value) in new["story_event_script_constants"]["global"]
        .as_object_mut()
        .context("donor globals")?
    {
        if !name.starts_with("EVENT_") {
            continue;
        }
        if let Some(id) = existing.get(name) {
            *value = json!(id);
        } else {
            next = next.checked_add(1).context("event identity overflow")?;
            u16::try_from(next).context("event identity space exhausted")?;
            *value = json!(next);
        }
    }
    ensure!(
        !current.data.trainers.trainers.contains_key(NATURALIST),
        "naturalist already installed"
    );
    let trainer = donor
        .data
        .trainers
        .trainers
        .get(NATURALIST)
        .context("reviewed naturalist")?;
    let maximum = current
        .data
        .trainers
        .trainers
        .values()
        .filter(|t| t.trainer_class == trainer.trainer_class)
        .filter_map(|t| existing.get(&t.trainer_id))
        .max()
        .context("native trainer class")?;
    let next = maximum
        .checked_add(1)
        .context("trainer identity overflow")?;
    ensure!(next <= 255, "trainer class exhausted");
    new["story_event_script_constants"]["global"][NATURALIST] = json!(next);
    Ok(())
}

fn catalogs(data: &mut Value) -> Result<Vec<String>> {
    let mut table = serde_json::Map::new();
    let mut order = Vec::new();
    for row in data["story_events"].as_array().context("story catalogs")? {
        let row = row.as_object().context("named story catalog")?;
        ensure!(row.len() == 1, "one name per story catalog");
        let (name, value) = row.iter().next().context("catalog entry")?;
        ensure!(
            table.insert(name.clone(), value.clone()).is_none(),
            "duplicate catalog"
        );
        order.push(name.clone());
    }
    data["story_events"] = Value::Object(table);
    Ok(order)
}

fn source_rows(rows: &Value) -> Result<BTreeMap<String, Vec<Value>>> {
    let mut grouped = BTreeMap::<String, Vec<Value>>::new();
    for row in rows.as_array().context("compiled source cache")? {
        grouped
            .entry(
                row["source_script"]
                    .as_str()
                    .context("source script")?
                    .into(),
            )
            .or_default()
            .push(row.clone());
    }
    Ok(grouped)
}

fn merge(
    old: Option<&Value>,
    live: Option<&Value>,
    new: Option<&Value>,
    path: &str,
) -> Result<Option<Value>> {
    if old == new || live == new {
        return Ok(live.cloned());
    }
    if old == live {
        return Ok(new.cloned());
    }
    let (old, live, new) = (
        old.context("overlap lacks common value")?,
        live.context("overlap removes campaign value")?,
        new.context("overlap removes habitat value")?,
    );
    // Keep campaign testimony and its existing cove hint in every source/cache.
    // The island directions are appended once after typed reconstruction.
    if path.contains(HINT) {
        return Ok(Some(live.clone()));
    }
    if path.ends_with(".fishing_group") {
        let plan: Value = serde_json::from_str(include_str!(
            "../../../planning/minnesota/pokemon/acquisition-v01/water-caves-v01/habitats.json"
        ))?;
        let approved = plan["fishing_repairs"]
            .as_array()
            .context("authored fishing repairs")?
            .iter()
            .any(|repair| {
                repair["map"].as_str().is_some_and(|map| {
                    path == format!("data.maps.{map}.attributes.fishing_group")
                        || path == format!("data.map_attributes.{map}.fishing_group")
                }) && repair["after"] == *new
            });
        ensure!(
            approved
                && new == "FISHGROUP_NONE"
                && live
                    .as_str()
                    .is_some_and(|s| s.starts_with("FISHGROUP_MN_")),
            "unreviewed fishing overlap {path}"
        );
        return Ok(Some(new.clone()));
    }
    if path == "data.map_blocks.MnLeechLake_Blocks" {
        let decode = |value: &Value| -> Result<Value> {
            Ok(serde_json::to_value(crate::decode_base64_bytes(
                value.as_str().context("encoded lake blocks")?,
            )?)?)
        };
        let (a, b, c) = (decode(old)?, decode(live)?, decode(new)?);
        let blocks = merge(Some(&a), Some(&b), Some(&c), "data.maps.MnLeechLake.blocks")?
            .context("retained lake block mirror")?;
        let bytes: Vec<u8> = serde_json::from_value(blocks)?;
        return Ok(Some(json!(crate::encode_base64_bytes(&bytes))));
    }
    if path == "data.maps.MnLeechLake.blocks" {
        let (a, b, c) = (
            old.as_array().context("common blocks")?,
            live.as_array().context("campaign blocks")?,
            new.as_array().context("island blocks")?,
        );
        ensure!(
            a.len() == b.len() && a.len() == c.len(),
            "lake dimensions changed"
        );
        let rows = a
            .iter()
            .zip(b)
            .zip(c)
            .map(|((a, b), c)| {
                merge(Some(a), Some(b), Some(c), "lake terrain cell")
                    .and_then(|v| v.context("retained terrain"))
            })
            .collect::<Result<Vec<_>>>()?;
        return Ok(Some(Value::Array(rows)));
    }
    if path.ends_with(".GlobalScriptRoots") {
        let mut rows = live.as_array().context("campaign script roots")?.clone();
        let donor = new.as_array().context("habitat script roots")?;
        ensure!(
            old.as_array()
                .context("common roots")?
                .iter()
                .all(|r| rows.contains(r) && donor.contains(r)),
            "script root removed"
        );
        for row in donor {
            if !rows.contains(row) {
                rows.push(row.clone());
            }
        }
        return Ok(Some(Value::Array(rows)));
    }
    if (path.starts_with("data.global_scripts.script_")
        || path == "data.maps.MnLeechLake.script_text_commands")
        && old.is_array()
        && live.is_array()
        && new.is_array()
    {
        let (a, b, c) = (source_rows(old)?, source_rows(live)?, source_rows(new)?);
        let changed = a
            .keys()
            .chain(c.keys())
            .filter(|k| a.get(*k) != c.get(*k) && b.get(*k) != c.get(*k))
            .cloned()
            .collect::<BTreeSet<_>>();
        for key in &changed {
            ensure!(
                key == HINT || a.get(key) == b.get(key),
                "divergent source program {key}"
            );
        }
        let changed = changed
            .into_iter()
            .filter(|k| k != HINT)
            .collect::<BTreeSet<_>>();
        let mut rows = live
            .as_array()
            .context("campaign cache")?
            .iter()
            .filter(|r| !changed.contains(r["source_script"].as_str().unwrap_or_default()))
            .cloned()
            .collect::<Vec<_>>();
        rows.extend(
            new.as_array()
                .context("habitat cache")?
                .iter()
                .filter(|r| changed.contains(r["source_script"].as_str().unwrap_or_default()))
                .cloned(),
        );
        return Ok(Some(Value::Array(rows)));
    }
    if matches!(
        path,
        "data.maps.MnLeechLake.map_event_section_commands"
            | "data.map_scripts.MnLeechLake_MapEvents"
            | "data.maps.MnLeechLake.script_field_pickups"
    ) {
        let a = live.as_array().context("campaign lake rows")?;
        let b = new.as_array().context("island lake rows")?;
        ensure!(
            a.iter().all(|row| b.iter().any(|other| {
                let (mut row, mut other) = (row.clone(), other.clone());
                if let Some(v) = row.as_object_mut() {
                    v.remove("command_index");
                }
                if let Some(v) = other.as_object_mut() {
                    v.remove("command_index");
                }
                row == other
            })),
            "habitat removes current lake event {path}"
        );
        return Ok(Some(new.clone()));
    }
    if let (Some(a), Some(b), Some(c)) = (old.as_object(), live.as_object(), new.as_object()) {
        let mut rows = serde_json::Map::new();
        for key in a
            .keys()
            .chain(b.keys())
            .chain(c.keys())
            .collect::<BTreeSet<_>>()
        {
            if let Some(value) =
                merge(a.get(key), b.get(key), c.get(key), &format!("{path}.{key}"))?
            {
                rows.insert(key.clone(), value);
            }
        }
        return Ok(Some(Value::Object(rows)));
    }
    bail!("unresolved habitat overlap at {path}")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::merge;

    #[test]
    fn lake_terrain_retains_both_disjoint_changes() {
        let before = json!([53, 89, 53]);
        let campaign = json!([53, 7, 53]);
        let island = json!([5, 89, 88]);
        let actual = merge(
            Some(&before),
            Some(&campaign),
            Some(&island),
            "data.maps.MnLeechLake.blocks",
        )
        .expect("disjoint native terrain changes");
        assert_eq!(actual, Some(json!([5, 7, 88])));
        let encoded_before = json!(crate::encode_base64_bytes(&[53, 89, 53]));
        let encoded_campaign = json!(crate::encode_base64_bytes(&[53, 7, 53]));
        let encoded_island = json!(crate::encode_base64_bytes(&[5, 89, 88]));
        assert_eq!(
            merge(
                Some(&encoded_before),
                Some(&encoded_campaign),
                Some(&encoded_island),
                "data.map_blocks.MnLeechLake_Blocks"
            )
            .expect("native block mirror"),
            Some(json!(crate::encode_base64_bytes(&[5, 7, 88])))
        );
        let incompatible = json!([5, 54, 88]);
        assert!(
            merge(
                Some(&before),
                Some(&campaign),
                Some(&incompatible),
                "data.maps.MnLeechLake.blocks"
            )
            .is_err()
        );
    }

    #[test]
    fn compiled_programs_merge_by_source_and_reject_divergent_programs() {
        let before = json!([{"source_script":"old","command":"end"}]);
        let campaign = json!([{"source_script":"old","command":"end"},{"source_script":"campaign","command":"end"}]);
        let habitat = json!([{"source_script":"old","command":"end"},{"source_script":"habitat","command":"end"}]);
        assert_eq!(
            merge(
                Some(&before),
                Some(&campaign),
                Some(&habitat),
                "data.global_scripts.script_runtime_commands"
            )
            .expect("independent source programs"),
            Some(json!([
                {"source_script":"old","command":"end"},{"source_script":"campaign","command":"end"},{"source_script":"habitat","command":"end"}
            ]))
        );
        let old_bodies = json!({"base": {"label": "base", "commands": []}});
        let campaign_bodies = json!({"base": {"label": "base", "commands": []}, "campaign": {"label": "campaign", "commands": []}});
        let habitat_bodies = json!({"base": {"label": "base", "commands": []}, "habitat": {"label": "habitat", "commands": []}});
        assert_eq!(
            merge(
                Some(&old_bodies),
                Some(&campaign_bodies),
                Some(&habitat_bodies),
                "data.global_scripts.script_text_bodies"
            )
            .expect("dictionary of native text bodies"),
            Some(
                json!({"base": {"label": "base", "commands": []}, "campaign": {"label": "campaign", "commands": []}, "habitat": {"label": "habitat", "commands": []}})
            )
        );
        let changed_campaign = json!([{"source_script":"old","command":"win"}]);
        let changed_habitat = json!([{"source_script":"old","command":"lose"}]);
        assert!(
            merge(
                Some(&before),
                Some(&changed_campaign),
                Some(&changed_habitat),
                "data.global_scripts.script_runtime_commands"
            )
            .is_err()
        );
    }

    #[test]
    fn island_cannot_remove_the_campaign_cove_reward() {
        let before = json!([]);
        let campaign = json!([{"source_script":"eddy","command":"hiddenitem"}]);
        let habitat = json!([{"source_script":"island","command":"itemball"}]);
        assert!(
            merge(
                Some(&before),
                Some(&campaign),
                Some(&habitat),
                "data.maps.MnLeechLake.script_field_pickups"
            )
            .is_err()
        );
    }
}
