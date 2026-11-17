//! Reviewed campaign tuning on the installed adventure release.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::CompiledGamePack;
use crate::northstar_battle_balance::{self, AuthoredMember};
use crate::northstar_regional_roster::RegionalRoster;

const MANIFEST: &str = "northstar-battle-economy-refinement-v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Adjustment {
    trainer: String,
    party_slot: usize,
    original_species: String,
    original_moves: Vec<String>,
    member: AuthoredMember,
    rationale: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReedChallenge {
    trainer_id: String,
    original_trainer: String,
    level_range: [u8; 2],
    party: Vec<AuthoredMember>,
}

/// Patches current teams and services without replaying historical builders.
pub(crate) fn install(
    pack: &mut CompiledGamePack,
    story_source: &str,
    pokemon_source: &str,
) -> Result<()> {
    ensure!(
        !pack.report.manifests.iter().any(|entry| entry == MANIFEST),
        "battle refinement already installed"
    );
    let source: Value = serde_json::from_str(pokemon_source)?;
    let adjustments: Vec<Adjustment> =
        serde_json::from_value(source["refinement_team_adjustments"].clone())?;
    ensure!(adjustments.len() == 2, "complete reviewed tuning required");
    let policy = RegionalRoster::current()?;
    let mut occupied = BTreeSet::new();
    for adjustment in adjustments {
        ensure!(
            occupied.insert((adjustment.trainer.clone(), adjustment.party_slot))
                && !adjustment.rationale.is_empty(),
            "duplicate or unexplained refinement"
        );
        let index = adjustment
            .party_slot
            .checked_sub(1)
            .context("one-based slot")?;
        let trainer = pack
            .data
            .trainers
            .get(&adjustment.trainer)
            .context("tuned trainer")?;
        let original = trainer.party.get(index).context("tuned trainer slot")?;
        ensure!(
            original.species == adjustment.original_species
                && original.level == adjustment.member.level
                && original.item == adjustment.member.item
                && original
                    .moves
                    .iter()
                    .map(|entry| &entry.name)
                    .eq(adjustment.original_moves.iter()),
            "refinement preimage changed: {}/{}",
            adjustment.trainer,
            adjustment.party_slot
        );
        let member = northstar_battle_balance::build_member(
            pack,
            &policy,
            &adjustment.member,
            original.dvs,
        )?;
        pack.data
            .trainers
            .trainers
            .get_mut(&adjustment.trainer)
            .context("retained trainer")?
            .party[index] = member;
    }
    crate::northstar_refinement_economy::install(pack, story_source)?;
    crate::northstar_rematch_circuit::install(pack, story_source)?;
    pack.report.manifests.push(MANIFEST.into());
    Ok(())
}

/// Registers Reed's optional final team; its scene owns the battle and win receipt.
pub(crate) fn register_reed_challenge(
    pack: &mut CompiledGamePack,
    pokemon_source: &str,
) -> Result<String> {
    let source: Value = serde_json::from_str(pokemon_source)?;
    let challenge: ReedChallenge =
        serde_json::from_value(source["refinement_reed_challenge"].clone())?;
    ensure!(
        challenge.trainer_id == "MN_REED_REFINEMENT"
            && challenge.original_trainer == "MN_REED_BOUNDARY"
            && challenge.level_range == [57, 59]
            && challenge.party.len() == 6
            && challenge
                .party
                .iter()
                .map(|member| &member.species)
                .collect::<BTreeSet<_>>()
                .len()
                == 6
            && challenge
                .party
                .iter()
                .all(|member| (57..=59).contains(&member.level)),
        "Reed challenge identity, size, diversity and disclosed levels"
    );
    ensure!(
        !pack
            .data
            .trainers
            .trainers
            .contains_key(&challenge.trainer_id)
            && !pack
                .data
                .story_event_script_constants
                .global
                .contains_key(&challenge.trainer_id),
        "Reed challenge already registered"
    );
    let mut trainer = pack
        .data
        .trainers
        .get(&challenge.original_trainer)
        .context("Reed boundary team")?
        .clone();
    ensure!(
        trainer
            .party
            .iter()
            .any(|member| member.species == "CROOKVEIL")
            && challenge
                .party
                .iter()
                .any(|member| member.species == "CROOKVEIL")
            && challenge
                .party
                .last()
                .is_some_and(|member| member.species == "MOSQUITAN"),
        "Reed's Crookveil and evolved Pupump continuity"
    );
    let dvs = trainer.party.first().context("Reed DVs")?.dvs;
    let next = pack
        .data
        .trainers
        .trainers
        .values()
        .filter(|entry| entry.trainer_class == trainer.trainer_class)
        .filter_map(|entry| {
            pack.data
                .story_event_script_constants
                .global
                .get(&entry.trainer_id)
        })
        .copied()
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .context("Reed trainer constant overflow")?;
    trainer.trainer_id = challenge.trainer_id.clone();
    trainer.items.clear();
    trainer.win_quote.clear();
    trainer.lose_quote.clear();
    let policy = RegionalRoster::current()?;
    trainer.party = challenge
        .party
        .iter()
        .map(|member| northstar_battle_balance::build_member(pack, &policy, member, dvs))
        .collect::<Result<_>>()?;
    let class = trainer.trainer_class.clone();
    pack.data.trainers.insert(trainer)?;
    pack.data
        .story_event_script_constants
        .global
        .insert(challenge.trainer_id, next);
    Ok(class)
}
