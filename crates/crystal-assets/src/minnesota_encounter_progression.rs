//! Authored encounter levels follow the Minnesota journey and its side trips.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use crystal_core::world::encounters::WildEncounterTable;
use serde::Deserialize;

use crate::CompiledGamePack;

const MANIFEST: &str = "minnesota-encounter-progression-v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    schema_version: u8,
    parent_content_hash: String,
    groups: Vec<Group>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Group {
    id: String,
    minimum_badges: usize,
    access: String,
    maps: Vec<String>,
    grass: [u8; 7],
    surf: [u8; 3],
    #[serde(default)]
    field_delta: u8,
    #[serde(default)]
    promotions: BTreeMap<String, String>,
    #[serde(default)]
    zone_replacements: BTreeMap<String, BTreeMap<String, String>>,
    #[serde(default)]
    residents: BTreeMap<String, [String; 7]>,
}

/// Authors a complete Minnesota encounter curve without changing geography.
///
/// Each installed walking, cave, Surf, swarm and field table must belong to one
/// reviewed journey group. Slot rates, weights, periods and zone boundaries are
/// preserved. Main-table promotions are explicit; species above their legal
/// evolution stage are walked back within their own family. Rod tiers, free
/// opening partners, revived fossils and voluntary rare encounters retain their
/// separate acquisition contracts. Johto and Kanto remain exact.
/// Local trainer teams below their journey floor use the production level,
/// evolution and learned-move adjustment, preserving their relative levels.
///
/// # Errors
/// Rejects a stale parent, omitted or duplicate habitats, illegal stage levels,
/// nonregional species, lost acquisition coverage or an invalid resulting pack.
/// The parent is unchanged.
pub fn build_minnesota_encounter_progression(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let source: Source = serde_json::from_str(source)?;
    ensure!(
        source.schema_version == 1
            && source.parent_content_hash == base.identity()?.content_hash
            && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "fresh progression plan bound to the actual parent required"
    );
    let policy = crate::authored_regional_approach::policy()?;
    let mut pack = base.clone();
    let mut seen = BTreeSet::new();
    let mut ids = BTreeSet::new();
    let mut trainer_floors = BTreeMap::<String, u8>::new();
    for group in &source.groups {
        validate_group(group)?;
        ensure!(ids.insert(&group.id), "duplicate journey group");
        for (from, to) in &group.promotions {
            let used = group
                .maps
                .iter()
                .filter_map(|name| base.data.wild_encounters.get(name))
                .flat_map(|w| {
                    w.grass
                        .iter()
                        .chain(w.swarm_overrides.values().map(|s| &s.grass))
                })
                .any(|table| {
                    table
                        .morning
                        .iter()
                        .chain(&table.day)
                        .chain(&table.night)
                        .any(|s| &s.species == from)
                });
            ensure!(
                used && descendant(base, from, to),
                "promotion must be used and stay in its actual family: {from} -> {to}"
            );
        }
        for name in &group.maps {
            ensure!(
                name.starts_with("Mn") && seen.insert(name.clone()),
                "duplicate or nonregional progression habitat: {name}"
            );
            let room = pack.data.maps.get(name).context("journey map")?;
            let floor = [2, 9, 13, 18, 23, 27, 30, 35, 40][group.minimum_badges];
            for request in room
                .trainer_scripts
                .values()
                .chain(room.scripted_trainer_battles.iter().map(|b| &b.request))
            {
                ensure!(
                    request.trainer_id.starts_with("MN_"),
                    "regional opponent required"
                );
                trainer_floors
                    .entry(request.trainer_id.clone())
                    .and_modify(|value| *value = (*value).min(floor))
                    .or_insert(floor);
            }
            let mut data = pack
                .data
                .wild_encounters
                .get(name)
                .with_context(|| format!("installed encounter habitat {name}"))?
                .clone();
            if let Some(table) = &mut data.grass {
                author_table(table, &group.grass, Some(group))?;
            }
            if let Some(table) = &mut data.water {
                author_table(table, &group.surf, None)?;
            }
            for zone in &mut data.zones {
                author_table(&mut zone.grass, &group.grass, None)?;
                if let Some(replacements) = group.zone_replacements.get(&zone.id) {
                    for (from, to) in replacements {
                        let mut count = 0;
                        for slot in zone
                            .grass
                            .morning
                            .iter_mut()
                            .chain(&mut zone.grass.day)
                            .chain(&mut zone.grass.night)
                        {
                            if &slot.species == from {
                                slot.species.clone_from(to);
                                count += 1;
                            }
                        }
                        ensure!(
                            count > 0,
                            "stale local-stage species replacement: {name}/{} {from}",
                            zone.id
                        );
                    }
                }
            }
            for swarm in data.swarm_overrides.values_mut() {
                author_table(&mut swarm.grass, &group.grass, Some(group))?;
            }
            // Correct underlevel inherited showcase forms without introducing
            // another family or bypassing the active roster's retirement rules.
            for table in data
                .grass
                .iter_mut()
                .chain(&mut data.water)
                .chain(data.zones.iter_mut().map(|z| &mut z.grass))
                .chain(data.swarm_overrides.values_mut().map(|s| &mut s.grass))
            {
                for slot in table
                    .morning
                    .iter_mut()
                    .chain(&mut table.day)
                    .chain(&mut table.night)
                {
                    slot.species = policy.target(&pack, &slot.species, Some(slot.level))?;
                    crate::minnesota_biome_ecology::validate_species(
                        &pack,
                        &policy,
                        &slot.species,
                        slot.level,
                    )?;
                }
            }
            pack.data.wild_encounters.insert(name.clone(), data);
            if let Some(mut field) = pack.data.field_encounters.get(name).cloned() {
                for table in field.tables.values_mut() {
                    for slot in table.common.iter_mut().chain(&mut table.rare) {
                        slot.level = slot
                            .level
                            .checked_add(group.field_delta)
                            .context("field level overflow")?;
                        crate::minnesota_biome_ecology::validate_species(
                            &pack,
                            &policy,
                            &slot.species,
                            slot.level,
                        )?;
                    }
                }
                pack.data.field_encounters.insert(name.clone(), field);
            } else {
                ensure!(
                    group.field_delta == 0,
                    "field offset requires a native table"
                );
            }
        }
    }
    let expected = base
        .data
        .wild_encounters
        .keys()
        .filter(|n| n.starts_with("Mn"))
        .cloned()
        .collect::<BTreeSet<_>>();
    ensure!(
        seen == expected,
        "every installed regional habitat must be reviewed; omitted {:?}, extra {:?}",
        expected.difference(&seen).collect::<Vec<_>>(),
        seen.difference(&expected).collect::<Vec<_>>()
    );
    ensure!(
        base.data
            .field_encounters
            .keys()
            .filter(|n| n.starts_with("Mn"))
            .all(|n| seen.contains(n)),
        "unreviewed regional field table"
    );
    for (id, floor) in trainer_floors {
        let trainer = pack
            .data
            .trainers
            .trainers
            .get(&id)
            .context("installed local opponent")?;
        let low = trainer
            .party
            .iter()
            .map(|p| p.level)
            .min()
            .context("nonempty local party")?;
        if low >= floor {
            continue;
        }
        let ace = trainer
            .party
            .iter()
            .map(|p| p.level)
            .max()
            .context("local ace")?
            .checked_add(floor - low)
            .context("trainer level overflow")?;
        let trainer = pack.data.rematch_trainer_at_ace(trainer, ace)?;
        for member in &trainer.party {
            crate::minnesota_biome_ecology::validate_species(
                &pack,
                &policy,
                &member.species,
                member.level,
            )?;
        }
        pack.data.trainers.trainers.insert(id, trainer);
    }
    crate::minnesota_ecology_coverage::preserve(base, &pack)?;
    pack.report.manifests.push(MANIFEST.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    crate::verify_northstar_publication_geometry(&pack)?;
    Ok(pack)
}

fn validate_group(group: &Group) -> Result<()> {
    let bounds = [
        (2, 8),
        (6, 13),
        (10, 18),
        (15, 23),
        (19, 30),
        (23, 32),
        (27, 38),
        (32, 44),
        (38, 49),
    ];
    let &(low, high) = bounds
        .get(group.minimum_badges)
        .context("eight-badge progression")?;
    ensure!(
        !group.id.is_empty() && !group.access.is_empty() && !group.maps.is_empty(),
        "named journey and actual access required"
    );
    ensure!(
        group.grass.iter().all(|l| (low..=high).contains(l)),
        "walking levels must fit the journey stage: {}",
        group.id
    );
    // Surf has a separate first-access floor after the Capitol Badge. Returning
    // to Winona with Surf does not expose level2 swimming encounters.
    ensure!(
        group
            .surf
            .iter()
            .all(|l| (20.max(low)..=28.max(high)).contains(l)),
        "Surf access levels: {}",
        group.id
    );
    for map in &group.maps {
        let required = if map == "MnGrandMarais"
            || map.starts_with("MnR11")
            || map.starts_with("MnR12")
            || map.starts_with("MnR13")
            || map.starts_with("MnR20")
        {
            Some(8)
        } else if map.starts_with("MnR09") || map.starts_with("MnR10") || map.starts_with("MnR17") {
            Some(7)
        } else if map.starts_with("MnR07")
            || map.starts_with("MnR08")
            || map.starts_with("MnR14")
            || map.starts_with("MnR15")
            || map.starts_with("MnR16")
            || map.starts_with("MnR18")
            || map.starts_with("MnR19")
        {
            Some(6)
        } else if map.starts_with("MnR06") {
            Some(5)
        } else if map.starts_with("MnR05") || map == "MnStateFairgrounds" {
            Some(4)
        } else if map.starts_with("MnR04") {
            Some(3)
        } else if map.starts_with("MnR03") {
            Some(2)
        } else if map.starts_with("MnR02") {
            Some(1)
        } else if map.starts_with("MnR01") {
            Some(0)
        } else {
            None
        };
        ensure!(
            required.is_none_or(|n| group.minimum_badges == n),
            "incorrect authored badge gate for {map}"
        );
    }
    ensure!(
        group.zone_replacements.is_empty() || group.maps == ["MnR02C00R00"],
        "reviewed early farm power correction required"
    );
    ensure!(
        group.residents.is_empty()
            || group.maps == ["MnGrandMarais"]
            || group.maps == ["MnR11C00R00"]
            || group.maps == ["MnR11C00R01"],
        "explicit residents belong to their three reviewed North Shore tables"
    );
    ensure!(
        group.residents.is_empty()
            || group
                .residents
                .keys()
                .map(String::as_str)
                .collect::<BTreeSet<_>>()
                == BTreeSet::from(["morning", "day", "night"]),
        "all three resident periods must be authored"
    );
    Ok(())
}

fn descendant(pack: &CompiledGamePack, from: &str, to: &str) -> bool {
    let mut visited = BTreeSet::new();
    let mut pending = vec![from];
    while let Some(species) = pending.pop() {
        if !visited.insert(species) {
            continue;
        }
        for edge in pack.data.evolutions.0.get(species).into_iter().flatten() {
            if edge.species == to {
                return true;
            }
            pending.push(edge.species.as_str());
        }
    }
    false
}

fn author_table<const N: usize>(
    table: &mut WildEncounterTable,
    levels: &[u8; N],
    group: Option<&Group>,
) -> Result<()> {
    for (time, slots) in [
        ("morning", &mut table.morning),
        ("day", &mut table.day),
        ("night", &mut table.night),
    ] {
        ensure!(slots.len() == N, "native encounter slot extent");
        let residents = group.and_then(|g| g.residents.get(time));
        for (i, (slot, level)) in slots.iter_mut().zip(levels).enumerate() {
            slot.level = *level;
            if let Some(residents) = residents {
                slot.species.clone_from(&residents[i]);
            } else if let Some(next) = group.and_then(|g| g.promotions.get(&slot.species)) {
                slot.species.clone_from(next);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "minnesota_encounter_progression_tests.rs"]
mod tests;
