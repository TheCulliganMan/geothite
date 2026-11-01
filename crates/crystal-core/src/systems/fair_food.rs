//! Fair-only purchase surprises. Food identities are exact item and vendor IDs.
use serde::{Deserialize, Serialize};

use crate::battle::start::WildBattleStart;
use crate::random::{CrystalRandom, DividerSource};
use crate::state::GameState;

pub const FAIR_MAP: &str = "MnStateFairgrounds";
pub const SURPRISE_LEVEL: u8 = 10;
const PENDING_KEY: &str = "GEOTHITE_FAIR_FOOD_PENDING_V1";

/// A resolved encounter, including its random attributes, saved with payment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingFairFood {
    pub item_id: String,
    pub battle: WildBattleStart,
}

/// Returns the matching basic form only at its authored Fair food counters.
pub fn species_for_purchase(map: &str, mart: &str, item: &str) -> Option<&'static str> {
    if map != FAIR_MAP {
        return None;
    }
    match (item, mart) {
        ("FAIR_CHEESE_CURDS", "MART_MnFairCurdsVendor") => Some("CURDLET"),
        ("FAIR_HOTDISH", "MART_MnFairCurdsVendor") => Some("TOTLIT"),
        ("FAIR_COOKIE_PAIL", "MART_MnFairCookieVendor" | "MART_MnFairBakerVendor") => {
            Some("DOUGHLIN")
        }
        ("FAIR_KETTLE_CORN", "MART_MnFairKettleVendor" | "MART_MnFairGardenSnack") => {
            Some("PIPCORN")
        }
        _ => None,
    }
}

/// Rejects the six surplus byte values so exactly 50/250 accepted rolls win.
/// Divider failure is returned without silently turning it into a losing roll.
pub fn roll_surprise<S>(rng: &mut CrystalRandom<&mut S>) -> Result<bool, S::Error>
where
    S: DividerSource + ?Sized,
{
    loop {
        let value = rng.random(false)?.value;
        if value < 250 {
            return Ok(value < 50);
        }
    }
}

/// Select a living, non-Egg partner for the optional encounter.
pub fn usable_party_index(state: &GameState) -> Option<usize> {
    state
        .storage
        .party
        .pokemon
        .iter()
        .enumerate()
        .find_map(|(index, pokemon)| {
            pokemon
                .as_ref()
                .filter(|pokemon| pokemon.hp > 0 && !pokemon.is_egg)
                .map(|_| index)
        })
}

impl PendingFairFood {
    /// Read the existing saved memory extension without changing binary WRAM shape.
    /// Malformed pending data is rejected instead of silently rerolling it.
    pub fn read(state: &GameState) -> Result<Option<Self>, String> {
        state
            .script_runtime
            .memory
            .get(PENDING_KEY)
            .map(|encoded| {
                serde_json::from_str(encoded)
                    .map_err(|error| format!("invalid saved Fair food surprise: {error}"))
            })
            .transpose()
    }

    /// Persist the already resolved encounter with its completed transaction.
    pub fn write(&self, state: &mut GameState) -> Result<(), String> {
        let encoded = serde_json::to_string(self)
            .map_err(|error| format!("encode Fair food surprise: {error}"))?;
        state
            .script_runtime
            .memory
            .insert(PENDING_KEY.into(), encoded);
        Ok(())
    }

    /// Clear a consumed event; repeating this operation has no effect.
    pub fn clear(state: &mut GameState) {
        state.script_runtime.memory.remove(PENDING_KEY);
    }

    pub fn reveal(&self) -> &'static str {
        match self.item_id.as_str() {
            "FAIR_CHEESE_CURDS" => "The bag squeaked!\nIt's a CURDLET!",
            "FAIR_COOKIE_PAIL" => "The pail wobbled!\nIt's a DOUGHLIN!",
            "FAIR_KETTLE_CORN" => "Pop! Pop!\nIt's a PIPCORN!",
            _ => "The hotdish moved!\nIt's a TOTLIT!",
        }
    }

    /// Rejects stale or forged pending events before a save can be resumed.
    pub fn validate(&self, state: &GameState) -> Result<(), String> {
        let expected = match self.item_id.as_str() {
            "FAIR_CHEESE_CURDS" => "CURDLET",
            "FAIR_COOKIE_PAIL" => "DOUGHLIN",
            "FAIR_KETTLE_CORN" => "PIPCORN",
            "FAIR_HOTDISH" => "TOTLIT",
            _ => return Err("unknown pending Fair food item".into()),
        };
        if self.battle.encounter.map_name != FAIR_MAP
            || self.battle.enemy_pokemon.species.id != expected
            || self.battle.enemy_pokemon.level != SURPRISE_LEVEL
            || self.battle.battle_type != "BATTLETYPE_NORMAL"
            || self.battle.encounter.roaming_slot.is_some()
            || !self
                .battle
                .encounter
                .resolved
                .as_ref()
                .is_some_and(|resolved| {
                    resolved.encounter.species == expected
                        && resolved.encounter.level == SURPRISE_LEVEL
                        && resolved.level == SURPRISE_LEVEL
                })
            || self.battle.enemy_party != [self.battle.enemy_pokemon.clone()]
            || !matches!(state.battle, crate::state::BattleMemory::Inactive)
        {
            return Err("invalid pending Fair food battle".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::{CrystalRandomState, ReplayDivider};

    #[test]
    fn fair_food_twenty_percent_boundary_and_rejection() {
        for (value, expected) in [(0, true), (49, true), (50, false), (249, false)] {
            let mut divider = ReplayDivider::new([0, 0]);
            let mut rng =
                CrystalRandom::new(CrystalRandomState { add: 0, sub: value }, &mut divider);
            assert_eq!(roll_surprise(&mut rng), Ok(expected));
            assert_eq!(divider.consumed(), 2);
        }
        let mut divider = ReplayDivider::new([0, 0, 0, 201]);
        let mut rng = CrystalRandom::new(CrystalRandomState { add: 0, sub: 250 }, &mut divider);
        assert_eq!(roll_surprise(&mut rng), Ok(true));
        assert_eq!(divider.consumed(), 4);
        let mut wins = 0;
        for value in 0..250 {
            let mut divider = ReplayDivider::new([0, 0]);
            let mut rng =
                CrystalRandom::new(CrystalRandomState { add: 0, sub: value }, &mut divider);
            wins += usize::from(roll_surprise(&mut rng).expect("complete trace"));
        }
        assert_eq!(wins, 50);
    }

    #[test]
    fn fair_food_exact_item_vendor_and_map_mappings() {
        for (item, mart, species) in [
            ("FAIR_CHEESE_CURDS", "MART_MnFairCurdsVendor", "CURDLET"),
            ("FAIR_HOTDISH", "MART_MnFairCurdsVendor", "TOTLIT"),
            ("FAIR_COOKIE_PAIL", "MART_MnFairCookieVendor", "DOUGHLIN"),
            ("FAIR_COOKIE_PAIL", "MART_MnFairBakerVendor", "DOUGHLIN"),
            ("FAIR_KETTLE_CORN", "MART_MnFairKettleVendor", "PIPCORN"),
            ("FAIR_KETTLE_CORN", "MART_MnFairGardenSnack", "PIPCORN"),
        ] {
            assert_eq!(species_for_purchase(FAIR_MAP, mart, item), Some(species));
            assert_eq!(species_for_purchase("CherrygroveMart", mart, item), None);
            assert_eq!(
                species_for_purchase(FAIR_MAP, "MART_MnFairSupplyVendor", item),
                None
            );
        }
        for item in [
            "POTION",
            "FAIR_BEER",
            "FAIR_WALLEYE",
            "FAIR_ICE_CREAM",
            "FAIR_PRETZEL",
        ] {
            assert_eq!(
                species_for_purchase(FAIR_MAP, "MART_MnFairCurdsVendor", item),
                None
            );
        }
    }
}
