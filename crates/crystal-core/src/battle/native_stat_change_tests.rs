//! Execute native effects whose pack entries omit optional stat annotations.
use std::collections::BTreeMap;

use super::{
    ScriptedBattleRandom, battle_state, move_data_with_effect, move_priorities, pokemon,
    stat_multipliers, type_categories, type_effectiveness_table, weather_modifiers,
};
use crate::battle::turn::{BattleAction, BattleSide, BattleTurnInput, resolve_battle_turn};
use crate::models::pokemon::{Stat, pokemon_type};

#[test]
fn native_stat_effect_programs_apply_without_optional_pack_annotations() {
    for (name, effect, stat, stages) in [
        ("LEER", "DEFENSE_DOWN", Stat::Defense, -1),
        ("SMOKESCREEN", "ACCURACY_DOWN", Stat::Accuracy, -1),
        ("GROWL", "ATTACK_DOWN", Stat::Attack, -1),
        ("SWORDS_DANCE", "ATTACK_UP_2", Stat::Attack, 2),
    ] {
        let player = pokemon("CYNDAQUIL", 90, pokemon_type("FIRE"), name);
        let enemy = pokemon("SENTRET", 40, pokemon_type("NORMAL"), "SPLASH");
        let effect_move = move_data_with_effect(name, pokemon_type("NORMAL"), 0, 100, effect);
        assert_eq!((effect_move.stat, effect_move.amount), (None, None));
        let moves = BTreeMap::from([
            (name.into(), effect_move),
            (
                "SPLASH".into(),
                move_data_with_effect("SPLASH", pokemon_type("NORMAL"), 0, 100, "SPLASH"),
            ),
        ]);
        let outcome = resolve_battle_turn(
            battle_state(player, enemy, 0),
            BattleTurnInput {
                player: BattleAction::Move { slot: 0 },
                enemy: BattleAction::Move { slot: 0 },
            },
            &moves,
            &move_priorities(),
            &stat_multipliers(),
            &type_categories(),
            &type_effectiveness_table(),
            &weather_modifiers(),
            &mut ScriptedBattleRandom::new(vec![0; 128]),
        )
        .expect("native stat program resolves");
        let target = if stages > 0 {
            BattleSide::Player
        } else {
            BattleSide::Enemy
        };
        assert_eq!(
            outcome.state.pokemon(target).stat_boosts[&stat],
            stages,
            "{name}"
        );
        assert_eq!(
            outcome.state.player.hp, outcome.state.player.max_hp,
            "{name} must not damage the player"
        );
        assert_eq!(
            outcome.state.enemy.hp, outcome.state.enemy.max_hp,
            "{name} must not fall through to damage"
        );
        assert_eq!(outcome.state.player.moves[0].current_pp, 4);
    }
}

#[test]
fn native_stat_annotations_keep_explicit_authored_overrides_and_partial_pairs() {
    let mut m = move_data_with_effect("LEER", pokemon_type("NORMAL"), 0, 100, "DEFENSE_DOWN");
    m.stat = Some(Stat::Speed);
    m.amount = Some(-2);
    assert_eq!(
        crate::battle::native_stat_change::parameters(&m),
        Some((Stat::Speed, -2))
    );
    m.amount = None;
    assert_eq!(crate::battle::native_stat_change::parameters(&m), None);
    m.stat = None;
    m.amount = Some(0);
    assert_eq!(crate::battle::native_stat_change::parameters(&m), None);
    m.stat = Some(Stat::Speed);
    assert_eq!(
        crate::battle::native_stat_change::parameters(&m),
        Some((Stat::Speed, 0))
    );
}
