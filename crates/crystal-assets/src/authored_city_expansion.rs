//! Expand installed cities without reinstalling their story or original rooms.

use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::authored_city_expansion_anchors as anchors;
use crate::authored_city_expansion_design::{Expansion, Room};
use crate::authored_city_expansion_events as events;
use crate::authored_route_layout::Layout;
use crate::authored_town_publication;
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, CompiledMapExtension, MapModule, RuntimeSpawnPoint};

#[cfg(test)]
#[path = "authored_city_expansion_tests.rs"]
mod tests;

/// Expands one installed Minnesota city from explicit terrain and event inputs.
///
/// The contract certifies the complete native city preimage. Existing anchors
/// translate or move by exact indexed overrides; old warp indices, room modules,
/// compiled story programs, gyms and identities remain intact. New activities
/// compile through the native parsers. Reciprocal offsets and exterior recovery
/// coordinates move with the city. This returns an in-memory pack; it neither
/// writes files nor migrates existing saves or establishes visual acceptance.
///
/// # Errors
/// Returns an error for stale native preimages, unsupported extents, invalid
/// art/anchors/connections, conflicting new identities, unsafe geometry or native
/// verification failure. Borrowed inputs remain unchanged on failure.
pub fn expand_authored_city(
    base: &CompiledGamePack,
    layout_json: &str,
    contract_json: &str,
) -> Result<CompiledGamePack> {
    let plan: Expansion = serde_json::from_str(contract_json).context("city expansion contract")?;
    ensure!(
        plan.schema_version == 1
            && matches!(
                plan.city.as_str(),
                "MnMinneapolis" | "MnStPaul" | "MnDuluth"
            ),
        "unsupported installed city expansion"
    );
    ensure!(
        plan.old_dimensions_metatiles == [32, 24] && plan.dimensions_metatiles == [64, 48],
        "city expansion extent"
    );
    let original = base.data.maps.get(&plan.city).context("installed city")?;
    let hash = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&serde_json::to_value(original)?)?)
    );
    ensure!(
        hash == plan.native_module_sha256,
        "installed city module preimage differs: {hash}"
    );
    ensure!(
        [original.attributes.width, original.attributes.height] == plan.old_dimensions_metatiles,
        "installed city dimensions differ"
    );
    let layout: Layout = serde_json::from_str(layout_json).context("explicit city layout")?;
    ensure!(
        layout.schema_version == 2
            && layout.id == plan.city
            && layout.dimensions_metatiles == plan.dimensions_metatiles,
        "city layout identity/extent differs"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let mut pack = crate::authored_city_expansion_art::install(
        base,
        layout_json,
        &layout.compiled_tileset,
        &original.attributes.tileset_name,
        &plan.city,
    )?;
    events::flags(&mut pack, &plan.event_flags)?;
    let mut city = original.clone();
    anchors::relocate(&mut city, &plan)?;
    city.attributes.width = layout.dimensions_metatiles[0];
    city.attributes.height = layout.dimensions_metatiles[1];
    city.attributes.tileset_name = layout.compiled_tileset.clone();
    city.blocks = layout.compile()?;
    let mut names = pack
        .data
        .runtime_map_metadata
        .values()
        .map(|m| (m.constant.clone(), m.name.clone()))
        .collect::<BTreeMap<_, _>>();
    for room in &plan.rooms {
        ensure!(
            !pack.data.maps.contains_key(&room.id) && !names.contains_key(&room.native_constant),
            "occupied expansion room {}",
            room.id
        );
        ensure!(
            names
                .insert(room.native_constant.clone(), room.id.clone())
                .is_none(),
            "duplicate expansion room constant"
        );
    }
    let mut additions = authoring::blank(
        &pack,
        &plan.city,
        &plan.city,
        original
            .attributes
            .map_constant
            .as_deref()
            .context("city constant")?,
    )?;
    events::ordinary(
        &mut pack,
        &mut additions,
        &plan.residents,
        &plan.signs,
        &plan.clinic_constant,
        &plan.default_mart,
    )?;
    events::activities(&mut pack, &mut additions, &plan.trainers, &plan.items)?;
    additions.scripts.extend(plan.programs.clone());
    events::finish(&pack, &mut additions, &names)?;
    for label in additions
        .scripts
        .keys()
        .chain(additions.script_text_bodies.keys())
    {
        ensure!(
            !pack.data.map_scripts.contains_key(label),
            "occupied native city activity label {label}"
        );
    }
    merge_new_content(&mut city, additions)?;
    ensure!(
        city.objects.len() <= usize::from(u8::MAX),
        "city map-event identities exceed native byte addressing"
    );
    revise_text(&mut city, &plan)?;
    for door in &plan.doors {
        ensure!(
            usize::from(door.warp_index) == city.events.warps.len() + 1,
            "new city doors must append after old warp indices"
        );
        let constant = names
            .iter()
            .find(|(_, name)| *name == &door.target)
            .map(|(constant, _)| constant)
            .context("new city door target")?;
        authoring::warp(
            &mut city,
            (door.tile[0], door.tile[1]),
            constant,
            door.target_warp,
        );
        let index = door.return_warp_index.context("new city passive return")?;
        ensure!(
            usize::from(index) == city.events.warps.len() + 1,
            "city passive return ordering"
        );
        let constant = city
            .attributes
            .map_constant
            .clone()
            .context("city constant")?;
        authoring::warp(
            &mut city,
            (door.return_tile[0], door.return_tile[1]),
            &constant,
            i16::try_from(index)?,
        );
    }
    anchors::connections(&mut pack, &mut city, &plan)?;
    anchors::spawns(&mut pack, &plan)?;
    crate::authored_city_expansion_geometry::validate_relocations(base, original, &pack, &city)?;
    authoring::refresh_event_section(&mut city);
    authored_town_publication::replace(&mut pack, city)?;
    for room in &plan.rooms {
        let map = build_room(&mut pack, &plan, room, &names)?;
        publish_room(&mut pack, &plan, room, map)?;
    }
    register_clinic_recovery(&mut pack, &plan)?;
    anchors::validate_connections(&pack, &plan)?;
    crate::authored_city_expansion_geometry::validate(&pack, &plan)?;
    pack.report.maps = pack.data.maps.len();
    pack.report.reachable_maps.sort();
    pack.report.solvable_maps.sort();
    pack.report
        .manifests
        .push(format!("{}-expanded-city", plan.city.to_ascii_lowercase()));
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

fn register_clinic_recovery(pack: &mut CompiledGamePack, plan: &Expansion) -> Result<()> {
    let Some([x, y]) = plan.clinic_recovery_core else {
        return Ok(());
    };
    ensure!(
        plan.city == "MnDuluth",
        "only the unregistered Duluth clinic needs recovery registration"
    );
    ensure!(
        !pack
            .data
            .runtime_spawn_points
            .values()
            .any(|spawn| spawn.map_constant == plan.clinic_constant),
        "city clinic already has a recovery record"
    );
    let metadata = pack
        .data
        .runtime_map_metadata
        .get(&plan.clinic_constant)
        .context("native city clinic metadata")?;
    ensure!(
        crate::authored_town_geometry::kind(pack, &pack.data.maps[&metadata.name], [x, y])?
            == "FLOOR",
        "clinic recovery must be safe ordinary floor"
    );
    let identifier = pack
        .data
        .runtime_spawn_points
        .values()
        .map(|spawn| spawn.identifier)
        .max()
        .context("native recovery catalog")?
        .checked_add(1)
        .context("clinic recovery overflow")?;
    pack.data.runtime_spawn_points.insert(
        identifier.to_string(),
        RuntimeSpawnPoint {
            identifier,
            map_name: metadata.name.clone(),
            map_constant: metadata.constant.clone(),
            group_name: metadata.group_name.clone(),
            group_id: i16::try_from(metadata.group_id)?,
            map_id: i16::try_from(metadata.map_id)?,
            tile_x: i16::try_from(x)?,
            tile_y: i16::try_from(y)?,
            metatile_x: i16::try_from(x / 2)?,
            metatile_y: i16::try_from(y / 2)?,
            subtile_x: i16::try_from(x % 2)?,
            subtile_y: i16::try_from(y % 2)?,
        },
    );
    Ok(())
}

fn merge_new_content(city: &mut MapModule, new: MapModule) -> Result<()> {
    let mut old_value = serde_json::to_value(&*city)?;
    let new_value = serde_json::to_value(&new)?;
    let old_fields = old_value.as_object_mut().context("native city object")?;
    for (key, incoming) in new_value.as_object().context("new city content")? {
        if matches!(
            key.as_str(),
            "id" | "attributes"
                | "blocks"
                | "objects"
                | "events"
                | "scenes"
                | "map_script_section_commands"
                | "map_event_section_commands"
        ) {
            continue;
        }
        let existing = old_fields.get_mut(key).context("native city field")?;
        match (existing, incoming) {
            (Value::Array(old), Value::Array(new)) => old.extend(new.iter().cloned()),
            (Value::Object(old), Value::Object(new)) => {
                for (label, value) in new {
                    ensure!(
                        !old.contains_key(label),
                        "new city content overwrites old label {label}"
                    );
                    old.insert(label.clone(), value.clone());
                }
            }
            _ => anyhow::bail!("unsupported city content field {key}"),
        }
    }
    let mut merged: MapModule = serde_json::from_value(old_value)?;
    merged.objects.extend(new.objects);
    merged.events.bg_events.extend(new.events.bg_events);
    *city = merged;
    Ok(())
}

fn revise_text(city: &mut MapModule, plan: &Expansion) -> Result<()> {
    for revision in &plan.text_revisions {
        ensure!(
            plan.city == "MnStPaul",
            "only St Paul obsolete fair wayfinding is authorized"
        );
        ensure!(
            matches!(
                revision.label.as_str(),
                "MN_ST_PAUL_FAIR_STEWARDText"
                    | "MN_ST_PAUL_FAIR_GUARD_WESTText"
                    | "MN_ST_PAUL_FAIR_GUARD_EASTText"
                    | "MN_ST_PAUL_SIGN_FAIRText"
            ),
            "unapproved old city text revision"
        );
        let old = city
            .scripts
            .get(&revision.label)
            .context("wayfinding text label")?;
        let lines = old
            .as_array()
            .context("native text body")?
            .iter()
            .filter_map(|command| {
                matches!(
                    command["command"].as_str(),
                    Some("text" | "line" | "cont" | "para")
                )
                .then(|| {
                    command["args"][0]
                        .as_str()
                        .map(|line| line.trim_matches('"').to_owned())
                })
                .flatten()
            })
            .collect::<Vec<_>>();
        ensure!(
            lines == revision.before,
            "stale wayfinding text {}",
            revision.label
        );
        authoring::text(
            city,
            &revision.label,
            &revision
                .after
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        );
        let only = BTreeMap::from([(
            revision.label.clone(),
            city.scripts[&revision.label].clone(),
        )]);
        let parsed = crate::parse_script_text_bodies(&city.id, &only)?;
        city.script_text_bodies
            .insert(revision.label.clone(), parsed[&revision.label].clone());
    }
    Ok(())
}

fn build_room(
    pack: &mut CompiledGamePack,
    plan: &Expansion,
    room: &Room,
    names: &BTreeMap<String, String>,
) -> Result<MapModule> {
    let mut map = authoring::blank(pack, &room.template, &room.id, &room.native_constant)?;
    authoring::blocks(
        &mut map,
        room.dimensions_metatiles[0],
        room.dimensions_metatiles[1],
        &room.rows.join(" "),
    )?;
    map.attributes.tileset_name = room.tileset.clone();
    map.attributes.border_block = room.border_block;
    map.attributes.map_group_constant = pack.data.maps[&plan.city]
        .attributes
        .map_group_constant
        .clone();
    map.attributes.location = pack.data.maps[&plan.city].attributes.location.clone();
    for warp in &room.warps {
        ensure!(
            usize::from(warp.index) == map.events.warps.len() + 1
                && names.contains_key(&warp.target),
            "room warp order/target {}",
            room.id
        );
        authoring::warp(
            &mut map,
            (warp.tile[0], warp.tile[1]),
            &warp.target,
            warp.target_warp,
        );
    }
    events::ordinary(
        pack,
        &mut map,
        &room.residents,
        &room.signs,
        &plan.clinic_constant,
        &plan.default_mart,
    )?;
    events::activities(pack, &mut map, &room.trainers, &room.items)?;
    ensure!(
        map.objects.len() <= usize::from(u8::MAX),
        "room map-event identities exceed native byte addressing"
    );
    map.scripts.extend(room.programs.clone());
    events::finish(pack, &mut map, names)?;
    Ok(map)
}

fn publish_room(
    pack: &mut CompiledGamePack,
    plan: &Expansion,
    room: &Room,
    map: MapModule,
) -> Result<()> {
    let city_constant = pack.data.maps[&plan.city]
        .attributes
        .map_constant
        .as_ref()
        .context("city constant")?;
    let mut metadata = pack.data.runtime_map_metadata[city_constant].clone();
    metadata.name = room.id.clone();
    metadata.constant = room.native_constant.clone();
    metadata.width = map.attributes.width;
    metadata.height = map.attributes.height;
    metadata.environment = map
        .attributes
        .environment
        .clone()
        .context("room environment")?;
    metadata.phone_service = map.attributes.phone_service;
    metadata.map_id = (1..=u8::MAX)
        .find(|id| {
            !pack
                .data
                .runtime_map_metadata
                .values()
                .any(|m| m.group_id == metadata.group_id && m.map_id == u16::from(*id))
        })
        .map(u16::from)
        .context("city map group full")?;
    let identifier = pack
        .data
        .runtime_spawn_points
        .values()
        .map(|s| s.identifier)
        .max()
        .context("native spawn catalog")?
        .checked_add(1)
        .context("temporary room spawn overflow")?;
    let [x, y] = room.arrival_core.map(i16::try_from);
    let (x, y) = (x?, y?);
    let extension = CompiledMapExtension {
        manifest_id: format!("{}-city-room", room.id.to_ascii_lowercase()),
        map_name: room.id.clone(),
        map_constant: room.native_constant.clone(),
        module: map,
        spawn_key: identifier.to_string(),
        spawn: RuntimeSpawnPoint {
            identifier,
            map_name: room.id.clone(),
            map_constant: room.native_constant.clone(),
            group_name: metadata.group_name.clone(),
            group_id: i16::try_from(metadata.group_id)?,
            map_id: i16::try_from(metadata.map_id)?,
            tile_x: x,
            tile_y: y,
            metatile_x: x / 2,
            metatile_y: y / 2,
            subtile_x: x % 2,
            subtile_y: y % 2,
        },
        metadata,
        wild_encounters: None,
        start_new_game_here: false,
    };
    crate::validate_compiled_map_extension(&extension)?;
    crate::insert_compiled_map_extension(pack, extension)?;
    pack.data
        .runtime_spawn_points
        .remove(&identifier.to_string());
    let landmark = pack.data.maps[&plan.city]
        .attributes
        .location
        .clone()
        .context("city landmark")?;
    pack.data
        .pokegear_landmarks
        .map_to_landmark
        .insert(room.id.clone(), landmark);
    Ok(())
}
