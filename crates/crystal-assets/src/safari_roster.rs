//! Active Minnesota-only wildlife, including the preserve's existing rod tables.

use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};

use crate::{GameDataSet, SafariZone};

pub(crate) fn validate_safari_roster(data: &GameDataSet, zone: &SafariZone) -> Result<()> {
    let policy = crate::authored_regional_approach::policy()?;
    let mut used = BTreeSet::new();
    for habitat in &zone.habitats {
        used.extend(
            habitat
                .grass
                .iter()
                .chain(&habitat.water)
                .map(|slot| slot.species.as_str()),
        );
    }
    let mut groups = BTreeSet::new();
    for name in zone.maps.keys() {
        if let Some(group) = data.map_fishing_group(name)? {
            groups.insert(group);
            groups.extend(
                data.fishing
                    .swarm_rules
                    .values()
                    .filter(|rule| rule.base_group == group)
                    .map(|rule| rule.swarm_group.as_str()),
            );
        }
    }
    for name in groups {
        let group = data
            .fishing
            .groups
            .get(name)
            .with_context(|| format!("Safari rod group {name} missing"))?;
        for slot in group.rod_tables.values().flat_map(|table| &table.slots) {
            if let Some(species) = &slot.species {
                used.insert(species);
            } else if let Some(time) = &slot.time_group {
                let time = data
                    .fishing
                    .time_groups
                    .get(time)
                    .context("Safari rod time group missing")?;
                used.insert(&time.day_species);
                used.insert(&time.night_species);
            }
        }
    }
    for species in used {
        ensure!(
            policy.allowed.contains(species),
            "Safari wildlife is not in the active Minnesota roster: {species}"
        );
        if let Some(evolutions) = data.evolutions.0.get(species) {
            ensure!(
                evolutions
                    .iter()
                    .all(|evolution| policy.allowed.contains(&evolution.species)),
                "Safari species {species} evolves outside the active Minnesota roster"
            );
        }
    }
    Ok(())
}
