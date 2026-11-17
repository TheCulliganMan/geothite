//! Install reviewed Headbutt and Safari habitat species.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use crystal_core::world::collision::permissions;
use crystal_core::world::encounters::{
    FieldEncounterData, FieldEncounterEntry, FieldEncounterTable,
};
use serde::Deserialize;

use crate::CompiledGamePack;
use crate::minnesota_biome_ecology;
use crate::northstar_regional_roster::RegionalRoster;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Trees {
    common: Vec<String>,
    rare: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PreserveHabitat {
    grass: Vec<String>,
    water: Vec<String>,
}

pub(super) fn install(
    base: &CompiledGamePack,
    pack: &mut CompiledGamePack,
    policy: &RegionalRoster,
    trees: &BTreeMap<String, Trees>,
    preserve: &BTreeMap<String, PreserveHabitat>,
) -> Result<()> {
    ensure!(
        pack.data
            .field_encounters
            .keys()
            .filter(|map| map.starts_with("Mn"))
            .count()
            == trees.len(),
        "every installed Minnesota tree table needs an authored lineup"
    );
    for (name, lineup) in trees {
        ensure!(name.starts_with("Mn"), "Minnesota tree map required");
        let field = pack
            .data
            .field_encounters
            .get_mut(name)
            .context("installed tree map")?;
        ensure!(
            field.tables.len() == 1,
            "unreviewed regional field encounter method"
        );
        let table = field
            .tables
            .get_mut("headbutt")
            .context("installed Headbutt table")?;
        for (slots, species) in [
            (&mut table.common, &lineup.common),
            (&mut table.rare, &lineup.rare),
        ] {
            ensure!(slots.len() == species.len(), "Headbutt slot count changed");
            for (slot, species) in slots.iter_mut().zip(species) {
                minnesota_biome_ecology::validate_active_species(base, policy, species, slot.level)
                    .with_context(|| format!("Headbutt in {name}"))?;
                slot.species.clone_from(species);
            }
        }
    }
    let safari = pack
        .data
        .safari_zone
        .as_mut()
        .context("installed Boundary Waters Safari")?;
    ensure!(
        safari.id == "MN_BOUNDARY_WATERS_SAFARI" && safari.habitats.len() == preserve.len(),
        "every Boundary Waters Safari habitat needs an authored lineup"
    );
    for habitat in &mut safari.habitats {
        let lineup = preserve
            .get(&habitat.id)
            .context("missing Safari habitat lineup")?;
        for (slots, species) in [
            (&mut habitat.grass, &lineup.grass),
            (&mut habitat.water, &lineup.water),
        ] {
            ensure!(slots.len() == species.len(), "Safari slot count changed");
            for (slot, species) in slots.iter_mut().zip(species) {
                minnesota_biome_ecology::validate_active_species(base, policy, species, slot.level)
                    .with_context(|| format!("Safari habitat {}", habitat.id))?;
                slot.species.clone_from(species);
            }
        }
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TreeSite {
    level: u8,
    profile: String,
}

pub(super) fn install_new_trees(
    base: &CompiledGamePack,
    pack: &mut CompiledGamePack,
    policy: &RegionalRoster,
    sites: &BTreeMap<String, TreeSite>,
    profiles: &BTreeMap<String, Trees>,
) -> Result<()> {
    let mut expected = BTreeSet::new();
    for (name, map) in &base.data.maps {
        if !name.starts_with("Mn")
            || base.data.field_encounters.contains_key(name)
            || base
                .data
                .safari_zone
                .as_ref()
                .is_some_and(|safari| safari.maps.contains_key(name))
        {
            continue;
        }
        let tileset = base
            .data
            .tilesets
            .get(&map.attributes.tileset_name)
            .context("native tree tileset")?;
        for block in &map.blocks {
            let quadrants = tileset
                .collision
                .get(&format!("{block:02x}"))
                .context("native tree collision")?;
            let has_tree = quadrants.iter().try_fold(false, |found, token| {
                crate::resolve_collision_token(token).map(|permission| {
                    found
                        || matches!(
                            permission,
                            permissions::HEADBUTT_TREE | permissions::HEADBUTT_TREE_1D
                        )
                })
            })?;
            if has_tree {
                expected.insert(name.as_str());
                break;
            }
        }
    }
    ensure!(
        expected == sites.keys().map(String::as_str).collect(),
        "every non-Safari map with native Headbutt terrain needs an encounter table"
    );
    for (name, site) in sites {
        let lineup = profiles
            .get(&site.profile)
            .context("Headbutt habitat profile")?;
        let table = FieldEncounterTable {
            common: tree_bucket(base, policy, &lineup.common, site.level)?,
            rare: tree_bucket(base, policy, &lineup.rare, site.level)?,
        };
        pack.data.field_encounters.insert(
            name.clone(),
            FieldEncounterData {
                map_name: name.clone(),
                tables: BTreeMap::from([("headbutt".into(), table)]),
            },
        );
    }
    Ok(())
}

fn tree_bucket(
    base: &CompiledGamePack,
    policy: &RegionalRoster,
    lineup: &[String],
    level: u8,
) -> Result<Vec<FieldEncounterEntry>> {
    let weights = [50, 30, 20];
    ensure!(
        lineup.len() == weights.len(),
        "Headbutt bucket needs three species"
    );
    lineup
        .iter()
        .zip(weights)
        .map(|(name, weight)| {
            let species = policy.target(base, name, Some(level))?;
            minnesota_biome_ecology::validate_active_species(base, policy, &species, level)?;
            Ok(FieldEncounterEntry {
                weight,
                species,
                level,
                sleep_turns_by_time: BTreeMap::new(),
            })
        })
        .collect()
}
