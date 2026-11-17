//! Rework Minnesota rod catches while preserving native group indices and swarm mechanics.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use crystal_core::world::fishing::{FISHGROUP_NONE, ROD_GOOD, ROD_SUPER};

use crate::CompiledGamePack;
use crate::authored_town_geometry;
use crate::minnesota_biome_ecology;
use crate::northstar_regional_roster::RegionalRoster;

pub(super) fn install(
    base: &CompiledGamePack,
    pack: &mut CompiledGamePack,
    policy: &RegionalRoster,
    assignments: &BTreeMap<String, String>,
    profiles: &BTreeMap<String, Vec<String>>,
) -> Result<()> {
    let mut reviewed = BTreeSet::new();
    for (name, map) in &base.data.maps {
        if !name.starts_with("Mn") {
            continue;
        }
        let Some(group) = map.attributes.fishing_group.as_deref() else {
            continue;
        };
        if group == FISHGROUP_NONE {
            continue;
        }
        for y in 0..map.attributes.height * 2 {
            if (0..map.attributes.width * 2)
                .map(|x| authored_town_geometry::kind(base, map, [x, y]))
                .try_fold(false, |wet, tile| tile.map(|tile| wet || tile == "WATER"))?
            {
                reviewed.insert(group);
                break;
            }
        }
    }
    ensure!(
        reviewed == assignments.keys().map(String::as_str).collect(),
        "each installed Minnesota rod header with water needs an ecology assignment"
    );
    let mut group_profiles = BTreeMap::new();
    for (group, profile) in assignments {
        for name in std::iter::once(group.as_str()).chain(
            base.data
                .fishing
                .swarm_rules
                .values()
                .filter(|rule| &rule.base_group == group)
                .map(|rule| rule.swarm_group.as_str()),
        ) {
            ensure!(
                name.starts_with("FISHGROUP_MN_"),
                "regional fishing header required: {name}"
            );
            ensure!(
                !base.data.maps.iter().any(|(map, module)| {
                    !map.starts_with("Mn")
                        && module.attributes.fishing_group.as_deref() == Some(name)
                }),
                "original-region fishing shares {name}"
            );
            let old = group_profiles.insert(name, profile);
            ensure!(
                old.is_none_or(|old| old == profile),
                "conflicting rod profile for {name}"
            );
        }
    }
    for (name, profile) in group_profiles {
        let candidates = profiles
            .get(profile)
            .context("missing rod habitat profile")?;
        ensure!(
            candidates.len() >= 3,
            "rod habitat needs at least three candidates"
        );
        let group = pack
            .data
            .fishing
            .groups
            .get_mut(name)
            .context("installed rod group")?;
        for (rod, table) in &mut group.rod_tables {
            if !matches!(rod.as_str(), ROD_GOOD | ROD_SUPER) {
                continue;
            }
            let mut offset = if rod == ROD_SUPER {
                candidates.len() / 2
            } else {
                0
            };
            if !assignments.contains_key(name) {
                offset += 1;
            }
            for (index, slot) in table.slots.iter_mut().enumerate() {
                if let Some(species) = &mut slot.species {
                    *species = choose(base, policy, candidates, offset, slot.level)?;
                    offset += 1;
                } else {
                    let clock = slot.time_group.as_ref().context("native timed rod slot")?;
                    let mut entry = base
                        .data
                        .fishing
                        .time_groups
                        .get(clock)
                        .context("installed fishing clock group")?
                        .clone();
                    entry.day_species = choose(base, policy, candidates, offset, entry.day_level)?;
                    entry.night_species =
                        choose(base, policy, candidates, offset + 1, entry.night_level)?;
                    let clock = format!("MN_REGION_{name}_{rod}_{index}");
                    ensure!(
                        pack.data
                            .fishing
                            .time_groups
                            .insert(clock.clone(), entry)
                            .is_none(),
                        "occupied rod clock group"
                    );
                    slot.time_group = Some(clock);
                    offset += 2;
                }
            }
        }
    }
    Ok(())
}

fn choose(
    base: &CompiledGamePack,
    policy: &RegionalRoster,
    candidates: &[String],
    index: usize,
    level: u8,
) -> Result<String> {
    let species = policy.target(base, &candidates[index % candidates.len()], Some(level))?;
    minnesota_biome_ecology::validate_active_species(base, policy, &species, level)?;
    Ok(species)
}
