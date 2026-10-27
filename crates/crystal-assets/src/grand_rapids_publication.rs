//! Additive native map publication with original HOME and recovery preserved.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};

use super::{CONSTANT, GROUP, GROUP_ID, LANDMARK, MANIFEST, TOWN};
use crate::{
    CompiledGamePack, CompiledMapExtension, MapModule, PokegearLandmark, RuntimeMapMetadata,
    RuntimeSpawnPoint,
};

pub(super) fn available(base: &CompiledGamePack, names: &BTreeSet<String>) -> Result<()> {
    ensure!(
        !base
            .data
            .runtime_map_metadata
            .values()
            .any(|m| m.group_id == GROUP_ID)
            && !base
                .report
                .manifests
                .iter()
                .any(|s| s.starts_with(MANIFEST))
            && !base
                .data
                .story_event_script_constants
                .global
                .iter()
                .any(|(k, v)| k == &format!("MAPGROUP_{GROUP}")
                    || (k.starts_with("MAPGROUP_") && *v == i64::from(GROUP_ID))),
        "Grand Rapids group71 already occupied"
    );
    ensure!(
        names
            .iter()
            .all(|n| !base.data.maps.contains_key(n)
                && !base.data.runtime_map_metadata.contains_key(n)),
        "Grand Rapids native map already installed"
    );
    ensure!(
        !base
            .data
            .pokegear_landmarks
            .landmarks
            .iter()
            .any(|l| l.constant == LANDMARK),
        "Grand Rapids landmark occupied"
    );
    Ok(())
}

pub(super) fn publish(
    mut pack: CompiledGamePack,
    modules: Vec<(MapModule, [u16; 2])>,
) -> Result<CompiledGamePack> {
    pack.data
        .story_event_script_constants
        .global
        .insert(format!("MAPGROUP_{GROUP}"), i64::from(GROUP_ID));
    let landmark = pack
        .data
        .pokegear_landmarks
        .landmarks
        .iter()
        .map(|l| l.id)
        .max()
        .context("source landmarks")?
        .checked_add(1)
        .context("Grand Rapids landmark overflow")?;
    pack.data
        .pokegear_landmarks
        .landmarks
        .push(PokegearLandmark {
            id: landmark,
            constant: LANDMARK.into(),
            label: CONSTANT.into(),
            name: "GRAND RAPIDS".into(),
            x: 112,
            y: 64,
            region: "JOHTO".into(),
        });
    let first = pack
        .data
        .runtime_spawn_points
        .values()
        .map(|s| s.identifier)
        .max()
        .context("source recovery records")?
        .checked_add(1)
        .context("Grand Rapids recovery identity overflow")?
        .max(crystal_core::systems::special_routines::FIRST_ADDON_RECOVERY_SPAWN);
    for (index, (module, [x, y])) in modules.into_iter().enumerate() {
        let name = module.id.clone();
        let constant = module
            .attributes
            .map_constant
            .clone()
            .context("Grand Rapids map constant")?;
        let map_id = u16::try_from(index + 1)?;
        let identifier = first
            .checked_add(u16::try_from(index)?)
            .context("Grand Rapids recovery identity overflow")?;
        pack.data
            .pokegear_landmarks
            .map_to_landmark
            .insert(name.clone(), LANDMARK.into());
        let extension = CompiledMapExtension {
            manifest_id: format!("{MANIFEST}-{index}"),
            map_name: name.clone(),
            map_constant: constant.clone(),
            metadata: RuntimeMapMetadata {
                constant: constant.clone(),
                name: name.clone(),
                group_name: GROUP.into(),
                group_id: GROUP_ID,
                map_id,
                width: module.attributes.width,
                height: module.attributes.height,
                environment: module
                    .attributes
                    .environment
                    .clone()
                    .context("Grand Rapids environment")?,
                phone_service: module.attributes.phone_service,
            },
            spawn_key: identifier.to_string(),
            spawn: RuntimeSpawnPoint {
                identifier,
                map_constant: constant,
                map_name: name.clone(),
                group_name: GROUP.into(),
                group_id: i16::try_from(GROUP_ID)?,
                map_id: i16::try_from(map_id)?,
                tile_x: i16::try_from(x)?,
                tile_y: i16::try_from(y)?,
                metatile_x: i16::try_from(x / 2)?,
                metatile_y: i16::try_from(y / 2)?,
                subtile_x: i16::try_from(x % 2)?,
                subtile_y: i16::try_from(y % 2)?,
            },
            module,
            wild_encounters: None,
            start_new_game_here: false,
        };
        crate::validate_compiled_map_extension(&extension)?;
        crate::insert_compiled_map_extension(&mut pack, extension)?;
        if name != TOWN && name != "MnGrandRapidsPokecenter1F" {
            pack.data
                .runtime_spawn_points
                .remove(&identifier.to_string());
        }
    }
    pack.report.manifests.push(MANIFEST.into());
    pack.report.maps = pack.data.maps.len();
    pack.report.reachable_maps.sort();
    pack.report.solvable_maps.sort();
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
