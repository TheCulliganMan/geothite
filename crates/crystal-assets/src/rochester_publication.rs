//! Additive city identities and synchronized replacement of installed modules.
use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::{
    CompiledGamePack, CompiledMapExtension, MapModule, PokegearLandmark, RuntimeMapMetadata,
    RuntimeSpawnPoint,
};

pub(super) fn replace(pack: &mut CompiledGamePack, map: MapModule) -> Result<()> {
    let constant = map
        .attributes
        .map_constant
        .as_ref()
        .context("map constant")?;
    if let Some(old) = pack.data.maps.get(&map.id) {
        for label in old.scripts.keys().chain(old.script_text_bodies.keys()) {
            ensure!(
                !pack.data.maps.values().any(|m| m.id != map.id
                    && (m.scripts.contains_key(label) || m.script_text_bodies.contains_key(label))),
                "replaced script label is shared: {label}"
            );
            pack.data.map_scripts.remove(label);
        }
    }
    crate::insert_compiled_map_split_payloads(
        &mut pack.data.map_scripts,
        &mut pack.data.npcs,
        &map.id,
        map.attributes
            .map_scripts_label
            .as_deref()
            .context("script section")?,
        map.attributes
            .map_events_label
            .as_deref()
            .context("event section")?,
        &map,
    )?;
    let bytes = map
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    pack.data.map_blocks.insert(
        map.attributes.blocks_label.clone().context("blocks")?,
        crate::encode_base64_bytes(&bytes),
    );
    pack.data
        .map_attributes
        .insert(map.id.clone(), map.attributes.clone());
    pack.data.map_dimensions.insert(
        constant.clone(),
        json!({"width":map.attributes.width,"height":map.attributes.height}),
    );
    let meta = pack
        .data
        .runtime_map_metadata
        .get_mut(constant)
        .context("installed metadata")?;
    meta.width = map.attributes.width;
    meta.height = map.attributes.height;
    pack.data.maps.insert(map.id.clone(), map);
    Ok(())
}

pub(super) fn publish(
    pack: &mut CompiledGamePack,
    modules: Vec<(MapModule, [u16; 2])>,
) -> Result<()> {
    const GROUP: &str = "MINNESOTA_ROCHESTER";
    const LANDMARK: &str = "LANDMARK_MN_ROCHESTER";
    ensure!(
        !pack
            .data
            .runtime_map_metadata
            .values()
            .any(|m| m.group_id == 74)
            && !pack
                .data
                .story_event_script_constants
                .global
                .contains_key("MAPGROUP_MINNESOTA_ROCHESTER")
            && !pack
                .data
                .pokegear_landmarks
                .landmarks
                .iter()
                .any(|l| l.constant == LANDMARK),
        "Rochester identity already occupied"
    );
    pack.data
        .story_event_script_constants
        .global
        .insert(format!("MAPGROUP_{GROUP}"), 74);
    let landmark = pack
        .data
        .pokegear_landmarks
        .landmarks
        .iter()
        .map(|l| l.id)
        .max()
        .context("landmarks")?
        .checked_add(1)
        .context("landmark overflow")?;
    pack.data
        .pokegear_landmarks
        .landmarks
        .push(PokegearLandmark {
            id: landmark,
            constant: LANDMARK.into(),
            label: "MN_ROCHESTER".into(),
            name: "ROCHESTER".into(),
            x: 96,
            y: 128,
            region: "JOHTO".into(),
        });
    let first = pack
        .data
        .runtime_spawn_points
        .values()
        .map(|s| s.identifier)
        .max()
        .context("spawn table")?
        .checked_add(1)
        .context("recovery overflow")?;
    for (index, (mut module, [x, y])) in modules.into_iter().enumerate() {
        let name = module.id.clone();
        let constant = module
            .attributes
            .map_constant
            .clone()
            .context("Rochester constant")?;
        module.attributes.location = Some(LANDMARK.into());
        if name == "MnRochesterGym" {
            pack.data
                .pokegear_landmarks
                .map_to_landmark
                .insert(name, LANDMARK.into());
            replace(pack, module)?;
            continue;
        }
        ensure!(
            !pack.data.maps.contains_key(&name),
            "Rochester map already installed: {name}"
        );
        module.attributes.map_group_constant = Some(format!("MAPGROUP_{GROUP}"));
        let map_id = u16::try_from(index + 1)?;
        let identifier = first
            .checked_add(u16::try_from(index)?)
            .context("recovery overflow")?;
        pack.data
            .pokegear_landmarks
            .map_to_landmark
            .insert(name.clone(), LANDMARK.into());
        let extension = CompiledMapExtension {
            manifest_id: format!("minnesota-rochester-city-v1-{index}"),
            map_name: name.clone(),
            map_constant: constant.clone(),
            metadata: RuntimeMapMetadata {
                name: name.clone(),
                constant: constant.clone(),
                group_name: GROUP.into(),
                group_id: 74,
                map_id,
                width: module.attributes.width,
                height: module.attributes.height,
                environment: module
                    .attributes
                    .environment
                    .clone()
                    .context("environment")?,
                phone_service: module.attributes.phone_service,
            },
            spawn_key: identifier.to_string(),
            spawn: RuntimeSpawnPoint {
                identifier,
                map_name: name.clone(),
                map_constant: constant,
                group_name: GROUP.into(),
                group_id: 74,
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
        crate::insert_compiled_map_extension(pack, extension)?;
        if !matches!(name.as_str(), "MnRochester" | "MnRochesterPokecenter1F") {
            pack.data
                .runtime_spawn_points
                .remove(&identifier.to_string());
        }
    }
    Ok(())
}
