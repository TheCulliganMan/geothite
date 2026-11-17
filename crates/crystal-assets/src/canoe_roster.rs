//! Repair retired species on the canoe's touched maps and in the Safari expedition.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::northstar_regional_cast::rewrite_encounters;
use crate::northstar_regional_roster::RegionalRoster;

pub(crate) fn install(base: &CompiledGamePack, pack: &mut CompiledGamePack) -> Result<()> {
    let policy = RegionalRoster::current()?.with_bindings(&[
        ("AUGRUB", "BURRBUD"),
        ("BOBCAST", "PINGLET"),
        ("BUOYOSTIC", "PADDARRAY"),
        ("DENSY", "BRISTUB"),
        ("DENUMBRA", "CROOKVEIL"),
        ("DOZEBEAR", "LATCHQUILL"),
        ("DRIFTILE", "RILLBOW"),
        ("GILLPUP", "CASTOROCK"),
        ("GLACIRIS", "FENCICLE"),
        ("GOOSNIDE", "DARTDUCK"),
        ("GOUGECREST", "SILOHAWK"),
        ("HELIKNOCK", "LASSERP"),
        ("MUDMENDER", "PIKEMAN"),
        ("NOTCHBILL", "VANECHICK"),
        ("RIVVIT", "CHORIP"),
        ("SAMURREL", "SQUEAKURD"),
        ("SLIDEWAKE", "PADDARRAY"),
        ("SOLSTORT", "BARBELISK"),
        ("TICKLET", "SCUTLET"),
        ("TIPULORD", "SKEETEREX"),
        ("WINKFIN", "JIGLET"),
    ])?;
    let maps: BTreeSet<_> = pack
        .data
        .maps
        .iter()
        .filter(|(name, map)| base.data.maps.get(*name) != Some(*map))
        .map(|(name, _)| name.clone())
        .collect();
    let mut trainer_ids = BTreeSet::new();
    let mut rod_groups = BTreeSet::new();
    for name in &maps {
        let map = &pack.data.maps[name];
        trainer_ids.extend(map.trainer_scripts.values().map(|s| s.trainer_id.clone()));
        for command in map
            .scripts
            .values()
            .filter_map(|script| script.as_array())
            .flatten()
        {
            if command["command"] == "loadtrainer"
                && let Some(id) = command["args"]
                    .as_array()
                    .and_then(|args| args.last())
                    .and_then(|id| id.as_str())
            {
                trainer_ids.insert(id.into());
            }
        }
        if let Some(group) = pack.data.map_fishing_group(name)? {
            rod_groups.insert(group.to_owned());
        }
        if let Some(table) = pack.data.wild_encounters.get(name) {
            let mut value = serde_json::to_value(table)?;
            rewrite_encounters(pack, &policy, &mut value)?;
            pack.data
                .wild_encounters
                .insert(name.clone(), serde_json::from_value(value)?);
        }
        if let Some(table) = pack.data.field_encounters.get(name) {
            let mut value = serde_json::to_value(table)?;
            rewrite_encounters(pack, &policy, &mut value)?;
            pack.data
                .field_encounters
                .insert(name.clone(), serde_json::from_value(value)?);
        }
    }
    for id in trainer_ids {
        let mut trainer = pack
            .data
            .trainers
            .get(&id)
            .with_context(|| format!("canoe-area trainer {id}"))?
            .clone();
        for member in &mut trainer.party {
            if !policy.allowed.contains(&member.species) {
                let species = policy.target(pack, &member.species, Some(member.level))?;
                member.moves = pack
                    .data
                    .create_pokemon(&species, member.level, member.dvs)?
                    .moves;
                member.species = species;
            }
            ensure!(
                pack.data
                    .evolutions
                    .0
                    .get(&member.species)
                    .is_none_or(|entries| {
                        entries
                            .iter()
                            .all(|entry| policy.allowed.contains(&entry.species))
                    }),
                "canoe-area trainer {id} evolves outside the active roster"
            );
        }
        pack.data.trainers.trainers.insert(id, trainer);
    }
    if let Some(zone) = &pack.data.safari_zone {
        for name in zone.maps.keys() {
            if let Some(group) = pack.data.map_fishing_group(name)? {
                rod_groups.insert(group.to_owned());
            }
        }
        let mut value = serde_json::to_value(zone)?;
        rewrite_encounters(pack, &policy, &mut value)?;
        pack.data.safari_zone = Some(serde_json::from_value(value)?);
    }
    let swarms: Vec<_> = pack
        .data
        .fishing
        .swarm_rules
        .values()
        .filter(|rule| rod_groups.contains(&rule.base_group))
        .map(|rule| rule.swarm_group.clone())
        .collect();
    rod_groups.extend(swarms);
    let mut time_groups = BTreeSet::new();
    for name in rod_groups {
        ensure!(
            name.starts_with("FISHGROUP_MN_"),
            "shared source rod group {name}"
        );
        let group = pack
            .data
            .fishing
            .groups
            .get(&name)
            .context("canoe-area rod group")?;
        time_groups.extend(
            group
                .rod_tables
                .values()
                .flat_map(|table| &table.slots)
                .filter_map(|slot| slot.time_group.clone()),
        );
        let mut value = serde_json::to_value(group)?;
        rewrite_encounters(pack, &policy, &mut value)?;
        pack.data
            .fishing
            .groups
            .insert(name, serde_json::from_value(value)?);
    }
    for name in time_groups {
        ensure!(
            name.starts_with("MN_"),
            "shared source timed rod group {name}"
        );
        let mut value = serde_json::to_value(
            pack.data
                .fishing
                .time_groups
                .get(&name)
                .context("canoe-area timed fish")?,
        )?;
        rewrite_encounters(pack, &policy, &mut value)?;
        pack.data
            .fishing
            .time_groups
            .insert(name, serde_json::from_value(value)?);
    }
    if let Some(zone) = &pack.data.safari_zone {
        zone.validate(&pack.data)?;
    }
    Ok(())
}
