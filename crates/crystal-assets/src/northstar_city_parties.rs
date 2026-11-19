//! Hand-authored partners for the bike lanes and State Fair exhibition matches.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use crystal_core::models::LearnedMove;
use serde::Deserialize;

use crate::CompiledGamePack;
use crate::northstar_regional_roster::RegionalRoster;

const SOURCE: &str =
    include_str!("../../../planning/minnesota/revisions/trail-battles-v01/city-parties.json");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Team {
    name: String,
    party: Vec<Member>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Member {
    from: String,
    species: String,
    level: u8,
    moves: Vec<String>,
}

pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let roster = RegionalRoster::current()?;
    let cities: BTreeMap<String, BTreeMap<String, Team>> = serde_json::from_str(SOURCE)?;
    for (city, teams) in cities {
        let room = pack.data.maps.get(&city).context("city battle map")?;
        for (id, team) in teams {
            ensure!(
                room.objects.iter().any(|o| room
                    .trainer_scripts
                    .get(&o.script)
                    .is_some_and(|s| s.trainer_id == id)),
                "{city} has no battle with {id}"
            );
            let trainer = pack
                .data
                .trainers
                .trainers
                .get_mut(&id)
                .context("city trainer")?;
            ensure!(
                trainer.name == team.name && trainer.party.len() == team.party.len(),
                "changed team for {id}"
            );
            for (slot, member) in trainer.party.iter_mut().zip(team.party) {
                ensure!(
                    slot.species == member.from
                        && slot.level == member.level
                        && roster.allowed.contains(&member.species)
                        && (1..=4).contains(&member.moves.len())
                        && member.moves.iter().collect::<BTreeSet<_>>().len() == member.moves.len(),
                    "unexpected party member for {id}"
                );
                let profile = pack
                    .data
                    .pokemon
                    .get(&member.species)
                    .context("city partner")?;
                let learnset = pack
                    .data
                    .learnsets
                    .get(&member.species)
                    .context("city partner learnset")?;
                let mut moves = Vec::new();
                for name in member.moves {
                    ensure!(
                        profile.tmhm_learnset.contains(&name)
                            || learnset
                                .iter()
                                .any(|entry| entry.0 <= slot.level && entry.1 == name),
                        "{id}: {} cannot learn {name}",
                        member.species
                    );
                    let pp = pack.data.moves.get(&name).context("city battle move")?.pp;
                    moves.push(LearnedMove {
                        name,
                        current_pp: pp,
                        pp_ups: 0,
                    });
                }
                slot.species = member.species;
                slot.moves = moves;
            }
        }
    }
    Ok(())
}
