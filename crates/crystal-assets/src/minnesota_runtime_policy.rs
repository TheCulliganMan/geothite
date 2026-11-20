//! Runtime rules shared with Minnesota pack authoring.

use crystal_core::state::GameState;

use crate::GameDataSet;

pub(crate) const DEX_MEMBER_PREFIX: &str = "MN_DEX_MEMBER_";
pub(crate) const DEX_CAUGHT_VARIABLE: &str = "VAR_MINNESOTA_DEX_CAUGHT";

impl GameDataSet {
    fn minnesota_dex_markers(&self) -> impl Iterator<Item = (&str, i64)> {
        self.story_event_script_constants
            .global
            .range(DEX_MEMBER_PREFIX.to_owned()..)
            .map_while(|(key, value)| Some((key.strip_prefix(DEX_MEMBER_PREFIX)?, *value)))
    }

    /// Lists active Minnesota species in this pack, excluding other regions and retired families.
    pub fn minnesota_dex_members(&self) -> impl Iterator<Item = &str> {
        self.minnesota_dex_markers().map(|(name, _)| name)
    }

    /// Returns displayed regional numbers. Older packs mark every member with one,
    /// so they receive consecutive numbers in their installed species order.
    pub fn minnesota_dex_numbers(&self) -> std::collections::BTreeMap<&str, u16> {
        let mut markers: Vec<_> = self.minnesota_dex_markers().collect();
        if markers.iter().all(|(_, value)| *value == 1) {
            markers.sort_by_key(|&(name, _)| self.pokemon.get(name).map(|s| s.int_id));
            return markers
                .into_iter()
                .zip(1..=u16::MAX)
                .map(|((name, _), number)| (name, number))
                .collect();
        }
        markers
            .into_iter()
            .filter_map(|(name, value)| Some((name, u16::try_from(value).ok()?)))
            .collect()
    }
}

/// Native city-note encounters retain their authored level, even with scaling.
pub(crate) fn preserves_city_note_level(map: &str, script: &str) -> bool {
    matches!(map, "MnMinneapolis" | "MnStPaul" | "MnDuluth")
        && script.starts_with("MnCityNote")
        && script.ends_with("Native")
}

/// Counts caught active regional members from the pack, never a save variable.
pub(crate) fn caught_count(data: &GameDataSet, state: &GameState) -> usize {
    data.minnesota_dex_members()
        .filter(|species| state.pokedex.has_caught(species))
        .count()
}
