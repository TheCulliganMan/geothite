//! Authored river, lake, spring and cave wildlife on the current regional map.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use crystal_core::world::encounters::{WildEncounterData, WildEncounterTable};
use serde::Deserialize;

use crate::CompiledGamePack;
use crate::authored_route_surface::{reachable_map_with_surf, walking_map_distances};
use crate::authored_town_geometry::kind;

const MANIFEST: &str = "minnesota-water-cave-habitats-v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    schema_version: u8,
    id: String,
    parent_content_hash: String,
    reference_structure: String,
    maps: Vec<Habitat>,
    exceptions: Vec<Exception>,
    fishing_repairs: Vec<FishingRepair>,
    moorhead_access: crate::minnesota_water_cave_access::Access,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Habitat {
    map: String,
    walking: Option<Walking>,
    water: Option<Water>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Walking {
    theme: String,
    rate: u8,
    anchor: [u16; 2],
    sample: [u16; 2],
    #[serde(rename = "loop")]
    next: [u16; 2],
    table: WildEncounterTable,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Water {
    theme: String,
    rate: u8,
    #[serde(default)]
    replace_zero_rate_fallback: bool,
    anchor: [u16; 2],
    shore: [u16; 2],
    sample: [u16; 2],
    #[serde(rename = "loop")]
    next: [u16; 2],
    table: WildEncounterTable,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Exception {
    map: String,
    surface: String,
    reason: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FishingRepair {
    map: String,
    before: String,
    after: String,
}

/// Adds the missing ordinary water/cave habitats and a bounded Red River access.
///
/// Existing active wild tables, grass zones, fishing catalogs, story rules, native art
/// and audio are preserved. Three healer floors stay peaceful. Gym puzzle water
/// and the separately controlled Safari remain excluded. Later template rooms
/// lose their original-region fishing headers; the wet intake uses the existing
/// regional shore group. The external pinned Sweet Scent source restores that
/// native field entry without replacing existing program definitions.
/// Static access assumes the required field/story gates;
/// production-controller validation is still needed to prove gameplay.
///
/// # Errors
/// Rejects stale parents, duplicate/incomplete plans, occupied tables, invalid
/// regional species/levels, inaccessible samples, unexpected fishing headers,
/// unsafe Moorhead geometry or invalid compiled output. The parent is borrowed
/// and unchanged on failure.
pub fn build_minnesota_water_cave_habitats(
    base: &CompiledGamePack,
    source: &str,
    native_sweet_scent: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::verify_northstar_publication_geometry(base)?;
    let source: Source = serde_json::from_str(source)?;
    ensure!(
        source.schema_version == 1
            && source.id == MANIFEST
            && !source.reference_structure.is_empty()
            && source.maps.len() == 26
            && source.exceptions.len() == 10
            && source.fishing_repairs.len() == 26
            && base.identity()?.content_hash == source.parent_content_hash
            && !base.report.manifests.iter().any(|id| id == MANIFEST),
        "complete current-parent water/cave plan required"
    );
    let policy = crate::authored_regional_approach::policy()?;
    let mut pack = base.clone();
    crate::minnesota_sweet_scent_source::install(&mut pack, native_sweet_scent)?;
    crate::minnesota_water_cave_access::install(&mut pack, &source.moorhead_access)?;
    repair_fishing(&mut pack, &source.fishing_repairs)?;
    let mut names = BTreeSet::new();
    let mut counts = [0, 0];
    for habitat in &source.maps {
        ensure!(
            habitat.map.starts_with("Mn") && names.insert(&habitat.map),
            "unique regional map required"
        );
        let map = pack.data.maps.get(&habitat.map).context("habitat map")?;
        let mut table = pack
            .data
            .wild_encounters
            .get(&habitat.map)
            .cloned()
            .unwrap_or_else(|| WildEncounterData {
                map_name: habitat.map.clone(),
                ..Default::default()
            });
        if let Some(walking) = &habitat.walking {
            ensure!(
                map.attributes.environment.as_deref() == Some("CAVE")
                    && table.grass.is_none()
                    && table.grass_rates.is_none()
                    && !walking.theme.is_empty()
                    && matches!(walking.rate, 5 | 10),
                "missing cave floor table required: {}",
                habitat.map
            );
            validate_slots(&pack, &policy, &walking.table, 7)?;
            let occupied = map.objects.iter().map(|o| [o.x, o.y]).collect();
            let dry = walking_map_distances(&pack, map, walking.anchor, &occupied)?;
            ensure!(
                adjacent(walking.sample, walking.next)
                    && kind(&pack, map, walking.sample)? == "FLOOR"
                    && kind(&pack, map, walking.next)? == "FLOOR"
                    && dry.contains_key(&walking.sample)
                    && dry.contains_key(&walking.next),
                "reachable native cave-floor loop required: {}",
                habitat.map
            );
            table.grass = Some(walking.table.clone());
            table.grass_rates = Some(BTreeMap::from_iter(
                ["morning", "day", "night"].map(|time| (time.into(), walking.rate)),
            ));
            counts[0] += 1;
        }
        if let Some(water) = &habitat.water {
            let vacant = if water.replace_zero_rate_fallback {
                table.water.is_some() && table.water_rate == Some(0)
            } else {
                table.water.is_none() && table.water_rate.is_none()
            };
            ensure!(
                vacant && !water.theme.is_empty() && water.rate == 5,
                "missing or explicitly declared zero-rate native Surf table required: {}",
                habitat.map
            );
            validate_slots(&pack, &policy, &water.table, 3)?;
            let occupied = map.objects.iter().map(|o| [o.x, o.y]).collect();
            let dry = walking_map_distances(&pack, map, water.anchor, &occupied)?;
            let wet = reachable_map_with_surf(&pack, map, water.anchor, &occupied)?;
            ensure!(
                dry.contains_key(&water.shore)
                    && kind(&pack, map, water.shore)? == "FLOOR"
                    && kind(&pack, map, water.sample)? == "WATER"
                    && kind(&pack, map, water.next)? == "WATER"
                    && adjacent(water.shore, water.sample)
                    && adjacent(water.sample, water.next)
                    && wet.contains(&water.sample)
                    && wet.contains(&water.next),
                "reachable native shore and adjacent Surf loop required: {}",
                habitat.map
            );
            table.water = Some(water.table.clone());
            table.water_rate = Some(water.rate);
            counts[1] += 1;
        }
        ensure!(
            habitat.walking.is_some() || habitat.water.is_some(),
            "empty habitat"
        );
        pack.data.wild_encounters.insert(habitat.map.clone(), table);
    }
    ensure!(counts == [10, 19], "ten cave and19 water habitats required");
    validate_coverage(&pack, &source.exceptions)?;
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

fn adjacent(a: [u16; 2], b: [u16; 2]) -> bool {
    a[0].abs_diff(b[0]) + a[1].abs_diff(b[1]) == 1
}

fn validate_slots(
    pack: &CompiledGamePack,
    policy: &crate::northstar_regional_roster::RegionalRoster,
    table: &WildEncounterTable,
    length: usize,
) -> Result<()> {
    for slots in [&table.morning, &table.day, &table.night] {
        ensure!(slots.len() == length, "native slot count required");
        for slot in slots {
            ensure!(
                (1..=100).contains(&slot.level)
                    && policy.target(pack, &slot.species, Some(slot.level))? == slot.species
                    && pack
                        .data
                        .story_event_script_constants
                        .global
                        .contains_key(&format!("MN_DEX_MEMBER_{}", slot.species)),
                "active regional species at its legal level required: {}",
                slot.species
            );
        }
    }
    Ok(())
}

fn repair_fishing(pack: &mut CompiledGamePack, repairs: &[FishingRepair]) -> Result<()> {
    let expected = pack
        .data
        .maps
        .iter()
        .filter(|(name, map)| {
            name.starts_with("Mn")
                && matches!(
                    map.attributes.fishing_group.as_deref(),
                    Some("FISHGROUP_LAKE" | "FISHGROUP_SHORE")
                )
        })
        .map(|(name, _)| name.clone())
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    for repair in repairs {
        ensure!(seen.insert(repair.map.clone()), "duplicate fishing repair");
        let map = pack.data.maps.get(&repair.map).context("fishing map")?;
        let wet = map.blocks.iter().any(|b| {
            pack.data.tilesets[&map.attributes.tileset_name].collision[&format!("{b:02x}")]
                .iter()
                .any(|c| c == "WATER")
        });
        let target = if wet {
            ensure!(
                repair.map == "MnBoundaryIntake",
                "unexpected wet original-region binding"
            );
            "FISHGROUP_MN_SHORE"
        } else {
            "FISHGROUP_NONE"
        };
        ensure!(
            map.attributes.fishing_group.as_deref() == Some(&repair.before)
                && repair.after == target,
            "fishing repair preimage differs: {}",
            repair.map
        );
        let raw = pack
            .data
            .map_attributes
            .get_mut(&repair.map)
            .context("raw fishing map")?;
        ensure!(
            raw.fishing_group.as_deref() == Some(&repair.before),
            "raw fishing preimage differs"
        );
        raw.fishing_group = Some(repair.after.clone());
        pack.data
            .maps
            .get_mut(&repair.map)
            .context("fishing map")?
            .attributes
            .fishing_group = Some(repair.after.clone());
    }
    ensure!(
        seen == expected,
        "complete inherited fishing repair set required"
    );
    Ok(())
}

fn validate_coverage(pack: &CompiledGamePack, exceptions: &[Exception]) -> Result<()> {
    let mut allowed = BTreeSet::new();
    for exception in exceptions {
        ensure!(
            !exception.reason.is_empty()
                && allowed.insert((exception.map.as_str(), exception.surface.as_str())),
            "distinct explained exception required"
        );
    }
    let mut used = BTreeSet::new();
    for (name, map) in pack
        .data
        .maps
        .iter()
        .filter(|(name, _)| name.starts_with("Mn"))
    {
        let table = pack.data.wild_encounters.get(name);
        for (surface, needed, present) in [
            (
                "walking",
                map.attributes.environment.as_deref() == Some("CAVE"),
                table.is_some_and(|t| t.grass.is_some()),
            ),
            (
                "water",
                map.blocks.iter().any(|b| {
                    pack.data.tilesets[&map.attributes.tileset_name].collision[&format!("{b:02x}")]
                        .iter()
                        .any(|c| c == "WATER")
                }),
                table.is_some_and(|t| t.water.is_some()),
            ),
        ] {
            if needed && !present {
                ensure!(
                    allowed.contains(&(name.as_str(), surface)),
                    "uncovered {surface} surface: {name}"
                );
                used.insert((name.as_str(), surface));
            }
        }
    }
    ensure!(
        used == allowed,
        "exception must identify a real missing surface"
    );
    Ok(())
}

#[cfg(test)]
#[path = "minnesota_water_cave_tests.rs"]
mod tests;
