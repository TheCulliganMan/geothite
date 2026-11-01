//! Additive authored-city identities with one explicitly rebound canonical gym.
use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::{
    CompiledGamePack, CompiledMapExtension, MapModule, PokegearLandmark, RuntimeMapMetadata,
    RuntimeSpawnPoint,
};

pub(crate) fn replace(pack: &mut CompiledGamePack, map: MapModule) -> Result<()> {
    let constant = map
        .attributes
        .map_constant
        .as_ref()
        .context("map constant")?;
    if let Some(old) = pack.data.maps.get(&map.id) {
        for label in old.scripts.keys().chain(old.script_text_bodies.keys()) {
            let shared = pack
                .data
                .maps
                .values()
                .filter(|m| {
                    m.id != map.id
                        && (m.scripts.contains_key(label)
                            || m.script_text_bodies.contains_key(label))
                })
                .collect::<Vec<_>>();
            if shared.is_empty() {
                pack.data.map_scripts.remove(label);
            } else {
                // An existing common helper can stay shared when every native
                // definition is identical and this publication leaves it intact.
                ensure!(
                    map.scripts.get(label) == old.scripts.get(label)
                        && map.script_text_bodies.get(label) == old.script_text_bodies.get(label)
                        && shared
                            .iter()
                            .all(|other| other.scripts.get(label) == old.scripts.get(label)
                                && other.script_text_bodies.get(label)
                                    == old.script_text_bodies.get(label)),
                    "shared script must remain identical and unchanged: {label}"
                );
            }
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

pub(crate) struct CityIdentity<'a> {
    pub group: &'a str,
    pub group_id: i16,
    pub landmark: &'a str,
    /// Existing geographic area used when the native seven-bit table is full.
    pub fallback_landmark: Option<&'a str>,
    pub constant: &'a str,
    pub name: &'a str,
    pub position: [u8; 2],
    pub manifest: &'a str,
    pub town: &'a str,
    pub clinic: &'a str,
    pub gym: &'a str,
}

pub(crate) fn publish(
    pack: &mut CompiledGamePack,
    modules: Vec<(MapModule, [u16; 2])>,
    city: CityIdentity<'_>,
) -> Result<()> {
    let group = city.group;
    let group_id = u16::try_from(city.group_id).context("nonnegative authored city group")?;
    ensure!(
        !pack
            .data
            .runtime_map_metadata
            .values()
            .any(|m| m.group_id == group_id)
            && !pack
                .data
                .story_event_script_constants
                .global
                .contains_key(&format!("MAPGROUP_{group}"))
            && !pack
                .data
                .pokegear_landmarks
                .landmarks
                .iter()
                .any(|l| l.constant == city.landmark),
        "authored city identity already occupied"
    );
    pack.data
        .story_event_script_constants
        .global
        .insert(format!("MAPGROUP_{group}"), i64::from(city.group_id));
    let available = (1..=0x7f).find(|id| {
        !pack
            .data
            .pokegear_landmarks
            .landmarks
            .iter()
            .any(|l| l.id == *id)
    });
    let landmark_constant = if let Some(id) = available {
        pack.data
            .pokegear_landmarks
            .landmarks
            .push(PokegearLandmark {
                id,
                constant: city.landmark.into(),
                label: city.constant.into(),
                name: city.name.into(),
                x: i16::from(city.position[0]),
                y: i16::from(city.position[1]),
                region: "JOHTO".into(),
            });
        city.landmark
    } else {
        let fallback = city
            .fallback_landmark
            .context("native landmark table is full; an explicit geographic area is required")?;
        ensure!(
            pack.data
                .pokegear_landmarks
                .landmarks
                .iter()
                .any(|l| l.constant == fallback && l.id <= 0x7f),
            "city fallback must name an existing native caught-location area"
        );
        fallback
    };
    let first = pack
        .data
        .runtime_spawn_points
        .values()
        .map(|s| s.identifier)
        .max()
        .context("spawn table")?
        .checked_add(1)
        .context("recovery overflow")?
        .max(256);
    for (index, (mut module, [x, y])) in modules.into_iter().enumerate() {
        let name = module.id.clone();
        let constant = module
            .attributes
            .map_constant
            .clone()
            .context("authored city constant")?;
        module.attributes.location = Some(landmark_constant.into());
        if name == city.gym {
            pack.data
                .pokegear_landmarks
                .map_to_landmark
                .insert(name, landmark_constant.into());
            replace(pack, module)?;
            continue;
        }
        ensure!(
            !pack.data.maps.contains_key(&name),
            "authored city map already installed: {name}"
        );
        module.attributes.map_group_constant = Some(format!("MAPGROUP_{group}"));
        let map_id = u16::try_from(index + 1)?;
        let identifier = first
            .checked_add(u16::try_from(index)?)
            .context("recovery overflow")?;
        pack.data
            .pokegear_landmarks
            .map_to_landmark
            .insert(name.clone(), landmark_constant.into());
        let extension = CompiledMapExtension {
            manifest_id: format!("{}-{index}", city.manifest),
            map_name: name.clone(),
            map_constant: constant.clone(),
            metadata: RuntimeMapMetadata {
                name: name.clone(),
                constant: constant.clone(),
                group_name: group.into(),
                group_id,
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
                group_name: group.into(),
                group_id: city.group_id,
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
        if name != city.town && name != city.clinic {
            pack.data
                .runtime_spawn_points
                .remove(&identifier.to_string());
        }
    }
    Ok(())
}
