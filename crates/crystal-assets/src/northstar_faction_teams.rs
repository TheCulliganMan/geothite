//! Explicit native teams for ICE and the allied project bosses.
use anyhow::{Context, Result, ensure};
use crystal_core::models::TrainerPartyPokemon;
use crystal_core::models::pokemon::LearnedMove;
use serde::Deserialize;
use serde_json::Value;

use crate::CompiledGamePack;
use crate::northstar_regional_roster::RegionalRoster;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Member {
    species: String,
    level: u8,
    moves: Vec<String>,
    #[serde(default)]
    item: Option<String>,
}

pub(super) fn install(
    pack: &mut CompiledGamePack,
    policy: &RegionalRoster,
    source: &Value,
) -> Result<()> {
    let teams = source["faction_teams"]
        .as_object()
        .context("authored faction teams")?;
    for (id, rows) in teams {
        // Optional chapters are not present in every geography branch. Their
        // explicit teams apply once installed; ordinary missing guards fail.
        let optional = source["optional_faction_teams"]
            .as_array()
            .is_some_and(|ids| ids.iter().any(|entry| entry.as_str() == Some(id)))
            || id == "MN_REED_MINE";
        if optional && !pack.data.trainers.trainers.contains_key(id) {
            continue;
        }
        let members: Vec<Member> = serde_json::from_value(rows.clone())?;
        ensure!(
            !members.is_empty() && members.len() <= 6,
            "native party size {id}"
        );
        let original = pack
            .data
            .trainers
            .trainers
            .get(id)
            .with_context(|| format!("faction trainer {id}"))?;
        let dvs = original.party.first().context("original native DVs")?.dvs;
        let mut party = Vec::new();
        for member in members {
            ensure!(
                policy.target(pack, &member.species, Some(member.level))? == member.species,
                "illegal evolved faction member {id}/{}",
                member.species
            );
            let learned = pack
                .data
                .learnsets
                .get(&member.species)
                .context("native faction learnset")?;
            let machines = &pack.data.pokemon[&member.species].tmhm_learnset;
            ensure!(
                !member.moves.is_empty() && member.moves.len() <= 4,
                "native move slots {id}"
            );
            let unique = member
                .moves
                .iter()
                .collect::<std::collections::BTreeSet<_>>();
            ensure!(
                unique.len() == member.moves.len(),
                "duplicate faction move {id}"
            );
            let moves = member
                .moves
                .iter()
                .map(|name| {
                    ensure!(
                        learned
                            .iter()
                            .any(|entry| entry.0 <= member.level && &entry.1 == name)
                            || machines.contains(name),
                        "illegal move {id}/{}/{name}",
                        member.species
                    );
                    let definition = pack.data.moves.get(name).context("compiled faction move")?;
                    Ok(LearnedMove {
                        name: name.clone(),
                        current_pp: definition.pp,
                        pp_ups: 0,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            if let Some(item) = &member.item {
                ensure!(
                    pack.data.items.contains_key(item),
                    "native held item {item}"
                );
            }
            party.push(TrainerPartyPokemon {
                species: member.species,
                level: member.level,
                item: member.item,
                moves,
                dvs,
            });
        }
        let ai = source["faction_ai_templates"]
            .get(id)
            .and_then(Value::as_str)
            .map(|template| {
                let trainer = pack
                    .data
                    .trainers
                    .trainers
                    .get(template)
                    .context("native boss AI template")?;
                Ok::<_, anyhow::Error>((
                    trainer.ai_move_flags,
                    trainer.ai_item_switch_flags,
                    trainer.ai_layers.clone(),
                ))
            })
            .transpose()?;
        let trainer = pack
            .data
            .trainers
            .trainers
            .get_mut(id)
            .context("faction trainer")?;
        trainer.party = party;
        if let Some((move_flags, switch_flags, layers)) = ai {
            trainer.ai_move_flags = move_flags;
            trainer.ai_item_switch_flags = switch_flags;
            trainer.ai_layers = layers;
        }
    }
    Ok(())
}
