//! Read-only party choices from carried moves and the visible League opponent.
use crystal_core::battle::damage::{calculate_type_effectiveness_multiplier, is_physical_move};
use crystal_core::models::Pokemon;
use crystal_runtime::{CrystalRuntime, RuntimeShellSnapshot};

fn types(pokemon: &Pokemon) -> Vec<String> {
    if pokemon.species.type1 == pokemon.species.type2 {
        vec![pokemon.species.type1.clone()]
    } else {
        vec![pokemon.species.type1.clone(), pokemon.species.type2.clone()]
    }
}

/// Choose a living counter without executing a turn or previewing RNG.
/// Uses known move power, friendly stats, typing, current health and possible
/// opposing STAB types. It does not inspect the opponent's hidden move list.
pub(super) fn partner(r: &CrystalRuntime, s: &RuntimeShellSnapshot) -> Option<usize> {
    let battle = s.battle.as_ref()?;
    if battle.player_semi_invulnerable || battle.player_wrapped || battle.player_cannot_escape {
        return battle.active_player_party_index;
    }
    // Wait for the actual replacement to appear, rather than guessing its
    // identity from party order when competitive opponents can also switch.
    if battle.enemy_pokemon.hp == 0 {
        return None;
    }
    let enemy = &battle.enemy_pokemon;
    let enemy_types = types(enemy);
    s.party
        .slots
        .iter()
        .filter(|slot| slot.pokemon.hp > 0)
        .filter_map(|slot| {
            let p = &slot.pokemon;
            let own_types = types(p);
            let risk = enemy_types
                .iter()
                .map(|kind| {
                    calculate_type_effectiveness_multiplier(
                        &r.data().type_effectiveness,
                        kind,
                        &own_types,
                    )
                    .expect("known visible opponent typing")
                })
                .max_by(|left, right| {
                    (u64::from(left.numerator) * u64::from(right.denominator))
                        .cmp(&(u64::from(right.numerator) * u64::from(left.denominator)))
                })
                .expect("opponent has a type");
            let score = p
                .moves
                .iter()
                .filter(|learned| {
                    learned.current_pp > 0
                        && (Some(slot.index) != battle.active_player_party_index
                            || battle.player_disabled_move.as_deref()
                                != Some(learned.name.as_str()))
                })
                .filter_map(|learned| {
                    let m = &r.data().moves[&learned.name];
                    if m.power == 0 {
                        return None;
                    }
                    let effect = calculate_type_effectiveness_multiplier(
                        &r.data().type_effectiveness,
                        &m.move_type,
                        &enemy_types,
                    )
                    .expect("known visible opponent typing");
                    let attack = if is_physical_move(&r.data().type_categories, m)
                        .expect("native move category")
                    {
                        p.attack
                    } else {
                        p.special_attack
                    };
                    let stab = if m.move_type == p.species.type1 || m.move_type == p.species.type2 {
                        3_u64
                    } else {
                        2
                    };
                    Some(
                        u64::from(m.power)
                            * u64::from(attack)
                            * u64::from(p.level)
                            * stab
                            * u64::from(effect.numerator)
                            * u64::from(m.accuracy.max(1))
                            * u64::from(p.hp)
                            * u64::from(risk.denominator)
                            / (u64::from(effect.denominator)
                                * u64::from(risk.numerator.max(1))
                                * u64::from(p.max_hp.max(1))),
                    )
                })
                .max()
                .unwrap_or(0);
            (score > 0).then_some((
                slot.index,
                score,
                Some(slot.index) == battle.active_player_party_index,
            ))
        })
        .max_by_key(|(_, score, active)| (*score, *active))
        .map(|(index, _, _)| index)
}
