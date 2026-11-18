//! Three short, readable battles on the prairie road, using existing native parties.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use crystal_core::models::{LearnedMove, TrainerPartyPokemon};
use serde::Deserialize;

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::northstar_regional_roster::RegionalRoster;
use crate::{CompiledGamePack, MapModule};

const SOURCE: &str =
    include_str!("../../../planning/minnesota/revisions/route-challenges-v01/trainers.json");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Challenge {
    id: String,
    map: String,
    name: String,
    expected_party: Vec<TrainerPartyPokemon>,
    party: Vec<Member>,
    tile: [u16; 2],
    facing: String,
    range: u16,
    dialogue: BTreeMap<String, Vec<AuthoredDialoguePage>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Member {
    source_slot: usize,
    moves: Vec<String>,
}

/// Installs three ordinary route battles without changing levels, AI or event flags.
///
/// # Errors
/// Rejects changed parent teams, illegal moves, inactive species, inaccessible
/// sightlines, missing dialogue branches or an invalid scaled regional team.
pub fn install_northstar_route_challenges(pack: &mut CompiledGamePack) -> Result<()> {
    let designs: Vec<Challenge> = serde_json::from_str(SOURCE)?;
    let roster = RegionalRoster::current()?;
    for design in designs {
        let previous = pack.data.maps.get(&design.map).context("prairie road")?;
        check_lane(pack, previous, &design)?;
        let mut room = previous.clone();
        let mut trainer = pack
            .data
            .trainers
            .trainers
            .get(&design.id)
            .context("road trainer")?
            .clone();
        ensure!(
            trainer.name == design.name && trainer.party == design.expected_party,
            "{} parent party changed",
            design.id
        );
        let mut party = Vec::new();
        for member in &design.party {
            let mut slot = trainer
                .party
                .get(member.source_slot)
                .context("original party slot")?
                .clone();
            ensure!(
                roster.allowed.contains(&slot.species),
                "inactive regional species"
            );
            let profile = &pack.data.pokemon[&slot.species];
            let learnset = &pack.data.learnsets[&slot.species];
            ensure!(
                (1..=4).contains(&member.moves.len())
                    && member.moves.iter().collect::<BTreeSet<_>>().len() == member.moves.len(),
                "unique native move slots required"
            );
            slot.moves.clear();
            for name in &member.moves {
                ensure!(
                    profile.tmhm_learnset.contains(name)
                        || learnset.iter().any(|m| m.0 <= slot.level && &m.1 == name),
                    "illegal move {}/{name}",
                    slot.species
                );
                let pp = pack.data.moves.get(name).context("native move")?.pp;
                slot.moves.push(LearnedMove {
                    name: name.clone(),
                    current_pp: pp,
                    pp_ups: 0,
                });
            }
            party.push(slot);
        }
        trainer.party = party;
        // The existing engine learns later moves and evolves at the original
        // thresholds. Check all possible ace levels, without simulating a battle.
        let ace = trainer
            .party
            .iter()
            .map(|p| p.level)
            .max()
            .context("nonempty team")?;
        let first = pack.data.rematch_trainer_at_ace(&trainer, ace)?;
        ensure!(
            first.party == trainer.party,
            "first battle must retain authored roles"
        );
        for level in ace..=100 {
            let scaled = pack.data.rematch_trainer_at_ace(&trainer, level)?;
            ensure!(
                scaled
                    .party
                    .iter()
                    .all(|m| roster.allowed.contains(&m.species)),
                "inactive scaled team at {level}"
            );
        }
        for (label, pages) in design.dialogue {
            ensure!(
                room.script_text_bodies.contains_key(&label),
                "existing trainer speech {label}"
            );
            room.scripts
                .insert(label, authored_dialogue::compile(&pages)?);
        }
        crate::ely_league_rooms::finish(pack, &mut room)?;
        room.script_audio_commands = crate::parse_script_audio_commands(&room.id, &room.scripts)?;
        room.map_event_section_commands
            .clone_from(&previous.map_event_section_commands);
        ensure!(
            room.trainer_scripts == previous.trainer_scripts && room.objects == previous.objects,
            "preserve battle flags, callbacks and sightline actors"
        );
        pack.data.trainers.trainers.insert(design.id, trainer);
        crate::authored_town_publication::replace(pack, room)?;
    }
    Ok(())
}

fn check_lane(pack: &CompiledGamePack, room: &MapModule, design: &Challenge) -> Result<()> {
    let actor = room
        .objects
        .iter()
        .find(|a| a.script == design.id)
        .context("trainer actor")?;
    ensure!(
        [actor.x, actor.y] == design.tile
            && actor.radius == design.range
            && actor.sightline_direction_override.as_deref() == Some(&design.facing)
            && actor.spritemovedata == format!("SPRITEMOVEDATA_STANDING_{}", design.facing)
            && actor.object_type == "OBJECTTYPE_TRAINER",
        "expected standing trainer lane"
    );
    let (dx, dy) = match design.facing.as_str() {
        "UP" => (0, -1),
        "DOWN" => (0, 1),
        "LEFT" => (-1, 0),
        "RIGHT" => (1, 0),
        _ => anyhow::bail!("invalid direction"),
    };
    let mut floor = BTreeSet::new();
    let mut queue = VecDeque::new();
    for y in 0..room.attributes.height * 2 {
        for x in 0..room.attributes.width * 2 {
            if matches!(
                crate::authored_town_geometry::kind(pack, room, [x, y])?,
                "FLOOR" | "TALL_GRASS"
            ) && !room.objects.iter().any(|a| a.x == x && a.y == y)
            {
                floor.insert([x, y]);
                if room
                    .attributes
                    .connections
                    .iter()
                    .any(|c| match c.direction.as_str() {
                        "north" => y == 0,
                        "south" => y + 1 == room.attributes.height * 2,
                        "west" => x == 0,
                        "east" => x + 1 == room.attributes.width * 2,
                        _ => false,
                    })
                {
                    queue.push_back([x, y]);
                }
            }
        }
    }
    let mut reached = BTreeSet::new();
    while let Some([x, y]) = queue.pop_front() {
        if !floor.contains(&[x, y]) || !reached.insert([x, y]) {
            continue;
        }
        for (a, b) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            if let Some(tile) = x.checked_add_signed(a).zip(y.checked_add_signed(b)) {
                queue.push_back([tile.0, tile.1]);
            }
        }
    }
    for distance in 1..=design.range {
        let x = actor
            .x
            .checked_add_signed(dx * i16::try_from(distance)?)
            .context("lane x")?;
        let y = actor
            .y
            .checked_add_signed(dy * i16::try_from(distance)?)
            .context("lane y")?;
        ensure!(
            reached.contains(&[x, y]),
            "{} has an inaccessible sight tile",
            design.id
        );
    }
    Ok(())
}

#[cfg(test)]
fn rollout_damage(pack: &CompiledGamePack) {
    use crystal_core::battle::damage::{DamageContext, calculate_damage};
    let materialize = |id: &str| {
        crate::materialize_trainer_party(
            &pack.data.trainers.trainers[id],
            &pack.data.pokemon,
            &pack.data.learnsets,
            &pack.data.moves,
            &pack.data.growth_rates,
        )
        .expect("static party materialization")
    };
    let june = materialize("MN_R16_T04");
    let lyle = materialize("MN_R16_T01");
    for (name, defender) in [("FLOSSAIL", &lyle[0]), ("CACHEEK", &lyle[1])] {
        let mut ranges = Vec::new();
        for multiplier in [1, 2, 4] {
            let mut hits = Vec::new();
            for roll in [217, 255] {
                hits.push(
                    calculate_damage(
                        &june[1],
                        defender,
                        &pack.data.moves["ROLLOUT"],
                        &pack.data.battle_stat_multipliers,
                        &pack.data.type_categories,
                        &pack.data.type_effectiveness,
                        &pack.data.weather_modifiers,
                        DamageContext {
                            random_roll: roll,
                            post_type_damage_multiplier: multiplier,
                            ..DamageContext::default()
                        },
                    )
                    .expect("static native damage")
                    .damage,
                );
            }
            ranges.push(hits);
        }
        println!("TOTLIT31 ROLLOUT into {name}31: plain/curl/curl-second {ranges:?}");
    }
}

#[cfg(test)]
mod tests {
    use super::{Challenge, SOURCE, install_northstar_route_challenges, rollout_damage};
    use crate::authored_dialogue;

    #[test]
    fn route_challenge_pages_fit_native_text() {
        let designs: Vec<Challenge> = serde_json::from_str(SOURCE).expect("challenge source");
        for design in designs {
            assert_eq!(design.party.len(), 2);
            assert_eq!(design.dialogue.len(), 4);
            for pages in design.dialogue.values() {
                authored_dialogue::compile(pages).expect("native pages");
            }
        }
    }

    #[test]
    #[ignore = "requires the external selected Crossings pack; static checks only"]
    fn route_challenges_preserve_world_and_check_scaled_teams() {
        let path = std::env::var_os("CRYSTAL_ROUTE_CHALLENGE_BASE").expect("external parent pack");
        let base = crate::read_verified_compiled_game_pack(path).expect("verified parent");
        let mut candidate = base.clone();
        install_northstar_route_challenges(&mut candidate).expect("install legal accessible teams");
        println!(
            "rules: scaling={}, general_rematches={}",
            base.data.minnesota_trainer_scaling, base.data.trainer_rematches
        );
        rollout_damage(&candidate);
        assert_eq!(candidate.runtime_files, base.runtime_files);
        assert_eq!(candidate.audio_manifest, base.audio_manifest);
        let changed_trainers: Vec<_> = candidate
            .data
            .trainers
            .trainers
            .iter()
            .filter(|(id, trainer)| base.data.trainers.trainers.get(*id) != Some(*trainer))
            .map(|(id, _)| id.as_str())
            .collect();
        assert_eq!(changed_trainers, ["MN_R16_T01", "MN_R16_T04", "MN_R16_T06"]);
        for (id, map) in &candidate.data.maps {
            let old = &base.data.maps[id];
            assert_eq!(map.blocks, old.blocks);
            assert_eq!(map.events, old.events);
            assert_eq!(map.objects, old.objects);
            assert_eq!(map.trainer_scripts, old.trainer_scripts);
            if id != "MnR16C00R00" && id != "MnR16C01R00" {
                assert_eq!(map, old);
            }
        }
        // Compare the complete data set after restoring precisely the allowed
        // publication fields. This catches accidental changes outside the teams.
        let mut preserved = candidate.data.clone();
        preserved.trainers = base.data.trainers.clone();
        preserved.maps = base.data.maps.clone();
        preserved.map_scripts = base.data.map_scripts.clone();
        assert_eq!(preserved, base.data);
        candidate.identity = crate::derive_compiled_game_pack_identity_from_manifest(
            candidate.format_version,
            &candidate.data,
            &candidate.audio_manifest,
            &candidate.runtime_files,
            &candidate.report,
        )
        .expect("candidate identity");
        crate::verify_compiled_game_pack_for_runtime(&candidate).expect("runtime-valid candidate");
        if let Some(output) = std::env::var_os("CRYSTAL_ROUTE_CHALLENGE_OUTPUT") {
            assert!(!std::path::Path::new(&output).exists(), "fresh output only");
            candidate
                .write_preserving_storage(&output)
                .expect("write candidate");
            assert_eq!(
                crate::read_verified_compiled_game_pack(output).expect("readback"),
                candidate
            );
        }
    }
}
