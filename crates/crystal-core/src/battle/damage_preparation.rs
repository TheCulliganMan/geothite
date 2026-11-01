//! Shared damage preparation for live turns and read-only move previews.

use std::collections::BTreeMap;

use crate::battle::damage::{
    DamageContext, DamageResult, TypeCategories, TypeEffectivenessTable, TypeMultiplier,
    WeatherModifiers, calculate_damage, calculate_type_effectiveness_multiplier_with_foresight,
    is_physical_move,
};
use crate::battle::stats::BattleStatMultiplierTables;
use crate::models::pokemon::default_stat_boosts;
use crate::models::{Item, Move, Stat};
use crate::random::BattleRandomSource;

use super::damage_preview_cache::DamagePreviewCache;

use super::{
    BattleCombatState, BattleEvent, BattleSide, BattleTurnError, PreparedCriticalHit,
    PreparedDamageHit, PreparedDamageHitOutcome, active_damage_screen,
    apply_pre_check_hit_final_damage_modifier, apply_state_damage_power_modifiers,
    badge_boost_active, badge_type_boost_active, crystal_damage_variation_roll,
    crystal_type_register, ditto_holds_metal_powder, dynamic_move_power,
    effective_battle_pokemon_borrowed, effective_pokemon_types, fixed_damage_amount,
    held_item_type_boost_percent, identified_state, is_fixed_damage_effect, loaded_stats,
    rage_counter, roll_critical_hit, species_held_attack_boost_active,
};

/// Optional read-only bounds requested by the move preview path.
pub(super) struct DamagePreview<'a, 'rules> {
    pub(super) cache: Option<&'a mut DamagePreviewCache<'rules>>,
    pub(super) bounds: &'a mut [u16; 2],
}

// Present's state changes are handled by the live wrapper before this function.
// The remaining calculation only reads combat state. Critical selection reports
// its register write separately so previews can use the same path without copying
// the battle, its parties, and their immutable species/move data.
#[expect(
    clippy::too_many_arguments,
    reason = "Shares the existing live damage preparation boundary"
)]
pub(super) fn finish_damage_hit(
    state: &BattleCombatState,
    side: BattleSide,
    move_name: &str,
    move_data: &Move,
    mut prepared_move_data: Move,
    hit_number: u8,
    target_switching: bool,
    stat_multipliers: &BattleStatMultiplierTables,
    type_categories: &TypeCategories,
    type_effectiveness: &TypeEffectivenessTable,
    weather_modifiers: &WeatherModifiers,
    items: &BTreeMap<String, Item>,
    rng: &mut dyn BattleRandomSource,
    events: &mut Vec<BattleEvent>,
    prepared_critical: Option<PreparedCriticalHit>,
    critical_register: &mut Option<u8>,
    mut preview: Option<DamagePreview<'_, '_>>,
) -> Result<PreparedDamageHitOutcome, BattleTurnError> {
    let post_type_damage_multiplier =
        apply_state_damage_power_modifiers(state, side, &mut prepared_move_data, events);
    let mut attacker = effective_battle_pokemon_borrowed(state, side);
    if side == BattleSide::Enemy && attacker.status.as_deref() == Some("BURN") {
        // Enemy battle stats retain whether burn was applied when the struct
        // was loaded; the party status byte alone is not authoritative.
        attacker.to_mut().status = None;
    }
    let mut defender = effective_battle_pokemon_borrowed(state, side.other());
    let link_present_registers = state.link_colosseum && prepared_move_data.effect == "PRESENT";
    if link_present_registers {
        // Crystal deliberately preserves Gold/Silver link compatibility by
        // not saving bc/de around Present's early STAB command. On return,
        // DamageCalc receives b=type matchup, c=user type 2, and e=target
        // type 2 instead of Attack, Defense, and level.
        let matchup = calculate_type_effectiveness_multiplier_with_foresight(
            type_effectiveness,
            &prepared_move_data.move_type,
            &effective_pokemon_types(state, side.other()),
            identified_state(state, side.other()),
        )
        .map_err(BattleTurnError::DamageCalculation)?;
        let matchup_register =
            u16::from(((u32::from(matchup.numerator) * 10) / u32::from(matchup.denominator)) as u8);
        let user_type2 = u16::from(
            crystal_type_register(&attacker.species.type2)
                .map_err(BattleTurnError::DamageCalculation)?,
        );
        let target_type2 = crystal_type_register(&defender.species.type2)
            .map_err(BattleTurnError::DamageCalculation)?;
        let attacker = attacker.to_mut();
        let defender = defender.to_mut();
        attacker.level = target_type2;
        attacker.attack = matchup_register;
        attacker.special_attack = matchup_register;
        attacker.stat_boosts = default_stat_boosts();
        attacker.status = None;
        attacker.item = None;
        defender.defense = user_type2;
        defender.special_defense = user_type2;
        defender.stat_boosts = default_stat_boosts();
    }
    let mut lower_damage = None;
    let (critical, critical_roll, critical_threshold, damage_roll, result) =
        if is_fixed_damage_effect(move_data) {
            // ConstantDamage precedes CheckHit and ResetTypeMatchup in all
            // four fixed-damage streams. Psywave must therefore complete its
            // rejection loop even if accuracy, Protect, or type immunity
            // prevents the retained amount from ever reaching ApplyDamage.
            let fixed_damage = fixed_damage_amount(&attacker, &defender, move_data, rng)
                .expect("fixed-damage effect lost its exact amount");
            let type_multiplier = calculate_type_effectiveness_multiplier_with_foresight(
                type_effectiveness,
                &move_data.move_type,
                &effective_pokemon_types(state, side.other()),
                identified_state(state, side.other()),
            )
            .map_err(BattleTurnError::DamageCalculation)?;
            let (damage, type_multiplier) = if type_multiplier.numerator == 0 {
                (0, TypeMultiplier::zero())
            } else {
                (fixed_damage, TypeMultiplier::one())
            };
            (
                false,
                0,
                0,
                255,
                DamageResult {
                    damage,
                    type_multiplier,
                },
            )
        } else {
            let held_type_boost_percent = held_item_type_boost_percent(
                state,
                side,
                &prepared_move_data.move_type,
                items,
                events,
            )?;
            let PreparedCriticalHit {
                critical: rolled_critical,
                roll: critical_roll,
                threshold: critical_threshold,
            } = if let Some(prepared_critical) = prepared_critical {
                prepared_critical
            } else {
                let (critical, roll, threshold) =
                    roll_critical_hit(side, move_name, &attacker, items, rng)?;
                // The caller applies this even if later damage validation fails,
                // matching the live preparation's register write on error.
                *critical_register = Some(u8::from(critical));
                PreparedCriticalHit {
                    critical,
                    roll,
                    threshold,
                }
            };
            let critical = rolled_critical;
            if move_data.effect == "MAGNITUDE" {
                prepared_move_data.power =
                    dynamic_move_power(side, move_name, &attacker, move_data, rng, events);
            }
            let damage_roll = crystal_damage_variation_roll(rng);
            let physical = is_physical_move(type_categories, &prepared_move_data)
                .map_err(BattleTurnError::DamageCalculation)?;
            let context = DamageContext {
                is_critical: critical,
                is_confusion_damage: false,
                defender_identified: identified_state(state, side.other()),
                weather: state.weather,
                random_roll: damage_roll,
                attacker_badge_boost: badge_boost_active(
                    state,
                    side,
                    if physical {
                        Stat::Attack
                    } else {
                        Stat::SpecialAttack
                    },
                ),
                defender_badge_boost: badge_boost_active(
                    state,
                    side.other(),
                    if physical {
                        Stat::Defense
                    } else {
                        Stat::SpecialDefense
                    },
                ),
                attacker_type_badge_boost: prepared_move_data.name != "STRUGGLE"
                    && badge_type_boost_active(state, side, &prepared_move_data.move_type),
                defender_metal_powder: !link_present_registers
                    && ditto_holds_metal_powder(state, side.other()),
                attacker_species_item_boost: !link_present_registers
                    && species_held_attack_boost_active(state, side, physical),
                defender_screen: !link_present_registers
                    && active_damage_screen(
                        state,
                        side.other(),
                        type_categories,
                        &prepared_move_data,
                    )?
                    .is_some(),
                link_colosseum: state.link_colosseum,
                held_type_boost_percent,
                pre_stab_multiplier: if prepared_move_data.effect == "TRIPLE_KICK" {
                    hit_number.max(1)
                } else {
                    1
                },
                post_type_damage_multiplier,
                rage_counter: if prepared_move_data.effect == "RAGE" {
                    rage_counter(state, side)
                } else {
                    0
                },
                attacker_burn_penalty: !link_present_registers
                    && side == BattleSide::Enemy
                    && state.enemy_burn_attack_penalty_active,
                attacker_badge_before_status: side == BattleSide::Player
                    && state.player_badge_before_status,
                attacker_loaded_stat: (!link_present_registers).then(|| {
                    let stats = loaded_stats(state, side);
                    if physical {
                        stats.attack
                    } else {
                        stats.special_attack
                    }
                }),
                defender_loaded_stat: (!link_present_registers).then(|| {
                    let stats = loaded_stats(state, side.other());
                    if physical {
                        stats.defense
                    } else {
                        stats.special_defense
                    }
                }),
            };
            let mut result = if let Some(cache) = preview
                .as_mut()
                .and_then(|preview| preview.cache.as_deref_mut())
            {
                cache.calculate(&attacker, &defender, &prepared_move_data, context)
            } else {
                calculate_damage(
                    &attacker,
                    &defender,
                    &prepared_move_data,
                    stat_multipliers,
                    type_categories,
                    type_effectiveness,
                    weather_modifiers,
                    context,
                )
            }
            .map_err(BattleTurnError::DamageCalculation)?;
            if preview.is_some() {
                // Previewed effects have deterministic preparation. Reuse the
                // same battlers, move and context for both variation endpoints.
                let lower_context = DamageContext {
                    random_roll: 217,
                    ..context
                };
                lower_damage = Some(
                    if let Some(cache) = preview
                        .as_mut()
                        .and_then(|preview| preview.cache.as_deref_mut())
                    {
                        cache.calculate(&attacker, &defender, &prepared_move_data, lower_context)
                    } else {
                        calculate_damage(
                            &attacker,
                            &defender,
                            &prepared_move_data,
                            stat_multipliers,
                            type_categories,
                            type_effectiveness,
                            weather_modifiers,
                            lower_context,
                        )
                    }
                    .map_err(BattleTurnError::DamageCalculation)?,
                );
            }
            apply_pre_check_hit_final_damage_modifier(
                state,
                side,
                move_name,
                move_data,
                &prepared_move_data,
                target_switching,
                &mut result,
                events,
            );
            (
                critical,
                critical_roll,
                critical_threshold,
                damage_roll,
                result,
            )
        };
    if let Some(preview) = preview {
        let lower = if let Some(mut lower) = lower_damage {
            apply_pre_check_hit_final_damage_modifier(
                state,
                side,
                move_name,
                move_data,
                &prepared_move_data,
                target_switching,
                &mut lower,
                &mut Vec::new(),
            );
            lower.damage
        } else {
            // Constant-damage effects have the same two bounds.
            result.damage
        };
        *preview.bounds = [lower, result.damage];
    }
    Ok(PreparedDamageHitOutcome::Damage(PreparedDamageHit {
        prepared_move_data,
        critical,
        critical_roll,
        critical_threshold,
        damage_roll,
        result,
    }))
}
