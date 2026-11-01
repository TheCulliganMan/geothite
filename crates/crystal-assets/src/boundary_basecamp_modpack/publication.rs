//! Atomic additive map publication; regional seam integration is caller-owned.
use anyhow::{Context, Result, ensure};

use super::{BOUNDARY_BASECAMP_GROUP, BOUNDARY_BASECAMP_MANIFEST, GROUP, LANDMARK, Town};
use crate::{
    CompiledGamePack, CompiledMapExtension, MapModule, PokegearLandmark, RuntimeMapMetadata,
    RuntimeSpawnPoint,
};

pub(super) fn validate_available(
    base: &CompiledGamePack,
    town: &Town,
    shelter_constant: &str,
) -> Result<()> {
    ensure!(
        !base
            .data()
            .runtime_map_metadata
            .values()
            .any(|map| map.group_id == BOUNDARY_BASECAMP_GROUP)
            && !base
                .report()
                .manifests
                .iter()
                .any(|id| id == BOUNDARY_BASECAMP_MANIFEST),
        "Boundary basecamp group62 or manifest already occupied"
    );
    for (name, constant) in [
        (&town.id, town.native_constant.as_str()),
        (&town.interiors[0].id, shelter_constant),
    ] {
        ensure!(
            !base.data().maps.contains_key(name)
                && !base.data().runtime_map_metadata.contains_key(constant)
                && !base
                    .data()
                    .runtime_map_metadata
                    .values()
                    .any(|map| map.name == *name),
            "Boundary basecamp map/constant already occupied: {name}"
        );
    }
    Ok(())
}

pub(super) fn publish(
    mut pack: CompiledGamePack,
    modules: [MapModule; 2],
    arrivals: [[u16; 2]; 2],
) -> Result<CompiledGamePack> {
    let group = format!("MAPGROUP_{GROUP}");
    ensure!(
        !pack
            .data
            .story_event_script_constants
            .global
            .contains_key(&group),
        "basecamp group constant occupied"
    );
    pack.data
        .story_event_script_constants
        .global
        .insert(group, i64::from(BOUNDARY_BASECAMP_GROUP));
    ensure!(
        !pack
            .data
            .pokegear_landmarks
            .landmarks
            .iter()
            .any(|entry| entry.constant == LANDMARK),
        "basecamp landmark occupied"
    );
    let landmark_id = pack
        .data
        .pokegear_landmarks
        .landmarks
        .iter()
        .map(|entry| entry.id)
        .max()
        .context("source landmark table")?
        .checked_add(1)
        .context("basecamp landmark identifier")?;
    pack.data
        .pokegear_landmarks
        .landmarks
        .push(PokegearLandmark {
            id: landmark_id,
            constant: LANDMARK.into(),
            label: "MN_BOUNDARY_BASECAMP".into(),
            name: "BOUNDARY WATERS".into(),
            x: 144,
            y: 8,
            region: "JOHTO".into(),
        });
    // The existing bounded recovery table remains byte-for-byte unchanged.
    // insert_compiled_map_extension requires an insertion anchor; remove only
    // this temporary record before the final production verification. This is
    // the existing authored-route publication pattern, not a new player spawn.
    let temporary_spawn = pack
        .data
        .runtime_spawn_points
        .values()
        .map(|spawn| spawn.identifier)
        .max()
        .context("original recovery table")?
        .checked_add(1)
        .context("temporary insertion anchor")?;
    for (index, (module, [x, y])) in modules.into_iter().zip(arrivals).enumerate() {
        ensure!(
            super::geometry::kind(&pack, &module, [x, y])? == "FLOOR"
                && !module
                    .objects
                    .iter()
                    .any(|actor| [actor.x, actor.y] == [x, y])
                && module.attributes.connections.is_empty(),
            "basecamp publication requires safe arrival and caller-owned regional connections"
        );
        let name = module.id.clone();
        let constant = module
            .attributes
            .map_constant
            .clone()
            .context("authored map constant")?;
        let map_id = u16::try_from(index + 1)?;
        pack.data
            .pokegear_landmarks
            .map_to_landmark
            .insert(name.clone(), LANDMARK.into());
        let extension = CompiledMapExtension {
            manifest_id: format!("{BOUNDARY_BASECAMP_MANIFEST}-{index}"),
            map_name: name.clone(),
            map_constant: constant.clone(),
            metadata: RuntimeMapMetadata {
                name: name.clone(),
                constant: constant.clone(),
                group_name: GROUP.into(),
                group_id: BOUNDARY_BASECAMP_GROUP,
                map_id,
                width: module.attributes.width,
                height: module.attributes.height,
                environment: module
                    .attributes
                    .environment
                    .clone()
                    .context("native map environment")?,
                phone_service: module.attributes.phone_service,
            },
            spawn_key: temporary_spawn.to_string(),
            spawn: RuntimeSpawnPoint {
                identifier: temporary_spawn,
                map_constant: constant,
                map_name: name,
                group_name: GROUP.into(),
                group_id: i16::try_from(BOUNDARY_BASECAMP_GROUP)?,
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
        pack.data
            .runtime_spawn_points
            .remove(&temporary_spawn.to_string());
    }
    pack.report
        .manifests
        .push(BOUNDARY_BASECAMP_MANIFEST.into());
    pack.report.maps = pack.data.maps.len();
    pack.report.reachable_maps.sort();
    pack.report.solvable_maps.sort();
    refresh_identity(&mut pack)?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}
pub(super) fn refresh_identity(pack: &mut CompiledGamePack) -> Result<()> {
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    Ok(())
}
