use super::*;
use crate::battle::damage::MoveCategory;

struct PreviewVariation(u8);
impl BattleRandomSource for PreviewVariation {
    fn battle_random_byte(&mut self) -> u8 {
        self.0.rotate_left(1)
    }
}

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

/// Inspect the same damage preparation used by real turns using borrowed combat data.
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
    preview_with_cache(
        state,
        side,
        move_data,
        items,
        priorities,
        stat_multipliers,
        categories,
        effectiveness,
        weather,
        None,
    )
}

/// Reuse pure damage results during one search against immutable battle rules.
/// The bounded cache checks complete inputs; game state and RNG are never changed.
/// Drop the previewer between decisions to release its cached battlers.
pub struct BattleMovePreviewer<'a> {
    items: &'a BTreeMap<String, Item>,
    priorities: &'a MovePriorityTable,
    multipliers: &'a BattleStatMultiplierTables,
    categories: &'a TypeCategories,
    effectiveness: &'a TypeEffectivenessTable,
    weather: &'a WeatherModifiers,
    damage: damage_preview_cache::DamagePreviewCache<'a>,
}

impl<'a> BattleMovePreviewer<'a> {
    /// Bind a new cache to rules that cannot change while it is borrowed.
    pub fn new(
        items: &'a BTreeMap<String, Item>,
        priorities: &'a MovePriorityTable,
        multipliers: &'a BattleStatMultiplierTables,
        categories: &'a TypeCategories,
        effectiveness: &'a TypeEffectivenessTable,
        weather: &'a WeatherModifiers,
    ) -> Self {
        Self {
            items,
            priorities,
            multipliers,
            categories,
            effectiveness,
            weather,
            damage: damage_preview_cache::DamagePreviewCache::new(
                multipliers,
                categories,
                effectiveness,
                weather,
            ),
        }
    }

    /// Calculate non-critical damage bounds without constructing display metadata.
    ///
    /// # Errors
    /// Reports invalid damage preparation or move-priority data.
    pub fn damage_on_hit(
        &mut self,
        state: &BattleCombatState,
        side: BattleSide,
        data: &Move,
    ) -> Result<Option<[u16; 2]>, BattleTurnError> {
        let category = preview_category(data, self.categories)?;
        let bounds = preview_bounds(
            state,
            side,
            data,
            category,
            self.items,
            self.multipliers,
            self.categories,
            self.effectiveness,
            self.weather,
            Some(&mut self.damage),
        )?;
        move_priority(data, self.priorities)?;
        Ok(bounds.map(|(_, bounds)| bounds))
    }

    /// Inspect a move without changing combat or consuming game RNG.
    ///
    /// # Errors
    /// Returns the same missing or invalid battle-data errors as `preview_battle_move`.
    pub fn preview(
        &mut self,
        state: &BattleCombatState,
        side: BattleSide,
        data: &Move,
    ) -> Result<BattleMovePreview, BattleTurnError> {
        preview_with_cache(
            state,
            side,
            data,
            self.items,
            self.priorities,
            self.multipliers,
            self.categories,
            self.effectiveness,
            self.weather,
            Some(&mut self.damage),
        )
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Shares the existing move preview boundary"
)]
fn preview_with_cache(
    state: &BattleCombatState,
    side: BattleSide,
    move_data: &Move,
    items: &BTreeMap<String, Item>,
    priorities: &MovePriorityTable,
    stat_multipliers: &BattleStatMultiplierTables,
    categories: &TypeCategories,
    effectiveness: &TypeEffectivenessTable,
    weather: &WeatherModifiers,
    cache: Option<&mut damage_preview_cache::DamagePreviewCache<'_>>,
) -> Result<BattleMovePreview, BattleTurnError> {
    let category = preview_category(move_data, categories)?;
    let conditional = preview_is_conditional(move_data);
    let mut resolved = move_data.clone();
    if !conditional {
        resolved = damage_move_data(
            side,
            &move_data.name,
            state.pokemon(side),
            move_data,
            &mut PreviewVariation(255),
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
    let damage = preview_bounds(
        state,
        side,
        move_data,
        category,
        items,
        stat_multipliers,
        categories,
        effectiveness,
        weather,
        cache,
    )?;
    if let Some((multiplier, _)) = damage {
        matchup = multiplier;
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
        damage_on_hit: damage.map(|(_, bounds)| bounds),
        per_hit: matches!(
            move_data.effect.as_str(),
            "MULTI_HIT" | "DOUBLE_HIT" | "POISON_MULTI_HIT" | "TRIPLE_KICK"
        ),
    })
}

fn preview_category(
    move_data: &Move,
    categories: &TypeCategories,
) -> Result<MoveCategory, BattleTurnError> {
    Ok(
        if let Some(category) = categories.moves.get(&move_data.name) {
            *category
        } else if move_data.power == 0 {
            MoveCategory::Status
        } else if is_physical_move(categories, move_data)
            .map_err(BattleTurnError::DamageCalculation)?
        {
            MoveCategory::Physical
        } else {
            MoveCategory::Special
        },
    )
}

fn preview_is_conditional(move_data: &Move) -> bool {
    matches!(
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
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "Shares the existing damage bounds boundary"
)]
fn preview_bounds(
    state: &BattleCombatState,
    side: BattleSide,
    move_data: &Move,
    category: MoveCategory,
    items: &BTreeMap<String, Item>,
    stat_multipliers: &BattleStatMultiplierTables,
    categories: &TypeCategories,
    effectiveness: &TypeEffectivenessTable,
    weather: &WeatherModifiers,
    cache: Option<&mut damage_preview_cache::DamagePreviewCache<'_>>,
) -> Result<Option<(TypeMultiplier, [u16; 2])>, BattleTurnError> {
    if preview_is_conditional(move_data) || category == MoveCategory::Status {
        return Ok(None);
    }
    let mut random = PreviewVariation(255);
    let mut events = Vec::new();
    let mut bounds = [0; 2];
    let prepared_move = damage_move_data(
        side,
        &move_data.name,
        state.pokemon(side),
        move_data,
        &mut random,
        &mut events,
    );
    if prepared_move.power == 0 {
        return Ok(None);
    }
    let outcome = damage_preparation::finish_damage_hit(
        state,
        side,
        &move_data.name,
        move_data,
        prepared_move,
        1,
        false,
        stat_multipliers,
        categories,
        effectiveness,
        weather,
        items,
        &mut random,
        &mut events,
        Some(PreparedCriticalHit {
            critical: false,
            roll: 255,
            threshold: 0,
        }),
        &mut None,
        Some(damage_preparation::DamagePreview {
            cache,
            bounds: &mut bounds,
        }),
    )?;
    if let PreparedDamageHitOutcome::Damage(hit) = outcome {
        if move_data.effect == "STOMP" && minimized_state(state, side.other()) {
            bounds = bounds.map(|damage| damage.saturating_mul(2));
        }
        return Ok(Some((hit.result.type_multiplier, bounds)));
    }
    Ok(None)
}
