//! Compile authored Minnesota habitat profiles into native encounter tables.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use crystal_core::world::encounters::WildEncounter;
use crystal_core::world::encounters::WildEncounterTable;
use serde::Deserialize;

use crate::CompiledGamePack;
use crate::icemyth_discovery_pack;
use crate::minnesota_biome_ecology;
use crate::minnesota_region_field_ecology;
use crate::minnesota_region_field_ecology::{PreserveHabitat, TreeSite, Trees};
use crate::minnesota_region_rod_ecology;
use crate::minnesota_runtime_policy::DEX_MEMBER_PREFIX;
use crate::northstar_regional_roster::RegionalRoster;

const MANIFEST: &str = "minnesota-region-ecology-v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    schema_version: u8,
    parent_content_hash: String,
    walking_profiles: BTreeMap<String, WalkingProfile>,
    water_profiles: BTreeMap<String, Vec<String>>,
    journey_habitats: BTreeMap<String, Habitat>,
    map_overrides: BTreeMap<String, Habitat>,
    zone_specialists: BTreeMap<String, Vec<String>>,
    walking_signatures: BTreeMap<String, Vec<String>>,
    surf_signatures: BTreeMap<String, Vec<String>>,
    headbutt: BTreeMap<String, Trees>,
    new_headbutt: BTreeMap<String, TreeSite>,
    tree_profiles: BTreeMap<String, Trees>,
    safari: BTreeMap<String, PreserveHabitat>,
    rod_habitats: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct Habitat(Option<String>, String);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WalkingProfile {
    residents: Vec<String>,
    night: Vec<String>,
    specialists: Vec<String>,
}

#[derive(Deserialize)]
struct Progression {
    groups: Vec<Journey>,
}

#[derive(Deserialize)]
struct Journey {
    id: String,
    maps: Vec<String>,
}

/// Rebuild native walking, Surf, fishing, Headbutt, and Safari tables for the reviewed Minnesota pack.
///
/// The plan assigns geographic species pools to every journey band. Levels,
/// encounter rates, geographic zones and slot weights are inherited.
///
/// # Errors
/// Rejects a stale parent, incomplete map coverage, illegal species, or invalid
/// runtime content.
pub fn build_minnesota_region_ecology(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::verify_northstar_publication_geometry(base)?;
    let plan: Plan = serde_json::from_str(source)?;
    ensure!(
        plan.schema_version == 1 && plan.parent_content_hash == base.identity()?.content_hash,
        "regional ecology plan must match its reviewed parent"
    );
    let progression: Progression = serde_json::from_str(include_str!(
        "../../../planning/minnesota/pokemon/acquisition-v01/encounter-progression-v01/progression.json"
    ))?;
    let policy = RegionalRoster::current()?;
    let mut pack = base.clone();
    register_research_members(&mut pack, &policy)?;
    let mut reviewed = BTreeSet::new();
    let mut reviewed_zones = BTreeSet::new();
    ensure!(
        progression.groups.len() == plan.journey_habitats.len(),
        "every journey needs exactly one habitat assignment"
    );
    for journey in &progression.groups {
        let default = plan
            .journey_habitats
            .get(&journey.id)
            .with_context(|| format!("missing habitat for {}", journey.id))?;
        for (ordinal, map) in journey.maps.iter().enumerate() {
            ensure!(
                map.starts_with("Mn") && reviewed.insert(map.as_str()),
                "unique Minnesota map required: {map}"
            );
            let habitat = plan.map_overrides.get(map).unwrap_or(default);
            let table = pack
                .data
                .wild_encounters
                .get_mut(map)
                .with_context(|| format!("missing encounter table for {map}"))?;
            let walking = habitat
                .0
                .as_ref()
                .map(|id| {
                    plan.walking_profiles
                        .get(id)
                        .context("missing walking profile")
                })
                .transpose()?;
            let water = plan
                .water_profiles
                .get(&habitat.1)
                .context("missing water profile")?;
            if let Some(grass) = &mut table.grass {
                install_walking(
                    base,
                    &policy,
                    grass,
                    walking.context("walking profile required")?,
                    ordinal,
                )
                .with_context(|| format!("walking table in {map}"))?;
                if let Some(signatures) = plan.walking_signatures.get(map) {
                    install_signatures(base, &policy, grass, signatures)?;
                }
            }
            if let Some(surf) = &mut table.water {
                for (period, slots) in periods(surf).into_iter().enumerate() {
                    install_slots(base, &policy, slots, water, water, ordinal + period.min(1))?;
                }
            }
            if let Some(signatures) = plan.surf_signatures.get(map) {
                install_signatures(
                    base,
                    &policy,
                    table
                        .water
                        .as_mut()
                        .context("Surf signature needs a table")?,
                    signatures,
                )?;
            }
            for zone in &mut table.zones {
                let specialist = plan
                    .zone_specialists
                    .get(&zone.id)
                    .with_context(|| format!("missing zone specialists for {}", zone.id))?;
                reviewed_zones.insert(zone.id.as_str().to_owned());
                let profile = walking.context("zone needs an adjacent walking profile")?;
                let pool: Vec<_> = specialist
                    .iter()
                    .chain(&profile.residents)
                    .chain(&profile.specialists)
                    .cloned()
                    .collect();
                for slots in periods(&mut zone.grass) {
                    install_slots(base, &policy, slots, &pool, specialist, 0)?;
                }
            }
            for swarm in table.swarm_overrides.values_mut() {
                install_walking(
                    base,
                    &policy,
                    &mut swarm.grass,
                    walking.context("swarm needs walking profile")?,
                    ordinal,
                )?;
            }
        }
    }
    ensure!(
        reviewed
            == base
                .data
                .wild_encounters
                .keys()
                .filter(|name| name.starts_with("Mn"))
                .map(String::as_str)
                .collect()
            && reviewed_zones.len() == plan.zone_specialists.len()
            && plan
                .map_overrides
                .keys()
                .all(|name| reviewed.contains(name.as_str())),
        "ecology source must cover exactly the regional wild maps and zones"
    );
    ensure!(
        plan.walking_signatures
            .keys()
            .chain(plan.surf_signatures.keys())
            .all(|name| reviewed.contains(name.as_str())),
        "signature map must have an installed native table"
    );
    minnesota_region_field_ecology::install(
        base,
        &mut pack,
        &policy,
        &plan.headbutt,
        &plan.safari,
    )?;
    minnesota_region_field_ecology::install_new_trees(
        base,
        &mut pack,
        &policy,
        &plan.new_headbutt,
        &plan.tree_profiles,
    )?;
    minnesota_region_rod_ecology::install(
        base,
        &mut pack,
        &policy,
        &plan.rod_habitats,
        &plan.water_profiles,
    )?;
    if !policy.allowed.contains("ICEMYTH") {
        icemyth_discovery_pack::retire_exhibit(&mut pack)?;
    }
    pack.report.manifests.push(MANIFEST.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

fn register_research_members(pack: &mut CompiledGamePack, policy: &RegionalRoster) -> Result<()> {
    let constants = &mut pack.data.story_event_script_constants.global;
    let prior_count = constants
        .keys()
        .filter(|key| key.starts_with(DEX_MEMBER_PREFIX))
        .count();
    ensure!(
        prior_count > 0 && prior_count == policy.allowed.len(),
        "research reward count needs its own story revision"
    );
    for name in &policy.allowed {
        ensure!(
            pack.data.pokemon.contains_key(name),
            "compile active species first: {name}"
        );
    }
    constants.retain(|key, _| !key.starts_with(DEX_MEMBER_PREFIX));
    for name in &policy.allowed {
        constants.insert(format!("{DEX_MEMBER_PREFIX}{name}"), 1);
    }
    Ok(())
}

fn periods(table: &mut WildEncounterTable) -> [&mut [WildEncounter]; 3] {
    [&mut table.morning, &mut table.day, &mut table.night]
}

fn install_signatures(
    base: &CompiledGamePack,
    policy: &RegionalRoster,
    table: &mut WildEncounterTable,
    signatures: &[String],
) -> Result<()> {
    for slots in periods(table) {
        ensure!(
            signatures.len() < slots.len(),
            "signature replaces the common residents"
        );
        for (slot, species) in slots.iter_mut().rev().zip(signatures) {
            minnesota_biome_ecology::validate_active_species(base, policy, species, slot.level)
                .with_context(|| format!("signature encounter {species}"))?;
            slot.species.clone_from(species);
        }
    }
    Ok(())
}

fn install_walking(
    base: &CompiledGamePack,
    policy: &RegionalRoster,
    table: &mut WildEncounterTable,
    profile: &WalkingProfile,
    ordinal: usize,
) -> Result<()> {
    let night: Vec<_> = profile
        .night
        .iter()
        .chain(&profile.residents)
        .cloned()
        .collect();
    for (period, slots) in periods(table).into_iter().enumerate() {
        let residents = if period == 2 {
            &night
        } else {
            &profile.residents
        };
        install_slots(
            base,
            policy,
            slots,
            residents,
            &profile.specialists,
            ordinal,
        )?;
    }
    Ok(())
}

fn install_slots(
    base: &CompiledGamePack,
    policy: &RegionalRoster,
    slots: &mut [WildEncounter],
    residents: &[String],
    specialists: &[String],
    ordinal: usize,
) -> Result<()> {
    ensure!(
        !residents.is_empty() && !specialists.is_empty(),
        "empty habitat pool"
    );
    let mut used = BTreeSet::new();
    for (index, slot) in slots.iter_mut().enumerate() {
        let (primary, secondary) = if index < 3 {
            (residents, specialists)
        } else {
            (specialists, residents)
        };
        let offset = (index + ordinal) % primary.len();
        let candidates = primary[offset..]
            .iter()
            .chain(&primary[..offset])
            .chain(secondary);
        let mut fallback = None;
        for candidate in candidates {
            let target = policy.target(base, candidate, Some(slot.level))?;
            if !used.contains(&target) {
                fallback = Some(target);
                break;
            }
            fallback.get_or_insert(target);
        }
        let species = fallback.context("habitat has no legal resident")?;
        minnesota_biome_ecology::validate_active_species(base, policy, &species, slot.level)?;
        used.insert(species.clone());
        slot.species = species;
    }
    Ok(())
}
