//! Retire whole authored families without deleting their source or render assets.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Result, ensure};
use serde::Deserialize;

use crate::minnesota_pokemon_profiles::{Profile, Roster};

#[derive(Deserialize)]
struct Curation {
    schema_version: u16,
    active_species_count: usize,
    active_family_count: usize,
    retired_families: Vec<RetiredFamily>,
}

#[derive(Deserialize)]
struct RetiredFamily {
    id: String,
}

/// Keep only active, complete families after validating the full source catalog.
/// Missing curation preserves the complete catalog. Errors leave the input
/// untouched and identify unknown/duplicate cuts or inconsistent declared counts.
pub(crate) fn apply(
    root: &Path,
    roster: &mut Roster,
    profiles: &mut BTreeMap<String, Profile>,
) -> Result<()> {
    let path = root.join("roster-curation.json");
    if !path.try_exists()? {
        return Ok(());
    }
    let curation: Curation = serde_json::from_slice(&fs::read(path)?)?;
    ensure!(curation.schema_version == 1, "unsupported curation schema");
    let retired = curation
        .retired_families
        .iter()
        .map(|family| family.id.as_str())
        .collect::<BTreeSet<_>>();
    ensure!(
        retired.len() == curation.retired_families.len(),
        "duplicate retired family"
    );
    let known = roster
        .families
        .iter()
        .map(|family| family.id.as_str())
        .collect::<BTreeSet<_>>();
    ensure!(retired.is_subset(&known), "unknown retired family");
    let active = roster
        .families
        .iter()
        .filter(|family| !retired.contains(family.id.as_str()));
    let active_families = active.clone().count();
    let active_ids = active
        .flat_map(|family| family.members.iter().map(|member| member.id.clone()))
        .collect::<BTreeSet<_>>();
    ensure!(
        active_families == curation.active_family_count
            && active_ids.len() == curation.active_species_count
            && !active_ids.is_empty(),
        "curation counts differ from active complete families"
    );
    roster
        .families
        .retain(|family| !retired.contains(family.id.as_str()));
    profiles.retain(|id, _| active_ids.contains(id));
    Ok(())
}
