//! Ordinary carried-team choices against visible mine opponents.
use crystal_core::battle::damage::calculate_type_effectiveness_multiplier;
use crystal_runtime::{CrystalRuntime, RuntimeShellSnapshot};

/// Read the current opponent, or the replacement announced by the native Shift
/// prompt. This never previews a turn, advances RNG or writes battle state.
pub(super) fn partner(runtime: &CrystalRuntime, snapshot: &RuntimeShellSnapshot) -> Option<usize> {
    let battle = snapshot.battle.as_ref()?;
    if battle.player_semi_invulnerable || battle.player_wrapped || battle.player_cannot_escape {
        return battle.active_player_party_index;
    }
    let enemy = if battle.enemy_pokemon.hp == 0 {
        battle
            .enemy_party
            .iter()
            .enumerate()
            .find_map(|(index, pokemon)| {
                (Some(index) != battle.active_enemy_party_index && pokemon.hp > 0)
                    .then_some(pokemon)
            })?
    } else {
        &battle.enemy_pokemon
    };
    let types = [enemy.species.type1.as_str(), enemy.species.type2.as_str()];
    let attack_type = if types.contains(&"FIRE") && types.contains(&"STEEL") {
        Some("GROUND")
    } else if types.contains(&"ICE") {
        Some("FIRE")
    } else if types.contains(&"FLYING") {
        Some("ICE")
    } else if types.contains(&"ROCK")
        || (types.contains(&"GROUND") && types.contains(&"FIGHTING"))
        || types.contains(&"WATER")
    {
        Some("GRASS")
    } else {
        None
    };
    let preferred = attack_type
        .into_iter()
        .chain(types.contains(&"FIRE").then_some("WATER"))
        .find_map(|attack_type| {
            snapshot
                .party
                .slots
                .iter()
                .filter(|slot| {
                    slot.pokemon.hp > 0
                        && slot.pokemon.moves.iter().any(|learned| {
                            let definition = &runtime.data().moves[&learned.name];
                            learned.current_pp > 0
                                && definition.power > 0
                                && definition.move_type == attack_type
                                && (Some(slot.index) != battle.active_player_party_index
                                    || battle.player_disabled_move.as_deref()
                                        != Some(learned.name.as_str()))
                        })
                })
                .max_by_key(|slot| {
                    let stab = if slot.pokemon.species.type1 == attack_type
                        || slot.pokemon.species.type2 == attack_type
                    {
                        3_u32
                    } else {
                        2
                    };
                    slot.pokemon
                        .moves
                        .iter()
                        .filter(|learned| learned.current_pp > 0)
                        .map(|learned| &runtime.data().moves[&learned.name])
                        .filter(|definition| definition.move_type == attack_type)
                        .map(|definition| {
                            u32::from(definition.power) * u32::from(slot.pokemon.level) * stab
                        })
                        .max()
                        .unwrap_or(0)
                })
                .map(|slot| slot.index)
        });
    if preferred.is_some() {
        return preferred;
    }
    // If no preferred counter is carried, avoid exhausting an immune attack
    // or forcing Struggle while another living partner still has usable PP.
    let enemy_types = if enemy.species.type1 == enemy.species.type2 {
        vec![enemy.species.type1.clone()]
    } else {
        vec![enemy.species.type1.clone(), enemy.species.type2.clone()]
    };
    snapshot
        .party
        .slots
        .iter()
        .filter(|slot| slot.pokemon.hp > 0)
        .filter(|slot| {
            slot.pokemon.moves.iter().any(|learned| {
                let definition = &runtime.data().moves[&learned.name];
                learned.current_pp > 0
                    && definition.power > 0
                    && (Some(slot.index) != battle.active_player_party_index
                        || battle.player_disabled_move.as_deref() != Some(learned.name.as_str()))
                    && calculate_type_effectiveness_multiplier(
                        &runtime.data().type_effectiveness,
                        &definition.move_type,
                        &enemy_types,
                    )
                    .expect("visible mine opponent types")
                    .numerator
                        > 0
            })
        })
        .min_by_key(|slot| {
            (
                Some(slot.index) != battle.active_player_party_index,
                slot.index,
            )
        })
        .map(|slot| slot.index)
}
