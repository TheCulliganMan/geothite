//! Preserve native levels in ordinary Minnesota habitats at every progression.
use crystal_core::state::GameState;

use crate::GameDataSet;

// The encounter-rate/bite draw is independent of the species-slot draw. Reuse
// one quarter of its byte values, without another RNG draw or a rarity change.
fn preserves_native_level(map: &str, rate_roll: u8) -> bool {
    map.starts_with("Mn") && rate_roll.is_multiple_of(4)
}

impl GameDataSet {
    /// Ordinary step/fishing encounters retain a native-level minority. Roamers,
    /// authored static encounters, contests and script-driven special habitats
    /// do not use this policy. Species selection and authored floors are exact.
    pub(crate) fn ordinary_challenge_level(
        &self,
        state: &GameState,
        map: &str,
        authored: u8,
        variation: u8,
        rate_roll: u8,
    ) -> u8 {
        if preserves_native_level(map, rate_roll) {
            authored
        } else {
            self.challenge_level(state, authored, variation)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::preserves_native_level;

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
