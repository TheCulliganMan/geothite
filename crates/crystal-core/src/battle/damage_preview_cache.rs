//! A decision-scoped cache of pure damage calculations, bound to immutable rules.

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::battle::damage::{
    DamageCalculationError, DamageContext, DamagePokemon, DamageResult, TypeCategories,
    TypeEffectivenessTable, WeatherModifiers, calculate_damage_with_inputs,
};
use crate::battle::stats::BattleStatMultiplierTables;
use crate::models::{Move, Pokemon};

const MAX_ENTRIES: usize = 1024;

struct Entry {
    attacker: DamagePokemon<'static>,
    defender: DamagePokemon<'static>,
    data: Move,
    context: DamageContext,
    result: DamageResult,
}

pub(super) struct DamagePreviewCache<'a> {
    multipliers: &'a BattleStatMultiplierTables,
    categories: &'a TypeCategories,
    effectiveness: &'a TypeEffectivenessTable,
    weather: &'a WeatherModifiers,
    entries: HashMap<u64, Vec<Entry>>,
    count: usize,
}

impl<'a> DamagePreviewCache<'a> {
    pub(super) fn new(
        multipliers: &'a BattleStatMultiplierTables,
        categories: &'a TypeCategories,
        effectiveness: &'a TypeEffectivenessTable,
        weather: &'a WeatherModifiers,
    ) -> Self {
        Self {
            multipliers,
            categories,
            effectiveness,
            weather,
            entries: HashMap::new(),
            count: 0,
        }
    }

    pub(super) fn calculate(
        &mut self,
        attacker: &Pokemon,
        defender: &Pokemon,
        data: &Move,
        context: DamageContext,
    ) -> Result<DamageResult, DamageCalculationError> {
        let attacker = DamagePokemon::from(attacker);
        let defender = DamagePokemon::from(defender);
        // A cheap coarse hash only chooses a bucket. Compare every actual
        // calculation input before reusing a value, including on collisions.
        let mut hasher = DefaultHasher::new();
        (
            attacker.id.as_ref(),
            defender.id.as_ref(),
            attacker.attack,
            defender.defense,
            &data.name,
            context.random_roll,
        )
            .hash(&mut hasher);
        let key = hasher.finish();
        if let Some(entry) = self.entries.get(&key).and_then(|entries| {
            entries.iter().find(|entry| {
                entry.context == context
                    && entry.data == *data
                    && entry.attacker == attacker
                    && entry.defender == defender
            })
        }) {
            return Ok(entry.result);
        }
        let result = calculate_damage_with_inputs(
            &attacker,
            &defender,
            data,
            self.multipliers,
            self.categories,
            self.effectiveness,
            self.weather,
            context,
        )?;
        if self.count < MAX_ENTRIES {
            self.entries.entry(key).or_default().push(Entry {
                attacker: attacker.into_owned(),
                defender: defender.into_owned(),
                data: data.clone(),
                context,
                result,
            });
            self.count += 1;
        }
        Ok(result)
    }
}
