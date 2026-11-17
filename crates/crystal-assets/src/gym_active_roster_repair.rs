//! Replace retired species in the composed Minnesota gym review lineage.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::models::LearnedMove;
use serde::Deserialize;
use serde_json::Value;

use crate::CompiledGamePack;

const PARENT: &str = "14f4840757c869f1f1aa4edc6ab71dbc2d587089f94f7a37e1d055c75621aa9e";
const MANIFEST: &str = "minnesota-gym-active-roster-review-v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Repair {
    schema_version: u8,
    status: String,
    replacements: Vec<Replacement>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Replacement {
    trainer: String,
    slot: usize,
    #[serde(rename = "from")]
    previous: String,
    species: String,
    level: u8,
    moves: Vec<String>,
}

/// Repairs retired gym species using the current curated Minnesota roster.
///
/// Only declared party species and their legal level-up moves change. Levels,
/// held items, DVs, trainer identities and all map content are preserved.
///
/// # Errors
/// Rejects an incompatible parent, malformed curation, duplicate or stale slot,
/// retired replacement, illegal move, overdue level evolution, remaining retired
/// gym species, or invalid compiled pack.
pub fn repair_gym_active_roster(
    base: &CompiledGamePack,
    source: &str,
    roster: &str,
    curation: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity.content_hash == PARENT,
        "eight-gym review parent required"
    );
    let repair: Repair = serde_json::from_str(source)?;
    ensure!(
        repair.schema_version == 1 && !repair.status.is_empty() && repair.replacements.len() == 11,
        "eleven declared gym repairs required"
    );
    let roster: Value = serde_json::from_str(roster)?;
    let curation: Value = serde_json::from_str(curation)?;
    let names = |key: &str| -> Result<BTreeSet<String>> {
        curation[key]
            .as_array()
            .with_context(|| format!("curation {key}"))?
            .iter()
            .map(|entry| Ok(entry["id"].as_str().context("curation id")?.to_owned()))
            .collect()
    };
    let retired = names("retired_families")?;
    let removed = names("removed_evolutions")?;
    let mut active = BTreeSet::new();
    for family in roster["families"].as_array().context("roster families")? {
        let id = family["id"].as_str().context("family id")?;
        if retired.contains(id) {
            continue;
        }
        for member in family["members"].as_array().context("family members")? {
            let id = member["id"].as_str().context("member id")?;
            if !removed.contains(id) {
                active.insert(
                    member["name"]
                        .as_str()
                        .context("member name")?
                        .to_ascii_uppercase(),
                );
            }
        }
    }
    let mut pack = base.clone();
    let mut slots = BTreeSet::new();
    for change in &repair.replacements {
        ensure!(
            change.trainer.starts_with("MN_GYM_")
                && slots.insert((&change.trainer, change.slot))
                && !active.contains(&change.previous)
                && active.contains(&change.species)
                && change.moves.len() == 4,
            "invalid gym repair {} slot {}",
            change.trainer,
            change.slot
        );
        ensure!(
            pack.data.pokemon.contains_key(&change.species),
            "replacement species absent from pack"
        );
        if let Some(evolutions) = pack.data.evolutions.0.get(&change.species) {
            ensure!(
                !evolutions
                    .iter()
                    .any(|e| e.method == "LEVEL" && e.level.is_some_and(|n| n <= change.level)),
                "{} should already be evolved at level {}",
                change.species,
                change.level
            );
        }
        let learnset = pack
            .data
            .learnsets
            .get(&change.species)
            .context("replacement learnset")?;
        let moves = change
            .moves
            .iter()
            .map(|name| {
                ensure!(
                    learnset
                        .iter()
                        .any(|entry| entry.0 <= change.level && &entry.1 == name),
                    "{} cannot learn {name} at level {}",
                    change.species,
                    change.level
                );
                Ok(LearnedMove {
                    name: name.clone(),
                    current_pp: pack.data.moves.get(name).context("replacement move")?.pp,
                    pp_ups: 0,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let trainer = pack
            .data
            .trainers
            .trainers
            .get_mut(&change.trainer)
            .context("gym trainer")?;
        let member = trainer
            .party
            .get_mut(change.slot)
            .context("gym party slot")?;
        ensure!(
            member.species == change.previous && member.level == change.level,
            "stale party slot {}:{}",
            change.trainer,
            change.slot
        );
        member.species = change.species.clone();
        member.moves = moves;
    }
    for (id, trainer) in &pack.data.trainers.trainers {
        if id.starts_with("MN_GYM_") {
            for member in &trainer.party {
                ensure!(
                    active.contains(&member.species),
                    "retired or nonregional species {} in {id}",
                    member.species
                );
            }
        }
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
