//! Carry only the reviewed two-habitat correction into the current full game.
use anyhow::{Result, ensure};

use crate::CompiledGamePack;

const BEFORE: &str = "e6972fb736c8efd788b0f7d5ac942550a7dc99c7aecd429dad4ca0f1c4c1ac04";
const MARKER: &str = "minnesota-encounter-refinements-v1";
const MAPS: [&str; 2] = ["MnR11C00R00", "MnR11C00R01"];

/// Composes the saved habitat correction without republishing current geography.
///
/// `before` is the exact original complete progression child. `after` must be
/// its freshly rebuilt successor with precisely two changed wild habitats.
/// Every other data field and all art/audio must match. Current title, maps,
/// story, services, teams and optional discoveries remain authoritative.
///
/// # Errors
/// Rejects stale or already modified current tables, an unexpected donor delta,
/// repeated installation, and invalid resulting runtime content.
pub fn compose_minnesota_encounter_refinements(
    current: &CompiledGamePack,
    before: &CompiledGamePack,
    after: &CompiledGamePack,
) -> Result<CompiledGamePack> {
    for pack in [current, before, after] {
        crate::verify_compiled_game_pack_for_runtime(pack)?;
    }
    ensure!(
        before.identity()?.content_hash == BEFORE,
        "exact progression ancestor required"
    );
    ensure!(
        current
            .report
            .manifests
            .iter()
            .any(|m| m == "minnesota-encounter-progression-v1")
            && !current.report.manifests.iter().any(|m| m == MARKER),
        "unrefined current progression child required"
    );
    let mut expected = before.data.clone();
    expected
        .wild_encounters
        .clone_from(&after.data.wild_encounters);
    ensure!(
        expected == after.data,
        "habitat donor changed unrelated data"
    );
    ensure!(
        before.runtime_files == after.runtime_files
            && before.compiled_audio == after.compiled_audio
            && before.audio_manifest == after.audio_manifest
            && before.report == after.report,
        "habitat donor changed resources or publication manifests"
    );
    let changed = before
        .data
        .wild_encounters
        .iter()
        .filter(|(name, table)| after.data.wild_encounters.get(*name) != Some(*table))
        .map(|(name, _)| name.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    ensure!(
        changed == std::collections::BTreeSet::from(MAPS),
        "expected two reviewed habitat changes"
    );
    ensure!(
        before.data.wild_encounters.len() == after.data.wild_encounters.len(),
        "habitat count changed"
    );
    let mut result = current.clone();
    for name in MAPS {
        ensure!(
            current.data.wild_encounters.get(name) == before.data.wild_encounters.get(name),
            "newer current habitat needs explicit reconciliation: {name}"
        );
        let table = after
            .data
            .wild_encounters
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("missing refined habitat {name}"))?;
        result
            .data
            .wild_encounters
            .insert(name.into(), table.clone());
    }
    result.report.manifests.push(MARKER.into());
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    Ok(result)
}
