use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::{
    battle::damage::MoveCategory,
    models::{LearnedMove, Move, Pokemon},
    state::GameState,
};
use serde::{Deserialize, Serialize};

use crate::{
    CompiledGamePack, GameDataSet, derive_compiled_game_pack_identity_from_manifest,
    verify_compiled_game_pack_for_runtime,
};

pub const WILD_CHALLENGE_MANIFEST_ID: &str = "wild-challenge";

/// Encounter rules composed into standard game packs. Legacy source packs can
/// omit them and retain their original encounters and identities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WildChallengeRules {
    pub party_members: u8,
    pub level_offset: i8,
    pub level_spread: u8,
}

impl Default for WildChallengeRules {
    fn default() -> Self {
        Self {
            party_members: 3,
            level_offset: -1,
            level_spread: 2,
        }
    }
}

impl WildChallengeRules {
    pub(crate) fn validate(self) -> Result<()> {
        ensure!(
            (1..=6).contains(&self.party_members),
            "wild challenge party_members must be 1..=6"
        );
        ensure!(
            (-10..=10).contains(&self.level_offset),
            "wild challenge level_offset must be -10..=10"
        );
        ensure!(
            self.level_spread <= 5,
            "wild challenge level_spread must be 0..=5"
        );
        Ok(())
    }

    pub fn encounter_level(self, party: &[Pokemon], authored_level: u8, variation: u8) -> u8 {
        // Fainting or moving a weak Pokémon to the lead cannot lower the anchor.
        let mut levels: Vec<_> = party
            .iter()
            .filter(|p| !p.is_egg)
            .map(|p| i16::from(p.level))
            .collect();
        levels.sort_unstable_by(|a, b| b.cmp(a));
        levels.truncate(usize::from(self.party_members.max(1)));
        if levels.is_empty() {
            return authored_level;
        }
        let count = levels.len() as i16;
        let anchor = (levels.iter().sum::<i16>() + count / 2) / count;
        let spread = i16::from(self.level_spread);
        let delta = i16::from(variation) % (spread * 2 + 1) - spread;
        (anchor + i16::from(self.level_offset) + delta)
            .clamp(1, 100)
            .max(i16::from(authored_level)) as u8
    }
}

pub fn build_wild_challenge_modpack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    verify_compiled_game_pack_for_runtime(base)?;
    if base.data.wild_challenge.is_some() {
        ensure!(
            base.report
                .manifests
                .iter()
                .any(|id| id == WILD_CHALLENGE_MANIFEST_ID),
            "wild challenge rules require their manifest identity"
        );
        return Ok(base.clone());
    }
    let mut pack = base.clone();
    pack.data.wild_challenge = Some(WildChallengeRules::default());
    pack.report
        .manifests
        .push(WILD_CHALLENGE_MANIFEST_ID.to_string());
    pack.identity = derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

impl GameDataSet {
    /// Live controls and mutation replay share this decision boundary. The
    /// catching tutorial and Bug-Catching Contest retain their original AI.
    pub fn select_wild_enemy_move_for_battle(
        &self,
        combat: &crystal_core::battle::turn::BattleCombatState,
        battle_type: &str,
        rng: &mut dyn crystal_core::random::BattleRandomSource,
    ) -> Result<crystal_core::battle::turn::EnemyMoveSelection> {
        if self.wild_challenge.is_some()
            && !matches!(battle_type, "BATTLETYPE_TUTORIAL" | "BATTLETYPE_CONTEST")
        {
            self.select_challenge_wild_move(combat, rng)
        } else {
            Ok(crystal_core::battle::turn::EnemyMoveSelection {
                slot: crystal_core::battle::turn::select_wild_enemy_move_slot(combat, rng),
                ai_damage_register: 0,
            })
        }
    }

    pub fn select_challenge_wild_move(
        &self,
        combat: &crystal_core::battle::turn::BattleCombatState,
        rng: &mut dyn crystal_core::random::BattleRandomSource,
    ) -> Result<crystal_core::battle::turn::EnemyMoveSelection> {
        // Competitive packs use the full planner. Standalone wild overlays
        // retain Basic, Setup, Types and Smart scoring. Neither grants wild
        // Pokémon trainer items or switching.
        self.select_trainer_enemy_move_with_scratch(
            combat,
            (1 << 0) | (1 << 1) | (1 << 2) | (1 << 4),
            rng,
        )
    }

    pub(crate) fn challenge_level(&self, state: &GameState, level: u8, variation: u8) -> u8 {
        self.wild_challenge.map_or(level, |rules| {
            rules.encounter_level(
                &state
                    .storage
                    .party
                    .pokemon
                    .iter()
                    .flatten()
                    .cloned()
                    .collect::<Vec<_>>(),
                level,
                variation,
            )
        })
    }

    /// Accept only levels at or above an actual authored origin, never new species.
    pub(crate) fn challenge_origin_level(&self, authored: u8, actual: u8) -> bool {
        authored == actual || (self.wild_challenge.is_some() && actual >= authored && actual <= 100)
    }

    pub fn competitive_wild_moves(&self, pokemon: &Pokemon) -> Result<Vec<LearnedMove>> {
        ensure!(
            self.learnsets.contains_key(&pokemon.species.id),
            "wild species missing learnset"
        );
        let mut lineage = BTreeSet::from([pokemon.species.id.clone()]);
        loop {
            let parents: Vec<_> = self
                .evolutions
                .0
                .iter()
                .filter(|(_, entries)| entries.iter().any(|entry| lineage.contains(&entry.species)))
                .map(|(id, _)| id.clone())
                .collect();
            let before = lineage.len();
            lineage.extend(parents);
            if lineage.len() == before {
                break;
            }
        }
        let cap = match pokemon.level {
            0..=9 => 50,
            10..=19 => 65,
            20..=29 => 85,
            _ => 150,
        };
        let mut legal = BTreeSet::new();
        for ancestor in &lineage {
            for entry in self
                .learnsets
                .get(ancestor)
                .into_iter()
                .flatten()
                .filter(|entry| entry.0 <= pokemon.level)
            {
                legal.insert(entry.1.clone());
            }
        }
        let mut bonus = BTreeSet::new();
        for ancestor in &lineage {
            if let Some(species) = self.pokemon.get(ancestor) {
                bonus.extend(species.tmhm_learnset.iter().cloned());
            }
        }
        for id in &bonus {
            if self.moves.get(id).is_some_and(|m| {
                m.power <= cap
                    && ((m.power > 0 && self.wild_attack_score(pokemon, id, m) > 0)
                        || support_score(id, pokemon) > 0
                        || (pokemon.level >= 30 && matches!(id.as_str(), "REST" | "SLEEP_TALK")))
            }) {
                legal.insert(id.clone());
            }
        }
        // Multi-hit attacks can overwhelm early teams despite low listed power.
        // Apply this to the full pool, including naturally learned moves.
        if pokemon.level < 10 {
            legal.retain(|id| {
                !matches!(
                    id.as_str(),
                    "DOUBLESLAP"
                        | "COMET_PUNCH"
                        | "FURY_ATTACK"
                        | "PIN_MISSILE"
                        | "BARRAGE"
                        | "FURY_SWIPES"
                        | "BONE_RUSH"
                        | "DOUBLE_KICK"
                        | "BONEMERANG"
                        | "TWINEEDLE"
                        | "TRIPLE_KICK"
                )
            });
        }
        let mut chosen: Vec<String> = Vec::new();
        // Reliable STAB, coverage, a useful support move, then complementary coverage.
        for role in 0..4 {
            let best = legal
                .iter()
                .filter(|id| !chosen.contains(id))
                .filter_map(|id| {
                    let m = self.moves.get(id)?;
                    let stab = m.move_type == pokemon.species.type1
                        || m.move_type == pokemon.species.type2;
                    let damaging = m.power > 0;
                    if role == 0 && (!damaging || !stab) {
                        return None;
                    }
                    if role == 1 && !damaging {
                        return None;
                    }
                    let support = support_score(id, pokemon);
                    if role == 2 && support == 0 {
                        return None;
                    }
                    let mut score = if damaging {
                        self.wild_attack_score(pokemon, id, m)
                    } else {
                        support
                    };
                    if damaging
                        && chosen.iter().any(|prior| {
                            self.moves[prior].move_type == m.move_type
                                && self.moves[prior].power > 0
                        })
                    {
                        score /= 4;
                    }
                    if !damaging && chosen.iter().any(|prior| self.moves[prior].power == 0) {
                        score /= 3;
                    }
                    (score > 0).then_some((score, id))
                })
                .max_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(a.1)));
            if let Some((_, id)) = best {
                chosen.push(id.clone());
            }
        }
        // Species with restricted movepools (e.g. Ditto, Unown, Magikarp) keep
        // legal moves rather than acquiring fabricated coverage.
        for id in &legal {
            if chosen.len() == 4 {
                break;
            }
            if !chosen.contains(id) {
                chosen.push(id.clone());
            }
        }
        let has_recovery = chosen.iter().any(|id| {
            matches!(
                id.as_str(),
                "RECOVER" | "SOFTBOILED" | "MILK_DRINK" | "SYNTHESIS" | "MOONLIGHT" | "MORNING_SUN"
            )
        });
        if pokemon.level >= 30
            && !has_recovery
            && legal.contains("REST")
            && legal.contains("SLEEP_TALK")
            && pokemon.species.base_stats.hp
                + pokemon.species.base_stats.defense
                + pokemon.species.base_stats.special_defense
                >= 250
        {
            chosen.retain(|id| self.moves[id].power > 0);
            chosen.truncate(2);
            chosen.extend(["REST".to_string(), "SLEEP_TALK".to_string()]);
        }
        chosen
            .into_iter()
            .map(|id| {
                let m = self
                    .moves
                    .get(&id)
                    .with_context(|| format!("unknown legal wild move {id}"))?;
                Ok(LearnedMove {
                    name: id,
                    current_pp: m.pp,
                    pp_ups: 0,
                })
            })
            .collect()
    }

    fn wild_attack_score(&self, pokemon: &Pokemon, id: &str, m: &Move) -> i32 {
        // Avoid attacks that need a partner move or sacrifice the encounter.
        if matches!(
            id,
            "SELFDESTRUCT"
                | "EXPLOSION"
                | "HYPER_BEAM"
                | "SOLARBEAM"
                | "DREAM_EATER"
                | "SNORE"
                | "FUTURE_SIGHT"
                | "ROLLOUT"
                | "BIDE"
                | "SKULL_BASH"
                | "RAZOR_WIND"
                | "RAGE"
                | "FLAIL"
                | "REVERSAL"
                | "PRESENT"
                | "FRUSTRATION"
                | "RETURN"
                | "FISSURE"
                | "GUILLOTINE"
                | "HORN_DRILL"
        ) {
            return 0;
        }
        let special = self.type_categories.moves.get(id).map_or_else(
            || self.type_categories.special.contains(&m.move_type),
            |category| *category == MoveCategory::Special,
        );
        let stat = if special {
            pokemon.species.base_stats.special_attack
        } else {
            pokemon.species.base_stats.attack
        };
        let stab = if m.move_type == pokemon.species.type1 || m.move_type == pokemon.species.type2 {
            150
        } else {
            100
        };
        let mut power = i32::from(m.power);
        if matches!(
            id,
            "DOUBLESLAP"
                | "COMET_PUNCH"
                | "FURY_ATTACK"
                | "PIN_MISSILE"
                | "BARRAGE"
                | "FURY_SWIPES"
                | "BONE_RUSH"
        ) {
            power *= 3;
        }
        if matches!(id, "DOUBLE_KICK" | "BONEMERANG" | "TWINEEDLE") {
            power *= 2;
        }
        if matches!(
            id,
            "SONICBOOM"
                | "DRAGON_RAGE"
                | "SEISMIC_TOSS"
                | "NIGHT_SHADE"
                | "PSYWAVE"
                | "SUPER_FANG"
                | "COUNTER"
                | "MIRROR_COAT"
        ) {
            return 0;
        }
        let mut score = power * i32::from(m.accuracy) * i32::from(stat) * stab / 100_000;
        if matches!(
            id,
            "DIG"
                | "FLY"
                | "DIVE"
                | "SKY_ATTACK"
                | "OUTRAGE"
                | "PETAL_DANCE"
                | "THRASH"
                | "DYNAMIC_PUNCH"
                | "ZAP_CANNON"
        ) {
            score /= 2;
        }
        if id == "HIDDEN_POWER" {
            score /= 2;
        }
        score
    }

    pub(crate) fn equip_wild_challenge_moves(&self, pokemon: &mut Pokemon) -> Result<()> {
        if self.wild_challenge.is_some() {
            pokemon.moves = self.competitive_wild_moves(pokemon)?;
        }
        Ok(())
    }

    pub(crate) fn scale_challenge_roamer(
        &self,
        state: &mut GameState,
        slot: usize,
        variation: u8,
    ) -> Result<u8> {
        let roaming = &state.roaming_pokemon[slot];
        let level = self.challenge_level(state, roaming.level, variation);
        let mut hp = roaming.current_hp();
        if level != roaming.level {
            let species = roaming
                .species
                .as_deref()
                .context("inactive challenge roamer")?;
            let dvs = crystal_core::models::Dv::from_non_hp(
                roaming.dvs_be[0] >> 4,
                roaming.dvs_be[0] & 15,
                roaming.dvs_be[1] >> 4,
                roaming.dvs_be[1] & 15,
            );
            let old = self.create_pokemon(species, roaming.level, dvs)?;
            let new = self.create_pokemon(species, level, dvs)?;
            hp = if roaming.hp == 0 {
                0
            } else {
                (u32::from(roaming.current_hp()) * u32::from(new.max_hp) / u32::from(old.max_hp))
                    .clamp(1, u32::from(new.max_hp)) as u16
            };
            state.roaming_pokemon[slot].level = level;
        }
        state.roaming_pokemon[slot].scaled_hp = Some(hp);
        state.roaming_pokemon[slot].set_current_hp(hp);
        Ok(level)
    }
}

fn support_score(id: &str, pokemon: &Pokemon) -> i32 {
    if pokemon.level < 10 {
        return 0;
    }
    match id {
        "RECOVER" | "SOFTBOILED" | "MILK_DRINK" | "SYNTHESIS" | "MOONLIGHT" | "MORNING_SUN" => 180,
        "SPORE" | "SLEEP_POWDER" | "THUNDER_WAVE" => 170,
        "HYPNOSIS" | "LOVELY_KISS" | "STUN_SPORE" => 140,
        "TOXIC" if pokemon.level >= 20 => 130,
        "SWORDS_DANCE"
            if pokemon.level >= 20
                && pokemon.species.base_stats.attack
                    >= pokemon.species.base_stats.special_attack =>
        {
            150
        }
        "CURSE"
            if pokemon.level >= 20
                && pokemon.species.base_stats.attack
                    >= pokemon.species.base_stats.special_attack
                && pokemon.species.base_stats.speed <= 65 =>
        {
            120
        }
        "GROWTH"
            if pokemon.level >= 20
                && pokemon.species.base_stats.special_attack
                    >= pokemon.species.base_stats.attack =>
        {
            120
        }
        "ENCORE" | "LEECH_SEED" | "REFLECT" | "LIGHT_SCREEN" => 110,
        "CONFUSE_RAY" | "GLARE" => 100,
        _ => 0,
    }
}
