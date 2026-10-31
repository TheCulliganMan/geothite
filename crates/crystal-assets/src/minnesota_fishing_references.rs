//! Repair inherited original-region fishing bindings on later authored rooms.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;

const MARKER: &str = "minnesota-current-fishing-references-v1";

/// Rebinds later Minnesota rooms to their existing regional fishing tables.
///
/// Only fishing-group attributes and their raw mirrors change. Original-region
/// tables, terrain, scripts, parties, assets and audio remain unchanged.
///
/// # Errors
/// Rejects repeated installation, an unexpected binding count, missing or
/// nonregional target tables, mismatched raw attributes or invalid runtime data.
pub fn repair_minnesota_fishing_references(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        !base.report.manifests.iter().any(|m| m == MARKER),
        "fresh fishing reference parent required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let policy = crate::authored_regional_approach::policy()?;
    let mut pack = base.clone();
    let mut changed = 0;
    for (name, map) in pack
        .data
        .maps
        .iter_mut()
        .filter(|(name, _)| name.starts_with("Mn"))
    {
        let Some(old) = map.attributes.fishing_group.as_deref() else {
            continue;
        };
        let target = match old {
            "FISHGROUP_SHORE" => "FISHGROUP_MN_SHORE",
            "FISHGROUP_LAKE" => "FISHGROUP_MN_LAKE",
            _ => continue,
        };
        let group = base
            .data
            .fishing
            .groups
            .get(target)
            .context("existing regional fishing table")?;
        for table in group.rod_tables.values() {
            for slot in &table.slots {
                if let Some(species) = &slot.species {
                    ensure!(
                        policy.target(base, species, Some(slot.level))? == *species,
                        "active regional fish required"
                    );
                }
                if let Some(time) = &slot.time_group {
                    let time = base
                        .data
                        .fishing
                        .time_groups
                        .get(time)
                        .context("regional timed fish")?;
                    for (species, level) in [
                        (&time.day_species, time.day_level),
                        (&time.night_species, time.night_level),
                    ] {
                        ensure!(
                            policy.target(base, species, Some(level))? == *species,
                            "active regional timed fish required"
                        );
                    }
                }
            }
        }
        let raw = pack
            .data
            .map_attributes
            .get_mut(name)
            .context("existing raw map attributes")?;
        ensure!(
            raw.fishing_group.as_deref() == Some(old),
            "raw fishing binding mismatch: {name}"
        );
        raw.fishing_group = Some(target.into());
        map.attributes.fishing_group = Some(target.into());
        changed += 1;
    }
    ensure!(
        changed == 34,
        "expected34 inherited bindings, found {changed}"
    );
    pack.report.manifests.push(MARKER.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    crate::verify_northstar_publication_geometry(&pack)?;
    crate::validate_compact_superior_geography(&pack)?;
    Ok(pack)
}
