//! Private biome rod groups with unchanged native selection mechanics.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::CompiledGamePack;
use crate::minnesota_biome_ecology::validate_species;
use crate::northstar_regional_roster::RegionalRoster;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Site {
    map: String,
    before_group: String,
    biome: String,
    #[serde(default)]
    bank: Option<Bank>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Bank {
    shore: [u16; 2],
    water: [u16; 2],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Recipe {
    good: BTreeMap<String, String>,
    #[serde(rename = "super")]
    super_: BTreeMap<String, String>,
    timed_good: [String; 2],
    timed_super: [String; 2],
}

pub(super) fn install(
    pack: &mut CompiledGamePack,
    sites: &[Site],
    recipes: &BTreeMap<String, Recipe>,
    policy: &RegionalRoster,
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for site in sites {
        ensure!(
            site.map.starts_with("Mn") && seen.insert(&site.map),
            "unique regional rod map"
        );
        let recipe = recipes.get(&site.biome).context("rod biome recipe")?;
        ensure!(
            site.biome
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b == b'_'),
            "native biome namespace"
        );
        let mut map = crate::northstar_rival_arc::map(pack, &site.map)?;
        if let Some(bank) = &site.bank {
            ensure!(
                crate::authored_town_geometry::kind(pack, &map, bank.shore)? == "FLOOR"
                    && crate::authored_town_geometry::kind(pack, &map, bank.water)? == "WATER"
                    && bank.shore[0].abs_diff(bank.water[0])
                        + bank.shore[1].abs_diff(bank.water[1])
                        == 1
                    && !map.objects.iter().any(|o| [o.x, o.y] == bank.shore),
                "declared native rod bank must be an adjacent unoccupied shore"
            );
        }
        ensure!(
            map.attributes.fishing_group.as_deref() == Some(site.before_group.as_str()),
            "rod header preimage"
        );
        let name = format!("FISHGROUP_MN_ECO_{}", site.map.to_ascii_uppercase());
        let swarms = pack
            .data
            .fishing
            .swarm_rules
            .values()
            .filter(|r| r.base_group == site.before_group)
            .cloned()
            .collect::<Vec<_>>();
        ensure!(swarms.len() <= 1, "one native fishing swarm per group");
        let mut groups = vec![(site.before_group.clone(), name.clone())];
        if let Some(swarm) = swarms.first() {
            groups.push((swarm.swarm_group.clone(), format!("{name}_SWARM")));
        }
        for (old, new) in groups {
            let mut group = pack
                .data
                .fishing
                .groups
                .get(&old)
                .context("original rod group")?
                .clone();
            let original = group.clone();
            group.source_index = pack
                .data
                .fishing
                .groups
                .values()
                .map(|g| g.source_index)
                .max()
                .context("fishing source indices")?
                .checked_add(1)
                .context("native fishing index overflow")?;
            for (rod, replacements, timed) in [
                ("GOOD_ROD", &recipe.good, &recipe.timed_good),
                ("SUPER_ROD", &recipe.super_, &recipe.timed_super),
            ] {
                for slot in &mut group
                    .rod_tables
                    .get_mut(rod)
                    .context("upgraded native rod")?
                    .slots
                {
                    if let Some(species) = &mut slot.species {
                        if let Some(target) = replacements.get(species) {
                            species.clone_from(target);
                        }
                        validate_species(pack, policy, species, slot.level)?;
                    } else {
                        let old_time = slot.time_group.as_ref().context("native timed rod slot")?;
                        let mut entry = pack
                            .data
                            .fishing
                            .time_groups
                            .get(old_time)
                            .context("original timed fish")?
                            .clone();
                        entry.day_species.clone_from(&timed[0]);
                        entry.night_species.clone_from(&timed[1]);
                        validate_species(pack, policy, &entry.day_species, entry.day_level)?;
                        validate_species(pack, policy, &entry.night_species, entry.night_level)?;
                        let time_name = format!(
                            "MN_ECO_{}_{}_{}_{}",
                            site.biome.to_ascii_uppercase(),
                            rod,
                            entry.day_level,
                            entry.night_level
                        );
                        if let Some(existing) = pack.data.fishing.time_groups.get(&time_name) {
                            ensure!(existing == &entry, "inconsistent shared biome clock table");
                        } else {
                            pack.data
                                .fishing
                                .time_groups
                                .insert(time_name.clone(), entry);
                        }
                        slot.time_group = Some(time_name);
                    }
                }
            }
            ensure!(
                group.bite_threshold == original.bite_threshold
                    && group.rod_tables.get("OLD_ROD") == original.rod_tables.get("OLD_ROD")
                    && group.rod_tables.iter().all(|(rod, table)| {
                        let old = &original.rod_tables[rod];
                        table.slots.len() == old.slots.len()
                            && table.slots.iter().zip(&old.slots).all(|(a, b)| {
                                a.threshold == b.threshold
                                    && a.level == b.level
                                    && a.species.is_some() == b.species.is_some()
                                    && a.time_group.is_some() == b.time_group.is_some()
                            })
                    }),
                "native rod thresholds, levels and Old Rod must remain exact"
            );
            ensure!(
                pack.data.fishing.groups.insert(new, group).is_none(),
                "occupied biome rod group"
            );
        }
        if let Some(mut rule) = swarms.into_iter().next() {
            rule.base_group.clone_from(&name);
            rule.swarm_group = format!("{name}_SWARM");
            ensure!(
                pack.data
                    .fishing
                    .swarm_rules
                    .insert(
                        format!("MN_ECO_SWARM_{}", site.map.to_ascii_uppercase()),
                        rule
                    )
                    .is_none(),
                "occupied biome swarm"
            );
        }
        pack.data
            .map_attributes
            .get_mut(&site.map)
            .context("map header mirror")?
            .fishing_group = Some(name.clone());
        map.attributes.fishing_group = Some(name);
        // Header changes preserve every existing typed/raw program vector.
        pack.data.maps.insert(site.map.clone(), map);
    }
    Ok(())
}
