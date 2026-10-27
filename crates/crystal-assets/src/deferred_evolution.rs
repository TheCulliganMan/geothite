//! Deferred level checks at the completed battle's pending reward boundary.
use anyhow::{Context, Result};
use crystal_core::models::Pokemon;
use crystal_core::state::{BattleMemory, GameState};
use crystal_core::systems::battle_items::require_field_party_pokemon_mut;

pub(crate) fn completed_reward(state: &GameState, party_index: usize) -> bool {
    state.battle_evolvable_party_indices.contains(&party_index)
        && match &state.battle {
            BattleMemory::Wild {
                enemy_pokemon,
                enemy_party,
                ..
            }
            | BattleMemory::StaticWild {
                enemy_pokemon,
                enemy_party,
                ..
            }
            | BattleMemory::Trainer {
                enemy_pokemon,
                enemy_party,
                ..
            } => {
                enemy_pokemon.hp == 0
                    && !enemy_party.is_empty()
                    && enemy_party.iter().all(|p| p.hp == 0)
            }
            BattleMemory::Inactive => false,
        }
}

pub(crate) fn party_member(state: &mut GameState, party_index: usize) -> Result<&mut Pokemon> {
    if completed_reward(state, party_index) {
        return state
            .storage
            .party
            .pokemon
            .get_mut(party_index)
            .and_then(Option::as_mut)
            .context("deferred battle reward party member");
    }
    require_field_party_pokemon_mut(state, party_index)
        .map_err(|error| anyhow::anyhow!("resolve deferred level evolution: {error:?}"))
}
