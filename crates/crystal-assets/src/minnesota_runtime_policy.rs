//! Runtime rules shared with Minnesota pack authoring.

use crystal_core::state::GameState;

use crate::GameDataSet;

pub(crate) const DEX_MEMBER_PREFIX: &str = "MN_DEX_MEMBER_";
pub(crate) const DEX_CAUGHT_VARIABLE: &str = "VAR_MINNESOTA_DEX_CAUGHT";

/// Native city-note encounters retain their authored level, even with scaling.
pub(crate) fn preserves_city_note_level(map: &str, script: &str) -> bool {
    matches!(map, "MnMinneapolis" | "MnStPaul" | "MnDuluth")
        && script.starts_with("MnCityNote")
        && script.ends_with("Native")
}

/// Counts caught active regional members from the pack, never a save variable.
pub(crate) fn caught_count(data: &GameDataSet, state: &GameState) -> usize {
    state
        .pokedex
        .caught_species
        .iter()
        .filter(|id| {
            data.story_event_script_constants
                .global
                .contains_key(&format!("{DEX_MEMBER_PREFIX}{id}"))
        })
        .count()
}
