//! Keep ordinary Minnesota encounters approachable and retain native habitats.
use crystal_core::state::GameState;

use crate::GameDataSet;

// The encounter-rate/bite draw is independent of the species-slot draw. Reuse
// one quarter of its byte values, without another RNG draw or a rarity change.
fn preserves_native_level(map: &str, rate_roll: u8) -> bool {
    map.starts_with("Mn") && rate_roll.is_multiple_of(4)
}

fn opening_level(map: &str, badges: bool, highest: Option<u8>, authored: u8, level: u8) -> u8 {
    if map.starts_with("Mn") && !badges && authored <= 10 {
        if let Some(highest) = highest {
            return level.min(highest.saturating_add(1).max(2));
        }
    }
    level
}

impl GameDataSet {
    /// Ordinary step/fishing encounters retain a native-level minority. Roamers,
    /// authored static encounters, contests and script-driven special habitats
    /// do not use this policy. Before the first badge, low-level Minnesota
    /// encounters are capped near the player's party; species are unchanged.
    pub(crate) fn ordinary_challenge_level(
        &self,
        state: &GameState,
        map: &str,
        authored: u8,
        variation: u8,
        rate_roll: u8,
    ) -> u8 {
        let level = if preserves_native_level(map, rate_roll) {
            authored
        } else {
            self.challenge_level(state, authored, variation)
        };
        let badges = state
            .badges
            .johto
            .iter()
            .chain(&state.badges.kanto)
            .any(|badge| *badge);
        let highest = state
            .storage
            .party
            .pokemon
            .iter()
            .flatten()
            .filter(|pokemon| !pokemon.is_egg)
            .map(|pokemon| pokemon.level)
            .max();
        opening_level(map, badges, highest, authored, level)
    }
}

#[cfg(test)]
mod tests {
    use super::{opening_level, preserves_native_level};

    #[test]
    fn opening_encounters_do_not_jump_five_levels_above_a_starter() {
        assert_eq!(opening_level("MnR01C00R00", false, Some(5), 10, 10), 6);
        assert_eq!(opening_level("MnR01C00R00", true, Some(5), 10, 10), 10);
        assert_eq!(opening_level("Route29", false, Some(5), 10, 10), 10);
        assert_eq!(opening_level("MnR01C00R00", false, None, 10, 10), 10);
        assert_eq!(opening_level("MnR01C00R00", false, Some(5), 20, 20), 20);
    }

    #[test]
    fn ordinary_native_level_policy_is_deterministic_and_minnesota_only() {
        assert_eq!(
            (0..=255)
                .filter(|r| preserves_native_level("MnR01C00R00", *r))
                .count(),
            64
        );
        for map in ["MnR01C00R00", "MnR10C00R00", "MnR18C00R00", "MnLakeItasca"] {
            assert!(preserves_native_level(map, 0));
            assert!(!preserves_native_level(map, 1));
        }
        for map in ["Route29", "Route4", "GoldenrodCity", ""] {
            assert!(!(0..=255).any(|r| preserves_native_level(map, r)));
        }
    }
}

#[cfg(test)]
#[path = "wild_level_refuges_tests.rs"]
mod table_tests;
