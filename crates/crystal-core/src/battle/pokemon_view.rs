//! Borrow effective battlers when neither transformation nor type overrides apply.

use std::borrow::Cow;

use crate::models::Pokemon;

use super::{
    BattleCombatState, BattleSide, effective_battle_pokemon, transform_state, type_override,
};

pub(super) fn effective_battle_pokemon_borrowed(
    state: &BattleCombatState,
    side: BattleSide,
) -> Cow<'_, Pokemon> {
    if transform_state(state, side).is_none() && type_override(state, side).is_none() {
        Cow::Borrowed(state.pokemon(side))
    } else {
        Cow::Owned(effective_battle_pokemon(state, side))
    }
}
