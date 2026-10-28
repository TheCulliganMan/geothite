//! Relocate existing native anchors and their unchanged program families.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::authored_route_design::RouteDesign;
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

use super::{LOWER, NECK, UPPER};

const NATIVE: [&str; 7] = [
    UPPER,
    LOWER,
    "MnR12C01R00",
    "MnR12C01R01",
    "MnR12C00R02",
    "MnR12C01R02",
    NECK,
];
const STRUCTURAL: [&str; 8] = [
    "id",
    "attributes",
    "blocks",
    "objects",
    "events",
    "map_script_section_commands",
    "map_event_section_commands",
    "scenes",
];

fn owner(tile: [u16; 2]) -> Result<&'static str> {
    ensure!(
        tile[0] < 64 && tile[1] < 64,
        "compact R12 anchor outside extent"
    );
    Ok(if tile[1] < 48 { UPPER } else { LOWER })
}
fn local(tile: [u16; 2]) -> [u16; 2] {
    [tile[0], tile[1] % 48]
}

pub(super) fn redistribute(
    base: &CompiledGamePack,
    blocks: &[u16],
    dossier: &str,
) -> Result<BTreeMap<String, MapModule>> {
    let design = RouteDesign::parse(dossier)?;
    ensure!(
        design.id == "MN_R12"
            && design.dimensions_metatiles == [32, 32]
            && design.chunk_columns == 1
            && design.chunk_rows == 2,
        "compact R12 dossier extent differs"
    );
    let mut destinations = BTreeMap::new();
    for (name, height, row) in [(UPPER, 24, 0), (LOWER, 8, 24)] {
        let mut value = serde_json::to_value(&base.data.maps[name])?;
        for (key, field) in value.as_object_mut().context("native module object")? {
            if STRUCTURAL.contains(&key.as_str()) {
                continue;
            }
            match field {
                Value::Array(v) => v.clear(),
                Value::Object(v) => v.clear(),
                _ => anyhow::bail!("unsupported native program field {key}"),
            }
        }
        let mut map: MapModule = serde_json::from_value(value)?;
        map.attributes.height = height;
        map.blocks = blocks[usize::from(row) * 32..usize::from(row + height) * 32].to_vec();
        map.objects.clear();
        map.events.warps.clear();
        map.events.bg_events.clear();
        map.events.coord_events.clear();
        ensure!(map.scenes.scenes.is_empty(), "R12 has unhandled scenes");
        destinations.insert(name.to_string(), map);
    }
    let originals = NATIVE
        .iter()
        .flat_map(|name| base.data.maps[*name].objects.iter())
        .map(|o| {
            Ok((
                o.object_identifier
                    .clone()
                    .context("source actor identity")?,
                o.clone(),
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    let mut ownership = BTreeMap::new();
    let mut actors = BTreeSet::new();
    let mut place = |id: &str, tile: [u16; 2], movement: Option<&str>| -> Result<()> {
        ensure!(actors.insert(id.to_string()), "duplicate compact actor");
        let mut object = originals
            .get(id)
            .context("compact actor is not an existing native identity")?
            .clone();
        let target = owner(tile)?;
        let [x, y] = local(tile);
        object.x = x;
        object.y = y;
        if let Some(movement) = movement {
            object.spritemovedata = movement.into();
        }
        destinations
            .get_mut(target)
            .context("compact owner")?
            .objects
            .push(object);
        ownership.insert(id.to_string(), target.to_string());
        Ok(())
    };
    for trainer in &design.trainers {
        let original = originals.get(&trainer.id).context("original trainer")?;
        ensure!(
            original.sprite == trainer.sprite && original.radius == u16::from(trainer.sight_range),
            "trainer sprite/sight changed without contract"
        );
        place(&trainer.id, trainer.tile, Some(trainer.facing.movement()))?;
    }
    for resident in &design.residents {
        ensure!(
            originals
                .get(&resident.id)
                .context("original resident")?
                .sprite
                == resident.sprite,
            "resident sprite changed"
        );
        place(
            &resident.id,
            resident.tile,
            Some(resident.facing.movement()),
        )?;
    }
    for item in &design.items {
        let original = originals.get(&item.id).context("original pickup")?;
        ensure!(
            original.event_flag == item.flag,
            "pickup persistent flag changed"
        );
        let pickup = NATIVE
            .iter()
            .flat_map(|n| base.data.maps[*n].script_field_pickups.iter())
            .find(|p| p.source_script == item.id)
            .context("original native pickup")?;
        ensure!(
            pickup.item_id.as_deref() == Some(item.item.as_str())
                && pickup.quantity == item.quantity,
            "native pickup changed"
        );
        place(&item.id, item.tile, None)?;
    }
    ensure!(
        actors == originals.keys().cloned().collect(),
        "an existing R12/camp actor was lost or replaced"
    );
    let old_signs = NATIVE
        .iter()
        .flat_map(|n| base.data.maps[*n].events.bg_events.iter())
        .map(|s| (s.script.clone(), s.clone()))
        .collect::<BTreeMap<_, _>>();
    ensure!(
        design.signs.len() == old_signs.len(),
        "existing route/camp sign omitted"
    );
    for sign in &design.signs {
        let mut event = old_signs
            .get(&sign.id)
            .context("unknown compact sign")?
            .clone();
        let target = owner(sign.tile)?;
        let [x, y] = local(sign.tile);
        event.x = x;
        event.y = y;
        destinations
            .get_mut(target)
            .context("sign owner")?
            .events
            .bg_events
            .push(event);
        ensure!(
            ownership.insert(sign.id.clone(), target.into()).is_none(),
            "sign/actor script identity overlap"
        );
    }
    redistribute_tables(base, &mut destinations, &ownership)?;
    ensure!(
        design.doors.len() == 2,
        "both existing Gunflint rooms must remain"
    );
    for door in &design.doors {
        ensure!(
            ["MnR12Cave1F", "MnR12RangerHut"].contains(&door.target.as_str()),
            "unexpected compact room"
        );
        let name = owner(door.tile)?;
        let module = destinations.get_mut(name).context("door owner")?;
        ensure!(
            usize::from(door.warp_index) == module.events.warps.len() + 1,
            "compact door warp indices must be unique in native owning chunk"
        );
        let constant = base.data.maps[&door.target]
            .attributes
            .map_constant
            .as_ref()
            .context("room constant")?;
        let [x, y] = local(door.tile);
        authoring::warp(module, (x, y), constant, door.target_warp);
        let exterior = module
            .attributes
            .map_constant
            .clone()
            .context("new exterior constant")?;
        let mut room = base.data.maps[&door.target].clone();
        for warp in &mut room.events.warps {
            ensure!(
                ["MN_R12_C01R00", "MN_R12_C01R01"].contains(&warp.target_map_constant.as_str()),
                "room has unexpected destination"
            );
            warp.target_map_constant = exterior.clone();
            warp.target_map = exterior.clone();
            warp.target_warp_id = i16::try_from(door.warp_index)?;
        }
        authoring::refresh_event_section(&mut room);
        destinations.insert(room.id.clone(), room);
    }
    for name in [UPPER, LOWER] {
        authoring::refresh_event_section(
            destinations.get_mut(name).context("compact event owner")?,
        );
    }
    Ok(destinations)
}

fn program_owner(label: &str, ownership: &BTreeMap<String, String>) -> Result<String> {
    let owners = ownership
        .iter()
        .filter(|(root, _)| label.starts_with(root.as_str()))
        .collect::<Vec<_>>();
    ensure!(
        owners.len() == 1,
        "native program label has no unique retained anchor: {label}"
    );
    Ok(owners[0].1.clone())
}
fn redistribute_tables(
    base: &CompiledGamePack,
    destinations: &mut BTreeMap<String, MapModule>,
    ownership: &BTreeMap<String, String>,
) -> Result<()> {
    let mut values = destinations
        .iter()
        .map(|(n, m)| Ok((n.clone(), serde_json::to_value(m)?)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    for name in NATIVE {
        let native = &base.data.maps[name];
        ensure!(
            native.script_map_commands.is_empty()
                && native.script_object_commands.is_empty()
                && native.script_block_changes.is_empty()
                && native.script_movements.is_empty()
                && native.events.coord_events.is_empty(),
            "native route has unsupported coordinate programs"
        );
        let source = serde_json::to_value(native)?;
        for (field, value) in source.as_object().context("program source object")? {
            if STRUCTURAL.contains(&field.as_str()) {
                continue;
            }
            match value {
                Value::Object(entries) => {
                    for (label, entry) in entries {
                        let target = program_owner(label, ownership)?;
                        ensure!(
                            values.get_mut(&target).context("program target")?[field]
                                .as_object_mut()
                                .context("program table object")?
                                .insert(label.clone(), entry.clone())
                                .is_none(),
                            "duplicate native program label"
                        );
                    }
                }
                Value::Array(entries) => {
                    for entry in entries {
                        let script = entry["source_script"]
                            .as_str()
                            .context("native command lacks source script")?;
                        let target = program_owner(script, ownership)?;
                        values.get_mut(&target).context("command target")?[field]
                            .as_array_mut()
                            .context("program table array")?
                            .push(entry.clone());
                    }
                }
                _ => anyhow::bail!("unsupported program table representation: {field}"),
            }
        }
    }
    for (name, value) in values {
        destinations.insert(name, serde_json::from_value(value)?);
    }
    Ok(())
}
