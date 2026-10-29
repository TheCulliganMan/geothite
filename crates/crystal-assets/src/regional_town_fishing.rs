//! Bind newly authored freshwater towns to the installed active regional fish.
use anyhow::{Context, Result, ensure};

use crate::{CompiledGamePack, MapModule};

const POND: &str = "FISHGROUP_MN_POND";

pub(crate) fn freshwater(pack: &CompiledGamePack, town: &mut MapModule) -> Result<()> {
    let policy = crate::authored_regional_approach::policy()?;
    let group = pack
        .data
        .fishing
        .groups
        .get(POND)
        .context("regional pond fish")?;
    ensure!(
        !pack
            .data
            .fishing
            .swarm_rules
            .values()
            .any(|rule| rule.base_group == POND),
        "regional town pond must not inherit an unreviewed swarm"
    );
    for table in group.rod_tables.values() {
        for slot in &table.slots {
            if let Some(species) = &slot.species {
                ensure!(
                    policy.target(pack, species, Some(slot.level))? == *species,
                    "town rod slot must use an active Minnesota species at its legal level"
                );
            }
            if let Some(id) = &slot.time_group {
                let time = pack
                    .data
                    .fishing
                    .time_groups
                    .get(id)
                    .context("regional timed fish")?;
                for (species, level) in [
                    (&time.day_species, time.day_level),
                    (&time.night_species, time.night_level),
                ] {
                    ensure!(
                        policy.target(pack, species, Some(level))? == *species,
                        "town timed rod slot must use an active Minnesota species at its legal level"
                    );
                }
            }
        }
    }
    town.attributes.fishing_group = Some(POND.into());
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{POND, freshwater};
    use crate::read_verified_compiled_game_pack;

    #[test]
    #[ignore = "requires the completed regional pack in CRYSTAL_INTERIOR_PACK"]
    fn regional_town_fishing_preserves_catalogs_and_rejects_legacy_time_fish() {
        let base = read_verified_compiled_game_pack(PathBuf::from(
            std::env::var_os("CRYSTAL_INTERIOR_PACK").expect("regional parent"),
        ))
        .expect("verified regional pack");
        let original = base.data.maps["GoldenrodCity"].clone();
        let mut town = original.clone();
        freshwater(&base, &mut town).expect("active pond and every timed rod slot");
        assert_eq!(town.attributes.fishing_group.as_deref(), Some(POND));
        town.attributes.fishing_group = original.attributes.fishing_group.clone();
        assert!(
            town == original,
            "only the new town's fishing binding changes"
        );

        for period in ["direct", "day", "night"] {
            let mut bad = base.clone();
            if period == "direct" {
                let group = bad.data.fishing.groups.get_mut(POND).expect("pond");
                let slot = group
                    .rod_tables
                    .values_mut()
                    .flat_map(|table| &mut table.slots)
                    .find(|slot| slot.species.is_some())
                    .expect("ordinary fish");
                slot.species = Some("MAGIKARP".into());
            } else {
                let id = bad.data.fishing.groups[POND]
                    .rod_tables
                    .values()
                    .flat_map(|table| &table.slots)
                    .find_map(|slot| slot.time_group.clone())
                    .expect("timed fish");
                let time = bad.data.fishing.time_groups.get_mut(&id).expect("time");
                if period == "day" {
                    time.day_species = "MAGIKARP".into();
                } else {
                    time.night_species = "MAGIKARP".into();
                }
            }
            let mut untouched = original.clone();
            assert!(freshwater(&bad, &mut untouched).is_err(), "reject {period}");
            assert!(untouched == original, "failed binding is atomic: {period}");
        }
    }
}
