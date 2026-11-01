//! Reject changes outside the new park and its single R18 road binding.
use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;

use super::{CONSTANT, MANIFEST, MAP, PICKUP, ROUTE};

pub(super) fn validate(base: &CompiledGamePack, output: &CompiledGamePack) -> Result<()> {
    ensure!(
        base.runtime_files == output.runtime_files && base.audio_manifest == output.audio_manifest,
        "park changes original art or audio"
    );
    let mut after = serde_json::to_value(&output.data)?;
    let before = serde_json::to_value(&base.data)?;
    let mut route = output.data.maps[ROUTE].clone();
    route.attributes.connections = base.data.maps[ROUTE].attributes.connections.clone();
    route.attributes.connection_flags = base.data.maps[ROUTE].attributes.connection_flags.clone();
    ensure!(
        route == base.data.maps[ROUTE],
        "park changes R18 terrain/programs/actors"
    );
    after["maps"][ROUTE] = before["maps"][ROUTE].clone();
    after["map_attributes"][ROUTE] = before["map_attributes"][ROUTE].clone();
    let recoveries = output
        .data
        .runtime_spawn_points
        .iter()
        .filter(|(key, _)| !base.data.runtime_spawn_points.contains_key(*key))
        .collect::<Vec<_>>();
    ensure!(recoveries.len() == 1, "one explicit park spawn required");
    for (key, point) in recoveries {
        ensure!(
            point.map_name == MAP && point.tile_x == 2 && point.tile_y == 12,
            "new park spawn differs"
        );
        after["runtime_spawn_points"]
            .as_object_mut()
            .context("spawn table")?
            .remove(key);
    }
    prune(&before, &mut after, "")?;
    let old = before["pokegear_landmarks"]["landmarks"]
        .as_array()
        .context("old landmarks")?;
    let new = after["pokegear_landmarks"]["landmarks"]
        .as_array_mut()
        .context("new landmarks")?;
    ensure!(
        new.starts_with(old)
            && new[old.len()..]
                .iter()
                .all(|v| v["constant"] == "LANDMARK_MN_MILLE_LACS_PARK"),
        "original caught-location areas changed"
    );
    new.truncate(old.len());
    ensure!(
        before == after,
        "unowned pack data changed during park publication"
    );
    let mut report = output.report.clone();
    report.maps = base.report.maps;
    report
        .manifests
        .retain(|m| m != MANIFEST && m != &format!("{MANIFEST}-0"));
    report.reachable_maps.retain(|m| m != MAP);
    report.solvable_maps.retain(|m| m != MAP);
    ensure!(report == base.report, "unrelated report change");
    Ok(())
}

fn prune(before: &Value, after: &mut Value, path: &str) -> Result<()> {
    if let (Some(old), Some(new)) = (before.as_object(), after.as_object_mut()) {
        let extras = new
            .keys()
            .filter(|k| !old.contains_key(*k))
            .cloned()
            .collect::<Vec<_>>();
        for key in extras {
            let owned = match path {
                "/maps" | "/map_attributes" | "/npcs" | "/wild_encounters" => key == MAP,
                "/map_dimensions" | "/runtime_map_metadata" => key == CONSTANT,
                "/map_blocks" => key == format!("{MAP}_Blocks"),
                "/map_scripts" => {
                    key.starts_with("MN_ML_PARK_") || key.starts_with("MnMilleLacs_Map")
                }
                "/story_event_script_constants/global" => {
                    key == PICKUP || key == "MAPGROUP_MINNESOTA_MILLE_LACS_PARK"
                }
                "/pokegear_landmarks/map_to_landmark" => key == MAP,
                _ => false,
            };
            ensure!(owned, "unowned new data at {path}/{key}");
            new.remove(&key);
        }
        for (key, value) in old {
            let next = new
                .get_mut(key)
                .with_context(|| format!("removed original {path}/{key}"))?;
            prune(value, next, &format!("{path}/{key}"))?;
        }
    }
    Ok(())
}
