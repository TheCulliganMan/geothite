//! Active regional species for an explicitly touched approach, with private rod tables.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::northstar_regional_roster::RegionalRoster;
use crate::{CompiledGamePack, MapModule};

pub(crate) struct FishingBinding<'a> {
    pub expected_group: &'a str,
    pub group: &'a str,
    pub time_prefix: &'a str,
    pub swarm_rule: Option<&'a str>,
}

pub(crate) fn policy() -> Result<RegionalRoster> {
    RegionalRoster::parse(
        include_str!("../../../planning/minnesota/story-pokemon.json"),
        include_str!("../../../planning/minnesota/pokemon/roster.json"),
        include_str!("../../../planning/minnesota/pokemon/roster-curation.json"),
        include_str!("../../../planning/minnesota/pokemon/species-profiles.json"),
    )
}

pub(crate) fn install(
    pack: &mut CompiledGamePack,
    road: &mut MapModule,
    binding: FishingBinding<'_>,
) -> Result<()> {
    let policy = policy()?;
    for script in road.trainer_scripts.values() {
        let mut trainer = pack
            .data
            .trainers
            .trainers
            .get(&script.trainer_id)
            .context("regional approach trainer")?
            .clone();
        for member in &mut trainer.party {
            let target = policy.target(pack, &member.species, Some(member.level))?;
            if target != member.species {
                member.moves = pack
                    .data
                    .create_pokemon(&target, member.level, member.dvs)?
                    .moves;
                member.species = target;
            }
        }
        *pack
            .data
            .trainers
            .trainers
            .get_mut(&script.trainer_id)
            .context("existing regional approach trainer")? = trainer;
    }
    if let Some(old) = pack.data.wild_encounters.get(&road.id) {
        let mut value = serde_json::to_value(old)?;
        encounters(pack, &policy, &mut value)?;
        pack.data
            .wild_encounters
            .insert(road.id.clone(), serde_json::from_value(value)?);
    }
    if let Some(old) = pack.data.field_encounters.get(&road.id) {
        let mut value = serde_json::to_value(old)?;
        encounters(pack, &policy, &mut value)?;
        pack.data
            .field_encounters
            .insert(road.id.clone(), serde_json::from_value(value)?);
    }
    let original = road
        .attributes
        .fishing_group
        .as_deref()
        .context("regional approach fishing group")?;
    ensure!(
        original == binding.expected_group,
        "original regional approach lake fishing required"
    );
    let swarms = pack
        .data
        .fishing
        .swarm_rules
        .values()
        .filter(|rule| rule.base_group == original)
        .cloned()
        .collect::<Vec<_>>();
    ensure!(
        swarms.len() <= 1 && swarms.is_empty() == binding.swarm_rule.is_none(),
        "the approach must preserve its exact native swarm behavior"
    );
    let mut groups = vec![(original.to_owned(), binding.group.to_owned())];
    if let Some(swarm) = swarms.first() {
        groups.push((
            swarm.swarm_group.clone(),
            format!("{}_SWARM", binding.group),
        ));
    }
    let mut times = BTreeMap::new();
    for (original, name) in &groups {
        let mut group = pack
            .data
            .fishing
            .groups
            .get(original)
            .context("original rod tables")?
            .clone();
        group.source_index = pack
            .data
            .fishing
            .groups
            .values()
            .map(|g| g.source_index)
            .max()
            .context("native fish indices")?
            .checked_add(1)
            .context("fish index overflow")?;
        for table in group.rod_tables.values_mut() {
            for slot in &mut table.slots {
                if let Some(species) = &slot.species {
                    slot.species = Some(policy.target(pack, species, Some(slot.level))?);
                }
                if let Some(original) = &slot.time_group {
                    let name = if let Some(name) = times.get(original) {
                        String::clone(name)
                    } else {
                        let mut time = serde_json::to_value(
                            pack.data
                                .fishing
                                .time_groups
                                .get(original)
                                .context("original timed fish")?,
                        )?;
                        encounters(pack, &policy, &mut time)?;
                        let name = format!("{}{original}", binding.time_prefix);
                        ensure!(
                            pack.data
                                .fishing
                                .time_groups
                                .insert(name.clone(), serde_json::from_value(time)?)
                                .is_none(),
                            "occupied regional approach time fish"
                        );
                        times.insert(original.clone(), name.clone());
                        name
                    };
                    slot.time_group = Some(name);
                }
            }
        }
        ensure!(
            pack.data
                .fishing
                .groups
                .insert(name.clone(), group)
                .is_none(),
            "occupied regional approach fishing group"
        );
    }
    if let (Some(mut swarm), Some(name)) = (swarms.into_iter().next(), binding.swarm_rule) {
        swarm.base_group = binding.group.into();
        swarm.swarm_group = format!("{}_SWARM", binding.group);
        ensure!(
            pack.data
                .fishing
                .swarm_rules
                .insert(name.into(), swarm)
                .is_none(),
            "occupied regional approach swarm"
        );
    }
    road.attributes.fishing_group = Some(binding.group.into());
    Ok(())
}

fn encounters(pack: &CompiledGamePack, policy: &RegionalRoster, value: &mut Value) -> Result<()> {
    match value {
        Value::Object(object) => {
            for (species, level) in [
                ("species", "level"),
                ("day_species", "day_level"),
                ("night_species", "night_level"),
            ] {
                if let Some(original) = object.get(species).and_then(Value::as_str) {
                    let level = object
                        .get(level)
                        .and_then(Value::as_u64)
                        .map(u8::try_from)
                        .transpose()?;
                    object.insert(
                        species.into(),
                        Value::String(policy.target(pack, original, level)?),
                    );
                }
            }
            for child in object.values_mut() {
                encounters(pack, policy, child)?;
            }
        }
        Value::Array(array) => {
            for child in array {
                encounters(pack, policy, child)?;
            }
        }
        _ => {}
    }
    Ok(())
}
