//! Competitive singles decisions use the production engine, on private clones.
//! Search randomness is independent of the live battle's exact replay stream.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::{
    battle::{
        damage::MoveCategory,
        turn::{
            BattleAction, BattleCombatState, BattleSide, BattleTurnInput, EnemyMoveSelection,
            battle_action_locked_before_menu, battle_moves, select_wild_enemy_move_slot,
        },
    },
    models::{LearnedMove, Pokemon, Stat},
    random::BattleRandomSource,
};

use crate::{
    CompiledGamePack, GameDataSet, TrainerBattleProfile, build_modern_move_split_modpack,
    derive_compiled_game_pack_identity_from_manifest, verify_compiled_game_pack_for_runtime,
};

pub const COMPETITIVE_BATTLES_MANIFEST_ID: &str = "competitive-battles";
const INVALID_SCORE: i32 = -1_000_000;
const SAMPLES: u32 = 8;

/// Compose with the exhaustive move split; stock data and audio stay intact.
/// Idempotent so the hosted default can consume an already modded local pack.
pub fn build_competitive_battle_modpack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    verify_compiled_game_pack_for_runtime(base)?;
    if base.data.competitive_battles {
        ensure!(
            base.report
                .manifests
                .iter()
                .any(|id| id == COMPETITIVE_BATTLES_MANIFEST_ID),
            "competitive rules require their manifest identity"
        );
        ensure!(
            base.data.type_categories.moves.len() == base.data.moves.len(),
            "competitive rules require an exhaustive move split"
        );
        return Ok(base.clone());
    }
    let mut pack = if base.data.type_categories.moves.is_empty() {
        build_modern_move_split_modpack(base)?
    } else {
        base.clone()
    };
    ensure!(
        pack.data.type_categories.moves.len() == pack.data.moves.len(),
        "competitive rules require an exhaustive move split"
    );
    pack.data.competitive_battles = true;
    pack.report
        .manifests
        .push(COMPETITIVE_BATTLES_MANIFEST_ID.into());
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

/// A reproducible, full-period sampling stream, never persisted as game RNG.
struct PlanningRandom {
    state: u32,
    antithetic: bool,
}

impl PlanningRandom {
    fn sample(index: u32) -> Self {
        // Avalanche the pair seed: adjacent LCG seeds otherwise cluster their
        // first byte and can make every sampled speed tie favor the same side.
        let mut state = (index / 2 + 1).wrapping_mul(0x9e37_79b9);
        state = (state ^ (state >> 16)).wrapping_mul(0x85eb_ca6b);
        state = (state ^ (state >> 13)).wrapping_mul(0xc2b2_ae35);
        state ^= state >> 16;
        Self {
            state,
            antithetic: index & 1 != 0,
        }
    }
}

impl BattleRandomSource for PlanningRandom {
    fn battle_random_byte(&mut self) -> u8 {
        self.state = self
            .state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        let byte = (self.state >> 24) as u8;
        if self.antithetic { !byte } else { byte }
    }
}

impl GameDataSet {
    /// Production and mutation replay supply the same class. Callers without
    /// trainer identity retain the equally strong balanced policy.
    pub fn select_trainer_enemy_move_for_class(
        &self,
        combat: &BattleCombatState,
        trainer_class: &str,
        ai_flags: u32,
        rng: &mut dyn BattleRandomSource,
    ) -> Result<EnemyMoveSelection> {
        if self.competitive_battles && !combat.link_battle {
            self.select_competitive_enemy_move(
                combat,
                TrainerBattleProfile::for_class(trainer_class),
                rng,
            )
        } else {
            self.select_trainer_enemy_move_with_scratch(combat, ai_flags, rng)
        }
    }

    pub(crate) fn select_competitive_enemy_move(
        &self,
        combat: &BattleCombatState,
        profile: TrainerBattleProfile,
        rng: &mut dyn BattleRandomSource,
    ) -> Result<EnemyMoveSelection> {
        if battle_action_locked_before_menu(combat, BattleSide::Enemy) {
            return Ok(EnemyMoveSelection {
                slot: select_wild_enemy_move_slot(combat, rng),
                ai_damage_register: 0,
            });
        }
        let planning = self.competitive_planning_state(combat)?;
        let mut scored = Vec::new();
        let mut matrix = Vec::new();
        for slot in competitive_move_slots(&planning, BattleSide::Enemy) {
            let responses = self.competitive_action_values(
                &planning,
                &BattleAction::Move { slot },
                true,
                profile,
            )?;
            let score = competitive_response_value(&responses);
            #[cfg(not(target_arch = "wasm32"))]
            if std::env::var_os("CRYSTAL_BATTLE_AI_TRACE").is_some() {
                eprintln!(
                    "[battle-ai] {profile:?} {} {} score={score}",
                    planning.enemy.species.id,
                    battle_moves(&planning, BattleSide::Enemy)[slot].name
                );
            }
            let expected = battle_moves(&planning, BattleSide::Enemy)
                .get(slot)
                .and_then(|m| {
                    self.competitive_move_expected_damage(&planning, BattleSide::Enemy, m)
                })
                .unwrap_or(0)
                .min(u32::from(planning.player.hp) * 100);
            scored.push((slot, score, expected));
            matrix.push(responses);
        }
        // In simultaneous-choice battles a fixed move can be exploited by
        // Counter, Protect, setup, or Encore. Mix only when a maximin mixture
        // improves the guaranteed value beyond the sampling uncertainty band.
        if let Some(weights) = competitive_mixed_policy(&matrix) {
            #[cfg(not(target_arch = "wasm32"))]
            if std::env::var_os("CRYSTAL_BATTLE_AI_TRACE").is_some() {
                eprintln!("[battle-ai] {profile:?} simultaneous-choice weights={weights:?}");
            }
            let total = weights.iter().sum::<u32>();
            let mut draw = u32::from(rng.battle_random_byte()) * total / 256;
            for (entry, weight) in scored.iter().zip(weights) {
                if draw < weight {
                    return Ok(EnemyMoveSelection {
                        slot: entry.0,
                        ai_damage_register: 0,
                    });
                }
                draw -= weight;
            }
            unreachable!("mixed policy probabilities must sum to one");
        }
        let best_score = scored
            .iter()
            .map(|entry| entry.1)
            .max()
            .unwrap_or(INVALID_SCORE);
        ensure!(
            best_score > INVALID_SCORE,
            "competitive planner found no executable enemy move"
        );
        // Finite chance samples cannot distinguish tiny score differences.
        // Within half a sample's HP resolution, prefer reliable damage over
        // fishing for a rare status/critical outcome. Keep meaningful recovery
        // and setup advantages outside that uncertainty band.
        let tolerance = 1_000 / (2 * SAMPLES as i32);
        scored.retain(|entry| best_score - entry.1 <= tolerance);
        let reliable = scored.iter().map(|entry| entry.2).max().unwrap_or(0);
        scored.retain(|entry| entry.2 == reliable);
        let best_score = scored.iter().map(|entry| entry.1).max().unwrap();
        let best = scored
            .iter()
            .filter_map(|entry| (entry.1 == best_score).then_some(entry.0))
            .collect::<Vec<_>>();
        // Only genuine ties consume game RNG. Hypothetical turns never do.
        let index = if best.len() == 1 {
            0
        } else {
            usize::from(rng.battle_random_byte()) * best.len() / 256
        };
        Ok(EnemyMoveSelection {
            slot: best[index],
            ai_damage_register: 0,
        })
    }

    pub(crate) fn select_competitive_enemy_action(
        &self,
        combat: &BattleCombatState,
        selected_move_slot: usize,
        profile: TrainerBattleProfile,
        items: &[String],
    ) -> Result<BattleAction> {
        let selected = BattleAction::Move {
            slot: selected_move_slot,
        };
        if battle_action_locked_before_menu(combat, BattleSide::Enemy) {
            return Ok(selected);
        }
        let planning = self.competitive_planning_state(combat)?;
        let mut best_score = self.competitive_action_score(&planning, &selected, true, profile)?;
        let mut best = selected;
        for (party_index, pokemon) in planning.enemy_party.iter().enumerate() {
            if combat.enemy_escape_trap.is_some() || combat.enemy_trap.is_some() {
                break;
            }
            if party_index == planning.enemy_party_index || pokemon.hp == 0 || pokemon.is_egg {
                continue;
            }
            let action = BattleAction::TrainerSwitch {
                selected_move_slot,
                party_index,
            };
            let score = self.competitive_action_score(&planning, &action, true, profile)?;
            // A switch must repay its lost turn and a margin. This also prevents
            // oscillating between equally good resistances and pays entry hazards.
            let margin = if profile == TrainerBattleProfile::Switching {
                60
            } else {
                100
            };
            if score - margin > best_score {
                best_score = score - margin;
                best = action;
            }
        }
        for item_id in items {
            let action = BattleAction::TrainerItem {
                selected_move_slot,
                item_id: item_id.clone(),
            };
            // Preserve a finite consumable unless it makes a meaningful gain.
            // A small opportunity cost also breaks equal outcomes in favor of
            // a move or the earlier, cheaper owned healing item.
            let score = self.competitive_action_score(&planning, &action, true, profile)? - 80;
            if score > best_score {
                best_score = score;
                best = action;
            }
        }
        Ok(best)
    }

    pub(crate) fn select_competitive_trainer_action(
        &self,
        combat: &BattleCombatState,
        trainer_id: &str,
        battle_type: &str,
        used: &mut BTreeSet<String>,
        selected_move_slot: usize,
        profile: TrainerBattleProfile,
    ) -> Result<BattleAction> {
        let trainer = self.trainers.trainers.get(trainer_id);
        let mut items = Vec::new();
        if battle_type != "BATTLETYPE_BATTLE_TOWER" {
            let trainer =
                trainer.with_context(|| format!("unknown competitive trainer {trainer_id}"))?;
            // These are the trainer consumables supported by the production
            // engine. No authored probability gate or strongest-party gate:
            // compare every owned, unused option against moves and switches.
            for id in [
                "POTION",
                "SUPER_POTION",
                "HYPER_POTION",
                "MAX_POTION",
                "FULL_RESTORE",
                "FULL_HEAL",
                "X_ACCURACY",
                "GUARD_SPEC",
                "DIRE_HIT",
                "X_ATTACK",
                "X_DEFEND",
                "X_SPEED",
                "X_SPECIAL",
            ] {
                if trainer.items.iter().enumerate().any(|(slot, item)| {
                    item.as_deref() == Some(id)
                        && !used.contains(&format!("{trainer_id}:{id}:{slot}"))
                }) {
                    items.push(id.to_string());
                }
            }
        }
        let action =
            self.select_competitive_enemy_action(combat, selected_move_slot, profile, &items)?;
        if let BattleAction::TrainerItem { item_id, .. } = &action {
            let slot = trainer
                .unwrap()
                .items
                .iter()
                .enumerate()
                .find_map(|(slot, item)| {
                    (item.as_ref() == Some(item_id)
                        && !used.contains(&format!("{trainer_id}:{item_id}:{slot}")))
                    .then_some(slot)
                })
                .context("selected trainer item is not owned or has already been used")?;
            used.insert(format!("{trainer_id}:{item_id}:{slot}"));
        }
        Ok(action)
    }

    /// Explicit class context also covers generated Battle Tower opponents,
    /// which do not have an authored main-catalog trainer ID.
    pub fn select_trainer_post_order_action_for_class(
        &self,
        combat: &BattleCombatState,
        trainer_id: &str,
        trainer_class: &str,
        battle_type: &str,
        ai_item_switch_flags: u32,
        used: &mut BTreeSet<String>,
        selected_move_slot: usize,
        rng: &mut dyn BattleRandomSource,
    ) -> Result<BattleAction> {
        if self.competitive_battles && !combat.link_battle {
            if battle_action_locked_before_menu(combat, BattleSide::Enemy) {
                return Ok(BattleAction::Move {
                    slot: selected_move_slot,
                });
            }
            self.select_competitive_trainer_action(
                combat,
                trainer_id,
                battle_type,
                used,
                selected_move_slot,
                TrainerBattleProfile::for_class(trainer_class),
            )
        } else {
            self.select_trainer_post_order_action(
                combat,
                trainer_id,
                battle_type,
                ai_item_switch_flags,
                used,
                selected_move_slot,
                rng,
            )
        }
    }

    pub(crate) fn competitive_enemy_replacement(
        &self,
        combat: &BattleCombatState,
        profile: TrainerBattleProfile,
    ) -> Result<Option<usize>> {
        let planning = self.competitive_planning_state(combat)?;
        let mut best = None;
        for (index, pokemon) in planning.enemy_party.iter().enumerate() {
            if index == planning.enemy_party_index || pokemon.hp == 0 || pokemon.is_egg {
                continue;
            }
            let entered = crystal_core::battle::turn::preview_battle_replacement(
                &planning,
                BattleSide::Enemy,
                index,
            )
            .map_err(|error| anyhow::anyhow!("plan trainer replacement: {error:?}"))?;
            // Forced replacement does not give the player a free attack.
            let mut score = self.competitive_position_value(&entered);
            if entered.enemy.hp > 0 && entered.player.hp > 0 {
                score = INVALID_SCORE;
                for slot in competitive_move_slots(&entered, BattleSide::Enemy) {
                    score = score.max(self.competitive_action_score(
                        &entered,
                        &BattleAction::Move { slot },
                        true,
                        profile,
                    )?);
                }
            }
            if best.is_none_or(|(_, value)| score > value) {
                best = Some((index, score));
            }
        }
        Ok(best.map(|(index, _)| index))
    }

    fn competitive_planning_state(&self, combat: &BattleCombatState) -> Result<BattleCombatState> {
        let mut planning = combat.clone();
        planning.player.item = None;
        // Never inspect the player's selected command, unrevealed moves, PP,
        // held item, or bench. Observed moves are certain; unfilled slots retain
        // plausible public learnset threats instead of assuming a single weak
        // revealed move is the player's entire set.
        let mut ids = combat.player_used_moves.clone();
        if let Some(id) = &combat.player_last_move {
            if !ids.contains(id) {
                ids.push(id.clone());
            }
        }
        if ids.len() < 4 {
            let mut lineage = BTreeSet::from([combat.player.species.id.clone()]);
            loop {
                let parents = self
                    .evolutions
                    .0
                    .iter()
                    .filter(|(_, entries)| {
                        entries.iter().any(|entry| lineage.contains(&entry.species))
                    })
                    .map(|(id, _)| id.clone())
                    .collect::<Vec<_>>();
                let before = lineage.len();
                lineage.extend(parents);
                if lineage.len() == before {
                    break;
                }
            }
            let legal = lineage
                .iter()
                .flat_map(|id| self.learnsets.get(id).into_iter().flatten())
                .filter(|entry| entry.0 <= combat.player.level)
                .map(|entry| entry.1.clone())
                .collect::<BTreeSet<_>>();
            let mut legal = legal
                .into_iter()
                .filter_map(|id| {
                    let m = self.moves.get(&id)?;
                    if m.power == 0
                        || matches!(m.effect.as_str(), "SELFDESTRUCT" | "DREAM_EATER" | "SNORE")
                    {
                        return None;
                    }
                    let learned = LearnedMove {
                        name: id.clone(),
                        current_pp: m.pp,
                        pp_ups: 0,
                    };
                    let threat = self.competitive_move_expected_damage(
                        &planning,
                        BattleSide::Player,
                        &learned,
                    )?;
                    Some((threat, id))
                })
                .collect::<Vec<_>>();
            legal.sort_by(|a, b| b.cmp(a));
            for (_, id) in legal {
                if !ids.contains(&id) {
                    ids.push(id);
                }
                if ids.len() == 4 {
                    break;
                }
            }
        }
        if ids.is_empty() {
            ids.push("STRUGGLE".into());
        }
        let moves = ids
            .into_iter()
            .take(4)
            .map(|name| {
                let m = self
                    .moves
                    .get(&name)
                    .with_context(|| format!("unknown revealed move {name}"))?;
                Ok(LearnedMove {
                    name,
                    current_pp: m.pp,
                    pp_ups: 0,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        planning.player.moves = moves.clone();
        if let Some(transform) = &mut planning.player_transform {
            transform.moves = moves;
        }
        planning.player.item = None;
        planning.player_party = vec![planning.player.clone()];
        planning.player_party_index = 0;
        // Obedience would require hidden trainer identity and make the opponent
        // appear to lose turns for reasons the AI cannot know.
        planning.obedience_trainer_id = None;
        Ok(planning)
    }

    /// Risk-aware two-turn search: average sampled chance outcomes for each
    /// opposing response, then favor the worst response over the mean response.
    /// The continuation reconsiders every legal attack, including support moves.
    fn competitive_action_score(
        &self,
        state: &BattleCombatState,
        action: &BattleAction,
        continuation: bool,
        profile: TrainerBattleProfile,
    ) -> Result<i32> {
        Ok(competitive_response_value(
            &self.competitive_action_values(state, action, continuation, profile)?,
        ))
    }

    fn competitive_action_values(
        &self,
        state: &BattleCombatState,
        action: &BattleAction,
        continuation: bool,
        profile: TrainerBattleProfile,
    ) -> Result<Vec<i32>> {
        let mut responses = Vec::new();
        for slot in competitive_move_slots(state, BattleSide::Player) {
            let mut sum = 0_i64;
            let samples = if continuation { SAMPLES } else { 2 };
            let mut valid = 0;
            for sample in 0..samples {
                // Common random numbers reduce comparison noise across choices.
                let mut random = PlanningRandom::sample(sample);
                let input = BattleTurnInput {
                    player: BattleAction::Move { slot },
                    enemy: action.clone(),
                };
                let outcome =
                    match self.resolve_battle_turn_with_items(state.clone(), input, &mut random) {
                        Ok(outcome) => outcome,
                        // An action can become illegal after a simulated KO/lock.
                        Err(_) => continue,
                    };
                let immediate = self.competitive_position_value(&outcome.state)
                    + self.competitive_style_value(&outcome.state, profile);
                let value =
                    if continuation && outcome.state.player.hp > 0 && outcome.state.enemy.hp > 0 {
                        let mut next = INVALID_SCORE;
                        for enemy_slot in competitive_move_slots(&outcome.state, BattleSide::Enemy)
                        {
                            next = next.max(self.competitive_action_score(
                                &outcome.state,
                                &BattleAction::Move { slot: enemy_slot },
                                false,
                                profile,
                            )?);
                        }
                        if next > INVALID_SCORE {
                            (immediate + next * 3) / 4
                        } else {
                            immediate
                        }
                    } else {
                        immediate
                    };
                sum += i64::from(value);
                valid += 1;
            }
            responses.push(if valid > 0 {
                (sum / valid) as i32
            } else {
                INVALID_SCORE
            });
        }
        Ok(responses)
    }

    fn competitive_position_value(&self, state: &BattleCombatState) -> i32 {
        let own = state
            .enemy_party
            .iter()
            .enumerate()
            .map(|(index, p)| {
                pokemon_value(if index == state.enemy_party_index {
                    &state.enemy
                } else {
                    p
                })
            })
            .sum::<i32>();
        let mut value = own - pokemon_value(&state.player);
        let has_category = |side, category| {
            battle_moves(state, side).iter().any(|m| {
                m.current_pp > 0 && self.type_categories.moves.get(&m.name) == Some(&category)
            })
        };
        for (side, sign) in [(BattleSide::Enemy, 1), (BattleSide::Player, -1)] {
            let p = state.pokemon(side);
            if p.hp == 0 {
                continue;
            }
            let boosts = match side {
                BattleSide::Player => state.player_transform.as_ref(),
                BattleSide::Enemy => state.enemy_transform.as_ref(),
            }
            .map_or(&p.stat_boosts, |transform| &transform.stat_boosts);
            let physical = has_category(side, MoveCategory::Physical);
            let special = has_category(side, MoveCategory::Special);
            let physical_threat = has_category(side.other(), MoveCategory::Physical);
            let special_threat = has_category(side.other(), MoveCategory::Special);
            for (stat, weight) in [
                (Stat::Attack, if physical { 110 } else { 0 }),
                (Stat::SpecialAttack, if special { 110 } else { 0 }),
                (Stat::Speed, 60),
                (Stat::Defense, if physical_threat { 40 } else { 0 }),
                (Stat::SpecialDefense, if special_threat { 40 } else { 0 }),
                (Stat::Accuracy, 65),
                (Stat::Evasion, 65),
            ] {
                value += sign * i32::from(*boosts.get(&stat).unwrap_or(&0)) * weight;
            }
        }
        value += (i32::from(state.player_spikes) - i32::from(state.enemy_spikes)) * 100;
        value += (i32::from(state.enemy_substitute_hp) * 500
            / i32::from(state.enemy.max_hp.max(1)))
            - (i32::from(state.player_substitute_hp) * 500 / i32::from(state.player.max_hp.max(1)));
        value += (i32::from(state.enemy_reflect_turns.min(3))
            * i32::from(has_category(BattleSide::Player, MoveCategory::Physical))
            - i32::from(state.player_reflect_turns.min(3))
                * i32::from(has_category(BattleSide::Enemy, MoveCategory::Physical)))
            * 30;
        value += (i32::from(state.enemy_light_screen_turns.min(3))
            * i32::from(has_category(BattleSide::Player, MoveCategory::Special))
            - i32::from(state.player_light_screen_turns.min(3))
                * i32::from(has_category(BattleSide::Enemy, MoveCategory::Special)))
            * 30;
        if state.player.hp > 0 && state.enemy.hp > 0 {
            // A short search needs a stable estimate beyond its horizon.
            // Otherwise a lucky critical/status sample can make a three-hit
            // attack look better than a reliable two-hit attack, or hide the
            // later payoff of Reflect. Use the real damage preparation rather
            // than base power or a second damage formula for this race estimate.
            let incoming = self.competitive_expected_damage(state, BattleSide::Player);
            let outgoing = self.competitive_expected_damage(state, BattleSide::Enemy);
            let turns = |hp: u16, damage: u32| -> i32 {
                if damage == 0 {
                    12_000
                } else {
                    // Keep fractional turns instead of abrupt score jumps.
                    (u32::from(hp) * 100_000 / damage).clamp(1_000, 12_000) as i32
                }
            };
            let surviving = turns(state.enemy.hp, incoming);
            let defeating = turns(state.player.hp, outgoing);
            let speed = |side| {
                crystal_core::battle::turn::battle_speed(state, side, &self.battle_stat_multipliers)
                    .unwrap_or(0)
            };
            let initiative = match speed(BattleSide::Enemy).cmp(&speed(BattleSide::Player)) {
                std::cmp::Ordering::Greater => 500,
                std::cmp::Ordering::Less => -500,
                std::cmp::Ordering::Equal => 0,
            };
            value += (surviving - defeating + initiative).clamp(-4_000, 4_000) * 120 / 1_000;
        }
        value
    }

    fn competitive_style_value(
        &self,
        state: &BattleCombatState,
        profile: TrainerBattleProfile,
    ) -> i32 {
        let own = &state.enemy;
        let foe = &state.player;
        if own.hp == 0 || foe.hp == 0 {
            return 0;
        }
        let hp = |p: &Pokemon| i32::from(p.hp) * 100 / i32::from(p.max_hp.max(1));
        let stage = |stat| {
            i32::from(
                *state
                    .enemy_transform
                    .as_ref()
                    .map_or(&own.stat_boosts, |t| &t.stat_boosts)
                    .get(&stat)
                    .unwrap_or(&0),
            )
        };
        let category = |category| {
            battle_moves(state, BattleSide::Enemy).iter().any(|m| {
                m.current_pp > 0 && self.type_categories.moves.get(&m.name) == Some(&category)
            })
        };
        let value = match profile {
            TrainerBattleProfile::Balanced => 0,
            TrainerBattleProfile::Offensive => 100 - hp(foe),
            TrainerBattleProfile::Setup => {
                stage(Stat::Attack) * i32::from(category(MoveCategory::Physical)) * 25
                    + stage(Stat::SpecialAttack) * i32::from(category(MoveCategory::Special)) * 25
                    + stage(Stat::Speed) * 10
            }
            TrainerBattleProfile::Disruption => {
                i32::from(foe.status.is_some()) * 55
                    + i32::from(state.player_encore.is_some()) * 25
                    + i32::from(state.player_disable.is_some()) * 20
                    + i32::from(state.player_attracted_by.is_some()) * 20
            }
            TrainerBattleProfile::Endurance => {
                hp(own)
                    + i32::from(state.player_leech_seed_source.is_some()) * 30
                    + i32::from(state.player_curse_source.is_some()) * 30
                    + i32::from(state.enemy_substitute_hp > 0) * 25
            }
            TrainerBattleProfile::Switching => {
                state
                    .enemy_party
                    .iter()
                    .enumerate()
                    .filter(|(i, p)| *i != state.enemy_party_index && p.hp > 0 && !p.is_egg)
                    .map(|(_, p)| hp(p) / 3)
                    .sum::<i32>()
                    + i32::from(state.player_spikes) * 30
            }
        };
        // Tactical gains dominate the bounded preference for a battle style.
        value.clamp(-100, 100)
    }

    fn competitive_expected_damage(&self, state: &BattleCombatState, side: BattleSide) -> u32 {
        competitive_move_slots(state, side)
            .into_iter()
            .filter_map(|slot| battle_moves(state, side).get(slot))
            .filter_map(|learned| self.competitive_move_expected_damage(state, side, learned))
            .max()
            .unwrap_or(0)
    }

    fn competitive_move_expected_damage(
        &self,
        state: &BattleCombatState,
        side: BattleSide,
        learned: &LearnedMove,
    ) -> Option<u32> {
        if learned.current_pp == 0 {
            return None;
        }
        let m = self.moves.get(&learned.name)?;
        if m.power == 0
            || m.effect == "SELFDESTRUCT"
            || (m.effect == "DREAM_EATER"
                && state.pokemon(side.other()).status.as_deref() != Some("SLEEP"))
            || (m.effect == "SNORE" && state.pokemon(side).status.as_deref() != Some("SLEEP"))
        {
            return None;
        }
        let preview = crystal_core::battle::turn::preview_battle_move(
            state,
            side,
            m,
            &self.items,
            &self.move_priorities,
            &self.battle_stat_multipliers,
            &self.type_categories,
            &self.type_effectiveness,
            &self.weather_modifiers,
        )
        .ok()?;
        let [min, max] = preview.damage_on_hit?;
        let hits = match m.effect.as_str() {
            "DOUBLE_HIT" | "POISON_MULTI_HIT" => 2,
            "MULTI_HIT" => 3,
            _ => 1,
        };
        let two_turn = matches!(
            m.effect.as_str(),
            "HYPER_BEAM" | "SKULL_BASH" | "SKY_ATTACK" | "RAZOR_WIND" | "FLY"
        ) || (m.effect == "SOLARBEAM"
            && state.weather != crystal_core::battle::damage::Weather::Sun);
        Some(
            (u32::from(min) + u32::from(max)) * u32::from(m.accuracy.min(100)) * hits
                / if two_turn { 4 } else { 2 },
        )
    }
}

fn competitive_response_value(responses: &[i32]) -> i32 {
    let Some(worst) = responses.iter().min() else {
        return INVALID_SCORE;
    };
    let mean = responses.iter().sum::<i32>() / responses.len() as i32;
    (worst * 3 + mean) / 4
}

/// Deterministic fictitious play over the small simultaneous-action matrix.
/// These are fixed-point counts, so native and browser replay choose the same
/// distribution. Live RNG is used only to draw the final action.
fn competitive_mixed_policy(values: &[Vec<i32>]) -> Option<Vec<u32>> {
    let columns = values.first()?.len();
    if values.len() < 2
        || columns < 2
        || values
            .iter()
            .any(|row| row.len() != columns || row.contains(&INVALID_SCORE))
    {
        return None;
    }
    let matrix = values
        .iter()
        .map(|row| {
            let mean = row.iter().sum::<i32>() / columns as i32;
            row.iter()
                .map(|v| i64::from((v * 3 + mean) / 4))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let pure = matrix.iter().map(|row| *row.iter().min().unwrap()).max()?;
    let mut row_totals = vec![0_i64; matrix.len()];
    let mut column_totals = vec![0_i64; columns];
    let mut counts = vec![0_u32; matrix.len()];
    const ITERATIONS: u32 = 1024;
    for _ in 0..ITERATIONS {
        let row = (0..matrix.len()).max_by_key(|&i| row_totals[i]).unwrap();
        counts[row] += 1;
        for (column, total) in column_totals.iter_mut().enumerate() {
            *total += matrix[row][column];
        }
        let column = (0..columns).min_by_key(|&j| column_totals[j]).unwrap();
        for (row, total) in row_totals.iter_mut().enumerate() {
            *total += matrix[row][column];
        }
    }
    let mixed = *column_totals.iter().min()? / i64::from(ITERATIONS);
    (mixed > pure + i64::from(1_000 / (2 * SAMPLES))).then_some(counts)
}

#[cfg(test)]
mod policy_tests {
    use super::*;

    #[test]
    fn simultaneous_policy_mixes_complementary_counters_and_rejects_dominated_moves() {
        let matrix = vec![
            vec![1_000, -1_000],
            vec![-1_000, 1_000],
            vec![-2_000, -2_000],
        ];
        let weights = competitive_mixed_policy(&matrix).unwrap();
        assert!(weights[0].abs_diff(weights[1]) < 70, "{weights:?}");
        assert!(weights[2] < 3, "{weights:?}");
        let guaranteed = (0..2)
            .map(|column| {
                matrix
                    .iter()
                    .zip(&weights)
                    .map(|(row, count)| i64::from(row[column]) * i64::from(*count))
                    .sum::<i64>()
            })
            .min()
            .unwrap();
        assert!(
            guaranteed > -70 * 1024,
            "mixture must resist either pure counter"
        );
        assert!(competitive_mixed_policy(&[vec![500, 800], vec![100, 200]]).is_none());
        assert!(competitive_mixed_policy(&[vec![100, 100], vec![99, 101]]).is_none());
    }
}

fn competitive_move_slots(state: &BattleCombatState, side: BattleSide) -> Vec<usize> {
    let (disable, encore) = match side {
        BattleSide::Player => (&state.player_disable, &state.player_encore),
        BattleSide::Enemy => (&state.enemy_disable, &state.enemy_encore),
    };
    let mut slots = battle_moves(state, side)
        .iter()
        .take(4)
        .enumerate()
        .filter(|(_, m)| {
            m.current_pp > 0
                && !disable
                    .as_ref()
                    .is_some_and(|d| d.turns_remaining > 0 && d.move_name == m.name)
                && !encore
                    .as_ref()
                    .is_some_and(|e| e.turns_remaining > 0 && e.move_name != m.name)
        })
        .map(|(slot, _)| slot)
        .collect::<Vec<_>>();
    // The production engine resolves Struggle/locked continuations itself.
    if slots.is_empty() {
        slots.push(0);
    }
    slots
}

fn pokemon_value(p: &Pokemon) -> i32 {
    if p.hp == 0 || p.is_egg {
        return 0;
    }
    let status_cost = match p.status.as_deref() {
        Some("SLEEP") | Some("FREEZE") => 300,
        Some("TOXIC" | "BAD_POISON") => 200,
        Some("PARALYSIS") | Some("BURN") => 150,
        Some("POISON") => 100,
        _ => 0,
    };
    1_600 + i32::from(p.hp) * 1_000 / i32::from(p.max_hp.max(1))
        - status_cost
        - if p.perish_song_turns > 0 {
            600 / i32::from(p.perish_song_turns)
        } else {
            0
        }
}
