//! Compare read-only bounds with the mutable live preparation path.

use std::borrow::Cow;
use std::collections::BTreeMap;

use crate::battle::damage::Weather;
use crate::random::BattleRandomSource;

use super::{
    BattleMovePreviewer, BattleSide, BattleTransformState, BattleTypeOverride, PreparedCriticalHit,
    PreparedDamageHitOutcome, ScriptedBattleRandom, battle_state, effective_battle_pokemon,
    effective_battle_pokemon_borrowed, move_data_with_effect, move_priorities, pokemon,
    pokemon_type, prepare_damage_hit, preview_battle_move, stat_multipliers, type_categories,
    type_effectiveness_table, weather_modifiers,
};

struct Variation(u8);

impl BattleRandomSource for Variation {
    fn battle_random_byte(&mut self) -> u8 {
        self.0.rotate_left(1)
    }
}

#[test]
fn readonly_bounds_match_live_damage_without_mutating_combat() {
    let base = battle_state(
        pokemon("CYNDAQUIL", 60, pokemon_type("FIRE"), "TACKLE"),
        pokemon("TOTODILE", 50, pokemon_type("WATER"), "TACKLE"),
        1,
    );
    let multipliers = stat_multipliers();
    let categories = type_categories();
    let effectiveness = type_effectiveness_table();
    let weather = weather_modifiers();
    let priorities = move_priorities();
    let items = BTreeMap::new();
    let mut previewer = BattleMovePreviewer::new(
        &items,
        &priorities,
        &multipliers,
        &categories,
        &effectiveness,
        &weather,
    );
    for variant in 0..6 {
        let mut state = base.clone();
        match variant {
            1 => {
                state.enemy.status = Some("BURN".to_string());
                state.enemy_burn_attack_penalty_active = true;
            }
            2 => {
                let p = &state.enemy;
                state.player_transform = Some(BattleTransformState {
                    species: p.species.clone(),
                    dvs: p.dvs,
                    moves: p.moves.clone(),
                    stat_boosts: p.stat_boosts.clone(),
                    attack: p.attack,
                    defense: p.defense,
                    speed: p.speed,
                    special_attack: p.special_attack,
                    special_defense: p.special_defense,
                });
            }
            3 => {
                state.enemy_type_override = Some(BattleTypeOverride {
                    type1: pokemon_type("FIRE"),
                    type2: pokemon_type("FIRE"),
                })
            }
            4 => {
                state.weather = Weather::Rain;
                state.enemy_reflect_turns = 3;
                state.player_light_screen_turns = 3;
            }
            5 => {
                state.player.hp = 1;
                state.enemy.hp = 1;
            }
            _ => {}
        }
        let original = state.clone();
        for side in [BattleSide::Player, BattleSide::Enemy] {
            let view = effective_battle_pokemon_borrowed(&state, side);
            assert_eq!(view.as_ref(), &effective_battle_pokemon(&state, side));
            let overridden = (variant == 2 && side == BattleSide::Player)
                || (variant == 3 && side == BattleSide::Enemy);
            assert_eq!(matches!(view, Cow::Borrowed(_)), !overridden);
            for (name, effect) in [
                ("TACKLE", "NORMAL_HIT"),
                ("FLAIL", "REVERSAL"),
                ("FALSE_SWIPE", "FALSE_SWIPE"),
                ("HIDDEN_POWER", "HIDDEN_POWER"),
                ("RAGE", "RAGE"),
            ] {
                let data = move_data_with_effect(name, pokemon_type("NORMAL"), 40, 100, effect);
                let preview = preview_battle_move(
                    &state,
                    side,
                    &data,
                    &items,
                    &priorities,
                    &multipliers,
                    &categories,
                    &effectiveness,
                    &weather,
                )
                .expect("preview damage bounds");
                for _ in 0..2 {
                    assert_eq!(
                        previewer
                            .preview(&state, side, &data)
                            .expect("cached preview"),
                        preview
                    );
                }
                assert_eq!(
                    previewer
                        .damage_on_hit(&state, side, &data)
                        .expect("prediction bounds"),
                    preview.damage_on_hit,
                );
                let mut changed = data.clone();
                changed.power += 20;
                assert_eq!(
                    previewer
                        .preview(&state, side, &changed)
                        .expect("changed move in same hash bucket"),
                    preview_battle_move(
                        &state,
                        side,
                        &changed,
                        &items,
                        &priorities,
                        &multipliers,
                        &categories,
                        &effectiveness,
                        &weather
                    )
                    .expect("changed move preview"),
                );
                let mut expected = [0; 2];
                for (index, roll) in [217, 255].into_iter().enumerate() {
                    let hit = prepare_damage_hit(
                        &mut state.clone(),
                        side,
                        name,
                        &data,
                        1,
                        false,
                        &multipliers,
                        &categories,
                        &effectiveness,
                        &weather,
                        &items,
                        &mut Variation(roll),
                        &mut Vec::new(),
                        Some(PreparedCriticalHit {
                            critical: false,
                            roll: 255,
                            threshold: 0,
                        }),
                    )
                    .expect("live damage preparation");
                    let PreparedDamageHitOutcome::Damage(hit) = hit else {
                        panic!("deterministic damage move must produce damage");
                    };
                    expected[index] = hit.result.damage;
                }
                assert_eq!(
                    preview.damage_on_hit,
                    Some(expected),
                    "{variant}: {side:?} {name}"
                );
                assert_eq!(
                    state, original,
                    "preview must not change combat or RNG state"
                );
            }
        }
    }
}

#[test]
fn failed_damage_validation_preserves_the_live_critical_register_write() {
    let mut state = battle_state(
        pokemon("CYNDAQUIL", 60, pokemon_type("FIRE"), "TACKLE"),
        pokemon("TOTODILE", 50, pokemon_type("WATER"), "TACKLE"),
        1,
    );
    let data = move_data_with_effect("TACKLE", pokemon_type("NORMAL"), 40, 100, "NORMAL_HIT");
    let multipliers = stat_multipliers();
    let mut categories = type_categories();
    categories.physical.clear();
    let mut random = ScriptedBattleRandom::new(vec![0, 255]);
    let result = prepare_damage_hit(
        &mut state,
        BattleSide::Player,
        "TACKLE",
        &data,
        1,
        false,
        &multipliers,
        &categories,
        &type_effectiveness_table(),
        &weather_modifiers(),
        &BTreeMap::new(),
        &mut random,
        &mut Vec::new(),
        None,
    );
    assert!(result.is_err());
    assert_eq!(state.critical_hit_register, 1);
}
