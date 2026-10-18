use super::*;
use crate::battle::damage::MoveCategory;

/// Simulate entry without spending a turn or advancing battle RNG. Planning
/// shares ordinary switch resets and Spikes with real forced replacements.
pub fn preview_battle_replacement(
    state: &BattleCombatState,
    side: BattleSide,
    party_index: usize,
) -> Result<BattleCombatState, BattleTurnError> {
    let mut entered = state.clone();
    switch_battle_combat_pokemon_normally(&mut entered, side, party_index)?;
    apply_switch_in_spikes_damage(&mut entered, side);
    Ok(entered)
}

/// Damage is one non-critical hit against the current target, before Focus
/// Band/Endure/Substitute routing. Conditional/random effects have no range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleMovePreview {
    pub move_id: String,
    pub move_type: String,
    pub category: MoveCategory,
    pub power: Option<u16>,
    pub accuracy: u8,
    pub priority: i8,
    pub stab: bool,
    pub effectiveness: TypeMultiplier,
    pub damage_on_hit: Option<[u16; 2]>,
    pub per_hit: bool,
}

/// Inspect the same damage preparation used by real turns on private clones.
/// This does not advance game RNG, mutate battle memory, or emit game events.
pub fn preview_battle_move(
    state: &BattleCombatState,
    side: BattleSide,
    move_data: &Move,
    items: &BTreeMap<String, Item>,
    priorities: &MovePriorityTable,
    stat_multipliers: &BattleStatMultiplierTables,
    categories: &TypeCategories,
    effectiveness: &TypeEffectivenessTable,
    weather: &WeatherModifiers,
) -> Result<BattleMovePreview, BattleTurnError> {
    struct Variation(u8);
    impl BattleRandomSource for Variation {
        fn battle_random_byte(&mut self) -> u8 {
            self.0.rotate_left(1)
        }
    }
    let category = if let Some(category) = categories.moves.get(&move_data.name) {
        *category
    } else if move_data.power == 0 {
        MoveCategory::Status
    } else if is_physical_move(categories, move_data).map_err(BattleTurnError::DamageCalculation)? {
        MoveCategory::Physical
    } else {
        MoveCategory::Special
    };
    let conditional = matches!(
        move_data.effect.as_str(),
        "PRESENT"
            | "MAGNITUDE"
            | "PSYWAVE"
            | "COUNTER"
            | "MIRROR_COAT"
            | "BIDE"
            | "OHKO"
            | "FUTURE_SIGHT"
            | "BEAT_UP"
            | "METRONOME"
            | "MIRROR_MOVE"
            | "SLEEP_TALK"
            | "TRIPLE_KICK"
    );
    let mut resolved = move_data.clone();
    if !conditional {
        resolved = damage_move_data(
            side,
            &move_data.name,
            state.pokemon(side),
            move_data,
            &mut Variation(255),
            &mut Vec::new(),
        );
    }
    let mut matchup = calculate_type_effectiveness_multiplier_with_foresight(
        effectiveness,
        &resolved.move_type,
        &effective_pokemon_types(state, side.other()),
        identified_state(state, side.other()),
    )
    .map_err(BattleTurnError::DamageCalculation)?;
    let mut damage = Vec::new();
    if !conditional && category != MoveCategory::Status && resolved.power > 0 {
        for roll in [217, 255] {
            let mut scratch = state.clone();
            let outcome = prepare_damage_hit(
                &mut scratch,
                side,
                &move_data.name,
                move_data,
                1,
                false,
                stat_multipliers,
                categories,
                effectiveness,
                weather,
                items,
                &mut Variation(roll),
                &mut Vec::new(),
                Some(PreparedCriticalHit {
                    critical: false,
                    roll: 255,
                    threshold: 0,
                }),
            )?;
            if let PreparedDamageHitOutcome::Damage(hit) = outcome {
                matchup = hit.result.type_multiplier;
                let amount = if move_data.effect == "STOMP" && minimized_state(state, side.other())
                {
                    hit.result.damage.saturating_mul(2)
                } else {
                    hit.result.damage
                };
                damage.push(amount);
            }
        }
    }
    Ok(BattleMovePreview {
        move_id: move_data.name.clone(),
        move_type: resolved.move_type.clone(),
        category,
        power: (!conditional).then_some(resolved.power),
        accuracy: move_data.accuracy,
        priority: move_priority(move_data, priorities)?,
        stab: category != MoveCategory::Status
            && move_data.name != "STRUGGLE"
            && !is_fixed_damage_effect(move_data)
            && effective_pokemon_types(state, side).contains(&resolved.move_type),
        effectiveness: matchup,
        damage_on_hit: (damage.len() == 2).then(|| [damage[0], damage[1]]),
        per_hit: matches!(
            move_data.effect.as_str(),
            "MULTI_HIT" | "DOUBLE_HIT" | "POISON_MULTI_HIT" | "TRIPLE_KICK"
        ),
    })
}
