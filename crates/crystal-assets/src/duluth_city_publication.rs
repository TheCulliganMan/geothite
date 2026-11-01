//! Additive publication preserving HOME, recovery, art, audio and existing groups.
use anyhow::{Context, Result, ensure};

use crate::{
    CompiledGamePack, CompiledMapExtension, MapModule, PokegearLandmark, RuntimeMapMetadata,
    RuntimeSpawnPoint,
};

pub(super) const GROUP_ID: u16 = 68;
const GROUP: &str = "MINNESOTA_DULUTH";
const LANDMARK: &str = "LANDMARK_MN_DULUTH";
pub(super) const MANIFEST: &str = "minnesota-duluth-city-v1";

pub(super) fn available(base: &CompiledGamePack) -> Result<()> {
    ensure!(
        !base
            .data
            .runtime_map_metadata
            .values()
            .any(|m| m.group_id == GROUP_ID)
            && !base
                .data
                .story_event_script_constants
                .global
                .iter()
                .any(|(name, value)| name == "MAPGROUP_MINNESOTA_DULUTH"
                    || (name.starts_with("MAPGROUP_") && *value == i64::from(GROUP_ID)))
            && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "Duluth native group68 or manifest already occupied"
    );
    ensure!(
        !base
            .data
            .pokegear_landmarks
            .landmarks
            .iter()
            .any(|l| l.constant == LANDMARK),
        "Duluth landmark already occupied"
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
        .insert("MAPGROUP_MINNESOTA_DULUTH".into(), i64::from(GROUP_ID));
    let landmark_id = pack
        .data
        .pokegear_landmarks
        .landmarks
        .iter()
        .map(|l| l.id)
        .max()
        .context("native landmark table")?
        .checked_add(1)
        .context("Duluth landmark identifier overflow")?;
    pack.data
        .pokegear_landmarks
        .landmarks
        .push(PokegearLandmark {
            id: landmark_id,
            constant: LANDMARK.into(),
            label: "MN_DULUTH".into(),
            name: "DULUTH".into(),
            x: 144,
            y: 72,
            region: "JOHTO".into(),
        });
    let temporary = pack
        .data
        .runtime_spawn_points
        .values()
        .map(|s| s.identifier)
        .max()
        .context("native recovery table")?
        .checked_add(1)
        .context("temporary Duluth insertion anchor")?;
    for (index, (mut module, [x, y])) in modules.into_iter().enumerate() {
        module.attributes.location = Some(LANDMARK.into());
        module.attributes.map_group_constant = Some("MAPGROUP_MINNESOTA_DULUTH".into());
        ensure!(
            module.attributes.connections.is_empty(),
            "Duluth regional bindings remain caller-owned"
        );
        let name = module.id.clone();
        let constant = module
            .attributes
            .map_constant
            .clone()
            .context("Duluth native map constant")?;
        let map_id = u16::try_from(index + 1)?;
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
                    .context("Duluth native environment")?,
                phone_service: module.attributes.phone_service,
            },
            spawn_key: temporary.to_string(),
            spawn: RuntimeSpawnPoint {
                identifier: temporary,
                map_constant: constant,
                map_name: name,
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
        pack.data
            .runtime_spawn_points
            .remove(&temporary.to_string());
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
