//! Preserve potential acquisition coverage during biome corrections.
use std::collections::BTreeSet;

use anyhow::{Result, ensure};
use crystal_core::world::encounters::WildEncounterTable;

use crate::CompiledGamePack;
use crate::authored_town_geometry::kind;

pub(super) fn preserve(base: &CompiledGamePack, child: &CompiledGamePack) -> Result<()> {
    let before = reachable_evolutions(base, ordinary_sources(base)?);
    let after = reachable_evolutions(child, ordinary_sources(child)?);
    let lost = before.difference(&after).collect::<Vec<_>>();
    ensure!(
        lost.is_empty(),
        "biome correction lost potential ordinary species: {lost:?}"
    );
    Ok(())
}

fn add_table(set: &mut BTreeSet<String>, table: &WildEncounterTable) {
    set.extend(
        table
            .morning
            .iter()
            .chain(&table.day)
            .chain(&table.night)
            .map(|s| s.species.clone()),
    );
}

fn reachable_evolutions(
    pack: &CompiledGamePack,
    mut species: BTreeSet<String>,
) -> BTreeSet<String> {
    loop {
        let more = species
            .iter()
            .flat_map(|s| pack.data.evolutions.0.get(s).into_iter().flatten())
            .map(|e| e.species.clone())
            .collect::<BTreeSet<_>>();
        let count = species.len();
        species.extend(more);
        if species.len() == count {
            return species;
        }
    }
}

fn ordinary_sources(pack: &CompiledGamePack) -> Result<BTreeSet<String>> {
    let mut species = BTreeSet::new();
    let mut groups = BTreeSet::new();
    for (name, map) in &pack.data.maps {
        if !name.starts_with("Mn") {
            continue;
        }
        species.extend(
            map.gift_pokemon_scripts
                .iter()
                .map(|g| g.species_id.clone()),
        );
        let data = pack.data.wild_encounters.get(name);
        let cave = matches!(
            map.attributes.environment.as_deref(),
            Some("CAVE" | "DUNGEON")
        );
        let mut land = false;
        let mut water = false;
        let mut trees = false;
        let mut zones = BTreeSet::new();
        for y in 0..map.attributes.height * 2 {
            for x in 0..map.attributes.width * 2 {
                let terrain = kind(pack, map, [x, y])?;
                water |= terrain == "WATER";
                trees |= terrain.starts_with("HEADBUTT_TREE");
                if terrain == "TALL_GRASS" || (cave && terrain == "FLOOR") {
                    if let Some(zone) = data.and_then(|d| d.zone_at(x as i16, y as i16)) {
                        zones.insert(zone.id.clone());
                    } else {
                        land = true;
                    }
                }
            }
        }
        if let Some(data) = data {
            if land {
                if let Some(table) = &data.grass {
                    for (time, rows) in [
                        ("morning", &table.morning),
                        ("day", &table.day),
                        ("night", &table.night),
                    ] {
                        if data
                            .grass_rates
                            .as_ref()
                            .and_then(|r| r.get(time))
                            .is_some_and(|r| *r > 0)
                        {
                            species.extend(rows.iter().map(|s| s.species.clone()));
                        }
                    }
                }
                for swarm in data.swarm_overrides.values() {
                    if swarm.grass_rates.values().any(|r| *r > 0) {
                        add_table(&mut species, &swarm.grass);
                    }
                }
            }
            for zone in &data.zones {
                if zones.contains(&zone.id) && zone.grass_rates.values().any(|r| *r > 0) {
                    add_table(&mut species, &zone.grass);
                }
            }
            if water
                && data.water_rate.is_some_and(|r| r > 0)
                && let Some(table) = &data.water
            {
                add_table(&mut species, table);
            }
        }
        if trees
            && let Some(table) = pack
                .data
                .field_encounters
                .get(name)
                .and_then(|d| d.tables.get("headbutt"))
        {
            species.extend(
                table
                    .common
                    .iter()
                    .chain(&table.rare)
                    .filter(|s| s.weight > 0)
                    .map(|s| s.species.clone()),
            );
        }
        if water && let Some(group) = &map.attributes.fishing_group {
            groups.insert(group.clone());
        }
    }
    loop {
        let count = groups.len();
        let more = pack
            .data
            .fishing
            .swarm_rules
            .values()
            .filter(|r| groups.contains(&r.base_group))
            .map(|r| r.swarm_group.clone())
            .collect::<Vec<_>>();
        groups.extend(more);
        if groups.len() == count {
            break;
        }
    }
    for name in groups {
        if let Some(group) = pack
            .data
            .fishing
            .groups
            .get(&name)
            .filter(|g| g.bite_threshold > 0)
        {
            for table in group.rod_tables.values() {
                let mut previous = 0;
                for slot in &table.slots {
                    let enabled = slot.threshold > previous;
                    previous = slot.threshold;
                    if !enabled {
                        continue;
                    }
                    if let Some(s) = &slot.species {
                        species.insert(s.clone());
                    } else if let Some(time) = slot
                        .time_group
                        .as_ref()
                        .and_then(|t| pack.data.fishing.time_groups.get(t))
                    {
                        species.extend([time.day_species.clone(), time.night_species.clone()]);
                    }
                }
            }
        }
    }
    Ok(species)
}
