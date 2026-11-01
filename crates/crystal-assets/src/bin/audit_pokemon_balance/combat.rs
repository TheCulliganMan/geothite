//! Data-only production-engine checks. Never exports art or publishes a pack.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use crystal_assets::GameDataSet;
use crystal_core::battle::turn::{BattleAction, BattleCombatState, BattleTurnInput};
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::random::BattleRandomSource;
use crystal_core::systems::evolution::{
    EvolutionContext, EvolutionEntry, LinkMode, check_and_evolve,
};
use crystal_core::systems::learnsets::LearnsetEntry;
use crystal_core::world::encounters::TimeOfDay;

use super::{Extensions, Member, Profile, Roster, attack_ratio, power, reliable, type_token};

pub(super) struct CombatReview {
    data: GameDataSet,
    pub(super) battles: usize,
    pub(super) evolutions: usize,
    pub(super) default_sets: usize,
}

struct Random(u32);

impl BattleRandomSource for Random {
    fn battle_random_byte(&mut self) -> u8 {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.0 >> 24) as u8
    }
}

impl CombatReview {
    pub(super) fn verify_published(&self, reference: &GameDataSet, member: &Member) -> Result<()> {
        let name = member.name.to_ascii_uppercase();
        let actual = reference
            .pokemon
            .get(&name)
            .with_context(|| format!("published pack lacks {name}"))?;
        let expected = &self.data.pokemon[&name];
        ensure!(
            actual.int_id == expected.int_id
                && actual.base_stats == expected.base_stats
                && actual.type1 == expected.type1
                && actual.type2 == expected.type2
                && actual.growth_rate == expected.growth_rate
                && actual.tmhm_learnset == expected.tmhm_learnset,
            "published stats/types/growth/TMs are stale for {name}"
        );
        ensure!(
            reference.learnsets.get(&name) == self.data.learnsets.get(&name),
            "published level moves are stale for {name}"
        );
        ensure!(
            reference.evolutions.0.get(&name) == self.data.evolutions.0.get(&name),
            "published evolution is stale for {name}"
        );
        Ok(())
    }

    pub(super) fn new(
        reference: &GameDataSet,
        roster: &Roster,
        profiles: &BTreeMap<String, Profile>,
        extensions: &Extensions,
    ) -> Result<Self> {
        let mut data = reference.clone();
        let names: BTreeMap<_, _> = roster
            .families
            .iter()
            .flat_map(|f| &f.members)
            .map(|m| (m.id.as_str(), m.name.to_ascii_uppercase()))
            .collect();
        // The donor supplies unrelated structural fields only. Every field used
        // by these stat, level-up, evolution and damage checks is set explicitly.
        // This lets balance checks run while an unrelated sprite awaits review.
        let donor = reference
            .pokemon
            .get("MEW")
            .context("native MEW reference")?;
        for family in &roster.families {
            for member in &family.members {
                let profile = &profiles[&member.id];
                let name = &names[member.id.as_str()];
                ensure!(
                    reference.pokemon.get(name).is_none_or(|p| p.int_id > 251),
                    "regional name conflicts with returning Pokémon: {name}"
                );
                let mut species = donor.clone();
                species.id = name.clone();
                species.int_id = member
                    .id
                    .strip_prefix("MN_")
                    .context("regional ID prefix")?
                    .parse::<u16>()?
                    .checked_add(251)
                    .context("regional number overflow")?;
                species.base_stats = profile.base_stats;
                species.type1 = type_token(&member.types[0]).to_string();
                species.type2 =
                    type_token(member.types.get(1).unwrap_or(&member.types[0])).to_string();
                species.growth_rate = profile.growth_rate.clone();
                species.ability = "NONE".to_string();
                species.item1 = None;
                species.item2 = None;
                species.tmhm_learnset = profile.tmhm_moves.clone();
                data.pokemon.insert(name.clone(), species);
                let mut learnset: Vec<_> = profile
                    .level_moves
                    .iter()
                    .map(|(l, m)| LearnsetEntry(*l, m.clone()))
                    .collect();
                for extension in &extensions.moves {
                    for learner in extension
                        .learners
                        .iter()
                        .filter(|l| l.species_id == member.id)
                    {
                        learnset.push(LearnsetEntry(learner.level, extension.id.clone()));
                    }
                }
                learnset.sort_by_key(|entry| entry.0);
                data.learnsets.insert(name.clone(), learnset);
                let evolution = match (&profile.evolution, &member.evolves_to) {
                    (Some(evolution), Some(target)) => vec![EvolutionEntry {
                        method: "LEVEL".to_string(),
                        species: names[target.as_str()].clone(),
                        level: Some(evolution.level),
                        item: None,
                        held_item: None,
                        happiness: None,
                        stat_ratio: None,
                    }],
                    (None, None) => Vec::new(),
                    _ => anyhow::bail!("{} evolution closure differs from profile", member.id),
                };
                data.evolutions.0.insert(name.clone(), evolution);
            }
        }
        Ok(Self {
            data,
            battles: 0,
            evolutions: 0,
            default_sets: 0,
        })
    }

    pub(super) fn check_defaults(&mut self, member: &Member) -> Result<Vec<String>> {
        let name = member.name.to_ascii_uppercase();
        let mut gaps = Vec::new();
        for level in 1..=100 {
            let pokemon =
                self.data
                    .create_pokemon(&name, level, Dv::from_non_hp(10, 10, 10, 10))?;
            ensure!(
                (1..=4).contains(&pokemon.moves.len()) && pokemon.hp == pokemon.max_hp,
                "invalid default party member {name} at {level}"
            );
            if !pokemon.moves.iter().any(|m| {
                let mov = &self.data.moves[&m.name];
                reliable(mov)
                    || ["NIGHT_SHADE", "MAGNITUDE", "DRAGON_RAGE"].contains(&mov.name.as_str())
            }) {
                gaps.push(format!("default moves lack a repeatable attack at {level}"));
                break;
            }
            self.default_sets += 1;
        }
        Ok(gaps)
    }

    pub(super) fn check_battles(
        &mut self,
        member: &Member,
        profile: &Profile,
        earned: &BTreeMap<String, u8>,
        arrival: u8,
    ) -> Result<()> {
        let name = member.name.to_ascii_uppercase();
        for level in BTreeSet::from([arrival.max(5), arrival.max(35), arrival.max(53)]) {
            let mut candidates: Vec<_> = earned
                .iter()
                .filter(|(name, at)| **at <= level && reliable(&self.data.moves[*name]))
                .collect();
            candidates.sort_by(|(left, _), (right, _)| {
                let score = |id: &String| {
                    let mov = &self.data.moves[id];
                    let stab = if member
                        .types
                        .iter()
                        .any(|kind| type_token(kind) == mov.move_type)
                    {
                        1.5
                    } else {
                        1.0
                    };
                    f64::from(power(mov))
                        * attack_ratio(&self.data, &profile.base_stats, mov)
                        * stab
                };
                score(right)
                    .total_cmp(&score(left))
                    .then_with(|| left.cmp(right))
            });
            ensure!(
                !candidates.is_empty(),
                "{name} lacks an ordinary attack at {level}"
            );
            let mut player =
                self.data
                    .create_pokemon(&name, level, Dv::from_non_hp(10, 10, 10, 10))?;
            player.moves = candidates
                .iter()
                .take(4)
                .map(|(id, _)| LearnedMove {
                    name: (*id).clone(),
                    current_pp: self.data.moves[*id].pp,
                    pp_ups: 0,
                })
                .collect();
            let mut enemy =
                self.data
                    .create_pokemon("MEW", 100, Dv::from_non_hp(10, 10, 10, 10))?;
            enemy.moves = vec![LearnedMove {
                name: "SPLASH".to_string(),
                current_pp: self.data.moves["SPLASH"].pp,
                pp_ups: 0,
            }];
            let mut combat = BattleCombatState::new(player, enemy);
            let initial_hp = combat.enemy.hp;
            let initial_pp = combat.player.moves[0].current_pp;
            let mut random = Random(47);
            for _ in 0..8 {
                combat = self
                    .data
                    .resolve_battle_turn_with_items(
                        combat,
                        BattleTurnInput {
                            player: BattleAction::Move { slot: 0 },
                            enemy: BattleAction::Move { slot: 0 },
                        },
                        &mut random,
                    )?
                    .state;
                if combat.enemy.hp < initial_hp {
                    break;
                }
            }
            ensure!(
                combat.enemy.hp < initial_hp && combat.player.moves[0].current_pp < initial_pp,
                "{name} at {level} must spend PP and deal real damage"
            );
            self.battles += 1;
        }
        Ok(())
    }

    pub(super) fn check_evolution(&mut self, member: &Member, profile: &Profile) -> Result<()> {
        let Some(evolution) = &profile.evolution else {
            return Ok(());
        };
        let name = member.name.to_ascii_uppercase();
        let context = EvolutionContext {
            species: &self.data.pokemon,
            moves: &self.data.moves,
            learnsets: &self.data.learnsets,
            time_of_day: TimeOfDay::Day,
            current_item: None,
            force_evolution: false,
            link_mode: LinkMode::None,
        };
        for (level, should_evolve) in [(evolution.level - 1, false), (evolution.level, true)] {
            let mut pokemon = self.data.create_pokemon(&name, level, Dv::default())?;
            let report = check_and_evolve(&mut pokemon, &self.data.evolutions, &context, false)?;
            let expected = self.data.evolutions.0[&name]
                .first()
                .context("validated level evolution entry")?
                .species
                .as_str();
            ensure!(
                report.target_species.as_deref()
                    == if should_evolve { Some(expected) } else { None },
                "{name} evolves at the wrong level"
            );
        }
        self.evolutions += 1;
        Ok(())
    }
}
