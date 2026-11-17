//! Explicit late-campaign parties, preserving levels, rewards and earned progress.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::models::{Dv, LearnedMove, TrainerPartyPokemon};
use serde::Deserialize;
use serde_json::Value;

use crate::CompiledGamePack;
use crate::northstar_regional_roster::RegionalRoster;

const MANIFEST: &str = "northstar-campaign-party-balance-v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AuthoredMember {
    pub species: String,
    pub level: u8,
    pub moves: Vec<String>,
    #[serde(default)]
    pub item: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Adjustment {
    trainer: String,
    party_slot: usize,
    original_species: String,
    species: String,
    level: u8,
    original_moves: Vec<String>,
    moves: Vec<String>,
    rationale: String,
}

/// Applies reviewed slot replacements without changing their encounter contracts.
pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let source: Value = serde_json::from_str(include_str!(
        "../../../planning/minnesota/story-pokemon.json"
    ))?;
    let adjustments: Vec<Adjustment> =
        serde_json::from_value(source["campaign_team_adjustments"].clone())?;
    ensure!(
        adjustments.len() == 12,
        "complete campaign party adjustments"
    );
    let policy = RegionalRoster::current()?;
    let mut occupied = BTreeSet::new();
    for adjustment in adjustments {
        ensure!(
            occupied.insert((adjustment.trainer.clone(), adjustment.party_slot))
                && !adjustment.rationale.is_empty(),
            "duplicate or unexplained campaign adjustment"
        );
        let slot = adjustment
            .party_slot
            .checked_sub(1)
            .context("campaign party slots begin at one")?;
        let trainer = pack
            .data
            .trainers
            .get(&adjustment.trainer)
            .context("installed campaign trainer")?;
        for member in &trainer.party {
            ensure!(
                policy.target(pack, &member.species, Some(member.level))? == member.species,
                "inactive or prematurely evolved campaign member"
            );
        }
        let original = trainer.party.get(slot).context("installed campaign slot")?;
        let old_moves: Vec<_> = original.moves.iter().map(|m| m.name.as_str()).collect();
        let original_matches = original.species == adjustment.original_species
            && old_moves == adjustment.original_moves;
        let installed_matches =
            original.species == adjustment.species && old_moves == adjustment.moves;
        ensure!(
            original.level == adjustment.level && (original_matches || installed_matches),
            "changed campaign party preimage: {}/{}",
            adjustment.trainer,
            adjustment.party_slot
        );
        let member = AuthoredMember {
            species: adjustment.species,
            level: adjustment.level,
            moves: adjustment.moves,
            item: original.item.clone(),
        };
        let replacement = build_member(pack, &policy, &member, original.dvs)?;
        pack.data
            .trainers
            .trainers
            .get_mut(&adjustment.trainer)
            .context("campaign trainer retained")?
            .party[slot] = replacement;
    }
    if !pack.report.manifests.iter().any(|name| name == MANIFEST) {
        pack.report.manifests.push(MANIFEST.into());
    }
    Ok(())
}

/// Builds one active regional member with legal evolution, moves and held item.
pub(crate) fn build_member(
    pack: &CompiledGamePack,
    policy: &RegionalRoster,
    member: &AuthoredMember,
    dvs: Dv,
) -> Result<TrainerPartyPokemon> {
    ensure!((1..=100).contains(&member.level), "native party level");
    ensure!(
        policy.target(pack, &member.species, Some(member.level))? == member.species,
        "inactive or prematurely evolved member {}",
        member.species
    );
    ensure!(
        member.moves.len() == 4 && member.moves.iter().collect::<BTreeSet<_>>().len() == 4,
        "four distinct authored moves required: {}",
        member.species
    );
    let learned = pack
        .data
        .learnsets
        .get(&member.species)
        .context("native party learnset")?;
    let machines = &pack.data.pokemon[&member.species].tmhm_learnset;
    let mut moves = Vec::new();
    for name in &member.moves {
        ensure!(
            learned
                .iter()
                .any(|entry| entry.0 <= member.level && &entry.1 == name)
                || machines.contains(name),
            "illegal authored move {}/{name} at {}",
            member.species,
            member.level
        );
        let definition = pack.data.moves.get(name).context("compiled party move")?;
        moves.push(LearnedMove {
            name: name.clone(),
            current_pp: definition.pp,
            pp_ups: 0,
        });
    }
    if let Some(item) = &member.item {
        ensure!(
            pack.data.items.contains_key(item),
            "unknown party held item {item}"
        );
    }
    Ok(TrainerPartyPokemon {
        species: member.species.clone(),
        level: member.level,
        item: member.item.clone(),
        moves,
        dvs,
    })
}
