//! Sparse optional native bank trainers and map-local encounter slots.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use crystal_core::world::encounters::{WildEncounter, WildEncounterData, WildEncounterTable};
use serde_json::json;

use crate::authored_river_connections_design::Manifest;
use crate::authored_route_design::{Facing, RouteTrainer};
use crate::authored_town_geometry::kind;
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

pub(crate) fn install(
    pack: &mut CompiledGamePack,
    maps: &mut BTreeMap<String, MapModule>,
    plan: &Manifest,
) -> Result<()> {
    let new_names = plan
        .new_maps
        .iter()
        .map(|m| (m.map_constant.clone(), m.id.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut ids = BTreeSet::new();
    let mut positions = BTreeSet::new();
    let mut flags = Vec::new();
    for actor in &plan.objects {
        ensure!(
            ids.insert(&actor.id)
                && positions.insert((&actor.map, actor.core))
                && new_names.values().any(|n| n == &actor.map)
                && actor.id.starts_with("MN_RIVER_")
                && actor.item.is_none()
                && actor.quantity.is_none()
                && actor.native_special.is_none()
                && !actor.text.is_empty()
                && actor
                    .text
                    .iter()
                    .all(|s| !s.is_empty() && s.chars().count() <= 18),
            "unique sparse new river actor contract"
        );
        let map = maps.get(&actor.map).context("new river actor map")?;
        ensure!(
            kind(pack, map, actor.core)? == "FLOOR",
            "river actor needs native Floor"
        );
        ensure!(
            pack.runtime_files.contains_key(&format!(
                "gfx/sprites/{}.png",
                actor
                    .sprite
                    .trim_start_matches("SPRITE_")
                    .to_ascii_lowercase()
            )),
            "unknown river sprite"
        );
        if actor.role == "optional_trainer" {
            flags.push(actor.event_flag.clone().context("river trainer flag")?);
        } else {
            ensure!(
                actor.role == "dialogue"
                    && actor.event_flag.is_none()
                    && actor.trainer_class.is_none()
                    && actor.trainer_name.is_none()
                    && actor.party.is_empty(),
                "unsupported river activity"
            );
        }
    }
    crate::authored_city_expansion_events::flags(pack, &flags)?;
    for actor in &plan.objects {
        let facing: Facing = serde_json::from_value(json!(actor.facing))?;
        if actor.role == "optional_trainer" {
            let class = actor
                .trainer_class
                .as_ref()
                .context("river trainer class")?;
            let global = &pack.data.story_event_script_constants.global;
            let next = pack
                .data
                .trainers
                .trainers
                .values()
                .filter(|t| &t.trainer_class == class)
                .filter_map(|t| global.get(&t.trainer_id))
                .copied()
                .max()
                .context("existing native trainer class namespace")?
                .checked_add(1)
                .context("river trainer constant overflow")?;
            ensure!(
                next <= 255 && !global.contains_key(&actor.id),
                "river trainer native identity overflow/collision"
            );
            pack.data
                .story_event_script_constants
                .global
                .insert(actor.id.clone(), next);
            let trainer: RouteTrainer = serde_json::from_value(json!({
                "id":actor.id,"display_name":actor.trainer_name.as_ref().context("river trainer name")?,
                "trainer_class":class,"sprite":actor.sprite,"tile":actor.core,"facing":actor.facing,
                "sight_range":0,"defeated_flag":actor.event_flag,"party":actor.party.iter().map(|p| json!({"species":p.species,"level":p.level})).collect::<Vec<_>>(),
                "before_lines":actor.text,"after_lines":["Safe travels!"],"loss_lines":["A strong current!"]
            }))?;
            crate::authored_city_expansion_events::activities(
                pack,
                maps.get_mut(&actor.map).context("river trainer map")?,
                &[trainer],
                &[],
            )?;
        } else {
            let map = maps.get_mut(&actor.map).context("river speaker map")?;
            let label = format!("{}Text", actor.id);
            authoring::text(
                map,
                &label,
                &actor.text.iter().map(String::as_str).collect::<Vec<_>>(),
            );
            map.scripts.insert(
                actor.id.clone(),
                json!([authoring::command("jumptextfaceplayer", &[&label])]),
            );
            authoring::npc(
                pack,
                map,
                &actor.id,
                &actor.sprite,
                i16::try_from(actor.core[0])?,
                i16::try_from(actor.core[1])?,
            )?;
        }
        let object = maps
            .get_mut(&actor.map)
            .context("river actor map")?
            .objects
            .last_mut()
            .context("new river actor")?;
        object.object_identifier = Some(actor.id.clone());
        object.spritemovedata = facing.movement().into();
    }
    let mut names = pack
        .data
        .map_attributes
        .iter()
        .filter_map(|(n, a)| a.map_constant.as_ref().map(|c| (c.clone(), n.clone())))
        .collect::<BTreeMap<_, _>>();
    names.extend(new_names);
    for source in &plan.new_maps {
        let map = maps.get_mut(&source.id).context("river source map")?;
        crate::authored_city_expansion_events::finish(pack, map, &names)?;
    }
    Ok(())
}

pub(crate) fn encounters(
    pack: &CompiledGamePack,
    plan: &Manifest,
    name: &str,
) -> Result<Option<WildEncounterData>> {
    let mut data = WildEncounterData {
        map_name: name.into(),
        ..Default::default()
    };
    let mut surfaces = BTreeSet::new();
    for contract in &plan.encounter_contracts {
        let targets = contract
            .map
            .iter()
            .chain(&contract.maps)
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        ensure!(
            !targets.is_empty()
                && targets
                    .iter()
                    .all(|n| plan.new_maps.iter().any(|m| m.id == *n)),
            "encounter contract must target only new river maps"
        );
        if !targets.contains(name) {
            continue;
        }
        ensure!(
            surfaces.insert(if contract.terrain == "surf" {
                "water"
            } else {
                contract.terrain.as_str()
            }),
            "duplicate river encounter surface"
        );
        let surface = if contract.terrain == "surf" {
            "water"
        } else {
            contract.terrain.as_str()
        };
        let expected: &[u8] = match surface {
            "grass" => &[30, 30, 20, 10, 5, 4, 1],
            "water" => &[60, 30, 10],
            _ => anyhow::bail!("unsupported river encounter surface"),
        };
        ensure!(
            contract.slots.iter().map(|s| s.weight).collect::<Vec<_>>() == expected,
            "river slots must use native per-slot weights"
        );
        let native = pack
            .data
            .encounter_slot_tables
            .tables
            .get(surface)
            .context("native encounter chooser")?;
        let mut counts = vec![0u8; expected.len()];
        for roll in 0..100u8 {
            let slot = native
                .iter()
                .find(|c| roll < c.threshold)
                .context("native encounter threshold")?
                .slot;
            let count = counts
                .get_mut(slot)
                .context("native encounter slot index")?;
            *count += 1;
        }
        ensure!(
            counts == expected,
            "native encounter chooser differs from declared weights"
        );
        let slots = contract
            .slots
            .iter()
            .map(|s| {
                ensure!(
                    pack.data.pokemon.contains_key(&s.species) && (1..=100).contains(&s.level),
                    "unknown river encounter species/level"
                );
                Ok(WildEncounter {
                    species: s.species.clone(),
                    level: s.level,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let table = WildEncounterTable {
            morning: slots.clone(),
            day: slots.clone(),
            night: slots,
        };
        if surface == "grass" {
            data.grass = Some(table);
            data.grass_rates = Some(
                ["morning", "day", "night"]
                    .map(|t| (t.into(), 10))
                    .into_iter()
                    .collect(),
            );
        } else {
            data.water = Some(table);
            data.water_rate = Some(4);
        }
    }
    Ok((!surfaces.is_empty()).then_some(data))
}
