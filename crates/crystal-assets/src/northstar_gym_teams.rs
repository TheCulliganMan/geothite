//! Type-coherent Gym parties with preserved level slots and native trainer IDs.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use crystal_core::models::LearnedMove;
use serde::Deserialize;
use serde_json::Value;

use crate::CompiledGamePack;
use crate::gym_design::Member;
use crate::northstar_regional_roster::RegionalRoster;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Teams {
    schema_version: u8,
    parties: BTreeMap<String, Vec<Member>>,
}

pub(crate) fn install(pack: &mut CompiledGamePack, source: &str) -> Result<()> {
    let source: Value = serde_json::from_str(source)?;
    let teams: Teams = serde_json::from_value(source["gym_overhaul_teams"].clone())?;
    let expected: BTreeSet<_> = (1..=8)
        .flat_map(|slot| {
            std::iter::once(format!("MN_GYM_{slot}")).chain(
                (1..=if slot >= 7 { 3 } else { 2 })
                    .map(move |index| format!("MN_GYM_{slot}_TRAINER_{index}")),
            )
        })
        .collect();
    ensure!(
        teams.schema_version == 1
            && teams.parties.keys().cloned().collect::<BTreeSet<_>>() == expected,
        "exactly eight leaders and eighteen supporting teams"
    );
    let policy = RegionalRoster::current()?;
    for (id, members) in teams.parties {
        let old = pack.data.trainers.get(&id).context("installed Gym party")?;
        let old_levels: Vec<_> = old.party.iter().map(|member| member.level).collect();
        let expected_levels = match id.as_str() {
            "MN_GYM_3" => {
                ensure!(old_levels == [19, 20, 21], "Electric leader level preimage");
                vec![19, 20, 22]
            }
            "MN_GYM_7" => {
                ensure!(old_levels == [38, 39, 40], "Grass leader level preimage");
                vec![38, 38, 39, 40]
            }
            _ => old_levels,
        };
        ensure!(
            members
                .iter()
                .map(|member| member.level)
                .eq(expected_levels),
            "only the reviewed Electric evolution and fourth Grass partner may change level slots: {id}"
        );
        let party = members
            .into_iter()
            .enumerate()
            .map(|(index, member)| {
                // The additional Grass partner inherits the native lead's DVs;
                // later existing slots retain their own DVs and held items.
                let source_index = if id == "MN_GYM_7" {
                    [0, 0, 1, 2][index]
                } else {
                    index
                };
                let before = &old.party[source_index];
                ensure!(
                    policy.target(pack, &member.species, Some(member.level))? == member.species,
                    "active, level-legal regional Gym member"
                );
                ensure!(
                    (1..=4).contains(&member.moves.len())
                        && member.moves.iter().collect::<BTreeSet<_>>().len() == member.moves.len(),
                    "distinct legal move slots"
                );
                let learnset = pack
                    .data
                    .learnsets
                    .get(&member.species)
                    .context("Gym learnset")?;
                let machines = &pack.data.pokemon[&member.species].tmhm_learnset;
                let moves = member
                    .moves
                    .iter()
                    .map(|name| {
                        ensure!(
                            machines.contains(name)
                                || learnset
                                    .iter()
                                    .any(|entry| entry.0 <= member.level && &entry.1 == name),
                            "illegal Gym move {}/{name} at {}",
                            member.species,
                            member.level
                        );
                        let definition = pack.data.moves.get(name).context("native Gym move")?;
                        Ok(LearnedMove {
                            name: name.clone(),
                            current_pp: definition.pp,
                            pp_ups: 0,
                        })
                    })
                    .collect::<Result<_>>()?;
                let mut after = before.clone();
                after.species = member.species;
                after.level = member.level;
                after.moves = moves;
                Ok(after)
            })
            .collect::<Result<Vec<_>>>()?;
        pack.data
            .trainers
            .trainers
            .get_mut(&id)
            .context("retained Gym trainer")?
            .party = party;
    }
    Ok(())
}
